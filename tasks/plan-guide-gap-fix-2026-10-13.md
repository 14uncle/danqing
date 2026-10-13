# Plan: 三新仓指南差距修复(2026-10-13 扫描批次)

> spec: `danqing/docs/intent/spec-guide-gap-fix-2026-10-13.md`(已锁定, 四裁决全批)
> 本文件 = plan + 任务清单(danqing/tasks/ 带名成例, 零覆盖冲突)。

## Overview

danqing 加 `LogoKind::Dropshelf`/`Luoban` 两分支(几何照 SVG 手工同步+几何锁,  Plainsight 成例),
一次 commit+push(已授权) → dropshelf/luoban 各升一次 rev、**补 TitleBar 组件**进视图根并双接线
+ 接线锁; dropshelf 顺手收口手搓(config_dir→persist 砍 dirs、自绘件色板全换既有 Theme token,
零新增); 杂项(luoban 删 .cargo 残留 / plainsight gitignore 三行 / 根 CLAUDE.md 收录三仓) +
入账(dropshelf 新建 release-checklist.md, 三仓补登 P2, 两仓肉眼项记 acceptance-pending)。

## spec 勘误(plan 期实证, spec 目标不变)

spec 假设两仓「有 TitleBar 未接线」。plan 期实证: **两仓视图根均无 TitleBar 组件**
(全仓 grep 零命中; dropshelf `ui.rs:26` 与 luoban `app.rs:301` 的「标题」都是纯 Text)。
danqing 窗口层只做 `with_decorations(false)`(handler.rs:808), TitleBar 由产品视图自摆
(家族成例: pomodoro/log/plainsight 均自摆)。

后果:

1. 两仓窗口现状 = **无 logo、无窗口按钮、无拖拽区**(luoban 只剩 Alt+F4 可关 = 壳层缺陷;
   dropshelf `placement: Remember` 位置记忆却没有拖拽手段 = 残疾)。
2. G1 从「接线」升级为「**补 TitleBar 组件 + 接线**」, spec 目标(标题栏画自家 logo)不变。
3. dropshelf 面板形制(三键取舍)上开放问题, 随本 plan 一并批。

## Architecture Decisions

- **LogoKind 几何**: 纯轴对齐圆角矩形即可(两 SVG 均无对角线/圆环), `d(v) = s * (v/256.0)`
  归一化照 Plainsight 分支成例(title_bar.rs:1016-1066)。
  - Dropshelf: 层板×2 + 在架件×4 + 朱砂外挂件×1 = 7 rect; **无框无玻璃内填**(架无围框 =
    「从任何地方来到任何地方去」, 与 Log/Plainsight 视窗剪影刻意不同, 注释写明)。
  - Luoban: 玉色框+玻璃内填(竖版纸张, inset 20/256 描边手法同 Log) + 排版行×3 +
    朱砂印章 + 白文内块 = 7 rect。
- **几何锁测试**: 照 `plainsight_logo_paints_gutter_and_aligned_segments`(title_bar.rs:2083)
  手法: paint 进 RectBatch → 按色过滤实例 → 断言数量/位置/对齐。
- **色板映射**(裁决②落实 = **零新增 token**, 全换既有):

  | dropshelf 硬编码 | 现值 | 换用 |
  |---|---|---|
  | `text_primary: BLACK` | 黑 | `t.text_primary()` |
  | `text_secondary: rgb(0.4,0.4,0.42)` | 灰 | `t.text_secondary()` |
  | `sel_bg: rgb(0.85,0.9,0.95)` | 淡蓝 | `t.selection()` |
  | `hover_bg: rgb(0.92,0.92,0.92)` | 淡灰 | `t.border()` (TitleBar 按钮 hover 底成例; 违和再裁 `surface_variant()`) |
  | `danger: rgb(0.8,0.2,0.2)` | 红 | `t.danger()` |
  | `name_px 14 / time_px 12` | | `t.font_size_body()` / `t.font_size_small()` |
  | `rad 4.0` | | `t.radius_sm()` |

  注: 框架无「列表行 hover 底」权威成例(Button hover 走亮度算法非 token, RowList 不在框架)——
  `border()` 是唯一既有 hover 底色语义。走 token = 接受主题色, 色值变化由肉眼验收兜。
- **组件 token 化手法**: `ShelfList::new`/`GroupPicker::new` 保留(= `themed(&LightTheme)` 兜底),
  加 `themed(theme, ...)` 注入; `ui.rs:32`/`455` 构造点手里有 `t: &AppTheme`(main_column 参数),
  改调 themed——照 danqing CLAUDE.md 小组件范式。
- **config_dir 替换**: `persist::config_dir("danqing-dropshelf")` 自动建目录(persist.rs:58),
  返回 `Option` 与现 `dirs::config_dir()` 同形, `save_paste_image` 内 `.join("paste")` 层不变;
  改后 `dirs` crate 零引用, Cargo.toml 整砍。
- **TitleBar 接线**: 照 plainsight 成例(`app/mod.rs:56,67,437`)——构造处 `.logo_kind(title_logo_kind())`,
  `title_logo_kind()` 提纯纯函数, 接线锁双断言(== 自家 && != Default)。
- **rev 跨度**(9c25929..新HEAD): 含 atomic_save 三强(0122288+bf00649, 红队质量强化) +
  Plainsight logo(04ea2d9) + 本批新提交。纯增量无 API 破面, 判可接受; 若 dropshelf/luoban
  persist 相关测试因「测试隔离」语义翻黄, 按新语义修测试(预期内, 不算返工)。
- **plan/todo 落盘**: 本文件单文件内含任务清单(danqing/tasks/ 带名成例), 不建 todo.md,
  不碰任何仓既有 plan/todo(技能红线: 零覆盖)。

## Task List

### Phase 1: danqing 框架侧(G1 上游)

- [ ] **T1** danqing: `LogoKind::Dropshelf` 分支 + 几何锁测试
  - Acceptance: paint 产出 7 矩形(层板×2/玉色件×4/朱砂件×1); 锁测试断言朱砂件挂出
    上层板右端且下沉(SVG 176,92,44,42)、玉色层板恰两条; `cargo test --lib` 绿
  - Files: `danqing/src/widget/title_bar.rs`
  - Verify: `cargo test widget::title_bar -- --exact`-级聚焦 + 全量
- [ ] **T2** danqing: `LogoKind::Luoban` 分支 + 几何锁测试
  - Acceptance: 竖版纸张框+玻璃内填+三排版行+朱砂印章+白文内块; 锁测试断言印章在
    纸内右下角、排版行恰三条左缘齐 x=82/256; 测试绿
  - Files: `danqing/src/widget/title_bar.rs`
- [ ] **T3** danqing 三件套 + commit + push(spec 批准即授权; T1+T2 一次提交, message 注明
  关联本批与两产品仓后续 lock 复钉)
  - Acceptance: origin/dev 含新 rev, 记录 rev 号备用
  - Verify: `git log origin/dev -1` + `git branch -r --contains`

### Checkpoint A(Phase 1 后)
- [ ] danqing `cargo test --lib --tests` 全绿、clippy 零警告、fmt 净、rev 已 push

### Phase 2: dropshelf(G1+G2)

- [ ] **T4** dropshelf 升 rev: `cargo check` 驱动重解(禁 `cargo update -p danqing`), lock 复钉
  - Acceptance: lock 钉 T3 新 rev; `grep 'source = "path+'` 零命中; 三件套绿
    (persist 测试若因 atomic_save 新语义翻黄 → 按新语义修, 记 plan 偏差)
  - Files: `Cargo.lock`(±受影响测试)
- [ ] **T5** dropshelf 补 TitleBar + 接线 + 锁(形制按开放问题①裁决)
  - Acceptance: `ui.rs` 主列顶端 TitleBar(替换纯 Text 标题); `.logo_kind(title_logo_kind())`;
    on_close/on_drag 接通(照 plainsight/log 成例); 锁测试双断言绿; 既有 84 测试零破
  - Files: `src/ui.rs`(或 main.rs, 以视图根实际为准)
- [ ] **T6** dropshelf `save_paste_image` 改 `persist::config_dir` + Cargo.toml 砍 `dirs`
  - Acceptance: 全仓 `grep dirs::` 零命中; Cargo.toml 无 dirs; 落盘路径不变
    (`%APPDATA%/danqing-dropshelf/paste/`); 测试绿
  - Files: `src/main.rs`, `Cargo.toml`, `Cargo.lock`
- [ ] **T7** dropshelf ShelfList/GroupPicker 色板 token 化
  - Acceptance: 两件 `themed()` 注入, ui.rs 构造点改调; 硬编码色零残留
    (`grep 'Color::rgb' src/shelf_list.rs src/group_picker.rs` 零命中);
    `main.rs:1189 clear_color` 不动(豁免在案); 测试绿
  - Files: `src/shelf_list.rs`, `src/group_picker.rs`, `src/ui.rs`

### Checkpoint B(Phase 2 后)
- [ ] dropshelf 三件套全绿、测试数 ≥84 入账、肉眼验收项记 acceptance-pending.md

### Phase 3: luoban(G1)

- [ ] **T8** luoban 升 rev: 同 T4 手法; Acceptance 同形(lock 钉新 rev、纯 git 态、三件套绿)
- [ ] **T9** luoban 补 TitleBar + `logo_name` + 接线 + 锁
  - Acceptance: `app.rs` 视图根顶端 TitleBar(替换纯 Text); 标准三键 +
    on_close→WindowAction::Close(默认 Exit 语义不变); `main.rs` WindowConfig 补
    `logo_name: "luoban"`; 锁测试双断言绿; 既有 86 测试零破
  - Files: `src/app.rs`, `src/main.rs`, `Cargo.lock`

### Checkpoint C(Phase 3 后)
- [ ] luoban 三件套全绿、测试数 ≥86 入账、肉眼验收项记 acceptance-pending.md

### Phase 4: 杂项 + 入账(G3, 无依赖可穿插)

- [ ] **T10** luoban 删 `.cargo/config.toml`(磁盘残留, 未入 git); plainsight `.gitignore`
  补 `tools/sdk-tools/` `*.secret` `*.license.key` 三行
- [ ] **T11** 根 `CLAUDE.md` 仓库地图补三行 + 路由补三条(dropshelf=第五件在制未发布 /
  luoban=A1 体裁落地层在制未发布 / plainsight=明见发布 hold 10-21)
- [ ] **T12** 入账: dropshelf 新建 `tasks/release-checklist.md`(P2 待办: E1 关于页 /
  E2 update feature / D5 截图 / A7 复用表 / F8 商店物料); luoban/plainsight 既有
  release-checklist.md 补登缺口(plainsight 关于页缺仓库地址等); dropshelf/luoban
  `acceptance-pending.md` 记标题栏肉眼验收

### Checkpoint D(全批收口)
- [ ] 对照 spec Success Criteria 1–9 逐项勾; spec 勘误节回写 spec 文件;
      各仓改动分仓 commit(联动注明关联; dropshelf/luoban/plainsight 的 commit/push
      按根约定仍等用户口令——本批已授权仅 danqing push)

## Risks and Mitigations

| 风险 | 级 | 对策 |
|---|---|---|
| rev 跨度含 atomic_save 三强, 老测试语义翻黄 | 低 | 按新语义修测试, 记 plan 偏差, 不算返工 |
| dropshelf 加标题栏改变面板外观(用户预期外) | 中 | 开放问题①随 plan 批; 肉眼验收记账兜底 |
| 色板 token 与原硬编码色值有差(视觉微变) | 低 | 走 token 即接受主题色; 肉眼验收记账 |
| 几何与 SVG 手工同步走样 | 中 | 几何锁测试 + 与 export-logo.py 产 PNG 肉眼对照 |
| 升 rev 炸 wgpu-hal(windows 边错配前科) | 低 | cargo check 驱动; 已炸则 `cargo update -p gpu-allocator` 外科修 |

## 裁决记录(2026-10-13 用户批准 plan)

1. **dropshelf 标题栏形制**: **仅关闭键**——`show_maximize(false)` + `show_minimize(false)`,
   on_close=ToggleShelf(收起语义, 与 CloseBehavior::Hide 一致), 栏高 28px 从列表区扣,
   窗高 360×420 不变。luoban 标准三键(无异议默认)。
