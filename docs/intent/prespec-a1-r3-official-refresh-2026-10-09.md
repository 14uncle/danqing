# A1 pre-spec R3 · 官方迁移刷新 + 失效标志全档复核（2026-10-09）

> 取证 agent 第 3 路。基线卷：`deep-dive-round8-t1-2026-09-25.md`（D3 官方迁移 2026-09-25 版）。
> 本轮 = 09-25 → 10-09 两周 diff + 补盲区（豆包优先）+ 失效标志 7a–e 逐条判。
> 时间盒实耗约 55 分钟。全程中文。**严禁编造**：本卷每条证据均带日期+来源；拿不准一律标「未验证」。

## 0. 取证可达性实况（本轮复测 2026-10-09）

| 源 | 状态 | 备注 |
|---|---|---|
| gh api / gh search | ✅ 可用 | `search/repositories?q=` 取 `total_count`；`gh search --json` 无 totalCount 字段，需走 REST |
| HN Algolia | ✅ 可用 | `hn.algolia.com/api/v1/search` |
| SE API (genai) | ✅ 可用 | quota_remaining 167 |
| OpenAI community Discourse | ✅ 可用 | `community.openai.com/search.json` |
| 厂商官网 curl（豆包/DeepSeek/Kimi/千问/135） | ✅ 可用 | 返回 200，JS bundle 可挖 i18n/module 名 |
| DuckDuckGo HTML | ❌ 死 | HTTP 000（与 09-25 一致） |
| B站搜索 API | ❌ 死 | HTTP 412 风控（与 09-25 一致），**豆包教程反证法本轮不可用** |
| smzdm 搜索 | ❌ 死 | HTTP 202 空壳，未验证 |
| 文心一言 yiyan.baidu.com | ⚠️ SPA | 仅 11KB 壳，导出能力未验证 |
| 微信 mp.weixin.qq.com | ⚠️ | 200 但只有登录门户，编辑器能力需生态反推 |

---

## 一、清单 1–6 逐项状态 + 证据 + 日期

### 清单 1 · 豆包「AI 文档」盲区补验 —— **重大更新：盲区基本被填平，产品面实锤**

**结论：豆包已具备 docx 产品面（含下载/导出 UI），不再是 09-25 的「最大盲区」。但「对话→docx 一键导出且格式保真」整句仍未验证（需人工实测）；公众号 HTML 一键导出零命中。**

证据（2026-10-09 直连 `https://www.doubao.com/chat/`，HTTP 200，页面 286KB）：

前端 JS bundle 模块/i18n 名实锤（均为页面源码原文，非推断）：

| 模块名 | 含义推断（标注） |
|---|---|
| `docx-export-popover` | docx 导出弹层（UI 存在） |
| `white-doc-exporter` | 文档导出器 |
| `s2-creation-export` / `s2-canvas-scene-export` | 创作/画布导出 |
| `s2-team-workspace-docx-download` | **团队工作区 docx 下载** |
| `s2-office-artifact-docx-card` | office artifact docx 卡（artifact 概念） |
| `s2-office-card-docx` + `s2-office-card-ppt` | office 卡片：docx + **PPT** |
| `canvas-inline-docx-card` / `canvas-docx-panel` | 对话画布内嵌 docx |
| `step-docx-panel` / `s1-step-docx-panel-v2-runtime` | 分步 docx 面板 |
| `s2-standalone-office-docx-canvas-definition` | 独立 office docx 画布 |
| `s2-wiki-docx-legacy-card` / `s2-wiki-docx-office-card` | 知识库 docx 卡 |
| `docx-preview-modal` / `docx_caption_component` / `docx_comment_module` / `docx_delay_find_replace` | docx 预览/题注/批注/查找替换 |
| `module_extensions_markdown` / `simple-markdown-previewer-modal` | Markdown 扩展/预览 |
| 「帮你一键生成 PPT」（页面可见中文文案） | PPT 一键生成 |

- **B站教程反证法本轮失效**（API 412），无法用「无教程」反证；但官网 bundle 已自证功能面存在，盲区性质从「未验证」改为「产品面已实锤」。
- **判定拆分**：①豆包有 docx 生成/编辑/下载面 = **兑现**（模块名+下载字样）；②「对话→docx 一键导出且格式保真」= **未验证**（保真度无人实测，形态更像「AI 文档/office 画布」而非「对话历史导出」）；③豆包做公众号 HTML 排版导出 = **零命中**。

### 清单 2 · Kimi / DeepSeek / 文心 / 通义 —— **DeepSeek 重大更新；Kimi/通义方向确认；文心未验证**

**DeepSeek（09-25 判「官方无导出」→ 本轮被推翻一半）**：
- 官网 `https://www.deepseek.com/`（2026-10-09 直连）导航栏出现 **「DeepSeek Harness」** + **「Harness 桌面端」**（Windows 10 或更高版本下载注记，i18n 键 `harnessHeroDownloadWindowsNote`）。Harness 在页面出现 **138 次**。
- Harness 能力面（i18n 原文）：「一切皆插件——安装现有插件，或在『创造模式』中通过对话创建插件」「自动化任务」「工作区/默认工作区」。
- 演示会话原文：**「生成 Q3 数据汇总 PPT」（9 分钟）**、「准备明天评审会的议题清单」「重构登录模块的校验逻辑」「给销售报表补上同比/环比列」「梳理接口文档并补充示例」。
- 工作区文件类型 i18n：**`简报.docx`（Word 文档）**、`数据.xlsx`（Excel 表格）、`报告.pdf`（PDF 文档）、`README.md`（Markdown 文档）、`analyze.py`、`label.ts`。
- **判定**：DeepSeek 官方已在推「桌面端 agentic 工作区 + 文档/PPT 生成」。但「对话→docx 一键导出且格式保真」是否已上线 = **未验证**（官网是营销演示层，未见导出保真实测）；公众号 HTML = 零命中。

**Kimi（方向确认，较 09-25 加宽）**：
- `https://www.kimi.com/`（2026-10-09，HTTP 200）meta keywords 原文含 **「PPT, Excel, PDF」**；description 含「全新 Kimi K3 系列……**PPT 制作**……Swarm 混合智能体集群、Goal 模式」；title =「Kimi AI 助手 - K3 系列」。
- 与 09-25「Kimi 已官方做 PPT 生成」一致且扩到 Excel/PDF + Agent 集群。**「对话→docx 一键导出」仍未直接验证**；公众号 HTML 零命中。

**通义/千问**：
- `tongyi.com` 302 → `qianwen.com`（2026-10-09，HTTP 200）。meta 原文：「AI 搜索、网页总结、**AI PPT**、AI 生图、**PPT 创作**和……」。PPT 创作确认；docx 导出未验证；公众号 HTML 零命中。

**文心一言**：
- `yiyan.baidu.com` → `wenxin.baidu.com`，仅 11KB SPA 壳，无关键词。**导出能力未验证**。

### 清单 3 · OpenAI —— **help 文有 diff；导出诉求仍在生；无继任导出**

| 项 | 状态 | 证据 |
|---|---|---|
| help 20001246（writing blocks） | **两周内有更新** | 页面显示「Updated: 3 days ago」；JSON `updatedAt` = **2026-10-06T20:03:46Z**（本轮抓取换算） |
| help 20001246 是否新增导出 | **否** | 正文（2026-10-09 抓取）仍为 writing blocks / code blocks 全文，动作=编辑/复制/全屏/撤销重做/发邮件草稿/**保存文档草稿到 Library**；格式=加粗斜体/标题/链接/列表/清单；预览=HTML/React/SVG/Mermaid/Vega。**全文无 docx、无 PDF 导出、无「export as」** |
| help 7260999（数据导出） | **两周内无变化** | JSON `updatedAt` = **2026-08-25T12:51:41Z**，页面「Updated: 2 months ago」；内容仍是 Data Controls 全量 ZIP 导出 |
| Canvas 继任导出 | **无** | writing blocks 仍是 Canvas 继任形态，未提供 docx/PDF 导出；社区 2026-08-28 仍在请求「Restore Canvas as a persistent workspace」（5 赞） |
| 导出诉求新表态 | **持续新鲜** | OpenAI community（`community.openai.com/search.json`）：2026-10-07「Ability to Export Individual / Single Chat Threads」；**2026-10-01「Export Individual Chats to PDF, Word, or Other Readable Formats」**（正文原话「export an individual ChatGPT conversation as a PDF or DOCX」）；2026-09-29「a ChatGPT conversation is no longer just a temporary chat」（1 赞）；2026-09-26 法语用户请求对话菜单加导出 |
| genai SE | **降温但不为零** | 自 2025-09-01 仅 3 条 export 问题，最新两条仍是 **2026-06-11**（与 09-25 基线同源）；**06-11 之后 genai SE 无新 export 问题** |

**OpenAI 两周 diff 一句话**：文章动了（10-06）但**没加导出**；用户侧导出请求本周仍在发；ChatGPT→docx 官方窗口仍关。

### 清单 4 · Anthropic —— **两周内重大加码：Claude Docs + Google Workspace，窗关得更死**

| 日期 | 事件 | 来源 |
|---|---|---|
| 2025-09-09 | 文件创建 .docx/.xlsx/.pptx/.pdf（基线已录） | HN 头版 |
| **2026-09-16** | **Anthropic 发布 Claude Docs（Google Docs 替代品）** | HN 标题「Anyhropic Launches Google Docs Alternative: Claude Docs」，链 theverge.com/ai-artificial-intelligence/996234/... |
| **2026-09-17** | 「Claude Slides, Claude Design and Claude Docs」 | HN，YouTube 演示 |
| **2026-10-06** | **Claude for Google Workspace 公测**：在 Google Docs/Sheets/Slides 内用 Claude，也可从 Claude 编辑 Google 文件 | claude.com/resources/articles/claude-now-works-in-google-docs-sheets-and-slides（正文标注 Date: **October 6, 2026**；Category: Announcements；Product: Claude Enterprise） |
| 2026-10-09（抓取日） | artifacts 帮助文「Updated today」，原文含「export to **PowerPoint or PDF**」「Use Claude Slides」 | support.claude.com/en/articles/17153992-what-are-artifacts-and-how-do-i-use-them |
| 抓取日 | 「Share artifacts」帮助文「Updated over 2 weeks ago」：协作者可 download artifact 提供的文件 | support.claude.com/en/articles/9547008-share-artifacts |

**格式保真细节（Workspace 文原文）**：改写标题「without touching the surrounding formatting」，大改以「suggestion cards」提案——**官方在把「格式保真」做成卖点**。

**对话历史保格式导出**：本轮未发现新进展（无新 help 文/无 HN 正面信号）；genai SE 2026-06-11 两条仍是 workaround 答案。**维持 09-25 判定：未做**。

**对 A1 形塑（较 09-25 加强）**：Anthropic 已从「文件创建」长到「Docs/Slides/Design + 进 Google Workspace」。**通用 docx/pdf 腿对 Claude 用户窗口关闭且在焊死**；公众号 HTML 依旧零沾染。

### 清单 5 · 微信侧 —— **原生 Markdown 仍无；但编辑器在加「检测/暗黑」治理；135 全面 AI 化 + 免费体验**

**官方编辑器是否原生支持 Markdown 粘贴**：
- 直连 mp.weixin.qq.com 只有登录门户（2026-10-09），无法实测编辑器。**维持生态反推**。
- **反证仍然成立且更新鲜**：①gh `search/repositories`：`微信 markdown created:>2026-09-01` = **102 仓**、`公众号 markdown created:>2026-09-01` = **99 仓**（5 周内，第三方工具仍在繁殖）；②gzh-design-skill issue #19（2026-09-22）「fix: preserve raw HTML when copying preview to WeChat editor」——**粘贴保真仍在修**，若原生支持 Markdown 这类 issue 不会存在；③若原生支持，doocs/md 等 13412★ 生态即无立足点。
- **判定：未兑现（间接证据强）**。

**「官方一键排版免费」信号（smzdm 2026-08-22）现状**：
- smzdm 本轮 HTTP 202 空壳，**原帖未复核（未验证）**。
- **替代证据：135editor 官网（2026-10-09 直连 205KB）已把 AI 排版做成免费引流入口**——页面原文：「135AI排版」「**免费体验AI排版**」「专业公众号排版与AI排版工具」「AI 一键生文、AI 文章配图改图」「135GEO 是 AI 搜索获客产品」「135Vizmax AI 图文视频生成工具」「笔记双平台分发」。AI 排版从付费墙内走到「免费体验」= **免费化信号在 135 侧坐实（新证据）**。
- 秀米：xiumi.us 返回 200 但仅 JS 常量壳，AI 状态**未验证**（09-25 基线「AI 化弱」暂维持）。

**编辑器治理新面（重要，平台依赖风险具体化）**：
- gzh-design-skill issue #21（**2026-10-07**）：「粘贴到公众号编辑器触发**「内容结构检测」弹窗**（darkmode-no-gradient）：文字/卡片背景用渐变，建议默认改纯色或在校验中提示」→ 10-09 修复合并「默认使用纯色文字背景并提示渐变风险」。
- issue #20（2026-09-29）：「关键视觉不要依赖『渐变』与『实底+白字』——**Dark Mode 转换**会让输出不可预测」。
- **含义**：公众号编辑器对粘贴内容做结构检测 + 强制 Dark Mode 转换。A1 的模板体系必须把「渐变禁区 + 暗黑可预期性」列为一等约束（风险④平台依赖的具体新条款）。

### 清单 6 · 温度旁证（gh 实查带日期）

**建仓/issue 计数（gh REST total_count，抓取 2026-10-09）**：

| query | 总量 | created:>2026-09-01（约 5.3 周） |
|---|---|---|
| `公众号 排版`（repos） | **757**（09-25 卷 2026YTD=619） | 136 条 issue 口径另计 |
| `markdown wechat`（repos） | **896**（09-25 卷 2026YTD=685） | 130 |
| `claude export`（repos） | **3168**（09-25 卷 2026YTD=2681） | **448** |
| `wechat format`（repos） | 249 | — |
| `公众号 HTML`（repos） | — | **48** |
| `微信 markdown`（repos） | — | **102** |
| `公众号 markdown`（repos） | — | **99** |
| `公众号 排版`（issues） | — | **160** |
| `wechat markdown`（issues） | — | **964** |
| `without losing formatting`（issues since 2026-08-01） | — | **216** |

**判读**：导出/排版类建仓**没有回落**，`claude export` 5 周新增 448 属爆发延续。失效标志 7c 的「增速回落到 2024 水平」**未兑现**。

**md2wechat-skill（geekjourneyx/md2wechat-skill）两周活动 —— 活跃且进攻性增强**：
- ★：3689（09-25 卷 3670）→ +19/两周。
- release 连发：v3.5.0（2026-09-07）、v3.6.0（09-12）、v3.7.0（09-23）、v3.8.0（**09-24**）。
- v3.6.0 特性名「multi-platform browser drafts」、v3.7.0「directed writing and **Tencent Cloud drafts**」。
- **README 现文案（2026-10-09 抓取）**：「写 Markdown，生成公众号排版，制作封面和文章配图，**预览校验后推送草稿箱**」「支持 Claude Code、Codex、WorkBuddy、Kimi Work、Hermes Agent、OpenClaw 等 Agent」「多平台草稿：CSDN、头条、腾讯云开发者社区草稿」「专业 API 统一版 **¥199/永久**」「83 个主推高级排版场景 + 59 个 `:::` 语法」。
- issue #29（2026-09-08 开）Vibe Pay 自助收银：**仍「规划·待启动」**；issue #30（09-12 关）「Optional token metering & paid API key support」= 付费 API key 意向（closed，非成交证据）。

**gzh-design-skill（isjiamu/gzh-design-skill）—— 沉寂论被推翻，已复活**：
- ★：3946（09-25 卷 3832）→ +114/两周（**涨速是 md2wechat 的 6 倍**）。
- 09-25 红队勘「2026-07-08 后沉寂近 3 月」**已过期**：commit 链 2026-08-12 → **09-22**（fix WeChat preview copy serialization）→ **10-03**（两 PR 合并：docx table pipe 修复 + wechat-copy-serialization）→ **10-09（今日）**（fix 渐变背景）。
- issue 流：#17（09-02）注册 project-scoped extension；#18（09-07）preview copy；#19–#22（09-22→10-09）全是「贴进公众号编辑器」保真/检测/暗黑问题。
- releases 仍停 v1.0.0（2026-07-06），无新 release——但 master 持续有社区 PR。
- **「skill 热度是脉冲式而非潮汐」假说偏负向**：两 skill 双双活跃，且 gzh-design 在加速。

**其他在位者**：
- doocs/md：13412★（+39/两周），pushed 2026-10-09，但 commit 全是依赖升级——**维护态、仍未做 docx/PDF**。
- wenyan-pc：95★（+1），pushed **2026-05-19**——停更约 5 个月，弱占座者未变强。

**HN「without losing formatting」教程（题名检索）**：
- 2026-08-17「copy equations from ChatGPT to word」
- 2026-08-21「copy equations from **Grok** to word」
- 2026-08-26「export **Gemini** chat to word」
- 2026-09-14「copy tables from ChatGPT to Google Docs」
- 2026-09-21「export ChatGPT to Google Docs」
- **2026-09-28「export ChatGPT to word」**（取证日前 11 天）
- **教程品类未停产，且已扩散到 Gemini/Grok**。

---

## 二、失效标志 7a–e 逐条判

### 7a.「ChatGPT 或豆包/DeepSeek/Kimi 官方上线对话→docx/公众号 HTML 一键导出且格式保真」→ **部分兑现（黄灯）｜公众号 HTML 半句未兑现｜保真度未验证**

| 子项 | 判定 | 依据（日期） |
|---|---|---|
| ChatGPT 对话→docx/HTML 一键导出 | **未兑现** | help 20001246（updatedAt 2026-10-06）无导出；社区 2026-10-01/10-07 仍在要 PDF/DOCX |
| 豆包 docx 产品面 | **兑现（产品面）** | 2026-10-09 doubao.com bundle：`docx-export-popover`/`s2-team-workspace-docx-download`/`s2-office-card-docx`/`s2-office-artifact-docx-card`/「一键生成 PPT」 |
| DeepSeek 文档/PPT 生成 | **兑现（产品面/营销面）** | 2026-10-09 deepseek.com：Harness 桌面端 + 工作区 `简报.docx` + 演示「生成 Q3 数据汇总 PPT」 |
| Kimi PPT/Excel/PDF | **兑现（营销面）** | 2026-10-09 kimi.com meta：PPT/Excel/PDF + K3 |
| 「且格式保真」 | **未验证** | 无任何保真实测/用户原话；Anthropic 官方自述文件创建仍需「review and refine」（09-25 基线） |
| 公众号 HTML 一键导出 | **未兑现（零命中）** | 豆包/DeepSeek/Kimi/千问官网均无公众号 HTML 字样；无任何官方做中文公众号排版导出 |

**总判**：这是一条「或」语句——豆包/DeepSeek/Kimi 的**文档生成面已长出来**，触发黄灯；但句子的技术要件「对话→docx 一键导出**且格式保真**」整句未验证，且「公众号 HTML」半句零命中。**按农场失效标志纪律：产品面兑现 = 黄灯预警，不能判红灯处决；主腿（公众号 HTML）未被官方接走。** 建议把这条拆成两条监控（docx 黄灯已亮 / 公众号 HTML 绿灯），spec 阶段把豆包 docx 保真实测列为必做。

### 7b.「微信公众号编辑器原生支持 Markdown 粘贴」→ **未兑现**

- 直连门户无编辑器能力信息（2026-10-09）。
- 强反证：5 周内 `微信 markdown` 新仓 102 / `公众号 markdown` 新仓 99；gzh-design-skill 2026-09-22 至 10-09 连续修「粘贴到公众号编辑器」保真与检测问题；若原生支持，这些工具的存续理由消失。
- **补充（本条的新伴生事实）**：编辑器新增「内容结构检测」弹窗 + Dark Mode 强制转换（gzh-design-skill #20/#21，2026-09-29/10-07）——官方在治理粘贴内容，而非开放 Markdown。

### 7c.「官方一键导出格式完好 **且** 导出类仓季度增速回落到 2024 水平」→ **未兑现（双条件均未满足）**

- 前半（官方一键导出格式完好）：未验证（见 7a 保真项）。
- 后半（增速回落 2024）：**明确未兑现**——`claude export` 累计 2681→3168，`公众号 排版` 619→757（09-25→10-09），`markdown wechat` 685→896；created:>2026-09-01 仍有 448/136/130 条新仓。曲线仍在爬升。
- 「且」字使本条更难触发；当前两侧都不成立。

### 7d.「without losing formatting」类教程停产 → **未兑现（品类在扩张）**

- HN 题名近两月仍连续产出，最新 **2026-09-28**「How to export ChatGPT to word without losing formatting」。
- 品类已从 ChatGPT 扩到 **Gemini**（2026-08-26）与 **Grok**（2026-08-21）。
- GitHub issue 口径 `without losing formatting created:>2026-08-01` = **216**。
- 本条是 4 条失效标志里最像「科普体温计」的一条——**体温仍在升**。

### 7e.「对话即最终交付」成为主流 → **未兑现（但观测口出现重大新变量）**

观测口读数（2026-10-09）：
- **HN `ChatGPT export` 题名故事**：累计 36，最近 2026-09-20/09-21/09-28 三条——**导出诉求仍在月频发生**，交付还在「导出」环节。
- **HN `Claude Docs` 题名故事**：累计 107，但月计数混入 Claude Code docs；**关键新变量**是 2026-09-16 Claude Docs + 2026-10-06 Google Workspace 公测（Claude 直接编辑 Google 文件并保格式）。
- **genai SE export 类新问题**：2026-06-11 后 **零新增**（月增=0）。
- **判定**：用户行为侧（教程/Show HN/社区 feature request）显示「导出」仍是主流动作；官方供给侧（Claude Docs/Workspace/豆包 office 画布/DeepSeek Harness）在把「对话/agent → 可交付物」内化。**「成为主流」尚未发生，但两侧在相向而行——这不是失效标志的兑现，恰是它的「倒计时变快」。**

---

## 三、头号风险位是否变化 —— 一句话结论

**变化了：头号风险从「AI-agent skill 形态替代（脉冲 or 潮汐未决）」升级为「形态替代 + 官方 harness 化 双面夹击」——gzh-design-skill 已复活且 ★ 涨速 6 倍于 md2wechat（3946★）、md2wechat 已把「推送草稿箱+多平台草稿」写成主推（正是 A1 v1 明确砍掉的直推腿），同时 DeepSeek Harness 桌面端与豆包 office 画布在两周内从「未验证/无导出」跳到「产品面实锤」；A1 仍守住的唯一空位收窄为「**公众号 HTML 落地 + 中文场景 + 桌面离线**」交集，且必须把「渐变禁区/暗黑可预期/结构检测过检」列为模板一等约束。**

---

## 四、取证局限（诚实披露）

1. B站 API 412，豆包「有无教程」反证法本轮不可用；豆包结论全部来自官网 JS bundle/i18n 字段，**功能面≠可用面**，保真度零实测。
2. DeepSeek/Kimi/千问结论均为官网营销层（含演示会话 i18n），未登录实测；「对话→docx 一键导出」整句一律标未验证。
3. 微信编辑器无直连实测；Markdown 粘贴判定为生态反推（反证强但仍是间接）。
4. smzdm 原帖、秀米 AI 状态、文心导出能力 = 未验证。
5. OpenAI help 20001246 知道 10-06 更新，但无历史快照可 diff，**具体改动点未验证**（仅知当前版无导出）。
6. HN/gh 计数含噪声（query 间重叠、教程搬运站同步分发）；月度切片用题名检索，口径与 09-25 卷不完全同构。

## 五、对 pre-spec 的可执行回传（供三路合成）

1. **风险⑤豆包盲区可销**，改立「豆包/DeepSeek office 面跟进」——次级腿 docx 的窗口比 09-25 更窄，spec 时建议把 docx 降为 v1.1 或继续砍。
2. **风险⑥免费化信号坐实**（135「免费体验AI排版」）——付费层若做，卖点不能是「AI 排版本身」。
3. **风险④平台依赖具体化**：模板必须过「内容结构检测」+ 预设 Dark Mode 双主题 + 禁渐变；这是 A1 相对 skill 的质量差异位（skill 仓自己都在修这些）。
4. **失效标志监控升级**：7a 拆 docx/公众号 HTML 两条；豆包 docx 导出保真人工实测列入 spec 前必做；`help.openai.com/en/articles/20001246` 的 updatedAt 继续双周 diff（本轮已证有效：10-06 有更新）。
5. **形态替代时间表（环/风险②）有了读数**：md2wechat 直推草稿箱已上线（>A1 v1 终步）、gzh-design 复活、DeepSeek/豆包 harness 化——「脉冲式」假说偏负，建议 spec 时把 A1 的存活空间明文定义为「**离线桌面 + 公众号 HTML 一次成型 + 不绑单一 AI 源**」，并把「用户自己粘贴」保留为合规差异点（不追草稿箱）。

---

*报告完。取证日 2026-10-09，时间盒约 55 分钟。*
