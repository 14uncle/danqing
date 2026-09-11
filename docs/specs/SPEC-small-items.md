# SPEC: 小件包下沉 —— 文件管理器显示 / 相对时间 / 缩略图降采样

> 来源: `docs/intent/framework-sinking.md` 小件包。
> 2026-09-10 审查: 6 件中 3 件不够格 (open_feedback 只有单消费者、CSV BOM 是产品格式、MSIX 成本 M 只 store feature), 砍至 3 件。
> 每件 S 级; 打包一个 spec, 统一五阶段推进。

## 能力地图

| # | 能力 | 职责 | 来源 | 成本 | 现状 |
|---|------|------|------|------|------|
| S1 | `reveal_in_file_manager` | 系统文件管理器定位文件 (Win Explorer / mac Finder / Linux xdg-open) | pomodoro `main.rs:1386-1416` | **S** | 通用文件交互原语, danqing 零 |
| S2 | 相对时间 + 历法 + 时区偏移 | 五档相对时间 + Hinnant 历法换算 + 本地时区秒偏移 | clipboard `history_list.rs:109-172` | **S** | 8 测试含闰日锚点, 纯整数算法, tz 走 chrono (零 Win32 FFI) |
| S3 | RGBA 缩略图降采样 | 等比缩进 + 双线性插值 (src RGBA → dst RGBA) | clipboard `history_list.rs:369-423` | **S** | 纯逻辑零依赖, any 图片 UI 必遇 |

**砍掉:**
- **`open_feedback`** (GitHub Issues 预填): log 侧只是裸 Link, 不是预填表单 → 仅 pomodoro 一个消费者; 且链接/标签是产品策略
- **CSV BOM 壳**: `\u{FEFF}` 前缀 + 产品表头 + 格式化 → 表头/格式是产品语义, 壳太薄不值得抽
- **MSIX `is_running_as_msix` + `find_main_window`**: 成本 M (EnumWindows 需 windows-sys 移植), `#[cfg(feature = "store")]` 门控, 仅 pomodoro 商店版用 → 第二消费者未现, 等时再沉

## API 设计

### S1 (`src/fs.rs` 新模块)

```rust
/// 在系统文件管理器中定位并高亮文件。
/// Win: `explorer /select`; mac: `open -R`; Linux: `xdg-open` 所在目录。
/// 失败仅日志, 不 panic (导出成功后显示文件, 失败不影响导出结果)。
pub fn reveal_in_file_manager(path: &std::path::Path);
```

### S2 (`src/time.rs` 新模块)

```rust
/// 相对时间戳五档: 刚刚 / N 分钟前 / HH:MM (今天) / 昨天 / MM-DD (更早)。
/// 纯函数: 时钟与时区由调用方注入 (tz_offset 为本地相对 UTC 的秒数)。
pub fn relative_time(last_seen: i64, now: i64, tz_offset: i64) -> String;

/// days since 1970-01-01 → (year, month, day)。Howard Hinnant 历法, 纯整数。
pub fn civil_from_days(z: i64) -> (i64, u32, u32);

/// 本地时区相对 UTC 的当前偏移秒 (含 DST)。走 chrono (零 Win32 FFI)。
pub fn local_tz_offset_seconds() -> i64;
```

**移植**: clipboard 的 `local_tz_offset_seconds` 走 `windows` 0.62 `GetTimeZoneInformation` FFI; 框架改走 `chrono::Local::now().offset()` (已依赖 chrono), 非 Windows 天然可用。实测两者返回一致。

### S3 (`src/image.rs` 新模块)

```rust
/// RGBA 数据降采样到目标尺寸 (双线性插值, 平滑边缘)。
pub fn downscale_rgba(data: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8>;

/// 等比缩进 max×max 方框 (不放大), 返回 (宽, 高)。
pub fn aspect_fit(w: u32, h: u32, max: f32) -> (f32, f32);
```

## 验收标准

1. **单元测试直接搬** (行为保持):
   - S1: 无测试 (Command::spawn 在 CI 无文件管理器); 不 panic 冒烟不够有意义, 省略
   - S2: 8 测试直接搬 (刚刚 / 分钟前 / 今天 HH:MM / 昨天 / 更早 MM-DD / 闰日 / 跨时区 / 纪元起点)
   - S3: 降采样函数无独立测试 (与缩略图业务耦合); 但 `aspect_fit` 可测 (正方形/横向/纵向/放大边界)
2. **三件套**: `cargo fmt` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib --tests` 全绿。
3. **产品迁移验证** (clipboard / pomodoro):
   - S1: pomodoro 删 `reveal_in_file_manager`, 改 `danqing::fs::reveal_in_file_manager`
   - S2: clipboard 删 `relative_time`/`civil_from_days`/`local_tz_offset_seconds`, 改 `danqing::time::*` (8 测试零改动全绿)
   - S3: clipboard 删 `downscale_rgba`/`aspect_fit`, 改 `danqing::image::*` (缩略图行为不变)

## 依赖

- S1: 无 (std::process::Command)
- S2: 无新增 (chrono 已依赖)
- S3: 无 (纯 std)

## Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| S2 tz 走 chrono vs Win32 FFI 结果不一致 | 显示时间偏差 | chrono::Local::now().offset() 在 Windows 上内部调同系 API, 实测一致 |
| S1 xdg-open 在 CI/Wayland 可能失败 | 测试 flaky | 无测试 (Command::spawn 不可测) |
| S3 降采样精度 (f32 舍入) | 缩略图微差异 | 与原版逐像素一致 (原版已用 f32 + round) |
