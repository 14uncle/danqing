//! @author 十四叔
//! @date 2026/07/17

//! Button 组件：可点击按钮，点击产出应用消息。

use std::any::Any;

use crate::event::{Event, Key, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, Node, Widget};
use crate::{Color, Constraints, Edges, LightTheme, Point, Rect, Size, Theme};

/// 消息工厂：点击时产出一条应用消息。
type MsgFactory = Box<dyn Fn() -> Box<dyn Any>>;

/// 按钮组件。
///
/// 内含一个子组件 (通常是文本标签),自带内边距与背景;
/// hover 变亮、pressed 变暗;点击 (按下并原地抬起) 或聚焦时按空格/回车产出消息。
pub struct Button {
    child: Node,
    on_click: Option<MsgFactory>,
    color: Color,
    hover_color: Option<Color>,
    focus_color: Color,
    radius: f32,
    padding: Edges,
    hovered: bool,
    pressed: bool,
    focused: bool,
    /// layout 缓存：内容区矩形 (相对自身原点)。
    child_size: Size,
    /// layout 缓存：自身绝对矩形 (用于焦点命中与 IME 区域)。
    area: Rect,
}

impl Button {
    /// 创建按钮，使用默认浅色主题 token。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self::themed(&LightTheme, child)
    }

    /// 使用指定主题创建按钮。
    pub fn themed(theme: &impl Theme, child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            on_click: None,
            color: theme.accent(),
            hover_color: None,
            focus_color: Color::WHITE,
            radius: theme.radius_md(),
            padding: Edges::symmetric(theme.spacing_lg(), theme.spacing_md()),
            hovered: false,
            pressed: false,
            focused: false,
            child_size: Size::ZERO,
            area: Rect::default(),
        }
    }

    /// 设置点击时产出的消息。
    pub fn on_click<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_click = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
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
        self.area = Rect::new(Point::ZERO, size);
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        rects.push_rect(area, self.effective_color(), self.radius);
        if self.focused {
            // 焦点环：内缩 3px 的白色圆角虚线边框 (线宽 1px),跟随按钮圆角。
            let inset = 3.0;
            let focus_rect = Rect::new(
                Point::new(area.origin.x + inset, area.origin.y + inset),
                Size::new(
                    area.size.width - inset * 2.0,
                    area.size.height - inset * 2.0,
                ),
            );
            // 虚线参数：划线 4px、空隙 2px。
            rects.push_dashed_border(focus_rect, self.focus_color, self.radius, 4.0, 2.0, 1.0);
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

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut Vec<Box<dyn Any>>) -> EventResult {
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

    fn ime_area(&self) -> Option<Rect> {
        Some(self.area)
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }
}

#[cfg(test)]
impl Button {
    /// 当前背景色 (测试用)。
    pub(crate) fn color_value(&self) -> Color {
        self.color
    }

    /// 当前焦点环颜色 (测试用)。
    pub(crate) fn focus_color_value(&self) -> Color {
        self.focus_color
    }

    /// 当前圆角半径 (测试用)。
    pub(crate) fn radius_value(&self) -> f32 {
        self.radius
    }

    /// 当前内边距 (测试用)。
    pub(crate) fn padding_value(&self) -> Edges {
        self.padding
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::Text;

    #[test]
    fn button_new_uses_light_theme_defaults() {
        let button = Button::new(Text::new("OK"));
        assert_eq!(button.color_value(), LightTheme.accent());
        assert_eq!(button.focus_color_value(), Color::WHITE);
        assert_eq!(button.radius_value(), LightTheme.radius_md());
        assert_eq!(
            button.padding_value(),
            Edges::symmetric(LightTheme.spacing_lg(), LightTheme.spacing_md())
        );
    }

    #[test]
    fn button_themed_uses_provided_theme() {
        let button = Button::themed(&LightTheme, Text::new("OK"));
        assert_eq!(button.color_value(), LightTheme.accent());
        assert_eq!(button.radius_value(), LightTheme.radius_md());
    }

    #[test]
    fn button_custom_overrides_theme() {
        let custom = Color::from_srgb8(255, 0, 0);
        let button = Button::new(Text::new("OK")).color(custom).radius(16.0);
        assert_eq!(button.color_value(), custom);
        assert_eq!(button.radius_value(), 16.0);
    }
}
