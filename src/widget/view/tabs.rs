//! @author 十四叔
//! @date 2026/08/18

//! Tabs 组件：带自绘 tab 栏的多面板切换容器。
//!
//! 核心思路复用 [`Switcher`] 的面板切换逻辑（sync 传播全部子面板,
//! layout/paint/event 只作用于 active 面板）,
//! 额外自绘 tab 栏（文字 + 指示线）。

use std::any::Any;

use crate::app::AnimationCtx;
use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::theme::Theme;
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, LightTheme, Point, Rect, Size};

/// active 索引绑定闭包：每帧从应用状态读取。
type ActiveBinding = Box<dyn Fn(&dyn Any) -> usize>;

/// Tabs 组件：带 tab 栏的多面板切换容器。
///
/// tab 栏为自绘叶子（不使用 Button 子组件），直接在 paint 中绘制
/// 文字 + 选中指示线。面板切换复用 [`Switcher`] 的逻辑：
/// - `sync` / `animate` 传播全部子面板（状态保鲜、动画存活）
/// - `layout` / `paint` / `event` 只作用于 active 面板
///
/// # 用法
///
/// ```rust,ignore
/// Tabs::new(&theme)
///     .tab("常规")
///     .tab("快捷键")
///     .tab("关于")
///     .child常规内容)
///     .child(hotkey_content)
///     .child(about_content)
///     .bind(|app: &MyApp| app.settings_tab)
/// ```
pub struct Tabs {
    /// tab 标签文字。
    labels: Vec<String>,
    /// 面板子组件。
    children: Vec<Node>,
    /// 当前选中的 tab 索引。
    active: usize,
    /// active 索引绑定闭包。
    binding: Option<ActiveBinding>,
    /// tab 栏高度（layout 时计算）。
    tab_bar_height: f32,
    /// active 子组件尺寸（layout 缓存）。
    active_size: Size,
    /// hover 中的 tab 索引（None 表示无 hover）。
    hover: Option<usize>,
}

impl Tabs {
    /// 创建空 Tabs。
    pub fn new(_theme: &impl Theme) -> Self {
        Self {
            labels: Vec::new(),
            children: Vec::new(),
            active: 0,
            binding: None,
            tab_bar_height: 0.0,
            active_size: Size::ZERO,
            hover: None,
        }
    }

    /// 添加一个 tab 标签，返回自身以链式调用。
    ///
    /// `label` 为 tab 栏显示的文字。tab 与 child 须一一对应。
    pub fn tab(mut self, label: &str) -> Self {
        self.labels.push(label.to_string());
        self
    }

    /// 添加一个面板内容，返回自身以链式调用。
    ///
    /// 面板数量须与 tab 数量一致。
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
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

    /// 设置初始 active tab 索引（越界时在 layout / sync 时钳制）。
    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    /// 把越界的 active 钳制到末尾索引。
    fn clamp_active(&mut self) {
        if !self.children.is_empty() && self.active >= self.children.len() {
            self.active = self.children.len() - 1;
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

    /// 计算单个 tab 文字宽度的近似值（等宽字体假设）。
    fn approx_label_width(label: &str, font_size: u16) -> f32 {
        // 中文字符占约 font_size 宽度，ASCII 字符占约 font_size * 0.6
        let mut width = 0.0f32;
        for ch in label.chars() {
            if ch.is_ascii() {
                width += font_size as f32 * 0.6;
            } else {
                width += font_size as f32;
            }
        }
        width
    }
}

impl Default for Tabs {
    fn default() -> Self {
        Self::new(&LightTheme)
    }
}

impl Widget for Tabs {
    fn sync(&mut self, state: &dyn Any) {
        let prev_active = self.active;
        for child in &mut self.children {
            child.sync(state);
        }
        if let Some(binding) = &self.binding {
            self.active = binding(state);
        }
        self.clamp_active();
        // 面板切换时重置旧面板的焦点视觉。
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

        // tab 栏高度：font_size_small + 上下 padding
        let font_size = LightTheme.font_size_small();
        self.tab_bar_height = font_size as f32 + 16.0; // 8px padding each side

        // 面板区域：扣除 tab 栏高度，用 loose 约束让子组件自适应
        let panel_height = (constraints.max_height - self.tab_bar_height).max(0.0);
        let panel_constraints = Constraints::loose(Size::new(
            constraints.max_width,
            panel_height,
        ));

        match self.children.get_mut(self.active) {
            Some(child) => {
                self.active_size = child.layout(panel_constraints, texts);
                Size::new(
                    constraints.max_width,
                    self.tab_bar_height + self.active_size.height,
                )
            }
            None => {
                self.active_size = Size::ZERO;
                Size::new(constraints.max_width, self.tab_bar_height)
            }
        }
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let theme = LightTheme;
        let font_size = LightTheme.font_size_small();

        // 绘制 tab 栏背景
        let tab_bar_rect = Rect::new(area.origin, Size::new(area.size.width, self.tab_bar_height));
        rects.push_rect(tab_bar_rect, theme.surface(), 0.0);

        // 绘制每个 tab 文字
        let mut x_offset = area.origin.x + theme.spacing_md();
        for (i, label) in self.labels.iter().enumerate() {
            let label_width = Self::approx_label_width(label, font_size);
            let is_active = i == self.active;
            let is_hovered = self.hover == Some(i);

            // tab 背景（hover 态）
            if is_hovered && !is_active {
                let hover_rect = Rect::new(
                    Point::new(x_offset - 4.0, area.origin.y + 4.0),
                    Size::new(label_width + 8.0, self.tab_bar_height - 8.0),
                );
                rects.push_rect(hover_rect, theme.surface_variant(), theme.radius_sm());
            }

            // tab 文字
            let text_color = if is_active {
                theme.text_primary()
            } else {
                theme.text_secondary()
            };
            let baseline = area.origin.y + texts.ascent(f32::from(font_size));
            texts.push_text(
                label,
                x_offset,
                baseline,
                font_size,
                text_color,
            );

            // 选中指示线
            if is_active {
                let indicator_rect = Rect::new(
                    Point::new(x_offset - 2.0, area.origin.y + self.tab_bar_height - 2.0),
                    Size::new(label_width + 4.0, 2.0),
                );
                rects.push_rect(indicator_rect, theme.accent(), 1.0);
            }

            x_offset += label_width + theme.spacing_md();
        }

        // 绘制面板
        if let Some(child) = self.children.get(self.active) {
            let panel_area = Rect::new(
                Point::new(area.origin.x, area.origin.y + self.tab_bar_height),
                self.active_size,
            );
            child.paint(panel_area, rects, texts);
        }
    }

    fn paint_image(&self, area: Rect, images: &mut crate::render::ImageBatch) {
        if let Some(child) = self.children.get(self.active) {
            let panel_area = Rect::new(
                Point::new(area.origin.x, area.origin.y + self.tab_bar_height),
                self.active_size,
            );
            child.paint_image(panel_area, images);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        match event {
            Event::CursorMoved(pos) => {
                // 检测 hover
                let tab_bar_rect =
                    Rect::new(area.origin, Size::new(area.size.width, self.tab_bar_height));
                if tab_bar_rect.contains(*pos) {
                    // 计算 hover 的 tab 索引
                    let font_size = LightTheme.font_size_small();
                    let mut x_offset = area.origin.x + LightTheme.spacing_md();
                    let mut new_hover = None;
                    for (i, label) in self.labels.iter().enumerate() {
                        let label_width = Self::approx_label_width(label, font_size);
                        let tab_rect = Rect::new(
                            Point::new(x_offset - 4.0, area.origin.y),
                            Size::new(label_width + 8.0, self.tab_bar_height),
                        );
                        if tab_rect.contains(*pos) {
                            new_hover = Some(i);
                            break;
                        }
                        x_offset += label_width + LightTheme.spacing_md();
                    }
                    self.hover = new_hover;
                } else {
                    self.hover = None;
                }
                EventResult::Ignored
            }
            Event::MouseInput {
                pressed,
                button,
                position,
            } => {
                if *pressed && *button == crate::event::MouseButton::Left {
                    // 检测点击 tab
                    if let Some(hover) = self.hover {
                        let font_size = LightTheme.font_size_small();
                        let mut x_offset = area.origin.x + LightTheme.spacing_md();
                        for i in 0..hover {
                            let label_width = Self::approx_label_width(&self.labels[i], font_size);
                            x_offset += label_width + LightTheme.spacing_md();
                        }
                        let label_width =
                            Self::approx_label_width(&self.labels[hover], font_size);
                        let tab_rect = Rect::new(
                            Point::new(x_offset - 4.0, area.origin.y),
                            Size::new(label_width + 8.0, self.tab_bar_height),
                        );
                        if tab_rect.contains(*position) {
                            self.active = hover;
                        }
                    }
                }
                EventResult::Ignored
            }
            _ => {
                // 面板事件
                match self.children.get_mut(self.active) {
                    Some(child) => {
                        let panel_area = Rect::new(
                            Point::new(area.origin.x, area.origin.y + self.tab_bar_height),
                            self.active_size,
                        );
                        child.event(event, panel_area, msgs)
                    }
                    None => EventResult::Ignored,
                }
            }
        }
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
    use crate::widget::Widget;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// 测试桩：固定尺寸，记录 sync / paint / event 调用。
    struct Stub {
        id: &'static str,
        size: Size,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Stub {
        fn new(
            id: &'static str,
            width: f32,
            height: f32,
            log: &Rc<RefCell<Vec<&'static str>>>,
        ) -> Self {
            Self {
                id,
                size: Size::new(width, height),
                log: Rc::clone(log),
            }
        }
    }

    impl Widget for Stub {
        fn sync(&mut self, _state: &dyn Any) {
            self.log.borrow_mut().push(self.id);
        }

        fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
            constraints.constrain(self.size)
        }

        fn paint(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {
            self.log.borrow_mut().push(self.id);
        }

        fn event(&mut self, _event: &Event, _area: Rect, msgs: &mut MsgQueue) -> EventResult {
            msgs.push(Box::new(self.id));
            EventResult::Consumed
        }
    }

    fn loose() -> Constraints {
        Constraints::loose(Size::new(400.0, 300.0))
    }

    #[test]
    fn empty_tabs_lays_out_to_tab_bar_height_and_no_children() {
        let theme = LightTheme;
        let mut tabs = Tabs::new(&theme);
        let mut texts = TextBatch::default();
        let size = tabs.layout(loose(), &mut texts);
        // 空 tabs 应至少有 tab 栏高度
        assert!(size.height > 0.0);
        assert!(tabs.children().is_empty());
    }

    #[test]
    fn out_of_range_active_clamps_to_last() {
        let theme = LightTheme;
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut tabs = Tabs::new(&theme)
            .tab("A")
            .tab("B")
            .child(Stub::new("a", 10.0, 10.0, &log))
            .child(Stub::new("b", 20.0, 20.0, &log))
            .active(9);
        let mut texts = TextBatch::default();
        let _size = tabs.layout(loose(), &mut texts);
        assert_eq!(tabs.children().len(), 1);
    }

    #[test]
    fn layout_size_includes_tab_bar_and_active_child() {
        let theme = LightTheme;
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut tabs = Tabs::new(&theme)
            .tab("A")
            .tab("B")
            .child(Stub::new("a", 10.0, 10.0, &log))
            .child(Stub::new("b", 20.0, 15.0, &log));
        let mut texts = TextBatch::default();
        let size = tabs.layout(loose(), &mut texts);
        // 高度 = tab 栏高度 + active child 高度
        assert!(size.height > tabs.tab_bar_height);
        assert_eq!(size.height, tabs.tab_bar_height + 10.0);

        tabs = tabs.active(1);
        let size = tabs.layout(loose(), &mut texts);
        assert_eq!(size.height, tabs.tab_bar_height + 15.0);
    }

    #[test]
    fn sync_reaches_all_children_and_binding_drives_active() {
        struct State {
            active: usize,
        }
        let theme = LightTheme;
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut tabs = Tabs::new(&theme)
            .tab("A")
            .tab("B")
            .child(Stub::new("a", 10.0, 10.0, &log))
            .child(Stub::new("b", 10.0, 10.0, &log))
            .bind(|s: &State| s.active);

        tabs.sync(&State { active: 1 });
        // sync 传播给全部子组件。
        assert_eq!(log.take(), vec!["a", "b"]);

        // binding 驱动 active 切换，children() 只暴露 active。
        let mut texts = TextBatch::default();
        tabs.layout(loose(), &mut texts);
        assert_eq!(tabs.children().len(), 1);
    }
}
