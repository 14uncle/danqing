//! @author 十四叔
//! @date 2026/08/25
//!
//! 图标输入框: 左侧文本输入 + 右侧可点击图标按钮。
//!
//! 复合组件: 内部持有 [`TextInput`] (chromeless) + 矢量图标按钮,
//! 共享背景与边框绘制。图标点击产出独立消息, 适用于「搜索框 + 放大镜」
//! 「路径输入 + 文件选择」等场景。

use std::any::Any;
use std::cell::Cell;

use crate::event::{Event, MouseButton};
use crate::render::{RectBatch, TextBatch};
use crate::widget::form::TextInput;
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, Edges, LightTheme, Point, Rect, Size, Theme};

/// 消息工厂: 图标点击时产出一条应用消息。
type MsgFactory = Box<dyn Fn() -> Box<dyn Any>>;
/// 颜色绑定闭包: 从类型擦除的应用状态产出颜色。
type ColorBinding = Box<dyn Fn(&dyn Any) -> Color>;
/// 文本绑定闭包: 从类型擦除的应用状态产出文本。
type TextBinding = Box<dyn Fn(&dyn Any) -> String>;

/// 图标按钮区域宽度 (逻辑像素)。
const ICON_AREA_WIDTH: f32 = 32.0;
/// 放大镜圆圈半径 (相对图标区域中心)。
const MAG_RADIUS: f32 = 5.0;
/// 放大镜手柄粗细。
const MAG_THICKNESS: f32 = 2.0;

/// 图标输入框: 左侧文本输入 + 右侧可点击图标。
///
/// 文本输入复用 [`TextInput`] (chromeless 模式), 外框由本组件统一绘制。
/// 图标通过矢量几何绘制 (放大镜: 圆圈 + 斜线), 点击产出独立消息。
pub struct IconInput {
    /// 内部文本输入框 (chromeless)。
    input: TextInput,
    /// 是否获得焦点 (来自内部 TextInput)。
    focused: bool,
    /// 图标区域是否悬停。
    icon_hovered: bool,
    /// 图标区域是否按下。
    icon_pressed: bool,
    /// 图标点击时产出的消息工厂。
    on_icon_click: Option<MsgFactory>,
    /// 背景色。
    background: Color,
    /// 边框颜色。
    border_color: Color,
    /// 获得焦点时的边框颜色。
    focus_border_color: Color,
    /// 边框粗细。
    border_width: f32,
    /// 背景圆角半径。
    radius: f32,
    /// 图标颜色绑定。
    icon_color_binding: Option<ColorBinding>,
    /// 最近一帧同步的图标颜色。
    icon_color: Color,
    /// 图标悬停背景色绑定。
    icon_hover_binding: Option<ColorBinding>,
    /// 最近一帧同步的图标悬停背景色。
    icon_hover_bg: Color,
    /// 文本绑定: 从应用状态读取文本, 与内部 TextInput 同步。
    text_binding: Option<TextBinding>,
    /// 显式宽度 (未指定则按约束上限)。
    width: Option<f32>,
    /// layout 缓存: 自身绝对矩形。
    area: Cell<Rect>,
    /// layout 缓存: 图标区域绝对矩形。
    icon_area: Cell<Rect>,
}

impl IconInput {
    /// 创建图标输入框，使用默认浅色主题 token。
    pub fn new() -> Self {
        Self::themed(&LightTheme)
    }

    /// 使用指定主题创建图标输入框。
    pub fn themed(theme: &impl Theme) -> Self {
        Self {
            input: TextInput::themed(theme).chromeless(),
            focused: false,
            icon_hovered: false,
            icon_pressed: false,
            on_icon_click: None,
            background: theme.surface_input(),
            border_color: theme.border(),
            focus_border_color: theme.accent(),
            border_width: 1.0,
            radius: theme.radius_sm(),
            icon_color_binding: None,
            icon_color: Color::rgb(0.5, 0.5, 0.5),
            icon_hover_binding: None,
            icon_hover_bg: Color::TRANSPARENT,
            text_binding: None,
            width: None,
            area: Cell::new(Rect::default()),
            icon_area: Cell::new(Rect::default()),
        }
    }

    /// 设置显式宽度。
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// 设置字号。
    pub fn font_size(mut self, size: u16) -> Self {
        self.input = self.input.font_size(size);
        self
    }

    /// 设置占位文字与颜色。
    pub fn placeholder(mut self, text: impl Into<String>, color: Color) -> Self {
        self.input = self.input.placeholder(text, color);
        self
    }

    /// 设置文本变化回调。
    pub fn on_change<M: 'static>(mut self, f: impl Fn(&str) -> M + 'static) -> Self {
        self.input = self.input.on_change(f);
        self
    }

    /// 设置图标点击时产出的消息。
    pub fn on_icon_click<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_icon_click = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 绑定图标颜色: 每帧从应用状态读取。
    pub fn bind_icon_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.icon_color_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("IconInput 图标色绑定的状态类型不匹配"))
        }));
        self
    }

    /// 绑定图标悬停背景色。
    pub fn bind_icon_hover_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.icon_hover_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("IconInput 悬停色绑定的状态类型不匹配"))
        }));
        self
    }

    /// 设置背景色。
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// 设置边框颜色。
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// 设置焦点边框颜色。
    pub fn focus_border_color(mut self, color: Color) -> Self {
        self.focus_border_color = color;
        self
    }

    /// 设置圆角半径。
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// 设置内边距。
    pub fn padding(mut self, padding: Edges) -> Self {
        self.input = self.input.padding(padding);
        self
    }

    /// 清空文本输入。
    pub fn clear(&mut self) {
        self.input.clear();
    }

    /// 设置文本内容。
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.input.set_text(text);
    }

    /// 绑定文本: 每帧从应用状态读取文本, 与内部 TextInput 同步。
    ///
    /// 用于外部设置文本 (如文件对话框选中路径后回写)。
    /// 仅在应用状态文本与内部文本不同时更新, 避免覆盖用户输入。
    pub fn bind_text<S: 'static>(mut self, f: impl Fn(&S) -> String + 'static) -> Self {
        self.text_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("IconInput 文本绑定的状态类型不匹配"))
        }));
        self
    }

    /// 当前文本值。
    pub fn value(&self) -> &str {
        self.input.value()
    }

    /// 绘制放大镜图标 (圆圈 + 斜线手柄)。
    fn paint_magnifier(area: Rect, rects: &mut RectBatch, color: Color) {
        let cx = area.origin.x + area.size.width / 2.0;
        let cy = area.origin.y + area.size.height / 2.0;
        // 圆圈: 用圆角矩形近似
        let circle_rect = Rect::from_xywh(
            cx - MAG_RADIUS,
            cy - MAG_RADIUS,
            MAG_RADIUS * 2.0,
            MAG_RADIUS * 2.0,
        );
        rects.push_rounded_border(circle_rect, color, MAG_RADIUS, MAG_THICKNESS);
        // 手柄: 右下斜线
        let handle_start = Point::new(cx + MAG_RADIUS * 0.6, cy + MAG_RADIUS * 0.6);
        let handle_end = Point::new(cx + MAG_RADIUS * 1.5, cy + MAG_RADIUS * 1.5);
        crate::widget::push_diagonal(rects, handle_start, handle_end, MAG_THICKNESS, color);
    }
}

impl Default for IconInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for IconInput {
    fn sync(&mut self, state: &dyn Any) {
        self.input.sync(state);
        self.focused = self.input.is_focused();
        if let Some(bind) = &self.icon_color_binding {
            self.icon_color = bind(state);
        }
        if let Some(bind) = &self.icon_hover_binding {
            self.icon_hover_bg = bind(state);
        }
        // 文本绑定: 应用状态文本与内部 TextInput 同步
        if let Some(bind) = &self.text_binding {
            let external = bind(state);
            if self.input.value() != external {
                self.input.set_text(external);
            }
        }
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        self.input.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        let total_width = self
            .width
            .unwrap_or(constraints.max_width)
            .max(ICON_AREA_WIDTH + 40.0);
        let input_constraints = Constraints::loose(Size::new(
            total_width - ICON_AREA_WIDTH,
            constraints.max_height,
        ));
        let input_size = self.input.layout(input_constraints, texts);
        let height = input_size.height;
        let size = constraints.constrain(Size::new(total_width, height));
        self.area.set(Rect::new(Point::ZERO, size));
        // 设置图标区域缓存
        self.icon_area.set(Rect::from_xywh(
            size.width - ICON_AREA_WIDTH,
            0.0,
            ICON_AREA_WIDTH,
            size.height,
        ));
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        self.area.set(area);

        // 分割区域: 左侧输入框, 右侧图标
        let input_rect = Rect::from_xywh(
            area.origin.x,
            area.origin.y,
            area.size.width - ICON_AREA_WIDTH,
            area.size.height,
        );
        let icon_rect = Rect::from_xywh(
            area.origin.x + area.size.width - ICON_AREA_WIDTH,
            area.origin.y,
            ICON_AREA_WIDTH,
            area.size.height,
        );
        self.icon_area.set(icon_rect);

        // 共享背景
        rects.push_rect(area, self.background, self.radius);

        // 边框: 聚焦时使用 accent
        let border_color = if self.focused {
            self.focus_border_color
        } else {
            self.border_color
        };
        rects.push_rounded_border(area, border_color, self.radius, self.border_width);

        // 分割线
        let divider_x = area.origin.x + area.size.width - ICON_AREA_WIDTH;
        rects.push_rect(
            Rect::from_xywh(divider_x, area.origin.y + 4.0, 1.0, area.size.height - 8.0),
            border_color,
            0.0,
        );

        // 图标悬停背景
        if self.icon_hovered {
            let icon_bg = Rect::from_xywh(
                icon_rect.origin.x + 1.0,
                icon_rect.origin.y + 1.0,
                icon_rect.size.width - 2.0,
                icon_rect.size.height - 2.0,
            );
            rects.push_rect(icon_bg, self.icon_hover_bg, self.radius);
        }

        // 放大镜图标
        let scale = if self.icon_pressed { 0.7 } else { 1.0 };
        let color = Color::rgba(
            self.icon_color.r * scale,
            self.icon_color.g * scale,
            self.icon_color.b * scale,
            self.icon_color.a,
        );
        Self::paint_magnifier(icon_rect, rects, color);

        // 内部文本输入 (chromeless, 只画文本/光标/选区)
        self.input.paint(input_rect, rects, texts);
    }

    fn event(&mut self, event: &Event, _area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let area = self.area.get();
        let icon_rect = self.icon_area.get();

        match event {
            Event::CursorMoved(p) => {
                let was_icon_hovered = self.icon_hovered;
                self.icon_hovered = icon_rect.contains(*p);
                // 图标区域事件优先
                if self.icon_hovered {
                    self.input.event(event, area, msgs); // 让 input 失去悬停态
                    return EventResult::Consumed;
                }
                // 输入框区域
                let input_rect = Rect::from_xywh(
                    area.origin.x,
                    area.origin.y,
                    area.size.width - ICON_AREA_WIDTH,
                    area.size.height,
                );
                if input_rect.contains(*p) {
                    return self.input.event(event, input_rect, msgs);
                }
                if was_icon_hovered {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::CursorLeft => {
                self.icon_hovered = false;
                self.icon_pressed = false;
                self.input.event(event, area, msgs);
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position,
            } => {
                if icon_rect.contains(*position) {
                    self.icon_pressed = true;
                    return EventResult::Consumed;
                }
                let input_rect = Rect::from_xywh(
                    area.origin.x,
                    area.origin.y,
                    area.size.width - ICON_AREA_WIDTH,
                    area.size.height,
                );
                if input_rect.contains(*position) {
                    return self.input.event(event, input_rect, msgs);
                }
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position,
            } => {
                if self.icon_pressed && icon_rect.contains(*position) {
                    self.icon_pressed = false;
                    if let Some(factory) = &self.on_icon_click {
                        msgs.push(factory());
                    }
                    return EventResult::Consumed;
                }
                self.icon_pressed = false;
                let input_rect = Rect::from_xywh(
                    area.origin.x,
                    area.origin.y,
                    area.size.width - ICON_AREA_WIDTH,
                    area.size.height,
                );
                self.input.event(event, input_rect, msgs);
                EventResult::Ignored
            }
            Event::FocusIn => self.input.event(event, area, msgs),
            Event::FocusOut => self.input.event(event, area, msgs),
            _ => self.input.event(event, area, msgs),
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    fn reset_focus(&mut self) {
        self.input.reset_focus();
        self.focused = false;
        self.icon_pressed = false;
    }

    fn children(&self) -> &[crate::widget::Node] {
        &[]
    }

    fn selected_text(&self) -> Option<String> {
        self.input.selected_text()
    }

    fn wants_ime(&self) -> bool {
        self.input.wants_ime()
    }

    fn ime_area(&self) -> Option<Rect> {
        self.input.ime_area()
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Key;
    use crate::render::TextBatch;
    use crate::widget::MsgQueue;

    fn icon_input_area() -> Rect {
        Rect::from_xywh(0.0, 0.0, 200.0, 32.0)
    }

    #[test]
    fn layout_respects_width() {
        let mut icon_input = IconInput::new().width(200.0);
        let mut texts = TextBatch::new();
        let size = icon_input.layout(Constraints::loose(Size::new(400.0, 400.0)), &mut texts);
        assert_eq!(size.width, 200.0, "应使用指定宽度");
    }

    #[test]
    fn layout_height_equals_control_height() {
        let mut icon_input = IconInput::new().width(200.0);
        let mut texts = TextBatch::new();
        let size = icon_input.layout(Constraints::loose(Size::new(400.0, 400.0)), &mut texts);
        assert!(
            (size.height - LightTheme.control_height()).abs() < 0.01,
            "IconInput 高度应精确等于 control_height {}, 实际 {}",
            LightTheme.control_height(),
            size.height
        );
    }

    #[test]
    fn icon_click_emits_message() {
        let mut icon_input = IconInput::new().width(200.0).on_icon_click(|| 42u8);
        let mut msgs = MsgQueue::new();
        icon_input.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = icon_input_area();
        // 点击图标区域 (右侧 32px)
        let icon_point = Point::new(184.0, 16.0);
        icon_input.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: icon_point,
            },
            area,
            &mut msgs,
        );
        icon_input.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: icon_point,
            },
            area,
            &mut msgs,
        );
        assert_eq!(msgs.len(), 1, "图标点击应产出消息");
        assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&42));
    }

    #[test]
    fn text_input_still_works() {
        let mut icon_input = IconInput::new()
            .width(200.0)
            .on_change(|t: &str| t.to_string());
        let mut msgs = MsgQueue::new();
        icon_input.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = icon_input_area();
        // 点击输入框区域
        let input_point = Point::new(50.0, 16.0);
        icon_input.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: input_point,
            },
            area,
            &mut msgs,
        );
        icon_input.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: input_point,
            },
            area,
            &mut msgs,
        );
        // 输入字符
        icon_input.event(
            &Event::Key {
                key: Key::Character("a".into()),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            area,
            &mut msgs,
        );
        assert_eq!(icon_input.value(), "a", "文本输入应正常工作");
        assert!(!msgs.is_empty(), "应产生 on_change 消息");
    }
}
