# 磁盘空间分析器 — Task List

> 22 tasks, 5 phases, 5 checkpoints
> 基于: [plan.md](plan.md)

## Phase 1: Foundation (scanner)

- [x] Task 1: 项目骨架 + FileTree 数据结构 ✅ 2026-09-11
  - Acceptance: FileTree 能存储 100万节点，聚合子树大小 < 1秒
  - Verify: `cargo test -p danqing-disk-scanner` — 9 测试 + 1 doc-test 全绿
  - Files: `lib.rs`, `tree.rs`, `error.rs`, `Cargo.toml`

- [ ] Task 2: Walker 引擎（标准目录遍历）
  - Acceptance: 扫描 C:\Windows 不无限递归；结果与 `dir /s` 总大小误差 < 0.1%
  - Verify: `cargo test -p danqing-disk-scanner` + 手动对比 `dir /s`
  - Files: `walker/mod.rs`, `walker/symlink.rs`

- [ ] Task 3: MFT 引擎（NTFS 直读）
  - Acceptance: 扫描 1TB NTFS < 5秒（管理员）；非管理员自动降级到 Walker
  - Verify: `cargo bench -p danqing-disk-scanner` + 手动计时
  - Files: `mft/mod.rs`, `mft/record.rs`, `mft/volume.rs`

- [ ] Task 4: Scanner 公开 API + 进度回调
  - Acceptance: `Scanner::scan(path, on_progress)` 能自动选择引擎并回调进度
  - Verify: `cargo test -p danqing-disk-scanner`
  - Files: `lib.rs`

### Checkpoint: Foundation
- [ ] scanner crate 全绿
- [ ] MFT + Walker 双引擎结果一致
- [ ] 1TB 扫描性能达标

## Phase 2: Core Modules (visualizer + analyzer 并行)

- [ ] Task 5: Treemap 布局算法（Squarified）
  - Acceptance: 矩形无重叠无空隙；面积与文件大小成正比
  - Verify: `cargo test -p danqing-disk-visualizer`
  - Files: `treemap/mod.rs`, `treemap/layout.rs`

- [ ] Task 6: 扩展名分布统计 + 饼图布局
  - Acceptance: 100万文件统计 < 500ms；饼图扇形角度总和 = 360°
  - Verify: `cargo test -p danqing-disk-visualizer`
  - Files: `ext_dist.rs`, `pie/mod.rs`

- [ ] Task 7: SSD 性能悬崖检测
  - Acceptance: 正确读取磁盘总空间/可用空间；阈值可配置（默认 85%）
  - Verify: `cargo test -p danqing-disk-analyzer`
  - Files: `ssd_warn.rs`, `types.rs`

- [ ] Task 8: 开发者缓存目录识别
  - Acceptance: 覆盖 9 种常见缓存模式；零误报
  - Verify: `cargo test -p danqing-disk-analyzer`
  - Files: `dev_cache.rs`

- [ ] Task 9: 云文件占位符检测
  - Acceptance: 占位符标记为 CloudPlaceholder，不计入实际占用
  - Verify: `cargo test -p danqing-disk-analyzer`
  - Files: `cloud_placeholder.rs`

### Checkpoint: Core Modules
- [ ] visualizer crate 全绿（treemap + 饼图）
- [ ] analyzer crate 全绿（SSD + 缓存 + 云文件）
- [ ] 两个 crate 互不依赖，可独立编译

## Phase 3: UI 整合

- [ ] Task 10: 主窗口骨架 + 磁盘选择器
  - Acceptance: 启动后列出所有分区（卷标+总大小+可用空间）；深色主题默认
  - Verify: `cargo run --release` 手动验收
  - Files: `main.rs`, `app.rs`, `drive_select.rs`

- [ ] Task 11: 扫描进度 + 状态栏
  - Acceptance: 进度实时更新；扫描可取消
  - Verify: `cargo run --release` 手动验收
  - Files: `scan_progress.rs`, `status_bar.rs`

- [ ] Task 12: Treemap 渲染（wgpu 实例化）
  - Acceptance: 100万节点 ≥ 60fps；hover 延迟 < 16ms
  - Verify: `cargo run --release` 手动验收
  - Files: `treemap/render.rs`

- [ ] Task 13: Treemap 交互（下钻 + 返回）
  - Acceptance: 下钻延迟 < 100ms；路径面包屑正确显示
  - Verify: `cargo run --release` 手动验收
  - Files: `interaction.rs`

- [ ] Task 14: 分析面板（SSD 警告 + 缓存列表）
  - Acceptance: 警告卡片按严重程度排序；点击路径跳转 treemap
  - Verify: `cargo run --release` 手动验收
  - Files: `analysis_panel.rs`, `layout.rs`

- [ ] Task 15: CSV/JSON 导出
  - Acceptance: CSV 可被 Excel 正确打开（UTF-8 BOM）；JSON 格式与 CLI 一致
  - Verify: `cargo test -p danqing-disk` + 手动 Excel 验证
  - Files: `export.rs`

### Checkpoint: UI 整合
- [ ] 完整流程可跑：选择磁盘 → 扫描 → treemap → 分析面板 → 导出
- [ ] 深色/浅色主题切换正常
- [ ] 性能达标（100万节点 treemap 60fps）

## Phase 4: Pro 版

- [ ] Task 16: 授权码验证
  - Acceptance: 有效授权码解锁 Pro；无效/过期降级；离线可用
  - Verify: `cargo test -p danqing-disk-pro`
  - Files: `license.rs`

- [ ] Task 17: 历史趋势（SQLite 快照 + diff）
  - Acceptance: SQLite 存取 round-trip；diff 计算 < 1秒
  - Verify: `cargo test -p danqing-disk-pro`
  - Files: `history/mod.rs`, `store.rs`, `diff.rs`

- [ ] Task 18: 重复文件检测（Blake3 哈希）
  - Acceptance: 100万文件哈希 < 30秒；报告格式清晰
  - Verify: `cargo test -p danqing-disk-pro` + 手动验收
  - Files: `dedup/mod.rs`, `hash.rs`, `report.rs`

- [ ] Task 19: WSL2 VHDX 分析
  - Acceptance: 50GB VHDX 导出+解析 < 5分钟
  - Verify: `cargo test -p danqing-disk-pro`（mock tar）
  - Files: `wsl/mod.rs`, `vhdx.rs`, `tree.rs`

- [ ] Task 20: CLI 命令行
  - Acceptance: `danqing-disk scan C: --format json` 输出与 UI 导出一致
  - Verify: `cargo test -p danqing-disk-pro` + 手动 CLI 验收
  - Files: `cli/mod.rs`, `args.rs`

### Checkpoint: Pro 版
- [ ] pro crate 全绿
- [ ] 授权码验证 + 降级正常
- [ ] CLI 与 UI 输出格式一致

## Phase 5: 打包发布

- [ ] Task 21: 打包脚本 + logo + 便携版
  - Acceptance: `package_portable.ps1` 产出 zip + sha256
  - Verify: 解压后双击可运行
  - Files: `tools/package_portable.ps1`, `assets/logo.ico`

- [ ] Task 22: 人工验收（免费版全流程）
  - Acceptance: 全流程无 panic、无崩溃、性能达标
  - Verify: 手动操作 + 截图留存
  - Files: 无（验收）

### Checkpoint: 发布就绪
- [ ] 打包产物可运行
- [ ] 人工验收通过
- [ ] 首单外检待用户启动
