# A1 pre-spec 装备第 1 路：免费在位者 issues 逐条扫

- **日期**：2026-10-09
- **执行**：取证 agent（时间盒 ≤75 分钟）
- **产品形状**（已定）：中文 AI 对话/Markdown → 公众号 HTML 一键落地的 Windows 桌面 GUI（Rust，全本地单文件）；剪贴板进（纯文本 MD + CF_HTML 富文本双路）→ inline-CSS 公众号 HTML 出 + 排版预览 + 约 3 套模板；次级腿 = →docx；v1 不做 PDF、不做直推草稿箱。
- **方法**：gh api 拉 2024-10~2026-10 有实质内容的 issues（open+closed），抽「没把事办利索」原话。深调研卷 `deep-dive-round8-t1-2026-09-25.md` 已做 D1/D2/D3，本轮只做免费侧 issues 取证补强。
- **取证纪律**：所有引文/URL/日期来自 gh api 实拉，未编造；拿不准标「未验证」。

---

## 一、每仓现状（截至 2026-10-09 实拉）

### 1. doocs/md（13412★，open issues 12，pushed 2026-10-09）

- 高度活跃：维护者 yanglbme / YangFong 逐条回 issue，多数 bug 会在数日内修。
- **止于复制按钮**的定位在 issues 里反复被证实：用户把「复制 → 粘进公众号编辑器」之后的掉样式当成产品责任来报。
- issue 编号到 1987 但非 PR issue 稀疏（近期 issue 多被 PR 编号挤占），窗口期（2024-10 起）新建 issue 约 15–20 条量级有实质内容。
- 站点 md.doocs.org + Docker/CLI 自部署两条腿都在；API/CLI 自动化诉求长期 open。

### 2. mdnice/markdown-nice（4682★，2023-10 后实质停更）

- 窗口期 issue 只有 7 条（2024-11 ~ 2026-02），全部 open 无人回——维护者 guanpengchn 自 2024-08 后基本不响应 issue。
- 窗口期 issue 全是「没人管」的抱怨：粘贴卡死、登录二维码不显示、自定义主题改不了、贴进微信多 20px 边距、目录小黑点被切。
- 历史付费事故（2023 会员激活循环退款）在窗口期之前，但直接相关，单列在第三节。
- 结论：**死在位者**，其 issue 区 = 用户在别处凑合不掉而积压的抱怨堆。

### 3. lyricat/wechat-format（4550★，停更迹象确凿）

- 全仓 issue 只有 51 条，最新一条是 2025-06-20 的提问（#51 转知乎格式），其余 2019–2023。
- 2024-10 起窗口期**零条**新建 issue。仓库仍在被 push（2026-10-08），但 issue 区已无用户声音。
- 结论：**死在位者**；历史 issue 里的痛点（脚注互转、代码块高亮、图片粘贴失败）是老痛点，不作近两年证据用。

### 4. geekjourneyx/md2wechat-skill（3689★，活跃，AI-agent skill 形态）

- 2026-01 开仓，issue 全部落在窗口期后段；产品形态 = Claude/Agent skill + CLI + 专业 API（README 标价 ¥199/永久，见 issue #29 引述）。
- issue 构成：一半是「装不上」（clawhub 找不到、marketplace schema、Windows hook 路径错），一半是转换保真（手机端空 bullet）与商业化规划（Vibe Pay 自助购 Key）。
- 维护者对商业化的自我复核（#29 评论，2026-09-09）值得抄：把「自助付款 / 自动开通 / CLI 全流程」拆成三层投资决策。

### 5. isjiamu/gzh-design-skill（3946★，2026-07 后沉寂中，AI-agent skill 形态）

- 2026-07 开仓，issue 少而精（21 条），但含**全网最贴近微信规则现状**的两条：#20/#21（渐变触发「内容结构检测」弹窗 + Dark Mode 不可预测）。
- 维护者 2026-10-09 刚合并 PR #22 修渐变问题——说明该仓仍在动，但 issue 响应慢（#12/#13/#8/#5/#14 均 open 无人回）。
- 定位与我们 v1 高度重叠（Markdown → 可粘贴公众号的内联 HTML + 主题 + 校验），但是 skill 形态不是桌面 GUI。

### 6. caol64/wenyan（1069★）+ caol64/wenyan-pc（95★）— 桌面形态最正占座者

- wenyan = macOS 桌面（App Store + dmg），wenyan-pc = Windows 桌面（Tauri）。**这是我们 v1 形态上的直接在位者**，wenyan-pc 仅 95★。
- issue 密度高、答复认真（caol64 本人逐条回），但大量 issue 是「桌面版做不了，去用 CLI/MCP 版」——桌面形态在作者自己的产品线里是二等公民。
- 2026-09-22 #47：App Store 搜不到了，作者自述「没续费 apple developer」；2026-10-08 提供 dmg 版本。
- wenyan-pc 自 2026-05 后 issue 停在 #28（LaTeX 转义大括号丢），桌面版修复节奏慢。

### 7. LaoFeng-mouse/flyingmouse-format（6555★，活跃）

- 通用文件格式转换器（非 MD→公众号赛道），但含 Markdown → Word/PDF（Pandoc）。
- README 有明确**非商用声明**：「仅供个人免费使用，禁止任何形式的商业售卖、转卖、套壳换皮重新发布……发现闲鱼/淘宝等渠道倒卖请告知作者」——影子付费/倒卖现象的直接书面证据。
- issue 区极吵（近两月数十条）：卸载不了、C 盘吃 10G、缓存越堆越多、PDF 转换质量、下载慢。对我们的启示是**桌面转换器的工程卫生**（卸载/缓存/体积）比功能更招骂。

---

## 二、主题 × 原话 × 日期 × URL

### 主题 A：粘贴到公众号编辑器后掉样式 / 变形 / 图片消失（**头号主题**）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「导出的文章，粘贴到微信中没有排版了？」 | 2026-06-12 | [doocs/md#1662](https://github.com/doocs/md/issues/1662) |
| 「直接进行复制粘贴到公众号，表格里的文字竟然变为了靠右对齐」 | 2026-04-02 | [doocs/md#1430](https://github.com/doocs/md/issues/1430) |
| 「上传了图片，然后点击"复制"，粘贴到公众号后台的时候是会有图片，但发表出来就没有图片了」 | 2026-02-09 | [doocs/md#1324](https://github.com/doocs/md/issues/1324) |
| 「安卓Firefox在线编辑器复制到公众号助手App样式错误」 | 2025-12-25 | [doocs/md#1210](https://github.com/doocs/md/issues/1210) |
| 「在编辑器中预览是可以正确显示变更的字体……但是复制到公众号编辑器时就只有默认的无衬线字体了」 | 2026-08-17 | [doocs/md#1916](https://github.com/doocs/md/issues/1916)（多用户复现「的确失效了」） |
| 「微信表格横向无法拖动，溢出显示范围」+ 维护者承认「有样式在保存时被去除了」 | 2026-09-02 | [doocs/md#1944](https://github.com/doocs/md/issues/1944) |
| 「微信文章边上多了20px的边距」 | 2024-12-01 | [mdnice#363](https://github.com/mdnice/markdown-nice/issues/363) |
| 「目录标题前小黑点……显示不全」粘贴至公众号排版乱 | 2024-11-25 | [mdnice#362](https://github.com/mdnice/markdown-nice/issues/362) |
| 「复制到公众号导致公众号文章编辑页面卡死……浏览器内存占用不断上升」 | 2026-02-19 | [mdnice#367](https://github.com/mdnice/markdown-nice/issues/367) |

**对我们 v1 的启示（对位 spec F）**：用户把「粘进去之后还是不是那样」算作产品没办利索。v1 的验收必须是**粘贴后实机对照**，不是渲染预览好看就完——预览所见 ≠ 公众号编辑器所得是本赛道头号「没办利索」。建议 spec 明确：①输出以微信编辑器白名单 inline 样式为准（字体 font-family 只写系统字体栈，不指望衬线生效）；②表格做窄屏溢出预案（微信会剥 overflow 样式——doocs 维护者原话「有样式在保存时被去除了」）；③「粘贴后校验」作为 v1 预览窗格的验收关卡，而非可选项。

### 主题 B：微信「内容结构检测」弹窗 / 微信改规则踩踏（**最新、最硬的规则类证据**）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「粘贴到公众号编辑器触发「内容结构检测」弹窗（darkmode-no-gradient）……每次粘贴都要手动点「继续插入」，多篇连续发时比较打扰」；实测把 `radial-gradient` 改纯色后弹窗消失 | 2026-10-07 | [gzh-design-skill#21](https://github.com/isjiamu/gzh-design-skill/issues/21) |
| 「渐变会被 mix 混合计算、转成纯色后再转换」「算法会检测文本与背景对比度」——Dark Mode 下渐变与「实底+白字」输出不可预测；另注「validate 对 gradient 完全不设防，校验全绿照样弹」 | 2026-09-29 / 补充实测 2026-10-07 | [gzh-design-skill#20](https://github.com/isjiamu/gzh-design-skill/issues/20)（维护者 2026-10-09 回复已修 PR #22） |
| 「固定宽度677px的容器在手机端（约375px）被压缩或溢出；多列flex布局在窄屏下无法换行……平台红线又禁止@media查询」 | 2026-07-12 | [gzh-design-skill#8](https://github.com/isjiamu/gzh-design-skill/issues/8) |

**对我们 v1 的启示**：微信已上线「内容结构检测」，渐变背景 = 每篇粘贴手动确认一次，这在连续发文场景是硬打扰。**3 套默认模板必须零渐变、零「实底+白字」高对比反转、零 @media**；校验器（我们自己的预览关卡）要像 gzh-design 学习补 WARNING，但更要像它这次栽的跟头学到：**校验器漏掉的写法，微信会用弹窗替你 QA**。这条应写进 spec 的模板约束硬条款。

### 主题 C：复制按钮失效 / 剪贴板为空 / 数据丢失（**桌面形态的生死线**）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「目前是点击了复制按钮但是没有效果」 | 2024-11-14 | [wenyan#17](https://github.com/caol64/wenyan/issues/17) |
| 「点击复制按钮后 剪贴板是空的」 | 2024-12-23 | [wenyan#24](https://github.com/caol64/wenyan/issues/24) |
| 「国产安卓手机用了brave浏览器和via浏览器，复制均无内容」 | 2026-03-24 | [wenyan#44](https://github.com/caol64/wenyan/issues/44) |
| 「刚开始用的时候，复制功能能用，后来发现时能用时不能用，现在不能用了（重启电脑还是不行）」 | 2025-03-14 | [wenyan-pc#18](https://github.com/caol64/wenyan-pc/issues/18)（2025-08 还有人跟「怎么回事」） |
| 「点了一下帮助，用github登录，取消了，然后挂了，写的内容没了」（标题：不会自动保存吗） | 2026-09-22 | [doocs/md#1970](https://github.com/doocs/md/issues/1970) |
| 「WIn版使用自定义主题，微信公众号模式的复制按钮点击后无反应」 | 2024-11-07 | [wenyan#16](https://github.com/caol64/wenyan/issues/16) 评论串 |

**对我们 v1 的启示**：剪贴板写入失败/无提示是桌面在位者反复栽的坑（wenyan 系至少 4 条独立 issue）。我们 v1 的核心动作就是剪贴板进出——**CF_HTML + 纯文本双路写入必须有成功/失败可见反馈 + 自动重试**，并且「写入后立刻自读回校验」。顺带：doocs #1970 的内容丢失指向**本地自动保存**；我们全本地单文件，应把「每次转换产物 + 源文自动落盘」写进 spec 默认行为。

### 主题 D：本地图片 / 图床 / 防盗链

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「md文件中有图片，无论图片使用相对路径还是绝对路径，均无法预览和上传到公众号素材箱，只有网络路径才可以」；维护者：「桌面版不支持自动上传图片，如果有这方面需求，可以使用 CLI 版或者 MCP 版」 | 2025-09-22 | [wenyan-pc#23](https://github.com/caol64/wenyan-pc/issues/23) |
| 「放入本地图片的时候,不支持自动上传到图床吗?」 | 2024-11-27 | [wenyan#20](https://github.com/caol64/wenyan/issues/20) |
| 「微信素材库已经添加了防盗链功能……贴到其他网站后，图片无法显示。但那些网站基本都支持 docx 导入」 | 2025-08-30 | [wenyan#36](https://github.com/caol64/wenyan/issues/36) |
| 「配置好后上传图片一直转圈圈」（公众号图床白名单） | 2025-05-18 | [wenyan-pc#20](https://github.com/caol64/wenyan-pc/issues/20) |
| 「经测试 Windows 图床功能无法使用」（弹窗报错） | 2025-02-20 | [wenyan-pc#7](https://github.com/caol64/wenyan-pc/issues/7) |

**对我们 v1 的启示**：wenyan-pc 明确承认**桌面版不自动传图**，把需求推给 CLI/MCP——这是桌面形态空出来的缝。但 v1 范围是剪贴板进 HTML 出、用户自己粘进公众号：图片若已在公众号素材箱/外链则 inline 即可，**本地图片自动上传属于 v1 外的图床子系统**（需要 AppID/白名单，wenyan-pc 在 Windows 上就翻车了）。spec 应写死：v1 对本地图片路径给出「未上传、将保留原路径」的显式警告，不静默丢图（doocs #1324 的「发表出来就没有图片了」是用户最恨的静默失败）。

### 主题 E：转换保真（表格 / 代码 / 公式 / 字体 / 解析）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 加粗紧邻引号不解析：`我发现**"黑体"**展示不准确` | 2026-01-12 | [wenyan#39](https://github.com/caol64/wenyan/issues/39) |
| 「公式中用反斜杠转义的大括号 `\{` `\}` 丢失」 | 2026-05-30 | [wenyan-pc#28](https://github.com/caol64/wenyan-pc/issues/28) |
| 「latex不支持矩阵显示」 | 2024-10-26 | [wenyan#14](https://github.com/caol64/wenyan/issues/14) |
| 「头条公式现在还没办法处理吗？」 | 2024-12-24 | [wenyan#25](https://github.com/caol64/wenyan/issues/25) |
| 「复制到微信公众号格式不对」（附视频） | 2024-12-06 | [wenyan#23](https://github.com/caol64/wenyan/issues/23) |
| 「mermaid渲染错误」（后查明缓存问题） | 2026-08-09 | [doocs/md#1901](https://github.com/doocs/md/issues/1901) |
| 「两列表格首行单元格包含 \| 时……列数被错误计算为三列」（docx→md 提取阶段） | 2026-08-11 | [gzh-design-skill#14](https://github.com/isjiamu/gzh-design-skill/issues/14) |
| 「代码不高亮，建议参考各编辑器增加代码高亮效果」 | 2026-07-15 | [gzh-design-skill#12](https://github.com/isjiamu/gzh-design-skill/issues/12) |
| 「win版2.2，导入时文件编码不为utf-8，导入无效」（文件是 UTF-16 LE） | 2024-12-06 | [wenyan#22](https://github.com/caol64/wenyan/issues/22)（v2.2.2 修） |

**对我们 v1 的启示**：①公式（LaTeX→图片或弃用）是长期未办利索区，v1 不做 PDF 的前提下建议公式降级为「原样保留/显式提示不支持」，别静默出错；②表格单元格转义竖线这类解析边角（gzh-design #14）值得进我们 MD 解析测试集；③**UTF-16/GBK 导入失败**（wenyan #22）正中 danqing-encoding 枪口——我们兄弟 crate 的 BOM/UTF-16/GBK 检测是现成肌肉，spec 应把「任意编码 md 粘贴/打开」列为差异化硬条款；④代码高亮是模板 3 套里的高频期望（gzh #12、wechat-format 老 issue 同题）。

### 主题 F：模板 / 主题定制不够用

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「想自己创建一个主题，但是表格等等的类名不知道，没法写CSS」（10 评论，作者补文档） | 2024-11-07 | [wenyan#16](https://github.com/caol64/wenyan/issues/16) |
| 「希望自定义主题能在已有主题上修改……另外好像这个主题定义的代码和现在主流的在线编辑器自定义主题的代码不通用，比如 mdnice……目前我还是主要在mdnice上写」 | 2024-11-23 | [wenyan#18](https://github.com/caol64/wenyan/issues/18) |
| 「添加超过 3 个自定义主题就会覆盖【自定义主题】功能菜单。另外，自定义主题不能修改名称」 | 2025-09-19 | [wenyan-pc#22](https://github.com/caol64/wenyan-pc/issues/22) |
| 「我的自定义主题为什么无法再次编辑修改或者删除呢？」 | 2025-03-13 | [mdnice#364](https://github.com/mdnice/markdown-nice/issues/364) |
| 「可以支持一下自定义 li 的样式吗，我不太清楚微信公众号渲染逻辑」 | 2026-09-23 | [doocs/md#1974](https://github.com/doocs/md/issues/1974) |
| 「文章可选全局样式与自定义样式」（全局/局部样式区分诉求） | 2025-04-29 | [doocs/md#677](https://github.com/doocs/md/issues/677) |
| 「段落间距不是很大……感觉半佛文章的段落间距和字体看起来都比我的舒服」 | 2025-04-19 | [wenyan#33](https://github.com/caol64/wenyan/issues/33) |

**对我们 v1 的启示**：「3 套模板」打的是「开箱即用」而非「无限主题」；但用户真实痛苦集中在**可改性**（在既有主题上调、能删能改名、看得见类名）与**呼吸感**（段落间距）。spec 建议：3 套模板各暴露少量可调 token（字号/行距/段距/强调色），而不是自定义 CSS 编辑器（那是 doocs/mdnice/wenyan 已卷过的红海）；wenyan #18 的「主题语法不通用」提示我们——别发明新主题 DSL。

### 主题 G：多端预览差异 / 所见非所得

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「生成的HTML在电脑端和手机端排版差异明显，缺少移动端适配说明」 | 2026-07-12 | [gzh-design-skill#8](https://github.com/isjiamu/gzh-design-skill/issues/8) |
| 「手机端预览时列表项前面出现多余的空 bullet（空圆点）……（电脑端正常）」 | 2026-04-28 | [md2wechat-skill#9](https://github.com/geekjourneyx/md2wechat-skill/issues/9) |
| 「图片组件我插入图片后是在框外面的……容器没高度」 | 2026-07-11 | [gzh-design-skill#5](https://github.com/isjiamu/gzh-design-skill/issues/5) |

**对我们 v1 的启示**：预览窗格至少要给手机宽度（375px）一档视图；列表小圆点/空 bullet 是粘贴后高发视觉 bug，模板默认样式应显式重置 list-style。对应我们已定的「排版预览」要求——**预览 = 双栏（桌面宽 + 手机宽）**建议写进 spec F。

### 主题 H：导出格式诉求（docx / 长图 / PDF / 多平台）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「建议添加导出docx功能」——动机是防盗链+多平台导入 | 2025-08-30 | [wenyan#36](https://github.com/caol64/wenyan/issues/36) |
| 「希望可以增加pdf保存……写完笔记方便分享给同事」 | 2025-01-05 | [wenyan#26](https://github.com/caol64/wenyan/issues/26)；同题 [wenyan-pc#2](https://github.com/caol64/wenyan-pc/issues/2)（open） |
| 「能支持导出"小红书"的短图格式吗？现在只支持长图，需要自己截图，有点麻烦」 | 2026-03-15 | [wenyan#42](https://github.com/caol64/wenyan/issues/42) |
| 「请问有保留排版格式，转成知乎的工具吗」 | 2025-06-20 | [wechat-format#51](https://github.com/lyricat/wechat-format/issues/51) |
| 「复制到今日头条能否保留链接」（粘贴丢链接） | 2026-05-26 | [wenyan#46](https://github.com/caol64/wenyan/issues/46) |
| 「MD转PDF或WORD时存在部分问题」「markdown文件中存在latex公式时无法正确转换」 | 2026-08-30 / 2026-08-14 | [flyingmouse-format#62](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/62)、[#37](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/37) |

**对我们 v1 的启示**：**次级腿 →docx 的需求原话已在 wenyan#36 出现**（动机：公众号防盗链导致跨平台发文要走 docx 导入）——这是 v1 次级腿的正面证据。PDF/长图/小红书短图多平台诉求存在，但 v1 已明确不做 PDF，记录在案不扩权；今日头条/知乎是「同一份稿多平台」现象，可作 v2 方向不作 v1。

### 主题 I：桌面形态 / 性能 / 工程卫生

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「可以发布一个可执行EXE文件吗?」（issue 后演变为桌面/移动支持讨论，9 评论） | 2025-09-26 | [doocs/md#1010](https://github.com/doocs/md/issues/1010) |
| 「贴了个超大 svg 进去，左侧编辑栏被卡白屏……内存飙升，应用卡死」 | 2024-10-29 | [wenyan#15](https://github.com/caol64/wenyan/issues/15) |
| 「粘贴内容后，若干次点击、滚动视图，就会出现大块空白区域……光标无法点击到指定的位置」 | 2026-06-26 | [doocs/md#1734](https://github.com/doocs/md/issues/1734) |
| 「是否支持打开一个本地目录,里面可能有多个md文件,现在好像是只能打开一个文件」 | 2025-10-26 | [wenyan-pc#24](https://github.com/caol64/wenyan-pc/issues/24) |
| 「设置.md默认用文颜打开，双击markdown后显示的还是之前打开过的文件」 | 2025-05-26 | [wenyan-pc#21](https://github.com/caol64/wenyan-pc/issues/21)（open） |
| 「卸载不了？？？」「下载安装后，C盘空间额外占用10g」「任务结束后不会自动清理缓存……缓存在C盘越堆越多」 | 2026-09-29 / 2026-09-29 / 2026-09-03 | [flyingmouse#79](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/79)、[#82](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/82)、[#67](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/67) |
| 「能否做成免安装的形式 就像geek那样解压即用」 | 2026-08-21 | [flyingmouse#49](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/49) |

**对我们 v1 的启示**：①doocs 用户在要 EXE——桌面形态有真实需求原话支撑；②wenyan #15 的「大 SVG 卡死」对我们的 danqing 自绘编辑/预览是明确风险样本，spec 需要大文档/大内联资源的降级策略；③flyingmouse 的卸载/C 盘/缓存三连骂 = 便携单文件（我们已有 package_portable.ps1 传统）是**可写进卖点**的工程选择，而非纯技术偏好；④文件关联/多文件目录是 wenyan-pc 的 open 痛点，v1 若只做「剪贴板进」可不背，但不要在 spec 里承诺文件关联。

### 主题 J：AI 工作流 / API / 自动化（人群 = 用 AI 写稿的人）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「我想通过AI自主生成markdown再通过API生成wx_markdown一步到位，然后再通过微信API直接设置到草稿箱」 | 2025-05-06 | [doocs/md#716](https://github.com/doocs/md/issues/716)（13 评论，多人「同求」；2025-12 有人贴自研 API 仓库） |
| 「建议提供命令行工具……将本地 markdown 文件一键发布到公众号，避免去浏览器点点点」 | 2025-07-30 | [doocs/md#882](https://github.com/doocs/md/issues/882) |
| 「现在的文颜虽然可以很好地解决排版的问题，但是还是需要手工操作。如果能开发API，开发者就能很容易于其它应用集成」 | 2025-07-28 | [wenyan#35](https://github.com/caol64/wenyan/issues/35)（8 评论） |
| 「可否……增加【粘贴 HTML 自动转换为 Markdown】的功能」 | 2026-02-11 | [doocs/md#1327](https://github.com/doocs/md/issues/1327) |
| md2wechat-skill 产品形态本身：「README 里已经把公众号文章的 inspect、preview、convert --draft 这条 Agent 工作流讲得很清楚了」 | 2026-07-04 | [md2wechat-skill#18](https://github.com/geekjourneyx/md2wechat-skill/issues/18)（引流 issue，可证工作流已被命名） |

**对我们 v1 的启示**：「AI 生成 MD → 转公众号 HTML」的工作流在 doocs #716 有 13 条同求原话，是人群证据的正中靶心。我们的剪贴板双路输入（MD + CF_HTML）恰好覆盖「AI 对话里复制出来」的入口——spec 应把「从任意 AI 客户端复制富文本/MD 一次转换」写成核心场景，并考虑暴露无 UI 的转换入口（CLI/快捷键）作为 v1.x 钩子（doocs #882 / wenyan #35 的自动化诉求）。

### 主题 K：输入侧痛点（Obsidian / Word / 非 UTF-8 / 富文本粘贴）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「能否支持 Word 导入？Word 里面有图片。并且能否支持连接obsidian的笔记」 | 2026-08-07 | [md2wechat-skill#20](https://github.com/geekjourneyx/md2wechat-skill/issues/20) |
| 「加入对obsidian插件admonition的支持」（mdnice，2023 提、2025 仍 open） | 2023-02-22 | [mdnice#343](https://github.com/mdnice/markdown-nice/issues/343) |
| UTF-16 LE 导入无效 | 2024-12-06 | [wenyan#22](https://github.com/caol64/wenyan/issues/22) |
| 粘贴 HTML → Markdown | 2026-02-11 | [doocs/md#1327](https://github.com/doocs/md/issues/1327) |

**对我们 v1 的启示**：Obsidian 用户是中文 MD→公众号的高浓度人群（两条独立仓库都有诉求）；Word 导入含图片是 docx 次级腿的反方向（导入）诉求，v1 不做，记录。剪贴板 CF_HTML 双路正好是「Obsidian/AI 客户端/网页」三处复制的公约数输入。

---

## 三、付费证据单列（三档 + 影子付费）

> 档位定义：买断/会员自白 > 捐赠/订阅自述 > 标价不算付费。

### A 档：买断/会员自白（含退款）

| 原话（摘） | 日期 | 来源 |
|---|---|---|
| 「window 版本 我购买会员，每天打开时候提示需要激活，激活后还不行，继续提示购买服务，**要求退款**。耐心全部没有了。……是bug 还是骗人的呢？」维护者回复：「已经退款处理」 | 2023-08-15（退款处理 2023-08-30） | [mdnice#350](https://github.com/mdnice/markdown-nice/issues/350) |
| 「会员用户无法登录，提示操作失败」 | 2024-04-16 | [mdnice#355](https://github.com/mdnice/markdown-nice/issues/355) |

**解读**：mdnice 有真实付费会员（Windows 版），但激活/登录可靠性翻车导致退款。这是「中文 MD 排版工具**有人掏钱**」的 A 档证据，同时是「订阅/激活机制会反噬口碑」的负面样本——与农场「奶牛禁令」（不搞订阅运营型收入）一致，买断 + 本地单文件（无激活服务器）正好绕开此坑。

### B 档：捐赠/订阅自述

- 本轮未采到窗口期内的捐赠自述原话。（mdnice 会员属 A 档购买自白。）

### C 档：标价（不算付费，只证商品化意图）

| 事实 | 日期 | 来源 |
|---|---|---|
| md2wechat-skill：「README 展示专业 API ¥199/永久」（issue 正文引述），并规划「CLI 自助购买、服务端验付与 Key 自动交付」 | 2026-09-08 | [md2wechat-skill#29](https://github.com/geekjourneyx/md2wechat-skill/issues/29) |
| md2wechat 侧产品复核原话：「用户购买的是稳定、省事的文章排版与发布能力……核心目标应是：有购买意愿的用户，不用等待维护者，也能完成购买并得到第一个有效结果」 | 2026-09-09 | 同 #29 维护者评论 |
| gzh-design-skill 标价 ¥199（任务给定参考值；本轮未直接读到 README 标价行） | — | **未验证** |

### 影子付费 / 倒卖（单列）

| 事实 | 日期 | 来源 |
|---|---|---|
| flyingmouse-format README 非商用声明：「禁止任何形式的商业售卖、转卖、套壳换皮重新发布……**发现闲鱼/淘宝等渠道倒卖请告知作者**」 | README 现状 2026-10-09 实拉 | [LaoFeng-mouse/flyingmouse-format](https://github.com/LaoFeng-mouse/flyingmouse-format) README |
| 同仓出现「求 0.5.5 旧安装包」「之前的代码还能有机会被找到吗」等（疑似版本更迭/下架引发的找回） | 2026-09-16 / 2026-08-22 | [flyingmouse#76](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/76)、[#50](https://github.com/LaoFeng-mouse/flyingmouse-format/issues/50) |

**解读**：免费工具被闲鱼/淘宝倒卖 = 有人在为它收钱（影子付费），作者被迫写法律声明自卫。对我们：**开源/免费层 + 明确许可边界 + 买断解锁**的组合要在 v1 发布前就把 LICENSE/声明写死，避免重蹈「免费被套壳倒卖」。

### 相关旁证（非付费，但与收费文章人群有关）

- 「文章发布到微信公众号，**付费文章**，会不会构成字体侵权呢？」——2026-01-04，[doocs/md#1226](https://github.com/doocs/md/issues/1226)。可证人群里有人在发公众号付费文章，对字体授权敏感。

---

## 四、未验证清单

1. **B站公开搜索 API**：已试一次，返回「出错啦」验证码页（非 JSON），按纪律弃用。中文视频侧用户原话 = 零收获。
2. **WebSearch**：按纪律禁用，未试。
3. **ChatGPT 网页 / DeepSeek / Kimi / Claude 客户端剪贴板输入**的具体 issue 原话：本轮未检索到直接命中（最接近的是 doocs#1327「粘贴 HTML 自动转 Markdown」与 md2wechat-skill 的 Agent 工作流命名）。AI 客户端复制场景的用户原话**未验证**。
4. **gzh-design-skill 的 ¥199 标价**：任务给定，本轮未直接读 README 标价行（md2wechat-skill 的 ¥199 有 issue #29 正文引述）。标价数字均属 C 档，不算付费。
5. **mdnice 会员具体价格/档位**：未取证（只知存在「购买会员」与退款事实）。
6. **各 issue 的关闭原因**：除已贴出的评论串外，未逐条核对 close reason；标题/正文为一手，结论里「已修/未修」凡未见修复声明的均按 open 或未验证处理。
7. **微信「内容结构检测」官方文档条款原文**：只引到 gzh-design-skill #20/#21 用户转述（含「§4.1.1 / §4.1.2」编号），未拉官方文档原文核对编号。
8. **wenyan App Store 下架**：作者自述「没续费 apple developer」（2026-09-22，#47），未做 App Store 侧独立核对。
9. **wechat-format 停更判定**：基于 issue 区 2024-10 后零新建 + 2025-06 后无实质讨论；仓库 2026-10-08 仍有 push（可能是 CI/依赖），**未核对 push 内容**。
10. **flyingmouse-format 与 MD→公众号赛道的重叠度**：其 Markdown→Word/PDF 用 Pandoc，未实测输出质量；作为「工程卫生反面教材」引用，不作功能竞品。

---

## 五、对 spec F 的可直接落条款（汇总）

| # | 条款建议 | 证据锚点 |
|---|---|---|
| F1 | 粘贴后实机对照是验收关卡（预览 ≠ 公众号编辑器所得） | doocs#1662/#1430/#1324/#1916/#1944, mdnice#363/#362/#367 |
| F2 | 默认模板零渐变、零「实底+白字」、零 @media；输出前跑结构检测预警 | gzh-design#20/#21/#8 |
| F3 | CF_HTML + 纯文本双路写剪贴板，写后自读回校验 + 可见成败反馈 | wenyan#17/#24/#44, wenyan-pc#18 |
| F4 | 转换产物 + 源文本地自动落盘，不丢稿 | doocs#1970 |
| F5 | 本地图片显式警告不静默丢图；图床不进 v1 | wenyan-pc#23, doocs#1324 |
| F6 | 任意编码（UTF-8/UTF-16/GBK/BOM）粘贴与打开走 danqing-encoding | wenyan#22 |
| F7 | 表格窄屏溢出预案 + 单元格转义竖线进测试集 | doocs#1944, gzh-design#14 |
| F8 | 公式降级策略（显式提示，不静默出错） | wenyan#14/#25, wenyan-pc#28 |
| F9 | 预览双栏（桌面宽 + 375px 手机宽）；列表样式显式重置 | gzh-design#8, md2wechat#9 |
| F10 | 3 套模板各暴露少量可调 token（字号/行距/段距/强调色），不发明主题 DSL | wenyan#16/#18/#33, wenyan-pc#22, mdnice#364 |
| F11 | 次级腿 →docx 对位防盗链跨平台分发 | wenyan#36 |
| F12 | 大文档/大内联资源降级策略（防卡死） | wenyan#15, doocs#1734 |
| F13 | 便携单文件 + 干净卸载/无残留缓存 可写进卖点 | flyingmouse#79/#82/#67/#49 |
| F14 | LICENSE/非商用边界发布前写死（防套壳倒卖） | flyingmouse README 声明 |
| F15 | 核心场景 = 从任意 AI 客户端/Obsidian 复制一次转换；留 CLI/快捷键钩子 | doocs#716/#882/#1327, wenyan#35, md2wechat#18/#20 |
