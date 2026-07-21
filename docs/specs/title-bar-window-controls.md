# Spec: 自绘标题栏接管原生窗口控制

## Objective

让 `examples/showcase.rs` 使用自绘 `TitleBar` 完全替代操作系统原生标题栏。
标题栏自己管理：应用 LOGO、窗口标题、最小化 / 最大化（还原）/ 关闭 三个按钮，并支持拖拽移动与双击最大化。

## Motivation

阶段 1 设计系统采用毛玻璃风格，原生标题栏在视觉上与自绘界面割裂。
自绘标题栏能保证 LOGO、标题文本、按钮视觉与主题 token 一致，并为阶段 2 的效率工具 POC 提供统一的品牌入口。

## Tech Stack

- `winit` 0.30：`WindowAttributes::with_decorations(false)`、`Window::drag_window()`、`set_minimized()`、`set_maximized()`、`is_maximized()`。
- 丹青现有 `Widget` 事件/消息体系：`TitleBar` 通过回调产出消息，`window.rs` 的 `Handler` 识别并执行窗口动作。

## Commands

```bash
# 运行验证
cargo run --example showcase

# 静态检查
cargo fmt --check
cargo clippy -- -D warnings

# 测试
cargo test --lib --tests

# 发布构建
cargo build --release
```

## Project Structure

- `src/event.rs`：新增 `WindowAction` 枚举（纯逻辑，供 widget 与 window 共享）。
- `src/widget/title_bar.rs`：扩展按钮回调与拖拽回调；绘制 LOGO 占位（品牌色圆角矩形）。
- `src/window.rs`：Windows 下去装饰；消息消费时识别 `WindowAction` 并调用窗口 API。
- `examples/showcase.rs`：构造 `TitleBar` 时绑定窗口动作回调；根组件留出标题栏高度。
- `tests/`：更新/新增事件分发与标题栏集成测试。

## Code Style

- 公开 API 中文文档注释，内部英文命名。
- 新 `.rs` 文件头保留 `//! @author 十四叔` / `//! @date yyyy/MM/dd`。
- 平台相关代码只出现在 `window.rs`；`widget/` 保持纯逻辑。

## Testing Strategy

- 单元测试：`src/widget/title_bar.rs` 覆盖按钮 hover/pressed、回调触发、拖拽区域识别。
- 集成测试：`tests/title_bar_window.rs` 模拟点击关闭/最小化/最大化按钮，验证消息类型正确。
- 人工验证：`cargo run --example showcase` 在 Windows 上确认无原生标题栏、按钮可点击、窗口可拖拽/最大化/关闭。

## Boundaries

- **Always**：阶段 1 组件使用 theme token；提交前跑 fmt/clippy/test。
- **Ask first**：引入新 crate、改变跨平台装饰策略、修改 `Widget` trait 签名。
- **Never**：在 `widget/` 中直接调用 `winit`/`wgpu` API；把窗口控制逻辑泄漏到 App 层。

## Success Criteria

1. Windows 上 `showcase` 窗口不显示原生标题栏和边框。
2. 自绘标题栏左侧显示品牌色 LOGO 占位和标题文本。
3. 三个窗口按钮 hover/pressed 视觉反馈正确。
4. 点击关闭按钮干净退出；点击最小化按钮窗口最小化；点击最大化按钮在最大化/还原间切换。
5. 在标题栏非按钮区域按住左键可拖拽移动窗口；双击最大化/还原。
6. 其他平台保留原生标题栏作为降级，功能不受影响。
7. `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test --lib --tests` / `cargo build --release` 全绿。

## Open Questions

1. LOGO 是否必须渲染真实 PNG 图片？当前阶段先使用品牌色圆角矩形占位，后续可替换为图片 Widget。
2. 无边框窗口的拖拽缩放是否需要本次实现？本次不做自定义 resize 边框，依赖后续任务补充。
3. macOS / Linux 是否也去掉装饰？本次仅 Windows 去掉，其他平台保留 OS 标题栏降级。
