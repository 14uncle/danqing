# Implementation Plan: 丹青阶段 1 — 设计系统 + 品牌视觉

> 依据 `docs/specs/phase1-design-system.md` 细化而来。
> 本文档将阶段 1 拆分为 **11 个可验证任务**，按依赖顺序组织。

## Overview

在 丹青 M2（焦点、TextInput、剪贴板、IME）基础上，建立现代毛玻璃（Glassmorphism）设计系统与品牌视觉资产，使 `examples/showcase.rs` 呈现统一视觉语言，为阶段 2 的剪贴板历史管理器奠定视觉基础。

## Architecture Decisions

- **Theme  trait 体系**：`Theme` trait + `LightTheme` 结构体，预留 `DarkTheme` 扩展；token 通过 trait 方法暴露，组件按 `theme.background()` 等方式读取。
- **纯逻辑层不依赖平台**：`theme.rs`、`widget/` 仍为纯逻辑，不引入 `winit`/`wgpu`。
- **资源可提交仓库**：LOGO PNG/ICO 与背景噪声图作为静态资产提交到 `assets/`，由运行时或 `build.rs` 引用。
- **自绘标题栏仅视觉**：阶段 1 不接管窗口控制，三个按钮只做悬停/按下状态，不调用最小化/最大化/关闭 API。
- **以用代测**：所有组件改造完成后必须体现在 `examples/showcase.rs` 中。

## Dependency Graph

```
Task 1  theme.rs + Theme trait
 ├─ Task 3  Box 使用 theme token
 ├─ Task 4  Button 使用 theme token
 │    └─ Task 5  TitleBar（依赖 Button + theme）
 ├─ Task 6  TextInput 使用 theme token
 ├─ Task 7  TextArea 使用 theme token
 └─ Task 8  Scrollable 使用 theme token

Task 2  LOGO + 背景图资产
 └─ Task 9  window.rs 设置窗口图标

Task 10 Showcase 整合（依赖 3/4/5/6/7/8/9）
 └─ Task 11 测试 + 最终验收
```

关键路径：1 → 4 → 5 → 10 → 11。
并行车道：视觉资产（Task 2）∥ Theme 系统（Task 1）∥ 组件改造（Task 3/4/6/7/8）。

## Task List

### Phase 1: Design Tokens — 主题系统

- [ ] **Task 1: 实现 theme.rs 与 Theme trait**
  - **Description:** 创建 `src/theme.rs`，定义 `Theme` trait、`LightTheme` 结构体及基础 token（颜色 ≥8、字体层级 ≥3、间距 ≥5、圆角 ≥3、阴影 ≥2、动效曲线 ≥2）。
  - **Acceptance criteria:**
    - [ ] `Theme` trait 存在，含颜色、字体、间距、圆角、阴影、动效方法。
    - [ ] `LightTheme` 实现 `Theme`，数值符合浅色毛玻璃方向。
    - [ ] 公开类型经 `src/lib.rs` re-export。
  - **Verification:** `cargo check` 通过；`cargo clippy -- -D warnings` 零警告。
  - **Dependencies:** None
  - **Files:** `src/theme.rs`, `src/lib.rs`
  - **Scope:** M

### Phase 2: Visual Assets — 品牌视觉

- [ ] **Task 2: 设计并导出 LOGO 与背景图资产**
  - **Description:** 在 `build.rs` 中设计并生成 丹青 LOGO(多尺寸 PNG 与 ICO)以及固定渐变/噪声背景图,输出到 `OUT_DIR/assets/`,避免在仓库中提交二进制资产。
  - **Acceptance criteria:**
    - [ ] `OUT_DIR/assets/logo/logo_16.png`、`24`、`32`、`48`、`256.png` 存在。
    - [ ] `OUT_DIR/assets/logo/logo.ico` 存在。
    - [ ] `OUT_DIR/assets/background/gradient.png` 与 `OUT_DIR/assets/background/noise.png` 存在。
  - **Verification:** 集成测试 `tests/assets.rs` 验证文件存在且非空;`cargo test` 通过。
  - **Dependencies:** None
  - **Files:** `build.rs`, `tests/assets.rs`
  - **Scope:** M

### Phase 3: Component Theming — 组件 token 化

- [ ] **Task 3: 改造 Box 组件使用 theme token**
  - **Description:** 将 `widget/box.rs` 中的颜色、圆角、阴影替换为 theme token。
  - **Acceptance criteria:**
    - [ ] `Box` 不再使用裸魔法值。
    - [ ] 现有布局与事件行为不变。
  - **Verification:** `cargo test widget::box` 通过。
  - **Dependencies:** Task 1
  - **Files:** `src/widget/box.rs`
  - **Scope:** S

- [ ] **Task 4: 改造 Button 组件使用 theme token**
  - **Description:** 将 `widget/button.rs` 中的颜色、圆角、内边距、阴影替换为 theme token；保留悬停/按下状态。
  - **Acceptance criteria:**
    - [ ] `Button` 三态视觉均使用 theme token。
    - [ ] 点击/消息行为不变。
  - **Verification:** `cargo test widget::button` 通过。
  - **Dependencies:** Task 1
  - **Files:** `src/widget/button.rs`
  - **Scope:** S

- [ ] **Task 5: 实现自绘 TitleBar 组件**
  - **Description:** 新增 `widget/title_bar.rs`：左侧 LOGO + 标题，右侧三个窗口按钮视觉；按钮支持悬停/按下状态，不调用窗口控制 API。
  - **Acceptance criteria:**
    - [ ] `TitleBar` 可放入组件树。
    - [ ] 按钮有 hover/pressed 状态。
    - [ ] 命中区域正确。
  - **Verification:** `cargo test title_bar` 通过；showcase 中可见。
  - **Dependencies:** Task 1, Task 4
  - **Files:** `src/widget/title_bar.rs`, `src/widget/mod.rs`, `src/lib.rs`
  - **Scope:** M

- [ ] **Task 6: 改造 TextInput 组件使用 theme token**
  - **Description:** 将 `widget/text_input.rs` 中的背景、边框、文字、光标、选区颜色替换为 theme token。
  - **Acceptance criteria:**
    - [ ] 视觉使用 theme token。
    - [ ] 不破坏撤销/重做、IME、焦点行为。
  - **Verification:** `cargo test widget::text_input` 通过。
  - **Dependencies:** Task 1
  - **Files:** `src/widget/text_input.rs`
  - **Scope:** S

- [ ] **Task 7: 改造 TextArea 组件使用 theme token**
  - **Description:** 将 `widget/text_area.rs` 中的背景、边框、文字、光标、选区颜色替换为 theme token。
  - **Acceptance criteria:**
    - [ ] 视觉使用 theme token。
    - [ ] 不破坏撤销/重做、IME、焦点行为。
  - **Verification:** `cargo test widget::text_area` 通过。
  - **Dependencies:** Task 1
  - **Files:** `src/widget/text_area.rs`
  - **Scope:** S

- [ ] **Task 8: 改造 Scrollable 组件使用 theme token**
  - **Description:** 将 `widget/scrollable.rs` 中的滚动条颜色、轨道颜色、圆角替换为 theme token。
  - **Acceptance criteria:**
    - [ ] 滚动条/轨道视觉使用 theme token。
    - [ ] 滚动行为不变。
  - **Verification:** `cargo test widget::scrollable` 通过。
  - **Dependencies:** Task 1
  - **Files:** `src/widget/scrollable.rs`
  - **Scope:** S

### Phase 4: Window Integration — 窗口图标

- [x] **Task 9: 在 window.rs 中设置窗口图标**
  - **Description:** 使用 `OUT_DIR/assets/logo/` 下由 `build.rs` 生成的 PNG 资源,在 `WindowAttributes` 中设置窗口图标与任务栏图标;提供加载失败 fallback。
  - **Acceptance criteria:**
    - [ ] 窗口左上角显示新 LOGO。
    - [ ] 任务栏显示新 LOGO。
    - [ ] 图标加载失败时不 panic。
  - **Verification:** `cargo run --example showcase` 人工确认。
  - **Dependencies:** Task 2
  - **Files:** `src/window.rs`
  - **Scope:** S

### Phase 5: Showcase Integration — 整体呈现

- [ ] **Task 10: 整合 showcase 呈现毛玻璃效果**
  - **Description:** 更新 `examples/showcase.rs`，使用 `Theme`、改造后的组件、`TitleBar`、固定背景图，呈现统一毛玻璃视觉。
  - **Acceptance criteria:**
    - [ ] showcase 使用 `LightTheme`。
    - [ ] 背景为半透明 + 固定渐变/噪声图。
    - [ ] 各组件统一圆角、阴影、间距。
    - [ ] TitleBar 可见且按钮有视觉反馈。
  - **Verification:** `cargo run --example showcase` 人工确认。
  - **Dependencies:** Task 3, Task 4, Task 5, Task 6, Task 7, Task 8, Task 9
  - **Files:** `examples/showcase.rs`
  - **Scope:** M

### Phase 6: Testing & Acceptance — 验收

- [ ] **Task 11: 编写设计系统测试并做最终验收**
  - **Description:** 新增 `tests/design_system.rs` 集成测试；补充各模块单元测试；最终运行 fmt/clippy/test。
  - **Acceptance criteria:**
    - [ ] `tests/design_system.rs` 覆盖 theme token 非空、组件应用 theme、标题栏命中区域。
    - [ ] `cargo test` 全绿。
    - [ ] `cargo clippy -- -D warnings` 零警告。
    - [ ] `cargo fmt --check` 通过。
    - [ ] `cargo build --release` 成功。
  - **Verification:** 逐条执行验收命令。
  - **Dependencies:** Task 10
  - **Files:** `tests/design_system.rs`, 各 `#[cfg(test)]` 模块
  - **Scope:** M

## Checkpoints

| 检查点 | 条件 | 验证命令 |
|---|---|---|
| CP1 | Theme 系统可编译 | `cargo check` |
| CP2 | 组件改造不破坏测试 | `cargo test` |
| CP3 | TitleBar 命中/事件正确 | `cargo test title_bar` |
| CP4 | 零警告 | `cargo clippy -- -D warnings` |
| CP5 | 视觉验收 | `cargo run --example showcase` |
| CP6 | 最终验收 | `cargo fmt && cargo clippy -- -D warnings && cargo test && cargo build --release` |

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| `winit` 跨平台去装饰窗口行为不一致 | 中 | 阶段 1 先保证 Windows；macOS/Linux 可保留 OS 标题栏降级 |
| 半透明/alpha blending 在 wgpu 中效果不对 | 中 | 先以固定背景 + 半透明 surface 模拟，不急于真实模糊 shader |
| LOGO ICO 格式在不同平台显示异常 | 低 | 多尺寸 PNG 备用；ICO 包含 16/32/48/256 多层 |
| 标题栏视觉与 winit 默认标题栏冲突 | 中 | 仅在 Windows 用 `with_decorations(false)`，其他平台保留装饰 |
| 组件改造误改焦点/IME 行为 | 高 | 只改颜色/圆角/阴影常量，不改事件与状态机；改后跑对应测试 |

## Parallelization Notes

- **可并行**：Task 2（资产设计）∥ Task 1（theme 系统）∥ Task 3/4/6/7/8（组件改造）。
- **必须串行**：Task 5 在 Task 4 后；Task 10 在所有组件与 Task 9 后；Task 11 在最后。
