# Intent: 第四件产品 —— 大文件日志/JSONL 查看分析器 (Windows)

> 2026-09-05 选型流程：五域并行扫描 → 用户点选 1+2 双深潜 → 用户裁决开枪 A。
> 新选型方针（需求驱动·调研先行）首次实战，见用户级记忆 `product-selection-demand-first`。

- @author 十四叔
- @date 2026/09/05
- 状态: **已确认开枪**（用户显式选定）；待建仓 + 待 spec（spec→plan→build→review→code-simplify 五阶段，用户发起 spec 技能后逐段推进）

## 一句话

Windows 上的「原生快 + 现代 UI + JSONL 结构化」大文件日志查看分析器——十年阵地战无人认真打：免费侧一死一瘫，付费侧唯一活物有实锤性能裂缝，JSONL 列化桌面端真空。

## 已确认的意图

- **结局 (Outcome)**：GB 级日志/JSONL 秒开、虚拟化滚动、实时 tail + 过滤、JSONL 列化字段过滤的桌面工具，OSS 内核免费 + 结构化分析付费层
- **用户 (User)**：开发者 / 支持工程师 / 运维 / 系统管理员——造物主本人是日频用户（在场质检满格）
- **为什么是现在 (Why now)**：clipboard 终止腾出带宽；竞争格局十年最松（glogg 死 5 年、klogg 稳定版停更 4 年 273 open issue、BareTail 20 年未更新仍在分发）
- **成功 (Success)**：POC 阶段 = 性能对比可截图碾压（全部营销弹药）；发布阶段 = 首单外检（「有人用+愿意付费」是产品目标）
- **约束 (Constraint)**：单人带宽；长在 danqing 上（wgpu 虚拟化滚动 = 攻击面）；买断制否决订阅（Dadroit 按文件大小收订阅正是其口碑裂缝，反向操作即差异化）
- **不做 (Out of scope)**：编辑、SSH/远程、协作、图表仪表盘、SQL 查询（LogViewPlus 的 SQL 是十年后才补的）、订阅制

## 调研证据摘要（深潜 2026-09-05）

| 竞品 | 价位 | 近况 | 裂缝 |
|------|------|------|------|
| LogViewPlus | $45 个人 / $95 企业 / $2000·50席，买断 | v3.2.9 活跃（单人开发者 Toby 十年+） | 性能事故实锤：116KB 文件搜索 15 秒全程冻结（2023 官方论坛），大文件方案是切段而非真虚拟化 |
| klogg | 免费 GPL | 稳定版停 2022-06，nightly 停 2024-11 | 273 open issue；ANSI 颜色 5 年求而不得 |
| glogg | 免费 GPL | 2021-05 停更 | 事实废弃 |
| Dadroit | $98/年起，**订阅**（≤50MB 免费） | 活跃 | JSON 树非 JSONL 行场景；按大小订阅引怨气 |
| EmEditor | $60/年订阅（2025 转订阅引不满），买断 ~$260 | 活跃 | 16TB 上限技术标杆，但无日志语义 |
| lnav | 免费 OSS | 活跃 | JSONL 最强但 TUI，Windows 需 WSL 够不着 |
| VS Code 扩展 daucloud.json-viewer | 免费 | 活跃 | GB 级虚拟化表格 + JSON Pointer 过滤——**最锋利的免费对手**，JSONL demo 必须显著优于它 |

付费者画像（LogViewPlus Trustpilot 4.6）：排查生产日志的工程师/支持/运维；两档结构（$45 绑个人邮箱 / $95 绑域名可转让 + 席包）说明**企业报销通道是利润主体**（Log4View €2390 Site License 印证）。

## MVP 边界

**In**（均为日频刚需）：① mmap 秒开 GB 级文件 ② 虚拟化滚动（wgpu 差异化主战场）③ tail + 实时过滤 ④ 正则搜索+高亮（不劣于 klogg）⑤ **JSONL 列化+字段过滤+嵌套展开**（主炮）⑥ 日志级别着色 ⑦ 书签
**Out**：编辑、SSH/远程、协作、图表仪表盘、SQL 查询
**三大技术风险**：① 全文件正则搜索的 I/O 吞吐与索引内存（klogg #435 栽在这）② tail 时文件截断/轮转 ③ 编码检测（UTF-16/BOM/GBK 混排）

## 开枪前提（POC 入口判据，不达标不转正式开发）

1. 真实 1GB+ 日志实测，搜索/滚动对 klogg 与 LogViewPlus 形成**可截图的碾压**
2. JSONL 列化 demo 体验显著优于 daucloud 的 VS Code 扩展（独立窗口、秒开、不占编辑器）
3. 发布后首单外检

## 定价锚与渠道

- 个人买断 **$45**（正面贴 LogViewPlus，比它快+现代+JSONL）；企业 **$95/seat**；10 席 pack **$590**；不设订阅
- 渠道：GitHub 主（OSS 内核+付费层刀法：开源层对标 klogg 全功能抢「停更 4 年」接盘流量；付费层 = JSONL 列化/多文件时间戳合并/过滤器会话/导出）+ MS Store 辅（LogoRRR Pro 在架证明类目存在且稀薄）
- 冷启动：Show HN 打「klogg 四年未发版 + LogViewPlus $45 还卡」；r/sysadmin、r/devops；SO 十年老帖「JSON viewer to open large json files」与 LogViewPlus 性能事故帖下接客

## 候选队列（未选，归档备查）

- **B「Hazel for Windows」文件自动整理**：开枪（有条件）未选。空位 = $29-35 买断 + 现代 UI + undo 信任三件套（Hazel 仅单文件 revert、File Juggler 无 undo）；对手 File Juggler $50 活跃维护（前置扫描「定时扫描」说法已被深潜修正为事件驱动实时）；MS Store 可行（runFullTrust 打包，Sortly 过审先例）；开枪前提 = 信任三件套 P0 + ≤1 周 MSIX 打包尖刺 + 只做下载夹/桌面单场景。不选原因：在场质检弱于 A、引擎渲染肌肉闲置、首要风险是外部审核不可控。
- 枪毙归档：泛截图工具（两路独立验证红海）、AI 听写（空窗已关，三线挤压）、快速记录（入口被手机/微信掐死）、抓包（Fiddler 断供窗口诱人但团队赛道）、数字健康阻断器（潜力大但 MSIX 沙箱存在性风险未排除，护城河 70% 在 GUI 之外）。

## 悬而未决

- 产品命名与仓库名（danqing-?）
- spec 阶段细化（用户发起 spec 技能后启动，写完不立即编码）
