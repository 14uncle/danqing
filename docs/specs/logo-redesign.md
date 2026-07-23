# Spec: 丹青 LOGO 重新设计(破框朱砂)

## Objective

为 Rust 自绘 UI 框架 **丹青 (danqing)** 维护品牌标识。当前版本为第二代"破框朱砂"(2026-07-22 访谈确认),取代第一代"单色方框 + 框内圆点"——旧版存在三个痛点:构图像通用占位图标、画面静态无动感、单色蓝丢掉了"丹青"本义的朱砂 + 石青双色故事。新 LOGO 需:

- 采用**抽象图形**,避免文字或具象物体;
- 与阶段 1 浅色毛玻璃设计系统视觉一致;
- 可直接在自绘标题栏中用现有矩形 SDF 原语绘制;
- 提供 SVG 源文件与多尺寸 PNG/ICO 导出流程。

## Design Concept

### 名称联想

"丹青"本义为中国画所用的**朱砂(丹)**与**石青(青)**两种颜料,引申为绘画、艺术。框架的价值是"在屏幕上绘制界面",因此 LOGO 概念为:

> **一块半透亮的玻璃画布(石青描边),一滴朱砂颜料落在右下角,一半在画布内、一半破框而出。**

破框构图同时承载两层含义:绘画的动作感(颜料不停留在边界内),与框架"自绘、不受原生控件边界束缚"的技术性格。

### 图形构成

| 元素 | 形状 | 含义 | 绘制方式 |
|---|---|---|---|
| 外框 | 圆角正方形,石青描边 | 窗口 / 画布 / 界面 | 圆角矩形描边或填充 |
| 内点 | 实心圆,朱砂红,骑跨右下角框线 | 颜料滴 / 笔触,破框而出 | 实心圆 |

朱砂滴中心位于外框右下角圆角弧线中点(45° 方向),约一半在画布内、一半在边界外。16×16 下图形收缩为"蓝框 + 红色角点"两个形状,依然可辨。

## Tech Stack

- 矢量源文件:SVG(纯文本,可版本控制)
- 导出脚本:Python 3 + `Pillow`(不依赖系统 Cairo)
- 运行时绘制:`RectBatch::push_rect` 圆角矩形(标题栏内)
- 色彩来源:`LightTheme` token + 品牌专属朱砂红常量

## Commands

```bash
# 1. 导出 PNG 与 ICO(需先安装 Python 依赖)
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
  logo.svg              # 矢量源文件
  logo_16.png           # 由脚本导出
  logo_24.png
  logo_32.png
  logo_48.png
  logo_256.png
  logo.ico
src/widget/title_bar.rs # LOGO 绘制参数与 BRAND_CINNABAR 常量
tools/
  export-logo.py        # SVG → PNG/ICO 导出脚本(几何与 SVG 手工同步)
```

## Color Palette

| 颜色 | 值 | 用途 |
|---|---|---|
| 玉色(深青绿) `accent()` | `#0F766E` (15, 118, 110, 1.0) | 外框描边,来自 theme token |
| 玻璃白 `surface()` | `#FFFFFF` 不透明度 0.85 | SVG 与 TitleBar 中玻璃画布填充,来自 theme token |
| **朱砂红(品牌专属)** | `#E34234` (227, 66, 52, 1.0) | 颜料滴,**仅用于 LOGO**,不进 theme token、不出现在界面组件中 |

朱砂红是品牌资产色,在 `src/widget/title_bar.rs` 中以 `BRAND_CINNABAR` 常量定义,不随主题切换变化。这是"界面颜色一律使用 theme token"规则的唯一例外,仅限 LOGO 图形。

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

导出脚本应生成:`16, 24, 32, 48, 256` PNG 与包含 `16, 24, 32, 48, 256` 的 ICO。

## Code Style

TitleBar 内绘制 LOGO 应继续使用现有 `RectBatch` 原语,不引入图片纹理依赖。`logo_size` 取 `theme.spacing_lg()`,图形比例与 SVG(256 设计空间)一一对应:

```rust
let logo_rect = self.logo_rect(area);
let logo_size = logo_rect.size.width;

// 外框:accent 圆角矩形,与 SVG 外框内缩量一致(42/256 ≈ 16.4%)。
let outer_inset = logo_size * 0.164;
let frame_rect = logo_rect.inset(outer_inset);
let frame_radius = logo_size * 0.18; // 46/256
rects.push_rect(frame_rect, self.logo_frame_color, frame_radius);

// 内部填充:白色半透明,描边宽度 26/256 ≈ 10.2%。
let stroke = logo_size * 0.102;
let fill_rect = frame_rect.inset(stroke);
let fill_radius = (frame_radius - stroke).max(0.0);
rects.push_rect(fill_rect, self.logo_fill_color, fill_radius);

// 朱砂滴:直径 66/256 ≈ 25.8%,中心在 200/256 ≈ 78.1% 处骑跨右下角框线。
let dot_size = logo_size * 0.258;
let dot_offset = logo_size * 0.781;
let dot_rect = Rect::from_xywh(
    logo_rect.origin.x + dot_offset - dot_size / 2.0,
    logo_rect.origin.y + dot_offset - dot_size / 2.0,
    dot_size,
    dot_size,
);
rects.push_rect(dot_rect, self.logo_dot_color, dot_size / 2.0);
```

## Testing Strategy

- `tests/assets.rs`:继续断言 `logo.svg`、各尺寸 PNG 与 `logo.ico` 存在且非空。
- `src/widget/title_bar.rs` 单元测试:保留布局与事件测试;LOGO 颜色断言为外框/填充取 theme token、颜料滴取 `BRAND_CINNABAR`。
- 人工验证:`cargo run --example showcase` 确认标题栏左侧显示新 LOGO。

## Boundaries

- **Always:** 界面组件颜色使用 `LightTheme` token;新增资产必须提交到 `assets/`;位图一律由 `tools/export-logo.py` 从 SVG 几何导出。
- **Ask first:** 引入新的 crate 用于 SVG 渲染;改变 LOGO 概念方向;把朱砂红引入 theme token 或界面组件。
- **Never:** 在 TitleBar 中直接加载 PNG 纹理(阶段 1 不扩展通用 Image Widget);提交没有 SVG 源的位图。

## Success Criteria

- [x] `assets/logo/logo.svg` 存在,且只使用 `#0F766E`、`#E34234` 与白色半透明。
- [x] `tools/export-logo.py` 可从 SVG 几何生成全部 PNG 与 ICO。
- [x] `src/widget/title_bar.rs` LOGO 由"圆角正方形外框 + 骑跨右下角框线的朱砂滴"构成,外框/填充色来自 theme token,朱砂滴为 `BRAND_CINNABAR` 品牌常量。
- [x] `cargo fmt`、`cargo clippy -- -D warnings`、`cargo test --lib --tests` 全绿。
- [ ] `cargo run --example showcase` 标题栏左侧显示新 LOGO(人工确认)。

## Open Questions

1. 阶段 2 剪贴板 POC 的托盘图标是否复用同一套 LOGO?(倾向复用,托盘场景背景复杂时可去掉玻璃白填充、只留框 + 滴。)
