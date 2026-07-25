//! @author 十四叔
//! @date 2026/07/25

//! 多子组件层叠布局: 所有子组件共用同一区域, 后添加的覆盖先添加的。
//!
//! 用例: 模态、Toast、完成反馈脉冲等需要绘制在已有 UI 之上的临时层。
//! 事件分发按"后添加者优先" (即最上层先尝试消费), 焦点遍历同样从最上层开始。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Rect, Size};

/// 层叠容器: 所有子组件按相同 `area` 摆放, 绘制顺序为"后添加者居上"。
/// 布局结果是所有子组件布局尺寸的最大包络 (再夹到约束内)。
pub struct Stack {
    children: Vec<Node>,
}

impl Stack {
    /// 创建空层叠容器。
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// 添加子组件 (后添加者绘制在上层, 事件优先消费)。
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// 子组件数量。
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// 子组件是否为空。
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Stack {
    fn sync(&mut self, state: &dyn std::any::Any) {
        for child in &mut self.children {
            child.sync(state);
        }
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        for child in &mut self.children {
            child.animate(ctx);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        let mut envelope = Size::ZERO;
        for child in &mut self.children {
            let size = child.layout(constraints, texts);
            envelope.width = envelope.width.max(size.width);
            envelope.height = envelope.height.max(size.height);
        }
        constraints.constrain(envelope)
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        for child in &self.children {
            child.paint(area, rects, texts);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        for child in self.children.iter_mut().rev() {
            if child.event(event, area, msgs) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        EventResult::Ignored
    }

    fn focusable(&self) -> bool {
        self.children.iter().any(|c| c.focusable())
    }

    fn children(&self) -> &[Node] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Node] {
        &mut self.children
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Size;

    #[test]
    fn empty_stack_is_constrained() {
        let mut s = Stack::new();
        let out = s.layout(
            Constraints::tight(Size::new(100.0, 50.0)),
            &mut TextBatch::new(),
        );
        assert_eq!(out, Size::new(100.0, 50.0));
    }

    #[test]
    fn stack_with_two_children_has_two_children() {
        let s = Stack::new()
            .child(crate::widget::Text::new("a"))
            .child(crate::widget::Text::new("b"));
        assert_eq!(s.len(), 2);
        assert!(!s.is_empty());
    }
}
