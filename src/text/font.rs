//! @author 十四叔
//! @date 2026/07/17

//! 字体加载:font-kit 查系统字体,内嵌 OFL 字体兜底。
//!
//! 本模块为纯逻辑(CPU),不接触 GPU;字形栅格化由 fontdue 完成。

/// 内嵌黑体字节(Noto Sans SC / 思源黑体 GB2312 子集, OFL, 位于 `assets/fonts/ofl-sans.ttf`)。
const EMBEDDED_SANS_BYTES: &[u8] = include_bytes!("../../assets/fonts/ofl-sans.ttf");

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

    /// 加载内嵌黑体(Noto Sans SC / 思源黑体 GB2312 子集, OFL)。
    ///
    /// 笔画规整的正文字体, 系统黑体不可用时的兜底。
    pub fn embedded_sans() -> Self {
        Self::from_bytes(EMBEDDED_SANS_BYTES, "embedded Noto Sans SC subset (OFL)")
            .expect("内嵌黑体必须可解析")
    }

    /// 尝试从系统加载中文字体;成功返回 Some。
    fn system_cjk() -> Option<Self> {
        let sys_start = std::time::Instant::now();
        let source = font_kit::source::SystemSource::new();
        log::debug!("SystemSource::new 耗时: {:?}", sys_start.elapsed());
        for family in SYSTEM_CJK_CANDIDATES {
            let fam_start = std::time::Instant::now();
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
                log::info!(
                    "系统字体加载成功: {family}, 总耗时 {:?}",
                    fam_start.elapsed()
                );
                return Some(font);
            }
            log::debug!("系统字体 {family} 缺少中文字形,跳过");
        }
        log::info!("未找到可用系统 CJK 字体,耗时 {:?}", sys_start.elapsed());
        None
    }

    /// 系统黑体优先, 无系统字体时回退内嵌黑体的加载策略。
    pub fn load() -> Self {
        if let Some(font) = Self::system_cjk() {
            log::info!("字体加载:使用 {}", font.source);
            return font;
        }
        log::info!("字体加载:未找到系统中文字体,使用内嵌黑体");
        // 内嵌字节由仓库控制, 解析失败等于资产损坏, 直接 panic
        Self::embedded_sans()
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
    fn embedded_sans_parses_and_covers_cjk_latin_punctuation() {
        let font = Font::embedded_sans();
        for ch in [
            '你', '好', 'A', 'z', '0', '9', '，', '。', '：', '—', '·', '+',
        ] {
            assert_ne!(
                font.inner.lookup_glyph_index(ch),
                0,
                "内嵌黑体必须覆盖 '{ch}'"
            );
        }
    }

    #[test]
    fn embedded_sans_rasterizes_cjk_glyph() {
        let font = Font::embedded_sans();
        let (metrics, bitmap) = font.inner.rasterize('你', 16.0);
        assert!(metrics.width > 0 && metrics.height > 0);
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
