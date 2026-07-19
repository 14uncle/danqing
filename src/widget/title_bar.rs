//! @author 十四叔
//! @date 2026/07/19

//! 自绘标题栏组件。
//!
//! 左侧显示窗口标题,右侧提供最小化/最大化/关闭三个按钮的视觉反馈。
//! 阶段 1 不调用窗口控制 API,按钮仅产生悬停/按下状态变化。

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
    /// 三个按钮状态(0=关闭,1=最大化,2=最小化,从右往左)。
    buttons: [TitleButton; 3],
    /// 自身绝对矩形缓存。
    area: Rect,
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
            bg: theme.surface(),
            text_color: theme.text_primary(),
            button_color: theme.text_secondary(),
            button_hover_color: theme.text_primary(),
            close_hover_color: theme.danger(),
            buttons: [TitleButton::default(); 3],
            area: Rect::default(),
        }
    }

    /// 计算第 i 个按钮的矩形(0=关闭,1=最大化,2=最小化)。
    fn button_rect(&self, area: Rect, index: usize) -> Rect {
        let right = area.origin.x + area.size.width - self.margin;
        let x = right - (index as f32 + 1.0) * self.button_size - index as f32 * self.button_gap;
        let y = area.origin.y + (self.height - self.button_size) / 2.0;
        Rect::from_xywh(x, y, self.button_size, self.button_size)
    }

    /// 返回鼠标位置命中的按钮索引,无命中返回 `None`。
    fn hit_button(&self, area: Rect, position: Point) -> Option<usize> {
        (0..self.buttons.len()).find(|i| self.button_rect(area, *i).contains(position))
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

        // 标题文字,垂直居中。
        let font_size = 14;
        let baseline =
            area.origin.y + area.size.height / 2.0 + texts.ascent(f32::from(font_size)) / 2.0;
        texts.push_text(
            &self.title,
            area.origin.x + self.margin,
            baseline,
            font_size,
            self.text_color,
        );

        // 三个按钮绘制成圆形。
        for i in 0..self.buttons.len() {
            let rect = self.button_rect(area, i);
            let radius = self.button_size / 2.0;
            rects.push_rect(rect, self.button_color(i), radius);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, _msgs: &mut MsgQueue) -> EventResult {
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
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed,
                position,
            } => {
                let hit = self.hit_button(area, *position);
                if *pressed {
                    for (i, btn) in self.buttons.iter_mut().enumerate() {
                        btn.pressed = hit == Some(i) && btn.hovered;
                    }
                } else {
                    for btn in &mut self.buttons {
                        btn.pressed = false;
                    }
                }
                if hit.is_some() {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
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
}
