//! @author 十四叔
//! @date 2026/08/16

//! 关闭按钮: 固定 24×24 的 × 符号, 纯矢量绘制 (文字不参与)。
//!
//! 悬停出圆角背景高亮, 按下符号变暗; 点击 (原地按下抬起)
//! 或聚焦按回车/空格产出应用消息。提炼自 danqing-pomodoro
//! 的私有组件 (第二个使用者: 剪贴板设置面板)。

use std::any::Any;

use crate::event::{Event, Key, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, Widget, push_diagonal};
use crate::{Color, Constraints, Point, Rect, Size};

/// 颜色绑定闭包: 从类型擦除的应用状态产出颜色。
type ColorBinding = Box<dyn Fn(&dyn Any) -> Color>;
/// 消息工厂: 点击时产出一条应用消息。
type MsgFactory = Box<dyn Fn() -> Box<dyn Any>>;

/// 关闭按钮: 固定尺寸的 × 符号, 矢量绘制 (对角线小圆点队列,
/// 与 TitleBar 关闭按钮同算法, 见 [`push_diagonal`])。
pub struct CloseButton {
    /// 是否悬停。
    hovered: bool,
    /// 是否按下。
    pressed: bool,
    /// 点击时产出的消息工厂。
    on_click: Option<MsgFactory>,
    /// 符号颜色绑定: 每帧从应用状态读取。
    color_binding: Option<ColorBinding>,
    /// 最近一帧同步的符号颜色。
    symbol_color: Color,
    /// 悬停背景色绑定。
    hover_binding: Option<ColorBinding>,
    /// 最近一帧同步的悬停背景色。
    hover_bg: Color,
}

impl CloseButton {
    /// 创建关闭按钮 (默认符号色中灰, 无回调)。
    pub fn new() -> Self {
        Self {
            hovered: false,
            pressed: false,
            on_click: None,
            color_binding: None,
            symbol_color: Color::rgb(0.5, 0.5, 0.5),
            hover_binding: None,
            hover_bg: Color::TRANSPARENT,
        }
    }

    /// 设置点击时产出的消息。
    pub fn on_click<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_click = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 绑定符号颜色: 每帧从应用状态读取。
    pub fn bind_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.color_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("CloseButton 符号色绑定的状态类型不匹配"))
        }));
        self
    }

    /// 绑定悬停背景色: 每帧从应用状态读取。
    pub fn bind_hover_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.hover_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("CloseButton 悬停色绑定的状态类型不匹配"))
        }));
        self
    }

    /// 绘制两条对角线 × (基于给定区域)。
    fn paint_x(area: Rect, rects: &mut RectBatch, color: Color) {
        let inset = 0.3;
        let thickness = area.size.width.min(area.size.height) * 0.085;

        let left = area.origin.x + area.size.width * inset;
        let right = area.origin.x + area.size.width * (1.0 - inset);
        let top = area.origin.y + area.size.height * inset;
        let bottom = area.origin.y + area.size.height * (1.0 - inset);

        push_diagonal(
            rects,
            Point::new(left, top),
            Point::new(right, bottom),
            thickness,
            color,
        );
        push_diagonal(
            rects,
            Point::new(right, top),
            Point::new(left, bottom),
            thickness,
            color,
        );
    }
}

impl Default for CloseButton {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for CloseButton {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(bind) = &self.color_binding {
            self.symbol_color = bind(state);
        }
        if let Some(bind) = &self.hover_binding {
            self.hover_bg = bind(state);
        }
    }

    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        // 固定 24×24 逻辑像素
        constraints.constrain(Size::new(24.0, 24.0))
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, _texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        if self.hovered {
            rects.push_rect(area, self.hover_bg, 4.0);
        }
        let scale = if self.pressed { 0.7 } else { 1.0 };
        let color = Color::rgba(
            self.symbol_color.r * scale,
            self.symbol_color.g * scale,
            self.symbol_color.b * scale,
            self.symbol_color.a,
        );
        Self::paint_x(area, rects, color);
    }

    fn event(
        &mut self,
        event: &Event,
        area: Rect,
        msgs: &mut crate::widget::MsgQueue,
    ) -> EventResult {
        match event {
            Event::CursorMoved(p) => {
                self.hovered = area.contains(*p);
                if self.hovered {
                    EventResult::Consumed
                } else {
                    self.pressed = false;
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
            _ => EventResult::Ignored,
        }
    }

    fn focusable(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::MouseButton;
    use crate::widget::MsgQueue;

    /// 按钮摆放区域 (24×24 于原点)。
    fn btn_area() -> Rect {
        Rect::from_xywh(0.0, 0.0, 24.0, 24.0)
    }

    fn press(area: Rect, p: Point, msgs: &mut MsgQueue, btn: &mut CloseButton) {
        btn.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: p,
            },
            area,
            msgs,
        );
    }

    fn release(area: Rect, p: Point, msgs: &mut MsgQueue, btn: &mut CloseButton) {
        btn.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: p,
            },
            area,
            msgs,
        );
    }

    #[test]
    fn layout_fixed_24x24() {
        let mut btn = CloseButton::new();
        let mut texts = TextBatch::new();
        let size = btn.layout(Constraints::loose(Size::new(480.0, 480.0)), &mut texts);
        assert_eq!(size, Size::new(24.0, 24.0), "固定 24×24 逻辑像素");
    }

    #[test]
    fn click_inside_emits_message() {
        let mut btn = CloseButton::new().on_click(|| 42u8);
        let mut msgs = MsgQueue::new();
        let inside = Point::new(12.0, 12.0);
        press(btn_area(), inside, &mut msgs, &mut btn);
        release(btn_area(), inside, &mut msgs, &mut btn);
        assert_eq!(msgs.len(), 1, "原地按下抬起应产出消息");
        assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&42));
    }

    #[test]
    fn press_inside_release_outside_no_message() {
        let mut btn = CloseButton::new().on_click(|| 42u8);
        let mut msgs = MsgQueue::new();
        press(btn_area(), Point::new(12.0, 12.0), &mut msgs, &mut btn);
        release(btn_area(), Point::new(100.0, 100.0), &mut msgs, &mut btn);
        assert!(msgs.is_empty(), "拖出按钮抬起不应触发");
    }

    #[test]
    fn hover_consumes_cursor_moved() {
        let mut btn = CloseButton::new();
        let mut msgs = MsgQueue::new();
        let r = btn.event(
            &Event::CursorMoved(Point::new(12.0, 12.0)),
            btn_area(),
            &mut msgs,
        );
        assert_eq!(r, EventResult::Consumed, "悬停于按钮上应消费事件");
        let r = btn.event(
            &Event::CursorMoved(Point::new(100.0, 100.0)),
            btn_area(),
            &mut msgs,
        );
        assert_eq!(r, EventResult::Ignored, "按钮外不消费");
    }

    #[test]
    fn cursor_left_resets_state() {
        let mut btn = CloseButton::new().on_click(|| 42u8);
        let mut msgs = MsgQueue::new();
        press(btn_area(), Point::new(12.0, 12.0), &mut msgs, &mut btn);
        btn.event(&Event::CursorLeft, btn_area(), &mut msgs);
        // 离窗后原地抬起不触发 (pressed 已复位)
        release(btn_area(), Point::new(12.0, 12.0), &mut msgs, &mut btn);
        assert!(msgs.is_empty(), "离窗应复位按下态");
    }

    #[test]
    fn enter_key_emits_message() {
        let mut btn = CloseButton::new().on_click(|| 42u8);
        let mut msgs = MsgQueue::new();
        btn.event(
            &Event::Key {
                key: Key::Named(NamedKey::Enter),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            btn_area(),
            &mut msgs,
        );
        assert_eq!(msgs.len(), 1, "聚焦按回车应产出消息");
    }

    #[test]
    fn bind_color_syncs_from_state() {
        struct S {
            dark: bool,
        }
        let mut btn = CloseButton::new()
            .bind_color(|s: &S| if s.dark { Color::BLACK } else { Color::WHITE })
            .bind_hover_color(|_s: &S| Color::rgb(0.9, 0.9, 0.9));
        btn.sync(&(S { dark: true }) as &dyn Any);
        assert_eq!(btn.symbol_color, Color::BLACK);
        assert_eq!(btn.hover_bg, Color::rgb(0.9, 0.9, 0.9));
        btn.sync(&(S { dark: false }) as &dyn Any);
        assert_eq!(btn.symbol_color, Color::WHITE);
    }
}
