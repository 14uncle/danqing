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

/// 行尾统一为 LF: `\r\n` → `\n`, 孤立 `\r` → `\n`。
///
/// [`to_crlf`] 的对偶, 用于「读回 / 比对前先归一」—— 剪贴板写后校验时两边
/// 行尾口径不同 (Windows 源多返 CRLF) 会造成假不符, 归一后比内容才准。
/// 注意不是严格逆运算: `to_crlf` 保留孤立 `\r`, 本函数把它也收成 `\n`。
pub fn to_lf(text: &str) -> String {
    if !text.contains('\r') {
        return text.to_string();
    }
    text.replace("\r\n", "\n").replace('\r', "\n")
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

    #[test]
    fn to_lf_normalizes_crlf_and_lone_cr() {
        assert_eq!(to_lf("a\r\nb"), "a\nb");
        assert_eq!(to_lf("a\rb"), "a\nb", "孤立 \\r 也收成 \\n");
        assert_eq!(to_lf("a\nb"), "a\nb", "已是 LF 则原样");
        assert_eq!(to_lf(""), "");
        assert_eq!(to_lf("无换行"), "无换行");
    }

    #[test]
    fn to_lf_undoes_to_crlf() {
        assert_eq!(to_lf(&to_crlf("行一\n行二\n")), "行一\n行二\n");
    }
}
