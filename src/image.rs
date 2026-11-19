//! @author 十四叔
//! @date 2026/09/10
//!
//! 图像处理工具: RGBA 降采样、等比缩进、RGBA→PNG 纯编码。
//!
//! 下沉自 danqing-clipboard `history_list.rs:370-423` (2026-09-10, 小件包 S3)。
//! 降采样/缩进为纯 f32/usize 逻辑零依赖; encode_png 复用现有 `image` crate
//! (png feature) 做纯内存编码 —— 不做文件 IO, 磁盘写入留在调用方 (2026-11-19, 场景 A)。

/// RGBA 数据降采样到目标尺寸 (双线性插值, 平滑边缘)。
///
/// `data` 为 RGBA 原始字节 (src_w × src_h × 4), 返回 dst_w × dst_h × 4。
/// panics 若 `data.len() < src_w × src_h × 4` (debug 模式有断言)。
pub fn downscale_rgba(data: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    debug_assert!(
        data.len() >= (src_w * src_h * 4) as usize,
        "downscale_rgba: 数据长度 {} < src {}×{}×4={}",
        data.len(),
        src_w,
        src_h,
        src_w * src_h * 4
    );
    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
    let sx_ratio = src_w as f32 / dst_w as f32;
    let sy_ratio = src_h as f32 / dst_h as f32;

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let src_x = (dx as f32 + 0.5) * sx_ratio - 0.5;
            let src_y = (dy as f32 + 0.5) * sy_ratio - 0.5;

            let x0 = src_x.floor().max(0.0) as u32;
            let y0 = src_y.floor().max(0.0) as u32;
            let x1 = (x0 + 1).min(src_w - 1);
            let y1 = (y0 + 1).min(src_h - 1);

            let wx = src_x - x0 as f32;
            let wy = src_y - y0 as f32;

            let p00 = &data[(y0 * src_w + x0) as usize * 4..][..4];
            let p10 = &data[(y0 * src_w + x1) as usize * 4..][..4];
            let p01 = &data[(y1 * src_w + x0) as usize * 4..][..4];
            let p11 = &data[(y1 * src_w + x1) as usize * 4..][..4];

            let dst_idx = (dy * dst_w + dx) as usize * 4;
            for c in 0..4 {
                let v = p00[c] as f32 * (1.0 - wx) * (1.0 - wy)
                    + p10[c] as f32 * wx * (1.0 - wy)
                    + p01[c] as f32 * (1.0 - wx) * wy
                    + p11[c] as f32 * wx * wy;
                out[dst_idx + c] = v.round() as u8;
            }
        }
    }
    out
}

/// 等比缩进 max×max 方框 (不放大), 返回 (宽, 高)。
///
/// 图片小于 max 时返回原尺寸 (不放大)。
pub fn aspect_fit(w: u32, h: u32, max: f32) -> (f32, f32) {
    let (w, h) = (w as f32, h as f32);
    let scale = (max / w).min(max / h).min(1.0);
    (w * scale, h * scale)
}

/// PNG 编码失败原因。
///
/// 做什么: 穷举编码前校验与编码器两类失败; 不做什么: 不含文件 IO 错误
/// (本模块纯内存编码, 磁盘写入与清理留在调用方)。
#[derive(Debug)]
pub enum PngEncodeError {
    /// 宽或高为零
    ZeroDimension,
    /// `width × height × 4` 算术溢出
    DimensionOverflow,
    /// RGBA 字节数不足以覆盖全部像素
    BufferTooShort {
        /// 需要的字节数 (width × height × 4)
        need: usize,
        /// 实际拿到的字节数
        got: usize,
    },
    /// PNG 编码器返回失败 (含原因文本)
    Encode(String),
}

impl std::fmt::Display for PngEncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PngEncodeError::ZeroDimension => write!(f, "宽或高为零"),
            PngEncodeError::DimensionOverflow => write!(f, "width × height × 4 算术溢出"),
            PngEncodeError::BufferTooShort { need, got } => {
                write!(f, "RGBA 字节数不足: 需要 {need}, 实际 {got}")
            }
            PngEncodeError::Encode(e) => write!(f, "PNG 编码器失败: {e}"),
        }
    }
}

impl std::error::Error for PngEncodeError {}

/// RGBA 像素编码为 PNG 字节 (无损 RGBA8, 纯内存)。
///
/// 做什么: 把 `width × height × 4` 的原始 RGBA 编码成 PNG 文件字节,
/// 保留原始宽高与全部像素, 不缩放不裁切。
/// 不做什么: 不写文件、不碰磁盘; 文件路径与 IO 留在调用方。
/// 为什么: 场景导出需要真实 PNG 文件, 历史存储链路的 raw RGBA 格式不受影响
/// (danqing-clipboard 场景 A, 2026-11-19)。
///
/// `rgba` 只要求长度覆盖 `width × height × 4`, 多余字节忽略;
/// 零尺寸、算术溢出、长度不足返回显式错误, 不 panic。
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, PngEncodeError> {
    if width == 0 || height == 0 {
        return Err(PngEncodeError::ZeroDimension);
    }
    let need = (width as u64)
        .checked_mul(height as u64)
        .and_then(|px| px.checked_mul(4))
        .ok_or(PngEncodeError::DimensionOverflow)? as usize;
    if rgba.len() < need {
        return Err(PngEncodeError::BufferTooShort {
            need,
            got: rgba.len(),
        });
    }
    use image::codecs::png::PngEncoder;
    use image::{ExtendedColorType, ImageEncoder};

    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(&rgba[..need], width, height, ExtendedColorType::Rgba8)
        .map_err(|e| PngEncodeError::Encode(e.to_string()))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_png_roundtrip_2x2() {
        // 2×2 已知 RGBA → 编码 → 解码回验: 尺寸与像素一致
        let rgba = vec![
            255, 0, 0, 255, // (0,0) 红
            0, 255, 0, 128, // (1,0) 绿半透明
            0, 0, 255, 255, // (0,1) 蓝
            255, 255, 255, 0, // (1,1) 全透明
        ];
        let png = encode_png(2, 2, &rgba).expect("编码应成功");
        let img = image::load_from_memory(&png).expect("PNG 应可解码");
        assert_eq!((img.width(), img.height()), (2, 2), "尺寸保持");
        assert_eq!(img.to_rgba8().into_raw(), rgba, "像素逐字节一致 (RGBA8)");
    }

    #[test]
    fn encode_png_zero_dimension_rejected() {
        assert!(matches!(
            encode_png(0, 1, &[]),
            Err(PngEncodeError::ZeroDimension)
        ));
        assert!(matches!(
            encode_png(1, 0, &[]),
            Err(PngEncodeError::ZeroDimension)
        ));
    }

    #[test]
    fn encode_png_short_buffer_rejected() {
        // 2×2 需要 16 字节, 只给 15
        let mut rgba = vec![0u8; 15];
        rgba[0] = 1;
        assert!(matches!(
            encode_png(2, 2, &rgba),
            Err(PngEncodeError::BufferTooShort { need: 16, got: 15 })
        ));
    }

    #[test]
    fn encode_png_dimension_overflow_rejected() {
        // u32::MAX × u32::MAX × 4 在 u64 也算不下, 必须显式错误不 panic
        let rgba = [0u8; 4];
        assert!(matches!(
            encode_png(u32::MAX, u32::MAX, &rgba),
            Err(PngEncodeError::DimensionOverflow)
        ));
    }

    #[test]
    fn aspect_fit_scales_into_box() {
        let (w, h) = aspect_fit(200, 100, 36.0);
        assert!(
            (w - 36.0).abs() < 1e-4 && (h - 18.0).abs() < 1e-4,
            "横图: {w}×{h}"
        );
        let (w, h) = aspect_fit(100, 200, 36.0);
        assert!(
            (w - 18.0).abs() < 1e-4 && (h - 36.0).abs() < 1e-4,
            "竖图: {w}×{h}"
        );
    }

    #[test]
    fn aspect_fit_does_not_upscale() {
        let (w, h) = aspect_fit(20, 20, 36.0);
        assert!(
            (w - 20.0).abs() < 1e-4 && (h - 20.0).abs() < 1e-4,
            "小图不放大: {w}×{h}"
        );
    }

    #[test]
    fn downscale_rgba_identity() {
        // 1×1 → 1×1: 不变
        let data = vec![100, 150, 200, 255];
        let out = downscale_rgba(&data, 1, 1, 1, 1);
        assert_eq!(out, data);
    }

    #[test]
    fn downscale_rgba_2x2_to_1x1() {
        // 2×2 均匀色 → 1×1 同色
        let data = vec![100u8; 16]; // 2×2 全同色
        let out = downscale_rgba(&data, 2, 2, 1, 1);
        assert_eq!(out, vec![100; 4]);
    }

    #[test]
    fn downscale_rgba_2x2_nonuniform_to_1x1() {
        // 2×2 非均匀色 → 1×1: 中心点 (0.5,0.5) 四邻域等权插值
        // (0,0)=红 (1,0)=绿 (0,1)=蓝 (1,1)=黄
        let data = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 0, 255, // (1,1) yellow
        ];
        let out = downscale_rgba(&data, 2, 2, 1, 1);
        // R: (255+0+0+255)/4 = 127.5 →128; G: (0+255+0+255)/4 = 128; B: (0+0+255+0)/4 = 64; A: 255
        assert_eq!(out, vec![128, 128, 64, 255]);
    }
}
