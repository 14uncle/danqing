# Todo: widget/ 分类目录化 + MultiPanel + showcase 分类导航

> 精简勾选版，详见 [plan-widget-categorization.md](plan-widget-categorization.md)。

## Phase 1: 目录迁移（纯移动，行为不变）

- [x] **Task 1** 迁移 base/(button、text)— `cargo check` + `cargo test` 绿 ✅ 2026-07-21
- [x] **Task 2** 迁移 view/(scrollable)（依赖 1)— `cargo check` + `cargo test` 绿 ✅ 2026-07-21
- [x] **Task 3** 迁移 layout/(6 文件）+ 修 flow import + 更新 CLAUDE.md 测试路径（依赖 2)— `cargo test widget::layout` 11 绿 + 全量 139+43 绿 ✅ 2026-07-21
- [x] **Task 4** 迁移 form/(3 文件）+ 修 text_editor import（依赖 3)— `cargo test widget::form` 47 绿 + 全量绿 ✅ 2026-07-21

### ⏸ Checkpoint A: 迁移收尾验收

- [x] `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` + `cargo test` 全绿
- [x] showcase 冒烟（启动 6 秒无崩溃、首帧渲染正常;12 文件 git 改名检测 0 增删）
- [x] tests/ 与 examples/ 全程零改动（公开 API 不变的直接证据）

## Phase 2: MultiPanel 组件

- [x] **Task 5** multi_panel.rs + 单元测试（clamp/尺寸/paint/event/sync/children)（依赖 4)— `cargo test widget::view::multi_panel` 6 绿 + clippy 零警告 ✅ 2026-07-21
- [x] **Task 6** tests/multi_panel.rs 集成测试（焦点链/切走清焦/event_at_path/点击隐藏区)（依赖 5)— `cargo test --test multi_panel` 5 绿 + 全量绿 ✅ 2026-07-21

### ⏸ Checkpoint B: MultiPanel 验收

- [x] fmt + clippy -D warnings + 全测试绿 + `cargo doc` 无警告

## Phase 3: showcase 分类导航改造

- [x] **Task 7** Showcase 状态/Msg + 四页面函数归位（依赖 5)— `cargo check --example showcase` 通过 ✅ 2026-07-21
- [x] **Task 8** 侧边栏 + MultiPanel 接线（依赖 7)— 人工验收通过（切换四类 / 交互回归 / 焦点语义 / Tab 循环）✅ 2026-07-21
- [x] **Task 9** 文档同步（CLAUDE.md、README.md、showcase 文件头）✅ 2026-07-21
