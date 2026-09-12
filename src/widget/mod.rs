//! @author 十四叔
//! @date 2026/07/17

//! 组件：保留模式 UI 树的节点抽象。
//!
//! 本模块为纯逻辑：组件只依赖布局类型与 CPU 收集器，
//! 不接触任何平台 / 图形 API。
//!
//! 每帧流程：
//! 1. [`Widget::sync`] —— 从应用状态同步绑定属性;
//! 2. [`Widget::layout`] —— 约束向下传、尺寸向上算;
//! 3. [`Widget::paint`] —— 按缓存的几何收集绘制命令。

mod base;
mod focus;
mod form;
mod layout;
mod title_bar;
mod view;

pub use base::{Button, CloseButton, Image, Text};
pub use focus::FocusManager;
pub use form::{Dropdown, IconInput, Switch, TextArea, TextInput};
pub use layout::{Box, Center, Column, CrossAlign, DragArea, Padding, ReachArea, Row, Stack};
pub use title_bar::{LogoKind, TitleBar, TitleBarStyle};
pub use view::{MultiPanel, Overlay, ScrollAxis, Scrollable, Tabs};

use std::any::Any;

use crate::app::AnimationCtx;
use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::{Color, Constraints, Point, Rect, Size};

/// 事件处理结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// 事件被消费，停止分发。
    Consumed,
    /// 未消费，继续向父级冒泡。
    Ignored,
}

/// 应用消息队列：组件事件 (如按钮点击) 产出的类型擦除消息。
pub type MsgQueue = Vec<std::boxed::Box<dyn Any>>;

/// 用轴对齐小圆点队列近似一条对角线 (crate 内共享：
/// TitleBar 的 ×/时钟指针与 CloseButton 的 × 同一算法)。
///
/// 每个步进放置一个 `thickness × thickness` 的圆角矩形，
/// 圆角半径为 `thickness/2` 使其呈圆形，彼此重叠形成平滑线段。
pub(crate) fn push_diagonal(
    rects: &mut RectBatch,
    p1: Point,
    p2: Point,
    thickness: f32,
    color: Color,
) {
    if thickness <= 0.0 {
        return;
    }
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length < 1e-6 {
        return;
    }
    let half = thickness * 0.5;
    // 步长取 thickness 的一半，让小圆点高度重叠，对角线看起来更实心。
    let step = thickness * 0.5;
    let count = (length / step).ceil().max(1.0) as usize;
    for i in 0..=count {
        let t = i as f32 / count as f32;
        let x = p1.x + dx * t;
        let y = p1.y + dy * t;
        rects.push_rect(
            Rect::from_xywh(x - half, y - half, thickness, thickness),
            color,
            half,
        );
    }
}

/// 组件：保留模式 UI 树的一个节点。
pub trait Widget {
    /// 状态同步：从应用状态更新绑定属性 (每帧布局前调用)。
    ///
    /// 默认实现无操作 (静态组件)。
    fn sync(&mut self, _state: &dyn Any) {}

    /// 动画更新：由框架每帧在 `sync` 之后、`layout` 之前调用。
    ///
    /// 默认实现无操作。
    fn animate(&mut self, _ctx: &AnimationCtx) {}

    /// 布局：在约束下计算自身尺寸。
    ///
    /// 容器组件在此递归子组件并缓存各自的几何，供 paint 使用。
    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size;

    /// 绘制：按 layout 缓存的几何收集绘制命令。
    ///
    /// `area` 为父组件摆放本组件的矩形 (布局结果)。
    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch);

    /// 绘制图像纹理：向 ImageBatch 推送纹理实例。
    ///
    /// 默认实现无操作。Image 组件覆盖此方法以推送纹理。
    fn paint_image(&self, _area: Rect, _images: &mut crate::render::ImageBatch) {}

    /// 事件处理 (鼠标事件经命中分发到达; 键盘 /IME 事件经焦点路由到达)。
    ///
    /// `area` 与 paint 收到的矩形一致; 组件可经 `msgs` 产出应用消息
    /// (如按钮点击)。返回 [`EventResult::Consumed`] 表示消费该事件。
    /// 默认忽略所有事件。
    fn event(&mut self, _event: &Event, _area: Rect, _msgs: &mut MsgQueue) -> EventResult {
        EventResult::Ignored
    }

    /// 当前组件是否可接收键盘焦点。
    ///
    /// 默认可聚焦组件 (如 Button/TextInput) 应返回 true。
    fn focusable(&self) -> bool {
        false
    }

    /// 稳定焦点标识：按名聚焦用 (如弹层面板关闭后焦点回到打开面板的按钮)。
    ///
    /// 默认可聚焦组件 (如 Button) 经 `.id(...)` 设置后返回; 无标识返回 `None`。
    fn focus_id(&self) -> Option<&'static str> {
        None
    }

    /// 重置焦点视觉状态 (焦点环 / 按压态 / 光标), 不派发事件。
    ///
    /// 供容器在隐藏子面板时清除旧面板内残留的焦点高亮：面板切换后，
    /// FocusOut 经 MultiPanel 的可见切片无法送达隐藏面板内的旧焦点组件，
    /// 若不主动清除，重开面板会残留上一个会话的焦点环。
    /// 默认递归所有子组件; 可聚焦叶子组件覆盖本方法清除自身状态。
    fn reset_focus(&mut self) {
        for child in self.children_mut() {
            child.reset_focus();
        }
    }

    /// 子组件列表 (用于焦点遍历与命中测试)。
    ///
    /// 容器组件应返回其所有子节点; 叶子组件默认返回空。
    fn children(&self) -> &[Node] {
        &[]
    }

    /// 当前选中的文本 (用于 Copy/Cut)。
    ///
    /// 默认可编辑组件 (如 TextInput) 在选区非空时返回文本。
    fn selected_text(&self) -> Option<String> {
        None
    }

    /// 当前组件是否需要 IME 输入法服务。
    ///
    /// 默认可编辑文本组件 (如 TextInput) 返回 true。
    fn wants_ime(&self) -> bool {
        false
    }

    /// IME 候选框应吸附的矩形 (相对于窗口逻辑坐标)。
    ///
    /// 可输入组件返回光标或输入框区域; 无 IME 需求返回 None。
    fn ime_area(&self) -> Option<Rect> {
        None
    }

    /// 鼠标命中测试使用的矩形 (相对于窗口逻辑坐标)。
    ///
    /// 默认可点击/可聚焦组件 (如 Button/TextInput) 返回自身完整区域;
    /// 无命中需求返回 None。应与 `ime_area` 区分，后者可能只覆盖光标。
    fn hit_area(&self) -> Option<Rect> {
        None
    }

    /// 弹出层命中区域 (相对于窗口逻辑坐标)。
    ///
    /// 组件展开弹出内容 (如 Dropdown 选项列表) 时返回弹出区域;
    /// 返回 `Some` 即视为「本组件当前有弹层展开」。框架在
    /// [`dispatch_popup_event`] 中优先向此区域投递**鼠标按下**, 并由
    /// [`dismiss_popup_at`] 判定点外收起; 绘制见 [`Widget::paint_popup`]。
    ///
    /// 区域用**窗口绝对坐标** —— 弹层可超出组件自身布局矩形, 乃至跨越任意
    /// 多级祖先的矩形, 因此命中判定在根级进行, 与祖先矩形无关。
    /// 默认 None (无弹出层)。
    fn popup_area(&self) -> Option<Rect> {
        None
    }

    /// 绘制弹出层内容。
    ///
    /// 仅在 [`Widget::popup_area`] 返回 `Some` 时由框架调用, `area` 即该区域。
    /// 框架在 [`paint_popups`] 中负责配对开新渲染层,
    /// **组件不得自行调用 `push_layer`** —— 漏压一侧会让浮层文字滞留低层、
    /// 被浮层自身矩形盖住, 且静默错乱。
    /// 默认实现无操作 (无弹层组件)。
    fn paint_popup(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {}

    /// 弹层外点回调。
    ///
    /// 弹层展开期间, 鼠标按下落在「弹层区域之外 **且** 本组件 [`Widget::hit_area`]
    /// 之外」时由框架调用 (见 [`dismiss_popup_at`]); 组件应据此收起弹层
    /// (实现内自行改状态, 不产出消息)。
    ///
    /// 点在控件自身 `hit_area` 内**不算**点外 —— 那是常规事件路径, 由组件
    /// 自行决定开合。默认实现无操作。
    fn on_popup_dismiss(&mut self) {}

    /// 是否构成**模态屏障** (打开态的模态浮层)。
    ///
    /// 返回 true 的节点会把弹层通道的作用域**收束到自己的子树内**:
    /// [`paint_popups`] 不再绘制屏障之外的弹层, [`dispatch_popup_event`] 不再向
    /// 屏障之外投递, [`dismiss_popup_at`] 也不再触碰屏障之外。
    ///
    /// 这是 Overlay「模态不得冒泡到被盖住的底层」在弹层通道上的对应物。没有它,
    /// 底层展开的弹层会因为绘制晚于 scrim 而盖在模态之上, 并抢走模态本该吞掉的
    /// 点击 —— 即模态被弹层击穿。
    ///
    /// 语义是**冻结而非收起**: 屏障存在期间层外的弹层既不绘制也不响应, 屏障移除
    /// 后原样恢复 (不改变任何组件的展开态)。默认 false (非屏障)。
    fn modal_barrier(&self) -> bool {
        false
    }

    /// 可变子组件列表 (用于事件分发与动画)。
    ///
    /// 容器组件应返回其所有子节点; 叶子组件默认返回空。
    fn children_mut(&mut self) -> &mut [Node] {
        &mut []
    }
}

/// 组件树节点：盒装的组件对象。
pub type Node = std::boxed::Box<dyn Widget>;

/// 把组件装箱为节点。
pub fn node(widget: impl Widget + 'static) -> Node {
    std::boxed::Box::new(widget)
}

/// 绘制全部弹出层。
///
/// 深度优先遍历组件树, 对每个 [`Widget::popup_area`] 返回 `Some` 的节点,
/// 配对开启新的矩形/文本渲染层后调用其 [`Widget::paint_popup`]。
///
/// **必须在主树 `paint` 全部完成之后调用** —— 层号单调递增, 弹层因此恒在主树
/// 之上。树中间的组件自己调 `push_layer` 做不到这点: 那只能盖住此前已画完的,
/// 排在它后面绘制的兄弟会落进同一个新层、按序画在弹层之上。
///
/// `push_layer` 的配对调用收口于此唯一一处, 组件无从漏压一侧。
pub(crate) fn paint_popups(root: &Node, rects: &mut RectBatch, texts: &mut TextBatch) {
    let path = popup_scope(root);
    let scope = match &path {
        Some(path) => at_path(root, path),
        None => root,
    };
    paint_popups_in(scope, rects, texts);
}

/// [`paint_popups`] 的递归体。
fn paint_popups_in(node: &Node, rects: &mut RectBatch, texts: &mut TextBatch) {
    if let Some(area) = node.popup_area() {
        rects.push_layer();
        texts.push_layer();
        node.paint_popup(area, rects, texts);
    }
    for child in node.children() {
        paint_popups_in(child, rects, texts);
    }
}

/// 弹层优先事件分发。
///
/// 深度优先查找第一个弹层区域包含按下位置的组件, 以该区域为 `area` 直接送达
/// 事件, 使其先于被弹层盖住的同层兄弟命中。
///
/// **只处理鼠标按下**: 移动类事件必须继续走既有的全树广播 (各组件各自维护
/// hover 等状态), 若被弹层截走, 其余组件的 hover 残留将无法清除;
/// 键盘 / IME 事件一律返回 `None`, 由焦点路由负责。
///
/// 返回 `None` 表示无弹层命中, 调用方应继续常规的树内命中分发。
/// 子节点先于自身查 (绘制时子节点层号更大, 盖在父之上), 同层兄弟倒序查
/// (与 [`crate::widget::layout::flow`] 的「后绘制者优先」一致)。
///
/// **命中即拥有该区域**: 一旦某节点的弹层命中, 它返回的 `Ignored` 也**不会**
/// 回落到常规树内分发 (被盖住的兄弟同样收不到) —— 语义是「这块区域属于弹层持有
/// 者」。调用方据返回的 `EventResult` 决定要不要继续冒泡到 app 层。
///
/// 存在打开的模态屏障时, 作用域收束到最上层屏障的子树 (见 [`Widget::modal_barrier`])。
pub(crate) fn dispatch_popup_event(
    root: &mut Node,
    event: &Event,
    msgs: &mut MsgQueue,
) -> Option<EventResult> {
    if !matches!(event, Event::MouseInput { pressed: true, .. }) {
        return None;
    }
    let p = event.position()?;
    let path = popup_scope(root);
    let scope = match &path {
        Some(path) => at_path_mut(root, path),
        None => root,
    };
    dispatch_popup_in(scope, p, event, msgs)
}

/// [`dispatch_popup_event`] 的递归体: 子节点倒序优先, 自身最后。
fn dispatch_popup_in(
    node: &mut Node,
    p: Point,
    event: &Event,
    msgs: &mut MsgQueue,
) -> Option<EventResult> {
    for child in node.children_mut().iter_mut().rev() {
        if let Some(result) = dispatch_popup_in(child, p, event, msgs) {
            return Some(result);
        }
    }
    let area = node.popup_area()?;
    if area.contains(p) {
        return Some(node.event(event, area, msgs));
    }
    None
}

/// 弹层外点收起。
///
/// 找到第一个「弹层展开且 `p` 落在其弹层区域与 [`Widget::hit_area`] 之外」的
/// 组件, 调用其 [`Widget::on_popup_dismiss`] 并返回 `true`。
///
/// 一次按下最多收起一处 —— 多实例互斥由此自然达成: 点 A 之外的任何位置,
/// 先命中的展开弹层即被收起, 随后本次事件照常继续分发
/// (点别处的按钮 = 先收起弹层 + 按钮生效)。
pub(crate) fn dismiss_popup_at(root: &mut Node, p: Point) -> bool {
    let path = popup_scope(root);
    let scope = match &path {
        Some(path) => at_path_mut(root, path),
        None => root,
    };
    dismiss_in(scope, p)
}

/// 弹层语义的作用域: 存在打开的模态屏障时, 收束到**最上层** (z 序最后) 的那个
/// 屏障的路径; 无屏障返回 `None` (作用域即整棵树)。
///
/// 见 [`Widget::modal_barrier`]: 没有它, 底层弹层会盖过 scrim 并抢走模态该吞掉
/// 的点击。
fn popup_scope(root: &Node) -> Option<Vec<usize>> {
    let mut prefix = Vec::new();
    find_barrier(root, &mut prefix)
}

/// [`popup_scope`] 的递归体: 兄弟倒序 (z 序靠上者先查), 子树先于自身 (嵌套模态
/// 取更靠内的那个)。
fn find_barrier(node: &Node, prefix: &mut Vec<usize>) -> Option<Vec<usize>> {
    for (i, child) in node.children().iter().enumerate().rev() {
        prefix.push(i);
        if let Some(found) = find_barrier(child, prefix) {
            return Some(found);
        }
        prefix.pop();
    }
    node.modal_barrier().then(|| prefix.clone())
}

/// 沿路径取节点 (不变借用)。
fn at_path<'a>(node: &'a Node, path: &[usize]) -> &'a Node {
    match path.split_first() {
        Some((&i, rest)) => at_path(&node.children()[i], rest),
        None => node,
    }
}

/// 沿路径取节点 (可变借用)。
///
/// 路径由同一棵树上的 [`popup_scope`] 取得, 两趟之间树不会变化, 故索引必然合法。
fn at_path_mut<'a>(node: &'a mut Node, path: &[usize]) -> &'a mut Node {
    match path.split_first() {
        Some((&i, rest)) => {
            let child = &mut node.children_mut()[i];
            at_path_mut(child, rest)
        }
        None => node,
    }
}

/// [`dismiss_popup_at`] 的递归体。
fn dismiss_in(node: &mut Node, p: Point) -> bool {
    if let Some(area) = node.popup_area()
        && !area.contains(p)
        && !node.hit_area().is_some_and(|h| h.contains(p))
    {
        node.on_popup_dismiss();
        return true;
    }
    for child in node.children_mut() {
        if dismiss_in(child, p) {
            return true;
        }
    }
    false
}

/// 沿路径向子组件分发事件 (用于焦点路由)。
///
/// `path` 为子索引序列; 返回处理结果。
pub fn event_at_path(
    root: &mut Node,
    path: &[usize],
    event: &Event,
    area: Rect,
    msgs: &mut MsgQueue,
) -> EventResult {
    if path.is_empty() {
        return root.event(event, area, msgs);
    }
    let (first, rest) = path.split_first().expect("path 非空");
    if let Some(child) = root.children_mut().get_mut(*first) {
        // 容器未缓存子区域时，使用父区域作为近似; 焦点路由通常到达叶子组件。
        event_at_path(child, rest, event, area, msgs)
    } else {
        EventResult::Ignored
    }
}

/// 沿路径取组件的选中文本。
pub fn selected_text_at_path(root: &Node, path: &[usize]) -> Option<String> {
    if path.is_empty() {
        return root.selected_text();
    }
    let (first, rest) = path.split_first().expect("path 非空");
    root.children()
        .get(*first)
        .and_then(|child| selected_text_at_path(child, rest))
}

/// 沿路径取组件的 IME 吸附区域。
pub fn ime_area_at_path(root: &Node, path: &[usize]) -> Option<Rect> {
    if path.is_empty() {
        return root.ime_area();
    }
    let (first, rest) = path.split_first().expect("path 非空");
    root.children()
        .get(*first)
        .and_then(|child| ime_area_at_path(child, rest))
}

/// 沿路径判断组件是否需要 IME。
pub fn wants_ime_at_path(root: &Node, path: &[usize]) -> bool {
    if path.is_empty() {
        return root.wants_ime();
    }
    let (first, rest) = path.split_first().expect("path 非空");
    root.children()
        .get(*first)
        .is_some_and(|child| wants_ime_at_path(child, rest))
}

/// 沿路径触发动画更新。
pub fn animate_at_path(root: &mut Node, path: &[usize], ctx: &AnimationCtx) {
    root.animate(ctx);
    if path.is_empty() {
        return;
    }
    let (first, rest) = path.split_first().expect("path 非空");
    if let Some(child) = root.children_mut().get_mut(*first) {
        animate_at_path(child, rest, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Key, MouseButton, NamedKey};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// 探针观测句柄 (probe 装箱后仍可读)。
    #[derive(Clone)]
    struct Handles {
        /// 收到的定位事件坐标。
        seen: Rc<RefCell<Vec<Point>>>,
        /// `paint_popup` 调用次数。
        paints: Rc<Cell<usize>>,
        /// `on_popup_dismiss` 调用次数。
        dismissals: Rc<Cell<usize>>,
    }

    /// 弹层探针: 声明弹层区域与 hit_area, 记录触达情况。
    struct Probe {
        popup: Option<Rect>,
        hit: Rect,
        /// 事件返回 `Consumed` (true, 默认) 还是 `Ignored`。
        consume: bool,
        handles: Handles,
    }

    impl Probe {
        fn new(popup: Option<Rect>, hit: Rect) -> (Self, Handles) {
            let handles = Handles {
                seen: Rc::new(RefCell::new(Vec::new())),
                paints: Rc::new(Cell::new(0)),
                dismissals: Rc::new(Cell::new(0)),
            };
            let probe = Self {
                popup,
                hit,
                consume: true,
                handles: handles.clone(),
            };
            (probe, handles)
        }

        /// 事件一律返回 `Ignored` 的探针 (锁「命中即拥有」语义)。
        fn ignoring(popup: Option<Rect>, hit: Rect) -> (Self, Handles) {
            let (mut probe, handles) = Self::new(popup, hit);
            probe.consume = false;
            (probe, handles)
        }
    }

    impl Widget for Probe {
        fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
            constraints.constrain(Size::ZERO)
        }

        fn paint(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {}

        fn event(&mut self, event: &Event, _area: Rect, _msgs: &mut MsgQueue) -> EventResult {
            if let Some(p) = event.position() {
                self.handles.seen.borrow_mut().push(p);
            }
            if self.consume {
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }

        fn popup_area(&self) -> Option<Rect> {
            self.popup
        }

        fn hit_area(&self) -> Option<Rect> {
            Some(self.hit)
        }

        fn paint_popup(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {
            self.handles.paints.set(self.handles.paints.get() + 1);
        }

        fn on_popup_dismiss(&mut self) {
            self.handles
                .dismissals
                .set(self.handles.dismissals.get() + 1);
        }
    }

    /// 极简容器: 仅承载子节点, 自身无弹层 (分发由被测函数负责)。
    struct Host {
        children: Vec<Node>,
    }

    impl Host {
        fn new(children: Vec<Node>) -> Self {
            Self { children }
        }
    }

    impl Widget for Host {
        fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
            constraints.constrain(Size::ZERO)
        }

        fn paint(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {}

        fn children(&self) -> &[Node] {
            &self.children
        }

        fn children_mut(&mut self) -> &mut [Node] {
            &mut self.children
        }
    }

    /// 控件矩形 (0,0)-(100,20) 与弹层矩形 (0,20)-(100,100) 的常用几何。
    fn control_rect() -> Rect {
        Rect::from_xywh(0.0, 0.0, 100.0, 20.0)
    }

    fn popup_rect() -> Rect {
        Rect::from_xywh(0.0, 20.0, 100.0, 80.0)
    }

    fn press(x: f32, y: f32) -> Event {
        Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(x, y),
        }
    }

    fn release(x: f32, y: f32) -> Event {
        Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position: Point::new(x, y),
        }
    }

    #[test]
    fn popup_dispatch_reaches_owner_outside_its_hit_area() {
        // 弹出区的意义: 按下点落在控件矩形之外, 常规命中分发够不着它
        // (各容器按 layout 矩形设门), 只能由弹层通道送达。
        let p = Point::new(50.0, 60.0);
        assert!(popup_rect().contains(p), "前提: 该点落在弹层区内");
        assert!(!control_rect().contains(p), "前提: 该点不在控件矩形内");

        let (owner, h) = Probe::new(Some(popup_rect()), control_rect());
        let mut root = node(Host::new(vec![node(owner)]));
        let mut msgs = MsgQueue::new();

        assert_eq!(
            dispatch_popup_event(&mut root, &press(50.0, 60.0), &mut msgs),
            Some(EventResult::Consumed),
            "弹层内按下应直达持有者"
        );
        assert_eq!(*h.seen.borrow(), vec![p], "持有者收到按下");
    }

    #[test]
    fn popup_dispatch_prefers_later_painted_owner() {
        // 两个弹层区域重叠时, 后绘制 (后声明) 者优先 —— 与 Flow 的
        // 「后绘制者 (z 序靠上) 优先命中」同序。
        let (first, first_h) = Probe::new(Some(popup_rect()), control_rect());
        let (second, second_h) = Probe::new(Some(popup_rect()), control_rect());
        let mut root = node(Host::new(vec![node(first), node(second)]));
        let mut msgs = MsgQueue::new();

        assert_eq!(
            dispatch_popup_event(&mut root, &press(50.0, 60.0), &mut msgs),
            Some(EventResult::Consumed)
        );
        assert_eq!(second_h.seen.borrow().len(), 1, "后绘制者优先");
        assert!(first_h.seen.borrow().is_empty(), "先绘制者让位");
    }

    #[test]
    fn popup_dispatch_only_handles_mouse_press() {
        // D2 锁: 移动类必须继续走全树广播 (否则其余组件 hover 残留清不掉);
        // 键盘走焦点路由; 抬起不参与弹层优先。
        let (owner, h) = Probe::new(Some(popup_rect()), control_rect());
        let mut root = node(owner);
        let mut msgs = MsgQueue::new();

        assert_eq!(
            dispatch_popup_event(
                &mut root,
                &Event::CursorMoved(Point::new(50.0, 60.0)),
                &mut msgs
            ),
            None,
            "移动类不得被弹层截走"
        );
        assert_eq!(
            dispatch_popup_event(&mut root, &release(50.0, 60.0), &mut msgs),
            None,
            "抬起不参与弹层优先"
        );
        let key = Event::Key {
            key: Key::Named(NamedKey::Escape),
            pressed: true,
            shift: false,
            ctrl: false,
            alt: false,
        };
        assert_eq!(
            dispatch_popup_event(&mut root, &key, &mut msgs),
            None,
            "键盘走焦点路由"
        );
        assert!(h.seen.borrow().is_empty(), "非按下事件一次都不得送达");
    }

    #[test]
    fn popup_dispatch_without_popup_returns_none() {
        // 零影响锁: 无弹层时必须把事件完好交回常规分发。
        let (plain, h) = Probe::new(None, control_rect());
        let mut root = node(plain);
        let mut msgs = MsgQueue::new();

        assert_eq!(
            dispatch_popup_event(&mut root, &press(50.0, 10.0), &mut msgs),
            None,
            "无弹层 = 不干预"
        );
        assert!(h.seen.borrow().is_empty(), "弹层通道不得抢走常规分发的事件");
    }

    #[test]
    fn popup_paint_opens_paired_layers_only_when_open() {
        let (owner, h) = Probe::new(Some(popup_rect()), control_rect());
        let root = node(Host::new(vec![node(owner)]));
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();

        paint_popups(&root, &mut rects, &mut texts);

        assert_eq!(h.paints.get(), 1, "有弹层则绘制一次");
        assert_eq!(
            rects.layer_spans().len(),
            texts.layer_spans().len(),
            "push_layer 必须配对 (漏压一侧静默错乱)"
        );
        assert_eq!(rects.layer_spans().len(), 2, "主层 + 弹层");
    }

    #[test]
    fn popup_paint_without_popup_is_noop() {
        // 零影响锁: 无弹层时不得多开任何层。
        let (plain, h) = Probe::new(None, control_rect());
        let root = node(plain);
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();

        paint_popups(&root, &mut rects, &mut texts);

        assert_eq!(h.paints.get(), 0, "无弹层不绘制");
        assert_eq!(rects.layer_spans().len(), 1, "无弹层零层增加 (矩形)");
        assert_eq!(texts.layer_spans().len(), 1, "无弹层零层增加 (文本)");
    }

    #[test]
    fn popup_dismiss_fires_only_outside_popup_and_hit_area() {
        let (owner, h) = Probe::new(Some(popup_rect()), control_rect());
        let mut root = node(owner);

        assert!(
            !dismiss_popup_at(&mut root, Point::new(50.0, 60.0)),
            "弹层内: 不收起"
        );
        assert!(
            !dismiss_popup_at(&mut root, Point::new(50.0, 10.0)),
            "控件内: 不收起 (交常规事件路径自行开合)"
        );
        assert!(
            dismiss_popup_at(&mut root, Point::new(500.0, 500.0)),
            "两者之外: 收起"
        );
        assert_eq!(h.dismissals.get(), 1, "只收起一次");
    }

    #[test]
    fn popup_dismiss_without_popup_returns_false() {
        // 零影响锁: 无弹层时点任何地方都不得触达组件。
        let (plain, h) = Probe::new(None, control_rect());
        let mut root = node(plain);

        assert!(!dismiss_popup_at(&mut root, Point::new(500.0, 500.0)));
        assert_eq!(h.dismissals.get(), 0, "无弹层不收起");
    }

    /// 屏障探针: 可开关的模态屏障, 承载子节点; 关态不暴露子树 (与 `Overlay`
    /// 的 `children()` 契约一致)。
    struct Barrier {
        open: bool,
        children: Vec<Node>,
    }

    impl Widget for Barrier {
        fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
            constraints.constrain(Size::ZERO)
        }

        fn paint(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {}

        fn modal_barrier(&self) -> bool {
            self.open
        }

        fn children(&self) -> &[Node] {
            if self.open { &self.children } else { &[] }
        }

        fn children_mut(&mut self) -> &mut [Node] {
            if self.open {
                &mut self.children
            } else {
                &mut []
            }
        }
    }

    fn barrier(open: bool, children: Vec<Node>) -> Node {
        node(Barrier { open, children })
    }

    #[test]
    fn modal_barrier_confines_popup_paint_to_its_own_subtree() {
        // 屏障打开时, 屏障外的弹层必须被冻结 —— 否则它会因绘制晚于 scrim
        // 而盖在模态遮罩之上 (模态被弹层击穿)。
        let (outside, outside_h) = Probe::new(Some(popup_rect()), control_rect());
        let (inside, inside_h) = Probe::new(Some(popup_rect()), control_rect());
        let root = node(Host::new(vec![
            node(outside),
            barrier(true, vec![node(inside)]),
        ]));
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();

        paint_popups(&root, &mut rects, &mut texts);

        assert_eq!(outside_h.paints.get(), 0, "屏障外的弹层不得绘制");
        assert_eq!(inside_h.paints.get(), 1, "屏障内的弹层照常绘制");
        assert_eq!(
            rects.layer_spans().len(),
            texts.layer_spans().len(),
            "push_layer 仍须配对"
        );
        assert_eq!(rects.layer_spans().len(), 2, "只有屏障内那一个弹层开了新层");
    }

    #[test]
    fn modal_barrier_confines_dispatch_and_dismiss() {
        let (outside, outside_h) = Probe::new(Some(popup_rect()), control_rect());
        let (inside, inside_h) = Probe::new(Some(popup_rect()), control_rect());
        let mut root = node(Host::new(vec![
            node(outside),
            barrier(true, vec![node(inside)]),
        ]));
        let mut msgs = MsgQueue::new();
        // 该点同时落在两个探针的弹层区内 —— 只有屏障内那个该收到。
        let p = Point::new(50.0, 60.0);

        assert_eq!(
            dispatch_popup_event(&mut root, &press(p.x, p.y), &mut msgs),
            Some(EventResult::Consumed),
            "屏障内仍有弹层可命中"
        );
        assert!(
            outside_h.seen.borrow().is_empty(),
            "屏障外的弹层不得收到事件"
        );
        assert_eq!(*inside_h.seen.borrow(), vec![p], "屏障内的弹层收到事件");

        // 点在两个弹层之外: 屏障内那个照常收起, 屏障外那个不得被触碰。
        dismiss_popup_at(&mut root, Point::new(900.0, 900.0));
        assert_eq!(outside_h.dismissals.get(), 0, "屏障外的弹层不得被收起");
        assert_eq!(inside_h.dismissals.get(), 1, "屏障内的弹层照常收起");
    }

    #[test]
    fn closed_barrier_creates_no_scope() {
        // 关态屏障既不是屏障也不暴露子树 —— 弹层语义回到整棵树, 零影响。
        let (outside, outside_h) = Probe::new(Some(popup_rect()), control_rect());
        let root = node(Host::new(vec![node(outside), barrier(false, vec![])]));
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();

        paint_popups(&root, &mut rects, &mut texts);

        assert_eq!(outside_h.paints.get(), 1, "关态屏障不产生作用域");
    }

    #[test]
    fn popup_dispatch_returns_none_for_press_outside_an_open_popup() {
        // 弹层**已展开**但按下落在弹层之外 → 必须交回常规分发。不能「展开即拦截
        // 整棵树」—— 那会让被盖住的兄弟既点不动、又不会收起弹层。
        let (owner, h) = Probe::new(Some(popup_rect()), control_rect());
        let mut root = node(owner);
        let mut msgs = MsgQueue::new();

        assert_eq!(
            dispatch_popup_event(&mut root, &press(500.0, 500.0), &mut msgs),
            None,
            "弹层外的按下不归弹层通道"
        );
        assert!(h.seen.borrow().is_empty(), "持有者不得收到弹层外的按下");
    }

    #[test]
    fn one_press_dismisses_at_most_one_popup() {
        // 两个弹层都开着时, 一次外部按下只收起先命中的那一个 —— 多实例互斥
        // 由「点外收起」自然达成 (这正是 D5 不引绑定式展开态的依据)。
        let (a, a_h) = Probe::new(
            Some(Rect::from_xywh(0.0, 20.0, 100.0, 80.0)),
            Rect::from_xywh(0.0, 0.0, 100.0, 20.0),
        );
        let (b, b_h) = Probe::new(
            Some(Rect::from_xywh(300.0, 20.0, 100.0, 80.0)),
            Rect::from_xywh(300.0, 0.0, 100.0, 20.0),
        );
        let mut root = node(Host::new(vec![node(a), node(b)]));

        assert!(dismiss_popup_at(&mut root, Point::new(500.0, 500.0)));
        assert_eq!(
            a_h.dismissals.get() + b_h.dismissals.get(),
            1,
            "一次按下只收起一处"
        );
    }

    #[test]
    fn popup_hit_owns_the_area_even_when_the_owner_ignores() {
        // 命中即拥有: 持有者对事件返回 `Ignored` 时, 通道给的是 `Some(Ignored)`
        // 而非 `None` —— 驱动据此跳过常规树内分发 (被盖住的兄弟同样收不到),
        // 事件只可能冒泡到 app 层。
        let (owner, h) = Probe::ignoring(Some(popup_rect()), control_rect());
        let mut root = node(owner);
        let mut msgs = MsgQueue::new();

        assert_eq!(
            dispatch_popup_event(&mut root, &press(50.0, 60.0), &mut msgs),
            Some(EventResult::Ignored),
            "命中即拥有: 应是 Some(Ignored) 而非 None"
        );
        assert_eq!(h.seen.borrow().len(), 1, "事件确实送达了持有者");
    }
}
