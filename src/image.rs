//! @author 十四叔
//! @date 2026/09/10
//!
//! 图像处理工具: RGBA 降采样、等比缩进。
//!
//! 下沉自 danqing-clipboard `history_list.rs:370-423` (2026-09-10, 小件包 S3)。
//! 纯 f32/usize 逻辑, 零依赖, 任何图片 UI 必遇。

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

#[cfg(test)]
mod tests {
    use super::*;

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
