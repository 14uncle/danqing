# Todo: Dropdown 下拉选择器重写 + 框架弹层通道

> Plan: `tasks/plan-dropdown.md` (D1–D9 决策 / 依赖图 / 风险)。Spec: `docs/specs/SPEC-dropdown.md` (状态: 待批准)。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。
> 关键路径: T1 → (T2 ‖ T3) → T4。commit/push 待用户指示 (D8)。
> 三件套 = `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests`。

## Phase 1: 弹层通道 (框架设施)

- [x] **T1: `widget/mod.rs` — trait 三方法 + 三纯函数 + 探针单测 ✅ 2026-09-12**
  - 内容: `Widget` 加 `paint_popup` (默认空, `_area/_rects/_texts` 下划线风格对齐既有 `event`) 与 `on_popup_dismiss` (默认空); `popup_area` 保留并**改写 doc** (原文「Flow 分发事件时会优先检查此区域」已失实, 改为根级通道语义); 新增 `pub(crate) fn paint_popups` / `dispatch_popup_event` / `dismiss_popup_at` + 递归体 `dispatch_popup_in` / `dismiss_in`
  - Acceptance:
    - [x] 三函数实现完成, `push_layer` 配对**只在 `paint_popups` 内一处** ✅
    - [x] `popup_*` 命名 (可 `--lib popup_` 过滤): 弹层内按下**直达持有者** (含「该点在控件矩形之外」的前提断言) ✅
    - [x] 无弹层时 `dispatch_popup_event` 返 `None`、`paint_popups` 层数零增加、`dismiss_popup_at` 返 `false` — **「无弹层 = 与既有行为逐位等价」的锁** ✅
    - [x] `on_popup_dismiss` 三种判定 (弹层内/控件内 = 不调, 两者外 = 调) ✅
    - [x] `rects` 与 `texts` 层数**相等** (D3 漏压锁) ✅
    - [x] `CursorMoved` / 抬起 / 键盘遇弹层展开**均返 `None`** (D2 锁) ✅ — **策略内化进纯函数**, 驱动的调用点无需自行判断, 因此可单测
    - [x] 额外: 两弹层区域重叠时**后绘制者优先** (与 Flow「后绘制者优先」同序) ✅ (实写 8 条, 多于计划的 7 条)
  - Verify: `cargo test --lib popup_` — 8 全绿 ✅
  - **偏离备案**: 本条原先写的「三件套绿」在 T1 独立时**做不到** —— 三个函数此时无调用方, `clippy -D warnings` 的 `dead_code` 会硬报错。T1 与 T2 实为一个交付单位, 故合并过一次三件套 (见 T2)。
  - Files: `src/widget/mod.rs` | M

- [x] **T2: `window/handler.rs` — 绘制趟 + 事件趟接线 ✅ 2026-09-12**
  - 内容: `render_frame:680` 后插 `paint_popups` (在 `paint_image` 之前); `window_event:908` 前插按下态 `dismiss_popup_at` + 弹层优先分发 (`Some(r) => r, None => tree.event(..)`); 导入补三个函数
  - Acceptance:
    - [x] 弹层绘制插在 `tree.paint` 之后、`paint_image` 之前 ✅
    - [x] 无弹层时事件路径与既有**完全等价** ✅ — 既有全量测试**零改动**绿
    - [x] 点外收起后本次事件照常继续分发 (先 dismiss 再 dispatch, 不 early return) ✅
    - [x] 焦点掉落按裁决 (a) 处理, 不扩展 `focus.rs` ✅
  - Verify: 三件套绿 — `cargo fmt` 净 / `cargo clippy --all-targets -- -D warnings` 净 / `cargo test --lib --tests` **585 通过 0 失败** (基线 577 + 新增 8) ✅
  - Files: `src/window/handler.rs` | S

### ★ Checkpoint 1: 通道语义锁死
- [x] 三件套绿 (既有全量测试零回归) ✅ 2026-09-12
- [x] `popup_*` 单测全绿, 通道语义 (优先分发/点外收起/层序/零影响) 各有锁 ✅
- [ ] **人工: 既有交互无可感变化** (待人工; `cargo run --example danqing-showcase` 过 Overlay 开合 / Tabs 切换 / 输入框光标)
- [ ] **回报用户: 通道已就绪, 确认后进 Phase 2**
- 附带: `cargo fmt` 顺带重排了在飞的 `dropdown.rs` 与 `scrollable.rs` (纯空白, 无语义变更); `dropdown.rs` 将在 T3 被整体重写。

## Phase 2: Dropdown 重写

- [x] **T3: `widget/form/dropdown.rs` 全量重写 ✅ 2026-09-12**
  - 内容: 自足组件 (组件自管展开态, D5)。**删除** `on_click` / `on_area_change` / `expanded(bool)` setter / `options()`; 新增 `on_select<M>` / `is_expanded()`; 模块头 doc 锚「两版旧实现都被证伪」的根因 + 记入鼠标选中后焦点掉落的已知限制
  - Acceptance:
    - [x] `layout` 只返回控件高度, 弹层不参与布局 ✅ (`layout_height_is_control_height_regardless_of_option_count`, 1 项 vs 50 项等高)
    - [x] `popup_area()` 展开 = `Some` (控件下方、同宽) / 收起 = `None` ✅
    - [x] `paint_popup` 画底板+边框+行文本+hover/选中高亮, **不调 `push_layer`** ✅
    - [x] `hit_area()` 展开 = 控件 ∪ 弹层 (手工 min/max 并集, 未新增 `Rect::union`), 收起 = 仅控件 ✅
    - [x] 鼠标: 点控件 toggle / 点弹层行 = 选中 + 收起 + 经 `on_select` 发消息 (内容 = 行索引) ✅
    - [x] 键盘: ↑↓ 不越界 / Enter 选中发消息 / Esc 收起**且不改选中值** / 收起态 ↑↓ 展开 ✅; 收起态 Esc 返回 `Ignored` 留给 app 级协议 (另有专项测试)
    - [x] `on_popup_dismiss` / `reset_focus` 均收起; 全部走 `Theme` token ✅
    - [x] 额外: 空选项列表不展开; 光标移出弹层清 hover ✅ (实写 11 条)
  - **偏离备案**: spec 写的「弹层阴影 `shadow_md()`」**未做** —— 框架**无阴影绘制原语** (`Shadow` 只有 token, `grep shadow_` 全 src 零命中, 现有组件一律不画), 新增原语属「Ask first」范围。弹层改用 `surface_input()` (0.95 不透明, 足以遮住底下内容) + `border()` 达到分离感。
  - **次要取舍**: showcase 演示选项 10 → 6 —— v1 无滚动/无翻转, 10 项弹层高 368px 必越窗口底部, 演示会显示成坏的。
  - Verify: `cargo test --lib dropdown` — 11 全绿 ✅
  - Files: `src/widget/form/dropdown.rs` | M

- [x] **T4: `examples/showcase.rs` 换自足用法 + 删旧样板 ✅ 2026-09-12**
  - 内容: 演示卡改 `Dropdown::themed(t, options).width(200.0).bind_selected(..).on_select(Msg::DropdownSelect)`; 删除三字段 / 三 `Msg` / `on_key` 导航段 / `dropdown_overlay()` 整个函数 (及其在根 Stack 的挂载); 保留 `DROPDOWN_OPTIONS` 常量供回显
  - Acceptance:
    - [x] 四类样板**零残留** ✅ (grep `dropdown_overlay|dropdown_expanded|dropdown_hover_idx|dropdown_area|DropdownToggle|DropdownClose|DropdownArea` 在 showcase 内**唯一命中是我自己写的「已删除」说明注释**)
    - [x] 演示卡只留一个选中回调, 无应用侧弹层代码 ✅ (旧版: 4 Msg + 3 字段 + 20 行导航 + 50 行 Overlay 装配)
    - [x] 弹层**不再挂载到根 Stack** —— 由框架弹层通道在根绘制末尾统一绘制 ✅ (注释就地说明)
    - [ ] 人工姿势: 开合 / ↑↓ / Enter / Esc / 点外收起 / **弹层盖住下方兄弟卡片** / **选中后键盘仍可继续** (待人工)
  - Verify: 三件套绿 ✅ — `cargo fmt` 净 / `clippy --all-targets -D warnings` 净 / `cargo test --lib --tests` **596 通过 0 失败** (基线 577 + 弹层 8 + Dropdown 11)
  - Files: `examples/showcase.rs` | S

### ★ Checkpoint 2: 实机验收
- [x] 三件套绿; 代码侧验收标准逐条勾 ✅ 2026-09-12
- [x] **人工验收通过** ✅ 2026-09-12 (用户实机确认「可以了」)
  - 首轮实机揪出两个 bug, 均已修: ① 箭头躺平成横杠 (`push_line` 的 rotation 疑似失效 → 改用 `push_diagonal`, 立 T7); ② 下拉点不开 (**本次改动自身引入的回归** —— 见 T6 第二版翻车记录)
  - 复验时追加的收尾: 箭头由 10×8 缩到 7×4 (用户反馈「像大 V」), 笔画提为 `ARROW_STROKE` 常量
  - 追加收尾 2: 弹层**选中行 hover 时不换成 hover 底色** (选中态优先于 hover, 用户要求) —— 判断抽成可单测的 `row_fill`, 新增 `selected_row_keeps_its_fill_under_hover`
  - 追加收尾 3: 摘除 `hovered` 字段 —— 重写时留下的**只写不读**死重量 (5 处赋值、0 处读取), 控件本就不画 hover 态, 删除零行为变更
- [ ] **用户裁决 commit/push** (D8) —— 代码侧全部就绪, 一行未提交
- 备注: T5/T6 已完成; T7 (`push_line` rotation) 为衍生挂账, 未在本批修。

## 评审记录 (2026-09-12, `/agent-skills:code-review-and-quality`)

派**两个独立上下文**评审 (五轴通用 + 测试质量专项), 裁决 **Request changes**。发现与处置:

| 发现 | 级别 | 处置 |
|---|---|---|
| **弹层通道无视模态屏障**: `paint_popups`/`dispatch_popup_event`/`dismiss_popup_at` 从 root 无差别递归, `Overlay` 的模态语义只活在它自己里 → 程序化打开模态时, 底层弹层**画在 scrim 之上并抢走模态该吞的点击**, 正面违反 spec 非目标「不改 Overlay 模态语义」 | 结构 | **已修** — 新增 `Widget::modal_barrier()` (Overlay 开态返 true), 三个通道函数把作用域收束到最上层屏障子树; 语义为**冻结而非收起** |
| **已知限制的症状写错**: 焦点不是「落空」, 而是落到弹层下方**那个被遮住的控件**上 (`focus.rs` 的命中遍历只读 `hit_area`, 从不读 `popup_area`, 且在事件分发之后运行) | 必须改 | 已修 doc (组件模块头 + spec) |
| `push_line` 是 **pub** API、零调用方、零测试、rotation 疑似失效 —— clippy 因 pub 不报 `dead_code`, 留着是给下一个想画斜线的产品一个静默画错的入口 | 必须改 | **已删除** (T7 结案) |
| `child_area` 公式在 Scrollable 里重复三处 | 结构 | 已抽 `Scrollable::child_area()`, 三处共用 —— 让「逐字一致」成为**函数级**保证而非注释级约定 |
| **测试 C1**: 点选测试只点末行, 而末行恰是 `row_index_at` 的 clamp 边界 → 丢掉 `list_pad` 偏移也全绿 | Critical | 已补「首/中/末逐行」+「行边界 ±1px」两条 |
| **测试 C2**: 点击坐标从被测的 `popup_area()` 反推 → 弹层纵向几何的任何偏差两边同源抵消, 永远自洽 | Critical | 已改为**全由主题 token 字面推出**; 几何断言由「只有下界」改为精确相等 |
| 测试 M1–M5: `layout` 高度未钉死 (恒返 0 也全绿) / 键盘守卫无锁 / 键盘决策表 3 个空格 (ArrowUp×收起、Enter×收起、Space) / 弹层外按下未测 / 驱动层组合无锁 | 必须改 | 逐条补齐; 另补「经真实容器 (Column) 的端到端」一条 |
| `dispatch_popup_event` 「命中即拥有、owner 的 `Ignored` 不回落」的语义未写明 | Nit | 已补 doc |
| 弹层越出 Scrollable 视口时 hover 失效但点击可用 | Nit | **已知未修** (v1 非目标未覆盖此细节, 记此) |
| 弹层底色与控件同为 `surface_input()` (0.95), 可能极淡透出被遮文字 | Nit | 人工验收已过, 暂留; 若日后观感不佳再叠一层更实的面 |

**变异测试 (验证锁是真锁, 不是装饰)** —— 逐个注入反例实现, 确认对应测试变红后完整还原:

| 注入的反例 | 结果 |
|---|---|
| `row_index_at` 丢掉 `list_pad` 偏移 | 行边界测试 **FAILED** ✅ |
| `layout` 高度恒返回 0 | layout 测试 **FAILED** ✅ |
| 去掉 `event` 里的 `pos.is_some()` 守卫 | 几何缓存测试 **FAILED** ✅ |
| 弹层优先分发打瘸 (`dispatch_popup_in` 恒返 None) | 端到端测试 **FAILED** ✅ |

四发四中。**这是本批唯一能证明「测试锁住了」的方式** —— 上一轮的教训正是「测试全绿」本身不构成证据。

## Phase 3: 收尾 (待裁决)

- [x] **T5: 回退 `overlay.rs` 的 `bind_position` / `bind_width` ✅ 2026-09-12**
  - 处置: 销毁前先把要回退的 API 形状**留档进 `docs/specs/SPEC-dropdown.md` 裁决 2** (工作树改动无 reflog 可救, 不落档就真丢了), 再 `git checkout HEAD -- src/widget/view/overlay.rs`
  - Acceptance: 两绑定与 `cached_position`/`cached_width` **零残留** ✅ (grep 全 src/examples 无命中); `card_rect` 恢复居中语义 ✅
  - 附带收益: 同批还原了被删的两条注释 (「开→关边沿清内容子树焦点视觉」/「关态子树完全休眠」), 它们承载 C1/R1 判例依据, 本就不该随重构丢
  - Verify: overlay 单测全绿 ✅
  - Files: `src/widget/view/overlay.rs` | XS

- [x] **T6: `scrollable.rs` 坐标系修复 ✅ 2026-09-12 (两轮都错了, 第三版才对 —— 全过程留档)**
  - **正确契约** (最终结论): 子组件在 event 中收到的矩形必须与 **`paint` 给它的逐字一致** —— 原点「视口原点 − 滚动偏移」, 尺寸取内容尺寸; 且事件坐标**保持屏幕坐标不做任何变换**。因为子组件在 paint 里缓存这个矩形、在 event 里又拿它做命中判定 (`Button`/`Switch`/`Dropdown` 全是 `area.contains(pos)`), 两处必须同源。屏幕点落在这个被上移了偏移的矩形里, 恰好等价于内容坐标下压中的那条像素。
  - **第一版 (在飞 WIP) — 错**: 传「视口矩形减偏移」+ 变换坐标 → 差 **2×offset**, 比不改还远。探针实测: `area=(0,-900,100,100)` vs `pos=(50,950)`。
  - **第二版 (我) — 也错**: 传 `(0,0,child_size)`。逻辑上自洽 (内容坐标系), 但**与 paint 不同源**, 且只在「视口原点为 (0,0) 且未滚动」时才碰巧成立。实测当场炸掉整个 showcase 表单页: 视口 origin.x ≈ 200 (侧栏右侧), 点击坐标是屏幕坐标 → 全部落空, **下拉点不开**。用户实机截图揪出。
  - **第三版 (现)** — 见上「正确契约」; 一并删除 `transform_event` (它「加偏移」的语义正是错误源头, 删后成为死代码)。
  - Acceptance: `child_event_area_matches_paint_area_at_any_viewport_origin` —— 探针同时记录 **paint 收到的矩形**与 **event 收到的 (矩形, 坐标)**, 三条断言各锁一个坏过的点: ① `event area == paint area`; ② 坐标保持屏幕坐标; ③ **视口刻意不在原点 (250, 40)** —— 正是第二版翻车的条件 ✅
  - Verify: 全量 **597 通过 0 失败** (含 `focus::tests::click_outside_scrollable_viewport_does_not_focus_child` 零回归) ✅; 三件套绿
  - Files: `src/widget/view/scrollable.rs` | XS
  - **⚠ 影响面**: 框架**行为变更** —— 「滚动后的容器内点击」由不可用转为可用 (原版只在未滚动时正常)。产品侧若曾绕过 (例如把可点列表挪出 Scrollable), 需复验。**建议单独提交** (D9)。
  - **教训**: 这个 bug 有两个「隐身条件」—— ① 视口在原点; ② 未滚动。满足二者时一切正常。回归测试必须**主动打破这两个条件**, 否则锁不住。

- [ ] **T7 (衍生立案): `RectBatch::push_line` 的 rotation 疑似渲染不出, 斜线躺平**
  - 现象: Dropdown 折叠箭头原用两条 `push_line` 画折线, 实机渲染成一根**横杠**。
  - 证据 (数值吻合): `push_line` 未旋转时的轴对齐包围盒是 `(length + thickness) × thickness` = `(9.43 + 1.5) × 1.5` ≈ **11 × 1.5**; 截图里的横杠量级正是 ~11px 宽 / ~2px 高。若 rotation 生效应为 ~58° 斜线。
  - 已核实齐备的部分: `RectInstance` 字段顺序与 `vertex_attr_array!` (位置 4) **一致**; `rect.wgsl` 也读 `@location(4) rotation` 并做了绕中心旋转 + 像素映射 —— 数学本身看着是对的。所以根因**未定**, 需 GPU 侧复现。
  - 影响面: `push_line` 全仓库**只有 dropdown 一个调用方** (即本次新写的), 故长期无人验证; 该路径可能自加入起就是坏的。`push_diagonal` (圆点队列逼近对角线, 不依赖 rotation) 是 TitleBar × / CloseButton × / 时钟指针走的**已验证**路径。
  - 本批处置: dropdown 箭头改用 `push_diagonal`, 并在代码里留「勿改用 `push_line`」的注释。**`push_line` 本身未修** —— 属渲染管线改动, 需实机复现 + 独立评估, 不在本次范围。
  - Files: `src/render/rect.rs` + `src/render/rect.wgsl` | 规模待定

## 交接

(待 CP2 填写: 提交号 / 人工验收结论 / 衍生挂账)
