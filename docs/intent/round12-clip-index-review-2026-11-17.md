# round12 · 片段即单位桌面复核（2026-11-17）

> 状态：**索引级初查，不可作开枪证据，主会话尚未终裁**。
> 取证限制：本地旧卷已直读；研究路的 WebFetch 与 curl 被权限模式阻断。下列外链均为 2026-11-17 搜索索引可见或旧卷摘录，**原页均未直验**。日期、价格与功能必须在采用前复核，不能把本卷写成官方实测。

## 一、研究路阶段建议

**仅建议一轮修订后的 10 人筛选访谈；不开发、不进 spec，不把预售意向当成交。**宽口径「翻素材、人工 selects、自建索引」仍有行为线索；严格限定为「Windows 主用 + 10TB+ 原片 + 跨项目找镜头」后，没有一条样本完成原页直验，也没有近期独立付费闭环。访谈若继续，任务是证伪这个交集，不是演示「一句话找镜头」。

## 二、需求证据：宽口径有线索，严格交集未达标

| 来源、日期 | 行为线索 | 证明边界 |
|---|---|---|
| [V2EX 1104923](https://www.v2ex.com/t/1104923)，2025-01-14（日期与摘要来自旧卷） | 30T 素材「找半天」 | 有 30T 检索痛；OS、职业、付费均未知 |
| [V2EX 1188451](https://www.v2ex.com/t/1188451)，2026-01-26（旧卷） | TB 级原片，压代理并走 10G | 有代理/网络成本；不知是否达 10TB 或 Windows |
| [Documentary Editing Tips 2025](https://digitalfilms.wordpress.com/2025/05/24/documentary-editing-tips-2025/)，2025-05-24 | 纪录片剪辑师仍逐访谈听片、建 selects，再按主题重组 | 人工 logging 未消失；部分是编辑判断，不等于检索痛，OS/库容未知 |
| [Meta Player](https://github.com/renweilong7/meta-player)，2026-05-07（旧卷日期） | 个人自建 Electron 本地素材库、SRT 剧情检索、点击跳片段 | 有土法工具；未证 10TB/Windows 主用，且依赖 OpenAI 兼容接口，不是完全本地 |
| [Jumper Trustpilot](https://www.trustpilot.com/review/getjumper.io)，2026-01（具体日未核） | 用户称 13,000+ 文件在 M1 Pro 上处理四天 | 大库索引耗时；Mac，TB 与购买与否未知 |
| [Adobe Community](https://community.adobe.com/questions-729/media-intelligence-visual-search-not-working-1417865)，2025-04-21 | Windows/Surface 用户分析完成后视觉搜索仍无结果，台词搜索可用 | Windows 内置能力失效实例；不是 10TB，也不是手工凑合证据 |

**研究路判定**：至少有四个不同站点/项目的行为样本，但只有首个明确 30T；没有样本同时直验 Windows、10TB+、持续跨项目找片段。旧卷「Gate 2 过」措辞过强，应降为弱线索。

## 三、付费与价格锚

- 近两年独立实际购买证据：**未找到**。Jumper 官网购买自白无日期且经厂商筛选；Kyno「we bought our package」是 2020-10，超出窗口。使用评论不自动等于购买。
- 标价、评分、下载均不算销量，本路也未取得厂商销售数据。
- 索引所见买断锚（全部未直验）：Eagle 中国约 ¥229、Focus $49/$89、ClipCatalog $99、Kyno $159/$349、Jumper $169、Fast Video Cataloger $197。
- 因而已有 ¥99–299 探针价不高于精确 AI 工具锚，却与 Eagle 同档、高于 Billfish/Pixcall 本地免费锚；低价既不是需求证据，也可能放大持续支持预期。

## 四、竞品与吸收：旧卷漏扫直接对手

| 产品 | 索引所见当前边界（均未原页直验） |
|---|---|
| [Premiere Media Intelligence](https://helpx.adobe.com/premiere/desktop/organize-media/file-organization/search-for-media-using-ai-powered-media-intelligence.html) | Windows/Mac，本地视觉、台词、元数据搜索并回时间线；视觉查询仍英文，只搜已导入项目；sidecar 可复用分析，但未证全库统一搜索 |
| DaVinci Resolve 21 | 索引称 Studio IntelliSearch 已覆盖人/脸/物体/视觉/台词搜索与命中剪辑；项目型 Media Pool，未证跨项目总库 |
| [Jumper](https://docs.getjumper.io/guides/manage-search-context) | Windows/Mac，本地视觉/语音、中文、跨项目/跨盘/断盘检索及多 NLE 插件；索引称 2026-11-09 仍更新大库能力 |
| [ClipCatalog](https://clipcatalogpro.com/features) | Windows 直接对手：本地视觉/Whisper 台词/人脸、中文、断盘/NAS、拖入 Premiere/Resolve；未证代理生成 |
| [Kyno](https://lesspain.software/kyno/) | Windows/Mac，本地浏览、标记、subclip、转码/代理、Send to NLE；未找到官方视觉/台词 AI 搜索 |
| Eagle / Billfish / Pixcall | 本地素材管理、标签/筛选或部分语义搜索；未证完整视频台词定位与 NLE 片段交接 |

若上述索引信息经直验成立，则「本地多模态找镜头并回时间线」已被 NLE 覆盖，「跨项目持久索引、断盘、中文、多 NLE」又被 Jumper/ClipCatalog 等覆盖，不能假定存在 18 个月空白；只剩更窄的中国 Windows 档案库体验/服务缺口可验证。

## 五、技术与维护边界

- 没有 Windows + 30TB 基准，不能给统一索引小时数；30TB 也不能直接推出索引体积，必须知道时长、采样率、向量维度/精度、缩略图及代理码率。
- Whisper 代码与权重的许可、视觉模型逐权重商用条款、FFmpeg 构建义务均需逐项核实。
- 整文件拖放与带 in/out 的片段回时间线不是同一难度；后者涉及 NLE API/XML/EDL、重链接与版本回归。
- 没有销量和工时数据，不能宣称 ¥99–299 买断足以覆盖持续兼容维护，也不能轻诺无限期全兼容升级。

## 六、若继续访谈的硬失效标志

10 人都必须先筛为 Windows 主用、10TB+ 原片，再计数；要求演示最近一次真实找片段，不问泛化「wow 值多少钱」。任一命中即停止：

1. 公开渠道招募不到 10 名合格者；
2. ≥6/10 近 30 天无具体跨项目检索事故，或认为 Premiere/Resolve/Jumper/ClipCatalog/Focus/文件夹已够；
3. 少于 3/10 有真实相关付费经历或愿付 ¥99–299 可退订金；
4. ≥6/10 不能接受首轮后台索引/额外存储，或真正需要是云协作、自动剪片而非本地检索。

必须证伪的假设依次是：**频繁跨项目痛、现有工具真不够、本地主权值得付钱、带时间码交接是必需品。**

## 七、主会话采用边界

本卷不能单独支持「继续访谈」，更不能支持开枪。与 PDF/UA 路合成前，应优先补直验：Premiere sidecar 与搜索边界、Resolve IntelliSearch、Jumper/ClipCatalog 官方功能和价格，以及至少三条严格需求原帖。若直验无法完成，只能把本路判为「证据不足、暂不投入用户访谈」，不能把搜索索引当成交付级调研。
