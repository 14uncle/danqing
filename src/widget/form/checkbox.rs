//! @author 十四叔
//! @date 2026/09/28
//!
//! 复选框: 14×14 方盒 + 矢量勾, **纯矢量绘制** (文字不参与 —— 内嵌字体子集没有
//! ✓/✕ 字形, 矢量绕开字体约束; `CloseButton` 纯矢量先例, 勾与它的 × 符同一
//! `push_diagonal` 算法)。
//!
//! 两个消费形态 (danqing-log `SPEC-checkbox-widget` D2, **画法单真源**):
//! - `Checkbox` widget 本体: 与 `Switch` 同构 (bind / on_toggle / bind_theme /
//!   焦点环 / Space·Enter), 树内表单场景用; 首批消费者 = `examples/showcase.rs`。
//! - [`Checkbox::paint_box`] **静态画法**: 自绘行列表 (danqing-log `RowList`) 逐行
//!   调用; widget 自身 paint 也调它 —— 全框架只此一个画法入口, 杜绝两份画法漂移。
//!
//! **有意无动画** (spec D6): `Switch` 的 150ms tween 在逐行静态画法里无实例状态可
//! 依附; 若 widget 有动画而行内没有, 同一勾选两副面孔更糟 —— 族内一致性让位于
//! 画法单真源。
//!
//! 语义划线: `Switch` = 即时开关设置; `Checkbox` = 列表成员/显隐选择。

use std::any::Any;
use std::cell::Cell;

use crate::event::{Event, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Widget, push_diagonal};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 消息工厂: 切换时产出一条应用消息。
type MsgFactory = Box<dyn Fn() -> Box<dyn Any>>;
/// bool 绑定闭包: 从类型擦除的应用状态读取勾选状态。
type BoolBinding = Box<dyn Fn(&dyn Any) -> bool>;
/// 主题绑定: 每帧从应用状态产出随主题流动的颜色 (形状与 `Switch::bind_theme` 一致)。
type ThemeBinding = Box<dyn Fn(&dyn Any) -> CheckboxColors>;

/// 随主题流动的复选框颜色子集 (构建后仍可经 [`Checkbox::bind_theme`] 每帧刷新;
/// 「为什么需要 bind_theme」见 `Switch` 模块说明 —— 视图树只建一次)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CheckboxColors {
    /// 未选态边框色 (勾中态不使用 —— 填充即边界)。
    pub border: Color,
    /// 勾中态填充色; 兼作焦点环色 (与 `Switch` 焦点环 = accent 同规)。
    pub fill: Color,
    /// 勾的色。
    pub check: Color,
}

impl CheckboxColors {
    /// 常态套 (两构造器与 `bind_theme` 共用这一份口径): 勾中 = accent 实心 + 白勾;
    /// 未选 = `border()` 边框空盒。勾恒白 —— `Switch` 滑块「恒白与主题无关」先例。
    pub fn from_theme(theme: &impl Theme) -> Self {
        Self {
            border: theme.border(),
            fill: theme.accent(),
            check: Color::WHITE,
        }
    }

    /// 高亮行 (accent 底) 上的反白套 (spec D3b): accent 填充在 accent 底上不可见 →
    /// 勾中 = 白填充 + accent 勾; 未选 = 白边框空盒。与该行文字反白同规
    /// (主题无 on-accent token, 白即既定呈现)。
    pub fn on_accent(accent: Color) -> Self {
        Self {
            border: Color::WHITE,
            fill: Color::WHITE,
            check: accent,
        }
    }
}

impl Checkbox {
    /// 盒边长 (逻辑像素)。自绘消费者 (行列表) 的布局/居中与画法同源, 取此常量。
    pub const BOX_SIZE: f32 = 14.0;
}

/// 未选态边框线宽。
const BORDER_W: f32 = 1.5;
/// 盒圆角。
const BOX_RADIUS: f32 = 3.0;
/// 勾笔画粗细系数 (乘盒边长; `CloseButton` × 符同款 0.085 —— spec Open Q2:
/// 实机观感嫌细/嫌粗调这里, 不回 spec)。
const CHECK_STROKE: f32 = 0.085;
/// 焦点环外扩距离: 盒仅 14px, 环画盒内会遮住盒 —— 有意画出布局 area 外 2px
/// (环是持焦瞬态视觉, 布局与命中不变; 画圆角 = 盒圆角 + 外扩)。
const FOCUS_RING_OUTSET: f32 = 2.0;

/// 复选框组件: 固定 14×14 方盒, 勾中填充 accent + 白勾, 未选边框空盒。
///
/// 点击 (原地按下抬起) 或持焦按 Space/Enter 切换, 产出消息; 按下时盒 0.85
/// 居中缩放 (`Switch` 滑块按压缩放同语义); 持焦画外扩焦点环。
pub struct Checkbox {
    /// 当前 checked 状态 (每帧从绑定同步)。
    checked: bool,
    /// bool 状态绑定。
    checked_binding: Option<BoolBinding>,
    /// 主题绑定 (每帧刷新随主题流动的颜色)。
    theme_binding: Option<ThemeBinding>,
    /// 切换时产出的消息工厂。
    on_toggle: Option<MsgFactory>,
    /// 鼠标悬停。
    hovered: bool,
    /// 鼠标按下。
    pressed: bool,
    /// 是否获得焦点。
    focused: bool,
    /// 最近一帧同步的颜色套。
    colors: CheckboxColors,
    /// layout/paint 缓存: 自身绝对矩形。
    area: Cell<Rect>,
}

impl Checkbox {
    /// 创建复选框, 使用默认浅色主题 token, 未选态。
    pub fn new() -> Self {
        Self::themed(&LightTheme)
    }

    /// 使用指定主题创建复选框。
    pub fn themed(theme: &impl Theme) -> Self {
        Self {
            checked: false,
            checked_binding: None,
            theme_binding: None,
            on_toggle: None,
            hovered: false,
            pressed: false,
            focused: false,
            colors: CheckboxColors::from_theme(theme),
            area: Cell::new(Rect::default()),
        }
    }

    /// 绑定 bool 状态: 每帧从应用状态读取 checked 值。
    pub fn bind<S: 'static>(mut self, f: impl Fn(&S) -> bool + 'static) -> Self {
        self.checked_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("Checkbox bool 绑定的状态类型不匹配"))
        }));
        self
    }

    /// 设置切换时产出的消息。
    pub fn on_toggle<M: 'static>(mut self, f: impl Fn() -> M + 'static) -> Self {
        self.on_toggle = Some(Box::new(move || Box::new(f()) as Box<dyn Any>));
        self
    }

    /// 绑定主题: 每帧从应用状态重取主题, 刷新随主题流动的颜色
    /// (边框/填充/勾 —— 勾恒白但也走这一套, 免得两态分家)。
    /// 形状与 [`crate::widget::Switch::bind_theme`] 一致。
    pub fn bind_theme<S: 'static, T: Theme + 'static>(
        mut self,
        f: impl Fn(&S) -> T + 'static,
    ) -> Self {
        self.theme_binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Checkbox 主题绑定的状态类型不匹配");
            CheckboxColors::from_theme(&f(state))
        }));
        self
    }

    /// 当前 checked 状态。
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// **画法单真源** (静态): 在给定矩形内画盒 (+勾)。widget 自身 paint 与
    /// 自绘行列表逐行画法都调这里 —— 新增第三消费者也调这里, 不许另起画法。
    ///
    /// `colors` 由调用方按所在底面选套: 普通底面 [`CheckboxColors::from_theme`],
    /// accent 底 (高亮行) [`CheckboxColors::on_accent`]。
    pub fn paint_box(rects: &mut RectBatch, rect: Rect, checked: bool, colors: CheckboxColors) {
        let rect = rect.snap_to_pixels();
        let w = rect.size.width;
        if checked {
            rects.push_rect(rect, colors.fill, BOX_RADIUS);
            // 勾 = 两笔对角线 (短笔左下行 + 长笔右上行), 顶点比例按盒宽。
            let (x, y) = (rect.origin.x, rect.origin.y);
            let p1 = Point::new(x + w * 0.20, y + w * 0.52);
            let p2 = Point::new(x + w * 0.42, y + w * 0.72);
            let p3 = Point::new(x + w * 0.80, y + w * 0.28);
            let thickness = (w * CHECK_STROKE).max(1.0);
            push_diagonal(rects, p1, p2, thickness, colors.check);
            push_diagonal(rects, p2, p3, thickness, colors.check);
        } else {
            rects.push_rounded_border(rect, colors.border, BOX_RADIUS, BORDER_W);
        }
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Checkbox {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(bind) = &self.theme_binding {
            self.colors = bind(state);
        }
        if let Some(bind) = &self.checked_binding {
            self.checked = bind(state);
        }
    }

    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        let size = constraints.constrain(Size::new(Self::BOX_SIZE, Self::BOX_SIZE));
        self.area.set(Rect::new(Point::ZERO, size));
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, _texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        self.area.set(area);
        if self.focused {
            let ring = Rect::from_xywh(
                area.origin.x - FOCUS_RING_OUTSET,
                area.origin.y - FOCUS_RING_OUTSET,
                area.size.width + FOCUS_RING_OUTSET * 2.0,
                area.size.height + FOCUS_RING_OUTSET * 2.0,
            );
            rects.push_rounded_border(ring, self.colors.fill, BOX_RADIUS + FOCUS_RING_OUTSET, 1.5);
        }
        // 按压缩放: 盒 0.85 居中 (Switch 滑块同语义)。
        let scale = if self.pressed { 0.85 } else { 1.0 };
        let side = area.size.width * scale;
        let inset = (area.size.width - side) / 2.0;
        let box_rect = Rect::from_xywh(area.origin.x + inset, area.origin.y + inset, side, side);
        Self::paint_box(rects, box_rect, self.checked, self.colors);
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

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::MsgQueue;

    fn box_area() -> Rect {
        Rect::from_xywh(0.0, 0.0, Checkbox::BOX_SIZE, Checkbox::BOX_SIZE)
    }

    /// 主题 token 在 RectBatch **实例里**的表示 (线性空间) —— Switch 测试
    /// `rgba_of` 先例: 拿 token 原始 sRGB 分量直比, 断言的是双重 gamma 修复
    /// 之前的旧行为。
    fn rgba_of(c: Color) -> [f32; 4] {
        let l = crate::render::LinearRgba::from(c);
        [l.r, l.g, l.b, l.a]
    }

    /// 收集一次 paint 的实例 (矩形 + 颜色)。
    fn paint_instances(cb: &mut Checkbox) -> (Vec<crate::Rect>, Vec<[f32; 4]>) {
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        cb.paint(box_area(), &mut rects, &mut texts);
        (
            rects.instance_rects().to_vec(),
            rects.instance_colors().to_vec(),
        )
    }

    #[test]
    fn layout_returns_fixed_size() {
        let mut cb = Checkbox::new();
        let mut texts = TextBatch::new();
        let size = cb.layout(Constraints::loose(Size::new(400.0, 400.0)), &mut texts);
        assert_eq!(size.width, Checkbox::BOX_SIZE);
        assert_eq!(size.height, Checkbox::BOX_SIZE);
    }

    #[test]
    fn layout_constrains_to_smaller() {
        let mut cb = Checkbox::new();
        let mut texts = TextBatch::new();
        let size = cb.layout(Constraints::tight(Size::new(10.0, 8.0)), &mut texts);
        assert_eq!(size, Size::new(10.0, 8.0), "应受约束限制");
    }

    #[test]
    fn click_emits_toggle_message() {
        let mut cb = Checkbox::new().on_toggle(|| 42u8);
        let mut msgs = MsgQueue::new();
        cb.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = box_area();
        let center = Point::new(Checkbox::BOX_SIZE / 2.0, Checkbox::BOX_SIZE / 2.0);
        cb.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: center,
            },
            area,
            &mut msgs,
        );
        cb.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: center,
            },
            area,
            &mut msgs,
        );
        assert_eq!(msgs.len(), 1, "原地按下抬起应产出消息");
        assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&42));
    }

    #[test]
    fn press_inside_release_outside_no_message() {
        let mut cb = Checkbox::new().on_toggle(|| 42u8);
        let mut msgs = MsgQueue::new();
        cb.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = box_area();
        let inside = Point::new(7.0, 7.0);
        let outside = Point::new(100.0, 100.0);
        cb.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: inside,
            },
            area,
            &mut msgs,
        );
        cb.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: outside,
            },
            area,
            &mut msgs,
        );
        assert!(msgs.is_empty(), "按下拖出抬起不应触发");
    }

    #[test]
    fn right_click_does_not_emit() {
        let mut cb = Checkbox::new().on_toggle(|| 42u8);
        let mut msgs = MsgQueue::new();
        cb.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = box_area();
        let center = Point::new(7.0, 7.0);
        for pressed in [true, false] {
            cb.event(
                &Event::MouseInput {
                    button: MouseButton::Right,
                    pressed,
                    position: center,
                },
                area,
                &mut msgs,
            );
        }
        assert!(msgs.is_empty(), "右键不冒充左键 (P29)");
    }

    #[test]
    fn space_and_enter_emit_only_when_focused() {
        let mut cb = Checkbox::new().on_toggle(|| 99u8);
        let mut msgs = MsgQueue::new();
        cb.layout(
            Constraints::loose(Size::new(400.0, 400.0)),
            &mut TextBatch::new(),
        );
        let area = box_area();
        for key in [NamedKey::Space, NamedKey::Enter] {
            // 无焦点: 不产
            cb.event(
                &Event::Key {
                    key: Key::Named(key),
                    pressed: true,
                    shift: false,
                    ctrl: false,
                    alt: false,
                },
                area,
                &mut msgs,
            );
            assert!(msgs.is_empty(), "无焦点时键盘不应产出消息");
            // 持焦: 产
            cb.event(&Event::FocusIn, area, &mut msgs);
            cb.event(
                &Event::Key {
                    key: Key::Named(key),
                    pressed: true,
                    shift: false,
                    ctrl: false,
                    alt: false,
                },
                area,
                &mut msgs,
            );
            assert_eq!(msgs.len(), 1, "持焦按 Space/Enter 应产出消息");
            assert_eq!(msgs[0].downcast_ref::<u8>(), Some(&99));
            msgs.clear();
            cb.event(&Event::FocusOut, area, &mut msgs);
        }
    }

    /// 焦点环: FocusIn 多一笔、FocusOut 消失 (CloseButton「锁画出来了不只锁
    /// 标志位」同构 —— P8 的原始形态就是「有焦点态却没画」)。
    #[test]
    fn focus_ring_is_painted_only_while_focused() {
        let mut cb = Checkbox::new();
        let resting = paint_instances(&mut cb).0.len();
        let mut msgs = MsgQueue::new();
        cb.event(&Event::FocusIn, box_area(), &mut msgs);
        let focused = paint_instances(&mut cb).0.len();
        assert!(
            focused > resting,
            "持焦应多出焦点环 (resting {resting} vs focused {focused})"
        );
        cb.event(&Event::FocusOut, box_area(), &mut msgs);
        let after = paint_instances(&mut cb).0.len();
        assert_eq!(after, resting, "失焦后焦点环消失");
    }

    #[test]
    fn bind_syncs_checked_every_frame() {
        struct S {
            on: bool,
        }
        let mut cb = Checkbox::new().bind(|s: &S| s.on);
        cb.sync(&(S { on: false }));
        assert!(!cb.is_checked());
        cb.sync(&(S { on: true }));
        assert!(cb.is_checked(), "bind 应每帧读应用状态");
        cb.sync(&(S { on: false }));
        assert!(!cb.is_checked());
    }

    /// 造一支只改 accent 的场景主题 (Theme 方法无默认实现, 借 SceneTheme
    /// 免写 25 行委托)。
    fn scene_theme_with(accent: Color) -> crate::theme::SceneTheme {
        crate::theme::SceneTheme::new(crate::theme::ScenePalette {
            base: Color::BLACK,
            accent,
            text_primary: Color::WHITE,
            text_secondary: Color::WHITE,
            surface: Color::BLACK,
            surface_input: Color::BLACK,
            backdrop_light: Color::WHITE,
            backdrop_dark: Color::BLACK,
        })
    }

    #[test]
    fn bind_theme_syncs_colors_every_frame() {
        struct S {
            accent: Color,
        }
        let mut cb = Checkbox::new().bind_theme(|s: &S| scene_theme_with(s.accent));
        cb.sync(
            &(S {
                accent: Color::rgb(0.1, 0.2, 0.3),
            }),
        );
        assert_eq!(cb.colors.fill, Color::rgb(0.1, 0.2, 0.3));
        cb.sync(
            &(S {
                accent: Color::rgb(0.9, 0.8, 0.7),
            }),
        );
        assert_eq!(cb.colors.fill, Color::rgb(0.9, 0.8, 0.7), "主题色每帧刷新");
    }

    /// paint_box A/B 判罪锁: 勾中与未选的画法差异 —— 摘掉勾的两笔 = 精确红。
    #[test]
    fn paint_box_checked_adds_fill_and_check_strokes() {
        let colors = CheckboxColors::from_theme(&LightTheme);
        let mut unchecked = RectBatch::new();
        Checkbox::paint_box(&mut unchecked, box_area(), false, colors);
        let mut checked = RectBatch::new();
        Checkbox::paint_box(&mut checked, box_area(), true, colors);

        // 勾中: 存在 fill 色 (accent) 的填充实例, 且存在 check 色 (白) 的勾笔画实例
        let fill = rgba_of(colors.fill);
        let check = rgba_of(colors.check);
        let checked_colors = checked.instance_colors();
        assert!(checked_colors.contains(&fill), "勾中应有 accent 填充实例");
        assert!(
            checked_colors.iter().filter(|c| **c == check).count() > 2,
            "勾中应有两笔勾的圆点队列实例 (摘勾 = 本断言精确红)"
        );
        // 未选: 有 border 色边框实例; 无 fill 填充、无勾
        let unchecked_colors = unchecked.instance_colors();
        let border = rgba_of(colors.border);
        assert!(unchecked_colors.contains(&border), "未选应画 border 色边框");
        assert!(
            !unchecked_colors.contains(&fill),
            "未选盒内不许有 accent 填充"
        );
        assert!(!unchecked_colors.contains(&check), "未选不许画勾");
    }

    /// on_accent 反白套 (D3b): 勾中 = 白填充 + accent 勾 —— accent 底上
    /// accent 填充不可见, 高亮行必须反白。
    #[test]
    fn paint_box_on_accent_inverts_colors() {
        let accent = LightTheme.accent();
        let colors = CheckboxColors::on_accent(accent);
        let mut checked = RectBatch::new();
        Checkbox::paint_box(&mut checked, box_area(), true, colors);
        let colors_vec = checked.instance_colors();
        assert!(
            colors_vec.iter().any(|c| *c == rgba_of(Color::WHITE)),
            "高亮行勾中 = 白填充"
        );
        assert!(
            colors_vec.iter().any(|c| *c == rgba_of(accent)),
            "高亮行勾 = accent 色"
        );
        let mut unchecked = RectBatch::new();
        Checkbox::paint_box(&mut unchecked, box_area(), false, colors);
        assert!(
            unchecked
                .instance_colors()
                .iter()
                .any(|c| *c == rgba_of(Color::WHITE)),
            "高亮行未选 = 白边框"
        );
    }

    /// widget paint 与静态 paint_box 同一画法入口 (单真源): widget 勾中态
    /// 的实例集 = 直接 paint_box 勾中态 (同区域同颜色套)。
    #[test]
    fn widget_paint_delegates_to_paint_box() {
        let mut cb = Checkbox::new();
        cb.checked = true;
        let via_widget = paint_instances(&mut cb);
        let mut direct = RectBatch::new();
        Checkbox::paint_box(
            &mut direct,
            box_area(),
            true,
            CheckboxColors::from_theme(&LightTheme),
        );
        assert_eq!(via_widget.0, direct.instance_rects().to_vec());
        assert_eq!(via_widget.1, direct.instance_colors().to_vec());
    }

    #[test]
    fn default_is_unchecked() {
        assert!(!Checkbox::new().is_checked());
    }
}
