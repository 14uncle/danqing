// ! @author 十四叔
// ! @date 2026/07/19

// ! 自绘标题栏组件。
// !
// ! 左侧显示窗口 LOGO 与标题, 右侧提供最小化 / 最大化 / 关闭三个按钮。
// ! 阶段 1 按钮产出 `WindowAction` 消息, 由 `window.rs` 的 `Handler` 调用 OS 窗口 API。

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
    /// 按钮背景悬停 / 按下色。
    button_bg_color: Color,
    /// 窗口右上角圆角半径,关闭按钮 hover 背景右上角使用。
    window_corner_radius: f32,
    /// LOGO 外框色。
    logo_frame_color: Color,
    /// LOGO 内部填充色。
    logo_fill_color: Color,
    /// LOGO 颜料点色。
    logo_dot_color: Color,
    /// 三个按钮状态 (0= 关闭,1= 最大化,2= 最小化, 从右往左)。
    buttons: [TitleButton; 3],
    /// 关闭按钮回调。
    on_close: Option<ActionFactory>,
    /// 最小化按钮回调。
    on_minimize: Option<ActionFactory>,
    /// 最大化 / 还原按钮回调。
    on_maximize: Option<ActionFactory>,
    /// 标题栏拖拽回调。
    on_drag: Option<ActionFactory>,
    /// 自身绝对矩形缓存。
    area: Rect,
    /// 上次在非按钮区按下左键的时间与位置, 用于识别双击最大化。
    last_left_press: Option<(Instant, Point)>,
}

impl TitleBar {
    /// 创建标题栏, 使用默认浅色主题。
    pub fn new(title: impl Into<String>) -> Self {
        Self::themed(&LightTheme, title)
    }

    /// 使用指定主题创建标题栏。
    pub fn themed(theme: &impl Theme, title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            height: theme.spacing_xl() + theme.spacing_lg(),
            button_size: theme.spacing_lg() + theme.spacing_xs(),
            button_gap: 1.0,
            margin: theme.spacing_md(),
            logo_size: theme.spacing_lg() + theme.spacing_xs(),
            logo_gap: theme.spacing_sm(),
            bg: theme.surface(),
            text_color: theme.text_primary(),
            button_color: theme.text_secondary(),
            button_hover_color: theme.text_primary(),
            close_hover_color: theme.danger(),
            button_bg_color: theme.border(),
            window_corner_radius: theme.radius_window(),
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

    /// 设置最大化 / 还原按钮产出的消息。
    pub fn on_maximize<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_maximize = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 设置标题栏拖拽时产出的消息。
    pub fn on_drag<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_drag = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 设置窗口右上角圆角半径,关闭按钮 hover 背景右上角会适配此半径。
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.window_corner_radius = radius;
        self
    }

    /// 计算第 i 个按钮 hover 背景矩形 (0= 关闭,1= 最大化,2= 最小化)。
    fn button_rect(&self, area: Rect, index: usize) -> Rect {
        let size = self.height;
        let right = area.origin.x + area.size.width;
        let x = right - (index as f32 + 1.0) * size - index as f32 * self.button_gap;
        let y = area.origin.y;
        Rect::from_xywh(x, y, size, size)
    }

    /// 计算第 i 个按钮图标矩形,在 hover 背景内居中。
    fn button_icon_rect(&self, bg: Rect) -> Rect {
        let size = self.button_size;
        let x = bg.origin.x + (bg.size.width - size) / 2.0;
        let y = bg.origin.y + (bg.size.height - size) / 2.0;
        Rect::from_xywh(x, y, size, size)
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

    /// 返回鼠标位置命中的按钮索引, 无命中返回 `None`。
    fn hit_button(&self, area: Rect, position: Point) -> Option<usize> {
        (0..self.buttons.len()).find(|i| self.button_rect(area, *i).contains(position))
    }

    /// 第 i 个按钮的图形符号颜色。
    fn button_symbol_color(&self, index: usize) -> Color {
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
            Color::rgba(base.r * 0.7, base.g * 0.7, base.b * 0.7, base.a)
        } else {
            base
        }
    }

    /// 第 i 个按钮的背景颜色 (正常状态透明, 悬停 / 按下时显示)。
    fn button_background_color(&self, _index: usize) -> Option<Color> {
        if self.buttons[_index].pressed {
            Some(Color::rgba(
                self.button_bg_color.r * 0.85,
                self.button_bg_color.g * 0.85,
                self.button_bg_color.b * 0.85,
                self.button_bg_color.a,
            ))
        } else if self.buttons[_index].hovered {
            Some(self.button_bg_color)
        } else {
            None
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
                // 双击: 最大化 / 还原
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

    /// 用纯轴对齐几何图形绘制第 i 个按钮的符号 (0= 关闭,1= 最大化,2= 最小化)。
    ///
    /// 为避开旋转实例在部分 GPU 驱动下的表现不一致, 所有符号均用
    /// `push_rect` 实现: 水平 / 垂直线段用细长矩形, 对角线用小圆点队列近似。
    fn paint_button_symbol(&self, rects: &mut RectBatch, index: usize, rect: Rect, color: Color) {
        let cx = rect.origin.x + rect.size.width * 0.5;
        let cy = rect.origin.y + rect.size.height * 0.5;
        // 符号占用按钮内接正方形的约 58%, 线粗约 7.5%, 更纤细。
        let extent = rect.size.width.min(rect.size.height) * 0.58 * 0.5;
        let thickness = rect.size.width.min(rect.size.height) * 0.075;
        let half_thick = thickness * 0.5;

        match index {
            // 关闭:× 形两条对角线, 用小圆点队列近似。
            0 => {
                self.push_axis_aligned_diagonal(
                    rects,
                    Point::new(cx - extent, cy - extent),
                    Point::new(cx + extent, cy + extent),
                    thickness,
                    color,
                );
                self.push_axis_aligned_diagonal(
                    rects,
                    Point::new(cx - extent, cy + extent),
                    Point::new(cx + extent, cy - extent),
                    thickness,
                    color,
                );
            }
            // 最大化:□ 形方框, 四条直边。
            1 => {
                let side = extent * 2.0;
                let top = cy - extent;
                let left = cx - extent;
                // 上
                rects.push_rect(
                    Rect::from_xywh(left, top, side, thickness),
                    color,
                    half_thick,
                );
                // 下
                rects.push_rect(
                    Rect::from_xywh(left, top + side - thickness, side, thickness),
                    color,
                    half_thick,
                );
                // 左
                rects.push_rect(
                    Rect::from_xywh(left, top + thickness, thickness, side - 2.0 * thickness),
                    color,
                    half_thick,
                );
                // 右
                rects.push_rect(
                    Rect::from_xywh(
                        left + side - thickness,
                        top + thickness,
                        thickness,
                        side - 2.0 * thickness,
                    ),
                    color,
                    half_thick,
                );
            }
            // 最小化: 水平线段。
            _ => {
                rects.push_rect(
                    Rect::from_xywh(cx - extent, cy - half_thick, extent * 2.0, thickness),
                    color,
                    half_thick,
                );
            }
        }
    }

    /// 用轴对齐小圆点队列近似一条对角线。
    ///
    /// 每个步进放置一个 `thickness × thickness` 的圆角矩形,
    /// 圆角半径为 `thickness/2` 使其呈圆形, 彼此重叠形成平滑线段。
    fn push_axis_aligned_diagonal(
        &self,
        rects: &mut RectBatch,
        p1: Point,
        p2: Point,
        thickness: f32,
        color: Color,
    ) {
        if thickness <= 0.0 {
            return;
        }
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let length = (dx * dx + dy * dy).sqrt();
        if length < 1e-6 {
            return;
        }
        let half = thickness * 0.5;
        // 步长取 thickness 的一半, 让小圆点高度重叠, 对角线看起来更实心。
        let step = thickness * 0.5;
        let count = (length / step).ceil().max(1.0) as usize;
        for i in 0..=count {
            let t = i as f32 / count as f32;
            let x = p1.x + dx * t;
            let y = p1.y + dy * t;
            rects.push_rect(
                Rect::from_xywh(x - half, y - half, thickness, thickness),
                color,
                half,
            );
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

        // LOGO: 玻璃画布 + 颜料滴。
        let logo_rect = self.logo_rect(area);
        let logo_size = logo_rect.size.width;

        // 外框：accent 描边效果的圆角矩形。
        // 与 assets/logo/logo.svg 比例对应：外框内缩 6%，描边 16%，圆角 25%。
        let outer_inset = logo_size * 0.06;
        let frame_rect = logo_rect.inset(outer_inset);
        let frame_radius = logo_size * 0.25;
        rects.push_rect(frame_rect, self.logo_frame_color, frame_radius);

        // 内部填充：白色半透明，形成“环 + 面”。
        let stroke = logo_size * 0.16;
        let fill_rect = frame_rect.inset(stroke);
        let fill_radius = (frame_radius - stroke).max(0.0);
        rects.push_rect(fill_rect, self.logo_fill_color, fill_radius);

        // 颜料滴：实心 accent 圆，偏右下。
        let dot_size = logo_size * 0.38;
        let dot_offset = logo_size * 0.58;
        let dot_rect = Rect::from_xywh(
            logo_rect.origin.x + dot_offset - dot_size / 2.0,
            logo_rect.origin.y + dot_offset - dot_size / 2.0,
            dot_size,
            dot_size,
        );
        rects.push_rect(dot_rect, self.logo_dot_color, dot_size / 2.0);

        // 标题文字, 垂直居中。
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

        // 三个按钮: 正常仅显示几何符号, 悬停 / 按下时出现矩形背景。
        for i in 0..self.buttons.len() {
            let bg = self.button_rect(area, i);
            let icon = self.button_icon_rect(bg);
            if let Some(bg_color) = self.button_background_color(i) {
                let radii = if i == 0 {
                    [0.0, self.window_corner_radius, 0.0, 0.0]
                } else {
                    [0.0; 4]
                };
                rects.push_rounded_rect(bg, bg_color, radii);
            }
            self.paint_button_symbol(rects, i, icon, self.button_symbol_color(i));
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
                    // 非按钮区: 拖拽或双击最大化
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
    /// 指定按钮是否悬停 (测试用,0= 关闭,1= 最大化,2= 最小化)。
    pub(crate) fn button_hovered(&self, index: usize) -> bool {
        self.buttons[index].hovered
    }

    /// 指定按钮是否按下 (测试用)。
    pub(crate) fn button_pressed(&self, index: usize) -> bool {
        self.buttons[index].pressed
    }

    /// 指定按钮中心 (测试用)。
    pub(crate) fn button_center(&self, area: Rect, index: usize) -> Point {
        let r = self.button_rect(area, index);
        Point::new(
            r.origin.x + r.size.width / 2.0,
            r.origin.y + r.size.height / 2.0,
        )
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
        assert_eq!(
            bar.button_size,
            LightTheme.spacing_lg() + LightTheme.spacing_xs()
        );
        assert!((bar.button_gap - 1.0).abs() < f32::EPSILON);
        assert_eq!(bar.margin, LightTheme.spacing_md());
        assert_eq!(
            bar.logo_size,
            LightTheme.spacing_lg() + LightTheme.spacing_xs()
        );
        assert_eq!(bar.window_corner_radius, LightTheme.radius_window());
        assert_eq!(bar.bg, LightTheme.surface());
        assert_eq!(bar.logo_frame_color, LightTheme.accent());
        assert_eq!(bar.logo_fill_color, LightTheme.surface());
        assert_eq!(bar.logo_dot_color, LightTheme.accent());
    }

    #[test]
    fn cursor_over_close_button_hovers_only_close() {
        let mut bar = TitleBar::new("丹青");
        let area = title_bar_area();
        let close_center = bar.button_center(area, 0);
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
        let close_center = bar.button_center(area, 0);
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
        let close_center = bar.button_center(area, 0);
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
        let close_center = bar.button_center(area, 0);
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

    #[test]
    fn close_hover_background_uses_window_corner_radius() {
        let mut bar = TitleBar::themed(&LightTheme, "丹青");
        let area = title_bar_area();
        let mut texts = TextBatch::new();
        bar.layout(Constraints::tight(area.size), &mut texts);

        let center = bar.button_center(area, 0);
        let mut msgs = MsgQueue::new();
        bar.event(&Event::CursorMoved(center), area, &mut msgs);

        let mut rects = RectBatch::new();
        texts.clear();
        bar.paint(area, &mut rects, &mut texts);

        let height = bar.height;
        let matches: Vec<_> = rects
            .instance_rects()
            .iter()
            .zip(rects.instance_radii())
            .filter(|(r, radii)| r.size == Size::new(height, height) && radii[1] > 0.0)
            .map(|(r, radii)| (*r, radii))
            .collect();
        assert_eq!(matches.len(), 1, "应恰好找到关闭按钮 hover 背景");
        assert_eq!(matches[0].0.size, Size::new(height, height));
        assert_eq!(matches[0].1, [0.0, LightTheme.radius_window(), 0.0, 0.0]);
    }

    #[test]
    fn maximize_hover_background_is_sharp_rectangle() {
        let mut bar = TitleBar::themed(&LightTheme, "丹青");
        let area = title_bar_area();
        let mut texts = TextBatch::new();
        bar.layout(Constraints::tight(area.size), &mut texts);

        let center = bar.button_center(area, 1);
        let mut msgs = MsgQueue::new();
        bar.event(&Event::CursorMoved(center), area, &mut msgs);

        let mut rects = RectBatch::new();
        texts.clear();
        bar.paint(area, &mut rects, &mut texts);

        let height = bar.height;
        let matches: Vec<_> = rects
            .instance_rects()
            .iter()
            .zip(rects.instance_radii())
            .filter(|(r, radii)| r.size == Size::new(height, height) && radii == &[0.0; 4])
            .map(|(r, radii)| (*r, radii))
            .collect();
        assert!(!matches.is_empty(), "应找到最大化按钮 hover 背景");
    }

    #[test]
    fn corner_radius_builder_overrides_theme() {
        let bar = TitleBar::themed(&LightTheme, "丹青").corner_radius(8.0);
        assert!((bar.window_corner_radius - 8.0).abs() < f32::EPSILON);
    }
}
