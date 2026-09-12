# Spec: pro — 历史趋势 + 重复文件检测 + WSL2 VHDX + CLI

> 模块 id: `pro`
> 依赖: `scanner`, `analyzer`
> @author 十四叔
> @date 2026/09/11
> 状态: **已批准**

## Objective

Pro 版付费功能，提供高级分析能力。与免费版共享扫描引擎，通过授权码解锁。

**用户故事**：
- 作为用户，我希望看到"上次扫描后哪些文件夹变大了"，这样我能追踪磁盘使用趋势
- 作为用户，我希望找到重复文件，这样我能释放被浪费的空间
- 作为 WSL2 用户，我希望看到 ext4.vhdx 内部的文件分布，这样我能定位 WSL 占用的大文件
- 作为系统管理员，我希望用命令行导出磁盘分析报告，这样我能集成到自动化脚本

## Tech Stack

- **历史趋势**: SQLite（本地存储扫描快照）
- **重复文件检测**: Blake3 哈希（快速+安全）
- **WSL2 VHDX**: `wsl --export` + tar 解析（非侵入式，不挂载 VHDX）
- **CLI**: `clap` 参数解析

## Commands

```bash
# 构建
cargo build --release

# 测试
cargo test -p danqing-disk-pro

# CLI 用法示例
danqing-disk scan C: --format json --output report.json
danqing-disk scan C: --format csv --output report.csv
danqing-disk history C: --last 7     # 最近7次扫描趋势
danqing-disk dedup C: --dry-run      # 查找重复文件（不删除）
```

## Project Structure

```
danqing-disk-pro/
├── src/
│   ├── lib.rs              # 公开 API：ProFeatures, HistoryStore
│   ├── history/
│   │   ├── mod.rs          # 历史趋势入口
│   │   ├── store.rs        # SQLite 快照存储
│   │   └── diff.rs         # 两次扫描的 diff 计算
│   ├── dedup/
│   │   ├── mod.rs          # 重复文件检测入口
│   │   ├── hash.rs         # Blake3 分块哈希
│   │   └── report.rs       # 重复文件报告
│   ├── wsl/
│   │   ├── mod.rs          # WSL2 VHDX 分析入口
│   │   ├── vhdx.rs         # VHDX 导出 + tar 解析
│   │   └── tree.rs         # WSL 文件树映射
│   ├── cli/
│   │   ├── mod.rs          # CLI 入口
│   │   └── args.rs         # clap 参数定义
│   └── license.rs          # 授权码验证
├── tests/
│   ├── history_unit.rs
│   ├── dedup_unit.rs
│   └── cli_integration.rs
```

## Code Style

```rust
//! @author 十四叔
//! @date 2026/09/11

/// 扫描快照（存储在 SQLite 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSnapshot {
    /// 扫描时间（Unix timestamp）
    pub timestamp: i64,
    /// 磁盘分区
    pub drive: String,
    /// 总大小（字节）
    pub total_size: u64,
    /// 文件总数
    pub file_count: u64,
    /// 目录总数
    pub dir_count: u64,
    /// 扩展名分布 Top 20
    pub ext_distribution: Vec<(String, u64)>,
}

/// 两次扫描的差异
#[derive(Debug, Clone)]
pub struct ScanDiff {
    /// 总大小变化（正=增长，负=减少）
    pub size_delta: i64,
    /// 文件数变化
    pub file_delta: i64,
    /// 增长最多的目录 Top 10
    pub top_grown: Vec<DirDelta>,
    /// 减少最多的目录 Top 10
    pub top_shrunk: Vec<DirDelta>,
}

#[derive(Debug, Clone)]
pub struct DirDelta {
    pub path: PathBuf,
    pub delta: i64,
}
```

## Testing Strategy

| 测试层级 | 框架 | 位置 | 覆盖要求 |
|----------|------|------|----------|
| 单元测试 | `#[test]` | `src/` | diff 计算 100%；哈希一致性 100%；CLI 参数解析 100% |
| 集成测试 | `#[test]` | `tests/` | SQLite 存取 round-trip；mock VHDX 导出 |

## Boundaries

- **Always do**: 授权码验证离线可用（本地公钥验证）；CLI 输出 UTF-8
- **Ask first**: WSL2 VHDX 分析是否需要实时挂载（风险高）还是只做导出解析（安全但慢）
- **Never do**: 自动删除重复文件（只报告，用户自行操作）；在线验证授权码（隐私顾虑）

## Success Criteria

1. 历史趋势：SQLite 存储 ≥ 52 周扫描快照，diff 计算 < 1秒
2. 重复文件检测：100万文件 Blake3 哈希 < 30秒（本机 NVMe）
3. WSL2 VHDX：导出 + 解析 50GB VHDX < 5分钟
4. CLI：`danqing-disk scan C: --format json` 输出与 UI 导出格式一致
5. 授权码：离线验证，过期/无效时降级为免费版功能

## Open Questions

1. WSL2 VHDX 分析是否需要管理员权限？建议：需要（`wsl --export` 需要 WSL 运行）
2. 历史趋势 UI 在主窗口还是独立窗口？建议：主窗口右侧分析面板 tab 切换
3. 重复文件报告是否需要"一键选择保留最新/最大"的智能建议？建议 MVP 不做
