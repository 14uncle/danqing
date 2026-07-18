# Spec: 丹青 (danqing) M2 — 焦点系统与文本输入

> 状态: **已实现**(2026-07-16)
> 本规格定义 M2 最小闭环:在 M1 保留模式组件树上引入焦点、可编辑文本输入、剪贴板与 IME。

## 目标

M1 已打通"winit 事件 → 组件树 → wgpu 像素"。M2 让组件树能接收键盘焦点并输入文字,使丹青从"能看"进化到"能交互输入"。

- 增加全局焦点管理,支持 Tab/Shift+Tab 遍历、鼠标点击聚焦、焦点视觉反馈。
- 新增 `TextInput` 组件:单行可编辑文本,含光标、选区、键盘编辑。
- 支持系统剪贴板复制/剪切/粘贴。
- 支持 winit IME 合成(preedit + commit),为中文/日文等输入法做准备。
- 保持 M1 架构约束:`widget/`、`layout.rs`、`event.rs` 仍为纯逻辑,平台 API 只出现在 `window.rs`/`render/`。

## 技术栈

在 M1 栈基础上新增:

| 职责 | 选型 | 说明 |
|---|---|---|
| 剪贴板 | `arboard` 3.x | 纯 Rust 跨平台文本剪贴板 |
| IME | `winit` 0.30 内置 IME | `WindowEvent::Ime` + `Window::set_ime_cursor_area` |

M1 既有栈:`winit` 0.30、`wgpu` 30、`fontdue` 0.9、`etagere` 0.3、`font-kit` 0.14。

## 命令

```bash
# 运行 M2 演示页
cargo run --example showcase

# 全部测试(新增焦点/输入纯逻辑单测)
cargo test

# 静态检查(必须零警告)
cargo clippy -- -D warnings

# 格式化
cargo fmt
cargo fmt --check

# 发布构建
cargo build --release
```

## 项目结构

```
examples/showcase.rs            → 加入 TextInput 焦点演示
src/
  app.rs                        → 新增 AnimationCtx
  event.rs                      → 新增 Ime 事件与 Focus 相关事件;Key 携带 shift/ctrl
  window.rs                     → 焦点路由、Ime 转发、剪贴板、IME 光标区域
  widget/
    mod.rs                      → Widget trait 增加 focusable/children/ime_area/wants_ime/animate
    focus.rs                    → 焦点管理器:焦点链、Tab 遍历、点击聚焦
    text_input.rs               → 单行文本输入组件
    button.rs                   → 支持 focusable + 空格/回车触发 + 焦点环
tests/focus_input.rs            → 焦点与文本输入集成测试
```

## 代码风格

与 M1 保持一致:

- 公开 API 经 `src/lib.rs` re-export,不暴露深层路径。
- 公共类型/函数写中文文档注释。
- 提交前必须 `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test` 全绿。

新增约定:

- 焦点管理器是纯逻辑,放在 `widget/focus.rs`,不依赖 `winit`。
- 剪贴板操作由 `window.rs` 适配层发起或封装,不直接进入 `widget/`。

## 测试策略

- **单元测试**:焦点管理器(焦点链构建、Tab 顺序、点击聚焦)、`TextInput` 编辑逻辑(插入/删除/光标移动/选区)。
- **集成测试**:在 `tests/` 中构建包含 `TextInput` 与 `Button` 的树,模拟 Tab、字符键、Backspace、Ctrl+A/C/V 事件,断言焦点状态与文本内容。
- **渲染/IME 验证**:通过 `cargo run --example showcase` 人工确认:焦点环、光标闪烁、输入法合成、剪贴板操作。

## 边界

**Always:**
- 提交前跑 `cargo fmt`、`cargo clippy -- -D warnings`、`cargo test`。
- 新公共类型/函数写中文文档注释。
- 新增组件必须出现在 `examples/showcase.rs`。

**Ask first:**
- 新增外部依赖(如 `arboard`)。
- 修改现有公开 API(如 `Event` 枚举、`Widget` trait 签名)。
- 改动渲染管线架构(如引入 scissor/clip)。

**Never:**
- 在 `widget/`、`layout.rs`、`event.rs` 中写平台特定代码。
- 提交字体等二进制。
- 为通过测试删除/跳过失败测试。

## 成功标准

1. showcase 页面包含 `TextInput` 与 `Button`,按 Tab/Shift+Tab 可在二者间切换焦点,焦点组件有视觉焦点环。
2. `TextInput` 支持:输入字符、Backspace/Delete 删除、方向键移动光标、Home/End、Ctrl+A 全选、Ctrl+C/X/V 复制剪切粘贴。
3. IME 合成可见:输入中文时显示下划线 preedit 文本,按空格/回车 commit 到输入框。
4. 鼠标点击 `TextInput` 可聚焦并定位光标(如能实现选区更佳)。
5. `cargo test` 全绿,`cargo clippy -- -D warnings` 通过,`cargo fmt --check` 通过。
6. 适配层之外无新增平台专有 API。

## 假设(已确认)

1. M2 只做**单行** `TextInput`;多行/自动换行/滚动文本域留到 M3。
2. M2 不做通用滚动容器 `Scrollable`;延后到 M3。
3. 剪贴板依赖使用 **`arboard`** 3.x。
4. 焦点系统仅支持默认 Tab 顺序(按组件树深度优先/绘制顺序);显式 `tab_index` 留待后续。
5. 鼠标拖拽选区留到 M3;M2 仅支持键盘选区(Shift+方向键/Ctrl+A)。

## 开放问题

1. M3 是否优先做多行文本域 + 滚动容器?
2. 是否需要 `tab_index` 或自定义焦点顺序?
3. 是否需要把 `AnimationCtx` 扩展为通用动画系统?
4. 是否需要双击选词、右键菜单等桌面输入惯例?
