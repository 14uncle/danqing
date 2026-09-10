# Implementation Plan: `text` 文本类下沉

> Spec: `docs/specs/SPEC-text.md`。来源: `docs/intent/framework-sinking.md` 簇F。
> 政策门② 已落定 (非 UI 纯工具模块进框架合法)。
> 开放问题已裁决 (2026-09-10 用户「按建议」): ① ellipsize_tail 基于 fit_line 二分实现 ② log 先行验证 + clipboard 机械替换一并做。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。

## Overview

三件文本纯逻辑下沉: `text::selection` (选区模型) / `text::encoding` (编码检测转码) / `text::fit` (适配宽度截断)。
**零新增 crate** (encoding 的 GBK 走 Win32 FFI 已带非 Windows stub), **无 feature gate** (不拉依赖, 默认编译)。
先框架侧纯逻辑 (搬 log/clipboard 源码 + 测试), 再 log/clipboard 迁移验证。

## Architecture Decisions

1. **落点**: `danqing/src/text/` 下三个新文件 (`selection.rs` / `encoding.rs` / `fit.rs`),
   `mod.rs` `pub mod` + `lib.rs` `pub use text::{selection, encoding, fit}` 命名空间暴露 (同 persist 级)。
2. **measure 注入**: `fit` 三件统一 `impl FnMut(&str) -> f32` 回调 (clipboard 范式),
   替代 log 的 `&mut TextBatch` 绑定 —— 纯逻辑、可单测, 产品侧 `|s| texts.measure(s, px)` 适配。
3. **ellipsize_tail 二分化**: 基于 `fit_line` 实现 (裁后补 `…`), 消灭逐字符 `format!` O(n²);
   `ellipsize_middle` 保持原算法 (路径中部省略是不同腾位模型, 不强行合一)。
4. **COPY_MAX_LINES 不下沉**: log 的复制行数护栏 (业务决策) 留产品侧, `copy_text` 本身无行数限制。
5. **encoding GBK FFI**: 数据转码非窗口/渲染平台代码, 放 `text/encoding.rs` 保持 `#[cfg(windows)]` + 非 Windows stub, 不违反平台代码驻地约定。
6. **省略号统一 `…`** (U+2026, 两仓已一致)。

## Task List

### Phase 1: 框架组件

- [x] **T1: `text::selection` 下沉** — `danqing/src/text/selection.rs` (新建) ✅ 2026-09-10
  - 内容: `TextSelection` / `token_at` / `copy_text` / `row_slice` / `floor_char_boundary`
  - 验收: 搬 log `selection.rs` 16+4 测试零改动全绿; 三件套绿
  - 估时: S | 依赖: 无

- [x] **T2: `text::encoding` 下沉** — `danqing/src/text/encoding.rs` (新建) ✅ 2026-09-10
  - 内容: `Encoding` / `detect` / `transcode_utf16` / `decode_line` / `encode_query` + GBK FFI (cfg windows + stub)
  - 验收: 搬 log `encoding.rs` 9 测试零改动全绿; 三件套绿
  - 估时: S | 依赖: 无

- [x] **T3: `text::fit` 下沉** — `danqing/src/text/fit.rs` (新建) ✅ 2026-09-10
  - 内容: `fit_line` / `scroll_trim` / `ellipsize_tail` (二分, 基于 fit_line) / `ellipsize_middle` / `looks_like_path`
  - 验收: 新写测试 (measure 注入等宽/定宽函数): 不超宽不截 / 二分边界 / 省略号腾位 / 左截亚字符偏移 / 路径中部省略 / 无分隔符退化尾部; 三件套绿
  - 估时: S-M | 依赖: 无

- [x] **T4: re-export 接线** — `danqing/src/text/mod.rs` + `danqing/src/lib.rs` ✅ 2026-09-10
  - 内容: `pub mod selection/encoding/fit` + `pub use text::{selection, encoding, fit}`
  - 验收: `cargo check` 通过, 公开 API 可达 (`use danqing::text::selection::TextSelection` 等)
  - 估时: XS | 依赖: T1-T3

### Checkpoint 1: 框架侧完成

- [x] 三件套绿 (默认 feature, 无门控); selection/encoding/fit 测试全锁定 (39 新测试: 16+9+14) ✅ 2026-09-10
- [ ] review (全模块) — 五轴
- [ ] 用户过目

### Phase 2: 产品迁移 (danqing 先 push → 产品 `cargo update -p danqing` → 提交 lock)

- [x] **T5: log `selection.rs` → `danqing::selection`** ✅ 2026-09-10 (log `34c6a77`)
  - 文件: 删 `danqing-log/src/selection.rs`, view.rs 改 import; COPY_MAX_LINES 留产品 (view.rs 常量)
  - 验收: log 测试全绿 (selection 单测搬 danqing, 剩余绿)
  - 估时: S-M | 依赖: T1-T4 + danqing push

- [x] **T6: log `encoding.rs` → `danqing::encoding`** ✅ 2026-09-10 (log `34c6a77`)
  - 文件: 删 `danqing-log/src/encoding.rs`, logfile.rs/main.rs/view.rs 改 import
  - 验收: log 测试全绿
  - 估时: S | 依赖: 同上

- [x] **T7: log `view.rs` fit_line/scroll_trim → `danqing::fit`** ✅ 2026-09-10 (log `34c6a77`)
  - 文件: `danqing-log/src/view.rs` 删两函数, 改 `|t| texts.measure(t, px)` 适配
  - 验收: log 测试全绿 (scroll_trim_prefix_math 抓出 danqing fit 二分边界 bug → 修 9d214be)
  - 估时: S | 依赖: 同上

- [x] **T8: clipboard `ui/history_list.rs` ellipsize → `danqing::fit`** ✅ 2026-09-10 (clipboard `b97023c`)
  - 文件: `danqing-clipboard/src/ui/history_list.rs` 删 ellipsize_tail/middle/looks_like_path, 改框架
  - 验收: clipboard 测试全绿 (134)
  - 估时: S | 依赖: 同上

### Checkpoint 2: 全量验收

- [x] 各仓三件套绿; 分仓分别提交, message 注明关联 ✅ 2026-09-10 (danqing 7c690ff+9d214be / log 34c6a77 / clipboard b97023c)
- [ ] 人工过目 (log 选区/编码/截断行为不变; clipboard 列表省略行为不变)
- [x] 进 review 阶段 ✅ 2026-09-10 — 五轴 REQUEST CHANGES: 1 Critical (longest_prefix 多字节字符可容纳时漏判, scroll_trim("a中b",2.0) 残留「中」) + 2 Required (混合 ASCII+多字节截断测试 / GBK FFI 平台代码豁免) + 1 Optional (ellipsize_middle head 传 &line[..sep]), 已全修 (danqing `ebeab91`)

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| selection 坐标模型 (u64, usize) 与未来行式组件不匹配 | 低 | 已在 log 实战验证; 行+字节是行式组件通用模型 |
| encoding GBK FFI 跨平台编译 | 低 | 源码已带 `#[cfg(windows)]` + 非 Windows stub, 下沉时原样保留 |
| fit measure 回调与 TextBatch::measure 的 `&mut` 语义 | 低 | `FnMut` 捕获 `&mut TextBatch`, 产品侧闭包适配 |
| ellipsize_tail 二分重构改变逐字符行为 | 中 | 二分与逐字符语义等价 (最长可容纳前缀); 搬 log/clipboard 测试兜底, 超宽退化路径显式测 |
| log 在途批次冲突 | 无 | 已核对: selection.rs/encoding.rs/view.rs 均已在 `961c03b` 等提交, 工作区仅 `.cargo/config.toml` 未提交 |

## 开放问题 (已答)

- ellipsize_tail 二分化 → 是 (T3 落地)
- clipboard 迁移纳入本轮 → 是 (T8)
