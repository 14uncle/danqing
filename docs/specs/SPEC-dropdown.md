# SPEC: Dropdown 下拉选择器重写 + 框架弹层通道

- @author 十四叔
- @date 2026/09/12
- 状态: **已批准**（2026-09-12，用户「按你推荐开工」一次性批准：① 架构 = 框架延迟弹层通道；② v1 范围 = 键盘 + 点外关闭；③ 展开态组件自管；④ 回退 `overlay.rs` 定位绑定；⑤ 焦点掉落按 (a) 接受为 v1 已知限制；⑥ `scrollable.rs` 修复单独提交）
- 需求来源: 用户 2026-09-12 指令「之前写的乱七八糟，下拉选择器推倒重新写」+ 同日两项裁决（① 架构 = 框架延迟弹层通道；② v1 范围 = 键盘 + 点外关闭）
- 前置事实: 工作区在飞改动已勘察（`dropdown.rs` 纯控件版 / `overlay.rs` 定位绑定 / `widget/mod.rs` 的 `popup_area` 死代码 / `scrollable.rs` 坐标系修复 / `showcase.rs` 应用侧组装）

## 目标

两件事，第二件依赖第一件：

1. **框架新增弹层通道**——树的绘制末尾统一绘制弹层，事件分发优先查弹层区。这是框架级设施，任何组件都能长在上面。
2. **在其上重写 `Dropdown`** 为自足组件——`Dropdown::new(options).bind_selected(..).on_select(..)` 拖进页面树即可用，应用零样板。

**成功的样子**：`examples/showcase.rs` 的「下拉选择器」演示卡不再需要任何应用侧弹层代码——`Showcase` 里的 `dropdown_expanded` / `dropdown_area` / `dropdown_hover_idx` 三个字段、`DropdownSelect` / `DropdownClose` / `DropdownArea` 三个 `Msg`、`on_key` 里 20 行导航、以及 `dropdown_overlay()` 整个函数（50 行 Overlay 装配）**全部删除**。

### 为什么必须新建设施（根因）

两版旧实现都在绕同一个洞，且绕法都被证伪：

| 版本 | 做法 | 死因 |
|---|---|---|
| 第一版（已提交 `31e7b04`） | 弹层画在**自己区域内**（头注释自承「不使用 Overlay, 简化实现」） | 被后续兄弟覆盖；撑坏布局 |
| 第二版（工作区在飞） | 弹层**推给应用**，用 Overlay 组装 | 样板外溢到每个用 Dropdown 的产品；与「引擎复用率是产品线核心指标」正面冲突 |

框架侧两条硬约束（均已核到行）：

- **绘制**：`push_layer()` 只是**位置边界标记**（`render/rect.rs:116`），层号大者后画。树中间的组件在自己 paint 时调 `push_layer()`，只能盖住「此前已画完的」；排在它**后面**绘制的兄弟会落进同一个新层、按序画在弹层之上。→ 自足弹层**不可能**靠组件自己 `push_layer()` 实现，必须延迟到主树绘制之后再统一绘制。
- **命中**：各容器按子组件 **layout 矩形**设门（`layout/flow.rs:291`、`layout/box_.rs:238`、`view/scrollable.rs:348`；`layout/stack.rs` 无门是例外）。弹出区一旦超出控件自身矩形就收不到事件；且弹层可能跨越**任意多级**祖先的矩形，逐级放宽命中门不可行。→ 弹层命中必须发生在**根级**、用绝对坐标判定，与祖先矩形无关。

`Widget::popup_area()`（`widget/mod.rs:187`）此前已被加入 trait 但**无实现者、无调用方**——它是这个设施的第一块砖，本 spec 把它接线完毕（架构裁决为方案 A，故**保留并补全**，不是删除）。

### 非目标（明确不做）

- 长列表**滚动**（v1 弹层限制在可视行数内；超长列表请自行控制选项数）
- 空间不足**向上翻转**（v1 恒在控件下方展开；贴近窗口底部的使用姿势 v1 不保证）
- **打字过滤** / 分组 / 多选 / 禁用态 / 虚拟滚动
- 弹层开合**动画**（未来可叠 `anim` 原语）
- **嵌套弹层**（弹层里再开弹层；通道按「首次命中即送达」实现，嵌套语义未定义）
- **改动 Overlay 的模态语义**——弹层通道是 Modal 之外的另一条路，Overlay 现状不动（除下方「回退项」）
- xirang（已埋，勿主动推进）

## 设计一：框架弹层通道

「画得对」与「点得中」分开解。

### trait 新增（`src/widget/mod.rs`）

```rust
/// 弹出层命中区域 (相对于窗口逻辑坐标)。
/// 返回 Some 即视为「本组件当前有弹层展开」, None = 无弹层。
/// 已有方法, 本次接线。
fn popup_area(&self) -> Option<Rect> { None }

/// 绘制弹出层内容。
/// 仅在 `popup_area()` 返回 Some 时由框架调用, `area` 即该区域。
/// 框架负责开新层 (push_layer 配对), 组件不得自行 push_layer。
fn paint_popup(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {}

/// 弹层外点回调: 弹层展开期间, 按下点落在「弹层区域之外 且 本组件 hit_area 之外」时调用。
/// 组件应据此收起弹层 (实现内自行改状态, 不发消息)。
fn on_popup_dismiss(&mut self) {}
```

签名与默认体风格对齐既有 `Widget::event`（`_event/_area/_msgs` 下划线前缀）。

### 纯逻辑分发函数（`src/widget/mod.rs`，新增）

放 `widget/` 而非 `window/`——它们是纯树遍历，不碰平台/GPU，守住依赖方向铁律。

```rust
/// 绘制全部弹层: 深度优先遍历, 对每个 `popup_area()` 为 Some 的节点
/// 配对 `rects.push_layer()` + `texts.push_layer()` 后调 `paint_popup`。
/// 层号单调递增 → 弹层恒在主树之上, 后遍历者在上。
pub fn paint_popups(root: &Node, rects: &mut RectBatch, texts: &mut TextBatch);

/// 弹层优先事件分发: 深度优先找第一个「popup_area 包含事件位置」的节点,
/// 以该区域为 `area` 直接送达事件。返回 None = 无弹层命中, 调用方继续常规分发。
pub fn dispatch_popup_event(root: &mut Node, event: &Event, msgs: &mut MsgQueue) -> Option<EventResult>;

/// 点外收起: 若存在展开弹层且 `p` 落在该组件的 popup_area 与 hit_area 之外,
/// 调用其 `on_popup_dismiss`。返回是否有弹层被收起。
pub fn dismiss_popup_at(root: &mut Node, p: Point) -> bool;
```

`push_layer` 配对收口在 `paint_popups` 内**唯一一处**，组件无从漏压——规避 `render/mod.rs:291` 那条「漏压一侧静默错乱」的坑。

### 驱动接线（`src/window/handler.rs`）

**绘制**（`render_frame`，现 `handler.rs:680` 之后）：

```rust
self.tree.paint(self.root_area, &mut rects, &mut self.texts);
paint_popups(&self.tree, &mut rects, &mut self.texts);   // 弹层趟: 层号最大 → 恒在最上
self.tree.paint_image(self.root_area, &mut self.images);
```

**事件**（`window_event`，现 `handler.rs:908` 之前）：

```rust
// 1. 鼠标按下: 先判点外收起 (点在弹层与控件之外 → 收起, 本次事件照常继续分发)
if let Event::MouseInput { pressed: true, position, .. } = event {
    dismiss_popup_at(&mut self.tree, position);
}
// 2. 弹层优先: 命中则直达该组件; 未命中继续常规分发
match dispatch_popup_event(&mut self.tree, &internal, &mut self.msgs) {
    Some(result) => { /* 按 result 决定是否冒泡, 同现有 Consumed 语义 */ }
    None => { self.tree.event(&internal, self.root_area, &mut self.msgs); }
}
```

**为什么弹层优先只作用于鼠标按下，不作用于 `CursorMoved`**：`CursorMoved` 走既有**广播**路径（`flow.rs:272`，送达每个子组件），弹层内的 hover 高亮自然可达；若让弹层优先截走广播，其余组件的 hover 残留将无法清除。滚轮同理不入 v1（无滚动）。

**点外收起与常规分发的关系**：先收起、再照常分发——点别处的按钮时「收起弹层 + 按钮生效」同时发生，符合桌面惯例。点在控件**自身** `hit_area` 内不算点外，事件走常规分发让控件自行 toggle。

### 模态屏障（评审补充，2026-09-12）

初版通道从 root **无差别递归**，而 `Overlay` 的模态语义只活在它自己的 paint/event 里。后果是程序化打开模态（异步回调、快捷键）时：

- `paint_popups` 跑在 `tree.paint` 之后，弹层 `push_layer` 的层号**大于** Overlay 在树内开的 scrim 层 → **底层弹层盖在模态遮罩之上**；
- `dispatch_popup_event` 跑在 `tree.event` 之前 → 遮罩区内的点击被弹层持有者接走，Overlay「模态不得冒泡到被盖住的底层」被绕过。

这正面违反本 spec「不改动 Overlay 的模态语义」的非目标，故补一条设施：

```rust
/// 是否构成模态屏障 (打开态的模态浮层)。默认 false。
fn modal_barrier(&self) -> bool { false }
```

`Overlay` 开态返回 true；三个通道函数先用 `popup_scope()` 求出**最上层**（z 序最后、嵌套取更靠内）屏障的路径，再把作用域收束到该子树。没有屏障时作用域即整棵树——**关态屏障不产生任何作用域**，零影响。

**语义是「冻结」而非「收起」**：屏障存在期间层外的弹层既不绘制也不响应，屏障移除后原样恢复，不改变任何组件的展开态。（若日后要「模态一开就收起底层弹层」，那是另一条策略，不在本设施内。）

## 设计二：Dropdown 重写

`src/widget/form/dropdown.rs` **全量重写**，不留旧结构。

```rust
pub struct Dropdown { /* 选项 / 选中索引 / 绑定 / 消息工厂 / 展开态 / hover 索引 /
                         控件矩形缓存 / 弹层矩形缓存 / 宽度 / 主题色 */ }

impl Dropdown {
    /// 默认浅色主题创建。
    pub fn new(options: Vec<String>) -> Self;
    pub fn themed(theme: &impl Theme, options: Vec<String>) -> Self;
    /// 绑定选中索引 (从应用状态读取)。
    pub fn bind_selected<S: 'static>(self, f: impl Fn(&S) -> usize + 'static) -> Self;
    /// 选中回调: 选中某项时产出应用消息 (idx)。
    pub fn on_select<M: 'static>(self, f: impl Fn(usize) -> M + 'static) -> Self;
    /// 固定宽度 (None = 填满可用宽度)。
    pub fn width(self, w: f32) -> Self;
    /// 当前是否展开 (只读; 展开态由组件自管, 见「待裁决」1)。
    pub fn is_expanded(&self) -> bool;
}
```

**删除**：旧版的 `on_click`、`on_area_change`、`expanded(bool)` setter、`options()` 取值器——前两个是「把弹层推给应用」的产物，后两个在自足形态下无意义。

### Widget 实现要点

| 方法 | 行为 |
|---|---|
| `sync` | 仅同步 `selected`（展开态不绑定） |
| `layout` | **只按控件高度返回**：`(width.unwrap_or(constraints.max().width), theme.control_height())`。**弹层不参与布局**——不撑高、不占位。这是第一版老毛病的回归锁 |
| `paint` | 只画控件主体 + 选中文本 + 箭头（展开时箭头朝上） |
| `popup_area` | 展开时 `Some(弹层矩形)`（绝对坐标），收起时 `None` |
| `paint_popup` | 画弹层：底板 + 边框 + 阴影 + 每行文本 + hover/选中行高亮。**不调 `push_layer`** |
| `event` | 见下 |
| `focusable` | `true` |
| `hit_area` | 收起 = 控件矩形；展开 = 控件 ∪ 弹层（并集手工取 min/max 计算，`Rect` 无 `union`，不为本次新增该 API）。**必须含弹层**，否则点选项时焦点会掉，键盘导航随之中断 |
| `on_popup_dismiss` | 展开则收起，清 hover 索引 |
| `reset_focus` | 收起 + 清 hover（容器隐藏子树时由框架调用，行为与 Overlay 关层边沿一致） |

### 弹层几何

```
弹层矩形 = (控件.x, 控件底边 y + GAP, 控件宽, 行高 × 选项数)
行高   = theme.control_height()
```

控件矩形在 `layout`/`paint` 时缓存（`paint` 收到的 `area` 已是绝对坐标，容器逐级 translate）。弹层矩形每次按缓存控件矩形重算，随滚动/布局变化自然跟随。

### 事件语义

**鼠标**（弹层内的按下经 `dispatch_popup_event` 直达，`area` 参数即弹层矩形；控件上的按下走常规分发，`area` 即控件矩形——用两个缓存矩形区分）：

- 控件上左键按下 → 切换展开态；展开时 hover 索引置为当前选中项
- 弹层内左键按下 → 选中该行、收起、经 `on_select` 产出消息
- 弹层内 `CursorMoved` → 更新 hover 索引（经既有广播到达）
- 控件上 `CursorMoved` → 更新控件 hover 态

**键盘**（经焦点树 path 直达组件，`event_at_path`，`widget/mod.rs:210`；仅聚焦时可达）：

| 键 | 收起态 | 展开态 |
|---|---|---|
| `ArrowDown` | 展开 | hover 下移（不越界） |
| `ArrowUp` | 展开 | hover 上移（不越界） |
| `Enter` / `Space` | 展开 | 选中 hover 行、收起、发消息 |
| `Escape` | 忽略 | 收起（不改选中值） |

**主题 token**（一律走 `Theme`，零魔法值）：底板 `surface()`、边框 `border()`、阴影 `shadow_md()`、hover 行 `surface_variant()`、选中行 `selection()`、正文 `text_primary()`、圆角 `radius_md()`、内边距 `spacing_md()`/`spacing_sm()`、字号 `font_size_body()`、控件高 `control_height()`。

## 结构

- `src/widget/mod.rs` — `paint_popup` / `on_popup_dismiss` 两个 trait 方法；`popup_area` 保留并接线；新增 `paint_popups` / `dispatch_popup_event` / `dismiss_popup_at` 三个纯函数 + 内联单测
- `src/window/handler.rs` — `render_frame` 追加弹层绘制趟；`window_event` 追加点外收起 + 弹层优先分发
- `src/widget/form/dropdown.rs` — 全量重写（模块头 doc 写做什么/不做什么/为什么，为什么锚「两版旧实现都被证伪」的根因）
- `examples/showcase.rs` — 删除 `dropdown_overlay()` 与三字段/三 Msg/`on_key` 导航；演示卡改为自足用法，并**在卡片下方放一个带文字的兄弟卡片**，用于目视验证「弹层盖住后续兄弟的文本」
- `src/lib.rs` — 无需改动（`Dropdown` 已在 re-export 名单）

## 命令

```bash
cargo fmt                                                    # 格式化
cargo clippy -- -D warnings                                  # 静态检查 (必须零警告)
cargo test --lib --tests                                     # 全部测试 (纯逻辑, 无需 GPU)
cargo run --example danqing-showcase                         # 人工验收 (开 GUI 窗口)
```

## 代码风格

```rust
// 主题 token 取色, 零魔法值; 缓存矩形走 Cell (paint 是 &self)。
impl Widget for Dropdown {
    fn paint_popup(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        rects.push_rect(area, self.bg_color, self.radius);
        rects.push_rounded_border(area, self.border_color, self.radius, 1.0);
        for (i, opt) in self.options.iter().enumerate() {
            let row = Self::row_rect(area, i, self.row_height);
            if i == self.hover_idx || i == self.selected {
                let fill = if i == self.hover_idx { self.hover_color } else { self.selected_color };
                rects.push_rect(row, fill, 0.0);
            }
            texts.push_text(opt, row.origin.x + self.padding, baseline, size, self.text_color);
        }
    }
}
```

- 公开类型/函数中文文档注释；内部实现英文命名
- 新 `.rs` 文件头 `//! @author 十四叔` + `//! @date yyyy/MM/dd`
- 公开 API 经 `src/lib.rs` re-export

## 测试策略

纯逻辑单测，无需 GPU（`#[cfg(test)]` 内联，探针组件仿 `overlay.rs` 的 `Probe` 范式）。

**弹层通道（`widget/mod.rs`）**——构造带 `popup_area` 的探针，锁住框架语义：

- 点在弹层区 → 事件直达该组件，**同层兄弟不得收到**（优先于后绘制者）
- 点在弹层区外 → `dispatch_popup_event` 返回 `None`，常规分发不受干扰
- 无弹层（`None`）→ 三个函数均为空操作，既有分发零影响
- 点外松手/点在弹层内/点在控件内三种情形下 `on_popup_dismiss` 的调用与否
- `paint_popups` 后 `RectBatch`/`TextBatch` 层数各 +弹层数，且两者层数**相等**（`push_layer` 配对）

**Dropdown**——锁住组件语义：

- **`layout` 只返回控件高度**（弹层不撑高）——第一版老毛病回归锁
- 展开态 `popup_area()` 为 `Some` 且位于控件下方、宽度等于控件宽；收起为 `None`
- 点击控件 toggle；点击弹层行 → 选中 + 收起 + 经 `on_select` 发消息（消息内容 = 行索引）
- 键盘：↑↓ 移动不越界、Enter 选中并发消息、Esc 收起且**不改选中值**、收起态 ↑↓/Enter 展开
- `on_popup_dismiss` 收起；`reset_focus` 收起
- `hit_area` 展开时含弹层、收起时仅控件

**人工验收**（showcase）：开合、键盘全键位、点外收起、点选项、弹层盖住下方兄弟卡片文字。

## 验收标准

- [ ] `Showcase` 结构体中 `dropdown_expanded` / `dropdown_area` / `dropdown_hover_idx` 三字段删除；`DropdownSelect` / `DropdownClose` / `DropdownArea` 三 `Msg` 删除；`on_key` 中下拉导航段删除；`dropdown_overlay()` 函数删除
- [ ] Dropdown 自足：演示卡只需 `Dropdown::new(..).bind_selected(..).on_select(..)`
- [ ] 弹层绘制在**所有主树内容之上**（目视：盖住下方兄弟卡片的文字与矩形）
- [ ] 弹层区内点击能选中选项，且不被覆盖其下的兄弟组件抢走
- [ ] 键盘 ↑↓/Enter/Esc 全套可用；点外收起可用
- [ ] `popup_area()` 不再是死代码——有实现者、有调用方、有单测
- [ ] 三件套全绿：`cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests`
- [ ] 既有 Overlay / Tabs / 焦点 / Scrollable 测试零回归

## 边界

- **Always**：提交前三件套；新组件行为必须进 showcase；文件头 @author/@date；注释文档中文；公开 API 经 `lib.rs`；颜色/间距/圆角走 Theme token
- **Ask first**：改动 Overlay 既有语义；新增 Theme token；调整 `handler.rs` 中事件的既有处理顺序；给 `Rect` 增加新方法
- **Never**：未获指示 commit/push；改 `focus-history.json` 数据格式；为 Dropdown 引入第三方依赖；让 `widget/` 依赖 `winit`/`wgpu`

## 裁决结果（2026-09-12 全部落定）

1. **展开态归组件自管** —— 采纳。应用没有第二个开合入口，开/合全由控件点击、键盘、点外收起驱动；框架既有 `TextInput`/`Button` 同样自管临时 UI 态（光标、hover、pressed），并非首例。**代价已知**：日后若要「多实例互斥」或「程序化开合」，须改成绑定式。多实例互斥在 v1 由「点外收起」自然达成——点 A 之外的任何位置都会先收起 A。
2. **回退 `overlay.rs` 的 `bind_position` / `bind_width`** —— 采纳。其存在理由是被本 spec 否决的「应用侧 Overlay 组装弹层」方案；架构定为弹层通道后用途消失，仅余 `cached_position`/`cached_width` 的复杂度。回退见 `tasks/todo-dropdown.md` T5。

   **回退掉的代码留档**（2026-09-12）：工作树改动无 reflog 可救，故记于此，日后若「可定位浮层」对 tooltip/菜单另有价值可据此复活。回退的 API 是两个 builder + 四个字段 + `card_rect` 的分支：

   ```rust
   /// 绑定内容卡位置 (不设则居中)。
   pub fn bind_position<S: 'static>(mut self, f: impl Fn(&S) -> Point + 'static) -> Self;
   /// 绑定内容卡宽度 (不设则按内容)。
   pub fn bind_width<S: 'static>(mut self, f: impl Fn(&S) -> f32 + 'static) -> Self;

   // 字段: position_binding / width_binding (Box<dyn Fn(&dyn Any) -> _>)
   //       cached_position: Cell<Point> / cached_width: Cell<Option<f32>>
   // sync 缓存两者; card_rect 于 position_binding 为 Some 时用 (pos, 缓存宽)
   // 而非居中; layout 于 cached_width 为 Some 时以该宽 loose 约束内容。
   ```

   同批一并还原的还有 `sync` 中被删掉的两条注释（「开→关边沿清内容子树焦点视觉」与「关态子树完全休眠」），它们承载 C1/R1 的判例依据，本就不该随重构丢掉。
3. **`scrollable.rs` 坐标系修复单独提交** —— 采纳，不入本次方向（T6）。
4. **本次 plan / todo 落 `tasks/plan-dropdown.md` + `tasks/todo-dropdown.md`** —— 与原待裁决 4 的担忧不同，`docs/tasks/` 是磁盘分析器按 `danqing-disk/CLAUDE.md` 明文规定占用的位置，框架计划一直在 `tasks/`（`plan-overlay.md` 同族），两者不冲突。#4 系勘察失误，已作废。
5. **鼠标选中后焦点掉落** —— 裁为 **(a) 接受为 v1 已知限制**，写进组件 doc。纯键盘路径不受影响；受影响的是「先鼠标选、再键盘导航」这一条次要流。改动面收在本次范围内，不拖入焦点系统。
