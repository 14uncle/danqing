# Todo: 丹青 (danqing) M3

> 依据 `docs/spec-m3.md`（已批准，2026-07-18）与 `docs/plan-m3.md`。

## Phase 1: 前置清理与渲染基础
- [ ] **Task 0** M1/M2 文档清理：更新 `docs/spec.md`、`README.md`、`examples/showcase.rs` 头、`tasks/todo-m2.md` 测试数
- [ ] **Task 1** 渲染裁剪基础：`Rect::intersect`/`is_empty`、RectBatch/TextBatch clip stack、WGSL discard、单元测试

## Phase 2: 容器与排版
- [ ] **Task 2** `Scrollable` 容器：滚轮、偏移限幅、视口裁剪、焦点命中裁剪、单元测试
- [ ] **Task 3** 多行文本排版 `src/text/line_layout.rs`：显式 `\n` + 字符级 soft-wrap、单元测试

## Phase 3: 多行编辑与拖拽选区
- [ ] **Task 4** `TextArea` 组件：多行光标/选区/键盘/IME/剪贴板/命中测试、单元测试
- [ ] **Task 5** 拖拽选区：`TextInput` + `TextArea` 鼠标拖拽选区、单元测试

## Phase 4: 集成与验收
- [ ] **Task 6** showcase 更新：新增 `Scrollable` + `TextArea` 演示区
- [ ] **Task 7** M3 验收：更新 README、新建 `docs/plan-m3.md`，`cargo fmt`/`clippy`/`test`/`release` 全绿

---

**验证命令**

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --lib --tests
cargo build --release
cargo run --example showcase
```
