//! @author 十四叔
//! @date 2026/07/19

//! 丹青设计系统 token。
//!
//! 本模块定义 `Theme` trait、`LightTheme` 实现及颜色、字体、间距、圆角、阴影、动效曲线等 token。
//! 所有值为纯逻辑, 不依赖平台或图形 API。

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

/// 主题接口。
///
/// 定义一套面向效率工具的现代毛玻璃浅色设计 token; 后续可扩展 `DarkTheme`。
pub trait Theme: Clone + Copy + std::fmt::Debug {
    /// 窗口 / 页面背景色。
    fn background(&self) -> Color;
    /// 表面浮层色 (卡片、输入框背景)。
    fn surface(&self) -> Color;
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

    /// 小字号 (如提示、标签)。
    fn font_size_small(&self) -> u16;
    /// 正文字号。
    fn font_size_body(&self) -> u16;
    /// 标题字号。
    fn font_size_heading(&self) -> u16;

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
/// 采用毛玻璃风格: 低饱和度背景 + 半透明白色表面 + 蓝色强调。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightTheme;

impl Theme for LightTheme {
    fn background(&self) -> Color {
        // 与背景渐变 top 色一致的 fallback 清屏色。
        Color::from_srgb8(247, 249, 254)
    }

    fn surface(&self) -> Color {
        // 接近纯白但保留极淡透明,让阴影/背景能透出一丝氛围。
        Color::rgba(1.0, 1.0, 1.0, 0.95)
    }

    fn surface_variant(&self) -> Color {
        // 用于悬停、次级卡片等需要与主 surface 区分的场景。
        Color::from_srgb8(241, 245, 249)
    }

    fn accent(&self) -> Color {
        Color::from_srgb8(59, 130, 246)
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
        Color::rgba(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0, 0.30)
    }

    fn caret(&self) -> Color {
        Color::from_srgb8(59, 130, 246)
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

    fn font_size_small(&self) -> u16 {
        12
    }

    fn font_size_body(&self) -> u16 {
        15
    }

    fn font_size_heading(&self) -> u16 {
        20
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
    }

    #[test]
    fn light_theme_font_sizes_are_ordered() {
        let theme = LightTheme;
        assert!(theme.font_size_small() < theme.font_size_body());
        assert!(theme.font_size_body() < theme.font_size_heading());
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
}
