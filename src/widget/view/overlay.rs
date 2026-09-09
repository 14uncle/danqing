//! @author 十四叔
//! @date 2026/09/09

//! 模态浮层 (Overlay): open 态绑定的 scrim 遮罩 + 居中内容卡。
//!
//! 做什么 (六语义, 逐条有单测):
//! - open 态绑定: sync/animate/layout/paint/event/children 全线门控,
//!   关态子树完全休眠 (焦点遍历与命中分发经空 children 跳过; hit_area
//!   不覆写, 恒为 trait 默认 None —— 容器进焦点命中会遮蔽卡内组件, 见 C2 教训)。
//! - 关态零尺寸不拦事件: layout 返回 `(max.width, 0)` (勿置 Row 内, 关态仍占全宽)。
//! - 开态 paint 自开渲染新层: 矩形/文本分批次的渲染架构下同层文本恒在矩形之上,
//!   卡片须开新层才能盖住底层文本 (「关于」卡看不清的根因教训内化)。
//! - 模态吞事件: 开态定位事件一律 Consumed —— 卡内透传内容但结果不冒泡
//!   (内容 Ignored ≠ 放行底层), 卡外吞掉; 左键点遮罩且有 `on_scrim_click`
//!   时额外发关闭消息 (opt-in)。键盘等非定位事件透传且结果原样返回
//!   (Esc→清焦等协议依赖 Ignored 传播)。遮罩上的 CursorMoved 透传内容
//!   清 hover 残留 (hover 卫生)。
//! - 开→关边沿对内容子树调 `reset_focus` (关态后 FocusOut 送不进空 children;
//!   MultiPanel 判例: 藏子树的容器负责清焦点视觉)。
//!
//! 容器自身恒不可聚焦 (Tabs 范式: FocusManager collect 无条件递归子节点,
//! 容器无需自证可聚焦; 焦点直达卡内组件)。
//!
//! 不做什么: Esc 关闭不进组件 (各产品 app 级自理且语义不同 —— pomodoro 关面板
//! 顺带焦点回归; log 须先清输入焦点再关层, 见其 main.rs S3 回归注释; 组件统一
//! 接管会踩焦点优先级差异); 焦点回归编排不进组件 (用既有 `App::focus_request`/`focus_restored` 协议, 产品侧两行);
//! 玻璃卡片样式不进组件 (内容槽纯注入, 产品用 UiBox+Padding 自建); 无开合动画
//! (v1; 未来可叠 `anim` 原语)。
//!
//! 已知限制: 底层页面的 Image 恒穿 scrim (ImageBatch 无分层, 图像 pass 在
//! 矩形/文本之后; 反向看, 卡内 Image 因此天然在 scrim 之上, 无需分层)。
//!
//! 为什么: 模态浮层在农场被手写 6 份 (pomodoro 三面板 + clipboard 两处 +
//! log SettingsOverlay), 2026-09-09 下沉盘点 (docs/intent/framework-sinking.md
//! 簇C) 裁定收编, spec: docs/specs/SPEC-overlay.md。
//! C1/C2 教训 (下沉首评揪出): 卡内未消费事件不得冒泡 (log Scrim 全窗兜底语义
//! 不可丢); 容器不得覆写 hit_area 抢焦 (log 源头没有, 下沉时勿新增)。

use std::any::Any;
use std::cell::Cell;

use crate::app::AnimationCtx;
use crate::event::{Event, MouseButton};
use crate::render::{ImageBatch, RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget, node};
use crate::{Constraints, LightTheme, Rect, Size, Theme};

/// open 态绑定闭包: 从类型擦除的应用状态读取浮层开关。
type OpenBinding = Box<dyn Fn(&dyn Any) -> bool>;
/// 消息工厂: 点遮罩时产出一条应用消息。
type MsgFactory = Box<dyn Fn() -> Box<dyn Any>>;

/// 模态浮层: open 态绑定的 scrim 遮罩 + 居中内容卡。
///
/// 用法: 作为页面树的最后一个孩子 (盖顶) 常驻实例化, `bind_open` 绑定开关态;
/// 内容卡由产品注入 (UiBox+Padding 玻璃样式自理)。Esc 关闭与焦点回归见模块头。
pub struct Overlay {
    /// open 态 (sync 从绑定同步; 无绑定恒 false)。
    open: bool,
    /// open 态绑定。
    open_binding: Option<OpenBinding>,
    /// 内容卡 (产品注入的子树)。
    content: Node,
    /// 点遮罩关闭消息工厂 (opt-in; None = 遮罩点击仅消费不发消息)。
    on_scrim_click: Option<MsgFactory>,
    /// scrim 遮罩色 (主题 token, 构造时取定)。
    scrim_color: crate::Color,
    /// 内容卡布局尺寸缓存 (paint/event 居中计算)。
    content_size: Cell<Size>,
}

impl Overlay {
    /// 创建浮层, 使用默认浅色主题 token。
    pub fn new(content: impl Widget + 'static) -> Self {
        Self::themed(&LightTheme, content)
    }

    /// 使用指定主题创建浮层。
    pub fn themed(theme: &impl Theme, content: impl Widget + 'static) -> Self {
        Self {
            open: false,
            open_binding: None,
            content: node(content),
            on_scrim_click: None,
            scrim_color: theme.scrim(),
            content_size: Cell::new(Size::ZERO),
        }
    }

    /// 绑定 open 态: 每帧 sync 从应用状态读取浮层开关。
    pub fn bind_open<S: 'static>(mut self, f: impl Fn(&S) -> bool + 'static) -> Self {
        self.open_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("Overlay open 绑定的状态类型不匹配"))
        }));
        self
    }

    /// 点遮罩关闭消息 (opt-in): 开态左键点遮罩时产出; 不设则遮罩点击仅消费。
    pub fn on_scrim_click<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_scrim_click = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 内容卡居中矩形 (layout 缓存的尺寸 + 本组件矩形)。
    fn card_rect(&self, area: Rect) -> Rect {
        let size = self.content_size.get();
        Rect::from_xywh(
            area.origin.x + (area.size.width - size.width) / 2.0,
            area.origin.y + (area.size.height - size.height) / 2.0,
            size.width,
            size.height,
        )
    }
}

impl Widget for Overlay {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(bind) = &self.open_binding {
            let next = bind(state);
            // 开→关边沿: 清内容子树残留的焦点/按压视觉 (关态后 FocusOut
            // 送不进空 children; MultiPanel 判例: 藏子树的容器负责清焦点视觉)。
            if self.open && !next {
                self.content.reset_focus();
            }
            self.open = next;
        }
        // 关态子树完全休眠 (内容绑定不触达)。
        if self.open {
            self.content.sync(state);
        }
    }

    fn animate(&mut self, ctx: &AnimationCtx) {
        if self.open {
            self.content.animate(ctx);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        if !self.open {
            // 关态: 零高度不占空间, 不扰底层布局 (log settings.rs 实证形态)。
            return Size::new(constraints.max().width, 0.0);
        }
        // 开态占满父约束 (浮层须置于覆盖全窗的位置); 内容卡按自身内容定尺寸。
        let size = self
            .content
            .layout(Constraints::loose(constraints.max()), texts);
        self.content_size.set(size);
        constraints.max()
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        if !self.open {
            return;
        }
        let area = area.snap_to_pixels();
        // 开新渲染层: 同层文本恒在矩形之上, 卡片须开新层才能盖底层文本
        // (「关于」卡看不清的根因教训内化)。
        rects.push_layer();
        texts.push_layer();
        // scrim 遮罩直接画矩形 (不做子组件; 事件由 Overlay 自体按模态语义消费)。
        rects.push_rect(area, self.scrim_color, 0.0);
        let card = self.card_rect(area);
        self.content.paint(card, rects, texts);
    }

    /// 开态转发内容卡的图像收集: 图像 pass 在所有矩形/文本层之后,
    /// 卡内图像天然在 scrim 之上, 无需分层 (反向限制见模块头)。
    fn paint_image(&self, area: Rect, images: &mut ImageBatch) {
        if self.open {
            self.content.paint_image(self.card_rect(area), images);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        if !self.open {
            return EventResult::Ignored;
        }
        match event.position() {
            Some(p) => {
                let card = self.card_rect(area);
                if card.contains(p) {
                    // 卡内: 透传内容子树, 但结果一律 Consumed —— 模态不得冒泡到
                    // 被盖住的底层 (内容 Ignored 只是内容不收, 不是放行)。
                    let _ = self.content.event(event, card, msgs);
                } else {
                    // 卡外: 移动透传内容清 hover 残留 (hover 卫生), 其余吞掉;
                    // 左键按下且有工厂时发关闭消息 (opt-in)。
                    if matches!(event, Event::CursorMoved(_)) {
                        let _ = self.content.event(event, card, msgs);
                    }
                    if let Some(factory) = &self.on_scrim_click
                        && let Event::MouseInput {
                            button: MouseButton::Left,
                            pressed: true,
                            ..
                        } = event
                    {
                        msgs.push(factory());
                    }
                }
                EventResult::Consumed
            }
            // 非定位事件 (键盘/FocusIn/Out/CursorLeft): 透传且结果原样返回
            // (Esc→清焦等协议依赖 Ignored 传播)。
            None => self.content.event(event, self.card_rect(area), msgs),
        }
    }

    fn children(&self) -> &[Node] {
        if self.open {
            std::slice::from_ref(&self.content)
        } else {
            &[]
        }
    }

    fn children_mut(&mut self) -> &mut [Node] {
        if self.open {
            std::slice::from_mut(&mut self.content)
        } else {
            &mut []
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Key, NamedKey};
    use crate::widget::MsgQueue;
    use crate::{Point, Size};
    use std::cell::RefCell;
    use std::rc::Rc;

    /// 演示状态: 只带一个 open 开关。
    struct Demo {
        open: bool,
    }

    /// 探针计数器组。
    #[derive(Default)]
    struct ProbeCounters {
        syncs: usize,
        events: usize,
        resets: usize,
        images: usize,
    }

    /// 探针组件: 计数 sync/event/reset_focus/paint_image 触达; 布局返回固定尺寸。
    struct Probe {
        counters: Rc<RefCell<ProbeCounters>>,
        /// true = 事件一律 Consumed; false = 一律 Ignored (卡内不收场景回归用)。
        consume: bool,
    }

    impl Probe {
        fn new(consume: bool) -> (Rc<RefCell<ProbeCounters>>, Self) {
            let counters = Rc::new(RefCell::new(ProbeCounters::default()));
            (Rc::clone(&counters), Self { counters, consume })
        }
    }

    impl Widget for Probe {
        fn sync(&mut self, _state: &dyn Any) {
            self.counters.borrow_mut().syncs += 1;
        }

        fn layout(&mut self, _constraints: Constraints, _texts: &mut TextBatch) -> Size {
            Size::new(120.0, 80.0)
        }

        fn paint(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {}

        fn paint_image(&self, _area: Rect, _images: &mut ImageBatch) {
            self.counters.borrow_mut().images += 1;
        }

        fn event(&mut self, _event: &Event, _area: Rect, _msgs: &mut MsgQueue) -> EventResult {
            self.counters.borrow_mut().events += 1;
            if self.consume {
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }

        fn reset_focus(&mut self) {
            self.counters.borrow_mut().resets += 1;
        }
    }

    fn loose_800x600() -> Constraints {
        Constraints::loose(Size::new(800.0, 600.0))
    }

    fn left_click(x: f32, y: f32) -> Event {
        Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(x, y),
        }
    }

    /// 开态下 800×600 区域内的 120×80 内容卡居中于 (340,260)。
    fn card_center_click() -> Event {
        left_click(400.0, 300.0)
    }

    fn window_area() -> Rect {
        Rect::from_xywh(0.0, 0.0, 800.0, 600.0)
    }

    #[test]
    fn closed_layout_zero_height_and_content_sleeps() {
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: false });
        let size = ov.layout(loose_800x600(), &mut TextBatch::new());
        assert_eq!(size, Size::new(800.0, 0.0), "关态零高度 (宽度保留)");
        assert_eq!(c.borrow().syncs, 0, "关态 sync 不触达内容");
    }

    #[test]
    fn closed_is_invisible_to_events_and_focus() {
        let (_c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: false });
        let mut msgs = MsgQueue::new();
        assert_eq!(
            ov.event(&card_center_click(), window_area(), &mut msgs),
            EventResult::Ignored,
            "关态事件 Ignored"
        );
        assert!(!ov.focusable(), "容器恒不可聚焦 (Tabs 范式)");
        assert!(ov.children().is_empty(), "关态无子节点");
        assert!(ov.hit_area().is_none(), "hit_area 不覆写 (trait 默认 None)");
    }

    #[test]
    fn open_layout_fills_and_wakes_content() {
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: true });
        let size = ov.layout(loose_800x600(), &mut TextBatch::new());
        assert_eq!(size, Size::new(800.0, 600.0), "开态占满约束");
        assert_eq!(c.borrow().syncs, 1, "开态 sync 透达内容");
        assert_eq!(ov.children().len(), 1, "开态子节点 = 内容卡");
        assert!(
            !ov.focusable(),
            "容器恒不可聚焦: FocusManager collect 递归直达卡内组件 (Tabs 范式)"
        );
        assert!(
            ov.hit_area().is_none(),
            "hit_area 不覆写: 模态拦截由 event 门控承载 (C2 教训)"
        );
    }

    #[test]
    fn open_event_inside_card_reaches_content() {
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut msgs = MsgQueue::new();
        let result = ov.event(&card_center_click(), window_area(), &mut msgs);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(c.borrow().events, 1, "卡片内点击透传内容子树");
    }

    #[test]
    fn inside_card_ignored_pointer_still_consumed() {
        // C1 回归: 内容不收 (Ignored) 的卡内点击不得冒泡到被盖住的底层。
        let (c, probe) = Probe::new(false);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut msgs = MsgQueue::new();
        let result = ov.event(&card_center_click(), window_area(), &mut msgs);
        assert_eq!(c.borrow().events, 1, "卡内点击照常透传内容");
        assert_eq!(result, EventResult::Consumed, "内容不收 ≠ 放行底层 (模态)");
        assert!(msgs.is_empty(), "卡内点击不触发遮罩消息");
    }

    #[test]
    fn scrim_click_with_factory_emits_and_consumes() {
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe)
            .bind_open(|s: &Demo| s.open)
            .on_scrim_click(|| 42u8);
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut msgs = MsgQueue::new();
        let result = ov.event(&left_click(10.0, 10.0), window_area(), &mut msgs);
        assert_eq!(result, EventResult::Consumed, "遮罩点击消费");
        assert_eq!(msgs.len(), 1, "点遮罩产出关闭消息");
        assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&42));
        assert_eq!(c.borrow().events, 0, "遮罩左键不透传内容");
    }

    #[test]
    fn scrim_click_without_factory_consumes_silently() {
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut msgs = MsgQueue::new();
        let result = ov.event(&left_click(10.0, 10.0), window_area(), &mut msgs);
        assert_eq!(result, EventResult::Consumed, "无工厂也消费 (模态吞事件)");
        assert!(msgs.is_empty(), "无工厂不发消息");
        assert_eq!(c.borrow().events, 0);
    }

    #[test]
    fn cursor_moved_on_scrim_forwards_but_wheel_swallowed() {
        // hover 卫生: 遮罩上的移动透传内容 (清卡内残留 hover); 滚轮不透传
        // (卡外滚轮不得滚动卡内容) 但仍消费 (不达底层)。
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut msgs = MsgQueue::new();
        let moved = Event::CursorMoved(Point::new(10.0, 10.0));
        assert_eq!(
            ov.event(&moved, window_area(), &mut msgs),
            EventResult::Consumed
        );
        assert_eq!(c.borrow().events, 1, "遮罩上移动透传内容 (hover 卫生)");
        let wheel = Event::MouseWheel {
            delta: (0.0, 1.0),
            position: Point::new(10.0, 10.0),
            shift: false,
            ctrl: false,
            alt: false,
        };
        assert_eq!(
            ov.event(&wheel, window_area(), &mut msgs),
            EventResult::Consumed
        );
        assert_eq!(c.borrow().events, 1, "遮罩上滚轮不透传内容");
    }

    #[test]
    fn key_events_forward_with_result_passthrough() {
        // 非定位事件透传且结果原样返回 (Esc→清焦协议依赖 Ignored 传播):
        // 内容 Ignored 时 Overlay 也 Ignored (可冒泡到 app 级 Esc 处理)。
        let (c, probe) = Probe::new(false);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut msgs = MsgQueue::new();
        let key = Event::Key {
            key: Key::Named(NamedKey::Escape),
            pressed: true,
            shift: false,
            ctrl: false,
            alt: false,
        };
        assert_eq!(
            ov.event(&key, window_area(), &mut msgs),
            EventResult::Ignored
        );
        assert_eq!(c.borrow().events, 1, "键盘事件透传内容子树");
    }

    #[test]
    fn open_paint_pushes_render_layers() {
        // push_layer 教训内化: 开态 paint 后矩形/文本各开一新层; 关态不开。
        let (_c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: false });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        ov.paint(window_area(), &mut rects, &mut texts);
        assert_eq!(rects.layer_spans().len(), 1, "关态不开层 (矩形)");
        assert_eq!(texts.layer_spans().len(), 1, "关态不开层 (文本)");
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        ov.paint(window_area(), &mut rects, &mut texts);
        assert_eq!(rects.layer_spans().len(), 2, "开态矩形批 +1 层");
        assert_eq!(texts.layer_spans().len(), 2, "开态文本批 +1 层");
    }

    #[test]
    fn close_edge_resets_content_focus() {
        // R1: 开→关边沿清内容焦点视觉 (MultiPanel 判例); 电平不重复触发。
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: true });
        assert_eq!(c.borrow().resets, 0);
        ov.sync(&Demo { open: false });
        assert_eq!(c.borrow().resets, 1, "开→关边沿清一次");
        ov.sync(&Demo { open: false });
        assert_eq!(c.borrow().resets, 1, "持续关态不重复清");
        ov.sync(&Demo { open: true });
        ov.sync(&Demo { open: false });
        assert_eq!(c.borrow().resets, 2, "每个关层边沿各清一次");
    }

    #[test]
    fn paint_image_forwarded_only_when_open() {
        // R2: 开态转发图像收集 (卡内 Image 可渲染); 关态不转发。
        let (c, probe) = Probe::new(true);
        let mut ov = Overlay::new(probe).bind_open(|s: &Demo| s.open);
        ov.sync(&Demo { open: false });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        ov.paint_image(window_area(), &mut ImageBatch::new());
        assert_eq!(c.borrow().images, 0, "关态不转发图像收集");
        ov.sync(&Demo { open: true });
        ov.layout(loose_800x600(), &mut TextBatch::new());
        ov.paint_image(window_area(), &mut ImageBatch::new());
        assert_eq!(c.borrow().images, 1, "开态转发图像收集");
    }
}
