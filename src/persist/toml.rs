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

    /// 取 bool 值。**兼容带引号写法** (`histogram = "false"`) —— log 的读侧会先剥引号
    /// 再比字面量, 为保迁移前后等价, 这里对 `"true"` / `"false"` 也认。
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
    pub fn str_list(&self, key: &str) -> Option<&[String]> {
        match self.get(key)? {
            Value::List(v) => Some(v),
            _ => None,
        }
    }

    /// 末次出现优先 (重复键后者胜)。
    fn get(&self, key: &str) -> Option<&Value> {
        self.entries
            .iter()
            .rev()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
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
fn unquote(raw: &str) -> Option<&str> {
    raw.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
}

/// 字符串反转义: 只认 `\"` 与 `\\`, 其余反斜杠原样保留 (与 dropshelf 列表口径一致)。
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
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

/// 数组元素解析: 逗号只在引号外切分; `\` 转义只认 `\"` 与 `\\`, 其余反斜杠原样保留;
/// 空元素丢弃。
///
/// **与 dropshelf 原实现的一处有意加严**: 原版对每个元素一律 `trim()`, 于是
/// `["  padded  "]` 读回变 `padded` (引号内的空白被吃掉)。本件**带引号的元素保留原样**,
/// 只对旧式无引号元素两端 trim —— 写出→读回才真正逐字相等, 且不破两仓既有用例。
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
/// (**内层空白保留**); 否则按旧式裸元素原样取用。空元素丢弃。
fn push_list_element(out: &mut Vec<String>, cur: &mut String) {
    let raw = cur.trim();
    let elem = match unquote(raw) {
        Some(inner) => unescape(inner),
        None => raw.to_string(),
    };
    if !elem.is_empty() {
        out.push(elem);
    }
    cur.clear();
}

/// 字符串转义: `\` → `\\`, `"` → `\"` (与 dropshelf `quote_str` 同口径)。
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
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
