//! @author 十四叔
//! @date 2026/07/18

//! 多行文本排版:按字符宽度做贪心换行。
//!
//! 本模块为纯逻辑,不依赖 GPU 或平台 API。
//! 调用方提供字符宽度测量回调(通常来自 `TextBatch` 或 `Font`)。

/// 一行文本在原文本中的字符区间与像素宽度。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// 起始字符索引(含)。
    pub start: usize,
    /// 结束字符索引(不含)。
    pub end: usize,
    /// 该行像素宽度。
    pub width: f32,
}

impl Line {
    /// 空行。
    pub const fn empty() -> Self {
        Self {
            start: 0,
            end: 0,
            width: 0.0,
        }
    }

    /// 字符数量。
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// 是否为空行。
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 对文本进行换行,返回每行的字符区间与宽度。
///
/// - 遇到 `\n` 强制换行。
/// - 当前行累积宽度超过 `max_width` 时按字符换行。
/// - 单个字符宽度即超过 `max_width` 时,该字符独占一行(避免死循环)。
/// - `max_width <= 0.0` 时不换行,仅按 `\n` 分段。
pub fn break_lines(text: &str, max_width: f32, measure: &mut dyn FnMut(char) -> f32) -> Vec<Line> {
    if text.is_empty() {
        return vec![Line::empty()];
    }

    let mut lines = Vec::new();
    let mut line_start = 0usize;
    let mut line_width = 0.0f32;
    let mut line_chars = 0usize;
    let wrap = max_width > 0.0;

    for (char_idx, ch) in text.chars().enumerate() {
        if ch == '\n' {
            lines.push(Line {
                start: line_start,
                end: char_idx,
                width: line_width,
            });
            line_start = char_idx + 1;
            line_width = 0.0;
            line_chars = 0;
            continue;
        }

        let char_width = measure(ch);

        if wrap && line_chars > 0 && line_width + char_width > max_width {
            // 当前行已满,在插入该字符前换行。
            lines.push(Line {
                start: line_start,
                end: char_idx,
                width: line_width,
            });
            line_start = char_idx;
            line_width = char_width;
            line_chars = 1;
        } else {
            // 继续当前行(即使单个字符超宽,也先让它占一行)。
            line_width += char_width;
            line_chars += 1;
        }
    }

    // 最后一行。三种情形需要落一行:
    // - 仍有未结束的内容(行已开启但循环未关闭)
    // - 空文本(必须有一行占位)
    // - 文本以 '\n' 结尾(光标需要落在换行后的新行,故追加空行占位)
    let has_unfinished = line_start < text.chars().count();
    let ends_with_newline = text.ends_with('\n');
    if has_unfinished || lines.is_empty() || ends_with_newline {
        // 占位空行(start == end, width = 0)用于文本以 '\n' 结尾或全空场景;
        // 否则取循环结束时的累计 end/width。
        let (end, width) = if has_unfinished {
            (text.chars().count(), line_width)
        } else {
            (line_start, 0.0)
        };
        lines.push(Line {
            start: line_start,
            end,
            width,
        });
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 固定宽度字体测量器:每个字符宽 10.0,换行符不会被调用。
    fn fixed_width(ch: char) -> f32 {
        let _ = ch;
        10.0
    }

    #[test]
    fn empty_text_yields_one_empty_line() {
        let lines = break_lines("", 100.0, &mut fixed_width);
        assert_eq!(lines, vec![Line::empty()]);
    }

    #[test]
    fn explicit_newline_breaks() {
        let lines = break_lines("ab\ncd", 100.0, &mut fixed_width);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, 2);
        assert_eq!(lines[0].width, 20.0);
        assert_eq!(lines[1].start, 3);
        assert_eq!(lines[1].end, 5);
        assert_eq!(lines[1].width, 20.0);
    }

    #[test]
    fn soft_wrap_at_max_width() {
        let lines = break_lines("abcdef", 30.0, &mut fixed_width);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].end, 3);
        assert_eq!(lines[0].width, 30.0);
        assert_eq!(lines[1].start, 3);
        assert_eq!(lines[1].end, 6);
        assert_eq!(lines[1].width, 30.0);
    }

    #[test]
    fn oversized_word_occupies_own_line() {
        let lines = break_lines("abc", 25.0, &mut fixed_width);
        // 每字符 10,ab=20 不超过 25,abc=30 超过,所以 "ab" 一行,"c" 一行。
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].end, 2);
        assert_eq!(lines[0].width, 20.0);
        assert_eq!(lines[1].start, 2);
        assert_eq!(lines[1].end, 3);
        assert_eq!(lines[1].width, 10.0);
    }

    #[test]
    fn single_char_wider_than_max_width() {
        let mut measure = |ch: char| {
            let _ = ch;
            100.0
        };
        let lines = break_lines("ab", 50.0, &mut measure);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 1);
        assert_eq!(lines[1].len(), 1);
    }

    #[test]
    fn cjk_characters_wrap_individually() {
        // 模拟每个 CJK 字符宽 20.0,允许每行两个字符。
        let mut measure = |ch: char| {
            let _ = ch;
            20.0
        };
        let lines = break_lines("你好世界", 40.0, &mut measure);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 2);
        assert_eq!(lines[1].len(), 2);
        assert_eq!(lines[0].width, 40.0);
        assert_eq!(lines[1].width, 40.0);
    }

    #[test]
    fn non_positive_max_width_does_not_wrap() {
        let lines = break_lines("abcdef", 0.0, &mut fixed_width);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].end, 6);
        assert_eq!(lines[0].width, 60.0);
    }

    #[test]
    fn trailing_newline_emits_placeholder_line() {
        // 文本以 '\n' 结尾时,必须追加一行占位空行,
        // 否则光标落在末尾 '\n' 之后找不到归属行,
        // 会 fallback 到最后一行(== 第一行),造成 caret 视觉跑回首行。
        let lines = break_lines("ab\n", 100.0, &mut fixed_width);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start, 0);
        assert_eq!(lines[0].end, 2);
        assert_eq!(lines[0].width, 20.0);
        assert_eq!(lines[1].start, 3);
        assert_eq!(lines[1].end, 3);
        assert!(lines[1].is_empty());
        assert_eq!(lines[1].width, 0.0);
    }

    #[test]
    fn only_newline_yields_two_empty_lines() {
        let lines = break_lines("\n", 100.0, &mut fixed_width);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].is_empty());
        assert!(lines[1].is_empty());
    }

    #[test]
    fn trailing_newline_after_blank_line_keeps_blank_placeholder() {
        let lines = break_lines("ab\n\n", 100.0, &mut fixed_width);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].end, 2);
        assert!(lines[1].is_empty());
        assert!(lines[2].is_empty());
    }
}
