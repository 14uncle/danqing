# Spec: scanner — MFT 双引擎文件树扫描

> 模块 id: `scanner`
> @author 十四叔
> @date 2026/09/11
> 状态: **已批准**

## Objective

为磁盘空间分析器提供文件树数据源。双引擎架构：管理员权限时走 MFT 直读（极速），否则/非NTFS 时走标准目录遍历（兼容）。

**用户故事**：
- 作为开发者，我希望 1TB 磁盘在 5 秒内完成扫描，这样我可以快速定位大文件
- 作为非管理员用户，我希望扫描仍能正常工作（只是慢一些），这样我不需要强制提权
- 作为 ReFS 用户，我希望扫描不崩溃，自动降级到标准遍历

## Tech Stack

- **语言**: Rust (stable-x86_64-pc-windows-gnu)
- **MFT 解析**: 手动解析 NTFS `$MFT` 文件记录（raw disk read, 需管理员权限）
- **标准遍历**: `std::fs::read_dir` 递归 + `winapi` 获取文件属性
- **数据结构**: 自定义 `FileTree` 内存结构，支持快速子树聚合
- **兄弟 crate 复用**: `danqing-encoding`（文件名编码检测）

## Commands

```bash
# 构建
cargo build --release

# 测试
cargo test -p danqing-disk-scanner

# 性能基准
cargo bench -p danqing-disk-scanner
```

## Project Structure

```
danqing-disk-scanner/
├── src/
│   ├── lib.rs              # 公开 API：Scanner, ScanResult, FileTree
│   ├── mft/
│   │   ├── mod.rs          # MFT 引擎入口
│   │   ├── record.rs       # MFT 记录解析（$FILE_NAME, $DATA 属性）
│   │   └── volume.rs       # 卷句柄打开 + $MFT 文件定位
│   ├── walker/
│   │   ├── mod.rs          # 标准遍历引擎入口
│   │   └── symlink.rs      # 符号链接/挂载点循环检测
│   ├── tree.rs             # FileTree 数据结构
│   └── error.rs            # 错误类型
├── tests/
│   ├── mft_integration.rs  # MFT 引擎集成测试（需管理员）
│   ├── walker_integration.rs
│   └── tree_unit.rs
└── benches/
    └── scan_bench.rs
```

## Code Style

```rust
//! @author 十四叔
//! @date 2026/09/11

/// 单个文件/目录节点
#[derive(Debug, Clone)]
pub struct FileNode {
    /// 完整路径
    pub path: PathBuf,
    /// 文件大小（字节），目录为 0（聚合值在 FileTree 中计算）
    pub size: u64,
    /// 是否为目录
    pub is_dir: bool,
    /// 创建时间（Unix timestamp）
    pub created: Option<i64>,
    /// 最后修改时间（Unix timestamp）
    pub modified: Option<i64>,
    /// 文件属性标志（只读/隐藏/系统/压缩/稀疏等）
    pub attributes: u32,
}
```

## Testing Strategy

| 测试层级 | 框架 | 位置 | 覆盖要求 |
|----------|------|------|----------|
| 单元测试 | `#[test]` | `src/` 内 `mod tests` | FileTree 聚合逻辑 100% |
| 集成测试 | `#[test]` | `tests/` | MFT + Walker 对同一目录结果一致 |
| 性能基准 | `criterion` | `benches/` | 1TB 扫描 < 5秒（MFT）|

## Boundaries

- **Always do**: `cargo fmt` + `cargo clippy -- -D warnings` + 测试全绿再提交
- **Ask first**: MFT 解析粒度（只读 $FILE_NAME 还是也读 $DATA resident）、是否支持 NTFS 压缩/稀疏文件
- **Never do**: 在非管理员模式下尝试 raw disk read；遍历中 follow 符号链接导致无限循环

## Success Criteria

1. MFT 引擎：扫描 1TB NTFS 分区 < 5秒（管理员权限，本机测试）
2. Walker 引擎：扫描同一分区 < 60秒（非管理员）
3. 双引擎结果一致：同一目录的文件数量和总大小误差 < 0.1%
4. ReFS/FAT32/exFAT 分区自动降级到 Walker，不崩溃
5. 符号链接/挂载点循环检测有效，不会无限递归

## Open Questions

1. MFT 引擎是否需要处理 NTFS 加密文件（EFS）？建议 MVP 跳过，标记为 `ENCRYPTED` 属性
2. 扫描过程中是否需要实时进度回调？建议需要（UI 层依赖）
3. 是否支持排除指定路径？建议 MVP 不做，Pro 版加
