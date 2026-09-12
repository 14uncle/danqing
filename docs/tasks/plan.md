# Implementation Plan: 磁盘空间分析器

> @author 十四叔
> @date 2026/09/11
> 基于: [disk-analyzer-capability-map.md](../spec/disk-analyzer-capability-map.md) + 五模块 spec

## Overview

为 danqing 产品线构建第六件产品：磁盘空间分析器。采用垂直切片策略，每个 task 交付可测试的功能路径。

## Architecture Decisions

1. **scanner 作为独立 crate** (`danqing-disk-scanner`)：零 UI 依赖，可独立测试和 benchmark
2. **visualizer 和 analyzer 作为独立 crate**：互不依赖，可并行开发
3. **ui 整合层为最终二进制** (`danqing-disk`)：消费所有模块，输出桌面应用
4. **pro 作为独立 crate** (`danqing-disk-pro`)：付费层，授权码解锁
5. **复用 danqing 框架**：wgpu 渲染 + Theme trait + 窗口管理
6. **复用 danqing-encoding**：文件名编码检测

## Dependency Graph

```
danqing-disk-scanner (FileTree, MFT, Walker)
    │
    ├── danqing-disk-visualizer (Treemap, Pie, ExtDist)
    │       │
    │       └── danqing-disk (UI binary)
    │
    ├── danqing-disk-analyzer (SSD, DevCache, Cloud)
    │       │
    │       └── danqing-disk (shared)
    │
    └── danqing-disk-pro (History, Dedup, WSL, CLI)
```

## Task List

### Phase 1: Foundation (scanner)

- [x] **Task 1**: 项目骨架 + FileTree 数据结构 ✅ 2026-09-11
  - Description: 创建 danqing-disk-scanner crate，实现 FileTree 内存结构（节点插入、子树聚合、路径查找）
  - Acceptance: FileTree 能存储 100万节点，聚合子树大小 < 1秒
  - Verify: `cargo test -p danqing-disk-scanner`
  - Files: `danqing-disk-scanner/src/lib.rs`, `tree.rs`, `error.rs`, `Cargo.toml`
  - Size: M

- [x] **Task 2**: Walker 引擎（标准目录遍历）✅ 2026-09-11
  - Description: 实现递归目录遍历，填充 FileTree。包含符号链接/挂载点循环检测。
  - Acceptance: 扫描 C:\Windows 不无限递归；结果与 `dir /s` 总大小误差 < 0.1%
  - Verify: `cargo test -p danqing-disk-scanner` + 手动对比 `dir /s`
  - Files: `walker/mod.rs`, `walker/symlink.rs`
  - Size: M

- [x] **Task 3**: MFT 引擎（NTFS 直读）✅ 2026-09-11
  - Description: 实现 NTFS $MFT 文件记录解析，支持管理员权限下极速扫描
  - Acceptance: 扫描 1TB NTFS < 5秒（管理员）；非管理员自动降级到 Walker
  - Verify: `cargo bench -p danqing-disk-scanner` + 手动计时
  - Files: `mft/mod.rs`, `mft/record.rs`, `mft/volume.rs`
  - Size: L

- [x] **Task 4**: Scanner 公开 API + 进度回调 ✅ 2026-09-11
  - Description: 统一 MFT/Walker 双引擎入口，提供进度回调接口
  - Acceptance: `Scanner::scan(path, on_progress)` 能自动选择引擎并回调进度
  - Verify: `cargo test -p danqing-disk-scanner`
  - Files: `lib.rs`
  - Size: S

### Checkpoint: Foundation
- [ ] scanner crate 全绿
- [ ] MFT + Walker 双引擎结果一致
- [ ] 1TB 扫描性能达标

### Phase 2: Core Modules (visualizer + analyzer 并行)

- [x] **Task 5**: Treemap 布局算法（Squarified）✅ 2026-09-11
  - Description: 实现 Squarified Treemap 算法，输入 FileTree → 输出 TreemapRect 列表
  - Acceptance: 矩形无重叠无空隙；面积与文件大小成正比
  - Verify: `cargo test -p danqing-disk-visualizer`
  - Files: `treemap/mod.rs`, `treemap/layout.rs`
  - Size: M

- [x] **Task 6**: 扩展名分布统计 + 饼图布局 ✅ 2026-09-11
  - Description: 统计 Top 20 扩展名分布，生成饼图扇形数据
  - Acceptance: 100万文件统计 < 500ms；饼图扇形角度总和 = 360°
  - Verify: `cargo test -p danqing-disk-visualizer`
  - Files: `ext_dist.rs`, `pie/mod.rs`
  - Size: S

- [x] **Task 7**: SSD 性能悬崖检测 ✅ 2026-09-11
  - Description: 检测磁盘占用率，> 85% 输出 Critical 警告
  - Acceptance: 正确读取磁盘总空间/可用空间；阈值可配置
  - Verify: `cargo test -p danqing-disk-analyzer`
  - Files: `ssd_warn.rs`, `types.rs`
  - Size: S

- [x] **Task 8**: 开发者缓存目录识别 ✅ 2026-09-11
  - Description: 识别 node_modules, .cargo/target, __pycache__ 等缓存目录
  - Acceptance: 覆盖 9 种常见缓存模式；零误报（不误判用户数据）
  - Verify: `cargo test -p danqing-disk-analyzer`
  - Files: `dev_cache.rs`
  - Size: S

- [x] **Task 9**: 云文件占位符检测 ✅ 2026-09-11
  - Description: 检测 OneDrive `FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS` 属性
  - Acceptance: 占位符文件标记为 CloudPlaceholder，不计入实际占用
  - Verify: `cargo test -p danqing-disk-analyzer`
  - Files: `cloud_placeholder.rs`
  - Size: S

### Checkpoint: Core Modules
- [ ] visualizer crate 全绿（treemap + 饼图）
- [ ] analyzer crate 全绿（SSD + 缓存 + 云文件）
- [ ] 两个 crate 互不依赖，可独立编译

### Phase 3: UI 整合

- [x] **Task 10**: 主窗口骨架 + 磁盘选择器 ✅ 2026-09-11
  - Description: 创建 danqing-disk 二进制，实现主窗口 + 磁盘分区列表
  - Acceptance: 启动后列出所有分区（卷标+总大小+可用空间）；深色主题默认
  - Verify: `cargo run --release` 手动验收
  - Files: `main.rs`, `app.rs`, `drive_select.rs`, `Cargo.toml`
  - Size: M

- [x] **Task 11**: 扫描进度 + 状态栏 ✅ 2026-09-11
  - Description: 扫描期间显示进度条、文件计数、当前路径、预估剩余时间
  - Acceptance: 进度实时更新；扫描可取消
  - Verify: `cargo run --release` 手动验收
  - Files: `scan_progress.rs`, `status_bar.rs`
  - Size: S

- [x] **Task 12**: Treemap 渲染（wgpu 实例化）✅ 2026-09-11
  - Description: 将 TreemapRect 数据渲染为 wgpu 实例化矩形，支持 hover 高亮
  - Acceptance: 100万节点 ≥ 60fps；hover 延迟 < 16ms
  - Verify: `cargo run --release` 手动验收
  - Files: `treemap/render.rs`（visualizer crate 内）
  - Size: L

- [x] **Task 13**: Treemap 交互（下钻 + 返回）✅ 2026-09-11
  - Description: 点击区块下钻到子目录，右键返回上级
  - Acceptance: 下钻延迟 < 100ms；路径面包屑正确显示
  - Verify: `cargo run --release` 手动验收
  - Files: `interaction.rs`（visualizer crate 内）
  - Size: M

- [x] **Task 14**: 分析面板（SSD 警告 + 缓存列表）✅ 2026-09-11
  - Description: 右侧面板展示 analyzer 输出的 AnalysisResult 列表
  - Acceptance: 警告卡片按严重程度排序；点击路径跳转 treemap 对应区块
  - Verify: `cargo run --release` 手动验收
  - Files: `analysis_panel.rs`, `layout.rs`
  - Size: M

- [x] **Task 15**: CSV/JSON 导出 ✅ 2026-09-11
  - Description: 扫描结果导出为 CSV（UTF-8 BOM）和 JSON
  - Acceptance: CSV 可被 Excel 正确打开；JSON 格式与 CLI 输出一致
  - Verify: `cargo test -p danqing-disk` + 手动 Excel 打开验证
  - Files: `export.rs`
  - Size: S

### Checkpoint: UI 整合
- [ ] 完整流程可跑：选择磁盘 → 扫描 → treemap 展示 → 分析面板 → 导出
- [ ] 深色/浅色主题切换正常
- [ ] 性能达标（100万节点 treemap 60fps）

### Phase 4: Pro 版

- [x] **Task 16**: 授权码验证 ✅ 2026-09-11
  - Description: 本地公钥验证授权码，无效时降级为免费版
  - Acceptance: 有效授权码解锁 Pro 功能；无效/过期降级；离线可用
  - Verify: `cargo test -p danqing-disk-pro`
  - Files: `license.rs`
  - Size: S

- [x] **Task 17**: 历史趋势（SQLite 快照 + diff）✅ 2026-09-11
  - Description: 存储扫描快照，计算两次扫描的差异（增长/减少 Top 10）
  - Acceptance: SQLite 存取 round-trip；diff 计算 < 1秒
  - Verify: `cargo test -p danqing-disk-pro`
  - Files: `history/mod.rs`, `history/store.rs`, `history/diff.rs`
  - Size: M

- [x] **Task 18**: 重复文件检测（Blake3 哈希）✅ 2026-09-11
  - Description: 对同大小文件分块哈希，报告重复文件组
  - Acceptance: 100万文件哈希 < 30秒；报告格式清晰
  - Verify: `cargo test -p danqing-disk-pro` + 手动验收
  - Files: `dedup/mod.rs`, `dedup/hash.rs`, `dedup/report.rs`
  - Size: M

- [x] **Task 19**: WSL2 VHDX 分析 ✅ 2026-09-11
  - Description: `wsl --export` + tar 解析，映射为 FileTree
  - Acceptance: 50GB VHDX 导出+解析 < 5分钟
  - Verify: `cargo test -p danqing-disk-pro`（mock tar）
  - Files: `wsl/mod.rs`, `wsl/vhdx.rs`, `wsl/tree.rs`
  - Size: M

- [x] **Task 20**: CLI 命令行 ✅ 2026-09-11
  - Description: `clap` 实现 `scan`/`history`/`dedup` 子命令
  - Acceptance: `danqing-disk scan C: --format json` 输出与 UI 导出一致
  - Verify: `cargo test -p danqing-disk-pro` + 手动 CLI 验收
  - Files: `cli/mod.rs`, `cli/args.rs`
  - Size: S

### Checkpoint: Pro 版
- [ ] pro crate 全绿
- [ ] 授权码验证 + 降级正常
- [ ] CLI 与 UI 输出格式一致

### Phase 5: 打包发布

- [x] **Task 21**: 打包脚本 + logo + 便携版 ✅ 2026-09-11
  - Description: 复制 `package_portable.ps1`，适配 danqing-disk；添加 logo.ico
  - Acceptance: `package_portable.ps1` 产出 `danqing-disk-vX.Y.Z-win-x64.zip` + `.sha256`
  - Verify: 解压后双击可运行
  - Files: `tools/package_portable.ps1`, `assets/logo.ico`
  - Size: S

- [x] **Task 22**: 人工验收（免费版全流程）✅ 2026-09-11
  - Description: 本机实际磁盘扫描 + treemap 交互 + 分析面板 + 导出
  - Acceptance: 全流程无 panic、无崩溃、性能达标
  - Verify: 手动操作 + 截图留存
  - Files: 无（验收）
  - Size: S

### Checkpoint: 发布就绪
- [ ] 打包产物可运行
- [ ] 人工验收通过
- [ ] 首单外检待用户启动

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| MFT 解析实现复杂度 | High | Task 3 提前做，失败则全走 Walker |
| wgpu treemap 100万节点性能 | High | Task 12 用实例化渲染 + 层级 LOD |
| ReFS 无 MFT 降级 | Medium | Walker 引擎兜底，功能完整 |
| WSL2 VHDX 导出权限 | Medium | 需 WSL 运行，文档注明前置条件 |

## Open Questions

1. MFT 引擎是否需要支持 NTFS 压缩/稀疏文件？（Task 3 实现时决定）
2. Treemap 配色方案最终定稿？（Task 12 实现时与用户确认）
3. Pro 版授权码分发方式？（发布前决定）
