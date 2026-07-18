//! @author 十四叔
//! @date 2026/07/17

//! 单轴流式容器(Column/Row)的共享布局实现。

use crate::render::TextBatch;
use crate::widget::Node;
use crate::{Constraints, FlowChild, Rect, Size, distribute};

/// 主轴方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// 水平(Row)。
    Horizontal,
    /// 垂直(Column)。
    Vertical,
}

impl Axis {
    fn main(self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn cross(self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

    fn make_size(self, main: f32, cross: f32) -> Size {
        match self {
            Axis::Horizontal => Size::new(main, cross),
            Axis::Vertical => Size::new(cross, main),
        }
    }

    fn make_rect(self, main_offset: f32, size: Size) -> Rect {
        match self {
            Axis::Horizontal => Rect::from_xywh(main_offset, 0.0, size.width, size.height),
            Axis::Vertical => Rect::from_xywh(0.0, main_offset, size.width, size.height),
        }
    }

    fn cross_min(self, c: Constraints) -> f32 {
        match self {
            Axis::Horizontal => c.min_height,
            Axis::Vertical => c.min_width,
        }
    }

    /// Fill 子项的约束:主轴固定(分得的尺寸),交叉轴宽松。
    ///
    /// 交叉轴不用 tight —— 让显式尺寸的子项保留自己的交叉尺寸
    /// (如定高色块),未指定尺寸的子项自然占满交叉轴。
    fn fill_constraints(self, main_alloc: f32, cross_max: f32) -> Constraints {
        match self {
            Axis::Horizontal => Constraints {
                min_width: main_alloc,
                max_width: main_alloc,
                min_height: 0.0,
                max_height: cross_max,
            },
            Axis::Vertical => Constraints {
                min_width: 0.0,
                max_width: cross_max,
                min_height: main_alloc,
                max_height: main_alloc,
            },
        }
    }
}

/// 单轴流式容器的状态(Column/Row 共用)。
pub struct Flow {
    /// 子组件。
    children: Vec<Node>,
    /// 每个子组件的填充权重(与 `children` 一一对应)。
    weights: Vec<u32>,
    /// 子项间距。
    gap: f32,
    /// layout 阶段缓存:各子组件相对容器原点的摆放矩形。
    areas: Vec<Rect>,
}

impl Flow {
    pub fn new(gap: f32) -> Self {
        Self {
            children: Vec::new(),
            weights: Vec::new(),
            gap,
            areas: Vec::new(),
        }
    }

    pub fn push(&mut self, child: Node, weight: u32) {
        self.children.push(child);
        self.weights.push(weight);
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [Node] {
        &mut self.children
    }

    /// 流式布局:Fit 子项先量,Fill 子项按权重分剩余空间。
    pub fn layout(&mut self, axis: Axis, constraints: Constraints, texts: &mut TextBatch) -> Size {
        // 第一遍:仅量 Fit 子项,得到其自然主轴/交叉轴尺寸。
        let mut flows: Vec<FlowChild> = Vec::with_capacity(self.children.len());
        let mut fit_cross: Vec<f32> = Vec::with_capacity(self.children.len());
        for (child, weight) in self.children.iter_mut().zip(self.weights.iter()) {
            if *weight == 0 {
                let size = child.layout(Constraints::loose(constraints.max()), texts);
                flows.push(FlowChild {
                    main_fixed: axis.main(size),
                    fill_weight: 0,
                });
                fit_cross.push(axis.cross(size));
            } else {
                flows.push(FlowChild {
                    main_fixed: 0.0,
                    fill_weight: *weight,
                });
                fit_cross.push(0.0);
            }
        }

        let main_max = axis.main(constraints.max());
        let dist = distribute(main_max, self.gap, &flows);

        // 容器实际交叉高度:Fit 子项自然交叉高的最大值,同时不低于父约束的交叉下限
        // (保证 tight 约束下 Fill 子项能撑满容器)。
        let cross_max = fit_cross
            .iter()
            .copied()
            .fold(axis.cross_min(constraints), f32::max);

        self.areas.clear();
        let mut used_main = 0.0f32;
        let mut has_fill = false;
        for (i, (child, weight)) in self
            .children
            .iter_mut()
            .zip(self.weights.iter())
            .enumerate()
        {
            let (offset, main_size) = dist[i];
            let child_size = if *weight == 0 {
                axis.make_size(main_size, fit_cross[i])
            } else {
                has_fill = true;
                child.layout(axis.fill_constraints(main_size, cross_max), texts)
            };
            used_main = used_main.max(offset + axis.main(child_size));
            self.areas.push(axis.make_rect(offset, child_size));
        }

        // 有 Fill 子项时容器占满主轴;否则按内容自然尺寸
        let main = if has_fill { main_max } else { used_main };
        constraints.constrain(axis.make_size(main, cross_max))
    }

    /// 按缓存的摆放矩形绘制(相对原点平移)。
    pub fn paint(&self, origin: crate::Point, rects: &mut crate::RectBatch, texts: &mut TextBatch) {
        for (child, area) in self.children.iter().zip(self.areas.iter()) {
            child.paint(area.translate(origin.x, origin.y), rects, texts);
        }
    }

    /// 事件分发:移动类广播全树;其他事件沿命中路径(后绘制者优先)。
    pub fn event(
        &mut self,
        origin: crate::Point,
        event: &crate::event::Event,
        msgs: &mut crate::widget::MsgQueue,
    ) -> crate::widget::EventResult {
        use crate::widget::EventResult;
        let broadcast = matches!(
            event,
            crate::event::Event::CursorMoved(_) | crate::event::Event::CursorLeft
        );
        // 后绘制者(z 序靠上)优先命中
        for (child, area) in self.children.iter_mut().zip(self.areas.iter()).rev() {
            let child_area = area.translate(origin.x, origin.y);
            let hit = event.position().is_none_or(|p| child_area.contains(p));
            if (broadcast || hit) && child.event(event, child_area, msgs) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        EventResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::widget::{Box as UiBox, node};

    fn screen(w: f32, h: f32) -> Constraints {
        Constraints::tight(Size::new(w, h))
    }

    #[test]
    fn column_stacks_fit_children() {
        let mut texts = TextBatch::new();
        let mut flow = Flow::new(10.0);
        flow.push(node(UiBox::new(Color::BLACK).size(50.0, 30.0)), 0);
        flow.push(node(UiBox::new(Color::BLACK).size(80.0, 20.0)), 0);
        // 宽松约束:容器按内容取自然尺寸
        let size = flow.layout(
            Axis::Vertical,
            Constraints::loose(Size::new(200.0, 400.0)),
            &mut texts,
        );
        // 主(column 高)= 30+20+10 = 60;交叉取最大宽 80
        assert_eq!(size, Size::new(80.0, 60.0));
        assert_eq!(flow.areas[0], Rect::from_xywh(0.0, 0.0, 50.0, 30.0));
        assert_eq!(flow.areas[1], Rect::from_xywh(0.0, 40.0, 80.0, 20.0));
    }

    #[test]
    fn row_fill_shares_remaining() {
        let mut texts = TextBatch::new();
        let mut flow = Flow::new(0.0);
        flow.push(node(UiBox::new(Color::BLACK).size(50.0, 10.0)), 0);
        flow.push(node(UiBox::new(Color::BLACK)), 1);
        flow.push(node(UiBox::new(Color::BLACK)), 1);
        let size = flow.layout(Axis::Horizontal, screen(250.0, 40.0), &mut texts);
        assert_eq!(size, Size::new(250.0, 40.0));
        // 两个 fill 各得 (250-50)/2 = 100
        assert_eq!(flow.areas[1], Rect::from_xywh(50.0, 0.0, 100.0, 40.0));
        assert_eq!(flow.areas[2], Rect::from_xywh(150.0, 0.0, 100.0, 40.0));
    }

    #[test]
    fn row_height_follows_fit_children_not_parent_max() {
        let mut texts = TextBatch::new();
        let mut flow = Flow::new(0.0);
        // Fit 子项高 20,Fill 子项不应把 Row 撑到父约束的 800 高
        flow.push(node(UiBox::new(Color::BLACK).size(50.0, 20.0)), 0);
        flow.push(node(UiBox::new(Color::BLACK)), 1);
        let size = flow.layout(
            Axis::Horizontal,
            Constraints::loose(Size::new(300.0, 800.0)),
            &mut texts,
        );
        assert!(
            size.height <= 30.0,
            "Row 高度应接近 Fit 子项,而非父约束的 800;实际 {size:?}"
        );
    }
}
