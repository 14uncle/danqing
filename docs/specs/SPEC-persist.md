# SPEC: `persist` 持久化模块

> 来源: `docs/intent/framework-sinking.md` 簇D。
> 形态: feature-gated (`persist`)，产品 opt-in，保持 danqing 默认零文件 I/O。
> 政策门① 裁决: 2026-09-09 用户「开口子」，与 update.rs 同级。

## 产品一句话

产品通用的 JSON/TOML 持久化基础设施：原子写防损坏、容错读、脏标记节流落盘、版本化文档降级保护。

## 能力地图

| 能力 | 职责 | 消费者 |
|------|------|--------|
| `config_dir(name)` | OS 配置目录 + 产品子目录 (`%APPDATA%/danqing/<name>`) | pomodoro / clipboard / log / 所有产品 |
| `atomic_save(path, data)` | tmp + write + rename 原子写 | pomodoro state.rs / clipboard config.rs |
| `load_or_default<T>(path)` | 缺失 → 默认值; 解析失败 → 默认值 + warn | pomodoro state.rs / clipboard config.rs |
| `DirtyFlag<T>` | 脏标记 + 节流 flush + 退出 flush | pomodoro main.rs / xirang |
| `VersionedDoc<T>` | 格式版本 + 未来版本拒读拒写保护 | pomodoro stats.rs |

## 为什么

四仓产品各自手写持久化（pomodoro ~100 行 / clipboard ~28 行 / xirang ~30 行 / danqing update.rs ~50 行），模式高度同构：
- 配置目录解析重复（`dirs::config_dir()` + 拼路径）
- 原子写 tmp+rename 重复（pomodoro 硬化版 / clipboard 缺失）
- 容错读重复（缺失 → 默认 / 损坏 → 默认 + warn）
- 脏标记+节流重复（pomodoro + xirang 各一份）
- 版本化文档只 pomodoro 有（22+ 测试），其他仓无保护

## API 设计

### 模块入口

```rust
// danqing/src/persist.rs (feature = "persist")
//! 产品通用持久化: 原子写 / 容错读 / 脏标记 / 版本化文档。
//! feature-gated: 默认不开，产品 opt-in。

mod atomic;
mod dirty;
mod versioned;

pub use atomic::{config_dir, atomic_save, load_or_default};
pub use dirty::DirtyFlag;
pub use versioned::VersionedDoc;
```

### `config_dir(name: &str) -> Option<PathBuf>`

```rust
/// OS 配置目录 + danqing 子目录 + 产品名。
/// Windows: %APPDATA%/danqing/<name>/
/// macOS: ~/Library/Application Support/danqing/<name>/
/// Linux: ~/.config/danqing/<name>/
/// 目录不存在时自动创建。
/// 返回 None = 配置目录不可得 (极端环境)。
pub fn config_dir(name: &str) -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("danqing").join(name))
}
```

依赖: `dirs` crate (已有，update.rs 消费)。

### `atomic_save(path: &Path, data: &[u8]) -> io::Result<()>`

```rust
/// 原子写: 写 .tmp 文件 + rename 覆盖目标。
/// 保证: 要么完整写入，要么原文件不变 (崩溃安全)。
/// 前提: .tmp 与目标在同一文件系统 (同目录，rename 原子)。
pub fn atomic_save(path: &Path, data: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!("{}.tmp",
        path.extension().unwrap_or_default().to_string_lossy()));
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
```

### `load_or_default<T: Default + serde::de::DeserializeOwned>(path: &Path) -> T`

```rust
/// 容错读: 文件缺失 → T::default(); 解析失败 → T::default() + log::warn。
/// 产品侧按需包装更具体的错误语义。
pub fn load_or_default<T: Default + serde::de::DeserializeOwned>(path: &Path) -> T {
    match fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("持久化文件损坏, 使用默认值: {} — {e}", path.display());
                T::default()
            }
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => T::default(),
        Err(e) => {
            log::warn!("持久化文件读取失败, 使用默认值: {} — {e}", path.display());
            T::default()
        }
    }
}
```

### `DirtyFlag<T>`

```rust
/// 脏标记 + 节流落盘。
/// - `mark_dirty()`: 标记脏 (每帧调用零成本)
/// - `tick(now)`: 检查是否该 flush (节流间隔内不重复写)
/// - `flush()`: 强制写盘 (退出时调用)
/// - 写盘失败保留 dirty + 更新 last_flush 防风暴 (不每帧重试)
pub struct DirtyFlag<T: serde::Serialize> {
    value: T,
    dirty: bool,
    last_flush: Instant,
    throttle: Duration,
    save_fn: Box<dyn Fn(&T) -> io::Result<()>>,
}

impl<T: serde::Serialize> DirtyFlag<T> {
    pub fn new(value: T, throttle: Duration, save_fn: impl Fn(&T) -> io::Result<()> + 'static) -> Self;
    pub fn get(&self) -> &T;
    pub fn get_mut(&mut self) -> &mut T { self.dirty = true; ... }
    pub fn mark_dirty(&mut self);
    pub fn tick(&mut self, now: Instant) -> io::Result<()>; // 节流 flush
    pub fn flush(&mut self) -> io::Result<()>; // 强制 flush
    pub fn is_dirty(&self) -> bool;
}
```

### `VersionedDoc<T>`

```rust
/// 版本化文档: 格式版本 + 降级保护。
/// - 加载时: 未来版本 → 清空数据 + refuse_overwrite
/// - 保存时: refuse_overwrite → 拒绝写入 (保护新版数据不被旧程序覆盖)
/// - 版本匹配: 正常读写
pub struct VersionedDoc<T> {
    format_version: u32,
    data: T,
    refuse_overwrite: bool,
}

impl<T: Default + serde::Serialize + serde::de::DeserializeOwned> VersionedDoc<T> {
    /// 当前格式版本 (产品定义常量)。
    pub const CURRENT_VERSION: u32;

    /// 从文件加载; 未来版本 → 清空 + 拒写; 损坏 → 默认。
    pub fn load(path: &Path) -> Self;

    /// 保存; refuse_overwrite → 跳过 + warn。
    pub fn save(&self, path: &Path) -> io::Result<()>;

    pub fn data(&self) -> &T;
    pub fn data_mut(&mut self) -> &mut T;
    pub fn is_future_version(&self) -> bool;
}
```

## Feature Gate

```toml
# danqing/Cargo.toml
[features]
default = []
persist = ["dirs", "serde", "serde_json"]
```

产品 opt-in:
```toml
# pomodoro/Cargo.toml
[dependencies]
danqing = { ..., features = ["persist"] }
```

## 依赖

- `dirs` (已有，update.rs 消费)
- `serde` + `serde_json` (已有，多处消费)
- 无新增外部依赖

## 验收标准

1. **单元测试** (每个能力至少 2 条):
   - `atomic_save`: 写入+读回 / 崩溃模拟 (tmp 残留不影响正确性)
   - `load_or_default`: 缺失 → 默认 / 损坏 → 默认 + 不 panic
   - `DirtyFlag`: mark_dirty → tick 到阈值 → flush / 退出 flush / 写失败保留 dirty
   - `VersionedDoc`: 正常读写 / 未来版本 → 拒写 / 损坏 → 默认
   - `config_dir`: 返回路径包含产品名 / 自动创建

2. **三件套**: `cargo fmt` + `cargo clippy --all-targets -- -D warnings` + `cargo test` 全绿

3. **产品迁移验证** (pomodoro 先行):
   - pomodoro state.rs 改用 `atomic_save` + `load_or_default` (删手写原子写)
   - pomodoro stats.rs 改用 `VersionedDoc` (删手写版本保护)
   - pomodoro main.rs 脏标记改用 `DirtyFlag` (删手写节流)
   - 既有测试零改动全绿 = 行为保持判据

## Boundaries

- **Always**: 原子写保证崩溃安全; 容错读不 panic; 脏标记写失败保留 dirty
- **Ask first**: 持久化格式变更 (影响现有用户数据); 新增持久化后端 (SQLite 等); 跨平台路径差异超出 config_dir 范围
- **Never**: 热路径写盘 (必须节流); 无版本保护的写入 (必须 VersionedDoc 或等价); 加密/压缩 (产品自理)

## 开放问题

- `DirtyFlag` 的 `Instant` 类型: 用 `std::time::Instant` (单调时钟) 还是产品注入的时间源 (可单测)? 建议后者 (与 anim 模块同策略)
- `VersionedDoc` 的版本号: 每个文档类型独立版本还是全局版本? 建议独立 (stats.rs 的 format_version 与 state.rs 的可能不同步)
