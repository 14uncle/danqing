//! @author 十四叔
//! @date 2026/07/17

//! 字形图集：把栅格化字形缓存到一张大位图 (shelf/bucket 分配)。
//!
//! 纯逻辑模块：图集只是一块 CPU 位图; 上传 GPU 由渲染层负责。
//! 缓存键为 (字符，像素字号); 同一键只栅格化一次。

use std::collections::HashMap;

use etagere::{BucketedAtlasAllocator, size2};

/// 单个字形在图集中的信息。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphInfo {
    /// 图集内左上角 (纹素坐标，含 1px 内边距后的实际字形起点)。
    pub uv_min: (u32, u32),
    /// 图集内右下角 (不含)。
    pub uv_max: (u32, u32),
    /// 字形位图宽度 (像素，空格等为 0)。
    pub width: u32,
    /// 字形位图高度 (像素)。
    pub height: u32,
    /// 水平 bearing: 位图左边相对笔迹原点的偏移 (fontdue xmin)。
    pub bearing_x: i32,
    /// 垂直 bearing: 基线到位图顶边的距离 (fontdue 坐标系中 y 向上，
    /// 等于 `ymin + height`,即 ymax)。
    pub bearing_y: i32,
    /// 笔迹前进宽度 (逻辑像素)。
    pub advance: f32,
}

/// 图集错误。
#[derive(Debug, thiserror::Error)]
pub enum AtlasError {
    /// 图集已满，无法分配新字形。
    #[error("字形图集已满，无法放入 '{ch}' ({px}px)")]
    Full {
        /// 无法放入的字符。
        ch: char,
        /// 像素字号。
        px: u16,
    },
}

/// 字形图集 (u8 alpha 单通道位图 + bucket 分配器)。
pub struct GlyphAtlas {
    packer: BucketedAtlasAllocator,
    /// 正方形图集边长。
    width: u32,
    /// 行主序 alpha 位图。
    pixels: Vec<u8>,
    /// (字符，像素字号) → 图集信息。
    glyphs: HashMap<(char, u16), GlyphInfo>,
    /// 尚未上传 GPU 的脏区域 (min_x, min_y, max_x, max_y), 纹素坐标。
    dirty: Option<(u32, u32, u32, u32)>,
}

impl GlyphAtlas {
    /// 默认图集边长 (1024×1024, 可存约数千个 16px 字形)。
    pub const DEFAULT_SIZE: u32 = 1024;
    /// 字形四周内边距 (防采样串色)。
    const PADDING: u32 = 1;

    /// 创建默认大小图集。
    pub fn new() -> Self {
        Self::with_size(Self::DEFAULT_SIZE)
    }

    /// 创建指定边长的正方形图集 (测试用小图集)。
    pub fn with_size(size: u32) -> Self {
        Self {
            packer: BucketedAtlasAllocator::new(size2(size as i32, size as i32)),
            width: size,
            pixels: vec![0; (size * size) as usize],
            glyphs: HashMap::new(),
            dirty: None,
        }
    }

    /// 图集边长。
    pub fn size(&self) -> u32 {
        self.width
    }

    /// 取字形信息; 未缓存则栅格化并放入图集。
    ///
    /// 返回的字形信息 Copy 出图集借用，避免与后续 &mut 调用冲突。
    pub fn get_or_rasterize(
        &mut self,
        font: &fontdue::Font,
        ch: char,
        px: u16,
    ) -> Result<GlyphInfo, AtlasError> {
        let key = (ch, px);
        if let Some(info) = self.glyphs.get(&key) {
            return Ok(*info);
        }

        let (metrics, bitmap) = font.rasterize(ch, f32::from(px));
        let info = if metrics.width == 0 || metrics.height == 0 {
            // 空格等无位图字形：只占缓存，不占图集
            GlyphInfo {
                uv_min: (0, 0),
                uv_max: (0, 0),
                width: 0,
                height: 0,
                bearing_x: metrics.xmin,
                bearing_y: metrics.ymin + metrics.height as i32,
                advance: metrics.advance_width,
            }
        } else {
            let alloc_w = metrics.width as i32 + Self::PADDING as i32 * 2;
            let alloc_h = metrics.height as i32 + Self::PADDING as i32 * 2;
            let allocation = self
                .packer
                .allocate(size2(alloc_w, alloc_h))
                .ok_or(AtlasError::Full { ch, px })?;
            let (x0, y0) = (
                allocation.rectangle.min.x as u32 + Self::PADDING,
                allocation.rectangle.min.y as u32 + Self::PADDING,
            );
            // 拷贝字形位图到图集 (行主序)
            for row in 0..metrics.height {
                let dst = ((y0 + row as u32) * self.width + x0) as usize;
                let src = row * metrics.width;
                self.pixels[dst..dst + metrics.width]
                    .copy_from_slice(&bitmap[src..src + metrics.width]);
            }
            self.expand_dirty(
                x0 - Self::PADDING,
                y0 - Self::PADDING,
                x0 + metrics.width as u32 + Self::PADDING,
                y0 + metrics.height as u32 + Self::PADDING,
            );
            GlyphInfo {
                uv_min: (x0, y0),
                uv_max: (x0 + metrics.width as u32, y0 + metrics.height as u32),
                width: metrics.width as u32,
                height: metrics.height as u32,
                bearing_x: metrics.xmin,
                bearing_y: metrics.ymin + metrics.height as i32,
                advance: metrics.advance_width,
            }
        };
        self.glyphs.insert(key, info);
        Ok(info)
    }

    /// 已缓存字形数量。
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    /// 图集位图数据 (行主序 u8 alpha)。
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// 取走脏区域 (上传后调用); 无脏区域返回 None。
    pub fn take_dirty(&mut self) -> Option<(u32, u32, u32, u32)> {
        self.dirty.take()
    }

    fn expand_dirty(&mut self, min_x: u32, min_y: u32, max_x: u32, max_y: u32) {
        self.dirty = Some(match self.dirty {
            Some((a, b, c, d)) => (a.min(min_x), b.min(min_y), c.max(max_x), d.max(max_y)),
            None => (min_x, min_y, max_x, max_y),
        });
    }
}

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Font;

    fn test_font() -> fontdue::Font {
        Font::fallback().inner().clone()
    }

    #[test]
    fn two_glyphs_no_overlap() {
        let font = test_font();
        let mut atlas = GlyphAtlas::new();
        let a = atlas.get_or_rasterize(&font, '你', 16).unwrap();
        let b = atlas.get_or_rasterize(&font, '好', 16).unwrap();
        // 矩形不相交
        let overlap = a.uv_min.0 < b.uv_max.0
            && b.uv_min.0 < a.uv_max.0
            && a.uv_min.1 < b.uv_max.1
            && b.uv_min.1 < a.uv_max.1;
        assert!(!overlap, "两个字形的图集区域不得重叠：{a:?} {b:?}");
        assert!(a.width > 0 && b.width > 0);
    }

    #[test]
    fn cache_hit_rasterizes_once() {
        let font = test_font();
        let mut atlas = GlyphAtlas::new();
        let first = atlas.get_or_rasterize(&font, '世', 16).unwrap();
        let second = atlas.get_or_rasterize(&font, '世', 16).unwrap();
        assert_eq!(first, second);
        assert_eq!(atlas.glyph_count(), 1);
        // 不同字号是不同缓存项
        atlas.get_or_rasterize(&font, '世', 24).unwrap();
        assert_eq!(atlas.glyph_count(), 2);
    }

    #[test]
    fn dirty_region_tracked_and_cleared() {
        let font = test_font();
        let mut atlas = GlyphAtlas::new();
        assert_eq!(atlas.take_dirty(), None);
        let a = atlas.get_or_rasterize(&font, '你', 16).unwrap();
        let dirty = atlas.take_dirty().expect("栅格化后必须有脏区域");
        assert!(dirty.0 <= a.uv_min.0 && dirty.2 >= a.uv_max.0);
        assert_eq!(atlas.take_dirty(), None, "脏区域取走后必须清空");
    }

    #[test]
    fn atlas_full_reports_error() {
        let font = test_font();
        let mut atlas = GlyphAtlas::with_size(8); // 微型图集
        let result = atlas.get_or_rasterize(&font, '你', 16);
        assert!(matches!(result, Err(AtlasError::Full { .. })));
    }

    #[test]
    fn space_glyph_uses_no_atlas_space() {
        let font = test_font();
        let mut atlas = GlyphAtlas::new();
        let info = atlas.get_or_rasterize(&font, ' ', 16).unwrap();
        assert_eq!(info.width, 0);
        assert!(info.advance > 0.0, "空格必须有前进宽度");
        assert_eq!(atlas.take_dirty(), None, "无位图字形不产生脏区域");
    }

    #[test]
    fn glyph_bearing_y_points_to_bitmap_top() {
        let font = test_font();
        let mut atlas = GlyphAtlas::new();
        let info = atlas.get_or_rasterize(&font, 'A', 16).unwrap();
        let (metrics, _) = font.rasterize('A', 16.0);
        assert_eq!(
            info.bearing_y,
            metrics.ymin + metrics.height as i32,
            "bearing_y 必须指向位图顶边 (基线以上)"
        );
        assert!(info.bearing_y > 0, "顶边必须在基线之上");
    }
}
