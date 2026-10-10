//! @author 十四叔
//! @date 2026/10/10
//!
//! 朴素 TOML 子集读写 (下沉自 danqing-log `config.rs` + danqing-dropshelf `store.rs`)。
//!
//! **不引 `toml` 依赖** —— 全家风格是「够用就手写」。本子集按两仓**并集**取,
//! 覆盖它们实际用到的全部形态 (2026-10-10 只读侦察定稿):
//! 节头 `[s]` (读侧容忍、写侧可产出) / 整行 `#` 注释 / 空行 / `key = value`;
//! 值类型 = 带引号字符串 (含 `\"` `\\` 转义) / bool / 整数 / 字符串数组 `["a", "b"]`。
//!
//! 不支持 (两仓都未用, 也刻意不做): 内联注释、多行字符串、日期/浮点、嵌套表、`[[数组表]]`。
//!
//! # 读侧语义 (两仓一致, 原样复刻)
//!
//! - 逐行; `trim`; 跳过空行与 `#` 开头行; 无 `=` 的行跳过 (故 `[section]` 节头被吞掉)
//! - `split_once('=')` 取**第一个** `=`; key / value 各自 trim
//! - **未知键忽略、坏值不致命** —— 取值口一律返回 [`Option`], `None` = 保留调用方旧值
//!
//! # 写侧语义
//!
//! - 保序输出; 节头前自动空一行 (首个节除外), 复刻 log 的 `[theme]\n…\n\n[view]\n…` 形态
//! - 字符串一律加引号并转义; 数组元素逐个转义; bool / 整数裸写

use std::fmt::Write as _;

/// 一个键的原始值形态。
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    /// 带引号字符串 (引号已剥、转义已解)。
    Str(String),
    /// `true` / `false`。
    Bool(bool),
    /// 整数。
    Int(i64),
    /// 字符串数组。
    List(Vec<String>),
}

/// 解析后的 TOML 文档 (取值口)。
#[derive(Debug, Default, Clone)]
pub struct Table {
    /// 保序键值; 重复键后者覆盖前者 (与两仓逐行赋值同语义)。
    entries: Vec<(String, Value)>,
}

impl Table {
    /// 取字符串值 (仅 `"…"` 形态)。不存在或类型不符 = `None`。
    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// 取 bool 值。**兼容带引号写法** (`histogram = "false"`) —— 这是 **log 口径**
    /// (它先 `trim_matches('"')` 再比字面量)。
    ///
    /// ⚠️ **与 dropshelf 口径不同**: dropshelf 的 `parse_bool` 只认裸字面量, 不剥引号。
    /// 一个统一的取值口不可能同时对上两者, 本件取 log 向。差别只在**手改文件**:
    /// 手写 `dark = "true"` 在 dropshelf 旧读法下是「坏值 → 保留默认」, 新读法是 `Some(true)`。
    /// 两仓**自己写出**的 bool 永远是裸的, 故存量文件不受影响。
    pub fn bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            Value::Bool(b) => Some(*b),
            Value::Str(s) if s == "true" => Some(true),
            Value::Str(s) if s == "false" => Some(false),
            _ => None,
        }
    }

    /// 取整数值。**只认裸整数** (带引号不算 —— 与 dropshelf `value.parse()` 同口径)。
    pub fn int(&self, key: &str) -> Option<i64> {
        match self.get(key)? {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// 取 `u32` (非负且在范围内, 否则 `None`)。
    pub fn u32(&self, key: &str) -> Option<u32> {
        u32::try_from(self.int(key)?).ok()
    }

    /// 取字符串数组 (仅 `[…]` 形态)。不存在或非数组 = `None`。
    ///
    /// ⚠️ dropshelf 旧读法把「键在但括号不合法」也算成空数组 (即**清空**), 本件返回
    /// `None` (调用方 `unwrap_or` 保留旧值)。今日两仓从 `Config::default()` 起读、默认就是
    /// 空, 结果相同; 若将来有调用方从**非空基线**起读, 此处语义不同 —— 迁移时留意。
    pub fn str_list(&self, key: &str) -> Option<&[String]> {
        match self.get(key)? {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    /// 取键值。`parse` 在插入时就地覆盖重复键, 故 `entries` 里同键**至多一条**,
    /// 无需「末次优先」的反向查找。
    fn get(&self, key: &str) -> Option<&Value> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

/// 解析 TOML 文本为 [`Table`]。
pub fn parse(text: &str) -> Table {
    let mut entries: Vec<(String, Value)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // `[theme]` 节头、`[[t]]`、垃圾行都没有 `=` —— 跳过。
        let Some((key, raw)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let value = parse_value(raw.trim());
        match entries.iter_mut().find(|(k, _)| k == key) {
            Some(slot) => slot.1 = value,
            None => entries.push((key.to_string(), value)),
        }
    }
    Table { entries }
}

/// 单值解析。顺序有讲究: **先判引号**, 引号包着的 `"true"` / `"68"` 必须是字符串。
fn parse_value(raw: &str) -> Value {
    if let Some(inner) = unquote(raw) {
        return Value::Str(unescape(inner));
    }
    if raw.len() >= 2 && raw.starts_with('[') && raw.ends_with(']') {
        return Value::List(parse_list(&raw[1..raw.len() - 1]));
    }
    match raw {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    if let Ok(i) = raw.parse::<i64>() {
        return Value::Int(i);
    }
    // 兜底: 无引号的裸串 (两仓宽松读法都允许) —— 取用时类型不符自然落到 None。
    Value::Str(raw.to_string())
}

/// 两端去引号: 仅当整段被**一对** `"` 包住。
///
/// 注: log 旧读法用 `trim_matches('"')`, 两端**任意数量**引号都剥。对本子集写出的文件
/// 两者等价; 病理手改 (`mode = ""dark""`) 下 log 得 `dark`、本件得 `"dark"` —— 取更严的一对。
fn unquote(raw: &str) -> Option<&str> {
    raw.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
}

/// 字符串反转义: `\"` `\\` `\n` `\r` `\t` 五式; 其余 `\x` 原样保留 (反斜杠 + 字母)。
///
/// `\n` `\r` `\t` 是对**写侧** [`escape`] 的配套 —— 没有它, 含换行的值写出即成非法 TOML
/// 且读回丢数据 (红队 2026-10-10 实跑: 随机对拍 20000 组 5098 组不等)。
/// 这三式与**真 TOML 基本串**同义, 故手写 `"C:\temp"` 也会被读成 tab —— 与真解析器一致,
/// 要字面反斜杠必须写成 `\\` (写侧本来就这么产出)。
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// 数组元素解析: 逗号只在引号外切分; `\` 转义走 [`unescape`] 那五式 (扫描期只做配对,
/// 不提前解释, 免得 `\"` 被当成引号翻转)。
///
/// **与 dropshelf 原实现的两处有意加严**: ① 原版对每个元素一律 `trim()`, 于是
/// `["  padded  "]` 读回变 `padded` (引号内空白被吃掉) —— 本件带引号的元素保留原样;
/// ② 原版把空串元素也丢掉, 本件保留显式写出的 `""`。两处都是为让「写出→读回逐字相等」
/// 这个承诺**真成立**, 且不破两仓既有用例。
fn parse_list(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut pending_bs = false;
    for ch in inner.chars() {
        if pending_bs {
            // `\` + 下一字符原样留在 cur, 交给去引号后的 unescape 处理 (且不触发引号翻转)
            cur.push('\\');
            cur.push(ch);
            pending_bs = false;
        } else if ch == '\\' && in_quotes {
            pending_bs = true;
        } else if ch == '"' {
            in_quotes = !in_quotes;
            cur.push(ch);
        } else if ch == ',' && !in_quotes {
            push_list_element(&mut out, &mut cur);
        } else {
            cur.push(ch);
        }
    }
    if pending_bs {
        cur.push('\\');
    }
    push_list_element(&mut out, &mut cur);
    out
}

/// 收一个数组元素: 整段两端 trim 后, 若被一对引号包住则剥引号 + 反转义
/// (**内层空白保留; 显式写出的空串 `""` 也保留** —— 它是一个值, 丢了就不叫逐字相等);
/// 否则按旧式裸元素原样取用, 空串丢弃 (`[a,,b]` / 尾逗号这类分隔符残留)。
fn push_list_element(out: &mut Vec<String>, cur: &mut String) {
    let raw = cur.trim();
    match unquote(raw) {
        Some(inner) => out.push(unescape(inner)),
        None if !raw.is_empty() => out.push(raw.to_string()),
        None => {}
    }
    cur.clear();
}

/// 字符串转义: `\` `"` 与三个控制字符 `\n` `\r` `\t`, 使**任何**字符串都能写进一行
/// 基本串且读回逐字相等 (红队 2026-10-10: 不转义换行会写出非法 TOML 并丢数据)。
/// 顺序有讲究: 先加倍反斜杠, 再补引号与转义 (否则会把自己插入的 `\` 再翻一倍)。
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// TOML 写侧 (保序构造)。
///
/// ```ignore
/// let mut w = Writer::new();
/// w.comment("丹青日志配置").section("theme").str("mode", "dark")
///  .section("view").bool("histogram", true);
/// let text = w.to_string();
/// ```
#[derive(Debug, Default)]
pub struct Writer {
    out: String,
    wrote_section: bool,
}

impl Writer {
    /// 空文档。
    pub fn new() -> Self {
        Self::default()
    }

    /// 写一行 `# 注释`。
    pub fn comment(&mut self, text: &str) -> &mut Self {
        let _ = writeln!(self.out, "# {text}");
        self
    }

    /// 开一个 `[section]` 节 (非首个节前自动空一行)。
    pub fn section(&mut self, name: &str) -> &mut Self {
        if self.wrote_section {
            self.out.push('\n');
        }
        let _ = writeln!(self.out, "[{name}]");
        self.wrote_section = true;
        self
    }

    /// 写带引号字符串 (自动转义)。
    pub fn str(&mut self, key: &str, value: &str) -> &mut Self {
        let _ = writeln!(self.out, "{key} = \"{}\"", escape(value));
        self
    }

    /// 写 bool (裸字面量)。
    pub fn bool(&mut self, key: &str, value: bool) -> &mut Self {
        let _ = writeln!(self.out, "{key} = {value}");
        self
    }

    /// 写整数 (裸字面量)。
    pub fn int(&mut self, key: &str, value: i64) -> &mut Self {
        let _ = writeln!(self.out, "{key} = {value}");
        self
    }

    /// 写字符串数组 (元素逐个转义)。
    pub fn str_list(&mut self, key: &str, values: &[String]) -> &mut Self {
        let items: Vec<String> = values
            .iter()
            .map(|v| format!("\"{}\"", escape(v)))
            .collect();
        let _ = writeln!(self.out, "{key} = [{}]", items.join(", "));
        self
    }
}

impl std::fmt::Display for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 写→读 round-trip (两仓真实写出物形态) ──

    #[test]
    fn log_shape_round_trips() {
        // log 的 config.toml 形态: 分节 + 空行 + 带引号字符串 + 裸 bool。
        let mut w = Writer::new();
        w.section("theme")
            .str("mode", "dark")
            .section("view")
            .bool("histogram", false);
        let text = w.to_string();
        assert_eq!(
            text,
            "[theme]\nmode = \"dark\"\n\n[view]\nhistogram = false\n"
        );

        let t = parse(&text);
        assert_eq!(t.str("mode"), Some("dark"));
        assert_eq!(t.bool("histogram"), Some(false));
    }

    #[test]
    fn dropshelf_shape_round_trips() {
        // dropshelf 形态: 头部注释 + 无节平铺 + 整数 + 字符串 + 数组。
        let mut w = Writer::new();
        w.comment("丹青暂存架配置");
        w.comment("改键说明: hotkey_vk = 虚拟键码");
        w.bool("auto_collapse", true)
            .int("hotkey_vk", 68)
            .str("window_pos", "100,-50")
            .str_list(
                "excluded_apps",
                &["a.exe".to_string(), "b, c.exe".to_string()],
            );
        let text = w.to_string();

        let t = parse(&text);
        assert_eq!(t.bool("auto_collapse"), Some(true));
        assert_eq!(t.u32("hotkey_vk"), Some(68));
        assert_eq!(t.str("window_pos"), Some("100,-50"));
        assert_eq!(
            t.str_list("excluded_apps"),
            Some(["a.exe".to_string(), "b, c.exe".to_string()].as_slice())
        );
    }

    // ── 读侧容忍 (log 既有用例) ──

    #[test]
    fn legacy_file_without_view_section_still_loads() {
        let t = parse("[theme]\nmode = \"light\"\n");
        assert_eq!(t.str("mode"), Some("light"));
        assert_eq!(t.bool("histogram"), None, "缺键 → None → 调用方保留默认");
    }

    #[test]
    fn whitespace_free_assignment_is_honoured() {
        // log review 教训: 子串判定会把 `mode="dark"` 漏掉; 按行 split 才对。
        let t = parse("mode=\"dark\"");
        assert_eq!(t.str("mode"), Some("dark"));
    }

    #[test]
    fn commented_line_is_ignored() {
        let t = parse("# mode = \"dark\"\nhistogram = true\n");
        assert_eq!(t.str("mode"), None);
        assert_eq!(t.bool("histogram"), Some(true));
    }

    #[test]
    fn empty_and_junk_documents_yield_none() {
        for text in ["", "   \n\n", "垃圾\n[section]\nno equals here\n"] {
            let t = parse(text);
            assert_eq!(t.str("mode"), None, "输入 {text:?}");
            assert_eq!(t.bool("histogram"), None);
        }
    }

    #[test]
    fn bad_value_line_keeps_caller_default() {
        // 坏 bool / 坏数组 → None (调用方 unwrap_or 旧值), 不致命。
        let t = parse("auto_collapse = maybe\nx = bareword\n");
        assert_eq!(t.bool("auto_collapse"), None);
        assert_eq!(t.int("x"), None);
        assert_eq!(t.str("x"), Some("bareword"), "裸串兜底为字符串");
    }

    // ── 类型口径 (两仓差异的等价锁) ──

    #[test]
    fn quoted_bool_is_accepted() {
        // log 读侧先剥引号再比 → 带引号的 false 也算 false; 框架须同.
        assert_eq!(
            parse("histogram = \"false\"").bool("histogram"),
            Some(false)
        );
        assert_eq!(parse("histogram = \"true\"").bool("histogram"), Some(true));
    }

    #[test]
    fn quoted_int_is_not_int() {
        // dropshelf 用 value.parse() 且**不**剥引号 → 带引号整数不算数。
        assert_eq!(parse("hotkey_vk = \"68\"").int("hotkey_vk"), None);
        assert_eq!(parse("hotkey_vk = 68").u32("hotkey_vk"), Some(68));
    }

    #[test]
    fn negative_and_overflow_u32() {
        assert_eq!(parse("n = -1").u32("n"), None);
        assert_eq!(parse("n = 4294967296").u32("n"), None);
        assert_eq!(parse("n = 0").u32("n"), Some(0));
    }

    // ── 数组边界 (dropshelf 既有用例 + 扩展) ──

    #[test]
    fn list_elements_escape_and_roundtrip() {
        // 含引号 / 反斜杠 / 逗号 / 空格的元素: 写出→读回逐字相等。
        let items = vec![
            "plain".to_string(),
            "has \"quote\"".to_string(),
            "back\\slash".to_string(),
            "with, comma".to_string(),
            "  padded  ".to_string(),
        ];
        let mut w = Writer::new();
        w.str_list("xs", &items);
        assert_eq!(parse(&w.to_string()).str_list("xs"), Some(items.as_slice()));
    }

    #[test]
    fn list_edge_shapes() {
        assert_eq!(parse("xs = []").str_list("xs"), Some([].as_slice()));
        assert_eq!(
            parse("xs = [\"a\", \"b\",]").str_list("xs"),
            Some(["a".to_string(), "b".to_string()].as_slice()),
            "尾逗号不该产生空元素"
        );
        // 未闭合括号 = 非数组 → None (dropshelf: strip_prefix/suffix 不成立 → 空)
        assert_eq!(parse("xs = [\"a\"").str_list("xs"), None);
        assert_eq!(parse("xs = bare").str_list("xs"), None);
        // 未加引号的元素 (旧格式容忍): 逐字保留
        assert_eq!(
            parse("xs = [a, b]").str_list("xs"),
            Some(["a".to_string(), "b".to_string()].as_slice())
        );
    }

    #[test]
    fn list_with_equals_sign_in_element() {
        let mut w = Writer::new();
        w.str_list("xs", &["a=b".to_string(), "c=d,e".to_string()]);
        assert_eq!(
            parse(&w.to_string()).str_list("xs"),
            Some(["a=b".to_string(), "c=d,e".to_string()].as_slice())
        );
    }

    // ── 转义强度 (红队 2026-10-10: F1 换行 / F2 空串 / F8 反斜杠连写) ──

    #[test]
    fn control_chars_round_trip_in_scalar_and_list() {
        // F1: 写侧不转义换行 → 写出非法 TOML 且读回丢数据 (随机对拍 20000 组 5098 组不等)。
        let v = "第一行\n第二行\r\n带\t制表".to_string();
        let mut w = Writer::new();
        w.str("k", &v);
        w.str_list("xs", &[v.clone(), "a\nb".to_string()]);
        let text = w.to_string();
        assert_eq!(
            text.lines().count(),
            2,
            "写出物必须是两行(值里不许出现真换行): {text:?}"
        );
        let t = parse(&text);
        assert_eq!(t.str("k"), Some(v.as_str()));
        assert_eq!(t.str_list("xs"), Some([v, "a\nb".to_string()].as_slice()));
    }

    #[test]
    fn backslash_runs_round_trip() {
        // F8 判别输入: 含**连续反斜杠**、反斜杠+字母、反斜杠+引号。
        // 原用例的 `\ `(反斜杠+空格) 恰好是不动点, 锁不住 `\`→`\\` 这一步。
        let vals = [
            r"C:\server\share".to_string(),
            r"\\双反斜杠\\".to_string(),
            r"\n 不是换行".to_string(),
            r#""引号" 与 \\"#.to_string(),
            "尾部反斜杠\\".to_string(),
        ];
        let mut w = Writer::new();
        w.str("p", &vals[1]);
        w.str_list("xs", &vals);
        let t = parse(&w.to_string());
        assert_eq!(t.str("p"), Some(vals[1].as_str()));
        assert_eq!(t.str_list("xs"), Some(vals.as_slice()));
    }

    #[test]
    fn empty_string_list_element_is_preserved() {
        // F2: 显式写出的空串元素是一个值, 丢了就不叫「逐字相等」。
        let mut w = Writer::new();
        w.str_list("xs", &["".to_string(), "a".to_string(), "".to_string()]);
        assert_eq!(
            parse(&w.to_string()).str_list("xs"),
            Some(["".to_string(), "a".to_string(), "".to_string()].as_slice())
        );
        // 但分隔符产出的空槽仍丢弃 (`[a,,b]` 与尾逗号)
        assert_eq!(
            parse("xs = [a,,b]").str_list("xs"),
            Some(["a".to_string(), "b".to_string()].as_slice())
        );
    }

    #[test]
    fn quoted_bool_divergence_is_log_oriented() {
        // F3: 本件 bool() 取 log 口径 (剥引号后比字面量); dropshelf 旧读法只认裸字面量。
        // 这条锁把「有意向 log 靠」钉住 —— 改口径必须改这里并同步 dropshelf 迁移验收。
        assert_eq!(parse(r#"dark = "true""#).bool("dark"), Some(true));
        // 而 int() 取 dropshelf 口径 (不剥引号)
        assert_eq!(parse(r#"n = "68""#).int("n"), None);
    }

    // ── 杂项 ──

    #[test]
    fn string_with_equals_signs_preserved() {
        // 只切第一个 `=`: 值里再有 `=` 必须原样保留。
        let t = parse("cmd = \"a=b=c\"");
        assert_eq!(t.str("cmd"), Some("a=b=c"));
    }

    #[test]
    fn unicode_and_escapes_round_trip() {
        let mut w = Writer::new();
        w.str("名字", "中文「测试」\\ \"引号\"");
        let t = parse(&w.to_string());
        assert_eq!(t.str("名字"), Some("中文「测试」\\ \"引号\""));
    }

    #[test]
    fn duplicate_key_last_wins() {
        let t = parse("a = 1\na = 2\n");
        assert_eq!(t.int("a"), Some(2));
    }

    #[test]
    fn empty_value_is_empty_string() {
        assert_eq!(parse("k = \"\"").str("k"), Some(""));
        assert_eq!(parse("k =").str("k"), Some(""));
    }

    #[test]
    fn section_header_without_equals_is_skipped() {
        let t = parse("[section]\nkey = 5\n");
        assert_eq!(t.int("key"), Some(5));
    }
}
