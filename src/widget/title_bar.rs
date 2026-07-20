//! @author 十四叔
//! @date 2026/07/19

//! 自绘标题栏组件。
//!
//! 左侧显示窗口 LOGO 与标题,右侧提供最小化/最大化/关闭三个按钮。
//! 阶段 1 按钮产出 `WindowAction` 消息,由 `window.rs` 的 `Handler` 调用 OS 窗口 API。

use std::any::Any;
use std::time::{Duration, Instant};

use crate::event::{Event, MouseButton};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 标题栏右侧按钮。
#[derive(Debug, Default, Clone, Copy)]
struct TitleButton {
    /// 鼠标是否悬停。
    hovered: bool,
    /// 鼠标是否按下。
    pressed: bool,
}

/// 窗口动作回调工厂。
type ActionFactory = Box<dyn Fn() -> Box<dyn Any>>;

/// 自绘标题栏组件。
pub struct TitleBar {
    /// 窗口标题。
    title: String,
    /// 栏高度。
    height: f32,
    /// 按钮尺寸。
    button_size: f32,
    /// 按钮间距。
    button_gap: f32,
    /// 左右边距。
    margin: f32,
    /// LOGO 尺寸。
    logo_size: f32,
    /// LOGO 与标题间距。
    logo_gap: f32,
    /// 背景色。
    bg: Color,
    /// 标题文字颜色。
    text_color: Color,
    /// 按钮正常色。
    button_color: Color,
    /// 按钮悬停色。
    button_hover_color: Color,
    /// 关闭按钮悬停色。
    close_hover_color: Color,
    /// LOGO 外框色。
    logo_frame_color: Color,
    /// LOGO 内部填充色。
    logo_fill_color: Color,
    /// LOGO 颜料点色。
    logo_dot_color: Color,
    /// 三个按钮状态(0=关闭,1=最大化,2=最小化,从右往左)。
    buttons: [TitleButton; 3],
    /// 关闭按钮回调。
    on_close: Option<ActionFactory>,
    /// 最小化按钮回调。
    on_minimize: Option<ActionFactory>,
    /// 最大化/还原按钮回调。
    on_maximize: Option<ActionFactory>,
    /// 标题栏拖拽回调。
    on_drag: Option<ActionFactory>,
    /// 自身绝对矩形缓存。
    area: Rect,
    /// 上次在非按钮区按下左键的时间与位置,用于识别双击最大化。
    last_left_press: Option<(Instant, Point)>,
}

impl TitleBar {
    /// 创建标题栏,使用默认浅色主题。
    pub fn new(title: impl Into<String>) -> Self {
        Self::themed(&LightTheme, title)
    }

    /// 使用指定主题创建标题栏。
    pub fn themed(theme: &impl Theme, title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            height: theme.spacing_xl() + theme.spacing_lg(),
            button_size: theme.spacing_md(),
            button_gap: theme.spacing_sm(),
            margin: theme.spacing_md(),
            logo_size: theme.spacing_md(),
            logo_gap: theme.spacing_sm(),
            bg: theme.surface(),
            text_color: theme.text_primary(),
            button_color: theme.text_secondary(),
            button_hover_color: theme.text_primary(),
            close_hover_color: theme.danger(),
            logo_frame_color: theme.accent(),
            logo_fill_color: theme.surface(),
            logo_dot_color: theme.accent(),
            buttons: [TitleButton::default(); 3],
            on_close: None,
            on_minimize: None,
            on_maximize: None,
            on_drag: None,
            area: Rect::default(),
            last_left_press: None,
        }
    }

    /// 设置关闭按钮产出的消息。
    pub fn on_close<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_close = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 设置最小化按钮产出的消息。
    pub fn on_minimize<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_minimize = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 设置最大化/还原按钮产出的消息。
    pub fn on_maximize<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_maximize = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 设置标题栏拖拽时产出的消息。
    pub fn on_drag<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_drag = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 计算第 i 个按钮的矩形(0=关闭,1=最大化,2=最小化)。
    fn button_rect(&self, area: Rect, index: usize) -> Rect {
        let right = area.origin.x + area.size.width - self.margin;
        let x = right - (index as f32 + 1.0) * self.button_size - index as f32 * self.button_gap;
        let y = area.origin.y + (self.height - self.button_size) / 2.0;
        Rect::from_xywh(x, y, self.button_size, self.button_size)
    }

    /// 计算 LOGO 矩形。
    fn logo_rect(&self, area: Rect) -> Rect {
        let y = area.origin.y + (self.height - self.logo_size) / 2.0;
        Rect::from_xywh(
            area.origin.x + self.margin,
            y,
            self.logo_size,
            self.logo_size,
        )
    }

    /// 返回鼠标位置命中的按钮索引,无命中返回 `None`。
    fn hit_button(&self, area: Rect, position: Point) -> Option<usize> {
        (0..self.buttons.len()).find(|i| self.button_rect(area, *i).contains(position))
    }

    /// 第 i 个按钮的图形符号(0=关闭,1=最大化,2=最小化)。
    fn button_symbol(index: usize) -> &'static str {
        match index {
            0 => "×",
            1 => "□",
            2 => "_",
            _ => "",
        }
    }

    /// 第 i 个按钮当前应绘制的颜色。
    fn button_color(&self, index: usize) -> Color {
        let base = if self.buttons[index].hovered {
            if index == 0 {
                self.close_hover_color
            } else {
                self.button_hover_color
            }
        } else {
            self.button_color
        };
        if self.buttons[index].pressed {
            Color::rgba(base.r * 0.8, base.g * 0.8, base.b * 0.8, base.a)
        } else {
            base
        }
    }

    /// 触发指定索引按钮的回调。
    fn emit_button_action(&self, index: usize, msgs: &mut MsgQueue) {
        let factory = match index {
            0 => &self.on_close,
            1 => &self.on_maximize,
            2 => &self.on_minimize,
            _ => &None,
        };
        if let Some(factory) = factory {
            msgs.push(factory());
        }
    }

    /// 尝试触发拖拽或识别双击最大化。
    fn handle_drag_or_double_click(&mut self, position: Point, msgs: &mut MsgQueue) {
        const DOUBLE_CLICK_INTERVAL: Duration = Duration::from_millis(300);
        const DOUBLE_CLICK_DISTANCE: f32 = 4.0;

        if let Some((last_time, last_pos)) = self.last_left_press {
            let dt = Instant::now().duration_since(last_time);
            let dist = Point::new(position.x - last_pos.x, position.y - last_pos.y);
            if dt < DOUBLE_CLICK_INTERVAL
                && dist.x.abs() < DOUBLE_CLICK_DISTANCE
                && dist.y.abs() < DOUBLE_CLICK_DISTANCE
            {
                // 双击:最大化/还原
                if let Some(factory) = &self.on_maximize {
                    msgs.push(factory());
                }
                self.last_left_press = None;
                return;
            }
        }

        // 单击开始拖拽
        if let Some(factory) = &self.on_drag {
            msgs.push(factory());
        }
        self.last_left_press = Some((Instant::now(), position));
    }
}

impl Widget for TitleBar {
    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        let size = constraints.constrain(Size::new(constraints.max_width, self.height));
        self.area = Rect::new(crate::Point::ZERO, size);
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        // 背景条。
        rects.push_rect(area, self.bg, 0.0);

        // LOGO:玻璃画布 + 颜料滴。
        let logo_rect = self.logo_rect(area);
        let logo_size = logo_rect.size.width;

        // 外框：accent 描边效果的圆角矩形。
        let frame_radius = logo_size * 0.25;
        rects.push_rect(logo_rect, self.logo_frame_color, frame_radius);

        // 内部填充：白色半透明，形成“环+面”。
        let stroke = logo_size * 0.10;
        let fill_rect = logo_rect.inset(stroke);
        let fill_radius = (frame_radius - stroke).max(0.0);
        rects.push_rect(fill_rect, self.logo_fill_color, fill_radius);

        // 颜料滴：实心 accent 圆，偏右下。
        let dot_size = logo_size * 0.30;
        let dot_offset = logo_size * 0.58;
        let dot_rect = Rect::from_xywh(
            logo_rect.origin.x + dot_offset - dot_size / 2.0,
            logo_rect.origin.y + dot_offset - dot_size / 2.0,
            dot_size,
            dot_size,
        );
        rects.push_rect(dot_rect, self.logo_dot_color, dot_size / 2.0);

        // 标题文字,垂直居中。
        let font_size = LightTheme.font_size_body();
        let baseline =
            area.origin.y + area.size.height / 2.0 + texts.ascent(f32::from(font_size)) / 2.0;
        texts.push_text(
            &self.title,
            logo_rect.origin.x + logo_rect.size.width + self.logo_gap,
            baseline,
            font_size,
            self.text_color,
        );

        // 三个按钮:圆形背景 + 图形符号。
        for i in 0..self.buttons.len() {
            let rect = self.button_rect(area, i);
            let radius = self.button_size / 2.0;
            rects.push_rect(rect, self.button_color(i), radius);

            let symbol = Self::button_symbol(i);
            let symbol_size = (self.button_size * 0.55) as u16;
            let symbol_width = texts.measure(symbol, symbol_size);
            let symbol_baseline =
                rect.origin.y + rect.size.height / 2.0 + texts.ascent(f32::from(symbol_size)) / 2.0;
            texts.push_text(
                symbol,
                rect.origin.x + (rect.size.width - symbol_width) / 2.0,
                symbol_baseline,
                symbol_size,
                self.bg,
            );
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.area = area;
        match event {
            Event::CursorMoved(p) => {
                let hit = self.hit_button(area, *p);
                for (i, btn) in self.buttons.iter_mut().enumerate() {
                    btn.hovered = hit == Some(i);
                }
                if hit.is_some() {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::CursorLeft => {
                for btn in &mut self.buttons {
                    btn.hovered = false;
                    btn.pressed = false;
                }
                self.last_left_press = None;
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position,
            } => {
                let hit = self.hit_button(area, *position);
                if let Some(idx) = hit {
                    for (i, btn) in self.buttons.iter_mut().enumerate() {
                        btn.pressed = i == idx;
                    }
                    EventResult::Consumed
                } else {
                    // 非按钮区:拖拽或双击最大化
                    self.handle_drag_or_double_click(*position, msgs);
                    EventResult::Consumed
                }
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position,
            } => {
                let hit = self.hit_button(area, *position);
                let mut triggered = [false; 3];
                for (i, btn) in self.buttons.iter_mut().enumerate() {
                    if btn.pressed && hit == Some(i) {
                        triggered[i] = true;
                    }
                    btn.pressed = false;
                }
                for (i, was_triggered) in triggered.into_iter().enumerate() {
                    if was_triggered {
                        self.emit_button_action(i, msgs);
                    }
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area)
    }
}

#[cfg(test)]
impl TitleBar {
    /// 指定按钮是否悬停(测试用,0=关闭,1=最大化,2=最小化)。
    pub(crate) fn button_hovered(&self, index: usize) -> bool {
        self.buttons[index].hovered
    }

    /// 指定按钮是否按下(测试用)。
    pub(crate) fn button_pressed(&self, index: usize) -> bool {
        self.buttons[index].pressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::WindowAction;

    fn title_bar_area() -> Rect {
        Rect::from_xywh(0.0, 0.0, 400.0, 40.0)
    }

    #[test]
    fn title_bar_uses_theme_defaults() {
        let bar = TitleBar::new("丹青");
        assert_eq!(
            bar.height,
            LightTheme.spacing_xl() + LightTheme.spacing_lg()
        );
        assert_eq!(bar.button_size, LightTheme.spacing_md());
        assert_eq!(bar.bg, LightTheme.surface());
        assert_eq!(bar.logo_frame_color, LightTheme.accent());
        assert_eq!(bar.logo_fill_color, LightTheme.surface());
        assert_eq!(bar.logo_dot_color, LightTheme.accent());
    }

    #[test]
    fn cursor_over_close_button_hovers_only_close() {
        let mut bar = TitleBar::new("丹青");
        let area = title_bar_area();
        let origin = bar.button_rect(area, 0).origin;
        let close_center = crate::Point::new(
            origin.x + bar.button_size / 2.0,
            origin.y + bar.button_size / 2.0,
        );
        let mut msgs = MsgQueue::new();

        bar.event(&Event::CursorMoved(close_center), area, &mut msgs);

        assert!(bar.button_hovered(0));
        assert!(!bar.button_hovered(1));
        assert!(!bar.button_hovered(2));
    }

    #[test]
    fn mouse_press_on_button_sets_pressed() {
        let mut bar = TitleBar::new("丹青");
        let area = title_bar_area();
        let origin = bar.button_rect(area, 0).origin;
        let close_center = crate::Point::new(
            origin.x + bar.button_size / 2.0,
            origin.y + bar.button_size / 2.0,
        );
        let mut msgs = MsgQueue::new();

        bar.event(&Event::CursorMoved(close_center), area, &mut msgs);
        bar.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: close_center,
            },
            area,
            &mut msgs,
        );

        assert!(bar.button_pressed(0));
        assert!(!bar.button_pressed(1));
        assert!(!bar.button_pressed(2));
    }

    #[test]
    fn cursor_left_clears_hover_and_pressed() {
        let mut bar = TitleBar::new("丹青");
        let area = title_bar_area();
        let origin = bar.button_rect(area, 0).origin;
        let close_center = crate::Point::new(
            origin.x + bar.button_size / 2.0,
            origin.y + bar.button_size / 2.0,
        );
        let mut msgs = MsgQueue::new();

        bar.event(&Event::CursorMoved(close_center), area, &mut msgs);
        bar.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: close_center,
            },
            area,
            &mut msgs,
        );
        bar.event(&Event::CursorLeft, area, &mut msgs);

        assert!(!bar.button_hovered(0));
        assert!(!bar.button_pressed(0));
    }

    #[test]
    fn button_outside_area_is_ignored() {
        let mut bar = TitleBar::new("丹青");
        let area = title_bar_area();
        let mut msgs = MsgQueue::new();

        let result = bar.event(
            &Event::CursorMoved(crate::Point::new(10.0, 10.0)),
            area,
            &mut msgs,
        );

        assert_eq!(result, EventResult::Ignored);
        assert!(!bar.button_hovered(0));
    }

    #[test]
    fn close_button_emits_message_on_click() {
        let mut bar = TitleBar::new("丹青").on_close(|| WindowAction::Close);
        let area = title_bar_area();
        let origin = bar.button_rect(area, 0).origin;
        let close_center = crate::Point::new(
            origin.x + bar.button_size / 2.0,
            origin.y + bar.button_size / 2.0,
        );
        let mut msgs = MsgQueue::new();

        bar.event(&Event::CursorMoved(close_center), area, &mut msgs);
        bar.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: close_center,
            },
            area,
            &mut msgs,
        );
        bar.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: close_center,
            },
            area,
            &mut msgs,
        );

        assert_eq!(msgs.len(), 1);
        let action = msgs[0].downcast_ref::<WindowAction>().unwrap();
        assert_eq!(*action, WindowAction::Close);
    }

    #[test]
    fn drag_area_emits_drag_message() {
        let mut bar = TitleBar::new("丹青").on_drag(|| WindowAction::Drag);
        let area = title_bar_area();
        let mut msgs = MsgQueue::new();

        bar.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: crate::Point::new(50.0, 20.0),
            },
            area,
            &mut msgs,
        );

        assert_eq!(msgs.len(), 1);
        let action = msgs[0].downcast_ref::<WindowAction>().unwrap();
        assert_eq!(*action, WindowAction::Drag);
    }
}
