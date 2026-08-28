# Implementation Plan: Tabs 组件 (danqing 引擎)

## Overview

在 danqing 框架 `src/widget/view/tabs.rs` 新增 `Tabs` 容器组件。核心是 MultiPanel 的面板切换逻辑 + 自绘 tab 栏头部。完成后产品侧用 `danqing::widget::Tabs` 替换手写 TabBar。

## Dependency Graph

```
Tabs struct + builder API
    │
    ├── Widget trait impl (sync/layout/paint/event)
    │       │
    │       ├── tab_bar 绘制 (paint 中自绘文字+指示线)
    │       │
    │       └── 面板切换 (复用 MultiPanel 的 sync/显隐逻辑)
    │
    ├── mod.rs 注册 + re-export
    │
    └── 单元测试
```

## Task List

### Phase 1: 引擎侧

- [x] **T1: Tabs 组件实现** — `danqing/src/widget/view/tabs.rs` 新增 Tabs struct + builder + Widget impl
  - 验收: Tabs 可编译, tab 栏渲染文字+指示线, 面板切换正确
  - 验证: `cargo clippy` + `cargo test` (在 danqing 仓库)
  - 文件: `danqing/src/widget/view/tabs.rs`

- [x] **T2: 模块注册 + re-export** — `view/mod.rs` 添加 pub mod + pub use, `widget/mod.rs` 添加 pub use
  - 验收: `danqing::widget::Tabs` 可用
  - 验证: `cargo clippy` 零警告
  - 依赖: T1
  - 文件: `danqing/src/widget/view/mod.rs`, `danqing/src/widget/mod.rs`

- [x] **T3: 单元测试** — Tabs 的 active 钳制、tab/child 数量一致性、bind 驱动切换
  - 验收: 测试全绿
  - 验证: `cargo test`
  - 依赖: T1
  - 文件: `danqing/src/widget/view/tabs.rs` (底部 mod tests)

### Checkpoint: 引擎侧完成

- [ ] danqing 仓库 `cargo clippy` 零警告
- [ ] danqing 仓库 `cargo test` 全绿

### Phase 2: 产品侧

- [ ] **T4: 产品侧迁移** — `danqing-clipboard/src/ui/settings.rs` 用 `Tabs` 替换手写 TabBar
  - 验收: 删除手写 TabBar struct, 改用 `danqing::widget::Tabs`, 功能不变
  - 验证: `cargo clippy` + `cargo test` (在 clipboard 仓库)
  - 依赖: T2
  - 文件: `danqing-clipboard/src/ui/settings.rs`, `danqing-clipboard/src/main.rs`

### Checkpoint: 产品侧完成

- [ ] clipboard 仓库 `cargo clippy` 零警告
- [ ] clipboard 仓库 `cargo test` 全绿
- [ ] 手动验证: 三个 tab 切换正常

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Tabs 的 tab 栏高度与产品现有布局不协调 | Low | Theme token 驱动, 可微调 |
| 面板切换焦点语义与 MultiPanel 不一致 | Low | 直接复用 MultiPanel 的 reset_focus 逻辑 |
