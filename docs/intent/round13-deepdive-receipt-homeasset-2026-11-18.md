# round13 深挖卷 · 收据/家财/保修记账自建潮（2026-11-18）

> 源卡：`round13-selection-2026-11-18.md` R2-卡2（初判活，用户裁「1开挖」）。三闸要求：裁决段 + 红队 + 一页粗算，齐后回用户终裁。
> 卖点假设：**本地/离线 + 买断制（非订阅）+ 数据主权（可导出）+ 好用的 OCR 录入** 的个人/家庭收据+保修+家财工具。

## 状态

- [x] B 路：需求验证 + 付费态度（2026-11-18 已回）
- [x] A 路：竞品 + issues + 定价（**压缩版**——agent 被环境掐死，主会话补跑开源侧；闭源 issues/买断流水先例【待补】）
- [x] C 路：吸收距离 + 形态 + 定价 + 粗算原料（2026-11-18 已回）
- [x] 推理链书面（含断环诚实标注）
- [x] 失效标志前置
- [x] 一页粗算
- [x] 红队（doubt-driven，见红队记录）
- [x] **桌面侧定向扫描（用户 2026-11-18 选 B 授权，已交卷）**：合格独立桌面证据 **0/3**；只收近两年的 Windows 桌面凑合/付费行为，未触发窄形态翻案重评（见 D 路）
- [x] **裁决段终稿 → 用户终裁（2026-11-18 用户原话「接受判死」）**：产品轨判死，埋葬条款生效，仅用户本人可复活；不进 spec、不启动 POC

## B 路 · 需求验证 + 付费态度（2026-11-18 交卷）

**通道**：HN Algolia ✅ / iTunes Search+customerreviews RSS ✅ / WebSearch 死 / WebFetch itunes 域被拦 / Google Play 未成 / **原卡引证两条未回证**（2026-02-09、2026-08-20 线程多轮检索未命中——间接证据部分替代，记入证据瑕疵）。

### 凑合行为证据（行为级）
| 日期 | 原话（摘） | 来源 |
|---|---|---|
| 2026-04-13 | ReceiptBin 作者：「lock basic features like OCR and autofill behind paywalls」→ 自建免费 OCR | HN 47751091 |
| 2026-06-09 | HomeLogger 作者：「couldn't find a home maintenance tracker I like, so I made my own」（Lubelogger 凑合管车+自建管房） | HN 48463090 |
| 2026-08~09 | Stowly 三连自述：家财+保修+维护+收据+「灾后一键导出保险索赔清单」 | HN 49212442/49299223/49657976 |
| 2026-11-07 | Receipt Inbox 作者：「tired of keeping track of receipts」（$6/月订阅，**反证买断假设**） | HN 50405605 |
| 近期 | Saldo 评论用户：「take photos on my phone… use the OCR already baked into my iPhone… organize my receipts myself」= **相册+系统 OCR+手工整理的凑合现状自述** | App Store 1550270774 |
| 近期 | 「just want a dedicated place to store pictures of receipts, that's not worth $10/mo」= 需求形状（存图+找得到，报表伪需求） | 同上 |
| 近期 | Under My Roof 用户：野火 cabin 烧毁，**保险公司要求逐件家财清单才赔付**，凭记忆+零散照片补清单 | App Store 1524335878 |
| 近期 | Home Contents 用户：因「not stored on vendors website… export to your personal iCloud」选型 = 数据主权 | App Store 420151922 |
| 2026-02-28 | Surebeans：YNAB4 弃养+涨价激怒→自建「one-time payment, local plaintext, data sovereignty」（邻域同构） | HN 47196050 |
| 2025-08~2026-03 | 同域 Show HN ≥8 个（Receiptly/Warrify/Good Til/ClaimVault/ExpenseSpy…）**但几乎全部零回帖——供给热≠需求铁证** | HN 多条 |

### 付费态度（单列）
**订阅抱怨（铁证级量大）**：Sortly「$468→$1428/年，涨 300%+，cancel 找替代」(2025-09-08) /「Priced out as a home user!」(2021) /「pay something upfront… highway robbery」(2023) / Smart Receipts「Subscription fee city」/ Saldo「没订阅就**删你照片**」「hold your data hostage… Predatory」（数据人质）。
**买断偏好（≥6 条表态）**：Under My Roof「pay $5-$10 as a one-time purchase, recurring indefinitely is absurd」/「Needs lifetime purchase option. Please.」/ Itemtopia「one time purchase but these subscription apps are ridiculous… let me know」/ Saldo「Rather pay for a good app than in app purchases」。
**已付费实证**：Home Contents **$5.99 买断、4.7 星/1465 评**——好评大面积点名买断（「one time fee… as someone on the low income side」「subscription-free… doesn't siphon your data… no hesitation」「Worth the 5.99」）。
**反向锚点**：Nest Egg「$5.99 for the full version is ridiculous. Bring the price down」。
**OCR 双刃**：需求存在（锁 OCR 付费墙被骂）；兑现风险大（「10 张扫描 ZERO correct」「Not one accurate scan」vs SimplyWise 5 星「一张图它全干了」）。

### 用户画像分裂判断
- 商家侧（报销/库存）：刚需但拥挤（Expensify/QuickBooks），付费力高、订阅容忍高（骂涨价不骂订阅）。
- **个人侧（报税/保险/保修/家财）：凑合更硬**——被商业工具按件数定价挤出（Sortly 100 项免费墙）、灾难/保修两个触发场景行为级扎实、买断偏好全部出自这侧、$5-20 愿付锚点在这侧。
- **结论：定位个人/家庭，商家侧不碰**（卖点四件套只在个人侧共振）。

### 证据强度诚实评估（心法四要件）
| 要件 | 判定 |
|---|---|
| ≥3 独立来源 | **过**（HN 多线程 + 7 个独立 App 评论群） |
| 近两年 | **过**（核心 2025-07~2026-11） |
| 带日期 | **半过**（HN/Sortly 带日期；部分评论日期解析遗漏，表中如实标注） |
| 付费单列 | **过**（买断偏好 ≥6 + Home Contents 已付费实证） |

**结论**：需求侧过线（凑合行为直接自述+多源）；**定价侧未证**——最薄一环：明确愿付仅 $5-10 一条 + $5.99 已验证，**$19-29 买断档无直接铁证**；「好用的 OCR」降级为待验技术承诺（需求在、兑现风险大于需求风险）。
**B 路裁决建议**：「本地/买断/数据主权」三件套作不满侧卖点成立（订阅抱怨铁证直接推论）；OCR 降级待验；定价带按证据锚定 **$5-19 买断**，$29 档另找证据。

## A 路 · 竞品全景 + issues + 定价（压缩版 · 主会话代跑 2026-11-18）

> A 路 agent 被会话级 auto-mode 安全检查故障掐死（重启同死），按指令主会话补跑核心料；闭源竞品定价明细归 C 路。

**开源供给侧（GitHub 实抓 2026-11-18）**
| 仓 | 星 | 建/更 | 定位 |
|---|---|---|---|
| sassanix/Warracker | **1571** | 2025-03 建 / 2026-11-17 更新 | 自托管保修追踪「You own the data, your rules!」——**免费在位者正在成型** |
| lmmendes/attic | 175 | 2026-01 建 / 2026-11-15 更新 | 自托管家财 |
| ferret-project/ferret | 新 | 2026-11-13 | 家财 + REST API + MCP |
| Cordonnier-Software/Nestarr | 新 | **2026-11-18 当天建仓** | 家财 |

**Warracker open issues（缺口信号）**：#261 维护活动/备件/许可 (2026-10-03) / #182 备份导出 (2025-10-31) / #179 CSV 导入合并 (2025-10-31) / #152 单条保修导出 PDF (2025-10-09) / #99 重叠保修建模 (2025-10-21)——**维护记录与导出是共同缺口**，与 HomeLogger「找不到 home maintenance tracker」互证。attic issues：图片优化/批量操作/导入流/共享收藏（家财长尾细化）。

**判读**
1. 自建潮在开源侧同步燃烧（本周仍有新仓/日更），但形态是 **self-hosted/Docker**——服务 homelab 人群，不是「装个 exe 就用」的普通消费者；**桌面买断侧（Windows）空着**
2. **免费在位者警报**：Warracker 20 个月 1571 星、口号即数据主权——「买断+数据主权」护城河有一半正被开源免费拿走；差异化只剩「零部署桌面买断 + 好用 OCR」半件
3. B 路「Home Contents $5.99 买断 iOS 立住」+ A 路「桌面侧空」= 形态判断两根柱子，等 C 合围

**未覆盖（被环境掐，如实记）**：Sortly/Itemtopia/HomeZada/Centriq 官网定价页明细、功能矩阵逐项——由 C 路定价区间部分覆盖。

## C 路 · 吸收距离 + 形态 + 定价 + 粗算原料（2026-11-18 交卷）

**通道**：HN Algolia/GitHub API/iTunes API/Sortly+Itemtopia 官网实采 ✅；Bash 中段反复封停、WebFetch 全拦；Apple/Google/钱包/TurboTax/HomeZada/Centriq 实价未实采=【先验】标注（置信中，但无一能翻转主结论）；路2 遗留 stashdog.html 已消化。

**① 吸收距离（拆两层）**
- 上半层「票据流水」= **高危，已在发生**：Apple Wallet 订单/票据聚合（iOS17+）、支付宝发票管家/华为票据夹【先验】、Gmail 收据抽取【先验】——「存得起找得到」正被钱包/相册吸收；残余=纸票 OCR 结构化质量（质量竞赛非形态缺口）
- 下半层「保修/维护/家财」= **低危 10-15%**：全平台只有自家设备保修查询，无跨品牌家财台账+维护日程+保险报告；无支付钩子/无网络效应/需长期录入——不是 checkbox 是产品线；微软消费者面空白。**脊柱只能长在下半层，OCR 只是入口钩子**

**② 形态定位（实采）**
自建潮形态 = 自托管 Web（homebox ★7904 / Warracker ★1571 / attic / Nestarr）+ iOS/Android 捕获侧（ReceiptBin / StashDog / SimplyWise 38.6k 评 / ReceiptJar 66k）+ Mac 买断长尾（BluePlum $24.99，2016-2025）——**Windows 桌面 = 零**。
「谁在 Windows 上？」**≈0%，证据不支撑**。Windows 沾边人群=Docker 自托管者（服务器需求，免费双雄在位，买断无杠杆）。付费动机（OCR 付费墙）在捕获侧手机、农场优势域在管理侧 Windows——**两头错位**。
单人成本：Windows 6-10 周（证据不支撑=赌证据外人群）＜ PWA 8-14 周（正面撞免费自托管）＜ iOS 3-6 人月（证据对位但零引擎复用+OCR 深坑）。
**诚实结论：需求证据支撑的形态与农场优势域不重叠。**

**③ 定价原料**
Sortly 实价：Free 100 项 / Advanced $288/yr / Ultra $888/yr（B2B 逻辑）；HomeZada ~$59/yr【先验】；**买断先例**：BluePlum $24.99 Mac 九年存活、Home Contents $4.99、MyStuff2 Pro——「买断能活不能火」，无健康流水先例。用户抱怨三条全指向「捕获侧 OCR + 订阅」，不指向管理侧功能。$19-29 = 存活带 ≠ 繁荣带。

**④ 粗算原料（$24 中位单价）**
MS Store 45 浏览/月 ×10% 安装 ×2-5% 付费 ≈ 0.1-0.2 单/月；GitHub+内容 ≈ 0.1-0.25 单/月 → **基准 ~$60-130/年**，乐观（内容爆量）$288-720/年；工时 3-6 人月（iOS 对位）/ 6-10 周（Windows 无据）；**品类流量在移动 ASO 战场（SimplyWise/ReceiptJar），农场渠道恰落在流量真空**。

**C 路结论**：脊柱=下半层；形态与农场结构性错位；买断带=存活带；粗算不等价。

## 推理链书面

- **现象**：2025-2026 个人/家庭收据+保修+家财的凑合潮——相册+系统 OCR+手工整理（Saldo 评论原话）、灾后凭记忆补保险清单（野火索赔）、找不到就自建（HN Show HN ≥8 + 开源 Warracker/homebox 双雄）、订阅付费墙骂声+「lifetime purchase please」表态 ≥6 条。
- **本质**：商业工具按 B2B 逻辑定价（Sortly $288-888/yr、100 项免费墙）把个人/家庭挤出；订阅把数据扣作人质（「没订阅就删你照片」）；平台只吸收票据流水、不碰保修/家财台账——个人资产台账是商业真空+平台真空。
- **什么在变**：①自建成本崩塌（AI 辅助）→ 供给端自建潮爆发（本卡现象即此）②钱包/发票夹吸收票据层 → 需求重心从「存票据」滑向「保修/维护/保险就绪」③订阅疲劳 → 买断偏好成声（Home Contents $5.99 实证立住）。
- **方向**：买断+本地+数据主权的**保修/维护/家财台账**（下半层脊柱），票据 OCR 只做入口钩子。
- **⚠️ 断环（诚实标注）**：「方向→载体」环无据——证据里的「谁」≈0% 在农场可服务形态上（捕获在手机、归档在自托管/云，Windows 为零）。按心法「哪一环没证据哪一环断，推理不许跑到证据前面」：**方向成立，载体无据**。

## 失效标志（写死）

- 吸收预判验证钟：2027-05 前若 Apple/Google 推出**跨品牌**保修/家财台账（概率判 10-15%）→ 下半层「平台不碰」判定反转，本域永埋
- 形态预判验证钟：Windows/桌面侧凑合证据若出现 ≥3 独立来源带日期 → 「载体无据」断环弥合，立即重开评估
- 自托管在位者预判：Warracker/homebox 半年不更新+issue 区求救潮 → 免费在位者威胁解除（反向失效，利好重开）

## 一页粗算

| 项 | 数 |
|---|---|
| 渠道可见流量 | MS Store 45 浏览/月（pomodoro 实测基准）+ 内容 500 阅读/月 |
| 转化 | 浏览→安装 10%；安装→付费 2-5%（pomodoro 0 付费、log 付费层开业后的历史区间） |
| 单价 | $24 中位（B 路证据锚 $5-19，$29+ 无铁证；BluePlum $24.99 存活带） |
| **年收** | **基准 $60-130 / 乐观 $288-720** |
| 工时 | iOS 对位形态 3-6 人月；Windows 形态 6-10 周（需求证据零支撑） |
| 判定 | **不等价**；且品类流量在移动 ASO，农场渠道在流量真空 |

## 红队记录（2026-11-18，fresh-context doubt-driven；总裁决=修改后维持）

攻击清单（致命 4 / 不致命 5，全文见会话记录）：
1. **【致命】依据1 越权**：「载体/农场优势域」是 11-17 已降级的引擎约束借尸还魂——推理链四环本就不含「载体」，「农场优势域错位」不作毙掉理由；「载体无据」实为「未搜」（桌面渠道未取证）
2. **【致命】依据3 踩红线**：竞品/护城河是执行问题，不许冒充需求问题进生死判定（心法红线明文）；连带复活钥匙③「竞品停更」与红线不对称——须删
3. **【致命】依据4 偷渡**：需求侧自评「带日期半过」却写成「成立归档」；「买断偏好表态」按「言词不算」应折价；付费实证仅 $5.99 单点
4. **【致命】「Windows ≈0%」= 幸存者偏差**：取证通道（HN/GitHub/iTunes）天然不覆盖桌面份额软件；且窄形态「照片档案柜」（存图找得到引语+Excel 土法线索+BluePlum 先例）**被跳过而非被驳倒**
5. 反向律师救活稿：窄形态 POC（Windows 照片档案柜，$9-19，4-6 周）+ 免费档案柜/付费保险报告模型 + $12 中位——差距缩一个量级但仍不等价
6. 【不致命】粗算：$24 与证据锚 $5-19 自相矛盾（三实证中位=5.99）、2-5% 转化无自家支撑——**改 $12 中位+转化标上限假设后更硬；参数全错不翻符号**
7. 【不致命】A 路压缩不满足三闸完成度、闭源 issues/买断流水【待补】——大概率加固不翻转
8. 【不致命】复活钥匙一虚一错一脆——须按杀因重做
9. 【不致命】杂项：单引语过度推断×2、两条 HN 引证未回证（塌了仍过近两年线）、【先验】条目不动主结论

## D 路 · 桌面侧定向补扫（2026-11-18 用户选 B 后交卷）

**硬结论：合格独立证据 0 条（0 < 3）。** 没有把商品存在、模板存在、销量摘要、开发者自荐、移动场景、非 Windows 平台或日期不明条目充数；本次未触发窄形态翻案重评，更不自动授权 POC。

### 近失、窗外及未回证条目（全部不计数）

| 日期 | 条目 | 实际行为/原话短引 | 排除理由 |
|---|---|---|---|
| 2026-02-24 | 小众软件 liukp981，[家庭用品维护](https://meta.appinn.net/t/topic/81863) | 「查找淘宝记录……分不清这个卫生间的是哪家的了」「我不记得当时卖家是否承诺三年保修」 | 有真实保修痛点和淘宝记录土法，但楼主后选「模版记 App」，未证明 Windows；15 楼「在 Win10 平台用一个 excel 表格，再加上文件架」只是方案建议，不是本人正在这样用 |
| 2025-09-18 | ZDNET Jack Wallen，[五款家财工具实测](https://www.zdnet.com/home-and-office/work-life/have-a-lot-of-stuff-to-track-my-5-favorite-home-inventory-apps-can-help/) | 「It took me five minutes to build my app and start using it.」 | 有实际使用，但只说 Airtable 支持 Windows，未证明作者实际在 Windows 使用；同文 Homebox 是 LAN 自托管 |
| 2026-10-29 | SuperUser，[保存 Shopify 收据](https://superuser.com/questions/1941290/how-to-save-shopify-receipts-in-firefox) | 「If I wish to save a receipt … my own records … I cannot print … as PDF.」 | 是实际存档行为，但原文明写使用 Linux，排除出 Windows 桌面证据 |
| 2026-10-16 | 小众软件 Jack_Zhu，[Thingary](https://meta.appinn.net/t/topic/93726) | Windows 本地物品档案含价格、照片、保修与维修，形态高度吻合 | 全文仅开发者自荐，无独立用户使用行为；供给侧存在不等于需求证据 |
| 2026-06-19 / 2026-05-02 | SuperUser，[ReceiptToCSV 回答](https://superuser.com/a/1938530) / [Photoshop 回答](https://superuser.com/a/1937259) | 前者披露「I am the developer/creator」；后者称「methods I have used」 | 前者是上传式 Web 工具且商家自荐；后者只增强收据黑字，没有整理、保修或保险流程，亦未明确 Windows |
| 日期不明 | HomeManage 厂商证言 Doug，[原页](https://www.libertystreet.com/Home-Inventory-Software.htm) | 「I downloaded and tried a half dozen other home inventory software programs…」 | 有寻找/迁移行为，但证言无日期，不能证明落在近两年 |
| 2023-06-06 | 小众软件 tthy，[Joplin 存档帖](https://meta.appinn.net/t/topic/44373) | 家电说明书拍照塞入 Joplin，电子发票也塞入留档 | 明确土法但在时间窗外，且未明确 Windows |

### 实际通道覆盖与失败（不得外推成「全网无需求」）

- **中文桌面渠道**：实际调用 `meta.appinn.net/search.json` 检索收据、收据整理、保修、家电记录、说明书拍照、发票留档、票据整理、保单管理、家财、`home inventory` 等，并回取主题 JSON；appinn 主站未得到可核验用户行为；52pojie 请求跳成 Bing 页面；华军、天空定向搜索无可用原帖结果。另一次「小众软件物品清单」后台查询因系统内存严重不足被自动终止，没有可用结果，未计入证据或通道结论，也未重启。
- **SuperUser / StackExchange**：API 时间窗 `1731888000–1794960000`，查询 `receipt`、`warranty`、`home inventory`、`receipts organize`、`insurance inventory`、`photo receipts`、`warranty receipts`，并查五个相关老问题的新回答及评论；近窗仅得上述 Linux、自荐和图像修补等不合格项。
- **模板行为**：打开 Vertex42、Spreadsheet123、Smartsheet、Microsoft Create、UPHelp，只见模板或旧幸存者示例，无近窗带日期使用评价；Etsy 原页连接失败，商品与销量均未计数；Gumroad 未找到对应家财模板评价。
- **补充桌面渠道**：[微软商店 Home Inventory](https://apps.microsoft.com/detail/9wzdncrdp97b?hl=en-US&gl=US) 原页可开但只有评分汇总，评论正文和日期未返回，搜索摘要未回证不计；Microsoft Answers、MajorGeeks 连接失败；NerdWallet 被 Cloudflare 拦截；WebSearch 多次空返回，Bing RSS 多次偏题。

### 反方检查与机械含义

合格条目为 0，故无报销/企业记账、移动捕获、云自托管、模板 SEO 或商家自荐混入计数；票据搜索中的财税 Agent、门诊工具、报销 App 均已剔除。最强痛点帖落在手机 App/淘宝记录，最强 Windows 软件命中则是开发者自荐或无日期厂商证言，均不能抬高 N。

**N=0<3：本次定向扫描未补足 Windows 桌面证据，维持产品轨放弃建议；不触发窄形态翻案重评，不启动 POC。扫描存在上述失败通道，结论是「本次未找到合格证据」，不是「全网需求已证伪」。**

## 裁决段（用户终裁已落 · 2026-11-18）

**终裁：放弃，产品轨判死。用户原话「接受判死」。埋葬条款生效：不主动提起、永不开工，仅用户本人可复活；不进 spec、不启动 POC。判法=产品轨死、个人/家庭需求「倾向成立（半过）」非证伪；Windows 窄形态经定向补扫仍为 0/3，未过立项证据门槛；复活钥匙按杀因保留。**

依据（红队修订 + D 路补扫）：
1. **证据形态分布 + 推断不许越线**：已检索渠道内（HN/GitHub/iTunes）载体集中在移动端捕获与自托管 Web；针对红队指出的桌面盲区，D 路已补扫华军/天空/appinn/SuperUser/模板行为等渠道，**合格独立 Windows 桌面证据 0/3**。按心法二「推理不许跑到证据前面」，桌面形态仍是未证推断，**不按证据外形态直接立项**。载体差异按 11-17 口径不作毙掉理由；扫描有失败通道，结论是「本次未找到」，不是「全网无需求」。
2. **粗算闸不等价（主杀依据）**：单价按证据锚 **$12 中位**（$4.99/5.99/24.99 三实证），转化标注自家实测趋近 0、2-5% 为上限假设；基准 $60-130/年、乐观 $288-720/年 vs 对位工时 3-6 人月；**刻意偏乐观仍不等价——参数全错不翻符号**（收入×5、$12 中位、工时压 4 周仍是数周换 $300-700/年）。
3. **付费证据带太低（主杀依据之二）**：$5.99 单点实证 + 表态折价（言词不算），$19-29 买断档无铁证——窄形态救活也只够存活带。
4. 需求侧判「**倾向成立（半过）**」：≥3 源/近两年过、日期要件半过、付费单点；归档「待补日期后复核」，**不写「成立归档」**。

注脚（执行风险，不入生死判定）：买断+数据主权护城河半边正被开源双雄零价侵蚀（homebox ★7904 / Warracker ★1571），差异化剩零部署+OCR 半件——POC/spec 阶段再算；「免费档案柜+付费保险报告」变现模型本轮未评估。

**窄形态补扫结论（归档，产品轨不立项；非全网需求证伪）**：「Windows 照片档案柜」——照片导入+检索+保修到期+保险索赔清单导出，$9-19 买断，OCR 留 v1.x，工时 4-6 周。原证据形状=「dedicated place to store pictures」引语 +「Excel 手工录」土法线索 + BluePlum 桌面买断九年先例；D 路补扫后最强 Windows 命中 Thingary 仍只有开发者自荐，其余近失均因平台、日期或非本人行为被剔除，故 **0/3，未达到重评门槛**。重开须同时走钥匙①②。

**复活钥匙（按杀因重做，三条）**：
① **新增**桌面侧凑合证据 ≥3 独立来源带日期——2026-11-18 定向补扫已完成且 0/3；只接受新出现或原失败通道中可回证的行为证据，D 路近失、自荐、日期不明条目不得换皮重复计数
② 粗算基准反转：农场渠道可见流量或内容转化出现 10× 级变化（与杀因 2 对称）
③ ~~用户投移动~~ → 改：移动/iOS 形态若用户本人立项，**记为新选型不叫复活**（用户本就有此权，不作钥匙计）
（原钥匙③「自托管双雄烂尾」删除——竞品不能杀它，竞品停更也不能复活它；降级为环境观察项。）
