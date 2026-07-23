# Spec: 全平台统一自绘标题栏

## Objective

让自绘 `TitleBar` 在全部平台（Windows / Linux / macOS）接管原生标题栏：
Linux 与 Windows 行为一致（右侧最小化 / 最大化 / 关闭三键）,macOS 适配左侧红绿灯样式
（hover 显示 × / − / + 符号）。

## Motivation

`TitleBar` 组件无条件进入组件树，而 `with_decorations(false)` 此前仅在 Windows 生效
（`src/window.rs`),Linux / macOS 上出现「原生标题栏 + 自绘标题栏」并存的双标题栏。
统一自绘后三平台品牌视觉一致，落实阶段 1 设计系统的跨平台承诺。

本 spec 关闭 `docs/specs/title-bar-window-controls.md` 的 Open Question 3,
并取代其 Success Criteria 第 6 条（其他平台保留原生标题栏降级）。

## Tech Stack

- `winit` 0.30:`WindowAttributes::with_decorations(false)` 全平台生效;
  Windows 保留既有 DWM 圆角 / 阴影设置（`apply_windows_undecorated_style`)。
- `cfg!(target_os = "macos")` 编译期分支：纯逻辑手段，不引入 `winit` 到 `widget/`。

## Commands

```bash
# 运行验证 (Windows 真机)
cargo run --example showcase

# 静态检查
cargo fmt --check
cargo clippy -- -D warnings

# 测试
cargo test --lib --tests

# 跨平台编译检查 (无真机平台的验收手段)
cargo check --target x86_64-unknown-linux-gnu
cargo check --target x86_64-apple-darwin
```

## Project Structure

- `src/window.rs`：移除 `with_decorations(false)` 的 `#[cfg(target_os = "windows")]` 门控，全平台去装饰。
- `src/widget/title_bar.rs`：新增按钮布局样式（如 `TitleBarStyle::Standard` / `TrafficLights` 枚举）,
  默认值由 `cfg!(target_os = "macos")` 解析；样式显式可构造，保证任意宿主机上两种布局均可单测。
  - 红绿灯：左侧排列，标准 macOS 规格（直径约 12px、红 `#FF5F57` / 黄 `#FEBC2E` / 绿 `#28C840`,
    颜色经 theme token 落地）;hover 显示 × / − / + 符号。
  - 绿灯复用 `WindowAction::MaximizeOrRestore`,`WindowAction` 枚举不变。
  - macOS 布局下 LOGO 与标题顺排在红绿灯之后，不做居中标题。
- `examples/showcase.rs`：无需平台分支，同一构建代码在各平台呈现对应样式（以用代测）。
- `tests/`：标题栏集成测试覆盖两种布局的按钮命中与消息类型。

## Code Style

- 公开 API 中文文档注释，内部英文命名。
- 新 `.rs` 文件头保留 `//! @author 十四叔` / `//! @date yyyy/MM/dd`。
- 平台差异通过编译期 `cfg!` 解析样式默认值；`widget/` 保持纯逻辑，不依赖 `winit`/`wgpu`。

## Testing Strategy

- 单元测试：`src/widget/title_bar.rs` 覆盖两种布局的按钮位置、hover 符号触发、回调映射
  （绿灯 → `MaximizeOrRestore`)。样式显式可构造，Windows 宿主机即可测红绿灯布局。
- 集成测试：`tests/title_bar_window.rs` 扩展，验证红绿灯布局下三个按钮产出正确的 `WindowAction`。
- 跨平台编译：`cargo check --target x86_64-unknown-linux-gnu` 与 `--target x86_64-apple-darwin` 通过。
- 人工验证 (Windows):`cargo run --example showcase` 确认行为与视觉无回归。
- Linux / macOS 无真机，以编译通过 + 逻辑单测为验收，真机问题后续按缺陷处理。

## Boundaries

- **Always**：阶段 1 组件使用 theme token；提交前跑 fmt/clippy/test。
- **Ask first**：引入新 crate；修改 `WindowAction` 枚举语义；为 macOS 引入原生全屏概念。
- **Never**：在 `widget/` 中直接调用 `winit`/`wgpu` API；为本次改动顺带实现边缘拖拽缩放。

## Success Criteria

1. 三平台 `with_decorations(false)` 生效，非 Windows 平台不再出现双标题栏。
2. Windows 现有行为与视觉零回归（DWM 圆角 / 阴影保留）。
3. Linux 标题栏与 Windows 一致：右侧最小化 / 最大化 / 关闭三键。
4. macOS 标题栏左侧红绿灯，hover 显示 × / − / + 符号；绿灯触发最大化 / 还原。
5. 红绿灯布局在 Windows 宿主机上有单测覆盖（样式显式可构造）。
6. `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test --lib --tests` 全绿，
   Linux / macOS target `cargo check` 通过。

## Open Questions

1. Wayland 下去装饰窗口可能丢失合成器阴影，先观察，不本次处理。
2. 窗口失焦时红绿灯变灰（需新增 OS 窗口焦点事件管线）留作后续增强。
3. 边缘拖拽缩放（`drag_resize_window` + 边缘热区）维持现状缺失，后续独立任务评估。
