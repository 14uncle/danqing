# Implementation Plan: 著作型旗舰十年路线图 + 里程碑 0「旗舰化第一刀」

> 依据 `docs/intent/companion-flagship.md`(2026-08-01 interview-me 确认)。
> 十年总纲 + 近 1-2 季度可执行任务;战略决策:剪贴板顺延,先做旗舰数据层。

## Overview

把"用 danqing 做的番茄钟 POC"经营成十年著作型旗舰(专注陪伴系统)。三台复利机器:引擎复利(成本侧)/ 数据复利(切换侧)/ 思想史复利(作者侧)。本计划先交付**十年路线图**,再把第一块里程碑「旗舰化第一刀」拆为 4 个可验证任务。

## 十年路线图

### 前 5 年(筑基):从 POC 到值得付费的旗舰

- [ ] **第 1-2 年 · 旗舰化**: 场景动效全量终审 / 白噪音+环境音频 / 数据层(会话记录+统计+年度报告, 数据格式为十年设计)/ 付费边界定稿 / 建造实录开写 / 剪贴板作第 2 件产品验证引擎复用
- [ ] **第 3 年 · 第一笔钱**: 旗舰订阅上线(买断+订阅双轨); 数据迁移成本成型; 建造实录 100+ 篇读者群成型
- [ ] **第 4-5 年 · 品牌成型**: 第 3 件产品(启动器/便签); "丹青出品"成可信署名; 建造史沉淀书/课程雏形; 社区 1000 真粉丝
- [ ] **验收**: 第 3 年有稳定付费; 第 5 年 ≥3 件产品共享引擎、单品成本下降 10 倍、有可出售内容雏形

### 后 5 年(变现与防御):从工具到服务

- [ ] **工具 → 服务**: 订阅制持续更新、跨设备同步、年度深度报告
- [ ] **内容变现**: 书/课程/订阅正式出版——著作落地
- [ ] **品牌防御**: 作品全集 + 读者群 + 引擎三重墙(品味抄不走 / 用户数据换不动 / 署名史编不出)
- [ ] **验收**: 第 10 年有一部能传的著作,且仍在产生收入

## 里程碑 0:旗舰化第一刀(未来 1-2 季度)

### Task A: 山/森林场景动效人工终审

- **Description:** 既有待办(2026-07-30 代码 + 门槛全绿,等待用户终审),完成旗舰视觉完整度的最后一块拼图;终审通过后收尾归档。
- **Acceptance criteria:**
  - [x] 用户人工终审山/森林动效通过 (2026-08-01 用户通过)
  - [x] 归档 `tasks/archive/` + spec 验收勾选 (T5 已勾, spec 验收 8 已注)
- **Verification:** 用户终审 + `cargo test` 全绿。
- **Dependencies:** None(用户终审)
- **Files:** `tasks/archive/{plan,todo}-pomodoro-scene-motion-mountain-forest.md`(归档)
- **Scope:** S
- **Status:** ✅ 2026-08-01 终审通过并归档。**终审修正**: 无 1-5 场景快捷键(仅 ◀/▶); 森林副层已去(单层 mist_pattern)

### Task B: 付费边界 spec

- **Description:** 定稿免费版 vs 旗舰版的分割线,写成一页 spec(`docs/specs/`)。建议:免费版 = 基础番茄钟 + 1 个场景;旗舰 = 全场景 + 数据同步 + 深度定制。**产品形态会被边界塑造,现在就定。**
- **Acceptance criteria:**
  - [x] spec 明确免费/旗舰边界与理由 (2026-08-01: `docs/specs/companion-flagship-pricing.md`)
  - [x] 用户确认 (2026-08-01: 固定篝火 / 自定义时长免费 / 买断 ¥68 + 订阅留口双轨 / 订阅展示本期不做)
- **Verification:** 用户终审。
- **Dependencies:** None
- **Files:** `docs/specs/companion-flagship-pricing.md`(新)
- **Scope:** S
- **Status:** ✅ 2026-08-01 已确认; 付费门禁与数据同步后端为遗留, 非本期

### Task C: 数据层 MVP(战略关键)

- **Description:** 会话记录 + 专注统计(本地优先)。数据格式**从第一天为十年设计**:版本化、可导出、可迁移(格式/路径/schema 演进兼容)。这是用户侧数据复利的第一笔本金。
- **Acceptance criteria:**
  - [x] 每次 Focus 会话持久化(开始/结束时间、时长、场景、轮次) (2026-08-01: `stats.rs` + `main.rs` 接线)
  - [x] 专注统计视图(今日/本周/累计,复用现有今日计数) (2026-08-01: 统计面板)
  - [x] 数据导出(明文格式,版本化),旧版本数据可加载 (2026-08-01: CSV 导出 + format_version 兼容/拒读)
  - [x] benchmark 双门槛不破 (框架 src/ 零改动, 数据层仅 pomodoro example; showcase 结构不受影响, 如需可跑 `tools/benchmark.ps1` 复核)
- **Verification:** `cargo test --example pomodoro` 全绿 (170) + 人工验统计视图。
- **Dependencies:** Task A(视觉完整), Task B(边界决定数据是否分版 — spec 明确数据层全量收集不分版, 故不阻塞)
- **Files:** `examples/pomodoro/stats.rs`(新), `examples/pomodoro/main.rs`, `examples/pomodoro/timer.rs`(`TickReport.completed_round`)
- **Scope:** L
- **Status:** ✅ 2026-08-01 完成 (fmt + clippy 零警告 + lib 236 + 集成 57 + pomodoro 170 全绿 + release 构建通过)

### Task D: 建造实录第 1-3 篇开写

- **Description:** 连载建造实录,每周一篇。不是营销,是思想史复利的第一笔本金(记录设计决策与踩坑)。
- **Acceptance criteria:**
  - [x] 第 1-3 篇草稿 (2026-08-01: `docs/chronicle/{01-why-ten-years,02-eighteen-iterations,03-data-is-the-moat}.md`)
  - [ ] 第 1-3 篇发布 (仓库外: 博客/公众号)
- **Verification:** 发布可见。
- **Dependencies:** None
- **Files:** 草稿在 `docs/chronicle/`(可回写); 发布在仓库外
- **Scope:** S
- **Status:** 三篇草稿完成(为什么十年磨著作 / 场景动效 18 次迭代 / 数据即护城河); 待用户校订后发布

## 里程碑 1:沉浸世界定位落地(2026-08-01 竞品定位校验反推)

> 依据 `docs/ideas/pomodoro-competitor-memo.md` 裁决: 定位从「番茄钟」改为「专注陪伴的沉浸世界」, 用体验层回应功能层; 旗舰版清单须兑现, ¥68 才有实物。本里程碑 = 把「旗舰」从 spec 变成实物 + 品牌出仓。**年度报告边界已裁定: 原始数据 + 基础统计免费, 深度洞察(年度报告)付费。**

### Task E: 统计增强 · 年度报告(旗舰版统计增强兑现)

- **Description:** 数据层 MVP(今日/本周/累计)→ 月度/年度深度洞察(聚焦时长、轮次、场景分布、趋势)。复用 `focus-history.json` 十年演进格式,不新造数据格式。**年度报告 = 旗舰版付费**(2026-08-01 用户裁定),原始数据 + 基础统计保持免费。
- **Acceptance criteria:**
  - [x] 年度/月度报告视图(时长/轮次/场景分布/趋势)
  - [x] 报告入口按旗舰边界门控(本期不实现真实支付,语义先行;「旗舰」角标作语义标记)
  - [x] 旧数据可读,format_version 兼容不新增分支(纯读聚合,无格式改动)
- **Verification:** `cargo test --example pomodoro` 全绿 + benchmark 双门槛不破(数据层在 example 侧)。
- **Dependencies:** 年度报告边界裁定(✅ 2026-08-01 用户裁定: 旗舰)。
- **Files:** `examples/pomodoro/stats.rs`(增强), `examples/pomodoro/main.rs`
- **Scope:** M
- **Status:** ✅ 2026-08-01 完成 (fmt + clippy 零警告 + pomodoro 196 + lib 239 + 集成 58 全绿; 人工面板核验待用户)

### Task F: 深度定制(旗舰版清单补全)

- **Description:** 默认计时方案、每场景音景开关、主题细节。让「旗舰」有实物——裁决: ¥68 全押体验层,清单必须兑现。
- **Acceptance criteria:**
  - [x] 每场景音景开关 — 用户裁定「全局环境音开关」(2026-08-02: 设置面板「环境音」行 + 开/关状态按钮, AmbientMixer `enabled` 300ms 包络平滑静音/恢复, `sound_on` 持久化, 旧 JSON 默认 true)
  - [x] 默认计时方案 (2026-08-02: 用户裁定设置面板已有专注/短休/长休步进 + 持久化 + 重置即满足, 勾掉)
  - [x] 主题细节(`src/theme.rs` token) (2026-08-02: 新增 `scrim`/`radius_xl` token, 面板遮罩与控制条胶囊改用 token, 魔法值清零)
- **Verification:** `cargo test` 全绿 (lib 241 + pomodoro 211 + 集成 8) + showcase 复用 (release 构建通过) + benchmark 双门槛 PASS (startup 850.8ms ≤1000ms / WS 182.2MB ≤360MB)。
- **Dependencies:** None
- **Files:** `examples/pomodoro/*.rs`, `src/theme.rs`
- **Scope:** M
- **Status:** ✅ 2026-08-02 完成 (F1 全局环境音开关 + F3 去魔法值; F2 用户裁定已满足)

### Task G: 建造实录发布(里程碑 0 Task D 剩余)

- **Description:** 校订 + 发布第 1-3 篇(仓库外: 博客/公众号)。思想史复利第一笔本金出仓,"沉浸世界"品牌叙事的外部可见面。
- **Acceptance criteria:**
  - [ ] 用户校订完成
  - [ ] 第 1-3 篇发布可见
- **Verification:** 发布可见。
- **Dependencies:** None(可并行于 Task E/F)
- **Files:** 仓库外(发布)
- **Scope:** S
- **Status:** ⏳ 草稿已完成,待校订发布

### Task H: 数据同步 spec(裂缝 2 前置补位)

- **Description:** 界定云端边界、同步内容、数据格式复用(格式已是十年设计)、与订阅留口的关系。**不建后端**。
- **Acceptance criteria:**
  - [x] spec 界定同步范围与格式复用方式(`docs/specs/companion-flagship-sync.md`)
  - [x] 明确与订阅留口(买断 ¥68 + 订阅双轨)的关系
- **Verification:** 用户审阅。
- **Dependencies:** None
- **Files:** `docs/specs/companion-flagship-sync.md`(新)
- **Scope:** S
- **Status:** ✅ 2026-08-02 用户裁定 Q1-Q5, spec 已确认 (`docs/specs/companion-flagship-sync.md`)

### Task I: 沉浸世界补全 —— 执行后收敛为「仅星夜」(2026-08-01 反馈收敛)

- **Description:** 里程碑 1「沉浸世界」的实物主体,原计划补 4 新场景凑齐 9(星夜/雪原/沙漠/云海,2026-08-01 interview-me 裁定)。**实际执行即收敛**: 4 场景资产生成 → 3 轮背景图反馈 → 雪原/沙漠/云海(及迭代中的瀑布/晨雾湖泊/麦田黄昏)**全部刻意删除**,仅星夜晋升为第 6 场景并完整落地。最终阵容 6 场景: 篝火/海/雨/山/森林/星夜。原则(极不重叠)保留,但执行裁决「9 场景过载,星夜已是补全的极」。
- **Acceptance criteria:**
  - [x] 星夜入 `scenes.rs`(SCENES[5], 尾部追加, 索引 0-4 不动; motion.rs 常量 0-5 不失效) — commit 8362494 起, 630a7b4 收敛后定稿
  - [x] 星夜背景图 + 调色板(护栏: 大字 ≥3:1、控件 ≥4:1, 同 export-scenes.py 规则) — 静态图去星, 星野运行时程序化(用户裁定)
  - [x] 星夜环境音(ambient.rs SCENE_AUDIO[5] → `assets/audio/starry.ogg`, `tools/export-ambient.py` 程序化合成)+ 动效包络(motion.rs `STAR_SCENE` / `starry_intensity` / `starry_base`)
  - [x] ◀/▶ 循环覆盖 6 个场景(现逻辑 `% SCENES.len()`,自动覆盖;**不做** 1-6 快捷键——Task A 终审已去)
  - [x] 星夜动效 spec — `docs/specs/pomodoro-scene-motion-starry.md`(13643cf, 329c1c9)
  - [x] 用户终审通过(星点数 188→47 用户裁定 / 独立明暗呼吸 / 夜风音效重做 b35c887, 2026-08-02 收尾)
  - [x] 免费/旗舰边界不变(免费=篝火 1 个)
- **Verification:** `cargo test --example pomodoro` 全绿 + 用户终审 + benchmark 双门槛不破。
- **Dependencies:** None(与 E/F/H 并行);场景开发范式参考 Task A 归档(山/森林终审)
- **Files:** `examples/pomodoro/scenes.rs`, `examples/pomodoro/ambient.rs`, `examples/pomodoro/motion.rs`; `assets/scenes/starry.png`, `assets/audio/starry.ogg`; `docs/specs/pomodoro-scene-motion-starry.md`
- **Scope:** M(执行时含 4 场景, 收敛后仅星夜落地)
- **Status:** ✅ 2026-08-01 收敛为 6 场景(630a7b4 删沙漠等), 星夜为第 6 场景完整落地; 2026-08-02 星夜收尾(b35c887)。**结论: 里程碑交付「星夜」一个极, 不做 9 场景**; 雪原/沙漠/云海如需重启须另立任务。

## 战略决策(已定)

- **剪贴板历史管理器顺延**: 从"第二 POC"降级为"引擎复用验证"(第 2 件产品);优先级让给旗舰数据层——它是数据复利的第一笔本金,拖得越晚,用户积累的专注历史越少。

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| 专注品类免费对手强 / 付费意愿不足 | 旗舰楔子打不进 | 结构可平移:思想史 + 品牌 + 引擎整体平移到下一个旗舰;先以专注数据层验证 |
| 数据层从第一天设计过重 | 拖慢 MVP | 只做"十年可演进"的最小结构:版本化 + 可导出 + 可迁移,不做大而全 |
| 单人带宽不足(数据层 + 连载并行) | 两个都慢 | 连载可与实现交错,优先级:数据层 > 建造实录 |

## Open Questions

1. **付费边界具体形态**: 买断 vs 订阅双轨?免费场景数?数据同步是否上云(云同步是付费点,但引入后端成本)?
2. **数据层放 example 还是上收框架**: 番茄钟专属先放 example;若剪贴板等后续产品也需要统计数据,再议上收 `src/`。
