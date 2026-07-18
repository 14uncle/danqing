//! @author 十四叔
//! @date 2026/07/17

//! Padding 组件:为子组件添加四边间距。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Edges, Point, Rect, Size};

/// 间距容器:在子组件四周留白。
pub struct Padding {
    edges: Edges,
    child: Node,
    /// layout 缓存:子组件尺寸。
    child_size: Size,
}

impl Padding {
    /// 创建四边等距的间距容器。
    pub fn new(edges: Edges, child: impl Widget + 'static) -> Self {
        Self {
            edges,
            child: std::boxed::Box::new(child),
            child_size: Size::ZERO,
        }
    }

    /// 便捷构造:四边相同间距。
    pub fn all(value: f32, child: impl Widget + 'static) -> Self {
        Self::new(Edges::all(value), child)
    }
}

impl Widget for Padding {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.child.sync(state);
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        self.child.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.child_size = self.child.layout(constraints.deflate(self.edges), texts);
        constraints.constrain(Size::new(
            self.child_size.width + self.edges.horizontal(),
            self.child_size.height + self.edges.vertical(),
        ))
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let inner = Rect::new(
            Point::new(
                area.origin.x + self.edges.left,
                area.origin.y + self.edges.top,
            ),
            self.child_size,
        );
        self.child.paint(inner, rects, texts);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let inner = Rect::new(
            Point::new(
                area.origin.x + self.edges.left,
                area.origin.y + self.edges.top,
            ),
            self.child_size,
        );
        self.child.event(event, inner, msgs)
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }
}
