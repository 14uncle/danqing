# Drop Shelf · 交接文档(随时开工包,2026-10-09)

> **状态**:开工许可证·条件挂起(round9 二次终判过,见 `deep-dive-round9-t1-2026-10-09.md`)。
> **开枪闸**:组合纪律 = log v1.x 发布后自动开;**用户显式口令「开工」= 提前开闸**(用户本人可开,round9 空窗裁决推翻先例)。
> **新会话接手顺序**:读本文 → 需要证据细节时读深卷(`deep-dive-round9-t1-2026-10-09.md`)及五份原始报告 → 按 §6 开工流程走。**不写代码前先过 §6 checklist**。

## 0. 一句话

**完整版拖拽暂存架**:从各处(邮件附件/U盘/归档/聊天)收文件上架、一次拖出到目的地,替代多开资源管理器互拖。以「买后缺件清单」当 spec,以「不丢文件 / 装得开 / 权益终身」当承诺,便携单文件、买断制(¥18–28 假设,待校准钟)。

定位口诀(对三家):**Dropshelf 有钱但体验债缠身(装不开/丢文件/权益丢)、DropPoint 全链但停更、Edge-Drop 热但形态偏(贴边混合架)——我们卖「把事办利索」。**

## 1. 产品定义(候选卡 v2 摘要,全文在深卷「降级修补执行」节)

- **需求(已降级改写)**:暂存架「形态」已被免费工具覆盖;未满足的是「把这个形态办利索」——付费买家掏钱仍缺件(12 主题)+ 免费品质量不足(停更/重壳/数据丢失)。
- **谁还在凑合(可指名,带日期)**:① Dropshelf Pro 买家 10 笔自白(2023-10→2026-08,含「I PAID… Not paying twice!」权益丢失、Dropover 迁移者「I paid for Pro immediately」);② DropPoint 用户(DropPoint+ 给 2022 停更擦屁股);③ PowerToys 请愿者(#128 七年链 + dark13b 自写 MVP 规格零回复);④ Mac Dropover 买家(「I bought it exclusively for cloud links」买后缺件)。
- **需求温度**:活(2026 年 6 个月付费事件,近 30 天 3 新评);警讯:近 30/7 天均分 3.7/3.0 低于全时 4.3。
- **生意层**:粗算直接收入期望 ≈¥0–21/月(第一方转化 0.1–0.5%)——**需求为真、生意薄**,本品价值 = 判断力 + 拖拽壳层肌肉 + 第二果树试验;算账基准 10-18/10-21 校准钟回填。

## 2. v1 范围(铁三角 + 清单)

**三承诺(USP,全部来自买后缺件/体验债证据)**:
1. **缺件清单当 spec**(Dropshelf 买家点名要的,我们原生有);
2. **质量承诺**:不丢文件(反 William .bmp / 0KB 空文件)、装得开(反「装完打不开」×4)、**权益一次买断终身**(反 Dan「Not paying twice」/IAP 不披露);
3. **便携单文件**(反 DropPoint Electron 重壳)。

**v1 功能(按缺件 12 主题对位)**:
- 核心闭环:悬浮置顶架(拖入收集 → 批量拖出到目的地)+ 多目的地预设 + 架持久化(Dropshelf Pro 才有的「Save shelves」我们放进本体)
- 缺件清单进 v1:修饰键激活(hold-to-show,Alex「加了就改 5 星」)/ 永久架 + 分组 + Aero Snap 兼容(买家 7 条)/ 批量选择 / Move-vs-Copy 切换 / 粘贴图片
- 明确不做(坟头与挂钟界线,D2 书面判过):剪贴板历史(clipboard 坟,仅允许 Ctrl+V 单步入架)/ 窗口布局·贴边分区·工作区(tile 坟)/ 文本格式清洗(粘贴清洗挂钟)
- **v1.x(明确推迟)**:Outlook 附件上架(CFSTR_FILEDESCRIPTOR 虚拟文件 OLE 坑,红队 A4:含此场景工时翻倍)/ 文件夹监视 / 跨设备同步 / 企业无商店购买渠道(L8:渠道形态本身是付费漏损点)

## 3. 引擎适配(实现注记,不作选型依据)

- **有**(danqing 现成):置顶悬浮(`WindowConfig.topmost` + `set_topmost`)/ 托盘 / 热键 / 剪贴板(`clipboard/`)。
- **缺(唯一缺口肌肉:拖拽壳层)**:拖入 = `WM_DROPFILES`(易,1–2 人日);拖出到资源管理器 = OLE `IDropSource`(中,2–3 人日);Outlook 虚拟文件(难,v1.x)。
- **可复用**:licensing 肌肉(log v1.x 许可链路,对位「权益终身」承诺)/ Theme token(不自定义颜色字体)/ `package_portable.ps1` 打包链。
- 性能门槛沿用 farm:启动 ≤1s、常驻 WS ≤360MB。

## 4. 证据基座(指针)

- **全卷**:round9 扫描卷 `round9-selection-2026-10-09.md` + 深卷 `deep-dive-round9-t1-2026-10-09.md`;原始报告五份 `round9-deepdive-D1/D2/D3-…` + `round9-deepdive-L6-…` + `round9-deepdive-redteam-…`。
- 核心事实:买家自白 10 笔(issue 区)/ 商店差评正文 25 条(L6 破墙,退款潮=已扫为无)/ Mac 同刀法双侧演化(Dropover 试用到期 3 秒惩罚 = Dropshelf 免费层 3 秒惩罚)/ PowerToys 七年请愿链 / 手搓簇 ≥6 仓。
- **诚实栏(弱点,开工后不许忘)**:生意 ≈0;Edge-Drop 窗口在关;收入级数字与 Pro 标价双侧全缺;近 30 天均分走低;R2 扫描漏扫 54% 的教训(供给盘点必须二次穷尽)。
- **取证手筋**:商店评论 API `GET /api/products/getReviews/{id}?orderBy=4&…&hl=en-us&gl={market}`(必须带 hl+gl;**2026-10-09 实测 host 更新:storeedgefd.dsx.mp.microsoft.com 已 404,改走 `apps.microsoft.com/api/products/getReviews/`**);gh 全路径 `"C:\Program Files/GitHub CLI/gh.exe"`。

## 5. 竞争与盯防(开工后常态化)

| 对手 | 态势 | 动作 |
|---|---|---|
| **Edge-Drop** 632★ 贴边混合架 | **头号**;3 个月冲星,09-26 仍发版 | **出浮动置顶模式 = ④'a 判毙扳机**,盯 release notes |
| DropPoint 1094★ | 发版钝(末版 2022-09) | 复活 1.3 发版且产品化 = ④' 重评 |
| Dropshelf(在位付费) 4.3★/122 | 体验债缠身 = 我们的入口 | 盯缺件修复节奏(④'b) |
| StashIt / KyteShelf | 功能全/新生,无声量 | 季度扫一眼 |
| PowerToys #128 | 七年零落地,架构未裁 | 出实现 PR = ④'d 重评 |

## 6. 开工流程(顺序执行,不许跳步)

1. **闸确认**:用户口令「开工」,或 log v1.x 已发布(组合纪律自动开)。
2. **失效标志全档复盘**(纪律:开枪前必扫):④' 四条 + 多显示器两条 + c+ 探针(90 天三中二即杀)+ Notepad「官方可能已修」+ 分镜 K1/K2——兑现即判证伪,先复盘再建仓。
3. **建仓六件套**(farm01 CLAUDE.md「打包」节):从 danqing-clipboard 复制 `tools/package_portable.ps1`(改 `$BinaryName`/`$OutDir`)→ 复制 `.cargo/config.toml`(不含 target-dir)→ `src/main.rs` 加 `#![windows_subsystem = "windows"]` → `assets/logo.ico` → `rustup override set stable-x86_64-pc-windows-gnu`(**第一件事**,本机 msvc 不可用)→ git 身份 `十四叔 <gwhun@qq.com>`。
4. **spec 流水线**(强制五阶段):spec → plan → build → review → code-simplify;spec 里写死定位口诀(§0)与三承诺(§2);review 阶段用 `/agent-skills:code-review-and-quality`。
5. **发布链**:MS Store + GitHub 双轨(免费本体 + 买断 add-on,参照 pomodoro/log;**奶牛禁令**:不订阅);发布 checklist 含信号埋点(impressions/traffic/下载),30 天校准钟机制照 farm 惯例。
6. **测试纪律**:测试严禁真实桌面副作用(拖拽/SendInput 类测试必须 mock)。

## 7. 相关文档

- 意图与漏斗:`round9-selection-2026-10-09.md`;深调研:`deep-dive-round9-t1-2026-10-09.md`
- 原始报告:`round9-deepdive-D1/D2/D3/L6/redteam-2026-10-09.md`;扫描原文:`round9-scan-R1/R2/R3-2026-10-09.md`
- 上游方针:农场根 CLAUDE.md「产品心法」两条 + 调研三闸;十年愿景 `farm-10yr-vision.md`
- 范式参考:`deep-dive-round8-t1-2026-09-25.md`(深调研卷式样)
