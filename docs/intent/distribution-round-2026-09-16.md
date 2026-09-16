# 分发轮 · 工作文档（2026-09-16 开局）

> 用户裁决（2026-09-16）：**转分发，暂停挖新品**。理由：七件建成的产品渠道实测趋零，瓶颈不在供给在发现。
> 本文件是进行中会话的可重启边界。

## 任务定义（保护项）

把资源压到「让已建成的产品被看见」。范围：
1. 核实 pomodoro 商店状态（已完成，见下）
2. 把 log v1.0 发布做扎实
3. 领先指标埋点 + 商店 impressions + 内容平台直链（用起 CID 95/5 分成）

**不在范围内**：开新产品（选型轮已关闭，遗留资产见 `product-selection-round4-2026-09-16.md`）。

## 第一方渠道实测（2026-09-16，本轮最重要的产出）

### GitHub 渠道 —— 趋零
danqing-pomodoro 发布 5 周（v0.1.0 2026-08-12 / v0.2.0 09-03 / v0.2.8 09-05）：
- **0 star / 0 fork / 0 watcher**
- 14 天浏览 **20 次 / 2 unique**（≈ 只有作者自己）
- 14 天克隆 179 / 78 unique —— 浏览 2 人却 78 克隆，判为爬虫/镜像噪音，**不是人**
- 发布下载量合计 **约 6 次**（v0.1.0:2 + v0.2.0:1 + v0.2.8:3），含自测
- 来源：github.com×8 / apps.microsoft.com×1 / 掘金×1

### MS Store 渠道 —— 已上线，但同样趋零
- **pomodoro 商店页是活的**：`https://apps.microsoft.com/detail/9P3W6W1SR6DS` HTTP 200
  （标题「丹青-番茄钟」，发行者 `14uncle`，价格 0）
- **`ratingCount: 0`** —— 上线至今零评分（无评分≠无安装，但与 GitHub 侧一致指向同一结论）
- 商店页无内购字样出现（add-on `9P4B2MPB8HNN` 是否已生效待 Partner Center 核实）
- 注：此前农场记忆写「上架进行中/卡点 Partner Center」——**已过期，页面是活的**

### 结论
**两个渠道合起来是「双双趋零」。农场的问题被精确定位在「发现」（discovery），不在供给。**
推论：① 再建产品不会改善处境；② 「发布了就会有人来」在 GitHub 与 MS Store 上**均被证伪**；
③ 领先指标（商店 impressions / acquisition）目前**只有 Partner Center 拿得到**，是下一步的第一手数据源。

## 关键未知（必须由用户提供，我拿不到）

1. **Partner Center 的 acquisition 报表**：impressions / page views / conversions / acquisitions 各是多少？覆盖多长周期？→ 这是判断商店渠道是「零」还是「低但有转化」的唯一依据。
2. **add-on 是否已生效**（¥18 完整版，Store ID `9P4B2MPB8HNN`）→ 决定 pomodoro 的「首单外检」（前提③）现在能不能判。
3. **商店页上线日期** → 决定「上线多久了」这个分母。

## ⚠️ 本轮最高优先动作（数据已存在，只是没被读）

`danqing-pomodoro` 记忆实测记录：**父应用 2026-09-04 已发布 + add-on 同日上线并真机验证过购买链路**（用户自购 2 单 ¥18 验证管道，明确不算外检）。当时即写明「**Add-on acquisitions 报表 09-05 起可见**（见解→Add-on acquisitions；数据延迟 24-72h）」。

→ **首单外检的数据已经积累 11 天**。去 Partner Center 读三张表：
- **见解 → Add-on acquisitions**（订单数 —— 外检的直接答案）
- **下载报告 → 销售/交易**（下载量）
- **见解 → 应用表现**（impressions / page views / conversion rate —— 农场缺的领先指标基线）

**这三个数字决定整个分发轮的算账基准**（也是农场「30 天校准机制」承诺要回填的东西）。

## 进行中（2026-09-16 发出）

| # | 调研区 |
|---|---|
| 1 | 内容平台渠道实证（中文圈 B站/小红书/知乎/掘金/V2EX/少数派 + 英文圈 HN/Reddit/PH + GitHub Trending/Alternativeto），带案例与转化证据、门槛、执行形态 |
| 2 | MS Store 分发机制（真实规模 / 那个「90% 应用 impression<1000、35% 为零」数据的查证 / ASO 机制 / 精选位 / Partner Center 数据字段与滞后 / 开发者工具类目案例 / winget 覆盖面） |

**摄取约定**：报告回来后只提「结论 + 证据 URL + 可执行形态」进本文档。

## 渠道调研结果 · 压缩摄取

### ② MS Store 分发机制（已完成 2026-09-16）

> 方法学限制（agent 自披露）：本环境 **WebFetch 全域被策略封锁**（含 example.com，非域名问题）+ **WebSearch 配额 200/200 耗尽**，故官方文档内容均经搜索摘要转述、未逐字核对。**带数字的结论用于决策前应人工开原页复核**（尤其：Partner Center 延迟 3 小时、关键词 7 个/40 字符/21 词、winget 的 `Validation-Indirect-URL` 规则）。第一方数据只有 1 个样本，**当方向验证不当基准值**。

**核心结论：MS Store 对 LogLens 是「分发层」不是「获客层」。商店自然发现 ≈ 0。**

**决定性第一方证据（StreamVox，英国单人开发者，实时 AI 翻译，上架 10 天自述）**：
| 指标 | 值 |
|---|---|
| 商店页浏览 | 180 |
| 下载 | 82（转化 45%） |
| 收入 | **$0** |
| 来源拆解 | 自发 Reddit/Twitter/FB **~140** · Product Hunt ~20 · **商店自然搜索仅 10–20** |

原话定性：「**MS Store 只是一个下载按钮，不是营销引擎**」「上了架才知道发现完全是你自己的事」；他现在的精力分配是 **70% 营销 / 30% 写代码**。搜索体验：搜 "AI subtitles" **什么都不显示**；鸡生蛋（没曝光→没评论→没用户→没曝光）。
（来源：Product Hunt 自荐帖 + 产品页，2026）

**佐证链**：
- 排名输入 = CTR / 评分 / 评论 / 下载量（**唯一官方说明是 2015-07-24 博客，已过期 11 年**）→ 冷启四项全 0，**结构上不可能被搜到**
- 官方口径「商店 MAU 2.5 亿」是**「Store 应用月活」不是「在商店搜软件的人」**（Store 是预装组件，承担更新/库管理/卸载）——**用它推潜在用户数是实质性误读，不许写进算账**
- 开发者实测商店报表长期全 0（[MS Q&A 5932789](https://learn.microsoft.com/en-us/answers/questions/5932789/microsoft-store-partner-center-acquisition-and-ana)）；2026-05 起安装量异动卡在 page view→install（[SO 78999244](https://stackoverflow.com/questions/78999244/application-installation-issue-from-the-microsoft-store)）
- **「90% 应用 impression<1000 / 35% 为零」查证结论：16 种措辞检索零命中，无任何出处 → 不可当事实使用**（我方引入该说法时未核过来源，记一条教训）
- 唯一正向案例是**游戏**（发行商自宣材料），**不可外推**到开发者工具

**领先指标机制（这是农场最需要的，且比预期好一个量级）**：
- **Recent data 视图：约 3 小时刷新**（官方口径从 ~30 小时降下来），提供 Last 24h / 48h，**按小时刷新**
- 字段：**PageViews / Installs / Conversion Rate / Install Success Rate**
- **粒度陷阱**：Recent data 按小时唯一设备、Daily 按天唯一设备 → 小时数之和 ≠ 日报数；官方明说仅供早期趋势、**不是最终计数**
- **不要建 API 看板**：Store Analytics API 与 UI 数值已证实严重不符（funnel 接口 4 vs UI 48）

**上架的坑（新账号，与农场直接相关）**：
- **IAP 对新账号锁定 30–45 天**（StreamVox 实盘支付全挂、已购用户报 Page not found，沙盒正常、PC 状态 Complete；无官方文档）→ **支持「商店只放免费层、付费走站外」的方案**；若仍走商店内购，**提前 45 天做一次真实小额购买实测**
- `runFullTrust` 理由框字符上限极低且**静默截断**（写两句不写两段）；政策 11.16 生成式 AI 内容（LogLens 无此输出，可跳过）；WACK 预检 + [官方自查清单](https://learn.microsoft.com/fr-fr/windows/apps/publish/publish-your-app/avoid-common-certification-failures)
- **关键词 7 个**（每个 ≤40 字符、去重后 ≤21 词、不显示给用户、避免品牌词与 free/best 泛词）；Partner Center 有 AI 推荐关键词功能
- **推荐位/精选：完全找不到入口 → 从计划中划掉，不纳入算账**
- **winget 与商店是两条独立通道**（上架商店 ≠ 进 winget）；覆盖面**未查证**，**不算进量的预期**，只当低成本补一条安装路径

**由此推出的打法（agent 建议，我认同）**：
1. **商店 = landing page + 安装器 + 埋点，不是渠道**；「靠内容平台直链导流」**是唯一可行的路，不是备选**
2. 所有内容平台外链一律走 **CID 直链**（95/5 分成 + 归因）
3. **上架后先做「零推广基线」**：一周内什么都不发，看自然 page view 是否真为 0 → 若是，此后 page view 可 100% 归因于自己的推流，**领先指标的分母才干净**
4. 不要为 ASO 投入大量时间（现行规则无公开文档）；不要指望推荐位；不要把 2.5 亿 MAU 写进算账
5. **待人工补的洞（5 分钟，我做不到需用户开商店客户端）**：Developer Tools 类目 top free/paid 榜的实际**评论数**（评论量≈下载量的粗代理）——这是给「商店能带多少量」赋值的唯一第一手依据

### ① 内容平台渠道（已完成 2026-09-16）

> 方法学：WebFetch 全域被拦 + WebSearch 配额耗尽，但 agent 发现**本机 curl 直连可用**，故凡标【实测】的都是当天一手 API 数据。

**🔴 机制性发现一：决定零粉丝开发者生死的不是平台大小，是「时间序 vs 推荐逻辑」。**
唯一一条同时段同产品横评五渠道的一手证词（V2EX t/1225594，2026-07-07）原话：「效果最好的是豆瓣 women in tech 小组……其次是 v2ex……**小红书和 x 很难，估计 0 转化**。因为豆瓣和 V2EX 都有**时间序排列**，所以我很容易被大家看见。而小红书和 x 是**推荐逻辑**，我这种小号，很难有可观的浏览量……即刻，我发过 2 个组，**互动可以说是 0**。」
→ **推荐逻辑平台对零粉丝默认 0 曝光；时间序阵地（V2EX 节点 / 豆瓣小组 / 小众软件发现频道）对零粉丝一视同仁。** 这是选渠道的第一判据。

**🔴 机制性发现二：HN 品类不死，但 Windows GUI 结构性吃亏。** 【实测】拉全量 Show HN（hn.algolia.com）发现**双峰分布**——
| 形态 | 分数 |
|---|---|
| web / TUI 日志查看器（Telescope 172 / Nerdlog 134 / Kubetail 126 / Logdy 111） | **41–172** |
| **Windows/Mac 桌面 GUI**（Chipmunk Rust+Egui **1 分** / Jsonl Viewer 2–3 分） | **1–3** |
→ HN 人群不会为看一眼去装 Windows exe。**直接发 Show HN = 拿 1 分。**

**🔴 机制性发现三：Show HN 中位数 2 分。**【实测】最近 100 条 Show HN：**中位数 2.0 / 均值 3.3 / 72% 零评论 / 37% ≤1 分**。拿到 1 个陌生人的赞已进前 1/3。

**🔴 最强的一组硬数据（我认为本轮最有价值的发现）**：klogg 的 `v22.06` release（**2022-06-13，四年未更新**）**157,260 次下载**；Chocolatey 全生命周期仅 15,538；Homebrew cask 30 天 121。对照 lnav `v0.14.0` 44,406。
→ **GitHub 直下比包管理器高约一个数量级，且 klogg 靠的不是发布事件，是四年累积的 GitHub 搜索位置。**
→ **LogLens 的病灶不是「没发够平台」，是 0 star = 在任何 GitHub 搜索里根本不存在。**

**✅ 有效渠道（按实际带量能力排序）**

1. **小众软件 / Appinn 发现频道 + 主动私信推荐人** — 唯一有「0 预算 → 90 天 1,800★」一手复盘的渠道（Catime 作者，meta.appinn.net/t/topic/71215，2025-05-09，同帖 1,326 浏览）。**关键动作不是投稿而是私信**——作者原话「我基本都是一个个主动私信联系的……比你想象中有效得多」；点名 HelloGitHub / 小众软件发现频道 / LINUX DO。**本机实测可达（200）**。量级：发现频道新帖 46–567 浏览，常青帖 21,540 浏览。**闸**：「售价超 100 元/年的产品需选付费商业推广」（LogLens $29≈¥210 **可能落在付费线下，边界待问管理员**）；「不欢迎 SEO 项目」（以 GitHub 开源项目形态提交更安全）。
2. **V2EX `分享创造` 节点** — 唯一有一手证明「能带下载」的中文渠道（amybiubiu 三周 400+ 下载，「**V2EX 能看到我的帖子时，下载量就比平常多**」）。零粉丝门槛，但版规要求**附带思考**（拒绝纯丢链接硬广，须写开发故事/技术选型/痛点）。量级：发布 1h 74–276 点击 / 1 天 600–1,000 / 热帖 2,000–3,000。**⚠️ 本机不可达，需代理。**
3. **少数派 `App+1`（送码）** — 曝光天花板最高（日 IP 40 万），【实测】「开发者自推 + 送码」是被接受的常规形态（`sspai.com/post/111645` 2026-07-04 存在）。**但冷水**：Matrix 自由写作虽无需申请，公开列表 engagement 实测只有 **0–2 赞**；真正流量在编辑推荐的 App+1/首页，**不可控**。建议作并行的第三条（写作成本可复用 V2EX 那篇）。
4. **HN 技术博文（不是 Show HN）** — 写 `How I indexed 1GB of logs in 425ms in Rust`，让别人投。**这是 1–2 个月的养号投入，不是发布日动作。**

**❌ 死胡同（全部带证据）**：Product Hunt（零粉丝实测 1–11 upvote，月 UV 从 ~5M 降到 ~2.8M，付费操纵公开化）· X/Twitter（2025-03 起外链降权 **-94%**）· GitHub Trending（**吃 24-48h star 增速不吃总量**，0 star = 没有输入）· Awesome 列表（逐渠道对照实验实测 **no effect**；且 awesome-rust 需 50+ star 门槛）· dev.to（`windows` tag 近一年 Top8 reactions = **0,1,1,1,4,5,7,7**）· 掘金（读者是同行不是买家）· 小红书（**《交易导流违规管理细则》2025-03-12 生效**，外链彻底封死，处罚至永久封号）· B站（**付费推广实测无效**——Catime 买「必火」¥20 多「只有几个点赞」）· 知乎（**本次零带数字案例**）· 微信公众号（是承接种不是获客渠道）· 什么值得买（无开发者自荐通道）· 百度贴吧（转化率几乎为 0）· Lobsters（邀请制 + 强反自推）· winget/Scoop（**不是无效，是永久无法归因**）

**⚠️ 数据可信度警告（agent 主动标注，我认可）**：**所有「注册转化 15-25%、付费 2-5%」的漂亮数字全部来自卖营销服务的博客**，与唯一的第一方 nginx 日志（**15,000 浏览 → 8 注册 = 0.2%**）差两个数量级。**不要把卖方话术当基准。**

**🌐 本机可达性地图【实测】（操作层，非渠道结论）**：
- **可达**：sspai.com / juejin.cn / bilibili.com / smzdm.com / meta.appinn.net / dev.to / codeload.github.com
- **不可达（需代理）**：**www.v2ex.com / news.ycombinator.com / www.reddit.com / x.com / github.com**（GitHub 时好时坏，DNS 级抖动）
→ **英文侧全渠道 + V2EX 从命令行都需要先解决出口。**

## log v1.0 发布状态（发布链已备到最后一步）

已就绪：代码闭环（184 测试绿）/ MSIX 已签（指纹 `CFC2703D`）/ 便携包已打（`danqing-log-v1.0.0-win-x64.zip`，
sha256 `6d95317c…`）/ 商店文案 `docs/ms-store-copy.md` / 隐私政策 `docs/privacy-policy.md` /
发布说明 `docs/release-notes-v1.0.0.md` / 演示素材 `release-archives/log/demo/`（含 GBK 中文样本）
**未做**：① 截图（用户从已装的 MSIX 拍）② 打 tag ③ GitHub Release ④ 商店提交
**用户计划**：自用 2 周后发布（≈ 2026-09-30）

## 合成：渠道判断（2026-09-16，两份调研齐全后）

**1. 三层定位彻底分明**

| 层 | 结论 | 证据 |
|---|---|---|
| **MS Store** | **分发层，不是获客层。自然发现 ≈ 0** | StreamVox 10 天 180 浏览 / 自然搜索仅 10–20；2015 年（已过期 11 年）的排名文档 + 冷启四项全 0 |
| **内容平台** | **唯一获客路径**，且**只有时间序平台对零粉丝有效** | 一手横评：时间序（V2EX/豆瓣/小众软件发现频道）能被看见；推荐逻辑（小红书/X/抖音）对零粉丝默认 0 曝光 |
| **GitHub 站内** | **可能是最大的下载面，而农场完全没做** | klogg 死 release 157,260 下载 vs Chocolatey 15,538 |

**2. 🔴 本轮最高价值发现（已被本机实测证实）**：danqing-log 仓库 `description: null` / `topics: []` / `homepage: null` —— **它在 GitHub 搜索里等于不存在**。这不是「渠道没铺够」，是**最基础的存在感缺失**，且修复成本 = 5 分钟，不依赖任何平台、不需要代理、不需要内容生产。**证据强度高于本报告任何一条渠道结论。**

**3. 行动优先级（按 证据强度 × 成本 排序）**

| # | 动作 | 成本 | 证据强度 | 依赖 |
|---|---|---|---|---|
| **1** | **GitHub 搜索可见性**：填 description + topics + homepage；README 关键词对齐 `log viewer`/`large log file`/`JSONL viewer`/`klogg alternative` | **5 分钟** | **最高**（157k vs 15k 硬数据 + 本机实测证实缺失） | 无 |
| **2** | **去 klogg 273 个 open issues 下以「解决问题」的方式留痕** | 数小时 | 高（同一证据链） | 无 |
| **3** | **小众软件/Appinn 发现频道 + 主动私信推荐人**（HelloGitHub / LINUX DO） | 一天 | 中高（唯一 0→1800★ 一手复盘） | 本机可达 ✅；**闸：售价>¥100/年 可能需走付费商业推广，须先问管理员** |
| **4** | **V2EX `分享创造`**（须写开发故事/技术选型，拒绝硬广） | 半天 | 中（唯一一手证明「能带下载」：三周 400+） | **需代理** ⚠️ |
| **5** | **少数派 App+1 送码** | 半天（可复用 #4 的稿） | 中（送码形态已被接受；但流量在编辑推荐，不可控） | 本机可达 ✅ |
| 6 | HN 技术博文（**不是 Show HN**） | 1–2 月养号 | 中（HN 对纯技术内容友好，对桌面 GUI 冷淡） | **需代理** ⚠️ |

**4. 必须同时做的埋点设计**
- 所有内容平台外链一律走 **MS Store CID 直链**（95/5 分成 + 归因）
- 上架后先做**「零推广基线」**：一周内什么都不发，确认自然 page view 是否真为 0 → 若是，此后 page view 可 100% 归因于自己的推流
- 只用 Partner Center **Recent data（3 小时延迟，Last 24h/48h）**；**不建 API 看板**（已证实与 UI 不符）

**5. 一条提醒**：农场过去把「发布」当成「分发」。**发布是动作，分发是存在感。** 两份调研独立指向同一件事——**没人知道它存在，不是因为它不够好，是因为它没出现在任何人搜索的路径上。**

## 执行进度

**✅ 已完成 2026-09-16（用户批准后执行）**
1. **danqing-log 仓库元数据**（原来 `description: null` / `topics: []` / `homepage: null`）：
   - description 已写入：`Fast large-file log & JSONL viewer for Windows — 1 GiB indexed in 92 ms, regex search in 115 ms. 丹青日志 · Windows 大文件日志/JSONL 查看分析器 (Rust)`
   - **15 个 topics** 已写入（log-viewer / log-analysis / jsonl / jsonl-viewer / large-files / log-parser / tail / observability / logging / log-management / windows / rust / desktop-app / developer-tools / mmap）
   - homepage 按约定留空（上架后回填商店页）
   - 注：GitHub 的 **topic 索引有延迟**，`topic:log-viewer` 检索尚未收录，需隔期复核
2. **README 英文摘要块**（本地改动，**未提交**）：补了英文首段（含 `log viewer` / `JSONL viewer` / `large file` / `Rust` / `Windows` / `free & open source` 关键词）。改前 README 英文词频以 `error/jsonl/ctrl` 等配置词为主，**"viewer" 0 次 / "Rust" 0 次 / "large file" 0 次** —— 英文搜索者匹配不到。

**校准发现**
- **`topic:log-viewer` 不拥挤**：榜单靠前的仓库只有 **7–10★** → GitHub topic 检索是可触达的面
- **同名撞车**：存在 `zrnge/LogLens`（7★），需留意品牌辨识

**✅ 已完成 2026-09-16（pomodoro 渠道试点准备）**
3. **pomodoro 仓库元数据**（原 `topics: []` / `homepage: null`）：description 改写（并入英文关键词）+ **14 个 topics** + **homepage 指向商店页**（`https://apps.microsoft.com/detail/9P3W6W1SR6DS`，免费流量入口）
4. **渠道稿已写** → `danqing-pomodoro/docs/channel-appinn-post.md`：目标**小众软件发现频道**（meta.appinn.net/c/10）。已按站点**规则原文**适配（不复制粘贴商店文案 / 不需走付费商业推广 / 提供商店链接）；含**私信模板**（Catime 打法：主动联系 > 被动投稿）
5. **基线已钉死**（实验归因的前提）

## 试点实验：pomodoro 渠道验证（用户裁决 2026-09-16）

**目的**：农场**从未跑过任何一次渠道动作**。拿已上线的 pomodoro 先试，回答一个渠道级问题——**「时间序平台对零粉丝到底带不带量」**。答案对 log 同样成立，且**风险前置**（在 log 首发之前知道，而不是拿首发去撞）。

**🔒 基线快照（2026-09-16，发布前，防事后找理由）**

| 指标 | 基线值 |
|---|---|
| GitHub stars / forks / watchers | **0 / 0 / 0** |
| GitHub 14 天浏览 | **20 次 / 2 unique** |
| GitHub 14 天克隆 | 179 / 78 unique（判为爬虫噪音） |
| Release 下载量 | v0.2.8: **3** · v0.2.0: **1** · v0.1.1: **2** |
| 商店评分/评论 | **0** |
| 商店 PageViews / Installs | **待用户从 Partner Center 读**（Recent data，3 小时延迟） |

**判读规则（先写死）**：
- 7 天后任一指标**有可辨增量** → 渠道有效，log 发布复用并加投
- **全部为 0** → 时间序平台对零粉丝也不带量 → **农场必须换一条路**，而不是换个平台再试
- 归因保守：发现频道会把内容**自动推送**到 Twitter/Telegram/Fediverse，增量不纯来自本渠道

**⏸ 需要用户执行（我做不了的部分）**
1. 注册 meta.appinn.net 账号（**新用户审核通过前会被临时禁用**，第一次发帖有延迟）
2. 按 `channel-appinn-post.md` 粘贴发布（标题已定）
3. 私信 HelloGitHub / 小众软件编辑 / LINUX DO
4. 读完 Partner Center 三张表，补上「商店 PageViews / Installs」这一行基线

## 待办（按序）

1. 收 2 份调研 → 合成渠道判断 → 写入本文档
2. 用户提供 Partner Center 三个未知项
3. log 发布链执行（截图 → tag → Release → 商店）
4. 领先指标埋点方案落地（商店 impressions 复核节奏 / GitHub traffic 基线 / 下载量追踪）
5. 按调研结论执行首批渠道动作
