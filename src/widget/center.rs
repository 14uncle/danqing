//! @author 十四叔
//! @date 2026/07/17

//! Center 组件:把子组件居中摆放。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Point, Rect, Size};

/// 居中容器:自身占满约束上限,子组件按内容尺寸居中。
pub struct Center {
    child: Node,
    /// layout 缓存:子组件尺寸。
    child_size: Size,
}

impl Center {
    /// 创建居中容器。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: std::boxed::Box::new(child),
            child_size: Size::ZERO,
        }
    }
}

impl Widget for Center {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.child.sync(state);
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        self.child.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.child_size = self
            .child
            .layout(Constraints::loose(constraints.max()), texts);
        constraints.constrain(constraints.max())
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let origin = Point::new(
            area.origin.x + (area.size.width - self.child_size.width) / 2.0,
            area.origin.y + (area.size.height - self.child_size.height) / 2.0,
        );
        self.child
            .paint(Rect::new(origin, self.child_size), rects, texts);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let origin = Point::new(
            area.origin.x + (area.size.width - self.child_size.width) / 2.0,
            area.origin.y + (area.size.height - self.child_size.height) / 2.0,
        );
        self.child
            .event(event, Rect::new(origin, self.child_size), msgs)
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }
}
