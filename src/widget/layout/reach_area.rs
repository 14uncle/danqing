//! @author 十四叔
//! @date 2026/08/30
//!
//! 伸手仲裁层: 无边框常驻窗的「刻意微互动」手势空间半区 (口味题 #2:
//! 默认穿透 + 刻意「伸手」= 悬停长按)。
//!
//! 仲裁分两半, 各就其位:
//! - **空间仲裁在本层**: 按下登记 → 移动超阈值转拖拽 (转发
//!   [`WindowAction::Drag`], 系统模态移动此时才进入, 不与长按计时冲突)
//!   → 提前抬起/转拖拽都发撤防。
//! - **时间仲裁在产品 tick**: 600ms 长按判定需要时钟; widget 事件流里
//!   没有周期心跳 (animate 无 msg 通道), 产品 tick 有时钟有状态 ——
//!   产品收到「按下登记」后自行计时, 到点触发微互动。
//!
//! 消息协议: on_arm(按下点) 立即发; on_cancel() 在手势终结 (转拖拽/
//! 抬起) 时发 —— 产品是幂等消费方 (已触发则撤防无操作)。
//! 子组件消费按下则本层静默 (DragArea 同款避让: 交互元素天然优先)。

use std::any::Any;
use std::rc::Rc;

use crate::event::{Event, MouseButton, WindowAction};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Point, Rect, Size};

/// 拖拽位移阈值 (逻辑像素): 按住微抖不转拖拽, 明显拖动才接管。
const DRAG_THRESHOLD: f32 = 8.0;

/// 按下登记消息工厂 (带按下点载荷)。
type ArmFactory = Box<dyn Fn(Point) -> Box<dyn Any>>;
/// 撤防消息工厂。
type CancelFactory = Box<dyn Fn() -> Box<dyn Any>>;

/// 伸手仲裁容器: 子组件优先消费, 空白处按下走伸手手势。
pub struct ReachArea {
    child: Node,
    /// layout 缓存: 子组件尺寸 (子组件从左上角摆放)。
    child_size: Size,
    on_arm: Option<Rc<ArmFactory>>,
    on_cancel: Option<Rc<CancelFactory>>,
    /// 进行中的手势按下点 (None = 无手势)。
    pending: Option<Point>,
}

impl ReachArea {
    /// 创建仲裁层, 子组件摆放在区域左上角; 需要居中/边距时
    /// 在外层自行组合 Center/Padding。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: std::boxed::Box::new(child),
            child_size: Size::ZERO,
            on_arm: None,
            on_cancel: None,
            pending: None,
        }
    }

    /// 按下登记消息 (立即发, 产品开始计时长按)。
    pub fn on_arm<M: 'static>(mut self, f: impl Fn(Point) -> M + 'static) -> Self {
        self.on_arm = Some(Rc::new(Box::new(move |p| {
            std::boxed::Box::new(f(p)) as Box<dyn Any>
        })));
        self
    }

    /// 撤防消息 (转拖拽/抬起时发, 产品幂等消费)。
    pub fn on_cancel<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_cancel = Some(Rc::new(Box::new(move || {
            std::boxed::Box::new(f()) as Box<dyn Any>
        })));
        self
    }
}

impl Widget for ReachArea {
    fn sync(&mut self, state: &dyn Any) {
        self.child.sync(state);
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        self.child.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        // 逐轴判定 (同 DragArea/Center): tight 轴占满, loose 轴包裹子组件
        // (Scrollable 百万像素上限教训见 drag_area.rs)。
        let tight_w = constraints.min_width == constraints.max_width;
        let tight_h = constraints.min_height == constraints.max_height;
        self.child_size = self
            .child
            .layout(Constraints::loose(constraints.max()), texts);
        let size = Size::new(
            if tight_w {
                constraints.max_width
            } else {
                self.child_size.width
            },
            if tight_h {
                constraints.max_height
            } else {
                self.child_size.height
            },
        );
        constraints.constrain(size)
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        self.child
            .paint(Rect::new(area.origin, self.child_size), rects, texts);
    }

    fn paint_image(&self, area: Rect, images: &mut crate::render::ImageBatch) {
        self.child
            .paint_image(Rect::new(area.origin, self.child_size), images);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        // 先给子组件: 交互子组件消费的按下不进仲裁 (避让语义)。
        let child_area = Rect::new(area.origin, self.child_size);
        let result = self.child.event(event, child_area, msgs);
        if result == EventResult::Consumed {
            return result;
        }
        match event {
            // 左键按下且命中 → 登记手势 (空间仲裁起点)。
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position,
            } if area.contains(*position) => {
                self.pending = Some(*position);
                if let Some(f) = &self.on_arm {
                    msgs.push(f(*position));
                }
                EventResult::Consumed
            }
            // 移动超阈值 → 转拖拽 (系统模态移动此刻才进入; 长按计时让位)。
            Event::CursorMoved(position) => {
                let Some(start) = self.pending else {
                    return result;
                };
                let dx = position.x - start.x;
                let dy = position.y - start.y;
                if dx * dx + dy * dy > DRAG_THRESHOLD * DRAG_THRESHOLD {
                    self.pending = None;
                    if let Some(f) = &self.on_cancel {
                        msgs.push(f());
                    }
                    msgs.push(std::boxed::Box::new(WindowAction::Drag));
                }
                EventResult::Consumed
            }
            // 抬起 → 手势终结, 撤防 (未到长按 = 单击不触发, 误触防护;
            // 已触发 = 产品幂等撤防)。
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                ..
            } if self.pending.is_some() => {
                self.pending = None;
                if let Some(f) = &self.on_cancel {
                    msgs.push(f());
                }
                EventResult::Consumed
            }
            _ => result,
        }
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, MouseButton, WindowAction};
    use crate::widget::{EventResult, Node, Widget};
    use crate::{Constraints, Point, Rect, Size};

    /// 探针组件 (DragArea 测试同款): 记录事件, 按构造参数返回消费态。
    struct Spy {
        consume: bool,
    }

    impl Spy {
        fn new(consume: bool) -> Self {
            Self { consume }
        }
    }

    impl Widget for Spy {
        fn layout(&mut self, _c: Constraints, _t: &mut crate::render::TextBatch) -> Size {
            Size::new(10.0, 10.0)
        }

        fn paint(
            &self,
            _a: Rect,
            _r: &mut crate::render::RectBatch,
            _t: &mut crate::render::TextBatch,
        ) {
        }

        fn event(
            &mut self,
            _event: &Event,
            _area: Rect,
            _msgs: &mut crate::widget::MsgQueue,
        ) -> EventResult {
            if self.consume {
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }

        fn children(&self) -> &[Node] {
            &[]
        }

        fn children_mut(&mut self) -> &mut [Node] {
            &mut []
        }
    }

    /// 测试消息 (arm/cancel 探针)。
    #[derive(Debug, PartialEq)]
    enum TestMsg {
        Armed(Point),
        Cancelled,
    }

    fn area_and_layer() -> (ReachArea, Rect) {
        let layer = ReachArea::new(Spy::new(false))
            .on_arm(TestMsg::Armed)
            .on_cancel(|| TestMsg::Cancelled);
        let area = Rect::new(Point::ZERO, Size::new(320.0, 240.0));
        (layer, area)
    }

    fn press_at(x: f32, y: f32) -> Event {
        Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(x, y),
        }
    }

    fn release() -> Event {
        Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position: Point::ZERO,
        }
    }

    fn move_to(x: f32, y: f32) -> Event {
        Event::CursorMoved(Point::new(x, y))
    }

    /// 按下 → 立即登记 (arm 消息带按下点) + 消费。
    #[test]
    fn press_arms_with_position() {
        let (mut layer, area) = area_and_layer();
        let mut texts = crate::render::TextBatch::new();
        layer.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = MsgQueue::new();
        let r = layer.event(&press_at(100.0, 100.0), area, &mut msgs);
        assert_eq!(r, EventResult::Consumed);
        assert_eq!(msgs.len(), 1, "按下应产 arm 消息");
        assert_eq!(
            msgs[0].downcast_ref::<TestMsg>(),
            Some(&TestMsg::Armed(Point::new(100.0, 100.0))),
            "arm 带按下点"
        );
    }

    /// 按住微抖 (<8px): 不转拖拽不撤防, 手势保持。
    #[test]
    fn micro_jitter_stays_armed() {
        let (mut layer, area) = area_and_layer();
        let mut texts = crate::render::TextBatch::new();
        layer.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = MsgQueue::new();
        layer.event(&press_at(100.0, 100.0), area, &mut msgs);
        msgs.clear();
        let r = layer.event(&move_to(105.0, 103.0), area, &mut msgs);
        assert_eq!(r, EventResult::Consumed);
        assert!(msgs.is_empty(), "微抖不产消息");
    }

    /// 明显拖动 (>8px) → 撤防 + 转发拖拽 (模态移动此刻才进)。
    #[test]
    fn big_move_cancels_and_drags() {
        let (mut layer, area) = area_and_layer();
        let mut texts = crate::render::TextBatch::new();
        layer.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = MsgQueue::new();
        layer.event(&press_at(100.0, 100.0), area, &mut msgs);
        msgs.clear();
        let r = layer.event(&move_to(140.0, 100.0), area, &mut msgs);
        assert_eq!(r, EventResult::Consumed);
        assert_eq!(msgs.len(), 2, "撤防 + 拖拽两条");
        assert_eq!(msgs[0].downcast_ref::<TestMsg>(), Some(&TestMsg::Cancelled));
        assert!(
            msgs[1].downcast_ref::<WindowAction>() == Some(&WindowAction::Drag),
            "第二条应为拖拽"
        );
        // 拖拽后手势终结: 再抬起不重复撤防。
        msgs.clear();
        layer.event(&release(), area, &mut msgs);
        assert!(msgs.is_empty(), "拖拽后抬起静默");
    }

    /// 抬起 (未到长按) → 撤防, 无拖拽无触发 (单击不触发 = 误触防护)。
    #[test]
    fn early_release_cancels_without_drag() {
        let (mut layer, area) = area_and_layer();
        let mut texts = crate::render::TextBatch::new();
        layer.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = MsgQueue::new();
        layer.event(&press_at(50.0, 50.0), area, &mut msgs);
        msgs.clear();
        let r = layer.event(&release(), area, &mut msgs);
        assert_eq!(r, EventResult::Consumed);
        assert_eq!(msgs.len(), 1, "只产撤防");
        assert_eq!(msgs[0].downcast_ref::<TestMsg>(), Some(&TestMsg::Cancelled));
    }

    /// 子组件消费按下 → 本层静默 (交互元素避让, 不登记不拖拽)。
    #[test]
    fn child_consumed_press_bypasses_arbitration() {
        let mut layer = ReachArea::new(Spy::new(true))
            .on_arm(TestMsg::Armed)
            .on_cancel(|| TestMsg::Cancelled);
        let area = Rect::new(Point::ZERO, Size::new(320.0, 240.0));
        let mut texts = crate::render::TextBatch::new();
        layer.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = MsgQueue::new();
        let r = layer.event(&press_at(5.0, 5.0), area, &mut msgs);
        assert_eq!(r, EventResult::Consumed, "子组件消费原样透传");
        assert!(msgs.is_empty(), "不进仲裁");
        // 后续移动/抬起: 无 pending, 静默。
        layer.event(&move_to(200.0, 200.0), area, &mut msgs);
        assert!(msgs.is_empty());
    }

    /// 区域外按下: 不登记不消费 (嵌套用法)。
    #[test]
    fn press_outside_area_ignored() {
        let (mut layer, _) = area_and_layer();
        let mut texts = crate::render::TextBatch::new();
        layer.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let small = Rect::new(Point::ZERO, Size::new(50.0, 50.0));
        let mut msgs = MsgQueue::new();
        let r = layer.event(&press_at(200.0, 200.0), small, &mut msgs);
        assert_eq!(r, EventResult::Ignored);
        assert!(msgs.is_empty());
    }

    /// 布局: tight 轴占满 / loose 轴包裹 (DragArea 同款, 防 1M 回归)。
    #[test]
    fn layout_tight_fill_loose_wrap() {
        let mut layer = ReachArea::new(Spy::new(false));
        let mut texts = crate::render::TextBatch::new();
        let size = layer.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        assert_eq!(size, Size::new(320.0, 240.0), "tight 占满");
        let mut layer = ReachArea::new(Spy::new(false));
        let size = layer.layout(
            Constraints::loose(Size::new(320.0, 1_000_000.0)),
            &mut texts,
        );
        assert_eq!(size, Size::new(10.0, 10.0), "loose 包裹子组件");
    }
}
