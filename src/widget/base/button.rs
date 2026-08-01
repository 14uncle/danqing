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

/// 颜色绑定闭包：从类型擦除的应用状态产出按钮背景色。
type ColorBinding = Box<dyn Fn(&dyn Any) -> Color>;

/// 按钮组件。
///
/// 内含一个子组件 (通常是文本标签),自带内边距与背景;
/// hover 变亮、pressed 变暗;点击 (按下并原地抬起) 或聚焦时按空格/回车产出消息。
pub struct Button {
    child: Node,
    on_click: Option<MsgFactory>,
    color: Color,
    color_binding: Option<ColorBinding>,
    hover_color: Option<Color>,
    hover_binding: Option<ColorBinding>,
    focus_color: Color,
    focus_binding: Option<ColorBinding>,
    radius: f32,
    padding: Edges,
    hovered: bool,
    pressed: bool,
    focused: bool,
    /// 稳定焦点标识 (按名聚焦: 弹层面板关闭后焦点回到打开面板的按钮)。
    id: Option<&'static str>,
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
            color_binding: None,
            hover_color: None,
            hover_binding: None,
            focus_color: Color::WHITE,
            focus_binding: None,
            radius: theme.radius_md(),
            padding: Edges::symmetric(theme.spacing_lg(), theme.spacing_md()),
            hovered: false,
            pressed: false,
            focused: false,
            id: None,
            child_size: Size::ZERO,
            area: Rect::default(),
        }
    }

    /// 设置稳定焦点标识 (按名聚焦: 面板关闭后焦点回到此按钮)。
    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
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

    /// 绑定背景色：每帧从应用状态读取背景色 (如导航选中态)。
    ///
    /// 与 [`crate::widget::Text::bind`] 同构;设置后覆盖 `color` 的静态值。
    pub fn bind_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.color_binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Button 颜色绑定的状态类型不匹配");
            f(state)
        }));
        self
    }

    /// 绑定悬停背景色：每帧从应用状态读取悬停色。
    ///
    /// 与 [`Button::bind_color`] 同构;设置后悬停不再按 1.2 倍提亮,
    /// 而是直接使用绑定值 (适用于 ghost 按钮等需要精确悬停色的场景)。
    pub fn bind_hover_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.hover_binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Button 悬停色绑定的状态类型不匹配");
            f(state)
        }));
        self
    }

    /// 绑定焦点环颜色：每帧从应用状态读取焦点环色。
    ///
    /// 与 [`Button::bind_color`] 同构;ghost 按钮 (透明背景) 上白色焦点环
    /// 不可见, 可用此绑定切换为 accent 等可见色。
    pub fn bind_focus_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.focus_binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Button 焦点环色绑定的状态类型不匹配");
            f(state)
        }));
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
        if let Some(binding) = &self.color_binding {
            self.color = binding(state);
        }
        if let Some(binding) = &self.hover_binding {
            self.hover_color = Some(binding(state));
        }
        if let Some(binding) = &self.focus_binding {
            self.focus_color = binding(state);
        }
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

    /// 重置焦点视觉: 清除焦点环与按压态 (面板隐藏时被容器调用)。
    fn reset_focus(&mut self) {
        self.focused = false;
        self.pressed = false;
    }

    /// 稳定焦点标识 (`.id()` 设置; 供 `App::focus_request` 按名聚焦)。
    fn focus_id(&self) -> Option<&'static str> {
        self.id
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

    /// 当前悬停色 (测试用)。
    pub(crate) fn hover_color_value(&self) -> Option<Color> {
        self.hover_color
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

    #[test]
    fn bind_color_updates_color_on_sync() {
        struct Nav {
            selected: bool,
        }
        let idle = LightTheme.accent();
        let active = Color::from_srgb8(37, 99, 235);
        let mut button = Button::new(Text::new("基础"))
            .bind_color(move |s: &Nav| if s.selected { active } else { idle });
        button.sync(&(Nav { selected: true }) as &dyn Any);
        assert_eq!(button.color_value(), active, "选中时应取绑定色");
        button.sync(&(Nav { selected: false }) as &dyn Any);
        assert_eq!(button.color_value(), idle, "未选中时应回退绑定色");
    }

    #[test]
    fn bind_hover_color_updates_on_sync() {
        struct Nav {
            selected: bool,
        }
        let hover_selected = Color::from_srgb8(12, 94, 88);
        let hover_idle = Color::from_srgb8(238, 246, 242);
        let mut button = Button::new(Text::new("基础")).bind_hover_color(move |s: &Nav| {
            if s.selected {
                hover_selected
            } else {
                hover_idle
            }
        });
        assert_eq!(button.hover_color_value(), None, "sync 前无悬停色");
        button.sync(&(Nav { selected: true }) as &dyn Any);
        assert_eq!(button.hover_color_value(), Some(hover_selected));
        button.sync(&(Nav { selected: false }) as &dyn Any);
        assert_eq!(button.hover_color_value(), Some(hover_idle));
    }

    #[test]
    fn bind_focus_color_updates_on_sync() {
        struct Nav {
            selected: bool,
        }
        let focus_selected = Color::WHITE;
        let focus_idle = LightTheme.accent();
        let mut button = Button::new(Text::new("基础")).bind_focus_color(move |s: &Nav| {
            if s.selected {
                focus_selected
            } else {
                focus_idle
            }
        });
        button.sync(&(Nav { selected: true }) as &dyn Any);
        assert_eq!(button.focus_color_value(), focus_selected);
        button.sync(&(Nav { selected: false }) as &dyn Any);
        assert_eq!(button.focus_color_value(), focus_idle);
    }
}
