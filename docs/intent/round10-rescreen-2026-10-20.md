# round10 选型复审台账（2026-10-20）

> 依据：`danqing/docs/specs/round10-selection-guide-2026-10-20.md` + `danqing/tasks/todo-round10-rescreen.md`
> 闸门顺序：Gate 0 去重 → Gate 1 agent 吃不掉 → Gate 2 需求真实 → Gate 3 陌生人掏钱；命中即停。
> 本文件只记录结论 + 证据，不复述深卷全文。存量两件的处置只出建议，最终由用户裁决。
> 状态：**复审完成（2026-10-20）**。C1/C2 已由用户裁决埋掉；A3/A4 过三道闸但调研三闸判放弃；本轮判无。Checkpoint 4 待用户裁决是否开新域。

## 总表

| 条目 | 来源 | Gate 0 | Gate 1 | Gate 2 | Gate 3 | 结论 | 建议 |
|------|------|--------|--------|--------|--------|------|------|
| dropshelf | round9 T1 / 已建成 | 未命中 | 活（带反证） | **死**（严格口径） | 有（单价极低） | 判死 | 埋掉 |
| luoban | round9 A1 / 已建成 | 未命中 | **死** | 未执行 | 未执行 | 判死 | 埋掉 |
| A2 log 家族 | round9 | 延后 | — | — | — | 未审 | 10-21 后复议 |
| A3 iPhone 备份 | round7 | 未命中 | 活 | 活 | 有 | 过闸候选 | 调研三闸→放弃 |
| A4 多显示器 | round7 | 未命中 | 活 | 活 | 有 | 过闸候选 | 调研三闸→放弃 |
| Hazel | rewalk | 未命中 | **死** | 通过 | 未执行 | 判死 | 不投入 |
| 物格 | 0→1 / round9 | 未命中 | 活 | 未过（条件性） | 有（弱） | 不立项 | 维持五条件 |
| 徒弟 | 0→1 | 未命中 | **死** | **死** | — | 维持归档 | 三钥匙 |
| 主张器 | 0→1 | 未命中 | 条件性 | 条件性 | 无 | 判死 | — |
| 预演台 | 0→1 | 未命中 | 条件性 | 通过 | 弱（形态不对位） | 判死 | — |
| 采访员 | 0→1 | 未命中 | 条件性 | **失败** | 无 | 判死 | — |
| 自传 | 0→1 | 未命中 | **死** | — | — | 判死 | — |

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

## Phase 2 · round9 幸存者

### A1 · 体裁落地层（= luoban）

**去重**：与 C2 同一方向，见 C2（Gate 1 判死），不重复审。

### A2 · log 家族

**Gate 0：延后，本轮不审。**

- log v1.x 在制已发布，组合纪律保护；round9 明文「log 家族不碰至 10-21」；校准钟未响。
- 10-21 后复议，届时按同一闸门补审。
- 证据：`round9-selection-2026-10-09.md` §去重名单（2026-10-09）。

### A3 · iPhone → Windows 照片/备份窄角

- **Gate 1：通过。** 正常用户不会为备份照片去跑 `pymobiledevice3` 或自写脚本；5 个 DIY 仓是开发者行为，构成反证但不构成主流。
- **Gate 2：通过。** 5 个独立 DIY 仓 2024-01→2026-09 连续：iPhone_photos_windows_backup（2026-06-24）、iVault（2026-05-03）、isynclocal（2026-07-14）、OrchardBridge（2026-09-19）、backup_iphonepics_windows（2024-01）；iMazing Windows 在售有 Buy（2026-09-25）。
  - 失效标志：MS Photos 修好导入/差异同步；Apple 官方 Windows 端口碑转好；DIY 仓集体停更零 issue；pymobiledevice3 归档。
- **Gate 3：有。** iMazing Windows 付费在位（2026-09-25 在售），形态侧对位成立。
- **结论：过三道闸，进入调研三闸。**
- 证据：`product-selection-round7-2026-09-25.md` §A1 卡①（2026-09-25）；https://github.com/dendr203/iPhone_photos_windows_backup ；https://imazing.com/

### A4 · 多显示器管理

- **Gate 1：通过。** AHK 手搓与 DisplayFusion 付费并存；正常用户不会自己写 Win32 配置管理器。
- **Gate 2：通过。** DisplayFusion ≥3 笔独立购买自白（2025-05 / 2024-01 / 2026-03）+ 比价 + AHK 手搓潮；温度 2024→2026 无衰减。
  - 失效标志：Win11 逐屏任务栏 / 原生窗口位置 profiles；PowerToys 出 profile 管理。
- **Gate 3：有。** 3 笔买断自白，形态侧对位成立。
- **结论：过三道闸，进入调研三闸。**
- 证据：`product-selection-round7-2026-09-25.md` §A2 卡3（2026-09-25）；`rewalk-rounds1-8-newguide-2026-10-09.md`（2026-10-09）；https://www.displayfusion.com/

### A5 · Hazel 型自动文件整理

- **Gate 1：失败。** 手搓簇 2024-10→2026-10 新建同类小仓 ≥40 个（hazelnut 312★ 2026-02；stevedore-flutter 2026-10-01；mouzi 1012★ 2026-05），且 AI 文件助手形态已起（llama-fs 5831★）。用户会自己造；核心「按规则搬家」可被一个提示词覆盖，命中 Gate 1 死刑句。
- **Gate 2：通过**（≥4 类独立源、温度活），但 Gate 1 已停。
- **Gate 3：未执行。** 旁注：付费在位者在收钱（File Juggler $40 / Hazel $42），公开买断自白 0。
- **结论：判死。**
- 证据：`rewalk-rounds1-8-newguide-2026-10-09.md` §Hazel 自动文件整理（2026-10-09）。

---

## Phase 3 · 0→1 六案 Gate 1 快筛

| # | 案 | Gate 1 | 后续闸 | 结论 |
|---|----|--------|--------|------|
| 1 | 物格 · 万物户口 | 活 | Gate 2 未过（条件性）；Gate 3 有（弱） | 不立项，维持转向/挂起 |
| 2 | 徒弟 · 仪式 | **死** | — | 维持归档 |
| 3 | 主张器 | 条件性（只剩案件管家/跟单） | Gate 3 无同形态付费 | 判死（本轮） |
| 4 | 预演台 | 条件性（只剩核验+预填+跟单） | Gate 3 弱（黄牛=邻近服务，形态不对位） | 判死（本轮） |
| 5 | 采访员 | 条件性（聊天被吸收，只剩采访工程） | Gate 2 失败（培养路线证据弱）；Gate 3 无 | 判死 |
| 6 | 自传 | **死**（Recall 系统吸收 + 一次叙事生成 + 隐私恐惧） | — | 判死 |

### 物格 · 万物户口（详见 Phase 2 同名段）

- Gate 1 **活**：持久台账 + 保修倒计时 + 家庭共享，不是一次转换；核心可零 LLM。
- Gate 2 **未过（条件性）**：需求存在性为真（App Store 中国区 2025Q4–2026Q4 至少 8 款井喷 + 评论区付费实证），但红队三炮弹——物理 ¥15 收纳册够用、「户口」是自造隐喻、Windows 主场形态存疑；增量价值未证。
- Gate 3 **有（弱）**：家维录 ¥198 终身（促销 ¥6）、物有凭买断在售；「我的物品/过期啦」评论区真金白银（过期啦 4.85★/24705）。
- **结论：不立项，维持转向/挂起**；五条件未满足前不进 spec。
- 证据：`deep-dive-wuge-2026-10-17.md` §裁决（2026-10-17）。

### 徒弟 · 仪式（详见 Phase 2 同名段）

- Gate 1 **死**：通用 computer-use / 办公 agent 已先发布（WorkBuddy 实测 2026-10-17）；产品形态本身是 agent 同构物。
- Gate 2 **死**：付费行为证据全在企业 RPA（按键精灵企业定制 / 影刀考证 / 企培）；个人买断位付费证据缺位；「改版不死」痛点原话无铁证。
- **结论：维持归档**（三条复活钥匙，缺一不复活）。
- 证据：`deep-dive-tudi-2026-10-17.md` §八（2026-10-17）。

---

## Phase 4 · 过闸候选的调研三闸（Task 9）

### 候选 1 · A3 iPhone 备份

**裁决段：放弃（本轮）。**

依据：需求与付费证据真（5 DIY 仓连续 + iMazing 付费在位），但利基窄、系统吸收近、粗算不成立；当前农场不缺「又一个零收入小工具」。

**红队（doubt-driven）：**

1. **付费形态不对位**：iMazing 是全功能 iOS 管理器，不是「照片备份」单点；3 笔买断自白买的不是拟做形态。
2. **系统吸收**：Apple 修 Windows 端或 MS Photos 修导入 → 需求消失（失效标志已写死）。
3. **技术依赖**：`pymobiledevice3` 是 Python 生态，Rust 产品需 FFI/子进程，维护面扩大。
4. **用户会自己造**：5 个 DIY 仓 = Gate 1 的反证；3uTools/iMazing 免费层已覆盖大部分场景。
5. **人群交叉窄**：iPhone + Windows + 在意照片备份 + 愿为单点买断，四重交集。

**一页粗算：**

| 项 | 数 | 依据 |
|---|---|---|
| 渠道可见流量 | 月 50–150 商店展示 + 200–500 GitHub/内容浏览 | 农场第一方实测（pomodoro 46 展示/月；log 14 天 180 views） |
| 转化 | 0.1–0.5% | 8 渠道 ≈570 阅读→24 安装→0 付费 |
| 单价 | ¥29–49 | iMazing 邻价；农场买断区间 |
| 月收入期望 | ≈¥0–37 | 上限 150×0.5%×¥49 |
| 工时 | 15–25 人日 | 移动设备协议 + GUI + 备份事务 |

**结论：放弃/挂起，不进 spec。**

### 候选 2 · A4 多显示器管理

**裁决段：放弃（本轮）。**

依据：需求与付费硬（3 笔买断自白），但 DisplayFusion 功能护城河深、Win11 内化风险高、单人难做出差异；粗算不成立。

**红队（doubt-driven）：**

1. **形态对位弱**：3 笔自白买的是 DisplayFusion 全功能，不是「轻量多显示器管理」。
2. **系统吸收**：Win11 逐屏任务栏 / 原生窗口位置 profiles 持续内化（失效标志两条）。
3. **PowerToys 补位**：profile 管理被第一方工具箱收走的概率不低。
4. **用户会自己造**：AHK 手搓潮 = Gate 1 反证。
5. **功能广度打不过**：单人只能做窄切口，而窄切口的付费证据未单独成立。

**一页粗算：** 同上（月 50–150 展示 + 200–500 浏览；转化 0.1–0.5%；单价 ¥29–49 → ≈¥0–37/月；工时 15–25 人日）。

**结论：放弃/挂起，不进 spec。**

---

## Task 8 · 总结论

- **复审覆盖**：C 2 + A 5 + B 6 = 13 条目；去重后 12 条（A1 = luoban；物格跨卷只审一次）。
- **三道闸全过**：A3 iPhone 备份、A4 多显示器。
- **调研三闸后**：两条均判 **放弃/挂起**（需求真、生意不成立）。
- **本轮结论：判无——零可开工候选。**
- **开新域扫描**：按 spec 不自动开，**请示用户**。

**本轮新增死因清单：**

| 条目 | 死因 |
|------|------|
| dropshelf | 免费全链已覆盖（Gate 2） |
| luoban | 一次转换，gzh-design-skill 可直接替代（Gate 1） |
| Hazel | 手搓簇 ≥40 仓 + AI 文件助手；规则搬家可被一个提示词覆盖（Gate 1） |
| 徒弟 | 通用 agent 已先发布 + 付费证据全在企业侧（Gate 1/2） |
| 采访员 | 培养路线证据弱 + 无付费（Gate 2/3） |
| 自传 | Recall 系统吸收 + 一次生成 + 隐私（Gate 1） |
| 主张器 | 只剩跟单骨架 + 无同形态付费（Gate 1/3） |
| 预演台 | 核验活可被 agent 做 + 黄牛付费形态不对位（Gate 1/3） |
| 物格 | 物理 ¥15 册子够用 + 自造隐喻 + 增量价值未证（Gate 2） |
| A3 / A4 | 过三道闸，但粗算 ≈¥0–37/月 vs 15–25 人日（调研三闸） |
| A2 log 家族 | 未审；10-21 校准钟后复议 |

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
