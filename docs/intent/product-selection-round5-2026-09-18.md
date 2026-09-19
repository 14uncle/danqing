# 第五轮选型扫描 · 工作文档（2026-09-18 开局）

> 本文档是进行中会话的可重启边界：范围、决策、进度、待办全部在此。
> 新会话续接时先读本文件 + round4 文档（`product-selection-round4-2026-09-16.md`，含闸门全集与累计死池）+ farm01 CLAUDE.md「产品心法」+ 记忆 `product-selection-demand-first` / `product-selection-seam-criterion` / `next-product-selection-round`。

## 任务定义（保护项，勿丢）

用户指令（2026-09-18）：danqing-log GitHub release + 微软商店均已发布；**拿上产品选型指南踏上征程**。

发布事实核验：GitHub `14uncle/danqing-log` release `v1.0.0`（丹青日志 LogLens）published 2026-09-18T09:08:38Z（gh 实证）。MS Store 上架为用户口述，CLI 不可核验。**30 天校准钟起算 2026-09-18，2026-10-18 前后回填 CLAUDE.md「及格线校准机制」**（商店 impressions/转化/下载量 + GitHub traffic）。

interview-me 确认（2026-09-18，用户显式 yes）：

- **Outcome**：候选短名单（每条：痛点假说 + 失效标志 +「有没有人免费做」预判）供用户勾选方向
- **时点**：现在就扫，不等 30 天校准数据（数据落地后回填算账基准，喂的是裁决阶段的调研三闸③）
- **方向来源**：agent 先提、用户勾（市场调研 agent 执行、方向用户给）
- **边界**：**先不选**——本轮只扫不定；不写代码/不建仓/不写 spec；深度调研是选定后 spec 前那轮
- 全部否决是合法产出（round3/round4 连续两轮零产出先例）

**方法论修正（同日）**：采访框架原拟「扰动事件清单」；读 round4 文档后确认该产生器已连续两轮零产出、已降级季度曲线监测（09-16 刚扫过，本轮不重跑）。用户重裁选定**僵尸在位者普查**（round4 方法论修订指定的接班产生器）。

## 产生器：僵尸在位者普查（klogg 型缝）五条件

缺一不可：

1. 品类内有免费开源工具 ≥1k★，**实质停更 ≥2 年**（最后 release ≤ 2024-09；分别记录最后 release 版本+日期、最后 push 日期）
2. 无活跃 fork 接班
3. **其余在位者全付费**（付费在位者 = 付费意愿铁证，记录定价页 URL + 价格 + 查证日期）或缺席
4. 僵尸仓库 issues 有**近两年（2024-09 后）新创建诉求**——看创建日期分布不看总票数（需求衰减检查）
5. **无活跃免费 catcher**——任何活跃免费方案已接住该品类 = 封死（第一杀手，免费例外）

形状原型：LogLens = klogg（免费，release 停 4.3 年 / commit 停 22 月）× LogViewPlus（付费在位）。

**实现约束自查**：必须能长在 danqing 上（Rust 自绘 UI，单人带宽，UI 占比要高）；不碰协议逆向深水区（C3 教训）；不碰重 ML 栈（C1 / Windrecorder / OpenRecall 死因）；在场质检（读得懂该领域抱怨）；渠道至少一条够得着（round4 校准：GitHub 免费发布 ≈ 零自然流量；主渠道 = MS Store 搜索 + 内容平台直链 CID 95/5）。

闸门全集沿用 round4「选型闸门全集」节，不复制。死池 30+ 品类见 round4「累计死池」节 + 自家枪毙归档（泛截图/AI 听写/快速记录/抓包/数字健康阻断器），普查前排除。

## 扫描设计（5 agent 并行，2026-09-18 发出）

| # | 普查域 | 重点查证对象（举例，agent 须自行扩充） |
|---|---|---|
| 1 | 系统/硬件/外设 | 多显示器管理（DisplayFusion 付费）/ 显示器校准（DisplayCAL）/ 外设配置 / WiFi 分析 / 电池电源 |
| 2 | 文件/文档/数据周边 | 批量重命名 / EXIF 元数据 / 数据恢复（Recuva）/ 分区 / 文件标签（Tabbles 付费）/ 虚拟光驱 / HEIC 转换 |
| 3 | 创意/垂直制作 | 字体管理（NexusFont）/ 图标编辑（IcoFX $59）/ 十字绣编织 / 激光 CNC（LightBurn）/ 写作剧本（Scrivener·Final Draft）/ TTRPG 地图 / 照片 culling |
| 4 | 效率/桌面体验/身心 | 专注阻断（Cold Turkey $39）/ 日记（Diarium）/ 鼠标手势 / 快速预览（Seer）/ 时间追踪（ManicTime） |
| 5 | 收藏/生活/利基 | 食谱（Paprika $29.99）/ 收藏品管理 / 家庭资产清单 / 气象站（Weather Display $70）/ 业余无线电（HRD $99）/ 卫星追踪（Orbitron） |

**报告摄取约定**：agent 报告回来后只提取「过闸结论 + 证据 URL 指针」进本文档，原始报告不整存（防上下文淹没）。

## 扫描结果 · 压缩摄取

### ④ 效率/桌面体验/身心（已完成 2026-09-18）— **零候选**

**域总评**：19 品类中 14 个被活跃免费 catcher 封死（开源社区或微软第一方）、4 个无合格僵尸、1 个死池邻近核销——在位者要么不睡觉、要么睡醒了（Nimi Places 复活）、要么尸体旁站着免费接班人（Power Automate Desktop / keyviz / Quicker）。

走近后死亡留档：

- **鼠标手势（全场最接近缝的形状）**：WGestures 1.6k★ 真僵尸（push 停 2022-09；新 issue #123 2026-01-19「这么好的软件居然停更了」）+ 付费在位 WGestures2 ¥36.99 买断（同作者；其 bug 仓 #368 2026-03「没维护了」= 付费版也在停更边缘）——但 **Quicker 免费版活跃且内置鼠标手势**（阉割 8 轨迹）→ 条件⑤挂。按五条件原文不放宽，留档。
- **宏录制 GUI**：Pulover's Macro Creator 2k★ 真僵尸 + 2026-03 新 issue——但 **Power Automate Desktop 第一方免费内置 Win11** → 封死。
- **按键可视化**：carnac 4.5k★ + KeyCastOW 1.4k★ 双尸——但 keyviz 9.6k★ 活跃 → 封死。
- **任务栏时钟**：T-Clock 1.8k★ 僵尸 + ElevenClock 归档（2025-09-05）——Win11 原生托盘秒针，OS 追平（ElevenClock 归档即旁证）→ 核销。

**拒清单（19 条，证据日期全 2026-09-18，并入死池）**：时间追踪（ActivityWatch 18.9k★ 当日 push）/ 快速预览（QuickLook 24.7k★ 活跃；Seer 付费在位无效）/ 音频设备切换（EarTrumpet + SoundSwitch 双活跃）/ 休息用眼提醒（Workrave + stretchly 双活跃 + 26H2 Screen Tint）/ 专注阻断（死池核销：Cold Turkey 同形状同 MSIX 风险）/ 窗口置顶（PowerToys 第一方）/ 虚拟桌面任务切换（PowerToys 雷区 + Window Hopper 已吃）/ 日记（RedNotebook 活跃；mini-diary 1k★ 僵尸但条件⑤挂）/ 宏录制（上文）/ 按键可视化（上文）/ 鼠标手势（上文）/ 任务栏时钟（上文）/ 右键菜单定制（Nilesoft Shell 6.8k★ push 仅 7 月前不满僵尸线）/ 习惯追踪（无桌面 ≥1k 僵尸 + Habitica/uhabits web 移动活跃）/ 打字练习（无桌面僵尸 + web 活跃）/ 快捷键冲突检测（无 ≥1k 仓）/ 桌面图标整理（Nimi Places 复活 + iTop + 腾讯桌面整理免费）/ 最小化到托盘（RBTray 18 个月不足线 + UI 占比极低）/ 显示器亮度（Twinkle Tray 活跃）

### ③ 创意/垂直制作（已完成 2026-09-18）— **候选 ×1（C1 分镜/故事板）**

**C1：Windows 桌面分镜/故事板工具**（Storyboarder 现代重建：手绘画板 + 镜头管理 + 动态分镜导出，本地买断）

五条件全过（证据日期全 2026-09-18）：

1. **僵尸在位者**：`wonderunit/storyboarder` ★3845 / 最后 release v3.0.0 **2021-02-17** / 最后 push 2024-03-17 / fork 接班无（forks 全是 0★ 同步副本）。Wonder Unit ≠ Wonder Dynamics（被 Autodesk 收购那家），弃坑原因〔未深挖〕但仓库事实性停摆
2. **付费在位者密集**：Toon Boom Storyboard Pro **订阅-only $87/月**（永久授权已停售，官方原文 "No longer available"）/ FrameForge 买断 $379-849 / StudioBinder $29-42/月（免费档=1 项目试用级）/ Boords ~$49-75/月（免费档试用级）/ Previs Pro $119.99/年或 ~$360 买断**仅 Apple 平台不覆盖 Windows**
3. **Windows 侧免费 catcher 缺位**：2026 年评测仍把僵尸本体列为「标准免费分镜应用」；MS Store 仅一款中文「闪电分镜」且自身停在 2024-07；Krita/Canva 是通用画布非专用
4. **需求未衰减**（gh api 实证僵尸仓 2026 新 issue）：#2676（09-01 视频导出全坏）/ #2674（08-26 FDX 导入报错）/ #2671、#2665（两条直接问「还维护吗」）
5. danqing-fit：UI 占比高；**两块新肌肉未验证**——压感手绘（Windows Ink 笔画引擎）+ 动态分镜视频导出（ffmpeg 管线）；v1 不含 AI

**渠道预判**：MS Store「storyboard/分镜/故事板」货架极空；僵尸仓 3845★ 自带搜索流量；YouTube/B站分镜教程普遍以 Storyboarder 为推荐（内容平台入口现成）。

**毒点（深挖轮必答）**：①**需求迁移风险（最大）**——AI 分镜生成（Boords AI / storyboarder.ai / LTX）与中文 AI 短剧管线（Toonflow 15.7k★ / waoowaoo 14k★ / ArcReel 5k★，2026-08/09 集体爆量）若成为分镜主流入口，手绘分镜需求迁移；②短视频创作者可能直接用剪映/CapCut 模板跳过独立分镜环节（深挖期量化人群规模）；③在场质检中等——抱怨是工具质量型（可读）但「什么是好分镜工具」需领域感（镜头语言/FDX/动态分镜时序），非开发者原生域。

**失效标志**：AI 分镜工具免费档放开到可用级 / 中文 AI 短剧管线成分镜主流入口 / Storyboarder 原仓复活或 fork 接班 / 深挖发现手绘分镜真实人群过小。

**拒清单（19 条，证据日期全 2026-09-18，并入死池）**：字体管理（NexusFont 闭源 2.6.2@2015 无开源锚 + FontBase 免费档活跃——⑤② 冲突条目以此为准）/ 图标光标编辑（无 ≥1k 锚 + Greenfish 闭源停更；IcoFX $59 在位）/ 十字绣编织（无锚 + Stitch Fiddle freemium + 新 catcher embroiderly 涌现）/ 刺绣（Ink/Stitch 1.3k★ 活跃）/ 木工下料（MaxCut 免费版活跃 + 免费网页优化器扎堆）/ 小说写作（Manuskript 0.17.0 2025-06 非僵尸 + yWriter 在位）/ 剧本写作（无锚——Trelby 346★/KIT 350★ archived + WriterSolo/Arc Studio 免费档承接）/ 时间线制作（无 ≥1k 桌面锚 + 免费 web catcher 扎堆 + Plottr $60/年在位）/ TTRPG 地图（Dungeon Scrawl 免费活跃 + Inkarnate freemium + Azgaar's OSS + AI 地图生成器一波）/ 像素画（LibreSprite 8.4k★ + Piskel 12.8k★ 双活跃）/ 定格动画（无锚 + Boats Animator/Tahoma2D 免费活跃）/ 照片 culling（无锚 + 新免费 catcher 近两周密集涌现 pixcull/lenslink/rawblow/teststrip——**2026 免费 catcher 涌现速度又一实证**）/ 联机拍摄（digiCamControl 734★ 不足 + 厂商免费软件）/ 打谱（MuseScore 15k★ 活跃核销）/ 字体制作（FontForge 8k★ 活跃）/ 缝纫打版（Seamly2D 969★ 不足且活跃）/ 世界构建 campaign 管理（chronicler/Kanka 活跃）/ VJ 舞台灯光（无锚 + QLC+ 1.5k★ 活跃）

**观察名单（转下轮复查）**：激光切割 LaserGRBL 1.6k★ 最后 release 2025-03-01（18 个月，未满 2 年僵尸线）——**2027 年中复查**，若届时停更满 2 年且 LightBurn $60-199 付费位仍在，即成 klogg 型。

### ② 文件/文档/数据周边（已完成 2026-09-18）— **零候选**

**域总评**：Windows 工具软件最古老最稠密域——几乎每个品类被三层之一封死：①活跃免费老牌（ExifToolGUI 复活/fre:ac/TagSpaces/BleachBit/Picard/Recuva）②PowerToys 第一方模块（重命名/文件解锁）③Windows 系统内置（ISO 挂载/哈希/元数据移除/快速删除/Booklet 打印）。付费意愿铁证完好的品类（虚拟光驱/分区/数据恢复/批量打印）要么无开源僵尸可接、要么撞内核驱动/取证深水区实现约束。

走近后死亡的留档：

- **虚拟光驱**：僵尸确凿（WinCDEmu 1.4k★，release 停 2019-01，fork 全停，近一年新 issue 含 #53 Win11 BSOD 2026-08-19）但三杀——DAEMON Tools Lite 免费活跃（2026-07-27 版；2026-05 曾遭供应链投毒 CVE-2026-8398）+ Win11 原生挂载 + 必须写签名内核驱动（BSOD issue 即驱动维护代价实证）撞实现约束。
- **数据恢复 GUI**：免费三层接住（Recuva 仍发版 2026-06 / PhotoRec 活跃 / 微软 winfr）+ 取证深水区 + 数据丢失责任风险，实现约束独立否决。
- **自动文件整理（DropIt/Hazel 型）**：双重拒——DropIt 无 ≥1k★ GitHub 仓（条件①不过）+ 农场历史（Hazel 候选 2026-09-15 已被用户从队列去掉，不 resurfacing）。

**平台杀需求实例（留档）**：USB 安全弹出——Win10 1809 起默认「快速删除」可直接拔，付费在位 USB Safely Remove 的需求被系统默认设置掏空。

**字体管理冲突核销**：FontBase 免费层活跃（2026.5.x）；NexusFont 是**闭源免费软件**、无 ≥1k★ 开源僵尸 → 条件①不过。⑤ 号报告的「NexusFont 免费活跃」措辞不准但结论方向一致（该品类五条件不成立），死池入账。

**拒清单（20 品类，证据日期全 2026-09-18，并入死池）**：批量重命名（PowerRename 第一方 + Advanced Renamer 个人免费活跃）/ EXIF 编辑 GUI（ExifToolGUI 被 FrankBijnen 复活活跃）/ 元数据擦除（ExifCleaner 2.7k★ 活跃）/ 音频批转（fre:ac 活跃）/ 音频标签（Picard 5.2k★ 活跃 + Mp3tag）/ 数据恢复（上文）/ 分区管理（实现约束 + MiniTool/AOMEI 免费版在位）/ 文件标签（TagSpaces 5.3k★ 活跃）/ 虚拟光驱（上文）/ 拼版打印（Acrobat Reader 内置 Booklet）/ CHM（hh.exe 内置 + 需求衰减）/ 字体管理（上文）/ 文件粉碎（BleachBit 活跃 + cipher/sdelete 内置）/ 哈希校验（OpenHashTab 仓 2026-05 遭 DMCA 封锁插曲；HashMyFiles + Get-FileHash 内置）/ 离线介质目录索引（无 ≥1k★ 僵尸）/ 批量打印（无 ≥1k★ 僵尸；Print Conductor ~$149 付费佐证意愿但无僵尸即无缝）/ 自动文件整理（上文）/ HEIC 转换（CopyTrans HEIC + iMazing Converter 双免费活跃）/ USB 弹出增强（上文）/ 文件解锁（PowerToys File Locksmith + LockHunter 免费）

### ⑤ 收藏/生活/利基（已完成 2026-09-18）— **零候选**

**域总评**：「僵尸复活区」而非「僵尸区」——老牌开源爱好软件近年几乎全在活跃维护（Gpredict 停更多年后 2026 连发三版 v2.5.2/v2.6；GCstar 停 3 年后 2026-06 v1.8.1；Subsurface/Gramps/JMRI/Log4OM 全活）；未复活的品类从未长出 ≥1k★ 桌面开源在位者（需求沉淀在 web/移动端）。**建议后续普查不再覆盖本域。**

**方法论点观察（两条，载荷性）**：
1. **发版渠道假象**：Subsurface 的 GitHub releases 页停在 2020 但官网发版至 2026-09——僵尸判定条件①必须查**官方发版渠道**，GitHub releases 停更 ≠ 停更。
2. **「老牌 OSS 集体复活」疑与 AI 辅助编程降低单人维护成本有关**（agent 推测，**无证据**）——若属实，klogg 型缝存量随时间收窄，僵尸普查窗口也在关。留待复盘。

**拒清单（28 品类，证据日期全 2026-09-18，并入死池）**：
- 条件⑤免费活跃封死：播客桌面（gpodder 1.4k★）/ 收藏品通用（GCstar 复活 + Tellico 活跃）/ 硬币邮票专项（OpenNumismat 活跃，不足 1k）/ 漫画收藏阅读（YACReader 活跃）/ 卫星追踪（Gpredict 复活）/ 健身骑行（GoldenCheetah 2.2k★）/ 气象站（CumulusMX 活跃——Weather Display $70 付费在位无效）/ 业余无线电日志（Log4OM 2 免费 2026 已发 5 版 + N1MM——HRD $99 付费在位无效）/ 天文星图（Stellarium 10k★）/ 天文摄影（NINA）/ 电子书阅读（Koodo 28k★ + Readest 24k★ 双活跃）/ 家谱（Gramps）/ 家酿啤酒（Brewtarget）/ 潜水日志（Subsurface 官网发版）/ 模型铁路（JMRI）/ 战锤军表（BattleScribe 生态）/ 十字绣图纸（kxstitch + embroiderly 双活跃）/ 食谱（Recipe Keeper 免费+Pro $19.99 活跃 + Mealie 自托管 + Samsung Food web——Paprika $29.99 付费在位无效）/ 葡萄酒窖（CellarTracker web）
- 条件①无 ≥1k 开源在位者（需求在 web/移动/闭源免费）：家庭资产清单 / 观鸟（eBird web 垄断）/ 鱼缸 / 爬宠 / 桌游收藏（BGG web）/ 邮票钱币老付费软件域 / 园艺 / 天文观测日志 / 乐谱库 / 宠物繁殖 / 电影收藏（EMDB 闭源免费在位）
- ⚠️ 越界条目：⑤ 顺手判了「字体管理 Windows 侧 NexusFont 免费活跃」——属 ③ 号普查域且未附版本/日期证据，**该品类以 ③ 号报告为准**，此条存疑不入死池

### ① 系统/硬件/外设（已完成 2026-09-18）— **零候选**

**域总评**：Windows 免费工具供给最稠密域——NirSoft/CPUID/HWiNFO 长年免费更新，开源明星仓（FanControl/TrafficMonitor/Rufus/NAPS2/TwinkleTray/OpenRGB/G-Helper/Macro Deck）全活着，Win11 持续内化多显示器/手势/电池功能。僵尸五条件几乎无立锥之地。

两个走到最后一步才死的留档：

- **笔记本风扇/性能控制（NBFC 型）**：唯一条件①完全成立的僵尸（nbfc 3.2k★，release 停 2019-04，push 停 2024-07，无接班 fork），被条件⑤杀死——G-Helper 15.3k★ 当日仍 push + LenovoLegionToolkit 7.5k★（2025-07 归档）+ 厂商免费软件全覆盖；EC 寄存器逆向另撞「协议逆向深水区」实现约束。
- **多显示器管理**：DisplayFusion 付费在位但免费侧无 ≥1k 僵尸（Dual-Monitor-Tools GitHub 仅 24★，条件①不过）；Win11 原生多显示器任务栏/Snap/布局记忆持续内化；DisplayMagician 745★ 不足且活跃。

**拒清单（证据日期全为 2026-09-18，并入死池）**：硬件监控（TrafficMonitor 46.2k★ 活跃）/ 风扇控制（FanControl 20.9k★ 活跃）/ 系统信息（CPU-Z·HWiNFO 活跃）/ SMART（CrystalDiskInfo 活跃）/ 基准测试 / USB 启动盘（Rufus·Ventoy 活跃）/ 扫描仪（NAPS2 活跃）/ 卸载器（BCUninstaller 活跃）/ 音频路由（EarTrumpet 活跃）/ 音频 EQ（FxSound 活跃）/ 显示器亮度（Twinkle Tray 活跃）/ RGB（OpenRGB 1.0 2026-09-11）/ 数位板驱动（OpenTabletDriver 活跃）/ 鼠标加速度（rawaccel 活跃）/ 内存清理（MemReduct 活跃 + snake-oil 属性）/ Win11 兼容检查（WhyNotWin11 活跃 + EOL 已过衰减）/ 数据恢复（PhotoRec 活跃）/ 显示器校准（DisplayCAL 原仓 404，fork eoyilmaz/displaycal 1.6k★ 活跃接班 → 条件②不过）/ WiFi 分析（Vistumbler 作者 MAUI 重写仓活跃 + Acrylic 免费）/ 驱动更新（SDIO 月更 + gtumanyan/SDI 活跃）/ Stream Deck 类（Macro Deck 1.5k★ 当日 push）/ 电池电源（无 ≥1k 僵尸 + 厂商阈值功能）/ 蓝牙 USB 工具（NirSoft 活跃）/ 打印队列标签（无僵尸 + B2B 味）/ 分区管理（无 ≥1k 僵尸 + EaseUS freemium 活跃）/ GPU 超频（Afterburner 免费活跃）/ 鼠标按键配置（X-Mouse 闭源 freeware，无开源僵尸）/ 触控板手势（GestureSign 936★ 不足 + Win11 内置）/ 显示配置档切换（DisplayMagician 745★ 不足且活跃）

## 合成（2026-09-18，五路普查齐全）

**1. 产出账目**：~116 品类普查 → **候选 1 条（C1 分镜/故事板）** + 观察名单 1 条（LaserGRBL，2027 年中复查）+ 死池新增 ~100 条（全带证据日期）。四域结构性零产出：系统硬件/文件文档/效率桌面 = Windows 免费供给最稠密域（老牌免费工具 + PowerToys 第一方 + 系统内置三层封死）；收藏利基 = 僵尸复活区。唯一产出域 = 创意/垂直制作（专业邻近工具：付费在位者真 + 公司弃坑型 OSS 真）。

**2. 方法论产出**：

- 僵尸普查产出率 1/116 ≈ 0.9%——高于扰动扫描（两轮零）但仍是低产产生器；产出集中在「**专业邻近 + 公司弃坑（非商业失败）**」型（Wonder Unit 弃坑 Storyboarder 留下 3.8k★ 僵尸，付费侧集体订阅化）
- **僵尸判定必须查官方发版渠道**（Subsurface GitHub releases 停 2020 但官网发版至 2026-09 的假象）——已写入五条件执行细则
- 「老牌 OSS 集体复活」疑与 AI 辅助编程降低单人维护成本有关（agent 推测，无证据）——若为真，klogg 型缝存量随时间收窄，僵尸普查有保质期
- 2026 免费 catcher 涌现速度再实证：照片 culling 品类两周内冒出 pixcull/lenslink/rawblow/teststrip 四个新免费 catcher——**缝的保质期按月计在供给侧同样成立**
- ④ 域结论引用：「在位者要么不睡觉，要么睡醒了，要么尸体旁站着免费接班人」——效率/桌面域对僵尸缝结构性免疫

**3. 候选池**：

| # | 候选 | 五条件 | 毒点 | 建议下一步 |
|---|---|---|---|---|
| C1 | 分镜/故事板重建（Windows 桌面，本地买断） | 全过（③ 节，证据全带日期） | ①AI 分镜生成需求迁移（中文 AI 短剧管线 2026-08/09 爆量）②danqing 两块新肌肉（压感笔画 + ffmpeg 导出）③在场质检中等（非开发者原生域） | 深挖前置两杀验证：AI 迁移量化调研 + 笔画引擎 POC 技术验证，任一不过即毙 |

**4. 诚实总评**：C1 是三轮扫描以来第一个过五条件的候选，形状教科书级（弃坑僵尸 + 付费订阅化 + 免费 catcher 缺位 + 2026 仍有新 issue）。但它的两个一票否决项恰好都是农场历史上杀过候选的类型（C1-round4 死于技术栈不匹配；需求衰减型失效已立档）。**两杀验证成本 ~1-2 天，全在证据侧不在猜测侧**。备选合法裁决：留档不开枪回分发线（round4 裁决精神延续，log 首周数据积累中）。

## C1 深挖 · 两杀前置（用户裁决 2026-09-18：进深挖）

任一不过即毙（归档）；都过才进完整深度调研（竞品 open issues 逐条扒 / 需求验证 / 定价调研 → intent 含裁决段 → 红队 → 一页粗算）。

| # | 一票否决项 | 验证方式 | 状态 |
|---|---|---|---|
| K1 | **AI 需求迁移**：手绘分镜人群 2026 还剩多少、是否在缩；AI 分镜/中文 AI 短剧管线是否在吃这个环节 | 调研 agent ×2（K1a 替代侧：AI 分镜工具能力与采用度；K1b 需求侧：人群规模/Windows 占比/渠道可达/土法凑合证据） | **完成：否决触发**（K1a「强」+ K1b 付费意愿零直接证据） |
| K2 | **danqing 笔画引擎**：winit 0.30 压感支持 + wgpu 笔画渲染可行性 | 技术验证 agent ×1（只读不改，产出可行/不可行 + 人日粗估 + 风险清单 + POC 最小步骤） | **K2 完成：判定「可行」**（资产留档，不救 C1——K1a 已否决） |

### K1b（需求侧，已完成 2026-09-19）— 痛点门槛「勉强过线」，付费意愿零直接证据

- **人群规模**：全球仍在画/找分镜工具的活跃层年十万级——Storyboarder v3.0.0 死 5.5 年仍累计 148 万下载（全系列 >600 万），按 1/5 衰减估当前 5-15 万/年新增下载流向死软件；中国：动画在校生 13-15 万 + 微短剧就业 69 万（北大国发院 2026-02）；B站「老白的分镜课」151.5 万播放
- **Windows 占比**：专业层偏 Apple（Previs Pro 官方明示 Apple-only 无 roadmap）；长尾层偏 Windows（僵尸仓 issue 样本 Win10/Linux8/Mac1；中文短视频/漫剧团队全 Windows+剪映生态）→ 目标长尾 60-80% Windows
- **渠道**：GitHub 够得着（中文分镜仓能引爆：「分镜大师」2026-01 建仓即 473★；但热词流量偏 AI 管线——Seedance2-Storyboard-Generator 2.4k★）；MS Store 货架真空但**流量未验证**（「闪电分镜」0 评分 0 星，curl 实证 lastUpdate 2026-08-07——③ 号「停更 2024-07」二手情报有误，一手复核核销）；内容平台勉强（品类词活着，但 Storyboarder 专属内容已死：B站停 2025-01、2026 教程位被 AI 工具占据）
- **痛点门槛过线**：≥3 来源 ✓（僵尸仓 issue 流速稳定 95/68/97 非崩塌 + r/Storyboarding 三个 2025-26 求替代帖真实 URL + 中文生态 + 行业报告）；近两年 ✓；行为证据 ✓（asar 土方自救修导出 / CrossOver 也要跑 / 问维护无人理后 fork 自救 / Netflix 分镜师当日注册留言 / 纸笔→PS→Premiere 顶替）
- **减注①（致命倾向）**：行为证据**全是「抢救免费工具」或「免费顶替」，无一例为分镜掏钱的直接行为**——人群被免费工具自我筛选过
- **减注②**：痛点性质 = 维护真空（导出坏/autosave 丢稿/EULA 乱码/无图层），用户预期锚定「免费」
- **失效标志②核查（分层证实）**：模板/口播类确实跳过独立分镜（剪映闭环）；剧情短剧/漫剧**不跳过分镜但形态已迁移到 AI 生成管线**（AI 参与度 50-80%，「AI 分镜师」成新岗位溢价 10-30%）——增量人群要「文本→镜头→AI 出图」，不是手绘桌面分镜
- 推翻证据（留档）：MS Store impressions 30 天实测趋零 / AI 分镜仓用户完全不需手绘 / Krita storyboard docker 活跃化 / Previs Pro 出 Windows 版；反向加强信号：闪电分镜收费起量 / 公开付费意愿言论 / issue 流速逆势上升

### K2（笔画引擎可行性，已完成 2026-09-18）— 判定「**可行**」（复用资产，独立于 C1 生死）

- **核心发现**：danqing 锁定 winit 0.30.13（Cargo.lock:5287），Windows 原生走 WM_POINTER 系拿数位笔压感——`GetPointerFrameInfoHistory` 解合并高频采样不丢点（120Hz+ 无压力）、`GetPointerPenInfo` 拿 pressure 归一化 `Force::Normalized(p/1024)`、亚像素坐标保留。**不用动 winit、不用绕框架**
- **产品层限制（可缓解）**：winit Touch 抽象丢笔/手指区分、橡皮倒置旗标、倾角——缓解 = 工具按钮切橡皮（Xournal++ 同款）、仅笔模式启发式；无压感设备退路 = 固定粗细，降级影响小
- **落点全有现成模式**：`Event::Pen` 变体（event.rs:121）→ `convert_event` 加 Touch 臂（window/event.rs:210 现 `_=>None` 丢弃）→ 分发+笔捕获仿 mouse_capture（handler.rs:930-976）；渲染加第三 mesh pass（InkPipeline/InkBatch 照抄 rect.rs:564-632 模板；`paint_ink` 仿 `paint_image` 先例）
- **Tessellation 选型**：POC = 手写变宽 polyline 扩线（零依赖 ~150-250 行纯逻辑）；正式版 = perfect-freehand 式轮廓（MIT crate 2026-01）+ lyon fill（lyon_tessellation 1.0.22，2026-09-06 仍更新，纯几何零 wgpu 耦合）；**vello 出局**（wgpu ^29 vs danqing 30 版本错配）
- **延迟**：danqing 可见态实质 ~60fps 逐帧渲染（OnDemand 命名有误导，隐藏态才按需），笔事件→上屏 ≤1 帧，端到端 ≈17-34ms@60Hz——非专业绘图应用同档，分镜 POC 达标
- **参考实现**：Rnote 是 Rust（11.6k★ 活跃，cairo/CPU 渲染）——`PenPath{Vec<Element{pos,pressure}>}` 数据模型可照抄；ink-stroke-modeler-rs（Google 墨水平滑）留 polish
- **工作量**：POC 5.5-7 人日（半天压感 spike + 事件链 0.5-1 + mesh pass 1-1.5 + 笔画引擎 1.5-2 + 组件 1-1.5 + 测试 1）；产品级追加（平滑/撤销/缩放/擦除/持久化）另 5-10 人日
- **风险**：中——Touch 抽象信息损失 + 笔/鼠标双报未实测（spike 第一项验证）；低——驱动压感粒度 / 全量重绘 / wgpu 版本错配 / 测试纪律（禁 SendInput，农场封禁令）
- **半天 spike 分步**（留档）：压感链路 log 实测 → 双报检测 → 现有 rect pass 画点列验证跟手 → 压力→宽度映射 → 判定（压感连续 + ≥60 点/秒 + 无肉眼滞后）
- **完整证据索引见 K2 原始报告**（winit/danqing 文件:行号 + crates.io 版本日期全在）

### K1a（替代侧，已完成 2026-09-18）— 判定「**强**」，失效标志①②同时兑现

- **中文 AI 管线三仓（gh 实证）**：Toonflow 15.7k★（2026-01 建仓）/ waoowaoo 14.2k★ / ArcReel 5.1k★（当日仍 push）——管线内分镜步全部 AI 生图、无一处手绘入口（「AI 出初稿、人工修节点」）
- **分镜被模型内置化**：可灵 3.0「智能分镜」内置（MAU 1200 万+、月收 $2000 万+）；即梦 Seedance 2.0 自主镜头调度——分镜从独立工具品类变成视频模型的 feature
- **最硬替代证言**（投资界 2026-05）：「分镜师、摄影师、服化道团队被提示词优化和模型选择所替代」；招聘市场行为证据（storyboardworld.com 2026-09-14）：纯手绘分镜师月薪 ¥5-12K vs 会 AI 全管线 ¥25-50K，岗位要「人机双修」
- indie 首稿分镜成本/周期塌缩：1-4 周 / $500-5000 → 15 分钟-2 小时 / $0-50
- **厂商行为铁证**：Wonder Unit 官方确认桌面版不再维护，新产品 Sumugi =「Make Comics **without drawing**」——原作者的下一站连「画」都不要了
- 僵尸仓内无「我改用 AI 了」自白——迁移者静默流失（沉默证据），残留 = 手绘钉子户（issue #2509 有 Netflix 分镜师仍在用，存量忠诚）
- 反向证据存在但全是存量守势：35% 自称坚持手绘、AI 一致性/视线匹配/场面调度短板、Toon Boom 仍是工作室标配
- **结论**：手绘分镜工具市场不是「尚未被 AI 吃到」，而是「增量已被吃完、只剩收缩中的存量钉子户」，且免费 AI 在位者（开源管线 + 模型内置功能）双重封死 → **K1 不过**
- 推翻本判定的新证据（留档）：院校/协会硬性规范要求手绘 / Boords·Katalist·LTX 留存崩塌 / storyboarder 活跃 fork 持续增长 / 纯手绘岗位薪资回升 / 可灵即梦移除智能分镜

## 结案（2026-09-19）

**C1 判死归档**（用户预授权「任一不过即毙」自动生效）：

- K1a 替代侧「强」：增量已被 AI 吃完（分镜被做成模型内置 feature），失效标志①②兑现
- K1b 需求侧：痛点门槛勉强过线，但**付费意愿零直接证据** + 增量人群形态错位（要 AI 管线不要手绘桌面）
- K2 供给侧「可行」——技术不是短板，需求才是；笔画引擎评估成独立资产留档
- **复活条件**：K1a/K1b 各五条推翻证据（见上两节留档）

**连续第三轮零候选**（round3 六域 60+ 枪毙 → round4 四轴 0 → round5 僵尸普查 116 品类 1 候选仍毙）。**「瓶颈在分发不在找品」三次确认。**

**方法论产出（建议写入心法，待用户裁决）**：

1. **需求迁移型失效**（新失效子型）：痛点没死、但工作流被 AI 管线整个接走——判法 = 看**增量人群的当前工作流形态**，不看存量钉子户。与「需求衰减型」（老票多新票无）并列
2. 僵尸普查五条件全是供给侧——需求侧必须配 K1 式双半验证（替代侧 + 需求侧）才完整；其中**付费意愿行为证据**（为这类工具掏过钱）应单列，「抢救免费工具」不算付费证据
3. 僵尸判定查官方发版渠道（Subsurface 假象）；二手情报必须一手复核（「闪电分镜停更 2024-07」被 curl 实证推翻：lastUpdate 2026-08-07）

**遗留资产**：

- K2 笔画引擎可行（winit 0.30.13 WM_POINTER 压感原生、POC 5.5-7 人日、spike 分步在档）——未来绘画/手写/标注类产品直接复用
- 死池 +100 条全带证据日期；观察名单 LaserGRBL（2027 年中复查）
- round4 时钟在档：C1-round4（screenpipe 12 个月走向，2027-09 前）/ C2 kill criteria / C3 MTP 验证
- 30 天校准钟 2026-09-18 起算，2026-10-18 前后回填及格线

**用户裁决（2026-09-19，本轮终结）：时钟驱动 + 主攻分发。** 选型机器不设定期扫描；触发器：①log 30 天校准数据（2026-10-18 到点）②观察名单/复活条件时钟（LaserGRBL 2027 中 / screenpipe 走向 2027-09 前 / C2 kill criteria / C3 MTP 验证）③新发 klogg 型扰动事件（发现即开工）。期间主攻分发线：log 领先指标 + pomodoro 商店 + 内容平台直链（CID 95/5 打法）。方法论三条建议（需求迁移型失效 / K1 双半验证 / 付费意愿单列）留档本节，写入 CLAUDE.md 心法与否另裁。

## 待办（按序）

1. ~~收齐 5 份普查报告 → 过闸 → 候选池写入本文档~~（2026-09-18 完成，见合成）
2. ~~候选池呈用户裁方向~~（2026-09-18 裁决：C1 进深挖，两杀前置——见上节）
3. ~~两杀验证收齐 → 判活/判死~~（2026-09-19 完成：K1a「强」否决 + K1b 付费意愿零证据 + K2 可行留档 → C1 判死，本轮结案）；选型机器走向**已裁（2026-09-19）：时钟驱动 + 主攻分发**
4. 开枪前：重查在位者状态（带日期）+「有没有人免费做」+ 组合纪律确认
5. （分发线 = 主攻方向，用户裁决 2026-09-19）log 商店 impressions / GitHub traffic 埋点本周确认可读；day-1 GitHub 基线已拉取（见下）；C1/C2/C3 复活时钟（round4 留档）到点再查——C1 看 screenpipe 12 个月走向，2027-09 前

### 分发线 · GitHub 基线（2026-09-19 gh api 拉取，下次拉取对比用）

**danqing-log（发布次日）**：views 发布前两周四舍五入为零，发布日起小坡（09-16: 9 次/4 unique → 09-17: 15/5）；clones 爬虫噪音（09-14: 82/22u、09-15: 75/27u）；**v1.0.0 zip 下载 = 2**（sha256 = 0）；stars 0 / forks 0。
**danqing-pomodoro（对照）**：views 两周四舍五入为零（最高 5/1）；全 release 下载累计 ~6；stars 0。
**读法**：与 round4 校准结论一致（GitHub 免费发布 ≈ 零自然流量）；log 的真实渠道未知量在 MS Store impressions——Partner Center 数据只有用户本人能拉，10-18 校准钟到点前需确认可读。

### 分发线 · MS Store 基线（2026-09-19 用户 Partner Center 截图实证）

**丹青-番茄钟（过去 1 个月窗口）**：页面视图 **46** / 安装尝试 **12** / 安装成功 **12**（成功率 100%）/ **转化 26.09%**（按页面浏览量列出的安装数）。农场第一份真实渠道数据。
**读法**：① **MS Store 自然流量存在但薄** —— 无内容推广下约 1.5 次浏览/天，商店自己带不来量级，发帖冲刺才是引擎；② 26% 转化样本太小不下结论，但方向是正的；③ 这给了 log 商店侧一个**量级预期**：过审后自然流量大概率同量级（几十次浏览/月），10-18 校准时拿这个当基准线；④ pomodoro 商店本体是**免费**的（¥0 + ¥18 内购解锁完整版），这 12 笔是免费下载**不是销售**；**Add-on acquisitions 读数 = 2，均为用户 09-04 自购测试单 → 陌生人首单 = 0，首单闸门未过**（2026-09-19 用户查证）。漏斗全貌：46 浏览 → 12 免费安装 → 0 付费；样本太小无法判转化率，但判「发现」已足够 —— 漏斗顶部太薄，唯一杠杆是内容推广。

## 开放问题（等用户）

- ~~（沿用 round4）pomodoro Partner Center 商店页上架状态~~ → **2026-09-19 双闭环：已上架且有自然流量；首单外检已读数**（Add-on acquisitions = 2，均为用户自购 → 陌生人首单 0，闸门未过。见上方 MS Store 基线）。
- log 商店认证状态：认证中（2026-09-19 用户确认）；过审后用户贴 Store ID/链接，接进 README 下载节 + 仓库 homepageUrl + launch-copy 速查表（七处占位清单已备），随后按发稿执行清单七平台齐发。
