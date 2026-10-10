# 外来文本查看器（A2）· intent 文档（2026-10-09 开工）

> 用户指令（2026-10-09）：「A2 外来文本查看器 开干」——**用户口令提前开闸**（原排程=10-21 校准钟复议，见 `rewalk-rounds1-8-newguide-2026-10-09.md` 活物队列）。
> 流水线：深度调研收口 → 本卷三闸（裁决段/红队/一页粗算）→ **spec**（spec→plan→build→review→code-simplify 五阶段逐段推进，spec 获批前不写产品代码）。
> 上游案卷：round7 B1 卡A（`product-selection-round7-2026-09-25.md`）+ D4 log 家族会材料（`deep-dive-t1-2026-09-25.md`）。

## 在册纪律（开工前置，覆盖/被覆盖显式记录）

1. **发布 hold 至 10-21 校准钟回填后**（D4 红线：校准窗口内上 CSV 叙事会污染 log 下载归因）。开发/建仓不受限；**任何对外发布动作（GitHub release / MS Store 提交 / 传播帖）不早于 10-21 回填完成**。
2. **组合纪律让位条款**：若 Paddle hosted checkout 权限中途落地，log v1.x 收银台优先，A2 在当前阶段末尾暂停让位（A2 的自然暂停点 = spec 完成、build 未起）。
3. **边界红线**：只读「查看器」，不滑向 CSV 编辑（表格手术刀已毙：SmoothCSV3 开源双热 + Tablecruncher 2026-09）；与 danqing-log「不做编辑」边界共守。
4. **产品坟核对**：不碰 clipboard/xirang/disk/tile；A2 与 tile 无邻接，与 log 是家族邻接（引擎复用是卖点不是坟）。

## 卡案底（round7 B1 卡A，2026-09-25 立案·强）

- **需求**：「不是你写的文本」桌面查看器——大 CSV/日志/导出 dump（外来数据文件：大小/编码/格式不由人选择）。
- **谁在凑合**：Excel 开 581 万行 CSV **静默丢行**（MrEditor README，2026-06-26「Nothing on the screen says they are gone」）；VS Code 330MB 崩溃（#310440 2026-04-16）+ 大文件渲染损坏（#311373）；NPP 1MB SQL 卡死、**双人重复同一 workaround 勾 Word wrap**（#18258 2026-07-27）+ 大 XML Access violation（#18269）；手工拆文件土法（paperback #929 2026-09-23）。
- **自制替代品潮**（README 动机=凑合史）：Bascanka 10GB+ 编辑器 ★157（2026-02）/ csv-grid-editor（03）/ MrEditor ★28（06）/ Caxton「multi-GB logs and CSVs past spreadsheet row limits」（07）/ 浏览器 CSV Show HN（09-08「big popup asking for my credit card」）。
- **证据强度**：7 独立来源 ✓；付费单列 ✓（EmEditor Pro 仍收费 + HN 2025-10-12「Lifetime licenses…Beyond compare and Emedit」= 掏钱自白）。
- **需求温度**：活（2026-02→09 新票持续）。
- **推断链**：通用编辑器按「你写的代码」优化 × 外来数据大小/编码/格式不由人选择 × 导出与 AI 产出体量暴涨 × mmap 让 10GB 桌面可读 → 「文本你没写」专用查看器（mmap + 编码可靠 + 列对齐 + 不静默丢行）。断环（立项时）：付费买家画像（企业 IT vs 个人）无数据。
- **失效标志（开枪前须全档复盘）**：① EmEditor/VS Code 双修大文件+编码且工单关闭；② 2026-10 起 3 个月新工具仓星标中位 <5 且零 push；总失效：2026-12-31 前新仓季度数归零 **且** EmEditor 降价/免费化。

## D4 已有素材（log 家族会，2026-09-25）

- **引擎复用**：~70–80%（danqing-logfile mmap/行步进/全文搜索 + danqing-encoding 编码检测现成；唯一硬缺口 = CSV 记录语义 `csv.rs`，对标 `jsonl.rs` 规模）。产品层复用 ~30–40%。
- **三形态**：
  | 判据 | A. log 扩展 | B. 姊妹产品 | C. 独立第八件 |
  |---|---|---|---|
  | 品牌叙事 | ✗ 日志卖点稀释 | ✓ 各自干净，同引擎家族 | ✓ 但复用叙事不可见 |
  | 商店搜索词 | ✗ 挤掉日志词 | ✓ 双 listing 交叉引流 | 双货架但冷启动重来 |
  | 付费层定价 | 两难 | ✓ 继承 $29/$59 锚 | 无锚需重做定价 |
  | 开发带宽 | ✗ 压 log 主线 | 半 | ✗ 最重 |
  | 组合纪律 | ✓ | ⚠️ 灰（事实双活跃） | ✗ |
- **粗算素材**：topic:csv-viewer 72 仓 vs log-viewer 379 仓（csv 侧桌面 GUI 稀薄）；CSV 人群含 log 够不着的**非开发业务岗**（Excel 丢行受害者）。
- **命名**：LogLens 前缀族名 = 借信任也借稀释；MS Store 名称发布后不可改。

## 深度调研（2026-10-09 发出，3 路 · 75 分钟盒）

| # | 流 | 覆盖调研三闸项 | 状态 |
|---|---|---|---|
| D1 | 竞品 open issues 逐条扒（EmEditor/010/2026 自建潮/免费侧）+ 失效标志复核 | 竞品扒料 | ✅ 已回 |
| D2 | 定价调研（价格带/买断自白）+ 需求验证补强（Excel 丢行第二三来源/非开发人群） | 定价 + 需求验证 | ✅ 已回 |
| D3 | 系统吸收距离（Excel/VS Code/Explorer checkbox 判题）+ AI 需求迁移（增量人群工作流形态） | 系统吸收距离 | ✅ 已回 |

### D1 压缩摄取（2026-10-09）

- **竞品格局**：EmEditor Pro 订阅 $45/年 或 买断 $60/席（`emeditor.com/buy/` 实抓）；010 Editor $59.95/$149.95（hex 向，另一物种）；Tablecruncher 1019★ 已 GPL 开源但 **无 GBK**（只 UTF-8/16/Latin-1/CP1252）；Bascanka 158★「10GB+」宣称被用户打脸（issue #36 2026-07-30）且 **7 个月零回应、push 停 2026-02**；**MrEditor 35★（Mac-only）定位语与本品逐字撞车**「The Mac editor for text you didn't write」，2026-10-08 仍发版——**Windows 侧同定位空着**；VisiData/qsv = TUI/CLI 不接「双击看 dump」人群。
- **issue 缺口五簇**（15 条实取，URL 在原报告）：①行数/行边界不可信（csv-grid-editor #32 多行单元格被切开「rows around it are destroyed」2026-08-19）②列对齐/列宽截断 ③分隔符/换行语义（检测失效/写不进换行/CRLF 被静默改写——#31「saving writes back something different from what was read, without saying so」）④打不开/挂死/假完成（Bascanka 10GB 拒开、Tablecruncher 5GB Windows 挂死、EmEditor 6GB 挂死后「假加载完成」）⑤编码损坏（NPP #18356 转换污染剪贴板 2026-09）。**结论：没有一家把「行数可信+不改写数据+编码可靠」三件套同时做成产品承诺。**
- **⚠️ 红队级修正（叙事必须带）**：MrEditor 的「Nothing on the screen says they are gone」作者本人对冲——「whether a dialog appeared while it was loading, **I did not check**」。「Excel 丢行」= 硬事实（1,048,576 上限截断，5,816,535→1,048,576 数字自洽）；**「静默/零提示」降级为待验证**，引用必带 hedge。第二三独立源本轮未补到（SO 403/微软 specs 页 404/全库搜索 spam 污染）。
- **失效标志复核**：①**未兑现**（VS Code #310440 崩溃/#311373 渲染损坏 2026-10-09 仍 open；NPP 两张已修关但编码族 #18354 仍 open）——标志继续有效。②10 月新仓样本星标中位≈0，但**判定窗是 3 个月，不提前判**（1 月龄仓星标天然低；H1 自建潮也只有 2-3 家持续 push——新工具死亡率高是常态）。EmEditor 论坛转 read-only（半失效源）。
- **给 spec 三刀刃**：Windows + 只读查看 + **丢行可见**，不摊薄；「快速瞄一眼」让给 Peek。

### D3 压缩摄取（2026-10-09）

- **18 个月吸收概率：低（捆绑包整体）/ 中（「快速瞄一眼」切片）**。Excel 行上限十年一动且无「防丢行/防嗅探」工单方向（规格页 404 引证缺失）；VS Code 2026 还在收大文件崩溃且社区修复 PR #334861 当日被关未合；Explorer Peek 连纯文本回退 PR #50020（2026-08-19）都没合入。捆绑包（编码可靠+不静默丢行+引号内换行+列对齐+mmap 秒开）= 产品级工程，非 checkbox。
- **AI 迁移判「没接走，有反证」**：Copilot in Excel 试了「clueless」（HN 2026-02-05）；1GB CSV 传不上（2025-04-09）；「paste CSV into prompt → hot garbage」（2025-10-28）；agent 侧同样截断（VS Code #311068 MCP 静默截断 2026-04-17 open）；「装了 Claude Code，Sublime 看大文本使用不变」（2025-07-05）。**信任/核对场景天然反 AI**（与「不丢行」灵魂同构）。中期威胁=边界移动（上传上限打开/Copilot 补课/本地 agent 读文件成熟），失效标志三条已写死（见下）。
- **免费侧新情况（红队必打）**：「Windows 原生 GUI 只读秒开」层 2026 被反复发明——**leanrows（2026-08 建，0★，与拟做品几乎逐字同构）**/abriCSV 4★/timeline 1★/less-sheet（macOS/Linux，Show HN 2026-09-04，「closer to macOS's Preview than to visidata or excel」=「看一眼」被识别为独立品类）。全 0–4 星 = 赛道有需求无赢家，但 **「vacant because worthless」天然实验对质**（wenyan-pc 教训）必须进红队。VisiData 9326★ 后台全量加载非 mmap 秒开。
- **失效标志三条（AI 边界移动，写死）**：① Copilot-in-Excel 能正确回答「这列某值有多少行」类问题（行为实证）；② 主流 AI 助手稳定处理 ≥500MB CSV 且不丢行（官方 changelog）；③「不再打开查看器，直接把 CSV 给 AI」多人独立原话 ≥3 来源。

### D2 压缩摄取（2026-10-09）

- **价格带**：品类地板 = **$29–59 个人买断 + $149 商用买断**（SmoothCSV $29 / Modern CSV $39 Personal·$59 Business / Rons Data Edit $35 Pro；010 $59.95/$149.95 永久）；MS Store CSV 类买断 ¥33/¥49 CNY（JSON-LD 实抓）；EmEditor 现售**纯订阅**（$45–60/年，buy 页 lifetime 0 命中）+ UltraEdit 订阅/永久双轨。**订阅在本品类自带负评**（HN 2026-02「has a subscription fee…might as well use VS Code」）。与 log $29/$59 锚完全重合。
- **付费态度（比预期暖）**：买断自白实锤——「Lifetime licenses…Beyond compare and Emedit」（HN 2025-10-12，锚点复核命中）；Modern CSV 帖（HN 2023-08-16，324 分）「I might just buy a license now」+ **「my employer bought the business version…about 20 or 25 licenses」=企业批量买断实证**；用户自发拿 $29 一次性价做锚；「hate subscriptions for desktop software」跨年连续（2023–2026）。买断+大版本复购模型有原话（「Release a v2? Sure, I'll probably buy again」2026-02）。
- **需求补强（12 条行为证据，2010→2026 跨 16 年不断）**：Excel 另存丢数据（SU 2012）/导入截断丢行（2017）/**静默改坏基因名约 20% 论文**（Genome Biology via SU 2017）/字符串强转日期不可逆（2018）/行上限拆文件土法（2016、HN 2020「split up a CSV too big for Excel」）/50 万行 Excel 崩溃且不肯碰数据库（2018）/**60GB 表格律所咨询公司直接开 Excel**（HN 2017，画像「people who don't know what a command line is」）/PM 上 Postgres 扛 Excel 溢出（2021）/传感器日志 CSV（2017、2022/2014 GitHub issue 实证）/**Rons 官方 testimonial：账单 CSV「Excel could not handle」+「excel starter has stopped functioning」**（2026-10-09 抓取）。
- **非开发人群命中 ×6**（财务/遗留数据/分析师/咨询律所/PM/账单）——与 log 人群不重叠，D4「CSV 人群含 log 够不着的业务岗」补强成立。
- **温度**：活，未衰减（2025–26 SmoothCSV/Tablecruncher/Modern CSV 324 分讨论仍在进场；「too big for Excel」是 HN 惯用语）。老证据多在 2012–2018 密集带，但根因（行上限/类型嗅探）未修，2023–26 新票持续。
- **局限**：EmEditor 终身授权现售未证实（用户持有=历史售卖）；Reddit 丢 r/excel 原声；中文非开发求助未采（WebSearch 禁用）；MS Store 为 CNY 市场。

## 合成（2026-10-09，三路齐）

| 题 | 判定 | 最硬一条 |
|---|---|---|
| 需求 | **过**（≥3 来源+16 年行为+非开发 ×6） | 「too big for Excel」行为簇跨 16 年；Rons testimonial「Excel could not handle」 |
| 付费 | **暖**（判据①已废不作闸，仅披露） | 企业批量买断 ×20-25 席（Modern CSV）+ 终身授权自白 + 品类价格带 $29–59 与 log 锚重合 |
| 供给 | **空位在「行数可信+不改写+编码可靠」三件套承诺** | 无一家承诺三件套；MrEditor 逐字同构但 Mac-only；Windows 同构件全 0–4 星 |
| 吸收/迁移 | **低吸收 + AI 未接走**（反证在场） | VS Code 大文件工单仍 open；Copilot in Excel「clueless」 |
| 引擎 | 匹配高（logfile+encoding 现成，csv.rs 唯一硬缺口） | D4 复用率 70–80% |

**v1 形状（被证据塑形）**：Windows 单文件 GUI · **只读**查看器 · mmap 秒开 · **行数可信/丢行可见**（Excel 受害者的第一痛点）· RFC4180 语义（引号内换行/分隔符检测不瞎猜）· 编码可靠（GBK 等 CJK 正是 danqing-encoding 主场，Tablecruncher 无 GBK=差异化）· 列对齐。**不做**：编辑（表格手术刀已毙）、「快速瞄一眼」（让给 Peek）、分析/SQL（让给 VisiData/qsv/AI）。

**风险清单（进红队）**：①「静默」叙事有 hedge（MrEditor 作者自认没验过有没有对话框）；②「vacant because worthless」天然实验对质（leanrows/abriCSV/timeline 全 0–4 星——同构件没人要的假说 vs 无分发的假说）；③ 新工具死亡率极高（H1 浪潮仅 2-3 家持续 push）；④ 非开发人群渠道触达（MS Store 搜索词 vs GitHub——非开发不在 GitHub）；⑤ 分发瓶颈照旧（全品粗算≈0）；⑥ 三形态/命名未定。

## 红队（doubt-driven，2026-10-09）+ 补环

**抽查账**：31 项 0 编造；硬数字/状态大头对；**实锤误读 1 处**（见补环）；D1 价格误读 1 处（EmEditor「买断 $60/席」实为**订阅年价 $60/年**，buy 页 lifetime 0 命中——已正）；未验 3（Rons testimonial 复核失败 / MS Store ¥33-49 价位 / 弱）；「引文对、语境外推」6 处。数据底座不降级，修辞需收。

**七刀判定**：刀1「静默」部分成立 / **刀2 worthless 天然实验成立（最重）** / 刀3 非开发=画像≠可达性（偏重）/ 刀4 付费日期分布部分成立（2023 密集带）/ 刀5「行数可信」验证成本低估 / 刀6 交叉引流无数据+LogLens 前缀对非开发是负资产 / 刀7 免费地板·Power Query·系统侧监控·BI 免费层四路自由攻击。

### 补环（裁决前必须，2026-10-09 本 session）

1. **误读改正**：「my employer bought the business version…and it bought multiple licenses (about 20 or 25 whataburgers)」——**20/25 是 whataburger 计价的总价玩笑，不是席位数**；席位只说「multiple」。合成表「企业批量 ×20-25 席」作废，改「企业批量多席（数量不详，2023-08 单帖）」。
2. **付费段按日期分布重标**：品类付费自白密集带在 **2023-08**（Modern CSV 324 分帖）；2025-10 是**持有**历史终身授权（非当期掏钱、非 CSV 品类）；2025-26 活跃是供给活动非付费活动。**合成判定从「暖」改「暖但衰减待察」**——与农场「付费热情会衰减」元发现对齐。
3. **「静默」降级 + 卖点改写**：卡案底改「Excel 截断至 1,048,576 行，**加载完成后界面不显示已丢 476 万行**（加载期间有无提示，引用者未验）」。卖点从「丢行可见/不静默」改 **「行数可信 / 截断可见」**——正向属性，不依赖证成 Excel 静默。
4. **刀2 二假说书面对质（强制作答）**：
   - **假说 B「座位不值钱」证据**：2017 YC F1 的 CSV Explorer（同定位 web 工具）九年无人起量；leanrows 强同构免费 0★ 两个月；Bascanka 158★ 后停摆；「看一眼」动作价值时长短。
   - **假说 A「无分发」证据**：品类存在长期收费在位者（Modern CSV $39/Rons $35/SmoothCSV $29 在售多年）= **「查看/整理外来 CSV」这个座位有人付钱**；0★ 受试者全体是**零分发裸发布**（GitHub 免费发布≈零自然流量是农场第一方校准）——0★ 是分发缺席的预期读数，不是价值判读；wenyan-pc 型天然实验要求「热土+可见」前提（619 仓/年），leanrows 们不在热土（topic:csv-viewer 仅 79 仓）；MrEditor（Mac）持续发热 = 形态在被要。
   - **判：未决**——天然实验因「全体受试零分发」**失效**，不构成 worthless 正面证据；但「九年无人做成」base rate 仍压着。**判决实验 = 本品发布本身**（见失效标志 ⑤）。附：纯只读形态付费锚弱于品类（付费实证多在编辑器侧：Modern CSV/Rons Pro 含保存、SmoothCSV 是编辑器）——定价形态（纯买断/免费层+买断）留 spec/POC 阶段。
5. **GBK 缺口标注**：Tablecruncher 无 GBK 是供给侧事实；「我需要 GBK」需求侧原话**本轮未采到**——不作卖点主证据，作 CJK 人群适配（danqing-encoding 复用是工程事实不是需求证据）。
6. **画像≠可达性**：合成表拆两列——「非开发 ×6」是需求画像证据，**渠道可达是独立缺口**（非开发不在 GitHub；MS Store「CSV」货架**有在位应用**（CSV-Wizard 等，多为免费），「货架空」不成立）。中文非开发求助面（知乎/贴吧/小红书）本轮未采。
7. **工时上调**：csv.rs+对抗语料（非法引号/混合换行/嵌入 NUL/超长行）3–5 → **6–8 人日**；v1 总计 **15–22 人日**。「可信」须在 spec 给可证伪定义（文件不合法时显示「行数可能不真」也算兑现承诺）。
8. **失效标志补三条**（连同 D3 三条 AI 边界 + 原两条，全档）：⑤ **worthless 判决**：发布 90 天 GitHub 0★ 0 issue **且** MS Store「CSV」词 impressions 折算低于阈值 → 座位不值钱判实，代码入坟；⑥ 滚动 12 个月 CSV 工具买断自白 = 0 → 付费衰减坐实；⑦ 实测 Excel 新版截断时有明显提示 → 删「截断可见」话术。另：监控面加系统侧（Win11 Notepad 大文件/Peek PR #50020 合入）。
9. **非需求证据隔离**：csv.rs 引擎资产/三仓格局/「免费层即仪器」叙事 = 内部价值，**显式不计入需求判定加分**（心法一）。
10. **命名判据写死**：非开发为主 → **不用 LogLens 前缀**（「Log」词对财务/律所人群是错信号）；MS Store 名称发布后不可改。

## 一页粗算（v1，三闸③ · 第一方转化 0.1–0.5%，卖方话术禁用）

- **单价锚**：$29 个人 / $59 企业买断（品类价格带 × log 锚双背书；MS Store 侧可上 ¥33 档对位）。
- **工时粗估**：csv.rs 记录语义 3–5 人日 + 列对齐查看视图 3–5 + 编码/丢行可见 UI 2–3 + 打包发布 2–3 + 测试 2–3 ≈ **12–18 人日**（比 log v1.0 轻，引擎复用高；plan 阶段细排）。
- **触达档 A（现状）**：MS Store 自然量 ~26 安装/月 × 0.1–0.5% × $29 ≈ **$0.08–0.38/月 ≈ 0**；内容投放上限（600 安装/年 × 0.5% = 3 单）≈ **$87/年**。
- **读法（如实）**：直接收入期望 ≈ 0——与 pomodoro/log 同构，第一方口径下农场任何单品都趋近零，**本品不例外**。价值另账：①非开发人群是农场从未触达的受众，MS Store「CSV」搜索词是新货架实验；②csv.rs 成为引擎资产（三仓格局 encoding/logfile/csv）；③「行数可信」是可讲述的 10x 长板叙事（对齐免费软件三模型的「免费层即仪器」）。粗算按心法一属知情权裁决输入，**不作否决**。

## 裁决入口问题（三闸裁决段须答，D4 遗留）

1. 卡A 目标是「补 log 的 CSV 读入」还是「Excel 丢行人群独立产品」？（D4 倾向后者=叙事独立）
2. 三形态选 A/B/C？（D4 倾向 **B 姊妹产品**）
3. 若走 B：姊妹是否一 key 双产品？（产品线定价题）

## 裁决段（红队过稿+补环后 · 呈用户 · 继续/放弃/转向）

**态势一句话**：需求侧过（16 年行为簇+非开发画像 ×6）；付费「暖但衰减待察」（密集带 2023）；供给空位在「行数可信/截断可见」三件套承诺（但 worthless 天然实验未决——零分发受试使实验失效，九年 base rate 仍压着）；吸收低+AI 未接走；引擎匹配高；v1 15–22 人日；直接收入期望 ≈0（全品同构）。

| 选项 | 内容 | 代价/收益 |
|---|---|---|
| **a（agent 推荐）** | **修卷后进 spec**：形态 **B 姊妹产品**（独立命名不用 LogLens 前缀；一 key 双产品=否，各自独立 key，收银台复用 log Paddle 轨）；目标=「Excel 丢行人群独立产品」；v1 刀刃=**行数可信/截断可见**+RFC4180+GBK+列对齐+mmap 秒开，只读；失效标志全档 ①-⑦ 写进 spec；发布 hold 至 10-21 后；让位条款（Paddle 落地→log v1.x 优先，暂停点=spec 完成 build 未起） | 15–22 人日；座位值钱与否由 90 天失效标志⑤判决（本品即实验） |
| b | 先免费探针后 spec：最小免费查看器（核心=行数可信）90 天判决，数据回来再定付费层（round8 探针 b 同款） | 少花付费层工时；但 v1 本来就薄，探针与正片差 3-5 人日 |
| c | 暂停：worthless 未决不踏实，排到 10-21 校准钟后与 Drop Shelf 一起排序 | 保住带宽；但「开干」口令已下，等待无新信息可买 |

**agent 推荐 a 的依据**：① 刀2 未决不构成否决——天然实验已失效（全体受试零分发），按心法一「执行问题不许冒充需求问题」，分发缺席不是需求死亡；② 品类付费在位者存活多年（$29-59 在售）=座位有人付钱；③ 90 天失效标志⑤本身就是 worthless 的判决实验，输的成本 = 15-22 人日的一半（免费层先行可再降）；④ 与 log 引擎复用 70-80%，是全队列里边际成本最低的一枪。

**用户裁决（2026-10-09）：a** —— 进 spec。锁定：形态 **B 姊妹产品**（独立命名**不用 LogLens 前缀**；一 key 双产品=否，各自独立 key，收银台复用 log Paddle 轨）；目标=「Excel 丢行人群独立产品」；v1 刀刃=行数可信/截断可见+RFC4180+GBK+列对齐+mmap 秒开只读；失效标志全档写进 spec（intent 编号 ⑤ 有重叠，spec 内理顺为 FM1–FM8 共 8 条 = 原 2 + D3 3 + 补环 3，总失效并入 FM2）；发布 hold 至 10-21 回填后；让位条款生效（Paddle 落地→log v1.x 优先，暂停点=spec 完成、build 未起）。spec **已获批**（2026-10-09 用户「go」，四题按推荐落定：名=PlainSight（明见）/免费层+买断/搜索 v1.x/csv.rs 入 logfile）：`danqing-plainsight/docs/specs/foreign-text-viewer-spec-v1-2026-10-09.md`（2026-10-09 迁入产品仓）。

## 执行进度

| 步骤 | 状态 |
|---|---|
| 用户口令开闸 | ✅ 2026-10-09 |
| 本文档落盘 | ✅ 2026-10-09 |
| 深度调研 3 路（75 分钟盒） | ✅ 2026-10-09 三路全回 |
| 合成 + 一页粗算 | ✅ 2026-10-09 |
| 红队（doubt-driven，31 项抽查） | ✅ 2026-10-09 |
| 补环（误读改正/付费重标/静默降级/二假说对质等 10 项） | ✅ 2026-10-09 |
| 裁决段（用户） | ✅ 2026-10-09 裁 **a**（B 姊妹产品进 spec） |
| spec（五阶段之一） | ✅ 2026-10-09 获批（用户「go」，四题按推荐） |
| plan + todo（产品仓 tasks/） | ✅ 2026-10-09（plan.md：T0–T8/C1–C4/FM6 阈值/风险表） |
| 建仓 danqing-plainsight（六件套+lock 钉 rev） | ✅ 2026-10-09（零 commit 待收口点头） |
| build T1 引擎 csv.rs（danqing-logfile） | ✅ 2026-10-10 C1 过（28 新测/127 全绿/clippy 零警告） |
| build T2–T8（/build auto 全跑） | ✅ 2026-10-10 C2/C3/C4 全过（40 测全绿；8 提交含 init；1.2GB 开 94ms 实测；对抗样本 8/8；便携包已出；**发布 hold 至 10-21 生效**） |
| review + code-simplify（五阶段余两段） | 待办 |
