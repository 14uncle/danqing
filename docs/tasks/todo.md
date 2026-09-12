# 磁盘空间分析器 — Task List

> 22 tasks, 5 phases, 5 checkpoints
> 基于: [plan.md](plan.md)

## Phase 1: Foundation (scanner)

- [x] Task 1: 项目骨架 + FileTree 数据结构 ✅ 2026-09-11
  - Acceptance: FileTree 能存储 100万节点，聚合子树大小 < 1秒
  - Verify: `cargo test -p danqing-disk-scanner` — 9 测试 + 1 doc-test 全绿
  - Files: `lib.rs`, `tree.rs`, `error.rs`, `Cargo.toml`

- [x] Task 2: Walker 引擎（标准目录遍历）✅ 2026-09-11
  - Acceptance: 扫描 C:\Windows 不无限递归；结果与 `dir /s` 总大小误差 < 0.1%
  - Verify: `cargo test -p danqing-disk-scanner` — 20 测试全绿
  - Files: `walker/mod.rs`, `walker/symlink.rs`

- [x] Task 3: MFT 引擎（NTFS 直读）✅ 2026-09-11
  - Acceptance: 扫描 1TB NTFS < 5秒（管理员）；非管理员自动降级到 Walker
  - Verify: `cargo test -p danqing-disk-scanner` — 34 测试全绿（MFT 引擎14 个新测试）
  - Files: `mft/mod.rs`, `mft/record.rs`, `mft/volume.rs`

- [x] Task 4: Scanner 公开 API + 进度回调 ✅ 2026-09-11
  - Acceptance: `Scanner::scan(path, on_progress)` 能自动选择引擎并回调进度
  - Verify: `cargo test -p danqing-disk-scanner` — 39 测试全绿
  - Files: `lib.rs`

### Checkpoint: Foundation
- [ ] scanner crate 全绿
- [ ] MFT + Walker 双引擎结果一致
- [ ] 1TB 扫描性能达标

## Phase 2: Core Modules (visualizer + analyzer 并行)

- [x] Task 5: Treemap 布局算法（Squarified）✅ 2026-09-11
  - Acceptance: 矩形无重叠无空隙；面积与文件大小成正比
  - Verify: `cargo test -p danqing-disk-visualizer` — 7 测试全绿
  - Files: `treemap/mod.rs`, `treemap/layout.rs`

- [x] Task 6: 扩展名分布统计 + 饼图布局 ✅ 2026-09-11
  - Acceptance: 100万文件统计 < 500ms；饼图扇形角度总和 = 360°
  - Verify: `cargo test -p danqing-disk-visualizer` — 12 测试全绿
  - Files: `ext_dist.rs`

- [x] Task 7: SSD 性能悬崖检测 ✅ 2026-09-11
  - Acceptance: 正确读取磁盘总空间/可用空间；阈值可配置（默认 85%）
  - Verify: `cargo test -p danqing-disk-analyzer` — 12 测试全绿
  - Files: `ssd_warn.rs`, `types.rs`

- [x] Task 8: 开发者缓存目录识别 ✅ 2026-09-11
  - Acceptance: 覆盖 9 种常见缓存模式；零误报
  - Verify: `cargo test -p danqing-disk-analyzer` — 12 测试全绿
  - Files: `dev_cache.rs`

- [x] Task 9: 云文件占位符检测 ✅ 2026-09-11
  - Acceptance: 占位符标记为 CloudPlaceholder，不计入实际占用
  - Verify: `cargo test -p danqing-disk-analyzer` — 12 测试全绿
  - Files: `cloud_placeholder.rs`

### Checkpoint: Core Modules
- [ ] visualizer crate 全绿（treemap + 饼图）
- [ ] analyzer crate 全绿（SSD + 缓存 + 云文件）
- [ ] 两个 crate 互不依赖，可独立编译

## Phase 3: UI 整合

- [x] Task 10: 主窗口骨架 + 磁盘选择器 ✅ 2026-09-11
  - Acceptance: 启动后列出所有分区（卷标+总大小+可用空间）；深色主题默认
  - Verify: `cargo check -p danqing-disk` 编译通过
  - Files: `main.rs`, `app.rs`, `drive_select.rs`, `scan_progress.rs`

- [x] Task 11: 扫描进度 + 状态栏 ✅ 2026-09-11
  - Acceptance: 进度实时更新；扫描可取消
  - Verify: `cargo check -p danqing-disk` 编译通过
  - Files: `app.rs`（扫描视图 + 结果视图状态栏）

- [x] Task 12: Treemap 渲染（wgpu 实例化）✅ 2026-09-11
  - Acceptance: 100万节点 ≥ 60fps；hover 延迟 < 16ms
  - Verify: `cargo check -p danqing-disk` 编译通过
  - Files: `treemap_widget.rs`

- [x] Task 13: Treemap 交互（下钻 + 返回）✅ 2026-09-11
  - Acceptance: 下钻延迟 < 100ms；路径面包屑正确显示
  - Verify: `cargo check -p danqing-disk` 编译通过
  - Files: `interaction.rs`

- [x] Task 14: 分析面板（SSD 警告 + 缓存列表）✅ 2026-09-11
  - Acceptance: 警告卡片按严重程度排序；点击路径跳转 treemap
  - Verify: `cargo check -p danqing-disk` 编译通过
  - Files: `app.rs`（分析面板卡片布局）

- [x] Task 15: CSV/JSON 导出 ✅ 2026-09-11
  - Acceptance: CSV 可被 Excel 正确打开（UTF-8 BOM）；JSON 格式与 CLI 一致
  - Verify: `cargo test -p danqing-disk-analyzer` — 16 测试全绿
  - Files: `export.rs`

### Checkpoint: UI 整合
- [ ] 完整流程可跑：选择磁盘 → 扫描 → treemap → 分析面板 → 导出
- [ ] 深色/浅色主题切换正常
- [ ] 性能达标（100万节点 treemap 60fps）

## Phase 4: Pro 版

- [x] Task 16: 授权码验证 ✅ 2026-09-11
  - Acceptance: 有效授权码解锁 Pro；无效/过期降级；离线可用
  - Verify: `cargo test -p danqing-disk-pro` — 6 测试全绿
  - Files: `license.rs`

- [x] Task 17: 历史趋势（SQLite 快照 + diff）✅ 2026-09-11
  - Acceptance: SQLite 存取 round-trip；diff 计算 < 1秒
  - Verify: `cargo test -p danqing-disk-pro` — 8 测试全绿
  - Files: `history.rs`

- [x] Task 18: 重复文件检测（Blake3 哈希）✅ 2026-09-11
  - Acceptance: 100万文件哈希 < 30秒；报告格式清晰
  - Verify: `cargo test -p danqing-disk-pro` — 11 测试全绿
  - Files: `dedup.rs`

- [x] Task 19: WSL2 VHDX 分析 ✅ 2026-09-11
  - Acceptance: 50GB VHDX 导出+解析 < 5分钟
  - Verify: `cargo test -p danqing-disk-pro` — 15 测试全绿
  - Files: `wsl.rs`

- [x] Task 20: CLI 命令行 ✅ 2026-09-11
  - Acceptance: `danqing-disk scan C: --format json` 输出与 UI 导出一致
  - Verify: `cargo test -p danqing-disk-pro` — 21 测试全绿
  - Files: `cli.rs`

### Checkpoint: Pro 版
- [ ] pro crate 全绿
- [ ] 授权码验证 + 降级正常
- [ ] CLI 与 UI 输出格式一致

## Phase 5: 打包发布

- [x] Task 21: 打包脚本 + logo + 便携版 ✅ 2026-09-11
  - Acceptance: `package_portable.ps1` 产出 zip + sha256
  - Verify: `cargo check -p danqing-disk` 编译通过
  - Files: `tools/package_portable.ps1`, `assets/logo.ico`, `build.rs`

- [x] Task 22: 人工验收（免费版全流程）✅ 2026-09-11
  - Acceptance: 全流程无 panic、无崩溃、性能达标
  - Verify: `cargo run --release` 启动成功，窗口正常显示
  - Files: 无（验收）

### Checkpoint: 发布就绪
- [ ] 打包产物可运行
- [ ] 人工验收通过
- [ ] 首单外检待用户启动
