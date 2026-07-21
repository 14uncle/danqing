# Spec: 丹青 LOGO 重新设计

## Objective

为 Rust 自绘 UI 框架 **丹青 (danqing)** 设计一套新的品牌标识，替换当前含义不明的“两个半圆”旧版 LOGO。新 LOGO 需：

- 采用**抽象图形**，避免文字或具象物体；
- 与阶段 1 浅色毛玻璃设计系统视觉一致；
- 可直接在自绘标题栏中用现有矩形 SDF 原语绘制；
- 提供 SVG 源文件与多尺寸 PNG/ICO 导出流程。

## Design Concept

### 名称联想

“丹青”本义为中国画所用的朱砂与石青颜料，引申为绘画、艺术。框架的价值是“在屏幕上绘制界面”，因此 LOGO 概念抽象为：

> **一块半透亮的玻璃画布（窗口/界面），上面落有一滴颜料。**

### 图形构成

| 元素 | 形状 | 含义 | 绘制方式 |
|---|---|---|---|
| 外框 | 圆角正方形 | 窗口 / 画布 / 界面 | 圆角矩形描边或填充 |
| 内点 | 实心圆 | 颜料滴 / 笔触 | 实心圆 |

整体构图居中、对称中带有一点非对称（圆点偏右下），保持识别度在小尺寸（16×16）下依然清晰。

## Tech Stack

- 矢量源文件：SVG（纯文本，可版本控制）
- 导出脚本：Python 3 + `Pillow`（不依赖系统 Cairo）
- 运行时绘制：`RectBatch::push_rect` 圆角矩形（标题栏内）
- 色彩来源：`LightTheme` token

## Commands

```bash
# 1. 导出 PNG 与 ICO（需先安装 Python 依赖）
python tools/export-logo.py

# 2. 运行阶段 1 演示页查看标题栏新 LOGO
cargo run --example showcase

# 3. 静态检查与测试
cargo fmt
cargo clippy -- -D warnings
cargo test --lib --tests
```

## Project Structure

```
assets/logo/
  logo.svg              # 新增：矢量源文件
  logo_16.png           # 现有：更新后由脚本重新导出
  logo_24.png
  logo_32.png
  logo_48.png
  logo_256.png
  logo.ico
src/widget/title_bar.rs # 修改：用新图形替换占位圆角矩形
tools/
  export-logo.py        # 新增：SVG → PNG/ICO 导出脚本
```

## Color Palette

LOGO 使用 `LightTheme` 已定义的 token，禁止硬编码其他颜色：

| Token | RGBA | 用途 |
|---|---|---|
| `accent()` | `#3B82F6` (59, 130, 246, 1.0) | 外框描边、内点实心填充 |
| `surface()` | `#FFFFFF` 不透明度 0.85 | SVG 与 TitleBar 中玻璃画布填充 |

TitleBar 内用 `surface()` 作为内部填充、用 `accent()` 作为外框与颜料点，确保在毛玻璃标题栏背景上有足够对比度。

## Sizes and Formats

| 文件 | 尺寸 | 用途 |
|---|---|---|
| `logo.svg` | 矢量 | 源文件、官网、README |
| `logo_16.png` | 16×16 | 窗口小图标、任务栏小尺寸 |
| `logo_24.png` | 24×24 | 工具栏 |
| `logo_32.png` | 32×32 | 窗口图标、任务栏标准尺寸 |
| `logo_48.png` | 48×48 | 高 DPI 任务栏 |
| `logo_256.png` | 256×256 | winit 窗口图标 |
| `logo.ico` | 多尺寸 | Windows 可执行文件图标 |

导出脚本应生成：`16, 24, 32, 48, 256` PNG 与包含 `16, 32, 48, 256` 的 ICO。

## Code Style

TitleBar 内绘制 LOGO 应继续使用现有 `RectBatch` 原语，不引入图片纹理依赖。`logo_size` 取 `theme.spacing_xl()`，图形比例与 SVG 对应：

```rust
let logo_rect = self.logo_rect(area);
let logo_size = logo_rect.size.width;

// 外框：accent 圆角矩形，与 SVG 外框内缩量一致。
let outer_inset = logo_size * 0.06;
let frame_rect = logo_rect.inset(outer_inset);
let frame_radius = logo_size * 0.25;
rects.push_rect(frame_rect, self.logo_frame_color, frame_radius);

// 内部填充：白色半透明，描边宽度约 16%。
let stroke = logo_size * 0.16;
let fill_rect = frame_rect.inset(stroke);
let fill_radius = (frame_radius - stroke).max(0.0);
rects.push_rect(fill_rect, self.logo_fill_color, fill_radius);

// 颜料滴：实心 accent 圆，直径约 38%，中心偏右下。
let dot_size = logo_size * 0.38;
let dot_offset = logo_size * 0.58;
let dot_rect = Rect::from_xywh(
    logo_rect.origin.x + dot_offset - dot_size / 2.0,
    logo_rect.origin.y + dot_offset - dot_size / 2.0,
    dot_size,
    dot_size,
);
rects.push_rect(dot_rect, self.logo_dot_color, dot_size / 2.0);
```

## Testing Strategy

- `tests/assets.rs`：继续断言 `logo.svg`、各尺寸 PNG 与 `logo.ico` 存在且非空。
- `src/widget/title_bar.rs` 单元测试：保留布局与事件测试，新增 LOGO 颜色使用 theme token 的断言。
- 人工验证：`cargo run --example showcase` 确认标题栏左侧显示新 LOGO。

## Boundaries

- **Always:** 任何颜色使用 `LightTheme` token；新增资产必须提交到 `assets/`。
- **Ask first:** 引入新的 crate 用于 SVG 渲染；改变 LOGO 概念方向。
- **Never:** 在 TitleBar 中直接加载 PNG 纹理（阶段 1 不扩展通用 Image Widget）；提交没有 SVG 源的位图。

## Success Criteria

- [ ] `assets/logo/logo.svg` 存在，且只使用 `#3B82F6` 与白色半透明。
- [ ] `tools/export-logo.py` 可从 SVG 生成全部 PNG 与 ICO。
- [ ] `src/widget/title_bar.rs` 不再使用单一 `logo_color` 圆角矩形占位。
- [ ] TitleBar 内 LOGO 由“圆角正方形外框 + 实心圆点”构成，颜色来自 theme token。
- [ ] `cargo fmt`、`cargo clippy -- -D warnings`、`cargo test --lib --tests` 全绿。
- [ ] `cargo run --example showcase` 标题栏左侧显示新 LOGO（人工确认）。

## Open Questions

1. 是否需要在 ICO 中包含 24×24 尺寸？（Windows 通常需要 16/32/48/256，24 多用于 macOS/Linux。）
2. 阶段 2 剪贴板 POC 的托盘图标是否复用同一套 LOGO？
