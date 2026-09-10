//! @author 十四叔
//! @date 2026/09/10
//!
//! 文本适配宽度: 尾部省略 / 亚字符左截 / 路径中部省略。
//!
//! 统一 `measure: impl FnMut(&str) -> f32` 回调注入 (不绑 `TextBatch`), 纯逻辑可单测。
//! 产品侧 `|s| texts.measure(s, px)` 适配。省略号统一 `…` (U+2026)。
//! 来源: danqing-log `view.rs` (fit_line/scroll_trim) + danqing-clipboard
//! `ui/history_list.rs` (ellipsize_tail/middle/looks_like_path), 2026-09-09 下沉 (簇F/F3)。
//!
//! 下沉时修一处源 bug: log 原二分用 `(lo+hi).div_ceil(2)` 且 `hi = mid-1`, 在多字节
//! 字符 + 极窄宽度下会死循环 (mid 回退到 lo 后 measure 仍 ≤ max_w, lo 不前进)。
//! 改为下取整 + 「mid 回退越界即终止」+ `hi = mid` (边界值), 见 [`longest_prefix`]。

const ELLIPSIS: &str = "…";

/// 二分找最长字符边界 `lo` 使 `measure(&s[..lo]) <= w`。返回 `lo`。
///
/// 全文 ≤ w 时直接返回 `s.len()` (下取整二分会在 len-1 处提前停, 不达全文长);
/// 否则不变量 `measure(&s[..lo]) <= w < measure(&s[..hi])`, `mid = lo+(hi-lo)/2`
/// 下取整, 回退到字符边界后若 `m <= lo` 提前终止 (杜绝 lo 不前进的死循环)。
/// `hi = m` 而非 `m-1`, 因 m 是边界且 measure 超宽。
fn longest_prefix(s: &str, w: f32, measure: &mut impl FnMut(&str) -> f32) -> usize {
    if measure(s) <= w {
        return s.len();
    }
    let mut lo = 0usize;
    let mut hi = s.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let mut m = mid;
        while !s.is_char_boundary(m) {
            m -= 1;
        }
        if m <= lo {
            break;
        }
        if measure(&s[..m]) <= w {
            lo = m;
        } else {
            hi = m;
        }
    }
    lo
}

/// 尾部省略: 超宽则二分最长可容纳前缀 (字符边界对齐), 给省略号腾位。
/// 返回 (展示切片, 是否截断) —— 零分配, 渲染侧 paint 直接消费切片。
pub fn fit_line(s: &str, max_w: f32, mut measure: impl FnMut(&str) -> f32) -> (&str, bool) {
    if measure(s) <= max_w {
        return (s, false);
    }
    let mut lo = longest_prefix(s, max_w, &mut measure);
    // 给省略号腾位
    let ell = measure(ELLIPSIS);
    while lo > 0 && measure(&s[..lo]) + ell > max_w {
        lo -= 1;
        while !s.is_char_boundary(lo) {
            lo -= 1;
        }
    }
    (&s[..lo], true)
}

/// 水平滚动左截断: 内容左缘在 `w` 像素处被切断, 返回 (后缀, 亚字符偏移)。
/// 调用方把后缀画在 `x - sub` —— 亚像素平滑, 无需裁剪层。
pub fn scroll_trim(s: &str, w: f32, mut measure: impl FnMut(&str) -> f32) -> (&str, f32) {
    if w <= 0.0 {
        return (s, 0.0);
    }
    let lo = longest_prefix(s, w, &mut measure);
    let sub = w - measure(&s[..lo]);
    (&s[lo..], sub.max(0.0))
}

/// 尾部省略 (owned 便捷版): 基于 [`fit_line`] 二分, 截断时补 `…`。返回 String。
pub fn ellipsize_tail(line: &str, max_width: f32, mut measure: impl FnMut(&str) -> f32) -> String {
    let (shown, truncated) = fit_line(line, max_width, &mut measure);
    if truncated {
        let mut out = String::with_capacity(shown.len() + ELLIPSIS.len());
        out.push_str(shown);
        out.push_str(ELLIPSIS);
        out
    } else {
        shown.to_string()
    }
}

/// 是否形如路径 (含分隔符): 决定走中部省略。
pub fn looks_like_path(line: &str) -> bool {
    line.contains('\\') || line.contains('/')
}

/// 中部省略: 路径专用 —— 保留盘首与文件名, 中间补省略号; 文件名超预算退化尾部省略。
pub fn ellipsize_middle(
    line: &str,
    max_width: f32,
    mut measure: impl FnMut(&str) -> f32,
) -> String {
    if measure(line) <= max_width {
        return line.to_string();
    }
    // 尾部 = 最后一个分隔符起 (含分隔符, 让省略号落在目录与文件名之间)
    let Some(sep) = line.rfind(['\\', '/']) else {
        return ellipsize_tail(line, max_width, measure);
    };
    let tail = &line[sep..];
    let reserved = measure(ELLIPSIS) + measure(tail);
    if reserved >= max_width {
        return ellipsize_tail(line, max_width, measure);
    }
    let head_budget = max_width - reserved;
    let head_lo = longest_prefix(line, head_budget, &mut measure);
    format!("{}{}{}", &line[..head_lo], ELLIPSIS, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 等宽 measure: 每字符 (含 `…`) 宽 1, 便于断言二分边界。
    fn chars_width(s: &str) -> f32 {
        s.chars().count() as f32
    }

    // ---- fit_line ----

    #[test]
    fn fit_line_not_wide_returns_whole() {
        assert_eq!(fit_line("ab", 5.0, chars_width), ("ab", false));
    }

    #[test]
    fn fit_line_truncates_and_reserves_ellipsis() {
        // "abcdef" = 6 宽 > 4; 二分最长前缀 "abcd"(4 宽), 省略号腾位回退到 "abc"
        assert_eq!(fit_line("abcdef", 4.0, chars_width), ("abc", true));
    }

    #[test]
    fn fit_line_empty() {
        assert_eq!(fit_line("", 3.0, chars_width), ("", false));
    }

    #[test]
    fn fit_line_multibyte_never_splits_char() {
        // "中文" = 2 宽 > 1 → 最长前缀 "中"(1 宽), 省略号腾位回退到 ""
        assert_eq!(fit_line("中文", 1.0, chars_width), ("", true));
    }

    #[test]
    fn fit_line_extremely_narrow_multibyte_no_hang() {
        // 覆盖源 bug: log 原 div_ceil 二分在此死循环 (单多字节字符 + 极窄宽度)
        assert_eq!(fit_line("中文", 0.5, chars_width), ("", true));
    }

    // ---- scroll_trim ----

    #[test]
    fn scroll_trim_cuts_prefix() {
        // "ab" = 2 宽恰好切断, 亚字符偏移 0
        assert_eq!(scroll_trim("abcdef", 2.0, chars_width), ("cdef", 0.0));
    }

    #[test]
    fn scroll_trim_multibyte_aligned() {
        // "中" = 1 宽切断, 后缀 "文", 偏移 0
        assert_eq!(scroll_trim("中文", 1.0, chars_width), ("文", 0.0));
    }

    #[test]
    fn scroll_trim_nonpositive_returns_whole() {
        assert_eq!(scroll_trim("ab", 0.0, chars_width), ("ab", 0.0));
    }

    #[test]
    fn scroll_trim_whole_width_returns_empty() {
        // w 超过全文宽: 全滚出为空 (longest_prefix 前置检查直接返回全文长)
        assert_eq!(scroll_trim("abcdef", 99.0, chars_width).0, "");
    }

    // ---- ellipsize_tail ----

    #[test]
    fn ellipsize_tail_truncates() {
        assert_eq!(ellipsize_tail("abcdef", 4.0, chars_width), "abc…");
    }

    #[test]
    fn ellipsize_tail_not_wide_returns_whole() {
        assert_eq!(ellipsize_tail("ab", 5.0, chars_width), "ab");
    }

    // ---- ellipsize_middle ----

    #[test]
    fn ellipsize_middle_keeps_head_and_tail() {
        // "C:\dir\file.txt" = 15 宽; tail "\file.txt" = 9 宽, reserved 10 < 12,
        // head_budget 2 → 盘首 "C:", 结果 "C:…\file.txt" = 12 宽
        assert_eq!(
            ellipsize_middle("C:\\dir\\file.txt", 12.0, chars_width),
            "C:…\\file.txt"
        );
    }

    #[test]
    fn ellipsize_middle_no_separator_degrades_to_tail() {
        assert_eq!(ellipsize_middle("abcdef", 4.0, chars_width), "abc…");
    }

    #[test]
    fn ellipsize_middle_not_wide_returns_whole() {
        assert_eq!(ellipsize_middle("ab", 5.0, chars_width), "ab");
    }

    // ---- looks_like_path ----

    #[test]
    fn looks_like_path_detects_separators() {
        assert!(looks_like_path("C:\\a\\b.txt"));
        assert!(looks_like_path("/usr/bin/rust"));
        assert!(!looks_like_path("plain text"));
    }
}
