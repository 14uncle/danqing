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
    /// 三次缓入 (起手慢、收尾快): 适合淡出/离场。
    EaseIn,
    /// 三次缓出 (起手快、收尾慢): 适合淡入/进场。
    EaseOut,
}

impl Easing {
    /// 对进度 `t` 求值 (输入输出均夹到 0..1)。
    ///
    /// `EaseInOut` 采用三次缓入缓出：两端平缓、中段陡峭。
    /// `EaseIn` = t³; `EaseOut` = 1-(1-t)³。
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
            Self::EaseIn => t.powi(3),
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
        }
    }
}

/// 计算颜色的相对亮度 (WCAG 定义，0.0 黑 ~ 1.0 白)。
///
/// 输入视为 sRGB 编码 (与 [`Color::from_srgb8`] 的存储语义一致),
/// 先逐通道解码为线性，再按 Rec.709 权重加权。
///
/// 解码复用 [`crate::layout::srgb_to_linear`]，**不要在这里另写一份** ——
/// 它同时是 GPU 边界的转换实现；两处各写一份且说法矛盾，正是双重 gamma 事故的成因。
pub fn relative_luminance(color: Color) -> f32 {
    use crate::layout::srgb_to_linear;
    0.2126 * srgb_to_linear(color.r)
        + 0.7152 * srgb_to_linear(color.g)
        + 0.0722 * srgb_to_linear(color.b)
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
        36.0
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
        //
        // 原值 `#EEF6F2` 与页面底色 `#F0F8F6` **只差 2/255** (Δ`L*` −0.74) ——
        // 「次级表面」与底色同色, 等于不存在。2026-09-13 查实它在产品里**有 5 处
        // 直接压在页面底上** (表头底 / 过滤栏底 / 侧栏生效行 / 关闭按钮 hover /
        // 链接行 hover), 全部看不见; 唯独 `Dropdown` 弹层 hover 那处是好的
        // —— 因为那一处压在**近白弹层**上, 同一个值在那里是 Δ`L*` 3.8。
        // 也就是说: 原值不是「深了浅了」, 是**只对白底成立、对页面底不成立**。
        //
        // 新值按**暗色那一支的台阶**取: 暗色 `surface_variant` 对暗底是
        // Δ`L*` +6.42, 浅色这支取到 **Δ`L*` −5.92** —— 两个主题第一次对齐。
        // 对近白弹层它变成 Δ`L*` 8.97, 与用户在模块 5 已接受的行 hover (9.12)
        // 同一量级, 不嫌重。
        //
        // 台阶取 **−7.0** 而非原来的 −5.92: 表头底下紧挨着的常是**斑马行**
        // (产品的 `row_band_bg`, 对底 −3.49), 两者只差 2.43 —— 表格里表头与
        // 斑马行会糊在一起。挪到 −7.0 后差 3.51。
        // 这是 2026-09-13 真机截图审查查出的第二处「面撞车」, 与暗色表头↔斑马
        // (Δ`L*` 0.08) 同源: **各自对页面底取值, 却谁也没管邻居。**
        //
        // 回归锁: `surface_variant_is_a_comparable_step_in_both_themes`。
        Color::from_srgb8(0xDE, 0xE5, 0xE1)
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
        15
    }

    fn font_size_heading(&self) -> u16 {
        20
    }

    fn control_height(&self) -> f32 {
        36.0
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

/// 深色主题。
///
/// 深灰偏蓝背景 + 暗玻璃表面 + 玉色 accent 不变。
/// 分割线/边框跟随文字色自动变亮。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DarkTheme;

impl Theme for DarkTheme {
    fn background(&self) -> Color {
        // 深灰偏蓝，比纯黑柔和。
        Color::from_srgb8(25, 25, 32)
    }

    fn surface(&self) -> Color {
        // 暗玻璃：白色低透明度。
        // alpha 的推导见 `surface_variant` —— 渲染成 (50,50,54), 与底色差 +12 台阶。
        Color::rgba(1.0, 1.0, 1.0, 0.022)
    }

    fn surface_input(&self) -> Color {
        // 输入区略实，保证文字可读。
        // 比 `surface` 高一档 (渲染 (57,57,60), +15 台阶): 输入框要能读作「这里能打字」。
        Color::rgba(1.0, 1.0, 1.0, 0.031)
    }

    fn surface_variant(&self) -> Color {
        // 悬停/次级表面。
        //
        // alpha 是 0.0154 而非 0.10: 渲染目标按 **linear** 空间混合, 而近黑底
        // (#191920) 在 linear 空间**极度敏感** —— 白 10% 合成出 (93,93,94),
        // 是一块灰板（与底色对比度 2.65）; 白 1.5% 才是淡台阶（1.25）。
        // 浅色主题不受这个敏感度影响（同一个 10% 在近白底上只差 2/255),
        // 故两个主题的 alpha 不能同值 —— 判据是**知觉台阶**而非 alpha 本身。
        //
        // **2026-09-13 由 0.010 (Δ`L*` +6.30) 提到这里 (+9.00)。** 触发是真机
        // 截图审查: 表头（本支）与斑马行（产品的 `row_band_bg`, Δ`L*` +6.38）
        // **只差 0.08, 完全同色**。而两者各自的台阶都没问题 —— 错在**各自对页面底
        // 取值, 谁也没管邻居**。挪到 +9.00 后与斑马差 5.56。
        // 为什么是表头让路而不是斑马: 底色 L* 只有 9.04, 「底↔斑马↔表头」两点
        // 各要 3.0 就得 6.0, 而原区间总共只有 6.30 —— 窄到装不下, 必须有人往外让。
        // 回归锁: `dark_surface_variant_is_a_subtle_step_not_a_slab` (上界仍在)。
        Color::rgba(1.0, 1.0, 1.0, 0.0154)
    }

    fn accent(&self) -> Color {
        // 亮玉色，深色背景下对比度 ~5.3:1 (WCAG AA)。
        Color::from_srgb8(26, 158, 138)
    }

    fn text_primary(&self) -> Color {
        // 近白。
        Color::from_srgb8(229, 229, 234)
    }

    fn text_secondary(&self) -> Color {
        // 中灰。
        Color::from_srgb8(142, 142, 147)
    }

    fn divider(&self) -> Color {
        // 跟随文字色变亮。
        // 1px 细线要够亮才看得见 —— 台阶比填充高一档 (渲染 (68,68,72), +20); 但原来的
        // 0.15 在 linear 混合下会渲染成 (99,99,103) (+33), 比正文还抢眼, 已收。
        Color::rgba(229.0 / 255.0, 229.0 / 255.0, 234.0 / 255.0, 0.061)
    }

    fn border(&self) -> Color {
        // 轮廓线, 台阶最高的一档 (渲染 (87,87,90), +28)。原 0.28 渲染成 (131,131,135)
        // —— 比中灰还亮 (+46), 在暗色上是刺目的白框, 已收。
        Color::rgba(229.0 / 255.0, 229.0 / 255.0, 234.0 / 255.0, 0.109)
    }

    fn selection(&self) -> Color {
        // 跟随 accent 的 20% 透明选区 (原 30%)。
        //
        // 收窄的理由是**选区会吃掉前景色的对比度** (2026-09-13, 用户实机报
        // 「ERROR 选中行红色字体看得眼花」)。选区带是 accent 色相、又压在中间的
        // 亮度上, 于是它跟任何**中等亮度**的前景色都拉不开: 实测 30% 时合成
        // (25,93,83), 框架自己的 `danger()` 红压在上面只有 **2.03:1**。
        // 收到 20% 后合成 (25,78,71), 同一支红升到 **2.51:1**, 而选区带自身对底色的
        // 可见度仍有 1.85 (30% 时 2.27) —— 还看得出选中。
        //
        // 浅色主题**不动**: 那里选区合成后是淡青 (242,238,235 一类的浅底), 中亮度
        // 前景压在它上面本来就够 (实测 ERROR 红 4.04:1), 收了反而让选中变难认。
        // 回归锁: `dark_selection_band_does_not_swallow_mid_luminance_foregrounds`。
        let a = self.accent();
        Color::rgba(a.r, a.g, a.b, 0.20)
    }

    fn caret(&self) -> Color {
        self.accent()
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

    fn control_height(&self) -> f32 {
        LightTheme.control_height()
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
    fn easing_endpoints_are_exact() {
        // 端点精确: 所有曲线 eval(0)=0, eval(1)=1 (动画首尾帧不漂移)。
        for e in [
            Easing::Linear,
            Easing::EaseInOut,
            Easing::EaseIn,
            Easing::EaseOut,
        ] {
            assert_eq!(e.eval(0.0), 0.0, "{e:?} 起点");
            assert_eq!(e.eval(1.0), 1.0, "{e:?} 终点");
        }
    }

    #[test]
    fn easing_is_monotonic() {
        // 单调不减: 动画不倒退 (步进扫描, 允许浮点等值)。
        for e in [
            Easing::Linear,
            Easing::EaseInOut,
            Easing::EaseIn,
            Easing::EaseOut,
        ] {
            let mut prev = e.eval(0.0);
            for i in 1..=100 {
                let cur = e.eval(i as f32 / 100.0);
                assert!(cur >= prev, "{e:?} 在 {i}/100 处倒退: {prev} -> {cur}");
                prev = cur;
            }
        }
    }

    #[test]
    fn ease_in_out_midpoint_direction() {
        // 中点方向性: 缓入中点低于线性 (起手慢), 缓出中点高于线性 (收尾慢)。
        assert!(Easing::EaseIn.eval(0.5) < 0.5);
        assert!(Easing::EaseOut.eval(0.5) > 0.5);
        // 与 pomodoro hint.rs 私有实现语义对齐: 三次方曲线。
        assert!((Easing::EaseIn.eval(0.5) - 0.125).abs() < 1e-6);
        assert!((Easing::EaseOut.eval(0.5) - 0.875).abs() < 1e-6);
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
    fn light_theme_control_height_is_36() {
        assert_eq!(LightTheme.control_height(), 36.0);
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

    /// 在 **linear 空间**把 `fg` 合成到 `bg` 上, 返回相对亮度 —— 复现硬件在 sRGB
    /// 渲染目标上的真实行为（模块 1 修好双重编码后, 这条路径才真正生效）。
    fn composited_luminance(fg: Color, bg: Color) -> f32 {
        let f = crate::render::LinearRgba::from(fg);
        let b = crate::render::LinearRgba::from(bg);
        let mix = |fc: f32, bc: f32| f.a * fc + (1.0 - f.a) * bc;
        0.2126 * mix(f.r, b.r) + 0.7152 * mix(f.g, b.g) + 0.0722 * mix(f.b, b.b)
    }

    #[test]
    fn dark_translucent_tokens_are_calibrated_for_linear_blending() {
        // 判据是**渲染之后的知觉台阶**, 不是 token 里的 alpha ——
        // 模块 1 之前「护栏全绿、屏幕全灰」的成因正是护栏量错了对象。
        //
        // 台阶按**角色**定, 不是一律同值: 大面积填充要淡 (否则就是一块板),
        // 1px 细线要够亮才看得见。`selection` 不在此表 —— 它是**语义高亮**,
        // 该显眼, 拿它跟填充比会得出误导性的结论。
        //
        // 这些 alpha 一律比「照 sRGB 空间手感定」小一个量级: 渲染目标按 linear
        // 混合, 而近黑底在 linear 空间**极度敏感** (白 1% 就已经是 +6 台阶)。
        // **每个上界都故意卡在旧值之下** —— 防止有人按老手感把 alpha 调回去。
        let bg = DarkTheme.background();
        let base = relative_luminance(bg);
        let cases: [(&str, Color, f32, f32); 5] = [
            ("surface", DarkTheme.surface(), 1.20, 1.60),
            ("surface_input", DarkTheme.surface_input(), 1.35, 1.75),
            ("surface_variant", DarkTheme.surface_variant(), 1.08, 1.30),
            ("divider", DarkTheme.divider(), 1.60, 2.00),
            ("border", DarkTheme.border(), 2.20, 2.70),
        ];
        for (name, color, lo, hi) in cases {
            let composited = composited_luminance(color, bg);
            let (h, l) = if composited > base {
                (composited, base)
            } else {
                (base, composited)
            };
            let ratio = (h + 0.05) / (l + 0.05);
            assert!(
                (lo..=hi).contains(&ratio),
                "{name} 的台阶越界: 对比度 {ratio:.3} 不在 {lo}..{hi} 内 \
                 (合成亮度 {composited:.5} vs 底色 {base:.5})"
            );
        }
    }

    /// **两个主题**的半透明「面」都不许渲染成一块板 —— 模块 1 那次事故的形态。
    ///
    /// 与上一条 `dark_translucent_tokens_are_calibrated_for_linear_blending` 的分工:
    /// 那条给暗色卡了**上下双向**的紧窗口 (它是当时逐支重校的产物); 这条只管**上限**,
    /// 但**两个主题都管** —— 「α 照 sRGB 空间的手感定、换到 linear 混合后渲染成
    /// 一块灰板」这件事与主题无关, 换哪个主题都可能复发。
    ///
    /// **为什么不设下限**: 浅色的三支近白面 (`surface` 1.06 / `surface_input` 1.08 /
    /// `surface_variant` 1.02) 落在任何合理下限之下 —— 但那是**既有设计**
    /// (底色 `L*` 已 96.95, 头顶只有 ~3 个点的余量; 玻璃感本就靠贴近底色),
    /// 且**不是**混合空间迁移造成的 (近白对那次变更是钝感的, 这几个值几乎没动)。
    /// 拿护栏去卡它们等于借护栏之名改浅色主题 —— 那是独立决策, 已单独记档, 不夹带。
    #[test]
    fn no_theme_renders_a_translucent_surface_as_a_slab() {
        /// 上限: 超过这条就不该再叫「面」了, 是一块板。
        const SLAB: f32 = 2.0;

        fn check<T: Theme>(name: &str, th: &T) {
            let bg = th.background();
            let base = relative_luminance(bg);
            for (token, c) in [
                ("surface", th.surface()),
                ("surface_input", th.surface_input()),
                ("surface_variant", th.surface_variant()),
            ] {
                let composited = composited_luminance(c, bg);
                let (hi, lo) = if composited > base {
                    (composited, base)
                } else {
                    (base, composited)
                };
                let ratio = (hi + 0.05) / (lo + 0.05);
                assert!(
                    ratio < SLAB,
                    "{name} 的 {token} 渲染成了板: 对比度 {ratio:.2} ≥ {SLAB}"
                );
            }
        }
        check("LightTheme", &LightTheme);
        check("DarkTheme", &DarkTheme);
    }

    /// 感知明度 (CIELAB `L*`, 0 = 黑, 100 = 白), **输入是已经合成好的亮度**。
    ///
    /// 台阶判据用它而**不用 WCAG 对比度**: 对比度是**文字**指标 (小面积、高反差),
    /// 拿尺子量「大面积底色之间差多少」会**严重低估** —— 浅色 `surface_variant`
    /// 对底色算出来 1.02, 看着「也行」, 实际 `ΔL*` 只有 0.74, 屏幕上就是没有。
    /// 这条教训当场踩过: 我先用对比度 1.10 给 `Dropdown` hover 判了「等于没有」,
    /// 换 `ΔL*` 一量是 3.72 —— **判反了**。
    ///
    /// 参数取**亮度**而不是 `Color`: 这支 token 在暗色是**半透明白**,
    /// 而 `relative_luminance` 按契约忽略 alpha —— 直接喂进去会得到纯白的 `L*` 100,
    /// 暗色那支算出来 Δ`L*` 90.96 (实测踩过)。半透明色必须先经
    /// [`composited_luminance`] (线性) 合成。
    fn l_star_of(y: f32) -> f32 {
        if y > 0.008856 {
            116.0 * y.powf(1.0 / 3.0) - 16.0
        } else {
            903.3 * y
        }
    }

    /// **同一支「次级表面」在两个主题里必须是同一个量级的台阶。**
    ///
    /// 回归锁 (2026-09-13, 由 D1 查出): 浅色 `surface_variant` 对底色 `ΔL*` 只有
    /// **0.74** (差 2/255), 而暗色同一支是 **+6.42** —— 同一个角色差一个量级,
    /// 这不是设计选择, 是漏了。产品侧有 5 处直接拿它压页面底 (表头 / 过滤栏 /
    /// 侧栏生效行 / 两处 hover), 浅色下全部看不见。
    ///
    /// 区间 `[3.0, 10.0]` 是**导出**的:
    /// - 下限 3.0 落在「用户明确说过看不见」的 0.74 与「用户已接受的浅色斑马」
    ///   3.49 之间;
    /// - 上限 10.0 挡住「次级表面变成一块板」。
    ///
    /// **只锁 `surface_variant`, 不锁 `surface` / `surface_input`** —— 那两支是
    /// **玻璃卡**, 生来就该贴近底色 (浅色 1.06 / 1.08), 角色不同。
    /// 这正是当初「跨主题一致性守卫」查实后**决定不建**的原因: 照字面一刀切会
    /// 一上线就误报。现在有了确切的数据与角色边界, 才收窄成这一条。
    #[test]
    fn surface_variant_is_a_comparable_step_in_both_themes() {
        const MIN: f32 = 3.0;
        const MAX: f32 = 10.0;

        fn check<T: Theme>(name: &str, th: &T) {
            let bg = th.background();
            let base = l_star_of(relative_luminance(bg));
            // 走**线性**合成 —— 暗色这支是半透明白, 不合成量的是纯白。
            let variant = l_star_of(composited_luminance(th.surface_variant(), bg));
            let step = (variant - base).abs();
            assert!(
                (MIN..=MAX).contains(&step),
                "{name} 的 surface_variant 对底色 ΔL* 是 {step:.2}, 落在 [{MIN}, {MAX}] 之外 —— \
                 太小就是「次级表面与底色同色」(看不见), 太大就是一块板"
            );
        }
        check("LightTheme", &LightTheme);
        check("DarkTheme", &DarkTheme);
    }

    /// 选区带**不得离底色太远** —— 它每强一分, 压在它上面的前景色就少一分对比度。
    ///
    /// 回归锁 (2026-09-13, 用户实机报): 暗色下选中一行 ERROR, 红字压在 accent 色的
    /// 选区带上「看得眼花」。根因不是某一支配色不好, 而是选区带**自己就是一支
    /// 中亮度的彩色** (accent 色相压在中间亮度), 于是跟任何中亮度前景都拉不开 ——
    /// 这是结构性冲突, 换个色号也只是挪走症状。
    ///
    /// 故本锁量的**不是**某个产品前景 (框架无从知道产品的配色), 而是那个共性因子:
    /// 合成后的选区带相对底色跨了多远。`BAND_MAX = 2.0` 是**导出**的 ——
    /// 暗色原来的 30% α 是 **2.27** (越线), 收窄到 20% 后 **1.85**; 浅色 30% 是
    /// **1.32**, 本来就在线内, 故本锁不动浅色。
    ///
    /// 合成走 [`composited_luminance`] (线性空间), 与模块 2 其余守卫同一把尺子。
    /// **不要改用公开的 [`composite_over`]** —— 那个在 sRGB 空间混, 量的不是屏幕。
    #[test]
    fn selection_band_does_not_step_too_far_from_the_background() {
        const BAND_MAX: f32 = 2.0;

        fn check<T: Theme>(name: &str, th: &T) {
            let bg = th.background();
            let base = relative_luminance(bg);
            let band = composited_luminance(th.selection(), bg);
            let (hi, lo) = if band > base {
                (band, base)
            } else {
                (base, band)
            };
            let ratio = (hi + 0.05) / (lo + 0.05);
            assert!(
                ratio < BAND_MAX,
                "{name} 的选区带离底色太远: 对比度 {ratio:.2} ≥ {BAND_MAX} \
                 —— 压在带上的中亮度前景会被吞掉"
            );
        }
        check("LightTheme", &LightTheme);
        check("DarkTheme", &DarkTheme);
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

    // ---- DarkTheme 测试 ----

    #[test]
    fn dark_theme_implements_theme() {
        fn assert_theme<T: Theme>() {}
        assert_theme::<DarkTheme>();
    }

    #[test]
    fn dark_theme_colors_are_visible() {
        let theme = DarkTheme;
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
    fn dark_theme_text_primary_vs_background_contrast() {
        // WCAG AAA: 正文 vs 背景 ≥ 7:1
        let theme = DarkTheme;
        let ratio = contrast_ratio(theme.text_primary(), theme.background());
        assert!(
            ratio >= 7.0,
            "深色主题 text_primary vs background 对比度应 ≥7:1, 实际 {ratio:.2}"
        );
    }

    #[test]
    fn dark_theme_accent_vs_background_contrast() {
        // WCAG AA: 强调色 vs 背景 ≥ 4.5:1
        let theme = DarkTheme;
        let ratio = contrast_ratio(theme.accent(), theme.background());
        assert!(
            ratio >= 4.5,
            "深色主题 accent vs background 对比度应 ≥4.5:1, 实际 {ratio:.2}"
        );
    }

    #[test]
    fn dark_theme_non_color_tokens_match_light_theme() {
        let dark = DarkTheme;
        let light = LightTheme;
        assert_eq!(dark.font_size_small(), light.font_size_small());
        assert_eq!(dark.font_size_body(), light.font_size_body());
        assert_eq!(dark.font_size_heading(), light.font_size_heading());
        assert_eq!(dark.control_height(), light.control_height());
        assert_eq!(dark.spacing_md(), light.spacing_md());
        assert_eq!(dark.radius_lg(), light.radius_lg());
        assert_eq!(dark.scrim(), light.scrim());
        assert_eq!(dark.danger(), light.danger());
    }
}
