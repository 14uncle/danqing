# L6 限时补扫 · Dropshelf 商店评论正文 + Mac 差评 + Pro 标价(2026-10-09)

> 上游:`round9-deepdive-redteam-2026-10-09.md` A3 处方 / R1' 欠账 d 项。补环 agent,60 分钟硬时间盒(取证 ~40 + 写盘 ~15)。
> 纪律:每条证据带评分+日期+原文+来源 URL;转述级与第一方分开;端点失败记 HTTP 码;拉不到就记死。

## 结论先行

1. **L6 还清(第一方级)**。从 Microsoft Store 前端 SPA 的 `telemetry-service-BHr57qHt.js` 反编译出官方评论 API:
   - `GET /api/products/getReviews/{productId}?orderBy={n}&pgNo={p}&noItems={take}&hl=en-us&gl={market}`
   - `GET /api/Products/GetReviewsSummary/{productId}?catalogSource=SparkPartnerCenter&hl=en-us&gl={market}`
   - orderBy 枚举:MostRecent=1 / HighestRated=3 / **LowestRated=4** / MostHelpful=5(取自 `J=(e=>(e[e.MostRecent=1]=...` 枚举定义)
   - **必须带 `hl` + `gl` 参数**,缺了返 403。
2. **1-2★ 带正文差评实收 7 条**(US 3 / DE 2 / BR 2),另有 3★「买了但缺 Y」×2、4★×1、5★缺件×2。**头号主题不是 Outlook,是「根本打不开」×3 + 「买了权益丢」×1 + 「IAP 不标价」×1**。
3. **「买了但缺 Y」在商店差评里的最强原话**:Dan 1★(2025-12-11)「**I PAID for the Pro version yet ... it was the Free version and had the button for Upgrade to Pro AGAIN! Not paying twice!**」= 付费后重装权益丢失(licensing 缺陷),极性是愤怒不是退款。
4. **退款潮:无**(从「不可证伪」升级为「已扫可证伪通道,结果 0 命中」)——25 条带正文评论(6 市场)无一提 refund/money back/退款。残余不可检:122 条评分中 ~97 条无正文、真退款不留公开评论。
5. **Mac 差评:Dropover 还清 6 条 1-2★**(含 1 条「买了但缺 Y」原话:Jeremiah Bruh 2★「I bought it fairly exclusively to be able to use Dropover cloud links」);**Yoink MAS 评论正文记死**——iTunes RSS 空壳(US/GB/DE/AU/CA/FR 全空,JSON/XML/无 sort 均空),MZStore 旧端点只回 JS 壳,amp-api 401。
6. **Dropshelf Pro 标价:仍无数字(L1 记死保持)**,但升级为「定性确认存在且被嫌便宜/合理」:Vijay 2★「price is affordable and fair」+ Jonathan 5★(DE, 2023-06-08)「**pro price is fine**」。搜索/add-on 端点均不暴露 IAP 价目。

---

## 1. 端点攻坚记录(为什么 D1 红队全 403/400,本轮破墙)

| 步骤 | 动作 | 结果 |
|---|---|---|
| 1 | display catalog `/products/{id}/reviews` 等 | 404(该服务无评论资源) |
| 2 | storeedgefd / marketplaceedgeservice 旧评论端点 | 404/400/域名解析失败 |
| 3 | apps.microsoft.com `/api/products/{id}/reviews` 等直猜 | 403/400 |
| 4 | **抓商店页 HTML → 抽 `index-CqfCVmNg.js` → app-router 动态 import 清单** | 找到 `product-details-Hz01D3Tt.js` |
| 5 | product-details 里 `importRatingsAndReviews()` | 找到 `ratings-and-reviews-eCXkSXQi.js` |
| 6 | 该文件只是壳,`x.getReviews` 来自 `telemetry-service-BHr57qHt.js` 的 `a` 导出 | **telemetry-service 同时是 API client** |
| 7 | 抠出 `getReviews(t,s,i,n){const a=`/api/products/getReviews/${t}`,o={orderBy:s,pgNo:i,noItems:n}` + orderBy 枚举 | 调用成功,HTTP 200 |

**可复用手筋**(以后 MS Store 评论一律走这):带 UA + `hl`/`gl`,`orderBy=4` 直取差评;`getReviewsSummary` 拿星数直方图与 mostCriticalReview。

---

## 2. Dropshelf 1-2★ 差评正文全表(第一方,官方 API)

来源基 URL:`https://apps.microsoft.com/api/products/getReviews/9MZPC6P14L7N?orderBy=4&pgNo=1&noItems=20&hl={locale}&gl={market}`
单条深链样例:`https://ratingsedge.rnr.microsoft.com/v1.0/ratingsedge/content/product/9MZPC6P14L7N/review/{reviewId}`

### 2.1 一星 / 二星(任务最高优先)

| # | 评分 | 日期(UTC) | 市场 | 作者 | 标题 | 原文(原语言) | 主题标签 | reviewId |
|---|------|-----------|------|------|------|--------------|----------|----------|
| 1 | **1★** | **2025-12-11** | US | Dan | Already Paid Once | "I PAID for the Pro version yet when I tried to download the Dropshelf App again it was the Free version and had the button for Upgrade to Pro AGAIN! Not paying twice!" | **买了但缺 Y / 权益丢失 / 购买流程** | `2667e56d-a5ad-aec6-3ed2-25a4ff26bb97` |
| 2 | **1★** | **2024-12-31** | US | William | Saving .bmp images instead of .png or .jpg | "Can't save images properly because it saves them as .bmp, when it should be .png or .jpg, happens in every site of web using Firefox based browser. Also as they are .bmp can't open it later as it looks like moved another directory and can't copy it again." | **数据/文件损坏**(打不开=变相丢文件) | `28e776b0-4631-564a-a53e-28e388fa4039` |
| 3 | **2★** | **2025-04-07** | US | Vijay | Doesn't disclose trial version and actual price | "Even if it's well done and price is affordable and fair, the practice is deceitful, with no indication of in-app purchase on the app-store button." | **购买流程 / IAP 不披露**(间接确认 Pro 有价且「affordable」) | `61a5fb24-357f-dc88-1826-4d34df13bfac` |
| 4 | **1★** | **2026-10-03** | DE | k | öffnet nicht | "Nach Installation - reproduzierbar - keine Funktion. \"Öffnen\" steht da, aber nichts öffnet sich. Beim Schütteln eines Bildes öffnet sich Google Lense, \"bitte hier ablegen\". Außerhalb des Browsers öffnet sich beim Schütteln eines Bildes - nichts; habe zweimal installiert, dazwischen deinstalliert und rebootet. Aber es tut sich nichts. jedenfalls nichts mit dieser App. deinsalliere nun final." | **根本打不开**(6 天前新差评,已卸载) | (DE gl) |
| 5 | **1★** | 2025-01-24 | DE | Z | Not working | "Doesn't work with touchscreens. Write that in the description so that people don't waste time." | **平台不支持**(触屏) | (DE gl) |
| 6 | **1★** | 2025-09-19 | BR | Rafael | Does not work | "After you finish the introduction, you can never open the app again. The shelf does not show up, and any time you open the app, nothing happens. I with it gets fixed, as the concept seems pretty good..." | **根本打不开**(概念好但打不开) | (BR gl) |
| 7 | **2★** | 2025-10-10 | BR | Liandro | precisa de mais idiomas | "poderia ter o idioma português, valeria a pena compara." | **缺多语言** | (BR gl) |

### 2.2 3★ 及以上的「买了但缺 Y」/ 缺件(同卷顺带,直供 Y 清单)

| 评分 | 日期 | 市场 | 作者 | 标题 | 原文要点 | 缺件主题 |
|------|------|------|------|------|----------|----------|
| 3★ | 2023-06-08 | US | Alex | Great but not for gaming | "I wish there was some sort of button I can just hold and shake my mouse if I do want to access this app! **if this does get added ill change the review to 5 stars**" —— **开发者 William Ha 当日回复**(游戏模式设置 + Win+Shift+Z) | **修饰键激活**(与 issue #24 Pro 买家同款) |
| 3★ | 2025-07-26 | US | nham manh | lag | "pretty lag on first run on every startup (update: i mean everytime launch not only on startup)" | 启动卡顿 |
| 3★ | 2026-04-30 | IN | Ankur | moderate | "i changed the settings for how much i need to shake the file, it works at that time, but the next time it is the same" | **设置不持久** |
| 3★ | 2026-07-13 | BR | Namgzraidpjia | Não está querendo funcionar. | "o app não está querendo funcionar" | 又一例「不工作」 |
| 4★ | 2026-09-01 | US | Hamse | **Best Alternative of DropOver** | "we need the share Options to be added at the bottom right **like the Dropover lightning**" | **对标 Dropover 缺 share 选项** |
| 5★ | 2026-10-07 | US | Sidik | Drop Tray (Windows Share Sheet) compatibility | "there's still no dropshelf option in the Windows share sheet (Drop Tray)" | **系统分享面板集成** |
| 5★ | 2023-06-08 | DE | Jonathan | dang nice idea | "works well, **pro price is fine**" | **Pro 价格定性证词**(无数字) |
| 5★ | 2023-10-22 | IN | Krishnendu | **Yoink for Windows** | "Yoink for Windows is here..." | 用户自发 Yoink 对标 |
| 4★ | 2026-02-02 | CN | 龙 | 挺好的工具 | "工具不错,增加多语言支持就好了" | 缺多语言 |
| 5★ | 2025-11-16 | CN | 有有 | 非常好! | "用的很舒服,不过怎么没什么人用?**我从B站来的**。《Windows 效率慢?加个文件暂存栏!你与高手的差距只差这个「拖拽神器」DropShelf》" | **渠道信号:B 站中文视频引流已发生**(标题级转述) |

### 2.3 市场全量盘点(带正文评论)

| 市场 | 条数 | 1-2★ | 星分布 | 备注 |
|------|------|-------|--------|------|
| US | 10 | 3 | 1,1,2,3,3,4,5,5,5,5 | GetReviewsSummary: avg **4.2**, reviewCount 10, star1=2/star2=1 |
| DE | 4 | 2 | 1,1,5,5 | 含 2026-10-03 新差评 |
| BR | 4 | 2 | 1,2,3,5 | |
| CN | 3 | 0 | 4,5,5 | 中文多语言诉求 |
| IN | 3 | 0 | 3,4,5 | |
| CA | 1 | 0 | 5 | |
| GB/FR/JP/AU | 0 | — | — | 空 |
| **合计** | **25** | **7** | | |

**与 UsageData 122 的口径差(重要)**:display catalog `UsageData.AllTime` 是 **122 条评分 / 4.3★**(含大量无正文评分,可能跨市场);Web 评论 API `totalCount=10`(US)+ 他市场共 **25 条带正文**。两者不矛盾——**评分 122、带字评论 25**。红队 A3 说「1-2★ 必然存在」已证实:US 直方图 star1ReviewCount=2 / star2ReviewCount=1,全市场带正文 1-2★ 共 7 条。

### 2.4 主题归纳(对照任务指定四主题)

| 任务指定主题 | 商店评论命中? | 证据 |
|---|---|---|
| 买了但缺 Y | **命中(强)** | Dan 1★ 权益丢失「Not paying twice!」;Alex 3★ 缺修饰键(官方回复);Hamse 4★ 缺 Dropover 式 share |
| 数据丢失 | **命中(弱-中)** | William 1★ .bmp 后「can't open it later」= 保存损坏;无「丢整个架」级(那在 issue #26/#72) |
| Outlook | **商店评论未命中**(0 条) | Outlook 主题只在 GitHub issue #61/#62/#77;商店差评人群不提邮件 |
| 购买流程 | **命中(中)** | Dan 1★ 重装丢 Pro;Vijay 2★「deceitful, no indication of in-app purchase」;另有 issue #21/#23 购买菜单崩溃、#64 企业买不了 |

**新头号主题(评论侧独有)**:**「装完打不开 / 用不了」×4**(k DE 2026-10-03、Z DE 触屏、Rafael BR、Namgzraidpjia BR)——这是 GitHub issue 区样本看不到的「首启失败」人群,和 issue #8 「literally unusable」(UAC) 同族。

---

## 3. Mac 对位差评

### 3.1 Dropover(id 1355679052)——还清

来源:`https://itunes.apple.com/us/rss/customerreviews/id=1355679052/sortby=mostrecent/json`(iTunes 公开 RSS,第一方)

| 评分 | 日期 | 作者 | 标题 | 原文 | 主题 |
|------|------|------|------|------|------|
| **1★** | 2026-09-18 | soundneedle | Nothing | "Installed and nothing shows or happens. No welcome screen. No application icon in the menu bar." | 装完无反应 |
| **1★** | 2026-08-29 | Sam Altwomen | BUGS | "Buggy on macOS 27" | 新系统兼容 |
| **1★** | 2026-07-29 | vczhfj | Non existing support | "Stopped working after Golden Gate Beta. Sent them a note, dead silence since. Sending a note to Apple too." | 停止工作+支持沉默 |
| **2★** | **2026-04-12** | Jeremiah Bruh | Do not buy this if you want Dropover Cloud. | "I have been using this app for quite a while, and despite several attempts to bug report, Dropover Cloud almost NEVER functions correctly. The app is/can be great, but **I bought it fairly exclusively do be able to use Dropover cloud links** for sending files to clients. It cannot reliably upload without randomly erroring out with \"Failed to parse response from server\"..." | **买了但缺 Y(最强)**:为 Cloud 掏钱而 Cloud 不可用 |
| **1★** | 2026-01-17 | Silva Hawk | It keeps dying even on the latest version | "every once in awhile, the app just dies and I have to reopen it for it to work again." | 频繁崩溃 |
| **1★** | 2025-10-16 | DatBabyAintMine | Unresolved bug fixes. | "When two or more files are moved from the shelf to their desired location, they appear as a single file... nearly impossible to separate them" | 多文件出架粘连(功能缺陷) |

附:abzhe 2025-10-29 标题「Crashes often」但打了 5★(评分与正文极性分裂,单列不计入差评);Bob@Live 4★ 2026-03-30 缺「架内重命名」。

**Dropover 差评与 Dropshelf 同构点**:两侧都有「装完打不开/崩溃」与「买了但核心场景坏」;Dropover Cloud 付费点坏 = Dropshelf Pro 权益丢失,同属「付费后缺件」族。

### 3.2 Yoink(id 457622435)——记死

| 尝试 | 结果 |
|------|------|
| `itunes.apple.com/{us,gb,de,au,ca,fr}/rss/customerreviews/id=457622435/sortby=mostrecent/json` | HTTP 200 但 **entry 空壳**(仅 feed 元数据,0 条评论) |
| 同上 `/xml`、无 sort、`?cc=us` 变体 | 均空壳 |
| `WebObjects/MZStore.woa/wa/customerReviews?id=457622435` | HTTP 200 但只回「Connecting to the iTunes Store.」JS 壳,无正文 |
| `amp-api.apps.apple.com/v1/catalog/us/apps/457622435/reviews` | **HTTP 401** |
| `apps.apple.com/us/app/yoink/id457622435` | 1.9MB SPA,无 `user-review`/`im:rating`/`customerReviews` 埋点 |
| iTunes lookup | trackName 确认 `Yoink - Better Drag and Drop` price=8.99,但 `averageUserRating=0/userRatingCount=0`(该 API 已退化,不采信) |

**记死结论:Yoink MAS 差评正文本轮不可得**。已有替代证据(D1 已核):官网明码 $8.99 一次性 + Setapp 1955 评 99% + 官网转载好评。RSS 空壳可能是 Apple 后端对部分 app 关闭了旧评论 RSS,非我们参数问题。

---

## 4. Dropshelf Pro 标价(L1)

| 尝试 | 结果 |
|------|------|
| display catalog `DisplaySkuAvailabilities` | 仅本体 Sku 0010 full / 0011 trial,ListPrice **0.0 USD**(免费本体);无 add-on SKU |
| `HasAddOns` | **true**(确认 Pro IAP 存在) |
| `/api/pages/pdp?productId=9MZPC6P14L7N` | `price:0, displayPrice:"Free"`,描述含 "one time in-app purchase",**无价目** |
| `/api/products/search?query=Dropshelf%20Pro` 等 3 组查询 | 只返回本体 + 无关云盘应用;**无名为 Dropshelf Pro 的独立产品** |
| `/api/Products/SearchByPublisherName?publisherName=William%20Ha` | 仅本体一个产品 |
| JSON-LD `offers` | `price:0`(D1 已记) |

**定性证词(第一方评论,非数字)**:
- Vijay 2★ 2025-04-07(US):"price is **affordable and fair**" + 抱怨商店按钮不披露 IAP
- Jonathan 5★ 2023-06-08(DE):"works well, **pro price is fine**"

**结论:L1 记死保持——具体数字拉不到;但「Pro 有价、价位被用户评价为合理/公道」已从评论侧坐实**。粗算单价假设可锚定「用户口中的 affordable」,数量级仍需 Yoink $8.99 / pomodoro ¥18 邻价带。

---

## 5. 「退款潮」判定更新(红队 A3 点名)

| 口径 | 更新前 | 更新后 |
|---|---|---|
| 有无退款潮 | **不可证伪**(L6 未解) | **无**(25 条带正文评论 0 条提退款/money back/退款) |
| 可证伪性 | 评论正文不可见,无法检 | 官方评论 API 可检,已检 6 市场全量带正文评论 |
| 残余不可检 | — | ①122 条评分中 ~97 条无正文;②真退款/卸载不留公开评论;③issue 区 #77 是「买了才遇到」语气非「要退款」 |
| 相邻负面信号 | issue 数据丢失×2、购买流程崩×2 | + 商店评论「装完打不开」×4、「重装丢 Pro」×1、「IAP 不披露」×1、「保存损坏」×1 |

**一句更新:退款潮=无(评论通道已证伪),但付费后不满真实存在且主题是「权益/打不开/购买披露」,不是「不值这个钱」。**

---

## 6. 端点失败清单(照实记)

| 端点 | 失败方式 |
|---|---|
| `displaycatalog.mp.microsoft.com/v7.0/products/{id}/reviews` | HTTP 404 |
| `displaycatalog.../ratings/{id}` | HTTP 404 |
| `displaycatalog.../products/{id}/ratingsandreviews` 等 3 变体 | HTTP 400 |
| `displaycatalog...fieldsTemplate=Full` | HTTP 403 |
| `displaycatalog.apps.microsoft.com` 主机 | DNS 解析失败 |
| `storeedgefd.dsx.mp.microsoft.com/v9.0/reviews?productId=` | HTTP 404 |
| `storeedgefd.../products/{id}/reviews`、`/pages/pdp/` | HTTP 400 |
| `storeedgefd.../productFamilies/...` | HTTP 404 |
| `marketplaceedgeservice.windows.com` | DNS 解析失败 |
| `apps.microsoft.com/api/products/{id}/reviews`(直猜旧路径) | HTTP 403 |
| `apps.microsoft.com/api/ratingsHistogram`、`/api/products/{id}/reviews?skipItems=` | HTTP 403 |
| `apps.microsoft.com/api/reviews?productId=` 等 4 变体 | HTTP 400 |
| `apps.microsoft.com/store/detail/{id}/reviews` | 连接被重置 |
| **缺 hl/gl 的 getReviews/GetReviewsSummary** | **HTTP 403**(参数是钥匙) |
| web.archive.org / cdx / wayback available | 连接超时(网络层不可达,非 404) |
| alternativeto.net | HTTP 403 |
| softonic.com/app/dropshelf | HTTP 412 |
| windowsreport.com/dropshelf | HTTP 403 |
| filehorse.com | HTTP 000 |
| majorgeeks.com/files/details/dropshelf.html | HTTP 200 但**软 404**(canonical 在,正文是 MajorGeeks 站首页,无 Dropshelf 内容无评论) |
| WebSearch(Dropshelf 1-star 等 1 查) | 空壳无结果体(同 D1 L8) |
| iTunes RSS Yoink ×6 国 + XML + 无 sort | HTTP 200 空壳 |
| amp-api.apps.apple.com Yoink reviews | HTTP 401 |
| iTunes lookup `averageUserRating` | 字段退化为 0,不采信 |

---

## 7. 对卡面/红队的直接回答

1. **L6**:还清(第一方)。7 条 1-2★ 正文 + 10 条 3★+ 缺件正文到手,API 可复现。
2. **退款潮**:无(已可证伪通道检完为 0);付费后不满有,主题=权益丢失/打不开/IAP 披露,非「不值」。
3. **差评正文要点**:见 §2.1 七条逐条(评分+日期+原文);头号=「装完打不开」×4 与「重装丢 Pro」×1;Outlook 在商店评论缺席(只在 issue);数据损坏弱命中(.bmp)。
4. **Pro 标价**:无数字(L1 保持记死);有定性「affordable and fair」「pro price is fine」两条第一方证词。
5. **Mac 侧**:Dropover 6 条差评到手(含 2★「I bought it fairly exclusively…」买了但缺 Y);Yoink 评论正文记死。

## 8. 证据索引

- 评论 API(US 差评):`https://apps.microsoft.com/api/products/getReviews/9MZPC6P14L7N?orderBy=4&pgNo=1&noItems=20&hl=en-us&gl=US`
- 评论 API(DE/BR/CN/IN/CA):同 URL 换 `gl=`
- 摘要 API:`https://apps.microsoft.com/api/Products/GetReviewsSummary/9MZPC6P14L7N?catalogSource=SparkPartnerCenter&hl=en-us&gl=US`
- PDP API:`https://apps.microsoft.com/api/pages/pdp?productId=9MZPC6P14L7N&hl=en-us&gl=US`
- API client 反编译源:`https://apps.microsoft.com/assets/js/telemetry-service-BHr57qHt.js`(getReviews/getReviewsSummary 定义处)
- SPA 链路:`https://apps.microsoft.com/assets/js/index-CqfCVmNg.js` → `app-router-BsxZkAVO.js` → `product-details-Hz01D3Tt.js` → `ratings-and-reviews-eCXkSXQi.js`
- 评论深链样例:`https://ratingsedge.rnr.microsoft.com/v1.0/ratingsedge/content/product/9MZPC6P14L7N/review/2667e56d-a5ad-aec6-3ed2-25a4ff26bb97`
- Dropover RSS:`https://itunes.apple.com/us/rss/customerreviews/id=1355679052/sortby=mostrecent/json`
- display catalog(122 评分口径):`https://displaycatalog.mp.microsoft.com/v7.0/products?bigIds=9MZPC6P14L7N&market=US&languages=en-us`

---

## 9. 取证局限(照实)

| # | 局限 | 说明 |
|---|---|---|
| 1 | 122 条评分 vs 25 条正文 | 无正文评分里的 1-2★ 不可读(数量上 star1+star2 应 >7);幸存者偏差只部分修复 |
| 2 | 评论 API 星数直方图仅 US summary 可用 | DE/BR 等 summary 返回 0(疑似 catalogSource/市场未对齐),差评靠 getReviews 全量扫 |
| 3 | Yoink 正文记死 | RSS 空壳,原因不明 |
| 4 | Pro 标价数字 | IAP 价目不进公开 catalog/search API |
| 5 | 灰市/退款平台 | 本轮未排上(与 L5 同,时间盒优先评论正文) |
| 6 | B 站视频标题 | 来自 CN 5★ 评论引述,属**转述级**渠道线索,未打开原视频核实播放量 |
