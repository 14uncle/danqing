//! @author 十四叔
//! @date 2026/07/17

//! Center 组件:把子组件居中摆放。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Point, Rect, Size};

/// 居中容器:自身占满 tight 轴的约束上限,子组件按内容尺寸沿该轴居中。
///
/// 逐轴独立判定:tight 轴 (如 Fill 子项分得的主轴,或父容器开启
/// `cross_stretch` 后的交叉轴) 占满并居中;loose 轴按子组件内容自然尺寸,
/// 避免独占父约束上限、把后续兄弟挤出屏幕。需要在宽松轴上同样占满时
/// (如 Flow Fill 子项要求内容在全宽内居中) 使用 [`Center::fill_max`]。
pub struct Center {
    child: Node,
    /// layout 缓存:子组件尺寸。
    child_size: Size,
    /// 是否占满父组件提供的全部空间 (含宽松轴) 并在其中居中。
    fill_max: bool,
}

impl Center {
    /// 创建居中容器。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: std::boxed::Box::new(child),
            child_size: Size::ZERO,
            fill_max: false,
        }
    }

    /// 占满父组件提供的全部空间并在其中居中。
    ///
    /// 默认 Center 在宽松轴上包裹内容 (避免独占父约束、挤出后续兄弟);
    /// 开启后即使约束宽松也占满上限, 适合作为 Flow 的 Fill 子项
    /// (Fill 子项的交叉轴约束是宽松的, 默认行为会让内容贴边而非居中)。
    pub fn fill_max(mut self) -> Self {
        self.fill_max = true;
        self
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
        // 逐轴判定:tight 轴占满 (paint/event 沿该轴居中),loose 轴包裹内容。
        // 子组件一律按 loose 约束测自然尺寸;fill_max 时自身占满全部约束上限。
        let tight_w = constraints.min_width == constraints.max_width;
        let tight_h = constraints.min_height == constraints.max_height;
        self.child_size = self
            .child
            .layout(Constraints::loose(constraints.max()), texts);
        if self.fill_max {
            return constraints.constrain(constraints.max());
        }
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
        let origin = Point::new(
            area.origin.x + (area.size.width - self.child_size.width) / 2.0,
            area.origin.y + (area.size.height - self.child_size.height) / 2.0,
        );
        self.child
            .paint(Rect::new(origin, self.child_size), rects, texts);
    }

    fn paint_image(&self, area: Rect, images: &mut crate::render::ImageBatch) {
        let origin = Point::new(
            area.origin.x + (area.size.width - self.child_size.width) / 2.0,
            area.origin.y + (area.size.height - self.child_size.height) / 2.0,
        );
        self.child
            .paint_image(Rect::new(origin, self.child_size), images);
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
