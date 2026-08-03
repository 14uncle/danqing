//! @author 十四叔
//! @date 2026/08/03

//! 星表 (Yale BSC5) 解析与星等映射 —— 纯逻辑, 无 GPU。
//!
//! 数据由 `tools/export-stars.py` 预处理: 8B 自描述头 (魔数 "DQST" + 版本 +
//! 计数, LE) + 每星 6B (x/y u16 归一化, vmag/bv u8 量化)。记录顺序即目录
//! HR 顺序 (已剔除 UV 外与缺字段项), 不落 HR 号。
//! 版本策略同 stats.rs: 未来版本拒读 (防御性返回空)。

// Task 4 接线 (纹理烘焙 + 渲染上传) 前暂允 dead_code; 接线时移除此行。
#![allow(dead_code)]

/// 内置星表二进制 (`assets/stars.bin`, 由 `tools/export-stars.py` 生成)。
pub static STARS_BIN: &[u8] = include_bytes!("../../assets/stars.bin");

const MAGIC: &[u8; 4] = b"DQST";
const VERSION: u16 = 1;
const HEADER_BYTES: usize = 8;
const RECORD_BYTES: usize = 6;

/// 星等映射的亮端参考 (天狼星量级)。
const MAG_BRIGHT: f32 = -1.5;
/// 星等映射的暗端参考 (肉眼极限)。
const MAG_FAINT: f32 = 7.0;

/// 一颗目录星 (解码后)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatalogStar {
    /// 画面 UV (0..=1, y 向下)。
    pub x: f32,
    pub y: f32,
    /// 视星等 (越小越亮)。
    pub vmag: f32,
    /// B-V 色指数 (缺测为 None)。
    pub bv: Option<f32>,
}

impl CatalogStar {
    /// 相对亮度 (0..=1], 亮星趋 1。见 [`star_brightness`]。
    pub fn brightness(&self) -> f32 {
        star_brightness(self.vmag)
    }

    /// 光点半径 (px, 1280 宽画布基准)。见 [`star_radius`]。
    pub fn radius(&self) -> f32 {
        star_radius(self.vmag)
    }

    /// RGB 染色权重 (蓝白 → 暖黄)。见 [`star_tint`]。
    pub fn tint(&self) -> (f32, f32, f32) {
        star_tint(self.bv)
    }
}

/// 解析 stars.bin 为目录星列表。
///
/// 防御策略: 魔数/版本不符 → 返回空; 头声明计数与数据实际长度取小;
/// 截断的尾部记录静默跳过, 不 panic。
pub fn decode(data: &[u8]) -> Vec<CatalogStar> {
    if data.len() < HEADER_BYTES || &data[0..4] != MAGIC {
        return Vec::new();
    }
    let version = u16::from_le_bytes([data[4], data[5]]);
    if version != VERSION {
        return Vec::new();
    }
    let declared = u16::from_le_bytes([data[6], data[7]]) as usize;
    let available = (data.len() - HEADER_BYTES) / RECORD_BYTES;
    let count = declared.min(available);
    let mut stars = Vec::with_capacity(count);
    for i in 0..count {
        let o = HEADER_BYTES + i * RECORD_BYTES;
        let x = u16::from_le_bytes([data[o], data[o + 1]]) as f32 / 65535.0;
        let y = u16::from_le_bytes([data[o + 2], data[o + 3]]) as f32 / 65535.0;
        let vmag = data[o + 4] as f32 / 27.0 - 2.0;
        let bv_raw = data[o + 5];
        let bv = if bv_raw == 0xFF {
            None
        } else {
            Some(bv_raw as f32 / 85.0 - 0.5)
        };
        stars.push(CatalogStar { x, y, vmag, bv });
    }
    stars
}

/// 星等 → 相对亮度 (0..=1]: 单调递减, 二次曲线让亮星"跳出来";
/// 0.02 地板保证极限星仍可辨 (不塌成 0)。
pub fn star_brightness(vmag: f32) -> f32 {
    let t = magnitude_t(vmag);
    0.02 + 0.98 * (1.0 - t).powi(2)
}

/// 星等 → 光点半径 (px, 1280 宽画布基准): 亮星带小光晕, 6.5 等极限星 ~1px。
pub fn star_radius(vmag: f32) -> f32 {
    let t = magnitude_t(vmag);
    0.6 + 2.0 * (1.0 - t).powf(1.5)
}

/// 星等 → 归一化位置 t ∈ [0,1] (0 = 天狼级亮端, 1 = 肉眼极限暗端)。
fn magnitude_t(vmag: f32) -> f32 {
    ((vmag - MAG_BRIGHT) / (MAG_FAINT - MAG_BRIGHT)).clamp(0.0, 1.0)
}

/// B-V 色指数 → RGB 染色权重: 冷星纯白 (蓝通道不超 1.0, 不染蓝), 暖星掉蓝绿。
/// 缺测给中性白。冷星若要偏蓝需在此加蓝增益 (Task 4 渲染后按观感定)。
pub fn star_tint(bv: Option<f32>) -> (f32, f32, f32) {
    match bv {
        None => (1.0, 1.0, 1.0),
        Some(bv) => {
            let w = ((bv + 0.4) / 2.4).clamp(0.0, 1.0);
            (1.0, 1.0 - 0.25 * w, 1.0 - 0.55 * w)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 锚点 UV 与 export-stars.py 观测姿态常量 (L_CENTER=-45, THETA=60,
    /// FOV 260x150, SHIFT_Y=-0.03) 耦合; Task 8 回填常量后须同步更新。
    const VEGA_UV: (f32, f32) = (0.605, 0.031);
    const ALTAIR_UV: (f32, f32) = (0.730, 0.191);
    const ANCHOR_UV_TOL: f32 = 0.015;
    const ANCHOR_MAG_TOL: f32 = 0.2;

    fn nearest(stars: &[CatalogStar], uv: (f32, f32)) -> Option<&CatalogStar> {
        stars
            .iter()
            .filter(|s| (s.x - uv.0).abs() < ANCHOR_UV_TOL && (s.y - uv.1).abs() < ANCHOR_UV_TOL)
            .min_by(|a, b| a.vmag.partial_cmp(&b.vmag).unwrap())
    }

    #[test]
    fn decode_embedded_count_matches_bin_layout() {
        assert_eq!(
            (STARS_BIN.len() - HEADER_BYTES) % RECORD_BYTES,
            0,
            "bin 布局应整除"
        );
        let stars = decode(STARS_BIN);
        assert_eq!(stars.len(), 6743, "与 export-stars.py 自检计数一致");
    }

    #[test]
    fn decode_rejects_bad_magic_and_future_version() {
        assert!(decode(b"").is_empty(), "空数据");
        assert!(decode(b"XXXX\x01\x00\x00\x00").is_empty(), "魔数不符");
        let mut future = STARS_BIN.to_vec();
        future[4] = 0xFF; // version = 255
        assert!(decode(&future).is_empty(), "未来版本拒读");
    }

    #[test]
    fn decode_skips_truncated_tail_without_panic() {
        // 头声明 3 颗, 实际只有 2.5 条记录 → 得 2 颗, 不 panic。
        let mut data = Vec::new();
        data.extend_from_slice(MAGIC);
        data.extend_from_slice(&VERSION.to_le_bytes());
        data.extend_from_slice(&3u16.to_le_bytes());
        data.extend_from_slice(&STARS_BIN[HEADER_BYTES..HEADER_BYTES + 2 * RECORD_BYTES + 3]);
        let stars = decode(&data);
        assert_eq!(stars.len(), 2);
    }

    #[test]
    fn anchors_vega_altair_land_with_known_magnitudes() {
        let stars = decode(STARS_BIN);
        let vega = nearest(&stars, VEGA_UV).expect("织女应在画面内");
        assert!(
            (vega.vmag - 0.03).abs() < ANCHOR_MAG_TOL,
            "织女星等 {}",
            vega.vmag
        );
        let altair = nearest(&stars, ALTAIR_UV).expect("牛郎应在画面内");
        assert!(
            (altair.vmag - 0.77).abs() < ANCHOR_MAG_TOL,
            "牛郎星等 {}",
            altair.vmag
        );
    }

    #[test]
    fn brightness_and_radius_decrease_monotonically_with_magnitude() {
        let mut prev_b = f32::MAX;
        let mut prev_r = f32::MAX;
        let mut m = MAG_BRIGHT;
        while m <= MAG_FAINT {
            let b = star_brightness(m);
            let r = star_radius(m);
            assert!(b <= prev_b && b > 0.0, "亮度应单调且为正 (mag {m})");
            assert!(r <= prev_r && r > 0.0, "半径应单调且为正 (mag {m})");
            prev_b = b;
            prev_r = r;
            m += 0.25;
        }
    }

    #[test]
    fn radius_bounds_match_spec() {
        assert!(
            (star_radius(MAG_BRIGHT) - 2.6).abs() < 0.01,
            "天狼级带小光晕 ~2.6px"
        );
        let faint = star_radius(6.5);
        assert!(
            (0.55..=1.0).contains(&faint),
            "6.5 等极限星 ~1px, 实际 {faint}"
        );
    }

    #[test]
    fn tint_warms_monotonically_with_bv() {
        assert_eq!(star_tint(None), (1.0, 1.0, 1.0), "缺测中性白");
        let (_, _, cold_b) = star_tint(Some(-0.3));
        let (_, _, warm_b) = star_tint(Some(1.8));
        assert!(warm_b < cold_b, "暖星蓝通道应更低");
        let (r, _, _) = star_tint(Some(1.8));
        assert_eq!(r, 1.0, "红通道不动, 只掉蓝绿");
    }
}
