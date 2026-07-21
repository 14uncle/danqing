//! @author 十四叔
//! @date 2026/07/17

//! Column 组件:垂直流式容器。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::flow::{Axis, Flow};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Constraints, Rect, Size};

/// 垂直排列的容器。
///
/// 子组件自上而下排列;[`Column::fill`] 添加的子组件按权重
/// 瓜分剩余主轴空间,[`Column::child`] 添加的按内容自然尺寸。
pub struct Column {
    flow: Flow,
}

impl Column {
    /// 创建无间距列。
    pub fn new() -> Self {
        Self {
            flow: Flow::new(0.0),
        }
    }

    /// 设置子项间距。
    pub fn gap(mut self, gap: f32) -> Self {
        self.flow.set_gap(gap);
        self
    }

    /// 把 Fit 子项拉伸到容器宽度(最宽子项的自然宽),使卡片等宽。
    ///
    /// 注意:Fit 子项会被布局两遍(先量自然尺寸,再按拉伸约束重排);
    /// 要求子项的高度不随宽度增大而增大(如保持宽高比的图片组件不适用)。
    pub fn cross_stretch(mut self) -> Self {
        self.flow.set_cross_stretch(true);
        self
    }

    /// 添加按内容自然尺寸的子组件。
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.flow.push(std::boxed::Box::new(child), 0);
        self
    }

    /// 添加按权重瓜分剩余主轴空间的子组件。
    pub fn fill(mut self, child: impl Widget + 'static, weight: u32) -> Self {
        self.flow.push(std::boxed::Box::new(child), weight);
        self
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Column {
    fn sync(&mut self, state: &dyn std::any::Any) {
        for child in self.flow.children_mut().iter_mut() {
            child.sync(state);
        }
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        for child in self.flow.children_mut().iter_mut() {
            child.animate(ctx);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.flow.layout(Axis::Vertical, constraints, texts)
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        self.flow.paint(area.origin, rects, texts);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.flow.event(area.origin, event, msgs)
    }

    fn children(&self) -> &[crate::widget::Node] {
        self.flow.children()
    }

    fn children_mut(&mut self) -> &mut [crate::widget::Node] {
        self.flow.children_mut()
    }
}
