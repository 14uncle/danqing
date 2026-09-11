//! @author 十四叔
//! @date 2026/08/29
//!
//! DragArea 组件: 无边框窗口的拖拽移动层 (desk-window 验收 g 项缺口)。

use crate::event::{Event, MouseButton, WindowAction};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Rect, Size};

/// 拖拽层容器: 左键按下命中即发起窗口拖拽 (经 [`WindowAction::Drag`],
/// Handler 调 winit `drag_window` 系统模态移动)。
///
/// 无边框常驻窗 (桌景类) 没有标题栏, 本组件承担「背景拖动」语义:
/// 按下先给子组件, 子组件消费则不拖 —— 交互元素 (按钮/场景交互点)
/// 天然避让; 空背景按下才移动窗口。非按下事件 (移动/抬起/滚轮)
/// 原样透传子组件, 不产拖拽。
///
/// 布局逐轴判定 (同 [`Center`](crate::widget::Center)): tight 轴占满
/// (根用法下铺满窗口), loose 轴包裹子组件。
pub struct DragArea {
    child: Node,
    /// layout 缓存: 子组件尺寸 (子组件从左上角摆放)。
    child_size: Size,
}

impl DragArea {
    /// 创建拖拽层, 子组件摆放在区域左上角; 需要居中/边距时
    /// 在外层自行组合 Center/Padding。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: std::boxed::Box::new(child),
            child_size: Size::ZERO,
        }
    }
}

impl Widget for DragArea {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.child.sync(state);
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        self.child.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        // 逐轴判定 (同 Center): tight 轴占满 —— 根用法下窗口约束是 tight,
        // 拖拽层铺满窗口, 空白处处可拖; loose 轴包裹子组件 —— Scrollable
        // 给子组件的滚动轴上限是 MAX_CONTENT_SIZE (1e6), 占满会把内容区
        // 撑成百万像素空洞 (review Required #1 实证)。
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
        // 先给子组件: 交互子组件消费的按下不拖 (避让语义)。
        let child_area = Rect::new(area.origin, self.child_size);
        let result = self.child.event(event, child_area, msgs);
        if result == EventResult::Consumed {
            return result;
        }
        // 空背景左键按下且命中本区域 → 发起窗口拖拽。
        if let Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position,
        } = event
        {
            if area.contains(*position) {
                msgs.push(std::boxed::Box::new(WindowAction::Drag));
                return EventResult::Consumed;
            }
        }
        result
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

    /// 探针组件: 记录收到的事件, 按构造参数返回消费态。
    struct Spy {
        consume: bool,
        seen: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
    }

    impl Spy {
        fn new(consume: bool) -> Self {
            Self {
                consume,
                seen: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            }
        }

        /// 带外置记录器的构造: 测试经返回的 Rc 断言子组件实际收到的事件。
        fn tracked(consume: bool) -> (Self, std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>) {
            let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
            (
                Self {
                    consume,
                    seen: seen.clone(),
                },
                seen,
            )
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
            event: &Event,
            _area: Rect,
            _msgs: &mut crate::widget::MsgQueue,
        ) -> EventResult {
            let tag = match event {
                Event::MouseInput { pressed: true, .. } => "press",
                Event::MouseInput { pressed: false, .. } => "release",
                Event::CursorMoved(_) => "move",
                _ => "other",
            };
            self.seen.borrow_mut().push(tag);
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

    fn press_at(x: f32, y: f32) -> Event {
        Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(x, y),
        }
    }

    fn window_area() -> Rect {
        Rect::new(Point::new(0.0, 0.0), Size::new(320.0, 240.0))
    }

    #[test]
    fn layout_tight_constraints_fill() {
        // 根用法: 窗口约束 tight → 拖拽层铺满
        let mut area = DragArea::new(Spy::new(false));
        let mut texts = crate::render::TextBatch::new();
        let size = area.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        assert_eq!(size, Size::new(320.0, 240.0), "tight 轴占满");
    }

    #[test]
    fn layout_loose_constraints_wrap_child() {
        // 嵌套用法 (Scrollable 等): loose 轴包裹子组件, 不占百万像素上限
        let mut area = DragArea::new(Spy::new(false));
        let mut texts = crate::render::TextBatch::new();
        let size = area.layout(
            Constraints::loose(Size::new(320.0, 1_000_000.0)),
            &mut texts,
        );
        assert_eq!(
            size,
            Size::new(10.0, 10.0),
            "loose 轴包裹子组件 (spy 自报 10x10)"
        );
    }

    #[test]
    fn left_press_pushes_drag_action_and_consumes() {
        let mut area = DragArea::new(Spy::new(false));
        let mut texts = crate::render::TextBatch::new();
        area.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = crate::widget::MsgQueue::new();
        let result = area.event(&press_at(100.0, 100.0), window_area(), &mut msgs);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(msgs.len(), 1, "应产出一条拖拽消息");
        assert!(
            msgs[0].downcast_ref::<WindowAction>() == Some(&WindowAction::Drag),
            "消息应为 WindowAction::Drag"
        );
    }

    #[test]
    fn press_consumed_by_child_does_not_drag() {
        // 交互子组件吃掉按下 → 不触发拖拽 (场景世界交互元素的避让语义)
        let mut area = DragArea::new(Spy::new(true));
        let mut texts = crate::render::TextBatch::new();
        area.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = crate::widget::MsgQueue::new();
        let result = area.event(&press_at(5.0, 5.0), window_area(), &mut msgs);
        assert_eq!(result, EventResult::Consumed);
        assert!(msgs.is_empty(), "子组件消费的按下不产拖拽");
    }

    #[test]
    fn press_outside_area_ignored() {
        // 嵌套用法: 命中落在 DragArea 矩形之外 → 不拖拽不消费
        let mut area = DragArea::new(Spy::new(false));
        let mut texts = crate::render::TextBatch::new();
        area.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = crate::widget::MsgQueue::new();
        let small_area = Rect::new(Point::new(0.0, 0.0), Size::new(50.0, 50.0));
        let result = area.event(&press_at(200.0, 200.0), small_area, &mut msgs);
        assert_eq!(result, EventResult::Ignored);
        assert!(msgs.is_empty());
    }

    #[test]
    fn non_press_events_forward_to_child_without_drag() {
        let (spy, seen) = Spy::tracked(false);
        let mut area = DragArea::new(spy);
        let mut texts = crate::render::TextBatch::new();
        area.layout(Constraints::tight(Size::new(320.0, 240.0)), &mut texts);
        let mut msgs = crate::widget::MsgQueue::new();
        let moved = Event::CursorMoved(Point::new(30.0, 30.0));
        let result = area.event(&moved, window_area(), &mut msgs);
        assert_eq!(result, EventResult::Ignored, "非按下事件不消费");
        assert!(msgs.is_empty(), "非按下事件不产拖拽");
        assert_eq!(*seen.borrow(), vec!["move"], "事件实际透传到子组件");
    }
}
