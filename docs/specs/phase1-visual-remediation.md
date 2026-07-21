# Spec: 丹青阶段 1 视觉Remediation —— 从“苍白”到“有层次的品牌感”

> 前置意图：阶段 1 设计系统已实现，但 `showcase` 呈现效果过淡、层次弱、品牌感不足。本次改动在现有架构内做视觉升级，取“干净生产力”与“毛玻璃沉浸”的中间态。
>
> 对应文档：`docs/specs/phase1-design-system.md`。

## Objective

在**不新增业务组件、不大改布局系统、不实现运行时 backdrop blur、不落地完整暗色模式** 的前提下，通过调整 `LightTheme` token、重制背景资产、优化 `TitleBar` 与窗口装饰、优化 LOGO、重组 `showcase` 布局，让阶段 1 演示页达到：

- **可读**：文字与背景有足够对比，主次信息清晰。
- **有层次**：背景、卡片、输入框、按钮之间通过颜色、阴影、边框拉开深度。
- **有品牌感**：品牌蓝贯穿标题栏、按钮、光标、焦点；LOGO 与标题更醒目。

**用户故事：**

- 作为 丹青 开发者，运行 `cargo run --example showcase` 时，看到一个像样的效率工具界面，而不是未完成的 demo。
- 作为后续 POC（剪贴板管理器）的参考，我能直接复用这套更成熟的 token 与组件样式。

## Tech Stack

- **语言**：Rust 2021 edition
- **窗口/事件**：`winit` 0.30
- **自绘**：`wgpu` 0.30
- **字体**：OFL 回退字体 `ZCOOL XiaoWei`（`assets/fonts/fallback-font.ttf`）
- **位图**：`assets/background/gradient.png`、`assets/background/noise.png` 重制；不新增运行时位图加载依赖。
- **构建工具**：`cargo`

## Commands

```bash
# 视觉验证
 cargo run --example showcase

# 纯逻辑测试
 cargo test --lib --tests

# 静态检查
 cargo clippy -- -D warnings

# 格式化
 cargo fmt
 cargo fmt --check

# 发布构建验证
 cargo build --release
```

## Project Structure

变更范围集中在主题、资产、标题栏、showcase 布局：

```
src/
  theme.rs                  # 改造：调整 LightTheme 颜色/阴影/圆角 token
  window.rs                 # 改造：Windows 下去装饰（undecorated），让 TitleBar 接管
  widget/
    box.rs                  # 改造：默认绘制边框与阴影，增强卡片感
    button.rs               # 改造：hover/pressed 状态对比度；文字反白
    text_input.rs           # 改造：背景更实、边框更明确、focus 环使用 accent
    text_area.rs            # 改造：与 TextInput 一致的 surface/border/focus 处理
    scrollable.rs           # 改造：滚动条视觉更明显
    title_bar.rs            # 改造：完全接管标题栏绘制与窗口控制（拖拽、双击最大化、按钮）
examples/
  showcase.rs               # 改造：按功能分组卡片化，减少过度拉伸
assets/
  background/
    gradient.png            # 重制：品牌蓝倾向渐变 + 柔和径向光晕
    noise.png               # 重制：更低对比度、更细腻
    glow.png                # 新增：径向光晕叠加图（预渲染）
  logo/                     # 优化：提升小尺寸辨识度
    logo.svg                # 新增/更新：LOGO 矢量源文件
    logo_*.png
    logo.ico
tools/
  export-logo.py            # 新增/更新：从 logo.svg 批量导出 PNG/ICO
docs/specs/
  phase1-visual-remediation.md  # 本 spec
tasks/
  plan-phase1-visual-remediation.md  # 实现计划（本 spec 批准后产出）
```

## Code Style

保持现有约定：

- 公开 API 经 `src/lib.rs` re-export。
- 公共类型/函数写中文文档注释。
- 组件内部实现英文命名。
- 颜色/圆角/阴影一律从 `Theme` 读取，不保留魔法值。
- 新增 `.rs` 文件头必须包含 `//! @author 十四叔` 与 `//! @date yyyy/MM/dd`。

## Testing Strategy

- **单元测试**：`theme.rs` 测试 token 顺序/非负/可见性；各 widget 测试默认样式来源。
- **集成测试**：`tests/design_system.rs` 验证组件绘制后 RectBatch 颜色/半径符合主题。
- **视觉验证**：`cargo run --example showcase` 人工确认整体效果。
- **回归检查**：`cargo test --lib --tests`、`cargo clippy -- -D warnings`、`cargo fmt --check` 全绿。

## Boundaries

- **Always：**
  - 提交前跑 `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests`。
  - 所有视觉值走 `Theme` token。
  - 新增/修改公开 API 经 `src/lib.rs` re-export。

- **Ask first：**
  - 新增外部依赖。
  - 改变 `examples/showcase.rs` 的页面导航结构（本次仅做布局/分组调整，不动导航）。
  - 引入运行时 backdrop blur 或新渲染管线。
  - 替换 LOGO 设计方向（如需重绘 LOGO）。

- **Never：**
  - 破坏 `TextInput`/`TextArea` 的编辑、IME、焦点、撤销/重做行为。
  - 在 `widget/`、`layout.rs`、`event.rs`、`text/` 引入 `winit`/`wgpu` 依赖。
  - 删除或屏蔽现有测试以通过检查。

## Success Criteria

1. `cargo run --example showcase` 打开后，Windows 上**不再显示 native 标题栏**；`TitleBar` 完全接管标题、LOGO、三个窗口按钮、拖拽区、双击最大化/还原。
2. 界面不再“苍白”：背景、卡片、输入框之间有明显区分。
3. 文字可读性提升：`text_primary` 与背景对比度足够，`text_secondary` 仍明显可辨。
4. `Button` 使用品牌色填充 + 白色/浅色文字；hover/pressed 状态反馈清晰。
5. `TextInput`/`TextArea` 有明确的白色背景和边框；focus 时边框/光环使用 `accent`。
6. `TitleBar` 左侧 LOGO 与标题更醒目，右侧按钮 hover 背景清晰、关闭按钮使用 `danger`。
7. `showcase` 布局按功能分组（品牌/色板/输入/键盘），减少过度 `fill` 拉伸，更像演示页而非测试页。
8. 背景图 `gradient.png` 使用品牌蓝倾向渐变 + 柔和光晕，`noise.png` 更淡更细腻；新增 `glow.png` 叠加光晕。
9. LOGO 小尺寸辨识度提升，`logo.svg` 源文件与导出脚本同步更新。
10. 全部测试通过：`cargo test --lib --tests` 全绿。
11. 零警告：`cargo clippy -- -D warnings` 通过；`cargo fmt --check` 通过。
12. 二进制资产体积可控：背景图 ≤ 1 MB/张，LOGO PNG ≤ 100 KB/张。

## Open Questions

以下问题已在本 spec 创建后与作者确认：

1. **背景光晕**：✅ 接受预渲染的径向光晕位图（新增 `assets/background/glow.png`）。
2. **标题栏**：✅ 优化 `TitleBar` **并**隐藏 native 标题栏；Windows 下去装饰，由 `TitleBar` 接管拖拽、双击最大化、三个窗口按钮。
3. **LOGO**：✅ 对现有 LOGO 做小幅优化，提升小尺寸辨识度；`logo.svg` 源文件与导出脚本同步维护。
4. **暗色模式**：✅ 本次仅调优 `LightTheme`，`DarkTheme` 继续延后。

## Related Documents

- `docs/specs/phase1-design-system.md` — 阶段 1 原始规格
- `docs/ideas/danqing-efficiency-tool-glassmorphism.md` — 战略定位
- `CLAUDE.md` — 项目约定与命令
