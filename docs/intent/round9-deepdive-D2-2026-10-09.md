# D2 供给体检 / 免费在位者穷尽 · Drop Shelf 文件暂存架(2026-10-09)

> 上游:`deep-dive-round9-t1-2026-10-09.md` 卡A D2 题(四小题)。
> 取证窗口:2026-10-09 当日;证据日期一律标注。gh 全路径 `"C:\Program Files/GitHub CLI/gh.exe"`。
> 时间盒执行:取证 ≤60 分钟 + 写盘 15 分钟。

---

## 0. 结论先行

**「有没有人(任何人)免费做这件事」——有,而且不是一个,是一群。**

- **头号免费在位者 = GameGodS3/DropPoint(1094★,GPL-3.0,2021 建仓)**:「Drag and drop any file(s) or folder from the system into DropPoint, go to your desired location and drag it out」= **收→暂存→一次拖出全链原句**。R2 卷手搓 6 仓名单**完全没有它**——这是 tile「只盯一个上位者(漏 Seelen UI)」教训的本卡复刻风险点,本次补上。
- **2026-09/10 新一波免费实现正在扎堆**:StashIt(1★,Rust+Tauri,winget 可装)、KyteShelf(0★,Python,2026-10-08 仍在推)、Edge-Drop(**632★**,边缘 hover 架,3 个月冲星)、holdem(152★,2026-09-04 还发版)——功能清单已对齐甚至**超出 Dropshelf Pro**(多架命名/置顶 = Dropshelf Pro 专属,StashIt/KyteShelf 免费给)。
- **但「单文件便携 GUI 浮动置顶架 + 买断 Pro」这个精确形态位,免费侧还没有一个「成熟+轻量+活跃发版」三合一的在位者**。DropPoint 老但 Electron 重(StashIt 对标表称 150–300MB);StashIt/KyteShelf 是 0–1★ 新兵;Edge-Drop 是边缘 hover 形不是浮动窗形。空位窄、但在。
- **失效标志④半兑现**:「DropIt/Direct Folders 查实覆盖同形态且活跃」——**未兑现**(DropIt 死于 2018 且是规则归类器;Direct Folders 是文件夹导航器);**但被 DropPoint 1094★ 部分替代兑现**——免费侧确有全链覆盖者,只是形态/工程质量有差。判死与否归用户,本报告只呈证据。

---

## 1. 在位者盘点表(全形态逐个查证)

### 1A. 商业 / 老牌工具

| 工具 | 形态 | 价格(证据日 2026-10-09) | 活跃度(带日期) | 对「收→暂存→一次拖出」覆盖 | 证据源 |
|---|---|---|---|---|---|
| **DropIt** | 悬浮图标 + **规则引擎自动归类**(拖上→按规则 move/copy/compress 到目标目录,21 种动作) | 免费(donation,SourceForge) | **死**:末版 v8.5.1 **2018-02-10**(SF RSS);官网 dropit.com DNS 解析失败(HTTP 000) | **不覆盖**。拖上即触发规则分流,无「暂存→再拖出」环节;是「分类器」不是「暂存架」 | sourceforge.net/projects/dropit/ RSS 2018-02-10 |
| **Direct Folders** | 资源管理器**文件夹导航加速**(收藏夹快开、深层目录一键跳) | **Pro $20**(buyout,purchase 页 2026-10-09);有免费版 | **半死**:末版 4.3.5 **2024-12-24**(CodeSector news 栏) | **不覆盖**。加速「去哪」,不解决「先放哪」 | codesector.com/products / purchase |
| **TeraCopy** | 文件**拷贝/移动校验队列**(替换 Explorer 复制) | **Pro $25**(buyout) | **活**:4.0.3 **2026-07-14** | **不覆盖**。优化「A→B 点对点搬运」,无中转暂存位 | codesector.com/products |
| **Stardock Fences 6** | **桌面图标栅栏分区**(桌面分区块收纳图标,Folder Portals/Peek) | JSON-LD `price content="9.99"`(起价,2026-10-09;purchase 页 307 跳转未拉到完整价目) | **活**:Fences 6 为当前主推产品线(stardock.com/products/fences) | **不覆盖**。静态桌面分区收纳,不是「拖入暂存→拖出投递」 | stardock.com/products/fences/ JSON-LD |
| **Files App** | 现代资源管理器**替代品**(完整文件管理器) | 免费(MS Store/GitHub,45871★) | **活**:files-community/Files pushed **2026-10-09** | **不覆盖**。是「更好的浏览/移动」工具,仍缺「合流暂存」环节 | gh api repos/files-community/Files |
| **Dropshelf(在位付费对照)** | **浮动/摇鼠标召唤置顶架**(shake to open shelf,拖入暂存,拖出投递) | 免费层(**3 秒 wait timer**,「forced waiting time of 3 seconds before you can add more items」);Pro=**一次性应用内买断**(具体价 JSON-LD 只给 `price:0` 基础包,Pro 价未在页内暴露——记取证局限) | **活**:issue 区至 2026-10-05 仍有新帖(R2 卷);商店 rating **4.7/5,10 个评分**(2026-10-09 拉取) | **全链覆盖**(付费解除限速) | apps.microsoft.com/detail/9MZPC6P14L7N JSON-LD(2026-10-09) |

### 1B. 手搓簇 · R2 原名单 6 仓(复核 + 修正)

| 仓库 | 建仓 | ★(10-09) | 最近推送 / release | 形态 | 全链覆盖 | 备注 |
|---|---|---|---|---|---|---|
| iamzubin/holdem | 2024-09-29 | **152** | pushed **2026-09-24**;release 3.1.1 **2026-09-04** | Tauri 浮动暂存架 | 是 | 「The free Dropover for Windows」;点名打 Dropshelf wait timer |
| aungkokomm/Bucket | 2026-06-20 | 14 | pushed **2026-07-03**(停 3 个月) | C#/.NET10 WinUI3 置顶桶,支持文件/文件夹/文本/图片/链接,copy/move/zip/拖出 | 是 | 描述即全链;「On Mac, people happily pay for Yoink/Dropover」出生故事 |
| w1lls47/ghost-drop | 2026-07-01 | 0 | pushed **2026-07-02**(仅 1 天) | JS 无状态架,退出自动清 | 是(轻) | 「ready to sell」;周末作品级 |
| ophilbert/shelf(Mac) | 2026-02-25 | 0 | pushed **2026-02-25**(建仓当日) | Swift vibe-coded Mac 夹 | 是(轻) | 「Vibe coded」自述;一天作品 |
| andreas-bylund/omarchy-ledge | 2026-08-20 | 11 | pushed **2026-09-08** | Linux Omarchy bar 挂载 drop zone | 是 | **Linux 专用**,Windows 无关 |
| Roshan-R/PyDrop | 2021-06-12 | 36 | pushed **2025-09-12** | Python Dropover 替代 | 是 | 「A free and opensource alternative to Dropover」;老,佐证形态早有人想要 |

> R2 卷备注:ghost-drop 记 2026-07,本次 gh 核实为 w1lls47/ghost-drop(2026-07-01 建,0★)。kasodeep/ghost-drop(2024,Java)是「文件传输 CLI」,**形态不符,排除**。

### 1C. 手搓簇 · R2 漏扫补录(本轮新扫出,D2 增量)

| 仓库 | 建仓 | ★(10-09) | 最近推送 / release | 形态 | 全链覆盖 | 备注 |
|---|---|---|---|---|---|---|
| **GameGodS3/DropPoint** | **2021-06-16** | **1094** | pushed 2026-08-04;**末次正式 release v1.2.1 = 2022-09-12** | Electron 浮动架;Shift+CapsLock 鼠标处召唤新实例;托盘常驻;跨虚拟桌面粘滞 | **是(原句级)** | GPL-3.0;「inspired from Dropover」;Win/Linux/Mac;27 open issues;**发版停在 2022,源码 2026 仍有提交 = 维护态非功能态** |
| **Deepender25/Edge-Drop** | 2026-07-09 | **632** | pushed 2026-09-27;release v0.3.2 **2026-09-26** | **屏幕边缘 hover 弹出架**(click-through 透明贴边),收 text/image/files/links,多选拖出/堆叠合并拆分 | **是**(文件+剪贴板混合) | 「100% free and open-source forever」(Ko-fi/UPI 赞助);MS Store 9P3JMHN9M4NR;**3 个月 632★ = 冲星最快**;定位「clipboard shelf + desktop file-transfer hub」 |
| **kobaltgit/StashIt** | 2026-09-06 | 1 | pushed **2026-10-06** | Rust+Tauri v2+Svelte5 浮动架;<25MB RAM;摇鼠标/热键/双击修饰键/贴边抽屉;多架 Tab;Wi-Fi 手机互传 | **是**(最全) | **winget install kobaltgit.StashIt**;俄英双语;对标表把 DropPoint(150–300MB)与 Dropshelf 免费层同时列为可打对象 |
| **ais7896-hue/KyteShelf** | 2026-09-21 | 0 | pushed **2026-10-08** | Python 浮动置顶架;摇鼠标召唤(与 Dropshelf 同手势);Ctrl+V 入架;多架命名;**Outlook 附件一键挂载**;一键 zip | **是** | 繁中作者;「多檔案集中暫存:多次收集不同路徑的檔案,再一口氣拖曳釋放」= 全链原句 |
| rwrife/drop-shelf | 2026-08-25 | 0 | pushed 2026-09-02 | C# 置顶临时架,files/text/URLs,Win+Mac | 是 | local-first 表述 |
| manish10079/-droppointplus | 2026-08-10 | 0 | pushed 2026-08-23 | Python,DropPoint+fork(「keeps the exact workflow of the original」) | 是 | DropPoint 生态衍生 |
| kerimhypr/Droppoint | 2026-08-18 | 2 | pushed 2026-09-04 | (描述空) | ? | 另一 DropPoint 同名新生 |

> 另见 SEO 垃圾仓(Yoink-Mac-X/Yoink-Mac、Yoink-Mac-Y/.github、Dropover-Drag-Shelf/.github,2026-03~07 建,0★,描述含拼写错误「Hld」),判营销农场,**不计供给**。

**R2 漏扫率**:原手搓 6 仓 → 本轮同主题 GitHub 搜索共出 **≥13 个真实同形态仓**(不含垃圾),R2 捕获 6/13 ≈ 46%。漏网里含 1094★ 与 632★ 两个高星仓——**证伪了「手搓簇都是小星星」的印象**,必须进在位者视野。

---

## 2. 全链覆盖矩阵(收 → 暂存 → 一次拖出)

| 工具 | 收(多源丢入) | 暂存(指针驻留/置顶) | 一次拖出(批量投递) | 全链? | 形态位 |
|---|---|---|---|---|---|
| DropIt | ✓ 拖上图标 | ✗ 立即规则分流 | ✗ | **否** | 分类器 |
| Direct Folders | ✗ | ✗ | ✗ | **否** | 导航器 |
| TeraCopy | ✗ | ✗ 拷贝队列 | ✗ | **否** | 拷贝校验 |
| Fences 6 | ✗ | 半(桌面分区持图标) | ✗ | **否** | 桌面栅栏 |
| Files App | ✗ | ✗ | ✗ | **否** | 文件管理器 |
| **Dropshelf 免费层** | ✓ | ✓(受 3s 限速) | ✓ | **是(带节流)** | 浮动架 |
| **DropPoint(1094★)** | ✓ | ✓ | ✓ | **是** | 浮动架(Electron) |
| **Edge-Drop(632★)** | ✓(文件+剪贴板) | ✓ | ✓ 多选 | **是** | 贴边 hover 架 |
| holdem(152★) | ✓ | ✓ | ✓ | **是** | 浮动架(Tauri) |
| StashIt | ✓ | ✓ 多架 Tab | ✓ + 批量把手 | **是** | 浮动架(Rust) |
| KyteShelf | ✓ 多路径 | ✓ 多架命名 | ✓ + zip/Outlook | **是** | 浮动架(Python) |
| Bucket | ✓ | ✓ | ✓ + zip/move | **是** | 置顶桶(WinUI3) |
| ghost-drop / drop-shelf / DropPoint+ / shelf / omarchy-ledge / PyDrop | ✓ | ✓ | ✓ | 是(轻/平台限定) | 各形 |

**「谁在给谁擦屁股」(行为证据,非言词)**:

1. **手搓品公开点名给 Dropshelf 免费层擦屁股**:holdem README:「No paywall, no wait timers — unlike Dropshelf free tier or macOS-only Dropover」——免费层限速直接催生替代品。
2. **手搓品给「Windows 没有 Yoink/Dropover」擦屁股**:Bucket「On Windows… most free ones are outdated. So I built one myself」;StashIt/KyteShelf 自我定位「Dropover/Yoink alternative」;DropPoint README「inspired from Dropover」。
3. **DropPoint 给「PowerToys 不做」擦屁股**:PowerToys issue **#128「Temporary File Bin」,open 自 2019-05-29,6 条评论,至今 open**(2026-10-09 查证)——原帖即「drag a file in corner of screen to keep a reference… open or copy or drag and drop later」,并点名 Unclutter(Mac)。**七年 feature request 零落地 = 官方不接,民间自建**。
4. **2026 新仓给 DropPoint 的 Electron 重壳擦屁股**:StashIt README 对标表逐条列 DropPoint「150–300MB RAM / 1.5–2.5s 冷启动 / >80MB 安装包」,自己打「15–20MB / ~50ms / 4.5MB」。
5. **DropPoint+ 给 DropPoint 发版停更擦屁股**:DropPoint 末次 release 2022-09,DropPoint+(2026-08)「keeps the exact workflow of the original」= 维护断档衍生。
6. **教程/请愿行为**:PowerToys #128 六条评论是典型「请愿+互相推荐 Unclutter/Dropover」现场;Dropshelf issue 区 Pro 买家吐槽(见 D1)则是「付费后还在给产品擦 bug」。

---

## 3. 终极自检:免费是覆盖全链,还是只覆盖一段?

**答案:全链已被免费覆盖——但没有一个免费者同时做到「成熟 × 轻量 × 活跃发版 × 浮动置顶架形态」。**

- **全链免费者**(DropPoint/Edge-Drop/holdem/StashIt/KyteShelf/Bucket…)功能面都过线。
- **缺口在完成度与产品化**:
  - DropPoint:星最多(1094)、最老(2021),但 Electron 重壳 + **release 停更 4 年**(2022-09)。
  - Edge-Drop:最冲星(3 个月 632★),但形态是**贴边 hover**,不是「浮动置顶小窗」;且定位剪贴板优先(触 clipboard 坟邻接,见 §4)。
  - StashIt/KyteShelf:功能最全、2026-10 仍在推,但 **0–1★**,从未被大规模验收;KyteShelf 还要 Python 3.10+ 环境,不是「单文件便携」。
  - holdem(152★):最像「成熟便携位」,Tauri,2026-09 还发版,但功能面窄于 StashIt/KyteShelf。
- **免费层经济**:Edge-Drop 明示「100% free forever」(捐赠);DropPoint GPL;holdem「No paywall」;Bucket 开源。**没有任何一家免费者在做 freemium/买断**——Dropshelf Pro 是**唯一**在收「暂存架形态」钱的产品(D1 挖到 5 例 Pro 自白)。
- **推论(供裁决,不作结论)**:若本品进,差异化不在「做免费的」(免费已有 8 家),而在「做得像样 + 单文件便携 + Pro 收口」——这恰好是 Dropshelf 的打法,即与 Dropshelf **正面对位**,不是填空。

---

## 4. 坟头邻接复查(书面界线判断)

### 4A. vs **tile 坟**(窗口布局策略引擎 = tile 专属禁区)

| 维度 | tile(坟) | 本品 Drop Shelf | 界线判断 |
|---|---|---|---|
| 操作对象 | **应用窗口**(位置/尺寸/分区) | **文件/文件夹指针**(暂存清单) | 对象不同,不触 |
| 核心策略 | BSP/Dwindle/Master/Columns **布局策略引擎** | 无布局策略;架只有「置顶浮动 + 列表」 | 不做布局/贴边分区/工作区即不触坟 |
| 窗口行为 | 打开即归位/关闭即重排(自动平铺) | 仅 1 个置顶架窗口,用户手动拖放 | 不含自动窗口管理 |
| 多显示器 | 逐屏任务栏/工作区(遗留失效标志) | 与本品无关 | 不触 |

**界线一句话**:本品**只移动文件指针,不移动窗口**;明文不做「窗口布局策略 / 贴边分区 / 工作区」。技术上可复用 danqing 置顶悬浮窗肌肉(引擎复用≠产品撞坟)。**判:不触 tile 坟。**

### 4B. vs **clipboard 坟**(历史管理)

| 维度 | clipboard(坟) | 本品 | 界线判断 |
|---|---|---|---|
| 本体 | 剪贴板**历史管理器**(Win+V 竞品死因) | 文件暂存架(拖放指针) | 对象不同 |
| 历史语义 | 捕获一切 copy 流,回溯/搜索/paste | 无历史流;只有「当前架上有什么」 | 不做历史管理 |
| 激活 | 剪贴板变化钩子 | 拖入/摇鼠标/热键 | 不触 |

**风险点(须写进卡)**:Edge-Drop(632★)自定位「clipboard shelf + file-transfer hub」,KyteShelf 有「Ctrl+V 入架」,StashIt 收 text/snippets——**免费在位者已把剪贴板当附件功能**。本品若跟进「剪贴板历史/回溯」= 触 clipboard 坟;**只允许「Ctrl+V 把当前剪贴板内容当作一条暂存项入架」这种单步转存,不允许历史列表**。
**界线一句话**:无历史、无 Win+V 对抗、无粘贴回溯 = 不触 clipboard 坟。

### 4C. vs **round8 粘贴清洗挂钟**(c+ 体裁落地层,不碰)

- 挂钟范围:「粘贴时转换 / 清洗 / 外呼」(paste-time transform/clean/egress)。
- 本品动作是**拖放搬运文件指针**,不涉及文本粘贴转换、不涉及格式清洗。
- **界线一句话**:不碰文本粘贴管道;架上文本项只允许「原样拖出/原样复制」,不许长清洗/转换功能。
- **判:不触挂钟。**

---

## 5. 供给空位判断(D2 题 4)

**「单文件桌面 GUI 暂存架」形态位,两侧占座情况(2026-10-09)**:

| 座位 | 占用者 | 占用质量 |
|---|---|---|
| 付费成熟位 | **Dropshelf Pro**(MS Store,Pro 买断,4.7/5×10) | 独占。有真实买家(D1 5 例自白),UI 原生 Fluent |
| 免费头部位 | **DropPoint 1094★**(2021,Electron,release 停 2022) | 有座但「旧且重」;维护态 |
| 免费冲星位 | **Edge-Drop 632★**(2026-07,贴边 hover) | 形态相邻(非浮动窗);剪贴板混合定位 |
| 免费新兵位 | holdem 152★ / StashIt 1★ / KyteShelf 0★ / Bucket 14★ | 功能达标,用户基数≈0;2026-09/10 仍在出生 |
| 免费古老位 | PyDrop 36★(2021)/ ghost-drop 0★ | 形态证据,无威胁 |
| 系统位 | Windows 本体 ✗ / PowerToys ✗(#128 open 7 年) | **空** |
| 老牌工具位 | DropIt(死)/ Direct Folders(导航)/ Fences(栅栏)/ TeraCopy(拷贝) | **形态位全空**(都不做暂存) |

**「谁最像会变成免费在位者」**:

1. **Edge-Drop(632★)——最大威胁**:3 个月 632★、MS Store 已上架、v0.3.2 活跃、功能含多选拖出与堆叠。**若它把「文件暂存」从剪贴板附庸提为主场景**,就会长成 Seelen UI 式封死者。**盯防信号**:在其 README/路线图把 "file" 提级为第一关键词、或发「floating window mode」。
2. **StashIt(1★)——最像完整产品**:Rust+Tauri、winget、多架 Tab、Wi-Fi 互传、对标表写得像产品文档;作者俄语社区分发。星数是唯一短板,一旦被 awesome-list/软文点名会爆。
3. **DropPoint(1094★)——存量最大但钝**:除非作者恢复发版(1.3.x),否则继续吃老本。**盯防信号**:出 1.3 release 或重回活跃发版。
4. **KyteShelf(0★)——差异化最锋利**:Outlook 附件挂载=直指「发件人」画像;但 Python 依赖使其难成便携单文件在位者。

**空位结论(书面)**:形态位**不空**,免费侧已有 8+ 占座者;但**「单文件便携 + 浮动置顶 + 活跃发版 + 产品化 UI」四位一体的免费位仍空**。本品若进,是**与 Dropshelf Pro / DropPoint / Edge-Drop 三家正面竞速**,不是开荒。卡上「免费在位者薄」这句早筛透镜**须改为「免费在位者厚,但质量参差;头名 DropPoint 1094★ 是 R2 漏网」**。

---

## 6. 取证局限(如实记录)

1. **dropit.com / dropit.skwire.net DNS 解析失败**(curl exit 6, HTTP 000);DropIt 事实以 SourceForge RSS/files 为准(末版 2018-02-10)。官网是否复活无法判定。
2. **WebSearch 全程空回显**(仅回显查询词,无结果块)——DropIt/Direct Folders/Fences 的第三方评测与用户口碑未采到,本报告仅用官网/商店/gh 一手源。
3. **Dropshelf Pro 具体价格未拿到**:商店 JSON-LD `offers.price=0` 只标基础免费包;Pro 为应用内购买,页内无价。displayPrice 显示「免费下载」。Pro 价格需 D1 从 Partner Center/购买流补。
4. **Fences 完整价目未拿到**:页面 JSON-LD 只有 `price content="9.99"`(疑似起价/月费);purchase 页 HTTP 307 未跟完。不影响形态判断(Fences=桌面栅栏,非暂存架)。
5. **gh search/repositories 关键词搜索不稳定**:多次返回空数组,后改 `gh api search/repositories?q=` 才出数;**不排除仍有漏网仓**。本次同形态仓盘点应视为下界(≥13),非上界。
6. **Yoink/Dropover 第一方价格/销量未取**(D1 题,非 D2);本报告 Mac 侧仅持第二方转述。
7. **星数为 2026-10-09 快照**,无历史走势序列(gh API 不给 star timeline);「走势」描述均以建仓/推送/发版日期推断。
8. **kerimhypr/Droppoint(2★)描述为空**,未拉 README,形态待核,已标注「?」。

---

## 7. 对卡A 早筛透镜的修订建议(呈裁决)

| 早筛行(R2 原) | D2 复核结果 | 修订 |
|---|---|---|
| 免费在位者薄(Dropshelf 免费层有 wait timer) | **错**。免费在位者**厚**:DropPoint 1094★ 全链 + Edge-Drop 632★ 冲星 + 2026-09/10 新兵 4 家 | 改「免费在位者厚,头名 DropPoint 1094★;但无『便携×活跃×产品化』四合一者」 |
| 失效标志④(DropIt/Direct Folders 查实覆盖同形态且活跃) | **不兑现**(DropIt 死 2018 且形态不同;Direct Folders 形态不同) | 保留,但**加挂**:DropPoint/Edge-Drop 若 ≥500★ 且活跃发版且覆盖全链,应视同「免费在位者查实」触发④类判定 |
| 系统吸收远 | D2 侧证:PowerToys #128 open 自 2019-05-29 至今 | **支持「远」**(官方 7 年不接) |

---

*D2 完。取证窗 2026-10-09;数据快照同日。*
