# 第四轮选型扫描 · 工作文档（2026-09-16 开局）

> 本文档是进行中会话的可重启边界：范围、决策、进度、待办全部在此。
> 新会话续接时先读本文件 + farm01 CLAUDE.md「产品心法」+ 记忆 `product-selection-demand-first` / `product-selection-seam-criterion` / `next-product-selection-round`。

## 任务定义（保护项，勿丢）

用户指令（2026-09-16）：danqing-log v1.0 编码完成，自用 2 周后发布；**拿上产品选型指南，下个产品开工**。

**边界决策**：本轮只做扫描与调研，产出**候选池而非选定**；开枪裁决归用户本人。原决策「log 发布前不启动下一轮选型」（2026-09-15）被用户当前指令显式覆盖，但保留其精神——推理不许跑在证据前面，开枪前重查在位者状态（带证据日期）。

## 选型闸门全集（稳定约束，压缩版）

**需求侧（判真伪）**
- 痛点门槛：≥3 独立来源 + 近两年内 + ≥1 处行为证据（workaround/邻近消费/迁移，权重 > 言词），缺一挂科
- 每条候选带失效标志（含「需求衰减型」检查：看 issue/帖子**创建日期分布**，不看总票数）

**供给侧（判生死，递进且各持一票否决）**
1. 渠道可达：MS Store 搜索词 / GitHub / 内容平台自然流量，至少一条够得着痛点人群
2. 在场质检：能看懂该领域用户的抱怨、判断方案好坏；读不懂不做
3. 缝隙存在：
   - 在位者在不在动（睡着守得住/月更死缓/**免费在位者睡不睡都封死**）
   - 在位者结构上能否跟进（「从未做」≠「承诺不做」，禁区位必须查证不许推理认定）
   - **盘点必须穷尽**（tile 死因：只盯 FancyZones 漏了 Seelen UI）
   - 「停更/月更/免费」断言必须查证 + 标证据日期
- **开枪前自检：「有没有人（任何人）免费做这件事」**

**死刑条款（直接挂，不进候选池）**
- 能写成「X，但更好/更快/更美/更便宜」的方向
- 免费方案已够好的方向（clipboard 死因）
- AI 能力为核心卖点且无额外强理由（Copilot 付费转化 3.3% 起点）

**硬数据锚**：硬件在涨（NAND +300%，「省钱」故事会翻车）；模型质量/成本比在升；Copilot 转化 3.3%；GitHub 免费发布 ≈ 零自然流量（本轮实测，见下）。

**形状参考（非锁）**：已验证形状 = 在位者是睡着的老软件 + UI 品质是购买理由（log 对 LogViewPlus）；但 disk/tile 照此形状都死了——形状可用，状态断言必须查证。

**已排除方向（勿 resurfacing，除非附着 2025-26 新扰动事件）**：番茄钟、剪贴板历史、大文件日志查看器、磁盘空间分析、自动平铺窗口管理、文本扩展、内容搜索、表格/CSV 手术刀、步骤记录器、重复文件清理、本地照片搜索、Screen-Studio-for-Windows（缝已闭合）。

**幸存曲线（09-15 第三轮认证）**：云订阅涨价 / 买断→订阅行业转轨 / 系统级索引信任赤字（Recall）/ Win10 长尾（衰减中，12-24 月窗口）。
**弱存活候选（降权带入）**：P2 本机 AI 资产库（付费意愿最弱 + 与已埋 danqing-disk 邻接撞车风险）。

## 已完成

### 失效标志复盘（开枪前必扫）
| 标志 | 状态 |
|---|---|
| log 前提③首单外检 | 待兑现——发布后才能判，前提是埋点落地 |
| klogg「停更 4 年」 | 〔未查证〕→ 已派扫描 agent 顺带查证 |
| P1 本机私有数据层 | 已证伪归档（09-15，Build 2026 证据一小时内打掉） |
| P2 本机 AI 资产库 | 存活但弱，降权带入 |
| disk/tile 缝假设 | 09-15 已判证伪归档 |

### 自家第一方数据（pomodoro GitHub，2026-09-16 拉取）
- Stars 0；14 天浏览 20 次/2 unique；14 天克隆 179/78（爬虫噪音）；发布下载量合计 ~6（v0.1.0:2, v0.2.0:1, v0.2.8:3）
- 来源：github.com×8、apps.microsoft.com×1、掘金×1
- **校准结论**：GitHub 免费发布 ≈ 零自然流量。「发了就有人来」不成立 → 渠道可达闸权重升至最高；算账不许用「GitHub 自然星」当流量假设；log 发布 checklist 的商店 impressions 埋点 = 下一枪的数据来源，不做就没有数据。

## 进行中：扰动事件扫描（4 agent 并行，2026-09-16 发出）

| # | 扫描区 | 附带任务 |
|---|---|---|
| 1 | 弃用/停更/被收购（2025-01 至 2026-09） | 顺带查证 klogg 维护状态（关〔未查证〕项） |
| 2 | 涨价/转订阅/账号墙/广告化（中英文圈） | 找「没人用本地+买断接住」的出逃潮 |
| 3 | Windows 平台扰动（Win10 EOL/内置应用 enshittification/Recall/MS Store 政策/PowerToys 新模块/平台技术） | PowerToys+Seelen UI 在做什么必须查清 |
| 4 | 2026 逃亡信号 + 沉默需求信号（Reddit/HN/V2EX/知乎/小众软件） | 每条过 ≥3 来源+行为证据门槛，带失效标志 |

**报告摄取约定**：agent 报告回来后只提取「过闸结论 + 证据 URL 指针」进本文档，原始报告不整存（防上下文淹没）。

## 扫描结果 · 压缩摄取

### ③ 平台扰动（已完成 2026-09-16）

**结论一：Win10 长尾 = 受众定位资产，不是产品机会。** 留守 ~30% 全球存量（StatCounter 2026-08，注意其 2026-06 分类出过错），但即时软件缺口全被免费覆盖（ESU 免费至 2027-10 / 0patch 付费在位 / LTSC 转换教程 / 浏览器续命至 2028）。行动项：下一枪任何产品把 Win10 22H2 列一等支持平台；营销话术对「拒绝账号墙/遥测/订阅」人群讲定位。**此条对 log 发布同样适用。**

**结论二：「微软制造出逃」曲线获官方背书，但浅层痛点窗口在收窄。** 2026-03-20 微软「质量重置」（承认 Win11 gone off track，Copilot 撤出 Notepad/Photos/Snipping Tool、可移动任务栏回归）——痛点被官方盖章为真，但**制造者正在拆痛点**；「干净内建应用替代品」全死（Notepad++/Notepads 10.3k★/Open-Shell 9.3k★/ExplorerPatcher 33.9k★/Files 45.5k★ 全免费活跃）。没被拆的是「生态方向不可信」：GDID 追踪争议（2026-07）、MSA 每 60 天强登录、强制联网装机仍在。农场定位押后者，本轮无修正证据。

**结论三（唯一活缝，存疑带毒）：Recall 信任赤字 → 消费级「可验证无上传的本地屏幕记忆」。** 行为铁证：Windrecorder 3.9k★/OpenRecall 2.9k★ 零营销自然增长（但**双双停更整一年**，证据日期 2026-09-16）、Brave v1.81+ 默认屏蔽 Recall、微软三次撤退（默认开→opt-in→企业默认移除）、2026-04 TotalRecall Reloaded 微软定性「不修」。在位者：**screenpipe 21.6k★ 今日仍 push**，开发者导向（插件商店/API）——缝 = 消费级打磨窄缝。**自带毒点**：①与已枪毙的「内容搜索/本地照片搜索」家族相似；②微软自己是最大免费在位者（虽 opt-in 虽被 Brave 屏蔽）；③「为什么两个开源消费级竞品死了」回答不了就不许开枪。预设失效标志：screenpipe 推一键消费版起量，或 Recall 口碑翻身（敏感过滤实测全过），即判死。

**结论四（唯一直接利好）：MS Store 渠道经济性变好。** 个人开发者上架费 $19→0、免信用卡（2025-06 生效，2025-09 扩 200+ 市场）；**CID 直链引流 95/5 分成**；Win32 商店内更新。→ 买断制 + 内容平台直链引流打法分成 95%。

**结论五（雷区更新）：PowerToys 月更未断**（v0.90 2025-03 → v0.101 2026-08 + 周 preview），已吃：启动器（Command Palette 2025-03 + 扩展商店）、自动明暗（Light Switch 2025-10）、演示缩放（ZoomIt 2025-01）、同应用窗口切换（Window Hopper 2026-08）。Seelen UI 17.8k★ 今日仍 push。**启动器/桌面外壳/平铺/明暗切换周边全是雷区。**

**死胡同（13 条，全带证据日期，详见原始报告）**：干净记事本/WordPad 替代、debloat 隐私开关（winutil 62.7k★ 免费）、本地账户绕过、ESU 登记、启动器、明暗切换、开始菜单修复、资源管理器替代、演示标注、Publisher/.pub（Affinity 全免费）、离线邮件、「X 但更好」全家。

**其他记录**：WSA 坟场确认（2025-03 下架，平台子系统寄生生态=坟场）；新 Outlook 强制迁移灭 COM/VBA 插件生态（B2B，PST 转换已拥挤）；WordPad 删除后 Win11 无默认 RTF 阅读器（免费在位者已接住）。

### ① 弃用/停更/收购（已完成 2026-09-16）

**头条结论（战略假设打补丁）：窗口内真事件零生存。** 2025-26 消费级 Windows 桌面工具的每次离场（Pocket 2000 万用户关停/Skype 关停/Buttercup 关闭/Input Leap/DS4Windows 第三棒/Cakewalk 转订阅/TechSmith 转订阅/Fleet 停更/Evernote 重组/ModernFlyouts 等 ~19 件查实事件），**全部在数月内被免费方案接住，一例漏接都没找到**。免费继承者出现得越来越快（FluentFlyout/Deskflow/STranslate/Freelens…）。「工具死了」本身已不构成缝。

**反向事件（更狠的封锁）**：Canva 把 Affinity 全套改**永久免费**（2025-10-30）——「平价设计工具」方向枪毙。

**klogg 〔未查证〕核销（证据日期 2026-09-16）**：未 archive；3.5k★；最后 release v22.06 2022-06-13（**停 4.3 年**）；最后 commit 2024-11-26（**实质停 22 个月**）。「klogg 停更 4 年」发版口径成立 → LogLens「免费在位者停滞」前提坐实。

**唯一值得深挖的擦边事件：Pot 划词翻译 archive（19.4k★，2026-07 后归档，原因〔未查证〕，449 open issues 悬置）。** 按现行规则已作废（STranslate 8k★ 两周前仍发版，免费对位接住）。唯一深挖理由：**该品类连环死亡**（QTranslate 停 → CopyTranslator 半停 → Crow archived 2024-07 → Pot archived 2026）——四家免费无一存活，死因 = 结构性（云 API 代理维护负担 + 零变现），而「免费反复死」可能恰是买断制唯一能活的生态位。Kill criteria（任一即毙）：①STranslate 对齐度+迁移帖证明难民接全；②2026-07 后难民量级不足；③无付费意愿证据；④差异化只剩「本地 LLM 翻译」（撞 3.3% 警告）。

**方法论产出（下轮扫描换重心）**：与 LogLens 同构的缝不是「谁死了」，而是「**谁还在喘气但 2-4 年没发版、无活跃 fork、其余在位者全付费**」（klogg 正是此型）。建议下一轮做「僵尸在位者普查」：按品类列 2 年以上无 release 的高星工具 → 查活跃 fork/免费替代，信噪比比追新闻事件高一个数量级。

**被证伪的预想存档（勿再猜，gh 实证 2026-09-16 全活跃）**：marktext/Fluent Reader/Open-Shell/ScreenToGif/Greenshot/Wox/SumatraPDF/ExplorerPatcher/Windhawk/TrafficMonitor/BleachBit/mRemoteNG/Cmder/Ferdium 等 18 个。

**Watch 项**：微软 Whiteboard 独立 App 2026-09-25 起退役——draw.io desktop 63.1k★ 已免费对位，仅观察 9-25 后有无「draw.io 不好用」行为证据，无则忘。

### ② 涨价/转订阅/账号墙（已完成 2026-09-16）

**头条结论：14 件实锤事件，零候选。** Plex 涨 40%→Jellyfin $0 完全接住；M365 涨 43%→LibreOffice/WPS/微软自家 LTSC 买断三路接；Pocket 关停→Raindrop/Instapaper（专门做导入工具）/Wallabag/Karakeep 接住（且人群在移动端，载体不匹配）；ToDesk 三连砍→**网易 UU 远程大厂免费补贴**+RustDesk 自建；Snagit 转订阅→ShareX；Notion AI 强绑→**Obsidian 2025-02 起商用免费（反向事件）**；Todoist 涨→TickTick 更便宜；GitKraken 涨→Sourcetree 免费+**Fork $59 买断位已占**；剪映 SVIP→DaVinci/必剪免费；PotPlayer 广告→VLC/mpv 密度封顶；Win11 系统广告→ShutUp10++/Windhawk 免费满编。**「X，但买断」空位在这批事件路径上不存在；免费 catcher 反应速度比想象中快（Jellyfin 迁移工具链一个月成型）。**

**唯一「免费 catcher 睡着」案例：PureRef 细分（参考图板）——但已死缓。** BeeRef 停更 2024-06（睡着），但 ①refern $35 买断 2026-06 已进场（在位者在动）②PureRef 1.x 合法免费可商用，难民被压缩到极小一撮。**衍生价值不是开枪而是校准：refern 是「单人+本地+买断」在美术细分的天然对照实验，查证其销量 = 校准农场核心假说。**

**WSA 缝真实但工程量超限**（AOSP+Hyper-V+Play 认证），死因=工程量不是没缝，存档。

**方法论产出（与 agent① 独立收敛）**：扰动事件扫描作为候选产生器**连续两轮产出率为零**，建议降级为「曲线监测」（每季度确认趋势仍在），不再当候选挖掘工具。

## 累计死池（勿 resurfacing，全部带证据日期，详见各 agent 原始报告）

- **本轮新增**：媒体服务器（Jellyfin）/ 办公套件（LibreOffice+WPS+LTSC）/ 稍后读云形态（Raindrop+Instapaper）/ 远程协助（UU 远程+RustDesk）/ 截图贴图标注（ShareX+PixPin+系统自带）/ 录屏 GIF（OBS+ScreenToGif）/ 笔记知识库（Obsidian 商用免费+Joplin+Logseq+SiYuan）/ 待办（TickTick+MS To Do）/ Git GUI（Sourcetree+Fork 买断位）/ Windows 安卓运行环境（工程量）/ 视频播放器（VLC/mpv/MPC-HC）/ 系统去广告增强（ShutUp10+++Windhawk+ExplorerPatcher）/ 视频剪辑（DaVinci+必剪）/ 图片查看器（ImageGlass/qView 满编）/ 素材管理（Billfish 永久免费）/ 密码管理（KeePassXC+Bitwarden）/ KVM 切换（Deskflow）/ 手柄映射（Steam Input）/ DAW（Reaper+LMMS）/ 划词翻译（STranslate 8k★ 活跃，Pot 事件已被接住）/ 白板（draw.io desktop 63.1k★）/ 设计套件（Affinity 全免费）/ 备份（Hasleo）/ 邮件（Thunderbird）/ 音量悬浮（FluentFlyout）/ 数据库客户端（DBeaver 51.8k★）/ 启动器（CmdPal 第一方+Flow Launcher 15.6k★）/ 干净记事本（Notepad++/Notepads 10.3k★）/ RTF 阅读（LibreOffice+RectifyPad）/ 开始菜单任务栏修复（Open-Shell+ExplorerPatcher+StartAllBack 在位+微软自拆痛点）/ 资源管理器替代（Files 45.5k★）/ 明暗切换（Light Switch 第一方+ADM 9.7k★ 活跃）/ 演示缩放（ZoomIt 入 PowerToys）/ ESU 登记与装机绕过（免费满编）/ Win10 续命工具（ESU 免费+0patch 在位）/ Publisher/.pub（Affinity+Scribus）/ 通讯（Teams/Zoom）/ IRC（盘太小）
- **往轮已在档**：番茄钟、剪贴板历史、日志查看器（自家）、磁盘分析、平铺窗口管理、文本扩展、内容搜索、表格手术刀、步骤记录器、重复文件清理、本地照片搜索、Screen-Studio-for-Windows

### ④ 逃亡/沉默需求信号（已完成 2026-09-16）

**合格信号：0 条。** 20 组实证搜索，五起真实出逃潮（Snagit/CapCut/Cakewalk/Publisher/Rewind）全部被免费红毯接住。元发现与①②同构，补一条前置检查：**「先看逃亡终点站着谁」**。
（局限声明：WebSearch 对 Reddit 直查被 SEO 污染两轮，Reddit 侧证据偏弱；52pojie/HN/知乎本轮无产出。）

**差一口气 1：Rewind/Limitless 难民潮 → 与③的 Recall 缝双侧收敛。** 触发事件新且硬：2025-12-05 Meta 收购 Limitless 当日停售 Pendant；2025-12-19 Rewind Mac 录制功能永久关闭；多区域停服、导出窗口仅两周。付费铁证：用户付过 $99-199 硬件 + $19-29/月；竞品 Hedy 推「难民免费 Pro」抢人（迁移被商家验证）。卡点：screenpipe/LUCI/Recall 三家免费在位；缝 =「免费但难用」窄缝（screenpipe 自托管门槛、LUCI 维护不明、Recall 锁 Copilot+ 硬件），叠加 AI 3.3% 起点 + 隐私工程重投入。失效标志：screenpipe 一键桌面版起量 / Recall 下放非 Copilot+（Studio Effects 已在下放，趋势明确）/ r/RewindAI 帖量衰减。

**差一口气 2：iPhone↔Windows 管理（Apple Devices 应用持续烂尾）。** 人群最大、付费铁证硬（iMazing 活着=证明，用户忍 3uTools 捆绑也要用）。卡点：**3uTools 免费功能全** + iMazing 付费在位 + Apple 协议栈逆向深水区（在场质检不合格）+ Apple 随时可能修好。唯一窄角度：只做单一高频动作（iPhone 照片/备份到 PC 指定文件夹）且先验证 **MTP 直读够用**（够用=不碰私有协议，质检闸才过）。

**差一口气 3：全局麦克风静音可见性。** MuteMe 物理按钮 $39-59 有人买（行为证据奇特但真实），但无 2025-26 新触发 + Win11 内置在改善 + PowerToys 试过又撤。死缓偏死。

**死胡同 20 条**（全部实证，含：Sticky Notes 并入 OneNote 被 Stickies 接 / HP Smart 强登录被 NAPS2 接 / Krisp 被 Voice Focus 内置接 / Photos 变慢被 IrfanView 接 / 窗口布局记忆被 **PowerToys Workspaces 2024-09** 接 / Authy Desktop 被 Ente Auth 接 / Postman 被 Bruno 接 / 离线听写被 Handy、Buzzer 接 / Proxifier 被 ProxyBridge 接 / Fences 被 Nimi Places 接…）。全部并入上方累计死池。

## 合成（2026-09-16，四路扫描齐全）

**1. 方法论结论（本轮最大产出，三个 agent 独立收敛）**：扰动事件扫描（死亡/涨价/平台/逃亡四轴）作为候选产生器**连续两轮产出率为零**——免费替代生态反应速度以周计，「在位者离场→宽缝」假设在 2025-26 消费级 Windows 桌面全域不成立。处置：①扰动扫描降级为**季度曲线监测**；②候选产生器换 **僵尸在位者普查**（2-4 年无 release + 无活跃 fork + 其余在位者全付费，klogg 即此型）；③任何逃亡深挖前先问「终点站着谁」。

**2. 候选池（过闸后，全部带毒/带条件，无干净候选）**：

| # | 候选 | 收敛情况 | 毒点 | 下一步（若用户选） |
|---|---|---|---|---|
| C1 | 消费级本地屏幕记忆（一键安装/买断/可验证无上传） | ③④双侧收敛（平台信任赤字 × Rewind 难民付费铁证） | screenpipe 21.6k★ 今日活跃 + LUCI 免费 + Recall 免费锁硬件；「免费但难用」窄缝；AI 3.3% 起点；隐私工程重投入；两个消费级 OSS 已停更（需求真但单人经济死？两种读法） | **在位者质检**（一天级）：装 screenpipe/LUCI/Recall 实测，量化「免费但难用」程度；读两个死亡 OSS 的 issues 找死因 → 再裁决 |
| C2 | 划词翻译「不死买断版」（Pot 事件） | ①单源 | STranslate 8k★ 活跃免费对位（规则上已死）；品类连环死亡=结构性维护负担；差异化只剩本地 LLM 翻译（撞 3.3%） | 按 4 条 kill criteria 走深度调研轮 |
| C3 | iPhone→PC 单一高频动作（MTP 窄角度） | ④单源 | 3uTools 免费+iMazing 付费在位；MTP 够不够用未验证；Apple 随时修好 | 先验证 MTP 直读路径能力边界（技术验证，半天级） |

**3. 非候选产出（不管选不选都落袋）**：
- MS Store 经济性：个人上架 $0 + CID 直链 95/5 → **内容平台直链引流 = 主渠道打法**（log 发布即可用）
- Win10 22H2 列一等支持平台 + 对「拒绝账号墙/遥测/订阅」人群讲定位（log 话术立即可用）
- refern（$35 买断，PureRef 细分，2026-06 进场）销量查证 = 买断假说校准实验
- klogg 停滞坐实（LogLens 前提）+ 累计死池 30+ 品类（防未来重复挖掘）
- 免费在位者在变强：Obsidian 商用免费 / Affinity 全免费 / UU 远程大厂补贴 / PowerToys 月更——**「免费例外封死」清单只会变长**

**4. 诚实总评**：本轮开工时指南要求的「候选池」已产出，但**没有一条达到「建议开枪」标准**。最强项 C1 的下一步是花一天做在位者质检而非写代码；最弱项 C3 大概率卡在质检闸。「本轮不开火、采纳僵尸普查、等 log 数据」是合法且可能最优的裁决——组合纪律与「数据还没出生」的原始理由并未被本轮扫描推翻。

## C1 在位者质检（用户裁决 2026-09-16：只做 C1）

**本机实测部分（用户裁决：不继续安装实测，纯凭调研报告裁）**：
- 分发摩擦（消费者视角）：GitHub releases **无安装包**（app-v2.7.34，2026-09-15；近期 8 个 release 全空资产）；winget 查无此包；官网下载 JS 门控无直链（真实直链藏在 /api/download 307 → R2）。装机摩擦：本机 NSIS /S 静默安装**静默失败**（3 分钟后退出、零文件、零报错——非消费者流程，仅存档）；安装包 221MB、Mediar Inc EV 签名（无 SmartScreen 障碍）
- 定位铁证（README 2026-09-16 实测）：**YC S26 在孵**；自我定位 =「为你的 agents（Claude/Codex/Openclaw…）提供上下文」+ MCP server + CLI（npx，需 Node）+「company brain」B2B 话术；docs 含 Intune/Entra 企业部署——**「agents 的上下文基础设施」≠ Rewind 式消费级「你的人生可搜索」**，不是同一个产品
- Recall 本机不可测：**本机无 NPU ≠ Copilot+**（"Input" 误匹配已排除）——Recall 锁硬件在本机直接出局，文档侧结论由扫描③存档
- 发布节奏：8 个 release / 2 周，极度活跃

**尸检部分（agent 撞配额中断，主会话续跑完成 2026-09-16）**：

*Windrecorder（3,939★，中文作者）死因*：最后 commit 2025-07-20（feat: AI 自然语言搜索），最后 push 2025-09-16，停更整一年。**关键证据 = issue #294（2026-01-25，已关闭）「Related Project: Screenpipe - Cross-platform alternative」——社区自己把用户指向了 screenpipe**；#296（2026-03-05）有人在问 fork 事宜。Top issues 主题：托盘卡顿（#157，14 评论，仍 open）/ 系统声音录制（#187，13 评论，仍 open）/ 中文 OCR 准确率低（#18）——**高频诉求长期挂着没人做**。

*OpenRecall（2,942★）死因*：**安装地狱**。#19「Unable to install on Windows」（2024-06，13 评论，仍 open）、#22「One-click installer for Windows & macOS」（2024-06，仍 open）、#104「Can't Install On Windows, needs torch 2.6.0 & torchvision 2.2.0」（2025-06）、#95 torch 依赖冲突、#107「跑几小时没截图」。最后 push 2025-09-24。**#121「Is the project alive?」（2025-12-12）→ 维护者 koenvaneijk 2026-02-25 回「Project is still alive. We are working on a big update!」——六周后用户追问时间线，至今无更新，仓库也无提交**。承诺未兑现型僵尸。

*死因归纳*：**两者都死于「重 ML 技术栈的维护负担 + 零收入 + 单人/小团队」**——与农场处境同构，且它们要扛的技术栈比农场现有肌肉更重。它们的 issues 就是缝的地图（一键安装、系统声音、多显示器、资源占用、OCR 质量），但缝的另一侧站着资源充足得多的对手。

*screenpipe 消费级成熟度（失效标志检验）*：**21,589★ / 2,198 forks / 今日仍在 push / 46 open issues（关得极凶）**。分发：官网有 Windows/Windows-ARM/macOS/Linux 下载按钮（221MB NSIS 安装包，Mediar Inc EV 签名），但 GitHub releases 无资产、winget 无包。**许可（2026-06-09 变更，载荷性事实）**：Screenpipe Commercial License（Negentropy Labs Inc.）——**个人/非营利/教育/研究免费，商用需付费**；定价 Basic $21/月、Business $42/座/月。定位：**YC S26 在孵**，官方语「为你的 agents 提供上下文」+ MCP + CLI + company brain，docs 含 Intune/Entra 企业部署。
第三方一致评价（2026）：**「为 agents 的上下文基础设施」「a toolkit more than a finished assistant」「not for anyone who wants a small app they never think about」「getting friendlier, but developer-oriented at core」**。

*LUCI（memories.ai）*：发布仓 OpenInterYRZ/luci-electron-release（2026-05-28 建），**v1.0.27（2026-09-08）近 5 版下载 1,155/1,486/2,024/334/144**——有真实消费级下载量且持续发版，**不是纸面存在**。

**关键交叉证据（本轮最重要的一句）**：多个第三方评测一致指出——**「没有任何单一工具复刻了 Rewind 的体验」：本地=技术流（screenpipe），易用=云（Littlebird/Jarvis）**。「本地 + 零配置 + 买断」这一格**确实是空的**。

### C1 生死裁决（建议：判死）

**这一格空着，不是因为没人想做，是因为难做。** 三条理由叠加，任一条即挂：

1. **闸③ 缝隙——挂（明文规则）**。在位者 screenpipe **每 2 天一个 release、今日仍 push、YC 在孵、有桌面 app**（远超「月更=死缓」）；且**个人版免费** → 撞农场明文规则「**免费在位者是例外，睡不睡都封死**」。穷尽盘点：screenpipe（免费个人）/ Recall（免费，锁硬件）/ LUCI（免费+付费，活跃）/ Littlebird、Jarvis（云，免费层）/ Windrecorder、OpenRecall（死）——已穷尽，**「有没有人免费做这件事」答案为「有，多家」**。缝的保质期按月计，而 screenpipe 正从「developer-oriented」向消费级移动。
2. **实现约束——挂**。农场明文：产品必须长在 danqing 上，**引擎复用是实现约束**（CLAUDE.md）。屏幕记忆 = 屏录 + OCR + 向量检索 + 存储 + 脱敏，**90% 是 ML/后端、10% 是 UI**——农场肌肉（Rust 自研 UI 引擎）对这个域匹配度低，这恰恰是 Windrecorder/OpenRecall 熬死的那个技术栈。
3. **AI 核心卖点——挂**。候选卖点本质是 AI 能力（OCR 质量 + 语义检索）→ 直接命中「Copilot 付费转化 3.3%」起点警告。

**反方最强论点（记录在案，供复盘）**：「Rewind 复刻品确无人在做」+ Rewind 用户付过 $99-199 硬件 + $19-29/月（付费意愿铁证）+ 两个消费级 OSS 死在「没人做」而非「没人要」。**若 screenpipe 在 12 个月内完成消费化，此缝即闭合；若它转向 B2B（YC + company brain 话术指向此），缝会重新打开**——这是本候选的复活条件，留档。

**本轮最终账目**：四路扫描 + 一轮在位者质检，**候选池三条（C1/C2/C3）全部指向不开枪**。这是连续第三轮零候选（09-15 六域 60+ 枪毙 → 本轮四轴 0 → C1 质检判死）。**瓶颈不在产能，在候选产生器本身**——这一点必须在下一轮方法论修订中处理。

## 用户裁决（2026-09-16，本轮终结）

**「转分发，暂停挖新品」**——本轮选型到此关闭，不再开新候选。理由（主会话提出，用户采纳）：农场七件产品的渠道实测是零（pomodoro 五周 GitHub = 0★/20 浏览/~6 下载），**瓶颈不在「找不到下一个产品」，在「没人看见已建成的产品」**；再挖第八个产品的期望回报低于把 log 发布这一个动作做扎实。

**遗留资产（下次开选型时直接取用）**：
- 候选复活条件：C1 ← screenpipe 转 B2B 或 12 月内未消费化；C2 ← 按 4 条 kill criteria；C3 ← 先验 MTP 能力边界
- 方法论：扰动扫描降级为季度曲线监测；候选产生器改用**僵尸在位者普查**
- 渠道打法：MS Store 个人上架 $0 + CID 直链 95/5 分成 → 内容平台直链引流为主渠道；Win10 22H2 列一等支持 + 对「拒绝账号墙/遥测/订阅」人群讲定位
- 累计死池 30+ 品类（本文档上方）

**下一步工作移交**：分发（见 `danqing-log` 发布准备 + pomodoro 商店状态核实）。

## 待办（按序）

1. 收齐 4 份扫描报告 → 统一过闸 → 候选池写入本文档
2. 候选池呈用户裁方向（继续/放弃/转向）
3. 选定方向后：深度调研（竞品 open issues 逐条扒/需求验证/定价调研）→ intent 文档（含**裁决段**）→ 红队 → 一页粗算（渠道流量×转化×单价 vs 工时）
4. 开枪前：重查在位者状态（带日期）+「有没有人免费做」+ 组合纪律确认（log 应已发布）
5. log 发布 checklist：领先指标埋点（商店 impressions/GitHub traffic/下载量）

## 开放问题（等用户）

- pomodoro 的 Partner Center 状态：商店页是否已上架？apps.microsoft.com 来源×1 是否你自己点的预览？若已上架，30 天校准钟可立即开跑，不必等 log。
