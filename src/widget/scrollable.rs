//! @author 十四叔
//! @date 2026/07/18

//! 滚动容器:允许子组件在垂直/水平方向上滚动。
//!
//! `Scrollable` 负责维护滚动偏移、视口裁剪与滚轮事件;
//! 子组件只需报告自然内容尺寸。

use std::cell::Cell;

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Point, Rect, Size};

/// 滚动方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxis {
    /// 仅垂直滚动。
    Vertical,
    /// 仅水平滚动。
    Horizontal,
    /// 双向滚动。
    Both,
}

/// 子组件在滚动轴上允许的最大尺寸。
///
/// 用有限大值代替 `f32::INFINITY`,避免 Flow 等布局算法在分配 Fill 权重时溢出。
const MAX_CONTENT_SIZE: f32 = 1_000_000.0;

/// 滚动容器。
pub struct Scrollable {
    child: Node,
    axis: ScrollAxis,
    scroll_offset: Point,
    scroll_speed: f32,
    child_size: Size,
    viewport_size: Size,
    /// 自身绝对矩形,在 paint 阶段缓存,供 hit_area 使用。
    area: Cell<Rect>,
}

impl Scrollable {
    /// 创建滚动容器,默认垂直滚动。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            axis: ScrollAxis::Vertical,
            scroll_offset: Point::ZERO,
            scroll_speed: 40.0,
            child_size: Size::ZERO,
            viewport_size: Size::ZERO,
            area: Cell::new(Rect::default()),
        }
    }

    /// 设置滚动方向。
    pub fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    /// 设置滚轮每次滚动的逻辑像素数。
    pub fn scroll_speed(mut self, speed: f32) -> Self {
        self.scroll_speed = speed;
        self
    }

    /// 当前滚动偏移。
    pub fn scroll_offset(&self) -> Point {
        self.scroll_offset
    }

    fn max_offset(&self) -> Point {
        Point::new(
            (self.child_size.width - self.viewport_size.width).max(0.0),
            (self.child_size.height - self.viewport_size.height).max(0.0),
        )
    }

    fn clamp_offset(&mut self) {
        let max = self.max_offset();
        self.scroll_offset.x = self.scroll_offset.x.clamp(0.0, max.x);
        self.scroll_offset.y = self.scroll_offset.y.clamp(0.0, max.y);
    }

    fn child_constraints(&self) -> Constraints {
        match self.axis {
            ScrollAxis::Vertical => {
                Constraints::loose(Size::new(self.viewport_size.width, MAX_CONTENT_SIZE))
            }
            ScrollAxis::Horizontal => {
                Constraints::loose(Size::new(MAX_CONTENT_SIZE, self.viewport_size.height))
            }
            ScrollAxis::Both => Constraints::loose(Size::new(MAX_CONTENT_SIZE, MAX_CONTENT_SIZE)),
        }
    }

    fn transform_event(&self, event: &Event) -> Option<Event> {
        match event {
            Event::CursorMoved(p) => Some(Event::CursorMoved(Point::new(
                p.x + self.scroll_offset.x,
                p.y + self.scroll_offset.y,
            ))),
            Event::MouseInput {
                button,
                pressed,
                position,
            } => Some(Event::MouseInput {
                button: *button,
                pressed: *pressed,
                position: Point::new(
                    position.x + self.scroll_offset.x,
                    position.y + self.scroll_offset.y,
                ),
            }),
            Event::MouseWheel { delta, position } => Some(Event::MouseWheel {
                delta: *delta,
                position: Point::new(
                    position.x + self.scroll_offset.x,
                    position.y + self.scroll_offset.y,
                ),
            }),
            // 无位置事件直接转发。
            _ => Some(event.clone()),
        }
    }

    fn handle_wheel(&mut self, delta: (f32, f32)) {
        match self.axis {
            ScrollAxis::Vertical => {
                self.scroll_offset.y -= delta.1 * self.scroll_speed;
            }
            ScrollAxis::Horizontal => {
                self.scroll_offset.x -= delta.0 * self.scroll_speed;
            }
            ScrollAxis::Both => {
                if delta.1 != 0.0 {
                    self.scroll_offset.y -= delta.1 * self.scroll_speed;
                } else if delta.0 != 0.0 {
                    self.scroll_offset.x -= delta.0 * self.scroll_speed;
                }
            }
        }
        self.clamp_offset();
    }

    fn draw_scrollbar(&self, area: Rect, rects: &mut RectBatch) {
        let track_color = crate::Color::from_srgb8(0xD0, 0xD0, 0xD0);
        let thumb_color = crate::Color::from_srgb8(0x80, 0x80, 0x80);
        let track_width = 6.0;

        // 垂直滚动条
        if self.child_size.height > self.viewport_size.height {
            let ratio = self.viewport_size.height / self.child_size.height;
            let thumb_height = (self.viewport_size.height * ratio).max(track_width);
            let max_offset_y = (self.child_size.height - self.viewport_size.height).max(0.0);
            let thumb_offset_y = if max_offset_y > 0.0 {
                (self.scroll_offset.y / max_offset_y) * (self.viewport_size.height - thumb_height)
            } else {
                0.0
            };
            let track_x = area.origin.x + area.size.width - track_width;
            rects.push_rect(
                Rect::from_xywh(track_x, area.origin.y, track_width, area.size.height),
                track_color,
                0.0,
            );
            rects.push_rect(
                Rect::from_xywh(
                    track_x,
                    area.origin.y + thumb_offset_y,
                    track_width,
                    thumb_height,
                ),
                thumb_color,
                3.0,
            );
        }

        // 水平滚动条
        if self.child_size.width > self.viewport_size.width {
            let ratio = self.viewport_size.width / self.child_size.width;
            let thumb_width = (self.viewport_size.width * ratio).max(track_width);
            let max_offset_x = (self.child_size.width - self.viewport_size.width).max(0.0);
            let thumb_offset_x = if max_offset_x > 0.0 {
                (self.scroll_offset.x / max_offset_x) * (self.viewport_size.width - thumb_width)
            } else {
                0.0
            };
            let track_y = area.origin.y + area.size.height - track_width;
            rects.push_rect(
                Rect::from_xywh(area.origin.x, track_y, area.size.width, track_width),
                track_color,
                0.0,
            );
            rects.push_rect(
                Rect::from_xywh(
                    area.origin.x + thumb_offset_x,
                    track_y,
                    thumb_width,
                    track_width,
                ),
                thumb_color,
                3.0,
            );
        }
    }
}

impl Widget for Scrollable {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.child.sync(state);
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        self.child.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.viewport_size = constraints.max();
        self.child_size = self.child.layout(self.child_constraints(), texts);
        // 子组件可能比视口小;滚动偏移需要重新限幅。
        self.clamp_offset();
        self.area.set(Rect::new(Point::ZERO, self.viewport_size));
        self.viewport_size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        self.area.set(area);

        rects.push_clip(area);
        texts.push_clip(area);

        let child_area = Rect::new(
            Point::new(
                area.origin.x - self.scroll_offset.x,
                area.origin.y - self.scroll_offset.y,
            ),
            self.child_size,
        );
        self.child.paint(child_area, rects, texts);

        texts.pop_clip();
        rects.pop_clip();

        // 绘制滚动条(在裁剪区外,不需要再裁剪)。
        self.draw_scrollbar(area, rects);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.area.set(area);
        let inside = match event.position() {
            Some(p) => area.contains(p),
            None => false,
        };

        match event {
            Event::CursorLeft => {
                self.child.event(event, area, msgs);
                return EventResult::Ignored;
            }
            Event::MouseWheel { delta, .. } if inside => {
                self.handle_wheel(*delta);
                return EventResult::Consumed;
            }
            _ => {}
        }

        if !inside {
            return EventResult::Ignored;
        }

        let transformed = match self.transform_event(event) {
            Some(e) => e,
            None => return EventResult::Ignored,
        };

        // 对鼠标按键,只有真正落在子组件内容区(含滚动偏移)才消费;
        // 否则仍视为在视口内点击,消费事件防止冒泡到应用层。
        let child_result = self.child.event(&transformed, area, msgs);
        if child_result == EventResult::Consumed {
            child_result
        } else {
            EventResult::Consumed
        }
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::widget::{Box as UiBox, node};

    #[test]
    fn vertical_scroll_clamps_offset() {
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(50.0, 500.0));
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
        assert_eq!(scroll.viewport_size, Size::new(100.0, 100.0));
        assert_eq!(scroll.child_size, Size::new(50.0, 500.0));

        // 滚轮向下滚动 1000 像素,应被限幅到 content - viewport = 400。
        scroll.handle_wheel((0.0, -25.0));
        assert!((scroll.scroll_offset.y - 400.0).abs() < f32::EPSILON);

        // 滚轮向上回滚,应回到 0。
        scroll.handle_wheel((0.0, 25.0));
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
    }

    #[test]
    fn wheel_outside_viewport_is_ignored() {
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(50.0, 500.0));
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);

        let event = Event::MouseWheel {
            delta: (0.0, -5.0),
            position: Point::new(200.0, 200.0),
        };
        let result = scroll.event(
            &event,
            Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
            &mut Vec::new(),
        );
        assert_eq!(result, EventResult::Ignored);
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_axis_uses_x_delta() {
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(500.0, 50.0))
            .axis(ScrollAxis::Horizontal);
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
        scroll.handle_wheel((-5.0, 0.0));
        assert!(scroll.scroll_offset.x > 0.0);
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
    }
}
