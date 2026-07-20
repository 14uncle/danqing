# Todo: 丹青阶段 1 — 设计系统 + 品牌视觉

> 详见 `tasks/plan-phase1.md`（验收标准、依赖、风险）。每任务完成后勾选，检查点需人工确认。

## Phase 1: Design Tokens
- [x] **Task 1** 实现 theme.rs 与 Theme trait — `cargo check` / `clippy` 绿

## Phase 2: Visual Assets
- [x] **Task 2** 设计并导出 LOGO 与背景图资产 — 生成 dq LOGO 多尺寸 PNG/ICO 与渐变/噪声背景,提交到 `assets/`

## Phase 3: Component Theming
- [x] **Task 3** 改造 Box 使用 theme token（依赖 1）— `cargo test widget::box` 绿
- [x] **Task 4** 改造 Button 使用 theme token（依赖 1）— `cargo test widget::button` 绿
- [x] **Task 5** 实现自绘 TitleBar（依赖 1, 4）— 命中测试 + showcase 可见
- [x] **Task 6** 改造 TextInput 使用 theme token（依赖 1）— IME/焦点行为不变
- [x] **Task 7** 改造 TextArea 使用 theme token（依赖 1）— IME/焦点行为不变
- [x] **Task 8** 改造 Scrollable 使用 theme token（依赖 1）— 滚动行为不变

### ⏸ Checkpoint 1: 组件 token 化完成
- [x] `cargo test` 全绿
- [x] `cargo clippy -- -D warnings` 零警告
- [x] 人工 review 后进入 Phase 4

## Phase 4: Window Integration
- [x] **Task 9** 在 window.rs 设置窗口图标（依赖 2）— showcase 窗口显示新 LOGO

## Phase 5: Showcase Integration
- [x] **Task 10** 整合 showcase 呈现毛玻璃效果（依赖 3/4/5/6/7/8/9）— 视觉验收

### ⏸ Checkpoint 2: showcase 毛玻璃视觉可运行
- [x] `cargo run --example showcase` 无异常
- [x] TitleBar、背景、组件圆角/阴影一致

## Phase 6: Testing & Acceptance
- [x] **Task 11** 编写设计系统测试 + 最终验收（依赖 10）— `cargo fmt` / `clippy -D warnings` / `test` / `build --release` 全绿

## Phase 7: TitleBar 接管原生标题栏
- [x] **Task 12** 自绘标题栏窗口控制（依赖 5, 11）— Windows 去装饰、按钮可用、可拖拽 ✅ 2026-07-20（代码/测试/构建绿；showcase 人工 GUI 验证待补）

### ⏸ Checkpoint Complete: 阶段 1 关闭
- [ ] spec Success Criteria 10/10 通过
- [ ] 全部 Commands 绿；`tasks/todo-phase1.md` 全部勾选
- [ ] 人工终审，阶段 1 关闭
