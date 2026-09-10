# SPEC: `text` 文本类下沉 —— 选区 / 编码 / 截断

> 来源: `docs/intent/framework-sinking.md` 簇F。
> 形态: danqing `src/text/` 下三个纯逻辑模块, 不新增 crate, 不碰 winit/wgpu。
> 政策门② 已落定 (2026-09-09, 随 SPEC-job): 非 UI 纯工具模块进框架合法。

## 产品一句话

行式文本组件的三块通用纯逻辑: 只读文本选区模型、文件编码检测/转码、文本适配宽度截断。三仓 (log/clipboard/未来产品) 反复手写, 统一下沉为 `text::selection` / `text::encoding` / `text::fit`。

## 能力地图

| 能力 | 职责 | 来源 | 消费者 |
|------|------|------|--------|
| `text::selection` | 文本选区纯逻辑: token 取词 / 选区规范化 / 复制拼装 / 字符边界回钳 | log `selection.rs` | log (迁) / 任何行式组件 |
| `text::encoding` | 编码检测 (BOM→NUL→UTF-8→GBK→Latin-1) + 行级解码 + 查询转码 | log `encoding.rs` | log (迁) / 任何开文本文件的产品 |
| `text::fit` | 文本适配宽度: 二分尾部省略 / 亚字符左截 / 路径中部省略 | log `view.rs` + clipboard `ui/history_list.rs` | log + clipboard (迁) |

## 为什么

三块都是「通用机制、非产品业务逻辑」, 且被实战验证 (有测试):

- **选区模型** (F1): danqing 的 Copy 链路 (`Event::Copy` / `Widget::selected_text` / `read_clipboard`) 已下沉, 但「喂它的选区模型」还留在产品侧 —— 任何要做只读文本选中的行式组件 (日志/终端/代码查看) 都要重写 token 边界、规范化、跨行拼装。log 的 `selection.rs` 是唯一实现 (16+4 测试), 且注释已声明「偏移=解码行字节, 渲染/命中/复制同一路径」的通用纪律。
- **编码检测/转码** (F2): 最纯候选 —— 9 测试、零 crate 依赖、非 Windows 兜底已在 (GBK 走 CP936 FFI, `#[cfg(windows)]`)。任何开用户文本文件的产品必遇 BOM/UTF-16/GBK/Latin-1 的检测与转码。
- **文本截断** (F3): 两仓各发明一份 —— log 的 `fit_line`/`scroll_trim` 绑定 `&mut TextBatch` (渲染侧二分 + 亚字符), clipboard 的 `ellipsize_tail`/`ellipsize_middle` 用 `measure` 回调 (尾部逐字符 + 路径中部)。同一件事「把文本适配到给定宽度」写了五份函数, 且 clipboard 的 `measure` 回调范式比 log 的 `&mut TextBatch` 绑定更可测、更可下沉。

## API 设计

### 模块结构

```rust
// danqing/src/text/mod.rs (追加)
pub mod selection;
pub mod encoding;
pub mod fit;

// danqing/src/lib.rs (追加)
pub use text::{selection, encoding, fit};
```

用户 `use danqing::selection::TextSelection;` / `danqing::encoding::Encoding;` / `danqing::fit::fit_line;` —— 与 `danqing::persist` 同级的模块命名空间。

### `text::selection` (文件 `text/selection.rs`)

```rust
/// 文本选区: 锚点 + 光标点, 均为 (显示行, 解码行内字节偏移)。
/// 事件热路径只写不排序; 规范化 (ordered) 只在渲染/复制读取时做。
pub struct TextSelection {
    pub anchor: (u64, usize),
    pub caret: (u64, usize),
}

impl TextSelection {
    pub fn new(anchor: (u64, usize), caret: (u64, usize)) -> Self;
    pub fn is_empty(&self) -> bool;              // 锚点 == 光标点
    pub fn ordered(&self) -> ((u64, usize), (u64, usize)); // 归一到 (起, 止)
}

/// 空白分隔 token: 返回含 `off` 的同类字符连续段 [start, end)。
/// 偏移必落在 UTF-8 字符边界 (char_indices 扫描, 永不劈字符)。
pub fn token_at(line: &str, off: usize) -> (usize, usize);

/// 复制文本拼装: 首行取 [start..], 末行取 [..end], 中间整行, `\n` 拼接。
/// `line` 回调 = 显示行 → 解码行文本 (filtered/展开映射由调用方负责)。
pub fn copy_text(sel: &TextSelection, line: &dyn Fn(u64) -> String) -> String;

/// 选区在某行的切片 [from, to) —— 渲染与复制共用的唯一权威实现。
/// row 不在行域 / 切片零宽 → None。
pub fn row_slice(sel: &TextSelection, row: u64, line: &str) -> Option<(usize, usize)>;

/// 回钳到 <= off 的最近字符边界 (str::floor_char_boundary 稳定前的等效实现)。
pub fn floor_char_boundary(s: &str, off: usize) -> usize;
```

**不下沉**: `COPY_MAX_LINES` (复制行数护栏) 是 log 的业务决策 (R3 评审 + 用户拍板 10 万行), 留在产品侧; `copy_text` 本身无行数限制, 由调用方决定是否套护栏。

### `text::encoding` (文件 `text/encoding.rs`)

```rust
pub enum Encoding { Utf8, Utf16Le, Utf16Be, Gbk, Latin1 }

impl Encoding {
    pub fn label(self) -> &'static str;   // 状态栏/基准展示
    pub fn is_utf16(self) -> bool;        // 2 字节编码 (打开时转码副本)
}

pub const SAMPLE: usize = 64 * 1024;      // 检测采样上限

/// 检测流水线: BOM → UTF-16 交替 NUL → UTF-8 合法性 → GBK 覆盖率 → Latin-1 兜底。
pub fn detect(head: &[u8]) -> Encoding;

/// UTF-16 → UTF-8 转码 (剥 BOM; 奇数尾字节丢弃)。
pub fn transcode_utf16(le: bool, raw: &[u8]) -> Vec<u8>;

/// 行级解码到 UTF-8 文本 (Utf16 打开时已转码, 到达即防御性 lossy)。
pub fn decode_line(enc: Encoding, raw: &[u8]) -> std::borrow::Cow<'_, str>;

/// 搜索/过滤查询转码到文件编码字节 (GBK 中文查询核心; 其余原样)。
pub fn encode_query(enc: Encoding, q: &str) -> Vec<u8>;
```

GBK (CP936) 编解码私有: `#[cfg(windows)]` 走 Win32 `MultiByteToWideChar`/`WideCharToMultiByte` 直通 FFI, 非 Windows 降级 lossy 保编译。**这不是窗口/渲染平台代码, 是纯数据转码**, 不违反「平台代码只住 window/ 与 render/」约定 —— 数据转码 FFI 与窗口系统调用性质不同, 且已带非 Windows stub。

### `text::fit` (文件 `text/fit.rs`)

统一采用 clipboard 的 `measure` 回调注入 (替代 log 的 `&mut TextBatch` 绑定), 纯逻辑、可单测。

```rust
/// 尾部省略: 先整行测量, 超宽则二分最长可容纳前缀 (字符边界对齐), 给省略号腾位。
/// 返回 (展示切片, 是否截断) —— 零分配, 渲染侧 paint 直接消费切片。
pub fn fit_line<'a>(s: &'a str, max_w: f32, measure: impl FnMut(&str) -> f32) -> (&'a str, bool);

/// 水平滚动左截断: 内容左缘被切断, 返回 (后缀, 亚字符偏移)。
/// 调用方把后缀画在 x - sub —— 亚像素平滑。
pub fn scroll_trim<'a>(s: &'a str, w: f32, measure: impl FnMut(&str) -> f32) -> (&'a str, f32);

/// 尾部省略 (owned 便捷版): 返回 String。内部可基于 fit_line 二分 (替代逐字符 format!)。
pub fn ellipsize_tail(line: &str, max_width: f32, measure: impl FnMut(&str) -> f32) -> String;

/// 中部省略: 路径专用 —— 保留盘首与文件名, 中间补省略号; 文件名超预算退化尾部省略。
pub fn ellipsize_middle(line: &str, max_width: f32, measure: impl FnMut(&str) -> f32) -> String;

/// 是否形如路径 (含分隔符): 决定走中部省略。
pub fn looks_like_path(line: &str) -> bool;
```

省略号字符统一 `…` (U+2026, 两仓已一致)。

## 依赖

- 零新增 crate: 全部 std (`str`/`Cow`/`char`) + GBK 走 Win32 FFI (link kernel32, 零 crate)。
- 不放 `encoding_rs` (F2 意图文档边界: GBK 零依赖 FFI, 不引重型编码库)。

## 验收标准

1. **单元测试直接搬** (行为保持判据):
   - `selection`: log 现有 16+4 测试 (token 边界/规范化/复制拼装/超长回钳/多字节不劈字符) 全搬, 零改动。
   - `encoding`: log 现有 9 测试 (BOM 变体/UTF-16 无 BOM NUL 启发/GBK 覆盖率/Latin-1 兜底/转码/CP936 roundtrip/查询转码) 全搬, 零改动。
   - `fit`: log 的 fit_line/scroll_trim 测试 + clipboard 的 ellipsize 测试, 改为注入 `measure` 回调 (简单定宽/字符宽函数), 覆盖: 不超宽不截 / 二分边界 / 省略号腾位 / 左截亚字符偏移 / 路径中部省略 / 无分隔符退化尾部。
2. **三件套**: `cargo fmt` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib --tests` 全绿 (默认 feature, 无门控 —— text 不拉依赖)。
3. **产品迁移验证** (log 先行, 行为保持):
   - log `selection.rs` → `danqing::text::selection` (删手写, COPY_MAX_LINES 留产品)
   - log `encoding.rs` → `danqing::text::encoding` (删手写)
   - log `view.rs` 的 fit_line/scroll_trim → `danqing::text::fit` (measure 闭包适配 `|s| texts.measure(s, px)`)
   - clipboard `ui/history_list.rs` 的 ellipsize_tail/middle/looks_like_path → `danqing::text::fit`
   - 既有测试零改动全绿

## Boundaries

- **Always**: 纯逻辑零依赖; 选区偏移=解码行字节 (渲染/命中/复制同一路径); 编码检测永不 panic (非法字节降级兜底); 截断字符边界对齐 (永不劈多字节字符)。
- **Ask first**: 编码库替换 (引 encoding_rs 等重型库); 选区坐标模型变更 (行+字节 → 其他)。
- **Never**: 选区渲染 / 鼠标事件 / 剪贴板写入 (那是 widget 层职责); 猜具体单字节代码页 (Latin-1 原样映射, 不猜码); 热路径分配 (fit_line/scroll_trim 保持零分配切片)。

## 开放问题

- **ellipsize_tail 与 fit_line 的重叠**: 两者语义等价 (尾部省略), 现实现一逐字符 format! (O(n²)) 一二分 (O(log n))。下沉后是否让 `ellipsize_tail` 基于 `fit_line` 实现 (二分 + to_string), 消灭逐字符版? 建议: 是 (plan 阶段定)。
- **迁移顺序**: clipboard 已终止开发 (代码活, 用户日常自用)。F3 的 clipboard 迁移是否纳入本轮, 还是仅 log 先行验证、clipboard 留待下次? 建议: log 先行, clipboard 机械替换一并做 (簇 C 先例: 终止仓也做了 scrim 迁移)。
