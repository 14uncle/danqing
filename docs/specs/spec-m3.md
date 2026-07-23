# Spec: 丹青 (danqing) M3 — 滚动容器与多行文本域

> 状态：**已实现**（2026-07-18）  
> 依据：`docs/specs/spec-m2.md` 已实现并关闭；M1/M2 查漏补缺无阻塞性代码缺陷。

## Objective

M2 让丹青具备了单行文本输入能力。M3 在此基础上解决两个自然延伸：

1. **内容溢出容器**：引入可滚动的容器 `Scrollable`，让超出视口的内容可以被浏览。
2. **多行文本编辑**：引入 `TextArea`，支持显式换行、自动换行、光标/选区、键盘编辑、鼠标拖拽选区，并与 M2 的剪贴板、IME、焦点系统无缝集成。

成功标志：showcase 新增一个“多行文本”区域，用户可以在其中输入中文、换行、滚动、拖拽选区，并复制/粘贴。

## Tech Stack

延续 M1/M2 栈，不新增外部依赖：

| 职责 | 选型 | 说明 |
|---|---|---|
| 窗口/事件循环 | `winit` 0.30.13 | 已集成 |
| GPU 抽象 | `wgpu` 30 | 已集成 |
| 字形栅格化 | `fontdue` 0.9 | 已集成 |
| 字形图集 | `etagere` 0.3 | 已集成 |
| 系统字体 | `font-kit` 0.14 | 已集成 |

M3 所有新增能力均为框架内部实现，不引入新 crate。

## Commands

```bash
# 运行 M3 演示页
cargo run --example showcase

# 全部测试(纯逻辑,无需 GPU)
cargo test --lib --tests

# 静态检查(必须零警告)
cargo clippy -- -D warnings

# 格式化与格式检查
cargo fmt
cargo fmt --check

# 发布构建
cargo build --release
```

## Project Structure

M3 新增/修改：

```
docs/specs/spec-m3.md                → 本规格
tasks/archive/todo-m3.md         → M3 任务进度
examples/showcase.rs           → 新增 Scrollable + TextArea 演示区
src/
  layout.rs                    → 新增 Rect::intersect / is_empty
  render/rect.rs               → RectBatch clip stack + RectInstance clip 字段
  render/rect.wgsl             → 片元着色器按 clip discard
  render/text.rs               → TextBatch clip stack + GlyphInstance clip 字段
  render/text.wgsl             → 片元着色器按 clip discard
  text/line_layout.rs          → 多行文本排版(纯逻辑)
  widget/
    mod.rs                     → re-export Scrollable / TextArea
    focus.rs                   → hit_focusable 考虑祖先 hit_area 裁剪
    scrollable.rs              → 滚动容器(新建)
    text_area.rs               → 多行文本域(新建)
    text_input.rs              → 增加鼠标拖拽选区
tests/                         → 新增集成测试
```

## Code Style

与 M1/M2 保持一致：

- 公开 API 一律经 `src/lib.rs` re-export，不暴露深层模块路径。
- 所有公共类型/函数写中文文档注释；内部实现用英文命名。
- 新增 `.rs` 文件头必须包含 `//! @author 十四叔` 与 `//! @date yyyy/MM/dd`。
- 错误处理：库代码用 `thiserror`，example 用 `anyhow`。
- 依赖方向只允许向下：`widget/`、`layout.rs`、`event.rs` 不得依赖 `winit`/`wgpu`。

示例（目标风格）：

```rust
/// 滚动容器。
///
/// 包裹一个子组件，允许其在垂直/水平方向上滚动。
pub struct Scrollable {
    child: Node,
    axis: ScrollAxis,
    scroll_offset: Point,
    // ...
}
```

## Testing Strategy

- **单元测试**：写在对应模块的 `#[cfg(test)]` 中，覆盖纯逻辑：
  - `layout.rs`：`Rect::intersect`、`Rect::is_empty`。
  - `render/rect.rs` / `render/text.rs`：clip stack 行为、裁剪后实例数量/坐标。
  - `text/line_layout.rs`：换行、显式 `\n`、超宽单字、CJK。
  - `widget/focus.rs`：Scrollable 内焦点不被视口外区域命中。
  - `widget/text_area.rs`：光标移动、选区、编辑、命中测试、IME 区域。
  - `widget/text_input.rs`：鼠标拖拽选区。
- **集成测试**：在 `tests/` 中构建 `Scrollable<TextArea>` 树，模拟滚轮、Tab、字符键、鼠标事件，断言滚动偏移、焦点状态与文本内容。
- **渲染验证**：通过 `cargo run --example showcase` 人工确认滚动、换行、光标、选区、IME 候选框位置。
- **覆盖率**：M3 不设硬指标，但新增纯逻辑模块必须有单元测试。

## Boundaries

**Always：**
- 提交前跑 `cargo fmt`、`cargo clippy -- -D warnings`、`cargo test --lib --tests`。
- 新公共类型/函数带中文文档注释。
- 平台相关代码只允许出现在 `window.rs` / `render/`。
- 新增组件必须出现在 `examples/showcase.rs`。

**Ask first：**
- 新增外部依赖（M3 不计划新增）。
- 修改已稳定的公开 API（如 `Event` 枚举、`Widget` trait 签名）。
- 改动渲染管线架构（如引入 scissor/clip）。

**Never：**
- 在 `widget/`、`layout.rs`、`event.rs` 中写平台特定代码。
- 提交字体等二进制。
- 为通过测试删除/跳过失败测试。

## Success Criteria

1. showcase 页面新增 `Scrollable` 包裹的 `TextArea`，可输入多行文本并滚动浏览。
2. `TextArea` 支持：输入字符、Enter 换行、Backspace/Delete 删除、方向键跨行移动、Home/End、Ctrl+A 全选、Ctrl+C/X/V 复制剪切粘贴。
3. `TextArea` 支持鼠标拖拽选区；`TextInput` 也支持鼠标拖拽选区。
4. `Scrollable` 支持鼠标滚轮垂直滚动，滚动偏移正确限幅，视口外内容被裁剪。
5. IME 中文输入在 `TextArea` 中可见 preedit 下划线，commit 后插入文本；IME 候选框吸附在光标处。
6. `cargo test --lib --tests` 全绿，`cargo clippy -- -D warnings` 通过，`cargo fmt --check` 通过。
7. 适配层之外无新增平台专有 API；`widget/`、`layout.rs`、`event.rs` 仍为纯逻辑。

## 范围与取舍

- **Soft-wrap**：M3 先做**按字符换行**（无外部依赖），按词换行列为 M3 后期增强或 M4。
- **Scrollable 轴向**：默认垂直滚动；水平/双向滚动作为实现目标，验收以垂直为主。
- **延后到 M4**：`tab_index`、双击选词、右键菜单、通用动画系统。

## 开放问题

1. 按词 soft-wrap 是否需要在 M3 内完成，还是确认延后到 M4？
2. `Scrollable` 是否需要在 M3 支持拖拽滚动条，还是滚轮+触控板即可？
3. `TextArea` 是否需要支持placeholder提示文本？
4. M4 是否优先做 `tab_index` / 自定义焦点顺序？
