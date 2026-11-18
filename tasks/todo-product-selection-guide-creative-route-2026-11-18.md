# Todo: 产品选型指南双路线修订

- **Source spec**：`danqing/docs/specs/product-selection-guide-creative-route-2026-11-18.md`
- **Plan**：`danqing/tasks/plan-product-selection-guide-creative-route-2026-11-18.md`
- **状态**：全流程完成，用户终审通过（2026-11-18）；未 commit / push

## Build

### Task 1: 根指南产品心法双路线化

**Description：** 精确替换 `farm01/CLAUDE.md`「产品心法」标题至历史废除说明段，保留两条心法结构与后续「执行纪律」原文。第一条写清两种证据来源与共同开枪闸，第二条写清默认创造、创造链、结构重构、伸手实验和「卧槽」边界。

**Acceptance criteria：**
- [x] 同时命名创造型 / 补丁型，创造型默认先走，补丁型不附加惊艳门槛；
- [x] 创造型无社区三证可入场，开枪前必须有实验行为证据；补丁型 ≥3 来源等标准完整保留；
- [x] 包含解法重构、事实 / 假设分离、预注册字段、用户开枪裁决及「卧槽」非验证证据。

**Verification：**
- [x] 只读检索上述关键句，确认旧「推理不许跑到证据前面」不再以原句存在；
- [x] 对照 spec R1–R7 逐条打勾，确认执行纪律段未被改动。

**Dependencies：** 计划批准。

**Files likely touched：**
- `F:/github/farm01/CLAUDE.md`

**Estimated scope：** S（1 文件）。

---

### Task 2: 深调研条款按路线拆分

**Description：** 将根 `CLAUDE.md` 跨仓库约定中「选定后、spec 前必须做一轮深度调研」单条改为公共项与分路线专项；保留调研三闸、系统吸收距离、红队与粗算，不改变 spec 五阶段顺序。

**Acceptance criteria：**
- [x] 公共项、补丁型专项、创造型专项均可独立检索；
- [x] 无直接竞品时研究当前完成方式，不强制伪造竞品矩阵 / issues；
- [x] 裁决段、裁决前红队、一页粗算仍齐全，spec → plan → build → review → code-simplify 不变。

**Verification：**
- [x] `grep -n -E "创造型专项|补丁型专项|调研三闸|spec 技能发动后" F:/github/farm01/CLAUDE.md` 能定位全部条款；
- [x] 与 spec R8 对照，无新增付费或形态准入闸。

**Dependencies：** Task 1。

**Files likely touched：**
- `F:/github/farm01/CLAUDE.md`

**Estimated scope：** S（1 文件）。

---

## Checkpoint A: 根指南语义验收

- [x] 五条结构检查通过：双路线 / 默认创造 / 三证归补丁 / 假设诚实 / 分路线调研；
- [x] 五场景回放记录预期结果：补丁直达、无社区证据可实验、惊艳零伸手不开枪、微优化转补丁、无现象不入场；
- [x] 执行纪律四项及历史废除条款未变化；
- [x] 若任一项不通过，先修 T1 / T2，不进入外围同步。

**回放记录（2026-11-18）**：① 多条近期凑合证据命中补丁型，可直接进入深挖与开枪裁决；② 有真实现象与创造链、无社区三证者可入创造型并设计实验；③ 惊艳不能替代行为证据，零伸手不得开枪；④ 只「更快一点」须补写结构重构，否则退回补丁型证据标准；⑤ 无行为现象者明文不入场。五项均与 spec §8.2 一致。

## Synchronization

### Task 3: 开发指南最小同步

**Description：** 更新 `PRODUCT-DEV-GUIDE.md` 顶部对根指南的描述，使其指向「双路线两条心法 + 分路线调研 + 调研三闸」；不新增开工或发布检查项。

**Acceptance criteria：**
- [x] 顶部描述不再只暗示单一「谁还凑合」入口；
- [x] 开工前闸、开发中约束、发布前闸的实质条目零变化；
- [x] 条款冲突仍以根 `CLAUDE.md` 为准。

**Verification：**
- [x] `grep -n -E "产品心法|创造型|补丁型" F:/github/farm01/PRODUCT-DEV-GUIDE.md` 检查同步句；
- [x] 对比已读原文，A–F 各检查单无增删改。

**Dependencies：** Checkpoint A。

**Files likely touched：**
- `F:/github/farm01/PRODUCT-DEV-GUIDE.md`

**Estimated scope：** XS（1 文件）。

---

### Task 4: 用户级记忆现行口径同步

**Description：** 更新 `product-selection-demand-first.md` 的 description、顶部权威摘要及 How to apply 的现行部分，使其反映双路线、证据可生成与共同开枪闸；保留 2026-09-05 之后历史存档，不篡改历史裁决。同步 `MEMORY.md` 对应一行索引。

**Acceptance criteria：**
- [x] 现行摘要不再把 ≥3 社区来源写成所有路线的通用前置；
- [x] 明确创造优先、实验产证、开枪不降级及「卧槽」非验证；
- [x] 历史段仍可追溯，其他记忆文件与产品状态不变。

**Verification：**
- [x] 检索记忆顶部与索引，确认日期和双路线摘要一致；
- [x] 检查文件内历史裁决未被删改，未新增重复记忆文件。

**Dependencies：** Checkpoint A，可与 Task 3 并行但默认顺序执行。

**Files likely touched：**
- `C:/Users/gwhun/.claude/projects/F--github-farm01/memory/product-selection-demand-first.md`
- `C:/Users/gwhun/.claude/projects/F--github-farm01/memory/MEMORY.md`

**Estimated scope：** S（2 文件）。

---

### Task 5: 全量验收与阶段记账

**Description：** 按 spec §8–9 执行结构、场景、保持性和差异检查；在任务清单记录实际结果，将 spec 状态推进为「build 完成，待 review」，在意图文档添加实施产物指针。此任务不宣告用户终审通过。

**Acceptance criteria：**
- [x] spec §9 全部机器侧 Success Criteria 有逐项证据，未做项如实写；
- [x] 五场景回放全部得到预期结果，或列阻塞并停止后续完成声明；
- [x] spec / intent 状态准确反映「build 完成，待 review」，不写成已终审。

**Verification：**
- [x] 运行 spec §3 四条命令并记录关键结果；
- [x] 检查变更文件仅限 spec 批准清单，无产品代码 / 状态变更、无实验、无 commit / push；
- [x] 记录「未运行 cargo：纯文档变更，无 Rust 代码与配置改动」。

**Dependencies：** Tasks 3、4。

**Files likely touched：**
- `F:/github/farm01/danqing/docs/specs/product-selection-guide-creative-route-2026-11-18.md`
- `F:/github/farm01/danqing/docs/intent/product-selection-guide-creative-route-2026-11-18.md`
- `F:/github/farm01/danqing/tasks/todo-product-selection-guide-creative-route-2026-11-18.md`

**Estimated scope：** S（3 文件）。

## Checkpoint B: Build 完成

- [x] 根指南、开发指南、记忆摘要口径一致；
- [x] spec §9 机器侧可验证项全过；
- [x] 无范围外文件、无产品状态变化；
- [x] 可以进入独立评审，但不宣称用户终审完成。

**Build 验收记录（2026-11-18）**：

1. 根指南可检索双路线、假设诚实、实验预注册和创造型专项；≥3 来源只位于补丁型条款；执行纪律四项原样保留。
2. 五场景结果见 Checkpoint A；外围同步未改根指南语义，结论继续成立。
3. `git -C F:/github/farm01/danqing diff --check` 无输出；`status --short` 仅列本流程 5 个新文档。另用 trailing-whitespace 检索覆盖这 5 个未跟踪文档，零命中。
4. 开发指南只改顶部同步句；用户级记忆现行摘要与 `MEMORY.md` 索引已更新，历史段保留并加历史标签。
5. 未运行 cargo：纯文档变更，无 Rust 代码与配置改动。未执行真实实验、未复审 round13 或历史候选、未启动 POC、未修改产品状态，未 commit / push。
6. spec §9 的 10 项机器侧条件全部通过；用户终审保留在 Checkpoint D，不在 build 阶段预勾。

## Review

### Task 6: 双路评审与确认问题修复

**Description：** 按根项目约定调用 `/agent-skills:code-review-and-quality`，将本变更视为流程契约评审：重点检查证据闸是否被削弱、两条路线是否互相偷渡门槛、实验授权是否扩大、条款是否可执行。对确认的 Critical / Required 逐项修复并复验。

**Acceptance criteria：**
- [x] 评审覆盖根指南、开发指南、记忆摘要及 spec 验收结果；
- [x] 所有确认的 Critical / Required 已修复，或明确记录无此类发现；
- [x] 修复后五场景与保持性检查重新通过。

**Verification：**
- [x] 保存评审结论与修复对应关系到本任务清单；
- [x] 对每项修复做最小差异检查，不顺手重判候选或扩张发布闸。

**Dependencies：** Checkpoint B。

**Files likely touched：**
- `F:/github/farm01/CLAUDE.md`
- `F:/github/farm01/PRODUCT-DEV-GUIDE.md`
- `C:/Users/gwhun/.claude/projects/F--github-farm01/memory/product-selection-demand-first.md`
- `C:/Users/gwhun/.claude/projects/F--github-farm01/memory/MEMORY.md`
- `F:/github/farm01/danqing/tasks/todo-product-selection-guide-creative-route-2026-11-18.md`

**Estimated scope：** M（最多 5 文件；未命中问题的文件不改）。

## Checkpoint C: Review 完成

- [x] Critical / Required 清零；
- [x] 评审没有引入范围外改动；
- [x] 修复后核心场景仍全过。

**独立评审记录（2026-11-18）**：`agent-skills:code-reviewer` 只读评审结论为 **Approve**；Critical / Required / Optional / Nit 均为零，因此没有修复项或修复后差异。评审独立从根条款反推五场景全部通过，并确认创造型不免验证、补丁型三证不加减、实验不自动授权、付费前置与灵魂论未复活、调研三闸及执行纪律完整、开发指南未扩张 A–F。传统代码安全与性能不适用；授权边界已审。评审未把未跟踪文件的 `git diff --check` 空输出当作语义证明，实际阅读与定点检索才是判定依据。

## Code-simplify

### Task 7: 文档化简与最终验收

**Description：** 调用 `agent-skills:code-simplification` 的化思原则，删除重复、含混和过度设计的措辞，合并同义规则；只做不改变已审语义的文档化简。随后完成最终状态落账并向用户提交终审摘要。

**Acceptance criteria：**
- [x] 双路线仍清晰可执行，未因压缩丢失预注册、开枪或授权边界；
- [x] spec / intent 状态反映 review 与 simplify 已完成，用户终审仍单列；
- [x] 最终命令检查通过，变更清单与验证结果完整交付用户。

**Verification：**
- [x] 重跑 spec §3 命令、五场景和保持性检查；
- [x] `git -C F:/github/farm01/danqing diff --check` 无相关错误；
- [x] 明确报告未运行 cargo、未执行实验、未复审候选、未 commit / push。

**化简与终验记录（2026-11-18）**：权威指南零文字化简——现有条款已按路线和阶段分层，继续压缩只会损失契约清晰度；本任务只更新 spec / intent / plan / todo 状态与验收记录。最终四条 spec 命令均通过，5 个仓内新文档 trailing-whitespace 检索零命中；根指南五场景语义与独立评审时一致，历史废除条款及执行纪律四项仍在。未运行 cargo（纯文档变更），未执行真实实验、未复审候选、未启动 POC、未修改产品代码或状态，未 commit / push。

**Dependencies：** Checkpoint C。

**Files likely touched：**
- `F:/github/farm01/CLAUDE.md`
- `F:/github/farm01/danqing/docs/specs/product-selection-guide-creative-route-2026-11-18.md`
- `F:/github/farm01/danqing/docs/intent/product-selection-guide-creative-route-2026-11-18.md`
- `F:/github/farm01/danqing/tasks/todo-product-selection-guide-creative-route-2026-11-18.md`

**Estimated scope：** S（最多 4 文件；无需化简则只动状态与验收记录）。

## Checkpoint D: 用户终审

- [x] 用户审阅根指南最终文本，确认「创造优先、证据可生成、开枪不降级」；
- [x] 用户确认没有借本次调整启动 POC、复活候选或修改既有裁决；
- [x] 是否 commit / push 留待用户另行明确指示，本流程不执行。

**结案记录（2026-11-18）**：用户回复「确认」，终审通过；五阶段与四个 Checkpoint 全闭。权威指南、开发指南及用户级记忆已同步，未启动 POC、未复活候选、未修改既有产品裁决，未 commit / push。
