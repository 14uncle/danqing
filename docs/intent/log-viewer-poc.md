# Intent: 第四件产品 —— 大文件日志/JSONL 查看分析器 (Windows)

> 2026-09-05 选型流程：五域并行扫描 → 用户点选 1+2 双深潜 → 用户裁决开枪 A。
> 新选型方针（需求驱动·调研先行）首次实战，见用户级记忆 `product-selection-demand-first`。

- @author 十四叔
- @date 2026/09/05
- 状态: **三模块全闭环**（core-viewer / jsonl-table / live-tail 均走完 spec → plan → build → review；2026-09-05 POC 双前提判过 → spec 批准 → plan 批准 → /build auto 零 commit 连跑 core-viewer T1–T7 → review 三处 Required 修复；2026-09-06 jsonl-table T1–T5 + live-tail T1–T5 全绿、性能三门槛达标、各 review 通过；用户裁决四项：地图+3 模块 spec / 行内子行嵌套展开 / 编码 UTF-8+UTF-16+GBK / v1 单二进制全功能）；余前提③ = 发布后首单外检

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

1. 真实 1GB+ 日志实测，搜索/滚动对 klogg 与 LogViewPlus 形成**可截图的碾压** —— ✅ **判过（2026-09-05，有修正）**：碾压仅对 LogViewPlus 成立；klogg 速度同档（mmap 同构），对其差异点转为「停更 4 年 + 无 JSONL」。详见文末「前提①判定」
2. JSONL 列化 demo 体验显著优于 daucloud 的 VS Code 扩展（独立窗口、秒开、不占编辑器） —— ✅ **判过（2026-09-05）**：用户实测体感「极快」；字段过滤 1GB/483 万行 235ms。详见文末「前提②判定」
3. 发布后首单外检

## 定价锚与渠道

- 个人买断 **$45**（正面贴 LogViewPlus，比它快+现代+JSONL）；企业 **$95/seat**；10 席 pack **$590**；不设订阅
- 渠道：GitHub 主（OSS 内核+付费层刀法：开源层对标 klogg 全功能抢「停更 4 年」接盘流量；付费层 = JSONL 列化/多文件时间戳合并/过滤器会话/导出）+ MS Store 辅（LogoRRR Pro 在架证明类目存在且稀薄）
- 冷启动：Show HN 打「klogg 四年未发版 + LogViewPlus $45 还卡」；r/sysadmin、r/devops；SO 十年老帖「JSON viewer to open large json files」与 LogViewPlus 性能事故帖下接客

## 候选队列（未选，归档备查）

- **B「Hazel for Windows」文件自动整理**：开枪（有条件）未选。空位 = $29-35 买断 + 现代 UI + undo 信任三件套（Hazel 仅单文件 revert、File Juggler 无 undo）；对手 File Juggler $50 活跃维护（前置扫描「定时扫描」说法已被深潜修正为事件驱动实时）；MS Store 可行（runFullTrust 打包，Sortly 过审先例）；开枪前提 = 信任三件套 P0 + ≤1 周 MSIX 打包尖刺 + 只做下载夹/桌面单场景。不选原因：在场质检弱于 A、引擎渲染肌肉闲置、首要风险是外部审核不可控。
- 枪毙归档：泛截图工具（两路独立验证红海）、AI 听写（空窗已关，三线挤压）、快速记录（入口被手机/微信掐死）、抓包（Fiddler 断供窗口诱人但团队赛道）、数字健康阻断器（潜力大但 MSIX 沙箱存在性风险未排除，护城河 70% 在 GUI 之外）。

## POC v0 实测（2026-09-05，建仓当日）

环境：本机（Iris Xe 核显），release 构建（lto=fat）；数据 = `genlog` 合成 1024.0 MiB（确定性，可复跑）。数字源 = `danqing-log --bin logbench`。

**1GB 明文日志（635 万行）**：
- mmap 建立 **134 µs**；行索引 **425 ms**（2406 MiB/s）
- 搜索 `ERROR`：**69 ms**（14.6 GiB/s）；`ERROR|FATAL` 106 ms；`user_42\d{4}` 91 ms；结构时间戳正则 635 万全命中 1051 ms（974 MiB/s）
- 随机访问 10 万行：**0.37 µs/行**
- GUI 双击到窗口可见 1.26 s（其中 ~1.1 s 是 wgpu 管线初始化，danqing 全家共有成本；文件本身 0.43 s）

**1GB JSONL（483 万行）**：索引 427 ms；`"level":"ERROR"` 82 ms；`"status":50[02]` 193 ms（74.3 万命中）。

**前提①判定（2026-09-05 人工三方对比，同文件同机，用户实测体感）**：

| 工具 | 打开 1GB | 滚动 | 搜索 | 备注 |
|------|---------|------|------|------|
| klogg 22.06 | **秒级**（用户体感，无精确读数；mmap 同构，速度不构成碾压点） | 流畅 | 快 | 截图留存：用户桌面 Snipaste_2026-09-05_19-35-42；行数 6,349,886 与我方索引一致（正确性旁证）；编码检出保守（报 ISO-8859-1） |
| LogViewPlus 3.2.9 | **慢**（全量解析入表格） | — | — | 用户观察：表格化展示是慢的结构原因（解析换结构化） |
| danqing-log POC | **极快**（0.43s） | 流畅 | 69ms（CLI 基准） | — |

**碾压主张修正**：对 **LogViewPlus 成立**（其结构化表格 = 打开速度的结构性代价，恰是我方「快+结构化」楔子的受力面）；对 **klogg 不成立**（速度平手，差异在停更 4 年 / 273 open issue / 无 JSONL / 无 ANSI 颜色）。楔子收敛为一句话：**「klogg 的速度 × LogViewPlus 的结构化」**——这使**前提②（JSONL 列化 demo 优于 VS Code 扩展）升为决定性验证**：速度已证，结构化待证。

**前提②判定（2026-09-05 落地 + 用户实测判过）**：

demo 内容：打开自动检测 JSONL（64 行采样 ≥90% object）→ 列发现（512 行采样，首见顺序，≤16 列）→ 表格模式四区（过滤栏/表头/虚拟化行/状态栏；行号槽恒显文件真实行号，level 列级别着色）→ 字段过滤（`level=ERROR status=50*`：空格分词 AND、尾缀 `*` 前缀通配、裸词整行子串；Enter 应用走工作线程不冻界面，Esc 清除，Ctrl+T 原始/表格互切）。

引擎数字（release，同机 1GB JSONL / 4,833,705 行，`logbench --filter`）：

| 查询 | 命中行 | 耗时 | 吞吐 |
|------|--------|------|------|
| `level=ERROR` | 43,464 | **235 ms** | 4346 MiB/s |
| `level=ERROR status=50*` | 6,742 | **234 ms** | 4364 MiB/s |

正确性旁证：过滤命中行数与 regex 全文搜 `ERROR` 完全一致（43,464），memmem 字段提取无漏。

实现要点：显示/过滤路径**零 JSON parse**（memmem 定位 `"key":` + 前缀 `{`/`,` 校验 + 值 token 切取）；serde_json 仅用于检测与列采样，且必须开 `preserve_order`（默认 Object 是 BTreeMap 字典序，「首见列序」会失真——单元测试当场抓住）；OnDemand 可见态 ~60fps tick 直接拾取工作线程结果（完成至显示 ≤16ms，原设计的 boost_frames 唤醒整段删除）。

**用户判定：体感「极快」，「显著优于 daucloud VS Code 扩展」判过**（独立窗口、双击秒开 1GB、不占编辑器。扩展侧未同机复测——逐帧对比留待发布素材阶段）。

demo 边界（正式版必解）：仅扁平顶层字段，嵌套展开未做（= MVP⑤ 正式版内容）；字符串值内含 `,"key":"` 形态可能误判（memmem 提取已知边界，正式版换真 parser）；表格无水平滚动/列重排/命中高亮。

（底栏状态行即截图本体；klogg/LogViewPlus 对比截图用户已人工过目，未留档量级数字——重测随时可开，三方二进制均在 `D:\app`。）

**POC 已知边界（正式版必解）**：编码 UTF-8/UTF-16(转码副本)/GBK(行级 CP936)/Latin-1 兜底（T2 落地）；~~mmap 期间外部截断会崩~~ **2026-09-05 T3 实测修正：Windows 上 OS 直接拒绝截断被映射的文件（ERROR_USER_MAPPED_FILE），Linux SIGBUS 假设不成立**；真正的过期通道是 rename/delete/append/overwrite（视图滞留或内容被换），防御 = FileStat 快照 + 轮询 + 重建换入（实测表见 `danqing-log/tasks/plan.md` 附录）；行偏移已改步进索引（T1，驻留 3.03MiB/GB）；GUI 有字段过滤（前提②）与正则搜索框/书签/水平滚动（T5–T7），暂无 tail（live-tail 模块）。

**引擎缺口记录**（打磨寄生，修进 danqing 时不在这里绕）：~~NamedKey 缺 PageUp/PageDown~~（T4 已修进 danqing）；~~MouseWheel 无修饰键~~（T7 已修进 danqing：shift/ctrl/alt）；~~无焦点应用 IME 被关~~（review 阶段：`update_ime` 无焦点即 `set_ime_allowed(false)`，中文录不进 → 加 `App::wants_ime()` 钩子，默认 false 零波及）；~~App 层无剪贴板直连~~（review 阶段：加 `WindowEventSender::read_clipboard()` 回送 IME Commit 供粘贴）；等宽字体选择（日志场景心智，未动）。

## 悬而未决

- **LOGO 落地（2026-09-06）**：`assets/logo/log.svg`（设计源，家族语法：玉色 #0F766E 视窗框 + 玻璃白内填 + 四条日志行，**底部一条朱砂 #E34234 = live tail 正在写入的那一行**，区别于引擎破框/番茄钟轴心/剪贴板首行）+ `tools/export-logo.py`（Pillow 手工几何 + 4x 超采样，同三兄弟工艺）+ 已导出 `log_{16,24,32,48,128,256}.png` + `logo.ico`（build.rs 用，五帧）。运行时窗口/托盘读 `log_256/16.png`，exe 资源内嵌 `logo.ico`。**顺带修了重复资源**：danqing 库 build.rs 原默认 embed 自己的 logo.ico，与产品各自 embed 撞出 `.rsrc merge failure: GROUP_ICON/ICON/VERSION`（exe 图标二选一未定，属发布前必修）→ 库 build.rs 改为不再 embed（产品各自 embed，零副作用），danqing-log 重建零警告。两处改动均**未 commit**（跨仓：danqing build.rs + danqing-log assets/tools）。
- 待定：in-app 标题栏 logo（danqing `title_bar.rs` `LogoKind` 只有 Default 破框/Pomodoro/Clipboard，danqing-log 未设；任务栏是日志 logo、标题栏仍是丹青破框，视觉不一致——需给 danqing 加一个 Log logo_kind 或产品自绘）。
- **~~v1 任务：过滤/搜索栏重构成真 TextInput~~** **已闭环（2026-09-06）**：`Bar` widget 托管真 `TextInput`（焦点路径落容器、转发 wants_ime/ime_area/selected_text/hit_area/reset_focus，参考 danqing-clipboard bottom_bar）；`App::view → Column[Bar, LogView]`；键盘路由进焦点系统（方案 A：默认无焦点→app 导航，栏聚焦→方向键移光标，进表格/开搜索自动聚焦）；三个 IME 补丁（App::event IME 分支 + read_clipboard 粘贴 + wants_ime）已删；IME 候选窗帖光标（用户实测过）。配套 danqing 打磨寄生：TextInput 加 `caret_color`/`selection_color` setter（深色栏光标可辨）。三件套绿 + GUI 人工验收通过。
- ~~live-tail / jsonl-table 模块~~（2026-09-06 已各自走完 plan/build/review，三模块全闭环）
- 产品命名与仓库名（danqing-log = 工作名，公开发布前可改）
