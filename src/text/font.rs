//! @author 十四叔
//! @date 2026/07/17

//! 字体加载:font-kit 查系统字体,内嵌 OFL 字体兜底。
//!
//! 本模块为纯逻辑(CPU),不接触 GPU;字形栅格化由 fontdue 完成。

/// 内嵌回退字体字节(build.rs 构建期下载,仓库零二进制)。
const FALLBACK_FONT_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fallback-font.ttf"));

/// 中文系统字体候选(按优先级,覆盖 Windows/macOS/Linux)。
const SYSTEM_CJK_CANDIDATES: &[&str] = &[
    "Microsoft YaHei",
    "PingFang SC",
    "SimHei",
    "SimSun",
    "Hiragino Sans GB",
    "Noto Sans CJK SC",
    "Source Han Sans SC",
    "WenQuanYi Micro Hei",
];

/// 字体加载错误。
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    /// 字体数据解析失败。
    #[error("字体解析失败: {0}")]
    Parse(String),
}

/// 已加载的字体(fontdue 句柄 + 来源描述)。
pub struct Font {
    inner: fontdue::Font,
    source: String,
}

impl Font {
    /// 从字节数据解析字体。
    pub fn from_bytes(bytes: &[u8], source: impl Into<String>) -> Result<Self, FontError> {
        let inner = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .map_err(|e| FontError::Parse(e.to_string()))?;
        Ok(Self {
            inner,
            source: source.into(),
        })
    }

    /// 加载内嵌回退字体(ZCOOL XiaoWei,OFL)。
    pub fn fallback() -> Self {
        Self::from_bytes(FALLBACK_FONT_BYTES, "embedded ZCOOL XiaoWei (OFL)")
            .expect("内嵌回退字体必须可解析")
    }

    /// 尝试从系统加载中文字体;成功返回 Some。
    fn system_cjk() -> Option<Self> {
        let source = font_kit::source::SystemSource::new();
        for family in SYSTEM_CJK_CANDIDATES {
            let Ok(family_handle) = source.select_family_by_name(family) else {
                continue;
            };
            let Some(handle) = family_handle.fonts().first() else {
                continue;
            };
            let Ok(kit_font) = handle.load() else {
                continue;
            };
            let Some(data) = kit_font.copy_font_data() else {
                continue;
            };
            let Ok(font) = Self::from_bytes(&data, format!("system {family}")) else {
                continue;
            };
            // 必须具备中文覆盖
            if font.inner.lookup_glyph_index('你') != 0 {
                return Some(font);
            }
            log::debug!("系统字体 {family} 缺少中文字形,跳过");
        }
        None
    }

    /// 系统字体优先,失败回退内嵌字体的加载策略。
    pub fn load() -> Self {
        match Self::system_cjk() {
            Some(font) => {
                log::info!("字体加载:使用 {}", font.source);
                font
            }
            None => {
                log::info!("字体加载:未找到系统中文字体,使用内嵌回退");
                Self::fallback()
            }
        }
    }

    /// 字体来源描述(诊断用)。
    pub fn source(&self) -> &str {
        &self.source
    }

    /// 指定像素字号下的建议行高(ascent - descent + line_gap)。
    pub fn line_height(&self, px: f32) -> f32 {
        self.inner
            .horizontal_line_metrics(px)
            .map(|m| m.new_line_size)
            .unwrap_or(px * 1.2)
    }

    /// 访问内部 fontdue 字体(图集栅格化用)。
    pub(crate) fn inner(&self) -> &fontdue::Font {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_parses_and_covers_cjk() {
        let font = Font::fallback();
        assert_ne!(
            font.inner.lookup_glyph_index('你'),
            0,
            "回退字体必须覆盖中文"
        );
        assert_ne!(
            font.inner.lookup_glyph_index('A'),
            0,
            "回退字体必须覆盖拉丁"
        );
    }

    #[test]
    fn fallback_rasterizes_cjk_glyph() {
        let font = Font::fallback();
        let (metrics, bitmap) = font.inner.rasterize('你', 16.0);
        assert!(metrics.width > 0 && metrics.height > 0);
        assert_eq!(bitmap.len(), metrics.width * metrics.height);
        assert!(bitmap.iter().any(|&a| a > 0), "位图必须非空");
        assert!(metrics.advance_width > 0.0);
    }

    #[test]
    fn load_strategy_yields_cjk_font() {
        // 本机有微软雅黑则走系统路径,否则回退;两条路径都必须可用
        let font = Font::load();
        assert_ne!(font.inner.lookup_glyph_index('你'), 0);
        assert!(font.line_height(16.0) > 16.0);
    }
}
