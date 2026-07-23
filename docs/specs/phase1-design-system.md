# Spec: 丹青阶段 1 — 设计系统 + 品牌视觉

> 对应战略文档：`docs/ideas/danqing-efficiency-tool-glassmorphism.md` 阶段 1。
> 定稿前提：Open Questions 已全部确认（见文末）。

## Objective

为 丹青 建立一套面向效率工具的现代毛玻璃（Glassmorphism）设计系统，并同步完成品牌视觉资产。让 `examples/showcase.rs` 呈现出统一、可识别、具有毛玻璃质感的视觉体验，为阶段 2 的剪贴板管理器提供可复用的视觉与组件基础。

**用户故事：**

- 作为 丹青 的开发者，我能在代码中引用标准化的颜色、字体、间距、圆角、阴影、动效 token，而不是散落各处的魔法值。
- 作为 丹青 的示例使用者，我运行 `cargo run --example showcase` 时，能看到一个具有毛玻璃背景、自绘标题栏、统一圆角与阴影的窗口。
- 作为 丹青 的维护者，我能在 showcase 中验证 `Box`、`Button`、`TextInput`、`TextArea`、`Scrollable` 均遵循新设计系统。

**验收状态：** 本 spec 通过后才进入实现阶段。

## Tech Stack

- **语言**：Rust 2021 edition
- **窗口/事件**：`winit` 0.30
- **自绘**：`wgpu` 0.30
- **字体**：OFL 回退字体 `ZCOOL XiaoWei`（位于 `assets/fonts/`，提交在版本控制中）
- **位图加载**：阶段 1 使用固定渐变/噪声图作为毛玻璃背景，位图资源提交到 `assets/`。
- **构建工具**：`cargo`

## Commands

```bash
# 运行阶段 1 视觉验证
cargo run --example showcase

# 纯逻辑测试（布局、事件命中、图集分配）
cargo test

# 静态检查，必须零警告
cargo clippy -- -D warnings

# 格式化
cargo fmt

# 发布构建验证
cargo build --release
```

## Project Structure

```
src/
  lib.rs                    # 公开 API 统一 re-export
  theme.rs                  # 新增：设计 token（颜色、字体、间距、圆角、阴影、动效）
  widget/
    box.rs                  # 改造：使用 theme token
    button.rs               # 改造：使用 theme token
    text_input.rs           # 改造：使用 theme token
    text_area.rs            # 改造：使用 theme token
    scrollable.rs           # 改造：使用 theme token
    title_bar.rs            # 新增：自绘标题栏（LOGO + 标题 + 最小化/最大化/关闭按钮视觉）
examples/
  showcase.rs               # 改造：呈现毛玻璃风格整体效果
assets/
  logo/                     # 新增：LOGO 与图标资源（PNG/ICO 可提交到仓库）
    logo_16.png
    logo_24.png
    logo_32.png
    logo_48.png
    logo_256.png
    logo.ico
  background/               # 新增：毛玻璃背景素材（固定渐变/噪声图）
    noise.png
    gradient.png
docs/
  specs/
    phase1-design-system.md # 本 spec 文件
```

## Code Style

**设计 token 以 Rust trait + 结构体 + `const` 常量表达**，公开 API 配中文文档注释。示例：

```rust
//! 丹青设计系统 token。

use crate::render::Color;

/// 主题接口，预留暗色模式扩展。
pub trait Theme: Clone + Copy + std::fmt::Debug {
    /// 背景色。
    fn background(&self) -> Color;
    /// 表面浮层色。
    fn surface(&self) -> Color;
    /// 主强调色。
    fn accent(&self) -> Color;
    /// 主文字色。
    fn text_primary(&self) -> Color;
    /// 次级文字色。
    fn text_secondary(&self) -> Color;
    /// 边框/分割线颜色。
    fn divider(&self) -> Color;
}

/// 浅色主题。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightTheme;

impl Theme for LightTheme {
    fn background(&self) -> Color { Color::rgba(245, 247, 250, 0.85) }
    fn surface(&self) -> Color { Color::rgba(255, 255, 255, 0.60) }
    fn accent(&self) -> Color { Color::rgb(15, 118, 110) }
    fn text_primary(&self) -> Color { Color::rgb(30, 41, 59) }
    fn text_secondary(&self) -> Color { Color::rgb(100, 116, 139) }
    fn divider(&self) -> Color { Color::rgba(0, 0, 0, 0.08) }
}
```

**组件使用 token 而非魔法值。** 例如 `Button` 的圆角、内边距、阴影从 `theme.button()` 读取。

## Testing Strategy

- **单元测试**：布局、颜色计算、token 查找等纯逻辑写入对应模块的 `#[cfg(test)]`。
- **集成测试**：在 `tests/` 下新增 `design_system.rs`，验证：
  - 默认主题 token 不为空/透明；
  - 组件构造时正确应用 theme；
  - 标题栏事件命中区域正确。
- **视觉验证**：`cargo run --example showcase` 人工确认毛玻璃效果、标题栏、图标。
- **静态检查**：`cargo clippy -- -D warnings` 零警告；`cargo fmt --check` 通过。

## Boundaries

- **Always：**
  - 新增公开类型必须经 `src/lib.rs` re-export。
  - 所有公共类型/函数写中文文档注释。
  - 修改前先运行 `cargo test` 确认基线全绿。
  - 提交前运行 `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test`。

- **Ask first：**
  - 新增外部依赖（如 `image`、`vello` 等）。
  - 改变字体回退策略或新增字体文件。
  - 改变 `examples/showcase.rs` 的整体页面结构或导航。
  - 引入暗色模式的完整实现（本阶段仅预留接口）。

- **Never：**
  - 破坏现有 `TextInput`/`TextArea` 的撤销/重做、IME、焦点行为。
  - 在 `widget/`、`layout.rs`、`event.rs` 中引入 `winit`/`wgpu` 依赖。
  - 删除或屏蔽现有测试以通过 clippy。

## Success Criteria

1. `src/theme.rs` 存在并导出 `Theme` trait、`LightTheme` 及基础 token（颜色 ≥8 个、字体层级 ≥3 级、间距 ≥5 档、圆角 ≥3 档、阴影 ≥2 档、动效曲线 ≥2 条）。
2. `Box`、`Button`、`TextInput`、`TextArea`、`Scrollable` 不再使用裸魔法颜色/圆角/阴影值，改为从 `theme` 读取。
3. 新增 `src/widget/title_bar.rs`，实现自绘标题栏：左侧 LOGO + 标题，右侧最小化/最大化/关闭按钮；按钮仅提供视觉反馈（悬停/按下状态），本阶段不调用窗口控制 API。
4. `examples/showcase.rs` 展示毛玻璃整体效果，窗口背景使用固定渐变/噪声图营造半透明 + 模糊质感，组件统一圆角与阴影。
5. 完成 丹青 LOGO 设计，导出 `16/24/32/48/256 px` PNG 与 `logo.ico`。
6. 窗口图标与任务栏/exe 图标使用新 LOGO（在 `window.rs` 中设置 `WindowAttributes::with_window_icon`）。
7. 暗色模式接口预留：通过 `Theme` trait 支持后续扩展 `DarkTheme`。
8. 全部测试通过：`cargo test` 全绿。
9. 零警告：`cargo clippy -- -D warnings` 通过。
10. 人工验证：`cargo run --example showcase` 能看到统一毛玻璃视觉。

## Open Questions — 已确认

1. **LOGO 设计风格**：由本阶段一并设计，方向待实现时确定。
2. **暗色模式接口**：采用 `trait Theme` + `struct LightTheme/DarkTheme`。
3. **静态预渲染模糊背景**：使用固定渐变/噪声图作为背景，不截图桌面壁纸。
4. **标题栏按钮**：最大化/最小化/关闭只需视觉反馈，不调用窗口控制 API。
5. **资源目录**：字体、LOGO、背景图统一放 `assets/` 并提交版本控制。

## Related Documents

- `docs/ideas/danqing-efficiency-tool-glassmorphism.md` — 丹青战略定位与阶段划分
- `CLAUDE.md` — 项目约定与命令
- `tasks/archive/plan.md` — M1 实现计划
- `tasks/archive/plan-m2.md` — M2 焦点与输入计划
- `tasks/archive/plan-m3.md` — M3 滚动与多行文本计划
