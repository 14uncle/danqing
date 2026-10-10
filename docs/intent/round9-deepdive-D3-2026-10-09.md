# D3 深调研 · 官方迁移风险 / 系统吸收距离 / AI 迁移 · Drop Shelf(2026-10-09)

> 上游:`deep-dive-round9-t1-2026-10-09.md` 卡A。本卷只答 D3 四小题。
> 判别问题:「这个功能离 OS/平台的一个 checkbox 有多远?18 个月内被顺手做掉的概率多大?」+ AI 迁移。
> 取证纪律:gh 全路径 `\"C:\Program Files/GitHub CLI/gh.exe\"`;WebSearch 本会话全程空转(记局限);断言贴 issue/PR/原话,贴不出降级「推断」。

## 结论先放

1. **系统吸收距离:不是 checkbox,但也不是「官方永远不做」。** Windows 本体无任何路线图/checkbox 证据;PowerToys 有 **7 年需求链 + 2026-08 社区贡献者已写好 MVP 规格并获放行**,但规格搁置 7 周未获架构裁决、无实现 PR、无 milestone。CmdPal 在建的是**置顶书签架**(持久启动目标),不是暂存架(临时持有+拖出)——语义相邻、形态未覆盖。
2. **AI 迁移:未发生。** 2025-26 AI 文件工具全是「目录自动整理」,无人接管「多源收→暂存→一次发出」;反向效应是 vibe-code 让手搓暂存架变容易(与卡A 推断链一致)。
3. **失效标志③ 18 个月兑现概率:中(35–50%,偏 PowerToys 侧)。** Windows 本体 5–15% / PowerToys 35–50%。最大变数 = dark13b 的 MVP 能否落地;最大证据缺口 = WebSearch 空转、Windows Feedback Hub 不可达。

---

## 一、Windows / PowerToys 路线图证据

### 1.1 Windows 本体(checkbox 距离)

| 断言 | 证据 | 日期 | 强度 |
|---|---|---|---|
| Windows 本体无「拖拽暂存区」设置/功能 | 检索面:PowerToys 全库 issue + HN;**未见任何 Windows 本体路线图或 checkbox 提案**;WebSearch 空转 | 2026-10-09 | **弱(源受限)**——「搜不到 ≠ 不存在」,记局限 |
| 历史上有过 Drag Tray 且被移除 | OmniTray 提案原话:「Drag Tray was removed from Windows. Omni Tray is a proposal to revisit that interaction」(microsoft/PowerToys#50126 body) | 2026-08-26 | 中(提案者原话,未核 Windows 版本) |
| Windows 近期动向在 AI/Copilot,非文件暂存 | 本会话无第一方路线图可引 | — | 推断(不作证据用) |

**结论(推断级)**:Windows 本体离 checkbox 远——没有 checkbox,没有路线图发言,没有 Insider 证据。18 个月内 inbox 拉一个「文件暂存架」的概率**低**。

### 1.2 PowerToys 需求链(硬证据,gh api 可复核)

| # | Issue | 创建 | 状态 | 标签 | 内容要点 |
|---|---|---|---|---|---|
| 1 | [microsoft/PowerToys#128 Temporary File Bin](https://github.com/microsoft/PowerToys/issues/128) | **2019-05-29** | **OPEN** | `Idea-New PowerToy` | 「drag and drop a file in corner of a screen to keep a reference… to open or copy or drag and drop later」;2019-06-12 indierawk2k2「Tagged for the backlog」;2023-03-08 评论举 Mac 三件套 Dropover/Dropzone/Yoink;2024-10-05 wada3n 点名 Dropshelf |
| 2 | [microsoft/PowerToys#37136 QuickShelf](https://github.com/microsoft/PowerToys/issues/37136) | **2025-01-29** | OPEN | `Idea-New PowerToy` + `Needs-Triage` | 原文点名「There is a software called "Dropshelf"… Please consider adding a similar feature to Powertoys」;并自白 Dropshelf 质量差:「it crashes frequently and causes the system to freeze」 |
| 3 | [microsoft/PowerToys#47589 Drag to a movable clipboard (like Dropshelf)](https://github.com/microsoft/PowerToys/issues/47589) | **2026-05-02** | OPEN | — | 西语用户「hay una aplicación que hace esto Dropshelf」,1 评论无团队回复(弱) |
| 4 | [microsoft/PowerToys#50126 Proposal: OmniTray](https://github.com/microsoft/PowerToys/issues/50126) | **2026-08-26** | **CLOSED 2026-09-30(作者自关)** | — | 完整「drag, drop, stack, shelf anything」提案 + 原型 [jiripolasek/OmniTray](https://github.com/jiripolasek/OmniTray)(建仓 2026-08-25,**2★**,push 2026-08-31);zadjii-msft 2026-09-02 原话:「Is it a PowerToy or a standalone app? … everything I see here seems like it is good enough to live as its own thing」 |
| 5 | [microsoft/PowerToys#51046 Shift+shake 合并 Explorer 窗口为标签页](https://github.com/microsoft/PowerToys/issues/51046) | **2026-10-06** | OPEN | — | 同痛点不同解:「I often end up with several File Explorer windows open… gather them into one window」——窗口合流派,不走暂存架 |
| 6 | [microsoft/PowerToys#39940 Mouse Jump 文件拖放](https://github.com/microsoft/PowerToys/issues/39940) | 2025-06-07 | OPEN | — | 跨屏拖文件到 Mouse Jump 缩略图(相邻需求,非暂存) |

**七年间(2019→2026)至少 4 条独立请求点名要「文件暂存架」**,其中 2 条明确点名商业产品 Dropshelf——需求在 PowerToys 公共区持续浮出。

### 1.3 关键动态:dark13b 的 MVP 规格(2026-08-21)

[#128 评论 2026-08-21](https://github.com/microsoft/PowerToys/issues/128),用户 **dark13b**:

> 「Thanks @niels9001 for the go-ahead in #28769. Before implementation, here is a proposed narrow MVP specification for discussion.」

MVP 范围(摘要):
- 热键开紧凑 shelf → 拖入文件/文件夹 → 别处导航 → **拖回出去**/打开/移除/清空
- 「Adding an item must never copy, move, rename, upload, or otherwise modify the source item」
- 会话内内存持有、不持久化、去重、键盘可达、混 DPI
- **明确不做**(v1):URL/文本片段、shelf 自己复制移动、云上传/压缩/转换/自动化、多 shelf、搜索预览、Explorer 扩展
- 架构问题待裁:「separate PowerToy only if maintainers consider those temporary transfer semantics distinct enough. Otherwise… Command Palette enhancement」

**进度事实(2026-10-09 核查)**:
- #128 评论共 6 条,**最后一条即 2026-08-21 的 MVP 规格**——7 周无维护者回复
- dark13b 在 PowerToys 的 PR 只有一条无关的 [PowerRename CSV #49538](https://github.com/microsoft/PowerToys/pulls/49538)(2026-07-28, open);**无 shelf 实现 PR**
- #128 无 assignee、无 milestone,标签仅 `Idea-New PowerToy`
- dark13b 在 #28769(贡献征召帖,465 评论)报到后获一般性放行;niels9001 是否对**本功能**点头未见单独记录(「go-ahead」出自 dark13b 转述)

### 1.4 CmdPal「Shelf」系列——语义辨析(相邻但不同物)

| PR/Issue | 日期 | 实际做了什么 |
|---|---|---|
| [CmdPal Dock 拖拽书签 #47989](https://github.com/microsoft/PowerToys/issues/47989) | 2026-05-19 创建 → 2026-05-22 关 | 拖文件/URL 到 Dock **立即生成书签并固定**——持久启动目标 |
| [CmdPal: Shelf (3/n) 拖拽 #49932](https://github.com/microsoft/PowerToys/pulls/49932) | 2026-08-15 → **merged 2026-09-29** | 「Quick access shelf」:置顶项重排、recent→pinned 提升——**图钉管理** |
| [Drawer (8/n) #50282](https://github.com/microsoft/PowerToys/issues/50282) | 2026-09-01 | 按 shelf 位置定制右键菜单 |
| [pin drop zones #51051](https://github.com/microsoft/PowerToys/pulls/51051) | 2026-10-06 | CmdPal dock 分区感知的固定投放区 |

**语义差(dark13b 原话,2026-08-21)**:「bookmarks are persistent launch targets, while this shelf would be temporary, multi-item, session-only, and designed for dragging items back out.」

→ CmdPal 正在建的是**「钉在那儿的启动架」**,不是「临时暂存、一次拖出」。拖拽 UI 基建在长,但「暂存/合流」语义无人做。

### 1.5 「被顺手做掉」的具体距离

| 距离层 | 现状 | 离 checkbox 几步 |
|---|---|---|
| Windows 设置里的一个开关 | 不存在,无路线图 | **很远**(无 checkbox 可勾) |
| Explorer 自带功能 | 只有多标签(Win11);无暂存面板 | 远 |
| PowerToys 现成模块改改就有 | CmdPal Shelf=图钉;Dock=书签;**语义不重合** | 中——需要新语义,不是勾一下 |
| PowerToys 新 PowerToy | 需求在、MVP 规格在、实现者疑似在、**架构裁决未下** | 中——走的是「社区贡献新模块」长路径,不是顺手 |

参照反面教训(托盘禁闪 = checkbox 近在咫尺):本品**不是** checkbox 近邻。但也不是「官方永远不碰」——PowerToys 区有持续需求 + 有人在推。

---

## 二、邻近形态矩阵(收→暂存→发 覆盖度)

| 形态 | 收 | 暂存 | 一次拖出/发出 | 覆盖判定 | 证据 |
|---|---|---|---|---|---|
| **Files App(付费在位)** Shelf pane | 会 | 会 | 会 | **若建成则全覆盖——但未建** | [files-community/Files#12490](https://github.com/files-community/Files/issues/12490) **2023-05-30 开,至今 OPEN**(末次活动 2025-08-04);前身 [#6514](https://github.com/files-community/Files/issues/6514) 2021-10-15→2023-05-31 关闭合并入 #12490。#12490 规格详细(dropzone/去重/拖出复制),checkbox 仅少量勾,3 年未落地 |
| CmdPal Dock 书签 / Quick access shelf | 会(拖入) | 半(持久钉住) | 否(点开,非拖出) | **不覆盖暂存语义** | #47989(2026-05)、#49932(merged 2026-09-29) |
| OneDrive「文件请求」/ 云盘中转 | 链接收 | 云上 | 发链接 | **远程收集链**覆盖,不覆盖本地多源拖放暂存 | 推断级(第一方支持页本会话未取到,301/404;搜 HN title 无讨论) |
| 微信「文件传输助手」 | 会 | 会(自聊云存储) | 转发到人/设备 | **事实上的个人中转位**;但「多源合流→一次拖出到任意目的地(邮件/U 盘/客户目录)」不在其动作模型里 | 题目指定记为替代品;HN `weixin file transfer` title 检索 0 命中(局限) |
| 桌面「暂时放着」+ 多开 Explorer 互拖 | 会 | 会(土法) | 会(手工) | 凑合本身 | 卡A 案底:Holdem README 2024-09 / Bucket README 2026-06 |
| Explorer 窗口合并为标签页(#51046) | — | — | — | **同痛点、异形态**:用「窗口合流」消掉多窗互拖的痛苦,不提供暂存架 | 2026-10-06 |
| Stardock Fences / DropIt 等 | 分区/规则 | 部分 | 部分 | D2 供给体检管;本卷只记「邻近形态未见全链暂存架」 | (交 D2) |

**判定**:邻近形态里**没有任何一个已上线的东西完整覆盖「多源收→临时暂存→一次拖出」**;最接近的两个(PowerToys 有人在推、Files App 开了 3 年 issue)都**未落地**。替代品(微信文件传输助手/云盘)走的是「上传云中转」模型,不是本地拖放暂存模型——动作链不同,是弱替代不是同物。

---

## 三、AI 迁移判断

### 3.1 2025-26 AI 文件工具在做什么(github 检索面)

| 仓库 | 建仓 | ★ | 形态 |
|---|---|---|---|
| TheSethRose/AI-File-Organizer-Agent | 2025-04 | 36 | Gemini 驱动**目录整理提案+执行** |
| musa-labs/maid | 2025-03 | 12 | Mastra+MCP 整理文件 |
| viditraj/OsAgent | 2026-03 | 9 | Windows 本地「整理文件+系统监控」 |
| YuriyGuts/ordnung | 2026-02 | 5 | 本地无云 API 文件整理 |
| Everfern-AI/Everfern | 2026-04 | 94 | 本机 computer-use agent(通用) |
| 其余 9 个 | 2025-26 | ≤9 | 目录整理/Drive 整理/demo |

**全部是「把已有目录理干净」,没有一个是「从多处收进来、暂存、发出去」。**

### 3.2 增量人群工作流形态(行为面)

- HN 检索(`agent send files email automatically`、`file picker agent`):命中的是 agent 用文件搜索工具(fmm.nvim)、VPS 间搬 ML 数据集(2026-07-11)、Slack agent(Caddie 2026-03)——**都不是「收交付物→发给客户」的人类动作**。
- HN title 检索 `drop shelf` / `yoink` / `file staging`:**0 条同品类讨论**(Yoink 同名是 Spotify 工具)。此品类不在 AI 叙事聚光灯下。
- **反向效应(行为证据)**:vibe-code 让手搓暂存架变便宜——Bucket(2026-06)/ ghost-drop(2026-07)/ omarchy-ledge(2026-08)/ OmniTray 原型(2026-08)都是 2026 年新手搓。AI 降低的是**做这个工具**的成本,不是**替代这个动作**。

### 3.3 判定

**AI 迁移 = 未发生(截至 2026-10-09,证据面内)。** 「收文件→发出去」仍是人的拖放动作;agent 在做的是整理既有目录。失效类型「需求迁移型」**不成立**。保留观察点:若 2027 出现「agent 自动收邮件附件并归档/发出」的普及工作流(行为,不是 demo),此判断作废。

---

## 四、失效标志③复核:18 个月兑现概率

**失效标志③原文**:「Windows 本体/PowerToys 上线拖拽暂存区(系统吸收兑现)」。

### 分解打分

| 路径 | 18 个月概率 | 依据 |
|---|---|---|
| Windows 本体 inbox | **5–15%(低)** | 无 checkbox、无路线图、无 Insider 证据;Drag Tray 曾被移除;WebSearch 空转使本侧取证最薄 |
| PowerToys 出模块 | **35–50%(中)** | 见下 |
| **合并(任一兑现)** | **≈ 中,35–50%** | 两路径近独立,Windows 侧贡献可忽略 |

### PowerToys 路径的推高/压低因子

**推高**:
1. #128 自 2019 挂 `Idea-New PowerToy`,2019-06 已「backlog」
2. **2026-08-21 有人写好了 MVP 规格**并声称获 niels9001 放行——历史首次「可开工」状态
3. 需求 4 条独立 issue(2019/2025/2026×2),2 条点名 Dropshelf 商业品
4. CmdPal 拖拽/钉架 UI 基建已存在(书签 #47989、Shelf 图钉 #49932)
5. 市场已验证:Dropshelf Pro 买家自白 ×5(卡A)

**压低**:
1. MVP 规格 **7 周零回复**(2026-08-21→2026-10-09)
2. 架构裁决未下(独立 PowerToy vs CmdPal 增强)——dark13b 明说在等这个
3. zadjii-msft(2026-09-02)倾向「这更像独立 app 而非 PowerToy」——官方口径分裂
4. OmniTray 作者自关提案(2026-09-30),原型仅 2★
5. 无 assignee、无 milestone;PowerToys 主力在 CmdPal/Window Hopper(2026-08→10 大量 Hopper PR)
6. 先例:同类新 PowerToy 长路径——Hopper 原型 #29823(2023-11 WIP)最终关闭;#37136 `Needs-Triage` 挂了 20 个月
7. 就算落地,dark13b 的 MVP 是**窄口径**(会话内、无持久化、无搜索、无自动化)——商业 Pro 差异位仍在

### 结论(证据强度措辞)

**失效标志③ 18 个月兑现概率 = 中(35–50%),主体是 PowerToys 社区贡献路径,不是 Windows checkbox。**

- 若 dark13b 在 2026-Q4/2027-Q1 提交实现 PR 并获合,概率升至 60–70%(PowerToys 发版周期 1–2 月)
- 若 2027-03 前 #128 仍无实现 PR,概率降至 20–25%(贡献者流失是 PowerToys 新模块主死因)
- **「顺手做掉」的字面形态(一个 checkbox)不成立**;成立的是「社区贡献新 PowerToy 覆盖同形态」——这对付费 Drop Shelf 的杀伤力等价(tile 死因:免费在位者)

### 取证局限(不许当「官方不做」用)

1. WebSearch 全会话空转(0 条有效结果)→ Windows 官方博客/Build/Insider 面未覆盖
2. Windows Feedback Hub 不可达
3. OneDrive File Request 支持页 HTTP 301/404,第一方原文未取到
4. niels9001 的「go-ahead」只有 dark13b 转述,无一手评论可贴
5. 「Drag Tray was removed from Windows」出自 OmniTray 提案者,未核 Windows 版本沿革

---

## 证据表(全源一览)

| # | 源 | 日期 | 一句话 | URL |
|---|---|---|---|---|
| 1 | PowerToys#128 | 2019-05-29→2026-08-21 | 临时文件箱请求+backlog 标签+2026 MVP 规格 | https://github.com/microsoft/PowerToys/issues/128 |
| 2 | PowerToys#37136 QuickShelf | 2025-01-29 | 点名 Dropshelf 请加进 PowerToys | https://github.com/microsoft/PowerToys/issues/37136 |
| 3 | PowerToys#47589 | 2026-05-02 | 「like Dropshelf」低质请求 | https://github.com/microsoft/PowerToys/issues/47589 |
| 4 | PowerToys#50126 OmniTray | 2026-08-26→09-30 | 全形态提案,作者自关;zadjii「像独立 app」 | https://github.com/microsoft/PowerToys/issues/50126 |
| 5 | jiripolasek/OmniTray | 2026-08-25 | 原型 2★ | https://github.com/jiripolasek/OmniTray |
| 6 | CmdPal Dock 书签 #47989 | 2026-05 | 拖入=持久书签 | https://github.com/microsoft/PowerToys/issues/47989 |
| 7 | CmdPal Shelf (3/n) #49932 | merged 2026-09-29 | 图钉拖拽重排 | https://github.com/microsoft/PowerToys/pulls/49932 |
| 8 | PowerToys#51046 | 2026-10-06 | 同痛点异形态:窗口合并为标签 | https://github.com/microsoft/PowerToys/issues/51046 |
| 9 | Files#12490 Shelf pane | 2023-05-30, OPEN | 付费 Files 三连年未落地 | https://github.com/files-community/Files/issues/12490 |
| 10 | Files#6514 | 2021→2023 关 | 并入 #12490 | https://github.com/files-community/Files/issues/6514 |
| 11 | dark13b 贡献面 | 2026-07-28 | 仅有 PowerRename PR,无 shelf PR | https://github.com/microsoft/PowerToys/pulls/49538 |
| 12 | HN title 检索 | 2026-10-09 | drop shelf/yoink/file staging 同品类 0 命中 | Algolia API |
| 13 | GitHub AI 文件整理簇 | 2025-26 | 全是目录整理,无收→发 | gh api search/repositories |
| 14 | PowerToys#39940 | 2025-06-07 | Mouse Jump 跨屏拖文件 | https://github.com/microsoft/PowerToys/issues/39940 |

---

## 执行记录

- 2026-10-09:D3 开工,读 T1 卡A + round9 取证纪律;PowerToys 定点 gh api 扫描(shelf/bin/staging/drop/Dropshelf/Hopper/Drag Tray 多关键词)→ 命中需求链 #128/#37136/#47589/#50126 + CmdPal Shelf 系列辨析;Files App #12490/#6514;OmniTray 原型;HN Algolia title 检索;AI 文件工具仓扫描。WebSearch 全程空转记局限。≤60 分钟取证 + 写盘交卷。
