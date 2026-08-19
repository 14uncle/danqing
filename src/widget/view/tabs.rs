//! @author 十四叔
//! @date 2026/08/18
//!
//! Tabs 组件：带可视化 tab 栏的多面板切换容器。
//!
//! 水平顶部 tab 栏 + 面板切换。tab 栏自绘 (文字 + 选中指示线)，
//! 面板切换逻辑与 [`Switcher`] 一致：sync 传播全部子组件，
//! layout/paint/event 只作用于 active 面板。

use std::any::Any;

use crate::app::AnimationCtx;
use crate::event::{Event, MouseButton};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Color, Constraints, Point, Rect, Size, Theme};

/// active 索引绑定闭包：每帧从应用状态读取。
type ActiveBinding = Box<dyn Fn(&dyn Any) -> usize>;
/// tab 切换时产出的应用消息工厂。
type ChangeFactory = Box<dyn Fn(usize) -> Box<dyn Any>>;
/// icon 数据 (RGBA 像素, 宽, 高)。
type IconData = (Vec<u8>, u32, u32);

/// 带 tab 栏的多面板切换容器。
///
/// 与 [`Switcher`](crate::widget::Switcher) 类似，保留全部子组件实例，
/// 只让 active 子组件参与布局 / 绘制 / 事件。额外在顶部渲染可点击的 tab 栏。
///
/// 用法：
/// ```ignore
/// Tabs::new(&theme)
///     .tab("常规")
///     .tab("设置")
///     .child(常规面板)
///     .child(设置面板)
///     .bind(|app: &App| app.tab_index)
/// ```
pub struct Tabs {
    /// tab 标签文字。
    labels: Vec<String>,
    /// tab 图标 (与 labels 一一对应，None 表示无图标)。
    icons: Vec<Option<IconData>>,
    /// 面板子组件 (与 labels 一一对应)。
    children: Vec<Node>,
    /// 当前 active 索引。
    active: usize,
    /// 应用状态绑定闭包。
    binding: Option<ActiveBinding>,
    /// tab 栏中各 tab 的区域 (layout 缓存，点击判定用)。
    tab_areas: Vec<Rect>,
    /// 鼠标悬停的 tab 索引。
    hovered: Option<usize>,
    /// active 子组件的 layout 尺寸缓存。
    active_size: Size,
    /// tab 切换时的消息工厂。
    on_change: Option<ChangeFactory>,
    /// tab 栏高度 (含 indicator)。
    tab_bar_height: f32,
    /// 选中 tab 文字色。
    color_active: Color,
    /// 未选中 tab 文字色。
    color_inactive: Color,
    /// 选中指示线颜色。
    color_indicator: Color,
    /// hover 文字色。
    color_hover: Color,
    /// tab 文字字号。
    font_size: u16,
    /// icon 尺寸 (正方形，逻辑像素)。
    icon_size: f32,
    /// icon 与文字间距。
    icon_gap: f32,
    /// tab icon 目标区域 (layout 缓存，paint_image 用；与 icons 一一对应)。
    icon_rects: Vec<Option<Rect>>,
    /// tab 点击热区 (layout 缓存，hit_test 用；与 labels 一一对应)。
    hit_rects: Vec<Rect>,
    /// 面板顶部间距 (theme token)。
    panel_pad: f32,
}

/// tab 栏底部指示线高度。
const INDICATOR_H: f32 = 2.0;
/// 指示线文字两侧延伸量。
const INDICATOR_PAD: f32 = 8.0;
/// 面板内容与 tab 栏之间的间距 (取 theme spacing_md token，约 12px)。
fn panel_top_pad(theme: &impl Theme) -> f32 {
    theme.spacing_md()
}

impl Tabs {
    /// 创建空 Tabs, 从 theme 读取颜色和字号 token。
    pub fn new(theme: &impl Theme) -> Self {
        let font_size = theme.font_size_body();
        // tab 栏高度：字号 + 上下 padding
        let tab_bar_height = font_size as f32 + 16.0;
        Self {
            labels: Vec::new(),
            icons: Vec::new(),
            children: Vec::new(),
            active: 0,
            binding: None,
            tab_areas: Vec::new(),
            hovered: None,
            active_size: Size::ZERO,
            on_change: None,
            tab_bar_height,
            color_active: theme.accent(),
            color_inactive: theme.text_secondary(),
            color_indicator: theme.accent(),
            color_hover: theme.text_primary(),
            font_size,
            icon_size: 16.0,
            icon_gap: 4.0,
            icon_rects: Vec::new(),
            hit_rects: Vec::new(),
            panel_pad: panel_top_pad(theme),
        }
    }

    /// 追加一个 tab 标签。
    pub fn tab(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// 追加一个面板 (与 tab 标签一一对应)。
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// 为最近一次追加的 tab 设置图标 (RGBA 像素数据, 宽, 高)。
    ///
    /// 图标在 tab 栏中渲染于文字左侧，宽度计入 tab 标题总宽。
    /// 未设 icon 的 tab 在 measure_tabs 中自动填 None。
    pub fn icon(mut self, data: Vec<u8>, width: u32, height: u32) -> Self {
        // 补齐到 labels 同长 (前面没设 icon 的 tab 填 None)
        while self.icons.len() < self.labels.len() {
            self.icons.push(None);
        }
        // 覆盖当前位置 (labels 刚增长时 len 刚好对齐；连续调 icon 时覆盖上一个)
        let idx = self.labels.len().saturating_sub(1);
        if idx < self.icons.len() {
            self.icons[idx] = Some((data, width, height));
        } else {
            self.icons.push(Some((data, width, height)));
        }
        self
    }

    /// 设置 icon 尺寸 (正方形，逻辑像素，默认 16)。
    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    /// 设置 icon 与文字间距 (逻辑像素，默认 4)。
    pub fn icon_gap(mut self, gap: f32) -> Self {
        self.icon_gap = gap;
        self
    }

    /// 设置初始 active 索引。
    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    /// 绑定应用状态：每帧 `sync` 时经闭包读取 active 索引。
    pub fn bind<S: 'static>(mut self, f: impl Fn(&S) -> usize + 'static) -> Self {
        self.binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Tabs 绑定的状态类型不匹配");
            f(state)
        }));
        self
    }

    /// 设置 tab 切换回调：点击 tab 时经闭包产出消息，推入应用消息队列。
    ///
    /// 闭包接收新选中的 tab 索引，返回应用消息。
    pub fn on_change<M: 'static>(mut self, f: impl Fn(usize) -> M + 'static) -> Self {
        self.on_change = Some(Box::new(move |idx| Box::new(f(idx)) as Box<dyn Any>));
        self
    }

    /// 钳制 active 到合法范围。
    fn clamp_active(&mut self) {
        let len = self.children.len();
        if len == 0 {
            self.active = 0;
        } else if self.active >= len {
            self.active = len - 1;
        }
    }

    /// active 子组件的可见切片范围。
    fn active_range(&self) -> std::ops::Range<usize> {
        if self.children.is_empty() {
            return 0..0;
        }
        let active = self.active.min(self.children.len() - 1);
        active..active + 1
    }

    /// 计算各 tab 的 (icon+文字) 总宽度并填充 tab_areas / icon_rects (layout 阶段调用)。
    ///
    /// 布局方式：从左往右紧凑排列，每个 tab 宽度 = title_w + 左右 padding。
    /// 有 icon 时: 标题宽 = icon_size + icon_gap + text_w;
    /// 无 icon 时: 标题宽 = text_w。
    fn measure_tabs(&mut self, texts: &mut TextBatch) {
        let n = self.labels.len();
        if n == 0 {
            self.tab_areas.clear();
            self.icon_rects.clear();
            return;
        }
        // 补齐 icons 长度
        while self.icons.len() < n {
            self.icons.push(None);
        }
        self.tab_areas.resize(n, Rect::default());
        self.icon_rects.resize(n, None);
        self.hit_rects.resize(n, Rect::default());
        let tab_pad = 12.0; // 每个 tab 左右 padding
        let mut x = 0.0f32;
        for (i, label) in self.labels.iter().enumerate() {
            let text_w = texts.measure(label, self.font_size);
            let has_icon = self.icons.get(i).and_then(|o| o.as_ref()).is_some();
            let title_w = if has_icon {
                self.icon_size + self.icon_gap + text_w
            } else {
                text_w
            };
            let title_x = x + tab_pad;
            let line_h = texts.line_height(f32::from(self.font_size));
            let baseline =
                (self.tab_bar_height - line_h) / 2.0 + texts.ascent(f32::from(self.font_size));
            // origin.x 存标题起始 x (用于 paint 定位 icon 和文字)
            // size.width 存标题总宽 (用于指示线宽度计算)
            self.tab_areas[i] = Rect::new(Point::new(title_x, baseline), Size::new(title_w, 0.0));
            // icon 垂直居中于 tab 栏 (存储相对坐标，paint 时叠加 area.origin)
            if has_icon {
                let icon_y = (self.tab_bar_height - self.icon_size) / 2.0;
                self.icon_rects[i] = Some(Rect::new(
                    Point::new(title_x, icon_y),
                    Size::new(self.icon_size, self.icon_size),
                ));
            } else {
                self.icon_rects[i] = None;
            }
            // 点击热区：从 tab 左 padding 到标题右 + 右 padding
            self.hit_rects[i] = Rect::new(
                Point::new(x, 0.0),
                Size::new(title_w + tab_pad * 2.0, self.tab_bar_height),
            );
            // 下一个 tab 起始位置
            x = title_x + title_w + tab_pad;
        }
    }

    /// 将屏幕坐标转换为 tab 索引 (无命中返回 None)。
    fn hit_test(&self, pos: Point, area: Rect) -> Option<usize> {
        // 将屏幕坐标转换为 tab 栏相对坐标
        let rel = Point::new(pos.x - area.origin.x, pos.y - area.origin.y);
        for (i, hit) in self.hit_rects.iter().enumerate() {
            if hit.contains(rel) {
                return Some(i);
            }
        }
        None
    }
}

impl Default for Tabs {
    fn default() -> Self {
        // 默认主题：LightTheme
        Self::new(&crate::theme::LightTheme)
    }
}

impl Widget for Tabs {
    fn sync(&mut self, state: &dyn Any) {
        let prev_active = self.active;
        // sync 传播给全部子组件 (状态保鲜)
        for child in &mut self.children {
            child.sync(state);
        }
        // 读取绑定
        if let Some(binding) = &self.binding {
            self.active = binding(state);
        }
        self.clamp_active();
        // 面板切换时重置旧面板焦点
        if self.active != prev_active {
            if let Some(old) = self.children.get_mut(prev_active) {
                old.reset_focus();
            }
        }
    }

    fn animate(&mut self, ctx: &AnimationCtx) {
        for child in &mut self.children {
            child.animate(ctx);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.clamp_active();
        let total_w = constraints.max().width;
        // 无 tab 时直接返回零尺寸
        if self.labels.is_empty() {
            self.tab_areas.clear();
            self.active_size = Size::ZERO;
            return Size::ZERO;
        }
        // 测量 tab 栏
        self.measure_tabs(texts);
        // 布局 active 面板 (扣除 tab 栏高度 + 面板顶部间距)
        let child_max_h =
            (constraints.max().height - self.tab_bar_height - self.panel_pad).max(0.0);
        let child_constraints = Constraints::loose(Size::new(total_w, child_max_h));
        match self.children.get_mut(self.active) {
            Some(child) => {
                self.active_size = child.layout(child_constraints, texts);
            }
            None => {
                self.active_size = Size::ZERO;
            }
        }
        let total_h = self.tab_bar_height + self.panel_pad + self.active_size.height;
        constraints.constrain(Size::new(total_w, total_h))
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let n = self.labels.len();
        if n == 0 {
            return;
        }

        // ── tab 栏 ──
        for (i, label) in self.labels.iter().enumerate() {
            // 文字
            if let Some(text_info) = self.tab_areas.get(i) {
                let has_icon = self.icons.get(i).and_then(|o| o.as_ref()).is_some();
                // 有 icon 时文字右移 icon_size + icon_gap
                let text_x = area.origin.x
                    + text_info.origin.x
                    + if has_icon {
                        self.icon_size + self.icon_gap
                    } else {
                        0.0
                    };
                let text_y = area.origin.y + text_info.origin.y;
                let color = if self.active == i {
                    self.color_active
                } else if self.hovered == Some(i) {
                    self.color_hover
                } else {
                    self.color_inactive
                };
                texts.push_text(label, text_x, text_y, self.font_size, color);

                // 选中指示线：以标题区域为基准居中
                if self.active == i {
                    let indicator_w = text_info.size.width + INDICATOR_PAD * 2.0;
                    let indicator_x = area.origin.x + text_info.origin.x - INDICATOR_PAD;
                    let indicator_y = area.origin.y + self.tab_bar_height - INDICATOR_H;
                    rects.push_rect(
                        Rect::new(
                            Point::new(indicator_x, indicator_y),
                            Size::new(indicator_w, INDICATOR_H),
                        ),
                        self.color_indicator,
                        1.0,
                    );
                }
            }
        }

        // ── active 面板 ──
        let panel_area = Rect::new(
            Point::new(
                area.origin.x,
                area.origin.y + self.tab_bar_height + self.panel_pad,
            ),
            self.active_size,
        );
        if let Some(child) = self.children.get(self.active) {
            child.paint(panel_area, rects, texts);
        }
    }

    fn paint_image(&self, area: Rect, images: &mut crate::render::ImageBatch) {
        // ── tab 栏 icon ──
        for (i, icon_opt) in self.icons.iter().enumerate() {
            if let Some((data, w, h)) = icon_opt {
                if let Some(rel_rect) = self.icon_rects.get(i).and_then(|r| *r) {
                    let dst = Rect::new(
                        Point::new(
                            area.origin.x + rel_rect.origin.x,
                            area.origin.y + rel_rect.origin.y,
                        ),
                        rel_rect.size,
                    );
                    images.push_image(data, *w, *h, dst);
                }
            }
        }
        // ── active 面板 ──
        let panel_area = Rect::new(
            Point::new(
                area.origin.x,
                area.origin.y + self.tab_bar_height + self.panel_pad,
            ),
            self.active_size,
        );
        if let Some(child) = self.children.get(self.active) {
            child.paint_image(panel_area, images);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        // 先处理 tab 栏交互
        match event {
            Event::CursorMoved(p) => {
                self.hovered = self.hit_test(*p, area);
                if self.hovered.is_some() {
                    return EventResult::Consumed;
                }
            }
            Event::CursorLeft => {
                self.hovered = None;
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position,
            } => {
                if let Some(idx) = self.hit_test(*position, area) {
                    if idx != self.active {
                        self.active = idx;
                        self.clamp_active();
                        // 产出切换消息
                        if let Some(factory) = &self.on_change {
                            msgs.push(factory(idx));
                        }
                    }
                    return EventResult::Consumed;
                }
            }
            _ => {}
        }

        // 转发给 active 面板
        let panel_area = Rect::new(
            Point::new(
                area.origin.x,
                area.origin.y + self.tab_bar_height + self.panel_pad,
            ),
            self.active_size,
        );
        match self.children.get_mut(self.active) {
            Some(child) => child.event(event, panel_area, msgs),
            None => EventResult::Ignored,
        }
    }

    fn focusable(&self) -> bool {
        // tab 栏本身不可聚焦，但面板内容可能可聚焦
        false
    }

    fn children(&self) -> &[Node] {
        &self.children[self.active_range()]
    }

    fn children_mut(&mut self) -> &mut [Node] {
        let range = self.active_range();
        &mut self.children[range]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Constraints;
    use crate::widget::Text;

    struct State {
        active: usize,
    }

    fn loose() -> Constraints {
        Constraints::loose(Size::new(400.0, 300.0))
    }

    #[test]
    fn empty_tabs_lays_out_to_zero() {
        let mut tabs = Tabs::default();
        let mut texts = TextBatch::default();
        let size = tabs.layout(loose(), &mut texts);
        assert_eq!(size, Size::ZERO);
        assert!(tabs.children().is_empty());
    }

    #[test]
    fn out_of_range_active_clamps_to_last() {
        let mut tabs = Tabs::default()
            .tab("a")
            .tab("b")
            .child(Text::new("panel a"))
            .child(Text::new("panel b"))
            .active(9);
        let mut texts = TextBatch::default();
        tabs.layout(loose(), &mut texts);
        assert_eq!(tabs.active, 1, "active 应钳制到最后一个");
    }

    #[test]
    fn binding_drives_active() {
        let mut tabs = Tabs::default()
            .tab("a")
            .tab("b")
            .child(Text::new("panel a"))
            .child(Text::new("panel b"))
            .bind(|s: &State| s.active);

        let mut texts = TextBatch::default();
        tabs.sync(&State { active: 1 });
        tabs.layout(loose(), &mut texts);
        assert_eq!(tabs.active, 1);
        assert_eq!(tabs.children().len(), 1);
    }

    #[test]
    fn binding_clamps_on_sync() {
        let mut tabs = Tabs::default()
            .tab("a")
            .child(Text::new("panel a"))
            .bind(|s: &State| s.active);

        let mut texts = TextBatch::default();
        tabs.sync(&State { active: 42 });
        tabs.layout(loose(), &mut texts);
        assert_eq!(tabs.active, 0, "越界应钳制");
    }

    #[test]
    fn tab_count_matches_child_count() {
        let tabs = Tabs::default()
            .tab("a")
            .tab("b")
            .tab("c")
            .child(Text::new("1"))
            .child(Text::new("2"))
            .child(Text::new("3"));
        assert_eq!(tabs.labels.len(), tabs.children.len());
    }

    #[test]
    fn paint_only_renders_active_panel() {
        // 用 log 验证 paint 只调用 active 面板
        let mut tabs = Tabs::default()
            .tab("a")
            .tab("b")
            .child(Text::new("hidden"))
            .child(Text::new("shown"))
            .active(1);
        let mut texts = TextBatch::default();
        let size = tabs.layout(loose(), &mut texts);
        let mut rects = RectBatch::default();
        tabs.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);
        // paint 不 panic 即可 (Text 无 log, 但不调用 hidden 面板的 paint)
    }

    #[test]
    fn event_only_reaches_active_panel() {
        let mut tabs = Tabs::default()
            .tab("a")
            .tab("b")
            .child(Text::new("hidden"))
            .child(Text::new("shown"))
            .active(1);
        let mut texts = TextBatch::default();
        let size = tabs.layout(loose(), &mut texts);
        let mut msgs = MsgQueue::new();
        // 点击面板区域 (tab 栏下方) 不应触发 tab 切换
        let panel_click = Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(100.0, 50.0), // tab 栏下方
        };
        let _ = tabs.event(&panel_click, Rect::new(Point::ZERO, size), &mut msgs);
        assert_eq!(tabs.active, 1, "面板区域点击不应改变 active");
    }

    #[test]
    fn click_tab_switches_active() {
        let mut tabs = Tabs::default()
            .tab("a")
            .tab("b")
            .child(Text::new("panel a"))
            .child(Text::new("panel b"))
            .active(0);
        let mut texts = TextBatch::default();
        let size = tabs.layout(loose(), &mut texts);
        let mut msgs = MsgQueue::new();
        // 从左往右布局：点击第二个 tab 的热区中心
        let hit = tabs.hit_rects[1];
        let click = Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(
                hit.origin.x + hit.size.width / 2.0,
                hit.origin.y + hit.size.height / 2.0,
            ),
        };
        let _ = tabs.event(&click, Rect::new(Point::ZERO, size), &mut msgs);
        assert_eq!(tabs.active, 1, "点击 tab 应切换 active");
    }
}
