# 调研原始报告附录 — AI 时代开发者生存判据

主文档: `ai-era-developer-survival-2026-09-20.md`
四路调研 agent 的原始报告按返回顺序归档于此，框架合成时逐条引用。

---

## D 路：信任/本地/买断溢价反题（2026-09-20 返回）

调研日期：2026-09-20。证据类型标注：【行为】=定价页/收入/事故报告等花钱或出事的事实；【言词】=观点/讨论。

### 问题 1：「无聊但可靠」付费软件 2025-2026 存活证据

**正向（还在收费且活着）：**
- **Things 3（Cultured Code）**：2026 年仍一次性买断，iPhone $9.99 / iPad $19.99 / Mac $49.99，无订阅，Things Cloud 同步免费。来源：PCMag 评测页 + culturedcode.com（2026 年定价页，【行为】，2026 年确认）。
- **BBEdit 16（Bare Bones）**：2026-05-21 发布，官网仍卖永久授权 $59.99（升级 $29.99/$39.99），Mac App Store 另有订阅选项但买断并存。来源：barebones.com/store + 官方新闻稿（【行为】，2026-05）。33 年老牌还在卖买断。
- **OmniFocus 4（Omni Group）**：2026 年定价 $74.99 标准版 / $149.99 Pro 买断，与订阅并存。来源：ellieplanner.com 定价汇总 + omnigroup.com（【行为】，2026）。
- **Directory Opus 13（GP Software，Windows！）**：2024-01 发布时**改为一次性付费**（from $89），Windows 老牌文件管理器买断制活得很好。来源：reddit r/dopus 官方公告 + betanews（【行为】，2024-01）。与农场产品形态最接近的直接证据。
- **Sublime Text**：官网 sublimehq.com/store 仍在售 $99 永久授权（2025 年确认在售）。来源：官网（【行为】，2025）。
- **Procreate（Savage Interactive）**：$12.99 一次性买断（2025 年底从 $9.99 提价），连续 8 年+ iPad 付费榜第一，3000 万用户，估算年收入 $15M-46M，零融资 14 年盈利；**2024-08 公开宣布拒绝集成 AI**，反而赢得专业用户口碑。来源：SensorTower 数据经 checkthat.ai / saaspricepulse 汇总（【行为】，2025-2026）。这是「反订阅 + 反 AI」双姿态仍然赚钱的最强单例。
- **1Password**：2025-11 宣布 ARR 突破 **$4 亿**（2023 年 $2.5 亿），年增 ~29%，75% 收入来自企业，毛利率留存 >90%，正考虑 IPO。来源：1password.com/press/2025/nov + BetaKit（【行为】，2025-11）。安全/信任品类在 AI 时代加速增长，不是萎缩。
- **Bear**：订阅 $2.99/月 $29.99/年，2026-08 仍在做返校促销，活着。来源：bear.app/faq + blog.bear.app（【行为】，2026-08）。

**反向（投降或转向）：**
- **Obsidian**：2025-02-20 起核心应用对一切用途免费（原 $50/用户/年商用许可改为自愿），收入靠 Sync/Publish 订阅 + Catalyst 一次性 $25，估算 ARR 仅 ~$2M。来源：obsidian.md/pricing + 官方公告（【行为】，2025-02）。「本地优先+用户供养」旗帜手主动把本体免费化——信任溢价没能支撑它继续收本体钱。

### 问题 2：订阅疲劳与一次性付费回潮

**实证数据（行为调查）：**
- 美国人月均订阅支出 **$219-273，自以为只花 $86**（2.5 倍感知差，C+R Research / West Monroe，2024-2025）；41% 自认订阅疲劳；42% 在为已遗忘的订阅付钱（~$200+/年浪费）；52% 过去一年因不再使用取消过订阅。来源：justpricing.com 统计汇总 + lowermysubs.com（West Monroe 2025、C+R Research 2024）（【行为调查】，2024-2025）。
- **RevenueCat《State of Subscription Apps 2025》**（覆盖 $10B+ 交易、75,000+ 应用）：**35% 的应用已混合使用订阅 + 消耗品/终身买断，且趋势在增长**；报告原话「Subscriptions aren't enough anymore」；CEO 明说「很多应用开始测试 lifetime/一次性购买以捕获不愿订阅的用户」；年订阅首月取消率 ~30%，留存率逐年下降（年付 1 年留存 47.1%→44.1%）；头部 5% 应用收入是尾部 25% 的 400 倍（去年 200 倍，分化加剧）。来源：revenuecat.com/state-of-subscription-apps-2025 + subscriptioninsider.com（【行为】，2025-03 发布）。
- Adobe 订阅反弹后竞品（Affinity 当时还是买断）扑上来抢人，CNET 2024 报道。来源：cnet.com（【行为】，2024）。
- 中文市场：Eagle 素材管理 ¥229 买断永久授权，长期以「买断制、免订阅」为核心卖点做促销，中文设计师社区口碑案例。来源：appinn.com + 官方促销页（【行为】，2025-2026 持续在售）。

**结论**：订阅疲劳是**可量化、在恶化**的事实，且供给侧（RevenueCat 数据）已出现「纯订阅不够用、混合买断回潮」的可见转向。

### 问题 3：AI 生成应用的事故实证与信任折价

**具体事故（全部行为证据）：**
- **Tea App（2025-07）**：vibe-coded 约会安全 App，Firebase 未加防护暴露数万张身份证件照；三天后第二个漏洞暴露 **110 万条私信**。Barracuda 归因于典型 vibe coding「速度压倒安全」。来源：blog.barracuda.com 2025-12-22（【行为】，2025-07）。
- **Lovable（CVE-2025-48757，2025 年中 + 2026-04 二次事故）**：BOLA 漏洞致平台上 ~170 个应用（扫描量的 10%）泄露用户数据，单起 18,000 条记录；2026-04 再次爆出 2025-11 前全部项目暴露。来源：GitHub cso-vibecheck 审计记录 + bastion.tech（【行为】，2025/2026-04）。
- **Moltbook（2026 初）**：创始人「没写一行代码」的 AI 社交网络，上线 3 天泄露 **150 万个 API token + 3.5 万邮箱**（Supabase key 写进前端、RLS 全关）。来源：cso-vibecheck / CSA 研究注记（【行为】，2026 初）。
- **Cal AI**：Firebase 无鉴权，泄露 14.59GB / 约 320 万用户记录。来源：CSA 研究注记（【行为】，2025）。
- **规模化扫描**：RedAccess 扫 5,000 个 vibe-coded 应用，40% 暴露敏感信息（含医院排班、企业财务）；Escape.tech 扫 5,600 个发现 2,000+ 高危漏洞、400+ 明文密钥在生产环境；Apiiro：AI 代码特权提升路径 +322%，企业月度安全发现 10 倍化。来源：labs.cloudsecurityalliance.org 研究注记（2026-04/06）+ pcmag.com（【行为】，2025-2026）。

**信任折价已制度化（这是比事故更强的证据）：**
- **70% 的投资者**现在投 vibe-coded MVP 前要求技术验证；YC 2025 冬季批次 25% 是 AI 生成代码库。来源：indiehackers.com QuickLaunch 复盘（【行为】，2026-01 发布）。
- **「修 vibe 代码」已成收费市场**：QuickLaunch 2025 年纯口碑做了 $120k+「Rescue Sprint」（7 天固定价重建地基）；Vu Agency 等多家挂出「Fix My Vibe-Coded App」服务页。来源：indiehackers.com + vu.co.uk（【行为】，2025-2026）。**为「人写的、可信的代码」付钱这件事本身已经商品化。**
- 弃维护实证：社区统计 **63% 的 vibe coder 在第三个月弃坑**（「Month 3 Wall」）；企业侧 S&P Global 调查 42% 公司 2025 年放弃 AI 项目（上年 17%）；估 8,000 家 AI 生成的创业公司需要「救援工程」，清理成本 $4 亿-40 亿。来源：codingwithvibe.com + S&P Global 经 rapidclaw.app 转引（【行为/调查】，2025）。

### 问题 4：「本地/离线/无账号/无遥测/开源」作为付费卖点

**正向行为证据：**
- **Kagi**：无广告、无追踪、不卖数据的付费搜索引擎，付费用户从 2024-01 的 2.5 万 → **2025-06 破 5 万**（官方 changelog），18 个月翻倍，$5-25/月，靠订阅盈利（2024 年已盈利），拒绝 VC。来源：en.wikipedia.org/wiki/Kagi + kagi.com/changelog（【行为】，2025-06）。「为无追踪掏钱」有直接计数证据，且付费档里最高的 $25/月也有人买。
- **Obsidian Sync/Publish**：本地优先笔记的付费同步/发布服务（$4-10/月），商业本体免费后这是其全部收入；本地存储+端到端加密是付费理由的核心文案。来源：obsidian.md/pricing（【行为】，2025-2026）。
- **Procreate**（见问题 1）：买断 + 无账号 + 公开拒 AI，3000 万用户。
- **1Password / Mullvad**：隐私安全品类整体在涨价放量（1Password $4 亿 ARR；Mullvad 固定 €5/月、无账号体系、2025 年被多家评测列为隐私首选——但具体用户数未查到）。
- 注意：**「开源」本身作为直接付费驱动的证据弱**——本路调研未找到「因为开源所以掏钱」的计数案例；开源更多是信任背书/分发渠道，付费转化仍靠功能与服务。这本身是个发现。

### 问题 5：反向证据——信任溢价不成立的实证

- **隐私悖论（学术铁证）**：MIT 全校本科生田野实验（Athey et al.）——用户声称重视隐私，但很小的操作成本就能让他们放弃隐私保护；Deloitte：仅 47% 消费者信任在线服务，但 15 项常见保护措施没有一项过半数人执行；2026 年 IJAMA 实验（n=2,500）：收益即时、风险抽象时，用户分享敏感数据意愿高 40%。来源：alexmoehring.com（Athey 论文 PDF）+ deloitte.com + ijama.in（【行为实验】，2024-2026）。**嘴上说重视隐私与实际掏钱之间隔着一条有测量数据的鸿沟。**
- **Affinity（Serif）免费化（2025-10-30）**：「订阅疲劳者的旗帜」、买断制最成功的现代案例，被 Canva 收购后：V2 永久授权停售、不再维护，新三合一应用**完全免费**（需 Canva 账号，AI 功能绑 Canva Pro 订阅）。2024-03 收购时「保持买断」的承诺页被直接重定向埋掉。来源：cgchannel.com 2025-10 + alternativeto.net + glyndewis.com（【行为】，2025-10）。**买断制头牌不是被订阅杀死，是被「免费 + AI 增值订阅」杀死——这正是供给洪水剧本。**
- **Obsidian 本体免费化**（2025-02，见问题 1）：本地优先标杆主动放弃本体收费。
- **买断转订阅/停售潮（2025 集中爆发）**：TechSmith（Snagit/Camtasia）2025 转年订阅；ChemDraw 永久授权 2025-01-01 停售；Adobe Acrobat 永久版 EOL；微软 Business Central 永久授权 2025-04-01 终结；Capture One 停止永久授权功能更新。来源：support.techsmith.com + support.revvitysignals.com + schneider.im + navtech.net + imaging-resource.com（【行为】，2024-2025）。供给侧在集体逃离买断，不是消费者逼的——说明对厂商而言买断经济模型承压。
- **Jasper**：$15 亿估值 AI 写作工具，ChatGPT 免费版发布后 43 天即被抽掉地基：2023 裁员、ARR 预测砍 30%、内部估值砍 20% 至 $1.2B，收入从峰值 ~$1.2 亿跌至 ~$5,500 万（2024）。来源：The Information 经 maginative.com + sacra.com + electroiq.com（【行为】，2023-2024）。**「AI 免费化杀死收费工具」不是假设，已发生过一轮——被杀的是「功能即模型能力」的产品。**

### 问题 6：历史类比验证

- **WordPress/Wix 之后网页设计没归零，但向高端迁移**：BLS 2024-34 官方预测网页开发者就业 **+7.5%（快于全职业平均）**，年新增岗位 1.36 万+；但结构分化：初级新聘薪资压到市场中位 79.5%，高级岗位 130% 溢价；新增需求集中在系统集成、性能、安全、可访问性等 DIY 工具做不了的复杂工作。来源：bls.gov/ooh + bls.gov Monthly Labor Review 2026 + pave.com 分析（【行为/官方统计】，2024-2026）。低端模板站确实归零了（ThemeForest 销量计数器跌 70%+、2024 被 Shutterstock 收购后持续失血、2026-07 作者分成砍至 50%，Freemius CEO 预言其五年内消失）——**模板/低端被免费化清场，人做的复杂工作反而涨价**。来源：superbthemes.com + freemius.com（【行为】，2024-2026）。
- **Steam 供给爆炸后的收入分化（Unity/AI 让做游戏免费化的自然实验）**：2025 年 Steam 上线 19,000+ 款游戏（历史最高），总收入 $16.2B（历史最高）；但 **48.8% 的游戏评测数 <10，11.7% 零评测**，仅 3%（608 款）达到 1000+ 评测的商业存活线；独立游戏合计占平台收入 25%，全部价值集中在头部。来源：steamdb.info + PC Gamer 2025-12 + gameworldobserver.com 2025-11/12 + howtomarketagame.com 2026-01（【行为】，2025-12 至 2026-01）。**供给洪水没有压平价格，而是把中位数压向零、把头部抬更高——平台总盘子和赢家收入都创新高，沉默的大多数颗粒无收。**

### 正反证据对照

**信任溢价成立（最强三条）：**
1. **Procreate**：买断 + 无账号 + 公开拒 AI，3000 万用户、8 年畅销榜第一、年入数千万美元（【行为】2024-2026）——「反 AI 供给洪水」姿态直接换成忠诚度与收入。
2. **vibe coding 信任折价已制度化**：Tea App 110 万私信泄露等一串具体事故 → 70% 投资者要求技术验证 → 「修 vibe 代码」形成 $120k 级收费市场（【行为】2025-2026）——市场已经在为「人负责的软件」明码标价。
3. **RevenueCat 2025 报告**：35% 应用混合买断且增长 + 订阅留存逐年下降 + 官方结论「纯订阅不够用」（【行为】2025）——供给侧数据确认买断回潮不是怀旧情绪。

**信任溢价不成立（最强三条）：**
1. **Affinity 免费化（2025-10）**：买断制最成功头牌被「免费 + AI 订阅增值」剧本收编——供给洪水杀买断不是杀订阅，这就是正面靶心（【行为】）。
2. **隐私悖论实验链**（MIT/Deloitte/IJAMA 2024-2026）：声称重视隐私与掏钱行为之间有可测量鸿沟，操作成本稍高即放弃保护（【行为实验】）——「无遥测/本地」对大众市场可能永远只是嘴上卖点。
3. **2025 买断停售潮 + Obsidian 本体免费**：TechSmith/ChemDraw/Acrobat/Capture One 集体弃买断，本地优先标杆 Obsidian 放弃本体收费（【行为】2025）——厂商用钱包投票：买断模型在运维成本面前承压，免费才是流量解药。

### 综合判断

「本地/离线/买断/开源/无遥测」这套属性在 2026 年**能支撑收费，但只对已经知道自己在为什么付钱的人群成立**——Kagi 5 万、Procreate 3000 万、Directory Opus/BBEdit/Things 老牌长青证明这群人真实存在且在掏钱，而隐私悖论证明他们占总人口比例很小且不会为口号掏钱。这套属性的正确定位是**筛选器与信任状，不是需求本身**：它决定已经想要这个品类的人在你和免费 AI 替代品之间选谁，但不能凭空创造需求（农场自己 round3-5 的结论「瓶颈在分发不在找品」与此互证）。AI 供给洪水对这套定位**净利好**：事故实证 + 投资者技术验证制度化 + vibe 修复市场的出现，说明「有人负责的软件」正在被供给洪水反向抬高相对价值——但 Steam 数据同时警告：洪水里中位数归零，信任溢价全部归头部可见者，没有渠道可见性的信任状一文不值。历史类比（网页设计、Steam）给出同一剧本：低端被免费清场、人的/可信的/复杂的工作向高端迁移并涨价——农场的买断小工具要站的正是这个迁移方向的上岸点。最后一瓢冷水：Affinity 之死说明这套属性挡不住「在位者免费 + AI 增值」的降维打击，定价假设必须避开任何「功能 = 大模型白送能力」的品类（Jasper 坟场）。

### 未查到清单

- **Mullvad VPN 具体付费用户数/收入**：试过 "Mullvad VPN number of users 2025 accounts growth"、"Mullvad revenue 2025 paid subscribers"，只有评测文无计数数据（公司不公开）。
- **Bear（Shiny Frog）收入/用户规模**：试过 "Bear notes revenue 2025"，只有定价无收入数据（私人公司不公开）。
- **Things（Cultured Code）收入**：同上，只有定价行为证据，无收入计数。
- **RevenueCat 报告中一次性购买的逐年增速具体百分比**：公开摘要只有 35% 采用率与「趋势在增长」，YoY 细分在 263 页全文内（revenuecat.com/state-of-subscription-apps-2025），WebFetch 被本机网络策略拦截无法抓取。
- **网页设计服务市场规模具体美元数字**：mordorintelligence 等页面被 WebFetch 拦截；只拿到 BLS 就业侧数据与 ThemeForest 衰落侧数据。
- **中文市场订阅疲劳调查数据**：试过中文关键词，只拿到 Eagle 个案与微短剧支付偏好（Statista），无通用软件订阅疲劳中文调查。
- **「因为开源所以付费」的计数案例**：试过 "open source paid app customers pay because open source 2025"，未找到把开源作为首要付费理由且有人数/收入数据的产品——开源更像信任背书而非直接卖点（此「未查到」本身可作为弱信号）。

### 给框架写作的失效标志建议

出现以下任何一条，说明「信任溢价」是自我安慰而非市场事实：
1. **付费隐私工具的计数数据停滞或掉头**：Kagi 付费用户停在 5 万不再增长（其 changelog 持续公布，可直接观测）、Mullvad/Proton 公布付费转化下降——说明愿为「无遥测」掏钱的人群已见顶且太小。
2. **老牌买断幸存者接连转免费或转订阅**：Things/BBEdit/Directory Opus/Sublime/OmniFocus 中任何两家在 12 个月内宣布转订阅或免费——幸存者防线崩溃即证伪。
3. **vibe coding 事故率显著收敛**：CSA/Escape 的扫描显示 vibe-coded 应用漏洞率降至人写代码水平（如 OWASP 命中率 <10%）——信任折价的根基是事故，事故消失折价消失。
4. **AI 平台把「可信」做成免费标配**：如 Lovable/Cursor 推出平台级自动安全审计并免费提供、且获主流采用——「有人负责」的差异化被平台吃掉。
5. **买断标杆的新一代全部选择订阅或免费**：观察 2026-2027 新发布的知名独立软件（尤其 Mac/Windows 工具类）的定价选择，若买断出现率趋近于零，RevenueCat 的「混合回潮」被证伪。
6. **自家产品侧**：免费/开源本体 + 付费层的转化率长期低于行业基准（RevenueCat 数据可对标），且用户访谈中「本地/买断/无遥测」从未被主动提及为购买理由——说明这套属性在自己的渠道里也不转化，只是开发者自嗨。
7. **「功能 = 模型白送能力」的品类信号**：目标品类的主流抱怨开始被 ChatGPT/Copilot/Claude 的免费功能直接回答（Jasper 剧本），该品类一切定价假设即刻作废。

---

## C 路：产品之外四条变现路径的可验证性（2026-09-20 返回）

调研日期 2026-09-20。证据标注约定：【行为】=真实收入/成交/财报/平台数据；【弱行为】=公开报价单、自报晒单、第三方抓取；【言词】=观点/教程/指南（仅作背景）。WebFetch 在本环境被大面积拦截，部分数字经搜索引擎摘要交叉验证，个别站点无法直接打开原文。

### 路径 1：内容/受众（博客/newsletter/YouTube/B站/公众号）

**时间到首单与基线率**
- 【行为】beehiiv《State of Newsletters 2026》（2026-01 发布，覆盖 2025 数据，6.5 万+ newsletter）：2025 年启动的 newsletter **从创建到赚到第一美元的中位时间 = 66 天**；平台付费订阅收入 $19M（+138% YoY）；**有收入的创作者占比从 15%（2024Q1）升到 30%（2026Q1）**。注意幸存者偏差：只统计留在平台上的，且「第一美元」不区分陌生人/朋友。https://www.beehiiv.com/blog/the-state-of-newsletters-2026
- 【行为】Creator Spotlight《2025 Monetization Report》（427 名创作者问卷）：**近 50% 创作者全年收入 <$500**；仅 9% 年收入 >$10 万。https://www.creatorspotlight.com/p/monetization-report-2025
- 【行为】beehiiv 免费转付费**中位转化率仅 0.62%**（每千名免费读者约 6 人付费）；头部十分位财经类可达 20-30%。https://www.digitalapplied.com/blog/newsletter-statistics-2026-data-points
- 【弱行为】InboxReads《State of Newsletters 2025》：投稿中仅 2% 为全付费 newsletter，91% 纯免费；77% 创作者想靠赞助/广告变现——**赞助才是主变现模式，付费订阅是少数人的游戏**。https://inboxreads.co/blog/state-of-newsletters-2025
- 【弱行为】YouTube：仅 **~3% 频道达到变现门槛**（1000 订阅+4000 小时）；~9% 频道有 1000+ 订阅；随机采样中位频道只有 ~61 订阅。https://tukey.ai/blog/youtube-monetization-rate
- 【行为】B 站：花火商单平台门槛 1 万粉；创作激励单价崩塌——每万播放从 2018 年 ~30 元降到 2024-2025 年的 2-10 元，中小 UP 主收入降 80%，月度激励上限 2000 元（澎湃新闻 2024-04、新浪财经 2025-04-10）。但商单侧在涨：2025 年超 310 万 UP 主有收入、人均商单量 +28%、充电收入 +122%、B 站首次全年盈利（经济参考报 2026-03）。**平台分钱已死，品牌商单/充电活着**。
- 【弱行为】小报童个案：技术博主 BuildForever 公众号+newsletter 约 3500 关注 → 小报童 127 人付费（转化 3.5%）；IDO老徐称「大多数专栏付费人数不到 100 人」。http://istester.com/xiaobot.html
- 【言词】公众号：万粉技术号流量主约 2000-5000 元/月（壹伴/9ku 等 SEO 站，证据弱）。

**AI 冲击**：双向。一方面 AI 内容洪水+Google AI Overviews 零点击（2026 年多份 SEO 行业报告称 AI Overviews 使点击量降 ~58%，自然流量危机），靠 SEO 引流的内容型博客正在被掐死；另一方面 AI 厂商本身在大额投放博主（2025 年 400+ AI 产品在 B站/小红书推广），技术类「高转译」UP 主反而是受益垂类。**结论：AI 杀死的是「流量套利型内容」，没杀死「信任型内容」，但信任型内容恰恰是最慢的。**

### 路径 2：服务/接单

**市场总量收缩（低端已死）**
- 【行为】Ramp Economics Lab（企业信用卡支出数据）：自由职业平台支出占企业总支出从 2021 末 0.66% 崩到 2025 末 **0.14%**（近 5 倍缩）；2022 年用过自由职业平台的企业过半到 2025 年已完全停用；同期 AI 支出（OpenAI/Anthropic）从 0 涨到 ~3%。https://www.metaintro.com/blog/fiverr-buyers-fell-ai-freelance-jobs
- 【行为】Fiverr：买家数流失 **21.9%**，CEO 公开承认「AI 吸收了高频低值交易型任务」；2025-09 裁员 30% 转型 AI-first，转向 $1000+ 高价项目层。Fairwork Cloudwork Report 2025 另记录 20% 抽成、14 天账期等结构性问题。https://fair.work/wp-content/uploads/sites/17/2025/05/Fairwork-Cloudwork-Report-2025-FINAL.pdf
- 【行为】中国外包业（36氪/新浪财经 2026-04-29）：软件外包净利率从 ~10% 崩到 **~0.1%**；软通动力 2025 年净亏 3.5 亿（-77%）；蔻町智能案例称 100 人团队的活现在 2-3 人干、几十万的电商站 AI 报 $6-8；【言词部分】文章预测行业客单价降 70-90%。https://m.36kr.com/p/3786425303325953

**高端 AI 单在涨（窗口期）**
- 【行为】Upwork Q4 2025 财报（2026-02-09）：**AI 相关 GSV 年化破 $3 亿，+50% YoY**；AI 集成与自动化类 +90% YoY；参与 AI 工作的客户数 +50%，且**这类客户客单价是平台均值 3 倍**。但整体 GSV 只 +3%——蛋糕在向 AI 单集中。https://investors.upwork.com/node/12716/pdf
- 【言词】AI 咨询费率指南：美国资深 $150-500/小时，顶尖企业 LLM 部署 $500-1000/小时（Fortune 报道 $900/小时个案）；策略类比执行类溢价 20-40%；市场正从计时转向按价值定价。https://dancumberlandlabs.com/blog/ai-consulting-pricing/
- 【弱行为】中国独立 AI 顾问公开报价单（真实存在的服务商品）：陈虾仁 ¥900/小时咨询、¥4980 半天企业诊断、¥12800/月陪跑（chenxiaren.com）；马甲 Majia 企业 AI 内训 ¥1.2-1.5 万/天、专项 ¥5-15 万（majia.pro）。证明这个价位有真人挂单，成交率未知。
- 【言词】程序员客栈：入驻要 2 年经验+3 作品，项目单价 8000-50000 元、抽成 10-15%、派单制（知乎/CSDN 2025 软文居多）；猪八戒 20%+ 抽成。均未查到可信的单价趋势硬数据。
- **重要反向信号**：【行为】火山引擎 AgentKit 2026-05 商用，把 AI Agent 落地项目从行业均价 15 万/个打到「百元级入门」，首月 2000 企业注册，阿里腾讯跟进降价——**「帮企业落地 AI」这个服务本身也在被平台商品化，窗口在收窄**。https://www.volcengine.com/article/2736832

### 路径 3：数字商品（课程/模板/boilerplate）

**这条路径刚被 AI 实锤打击，证据链最完整**
- 【行为】ShipFast（品类标杆）：峰值单月 $10 万+（2024）→ 2025 年均 ~$1.8 万/月 → **2026-07 trailing 30 天仅 ~$2,800，较峰值 -97%**。Marc Lou 2026-03 公开承认「AI 杀死了我的编程课和 boilerplate 生意」，2026-09 停掉 ShipFast/CodeFast 联盟计划（X 帖原文：revenue has dropped significantly, mostly due to AI）。他靠 DataFast（$28.9K MRR 订阅制）+ TrustMRR 补回损失——**一次性买断的数字商品死了，订阅制 SaaS 活着**。https://stealwhatworks.com/blogs/news/shipfast-still-making-money 、https://x.com/marclou/status/2098446725942047075
- 【行为】Gumroad 全平台抓取（InsightRaider 2026，146,271 个商品）：**创作者中位收入 $72/月；44% 商品终身 $0；头部 1% 吃掉 77-99.5% 收入；中位商品终身 28 单 × $13 ≈ $364**。到第一个 $100：有受众 60-90 天，**无受众 6-12 个月**。https://insightraider.com/en/state-of-gumroad-2026
- 【行为】Udemy：讲师总收入 2024 $191.2M → 2025 $168M（-12%）；分成比例 25%→20%→17.5%→2026-01 降到 15%；**平均每门课收入 <$100；一半 AI 课程月入 <$31**；2024 年新增 5.4 万门课灌水。（Class Central 2025-01：政策变动一年让讲师少拿 $30M）https://www.classcentral.com/report/udemy-broken-promise-instructor-payouts/
- 【弱行为】中文侧幸存者：知识星球「Java 程序员进阶之路」¥189/年 5500+ 付费；「AI 破局俱乐部」¥2399/年 6 万人；小报童榜一「生财有术」2 万+份 GMV 百万+——全部是**先有大 IP 后有商品**的头部，不可复制为冷启动路径。

### 路径 4：micro-SaaS / B2B 小工具

- 【行为】Indie Hackers 全量抓取（ScrapingFish，937 个 Stripe 验证产品，2022 数据 2024 讨论）：**54% 产品收入为 $0；仅 ~5% 年收入 >$10 万**；另有 IH 站内统计：17,207 名会员中仅 193 个产品（~1%）自报 >$2K MRR，12 个 >$10K MRR。https://scrapingfish.com/blog/indie-hackers-revenue
- 【言词-二手汇总】Freemius《State of Micro-SaaS 2025》及多家汇总：**中位 3-4 个月到首单**（先验证再写码者快 ~40%）；~60% 产品能见到第一个 $1，仅 ~20% 到 $1K MRR（12-24 个月），~5% 到 $10K MRR；40% 永远到不了 $1K MRR。https://freemius.com/blog/state-of-micro-saas-2025/
- 【行为个案】Marc Lou 的 DataFast：124 天到 $1K MRR → 现 $28.9K MRR/1,336 订阅（build-in-public 公开数据）——证明 B2B 订阅制单人可做，但他自带 10 万+ 推特大受众。
- 【弱行为】中国独立开发者调研（CSDN/opcbase 2025，自称 1000+ 样本，小站问卷，证据强度中低）：收入金字塔 5% 月入 5 万+、15% 月入 1.5-5 万、**50% 月入 <8000**；外包占收入来源 40%、SaaS 25%、内容 15%。https://blog.csdn.net/FansUnion/article/details/148829546
- 【行为】供给端洪水：vibe coding 导致 2025 年 App Store 提交量暴涨，Apple 已开始打击 AI 生成应用（AppleInsider 2026-04、TNW）；2023-2025 初约 3,800 家 AI 创业公司关停（27%）；薄 AI 套壳防御窗口只有 3-6 个月。**「做出来的成本」趋零 = 「做出来」不再是壁垒，分发是唯一壁垒**——多篇分析共识。
- B2B vs C 端：B2B 客单价显著高（Upwork AI 客户 3 倍花费；汇总数据称 B2B niche $50+/月付费意愿 vs C 端 $5-10），但 B2B 陌生人获客需要销售/内容动作，无受众者冷启动同样难，只是单笔金额大、所需客户数少。

### 路径对比表

| 路径 | 陌生人首单难度 | 时间到首单（有数据处） | AI 冲击方向 | 与该开发者画像匹配度 |
|---|---|---|---|---|
| 服务/接单（AI 高端向） | 中：不需受众，但要过平台新手期；低端单已死 | 未查到中位数；个案量级为数周-2 个月 | **双刃剑最利**：低端外包被瓦解（Fiverr -21.9% 买家、外包净利率 0.1%），AI 落地/改造单 +50-90%（Upwork）；但落地服务正被平台商品化（AgentKit），窗口收窄 | ★★★★ 资深技能直接变现，晚间工时可接；风险=与全职抢时间、单干无杠杆 |
| micro-SaaS B2B | 中高：54% 产品 $0；但单笔大、所需客户少 | 中位 3-4 个月（汇总）；个案 124 天到 $1K MRR | 助力（开发提速）+ 杀手（供给洪水、套壳 3-6 个月被复制）并存 | ★★★☆ 工程能力匹配、AI 缩短工期；短板=无受众无销售经验，分发是纯短板 |
| 数字商品 | 高：无受众时 44% 终身 $0、中位 $72/月 | 无受众 6-12 个月到第一个 $100 | **已被实锤杀死品类**：boilerplate/模板/入门课（ShipFast -97%、Udemy 半数 AI 课 <$31/月）；剩「IP 驱动」头部 | ★★ 与他画像冲突最大：无受众=中位收入 $72/月 |
| 内容/受众 | 极高（作为首单路径）：~50% 创作者年 <$500、YouTube 3% 变现、B 站平台分钱已死 | beehiiv 中位 66 天到「第一美元」（不区分陌生人）；有意义收入普遍 12-24 个月 | 杀死流量套利型内容（AI Overview 零点击），养肥信任型/高转译技术内容（AI 厂商投放） | ★★ 无内容经验+晚间工时=最慢；但它是其他路径的乘数资产 |

### 综合判断

对这个「会 Rust 桌面、无受众、无内容经验、只有晚间周末」的开发者，**最快验证「陌生人掏钱」的路径是服务/接单的 AI 高端方向**（帮企业落地 AI、改造遗留系统、AI 集成自动化）——它是唯一不要求受众的路径，且平台数据实证该细分正在涨价（Upwork AI GSV +50%、AI 客户 3 倍客单），国内也有 ¥900/小时-¥1.5 万/天的真实挂单先例；但必须避开低端外包（已被 AI 瓦解到净利率 0.1%）并承认这是「卖时间」的及格线验证而非资产。**第二顺位是 micro-SaaS B2B**，中位 3-4 个月首单、AI 缩短他的开发时间，瓶颈纯在分发。数字商品路径对他的画像基本判死刑（无受众 Gumroad 中位 $72/月 + boilerplate 品类刚被 ShipFast 讣告实锤），内容路径最慢但应作为**并行埋点的分发资产**而非首单路径——它的正确用法是给服务和 SaaS 导流量，不是直接变现。元结论：**他的「陌生人首单」最短回路 = 用 AI 高端接单验证付费意愿（数周），用内容积累分发（数月），再把验证过的服务痛点产品化成 B2B SaaS**；所有非服务路径的共同约束是受众为零，AI 让「造」免费、让「被看见」更贵。

### 未查到清单

- **Upwork/Fiverr 新手从注册到首单的中位时间硬数据**（试过 "Upwork new freelancer first job how long statistics"、"new freelancer oversaturation data 2025"）——只有「算法偏好老卖家」的言词描述
- **猪八戒网 2024-2026 单价趋势硬数据**（试过 "猪八戒 外包 单价 内卷 2025"）——只有平台问答和软文
- **中国开发者从 0 做英文 newsletter/YouTube 到首单的完整可验证 timeline**（试过 "Chinese developer English newsletter first revenue case"）——个案均未公开收入时间线
- **小报童全平台中位销量**（试过 "小报童 中位数 销量 数据"）——只有头部榜单和个案
- **B 站技术区 UP 主 0→首商单中位时间**——只有花火 1 万粉门槛与头部案例
- **「帮企业落地 AI」独立顾问的真实成交率/月收入**——只有报价单，无成交数据
- 知识星球的 Marc Lou「AI killed ShipFast」原话出自其 2026-03 公开内容，经 stealwhatworks/startupik 二手引用，未拿到原始视频/长文 URL

### 给框架写作的失效标志建议

按「出现什么证据说明该路径不值钱」设计，全部带可复查的公开数据源：

1. **服务路径窗口关闭信号**：Upwork 季报 AI GSV 增速从 +50% 跌破 +15%（每季度 investors.upwork.com 可查）；或 Ramp Economics Lab 自由职业支出占比止跌回升（说明企业回流买人而非买 AI，AI 落地顾问需求证伪）；或火山引擎/阿里/腾讯把 Agent 落地服务打到千元级以下且渗透率 >50%（个人顾问被平台封死，clipboard 同构死法）
2. **数字商品讣告确认信号**：ShipFast trailing 30 天 <$1000 或 Marc Lou 宣布停售（他 build-in-public，每月可查）；Gumroad 零销售商品占比从 44% 升破 55%（InsightRaider 类年度抓取）
3. **内容路径恶化信号**：beehiiv 年报「有收入创作者占比」从 30% 回落（二八分化加剧）；Google AI Overview 零点击率继续上升 + beehiiv 中位首美元时间重新拉长
4. **micro-SaaS 信号（双向）**：负面=AI 创业关停率年度更新超 35%；正面=Apple/谷歌对 vibe-coded 应用的审核打击见效、上架量回落（供给洪水被闸门控制，在位者缝隙保质期延长）
5. **通用节奏**：以上数据源全部是季度/年度公开报告，适合做成「每季度扫一遍」的复查清单，与农场的失效标志复盘节奏（新品开枪前+发布后）对齐

---

## B 路：2024-2026 单人开发者「陌生人首单」实证（2026-09-20 返回）

调研日期 2026-09-20。方法说明：本环境 WebFetch 全程不可用（所有域名安全验证失败），全部证据来自 WebSearch 摘要（共约 20 组中英文查询），未能逐页核对原文；关键数字若要写进框架文档，建议人工复核一次原始链接。

### 问题 1：2024-2026 仍在拿陌生人付费的单人/极小团队开发者

**知名案例最新状态（均已查证 2025-2026，非旧数字）：**

1. **Tony Dinh / TypingMind**（AI 聊天客户端，web+桌面；个人买断 license + B2B 团队订阅）
   - 2025-04-05 本人 X 帖：单月 $148k 创历史新高（https://x.com/tdinh_me/status/1908345028327727335）
   - 2025-08：TypingMind 终身收入破 $1M；2025-10 本人 newsletter：$130k-160k/月，**B2B 团队版已占月收入 50% 以上**（https://news.tonydinh.com/p/oct-2025-updates-code-money-and-travel）
   - 类型：行为证据（本人公开数字）。注意：收入重心已从个人买断转向 B2B 订阅；他本人 2025 年已扩到 ~3 人。反面轨迹也真实：Black Magic 因 Twitter API 变动被迫 $128k 卖掉，Xnapper $150k 卖掉（2024-03）。

2. **Pieter Levels**（web 产品组合；Nomad List 会员买断/订阅混合，PhotoAI 订阅）
   - 2025-07 Cheeky Pint 采访：组合约 $3.1M/年（经 solopreneurship.eu 2026、operatorbook.dev 转引）
   - **但单品全线衰减**（stealwhatworks.com/blogs/news/indie-saas-lost-most-revenue，2025-2026 快照）：PhotoAI $105k/月(2024-03)→~$80k（-24%）；Nomads.com ~$38k→~$15k/月（-60%）；InteriorAI ~$27k→~$23k
   - 类型：行为证据 + 第三方追踪。本人自述 70 个项目死 66 个。

3. **Marc Lou**（ShipFast/CodeFast/DataFast/TrustMRR，web SaaS+ boilerplate，买断+订阅）
   - 2025 全年 $1,032,000（LinkedIn 转引其自述，https://www.linkedin.com/posts/abhinayguptha_marc-lou-casually-made-1032000-in-2025-activity-7462165107825881088-UNnI）
   - **ShipFast 已崩**：峰值 $21.1k/月 → $2.3k-3.5k/月（-85~89%），直接死因=AI 编程让 boilerplate 贬值（streakr.co/playbook/marc-lou；stealwhatworks.com/blogs/news/shipfast-still-making-money）
   - 接棒的是 TrustMRR（~$26.5k MRR）+ DataFast（~$29.9k MRR）；2026-02 组合 ~$81k MRR
   - 类型：行为证据（TrustMRR 本身即 Stripe 直连验证平台）。30 个失败创业在前。

**非知名案例（TrustMRR 支付数据直连验证，2026 年快照，来源 bigideasdb.com/trustmrr、mrrwars.com）：**
- Postiz（开源社媒调度，solo：Nevo David）~$211k MRR；GoTall（移动 app）~$60k MRR；Laper（剧本写作，2-5 人）~$44k MRR；StoryShort（AI 视频）$22.3k MRR；AbMaxx（AI 健身）$9.4k MRR；Elofoot $6.6k MRR；Practiceme $1.7k MRR；GenDesigns $109 MRR（长尾样本）
- 平台 6000+ 产品平均 MRR ~$4,682（注意这是「愿意公开收入的幸存者」口径）
- 类型：行为证据（支付处理器直连）。**关键观察：这些案例几乎全是 web SaaS / 移动 app / AI 套壳，没有一个是 Windows 桌面买断小工具。**

### 问题 2：中文独立开发者圈实证

1. **赵纯想「胃之书」**（iOS，美食记录 AI app，上线第一天即收费，月会员+永久会员）：首月（2024-05）收入 $12k，App Store 美食佳饮付费榜第 3，2024-07 月收入约 ¥4.7 万几乎纯利，0 投放；分发靠小红书自传播。来源：36kr（m.36kr.com/p/3433367684320899）、腾讯新闻 2024-08-02、小宇宙播客。类型：行为证据（媒体访谈+榜单位置）。**是 iOS 订阅 app，不是 Windows 桌面。**

2. **V2EX 第一人称数据**（均为自述行为证据）：
   - 「Vibe Coding 副业四个月：成本超出收益」（v2ex.com/t/1228385）：384 注册 / 29 付费（含赠送）/ 会员总收入 ¥1,600 / 日访 ~1,000——典型低转化基线
   - 「全职 3 年独立开发，我想说再见了」（t/1120126）：第 1 年零收入，第 3 年才自给，机会成本估算亏 ¥200 万
   - 「搞了一年的独立开发，累了」（t/1030738）：年入近 ¥20 万但疲于奔命
   - 正面：「失业后独立站上线 7 个月收获月度订阅 $5000+」（v2ex.com/go/isv 节点）

3. **诗片 app 作者**（小宇宙 EP39 播客）：首个付费功能 app，300+ 付费用户，苹果推荐时 17 万下载。类型：自述行为证据。

4. **中文 Windows 桌面工具仍在收费的活体**（定价页=行为证据）：
   - **Snipaste 2 Pro**（中国单人开发者）：买断 $8.99/1 设备、¥99/3 设备；**微软商店版 ¥99/10 设备与官网版并行销售**；个人免费+商用付费（github.com/Snipaste/feedback/wiki/PRO）
   - **MyDockFinder**（中国单人，Steam 分发）：$4.99 买断，2021-11 发布至今更新，~5,270 评测，第三方 Boxleiter 法估算总流水 ~$947k、净得 ~$279k（steam-revenue-calculator.com/app/1787090——估算非官方）
   - **utools**：¥99/年订阅 + 限时永久会员（¥299@2023-09 五周年 / ¥328@2024-12 六周年，官方称「再开放一定更贵」）；免费版限 10 插件，V2EX 有抱怨贴（t/1129804）和找平替贴——说明定价闸真实存在且有人付
   - **Quicker**：专业版 ¥57.6-96/年订阅（getquicker.net/pricing），8000+ 用户分享动作的生态
   - 低可信度：91wink 自述「截图工具 ¥199 买断难以持续、转订阅 ¥299/年后月入 3 万」——内容农场文风，存疑，仅作旁证

### 问题 3：基线率——拿到「任意一个陌生人付费」有多难

1. **54% 的 IndieHackers Stripe 验证产品收入恰为 $0**（ScrapingFish 分析，en.social/why-indie-products-earn-zero；buildmvpfast.com 引用）——行为统计
2. **TrustMRR 验证企业 67.8% 终身收入 <$1,000；仅 0.9% 破 $1M**（stealwhatworks.com/blogs/news/indie-hackers-going-extinct）——行为统计
3. 中位盈利 micro-SaaS ~$4.2k MRR（不到美国开发者工资 40%）；Stripe Atlas：新产品拿到首个付费客户比 2020 快 2.5 倍，但仅 ~12% 新 SaaS 三年内破 $1M ARR（从 15% 下降）——行为统计
4. 第一人称零收入样本：r/SaaS 2025-08「6+ months live, still $0 revenue」；r/SaaS 被引帖「1 year full-time indie dev. $0 revenue. 30 days left before I quit」；IndieHackers「4 年 26 项目 $115k：只有 8 个有过任何收入，第 1 年 $0」；dev.to 2025-11「自学编程做完整个 SaaS，上线无人问津」
5. HN 2026 个案（news.ycombinator.com/item?id=47600898）：第一个付费客户来自 HN 帖子本身——**首单几乎都靠自带流量，不是产品挂在那里自然来人**

### 问题 4：开发者/效率工具品类付费证据 2025-2026（均为定价页=行为证据）

- **TablePlus**：$99 买断（1 设备）/ $129（2 设备），含 1 年更新，续更新 $59/seat；价格从 $49→$89→$99 一路涨，2025 仍在售且活得不错（tableplus.com/pricing）
- **Beyond Compare 5**：Standard $35 / Pro $70 永久买断，大版本升级半价；Scooter Software 1996 年至今的极小团队，v5 已发布在售（scootersoftware.com/shop/pricing）
- **Directory Opus 13**（2024 发布）：**从订阅改回一次性付费**（from AUD $89），13.13 持续更新（reddit r/dopus + GPSoftware 官网）——买断制回潮的直接证据
- **Listary 6 Pro**：$19.95 买断终身更新，2025 年 3 月/7 月评测确认仍在售（listary.com/pro）；注意其大版本更新慢有争议
- **Raycast**：freemium 订阅 $10/月 Pro，2025 底改 AI 按量计费（自述之前在补贴 AI 成本）；拿过 $30M 融资，**不是 indie 参照系**
- 中文阵营见问题 2（Snipaste/utools/Quicker/Listary 全部活着且在收钱）
- 结论性观察：**老牌桌面工具的买断制全部活着，但 2024-2026「新产品买断冷启动成功」的案例只找到 MyDockFinder（且走 Steam 不走 MS Store）一个**

### 问题 5：Microsoft Store 生态

1. **政策面**：非游戏 app 自带支付 100% 留存、用微软支付抽 12-15%（MS Learn + 2021 年多篇报道）——分成全行业最优，但分成不是瓶颈
2. **Diarium**（Timo Partl，德国单人，日记 app）：每平台一次性买断 $9.99-$30、无订阅；**2024 年 Microsoft Store Award 获奖**；全平台 76 万+ 下载（Google Play 50 万+），周更中。收入数字未公开。这是「单人+买断+MS Store 上架」最接近农场模型的活案例——但其下载基本盘明显在移动端（tinkeringprod.com、AppBrain、noteapps.ca）
3. **Snipaste**：微软商店版 ¥99 与桌面版并行在售（见问题 2）——证明 MS Store 可作买断分发渠道，但无销量公开
4. **反面实证**：Reddit r/SoloDevelopment（2026）Xbox/MS Store 开发者自述：Store Ads 38,000 展示 / 549 点击 / **0 付费下载**（reddit.com/r/SoloDevelopment/comments/1w917pq/）——自述行为证据，与农场 pomodoro 的 46 浏览/0 陌生人付费同构
5. **消费者习惯**：未找到直接统计；归纳性证据是 Windows 用户长期「官网下载+免费+破解」心智、MS Store 付费讨论帖几乎不存在（6 组关键词均捞不到第一人称收入自述，见未查到清单）——缺席本身是信号

### 综合判断（4 句）

1. **「单人应用收费」2026 年依然成立，但成立的形态非常具体**：web SaaS/AI 套壳（订阅）、iOS/安卓消费 app（订阅/内购）、**Steam 分发的 Windows 桌面小工具（$3-5 买断）**、官网直销的老牌专业工具（$20-100 买断）——四条活路里三条不在 MS Store。
2. **陌生人首单的真实难度**：TrustMRR 67.8% 产品终身收入 <$1K、IndieHackers 54% 恰为 $0——中位数体验就是零；而拿到首单的案例几乎 100% 自带分发（X 粉丝 / 小红书爆量 / HN 首页 / Steam 新品位），没有任何一个被查证的案例是「挂在商店里自然来首单」。
3. **买断制定价本身无罪**：TablePlus/BC5/DOpus13/Listary/Snipaste 2025 全在售，Opus 13 甚至从订阅改回买断；农场的 ¥18-59 买断价位有活着的同类。死穴在渠道——MS Store 自然流量薄是全行业现象（农场 46 浏览/月 vs Xbox 开发者 38k 展示 0 转化，同构）。
4. **AI 对单人收费是双向的**：它杀死了一类（ShipFast -89%、boilerplate 全灭），同时造了一批新收入（TypingMind、TrustMRR 上的 AI 套壳长尾）——「AI 时代单人不能收费」不成立，「AI 时代不带分发、不做 AI 相关、靠商店自然流量的单人 Windows 小工具收费」才接近当前证据支持的判断。

### 幸存者偏差警示

- **Pieter Levels**：70 项目死 66；分发 = 十年积累的 60 万 X 粉；且 2025-2026 单品收入全线下滑（Nomads -60%），巅峰叙事已过期
- **Marc Lou**：30 个失败创业在前；旗舰 ShipFast 已被 AI 杀死；现在活得好的是 TrustMRR——**卖铲子给 indie hacker 的人**，不是卖工具给终端用户的人，模式不可照搬到终端工具
- **Tony Dinh**：AI 浪潮最早班车 + build-in-public 受众；收入主引擎已是 B2B 团队订阅，「个人买断工具」不是他现在的故事
- **Wallpaper Engine**：Steam 全站收入 #97 的品类之王，极端离群值，只能证明「Windows 桌面用户会为工具付费」，不能证明「你的工具会被付费」
- **胃之书**：iOS + 小红书流量红利 + AI 品类窗口三重叠加，三条 2024 后都在收窄
- **TrustMRR 平均 MRR $4,682**：是「愿意公开收入者」的平均，严重右偏；67.8% <$1K 才是全样本口径

### 未查到清单（附试过的关键词）

1. **单人开发者 MS Store 小工具陌生人付费的第一人称收入自述（2024-2026，中英文均零命中）**——试过 "Microsoft Store solo developer paid app revenue success story"、"published Microsoft Store my paid app revenue results reddit"、"make a living selling UWP apps"、"微软商店 付费应用 开发者 收入 知乎"、"MS Store barely any sales reddit"、Store Ads 转化等 6+ 组。此缺席本身是最强信号之一
2. OpenCat（baye）、沉浸式翻译的收入数字（查询零结果）
3. MS Store 消费者买断付费习惯的任何统计数据
4. StartAllBack（$4.99 买断、仍在售）与 Listary 的实际销量/收入——均无公开
5. Product Hunt 发布产品零收入比例的系统统计（只有碎片 anecdote）
6. 方法论限制：WebFetch 全程不可用（streakr.co、stealwhatworks.com、en.social、v2ex、scootersoftware 等 7 个域名全部安全验证失败），所有数字来自搜索引擎摘要，未逐页核对原文

### 给框架写作的失效标志建议

**证明「单人应用收费不成立」是错的（即收费其实成立）应监控的信号：**
1. TrustMRR / 同类验证平台上出现 Windows 桌面买断工具新条目且收入连续两季度增长（每季度扫一次）
2. 出现 2026 年之后的第一人称 MS Store 付费工具收入自述（监控 r/SideProject、r/dotnet、V2EX isv 节点、LINUX DO）
3. Steam 软件区新上架买断工具的评测数曲线（评测数 ≈ 销量代理变量，Boxleiter 法可换算）
4. 农场自身：danqing-log 付费层上线后 90 天内有陌生人首单——直接证伪「不成立」，且这是农场自己能制造的最强证据
5. Diarium 类孤例增多：Microsoft Store Award 2025/2026 获奖名单中出现新的单人买断工具

**反向失效标志（证明「收费成立」这一判断错了，应对称写入）：**
1. TablePlus / Beyond Compare / Directory Opus / Listary / Snipaste 五家中有 ≥2 家转纯订阅或停更（目前 0/5）
2. Diarium 停更、转订阅或开发者公开承认收入不可持续
3. utools/Quicker 永久会员彻底停售且年费续费率公开恶化（监控 V2EX/LINUX DO 相关帖的情绪转向）
4. Steam 软件区买断新品 12 个月存活率（以上架后仍有更新计）跌破可忽略水平

---

## A 路：AI 供给洪水实证（2026-09-20 返回）

### ⚠️ 方法论限制（必读，先于任何结论）

1. **WebFetch 在本环境对所有域名全部失败**（developer.apple.com、learn.microsoft.com、techspot.com、ceoworld.biz、telerik.com、36kr.com、a16z.news 均返回 "Unable to verify if domain is safe to fetch"）。**没有一条证据是打开原文核对的。**
2. **WebSearch 配额在 5 组查询后即耗尽**（会话级上限 200）。补检索经子 agent 复核确认同为会话级共享配额，六条补检索全部拒绝执行。
3. 因此**全部证据来自 WebSearch 返回的「摘要层」**（搜索引擎摘要 + 命中标题 + URL），数字与日期可靠性低于「读过原文」的级别。**任何要写进框架文档的数字，建议人工点开 URL 复核一次。**
4. 类型标注：【行为】=数字/政策原文/付费事实；【言词】=观点/抱怨/预测；【二手行为】=数字存在但只见二手引用。

### 问题 1：2024-2026 新应用提交量趋势与「AI 灌水」实证

**提交量激增（【二手行为】，多来源交叉）**
- **Appfigures**：2025 年全美 App Store 新应用提交约 **557,000 个，同比 +24%**，为 2016 年以来最高。来源：TechSpot (https://www.techspot.com/news/113213-apple-app-store-inundated-low-quality-vibecoded-apps.html) + CEOWORLD（**2026-03-11**，https://ceoworld.biz/2026/03/11/vibe-coding-is-flooding-the-app-stores-with-new-apps-standing-out-just-became-the-hardest-part/）。同组另给「约 600,000（+30%）」口径，**两数字并存、口径未说明，需复核**。
- **Sensor Tower**：美国 iOS 新应用发布量 1 月同比 **+54.8%**，约四年最高增速。另一路给 **2026 Q1 新 iOS 提交同比 +84%**，此前 2025 年 +30%。**两处数字（+54.8% vs +84%）口径不同，标待核。** 来源：Game Industry Library (https://gameindustrylibrary.com/documents/the-app-store-discoverability-squeeze)
- **Apple 官方口径（反向数字，重要）**：约 90% 提交 48 小时内审完、每周处理超 20 万次提交、2025 年全年评估超 **910 万次提交**。来源：ExtremeTech + Business Insider（**2026-03**）。⚠️ **910 万是「提交次数」（含更新、全球），Appfigures 55.7 万是「新增应用记录」（美国），不可直接比较**；但 910 万量级说明评审管线承压。
- **2026 上半**：约 56 万新提交，几乎追平 2025 全年；按此速度 2026 年可能超 100 万，破 2016 年 89 万记录。来源：bizstack.tech + ShiftDelete。【二手行为，单一来源线，需复核】

**需求侧不动（「洪水」判断的核心机制证据）**
- App Store 下载量 **2025 年 +3%**，**2026 上半 +2%**（17.6 亿次）。**供给增速（+54%~+84%）与需求增速（+2%~+3%）差 20-40 倍。**
- a16z《Charts of the Week: So Many Apps, So Little Time》(https://www.a16z.news/p/charts-of-the-week-so-many-apps-so)：应用供给在 iOS/Android/Chrome 翻倍甚至四倍，**下载与评分基本持平（仅 +2-3%）**；达到「最低热度」（10+ 评分或下载）的应用**占比骤降**。⚠️ 此文无法打开，仅摘要转述，**建议人工优先复核这一篇**。
- **收入集中度**：2025 年**收入前 1% 的应用吃掉全部 IAP 收入的 92.2%**（$154B vs 其余全部 $13.1B）。**与 D 路 Steam 数据结构同构：洪水不缩小总盘子，把中位数压向零、把头部抬更高。**
- 生成式 AI 类应用本身 2025 年向 Apple 支付近 $9 亿 App Store 费用——**钱在「把 AI 当功能」的层，不在「被 AI 生成」的层。**

**「AI slop」定性实证**：多篇报道一致描述涌入主体为低质、抄仿、不可用应用；审核队列等待数天到数周，个别报告 14-45 天或 >6 周。Apple 未披露长尾案例数据，批评者据此认为「90% 48 小时」掩盖尾部恶化。

**Google Play 侧**：Google 称 AI 原生应用数量同比三位数增长（Newsis 访谈，**2026-09-17**）。灌水量无公开数字。

### 问题 2：三商店政策应对（**本路最重要结构性发现**）

> **三家商店 2025 年的政策动作，全部是「AI 内容安全/披露」条款，不是「禁止 AI 生成应用」条款。真正拦得住 slop 的仍是老条款——最低功能性（Apple 4.2/4.2.6）、垃圾应用（Google Spam & Minimum Functionality）。结论：闸门没有因为洪水而关闭，只是在贴标签。**

**Apple**
- **2025-11 修订 App Review Guidelines**，新增 **Guideline 5.1.2(i)**：向第三方（**明示含第三方 AI**）共享个人数据须在应用内清晰披露并获用户明确许可；**合规截止 2025-12-01**，不合规下架。来源：Mashable + MacMagazine（**2025-11-13**）+ 站长之家。同日修订还含**反抄袭/反复制**条款（比 AI 披露更直接针对 slop，未拿到原文措辞）。
- **老条款是 AI 应用被拒主因**：**4.2 最低功能性**（拒 WebView 套壳）、**4.2.6**（模板/应用生成服务产出的应用除非由内容提供方直接提交）、2.1、3.1.1。来源：Nativeline + Newly.app + OpenForge。**【行业解读，非官方原文】**
- **Apple 开始封杀 vibe-coding 工具本身**：援引 **2.5.2**（应用不得审核后改变自身功能），阻断 Replit、Vibecode 等工具的部分更新。来源：AppleInsider 论坛 + ExtremeTech。日期未在摘要出现。
- **低置信度**：WWDC 2026 起 Apple 获权下架「不展示增长、更新或互动」的存量应用。来源：TechRound + KISA 文件。仅二手转述，**方向对独立开发者双刃**。

**Google Play**
- **2025-07-10 政策公告**（官方页 https://support.google.com/googleplay/android-developer/answer/16296680）：给开发者至少 30 天合规期。【原文页存在，未核对】
- 2025 年把 AI 生成内容单列为受监管区域：AI 类应用须主动预防有害输出、须内置举报/标记机制、须在 Play Console 声明 AI 使用。适用 AI 交互为主功能者，**不适用** AI 仅作辅助的生产力工具。来源：REVERA 律所解读。【言词】
- **Spam and Minimum Functionality 政策**（长期条款，官方诉讼附件可查）：禁止重复低质应用；禁止「批量创建内容与体验高度相似的应用」；**自动化工具/向导/模板产出的应用不得上架，除非由该工具用户以个人开发者账号自行发布**。来源：法院附件副本。**与 Apple 4.2.6 同构，是现存最直接的反 slop 条款。**
- **2025 年执法数字**：下架/拦截 **175 万+** 违规应用（2024 年 236 万，**同比下降**）；封禁 8 万+ 开发者账号；拦截 1.6 亿条垃圾评分评论。来源：IT PRO Magazine（**2026-02-23**）+ Geek Room。**注意方向**：拦截量同比下降 26%，可解读为「违规减少」或「审查口径变化」，单一指标不支持任一结论。
- **Anthropic 报告案（转引）**：某中国开发者运营 **4,700+ AI 人设 / 20+ 约会应用**，两周内与 25,000+ 用户对话（约 236 万条消息），并设计「仅审核期间生效」的界面。**「AI 灌水已工业化」最强单案，但二手转引。**

**Microsoft Store**
- **政策 7.19：发布 2025-09-10，生效 2025-10-14**。新增 **11.16 生成式 AI** 条款：① 披露（元数据/商店详情页 + 上传时披露）② 解释（如何部署 + 产出内容性质）③ 防害（模型不得生成有害内容）④ 举报（用户须有途径举报）。来源：WindowsReport + Neowin + IT之家。官方原文 https://learn.microsoft.com/en-us/windows/apps/publish/store-policies（**未核对**）。
- **同一版本取消个人开发者账号 $19 注册费**（企业账号仍 $99）。**方向：MS Store 在降门槛拉供给，不在设门槛拦供给。**
- **弱**：政策中存在「1.1 独特功能与价值」类条款（韩文版 MS Learn PDF 命中标题），即 MS Store 也有最低功能性要求，**未拿到原文**。

### 问题 3：独立开发者自然流量被稀释的实证

- **机制层**：Apple 与 Google「没有给搜索结果增加任何位置」；Apple 自己的数据，**70% 访客通过搜索发现应用，其中 90% 不会翻过第 10 个结果**。结论句：平均一个应用「在 2026 年比 App Store 十七年历史上任何时候都更可能不可见」。来源：CEOWORLD 2026-03-11。
- Sensor Tower 分析：新软件的增长「**超过了 App Store 呈现相关内容的能力，事实上打破了传统的自然发现机制**」。
- **第一人称稀释感受（全部言词，无测量）**：Apple 开发者论坛帖「Shadow-banned by design: the App Store visibility crisis for independent developers」(https://developer.apple.com/forums/thread/782382)：无知名品牌开发者称「几乎不可能获得哪怕最低限度的可见性」，日下载从 2 跳到 300 再掉回 2。另帖「Is It Still Possible for Indie iOS Apps to Thrive in 2025?」：「现在找到用户需要更多的工作与时间」。帖 812993：「不做推广几乎没有自然量」。
- **缺口（诚实标注）**：**本路没有拿到任何「新应用冷启动 impressions 绝对值下降」的量化报告**。HN / Reddit 定向讨论**因配额耗尽未能执行**。

### 问题 4：GitHub 侧 AI 灌水与 star 通胀（**对农场直接相关**）

- **Star 通胀已被学术量化（本路最强证据）**：CMU 研究（工具 StarScout），**被 ICSE 2026 接收**：识别出约 **600 万颗假星，分布在 18,617 个仓库，涉及 30 万+ 欺诈账号**，检测准确率 81%。个例：某 111 星仓库**至少 109 颗是假的**。高知名度项目可疑星占比：**Union Labs（74,000 星，47.4% 可疑）**、**Langflow（147,000 星，47.9% 可疑）**。来源：36Kr/QbitAI 英文版 + 腾讯云社区 + 智源社区。**ICSE 2026 接收 = 论文存在，未读到论文本身。**
- **明码标价**：廉价批量账号 **$0.03-0.10/星**，老龄高仿真账号 **$0.80-0.90/星**；中文口径「**0.5 元/颗**」；至少 12 个网站 + 24 个活跃 Fiverr 卖家；含 5 年提交历史的账号农场约 $5,000。
- **动机是融资**：种子轮星数中位门槛 **2,850**、A 轮 **4,980**。假星已把 78 个注水项目推上 GitHub Trending，但效果**存活 <2 个月**。**「Trending 位只得 2 个月」这条对农场渠道判断直接相关。**
- **OSS 贡献侧 AI slop（含具体机构动作与日期）**：**curl 2026-01 终止漏洞赏金计划**（确认漏洞率 >15% → <5%）；**tldraw 2026-01 宣布自动关闭外部 PR**；matplotlib 遭遇自主 AI agent 提交 PR，被拒后发博客攻击维护者；Godot 维护者称 AI slop PR「越来越令人疲惫与沮丧」；Gentoo 据报道正考虑迁往 Codeberg。**GitHub 自身宣布措施**：允许维护者删除 PR、条件式 PR、按 CONTRIBUTING.md 过滤的分流工具。来源：heise.de + Telerik 博客 + GitHub wiki 研究汇总。
- **信任信号崩塌的学术框架**：arXiv 预印本《The Software Supply Chain as a Market for Lemons: A Multivocal Review of Trust Signal Collapse》：star/issue/PR/甚至「能编译的代码」都已可廉价伪造。

### 问题 5：Microsoft Store 独立开发者自然流量基线

**外部公开数据：未查到。** 本路未能执行定向检索（配额耗尽）。交叉引用（非本路独立产出）：
- Reddit r/SoloDevelopment（2026）：某 Xbox/MS Store 开发者自述 **Store Ads 投放 38,000 次展示 / 549 次点击 / 0 次付费下载**。
- 农场自身：pomodoro MS Store 上架 2026-09-04，首月 46 浏览 / 12 安装 / 转化 26%，Add-on acquisitions 2 且均自购，**商店自然流量约 1.5 浏览/天**，陌生人首单 0。**这是本次唯一「可对照真实口径」的 MS Store 基线数据，且与 Reddit 自述同构（展示量可观、付费转化为零）。**

**MS Store 的供给侧方向（本路独立产出）**：政策 7.19 取消了个人开发者 $19 注册费——**MS Store 正在降低供给门槛，而非提高**。在自然流量本就薄的渠道里降门槛，对已有开发者是净负面的稀释信号。

### 综合判断（5 句）

1. **供给洪水是真的，且量级已可量化**：App Store 新应用提交 2025 年 +24%~+30%、2026 Q1 同比 +54.8%~+84%，可能冲向年 100 万；同期下载量只 +2%~+3%——**差 20-40 倍**。
2. **洪水不缩减总盘子，它压碎中位数、抬高头部**：前 1% 应用吃掉 92.2% 的 IAP 收入，与 D 路 Steam 数据结构完全同构——**对单人开发者的真实影响不是「卖不动」，而是「看不见」，即分发成本被洪水抬高，而非定价空间被压扁。**
3. **三家商店的闸门并没有为洪水关闭**：2025 年政策动作全是「AI 内容披露与安全」条款，不是「禁止 AI 生成应用」；真正拦 slop 的仍是老条款；MS Store 同期取消 $19 注册费——**渠道在贴标签，不在关门，甚至还在开新门。**
4. **农场脚下那条渠道（MS Store）是洪水稀释最无缓冲的一条**：它三家中唯一公开降准入门槛，自然流量基线又极薄（农场 1.5 浏览/天、Xbox 开发者 38k 展示 0 转化），平台无公开 impressions 基线可对标——**在此渠道谈「供给洪水稀释」不是主因，主因是渠道本身就没有自然流量池可被稀释。**
5. **GitHub 这一侧可信度地基正被抽掉**：CMU/ICSE 2026 测得约 600 万假星、30 万欺诈账号，Langflow 近一半星可疑；curl 停赏金、tldraw 自动关 PR 标志维护者开始「关门」。**对农场意味着：GitHub star 作信任状与分发杠杆的价值在贬值，Trending 位即使买到也只有 <2 个月寿命**——农场「开源 + GitHub 分发」这条腿的复利假设需要重估。

### 未查到清单（含试过的关键词）

| 缺口 | 试过关键词 | 状态 |
|---|---|---|
| MS Store 新应用提交量 / AI slop 规模 | "Microsoft Store policy AI generated apps 2025 developer requirements" | **零命中**，三商店里 MS Store 供给数据完全空白 |
| MS Store 独立开发者 impressions/转化基线 | （B 路 6+ 组） | **全部零命中** |
| 新应用冷启动 impressions 绝对值下降量化报告 | "indie app developer discoverability harder 2026 impressions declining app store" | **未命中**，只拿到论坛第一人称抱怨 |
| HN / Reddit r/iOSProgramming / r/androiddev / IndieHackers 定向讨论 | — | **未执行**（配额耗尽） |
| a16z《So Many Apps, So Little Time》原文 | — | **未读**，数字全为摘要转述，**建议人工优先复核这一篇** |
| Appfigures / Sensor Tower 一手报告页 | — | 未读；四组数字口径不一致 |
| Apple 官方提交量统计页 | — | 未找到 |
| Google Play / MS Store 政策原文、Apple 反抄袭条款措辞 | — | 未读原文（URL 已记录） |
| CMU/ICSE 2026 假星论文原文 | — | 未读，数字来自中英文媒体转述 |

### 失效标志建议

**「供给洪水正冲垮流量与收费」被证伪（洪水不猛）：**
1. 供给增速回落到 ≤5% 并连续两季；2. 头部集中度（前 1% 占 92.2%）回落并连续两年下降；3. 商店发布明确以「AI 生成」为由的拒审/限量条款，或上架量实际回落；4. 商店给长尾加位置且新应用平均 impressions 上升；5. GitHub 假星占比显著下降（对标 6M 基线）且农场 GitHub 渠道 star/下载转化回升；6. 出现 2026 年后新上架、零投放、商店自然流量够养活的收入自述（目前零命中，第一条出现即需重估）。

**「洪水更猛 / 机制判断错」的对称证据：**
1. App Store 下载增速转负而供给仍 +50%+（绝对萎缩）；2. 商店单独计量并公开 AI 生成应用占比（>50% 则「AI 是主因」实锤）；3. 商店引入 AI 推荐/对话式发现，搜索排名战场作废；4. 独立开发者 CPI 同比 +50%+ 且自然量占比下降；5. 农场自身：log v1.x 付费层上线后 MS Store + GitHub 双渠道 90 天 impressions 不上涨或低于 pomodoro 1.5 浏览/天基线。

**给框架的一句校准建议**：本路数据支持的是「**洪水压碎了中位数可见性，而非压碎了付费意愿**」——D 路买断老兵（TablePlus/Snipaste/DOpus）2025-2026 全活着就是需求侧未死的证据。**A 路的正确用法是校准分发难度，不是校准收费可能性；把 A 路结论写成「没人愿意付钱了」会是引用错误。**

---

## E 路：小 + AI 杠杆 vs 大公司（2026-09-20 返回）

**方法说明（重要）**：WebSearch 本会话触顶（200/200）；WebFetch 对 en.wikipedia.org / businessinsider.com / agentmarketcap.ai / news.ycombinator.com 四个不同域名全部失败，判定为系统性网络封锁。**所有证据来自 WebSearch 聚合摘要，无一条经原文 fetch 二次核验**。摘要自身数字打架处标「源冲突」。

### 1. 小胜大实证（2023–2026）

**1.1 Cursor / Anysphere vs GitHub Copilot（用户指定 Canonical 案例）**
- Cursor：~300 人（2025-08）；ARR 轨迹 $100M (2025-01) → $500M (2025-06) → $1B (2025-11) → $2B (2026-02) → ~$3B (2026-05)；1M+ DAU；**累计融资 $3.38–3.5B**，Series D $2.3B @ $29.3B 估值（2025-11）。
- Copilot：第三方估算 $900M–$1.1B ARR；累计 20M 用户，**付费订阅 4.7M**（FY26 Q2）。
- 赢在哪：方向缝隙 + 速度 + 品味/端体验。「AI-native 编辑器」新形态 vs 微软把 AI 贴在既有 IDE 上。
- 后来：仍在高速增长，**未被 Copilot 免费层 + M365 捆绑压死**（关键）。
- ⚠️ 「SpaceX $60B 收购 Anysphere」仅见 crypto/aggregator 低质站点，**未证实，不要写进框架**。

**1.2 Midjourney — 唯一真正的自筹资金大收入样本**
- ~$500M（2025 估算），$50M(2022)→~$500M(2025)，全订阅，**无免费层/无广告/无 API/无市场预算**；~107 人（源冲突：10/40/107/163 四说）；**$0 外部 VC，完全 bootstrap**，发布后数月盈利；用 Discord 做零成本分发。2026 仍在发版（V8.1）。**私有公司无审计财报，全部估算。**

**1.3 ElevenLabs — 融资型「小团队」**：ARR >$330M（2025 底），~330 人；累计融资 $780M，估值 $3.3B→$11B→传 $22B。OpenAI 上线 TTS 数周后反手融 $180M。**典型「小团队+大额融资」，不是 bootstrap。**

**1.4 Perplexity vs Google — 请谨慎解读**：ARR ~$450M（2026-03）；但 **Google 仍占全球搜索 ~89.87%**，Perplexity 在传统搜索份额小到进不了榜。**这不是「小胜大」，是「小活下来并在新子品类里领先」**。写成「Perplexity 击败 Google」是夸大。

**1.5 中文圈**：
- **死了么 / Demumu**：3 名 95 后、约 1 个月、成本 <1500 元，冲到中国区 App Store 付费榜第一，估值约 1 亿元。**上线即被抄**：App Store 出现 10+ 同类；一名工程师用 Gemini **6 小时**做出「活着么」，零代码，两天 2000+ 用户。
- **小猫补光灯（陈云飞）**：零编程基础，AI 约 1 小时做出，Pro 定价 1 元，上架 4 小时冲付费榜第一；两款下载各约 30–50 万，Pro 累计收入约 30–40 万元；**上线当天出现大量抄袭版本**。
- **求职精灵（奕甫）**：一人公司，注册用户近 100 万，年化「千万级」。

**1 的裁定**：真正「小胜大」的样本（Cursor/ElevenLabs/Perplexity）**全部是大额融资的小团队**；唯一自筹的 Midjourney 也不是单人。**没有一条证据显示「单人/自负盈亏」击败了大公司。**

### 2. 大公司反击实证

- **Apple Sherlocking（最硬的系统化反击证据）**：WWDC 2025 一次 sherlock 掉一大批——Enhanced Spotlight → Raycast/LaunchBar；Call Assist → Robokiller/Truecaller；Wallet 包裹追踪 → 包裹 App；航班 Live Activities → Flighty；菜单栏控制 → Bartender/Ice；Spotlight 剪贴板搜索 → Paste/Pastebot。（MacRumors/Macworld 2025-06-11）2024：NPR 报道 Apple 抄 TapeACall/Grammarly/1Password/Otter/AllTrails。**开发者不敢发声，因为 Apple 既是竞争者又是守门人。** 2026-01：Reincubate 就 Camo 被 Continuity Camera 覆盖起诉 Apple。
- **分发/捆绑/免费化反击**：微软 Copilot **M365 捆绑** + 免费个人层 + 企业采购，90% 财富 100 强在用。**但关键反证：这套打法没有杀死 Cursor。** → 「分发碾压」不是万能的，端体验可以扛住。Google 策略差异：AI 包进 Workspace 订阅不单独收费。
- **平台 API 层覆盖（OpenAI DevDay 2023，2023-11-06）**：GPT-4 Turbo / GPTs / Assistant API 直接覆盖一批 wrapper——PDF 分析类（ChatPDF/AskYourPDF/PDF.ai/ChatOCR）受重创；Jasper（曾估值 $1.5B）2023-07 裁员；创始人名言「Sam Altman 杀了我 $3M 的创业公司，我只拿到 $500 的 OpenAI API 券」。
- **大厂模型厂顺手覆盖（2026）**：Anthropic **Claude Cowork** 冲击 RiseAI；Google 把 **Lyria 3** 塞进 Gemini 威胁 Suno/Udio；**Claude Tag**（2026-06）打到 Viktor.com。Google VP Darren Mowry 公开表态：只做模型套壳的创业公司「已经没有位置了」。

### 3. 抄袭速度实证

**「任何人一周能抄你」→ 部分成立，且比一周更快，但「抄功能」≠「抄成产品」**
- **Base44（Maor Shlomo，以 ~$80M 卖给 Wix）**：直言「做一个 vibe coding 工具相对容易」，界面的「魔法时刻」是最容易复刻的部分；每个功能竞争者能在**数周到数月**内抄掉；只靠 prompt 技巧的创业公司「很难有护城河」。抄不走的：内置数据库、鉴权、用户管理、分析等底层 + 大量集成层与调优。（Business Insider 2025-11）
- **死了么**：走红后 10+ 雷同 App；工程师 **6 小时**克隆并两天 2000+ 用户。另有工程师称 2025-03 已用 Cursor + Claude 3.7 **约 5 分钟**生成完整原型。
- **levelsio**：称看到 Wispr Flow/Granola/WHOOP 类工具被「**一天之内**」逆向成免费/开源版。辩论焦点从「构建难度」转向「注意力成本」——技术门槛下降后竞争「拥挤 100 倍」，变成「每周都是营销周」。
- **Wordle/NYT**：NYT 对开源克隆 Reactle 发 DMCA，被 fork 约 1,900 次，一次性波及近 2,000 个克隆。但克隆未灭绝。
- **反面论据**：**快代码 ≠ 成品软件**——设计、QA、调试、打磨、部署、维护仍耗时；Wispr 克隆指南列举代码签名、macOS 权限、更新、自助支持等持续成本，用户也因「缺个人词典」拒绝切换。
- **Stack Overflow 被 AI 蚕食（最硬的「大平台被 AI 掏空」证据）**：提问量峰值 ~200,000/月(2014)→109,000(2022-11)→1.34M(2022 年)/790K(2023)/400K(2024)/110K(2025)；2025-12 为 3,312（同比 **-81.5%**）；2026-06 仅 **1,072**，史上最低完整月。活跃用户从峰值 ~20 万降到 2026 初的几千人。
- **但同一案例的反向教训**：Stack Overflow **公司本身健在**——收入 4 年近乎翻倍，亏损从 FY2023 $84M 收窄到 FY2025 $22M，靠裁员 28% + 与 OpenAI 合作 + OverflowAI 转型。**「社区被 AI 掏空」≠「公司死掉」。**

### 4. 赢了守不住 vs 赢了守得住

**守不住的（被抄死/被免费化压死/被收编）**

| 案例 | 下场 | 机制 |
|---|---|---|
| GPT wrapper 群（Jasper/ChatPDF/AskYourPDF/PDF.ai/ChatOCR） | DevDay 2023 后价值主张蒸发；Jasper 裁员 | 单模型套壳，无自有资产 |
| Wordle 克隆（~2,000 个） | 一次 DMCA 全灭 | 无法律资产，玩法不受保护 |
| RiseAI / Viktor.com | Claude Cowork / Claude Tag 上线后受损 | 与模型厂产能直接重叠 |
| 死了么之后的同类 | 品类被 6 小时克隆商品化 | 极低技术壁垒，靠运气走红 |
| Base44 | 被 Wix 以 ~$80M 收编（「赢但退出」） | 创始人自认功能可抄 |
| Stack Overflow 社区 | 提问量跌到 2008 水平 | 被 AI 直接替代 |

**守得住的**

| 案例 | 守住机制 |
|---|---|
| Cursor | 多模型集成 + 端体验；Claude Code/Codex 上线后仍增长 |
| ElevenLabs | OpenAI TTS 上线后反融更多钱；语音质量品味 + 企业销售 |
| Perplexity | ChatGPT 存在下仍长到 $21B+；切走「高意图答案引擎」子品类 |
| Midjourney | 自筹、订阅、零营销、Discord 分发；纯品味驱动 |
| Chatbase | 创始人自建底层（数据库/鉴权/集成），3 年 $0 融资到 $9M ARR |

**「守住」的公认机制（多来源收敛）**：① 复利型专有数据（Harvey 律所文档库；EvenUp 20 万+ 伤害案件数据集，$2B 估值）② 深度工作流嵌入（切换 = 重新培训团队、丢掉几百模板）③ 社区/信任/品牌 ④ 分发与注意力 ⑤ 多图层叠加的「创新栈」（Jim McKelvey）——单层优势打不过巨头，多层相互强化才抗揍。

**「moat 测试」**：**40% 测试**——若明天出现功能完全相同但免费/更便宜的 AI 对手，**你的用户里有没有 40%+ 拒绝切换**？没有 = 护城河本来就很薄。

### 5. 单人/微型 bootstrap 特别证据（与用户处境最贴近）

| 案例 | 团队/融资 | 数字 | 分发方式 |
|---|---|---|---|
| **Chatbase**（Yasser Elsaid） | 单人起步，**$0 外部融资**，现 30 人 | **$9M ARR，3 年** | 自建 B2B 产品 |
| **SEOBOT**（John Rush） | 无融资、无媒体报道 | **Stripe 实证 $57,068/月**，761 个 $49/月 订阅 | 定位「SEO 代理」而非工具 |
| **AudioPen**（Louis Pereira） | 非技术背景，半天做完 | **$15–20K/月，维持 2 年+** | 仅年付（筛用户、降 churn） |
| **Launch Fast**（Hasaam Bhatti） | 非技术亚马逊卖家，Cursor 48 小时 | 首月 $10K MRR，稳定 **$30K MRR** | — |
| **YourMove AI**（Dmitri Mirakyan） | bootstrap 无融资 | **$30K/月，80% 毛利**，1M+ 用户，已出售 | — |
| **Sleek.design**（Mattia Pomelli） | 单人，3 周 | **$10K MRR，零营销费** | X/Reddit 演示 |
| **NerdSip** | 副业，< $500 预算 | **$3,000/月**，100+ 付费订阅 | — |
| **小猫补光灯**（陈云飞，中国） | 单人、零编程 | Pro 累计 **~30–40 万元** | App Store 付费榜 |
| **求职精灵**（奕甫，中国） | 一人公司 | 近 100 万注册，年化「千万级」 | — |
| **Permito**（**反面样本**） | 单人 5 个月 | 仅 **~$650 MRR**，45 付费用户，**正因 B2C 流量数学不成立转 B2B** | — |

**宏观数据（Stripe）**：单人创业者年收入 >$1M 的群体 **2023→2025 翻倍**；>$10M 的增长近 **3 倍**。工具栈月成本从 2019 的 ~$5,000 降到 2026 的 **$85–200**（约 -96%）。

**必须剔除的伪样本**：媒体常把 **Ben Broca**（1 万付费客户、预计 $10M 收入）当「一人公司」——它**融了 $30M**。Cursor/ElevenLabs/Perplexity 都不是 bootstrap。

**共同模式**：① 卖「结果/代理」比卖「工具」定价权高得多 ② 垂直聚焦 + **预先存在的分发渠道** ③ 增长靠 SEO/X/Reddit/TikTok/PR 自然流量，非付费广告 ④ **真实天花板**：最强纯 bootstrap 单人案例是 Chatbase $9M ARR；大多数在 **$3K–$30K MRR**；Permito 这类被迫转型的才是多数。

### 6. 禁区位验证 —— **本轮证据严重不足，请勿据此下结论**

- **「本地优先/无账号/无遥测/一次性买断的独立产品，大公司是否确实不跟进」——未查到成体系证据。** 整块缺失。
- 唯一间接相关：Google 把 AI 包进 Workspace 订阅不单独收费（属捆绑，不是禁区位）；DevDay 2023 的覆盖方向全是云端 API/订阅，恰好没做「本地+买断」，但只算「一个案例没去」，不构成结构性证明。
- 结论：**禁区位部分目前是空的，需要补搜**（关键词建议：`one-time purchase software vs SaaS big tech not interested`、`offline first app acquired by big company 2025`、`大厂 为什么不做 本地 离线 买断 软件`、`Microsoft local-first app competitor 2026`）。

### 核心区分表：「单人 bootstrap 的小」vs「小团队+融资的小」

**证据强烈支持这两者必须分开讨论，且结论相反。**

| 维度 | 小团队 + 大额融资 | 单人/2–3 人 bootstrap |
|---|---|---|
| 代表 | Cursor（$3.4B 融资/~300 人）、ElevenLabs（$780M/~330 人）、Perplexity（~$23B 估值） | Chatbase（$0/起步 1 人）、SEOBOT、AudioPen、NerdSip、小猫补光灯 |
| 能否「比大公司牛逼」 | **有实证**：Cursor 在微软免费捆绑下发把 Copilot 赛道打成自己的主场 | **无实证**。天花板是「在小缝隙里赚到陌生人的钱」，不是取代大厂 |
| 收入量级 | $1B–$3B ARR | $9M ARR（最高纯样本）～ $3K–$30K MRR（典型） |
| 赢的结构因素 | 方向缝隙 + 资本换速度 + 端体验品味 | 垂直窄缝 + 分发渠道（X/Reddit/App Store）+ 低价或年付 |
| 输的机制 | 与大厂产能正面对撞（RiseAI、Viktor） | **没人看见**（渠道薄）、**被 6 小时克隆**（壁垒薄） |
| 「小」的含义 | 相对微软小，绝对不小；本质是**风险资本代理的小** | 字面意义的小；**自负盈亏 = 零容错** |

**判定**：用户命题里的「小团队」和「个人」被混为一谈，但证据显示这是**两种完全不同的博弈**。`小 + AI 杠杆 → 比大公司牛逼` 只在**拿到大额融资**的那一支成立。单/微型 bootstrap 那一支，AI 杠杆把「做出来」的成本打到近乎零，却**同时把对手做出来的成本也打到了近乎零**——它降低的是入场门槛，不是竞争门槛。

### 综合判断

证据支持「方向选对就能比大公司牛逼」的**前一半**，在**后半句断裂**。

1. **方向确实是小玩家唯一可控的变量，且确有回报**：Cursor 押 AI-native 编辑器、Midjourney 押 Discord 消费级生图、ElevenLabs 押语音质量、Chatbase 押自建底层——都是在位者当时没占的位，且都活下来了。
2. **但「小」的胜率证据全部来自有资本供给的小**：Cursor/ElevenLabs/Perplexity 三家合计融资超 $4B，没有一家自负盈亏。把它们当作「个人也能做到」的论据，是把风险资本的能力误记成了 AI 的能力。
3. **AI 同时抹平了攻方和守方的技术沟壑**：死了么 6 小时被克隆、Base44 自认功能几周可抄、DevDay 一夜覆盖一批 wrapper——「任何人都能一周抄你」有实证支持，且比一周更快。
4. **真正的瓶颈已迁移到分发，不在构建**：几乎所有 2026 证据都指向同一句——技术壁垒下降后竞争变「拥挤 100 倍」，胜负手是注意力/渠道/品牌/数据，而不是产品本身。**与农场此前三次结案结论（瓶颈在分发不在找品）完全一致。**
5. **能守住的，守的都是「抄不走的资产」**（复利数据/工作流嵌入/社区信任/多图层），没有一个靠功能领先守住。Cursor 自己承认是「集成最好的模型 + 做端体验」，不是模型更强。

### 未查到清单（诚实边界）

**因工具限额未执行**：Copilot 免费捆绑具体压死了哪些具名初创；Google AI Overviews 对出版方流量的量化冲击；**本地优先/无账号/一次性买断产品大厂是否跟进（Q6 核心证据，整块缺失）**；Chatbase 原始访谈原文核验；Wordle 原作者出售后境况金额；中文圈「小战胜大厂」案例（只拿到「小赢一笔」和「被抄袭」）；Anthropic/OpenAI 覆盖具名初创的更多配对与日期。

**已查但结果为空/矛盾**：Cursor ARR 时间线（聚合摘要自相矛盾，采用多源一致版本但标估算）；**Cursor 的 SpaceX $60B 收购（仅低质站点，未证实，不要写进框架）**；Midjourney 员工数（10/40/107/163 四说并存）；全部数字均为商业媒体或第三方估算，**无一条来自审计财报或官方披露**。

**建议补搜关键词（待额度恢复）**：`one-time purchase software offline big tech not interested 2026` / `Microsoft enters local-first app category killed indie 2026` / `大厂 不做 本地 离线 买断 软件 2025` / `Microsoft Copilot bundling killed competitor startup antitrust hearing`

### 失效标志建议

**出现以下任一，说明「小 + AI 的进攻面」是幻觉或已过期：**
1. **克隆时钟 < 30 天**——上线一个月内出现功能相当、有获取量的克隆（死了么=数天、小猫补光灯=当天、Base44=数周），说明方向没壁垒，AI 杠杆在中立地帮对手。
2. **40% 测试不通过**——问现有付费用户：明天出现功能相同但免费的 AI 竞品，你会不会走？拒绝切换者 < 40% = 护城河本来就不存在。
3. **差异化写在功能清单上**——若「我们做了什么功能」是唯一卖点，而功能可在数周到数月窗口内被复制，该卖点已进入倒计时。
4. **大厂把同一能力原生放进免费层**（Apple sherlocking / 微软 Copilot 捆绑）——**注意：这条只在产品没有「抄不走的资产」时才致命**（Cursor 扛住、RiseAI 没扛住）。
5. **产品是单模型 API 套壳**——DevDay 2023 剧本，2026 由 Anthropic/Google 复刻。
6. **增长完全依赖某个平台的自然流量**——Stack Overflow 是极限样本。
7. **分发计划写不出「已有渠道 + 具体人群」**——所有 bootstrap 成功样本都带**预先存在的分发渠道**；唯一的纯流量数学失败样本 Permito（$650 MRR）恰好就是没有。
8. **反向失效标志（防止把结论用反）**：若出现「无融资单人开发者做出 >$50M ARR 且正面打退大厂同功能产品」的**可查证**案例，则本报告「bootstrap 天花板约 $9M ARR / 无法击败大厂」的结论应被推翻——**目前未查到任何此类案例**。
