# round10 选型复审台账（2026-10-20）

> 依据：`danqing/docs/specs/round10-selection-guide-2026-10-20.md` + `danqing/tasks/todo-round10-rescreen.md`
> 闸门顺序：Gate 0 去重 → Gate 1 agent 吃不掉 → Gate 2 需求真实 → Gate 3 陌生人掏钱；命中即停。
> 本文件只记录结论 + 证据，不复述深卷全文。存量两件的处置只出建议，最终由用户裁决。
> 状态：**Checkpoint 1 待裁决**（Phase 1 完成，Phase 2 未开始）。

## 总表

| 条目 | 来源 | Gate 0 | Gate 1 | Gate 2 | Gate 3 | 结论 | 建议 |
|------|------|--------|--------|--------|--------|------|------|
| dropshelf | round9 T1 / 已建成 | 未命中 | 活（带反证） | **死**（严格口径） | 有（单价极低） | 判死 | 埋掉 |
| luoban | round9 A1 / 已建成 | 未命中 | **死** | 未执行 | 未执行 | 判死 | 埋掉 |

---

## C1 · dropshelf（丹青暂存架）

**Gate 0 去重：未命中。**
round9 立案、已建成未发布，不是坟头；未命中 round4–9 死池与已扫域。

**Gate 1 agent 吃不掉：活（带反证）。**

- 核心价值 = OS 层拖拽暂存 + 一次拖出，不是一次性转换；目标用户不会为它写代码。按 Gate 1 的「B 做不好」成立：即使用 agent 造，也要处理 `WM_DROPFILES` / `IDropTarget` / 冲突与事务，普通用户不会做。
- 反证（必须记录）：vibe-code 手搓簇 ≥6 仓——holdem（152★，2024-09）、Bucket（2026-06）、ghost-drop（2026-07）、shelf-Mac（2026-02）、omarchy-ledge（2026-08）、PyDrop（2021）。说明少数用户确实会 DIY；但付费买家同时存在（见 Gate 3），DIY 簇不是主流。
- 证据：
  - `deep-dive-round9-t1-2026-10-09.md` §D2（2026-10-09）
  - https://github.com/iamzubin/holdem
  - https://github.com/GameGodS3/DropPoint

**Gate 2 需求真实：失败（严格口径）。**

- 免费全链在位者已覆盖「收 → 暂存 → 一次拖出」：
  - DropPoint 1094★（pushed 2026-08-04，末版 release 2022-09-12）
  - Edge-Drop 632★（v0.3.2，2026-09-26）
  - holdem 152★（3.1.1，2026-09-04）
  - StashIt / KyteShelf 等小仓
- 红队 A2（2026-10-09）判定：「缺中转暂存架」的需求表述被免费全链证伪；真活证据只剩「买后缺 Y」。
- 残留需求 = 品质差（Outlook 附件、修饰键激活、持久架/分组、Move-vs-Copy），属于「免费工具已接走、只剩品质缝」。按心法「被免费工具接走 = 已满足」，不足以支撑新开品。
- 证据：
  - `deep-dive-round9-t1-2026-10-09.md` §D2 / §红队回卷 A2（2026-10-09）
  - https://github.com/Deepender25/Edge-Drop

**Gate 3 陌生人掏钱：有，但单价接近免费。**

- Dropshelf Pro 一次性 IAP 买家自白 10 笔：2023-10-08 / 2024-02-20 / 2024-06-07（「I paid for Pro immediately after downloading it」）/ 2025-04-29 / 2025-09-23 / 2026-01-12 / 2026-03-30 / 2026-05-17 / 2026-05-29 / 2026-07-10。
- Mac 侧付费生态：Yoink €9.99 一次性；Dropover 4.9/5 × 7.7K ratings。
- Windows Pro 标价约 **$0.50**（appx4fun，版本 1.0.10.0，2026-01-17）——付费行为成立，但客单价几乎等于免费。
- 证据：
  - https://apps.microsoft.com/detail/9mzpc6p14l7n
  - https://www.appx4fun.com/apps/57211/
  - https://eternalstorms.at/yoink/
  - https://dropoverapp.com/

**结论：判死（Gate 2 失败）。建议埋掉，仓库与代码保留作档案。**

附加事实（非闸门，供裁决参考）：

- 用户本人建成后**一次未运行**（2026-10-20 访谈），无自用需求。
- 一页粗算 ≈ ¥0–21/月 vs v1 15–25 人日（2026-10-09 深卷）。
- 现存失效标志④'：Edge-Drop 出浮动置顶窗模式 = 头号判毙扳机；现况 632★ + 活跃发版 + 全链，仅差「浮动置顶」一栏。

---

## C2 · luoban（丹青落版）

**Gate 0 去重：未命中。**
round9 A1 体裁 = luoban，同一方向只审一次；未命中死池与已扫域。

**Gate 1 agent 吃不掉：死（死刑句命中）。**

- 核心价值 = MD / CF_HTML → inline-CSS 公众号 HTML，**一次转换**；命中 Gate 1 死刑句「核心价值 = 一次转换」与「用户一个提示词就能拿到同样结果」。
- 现成替代不是理论，是可直接安装的 agent skill：
  - **gzh-design-skill**（isjiamu，≈1.7k★，2026-10-20 检索）：把 Markdown 一键排成可直接粘进公众号编辑器的 HTML，6 套主题 + 主题生成器 + 双关卡校验，`npx skills add` 安装。
  - crossoverJie 镜像（prespec R3 记 ★3946，2026-10-09）。
  - 另有 md2wechat（¥199 永久，2026-10-09 仍在售）、mdnice、doocs/md、wenyan 等免费/付费在位者。
- 官方迁移：微信编辑器 2026-09 上线「内容结构检测」+ Dark Mode；AI 客户端侧豆包 docx 导出、DeepSeek 桌面 Harness、Kimi PPT/Excel/PDF 已实锤（prespec R3，2026-10-09）。
- 用户侧证据：用户 2026-10-20 访谈原话「微信有自己的公众号助手，是不是可能解决排版问题」；产品建成后一次未运行。
- 证据：
  - https://github.com/isjiamu/gzh-design-skill
  - https://github.com/crossoverJie/gzh-design-skill
  - `prespec-a1-r3-official-refresh-2026-10-09.md`（2026-10-09）
  - `prespec-a1-r2-feature-matrix-2026-10-09.md`（2026-10-09）

**Gate 2 / Gate 3：未执行。** Gate 1 命中死刑即停，按流程不再收集后续证据。

**结论：判死。建议埋掉，仓库与代码保留作档案。**

---

## Sources

- [Dropshelf · Microsoft Store](https://apps.microsoft.com/detail/9mzpc6p14l7n)（检索 2026-10-20）
- [Dropshelf · appx4fun](https://www.appx4fun.com/apps/57211/)（版本 1.0.10.0，2026-01-17）
- [DropPoint · GitHub](https://github.com/GameGodS3/DropPoint)
- [Edge-Drop · GitHub](https://github.com/Deepender25/Edge-Drop)
- [holdem · GitHub](https://github.com/iamzubin/holdem)
- [gzh-design-skill · GitHub](https://github.com/isjiamu/gzh-design-skill)（≈1.7k★，2026-10-20 检索）
- [gzh-design-skill 镜像 · GitHub](https://github.com/crossoverJie/gzh-design-skill)
- [Yoink](https://eternalstorms.at/yoink/)
- [Dropover](https://dropoverapp.com/)
