# Todo: 丹青阶段 1 — 设计系统 + 品牌视觉

> 详见 `tasks/plan-phase1.md`（验收标准、依赖、风险）。每任务完成后勾选，检查点需人工确认。

## Phase 1: Design Tokens
- [x] **Task 1** 实现 theme.rs 与 Theme trait — `cargo check` / `clippy` 绿

## Phase 2: Visual Assets
- [ ] **Task 2** 设计并导出 LOGO 与背景图资产 — `assets/logo/*` + `assets/background/*` 就位

## Phase 3: Component Theming
- [ ] **Task 3** 改造 Box 使用 theme token（依赖 1）— `cargo test widget::box` 绿
- [ ] **Task 4** 改造 Button 使用 theme token（依赖 1）— `cargo test widget::button` 绿
- [ ] **Task 5** 实现自绘 TitleBar（依赖 1, 4）— 命中测试 + showcase 可见
- [ ] **Task 6** 改造 TextInput 使用 theme token（依赖 1）— IME/焦点行为不变
- [ ] **Task 7** 改造 TextArea 使用 theme token（依赖 1）— IME/焦点行为不变
- [ ] **Task 8** 改造 Scrollable 使用 theme token（依赖 1）— 滚动行为不变

### ⏸ Checkpoint 1: 组件 token 化完成
- [ ] `cargo test` 全绿
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] 人工 review 后进入 Phase 4

## Phase 4: Window Integration
- [ ] **Task 9** 在 window.rs 设置窗口图标（依赖 2）— showcase 窗口显示新 LOGO

## Phase 5: Showcase Integration
- [ ] **Task 10** 整合 showcase 呈现毛玻璃效果（依赖 3/4/5/6/7/8/9）— 视觉验收

### ⏸ Checkpoint 2: showcase 毛玻璃视觉可运行
- [ ] `cargo run --example showcase` 无异常
- [ ] TitleBar、背景、组件圆角/阴影一致

## Phase 6: Testing & Acceptance
- [ ] **Task 11** 编写设计系统测试 + 最终验收（依赖 10）— `cargo fmt` / `clippy -D warnings` / `test` / `build --release` 全绿

### ⏸ Checkpoint Complete: 阶段 1 关闭
- [ ] spec Success Criteria 10/10 通过
- [ ] 全部 Commands 绿；`tasks/todo-phase1.md` 全部勾选
- [ ] 人工终审，阶段 1 关闭
