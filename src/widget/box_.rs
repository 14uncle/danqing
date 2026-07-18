//! @author 十四叔
//! @date 2026/07/17

//! Box 组件:带背景色与圆角的矩形块,可含一个子组件。
//!
//! 默认可交互:hover 变亮、pressed 变暗([`Box::hoverable`] 可关闭)。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Color, Constraints, Rect, Size};

/// 背景色块组件。
///
/// 默认占满父组件给的最大尺寸;也可指定显式宽高。
pub struct Box {
    color: Color,
    radius: f32,
    width: Option<f32>,
    height: Option<f32>,
    child: Option<Node>,
    hoverable: bool,
    hovered: bool,
    pressed: bool,
}

impl Box {
    /// 创建背景色块(直角,占满父约束)。
    pub fn new(color: Color) -> Self {
        Self {
            color,
            radius: 0.0,
            width: None,
            height: None,
            child: None,
            hoverable: true,
            hovered: false,
            pressed: false,
        }
    }

    /// 设置圆角半径(逻辑像素)。
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// 设置显式宽高(未设的维度仍按父约束)。
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// 仅设置显式宽度。
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// 仅设置显式高度。
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// 设置子组件(占满 Box 内容区)。
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(std::boxed::Box::new(child));
        self
    }

    /// 开关 hover/pressed 交互效果(默认开)。
    pub fn hoverable(mut self, hoverable: bool) -> Self {
        self.hoverable = hoverable;
        self
    }

    /// 当前是否 hover。
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// 当前是否按下。
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// 按交互状态调制后的实际绘制颜色。
    fn effective_color(&self) -> Color {
        if !self.hoverable {
            return self.color;
        }
        let scale = if self.pressed {
            0.7
        } else if self.hovered {
            1.25
        } else {
            1.0
        };
        Color::rgba(
            (self.color.r * scale).min(1.0),
            (self.color.g * scale).min(1.0),
            (self.color.b * scale).min(1.0),
            self.color.a,
        )
    }
}

impl Widget for Box {
    fn sync(&mut self, state: &dyn std::any::Any) {
        if let Some(child) = &mut self.child {
            child.sync(state);
        }
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        if let Some(child) = &mut self.child {
            child.animate(ctx);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        let size = constraints.constrain(Size::new(
            self.width.unwrap_or(constraints.max_width),
            self.height.unwrap_or(constraints.max_height),
        ));
        if let Some(child) = &mut self.child {
            child.layout(Constraints::tight(size), texts);
        }
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        rects.push_rect(area, self.effective_color(), self.radius);
        if let Some(child) = &self.child {
            child.paint(area, rects, texts);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        // 先分发给子组件:移动类全发,其他类命中才发
        if let Some(child) = &mut self.child {
            let forward = match event {
                Event::CursorMoved(_) | Event::CursorLeft => true,
                e => e.position().is_some_and(|p| area.contains(p)),
            };
            if forward && child.event(event, area, msgs) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        if !self.hoverable {
            return EventResult::Ignored;
        }
        match event {
            Event::CursorMoved(p) => {
                self.hovered = area.contains(*p);
                if self.hovered {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::CursorLeft => {
                self.hovered = false;
                self.pressed = false;
                EventResult::Ignored
            }
            Event::MouseInput {
                pressed, position, ..
            } => {
                if *pressed {
                    if area.contains(*position) {
                        self.pressed = true;
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    let was_pressed = self.pressed;
                    self.pressed = false;
                    if was_pressed && area.contains(*position) {
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn children(&self) -> &[Node] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn children_mut(&mut self) -> &mut [Node] {
        match &mut self.child {
            Some(child) => std::slice::from_mut(child),
            None => &mut [],
        }
    }
}
