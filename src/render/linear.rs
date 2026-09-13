//! @author 十四叔
//! @date 2026/09/13

//! 色彩空间边界：作者态 sRGB → GPU 线性空间。
//!
//! 渲染目标是 sRGB 格式，硬件在写入时做 linear→sRGB 编码。所以**送进管线的颜色
//! 必须已经是线性的** —— 否则会被编码两次（双重 gamma），表现为暗色主题的近黑
//! 背景显示成中灰、正文与背景糊成一片。
//!
//! [`LinearRgba`] 是这条边界的**类型标记**：实例的颜色字段用它，把未转换的
//! [`Color`] 直接传过去**编译不过**。这比 doc comment 可靠 —— 双重 gamma 事故的
//! 根因正是 `Color` 的色彩空间契约含糊（`layout.rs` 与 `theme.rs` 两处 doc 说法相反）。
//!
//! 转换的数学在 [`crate::layout::srgb_to_linear`]，本模块只做包装，不另写一份。

use crate::Color;
use crate::layout::srgb_to_linear;

/// 线性空间 RGBA —— 已解码，可直接送 GPU。
///
/// 只应由 [`LinearRgba::from_srgb`]（或 `From<Color>`）产生。
///
/// `#[repr(C)]` 与 `Pod` 是**承重**的：实例结构体的顶点属性按 `Float32x4` 读取，
/// 本类型的内存布局必须与 `[f32; 4]` 逐字节一致（4 个 f32、16 字节、无填充）。
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LinearRgba {
    /// 红色分量 (线性)。
    pub r: f32,
    /// 绿色分量 (线性)。
    pub g: f32,
    /// 蓝色分量 (线性)。
    pub b: f32,
    /// 不透明度 —— **不参与**色彩空间转换，原样透传。
    pub a: f32,
}

impl LinearRgba {
    /// 由作者态 sRGB 颜色解码。alpha 原样透传。
    pub fn from_srgb(color: Color) -> Self {
        Self {
            r: srgb_to_linear(color.r),
            g: srgb_to_linear(color.g),
            b: srgb_to_linear(color.b),
            a: color.a,
        }
    }

    /// 转成 `LoadOp::Clear` 接收的 `wgpu::Color` (f64)。
    ///
    /// **清屏是「颜色进 GPU」的另一条路**, 同样不能漏解码: 清屏值写进 sRGB 目标
    /// 一样会被硬件再编码一次 —— 那正是「暗色窗口底色发灰」的成因之一。
    pub fn to_clear_value(self) -> wgpu::Color {
        wgpu::Color {
            r: f64::from(self.r),
            g: f64::from(self.g),
            b: f64::from(self.b),
            a: f64::from(self.a),
        }
    }
}

impl From<Color> for LinearRgba {
    fn from(color: Color) -> Self {
        Self::from_srgb(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_are_fixed_points() {
        assert_eq!(
            LinearRgba::from_srgb(Color::rgb(0.0, 0.0, 0.0)),
            LinearRgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0
            }
        );
        assert_eq!(
            LinearRgba::from_srgb(Color::WHITE),
            LinearRgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0
            }
        );
    }

    #[test]
    fn alpha_passes_through_untouched() {
        // 半透明表面 (白 10%) 解码后仍是 10% —— alpha 不参与色彩空间转换。
        let c = Color::rgba(1.0, 1.0, 1.0, 0.10);
        assert_eq!(LinearRgba::from_srgb(c).a, 0.10);
    }

    #[test]
    fn dark_theme_background_decodes_to_near_black() {
        // #191920 —— 解码后必须仍是近黑, 不能被抬成中灰。
        // 这是双重 gamma 事故的回归守卫。
        let lin = LinearRgba::from_srgb(Color::from_srgb8(25, 25, 32));
        assert!((lin.r - 0.00972).abs() < 1e-4, "r 实得 {}", lin.r);
        assert!((lin.g - 0.00972).abs() < 1e-4, "g 实得 {}", lin.g);
        assert!((lin.b - 0.01445).abs() < 1e-4, "b 实得 {}", lin.b);
        assert_eq!(lin.a, 1.0);
    }

    #[test]
    fn clear_value_decodes_like_every_other_color_path() {
        // 清屏走同一条边界: 暗色背景 #191920 的清屏值必须是**解码后**的近黑,
        // 否则窗口底色会被再编码一次抬成中灰。
        let v = LinearRgba::from(Color::from_srgb8(25, 25, 32)).to_clear_value();
        // 容差 1e-5: 参考值是手算的 5 位小数, 而实现走 f32 (见 layout 侧同款断言)。
        assert!((v.r - 0.00972).abs() < 1e-5, "r 实得 {}", v.r);
        assert!((v.g - 0.00972).abs() < 1e-5, "g 实得 {}", v.g);
        assert!((v.b - 0.01445).abs() < 1e-5, "b 实得 {}", v.b);
        assert_eq!(v.a, 1.0, "alpha 不参与转换");
    }

    #[test]
    fn clear_value_leaves_near_white_almost_unchanged() {
        // 已知代价 (spec §2): 浅色主题的清屏色是近白, 解码后几乎不动 ——
        // 这正是「浅色看着还行、暗色被毁」的原因。0.98 → ≈0.9551。
        let v = LinearRgba::from(Color::rgb(0.98, 0.98, 0.98)).to_clear_value();
        assert!((v.r - 0.9551).abs() < 1e-3, "r 实得 {}", v.r);
    }

    #[test]
    fn from_impl_matches_from_srgb() {
        let c = Color::rgba(0.2, 0.5, 0.9, 0.4);
        assert_eq!(LinearRgba::from(c), LinearRgba::from_srgb(c));
    }
}
