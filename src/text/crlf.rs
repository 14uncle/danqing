//! @author 十四叔
//! @date 2026/09/10
//!
//! 剪贴板文本换行归一。
//!
//! Windows 剪贴板文本事实惯例为 CRLF; 裸 LF 在部分应用 (如旧版记事本) 换行丢失。
//! 来源: danqing-clipboard `inject.rs` (2026-09-09 下沉, 簇E/E3)。

/// Windows 剪贴板文本事实惯例为 CRLF; 裸 LF 在部分应用 (如旧版记事本) 换行丢失。
/// 写入前归一化 (Ditto 同款行为); 已含 \r\n 的片段先坍缩再统一, 不重复加 \r。
pub fn to_crlf(text: &str) -> String {
    if !text.contains('\n') {
        return text.to_string();
    }
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_crlf_normalizes_bare_lf() {
        assert_eq!(to_crlf("a\nb"), "a\r\nb");
        assert_eq!(to_crlf("行一\n行二\n"), "行一\r\n行二\r\n");
    }

    #[test]
    fn to_crlf_keeps_existing_crlf() {
        assert_eq!(to_crlf("a\r\nb"), "a\r\nb", "不重复加 \\r");
    }

    #[test]
    fn to_crlf_edge_cases() {
        assert_eq!(to_crlf(""), "");
        assert_eq!(to_crlf("无换行"), "无换行");
        assert_eq!(to_crlf("a\rb"), "a\rb", "孤立 \\r (老 Mac) 原样保留");
    }
}
