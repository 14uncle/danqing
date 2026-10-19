# round10 选型复审 · 任务清单

> 依据：`danqing/tasks/plan-round10-rescreen.md` + `danqing/docs/specs/round10-selection-guide-2026-10-20.md`
> 主台账：`danqing/docs/intent/round10-rescreen-2026-10-20.md`（所有 Task 的唯一产出文件）
> 纪律：全程零产品代码、零 git 提交；存量处置只出建议。
> 说明：本清单独立成文，不占用仓库既有的 `tasks/todo.md`（旧计划已从 git 恢复）。

## Phase 1：存量待判

- [x] **Task 1：复审 dropshelf**
  - Description：按 Gate 0 去重 → Gate 1 agent 吃不掉 → Gate 2 需求真实 → Gate 3 陌生人掏钱，复审 dropshelf，产出完整台账行与「继续 / 埋掉」建议。
  - Acceptance：
    - [ ] Gate 0：未命中死池 / 已扫域，写明依据
    - [ ] Gate 1：判定 + 理由（OS 层拖拽，agent 替不了）
    - [ ] Gate 2：≥3 独立来源 + 日期 + 行为证据；证据不足则如实写「不足」
    - [ ] Gate 3：陌生人付费证据「有 / 无」+ URL + 日期
    - [ ] 结论一句 + 建议（继续 / 埋掉）
  - Verify：台账行六项齐全，证据可点开复核；Checkpoint 1 人工复核
  - Dependencies：None
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：S

- [x] **Task 2：复审 luoban**
  - Description：同 Task 1 流程。已知线索：核心价值是 MD → inline-CSS 转换，按 Gate 1 死刑句预期判死，需补一条用户侧证据后落行。
  - Acceptance：
    - [ ] Gate 0：去重结论
    - [ ] Gate 1：判定 + 理由 + 至少一条用户侧证据（原话 / 行为 / 竞品形态）
    - [ ] 若 Gate 1 判死即停，写明后续闸门未执行
    - [ ] 结论一句 + 建议（继续 / 埋掉）
  - Verify：台账行完整；Checkpoint 1 人工复核
  - Dependencies：None
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：S

### Checkpoint 1：存量处置建议确认

- [x] 两行台账完整可复核
- [x] 用户对 dropshelf / luoban 给出「继续 / 埋掉」明确裁决（2026-10-20：**两件均埋掉**）
- [x] 未获裁决不执行任何处置

## Phase 2：round9 幸存者

- [x] **Task 3：复审 物格 + 徒弟**
  - Description：两条各走 Gate 0→1→2→3。第一证据源为 round9 深卷、0→1 卷与 redteam 卷；只做增量查证。
  - Acceptance：
    - [ ] 每条一行，Gate 1 判定 + 理由
    - [ ] 活着：Gate 2 的 ≥3 来源 + 日期 + 失效标志；Gate 3 付费证据
    - [ ] 死者：一句话死因 + 证据 URL / 日期
    - [ ] 物格注明同时出现在 round9 与 0→1 卷
  - Verify：两行完整；Checkpoint 2 复核
  - Dependencies：Task 1–2（台账文件已建立）
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：M

- [x] **Task 4：复审 A2 log 家族 + A3 iPhone + A4 多显示器**
  - Description：三条串行过闸。A2 是产品线延伸，A3 / A4 是载体与场景变体；重点判 Gate 1（用户会不会自己让 agent 造）。
  - Acceptance：
    - [ ] 每条一行，Gate 1 判定 + 理由
    - [ ] 存活者完成 Gate 2 / 3；死者一句话死因 + 证据
  - Verify：三行完整；Checkpoint 2 复核
  - Dependencies：Task 3
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：M

- [x] **Task 5：复审 5 采访员 + Hazel**
  - Description：两条串行过闸。5 采访员此前未做深调研，若证据不足按「不足」落行，不硬判活。
  - Acceptance：
    - [ ] 每条一行，Gate 1 判定 + 理由
    - [ ] 存活者完成 Gate 2 / 3；证据不足如实记录
  - Verify：两行完整；Checkpoint 2 复核
  - Dependencies：Task 4
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：M

### Checkpoint 2：round9 全部有结论

- [x] 7 个条目全部落行（A1 体裁 = luoban，不重复审）
- [x] 每个条目 Gate 1 判定明确；活着的已走完 Gate 2 / 3
- [x] 死者有一句话死因 + 证据

## Phase 3：0→1 六案

- [x] **Task 6：六案 Gate 1 快筛**
  - Description：对 `intent-0to1-innovation-2026-10-17.md` 候选池 1–6 只做 Gate 1，产出存活名单。物格已在 Task 3 审过，此处只引用结论。
  - Acceptance：
    - [ ] 六案各一行 Gate 1 判定 + 理由
    - [ ] 存活名单明确（可能为空）
    - [ ] 物格标注「见 Task 3」
  - Verify：六行完整；存活名单可复核
  - Dependencies：Checkpoint 2
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：M

- [x] **Task 7：存活者走 Gate 2 / 3**
  - Description：对 Task 6 存活名单逐条补 Gate 2（≥3 来源 + 日期 + 失效标志）与 Gate 3（陌生人付费）。
  - Acceptance：
    - [ ] 每个存活者 Gate 2 / 3 完整
    - [ ] 证据不足的条目标明「不足」，不硬判活
  - Verify：台账行完整；Checkpoint 3 复核
  - Dependencies：Task 6
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：M

### Checkpoint 3：0→1 全部有结论

- [x] 六案全部落行
- [x] 存活名单明确，且已完成 Gate 2 / 3

## Phase 4：汇总与裁决材料

- [x] **Task 8：台账定稿 + 结论**
  - Description：汇总全部行，写出总结论：候选卷（含存活者清单）或零候选记录（逐条死因 + 一句总评 + 开新域扫描请示）。
  - Acceptance：
    - [ ] 台账覆盖 C + A + B 全部条目，无遗漏
    - [ ] 结论段明确：有候选 / 零候选
    - [ ] 零候选时附「是否开新域扫描」请示，不自动开
    - [ ] 更新意图文档状态行
  - Verify：按 spec 第 7 节逐项自检；Checkpoint 4 人工复核
  - Dependencies：Task 7
  - Files：
    - `danqing/docs/intent/round10-rescreen-2026-10-20.md`
    - `danqing/docs/intent/selection-round10-intent-2026-10-20.md`
  - Scope：S

- [x] **Task 9：候选走调研三闸（仅当有候选）**
  - Description：对每个存活候选执行调研三闸：裁决段（继续 / 放弃 / 转向 + 依据）、红队（逐条攻击）、一页粗算（渠道可见流量 × 转化 × 单价 vs 工时）。
  - Acceptance：
    - [ ] 每个候选三项齐全
    - [ ] 裁决段给出明确结论与依据
    - [ ] 粗算含四个乘数，不空喊
  - Verify：Checkpoint 4 人工复核
  - Dependencies：Task 8
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：M

## Phase 5：新域扫描（用户 2026-10-20 批准 A）

- [x] **Task 10：四路并行新域扫描 + 合成**
  - Description：按 Gate 1「agent 吃不掉」优先，扫四个未覆盖域，每域最多 3 张卡；只交结论 + URL + 日期，不写方案。
  - 域：① 个人数据/设备主权 ② 单人/小生意运营 ③ 家庭/生活行政 ④ 外设/硬件/创作工作流。
  - Acceptance：
    - [ ] 每域有「零候选 + 死因清单」或最多 3 张卡
    - [ ] 每张卡 Gate 1 判定 + ≥3 来源（URL + 日期）+ 付费证据
    - [ ] 命中死池的条目一行带过
  - Verify：合成进主台账；Checkpoint 4 复核
  - Dependencies：Checkpoint 4（A）
  - Files：`danqing/docs/intent/round10-rescreen-2026-10-20.md`
  - Scope：M

### Checkpoint 4：用户裁决

- [x] 台账完整、可复核
- [x] 有候选：调研三闸齐全（字体库 / 赊销单据）
- [x] 零候选：逐条死因 + 开新域请示（已开，四路扫描完成）
- [ ] 用户给出最终决定（停选型 / 先修分发 / 廉价探针）
