# Todo: 丹青 (danqing) M2

> 详见 `docs/spec-m2.md` 与 `docs/plan-m2.md`。

## Phase 1: 焦点与事件路由基础设施
- [x] 扩展 `Event` 枚举(Ime/Copy/Cut/Paste/FocusIn/FocusOut,Key 携带 shift/ctrl)
- [x] `Widget` trait 新增默认方法:focusable/children/children_mut/selected_text/ime_area/wants_ime/animate
- [x] 实现 `FocusManager`(焦点链、Tab/Shift+Tab、点击聚焦)
- [x] `window.rs` 集成焦点路由、IME 转发、剪贴板快捷键
- [x] 改造 `Flow`/`Column`/`Row`/`Padding`/`Center`/`Box`/`Button` 支持新 trait 方法
- [x] 单元测试与 `cargo test`/`clippy`/`fmt` 全绿

## Phase 2: TextInput 组件
- [x] 单行可编辑文本:文本内容 + 光标/选区
- [x] 键盘处理:字符输入、Backspace/Delete、方向键、Home/End、Ctrl+A
- [x] 渲染:背景、选区高亮、文本、光标闪烁
- [x] IME preedit 显示 + commit 插入
- [x] `on_change` 回调产出应用消息
- [x] 单元测试覆盖插入/删除/光标/选区

## Phase 3: 剪贴板与 IME 集成
- [x] 引入 `arboard` 依赖
- [x] Ctrl+C/X/V 路由与 `selected_text()` 回调
- [x] winit IME 事件转换、`set_ime_allowed`、`set_ime_cursor_area`
- [x] TextInput 处理 Ime Preedit/Commit/Disabled

## Phase 4: showcase 与 Button 焦点
- [x] `Button` 支持 focusable、空格/回车触发、焦点环视觉反馈
- [x] showcase 新增 TextInput 输入区与实时回显
- [x] 焦点顺序:Button → TextInput

## Phase 5: 打磨验收
- [x] 新增集成测试 `tests/focus_input.rs`
- [x] `cargo test` 全绿(53 项)
- [x] `cargo clippy -- -D warnings` 通过
- [x] `cargo fmt --check` 通过
- [x] `cargo build --release` 成功
- [x] `cargo run --example showcase` 启动无 wgpu 校验错误
- [x] 更新 `docs/spec-m2.md` 为已实现状态

---

**验收结果**:M2 6 条 Success Criteria 全部通过,2026-07-16 关闭。

---

## 2026-07-18 启动/关闭体验优化(已关闭后补充)

- [x] 定位 debug 构建启动白屏/关闭延迟根因:wgpu 校验层 + 适配器枚举在部分机器上耗时 1~3s。
- [x] 默认关闭 wgpu 校验层,提供 `DANQING_WGPU_VALIDATION=1` opt-in。
- [x] 窗口初始化期间隐藏,渲染上下文就绪后再显示,避免白屏。
- [x] 关闭窗口时立即隐藏,提升关闭响应感。
- [x] 更新 `docs/spec-m2.md` 与 `CLAUDE.md`。
