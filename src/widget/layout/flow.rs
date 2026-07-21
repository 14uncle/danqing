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

    /// 交叉轴拉伸约束(开启 cross_stretch 的 Fit 子项):
    /// 交叉轴固定为容器交叉尺寸,主轴保持宽松以保留自然主轴尺寸。
    fn stretch_constraints(self, main_max: f32, cross: f32) -> Constraints {
        match self {
            Axis::Horizontal => Constraints {
                min_width: 0.0,
                max_width: main_max,
                min_height: cross,
                max_height: cross,
            },
            Axis::Vertical => Constraints {
                min_width: cross,
                max_width: cross,
                min_height: 0.0,
                max_height: main_max,
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
    /// 是否把 Fit 子项拉伸到容器交叉尺寸(默认 false,保持自然尺寸)。
    cross_stretch: bool,
    /// layout 阶段缓存:各子组件相对容器原点的摆放矩形。
    areas: Vec<Rect>,
}

impl Flow {
    pub fn new(gap: f32) -> Self {
        Self {
            children: Vec::new(),
            weights: Vec::new(),
            gap,
            cross_stretch: false,
            areas: Vec::new(),
        }
    }

    /// 设置子项间距。
    pub fn set_gap(&mut self, gap: f32) {
        self.gap = gap;
    }

    /// 开关 Fit 子项的交叉轴拉伸。
    pub fn set_cross_stretch(&mut self, cross_stretch: bool) {
        self.cross_stretch = cross_stretch;
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
                if self.cross_stretch {
                    // Fit 子项拉伸:交叉轴 tight 为容器交叉尺寸,重新布局。
                    // 主轴保持宽松,绝大多数组件的主轴尺寸与交叉轴无关,
                    // 因此第一遍算出的主轴分配仍然成立。
                    let laid_out =
                        child.layout(axis.stretch_constraints(main_max, cross_max), texts);
                    debug_assert!(
                        axis.main(laid_out) <= main_size + f32::EPSILON,
                        "cross_stretch 要求 Fit 子项的主轴尺寸不随交叉轴增大而增大,\
                         否则后续兄弟会与它重叠"
                    );
                    laid_out
                } else {
                    axis.make_size(main_size, fit_cross[i])
                }
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
        if broadcast {
            // 广播必须送达每个子组件 (各自维护 hover 等状态);
            // 某个子组件消费事件不得阻止其余子组件接收。
            let mut consumed = false;
            for (child, area) in self.children.iter_mut().zip(self.areas.iter()).rev() {
                let child_area = area.translate(origin.x, origin.y);
                if child.event(event, child_area, msgs) == EventResult::Consumed {
                    consumed = true;
                }
            }
            return if consumed {
                EventResult::Consumed
            } else {
                EventResult::Ignored
            };
        }
        // 后绘制者(z 序靠上)优先命中
        for (child, area) in self.children.iter_mut().zip(self.areas.iter()).rev() {
            let child_area = area.translate(origin.x, origin.y);
            let hit = event.position().is_none_or(|p| child_area.contains(p));
            if hit && child.event(event, child_area, msgs) == EventResult::Consumed {
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
    use crate::event::{Event, MouseButton};
    use crate::widget::{Box as UiBox, EventResult, MsgQueue, Widget, node};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn screen(w: f32, h: f32) -> Constraints {
        Constraints::tight(Size::new(w, h))
    }

    /// 测试用记录组件: 记录是否收到事件, 并按配置返回消费结果。
    struct Recorder {
        received: Rc<RefCell<bool>>,
        consume: bool,
    }

    impl Widget for Recorder {
        fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
            constraints.constrain(Size::new(10.0, 10.0))
        }

        fn paint(&self, _area: Rect, _rects: &mut crate::RectBatch, _texts: &mut TextBatch) {}

        fn event(&mut self, _event: &Event, _area: Rect, _msgs: &mut MsgQueue) -> EventResult {
            *self.received.borrow_mut() = true;
            if self.consume {
                EventResult::Consumed
            } else {
                EventResult::Ignored
            }
        }
    }

    #[test]
    fn broadcast_reaches_all_children_even_if_one_consumes() {
        let mut texts = TextBatch::new();
        let mut flow = Flow::new(0.0);
        let first_seen = Rc::new(RefCell::new(false));
        let second_seen = Rc::new(RefCell::new(false));
        // 先压入的先绘制; 广播按 rev 顺序, 后压入者先收到事件并消费。
        flow.push(
            node(Recorder {
                received: Rc::clone(&first_seen),
                consume: false,
            }),
            0,
        );
        flow.push(
            node(Recorder {
                received: Rc::clone(&second_seen),
                consume: true,
            }),
            0,
        );
        flow.layout(Axis::Vertical, screen(100.0, 100.0), &mut texts);

        let mut msgs = MsgQueue::new();
        let result = flow.event(
            crate::Point::ZERO,
            &Event::CursorMoved(crate::Point::new(5.0, 15.0)),
            &mut msgs,
        );

        assert!(*second_seen.borrow(), "后绘制的子组件应先收到广播并消费");
        assert!(
            *first_seen.borrow(),
            "广播不得因兄弟组件消费而中断 (标题栏 hover 依赖此语义)"
        );
        assert_eq!(result, EventResult::Consumed);
    }

    #[test]
    fn hit_dispatch_skips_children_outside_position() {
        let mut texts = TextBatch::new();
        let mut flow = Flow::new(0.0);
        let first_seen = Rc::new(RefCell::new(false));
        let second_seen = Rc::new(RefCell::new(false));
        flow.push(
            node(Recorder {
                received: Rc::clone(&first_seen),
                consume: false,
            }),
            0,
        );
        flow.push(
            node(Recorder {
                received: Rc::clone(&second_seen),
                consume: true,
            }),
            0,
        );
        flow.layout(Axis::Vertical, screen(100.0, 100.0), &mut texts);

        let mut msgs = MsgQueue::new();
        // 命中第一个子组件 (y 0..10), 第二个 (y 10..20) 不应收到。
        let result = flow.event(
            crate::Point::ZERO,
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: crate::Point::new(5.0, 5.0),
            },
            &mut msgs,
        );

        assert!(*first_seen.borrow(), "被命中的子组件应收到事件");
        assert!(!*second_seen.borrow(), "未命中的子组件不应收到事件");
        assert_eq!(result, EventResult::Ignored);
    }

    #[test]
    fn column_cross_stretch_expands_fit_children() {
        // 开启交叉轴拉伸后,Fit 子项的交叉尺寸扩到容器交叉尺寸(最宽子项)。
        let mut texts = TextBatch::new();
        let mut flow = Flow::new(10.0);
        flow.set_cross_stretch(true);
        flow.push(node(UiBox::new(Color::BLACK).size(50.0, 30.0)), 0);
        flow.push(node(UiBox::new(Color::BLACK).size(80.0, 20.0)), 0);
        let size = flow.layout(
            Axis::Vertical,
            Constraints::loose(Size::new(200.0, 400.0)),
            &mut texts,
        );
        assert_eq!(size, Size::new(80.0, 60.0));
        assert_eq!(flow.areas[0], Rect::from_xywh(0.0, 0.0, 80.0, 30.0));
        assert_eq!(flow.areas[1], Rect::from_xywh(0.0, 40.0, 80.0, 20.0));
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
