# Spec: analyzer — SSD 悬崖警告 + 开发者缓存识别 + 云文件检测

> 模块 id: `analyzer`
> 依赖: `scanner`
> @author 十四叔
> @date 2026/09/11
> 状态: **已批准**

## Objective

对 scanner 产出的 FileTree 进行智能分析，输出人类可读的建议和警告。

**用户故事**：
- 作为 SSD 用户，我希望磁盘占用 > 85% 时收到性能悬崖警告，这样我能提前清理避免暴跌
- 作为开发者，我希望工具能识别 node_modules/.cargo/target 等缓存目录，这样我能一键找到"安全可删"的大块头
- 作为 Windows 用户，我希望 OneDrive 占位符文件不被计入实际占用，这样我能看清真实磁盘使用情况

## Tech Stack

- **纯逻辑模块**: 无 UI 依赖，输入 FileTree，输出 `Vec<AnalysisResult>`
- **Windows API**: `GetDriveTypeW`（判断 SSD/HDD）、`GetDiskFreeSpaceExW`（剩余空间）
- **云文件检测**: `winapi` 读取 `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS` 属性

## Commands

```bash
# 构建
cargo build --release

# 测试
cargo test -p danqing-disk-analyzer
```

## Project Structure

```
danqing-disk-analyzer/
├── src/
│   ├── lib.rs              # 公开 API：Analyzer, AnalysisResult
│   ├── ssd_warn.rs         # SSD 性能悬崖检测
│   ├── dev_cache.rs        # 开发者缓存目录识别
│   ├── cloud_placeholder.rs # OneDrive/Dropbox 占位符检测
│   └── types.rs            # 数据类型定义
├── tests/
│   ├── ssd_warn_unit.rs
│   ├── dev_cache_unit.rs
│   └── cloud_placeholder_unit.rs
```

## Code Style

```rust
/// 分析结果：一条警告或建议
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// 结果类型
    pub kind: AnalysisKind,
    /// 严重程度
    pub severity: Severity,
    /// 人类可读消息
    pub message: String,
    /// 涉及的路径
    pub path: Option<PathBuf>,
    /// 可回收空间（字节）
    pub reclaimable: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisKind {
    /// SSD 性能悬崖警告
    SsdWarning,
    /// 开发者缓存目录
    DevCache,
    /// 云文件占位符
    CloudPlaceholder,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}
```

## Testing Strategy

| 测试层级 | 框架 | 位置 | 覆盖要求 |
|----------|------|------|----------|
| 单元测试 | `#[test]` | `src/` | dev_cache 模式匹配 100%；ssd_warn 阈值逻辑 100% |
| 集成测试 | `#[test]` | `tests/` | mock FileTree → AnalysisResult 完整链路 |

## Boundaries

- **Always do**: SSD 阈值可配置（默认 85%）；开发者缓存列表可扩展
- **Ask first**: 是否检测 HDD vs SSD（需要 WMI 调用）；是否支持自定义缓存模式
- **Never do**: 自动删除任何文件（只建议不执行）；误报非缓存目录为"可删除"

## Success Criteria

1. SSD 悬崖警告：磁盘占用 > 85% 时输出 `Critical` 级别警告，附带可回收空间估算
2. 开发者缓存识别：覆盖 node_modules, .cargo/target, .git, __pycache__, .venv, DerivedData, .gradle/caches, .nuget/packages, .m2/repository
3. 云文件占位符：OneDrive `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS` 文件标记为 `CloudPlaceholder`，不计入实际占用
4. 零误报：不会将用户数据目录误判为"安全可删"

## Open Questions

1. 是否需要检测 WSL2 ext4.vhdx 虚拟磁盘？建议 Pro 版加
2. SSD 检测方式：WMI `Win32_DiskDrive.MediaType` vs `DeviceIoControl`？建议 WMI（简单可靠）
3. 开发者缓存是否需要显示"重建代价"（如 `node_modules` 重建需要 `npm install`）？建议 MVP 不做
