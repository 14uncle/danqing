# round9 · b1 补证 agent 报告:IM 托盘闪烁/悬浮卡片反打扰 · 外扩第二现场(2026-10-09)

> 使命:验证「IM 托盘闪烁/悬浮卡片反打扰」现象是否多现场成立。75 分钟硬时间盒(取证约 60 分 + 写盘)。
> 原卡:round9-scan-R3-2026-10-09.md 卡3(立案·弱);软肋=单帖集群 + 零付费 + 系统吸收 40-60%。
> 去重名单未命中(「Copilot 渗入对抗」为相邻现象已在册注明;明暗切换/文本扩展不同域)。

---

## 一、新现场逐条(原文 + 日期 + URL)

### 现场 B · appinn 72085「怎么让微信来信息时任务栏图标不泛红」(2025-06-08)

- URL: https://meta.appinn.net/t/72085 (7 帖 / 851 浏)
- 原文(楼主 4913,2025-06-08):「如图当微信开着的时候来信息会发红光,请问有什么办法可以解决,设置里面关闭消息通知声音不行」
- 凑合行为:
  - NeverMind(06-08):「这不是 windows 的机制嘛……可以考虑看看 windhawk 那种劫持的方案」
  - VictoryRoad(06-10):「没用的,我试过,无论用什么主题颜色,来通知的时候任务栏图标底色都是红的」
  - kimpvp(06-13):「右键任务栏>任务栏设置>任务栏行为中取消勾选:在任务栏应用上显示徽章 / 显示任务栏应用上的闪烁。win11已测试过,不会泛红。**缺点是来消息时要看托盘区**」
- 判读:现象面=微信来消息任务栏红光/闪烁(与托盘闪烁同根:IM 自绘通知不走系统);**凑合链清晰**——OS checkbox 只管任务栏,托盘成剩余缺口(「要看托盘区」);Windhawk 被点名为劫持方案。日期在窗(2025-06)。
- 同站不同帖、不同人(与 91522 无当事人重叠)。

### 现场 C · appinn 72938「如何让 Windows 任务栏在应用有消息通知时闪烁 3 秒后自动隐藏?」(2025-07-06)

- URL: https://meta.appinn.net/t/72938 (4 帖 / 341 浏)
- 原文(楼主 windwhim,2025-07-06):「Windows 任务栏隐藏后,当应用(**微信**、Zotero)有消息通知时会再显示任务栏,并且**应用图标一直闪烁,必须打开窗口再关闭才能继续隐藏**。」
- 同人补帖:「windows 有选项可以设置全部不闪烁,但这样会导致消息无法看到。希望有闪烁后自动隐藏的软件」
- 凑合行为:「目前只找到一个现成的解决方案: windhawk 中的 mod, **Better Taskbar Autohide**」
- 判读:现象面=IM 来消息图标持续闪、打断自动隐藏任务栏工作流;需求面与 91522 同族但目标不同(要「闪一下再收」而非「彻底不闪」)。**行为证据硬**:OS 全禁闪 checkbox 存在但「消息无法看到」→ 用户拒绝一刀切 → 找 Windhawk mod 凑合。日期在窗(2025-07)。
- 注意:本帖需求方向是「选择性闪烁管理」,与 91522「保图标、掐闪烁」是同一现象的两种裁法,补证时不能直接合并计数为同一需求,只能算同现象多现场。

### 现场 D · Super User 1712699「How to block all vendor-app notification and bottom-right corner pop-ups of Windows (11) Desktop?」(2022-03-25)

- URL: https://superuser.com/questions/1712699 (score 1 / 1 答)
- 原文:「They are very distractive. I want to totally silence apps running in the tray or background… **I don't want any other software to change any pixel of my screen.**」
- 关键行为句:「I've tried almost all the popular methods of disabling all notifications and set-up quite hours… **They are not effective: windows system stops to give notifications, but other apps, such as QQ, Wechat, and occasionally Skype, persist to give notifications.**」
- 回答承认缺口:「There is no single Windows setting that turns off all Vendor App notifications. Vendors can do as they wish here.」
- 判读:**跨站第二现场**(英文 Super User,点名 QQ/WeChat),与 91522「基本所有通讯软件都没接入 Windows 通知」同构。但**日期 2022-03,落在「近两年(2024-10 起)」取证窗外**——只能作历史跨站佐证,不能计入窗内三来源。

### 现场 E(反面/邻面,记谱不计数)· appinn 23619「如何把微信、Tim 通知收进 win10 的通知中心」(2021-05 开帖,2024-03 仍在)

- URL: https://meta.appinn.net/t/23619 (9 帖 / 4920 浏)
- 2024-03-08 InifintyJustice:「3 年了,挖个坟。一种解决方案是,用类似手机连接之类的应用把手机微信的通知转发到电脑。」
- 同日 schuh(楼主):「**实际上,我还没解决这个问题**」
- 凑合土法:MyDock/MyFinder 仿 Mac 通知、手机通知转发、WSA 装安卓微信让子系统同步到通知中心。
- 判读:**需求方向相反**(要把微信收进通知中心,不是掐闪烁),但本质同一条:IM 不走 Windows 通知系统。2024-03 仍在未解决 + 多土法 = 现象活着的旁证。**不计入「反打扰」三来源**,防止灌水。

### 现场 F(供给端,非需求现场)· Windhawk 模组缺口复核(2026-10-09 取证)

- NoFlashWindow(m417z,`ramensoftware/windhawk-mods/mods/no-flash-window.wh.cpp`):README 明文「Prevent programs from flashing their windows **on the taskbar**」,hook 的是 `FlashWindow`/`FlashWindowEx`——**任务栏窗口闪,不是托盘图标闪**;README 还注明 Win11 原生已有任务栏禁闪选项。
- Disable Tray Icons(aubymori,`disable-tray-icons.wh.cpp`):`Shell_NotifyIconW/A` hook 直接 return TRUE——**整个托盘图标干掉**,不是「保留图标但不闪烁」。
- 结论复核了原卡软肋中的供给判断:**现成免费模组 = 全隐藏 or 只管任务栏,没有「保图标、掐闪烁、杀悬浮卡」中间态**;72938 用户实际用的是 Better Taskbar Autohide(任务栏自动隐藏增强),仍非托盘禁闪。
- 附:windhawk-mods issues 无「托盘禁闪」求装贴(title:flash 检索 21 条全为 explorer 白闪/CE 按钮闪/任务栏揭示等)。模组下载量因 windhawk.net 为 SPA + WebFetch 域名拒、mods API 未公开,未取到,记取证局限。

---

## 二、付费证据单列

| 证据 | 日期 | 形态侧对位 | 级别 |
|---|---|---|---|
| appinn 36006 帖内 schuh:「完美错过 前几天刚买了 adGuard」 | 2022-09-08 | **不对位**——AdGuard 是广告拦截,不是「桌面 GUI 反打扰/托盘通知管理」 | 买断自白(邻品) |
| 其余(HN「I would pay」/ SE 付费专注 / KillAds 用户付费 / Windhawk 捐赠) | — | — | **零命中** |

- **结论:形态侧付费对位 = 0 笔**。未见任何「为反打扰/通知管理桌面工具掏钱」的买断自白;AdGuard 一笔是广告拦截邻品,不能算。
- 付费检索局限:HN Algolia「I would pay」/「bought notification」两轮 0 命中;Cold Turkey 官网 SSL 复核失败(CRYPT_E_REVOCATION_OFFLINE);MS Store 差评 API 本轮未开(与 R1' 已记欠账同源)。**不得据此断言「付费不存在」**,只记「本轮可达源内未挖到」。

---

## 三、免费在位者复核(谁覆盖「保图标、掐闪烁、杀悬浮卡」全链?)

| 在位者 | 覆盖面 | 证据日期 | 是否全链 |
|---|---|---|---|
| Windhawk NoFlashWindow | 任务栏窗口 FlashWindow 禁闪 | 模组现版(仓在 2026-10 活跃) | **否**(不碰托盘) |
| Windhawk Disable Tray Icons | 按进程干掉全部托盘图标 | 同上 | **否**(图标也没了) |
| Windhawk Better Taskbar Autohide | 任务栏自动隐藏行为修正 | 72938 用户 2025-07 在用 | **否**(任务栏侧) |
| 火绒弹窗拦截 | 弹窗类 | 91522「火绒弹窗拦截都管不住」2026-09 | **否**(悬浮卡管不住) |
| KillAds / 广告拦截助手 | 自述「支持拦截 mini 页、**右下角托盘图标闪烁**、锁屏弹窗」 | appinn 36006 2022-09 自荐;4495? 未见后续活跃证据 | **声称覆盖托盘闪烁**,但 2022 后无活跃/无用户复证;针对广告弹窗场景非 IM 消息场景 |
| AHK | 杀悬浮窗 / Windows Spy / Focus Assist 定时 | 91522 Fireman;21552 dms;49428 dog | 半覆盖(土法,须手搓) |
| Windows 任务栏禁闪 checkbox | 任务栏闪烁 | 72085/72938 实证有效 | **否**(托盘无对应 checkbox) |
| PowerToys | — | issue 检索无托盘闪/通知管理项 | 否 |

- **擦屁股关系**:Windhawk 社区在给 Windows 与 IM 擦屁股;AHK 在给火绒与 Windhawk 擦屁股(91522);KillAds 试图给「右下角托盘闪烁」擦屁股但似已停在 2022。
- **全链空缺仍在**:「保留图标 + 掐托盘闪烁 + 杀悬浮卡」的中间态,免费在位者无人稳定提供。这是原卡最强的资产,本轮复核成立。

---

## 四、系统吸收距离复核

- **已吸收(半)**:Windows 11 任务栏「显示任务栏应用上的闪烁」checkbox 存在且有效(72085 2025-06、72938 2025-07 双帖实证);NoFlashWindow README 亦引 ElevenForum「Enable or Disable Show Flashing on Taskbar Apps」。
- **未吸收**:托盘区禁闪 checkbox 至 2026-10-09 不存在;SE 1712699 回答明文「No single Windows setting that turns off all Vendor App notifications」;PowerToys issue 区无对应特性票。
- **方向观察**:微软在推通知中心统一(23619 用户求收编),但 IM(微信/QQ/钉钉)持续不接入 → 吸收受阻于三方,不是微软 checkbox 一笔能收走。
- **18 个月概率**:维持原卡 **40-60%**。理由:①任务栏禁闪 checkbox 已表明「闪一下烦」是微软认账的品类;②托盘禁闪是同族小步;③但 IM 自绘悬浮卡/托盘闪烁根本不在通知中心管辖内,即便微软加 checkbox 也只解决「闪」不解决「悬浮卡」,全链吸收概率低于半吸收。若吸收兑现 = 原卡失效标志①/③之一。

---

## 五、取证局限(本轮)

1. WebSearch 全程空转(返回空壳);Windhawk.net 为 SPA,WebFetch 域名验证拒,mods API 未探测到,模组下载量/评论未取到。
2. ElevenForum 403 + Cloudflare「Security Check」;TenForums/Reddit/DuckDuckGo/Cold Turkey 超时或 SSL 失败;V2EX 未试通(往轮 000);吾爱破解搜索被 Bing site: 登录墙拦截;知乎/小红书零覆盖。
3. Bing(cn)对中英文技术查询均退化为词典/官网导航结果,不可用作论坛发现通道。
4. SE API 可用但「托盘闪烁」类低分问题稀少,窗内(2024-10 后)0 命中——可能是真稀少,也可能是问法分散(blinking/flashing/annoying/popup),不作「现象不存在」推论。
5. GitHub 全局 issue 搜索 spam(开发侧 tray flashing bug 修复票)淹没需求侧;定点 `gh api repos/...` 可靠。
6. 付费侧栖息地(商店差评/闲鱼求购)与 R1' 同源整级墙,本轮未重开。

---

## 六、判别对照(便携版)

- **谁还在凑合?**:办公用户(微信/QQ/钉钉消息打断)。凑合行为=OS checkbox 只关任务栏闪留托盘(72085)、Windhawk Better Taskbar Autohide(72938)、AHK 杀弹窗(91522)、手搓 suppress-tray-flashing mod(91522)、拒绝全禁闪因「消息无法看到」(72938)。
- **证据强度**:窗内(2024-10 起)同站三帖(91522/72085/72938)+ 反面一帖(23619 至 2024-03 未解决)+ 跨站老帖(SE 2022-03)+ 供给端缺口(Windhawk 两模组边界清晰)。**独立站点数=2**(appinn + Super User),窗内独立站点=1。
- **付费**:零;形态对位不成立。
- **温度**:活(2025-06 / 2025-07 / 2026-09 连续有新帖);无衰减迹象。
- **去重**:未命中;与 round8「Copilot 渗入对抗」不同域(那是系统功能渗入,这是 IM 通知劫持),已在卡内注明关系。

---

## 七、判定

**维持弱(立案·弱不升不毙)**。

依据:
1. **不升强**:严格「多现场」口径未达标——窗内新需求帖仍集中在 meta.appinn.net 单站(72085/72938/91522);跨站(Super User)证据超窗(2022-03);付费对位 0 笔;「保图标、掐闪烁、杀悬浮卡」完整链的当事人仍基本在 91522 一帖。把同站邻面帖(任务栏红光/自动隐藏)灌成「第三现场」属于凑数。
2. **不毙**:现象活着且复现(2025-06 → 2025-07 → 2026-09 三连);免费在位者全链复核空缺成立(Windhawk 两模组边界实测:任务栏-only / 全隐藏-only);凑合行为从「OS checkbox 剩托盘」到「Windhawk 凑合」到「AHK+AI 手搓」层次齐全;失效标志③(2026-12 前同类求助帖不再出现)未兑现——反而在兑现其反面。
3. **相对原卡的增量**:①同站复现从 1 帖升到 3 帖(跨 2025-2026);②跨站英文佐证(Super User 1712699);③供给端缺口从「原卡断言」升为「源码级实证」(两模组 hook 面与 README 原文);④系统吸收半吸收实证(任务栏 checkbox 有、托盘无,双帖亲测)。
4. **升强还缺什么**:窗内第三独立站点的需求现场(知乎/吾爱/Reddit/ElevenForum 任一,带凑合行为),或一笔形态对位付费(「为反打扰桌面工具掏钱」),任一到手再议。

---

## 附:更新后的候选卡 8 项全文

### 卡3 · IM 托盘闪烁/悬浮卡片反打扰(维持立案·弱)

**1. 需求**:禁止微信/QQ/钉钉等 IM 接收消息后闪烁托盘图标、弹跟随鼠标的悬浮卡片(保留图标可点开)。

**2. 谁在凑合**:被 IM 通知劫持注意力的办公用户。凑合行为 = ① AHK 无脑杀弹窗;② 火绒弹窗拦截管不住后放弃;③ 找 Windhawk 模组 + 用 AI 手搓 mod(三版迭代);④ OS 只关任务栏闪烁、剩余问题留给托盘(「缺点是来消息时要看托盘区」);⑤ 用 Better Taskbar Autohide 等任务栏 mod 凑合 IM 图标常闪;⑥ 对着 Win32 API 现学现改。

- ① appinn 91522(2026-09-09,17 帖/459 浏):「求一个禁止微信、QQ、钉钉…闪烁托盘区图标」https://meta.appinn.net/t/91522
- ② 同帖 #2 Fireman(2026-09-09):「火绒弹窗拦截都管不住,只能用 ahk 无脑杀」
- ③ 同帖 #4/#6/#12-#17(2026-09-12→14):Windhawk 现成模组不满足「保留图标但不闪烁」→ AI 手搓 suppress-tray-flashing 三版迭代
- ④ **appinn 72085**(2025-06-08,7 帖/851 浏):微信来消息任务栏泛红,OS checkbox 关闪烁后「缺点是来消息时要看托盘区」https://meta.appinn.net/t/72085
- ⑤ **appinn 72938**(2025-07-06,4 帖/341 浏):「微信…图标一直闪烁,必须打开窗口再关闭」;「windows 有选项可以设置全部不闪烁,但这样会导致消息无法看到」→ 用 Windhawk Better Taskbar Autohide https://meta.appinn.net/t/72938
- ⑥ **Super User 1712699**(2022-03-25,超窗佐证):QQ/WeChat 在 Windows 通知与勿扰全关后「persist to give notifications」;回答承认无系统级总开关 https://superuser.com/questions/1712699

**3. 证据强度**:窗内(2024-10 起)≥3 当事帖但**同站**(appinn 91522/72085/72938);跨站 = Super User(2022 超窗)+ GitHub Windhawk 源码(供给端)。行为证据:AHK 杀弹窗、OS checkbox 剩托盘、Windhawk 凑合、AI 手搓 mod、拒绝全禁闪。**付费证据:0**(仅 AdGuard 买断 2022-09 邻品一笔,不对位)。日期分布:2025-06 / 2025-07 / 2026-09——活,非单发。

**4. 需求温度**:活(三连帖 + 2021→2024 反面帖未解决)。

**5. 推断链**:现象(AHK 杀 / OS 剩托盘 / Windhawk 不满足 / 手搓 mod)→ 本质(IM 不走 Windows 通知,托盘闪烁+自绘悬浮卡抢注意力;OS 只管任务栏不管托盘)→ 什么在变(IM 劫持持续;Windhawk+AI 降手搓门槛;任务栏禁闪 checkbox 反衬托盘缺口)→ 方向(「保图标、掐闪烁、杀悬浮卡」反打扰小工具)。**断点**:方向端零付费;「IM 劫持加码」仍单点推断。

**6. 失效标志**:① 微信/QQ/钉钉接入 Windows 通知或官方「免闪烁」开关;② Windhawk 模组被官方收录并稳定维护且覆盖「保图标中间态」;③ 2026-12 前同类求助帖不再出现。**(本轮复核:③ 未兑现——2025-06/2025-07 新帖仍在出现。)**

**7. 载体**:Windows 单文件 GUI 可为(Win32 托盘钩子/子类化);danqing 托盘/热键肌肉已有。

**8. 判定**:**维持立案·弱**。多帖复现 + 供给端缺口源码实证成立,但窗内独立站点=1、付费=0、「完整链」当事人仍集中 91522 → 不够升强;现象活着且免费在位者未收口 → 不毙。升强触发器:窗内第三独立站点需求现场,或一笔形态对位付费。

**早筛透镜**:① 系统吸收 40-60% 维持(任务栏 checkbox 已实证半吸收,托盘侧无;悬浮卡更远);② AI 不接走需求,反降手搓门槛;③ 免费在位者强但**全链空缺**(NoFlashWindow=任务栏-only,Disable Tray Icons=全隐藏-only,KillAds 停在 2022,火绒管不住悬浮卡);④ 付费分布:空。
