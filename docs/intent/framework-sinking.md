# Intent: 框架下沉盘点 —— 产品通用能力 → danqing

> 2026-09-09 全农场通读盘点：interview-me 确认意图（盘完即动，高分候选走 spec 流水线）→ 四个并行只读侦察兵（danqing 基线地图 + pomodoro / clipboard / log 三仓精读）→ 交叉比对汇总。
> 用户「存档」指示落地本文档；后续每簇下沉的 spec 直接引用本文对应条目。

- @author 十四叔
- @date 2026/09/09
- 状态: **盘点完成，待裁决**（三项政策门未定，见文末；未开工任何下沉）

## 一句话

三仓产品代码里长出了一批通用肌肉，重复发明最多达 6 次；系统化下沉进 danqing，服务引擎复用率核心指标 + 给下一件产品备料。

## 筛选标准

四条全占才是强候选：① 通用（非产品业务逻辑，是其下的通用机制）② 有第二消费者潜力 ③ 实现质量（有测试 / 被实战验证）④ danqing 尚未提供。
证据强度排序：跨产品重复发明 > 单产品强通用 > 边缘候选。

## danqing 基线（已有，勿再提议下沉）

- **窗口**：run_app/WindowConfig/CloseBehavior/WindowMode(OnDemand·Continuous·Adaptive)/置顶/ShowPlacement(光标跟随·位置记忆)/抢前台+焦点对账/点击穿透/前台全屏检测/自适应帧预算/无边框圆角阴影/开机自启
- **托盘**：install_tray/TrayHandle/菜单替换/快捷键 label 单一来源
- **输入**：焦点路由(Tab 链/点击聚焦)/app_key_filter/propagate_unhandled_keys/IME 链路(wants_ime·ime_area·候选窗吸附)/剪贴板读写(Copy·Paste 事件 + read_clipboard)/指针捕获/全局热键(声明式)/WindowAction
- **渲染**：wgpu 三管线(矩形 SDF/文本字形图集/图像缓存)/背景管线(场景交叉淡化·时辰)/break_lines/内嵌字体+font-kit 兜底/Theme token+SceneTheme/WCAG 对比度工具
- **组件**：Button/Text/Image/CloseButton/TextInput(撤销重做)/TextArea/Switch/IconInput/Box/Center/Column/Row/Padding/Stack/DragArea/ReachArea/Scrollable(非虚拟化)/MultiPanel/Tabs/TitleBar(Standard·TrafficLights·embed 嵌槽)
- **其他**：音频混音 Mixer+AudioPlayer/init_log 轮转/asset::resolve/update 检查(feature 门控)/App::tick 心跳

**框架没有**（候选对照空位）：AsyncJob 设施 / 剪贴板变更监听 / 虚拟化长列表 / 编码检测转码 / mmap / 设置持久化 / Overlay 组件 / 动画补间原语 / 文本截断助手。

## 候选清单（按建议动手顺序）

### 簇 A · `anim` 动画原语 —— 建议最先（成本 S）

- **内容**：通用包络/补间（目标值变化→当前值滑向新目标、中途反向续接无跳变、时间外部注入可单测）+ 一次性瞬态浮层时序（Flash 触发衰减 / Hint 静默延迟→淡入→停留→淡出）+ SceneFader 两态交叉淡化状态机（打断吸附）+ Easing 补 EaseIn/EaseOut/OutCubic 变体
- **重复发明 4+ 处**：pomodoro `motion.rs:53-93`（MotionEnvelope）+ `flash.rs:12-57` + `hint.rs:29-87` + `fader.rs:15-73`；danqing 自己 `audio/mixer.rs:24-63`（ChannelEnvelope 私有且焊死 300ms 音频语义）、`widget/form/switch.rs:154-168`（dt 步进 ad-hoc）
- **不对称证据**：框架渲染层 `BackgroundFrame::new(from, to, fade)` 已收淡化三元组，状态机却留在产品侧
- **测试**：pomodoro 侧 ~19 个（边沿/中点/反向续接/重复触发保护/打断吸附）直接可搬
- **下沉方向**：抽 ChannelEnvelope 出 mixer，参数化 duration+easing，放 `src/anim.rs`；mixer/switch 改用它；pomodoro 删 MotionEnvelope/AmbientMixer 残留 ~300 行

### 簇 B · `job` 异步作业（成本 M）

- **内容**：AsyncJob 泛化（launch 起 worker / poll 每帧零成本拾取 / generation 单调递增防旧轮晚到覆盖新轮 / invalidate 外部失效）+ IndexHooks 式进度/取消令牌 + `run_catched` panic 护栏；SearchNav 命中导航（升序表+partition_point+环绕跳转）搭车
- **重复发明 4 处**：danqing-log `search.rs:16-77`（本体）+ `open.rs:53-218`（OpenJob 第二消费者）；clipboard `main.rs:63-68`（revision AtomicU64 + tick 比对堂兄弟）；**danqing 自己 `update.rs:132,178-208` 裸 thread+全局 Mutex 无代次防护（潜在缺口）**
- **测试**：全场最厚 —— search.rs 6 + open.rs 7（drop-cancel/两连开替换）+ main.rs C1 回归
- **下沉方向**：`danqing::job` 模块（仿 update 的 feature 先例），一次 API 定形；落点是已有的 `App::tick` 心跳

### 簇 C · Overlay 模态浮层组件（成本 M）

- **内容**：scrim 遮罩 + 居中玻璃卡 + Esc/点遮罩关闭 + 关闭后焦点回锚点 + **push_layer 渲染层教训内化**（同层文本恒在矩形上，卡片须开新层才能盖底层表格文本）+ 关态零尺寸不拦事件
- **重复发明 6 份**（全场最多）：pomodoro `main.rs:1155-1224/1264-1343/1445-1509` 三处 + clipboard `ui/settings.rs:49-53,611-616` 两处 + danqing-log `settings.rs:57-139,211-248`
- **测试**：pomodoro 有（面板互斥/Esc/焦点回归/布局回归）；log 侧零测试，下沉时补纯逻辑测试
- **下沉方向**：widget/ 新 Overlay 组件；设计内容槽注入、open 态绑定、关闭消息；`focus_request`/`focus_restored` 协议已在 App trait；theme 已有 `scrim()` token；一次删掉 pomodoro ~350 行 + log ~200 行

### 簇 D · `persist` 持久化（成本 M，**撞政策门 1**）

- **内容**：① 设置/状态 JSON 原子写+配置目录+容错回退（pomodoro `state.rs:126-168` 最硬化版）② 脏旗标+节流/防抖落盘+退出 flush（pomodoro `main.rs:86-89,638-663`；xirang 两份变体含 `tick_flush_failure_keeps_dirty_for_retry` 直测）③ 版本化文档容器+未来版本拒读拒写保护（pomodoro `stats.rs:53-70,245-306`，starfield.rs 同款纪律）
- **重复发明 4 次**：pomodoro / clipboard `config.rs:194-221`（TOML 无原子写）/ xirang `config.rs:93-119`（逐行同构）/ **danqing 自己 update.rs 内部 ad-hoc 缓存**
- **测试**：pomodoro 22+ 专项（旧版兼容/损坏回退/拒写守护/.tmp 无残留）
- **下沉方向**：与 update.rs 私有实现合并；产品命名空间策略 `danqing/<product>.json`；版本闸门内置与否待 spec 定

### 簇 E · OS 级肌肉链（clipboard 日用实战验证，S→M-L 分阶）

| # | 能力 | 位置 | 成本 | 论据 |
|---|------|------|------|------|
| E1 | 焦点离开轮询等待 `wait_for_focus_leave` | clipboard `foreground.rs:75-100` | **S** | 补齐 danqing `simulate_paste` 文档配方缺的最后一环；固定延时打自己窗口的真实 bug 捶打出来 |
| E2 | 前台进程名获取 | clipboard `foreground.rs:12-53` | S-M | 启动器/专注工具复用面最宽；danqing 零 |
| E3 | 多格式剪贴板写入 + `to_crlf` 归一 | clipboard `inject.rs:15-77` | S-M | danqing 已有一半（arboard 内部用未公开）；to_crlf 4 测试即搬即走 |
| E4 | 剪贴板监听 Monitor（序列号门控/CF_HDROP/图片 alpha 修补） | clipboard `monitor.rs:37-326` | M-L | 肌肉 1 本体，20+ 测试全场最厚；CF_HDROP 缓冲区 +1、DIB alpha 全 0 等实证坑已内化 |
| E5 | 粘贴注入编排配方 `paste_into_previous()` | clipboard `main.rs:295-379` | M | 每步顺序都是 bug 换来的（防重入/隐藏非 toggle/焦点弹跳）；依赖 E1 先落 |

- **共同摩擦**：产品 OS 代码用 `windows` 0.62，danqing 用 `windows-sys` 0.59，E1/E2/E4 都需移植 + 非 Windows stub
- **形态裁决**（留 spec）：E4 下沉需新开平台代码驻地（现约定平台代码只住 `window/`/`render/`，监听不属窗口——`danqing::clipboard` 之类新模块？）
- 隐私门控（ExclusionList）属业务，**留产品侧**

### 簇 F · 文本类（log 高质量纯逻辑，S→S-M）

| # | 能力 | 位置 | 成本 | 论据 |
|---|------|------|------|------|
| F1 | 只读文本选区纯逻辑（token 取词/规范化/跨行拼装/字符边界回钳/行切片权威） | log `selection.rs:19-135` | S-M | danqing Copy 链路已沉（Event::Copy/Widget::selected_text/read_clipboard），缺的正是喂它的选区模型；16+4 测试；坐标系「行+行内字节」适配任何行式组件 |
| F2 | 编码检测/转码（BOM→交替NUL→UTF-8 合法性→GBK 统计→Latin-1 兜底 + CP936 零依赖 FFI） | log `encoding.rs:55-282` | **S** | 最纯候选：9 测试、零 crate 依赖、非 Windows 兜底已在；任何开用户文本文件的产品必遇 |
| F3 | 文本截断助手（fit_line 二分省略号 + scroll_trim 亚字符左切 + 路径感知中部省略） | log `view.rs:458-530`；clipboard `history_list.rs:426-470` | **S** | 两仓各发明一份；danqing TextBatch 只有 measure 原料；落 `text/` 或 render/text.rs 旁 |

### 小件包（S 级随手收，打包一个 spec）

- `reveal_in_file_manager`：Win `explorer /select` / mac `open -R` / Linux xdg-open（pomodoro `main.rs:1387-1417`）
- `open_feedback`：GitHub issues 预填版本/OS URL，可挂进 danqing::update 的 UpdateSpec（pomodoro `main.rs:1421-1436`；log `settings.rs:206-207` 裸 Link 是未拉齐的第二份）
- 相对时间+历法+本地时区偏移（clipboard `history_list.rs:109-166`，8 测试含闰日锚点）
- CSV 导出 BOM 壳（UTF-8 BOM 防 Excel 乱码 + 静态错误文案；表头语义留产品）（pomodoro `stats.rs:157-238`）
- 缩略图降采样 + alpha 修补（clipboard `history_list.rs:370-423` + `monitor.rs:58-64`；落 render/image.rs 邻近）
- MSIX/主窗口助手 `is_running_as_msix` + `find_main_window`（pomodoro `license.rs:195-248`，成本 M；商店渠道政策下确定性需求；纯平台运输非政策）

## 明确不下沉（已判负，勿再提议）

- **LogFile 引擎**（mmap/步进索引/并行构建/增量追加，logfile.rs 全体 27 测试）：数据引擎非 UI 设施，下沉让 clipboard/pomodoro 白扛 memmap2/memchr/regex 依赖；若复用应做**兄弟 crate**
- **expand 行映射**（log `expand.rs`，3 测试）：纯且已测，但脱离虚拟列表组件是孤儿，等宿主
- **行锚定虚拟视口**（log `view.rs`，f32 像素域 2 亿像素失守教训在模块头）：价值最高但 L 成本；分期——先沉滚动数学助手（S），组件化等第二个真实消费者
- **TextInput 托管五件套转发**（log Bar / clipboard bottom_bar 重复同一转发纪律）：Rust 抽不成助手，写进 widget 指南文档即可
- Pomodoro 状态机 / 跨日归零 / BSC5 星表烘焙 / license 购买流程（框架已裁「store/授权/IAP 是产品政策」）/ 文案格式化：产品语义

## 反向发现（产品侧收尾任务，引擎无需动作）

- **pomodoro `ambient.rs` 是未完成的迁移**：danqing::audio 已能表达 duck+淡化分槽（每帧 set_target），迁完删 ~300 行
- **clipboard 手写 TabBar**（`ui/settings.rs:218-354`，08-15）比 danqing Tabs（08-18）早 3 天，是抽取源但本地副本没删——产品应改用 Tabs
- **danqing 带 rfd 依赖只为 showcase 示例**；clipboard `file_dialog.rs` 也自带 rfd 封装——文件对话框两边都没正式归属，证据尚薄，暂不收

## 待定裁决（政策门，动手前需用户拍板）

1. **`persist` 簇撞「danqing 零文件 I/O」纪律**——窗口位置钩子注释明示「存储由产品负责」，但 update.rs 已破例（feature 门控缓存落 %APPDATA%）。开口子（feature-gated `persist` 模块）还是做兄弟 crate？
2. **非 UI 纯工具模块进框架的合法性**（encoding/selection/job）——update.rs 先例是否算数？
3. **LogFile 引擎兄弟 crate**——记录在案即可，还是现在立项？

## 使用方式

- 每簇走独立 spec 流水线（spec→plan→build→review→code-simplify），spec 引用本文对应条目作为需求来源
- 联动顺序照旧：danqing 提交并 push → 产品仓 `cargo update -p danqing` → 提交 lock，commit message 注明关联
- 建议动手顺序：**A(anim) → B(job) → C(Overlay) → D(待裁决1) → E 链(E1 先) → F → 小件包**
