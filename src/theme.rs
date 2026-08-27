//! @author 十四叔
//! @date 2026/07/19

//! 丹青设计系统 token。
//!
//! 本模块定义 `Theme` trait、`LightTheme` 实现及颜色、字体、间距、圆角、阴影、动效曲线等 token。
//! 所有值为纯逻辑，不依赖平台或图形 API。

use crate::{Color, Point};

/// 阴影描述。
///
/// 目前由偏移、模糊半径与颜色组成; 后续渲染管线可据此生成阴影实例。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    /// 阴影相对于组件的偏移。
    pub offset: Point,
    /// 模糊半径 (逻辑像素)。
    pub blur_radius: f32,
    /// 阴影颜色 (通常含透明度)。
    pub color: Color,
}

/// 动效曲线。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    /// 线性。
    Linear,
    /// 缓入缓出。
    EaseInOut,
}

impl Easing {
    /// 对进度 `t` 求值 (输入输出均夹到 0..1)。
    ///
    /// `EaseInOut` 采用三次缓入缓出：两端平缓、中段陡峭。
    pub fn eval(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
        }
    }
}

/// 计算颜色的相对亮度 (WCAG 定义，0.0 黑 ~ 1.0 白)。
///
/// 输入视为 sRGB 编码 (与 [`Color::from_srgb8`] 的存储语义一致),
/// 先逐通道解码为线性，再按 Rec.709 权重加权。
pub fn relative_luminance(color: Color) -> f32 {
    fn decode(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * decode(color.r) + 0.7152 * decode(color.g) + 0.0722 * decode(color.b)
}

/// 计算两颜色的 WCAG 对比度 (1.0 ~ 21.0)。
///
/// 忽略 alpha; 半透明色请先经 [`composite_over`] 合成到底色再比较。
pub fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// 将半透明顶层色合成到不透明底色上 (标准 over 运算)。
pub fn composite_over(top: Color, base: Color) -> Color {
    let a = top.a.clamp(0.0, 1.0);
    Color::rgb(
        top.r * a + base.r * (1.0 - a),
        top.g * a + base.g * (1.0 - a),
        top.b * a + base.b * (1.0 - a),
    )
}

/// 主题接口。
///
/// 定义一套面向效率工具的现代毛玻璃浅色设计 token; 后续可扩展 `DarkTheme`。
pub trait Theme: Clone + Copy + std::fmt::Debug {
    /// 窗口 / 页面背景色。
    fn background(&self) -> Color;
    /// 表面浮层色 (卡片、输入框背景)。
    fn surface(&self) -> Color;
    /// 输入区表面色 (TextInput/TextArea 背景)。
    ///
    /// 比 `surface` 更实：输入区以可读性优先，
    /// 卡片可以透出背景营造玻璃感，文字输入处不行。
    fn surface_input(&self) -> Color;
    /// 次级表面色 (悬停、禁用背景)。
    fn surface_variant(&self) -> Color;
    /// 主强调色 (按钮、光标、选区)。
    fn accent(&self) -> Color;
    /// 主要文字色。
    fn text_primary(&self) -> Color;
    /// 次级文字色 (提示、占位)。
    fn text_secondary(&self) -> Color;
    /// 分割线 / 边框色。
    fn divider(&self) -> Color;
    /// 组件边框色。
    fn border(&self) -> Color;
    /// 文本选区背景色。
    fn selection(&self) -> Color;
    /// 光标色。
    fn caret(&self) -> Color;
    /// 危险 / 关闭按钮色。
    fn danger(&self) -> Color;
    /// macOS 红绿灯关闭按钮色。
    fn traffic_close(&self) -> Color;
    /// macOS 红绿灯最小化按钮色。
    fn traffic_minimize(&self) -> Color;
    /// macOS 红绿灯最大化按钮色。
    fn traffic_maximize(&self) -> Color;
    /// 面板遮罩色 (浮层半透明罩, 压暗背景以突出浮层)。
    fn scrim(&self) -> Color;

    /// 小字号 (如提示、标签)。
    fn font_size_small(&self) -> u16;
    /// 正文字号。
    fn font_size_body(&self) -> u16;
    /// 标题字号。
    fn font_size_heading(&self) -> u16;
    /// 展示级字号 (如番茄钟大字倒计时)。
    fn font_size_display(&self) -> u16 {
        120
    }

    /// 标准控件高度 (按钮、输入框等单行表单控件)。
    ///
    /// 保证同类控件并排时默认对齐; 产品层可按需覆盖。
    fn control_height(&self) -> f32 {
        32.0
    }

    /// 超小间距。
    fn spacing_xs(&self) -> f32;
    /// 小间距。
    fn spacing_sm(&self) -> f32;
    /// 中间距。
    fn spacing_md(&self) -> f32;
    /// 大间距。
    fn spacing_lg(&self) -> f32;
    /// 超大间距。
    fn spacing_xl(&self) -> f32;

    /// 小圆角 (如输入框)。
    fn radius_sm(&self) -> f32;
    /// 中圆角 (如按钮)。
    fn radius_md(&self) -> f32;
    /// 大圆角 (如卡片)。
    fn radius_lg(&self) -> f32;
    /// 超大圆角 (如全圆胶囊控件条)。
    fn radius_xl(&self) -> f32;

    /// 小阴影 (如输入框)。
    fn shadow_sm(&self) -> Shadow;
    /// 中阴影 (如卡片、浮层)。
    fn shadow_md(&self) -> Shadow;
    /// 大阴影 (如模态、悬浮面板)。
    fn shadow_lg(&self) -> Shadow;

    /// 标准动效曲线。
    fn easing_standard(&self) -> Easing;
    /// 加速动效曲线。
    fn easing_accelerate(&self) -> Easing;
}

/// 浅色主题。
///
/// 采用毛玻璃风格：低饱和度背景 + 半透明白色表面 + 青绿 (玉色) 强调。
///
/// accent 取丹青矿物色中的深青绿/玉色 (#0F766E), 朱砂仅作品牌点睛 (logo), 不进 token。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightTheme;

impl Theme for LightTheme {
    fn background(&self) -> Color {
        // 与背景渐变 top 色一致的 fallback 清屏色 (微带青头，与玉色 accent 同温)。
        Color::from_srgb8(240, 248, 246)
    }

    fn surface(&self) -> Color {
        // 半透明白：卡片浮在渐变背景上透出玻璃感。
        Color::rgba(1.0, 1.0, 1.0, 0.72)
    }

    fn surface_input(&self) -> Color {
        // 接近纯白：输入区文字可读性优先，只允许一丝氛围透出。
        Color::rgba(1.0, 1.0, 1.0, 0.95)
    }

    fn surface_variant(&self) -> Color {
        // 用于悬停、次级卡片等需要与主 surface 区分的场景 (冷调微青)。
        Color::from_srgb8(238, 246, 242)
    }

    fn accent(&self) -> Color {
        // 深青绿/玉色 #0F766E: 丹青矿物色，白底对比度 ~5:1。
        Color::from_srgb8(15, 118, 110)
    }

    fn text_primary(&self) -> Color {
        Color::from_srgb8(15, 23, 42)
    }

    fn text_secondary(&self) -> Color {
        Color::from_srgb8(71, 85, 105)
    }

    fn divider(&self) -> Color {
        Color::rgba(0.0, 0.0, 0.0, 0.10)
    }

    fn border(&self) -> Color {
        Color::rgba(0.0, 0.0, 0.0, 0.18)
    }

    fn selection(&self) -> Color {
        // 跟随 accent 的 30% 透明选区。
        Color::rgba(15.0 / 255.0, 118.0 / 255.0, 110.0 / 255.0, 0.30)
    }

    fn caret(&self) -> Color {
        // 跟随 accent。
        Color::from_srgb8(15, 118, 110)
    }

    fn danger(&self) -> Color {
        Color::from_srgb8(239, 68, 68)
    }

    fn traffic_close(&self) -> Color {
        // macOS 红绿灯标准红 #FF5F57。
        Color::from_srgb8(255, 95, 87)
    }

    fn traffic_minimize(&self) -> Color {
        // macOS 红绿灯标准黄 #FEBC2E。
        Color::from_srgb8(254, 188, 46)
    }

    fn traffic_maximize(&self) -> Color {
        // macOS 红绿灯标准绿 #28C840。
        Color::from_srgb8(40, 200, 64)
    }

    fn scrim(&self) -> Color {
        // 面板浮层遮罩: 固定深色半透明, 不随明暗主题漂移 (压暗任何背景都成立)。
        Color::rgba(0.0, 0.0, 0.0, 0.35)
    }

    fn font_size_small(&self) -> u16 {
        12
    }

    fn font_size_body(&self) -> u16 {
        16
    }

    fn font_size_heading(&self) -> u16 {
        20
    }

    fn control_height(&self) -> f32 {
        32.0
    }

    fn spacing_xs(&self) -> f32 {
        4.0
    }

    fn spacing_sm(&self) -> f32 {
        8.0
    }

    fn spacing_md(&self) -> f32 {
        12.0
    }

    fn spacing_lg(&self) -> f32 {
        16.0
    }

    fn spacing_xl(&self) -> f32 {
        24.0
    }

    fn radius_sm(&self) -> f32 {
        6.0
    }

    fn radius_md(&self) -> f32 {
        10.0
    }

    fn radius_lg(&self) -> f32 {
        16.0
    }

    fn radius_xl(&self) -> f32 {
        // 全圆胶囊 (如番茄钟底部玻璃控件条)。
        28.0
    }

    fn shadow_sm(&self) -> Shadow {
        Shadow {
            offset: Point::new(0.0, 1.0),
            blur_radius: 4.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.08),
        }
    }

    fn shadow_md(&self) -> Shadow {
        Shadow {
            offset: Point::new(0.0, 4.0),
            blur_radius: 16.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.14),
        }
    }

    fn shadow_lg(&self) -> Shadow {
        Shadow {
            offset: Point::new(0.0, 8.0),
            blur_radius: 28.0,
            color: Color::rgba(0.0, 0.0, 0.0, 0.18),
        }
    }

    fn easing_standard(&self) -> Easing {
        Easing::EaseInOut
    }

    fn easing_accelerate(&self) -> Easing {
        Easing::Linear
    }
}

/// 场景调色板。
///
/// 由场景生成管线随场景大图一并产出 (见 `tools/export-scenes.py`);
/// 明暗随场景流动：暗场景 (篝火) 与亮场景 (海) 各给一套，
/// 玻璃表面、文字、控件态须在两套下都成立。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScenePalette {
    /// 场景基调色 (清屏 / fallback, 通常取场景主色)。
    pub base: Color,
    /// 主强调色 (按钮、光标、选区)。
    pub accent: Color,
    /// 主要文字色 (倒计时、控件标签)。
    pub text_primary: Color,
    /// 次级文字色 (阶段 / 场景名标注)。
    pub text_secondary: Color,
    /// 玻璃表面色 (半透明，控件条 / 卡片)。
    pub surface: Color,
    /// 输入区表面色 (比 surface 更实)。
    pub surface_input: Color,
    /// 场景最亮区域色 (文字可读性护栏用)。
    pub backdrop_light: Color,
    /// 场景最暗区域色 (文字可读性护栏用)。
    pub backdrop_dark: Color,
}

impl ScenePalette {
    /// 逐字段向另一调色板插值 (场景过渡动画用，`t` 夹到 0..1)。
    pub fn lerp(self, other: ScenePalette, t: f32) -> ScenePalette {
        ScenePalette {
            base: self.base.lerp(other.base, t),
            accent: self.accent.lerp(other.accent, t),
            text_primary: self.text_primary.lerp(other.text_primary, t),
            text_secondary: self.text_secondary.lerp(other.text_secondary, t),
            surface: self.surface.lerp(other.surface, t),
            surface_input: self.surface_input.lerp(other.surface_input, t),
            backdrop_light: self.backdrop_light.lerp(other.backdrop_light, t),
            backdrop_dark: self.backdrop_dark.lerp(other.backdrop_dark, t),
        }
    }

    /// 降低全调色板饱和度 (暂停态视觉反馈用)。`factor=0` 保留, `factor=1` 全灰。
    /// 透明色 (surface / surface_input) 也去饱和 RGB, alpha 保持。
    pub fn desaturate(self, factor: f32) -> ScenePalette {
        ScenePalette {
            base: self.base.desaturate(factor),
            accent: self.accent.desaturate(factor),
            text_primary: self.text_primary.desaturate(factor),
            text_secondary: self.text_secondary.desaturate(factor),
            surface: self.surface.desaturate(factor),
            surface_input: self.surface_input.desaturate(factor),
            backdrop_light: self.backdrop_light.desaturate(factor),
            backdrop_dark: self.backdrop_dark.desaturate(factor),
        }
    }
}

/// 场景规格：生成管线产出的单个场景资产描述。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneSpec {
    /// 场景名 (如 "篝火")。
    pub name: &'static str,
    /// 场景大图路径 (相对仓库根)。
    pub image: &'static str,
    /// 场景调色板。
    pub palette: ScenePalette,
}

/// 场景主题：由 [`ScenePalette`] 构造的跨明暗 [`Theme`] 实现。
///
/// 颜色 token 取自调色板; 选区 / 光标派生自 accent,
/// 分割线 / 边框派生自文字色 (暗场景下自动变亮);
/// 字号 / 间距 / 圆角 / 阴影 / 动效沿用 [`LightTheme`] 档位。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneTheme {
    /// 调色板本体。
    palette: ScenePalette,
}

impl SceneTheme {
    /// 用给定调色板创建场景主题。
    pub fn new(palette: ScenePalette) -> Self {
        Self { palette }
    }

    /// 读取调色板 (过渡插值后可重建主题)。
    pub fn palette(&self) -> ScenePalette {
        self.palette
    }
}

impl Theme for SceneTheme {
    fn background(&self) -> Color {
        self.palette.base
    }

    fn surface(&self) -> Color {
        self.palette.surface
    }

    fn surface_input(&self) -> Color {
        self.palette.surface_input
    }

    fn surface_variant(&self) -> Color {
        // 悬停等次级表面：玻璃合成到场景基调上的不透明色。
        composite_over(self.palette.surface, self.palette.base)
    }

    fn accent(&self) -> Color {
        self.palette.accent
    }

    fn text_primary(&self) -> Color {
        self.palette.text_primary
    }

    fn text_secondary(&self) -> Color {
        self.palette.text_secondary
    }

    fn divider(&self) -> Color {
        // 跟随文字色：暗场景分割线自动变亮。
        let t = self.palette.text_primary;
        Color::rgba(t.r, t.g, t.b, 0.15)
    }

    fn border(&self) -> Color {
        let t = self.palette.text_primary;
        Color::rgba(t.r, t.g, t.b, 0.28)
    }

    fn selection(&self) -> Color {
        let a = self.palette.accent;
        Color::rgba(a.r, a.g, a.b, 0.30)
    }

    fn caret(&self) -> Color {
        self.palette.accent
    }

    fn danger(&self) -> Color {
        LightTheme.danger()
    }

    fn traffic_close(&self) -> Color {
        LightTheme.traffic_close()
    }

    fn traffic_minimize(&self) -> Color {
        LightTheme.traffic_minimize()
    }

    fn traffic_maximize(&self) -> Color {
        LightTheme.traffic_maximize()
    }

    fn scrim(&self) -> Color {
        LightTheme.scrim()
    }

    fn font_size_small(&self) -> u16 {
        LightTheme.font_size_small()
    }

    fn font_size_body(&self) -> u16 {
        LightTheme.font_size_body()
    }

    fn font_size_heading(&self) -> u16 {
        LightTheme.font_size_heading()
    }

    fn font_size_display(&self) -> u16 {
        LightTheme.font_size_display()
    }

    fn spacing_xs(&self) -> f32 {
        LightTheme.spacing_xs()
    }

    fn spacing_sm(&self) -> f32 {
        LightTheme.spacing_sm()
    }

    fn spacing_md(&self) -> f32 {
        LightTheme.spacing_md()
    }

    fn spacing_lg(&self) -> f32 {
        LightTheme.spacing_lg()
    }

    fn spacing_xl(&self) -> f32 {
        LightTheme.spacing_xl()
    }

    fn radius_sm(&self) -> f32 {
        LightTheme.radius_sm()
    }

    fn radius_md(&self) -> f32 {
        LightTheme.radius_md()
    }

    fn radius_lg(&self) -> f32 {
        LightTheme.radius_lg()
    }

    fn radius_xl(&self) -> f32 {
        LightTheme.radius_xl()
    }

    fn shadow_sm(&self) -> Shadow {
        LightTheme.shadow_sm()
    }

    fn shadow_md(&self) -> Shadow {
        LightTheme.shadow_md()
    }

    fn shadow_lg(&self) -> Shadow {
        LightTheme.shadow_lg()
    }

    fn easing_standard(&self) -> Easing {
        LightTheme.easing_standard()
    }

    fn easing_accelerate(&self) -> Easing {
        LightTheme.easing_accelerate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_theme_traffic_colors_match_macos_palette() {
        let theme = LightTheme;
        assert_eq!(theme.traffic_close(), Color::from_srgb8(255, 95, 87));
        assert_eq!(theme.traffic_minimize(), Color::from_srgb8(254, 188, 46));
        assert_eq!(theme.traffic_maximize(), Color::from_srgb8(40, 200, 64));
    }

    #[test]
    fn light_theme_implements_theme() {
        fn assert_theme<T: Theme>() {}
        assert_theme::<LightTheme>();
    }

    #[test]
    fn light_theme_colors_are_visible() {
        let theme = LightTheme;
        assert!(theme.background().a > 0.0);
        assert!(theme.surface().a > 0.0);
        assert!(theme.accent().a > 0.0);
        assert!(theme.text_primary().a > 0.0);
        assert!(theme.text_secondary().a > 0.0);
        assert!(theme.divider().a > 0.0);
        assert!(theme.border().a > 0.0);
        assert!(theme.selection().a > 0.0);
        assert!(theme.caret().a > 0.0);
        assert!(theme.danger().a > 0.0);
        assert!(theme.scrim().a > 0.0);
    }

    #[test]
    fn light_theme_surface_is_translucent_glass() {
        // 玻璃感护栏：surface 必须半透明，让背景渐变透出; 又不能透明到丢失层次。
        let a = LightTheme.surface().a;
        assert!(
            (0.6..=0.8).contains(&a),
            "surface alpha 应在 0.6~0.8 玻璃区间，实际 {a}"
        );
    }

    #[test]
    fn light_theme_surface_input_is_more_solid_than_surface() {
        // 输入区可读性优先：输入框背景要比卡片更实。
        let theme = LightTheme;
        assert!(theme.surface_input().a >= 0.9);
        assert!(theme.surface_input().a > theme.surface().a);
    }

    #[test]
    fn light_theme_font_sizes_are_ordered() {
        let theme = LightTheme;
        assert!(theme.font_size_small() < theme.font_size_body());
        assert!(theme.font_size_body() < theme.font_size_heading());
    }

    #[test]
    fn light_theme_control_height_is_32() {
        assert_eq!(LightTheme.control_height(), 32.0);
    }

    #[test]
    fn scene_theme_control_height_matches_light_theme() {
        let theme = SceneTheme::new(sample_dark_palette());
        assert_eq!(theme.control_height(), LightTheme.control_height());
    }

    #[test]
    fn light_theme_spacings_are_ordered_and_non_negative() {
        let theme = LightTheme;
        assert!(theme.spacing_xs() >= 0.0);
        assert!(theme.spacing_xs() < theme.spacing_sm());
        assert!(theme.spacing_sm() < theme.spacing_md());
        assert!(theme.spacing_md() < theme.spacing_lg());
        assert!(theme.spacing_lg() < theme.spacing_xl());
    }

    #[test]
    fn light_theme_radii_are_ordered_and_non_negative() {
        let theme = LightTheme;
        assert!(theme.radius_sm() >= 0.0);
        assert!(theme.radius_sm() < theme.radius_md());
        assert!(theme.radius_md() < theme.radius_lg());
        assert!(theme.radius_lg() < theme.radius_xl());
    }

    #[test]
    fn light_theme_scrim_is_dark_overlay() {
        // 浮层遮罩: 近黑且半透明 — 压暗背景突出浮层, 又不全黑 (玻璃感护栏)。
        let s = LightTheme.scrim();
        assert!(s.r < 0.2 && s.g < 0.2 && s.b < 0.2, "遮罩应近黑: {s:?}");
        assert!(s.a > 0.0 && s.a < 1.0, "遮罩应半透明: alpha={}", s.a);
    }

    #[test]
    fn light_theme_shadows_have_color() {
        let theme = LightTheme;
        assert!(theme.shadow_sm().color.a > 0.0);
        assert!(theme.shadow_md().color.a > 0.0);
        assert!(theme.shadow_lg().color.a > 0.0);
        assert!(theme.shadow_sm().blur_radius >= 0.0);
        assert!(theme.shadow_md().blur_radius >= 0.0);
        assert!(theme.shadow_lg().blur_radius >= 0.0);
    }

    #[test]
    fn light_theme_easings_are_valid() {
        let theme = LightTheme;
        assert!(matches!(theme.easing_standard(), Easing::EaseInOut));
        assert!(matches!(theme.easing_accelerate(), Easing::Linear));
    }

    #[test]
    fn relative_luminance_black_is_zero_white_is_one() {
        assert!(relative_luminance(Color::BLACK).abs() < 0.01);
        assert!((relative_luminance(Color::WHITE) - 1.0).abs() < 0.01);
    }

    #[test]
    fn relative_luminance_decodes_srgb() {
        // sRGB 中灰 0.5 解码为线性后约为 0.214, 而非 0.5。
        let gray = Color::rgb(0.5, 0.5, 0.5);
        let l = relative_luminance(gray);
        assert!((l - 0.214).abs() < 0.01, "中灰线性亮度应约 0.214, 实际 {l}");
    }

    #[test]
    fn contrast_ratio_black_white_is_21() {
        let ratio = contrast_ratio(Color::BLACK, Color::WHITE);
        assert!(
            (ratio - 21.0).abs() < 0.1,
            "黑白对比度应约 21:1, 实际 {ratio}"
        );
    }

    #[test]
    fn contrast_ratio_is_symmetric_and_same_color_is_1() {
        let a = Color::from_srgb8(15, 118, 110);
        let b = Color::from_srgb8(240, 248, 246);
        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < f32::EPSILON);
        assert!((contrast_ratio(a, a) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn composite_over_opaque_top_returns_top() {
        let top = Color::rgba(0.2, 0.4, 0.6, 1.0);
        let base = Color::BLACK;
        assert_eq!(composite_over(top, base), top);
    }

    #[test]
    fn composite_over_half_white_on_black_is_mid_gray() {
        let top = Color::rgba(1.0, 1.0, 1.0, 0.5);
        let out = composite_over(top, Color::BLACK);
        assert!((out.r - 0.5).abs() < f32::EPSILON);
        assert!((out.g - 0.5).abs() < f32::EPSILON);
        assert!((out.b - 0.5).abs() < f32::EPSILON);
        assert!((out.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easing_eval_endpoints_are_identity() {
        for e in [Easing::Linear, Easing::EaseInOut] {
            assert!((e.eval(0.0) - 0.0).abs() < f32::EPSILON);
            assert!((e.eval(1.0) - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn easing_eval_is_monotonic() {
        for e in [Easing::Linear, Easing::EaseInOut] {
            let mut prev = e.eval(0.0);
            for i in 1..=10 {
                let cur = e.eval(i as f32 / 10.0);
                assert!(cur >= prev, "{e:?} 在 {i}/10 处不单调");
                prev = cur;
            }
        }
    }

    #[test]
    fn easing_eval_clamps_t() {
        for e in [Easing::Linear, Easing::EaseInOut] {
            assert!((e.eval(-0.5) - 0.0).abs() < f32::EPSILON);
            assert!((e.eval(1.5) - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn easing_ease_in_out_midpoint_is_half() {
        assert!((Easing::EaseInOut.eval(0.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn display_font_size_is_largest_tier() {
        let theme = LightTheme;
        assert!(theme.font_size_display() > theme.font_size_heading());
    }

    /// 合成暗场景调色板 (参照篝火：深底、近白文字、暗玻璃)。
    fn sample_dark_palette() -> ScenePalette {
        ScenePalette {
            base: Color::from_srgb8(26, 16, 12),
            accent: Color::from_srgb8(255, 159, 67),
            text_primary: Color::from_srgb8(250, 244, 235),
            text_secondary: Color::from_srgb8(190, 175, 160),
            surface: Color::rgba(1.0, 1.0, 1.0, 0.14),
            surface_input: Color::rgba(1.0, 1.0, 1.0, 0.22),
            backdrop_light: Color::from_srgb8(120, 70, 40),
            backdrop_dark: Color::from_srgb8(16, 10, 8),
        }
    }

    /// 合成亮场景调色板 (参照海：亮底、深色文字、白玻璃)。
    fn sample_bright_palette() -> ScenePalette {
        ScenePalette {
            base: Color::from_srgb8(210, 235, 240),
            accent: Color::from_srgb8(12, 74, 110),
            text_primary: Color::from_srgb8(8, 32, 48),
            text_secondary: Color::from_srgb8(60, 90, 105),
            surface: Color::rgba(1.0, 1.0, 1.0, 0.55),
            surface_input: Color::rgba(1.0, 1.0, 1.0, 0.85),
            backdrop_light: Color::from_srgb8(235, 248, 250),
            backdrop_dark: Color::from_srgb8(140, 190, 205),
        }
    }

    #[test]
    fn scene_theme_implements_theme() {
        fn assert_theme<T: Theme>() {}
        assert_theme::<SceneTheme>();
    }

    #[test]
    fn scene_theme_maps_palette_colors_directly() {
        let palette = sample_dark_palette();
        let theme = SceneTheme::new(palette);
        assert_eq!(theme.background(), palette.base);
        assert_eq!(theme.surface(), palette.surface);
        assert_eq!(theme.surface_input(), palette.surface_input);
        assert_eq!(theme.accent(), palette.accent);
        assert_eq!(theme.text_primary(), palette.text_primary);
        assert_eq!(theme.text_secondary(), palette.text_secondary);
    }

    #[test]
    fn scene_theme_derives_selection_and_caret_from_accent() {
        let palette = sample_dark_palette();
        let theme = SceneTheme::new(palette);
        let selection = theme.selection();
        assert!((selection.r - palette.accent.r).abs() < f32::EPSILON);
        assert!((selection.g - palette.accent.g).abs() < f32::EPSILON);
        assert!((selection.b - palette.accent.b).abs() < f32::EPSILON);
        assert!((selection.a - 0.30).abs() < 0.01);
        assert_eq!(theme.caret(), palette.accent);
    }

    #[test]
    fn scene_theme_derives_divider_and_border_from_text_color() {
        let palette = sample_dark_palette();
        let theme = SceneTheme::new(palette);
        let divider = theme.divider();
        let border = theme.border();
        // 暗场景下分割线应跟随文字色 (亮), 而非固定黑色。
        assert!((divider.r - palette.text_primary.r).abs() < f32::EPSILON);
        assert!(divider.a > 0.0 && divider.a < border.a);
        assert!(border.a <= 0.5);
    }

    #[test]
    fn scene_theme_surface_variant_is_opaque_composite() {
        let palette = sample_bright_palette();
        let theme = SceneTheme::new(palette);
        let variant = theme.surface_variant();
        assert!((variant.a - 1.0).abs() < f32::EPSILON);
        assert_eq!(variant, composite_over(palette.surface, palette.base));
    }

    #[test]
    fn scene_theme_non_color_tokens_match_light_theme() {
        let theme = SceneTheme::new(sample_dark_palette());
        let light = LightTheme;
        assert_eq!(theme.font_size_small(), light.font_size_small());
        assert_eq!(theme.font_size_body(), light.font_size_body());
        assert_eq!(theme.font_size_heading(), light.font_size_heading());
        assert_eq!(theme.font_size_display(), light.font_size_display());
        assert_eq!(theme.spacing_md(), light.spacing_md());
        assert_eq!(theme.radius_lg(), light.radius_lg());
        assert_eq!(theme.radius_xl(), light.radius_xl());
        assert_eq!(theme.scrim(), light.scrim());
        assert_eq!(theme.easing_standard(), light.easing_standard());
    }

    #[test]
    fn scene_palette_lerp_endpoints_and_midpoint() {
        let a = sample_dark_palette();
        let b = sample_bright_palette();
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        let mid = a.lerp(b, 0.5);
        assert!((mid.base.r - (a.base.r + b.base.r) * 0.5).abs() < f32::EPSILON);
        assert!((mid.accent.b - (a.accent.b + b.accent.b) * 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn scene_guard_text_reads_on_both_backdrop_extremes() {
        // 护栏方法学验证：明暗两族合成调色板，大字文字 vs 场景两极端 ≥ 3:1。
        for palette in [sample_dark_palette(), sample_bright_palette()] {
            for backdrop in [palette.backdrop_light, palette.backdrop_dark] {
                let ratio = contrast_ratio(palette.text_primary, backdrop);
                assert!(
                    ratio >= 3.0,
                    "大字文字 vs 场景极端色对比度应 ≥3:1, 实际 {ratio:.2}"
                );
            }
        }
    }

    #[test]
    fn scene_guard_control_text_reads_on_glass_surface() {
        // 控件文字 vs 玻璃合成色 ≥ 4:1 (表面分别合成到场景两极端上取不利值)。
        for palette in [sample_dark_palette(), sample_bright_palette()] {
            for backdrop in [palette.backdrop_light, palette.backdrop_dark] {
                let glass = composite_over(palette.surface, backdrop);
                let ratio = contrast_ratio(palette.text_primary, glass);
                assert!(
                    ratio >= 4.0,
                    "控件文字 vs 玻璃表面对比度应 ≥4:1, 实际 {ratio:.2}"
                );
            }
        }
    }
}
