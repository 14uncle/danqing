# Implementation Plan: 丹青阶段 1 视觉 Remediation

> 依据 spec：`docs/specs/phase1-visual-remediation.md`
> 目标：在现有架构内提升 showcase 可读性、层次感与品牌感，中间态（干净生产力 + 毛玻璃氛围）。
> **状态：✅ 已完成（2026/07/21 评审通过，7 任务 + 3 Checkpoint 全部达成）**

## Overview

本次计划把视觉升级拆成 7 个可独立验证的任务。关键依赖是：

1. **资产与 token 先就绪**（背景图、LOGO、`LightTheme` 调优）。
2. **组件与标题栏**基于新 token 和资产做样式升级。
3. **showcase** 重新分组布局，把升级后的组件组合成完整界面。
4. **最终验证**：测试 + 静态检查 + 人工运行 showcase。

`src/window.rs` 已支持 Windows 下去装饰并处理 `WindowAction`，本计划不再重复实现标题栏控制逻辑，只调整其视觉呈现。

## Architecture Decisions

- **背景玻璃感用预渲染资产实现**：`gradient.png` + `glow.png` + `noise.png`，不做运行时 backdrop blur，避免渲染管线大改。
- **LOGO 使用矢量源文件驱动**：`assets/logo/logo.svg` 作为唯一 truth，`tools/export-logo.py` 导出多尺寸 PNG/ICO，保证小尺寸清晰度。
- **主题 token 一次性调整**：`src/theme.rs` 中 `LightTheme` 颜色、阴影、圆角统一改，后续组件只消费 token，不再散落魔法值。
- **showcase 布局卡片化**：减少过度 `fill` 拉伸，按“品牌/色板/输入/键盘”分组，用 `UiBox` 卡片包裹。

## Task List

### Phase 1: 资产与主题基础

- [x] **Task 1: 重制背景资产**
  - **Description:** 生成新的 `assets/background/gradient.png`（品牌蓝倾向渐变 + 柔和径向光晕）、`assets/background/noise.png`（更低对比度、更细腻），并新增 `assets/background/glow.png` 作为叠加光晕。
  - **Acceptance:**
    - 三张 PNG 存在且非空。
    - `tests/assets.rs` 中新增 `glow.png` 存在性检查并 pass。
    - 文件体积：每张 ≤ 1 MB。
  - **Verify:** `cargo test --test assets`
  - **Dependencies:** None
  - **Files:** `assets/background/*.png`, `tests/assets.rs`
  - **Scope:** S

- [x] **Task 2: 优化 LOGO 并导出多尺寸资产**
  - **Description:** 更新 `assets/logo/logo.svg`，提升小尺寸辨识度；使用 `tools/export-logo.py` 重新导出 `16/24/32/48/256.png` 与 `logo.ico`。
  - **Acceptance:**
    - `logo.svg` 更新并保留源文件。
    - 所有 `logo_*.png` 与 `logo.ico` 存在且非空。
    - `tests/assets.rs` 中 LOGO 相关测试 pass。
  - **Verify:** `cargo test --test assets`
  - **Dependencies:** None
  - **Files:** `assets/logo/*`, `tools/export-logo.py`, `tests/assets.rs`
  - **Scope:** S

- [x] **Task 3: 调优 LightTheme token**
  - **Description:** 调整 `src/theme.rs` 中 `LightTheme`：加深 `text_primary`、提升 `surface` 不透明度、增强 `shadow_md`、微调边框/分割线颜色；必要时新增 `shadow_lg` 或 `surface_overlay` token。
  - **Acceptance:**
    - 所有 `theme.rs` 单元测试 pass。
    - 颜色/间距/圆角/阴影顺序与非负性约束仍满足。
    - 无 magic value 残留。
  - **Verify:** `cargo test --lib theme`
  - **Dependencies:** None
  - **Files:** `src/theme.rs`
  - **Scope:** S

### Checkpoint 1: Foundation

- [x] `cargo test --lib --tests` 全绿。
- [x] `cargo fmt --check` / `cargo clippy -- -D warnings` 通过。
- [x] 背景资产、LOGO、主题 token 已就绪。

### Phase 2: 组件与标题栏视觉升级

- [x] **Task 4: 升级基础组件样式**
  - **Description:** 改造 `Box`、`Button`、`TextInput`、`TextArea`、`Scrollable`，使用新 token：更明确的表面/边框、清晰的 hover/focus 反馈、`Button` 使用品牌色填充 + 反白文字。
  - **Acceptance:**
    - 各组件单元测试 pass（默认样式来源、自定义覆盖仍有效）。
    - `tests/design_system.rs` 中相关颜色/半径断言与新 token 一致。
    - 无组件继续使用硬编码颜色值。
  - **Verify:** `cargo test --lib --tests`
  - **Dependencies:** Task 3
  - **Files:** `src/widget/box.rs`, `src/widget/button.rs`, `src/widget/text_input.rs`, `src/widget/text_area.rs`, `src/widget/scrollable.rs`, `tests/design_system.rs`
  - **Scope:** M

- [x] **Task 5: 优化 TitleBar 视觉**
  - **Description:** 调整 `src/widget/title_bar.rs` 的 LOGO/标题/按钮视觉：标题加粗或颜色加深、按钮 hover 背景更明显、关闭按钮 danger 色更突出；确保与去装饰窗口的圆角/阴影协调。
  - **Acceptance:**
    - `TitleBar` 单元测试 pass。
    - 绘制出的标题栏在视觉上与新的浅色主题协调。
    - 关闭/最小化/最大化按钮消息仍正确产出。
  - **Verify:** `cargo test --lib --tests title_bar`
  - **Dependencies:** Task 2, Task 3
  - **Files:** `src/widget/title_bar.rs`
  - **Scope:** S

### Checkpoint 2: Components

- [x] `cargo test --lib --tests` 全绿。
- [x] 组件与标题栏的测试覆盖没有回归。

### Phase 3: Showcase 布局与最终验证

- [x] **Task 6: 重组 showcase 布局**
  - **Description:** 改造 `examples/showcase.rs`：用 `UiBox` 卡片包裹各功能区，减少过度 `fill` 拉伸；按“品牌/色板/输入/键盘”分组；调整标题字号与间距。
  - **Acceptance:**
    - `cargo run --example showcase` 能正常启动。
    - 各功能区仍正常工作（计数器、输入框、多行文本、键盘方块）。
    - 布局不再出现大面积空白拉伸。
  - **Verify:** 人工运行 `cargo run --example showcase`
  - **Dependencies:** Task 4, Task 5
  - **Files:** `examples/showcase.rs`
  - **Scope:** M

- [x] **Task 7: 集成验证与回归检查**
  - **Description:** 全量跑测试、静态检查、发布构建；人工确认 showcase 视觉效果。
  - **Acceptance:**
    - `cargo test --lib --tests` 全绿。
    - `cargo clippy -- -D warnings` 通过。
    - `cargo fmt --check` 通过。
    - `cargo build --release` 成功。
    - Windows 下无 native 标题栏；自绘标题栏可拖拽、双击最大化、按钮响应。
  - **Verify:** 上述命令全部执行
  - **Dependencies:** Task 1, Task 2, Task 3, Task 4, Task 5, Task 6
  - **Files:** 可能涉及少量修复
  - **Scope:** S

### Checkpoint 3: Complete

- [x] 所有验收标准满足（见 spec Success Criteria）。
- [x] 视觉问题（苍白、无层次、无品牌感）已解决。
- [x] 准备进入 `code-review-and-quality`。

## 完成记录（2026/07/21 评审）

评审证据：资产/LOGO 就位且体积达标、`LightTheme` token 已调优（`text_primary` 加深、`surface` 0.95、新增 `shadow_lg`）、组件边框/焦点样式落地、TitleBar 接管窗口控制、`cargo test --lib --tests` 全绿、`clippy --all-targets -D warnings` 零警告、`cargo build --release` 成功、showcase 人工确认无 native 标题栏。

收尾阶段额外修复三个布局/焦点回归（均有回归测试）：

1. **`Box::layout` 贪婪尺寸**：有子组件但未显式指定尺寸时占满父约束上限，导致 showcase 第一张卡片独占窗口高度、后续卡片全部被挤出屏幕。修复为有子组件时随内容收缩（`src/widget/box_.rs`）。
2. **卡片宽度参差**：新增 `Flow` 交叉轴拉伸能力（`Column/Row::cross_stretch()`），Fit 子项拉伸到容器交叉尺寸，showcase 内容列与卡片列启用后卡片等宽（`src/widget/flow.rs`、`column.rs`、`row.rs`）。
3. **键盘演示收不到按键**（M2 焦点系统遗留设计缺口）：`rebuild` 每帧自动聚焦首个可聚焦节点 + 点击空白不清焦，导致按键永远路由给焦点组件。修复为 `set_by_click` 未命中可聚焦组件时清焦、自动聚焦仅首次重建执行、新增 `Escape` 清焦（`src/widget/focus.rs`、`src/window.rs`）。

showcase 内容区同步接入 `Scrollable`（根 Column 的 `fill(1)` 子项），超高卡片可滚动到达。

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Windows 无边框窗口在不同 DPI/缩放比下圆角/阴影异常 | 中 | 使用 winit 平台扩展 `set_corner_preference` + `set_undecorated_shadow`，已有实现，只做视觉确认 |
| 新背景图与 `surface` 透明叠加后仍显脏 | 中 | Task 1 输出后先人工看效果，必要时迭代；不直接进 Task 6 |
| LOGO 小尺寸优化后仍模糊 | 低 | 用 SVG 源 + Python 脚本导出，保持矢量精度 |
| 组件样式改动破坏现有测试断言 | 中 | 每次改一个组件就跑测试；`tests/design_system.rs` 同步更新 |
| `Button` 反白文字在低对比度 surface 上不可读 | 低 | 使用 `text_primary` 的 inverse 计算或直接用白色 `#FFFFFF`，并在 showcase 中验证 |

## Open Questions

已在本 spec 中确认：
- ✅ 接受预渲染径向光晕位图。
- ✅ 隐藏 native 标题栏，完全自绘。
- ✅ 优化 LOGO。
- ✅ 暗色模式延后。

## Related Documents

- `docs/specs/phase1-visual-remediation.md`
- `docs/specs/title-bar-window-controls.md`
- `docs/specs/phase1-design-system.md`
- `CLAUDE.md`
