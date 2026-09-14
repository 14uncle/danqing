//! @author 十四叔
//! @date 2026/09/10
//!
//! 文本选区纯逻辑: token 边界 / 选区规范化 / 复制文本拼装。
//!
//! 偏移一律为「解码后行文本」的字节偏移 —— 渲染 (measure 前缀)、命中测试、
//! 复制走同一路径, GBK/Latin-1 等编码不存在文件字节 ↔ 显示字符的映射歧义。
//! 不做: 选区渲染、鼠标事件、剪贴板写入 (widget/产品层职责)。
//! 来源: danqing-log `src/selection.rs` (2026-09-09 下沉, 簇F/F1)。

/// 文本选区: 锚点 (按下处) 与光标点 (拖动当前处), 均为 (显示行, 解码行内字节偏移)。
///
/// 事件热路径只写不排序; 规范化 ([`TextSelection::ordered`]) 只在渲染/复制读取时做。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSelection {
    /// 锚点 (按下处)。
    pub anchor: (u64, usize),
    /// 光标点 (拖动当前处 / 双击词尾)。
    pub caret: (u64, usize),
}

impl TextSelection {
    pub fn new(anchor: (u64, usize), caret: (u64, usize)) -> Self {
        Self { anchor, caret }
    }

    /// 空选区 (锚点 == 光标点): 不渲染, Ctrl+C 忽略。
    pub fn is_empty(&self) -> bool {
        self.anchor == self.caret
    }

    /// 规范化为 (起, 止), 起 <= 止 (行号先比, 行内偏移后比)。
    /// 反向拖动 (caret 在 anchor 前) 在此归一, 复制内容两向一致。
    pub fn ordered(&self) -> ((u64, usize), (u64, usize)) {
        if self.anchor <= self.caret {
            (self.anchor, self.caret)
        } else {
            (self.caret, self.anchor)
        }
    }
}

/// 双击分词的字符分类 (2026-09-14 混合连接器规则)。
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cls {
    /// 空白 (Unicode 感知)。
    Space,
    /// CJK 汉字 (基本区/扩展A/兼容区) —— 逐字成词, 不连段。
    Cjk,
    /// 词字符: Unicode 字母数字 —— 含全角字母数字/假名/谚文/西里尔等。
    /// **只排除上面三个 CJK 区段** (基本区/扩展A/兼容区): 区段外如扩展 B
    /// `U+20000` 是 `is_alphanumeric()` 真, 落这里按词字符连段 (见代价清单)。
    Word,
    /// 连接符: 两侧都是词字符时并入复合词 (时间戳/IP/key=value/路径/URL 整选靠它)。
    Conn,
    /// 其余 (引号/括号/逗号/全角标点/组合符…) —— 标点段, 天然断开单词。
    Other,
}

fn cls_of(ch: char) -> Cls {
    if ch.is_whitespace() {
        return Cls::Space;
    }
    let cp = ch as u32;
    if (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
    {
        return Cls::Cjk;
    }
    if ch.is_alphanumeric() {
        return Cls::Word;
    }
    if matches!(
        ch,
        '-' | ':' | '.' | '/' | '\\' | '=' | '@' | '_' | '+' | '%'
    ) {
        return Cls::Conn;
    }
    Cls::Other
}

/// 混合连接器分词: 返回包含 `off` 的 token [start, end)。
///
/// 规则 (按归属字符的类分派):
/// - 空白 → 连续空白段 (编辑器惯例);
/// - 汉字 → 恰好这一个字。中文无空格, 整段连选不可用; **不引分词库** ——
///   jieba 级依赖 + MB 级词典对日志查看器不值, 单字 + 拖动框选兜底;
/// - 词字符 → 复合词: 词字符与「内部连接符段」交替、首尾必为词字符的最长段。
///   内部连接符段 = 紧前紧后都是词字符的连接符连续段 (**判定在段不在字符**,
///   故 `::` / `:\` / `://` 无需特例) —— 时间戳 / IPv4 / IPv6 / key=value /
///   路径 / URL 整选;
/// - 连接符 → 所处段是复合词内部段则选**整个复合词** (点在 `-` 上与点在数字上
///   同效); **引导段** (右侧是词字符、左侧是**空白或行首**) 并入右侧复合词 ——
///   绝对路径的引导 `/`、负号 `-1.5`、CLI 旗标 `--flag` 是 token 的一部分;
///   左侧是汉字/标点则**不并入** (`中文=abc` 的 `=` 不是词的一部分, 2026-09-14
///   实机回归); 尾随段 (仅左侧是词字符) 不粘连 —— 「word,」式的尾部标点
///   从不属于词; 否则按标点段;
/// - 其他标点 (含非内部连接符) → 连续标点段 —— 逗号/引号/括号断开单词
///   (`hello,world` 分得开)。
///
/// 已接受的代价 (防「改回」): `1,000,000` 千分位被逗号切开 (要 `hello,world`
/// 断开的直接推论); `foo---bar` / `a...b` (省略号夹在词间, 如 `ERROR...failed`)
/// 整选 (与 IPv6 `::` 同一条规则的两面); 组合字符 (e + 组合符) 拆开;
/// 假名/谚文按词字符连段; 扩展 B/C… 区汉字按词字符连段 (只有上述三个 CJK
/// 区段走逐字); URL 查询串在 `?` 处断开 (`?`/`&`/`#` 不是连接符 —— 它们是英文
/// 标点, 进连接符集会让散文单词粘连)。
///
/// `off` 归属字符 = 首个起点 >= off 的字符; `off` 在行尾 (== len) 归最后一个字符。
/// 偏移必落在 UTF-8 字符边界 (char_indices 扫描产生, 永不劈字符)。
pub fn token_at(line: &str, off: usize) -> (usize, usize) {
    let target = off.min(line.len());
    let mut owner = None; // (字符起点, 字符)
    for (i, ch) in line.char_indices() {
        owner = Some((i, ch));
        if i >= target {
            break;
        }
    }
    let Some((pos, ch)) = owner else {
        return (0, 0); // 空行
    };
    match cls_of(ch) {
        Cls::Space => run_over(line, pos, |c| cls_of(c) == Cls::Space),
        Cls::Cjk => (pos, pos + ch.len_utf8()),
        Cls::Word => (
            expand_compound_left(line, pos),
            expand_compound_right(line, pos + ch.len_utf8()),
        ),
        Cls::Conn => {
            // 所处连接符段 [cs, ce)
            let mut cs = pos;
            for (i, c) in line[..pos].char_indices().rev() {
                if cls_of(c) != Cls::Conn {
                    break;
                }
                cs = i;
            }
            let mut ce = pos + ch.len_utf8();
            let base = ce;
            for (i, c) in line[base..].char_indices() {
                if cls_of(c) != Cls::Conn {
                    break;
                }
                ce = base + i + c.len_utf8();
            }
            let left = line[..cs].chars().next_back();
            let left_word = left.is_some_and(|c| cls_of(c) == Cls::Word);
            let right_word = line[ce..]
                .chars()
                .next()
                .is_some_and(|c| cls_of(c) == Cls::Word);
            if left_word && right_word {
                // 内部段: 选整个复合词
                (
                    expand_compound_left(line, cs),
                    expand_compound_right(line, ce),
                )
            } else if right_word && left.is_none_or(|c| cls_of(c) == Cls::Space) {
                // 引导段 (右侧词字符, 左侧空白或行首): 并入右侧复合词 ——
                // 绝对路径/负号/CLI 旗标; 左侧是汉字/标点则不并 (中文=abc 的 =)
                (cs, expand_compound_right(line, ce))
            } else {
                // 尾随段或孤立段 → 标点段
                run_over(line, pos, |c| matches!(cls_of(c), Cls::Conn | Cls::Other))
            }
        }
        Cls::Other => run_over(line, pos, |c| matches!(cls_of(c), Cls::Conn | Cls::Other)),
    }
}

/// 同类连续段: 含 pos 归属字符 (起点), 逐字符满足 pred 的最大段。
fn run_over(line: &str, pos: usize, pred: impl Fn(char) -> bool) -> (usize, usize) {
    let mut start = pos;
    for (i, c) in line[..pos].char_indices().rev() {
        if !pred(c) {
            break;
        }
        start = i;
    }
    let mut end = pos;
    for (i, c) in line[pos..].char_indices() {
        if !pred(c) {
            break;
        }
        end = pos + i + c.len_utf8();
    }
    (start, end)
}

/// 复合词左扩: 吃词字符; 遇连接符段则看穿 —— 段左仍是词字符才一并吃掉**继续**;
/// 段左是**空白或行首** = **引导段** (绝对路径 `/`、负号、CLI 旗标), 并入即停;
/// 段左是汉字/标点 → 连接符不进选区 (`中文=abc` 双击右侧只选 `abc`)。
/// 尾随不对称: 右扩镜像里连接符段右无词字符则不并入 (「word,」尾部标点不属于词)。
fn expand_compound_left(line: &str, mut start: usize) -> usize {
    loop {
        for (i, c) in line[..start].char_indices().rev() {
            if cls_of(c) != Cls::Word {
                break;
            }
            start = i;
        }
        let mut conn_start = start;
        for (i, c) in line[..start].char_indices().rev() {
            if cls_of(c) != Cls::Conn {
                break;
            }
            conn_start = i;
        }
        if conn_start == start {
            return start; // 无连接符段
        }
        match line[..conn_start].chars().next_back() {
            Some(c) if cls_of(c) == Cls::Word => start = conn_start, // 内部段: 吃掉继续
            None => return conn_start,                               // 行首引导段: 并入即停
            Some(c) if cls_of(c) == Cls::Space => return conn_start, // 空白后引导段: 并入即停
            _ => return start,                                       // 汉字/标点后: 连接符不进选区
        }
    }
}

/// 复合词右扩: 与左扩镜像 (偏移为绝对字节, 内层循环各用固定 base)。
fn expand_compound_right(line: &str, mut end: usize) -> usize {
    loop {
        let base = end;
        for (i, c) in line[base..].char_indices() {
            if cls_of(c) != Cls::Word {
                break;
            }
            end = base + i + c.len_utf8();
        }
        let base = end;
        let mut conn_end = end;
        for (i, c) in line[base..].char_indices() {
            if cls_of(c) != Cls::Conn {
                break;
            }
            conn_end = base + i + c.len_utf8();
        }
        if conn_end == base
            || !line[conn_end..]
                .chars()
                .next()
                .is_some_and(|c| cls_of(c) == Cls::Word)
        {
            return end;
        }
        end = conn_end;
    }
}

/// 复制文本拼装: 规范化后首行取 `[start.1..]`, 末行取 `[..end.1]`, 中间整行,
/// `\n` 拼接。`line` 回调 = 显示行 → 解码行文本 (filtered/展开映射由调用方负责)。
/// 偏移超行长防御性回钳 (文件外部变更后旧选区不炸)。空选区返回空串。
pub fn copy_text(sel: &TextSelection, line: &dyn Fn(u64) -> String) -> String {
    if sel.is_empty() {
        return String::new();
    }
    let ((r0, _), (r1, _)) = sel.ordered();
    let mut out = String::new();
    for row in r0..=r1 {
        let text = line(row);
        if let Some((from, to)) = row_slice(sel, row, &text) {
            out.push_str(&text[from..to]);
        }
        if row < r1 {
            out.push('\n');
        }
    }
    out
}

/// 选区在某行的切片 `[from, to)`: 首行取锚点侧, 末行取光标侧, 中间整行;
/// 偏移超行长防御回钳到字符边界 (渲染与复制共用的唯一权威实现)。
/// row 不在选区行域内, 或切片为零宽 → None。
pub fn row_slice(sel: &TextSelection, row: u64, line: &str) -> Option<(usize, usize)> {
    let ((r0, c0), (r1, c1)) = sel.ordered();
    if !(r0..=r1).contains(&row) {
        return None;
    }
    let from = if row == r0 {
        floor_char_boundary(line, c0.min(line.len()))
    } else {
        0
    };
    let to = if row == r1 {
        floor_char_boundary(line, c1.min(line.len()))
    } else {
        line.len()
    };
    (from < to).then_some((from, to))
}

/// 回钳到 <= off 的最近字符边界 (Rust 1.9+ `str::floor_char_boundary`
/// 尚未稳定的等效实现, 稳定后替换)。渲染侧与复制拼装共用。
pub fn floor_char_boundary(s: &str, off: usize) -> usize {
    let mut i = off.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- token_at (混合连接器规则) ----

    #[test]
    fn token_covers_timestamp() {
        // 时间戳整选: -/T/: 都是内部连接符 (词中点与连接符上点同效)
        let line = "2026-09-08T12:34:56.789Z INFO boot";
        assert_eq!(token_at(line, 0), (0, 24));
        assert_eq!(token_at(line, 10), (0, 24));
        assert_eq!(token_at(line, 23), (0, 24));
        assert_eq!(token_at(line, 4), (0, 24), "点在连接符 - 上 = 整个复合词");
        // 带时区: + 与 : 同样内部化
        let tz = "2026-09-08T12:34:56+08:00 x";
        assert_eq!(token_at(tz, 0), (0, 25));
        assert_eq!(token_at(tz, 19), (0, 25), "点在 + 上同样整选");
    }

    #[test]
    fn token_covers_ip_keyvalue_path_url_email() {
        // IPv4 (点连接符上同样整选)
        let ip = "from 10.0.0.1 ok";
        assert_eq!(token_at(ip, 5), (5, 13));
        assert_eq!(token_at(ip, 7), (5, 13), "点在 . 上 = 整个 IPv4");
        // IPv6: :: 整段内部化 (连接符段判定在段不在字符)
        let v6 = "addr 2001:db8::1;";
        assert_eq!(token_at(v6, 5), (5, 16));
        assert_eq!(token_at(v6, 13), (5, 16), "点在 :: 上 = 整个 IPv6");
        // key=value
        let kv = "x level=ERROR y";
        assert_eq!(token_at(kv, 2), (2, 13));
        assert_eq!(token_at(kv, 7), (2, 13), "点在 = 上 = 整个 key=value");
        // Unix 路径 (引导 / 并入) / Windows 路径 (`:\` 是一段连接符)
        assert_eq!(token_at("p /var/log/app.log", 2), (2, 18));
        assert_eq!(token_at(r"p C:\Users\gwhun", 2), (2, 16));
        // URL (查询串在 ? 处断开 —— ? 不是连接符, 见模块文档「已接受的代价」)
        assert_eq!(token_at("u https://example.com/x", 2), (2, 23));
        let q = "u https://example.com/x?a=b"; // u=0 空=1 h=2 … x=22 ?=23 a=24 ==25 b=26
        assert_eq!(token_at(q, 2), (2, 23), "主体在 ? 前截止");
        assert_eq!(token_at(q, 23), (23, 24), "? 自成标点段");
        assert_eq!(token_at(q, 24), (24, 27), "查询参数 a=b 仍是复合词");
        // & 与 # 同为英文标点: 同样断开 (不因「像分隔符」就进连接符集)
        let amp = "p a=1&b=2";
        assert_eq!(token_at(amp, 2), (2, 5));
        assert_eq!(token_at(amp, 5), (5, 6));
        assert_eq!(token_at(amp, 6), (6, 9));
        assert_eq!(token_at("p x#y", 2), (2, 3));
        // email
        assert_eq!(token_at("m user@host.com", 2), (2, 15));
    }

    #[test]
    fn token_english_words_split_at_punctuation() {
        // 用户核心诉求: hello,world 双击得单词 (逗号是标点段, 不是连接符)
        let line = "hello,world";
        assert_eq!(token_at(line, 0), (0, 5));
        assert_eq!(token_at(line, 5), (5, 6), "逗号自成标点段");
        assert_eq!(token_at(line, 6), (6, 11));
        // 千分位连带切开 (已接受代价: 与 hello,world 断开是同一条规则)
        let n = "1,000,000";
        assert_eq!(token_at(n, 0), (0, 1));
        assert_eq!(token_at(n, 2), (2, 5));
        // 引号与词分离
        let q = r#"x "ERROR" y"#;
        assert_eq!(token_at(q, 2), (2, 3));
        assert_eq!(token_at(q, 3), (3, 8));
        // 括号断开; 词内下划线仍整选
        let m = "a(order_service)b";
        assert_eq!(token_at(m, 0), (0, 1));
        assert_eq!(token_at(m, 2), (2, 15), "order_service 整选 (含下划线)");
        assert_eq!(token_at(m, 1), (1, 2), "( 自成标点段");
    }

    #[test]
    fn token_cjk_selects_single_char() {
        // 中文逐字 (不引分词库; 单字 + 框选兜底)
        // 订=0..3 单=3..6 创=6..9 建=9..12 成=12..15 功=15..18
        let line = "订单创建成功";
        assert_eq!(token_at(line, 0), (0, 3));
        assert_eq!(token_at(line, 3), (3, 6));
        assert_eq!(token_at(line, 18), (15, 18), "行尾归属最后字符");
        // 混合行: 中文逐字 / 英文复合词 / 括号标点段 各归各类
        let m = "订单(order_service) 创建成功";
        assert_eq!(token_at(m, 0), (0, 3), "订 逐字");
        assert_eq!(token_at(m, 7), (7, 20), "order_service 整选 (含下划线)");
        assert_eq!(token_at(m, 6), (6, 7), "( 标点段");
        assert_eq!(token_at(m, 21), (21, 22), ") 后空白段");
        assert_eq!(token_at(m, 22), (22, 25), "创 逐字");
    }

    #[test]
    fn token_connector_runs_without_word_flanks() {
        // 装饰线: 无词字符包夹的连接符段 = 标点段整段
        assert_eq!(token_at("a --- b", 3), (2, 5));
        // 两侧是词字符则整段内部化 (与 IPv6 :: 同一条规则; 已接受代价)
        assert_eq!(token_at("foo---bar", 0), (0, 9));
        assert_eq!(token_at("foo---bar", 4), (0, 9));
        // 行尾孤立连接符不粘连
        assert_eq!(token_at("path,", 4), (4, 5));
    }

    #[test]
    fn token_leading_connector_joins() {
        // CLI 旗标: 引导 -- 并入 (点 - 上与点字母上同效)
        let f = "run --verbose now";
        assert_eq!(token_at(f, 7), (4, 13));
        assert_eq!(token_at(f, 5), (4, 13));
        // 负号: 引导 - 并入右侧 (- 左侧是空白)
        let n = "x: -1.5 ok";
        assert_eq!(token_at(n, 4), (3, 7));
        // key=value 带负值: =- 同属一个连接符段, 段两侧皆词字符 → 整体内部化
        // (段级判定的直接推论: 负值赋值整选, 对日志是想要的行为)
        let kv = "delta=-1.5 ok";
        assert_eq!(token_at(kv, 7), (0, 10));
        // 尾随连接符不并入: word- 的 - 自成标点段
        let t = "word- next";
        assert_eq!(token_at(t, 0), (0, 4));
        assert_eq!(token_at(t, 4), (4, 5));
        // 左侧是汉字 → 连接符不并入 (2026-09-14 实机回归:
        // 「中文=数字/英文, 双击等号右侧, 连同等号也被选进去」)
        let c = "中文=abc123";
        assert_eq!(token_at(c, 8), (7, 13), "双击右侧只选 abc123, 不带 =");
        assert_eq!(token_at(c, 6), (6, 7), "双击 = 本身只得 =");
        // 对称面: 右侧是汉字 → 尾随不并入
        assert_eq!(token_at("abc=中文", 1), (0, 3));
    }

    #[test]
    fn token_connector_only_lines_and_boundaries() {
        // 整行只有连接符: 两侧皆非词字符 → 标点段整段 (不并入任何复合词)
        assert_eq!(token_at("--", 0), (0, 2));
        assert_eq!(token_at("=", 0), (0, 1));
        // 连接符夹在两个汉字之间: 只选该连接符 (与「中文=abc」的右侧规则对称 ——
        // 汉字两侧都不并入, 因为判定要求词字符)
        let c = "中=中"; // 中=0..3, ==3..4, 中=4..7
        assert_eq!(token_at(c, 0), (0, 3));
        assert_eq!(token_at(c, 1), (3, 4), "腹中偏移归下一字符 (=)");
        assert_eq!(token_at(c, 4), (4, 7));
        // 空串非零偏移 / 超界偏移落在词字符或复合词上
        assert_eq!(token_at("", 5), (0, 0));
        assert_eq!(token_at("abc", 99), (0, 3), "超界归整词");
        assert_eq!(token_at("foo---bar", usize::MAX), (0, 9));
        // 省略号夹在词间 (已接受代价: 与 foo---bar 同一条规则)
        assert_eq!(token_at("a...b", 0), (0, 5));
    }

    #[test]
    fn token_middle_word() {
        let line = "2026-09-08 INFO boot";
        assert_eq!(token_at(line, 11), (11, 15)); // INFO
        assert_eq!(token_at(line, 16), (16, 20)); // boot
    }

    #[test]
    fn token_on_whitespace_selects_space_run() {
        let line = "ab   cd";
        assert_eq!(token_at(line, 3), (2, 5));
    }

    #[test]
    fn token_multibyte_never_splits_char() {
        // 中=0..3 文=3..6 空=6 日=7..10 志=10..13
        // 中文逐字后: off=1 (中字腹中) 归属 = 首个起点 >= 1 的字符 = 文 → (3,6)
        let line = "中文 日志";
        assert_eq!(token_at(line, 0), (0, 3));
        assert_eq!(token_at(line, 1), (3, 6), "腹中 off 归下一字符, 永不劈字");
        assert_eq!(token_at(line, 8), (10, 13));
        assert_eq!(token_at(line, 13), (10, 13)); // 行尾归属最后字符
    }

    #[test]
    fn token_empty_and_all_space() {
        assert_eq!(token_at("", 0), (0, 0));
        assert_eq!(token_at("   ", 1), (0, 3));
    }

    // ---- 规范化 ----

    #[test]
    fn empty_selection_detected() {
        assert!(TextSelection::new((3, 5), (3, 5)).is_empty());
        assert!(!TextSelection::new((3, 5), (3, 7)).is_empty());
        assert!(!TextSelection::new((3, 5), (4, 0)).is_empty());
    }

    #[test]
    fn ordered_normalizes_reversed_drag() {
        let sel = TextSelection::new((5, 10), (2, 3));
        assert_eq!(sel.ordered(), ((2, 3), (5, 10)));
        // 同行反向
        let sel = TextSelection::new((2, 10), (2, 3));
        assert_eq!(sel.ordered(), ((2, 3), (2, 10)));
        // 正向不变
        let sel = TextSelection::new((1, 2), (3, 4));
        assert_eq!(sel.ordered(), ((1, 2), (3, 4)));
    }

    // ---- copy_text ----

    fn lines3(row: u64) -> String {
        match row {
            0 => "aaa bbb".into(),
            1 => "ccc".into(),
            2 => "ddd eee".into(),
            _ => panic!("越界行 {row}"),
        }
    }

    #[test]
    fn copy_single_line_slice() {
        let sel = TextSelection::new((0, 4), (0, 7));
        assert_eq!(copy_text(&sel, &lines3), "bbb");
    }

    #[test]
    fn copy_cross_two_lines() {
        let sel = TextSelection::new((0, 4), (1, 2));
        assert_eq!(copy_text(&sel, &lines3), "bbb\ncc");
    }

    #[test]
    fn copy_cross_three_lines_takes_full_middle() {
        let sel = TextSelection::new((0, 2), (2, 3));
        assert_eq!(copy_text(&sel, &lines3), "a bbb\nccc\nddd");
    }

    #[test]
    fn copy_reversed_drag_equals_forward() {
        let fwd = TextSelection::new((0, 4), (2, 3));
        let rev = TextSelection::new((2, 3), (0, 4));
        assert_eq!(copy_text(&fwd, &lines3), copy_text(&rev, &lines3));
    }

    #[test]
    fn copy_empty_selection_is_empty_string() {
        let sel = TextSelection::new((1, 1), (1, 1));
        assert_eq!(copy_text(&sel, &lines3), "");
    }

    #[test]
    fn copy_clamps_offset_beyond_line_len() {
        // 文件外部变短后的陈旧选区: 回钳不炸
        let sel = TextSelection::new((1, 0), (1, 100));
        assert_eq!(copy_text(&sel, &lines3), "ccc");
    }

    #[test]
    fn copy_clamps_mid_multibyte_char() {
        // 偏移落在多字节字符中间 → 回钳到字符边界, 不劈字符 (floor 的存在理由)
        let line = |_: u64| "中文".to_string(); // 中=0..3 文=3..6
        let sel = TextSelection::new((0, 1), (0, 4));
        assert_eq!(copy_text(&sel, &line), "中");
    }

    #[test]
    fn floor_boundary_never_splits_multibyte() {
        assert_eq!(floor_char_boundary("中文", 0), 0);
        assert_eq!(floor_char_boundary("中文", 1), 0);
        assert_eq!(floor_char_boundary("中文", 2), 0);
        assert_eq!(floor_char_boundary("中文", 3), 3);
        assert_eq!(floor_char_boundary("中文", 4), 3);
        assert_eq!(floor_char_boundary("中文", 6), 6);
        assert_eq!(floor_char_boundary("中文", 99), 6);
        assert_eq!(floor_char_boundary("", 5), 0);
    }

    // ---- row_slice (渲染/复制共用) ----

    #[test]
    fn row_slice_boundaries() {
        let sel = TextSelection::new((0, 2), (2, 3));
        // 行域外 → None
        assert_eq!(row_slice(&sel, 5, "abcde"), None);
        // 首行取后缀 / 中间整行 / 末行取前缀
        assert_eq!(row_slice(&sel, 0, "abcde"), Some((2, 5)));
        assert_eq!(row_slice(&sel, 1, "abc"), Some((0, 3)));
        assert_eq!(row_slice(&sel, 2, "abcde"), Some((0, 3)));
        // 零宽 → None (行内空切不渲染)
        let z = TextSelection::new((1, 2), (1, 2));
        assert_eq!(row_slice(&z, 1, "abcde"), None);
        // 超行长回钳 + 中劈回钳
        let c = TextSelection::new((0, 1), (0, 99));
        assert_eq!(row_slice(&c, 0, "中文"), Some((0, 6)));
    }
}
