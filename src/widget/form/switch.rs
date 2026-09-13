//! @author 十四叔
//! @date 2026/08/27
//!
//! 滑动开关: iOS 风格 Switch 组件。
//!
//! 圆角矩形轨道 + 圆形滑块, 点击/键盘切换 on/off。
//! 支持 bool 状态绑定与消息产出, 150ms 过渡动画。

use std::any::Any;
use std::cell::Cell;
use std::time::Duration;

use crate::anim::Tween;
use crate::app::AnimationCtx;
use crate::event::{Event, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::theme::Easing;
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 消息工厂: 切换时产出一条应用消息。
type MsgFactory = Box<dyn Fn() -> Box<dyn Any>>;
/// bool 绑定闭包: 从类型擦除的应用状态读取开关状态。
type BoolBinding = Box<dyn Fn(&dyn Any) -> bool>;
/// 主题绑定：每帧从应用状态产出随主题流动的颜色。
type ThemeBinding = Box<dyn Fn(&dyn Any) -> SwitchColors>;

/// 随主题流动的开关颜色子集 (构建后仍可经 [`Switch::bind_theme`] 每帧刷新)。
///
/// **为什么需要它**: 视图树只在启动时构建一次 (`danqing/src/window/mod.rs` 的
/// `let tree = app.view();`, 之后整棵交给 Handler 不再重建), 所以 `themed(&theme)`
/// 烘进去的颜色不会跟随运行时切主题。`Switch` 原本只有 `bind`(checked 状态),
/// 主题色**没有任何每帧通路**。
///
/// `knob_color` 不在其中 —— 它恒为白, 本来就与主题无关。
/// 形状与 [`crate::widget::TitleBar::bind_theme`] 一致, 不要另发明一种。
#[derive(Debug, Clone, Copy, PartialEq)]
struct SwitchColors {
    track_off: Color,
    track_on: Color,
    focus_ring: Color,
}

impl SwitchColors {
    /// 从主题解析 —— `themed` 与 `bind_theme` **共用这一份**, 免得两处各写一套口径。
    fn from_theme(theme: &impl Theme) -> Self {
        Self {
            track_off: theme.border(),
            track_on: theme.accent(),
            focus_ring: theme.accent(),
        }
    }
}

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
/// 动画时长 (150ms 线性补间, 经 `anim::Tween` 推进)。
const ANIM_DURATION: Duration = Duration::from_millis(150);

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
    /// 主题绑定 (每帧刷新随主题流动的颜色; 见 [`Switch::bind_theme`])。
    theme_binding: Option<ThemeBinding>,
    /// 切换时产出的消息工厂。
    on_toggle: Option<MsgFactory>,
    /// 动画进度: 0.0 = OFF, 1.0 = ON。
    anim_progress: f32,
    /// 动画目标: 0.0 或 1.0。
    anim_target: f32,
    /// 进度补间 (150ms 线性, 目标翻转从当前值续接)。
    tween: Tween,
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
    /// 绑定主题：每帧从应用状态重取主题，刷新随主题流动的颜色
    /// (轨道 OFF/ON、焦点环); `knob_color` 恒为白, 不在其中。
    ///
    /// **构建态的颜色不会跟随运行时切主题** —— 视图树只建一次。产品侧若要支持
    /// 明暗切换, 必须挂上这个绑定。形状与 [`crate::widget::TitleBar::bind_theme`] 一致。
    pub fn bind_theme<S: 'static, T: Theme + 'static>(
        mut self,
        f: impl Fn(&S) -> T + 'static,
    ) -> Self {
        self.theme_binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Switch 主题绑定的状态类型不匹配");
            SwitchColors::from_theme(&f(state))
        }));
        self
    }

    /// 创建滑动开关, 使用默认浅色主题 token, OFF 态。
    pub fn new() -> Self {
        Self::themed(&LightTheme)
    }

    /// 使用指定主题创建滑动开关。
    pub fn themed(theme: &impl Theme) -> Self {
        let colors = SwitchColors::from_theme(theme);
        Self {
            checked: false,
            checked_binding: None,
            theme_binding: None,
            on_toggle: None,
            anim_progress: 0.0,
            anim_target: 0.0,
            tween: Tween::new(ANIM_DURATION, Easing::Linear),
            hovered: false,
            pressed: false,
            focused: false,
            track_off_color: colors.track_off,
            track_on_color: colors.track_on,
            knob_color: Color::WHITE,
            focus_ring_color: colors.focus_ring,
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
        if let Some(bind) = &self.theme_binding {
            let c = bind(state);
            self.track_off_color = c.track_off;
            self.track_on_color = c.track_on;
            self.focus_ring_color = c.focus_ring;
        }
        if let Some(bind) = &self.checked_binding {
            self.checked = bind(state);
            self.anim_target = if self.checked { 1.0 } else { 0.0 };
        }
    }

    fn animate(&mut self, ctx: &AnimationCtx) {
        self.anim_progress = self.tween.value(ctx.elapsed, self.anim_target);
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
        let scaled_rect =
            Rect::from_xywh(knob_x + offset, knob_y + offset, scaled_size, scaled_size);

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

    /// 主题 token 在**实例里**的表示 —— 与 `RectBatch::instance_colors` 同一表示。
    ///
    /// 实例里存的是**线性空间**值 (GPU 边界做 sRGB→linear, 见 `render/linear.rs`),
    /// 所以这里必须同样解码。拿 token 的原始 sRGB 分量去比, 断言的是修好双重
    /// gamma **之前**的旧行为。
    fn rgba_of(c: Color) -> [f32; 4] {
        let l = crate::render::LinearRgba::from(c);
        [l.r, l.g, l.b, l.a]
    }

    /// 焦点环**上直边**的颜色; 无焦点环则 `None`。
    ///
    /// 焦点环与轨道同为 accent 家族, 故按几何定位: 环的直边是 2px 厚、贴区域顶边、
    /// 横向成段 (`width > 2.0` 用以排除圆角处同样 2×2 的圆点, 它们也落在顶边)。
    fn focus_ring_top_color(sw: &mut Switch) -> Option<[f32; 4]> {
        let a = switch_area();
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        sw.paint(a, &mut rects, &mut texts);
        let colors = rects.instance_colors();
        rects
            .instance_rects()
            .iter()
            .position(|r| r.origin.y == a.origin.y && r.size.height == 2.0 && r.size.width > 2.0)
            .map(|i| colors[i])
    }

    #[test]
    fn focus_in_draws_an_accent_ring_and_focus_out_removes_it() {
        // Switch 的焦点视觉是**加性**的 (多画一圈 2px 环), 不是换边框色 ——
        // 与 TextInput/TextArea/IconInput 的换色不同, 故断言「环在 / 环不在」。
        let mut sw = Switch::new();
        assert_eq!(focus_ring_top_color(&mut sw), None, "静默态: 不画焦点环");

        let mut msgs = MsgQueue::new();
        assert_eq!(
            sw.event(&Event::FocusIn, switch_area(), &mut msgs),
            EventResult::Consumed,
            "FocusIn 应被消费"
        );
        assert_eq!(
            focus_ring_top_color(&mut sw),
            Some(rgba_of(LightTheme.accent())),
            "焦点态: 焦点环为 accent"
        );

        sw.event(&Event::FocusOut, switch_area(), &mut msgs);
        assert_eq!(focus_ring_top_color(&mut sw), None, "失焦后: 焦点环消失");
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
        // 第一帧: 补间从当前值 0 起 (边沿帧不跳变)
        assert_eq!(sw.anim_progress, 0.0, "第一帧不应跳变");

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
