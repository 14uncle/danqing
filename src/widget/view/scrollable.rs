//! @author 十四叔
//! @date 2026/07/18

//! 滚动容器:允许子组件在垂直/水平方向上滚动。
//!
//! `Scrollable` 负责维护滚动偏移、视口裁剪与滚轮事件;
//! 子组件只需报告自然内容尺寸。

use std::cell::Cell;

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 滚动方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxis {
    /// 仅垂直滚动。
    Vertical,
    /// 仅水平滚动。
    Horizontal,
    /// 双向滚动。
    Both,
}

/// 子组件在滚动轴上允许的最大尺寸。
///
/// 用有限大值代替 `f32::INFINITY`,避免 Flow 等布局算法在分配 Fill 权重时溢出。
const MAX_CONTENT_SIZE: f32 = 1_000_000.0;

/// 可见性区间绑定闭包: 每帧从应用状态读取 (revision, top, height)。
///
/// 语义见 [`Scrollable::bind_visible`]。
type VisibleBinding = Box<dyn Fn(&dyn std::any::Any) -> (u64, f32, f32)>;

/// 滚动容器。
pub struct Scrollable {
    child: Node,
    axis: ScrollAxis,
    scroll_offset: Point,
    scroll_speed: f32,
    child_size: Size,
    viewport_size: Size,
    /// 滚动条轨道颜色。
    track_color: Color,
    /// 滚动条滑块颜色。
    thumb_color: Color,
    /// 滚动条宽度。
    track_width: f32,
    /// 滑块圆角半径。
    thumb_radius: f32,
    /// 自身绝对矩形,在 paint 阶段缓存,供 hit_area 使用。
    area: Cell<Rect>,
    /// 可见性区间绑定 (键盘选中跟随等)。
    visible_binding: Option<VisibleBinding>,
    /// 已应用的绑定 revision; 仅在 revision 变化时纠偏, 滚轮自由。
    applied_rev: u64,
}

impl Scrollable {
    /// 创建滚动容器,默认垂直滚动,使用默认浅色主题。
    pub fn new(child: impl Widget + 'static) -> Self {
        Self::themed(&LightTheme, child)
    }

    /// 使用指定主题创建滚动容器。
    pub fn themed(theme: &impl Theme, child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            axis: ScrollAxis::Vertical,
            scroll_offset: Point::ZERO,
            scroll_speed: 40.0,
            child_size: Size::ZERO,
            viewport_size: Size::ZERO,
            track_color: theme.divider(),
            thumb_color: theme.text_secondary(),
            track_width: theme.spacing_xs(),
            thumb_radius: theme.radius_sm(),
            area: Cell::new(Rect::default()),
            visible_binding: None,
            applied_rev: 0,
        }
    }

    /// 设置滚动方向。
    pub fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    /// 设置滚轮每次滚动的逻辑像素数。
    pub fn scroll_speed(mut self, speed: f32) -> Self {
        self.scroll_speed = speed;
        self
    }

    /// 当前滚动偏移。
    pub fn scroll_offset(&self) -> Point {
        self.scroll_offset
    }

    /// 绑定「保持可见」区间: 每帧 `sync` 时经闭包读取 `(revision, top, height)`
    /// (内容坐标), revision 变化时调整滚动偏移使该区间落入视口。
    ///
    /// 典型用法: 键盘选中行跟随 —— 应用把选中行的 revision / y / 行高
    /// 放进状态, 选中变化 (revision 递增) 时容器把该行滚入视口。
    /// revision 不变则不纠偏: 滚轮滚远后不会被每帧绑回, 滚轮自由。
    ///
    /// 仅对纵向滚动生效 (纠偏只调 y 轴); 请勿用于 Horizontal / Both 容器。
    ///
    /// 状态类型 `S` 须与 [`App`](crate::App) 实现者一致。
    pub fn bind_visible<S: 'static>(mut self, f: impl Fn(&S) -> (u64, f32, f32) + 'static) -> Self {
        self.visible_binding = Some(Box::new(move |state: &dyn std::any::Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Scrollable::bind_visible 绑定的状态类型不匹配");
            f(state)
        }));
        self
    }

    /// 调整偏移使纵向区间 [top, top+height] 落入视口 (最小纠偏)。
    fn ensure_visible(&mut self, top: f32, height: f32) {
        let bottom = top + height;
        let view_bottom = self.scroll_offset.y + self.viewport_size.height;
        if top < self.scroll_offset.y {
            self.scroll_offset.y = top;
        } else if bottom > view_bottom {
            self.scroll_offset.y = bottom - self.viewport_size.height;
        }
        self.clamp_offset();
    }

    fn max_offset(&self) -> Point {
        Point::new(
            (self.child_size.width - self.viewport_size.width).max(0.0),
            (self.child_size.height - self.viewport_size.height).max(0.0),
        )
    }

    fn clamp_offset(&mut self) {
        let max = self.max_offset();
        self.scroll_offset.x = self.scroll_offset.x.clamp(0.0, max.x);
        self.scroll_offset.y = self.scroll_offset.y.clamp(0.0, max.y);
    }

    fn child_constraints(&self) -> Constraints {
        match self.axis {
            ScrollAxis::Vertical => {
                Constraints::loose(Size::new(self.viewport_size.width, MAX_CONTENT_SIZE))
            }
            ScrollAxis::Horizontal => {
                Constraints::loose(Size::new(MAX_CONTENT_SIZE, self.viewport_size.height))
            }
            ScrollAxis::Both => Constraints::loose(Size::new(MAX_CONTENT_SIZE, MAX_CONTENT_SIZE)),
        }
    }

    fn transform_event(&self, event: &Event) -> Option<Event> {
        match event {
            Event::CursorMoved(p) => Some(Event::CursorMoved(Point::new(
                p.x + self.scroll_offset.x,
                p.y + self.scroll_offset.y,
            ))),
            Event::MouseInput {
                button,
                pressed,
                position,
            } => Some(Event::MouseInput {
                button: *button,
                pressed: *pressed,
                position: Point::new(
                    position.x + self.scroll_offset.x,
                    position.y + self.scroll_offset.y,
                ),
            }),
            Event::MouseWheel { delta, position } => Some(Event::MouseWheel {
                delta: *delta,
                position: Point::new(
                    position.x + self.scroll_offset.x,
                    position.y + self.scroll_offset.y,
                ),
            }),
            // 无位置事件直接转发。
            _ => Some(event.clone()),
        }
    }

    fn handle_wheel(&mut self, delta: (f32, f32)) {
        match self.axis {
            ScrollAxis::Vertical => {
                self.scroll_offset.y -= delta.1 * self.scroll_speed;
            }
            ScrollAxis::Horizontal => {
                self.scroll_offset.x -= delta.0 * self.scroll_speed;
            }
            ScrollAxis::Both => {
                if delta.1 != 0.0 {
                    self.scroll_offset.y -= delta.1 * self.scroll_speed;
                } else if delta.0 != 0.0 {
                    self.scroll_offset.x -= delta.0 * self.scroll_speed;
                }
            }
        }
        self.clamp_offset();
    }

    fn draw_scrollbar(&self, area: Rect, rects: &mut RectBatch) {
        let track_color = self.track_color;
        let thumb_color = self.thumb_color;
        let track_width = self.track_width;

        // 垂直滚动条
        if self.child_size.height > self.viewport_size.height {
            let ratio = self.viewport_size.height / self.child_size.height;
            let thumb_height = (self.viewport_size.height * ratio).max(track_width);
            let max_offset_y = (self.child_size.height - self.viewport_size.height).max(0.0);
            let thumb_offset_y = if max_offset_y > 0.0 {
                (self.scroll_offset.y / max_offset_y) * (self.viewport_size.height - thumb_height)
            } else {
                0.0
            };
            let track_x = area.origin.x + area.size.width - track_width;
            rects.push_rect(
                Rect::from_xywh(track_x, area.origin.y, track_width, area.size.height),
                track_color,
                0.0,
            );
            rects.push_rect(
                Rect::from_xywh(
                    track_x,
                    area.origin.y + thumb_offset_y,
                    track_width,
                    thumb_height,
                ),
                thumb_color,
                self.thumb_radius,
            );
        }

        // 水平滚动条
        if self.child_size.width > self.viewport_size.width {
            let ratio = self.viewport_size.width / self.child_size.width;
            let thumb_width = (self.viewport_size.width * ratio).max(track_width);
            let max_offset_x = (self.child_size.width - self.viewport_size.width).max(0.0);
            let thumb_offset_x = if max_offset_x > 0.0 {
                (self.scroll_offset.x / max_offset_x) * (self.viewport_size.width - thumb_width)
            } else {
                0.0
            };
            let track_y = area.origin.y + area.size.height - track_width;
            rects.push_rect(
                Rect::from_xywh(area.origin.x, track_y, area.size.width, track_width),
                track_color,
                0.0,
            );
            rects.push_rect(
                Rect::from_xywh(
                    area.origin.x + thumb_offset_x,
                    track_y,
                    thumb_width,
                    track_width,
                ),
                thumb_color,
                self.thumb_radius,
            );
        }
    }
}

impl Widget for Scrollable {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.child.sync(state);
        if let Some(binding) = &self.visible_binding {
            let (rev, top, height) = binding(state);
            // 视口未就绪 (首帧 sync 在 layout 前, viewport 仍为 ZERO) 时不消费
            // revision: 此时 ensure_visible 算出的偏移会被 clamp 归零, 纠偏无效,
            // 若消费了 rev 该项将永远滚不进视口, 直到下一次 rev 变化。
            if rev != self.applied_rev && self.viewport_size.height > 0.0 {
                self.applied_rev = rev;
                self.ensure_visible(top, height);
            }
        }
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        self.child.animate(ctx);
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.viewport_size = constraints.max();
        self.child_size = self.child.layout(self.child_constraints(), texts);
        // 子组件可能比视口小;滚动偏移需要重新限幅。
        self.clamp_offset();
        self.area.set(Rect::new(Point::ZERO, self.viewport_size));
        self.viewport_size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        self.area.set(area);

        rects.push_clip(area);
        texts.push_clip(area);

        let child_area = Rect::new(
            Point::new(
                area.origin.x - self.scroll_offset.x,
                area.origin.y - self.scroll_offset.y,
            ),
            self.child_size,
        );
        self.child.paint(child_area, rects, texts);

        texts.pop_clip();
        rects.pop_clip();

        // 绘制滚动条(在裁剪区外,不需要再裁剪)。
        self.draw_scrollbar(area, rects);
    }

    fn paint_image(&self, area: Rect, images: &mut crate::render::ImageBatch) {
        // 与 paint 一致: 图像同样裁剪到视口, 滚出视口的不进批次
        images.push_clip(area);
        let child_area = Rect::new(
            Point::new(
                area.origin.x - self.scroll_offset.x,
                area.origin.y - self.scroll_offset.y,
            ),
            self.child_size,
        );
        self.child.paint_image(child_area, images);
        images.pop_clip();
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.area.set(area);
        let inside = match event.position() {
            Some(p) => area.contains(p),
            None => false,
        };

        match event {
            Event::CursorLeft => {
                self.child.event(event, area, msgs);
                return EventResult::Ignored;
            }
            Event::MouseWheel { delta, .. } if inside => {
                self.handle_wheel(*delta);
                return EventResult::Consumed;
            }
            _ => {}
        }

        if !inside {
            return EventResult::Ignored;
        }

        let transformed = match self.transform_event(event) {
            Some(e) => e,
            None => return EventResult::Ignored,
        };

        // 对鼠标按键,只有真正落在子组件内容区(含滚动偏移)才消费;
        // 否则仍视为在视口内点击,消费事件防止冒泡到应用层。
        let child_result = self.child.event(&transformed, area, msgs);
        if child_result == EventResult::Consumed {
            child_result
        } else {
            EventResult::Consumed
        }
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}

#[cfg(test)]
impl Scrollable {
    /// 当前轨道颜色(测试用)。
    pub(crate) fn track_color(&self) -> Color {
        self.track_color
    }

    /// 当前滑块颜色(测试用)。
    pub(crate) fn thumb_color(&self) -> Color {
        self.thumb_color
    }

    /// 当前滚动条宽度(测试用)。
    pub(crate) fn track_width(&self) -> f32 {
        self.track_width
    }

    /// 当前滑块圆角(测试用)。
    pub(crate) fn thumb_radius(&self) -> f32 {
        self.thumb_radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::widget::Box as UiBox;

    #[test]
    fn scrollable_uses_theme_defaults() {
        let scroll = Scrollable::new(UiBox::new(Color::BLACK).size(50.0, 500.0));
        assert_eq!(scroll.track_color(), LightTheme.divider());
        assert_eq!(scroll.thumb_color(), LightTheme.text_secondary());
        assert_eq!(scroll.track_width(), LightTheme.spacing_xs());
        assert_eq!(scroll.thumb_radius(), LightTheme.radius_sm());
    }

    #[test]
    fn vertical_scroll_clamps_offset() {
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(50.0, 500.0));
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
        assert_eq!(scroll.viewport_size, Size::new(100.0, 100.0));
        assert_eq!(scroll.child_size, Size::new(50.0, 500.0));

        // 滚轮向下滚动 1000 像素,应被限幅到 content - viewport = 400。
        scroll.handle_wheel((0.0, -25.0));
        assert!((scroll.scroll_offset.y - 400.0).abs() < f32::EPSILON);

        // 滚轮向上回滚,应回到 0。
        scroll.handle_wheel((0.0, 25.0));
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
    }

    #[test]
    fn wheel_outside_viewport_is_ignored() {
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(50.0, 500.0));
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);

        let event = Event::MouseWheel {
            delta: (0.0, -5.0),
            position: Point::new(200.0, 200.0),
        };
        let result = scroll.event(
            &event,
            Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
            &mut Vec::new(),
        );
        assert_eq!(result, EventResult::Ignored);
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
    }

    #[test]
    fn horizontal_axis_uses_x_delta() {
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(500.0, 50.0))
            .axis(ScrollAxis::Horizontal);
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
        scroll.handle_wheel((-5.0, 0.0));
        assert!(scroll.scroll_offset.x > 0.0);
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
    }

    #[test]
    fn bind_visible_scrolls_into_view_only_on_revision_change() {
        struct State {
            rev: u64,
            top: f32,
        }
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(100.0, 1000.0))
            .bind_visible(|s: &State| (s.rev, s.top, 50.0));
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);

        // 初次应用: 目标区间 [800, 850] 在视口 [0, 100] 之下, 纠偏为底对齐。
        scroll.sync(&State { rev: 1, top: 800.0 });
        assert!((scroll.scroll_offset.y - 750.0).abs() < f32::EPSILON);

        // revision 不变: 滚轮滚回顶部后不绑回 (滚轮自由)。
        scroll.handle_wheel((0.0, 25.0));
        scroll.sync(&State { rev: 1, top: 800.0 });
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);

        // revision 变化: 新区间 [0, 50] 在当前视口之上, 纠偏为顶对齐。
        scroll.handle_wheel((0.0, -25.0));
        scroll.sync(&State { rev: 2, top: 0.0 });
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
    }

    /// 生命周期是 sync → layout: 首帧 sync 时视口仍为 ZERO, 此时不得消费
    /// revision —— 否则纠偏被 clamp 归零且 rev 已记, 该项永远滚不进视口。
    #[test]
    fn bind_visible_defers_revision_until_viewport_ready() {
        struct State {
            rev: u64,
            top: f32,
        }
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(UiBox::new(Color::BLACK).size(100.0, 1000.0))
            .bind_visible(|s: &State| (s.rev, s.top, 50.0));

        // layout 前 sync: revision 不被消费, 偏移不变 (无可纠偏的视口)。
        scroll.sync(&State { rev: 1, top: 800.0 });
        assert!(scroll.scroll_offset.y.abs() < f32::EPSILON);
        assert_eq!(scroll.applied_rev, 0);

        // layout 后 rev 仍未消费: 下一帧 sync 正常纠偏 (底对齐 750)。
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
        scroll.sync(&State { rev: 1, top: 800.0 });
        assert!((scroll.scroll_offset.y - 750.0).abs() < f32::EPSILON);
        assert_eq!(scroll.applied_rev, 1);
    }

    /// 子组件在子坐标系底部推图像; 视口高 100。
    struct BottomImage;
    impl Widget for BottomImage {
        fn layout(&mut self, _c: Constraints, _t: &mut TextBatch) -> Size {
            Size::new(100.0, 1000.0)
        }
        fn paint(&self, _a: Rect, _r: &mut RectBatch, _t: &mut TextBatch) {}
        fn paint_image(&self, area: Rect, images: &mut crate::render::ImageBatch) {
            let data = [255u8; 4];
            images.push_image(
                &data,
                1,
                1,
                Rect::from_xywh(area.origin.x, area.origin.y + 950.0, 40.0, 40.0),
            );
        }
    }

    /// paint 有 push_clip/pop_clip, paint_image 也必须裁剪到视口,
    /// 否则滚出视口的图像仍会被绘制 (clipboard 列表缩略图场景踩实)。
    #[test]
    fn paint_image_clips_to_viewport() {
        let mut texts = TextBatch::new();
        let mut scroll = Scrollable::new(BottomImage);
        scroll.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
        let viewport = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);

        // 未滚动: 图像在子坐标 950..990, 完全在视口之下 → 应被裁掉
        let mut images = crate::render::ImageBatch::new();
        scroll.paint_image(viewport, &mut images);
        assert_eq!(images.len(), 0, "视口外的图像不应进入批次");

        // 滚动 900: 图像落在视口 y 50..90 → 保留
        scroll.handle_wheel((0.0, -900.0 / 25.0)); // 每单位 25px
        let mut images = crate::render::ImageBatch::new();
        scroll.paint_image(viewport, &mut images);
        assert_eq!(images.len(), 1, "滚入视口的图像应保留");
    }
}
