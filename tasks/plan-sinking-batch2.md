# Implementation Plan: 框架下沉第二批(TOML / 剪贴板写侧 / 主题委托宏 / 基线 helper / 小件包)

> Spec: `docs/specs/SPEC-sinking-batch2.md`(2026-10-10 用户「S1+B 批准」)。
> 来源: 三新仓复盘 `docs/intent/retro-trio-reuse-2026-10-10.md` S2/S4/S5/S6 + 小件包。
> 2026-10-10 建 plan 前跑了一轮只读侦察, 结论见下「侦察改判」节 —— 两处 spec 预设被实测推翻。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。

## Overview

五件下沉: 件1 `persist::toml` 子模块(log+dropshelf 两处手写 TOML) / 件2 剪贴板**写侧**(luoban 自填的
双路写+写后校验重试) / 件3 主题委托宏(单点成稿, dropshelf 有宏、log 手写) / 件4 `TextBatch` 垂直居中
基线 helper(全 3 仓 7 处) / 件5 小件包(`time` epoch 毫秒互转 + `danqing-encoding::decode_bytes`)。

落点: 件 1–4 进 danqing 框架; 件 5b 进兄弟 crate `danqing-encoding`(**引入链式 rev, 见风险**)。

## 侦察改判(建 plan 前的只读侦察, 2026-10-10)

| # | spec 预设 | 实测 | plan 取法 |
|---|---|---|---|
| 1 | 「读两边定**交集**」(S2) | log config.toml 落在朴素子集内; **dropshelf 超出**(字符串数组 `[...]`+转义、裸整数 `u32`、先写后建目录) | 按**并集**做: 子模块须支持 节头容忍/注释/空行/`key=value` trim/带引号字符串(剥引号+转义)/严格 bool/整数/字符串数组 |
| 2 | 「log 的 LogTheme 与 dropshelf 同款宏壳」(S5) | **log 没有宏** —— 是手写 34 个方法的 `impl Theme`(比 dropshelf 宏多显式委托 `control_height`); dropshelf 宏是模块私有 `macro_rules!` | 宏从 dropshelf 版成稿; log 迁移 = **手写 170 行 → 一行列表**(净删最多的一处) |
| 3 | 件2「读侧重复 ~15 行一并消」 | luoban trait `ClipboardIo`(读+写) 与框架 trait `ClipSource`(**只读**)不是同一抽象; 重复的只有 `new()`+arboard 读两行 | **扩框架 `ClipSource` 加写侧默认方法**(读侧天然一家), luoban 整体换用框架 trait |
| 4 | 件4「7 处同式」 | 算式逐字等价; 两处**形式**差异: log 版返回**绝对基线**(含 `y +`)、log 用 const 字号 | helper 返回**偏移**; log 迁时补 `y +` |

## Task List

### Phase 1: 框架侧(danqing 四件 + encoding 一件)

- [x] **T1: 件1 `persist::toml` 子模块(S2)** — `danqing/src/persist/toml.rs`(新建) + `persist.rs` 挂 `pub mod toml` ✅ 2026-10-10
  - 内容: 无依赖的朴素 TOML 子集 —— **读**: `TomlDoc::parse(text)` 容忍节头/`#` 注释/空行/无 `=` 行, key/value 各自 trim, 保序, 未知键忽略; 取值口 `str / bool / int / u32 / str_list / pair_i32`(均 `Option`, **None = 保留调用方旧值**, 复刻两仓「坏值不致命」语义)。**写**: 保序 `set_*` + 可选**分节**(`[theme]`) —— 保住 log 现文件形态, dropshelf 走无节平铺。
  - 语义红线: ① 引号字符串两侧剥引号 + `\"`/`\\` 转义(dropshelf 现制); ② 字符串数组 `["a","b"]` 要求括号, 逗号只在引号外切; ③ bool **严格** `true/false`。
  - 验收: 两仓既有 TOML 用例**参数化搬入并存活**(log 5 条 `round_trip_preserves_both_keys`/`legacy_file_without_view_section_still_loads`/`whitespace_free_assignment_is_honoured`/`commented_out_assignment_is_ignored`/`missing_file_yields_defaults`; dropshelf 5 条 `config_roundtrip_is_whole_file_source`/`parse_window_pos_shapes`/`config_bad_value_line_keeps_default`/`config_list_elements_escape_and_roundtrip`/`config_bad_lines_are_skipped_not_fatal`); 新增并集边界(空串/Unicode/含 `=` 的值/`[]` 空数组/尾逗号/未知节); **两仓真实 config 逐字段 round-trip 等价锁**
  - 估时: S-M | 依赖: 无

- [x] **T2: 件2 剪贴板写侧(S4, 旧账 E3 补完)** — `src/clipboard.rs` + `src/text/crlf.rs`(补 `to_lf`) + `lib.rs`/`text/mod.rs` 导出 ✅ 2026-10-10
  - 内容: `ClipSource` 加**带默认实现**的写侧方法 `set_html(&mut self, html, alt_text) -> Result<(), _>`(默认返 `Unsupported`, 不破既有实现者); `SystemClipSource` 落 arboard 实现(一次调用同时写 CF_HTML + 纯文本备选); 自由函数 `copy_html_verified(src, html, alt_text, attempts)`(写→读回→归一后**字符串相等**校验→不符/写空/失败均重试, 耗尽返 Err **绝不静默**); 归一出口 `normalize_clip_text`(**复用框架既有行尾归一**, 见下注)
  - 语义红线: **测试严禁碰真剪贴板**(2026-09-09 幻影粘贴教训) —— 全走注入假源
  - 验收: luoban 6 条测试参数化搬入并存活(首试成功/写失败重试成/耗尽报错/读回空/读回不符/CRLF 归一); 框架内自建内存假源
  - 估时: S | 依赖: 无
  - 注: build 时先确认框架既有行尾归一 API 名(复盘记 `to_crlf` 已落 `text/`), 有则复用、无则在本件内联一行并记

- [ ] **T3: 件3 主题委托宏(S5)** — `danqing/src/theme.rs` 导出宏(候选名 `delegate_theme!`)
  - 内容: `#[macro_export] macro_rules! delegate_theme` —— 按 `name -> Type;` 列表生成 `match self { Light => LightTheme.$m(), Dark => DarkTheme.$m() }`(成稿 = dropshelf `app_theme.rs:28-37` 版)
  - 语义红线: 委托面**只列各产品自己要的**方法; `source_palette` / `font_size_display` / `control_height` 有 trait 默认实现, 列与不列都合法(列则显式委托同一值)
  - 验收: 宏展开编译测试 + 双主题逐 token 抽查(dropshelf `delegation_is_verbatim` 形态); 33 个必填方法全委托
  - 估时: XS | 依赖: 无

- [ ] **T4: 件4 `TextBatch::vcenter_baseline`(S6)** — `danqing/src/render/text.rs`
  - 内容: `pub fn vcenter_baseline(&self, box_h: f32, px: f32) -> f32` = `(box_h - self.line_height(px)) / 2.0 + self.ascent(px)`(返回**偏移**, 不含 y; 入参 f32 与 `line_height`/`ascent` 同制)
  - 验收: 数值测试(奇/偶行高两例, 手算钉死); 三仓 7 处迁移后**像素级等价**(逐处对照原式)
  - 估时: XS | 依赖: 无

- [ ] **T5: 件5a `time` epoch 毫秒互转(小件包)** — `danqing/src/time.rs`
  - 内容: `epoch_millis() -> u64`(当前时刻, 饱和) + `system_time_to_millis(SystemTime) -> i64` + `millis_to_system_time(i64) -> SystemTime`(均饱和不 panic, 成稿 = dropshelf `store.rs:257-271` 双向 + luoban `app.rs:271-276` 单向)
  - 验收: 饱和边界(负值/i64::MAX/u128 溢出); 两处调用点语义不变
  - 估时: XS | 依赖: 无
  - 注: 两处**不逐字相同**(dropshelf 双向 `SystemTime↔i64`, luoban 单向 `now→u64`) —— 三个函数覆盖两者, 迁移时各取所需

- [ ] **T6: 件5b `danqing-encoding::decode_bytes`(小件包)** — **兄弟 crate `danqing-encoding`**(独立仓)
  - 内容: `pub fn decode_bytes(raw: &[u8]) -> (String, Encoding)` —— detect → UTF-16 转码/行解码 → 剥 BOM(成稿 = luoban `encoding.rs:16-31` ~15 行组合层)
  - 验收: 五编码矩阵(UTF-8/BOM/UTF-16LE/BE/GBK)参数化搬 luoban 用例; 兄弟 crate 三件套绿
  - 估时: XS | 依赖: 无

### Checkpoint 1: 框架侧完成

- [ ] 三件套全绿(danqing 含 `--features persist`; danqing-encoding 独立); 五件各自测试锁定
- [ ] review(五轴) — `/agent-skills:code-review-and-quality`
- [ ] **push danqing + danqing-encoding**(**对外动作, 用户逐项点头**)
- [ ] 用户过目

### Phase 2: 产品迁移(框架先 push → 产品 `cargo check` 最小重解, **顺序不能反**)

- [ ] **T7: dropshelf 迁 件1/件3/件4/件5a** — `danqing-dropshelf/src/{store,app_theme,shelf_list,group_picker}.rs`
  - 内容: `store.rs` 手写 TOML(130 行)→ `persist::toml`; `app_theme.rs` 私有 `delegate!` → `delegate_theme!`; `shelf_list.rs`/`group_picker.rs` 两处 inline 基线 → `TextBatch::vcenter_baseline`; `store.rs` `time_to_ms`/`ms_to_time` → `time::*`
  - 验收: 三件套绿; `config.toml` 逐字段 round-trip 与迁移前**逐字节或逐字段等价**(含数组/转义/窗口位置); 净删可得
  - 估时: S | 依赖: T1/T3/T4/T5 + danqing push

- [ ] **T8: luoban 迁 件2/件4/件5a/件5b** — `danqing-luoban/src/{clipboard,app,encoding}.rs`
  - 内容: 删自填 `ClipboardIo` 写侧 + `copy_html_verified` + `normalize`(~45 行), 改框架件2; 读侧 `new()`+两行 arboard 一并消(复盘 ①#7); `preview.rs` 基线 → 件4; `app.rs` `epoch_millis` → 件5a; `encoding.rs` `decode_bytes` → 件5b(**经 `danqing::encoding` re-export 消费 → danqing 须先复钉 encoding rev**)
  - 验收: 三件套绿; **`FakeClip` 改实现框架 trait 后 6 条写侧锁零改动存活**; 「不丢稿」链未受影响
  - 估时: S | 依赖: T2/T4/T5/T6 + danqing push + danqing 复钉 encoding

- [ ] **T9: plainsight 迁 件4** — `danqing-plainsight/src/view/mod.rs`
  - 内容: 删仓内 `vcenter_baseline` helper, 四处调用改 `texts.vcenter_baseline(...)`(复盘快修批已先收口成单 helper, 本件换框架版)
  - 验收: 三件套绿; 47+ 测试绿; 数值等价
  - 估时: XS | 依赖: T4 + danqing push

- [ ] **T10: log 迁 件1/件3/件4(挂 v1.x 发布后)** — `danqing-log/src/{config,analysis_panel}.rs` + 新 `app_theme` 面
  - 内容: `config.rs` 手写 TOML + **整文件写出** → `persist::toml`(需在 Cargo.toml 开 `persist` feature); `impl Theme for LogTheme` 手写 **170 行 → 一行列表**(件3); `analysis_panel.rs::vcenter_base` → 件4(补 `y +`)
  - 验收: 三件套绿(521 基线); `config.toml` round-trip 等价(**节头形态保住**); 主题逐 token 等价; **行为变化进 changelog**: bool 宽松→严格(实测对现文件等价)
  - 估时: S | 依赖: T1/T3/T4 + danqing push; **排期 = v1.x 发布后**(与 S8 log 侧迁移同窗口)
  - 观察(非本件任务, 待裁): log `config.rs:96` 写侧是**裸 `fs::write` 非原子** —— 若顺势改 `persist::atomic_save` 是一次质量单兑现(dropshelf 快修批同款), 但超出「去重」范围, 先记不动

### Checkpoint 2: 全量验收

- [ ] 各仓三件套绿; 分仓分别提交, message 注明关联; 净删行数对账
- [ ] 人工过目(各仓既有测试零改动存活数)
- [ ] 复盘报告 S2/S4/S5/S6 标落地; `framework-sinking.md` 使用方式节补「下沉验收须含既有消费者迁移」闸(S8 教训, 与 S1 plan T9 同笔)

## Risks

| 风险 | 影响 | 缓解 |
|---|---|---|
| 件1 写侧若不平铺节, log 的 `config.toml` 形态被改 | 用户可见文件形变 | 写侧支持可选分节, 保住 `[theme]`/`[view]` |
| 件1 bool 语义 log 宽松 → 严格 | 手改垃圾值的容忍度 | 实测等价(垃圾→None→默认 true, 与宽松同果), 记 changelog |
| 件2 写侧测试碰真剪贴板 | 幻影粘贴(2026-09-09) | 全走注入假源, 严禁真剪贴板 |
| 件3 宏 `#[macro_export]` 命名冲突 | 编译冲突 | `delegate_theme!` 唯一命名; 只列产品要的委托面 |
| 件4 log 版返回绝对值 | 迁移错 | 迁时补 `y +`(侦察已标 file:line) |
| 件5b 兄弟 crate 链式 rev: `encoding → danqing → 产品` | rev 对不上/构建失败 | 顺序不能反: encoding 先 push → danqing `cargo check` 复钉 → push → 产品再复钉 |
| 产品仓 lock 重解踩 wgpu-hal 边 | 构建坏 | 只用 `cargo check` 驱动; 已坏时 `cargo update -p gpu-allocator` 外科修复(记忆手法) |
| log 在 v1.x 发布窗口期动 config/主题 | 影响发布 | 排期挂发布后(与 S8 log 侧同窗口), T10 单列 |

## 工作量粗估

框架侧 5 件: T1 1–2 小时(并集读写+两仓测试搬迁) / T2/T3/T4/T5/T6 各 0.5–1 小时;产品迁移 T7/T8 各 1 小时、
T9/T10 各 0.5 小时。全程 1 天量级(不含 log 排期等待)。
