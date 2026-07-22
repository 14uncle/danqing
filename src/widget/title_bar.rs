//! @author 十四叔
//! @date 2026/07/19

//! 自绘标题栏组件。
//!
//! 左侧显示窗口 LOGO 与标题，按钮布局由 `TitleBarStyle` 决定：
//! `Standard` 右侧提供最小化 / 最大化 / 关闭三个按钮 (Windows / Linux),
//! `TrafficLights` 左侧提供红绿灯按钮 (macOS)。
//! 阶段 1 按钮产出 `WindowAction` 消息，由 `window.rs` 的 `Handler` 调用 OS 窗口 API。

use std::any::Any;
use std::time::{Duration, Instant};

use crate::event::{Event, MouseButton};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 标题栏按钮布局样式。
///
/// 默认值由 [`TitleBarStyle::platform_default`] 按平台解析，
/// 也可通过 [`TitleBar::style`] 显式指定 (如跨平台测试红绿灯布局)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleBarStyle {
    /// 右侧最小化 / 最大化 / 关闭三键 (Windows / Linux 风格)。
    Standard,
    /// 左侧红绿灯按钮 (macOS 风格): 红 = 关闭，黄 = 最小化，绿 = 最大化。
    TrafficLights,
}

impl TitleBarStyle {
    /// 当前平台的默认样式：macOS 为红绿灯，其余平台为标准右侧三键。
    pub fn platform_default() -> Self {
        if cfg!(target_os = "macos") {
            Self::TrafficLights
        } else {
            Self::Standard
        }
    }

    /// 按钮角色从放置边缘起的排列顺序。
    fn placed_roles(self) -> [ButtonRole; 3] {
        match self {
            Self::Standard => [
                ButtonRole::Close,
                ButtonRole::Maximize,
                ButtonRole::Minimize,
            ],
            Self::TrafficLights => [
                ButtonRole::Close,
                ButtonRole::Minimize,
                ButtonRole::Maximize,
            ],
        }
    }
}

/// 标题栏按钮角色; 数组下标即角色序 (0= 关闭，1= 最大化，2= 最小化)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonRole {
    Close,
    Maximize,
    Minimize,
}

impl ButtonRole {
    /// 全部角色，下标与角色序一致。
    const ALL: [Self; 3] = [Self::Close, Self::Maximize, Self::Minimize];

    /// 角色对应的按钮数组下标。
    fn index(self) -> usize {
        self as usize
    }
}

/// 红绿灯 hover 符号颜色 (macOS 惯例：半透明深灰，不随主题变化)。
const TRAFFIC_GLYPH_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.55);

/// 标题栏按钮。
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
    /// 按钮布局样式。
    style: TitleBarStyle,
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
    /// LOGO 外框色。
    logo_frame_color: Color,
    /// LOGO 内部填充色。
    logo_fill_color: Color,
    /// LOGO 颜料点色。
    logo_dot_color: Color,
    /// 红绿灯关闭按钮色。
    traffic_close_color: Color,
    /// 红绿灯最小化按钮色。
    traffic_minimize_color: Color,
    /// 红绿灯最大化按钮色。
    traffic_maximize_color: Color,
    /// 红绿灯按钮直径。
    traffic_diameter: f32,
    /// 红绿灯按钮间距。
    traffic_gap: f32,
    /// 红绿灯组前导边距。
    traffic_leading: f32,
    /// 标题字号。
    font_size: u16,
    /// 三个按钮状态，按角色序索引 (0= 关闭，1= 最大化，2= 最小化)。
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
    /// 上次在非按钮区按下左键的时间与位置，用于识别双击最大化。
    last_left_press: Option<(Instant, Point)>,
}

impl TitleBar {
    /// 创建标题栏，使用默认浅色主题。
    pub fn new(title: impl Into<String>) -> Self {
        Self::themed(&LightTheme, title)
    }

    /// 使用指定主题创建标题栏。
    pub fn themed(theme: &impl Theme, title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            style: TitleBarStyle::platform_default(),
            height: theme.spacing_xl() + theme.spacing_lg(),
            button_size: theme.spacing_lg() + theme.spacing_xs(),
            button_gap: 1.0,
            margin: theme.spacing_md(),
            logo_size: theme.spacing_lg(),
            logo_gap: theme.spacing_sm(),
            bg: theme.surface(),
            text_color: theme.text_primary(),
            button_color: theme.text_secondary(),
            button_hover_color: theme.text_primary(),
            close_hover_color: theme.danger(),
            button_bg_color: theme.border(),
            logo_frame_color: theme.accent(),
            logo_fill_color: theme.surface(),
            logo_dot_color: theme.accent(),
            traffic_close_color: theme.traffic_close(),
            traffic_minimize_color: theme.traffic_minimize(),
            traffic_maximize_color: theme.traffic_maximize(),
            // macOS 红绿灯规格：直径 12、间隙 8、前导边距 12, 取间距 token 近似值。
            traffic_diameter: theme.spacing_md(),
            traffic_gap: theme.spacing_sm(),
            traffic_leading: theme.spacing_md(),
            font_size: theme.font_size_body(),
            buttons: [TitleButton::default(); 3],
            on_close: None,
            on_minimize: None,
            on_maximize: None,
            on_drag: None,
            area: Rect::default(),
            last_left_press: None,
        }
    }

    /// 设置按钮布局样式，覆盖平台默认值。
    pub fn style(mut self, style: TitleBarStyle) -> Self {
        self.style = style;
        self
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

    /// 计算指定角色按钮的矩形 (Standard 为整高方形，红绿灯为圆形外接正方形)。
    fn button_rect(&self, area: Rect, role: ButtonRole) -> Rect {
        let placed = self.style.placed_roles();
        let pos = placed
            .iter()
            .position(|r| *r == role)
            .expect("placed_roles 覆盖全部角色");
        match self.style {
            TitleBarStyle::Standard => {
                let size = self.height;
                let right = area.origin.x + area.size.width;
                let x = right - (pos as f32 + 1.0) * size - pos as f32 * self.button_gap;
                Rect::from_xywh(x, area.origin.y, size, size)
            }
            TitleBarStyle::TrafficLights => {
                let d = self.traffic_diameter;
                let x = area.origin.x + self.traffic_leading + pos as f32 * (d + self.traffic_gap);
                let y = area.origin.y + (self.height - d) / 2.0;
                Rect::from_xywh(x, y, d, d)
            }
        }
    }

    /// 计算第 i 个按钮图标矩形，在 hover 背景内居中。
    fn button_icon_rect(&self, bg: Rect) -> Rect {
        let size = self.button_size;
        let x = bg.origin.x + (bg.size.width - size) / 2.0;
        let y = bg.origin.y + (bg.size.height - size) / 2.0;
        Rect::from_xywh(x, y, size, size)
    }

    /// 计算 LOGO 矩形。
    fn logo_rect(&self, area: Rect) -> Rect {
        let y = area.origin.y + (self.height - self.logo_size) / 2.0;
        let x = match self.style {
            TitleBarStyle::Standard => area.origin.x + self.margin,
            TitleBarStyle::TrafficLights => {
                // LOGO 顺排在红绿灯组之后。
                let buttons_end = area.origin.x
                    + self.traffic_leading
                    + 3.0 * self.traffic_diameter
                    + 2.0 * self.traffic_gap;
                buttons_end + self.logo_gap
            }
        };
        Rect::from_xywh(x, y, self.logo_size, self.logo_size)
    }

    /// 返回鼠标位置命中的按钮角色，无命中返回 `None`。
    fn hit_button(&self, area: Rect, position: Point) -> Option<ButtonRole> {
        ButtonRole::ALL
            .into_iter()
            .find(|role| self.button_rect(area, *role).contains(position))
    }

    /// 指定角色按钮的图形符号颜色。
    fn button_symbol_color(&self, role: ButtonRole) -> Color {
        let is_close = role == ButtonRole::Close;
        let btn = &self.buttons[role.index()];
        if is_close && btn.hovered {
            // 关闭按钮 hover 时背景变 danger，符号反白。
            return Color::WHITE;
        }
        let base = if btn.hovered {
            self.button_hover_color
        } else {
            self.button_color
        };
        if btn.pressed {
            Color::rgba(base.r * 0.7, base.g * 0.7, base.b * 0.7, base.a)
        } else {
            base
        }
    }

    /// 指定角色按钮的背景颜色 (正常状态透明，悬停 / 按下时显示)。
    fn button_background_color(&self, role: ButtonRole) -> Option<Color> {
        let btn = &self.buttons[role.index()];
        let is_close = role == ButtonRole::Close;
        if btn.pressed {
            let base = if is_close && btn.hovered {
                self.close_hover_color
            } else {
                self.button_bg_color
            };
            Some(Color::rgba(
                base.r * 0.85,
                base.g * 0.85,
                base.b * 0.85,
                base.a,
            ))
        } else if btn.hovered {
            if is_close {
                Some(self.close_hover_color)
            } else {
                Some(self.button_bg_color)
            }
        } else {
            None
        }
    }

    /// 指定角色的红绿灯填充色 (来自主题 token)。
    fn traffic_color(&self, role: ButtonRole) -> Color {
        match role {
            ButtonRole::Close => self.traffic_close_color,
            ButtonRole::Minimize => self.traffic_minimize_color,
            ButtonRole::Maximize => self.traffic_maximize_color,
        }
    }

    /// 触发指定角色按钮的回调。
    fn emit_button_action(&self, role: ButtonRole, msgs: &mut MsgQueue) {
        let factory = match role {
            ButtonRole::Close => &self.on_close,
            ButtonRole::Maximize => &self.on_maximize,
            ButtonRole::Minimize => &self.on_minimize,
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
                // 双击：最大化 / 还原
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

    /// 用纯轴对齐几何图形绘制指定角色按钮的符号。
    ///
    /// 为避开旋转实例在部分 GPU 驱动下的表现不一致，所有符号均用
    /// `push_rect` 实现：水平 / 垂直线段用细长矩形，对角线用小圆点队列近似。
    fn paint_button_symbol(
        &self,
        rects: &mut RectBatch,
        role: ButtonRole,
        rect: Rect,
        color: Color,
    ) {
        let cx = rect.origin.x + rect.size.width * 0.5;
        let cy = rect.origin.y + rect.size.height * 0.5;
        // 符号占用按钮内接正方形的约 58%, 线粗约 7.5%, 更纤细。
        let extent = rect.size.width.min(rect.size.height) * 0.58 * 0.5;
        let thickness = rect.size.width.min(rect.size.height) * 0.075;
        let half_thick = thickness * 0.5;

        match role {
            // 关闭:× 形两条对角线，用小圆点队列近似。
            ButtonRole::Close => {
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
            // 最大化：Standard 为 □ 形方框，红绿灯为 + 形 (水平 + 垂直线段)。
            ButtonRole::Maximize => {
                if self.style == TitleBarStyle::TrafficLights {
                    rects.push_rect(
                        Rect::from_xywh(cx - extent, cy - half_thick, extent * 2.0, thickness),
                        color,
                        half_thick,
                    );
                    rects.push_rect(
                        Rect::from_xywh(cx - half_thick, cy - extent, thickness, extent * 2.0),
                        color,
                        half_thick,
                    );
                    return;
                }
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
            // 最小化：水平线段。
            ButtonRole::Minimize => {
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
    /// 每个步进放置一个 `thickness × thickness` 的圆角矩形，
    /// 圆角半径为 `thickness/2` 使其呈圆形，彼此重叠形成平滑线段。
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
        // 步长取 thickness 的一半，让小圆点高度重叠，对角线看起来更实心。
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
        self.area = Rect::new(Point::ZERO, size);
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

        // 标题文字，垂直居中。
        let font_size = self.font_size;
        let baseline =
            area.origin.y + area.size.height / 2.0 + texts.ascent(f32::from(font_size)) / 2.0;
        texts.push_text(
            &self.title,
            logo_rect.origin.x + logo_rect.size.width + self.logo_gap,
            baseline,
            font_size,
            self.text_color,
        );

        // 按钮绘制按样式分支。
        match self.style {
            TitleBarStyle::Standard => {
                // 正常仅显示几何符号，悬停 / 按下时出现矩形背景。
                // 背景一律直角：窗口圆角由 DWM 裁剪 (Windows) 或原生装饰
                // (其他平台) 处理，自绘圆角反而无法与真实窗体圆角 / 最大化
                // 直角状态保持一致。
                for role in self.style.placed_roles() {
                    let bg = self.button_rect(area, role);
                    let icon = self.button_icon_rect(bg);
                    if let Some(bg_color) = self.button_background_color(role) {
                        rects.push_rect(bg, bg_color, 0.0);
                    }
                    self.paint_button_symbol(rects, role, icon, self.button_symbol_color(role));
                }
            }
            TitleBarStyle::TrafficLights => {
                // 红绿灯：始终绘制主题色实心圆，仅 hover 时叠加深色符号。
                for role in self.style.placed_roles() {
                    let circle = self.button_rect(area, role);
                    rects.push_rect(circle, self.traffic_color(role), circle.size.width / 2.0);
                    if self.buttons[role.index()].hovered {
                        self.paint_button_symbol(rects, role, circle, TRAFFIC_GLYPH_COLOR);
                    }
                }
            }
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.area = area;
        match event {
            Event::CursorMoved(p) => {
                let hit = self.hit_button(area, *p);
                for role in ButtonRole::ALL {
                    self.buttons[role.index()].hovered = hit == Some(role);
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
                if let Some(role) = hit {
                    for r in ButtonRole::ALL {
                        self.buttons[r.index()].pressed = r == role;
                    }
                    EventResult::Consumed
                } else {
                    // 非按钮区：拖拽或双击最大化
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
                for role in ButtonRole::ALL {
                    let btn = &mut self.buttons[role.index()];
                    if btn.pressed && hit == Some(role) {
                        triggered[role.index()] = true;
                    }
                    btn.pressed = false;
                }
                for role in ButtonRole::ALL {
                    if triggered[role.index()] {
                        self.emit_button_action(role, msgs);
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
    /// 指定按钮是否悬停 (测试用，0= 关闭，1= 最大化，2= 最小化)。
    pub(crate) fn button_hovered(&self, index: usize) -> bool {
        self.buttons[index].hovered
    }

    /// 指定按钮是否按下 (测试用)。
    pub(crate) fn button_pressed(&self, index: usize) -> bool {
        self.buttons[index].pressed
    }

    /// 指定按钮中心 (测试用，角色序：0= 关闭，1= 最大化，2= 最小化)。
    pub(crate) fn button_center(&self, area: Rect, index: usize) -> Point {
        let r = self.button_rect(area, ButtonRole::ALL[index]);
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
    fn default_style_is_platform_default() {
        let bar = TitleBar::new("丹青");
        assert_eq!(bar.style, TitleBarStyle::platform_default());
    }

    #[test]
    fn platform_default_matches_host_os() {
        let expected = if cfg!(target_os = "macos") {
            TitleBarStyle::TrafficLights
        } else {
            TitleBarStyle::Standard
        };
        assert_eq!(TitleBarStyle::platform_default(), expected);
    }

    #[test]
    fn traffic_colors_come_from_theme_tokens() {
        let bar = TitleBar::themed(&LightTheme, "丹青");
        assert_eq!(bar.traffic_close_color, LightTheme.traffic_close());
        assert_eq!(bar.traffic_minimize_color, LightTheme.traffic_minimize());
        assert_eq!(bar.traffic_maximize_color, LightTheme.traffic_maximize());
    }

    #[test]
    fn traffic_lights_buttons_are_left_ordered_and_centered() {
        let bar = TitleBar::new("丹青").style(TitleBarStyle::TrafficLights);
        let area = title_bar_area();

        let close = bar.button_rect(area, ButtonRole::Close);
        let minimize = bar.button_rect(area, ButtonRole::Minimize);
        let maximize = bar.button_rect(area, ButtonRole::Maximize);

        // 左置：整组位于左半区，顺序 关闭 → 最小化 → 最大化。
        assert!(close.origin.x >= area.origin.x);
        assert!(close.origin.x < minimize.origin.x);
        assert!(minimize.origin.x < maximize.origin.x);
        assert!(maximize.origin.x + maximize.size.width < area.size.width / 2.0);
        // 圆形 (外接矩形为正方形) 且垂直居中。
        for r in [close, minimize, maximize] {
            assert_eq!(r.size.width, r.size.height);
            let expected_y = area.origin.y + (bar.height - r.size.height) / 2.0;
            assert!((r.origin.y - expected_y).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn standard_and_traffic_lights_hit_areas_do_not_cross() {
        let area = title_bar_area();
        let standard = TitleBar::new("丹青").style(TitleBarStyle::Standard);
        let traffic = TitleBar::new("丹青").style(TitleBarStyle::TrafficLights);
        let center_of = |r: Rect| {
            Point::new(
                r.origin.x + r.size.width / 2.0,
                r.origin.y + r.size.height / 2.0,
            )
        };

        // 红绿灯关闭按钮中心，在 Standard 布局下不命中任何按钮。
        let p = center_of(traffic.button_rect(area, ButtonRole::Close));
        assert_eq!(standard.hit_button(area, p), None);
        assert_eq!(traffic.hit_button(area, p), Some(ButtonRole::Close));

        // Standard 关闭按钮中心，在红绿灯布局下不命中任何按钮。
        let p = center_of(standard.button_rect(area, ButtonRole::Close));
        assert_eq!(traffic.hit_button(area, p), None);
        assert_eq!(standard.hit_button(area, p), Some(ButtonRole::Close));
    }

    #[test]
    fn traffic_lights_paint_circles_with_theme_colors() {
        let mut bar = TitleBar::themed(&LightTheme, "丹青").style(TitleBarStyle::TrafficLights);
        let area = title_bar_area();
        let mut texts = TextBatch::new();
        bar.layout(Constraints::tight(area.size), &mut texts);

        let mut rects = RectBatch::new();
        texts.clear();
        bar.paint(area, &mut rects, &mut texts);

        let d = bar.traffic_diameter;
        let circles: Vec<_> = rects
            .instance_rects()
            .iter()
            .zip(rects.instance_radii())
            .zip(rects.instance_colors())
            .filter(|((r, radii), _)| r.size == Size::new(d, d) && *radii == [d / 2.0; 4])
            .map(|(_, c)| c)
            .collect();
        assert_eq!(circles.len(), 3, "应恰好绘制三个红绿灯圆形按钮");
        for color in [
            LightTheme.traffic_close(),
            LightTheme.traffic_minimize(),
            LightTheme.traffic_maximize(),
        ] {
            let expected = [color.r, color.g, color.b, color.a];
            assert!(
                circles.contains(&expected),
                "缺少主题色圆形按钮：{expected:?}"
            );
        }
        // 非 hover: 不绘制任何符号。
        let glyph = [
            TRAFFIC_GLYPH_COLOR.r,
            TRAFFIC_GLYPH_COLOR.g,
            TRAFFIC_GLYPH_COLOR.b,
            TRAFFIC_GLYPH_COLOR.a,
        ];
        assert!(!rects.instance_colors().contains(&glyph));
    }

    #[test]
    fn traffic_lights_hover_shows_glyph() {
        let mut bar = TitleBar::themed(&LightTheme, "丹青").style(TitleBarStyle::TrafficLights);
        let area = title_bar_area();
        let mut texts = TextBatch::new();
        bar.layout(Constraints::tight(area.size), &mut texts);

        let center = bar.button_center(area, ButtonRole::Close.index());
        let mut msgs = MsgQueue::new();
        bar.event(&Event::CursorMoved(center), area, &mut msgs);

        let mut rects = RectBatch::new();
        texts.clear();
        bar.paint(area, &mut rects, &mut texts);

        let glyph = [
            TRAFFIC_GLYPH_COLOR.r,
            TRAFFIC_GLYPH_COLOR.g,
            TRAFFIC_GLYPH_COLOR.b,
            TRAFFIC_GLYPH_COLOR.a,
        ];
        assert!(
            rects.instance_colors().contains(&glyph),
            "hover 关闭按钮应绘制深色 × 符号"
        );
    }

    #[test]
    fn traffic_lights_maximize_button_emits_registered_message() {
        let mut bar = TitleBar::new("丹青")
            .style(TitleBarStyle::TrafficLights)
            .on_maximize(|| WindowAction::MaximizeOrRestore);
        let area = title_bar_area();
        let center = bar.button_center(area, ButtonRole::Maximize.index());
        let mut msgs = MsgQueue::new();

        bar.event(&Event::CursorMoved(center), area, &mut msgs);
        bar.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: center,
            },
            area,
            &mut msgs,
        );
        bar.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: center,
            },
            area,
            &mut msgs,
        );

        assert_eq!(msgs.len(), 1);
        let action = msgs[0].downcast_ref::<WindowAction>().unwrap();
        assert_eq!(*action, WindowAction::MaximizeOrRestore);
    }

    #[test]
    fn style_builder_overrides_platform_default() {
        let bar = TitleBar::new("丹青").style(TitleBarStyle::TrafficLights);
        assert_eq!(bar.style, TitleBarStyle::TrafficLights);
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
        assert_eq!(bar.logo_size, LightTheme.spacing_lg());
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

        let result = bar.event(&Event::CursorMoved(Point::new(10.0, 10.0)), area, &mut msgs);

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
                position: Point::new(50.0, 20.0),
            },
            area,
            &mut msgs,
        );

        assert_eq!(msgs.len(), 1);
        let action = msgs[0].downcast_ref::<WindowAction>().unwrap();
        assert_eq!(*action, WindowAction::Drag);
    }

    #[test]
    fn close_hover_background_is_rightmost_sharp_rect() {
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

        // hover 背景为直角矩形，右上角由 DWM 窗体圆角裁剪适配。
        let height = bar.height;
        let matches: Vec<_> = rects
            .instance_rects()
            .iter()
            .zip(rects.instance_radii())
            .filter(|(r, radii)| r.size == Size::new(height, height) && radii == &[0.0; 4])
            .map(|(r, _)| *r)
            .collect();
        assert_eq!(matches.len(), 1, "应恰好找到关闭按钮 hover 背景");
        let bg = matches[0];
        assert_eq!(bg.origin.x + bg.size.width, area.origin.x + area.size.width);
        assert_eq!(bg.origin.y, area.origin.y);
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
}
