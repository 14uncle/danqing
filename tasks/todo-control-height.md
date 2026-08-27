# Todo: 统一控件高度 Token

> Plan: `tasks/plan-control-height.md`  
> Spec: `docs/specs/spec-control-height-token.md`

## Phase 1: Token 定义

- [x] T1: Theme trait 新增 control_height
  - Acceptance: `Theme` trait 有 `fn control_height(&self) -> f36 { 36.0 }`；`LightTheme` 显式实现返回 36.0
  - Verify: `cargo test theme::tests` 通过，新增测试断言 `control_height() == 36.0`
  - Files: `src/theme.rs`
  - 检查点: **CP1**

## Phase 2: 组件改造

- [x] T2: TextInput 使用 control_height
  - Acceptance: `TextInput` 新增 `control_height` 字段；`layout()` 中 `natural_height.max(control_height)`；空内容 layout 输出高度 == 36.0
  - Verify: `cargo test widget::form::text_input` 通过
  - Files: `src/widget/form/text_input.rs`
  - 检查点: **CP2**

- [x] T3: Button 使用 control_height
  - Acceptance: `Button` 新增 `control_height` 字段；`layout()` 中 `natural_height.max(control_height)`；空标签 layout 输出高度 == 36.0
  - Verify: `cargo test widget::base::button` 通过
  - Files: `src/widget/base/button.rs`
  - 检查点: **CP2**

## Phase 3: 复合适适配

- [x] T4: IconInput 适配 control_height
  - Acceptance: `IconInput` layout 输出高度与 TextInput 对齐（36.0）；图标区域高度同步
  - Verify: `cargo test widget::form::icon_input` 通过
  - Files: `src/widget/form/icon_input.rs`
  - 检查点: **CP3**

## Phase 4: 集成验证

- [ ] T5: 全量验证
  - Acceptance: showcase 中按钮与输入框并排视觉对齐；全量测试通过；clippy 零警告
  - Verify: `cargo test --lib --tests && cargo clippy -- -D warnings && cargo fmt --check`
  - Files: `examples/showcase.rs`（仅验证，不改代码）
  - 检查点: **CP4**
