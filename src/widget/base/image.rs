//! @author 十四叔
//! @date 2026/08/14
//!
//! Image 组件：在任意矩形内绘制 RGBA 图像。
//!
//! 最小实现：只做全幅采样，不做圆角裁剪/九宫格。
//! 纹理上传走 background.rs 的纹理机制 (BindGroup + Sampler)。

use crate::render::{RectBatch, TextBatch};
use crate::widget::Widget;
use crate::{Color, Constraints, Rect, Size};

/// Image 组件：在任意矩形内绘制 RGBA 图像。
///
/// 最小实现：只做全幅采样，不做圆角裁剪/九宫格。
/// 纹理上传走 background.rs 的纹理机制 (BindGroup + Sampler)。
pub struct Image {
    /// RGBA 像素数据 (宽 × 高 × 4 字节)。
    data: Vec<u8>,
    /// 图像宽度 (像素)。
    width: u32,
    /// 图像高度 (像素)。
    height: u32,
    /// 目标尺寸 (布局后确定)。
    target_size: Size,
}

impl Image {
    /// 创建 Image 组件。
    ///
    /// # Arguments
    /// * `data` - RGBA 像素数据 (宽 × 高 × 4 字节)
    /// * `width` - 图像宽度 (像素)
    /// * `height` - 图像高度 (像素)
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            target_size: Size::ZERO,
        }
    }

    /// 获取图像宽度 (像素)。
    pub fn width(&self) -> u32 {
        self.width
    }

    /// 获取图像高度 (像素)。
    pub fn height(&self) -> u32 {
        self.height
    }

    /// 获取 RGBA 像素数据。
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 计算图像主色 (取中心区域平均值)。
    ///
    /// 用于最小实现的矩形占位绘制。
    fn dominant_color(&self) -> Color {
        if self.data.is_empty() || self.width == 0 || self.height == 0 {
            return Color::rgba(0.5, 0.5, 0.5, 0.8);
        }

        // 取中心 16x16 区域的平均值
        let (cx, cy) = (self.width / 2, self.height / 2);
        let r = 8.min(self.width / 2).min(self.height / 2);
        let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);

        for dy in 0..r.min(8) {
            for dx in 0..r.min(8) {
                let px = (cx + dx) % self.width;
                let py = (cy + dy) % self.height;
                let off = ((py * self.width + px) * 4) as usize;
                if off + 3 < self.data.len() {
                    sr += self.data[off] as u32;
                    sg += self.data[off + 1] as u32;
                    sb += self.data[off + 2] as u32;
                    sa += self.data[off + 3] as u32;
                    n += 1;
                }
            }
        }

        if n == 0 {
            return Color::rgba(0.5, 0.5, 0.5, 0.8);
        }

        Color::rgba(
            sr as f32 / n as f32 / 255.0,
            sg as f32 / n as f32 / 255.0,
            sb as f32 / n as f32 / 255.0,
            sa as f32 / n as f32 / 255.0,
        )
    }
}

impl Widget for Image {
    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        // 在约束下保持原始宽高比
        let max_width = constraints.max_width;
        let max_height = constraints.max_height;
        let aspect = self.width as f32 / self.height as f32;

        let (width, height) = if max_width * self.height as f32 <= max_height * self.width as f32 {
            // 受限于宽度
            (max_width, max_width / aspect)
        } else {
            // 受限于高度
            (max_height * aspect, max_height)
        };

        self.target_size = Size::new(width, height);
        self.target_size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, _texts: &mut TextBatch) {
        // 最小实现：使用图像主色绘制矩形占位
        // TODO: 实现真正的 RGBA 纹理渲染
        let color = self.dominant_color();
        rects.push_rect(area, color, 0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::Widget;
    use crate::{Constraints, Rect, Size};

    fn create_test_image() -> Image {
        // 创建一个 2x2 的 RGBA 图像
        let data = vec![
            255, 0, 0, 255, // 红色
            0, 255, 0, 255, // 绿色
            0, 0, 255, 255, // 蓝色
            255, 255, 0, 255, // 黄色
        ];
        Image::new(data, 2, 2)
    }

    #[test]
    fn image_creation() {
        let img = create_test_image();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.data().len(), 16); // 2x2x4
    }

    #[test]
    fn image_layout_preserves_aspect_ratio() {
        let mut img = create_test_image();
        let mut texts = crate::TextBatch::new();
        let constraints = Constraints::tight(Size::new(100.0, 100.0));
        let size = img.layout(constraints, &mut texts);
        // 2x2 图像在 100x100 约束下应该保持 1:1 比例
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 100.0);
    }

    #[test]
    fn image_layout_width_constrained() {
        let mut img = Image::new(vec![0; 200], 20, 10); // 2:1 宽高比
        let mut texts = crate::TextBatch::new();
        let constraints = Constraints::tight(Size::new(100.0, 50.0));
        let size = img.layout(constraints, &mut texts);
        // 受限于宽度
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);
    }

    #[test]
    fn image_layout_height_constrained() {
        let mut img = Image::new(vec![0; 200], 10, 20); // 1:2 宽高比
        let mut texts = crate::TextBatch::new();
        let constraints = Constraints::tight(Size::new(50.0, 100.0));
        let size = img.layout(constraints, &mut texts);
        // 受限于高度
        assert_eq!(size.width, 50.0);
        assert_eq!(size.height, 100.0);
    }

    #[test]
    fn image_paint_does_not_panic() {
        let img = create_test_image();
        let mut rects = RectBatch::new();
        let mut texts = crate::TextBatch::new();
        let area = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);
        img.paint(area, &mut rects, &mut texts);
        // 验证 paint 不 panic
    }

    #[test]
    fn image_dominant_color() {
        // 纯红色图像 (2x2)
        let data = vec![255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255];
        let img = Image::new(data, 2, 2);
        let color = img.dominant_color();
        // 主色应该是红色
        assert!(color.r > 0.9, "红色分量应该接近 1.0");
        assert!(color.g < 0.1, "绿色分量应该接近 0.0");
        assert!(color.b < 0.1, "蓝色分量应该接近 0.0");
        assert!(color.a > 0.9, "Alpha 分量应该接近 1.0");
    }

    #[test]
    fn image_dominant_color_empty() {
        let img = Image::new(vec![], 0, 0);
        let color = img.dominant_color();
        // 空图像应该返回默认灰色
        assert!(color.r > 0.4 && color.r < 0.6, "红色分量应该在 0.5 附近");
        assert!(color.g > 0.4 && color.g < 0.6, "绿色分量应该在 0.5 附近");
        assert!(color.b > 0.4 && color.b < 0.6, "蓝色分量应该在 0.5 附近");
    }
}
