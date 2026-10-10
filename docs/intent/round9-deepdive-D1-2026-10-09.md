# D1 付费对位专项 · Drop Shelf 文件暂存架(2026-10-09)

> 上游:`deep-dive-round9-t1-2026-10-09.md` 卡A / D1 四小题。取证窗口 2026-10-09,时间盒 75 分钟(取证 ~60 + 写盘 15)。
> 纪律:付费行为单列(买断自白 > 捐赠订阅自述 > 标价不算);日期分布检打到付费证据上;被墙/不可达记 HTTP 码或失败方式,绝不编造。

## 结论先行

1. **Dropshelf 是「活着的、仍在收钱的」Windows 形态对位样本**,不是僵尸:MS Store 本体免费 + Pro 一次性 IAP(ESRB `InAppPur`),Issue 区买家自白**不止已知 5 笔,实测 10 笔**(2023-10 → 2026-07),其中 2 笔为「买了/立即买」显式掏钱原话,7 笔为 `Pro` 版本号自报,1 笔企业用户**想买被商店封死**;另有 3 笔购买流程崩溃/条件付费意向。付费分布**无衰减**,2026 年 5–7 月仍有新买家发声。
2. **Mac 对位从「转述级」升到「第一方数字级」**:Yoink 官网明码 `9,99 € / $8.99 / £8.99` 一次性(MAS+官网+Setapp 三渠道),Setapp 页 `99% · 1955 ratings`;Dropover 官网自报 `4.9/5 from 7.7K ratings on the Mac App Store`,且 **Dropover 的商业模式与 Dropshelf 同构到细节**(免费 + 14 天全功能试用 → 到期后 **3 秒等待** → Pro 一次性 IAP 解锁——和 Dropshelf 免费层的 3 秒 wait 一字不差是同一设计)。两者均在 2026-08/09 仍在发版。
3. **形态侧付费对位成立**:掏钱的 Windows 用户自白指向的就是「悬浮暂存架」本身(「paid for Pro immediately」/「recently purchased Dropshelf-Pro and am very pleased」),Mac 侧付费赞美原话亦是为同一形态(「Absolutely worth far more than the modest price!」)。
4. **「买了但缺 Y」差评级主矿挖到了**,缺件清单集中在:修饰键激活(不摇鼠标)/ Outlook 邮件附件与邮件上架 / 文件夹监视自动收件 / 永久架与分组 / 批量选择 / 粘贴图片进架 / 移动而非复制。其中多条在 Mac 对位品(Dropover Watched folders、Yoink email shelving)是**已有功能**,构成 Windows 侧「买了但缺 Y」硬缺口。
5. **流失信号真实存在但未见成规模退款/卸载潮**:数据丢失 bug(关架后文件变 0KB / 图片被自动删)×2、Outlook New 拖入坏(买家原话「That was because I bought Dropshelf」= 买了却核心场景坏)、购买流程崩溃 ×2。评价 122 条均 4.3 星 = 总体口碑仍好,缺陷是长尾差评不是崩盘。
6. **日期分布判:活水**(详见 §4)。2024-10 至 2026-07 付费相关证据持续出现,最长空窗约 5 个月(2025-09 → 2026-01 前后),2026 年 5/7 月仍有新买家;MS Store 近 30 天 3 条新评价(2026-10-09 拉取)。

---

## 1. Dropshelf 商业实况

### 1.1 MS Store 产品页(9MZPC6P14L7N)

抓取:`https://apps.microsoft.com/detail/9mzpc6p14l7n`(HTTP 200,229KB,2026-10-09)+ `https://displaycatalog.mp.microsoft.com/v7.0/products?bigIds=9MZPC6P14L7N&market=US`(HTTP 200)。

| 项 | 值 | 证据位置 |
|---|---|---|
| 产品 ID | `9MZPC6P14L7N`,包名 `371WilliamHa.Dropshelf`,开发者 William Ha | display catalog |
| 发布日 | 2023-05-31(`OriginalReleaseDate`) | display catalog `MarketProperties` |
| 商店条目最后更新 | **2026-09-22**(`LastUpdateDate` / `RevisionId`) | display catalog |
| 定价结构 | **本体免费** + **Pro 一次性应用内购买**(非订阅) | 产品描述原文 + ESRB `InteractiveElements: InAppPur` |
| JSON-LD `offers` | `{"@type":"Offer","price":0}`(仅本体免费,**不含 Pro IAP 价**) | 页面 JSON-LD |
| 星级/评价数 | **全时段 4.3 星 / 122 条**;近 30 天 3.7 / 3 条;近 7 天 3.0 / 2 条 | display catalog `UsageData`(US) |
| Pro 标价 | **未取到**(见取证局限 L1) | — |
| `HasAddOns` | true(确认存在商店 add-on,即 Pro) | display catalog `Properties` |

**免费层限制(wait timer)产品描述原文**:

> "Note: The free version of Dropshelf has a forced waiting time of 3 seconds before you can add more items to a shelf. Upgrading to Dropshelf Pro is a one time in-app purchase that removes this limitation and unlocks additional pro features."

**Pro 权益差异(产品描述点名的部分)**:

> "Give your shelf a name and custom color (**Dropshelf Pro feature**) — then stash it away and open it up later to pick up right from where you left off."

即官方明示 Pro 权益 = ①去掉 3 秒等待 ②架子命名 ③自定义颜色 ④「Save shelves for later」收放;描述里 "unlocks additional pro features" 含糊带过其余(界面内或有更多,页面未列)。

**评述**:122 条 4.3 星、近 30 天仍有 3 条新评——量级是「小众独立工具」不是「爆款」,但**不是零**。HTML 首屏 JSON 里 `ratingCount:0` 是 SSR 空载,真实计数在 `MarketProperties.UsageData`,以 catalog 为准。

### 1.2 GitHub issue 区全量买家取证

仓库:https://github.com/williamckha/dropshelf-repo(issue-only repo,README 自述 "Welcome to the issue-only repo for Dropshelf")。
抓取:`gh api repos/williamckha/dropshelf-repo/issues?state=all&sort=created&direction=asc&per_page=100 --paginate` → **79 个 issue(0 PR),54 条评论**,2023-06-05 → 2026-07-10。仓库 136★ / 59 open / `pushed 2026-09-20`(issue 仍在进)。

#### 1.2.1 付费行为全表(买断自白 / Pro 自报 / 付费受阻 / 意向)

| # | 日期 | issue | 用户 | 付费强度 | 原话/字段 | 缺件或诉求 |
|---|------|-------|------|----------|-----------|------------|
| 1 | 2023-10-08 | [#13](https://github.com/williamckha/dropshelf-repo/issues/13) | iikhsaan | Pro 版本自报 | `Dropshelf version: 1.0.4.0 Pro` | 「CANNOT BE USED」(故障) |
| 2 | 2024-02-20 | [#18](https://github.com/williamckha/dropshelf-repo/issues/18) | william200mph | Pro 版本自报 | `1.0.4.0 Pro` | 快捷键误触发(Shift+Z) |
| 3 | **2024-05-07** | [#21](https://github.com/williamckha/dropshelf-repo/issues/21) | ixieyi968 | **购买受阻**(流程崩溃) | "I failed to upgrade to pro… a new window flashed and then was forced termination" | 想付钱付不出 |
| 4 | **2024-06-06** | [#23](https://github.com/williamckha/dropshelf-repo/issues/23) | IcarianGod | **购买受阻**(菜单崩溃) | "Purchase menu seems to crash… Select Upgrade to Dropshelf Pro → Select onetime Purchase → Suffer" | 想付钱付不出 |
| 5 | **2024-06-07** | [#24](https://github.com/williamckha/dropshelf-repo/issues/24) | finnn1 | **买断自白(最强)** | "I was a Mac user and have been using **Dropover** for a long time… This app was exactly what I was looking for! I loved it so much, **I paid for Pro immediately after downloading it**." | 缺「修饰键激活架子」(Dropover 有) |
| 6 | **2025-04-29** | [#38](https://github.com/williamckha/dropshelf-repo/issues/38) | neoOpus | **买断自白** | "I recently purchased Dropshelf-Pro and am very pleased with it." | 7 条缺件(见 §1.2.2) |
| 7 | **2025-09-23** | [#43](https://github.com/williamckha/dropshelf-repo/issues/43) | deltaminimal | Pro 版本自报 | `1.0.7.0 Pro` | Files App 下 Move/Copy 失效;「Other than that great app!」 |
| 8 | **2026-01-12** | [#59](https://github.com/williamckha/dropshelf-repo/issues/59) | thientrile | Pro 版本自报 | `Version 1.0.9.0 pro` | 修饰键快捷键改不了 |
| 9 | **2026-03-30** | [#66](https://github.com/williamckha/dropshelf-repo/issues/66) | travibe | Pro 自称 | 标题 "Dark theme of **Dropshelf Pro**…" | 暗色主题全白不可用 |
| 10 | **2026-05-17** | [#72](https://github.com/williamckha/dropshelf-repo/issues/72) | Tibacon | Pro 版本自报 | `1.0.10.0 pro` | **图片自动丢失**(数据丢失) |
| 11 | **2026-05-29** | [#74](https://github.com/williamckha/dropshelf-repo/issues/74) | deltaminimal | Pro 版本自报 | `Version 1.0.10.0 Pro` | "Drop items here" 文字裁切(UI 瑕疵) |
| 12 | **2026-07-10** | [#77](https://github.com/williamckha/dropshelf-repo/issues/77) | manfredpascher-beep | **买断自白** | "That was because **I bought Dropshelf**." | Outlook New 附件拖不进(买了却核心场景坏) |
| — | 2023-07-07 | [#10](https://github.com/williamckha/dropshelf-repo/issues/10) | GGsource | 条件付费意向 | "If this was added **I will 100% pay for Dropshelf Pro**!" | PNG 透明背景保留 |
| — | 2026-02-25 | [#64](https://github.com/williamckha/dropshelf-repo/issues/64) | 74macdad | **付费受阻(渠道封死)** | "I'm a corporate user and my **IT dept blocks access to microsoft store**. how can i upgrade to the pro version?" | 想要侧载/代销授权 |

**与卡上已知 5 笔对照**:已知 = 2025-04-29 / 2025-09-23 / 2026-01-12 / 2026-05-17 / 2026-05-29。本卷**核验通过**并**新增 7 条**(2023-10-08、2024-02-20、2024-05-07 受阻、2024-06-06 受阻、2024-06-07 买断、2026-03-30、2026-07-10 买断、2026-02-25 受阻,后两条视口径计入或不计入「已知 5」)。**已知 5 笔全部在 issue 正文中复现,无虚报**;「1.0.10.0 pro」版本号自报(卡上注明)对应 #72/#74,核验一致。

#### 1.2.2 「买了但缺 Y」缺件清单(买家/用户点名)

按主题聚合(issue 原文):

| 缺件主题 | 提出者(日期) | 原话摘句 | Mac 对位品是否已有 |
|---|---|---|---|
| 修饰键激活架子(别让我摇鼠标) | finnn1 2024-06-07(**Pro 买家**) | "This feature saves me from having to wiggle the mouse… huge inconvenience" | Dropover 有 modifier 激活 |
| Outlook / 邮件附件上架 | pandrea-ui 2026-02-03 ×2(#61/#62)、manfredpascher-beep 2026-07-10(#77,买家) | "I'm a long time user of **Yoink**, the Mac OS app. Among other things, it supports the ability to **shelf emails**, in particular from outlook." | Yoink 有 email shelving |
| 文件夹监视自动收件 | wada3n 2024-11-24(#30) | 附 **Dropover 截图**做参照 | Dropover 有 Watched folders |
| 永久架 / 常驻架 / 分组 / 吸附工作区 | neoOpus 2025-04-29(#38,买家) | "Allow **permanent shelves**… Grouping shelves in a container… compatibility with Aero Snap / MS PowerToys" | Dropover 有 Pinned shelves |
| 批量选择架内多文件 | panda920 2023-07-05(#9) | "only supports working on a single file… not very convenient to deal with multiple files" | — |
| 粘贴图片/截图进架 | Blackfabric-io 2023-11-08/2023-11-28(#14/#16) | "save first the image into desktop, but thats not efficient" | Dropover 有 clipboard support |
| 移动而非复制 | Waradu 2023-06(#5)、deltaminimal 2025-09(#43,买家) | #43: Explorer 正常、Files App 反了 | Dropover 默认 move |
| 跨设备同步 | farmerboy66 2024-05(#22)、#38 | "my daily work happens on 3 different devices" | — |
| 自定义快捷键 | thientrile 2026-01(#59,买家) | 修饰键组合不识别 | — |
| 无商店购买/企业授权 | 74macdad 2026-02(#64) | IT 封 MS Store | Dropover 亦 MAS 独占(FAQ 自述) |
| 暗色主题可用性 | travibe 2026-03(#66) | 全白无字,「Light themed apps really hurt my eyes」 | — |
| 文件夹内导航/上跳 | neoOpus 2025-04(#38) | "move navigate up… without having to start over the dragging" | — |

**评述**:缺件清单不是泛泛功能请求堆,而是**跨 3 年、由已付费者多次点名、且多条在 Mac 对位品已有**——形态付费成立后,「买了但缺 Y」的 Y 也是硬的。

#### 1.2.3 流失 / 严重缺陷信号

| 日期 | issue | 信号 | 严重度 |
|---|---|---|---|
| 2023-06-28 | [#8](https://github.com/williamckha/dropshelf-repo/issues/8) | "literally unusable"(UAC 提权下拖入被拒,12 评论长贴) | 高(根因=WinUI 框架 issue) |
| 2024-08-31 | [#26](https://github.com/williamckha/dropshelf-repo/issues/26) | 关架重开后 jpg/gif 变 **0KB 空文件**——"effectively **lost my image files**" | 严重(数据丢失) |
| 2024-11-08 | [#29](https://github.com/williamckha/dropshelf-repo/issues/29) | "DropShelf not work" | 中 |
| 2025-11-30 | [#53](https://github.com/williamckha/dropshelf-repo/issues/53) | Disappearing Shelves | 中 |
| 2026-05-17 | [#72](https://github.com/williamckha/dropshelf-repo/issues/72) | **Pro 买家**:放入图片被自动删除,重开只剩最后一张 | 严重(数据丢失) |
| 2026-07-10 | [#77](https://github.com/williamckha/dropshelf-repo/issues/77) | **买家**:Outlook New 拖入永久转圈——"That was because I bought Dropshelf" | 高(核心场景坏,且是付费后) |

**未见**「退款 / 卸载 / 回退到别的工具」的成规模自白;#77 语气是「买了所以才遇到这个」而非「要退款」。整体 = **缺陷长尾 + 付费后差评**,不是变现崩盘。122 条 4.3 星与「不完美但值」的调性一致(#43 买家: "Other than that great app!")。

### 1.3 商业画像一段

Dropshelf 是 Windows 上**唯一查实仍在收钱的「拖拽暂存架」**:免费本体 + Pro 一次性 IAP(3 秒等待解锁 + 架子命名/配色/留存),上架 2023-05,商店条目 2026-09-22 仍有维护痕迹,issue 区 2026-07 仍有新买家。评价 122×4.3 星是小众健康盘;买家自白跨 2023-10→2026-07 共 10 笔(2 笔显式掏钱原话 + 7 笔 Pro 版本自报 + 1 笔买家缺陷贴),另有 3 笔付费意向/受阻。**强信号是「Mac Dropover 迁移者下载即买」**——形态侧付费对位一锤定音。弱信号是作者维护节奏偏慢(版本 1.0.2→1.0.10 跨 3 年,多个致命 bug 长期 open)与变现基建脆(购买流程崩溃 ×2、企业无法购 Pro)。

---

## 2. Mac 第一方付费取证

### 2.1 Yoink(Eternal Storms Software)

| 项 | 值 | 来源 |
|---|---|---|
| 官网标价 | **9,99 € / $8.99 / £8.99,一次性购买** | https://eternalstorms.at/yoink/mac/ 页内原句:"Free Trial Purchase on Website 9,99 € / $8.99 / £8.99 Yoink is a one-time purchase on the Mac App Store or on my website" |
| 渠道 | MAS + 官网 + **Setapp**;亦属 ESS Productivity Bundle | 同页:"There is no difference in functionality between the different versions (Mac App Store, Website, Setapp)" |
| Setapp 页 | **99% · 1955 ratings**;会员制 $14.99/月起(Setapp 整库) | https://setapp.com/apps/yoink |
| 版本/活跃 | v3.7.6,iTunes lookup `currentVersionReleaseDate 2026-08-03` | https://eternalstorms.at/yoink/mac/ + iTunes lookup API |
| 寿命 | 官网 blog 标题 "Happy 15th Birthday, Yoink for Mac"(2026) | https://eternalstorms.at/yoink/mac/ |
| 评论原话 | MAS 好评:"recommended." ⭐️⭐️⭐️⭐️⭐️ — j%nml (US);Setapp:"…can't use Setapp there. Highly recommended." — omer sadeh, **Aug 23, 2026** | 官网 + Setapp 页 |
| 销量/收入 | **未挖到任何第一方数字**(见 L4) | — |

### 2.2 Dropover(Tappable Software)

| 项 | 值 | 来源 |
|---|---|---|
| 商业模式 | **免费下载 + 14 天全功能试用**;到期后唯一变化 = **放下物品前 3 秒等待**;**Dropover Pro = 一次性应用内购买**(明文 "not a subscription")解除 | https://dropoverapp.com/faq 原文 |
| 评分(官网自报) | **Rated 4.9 out of 5 from 7.7K ratings on the Mac App Store** | https://dropoverapp.com/ 首页 |
| Pro 标价 | **未取到数字**(FAQ 只说 one-time IAP,未列价;MAS 页 SPA 未解析出 IAP 价目,见 L2) | — |
| 渠道 | **Mac App Store 独占**("Is Dropover available outside the Mac App Store? No.") | dropoverapp.com/faq |
| Setapp | **未上架**(https://setapp.com/apps/dropover → HTTP 404) | 本次实测 |
| 版本/活跃 | v5.3.1,iTunes lookup `currentVersionReleaseDate 2026-09-19`;官网 "What's New in 5.3.0" | iTunes lookup + 首页 |
| 付费赞美原话 | "**Absolutely worth far more than the modest price!** Incredibly useful, easy to use, and actually fun to use." — userAeroJ **08/31/2025**;"Wiggle it, just a …" — WriterJamie **09/04/2025**;另有 "Outstanding app" "Essential app for improving workflow" | dropoverapp.com/ 首页评论轮播 |
| 企业授权 | FAQ:"Business license seats are not available yet, but they are planned to become available **later this year**"(2026 口径) | dropoverapp.com/faq |
| 销量/收入 | 除「7.7K ratings」外**无收入数字** | — |

**与 Dropshelf 的形态同构(重要)**:Dropover 免费层惩罚 = 「3-second delay before you drop items onto the shelf」,Dropshelf 免费层惩罚 = 「forced waiting time of 3 seconds before you can add more items to a shelf」——**同一 freemium 手术**,双侧独立演化到同一刀法,可作为「该形态的付费点已被市场双向验证」的旁证。且 #24 是 Dropover 付费用户迁到 Windows 后**立刻买 Dropshelf Pro**。

### 2.3 「On Mac people happily pay」的转述降级判定

- **已落成第一方的**:Yoink 官网明码 $8.99 一次性(不是转述);Dropover 官网 7.7K 评分自报 + 官方 FAQ 付费结构(不是转述);两者 2026-08/09 均在发版(不是转述)。
- **已落成可引原话的**:Dropover 付费赞美 "worth far more than the modest price"(2025-08-31,官网转载评论,用户署名+日期);Yoink MAS/Setapp 好评(2026-08)。
- **仍为转述/未落成**:Yoink/Dropover 的**销量、收入、转化率**数字——媒体评测只言片语本次未挖到(Bing/DDG 被墙或空壳,iTunes lookup 无 sales 字段)。Bucket/Holdem README 的「On Mac, people happily pay」**仍停留在第二手转述级**,本卷用第一方**定价+评分规模+付费评论原话**替代了「销售数据」这一环,但不等价于收入证明。

---

## 3. 灰市温度计

**未取证**(时间盒内不可达/未排上)。闲鱼/淘宝需登录态与中文电商反爬,75 分钟盒内排在优先级末位;bing/duckduckgo 检索接口本次返回空壳或 HTTP 000。记入取证局限 L5,**不记「灰市不存在」**。

---

## 4. 日期分布表(付费行为,2024-10 起;此前压缩附录)

口径:Dropshelf Pro 买断自白 / Pro 版本自报 / 付费受阻(= 付费意图被阻断,单列)/ Mac 对位付费赞美。意向("I will pay")不计入付费行为,附录单列。

| 月份 | 付费事件 | 明细 |
|---|---|---|
| 2024-10 | 0 | — |
| 2024-11 | 0 | (缺件请求 #30 非付费) |
| 2024-12 | 0 | — |
| 2025-01 | 0 | — |
| 2025-02 | 0 | — |
| 2025-03 | 0 | — |
| 2025-04 | **1** | 04-29 Dropshelf Pro 买断自白 #38 |
| 2025-05 | 0 | — |
| 2025-06 | 0 | — |
| 2025-07 | 0 | — |
| 2025-08 | **1** | 08-31 Dropover 付费赞美原话(官网评论) |
| 2025-09 | **1** | 09-23 Pro 版本自报 #43 |
| 2025-10 | 0 | — |
| 2025-11 | 0 | — |
| 2025-12 | 0 | — |
| 2026-01 | **1** | 01-12 Pro 版本自报 #59 |
| 2026-02 | **1** | 02-25 付费受阻(企业被锁商店)#64 |
| 2026-03 | **1** | 03-30 Pro 自称 #66 |
| 2026-04 | 0 | — |
| 2026-05 | **2** | 05-17 Pro 自报 #72;05-29 Pro 自报 #74 |
| 2026-06 | 0 | — |
| 2026-07 | **1** | 07-10 买断自白 #77 |
| 2026-08 | **1** | 08-23 Setapp Yoink 订阅者好评(订阅自述,单列弱口径) |
| 2026-09 | 0 | (商店条目 09-22 仍有 revision) |
| 2026-10 | 0 | (至 10-09;MS Store 近 30 天 3 评=仍在用) |

**附录(2024-10 之前)**:2023-10 Pro 自报 #13;2024-02 Pro 自报 #18;2024-05 付费受阻 #21;2024-06 买断自白 #24 + 付费受阻 #23;2023-07 条件意向 #10。

**判定:活水。** 理由:①2024-10 至 2026-07 付费证据持续,2026 年内 6 个月有事件(1/2/3/5×2/7/8);②最长空窗 4–5 个月,未见「老票多新票无」的衰减形态;③2026-05 与 2026-07 仍是**新面孔买家**(#72 Tibacon、#77 manfredpascher-beep 皆非此前发声者);④MS Store UsageData 近 7/30 天仍有新评分。⚠️ 样本是 issue 区自报(幸存者偏差,总量级小,单月 0–2),不足以做收入曲线,只足以判「仍在有人为这个形态掏钱」。

---

## 5. 取证局限(照实记)

| # | 局限 | 失败方式 |
|---|---|---|
| L1 | **Dropshelf Pro 标价未取到** | 产品页 JSON-LD `offers.price=0` 只覆盖免费本体;display catalog `HasAddOns=true` 但 add-on product ID 不在本产品 JSON;`displaycatalog .../addons` HTTP 404;apps.microsoft.com API HTTP 403。Pro IAP 价仍是黑箱(用户侧可见「one time in-app purchase」) |
| L2 | **Dropover Pro 标价未取到** | FAQ 只写 "one-time in-app purchase" 不列价;apps.apple.com 页为 2MB SPA,iTunes lookup `price=0`(仅本体)且 `averageUserRating/userRatingCount=0`(该 API 已退化,不采信) |
| L3 | **apps.apple.com / itunes.apple.com / raw.githubusercontent.com / dropshelf.app / duckduckgo.com** | Git Bash curl 对 apple 域 **segfault**(进程崩溃非 HTTP 码);dropshelf.app HTTP 000;DDG HTTP 000;Bing HTTP 200 但为 JS 壳,正文不可读。iTunes lookup 靠 PowerShell `Invoke-RestMethod` 绕过成功 |
| L4 | **Yoink/Dropover 销量、收入、下载量** | 全渠道未出现第一方数字;「7.7K ratings」是评分规模不是销量。媒体评测检索(Bing/DDG)不可用 → 本轮无法把「people happily pay」升级到收入级 |
| L5 | **灰市(闲鱼/淘宝)未取证** | 时间盒末位未执行;中文电商需登录态。不判「无灰市」 |
| L6 | **MS Store 用户评论正文** | 产品页未带评论列表(SSR 空载);仅拿到 UsageData 聚合(122 条 4.3 星)。评论正文里的差评/退款关键词无法逐条核 |
| L7 | **issue 区样本偏差** | 会去 GitHub 提 issue 的是硬核用户/遇障者;付费后沉默满意者不可见。10 笔自白是**下界**不是全量买家数 |
| L8 | **WebSearch 本会话返回空壳**(两查均无结果体),全程改用 curl/gh/PowerShell 直取 |

---

## 6. 对 D1 四小题的直接回答

1. **Dropshelf 商业实况**:免费+Pro 一次性 IAP,Pro 标价未取到(L1);122 评/4.3 星,近 30 天 3 评;Pro 权益=去 3 秒等待+架命名/配色/留存;买家自白实测 **10 笔**(已知 5 全核验+新增),缺件清单 12 主题,流失信号=数据丢失×2+核心场景坏×1+购买流程坏×2,**无退款/卸载潮**。
2. **Mac 第一方付费**:Yoink $8.99 一次性(官网明码)+ Setapp 1955 评 99%;Dropover 官方 4.9/7.7K 评 + 与 Dropshelf 同构 freemium(3 秒等待刀法一致)+ 付费赞美原话 2025-08;二者 2026-08/09 仍发版;**销量收入仍缺**(L4),「happily pay」已部分落地为定价+评分规模+付费评论,**收入级仍为转述**。
3. **灰市**:未取证(L5)。
4. **日期分布**:**活水**(§4 表);2026 年 5/7 月仍有新买家。

## 7. 证据索引(全部 URL)

- MS Store 产品页:https://apps.microsoft.com/detail/9mzpc6p14l7n
- display catalog(评分/UsageData/HasAddOns):https://displaycatalog.mp.microsoft.com/v7.0/products?bigIds=9MZPC6P14L7N&market=US&languages=en-us
- Dropshelf issue 仓:https://github.com/williamckha/dropshelf-repo(issues 全量经 gh api,79 条 2023-06-05→2026-07-10)
- 买断自白 #24:https://github.com/williamckha/dropshelf-repo/issues/24(2024-06-07)
- 买断自白 #38:https://github.com/williamckha/dropshelf-repo/issues/38(2025-04-29)
- 买断自白 #77:https://github.com/williamckha/dropshelf-repo/issues/77(2026-07-10)
- 付费受阻 #64:https://github.com/williamckha/dropshelf-repo/issues/64(2026-02-25)
- Yoink 定价:https://eternalstorms.at/yoink/mac/
- Yoink Setapp:https://setapp.com/apps/yoink
- Dropover 首页(7.7K 评+付费赞美):https://dropoverapp.com/
- Dropover FAQ(freemium 同构):https://dropoverapp.com/faq
- iTunes lookup(Dropover id=1355679052 / Yoink id=457622435,经 PowerShell)
