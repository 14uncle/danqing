//! @author 十四叔
//! @date 2026/08/12

//! Image 组件：保持原始宽高比缩放到约束区域，居中渲染。
//!
//! GPU 纹理管线就绪后，`paint` 向 [`ImageBatch`] 推送纹理实例；
//! 回退时绘制主色调色块。

use crate::render::ImageBatch;
use crate::widget::{RectBatch, Widget};
use crate::{Color, Constraints, Rect, Size, TextBatch};

/// Image 组件：按原始宽高比缩放并居中渲染。
///
/// 像素数据 (`data`) 按行优先存储 8-bit RGBA (每像素 4 字节)。
pub struct Image {
    /// RGBA 像素数据 (按行优先, 每像素 4 字节)。
    data: Vec<u8>,
    /// 图像宽度 (像素)。
    width: u32,
    /// 图像高度 (像素)。
    height: u32,
    /// 期望渲染尺寸 (逻辑像素)。
    target_size: Size,
}

impl Image {
    /// 创建 Image 组件。
    ///
    /// * `data` - RGBA8 像素数据 (按行优先, 每像素 4 字节)。
    /// * `width` - 图像宽度 (像素)。
    /// * `height` - 图像高度 (像素)。
    pub fn new(data: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
            target_size: Size::new(200.0, 200.0),
        }
    }

    /// 设置目标渲染尺寸。
    pub fn target_size(mut self, size: Size) -> Self {
        self.target_size = size;
        self
    }

    /// 计算保持宽高比的缩放尺寸。
    /// - 图片小于约束：不缩放，居中显示
    /// - 图片大于约束：等比缩放，完整显示
    fn aspect_fit(&self, constraints: Constraints) -> Size {
        let max_w = constraints.max_width;
        let max_h = constraints.max_height;
        if self.width == 0 || self.height == 0 {
            return Size::ZERO;
        }
        let img_w = self.width as f32;
        let img_h = self.height as f32;

        // 图片小于约束，不缩放
        if img_w <= max_w && img_h <= max_h {
            return Size::new(img_w, img_h);
        }

        // 图片大于约束，等比缩放
        let aspect = img_w / img_h;
        let w = max_h * aspect;
        if w <= max_w {
            Size::new(w, max_h)
        } else {
            Size::new(max_w, max_w / aspect)
        }
    }

    /// 取图像中心 16×16 区域的平均 RGBA 颜色 (回退色块)。
    fn dominant_color(&self) -> Color {
        if self.data.is_empty() {
            return Color::rgba(0.7, 0.7, 0.7, 1.0);
        }
        let (cx, cy) = (self.width / 2, self.height / 2);
        let half = 8u32;
        let x0 = cx.saturating_sub(half);
        let y0 = cy.saturating_sub(half);
        let x1 = (cx + half).min(self.width);
        let y1 = (cy + half).min(self.height);
        let mut r_sum = 0u32;
        let mut g_sum = 0u32;
        let mut b_sum = 0u32;
        let mut count = 0u32;
        for y in y0..y1 {
            for x in x0..x1 {
                let idx = ((y * self.width + x) * 4) as usize;
                if idx + 3 < self.data.len() {
                    r_sum += self.data[idx] as u32;
                    g_sum += self.data[idx + 1] as u32;
                    b_sum += self.data[idx + 2] as u32;
                    count += 1;
                }
            }
        }
        if count == 0 {
            return Color::rgba(0.7, 0.7, 0.7, 1.0);
        }
        Color::rgba(
            r_sum as f32 / count as f32 / 255.0,
            g_sum as f32 / count as f32 / 255.0,
            b_sum as f32 / count as f32 / 255.0,
            1.0,
        )
    }
}

impl Widget for Image {
    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        let size = self.aspect_fit(constraints);
        self.target_size = size;
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, _texts: &mut TextBatch) {
        // 绘制主色调色块作为背景 (回退或纹理尚未上传时可见)
        let color = self.dominant_color();
        rects.push_rect(area, color, 0.0);
        rects.push_rounded_border(area, Color::rgba(0.6, 0.6, 0.6, 1.0), 0.0, 1.0);
    }

    fn paint_image(&self, area: Rect, images: &mut ImageBatch) {
        // 有数据时推送纹理实例
        if !self.data.is_empty() && self.width > 0 && self.height > 0 {
            images.push_image(&self.data, self.width, self.height, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个纯色 RGBA 测试图像。
    fn solid_image(w: u32, h: u32, rgba: [u8; 4]) -> (Vec<u8>, u32, u32) {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&rgba);
        }
        (data, w, h)
    }

    #[test]
    fn aspect_fit_landscape() {
        let (data, w, h) = solid_image(200, 100, [255, 0, 0, 255]);
        let img = Image::new(data, w, h);
        let c = Constraints::tight(Size::new(80.0, 60.0));
        let size = img.aspect_fit(c);
        assert!((size.width - 80.0).abs() < 1e-4);
        assert!((size.height - 40.0).abs() < 1e-4);
    }

    #[test]
    fn aspect_fit_portrait() {
        let (data, w, h) = solid_image(100, 200, [0, 255, 0, 255]);
        let img = Image::new(data, w, h);
        let c = Constraints::tight(Size::new(80.0, 60.0));
        let size = img.aspect_fit(c);
        assert!((size.width - 30.0).abs() < 1e-4);
        assert!((size.height - 60.0).abs() < 1e-4);
    }

    #[test]
    fn aspect_fit_square() {
        let (data, w, h) = solid_image(100, 100, [0, 0, 255, 255]);
        let img = Image::new(data, w, h);
        let c = Constraints::tight(Size::new(80.0, 60.0));
        let size = img.aspect_fit(c);
        assert!((size.width - 60.0).abs() < 1e-4);
        assert!((size.height - 60.0).abs() < 1e-4);
    }

    #[test]
    fn aspect_fit_empty_image() {
        let img = Image::new(vec![], 0, 0);
        let c = Constraints::tight(Size::new(80.0, 60.0));
        let size = img.aspect_fit(c);
        assert_eq!(size, Size::ZERO);
    }

    #[test]
    fn dominant_color_picks_center_region() {
        // 4×4 图像, 左半红色, 右半绿色。
        let mut data = Vec::with_capacity(4 * 4 * 4);
        for _y in 0..4u32 {
            for x in 0..4u32 {
                if x < 2 {
                    data.extend_from_slice(&[255, 0, 0, 255]);
                } else {
                    data.extend_from_slice(&[0, 255, 0, 255]);
                }
            }
        }
        let img = Image::new(data, 4, 4);
        let color = img.dominant_color();
        // 中心 16×16 区域实际裁到整个 4×4, 平均后 R≈0.5, G≈0.5。
        assert!(color.r > 0.4 && color.r < 0.6);
        assert!(color.g > 0.4 && color.g < 0.6);
    }

    #[test]
    fn layout_returns_aspect_fit_size() {
        // 图像 200x100 (宽高比 2:1), 约束 100x80
        // w = 80 * 2 = 160 > 100, 所以 size = (100, 50)
        let (data, w, h) = solid_image(200, 100, [128, 128, 128, 255]);
        let mut img = Image::new(data, w, h);
        let c = Constraints::tight(Size::new(100.0, 80.0));
        let mut texts = TextBatch::new();
        let size = img.layout(c, &mut texts);
        assert!((size.width - 100.0).abs() < 1e-4);
        assert!((size.height - 50.0).abs() < 1e-4);
    }

    #[test]
    fn paint_pushes_color_and_border() {
        let (data, w, h) = solid_image(10, 10, [100, 150, 200, 255]);
        let img = Image::new(data, w, h);
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        let area = Rect::from_xywh(0.0, 0.0, 80.0, 40.0);
        img.paint(area, &mut rects, &mut texts);
        // 应推送填充 + 描边 (描边包含多条线段)
        assert!(rects.len() >= 2);
    }

    #[test]
    fn paint_image_pushes_to_batch() {
        let (data, w, h) = solid_image(10, 10, [100, 150, 200, 255]);
        let img = Image::new(data, w, h);
        let mut images = ImageBatch::new();
        let area = Rect::from_xywh(0.0, 0.0, 80.0, 40.0);
        img.paint_image(area, &mut images);
        assert_eq!(images.len(), 1);
    }
}
