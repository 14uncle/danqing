---
name: danqing-flagship-strategy
description: "丹青十年战略:著作型旗舰→2026-08-10 pomodoro全部免费发布(练手);付费部分废弃;引擎/品牌/思想史复利仍有效,变现留给下一个产品"
metadata:
  node_type: memory
  type: project
  originSessionId: ab9650d8-95dd-4a46-8afc-9fa2108226b7
  modified: 2026-08-10T13:13:41.036Z
---

2026-08-01 interview-me 流程确认丹青十年战略「著作型旗舰」:表面是产品,实质是一部以"专注"为题材的著作——基于 danqing 的付费旗舰(专注陪伴系统)+ 十年建造史,能养人、能传世。意图落盘 `docs/intent/companion-flagship.md`,路线图落盘 `tasks/plan-flagship-roadmap.md`。

核心决策:
- **护城河排序**(个人开发者):品味/品牌 > 数据/内容存量 > 专有领域经验 > 技术本身;2026 AI 商品化"会写代码",技术作护城河在衰减,壁垒押在品味+存量。
- **否决四种通用方案**:垂直知识图谱 / 第二大脑 OS / 自动化协作协议 / 数字遗产——均需领域身份、团队或网络效应,单人结构性不成立(详见意图文档表格)。
- **三台复利机器**:引擎复利(每件产品增厚 danqing,第 N 件成本趋零)/ 数据复利(用户专注历史=切换成本=续费)/ 思想史复利(建造实录连载=读者群+署名权)。
- **三条变现**:旗舰订阅 / 内容(书/课程)/ 品牌("丹青出品")。
- **剪贴板降级顺延**:从"第二 POC"降为"引擎复用验证";优先级让给旗舰数据层(数据复利第一笔本金)。
- **第一块里程碑「旗舰化第一刀」**:山/森林终审 → 付费边界 spec → 数据层 MVP → 建造实录开写(用户拍板先做数据层,剪贴板顺延)。

**Why:** 原战略(两族工具+潮汐美学)是定位,这是定位的十年尺度放大;用户显式确认"不做团队游戏,一切竞争选'抄不走我'"。

**How to apply:** 新决策/新 POC 前先对照意图文档与路线图;数据层是战略关键,数据结构从第一天为十年设计(版本化/可导出/可迁移)。关联 [[danqing-project-state]] [[danqing-strategic-positioning-efficiency-tools]]。

**2026-08-01 里程碑 0 推进**: 用户拍板"剪贴板顺延,先做旗舰数据层",B+C 已完成——付费边界 spec 已确认(`docs/specs/companion-flagship-pricing.md`:固定篝火免费场景 / 自定义时长免费 / 买断 ¥68+订阅留口双轨 / 订阅展示本期不做)、数据层 MVP 落地(`examples/pomodoro/stats.rs`:`SessionRecord`+`FocusHistory`,独立存储 `%APPDATA%/danqing/focus-history.json`,format_version 版本化 + 字段 serde default 前后兼容 + 未来版本拒读不覆盖;自然完成的 Focus 每完成记一条:开始/结束墙钟、planned/focused 秒、场景、轮次;CSV 导出;统计面板 今日/本周/累计)。`TickReport` 加 `completed_round`(pre-advance 轮次,供记录)。评审修复:数据层迭代为 `focused_secs=计划时长`(自然完成=实际专注恒等于计划,砍掉整个 dt 累加——它在 huge overshoot 下污染数据,见 [[scene-motion-uv-displacement]] 式"机制错幅度救不回"同构教训)、`load_history_guarded`(文件存在但解析不出→拒写保护,防降级覆盖)。全绿:fmt+clippy 零警告+lib 236+集成 57+pomodoro 176+release。

**Task D 建造实录三篇草稿完成(2026-08-01)**: `docs/chronicle/{01-why-ten-years,02-eighteen-iterations,03-data-is-the-moat}.md`——为什么十年磨著作(护城河排序+著作型旗舰)/ 场景动效 18 次迭代(wrap-clean 数学陷阱+alpha 上限 0.50+sum-of-sines vs value noise+UV 位移适用边界)/ 数据即护城河(版本化+拒写保护+常数替代累加器)。待用户校订后发布(仓库外)。

**Task A 山/森林终审通过(2026-08-01)**: 用户目测通过, plan/todo 归档(T5 勾)、spec 验收 8 注✅、CLAUDE.md 下一步已更新。**终审修正两处**: ①无 1-5 场景快捷键——场景切换仅 ◀/▶ 按钮,全局热键只有显隐/暂停/退出;②森林副层已去——background.wgsl 当前为单层 forest_mist(单 mist_pattern, SPEED 0.0625 / SCALE 2.0 / ALPHA 0.25)。**里程碑 0「旗舰化第一刀」B+C+D+A 全部完成**。剩余: 付费门禁/数据同步后端(遗留)、建造实录发布(用户, 仓库外)。

**2026-08-10 免费发布决策**: 用户决定 pomodoro 全部功能免费,零门槛,不设付费分层。pomodoro 是第一个个人项目,目的是练手 AI 编码+产品发布+社区+品牌。付费边界 spec(`companion-flagship-pricing.md`)废弃;数据同步 spec 付费引用过时;竞品备忘录定价分析过时但定位校验仍有效。代码中"旗舰"角标已移除。意图落盘 `docs/intent/pomodoro-free-release.md`。
