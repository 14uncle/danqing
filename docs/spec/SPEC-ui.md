# Spec: ui — 主题/窗口/交互/导出整合层

> 模块 id: `ui`
> 依赖: `visualizer`, `analyzer`
> @author 十四叔
> @date 2026/09/11
> 状态: **已批准**

## Objective

整合 scanner/visualizer/analyzer，提供完整的桌面应用体验。

**用户故事**：
- 作为用户，我希望打开应用后选择磁盘分区一键扫描，这样我能快速开始分析
- 作为用户，我希望 treemap 和分析建议在同一窗口内，这样我不需要切换视图
- 作为用户，我希望导出扫描结果为 CSV/JSON，这样我能分享给同事或存档
- 作为用户，我希望深色主题是默认且美观的，这样日常使用舒适

## Tech Stack

- **UI 框架**: danqing（winit 0.30 + wgpu 自绘）
- **窗口管理**: danqing 窗口系统
- **主题**: danqing Theme trait（深色默认 + 浅色切换）
- **导出**: serde_json（JSON）、csv crate（CSV）

## Commands

```bash
# 构建
cargo build --release

# 运行
cargo run --release

# 测试
cargo test -p danqing-disk
```

## Project Structure

```
danqing-disk/
├── src/
│   ├── main.rs             # 入口 + #![windows_subsystem = "windows"]
│   ├── app.rs              # 应用状态机：Idle → Scanning → Result → DrillingDown
│   ├── layout.rs           # 主窗口布局：左侧 treemap + 右侧分析面板
│   ├── drive_select.rs     # 磁盘分区选择器（自动列出 C:, D:, ...）
│   ├── scan_progress.rs    # 扫描进度条 + 文件计数器
│   ├── analysis_panel.rs   # 分析结果列表（SSD 警告 + 缓存识别）
│   ├── export.rs           # CSV/JSON 导出
│   ├── status_bar.rs       # 底部状态栏（扫描耗时/文件总数/总大小）
│   └── tray.rs             # 系统托盘（可选，MVP 不做）
├── assets/
│   └── logo.ico
├── tools/
│   └── package_portable.ps1
└── Cargo.toml
```

## Code Style

```rust
//! @author 十四叔
//! @date 2026/09/11

/// 应用主状态
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// 空闲：显示磁盘选择器
    Idle,
    /// 扫描中：显示进度
    Scanning { progress: ScanProgress },
    /// 结果：显示 treemap + 分析面板
    Result { tree: FileTree, analysis: Vec<AnalysisResult> },
    /// 下钻中：treemap 进入子目录
    DrillingDown { path: PathBuf, tree: FileTree },
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    /// 已扫描文件数
    pub files_scanned: u64,
    /// 已扫描字节数
    pub bytes_scanned: u64,
    /// 当前扫描路径
    pub current_path: PathBuf,
    /// 预估剩余秒数（-1 = 未知）
    pub eta_secs: i64,
}
```

## Testing Strategy

| 测试层级 | 框架 | 位置 | 覆盖要求 |
|----------|------|------|----------|
| 单元测试 | `#[test]` | `src/` | AppState 状态机转换 100%；导出格式正确性 100% |
| 集成测试 | `#[test]` | `tests/` | mock 数据 → 完整 UI 流程不 panic |
| 手动验收 | — | — | 本机实际磁盘扫描 + treemap 交互 + 导出验证 |

## Boundaries

- **Always do**: 扫描可取消；treemap 交互不阻塞 UI 线程；导出 UTF-8 BOM（Windows Excel 兼容）
- **Ask first**: 是否需要扫描历史记录（Pro 版功能）；是否需要右键菜单
- **Never do**: 扫描期间冻结 UI；导出覆盖已有文件不提示

## Success Criteria

1. 磁盘选择器：自动列出所有可用分区（NTFS/ReFS/FAT32/exFAT），显示卷标+总大小+可用空间
2. 扫描进度：实时更新文件计数 + 当前路径 + 预估剩余时间
3. treemap 交互：点击下钻 / 右键返回 / 悬停 tooltip，延迟 < 100ms
4. 分析面板：SSD 警告 + 缓存识别结果以卡片列表展示，点击路径跳转 treemap 对应区块
5. 导出：CSV（Excel 兼容）+ JSON 两种格式，UTF-8 BOM
6. 深色主题默认，浅色主题可切换，切换即时生效

## Open Questions

1. 主窗口尺寸和布局比例？建议：treemap 占 70%，分析面板占 30%（可拖拽调整）
2. 是否需要"扫描完成"通知音？建议：不加（安静优先）
3. 系统托盘 MVP 是否做？建议：不做，v1 只做主窗口
