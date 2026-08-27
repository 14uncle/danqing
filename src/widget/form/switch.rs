//! @author 十四叔
//! @date 2026/08/27
//!
//! 滑动开关: iOS 风格 Switch 组件。
//!
//! 圆角矩形轨道 + 圆形滑块, 点击/键盘切换 on/off。
//! 支持 bool 状态绑定与消息产出, 150ms 过渡动画。

use std::any::Any;
use std::cell::Cell;

use crate::app::AnimationCtx;
use crate::event::{Event, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 消息工厂: 切换时产出一条应用消息。
type MsgFactory = Box<dyn Fn() -> Box<dyn Any>>;
/// bool 绑定闭包: 从类型擦除的应用状态读取开关状态。
type BoolBinding = Box<dyn Fn(&dyn Any) -> bool>;

/// 轨道宽度 (逻辑像素)。
const TRACK_WIDTH: f32 = 36.0;
/// 轨道高度。
const TRACK_HEIGHT: f32 = 20.0;
/// 滑块直径。
const KNOB_DIAMETER: f32 = 16.0;
/// 轨道圆角半径 (等于高度一半, 呈胶囊形)。
const TRACK_RADIUS: f32 = TRACK_HEIGHT / 2.0;
/// 滑块与轨道边缘的间距。
const KNOB_PADDING: f32 = (TRACK_HEIGHT - KNOB_DIAMETER) / 2.0;
/// 动画时长 (毫秒)。
const ANIM_DURATION_MS: f32 = 150.0;
/// 动画时长 (秒)。
const ANIM_DURATION_S: f32 = ANIM_DURATION_MS / 1000.0;

/// 滑动开关组件。
///
/// 轨道: 圆角矩形 36×20; 滑块: 圆形 ⌀16, 居中于轨道。
/// OFF 态轨道灰色, ON 态 accent 色; 滑块始终白色。
/// 点击或键盘 (Space/Enter) 切换状态, 产出消息。
pub struct Switch {
    /// 当前 checked 状态 (每帧从绑定同步)。
    checked: bool,
    /// bool 状态绑定。
    checked_binding: Option<BoolBinding>,
    /// 切换时产出的消息工厂。
    on_toggle: Option<MsgFactory>,
    /// 动画进度: 0.0 = OFF, 1.0 = ON。
    anim_progress: f32,
    /// 动画目标: 0.0 或 1.0。
    anim_target: f32,
    /// 上一帧的绝对时间 (用于计算 dt)。
    last_time: Option<std::time::Instant>,
    /// 鼠标悬停。
    hovered: bool,
    /// 鼠标按下。
    pressed: bool,
    /// 是否获得焦点。
    focused: bool,
    /// OFF 态轨道颜色。
    track_off_color: Color,
    /// ON 态轨道颜色 (accent)。
    track_on_color: Color,
    /// 滑块颜色。
    knob_color: Color,
    /// 焦点环颜色。
    focus_ring_color: Color,
    /// layout 缓存: 自身绝对矩形。
    area: Cell<Rect>,
}

impl Switch {
    /// 创建滑动开关, 使用默认浅色主题 token, OFF 态。
    pub fn new() -> Self {
        Self::themed(&LightTheme)
    }

    /// 使用指定主题创建滑动开关。
    pub fn themed(theme: &impl Theme) -> Self {
        Self {
            checked: false,
            checked_binding: None,
            on_toggle: None,
            anim_progress: 0.0,
            anim_target: 0.0,
            last_time: None,
            hovered: false,
            pressed: false,
            focused: false,
            track_off_color: theme.border(),
            track_on_color: theme.accent(),
            knob_color: Color::WHITE,
            focus_ring_color: theme.accent(),
            area: Cell::new(Rect::default()),
        }
    }

    /// 绑定 bool 状态: 每帧从应用状态读取 checked 值。
    pub fn bind<S: 'static>(mut self, f: impl Fn(&S) -> bool + 'static) -> Self {
        self.checked_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("Switch bool 绑定的状态类型不匹配"))
        }));
        self
    }

    /// 设置切换时产出的消息。
    pub fn on_toggle<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_toggle = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 设置 OFF 态轨道颜色。
    pub fn track_off_color(mut self, color: Color) -> Self {
        self.track_off_color = color;
        self
    }

    /// 设置 ON 态轨道颜色。
    pub fn track_on_color(mut self, color: Color) -> Self {
        self.track_on_color = color;
        self
    }

    /// 设置滑块颜色。
    pub fn knob_color(mut self, color: Color) -> Self {
        self.knob_color = color;
        self
    }

    /// 当前 checked 状态。
    pub fn is_checked(&self) -> bool {
        self.checked
    }
}

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Switch {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(bind) = &self.checked_binding {
            self.checked = bind(state);
            self.anim_target = if self.checked { 1.0 } else { 0.0 };
        }
    }

    fn animate(&mut self, ctx: &AnimationCtx) {
        let now = ctx.now;
        if let Some(last) = self.last_time {
            let dt = now.duration_since(last).as_secs_f32();
            if dt > 0.0 && (self.anim_progress - self.anim_target).abs() > 0.001 {
                let step = dt / ANIM_DURATION_S;
                if self.anim_progress < self.anim_target {
                    self.anim_progress = (self.anim_progress + step).min(self.anim_target);
                } else {
                    self.anim_progress = (self.anim_progress - step).max(self.anim_target);
                }
            }
        }
        self.last_time = Some(now);
    }

    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        let size = constraints.constrain(Size::new(TRACK_WIDTH, TRACK_HEIGHT));
        self.area.set(Rect::new(Point::ZERO, size));
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, _texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        self.area.set(area);

        let t = self.anim_progress;

        // 轨道颜色: 线性插值
        let track_color = Color::rgba(
            self.track_off_color.r + (self.track_on_color.r - self.track_off_color.r) * t,
            self.track_off_color.g + (self.track_on_color.g - self.track_off_color.g) * t,
            self.track_off_color.b + (self.track_on_color.b - self.track_off_color.b) * t,
            self.track_off_color.a + (self.track_on_color.a - self.track_off_color.a) * t,
        );

        // 轨道
        rects.push_rect(area, track_color, TRACK_RADIUS);

        // 焦点环
        if self.focused {
            rects.push_rounded_border(area, self.focus_ring_color, TRACK_RADIUS, 2.0);
        }

        // 滑块位置: 左侧 KNOB_PADDING → 右侧 KNOB_PADDING
        let knob_x_min = area.origin.x + KNOB_PADDING;
        let knob_x_max = area.origin.x + area.size.width - KNOB_PADDING - KNOB_DIAMETER;
        let knob_x = knob_x_min + (knob_x_max - knob_x_min) * t;
        let knob_y = area.origin.y + KNOB_PADDING;

        // 按压缩放: 按下时滑块略小
        let scale = if self.pressed { 0.85 } else { 1.0 };
        let scaled_size = KNOB_DIAMETER * scale;
        let offset = (KNOB_DIAMETER - scaled_size) / 2.0;
        let scaled_rect = Rect::from_xywh(
            knob_x + offset,
            knob_y + offset,
            scaled_size,
            scaled_size,
        );

        rects.push_rect(scaled_rect, self.knob_color, scaled_size / 2.0);
    }

    fn event(&mut self, event: &Event, _area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let area = self.area.get();

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
                button: MouseButton::Left,
                pressed: true,
                position,
            } => {
                if area.contains(*position) {
                    self.pressed = true;
                    return EventResult::Consumed;
                }
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position,
            } => {
                if self.pressed && area.contains(*position) {
                    self.pressed = false;
                    if let Some(factory) = &self.on_toggle {
                        msgs.push(factory());
                    }
                    return EventResult::Consumed;
                }
                self.pressed = false;
                EventResult::Ignored
            }
            Event::Key {
                key: Key::Named(NamedKey::Space) | Key::Named(NamedKey::Enter),
                pressed: true,
                ..
            } if self.focused => {
                if let Some(factory) = &self.on_toggle {
                    msgs.push(factory());
                }
                EventResult::Consumed
            }
            Event::FocusIn => {
                self.focused = true;
                EventResult::Consumed
            }
            Event::FocusOut => {
                self.focused = false;
                self.pressed = false;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    fn reset_focus(&mut self) {
        self.focused = false;
        self.pressed = false;
    }

    fn children(&self) -> &[crate::widget::Node] {
        &[]
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::TextBatch;
    use crate::widget::MsgQueue;
    use std::time::{Duration, Instant};

    fn switch_area() -> Rect {
        Rect::from_xywh(0.0, 0.0, TRACK_WIDTH, TRACK_HEIGHT)
    }

    #[test]
    fn layout_returns_fixed_size() {
        let mut sw = Switch::new();
        let mut texts = TextBatch::new();
        let size = sw.layout(Constraints::loose(Size::new(400.0, 400.0)), &mut texts);
        assert_eq!(size.width, TRACK_WIDTH, "宽度应为 TRACK_WIDTH");
        assert_eq!(size.height, TRACK_HEIGHT, "高度应为 TRACK_HEIGHT");
    }

    #[test]
    fn layout_constrains_to_smaller() {
        let mut sw = Switch::new();
        let mut texts = TextBatch::new();
        let size = sw.layout(Constraints::tight(Size::new(20.0, 10.0)), &mut texts);
        assert_eq!(size.width, 20.0, "应受约束限制");
        assert_eq!(size.height, 10.0, "应受约束限制");
    }

    #[test]
    fn click_emits_toggle_message() {
        let mut sw = Switch::new().on_toggle(|| 42u8);
        let mut msgs = MsgQueue::new();
        sw.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = switch_area();
        let center = Point::new(TRACK_WIDTH / 2.0, TRACK_HEIGHT / 2.0);
        // 按下
        sw.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: center,
            },
            area,
            &mut msgs,
        );
        // 抬起
        sw.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: center,
            },
            area,
            &mut msgs,
        );
        assert_eq!(msgs.len(), 1, "点击应产出消息");
        assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&42));
    }

    #[test]
    fn click_outside_does_not_emit() {
        let mut sw = Switch::new().on_toggle(|| 42u8);
        let mut msgs = MsgQueue::new();
        sw.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = switch_area();
        let outside = Point::new(TRACK_WIDTH + 10.0, TRACK_HEIGHT + 10.0);
        sw.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: outside,
            },
            area,
            &mut msgs,
        );
        sw.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: outside,
            },
            area,
            &mut msgs,
        );
        assert_eq!(msgs.len(), 0, "外部点击不应产出消息");
    }

    #[test]
    fn space_key_emits_toggle() {
        let mut sw = Switch::new().on_toggle(|| 99u8);
        let mut msgs = MsgQueue::new();
        sw.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = switch_area();
        // 获得焦点
        sw.event(&Event::FocusIn, area, &mut msgs);
        // 按 Space
        sw.event(
            &Event::Key {
                key: Key::Named(NamedKey::Space),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            area,
            &mut msgs,
        );
        assert_eq!(msgs.len(), 1, "Space 应产出消息");
        assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&99));
    }

    #[test]
    fn enter_key_emits_toggle() {
        let mut sw = Switch::new().on_toggle(|| 77u8);
        let mut msgs = MsgQueue::new();
        sw.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = switch_area();
        sw.event(&Event::FocusIn, area, &mut msgs);
        sw.event(
            &Event::Key {
                key: Key::Named(NamedKey::Enter),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            area,
            &mut msgs,
        );
        assert_eq!(msgs.len(), 1, "Enter 应产出消息");
        assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&77));
    }

    #[test]
    fn key_without_focus_does_not_emit() {
        let mut sw = Switch::new().on_toggle(|| 42u8);
        let mut msgs = MsgQueue::new();
        sw.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = switch_area();
        // 不获得焦点, 直接按 Space
        sw.event(
            &Event::Key {
                key: Key::Named(NamedKey::Space),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            area,
            &mut msgs,
        );
        assert_eq!(msgs.len(), 0, "无焦点时键盘不应产出消息");
    }

    #[test]
    fn anim_progress_interpolates_toward_target() {
        let mut sw = Switch::new();
        sw.checked = true;
        sw.anim_target = 1.0;
        sw.anim_progress = 0.0;

        let now = Instant::now();
        sw.animate(&AnimationCtx::new(now, Duration::ZERO));
        // 第一帧: last_time 为 None, 不插值
        assert_eq!(sw.anim_progress, 0.0, "第一帧不应插值");

        // 第二帧: 前进 75ms (一半)
        let t2 = now + Duration::from_millis(75);
        sw.animate(&AnimationCtx::new(t2, Duration::from_millis(75)));
        assert!(
            (sw.anim_progress - 0.5).abs() < 0.05,
            "75ms 后应接近 0.5, 实际 {}",
            sw.anim_progress
        );

        // 第三帧: 再前进 75ms (总计 150ms, 应到 1.0)
        let t3 = now + Duration::from_millis(150);
        sw.animate(&AnimationCtx::new(t3, Duration::from_millis(150)));
        assert!(
            (sw.anim_progress - 1.0).abs() < 0.05,
            "150ms 后应接近 1.0, 实际 {}",
            sw.anim_progress
        );
    }

    #[test]
    fn focusable_returns_true() {
        let sw = Switch::new();
        assert!(sw.focusable(), "Switch 应可聚焦");
    }

    #[test]
    fn default_is_unchecked() {
        let sw = Switch::new();
        assert!(!sw.is_checked(), "默认应为未选中");
    }
}
