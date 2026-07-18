//! @author 十四叔
//! @date 2026/07/17

//! Button 组件:可点击按钮,点击产出应用消息。

use std::any::Any;

use crate::event::{Event, Key, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, Node, Widget};
use crate::{Color, Constraints, Edges, Point, Rect, Size};

/// 消息工厂:点击时产出一条应用消息。
type MsgFactory = std::boxed::Box<dyn Fn() -> std::boxed::Box<dyn Any>>;

/// 按钮组件。
///
/// 内含一个子组件(通常是文本标签),自带内边距与背景;
/// hover 变亮、pressed 变暗;点击(按下并原地抬起)或聚焦时按空格/回车产出消息。
pub struct Button {
    child: Node,
    on_click: Option<MsgFactory>,
    color: Color,
    hover_color: Option<Color>,
    radius: f32,
    padding: Edges,
    hovered: bool,
    pressed: bool,
    focused: bool,
    /// layout 缓存:内容区矩形(相对自身原点)。
    child_size: Size,
    /// layout 缓存:自身绝对矩形(用于焦点命中与 IME 区域)。
    area: Rect,
}

impl Button {
    /// 创建按钮(默认青色背景、8px 圆角、12×20 内边距)。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            child: std::boxed::Box::new(child),
            on_click: None,
            color: Color::from_srgb8(0x2E, 0xB8, 0xA5),
            hover_color: None,
            radius: 8.0,
            padding: Edges::symmetric(20.0, 12.0),
            hovered: false,
            pressed: false,
            focused: false,
            child_size: Size::ZERO,
            area: Rect::default(),
        }
    }

    /// 设置点击时产出的消息。
    pub fn on_click<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_click = Some(std::boxed::Box::new(move || {
            std::boxed::Box::new(f()) as std::boxed::Box<dyn Any>
        }));
        self
    }

    /// 设置背景色。
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 设置圆角半径。
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// 设置内边距。
    pub fn padding(mut self, edges: Edges) -> Self {
        self.padding = edges;
        self
    }

    /// 当前是否获得焦点。
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// 按交互状态调制后的实际绘制颜色。
    fn effective_color(&self) -> Color {
        let base = if self.hovered {
            self.hover_color.unwrap_or(self.color)
        } else {
            self.color
        };
        let scale = if self.pressed {
            0.7
        } else if self.hovered && self.hover_color.is_none() {
            1.2
        } else if self.focused {
            1.1
        } else {
            1.0
        };
        Color::rgba(
            (base.r * scale).min(1.0),
            (base.g * scale).min(1.0),
            (base.b * scale).min(1.0),
            base.a,
        )
    }
}

impl Widget for Button {
    fn sync(&mut self, state: &dyn Any) {
        self.child.sync(state);
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        let _ = ctx;
        self.child.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.child_size = self.child.layout(constraints.deflate(self.padding), texts);
        let size = constraints.constrain(Size::new(
            self.child_size.width + self.padding.horizontal(),
            self.child_size.height + self.padding.vertical(),
        ));
        self.area = Rect::new(crate::Point::ZERO, size);
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        rects.push_rect(area, self.effective_color(), self.radius);
        if self.focused {
            // 焦点环:内缩 2px 的细边框,用 4 个小矩形模拟
            let inset = 2.0;
            let color = Color::WHITE;
            let thickness = 1.0;
            let r = Rect::new(
                crate::Point::new(area.origin.x + inset, area.origin.y + inset),
                crate::Size::new(
                    area.size.width - inset * 2.0,
                    area.size.height - inset * 2.0,
                ),
            );
            rects.push_rect(
                Rect::from_xywh(r.origin.x, r.origin.y, r.size.width, thickness),
                color,
                0.0,
            );
            rects.push_rect(
                Rect::from_xywh(
                    r.origin.x,
                    r.origin.y + r.size.height - thickness,
                    r.size.width,
                    thickness,
                ),
                color,
                0.0,
            );
            rects.push_rect(
                Rect::from_xywh(r.origin.x, r.origin.y, thickness, r.size.height),
                color,
                0.0,
            );
            rects.push_rect(
                Rect::from_xywh(
                    r.origin.x + r.size.width - thickness,
                    r.origin.y,
                    thickness,
                    r.size.height,
                ),
                color,
                0.0,
            );
        }
        let inner = Rect::new(
            Point::new(
                area.origin.x + self.padding.left,
                area.origin.y + self.padding.top,
            ),
            self.child_size,
        );
        self.child.paint(inner, rects, texts);
    }

    fn event(
        &mut self,
        event: &Event,
        area: Rect,
        msgs: &mut Vec<std::boxed::Box<dyn Any>>,
    ) -> EventResult {
        self.area = area;
        match event {
            Event::FocusIn => {
                self.focused = true;
                EventResult::Consumed
            }
            Event::FocusOut => {
                self.focused = false;
                self.pressed = false;
                EventResult::Consumed
            }
            Event::Key {
                key: Key::Named(NamedKey::Enter | NamedKey::Space),
                pressed: true,
                ..
            } => {
                if let Some(factory) = &self.on_click {
                    msgs.push(factory());
                }
                EventResult::Consumed
            }
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
                let inside = area.contains(*position);
                if *pressed {
                    if inside {
                        self.pressed = true;
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    let clicked = self.pressed && inside;
                    self.pressed = false;
                    if clicked {
                        if let Some(factory) = &self.on_click {
                            msgs.push(factory());
                        }
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }

    fn ime_area(&self) -> Option<Rect> {
        Some(self.area)
    }
}
