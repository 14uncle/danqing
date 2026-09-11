//! @author 十四叔
//! @date 2026/09/11
//!
//! 下拉选择器: 点击展开选项列表, 键盘导航, hover 高亮。
//!
//! 单选下拉, 选项构造时给定, 支持 `on_select(idx)` 回调与 `bind_selected` 绑定。
//! 展开态在组件区域内绘制选项列表 (不使用 Overlay, 简化实现)。

use std::any::Any;
use std::cell::Cell;

use crate::event::{Event, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 选项工厂: 选中时产出消息 (idx)。
type SelectFactory = Box<dyn Fn(usize) -> Box<dyn Any>>;
/// 选中绑定: 从应用状态读取当前选中索引。
type SelectedBinding = Box<dyn Fn(&dyn Any) -> usize>;

/// 控件高度。
const CONTROL_HEIGHT: f32 = 36.0;
/// 选项行高。
const OPTION_HEIGHT: f32 = 32.0;
/// 最大可见选项数 (超出滚动)。
const MAX_VISIBLE: usize = 8;
/// 圆角半径。
const RADIUS: f32 = 6.0;
/// 内边距。
const PADDING: f32 = 10.0;
/// 箭头区宽度。
const ARROW_W: f32 = 24.0;

/// 下拉选择器组件。
pub struct Dropdown {
    /// 选项文本列表。
    options: Vec<String>,
    /// 当前选中索引。
    selected: usize,
    /// 选中绑定。
    selected_binding: Option<SelectedBinding>,
    /// 选中回调。
    on_select: Option<SelectFactory>,
    /// 是否展开。
    expanded: bool,
    /// 展开时 hover 的选项索引 (usize::MAX = 无)。
    hover_idx: usize,
    /// 滚动偏移 (选项数)。
    scroll_offset: usize,
    /// 控件 hover 态。
    hovered: bool,
    /// 焦点态。
    focused: bool,
    /// 主题颜色。
    bg_color: Color,
    border_color: Color,
    text_color: Color,
    hover_color: Color,
    selected_color: Color,
    arrow_color: Color,
    /// layout 缓存。
    area: Cell<Rect>,
}

impl Dropdown {
    /// 创建下拉选择器。
    pub fn new(options: Vec<String>) -> Self {
        Self::themed(&LightTheme, options)
    }

    /// 使用指定主题创建。
    pub fn themed(theme: &impl Theme, options: Vec<String>) -> Self {
        Self {
            options,
            selected: 0,
            selected_binding: None,
            on_select: None,
            expanded: false,
            hover_idx: usize::MAX,
            scroll_offset: 0,
            hovered: false,
            focused: false,
            bg_color: theme.surface_input(),
            border_color: theme.border(),
            text_color: theme.text_primary(),
            hover_color: theme.surface_variant(),
            selected_color: theme.selection(),
            arrow_color: theme.text_secondary(),
            area: Cell::new(Rect::default()),
        }
    }

    /// 绑定选中索引。
    pub fn bind_selected<S: 'static>(mut self, f: impl Fn(&S) -> usize + 'static) -> Self {
        self.selected_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("Dropdown 选中绑定的状态类型不匹配"))
        }));
        self
    }

    /// 设置选中回调。
    pub fn on_select<M: 'static>(mut self, f: impl Fn(usize) -> M + 'static) -> Self {
        self.on_select = Some(Box::new(move |idx| Box::new(f(idx)) as Box<dyn Any>));
        self
    }

    /// 选项总数。
    fn option_count(&self) -> usize {
        self.options.len()
    }

    /// 可见选项数。
    fn visible_count(&self) -> usize {
        self.option_count().min(MAX_VISIBLE)
    }

    /// 展开时的总高度。
    fn expanded_height(&self) -> f32 {
        CONTROL_HEIGHT + self.visible_count() as f32 * OPTION_HEIGHT
    }

    /// 选项列表区域。
    fn list_rect(&self, area: Rect) -> Rect {
        Rect::from_xywh(
            area.origin.x,
            area.origin.y + CONTROL_HEIGHT,
            area.size.width,
            self.visible_count() as f32 * OPTION_HEIGHT,
        )
    }

    /// 点击位置对应的选项索引。
    fn option_at(&self, area: Rect, pos: Point) -> Option<usize> {
        let list = self.list_rect(area);
        if !list.contains(pos) {
            return None;
        }
        let rel_y = pos.y - list.origin.y;
        let idx = (rel_y / OPTION_HEIGHT) as usize + self.scroll_offset;
        if idx < self.option_count() {
            Some(idx)
        } else {
            None
        }
    }
}

impl Widget for Dropdown {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(bind) = &self.selected_binding {
            self.selected = bind(state);
        }
    }

    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        let h = if self.expanded {
            self.expanded_height()
        } else {
            CONTROL_HEIGHT
        };
        let size = constraints.constrain(Size::new(constraints.max().width, h));
        self.area.set(Rect::new(Point::ZERO, size));
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        self.area.set(area);

        // 控件主体 (选择框)
        let control = Rect::from_xywh(area.origin.x, area.origin.y, area.size.width, CONTROL_HEIGHT);
        rects.push_rect(control, self.bg_color, RADIUS);
        if self.hovered || self.focused {
            rects.push_rounded_border(control, self.border_color, RADIUS, 1.0);
        }

        // 选中文本
        if self.selected < self.option_count() {
            let text = &self.options[self.selected];
            let baseline = control.origin.y
                + (CONTROL_HEIGHT - texts.line_height(14.0)) / 2.0
                + texts.ascent(14.0);
            texts.push_text(text, control.origin.x + PADDING, baseline, 14, self.text_color);
        }

        // 箭头 ▼
        let arrow_x = control.origin.x + control.size.width - ARROW_W;
        let arrow_baseline = control.origin.y
            + (CONTROL_HEIGHT - texts.line_height(12.0)) / 2.0
            + texts.ascent(12.0);
        let arrow = if self.expanded { "▲" } else { "▼" };
        texts.push_text(arrow, arrow_x + 6.0, arrow_baseline, 12, self.arrow_color);

        // 展开态: 选项列表
        if self.expanded {
            let list = self.list_rect(area);
            // 列表背景
            rects.push_rect(
                Rect::from_xywh(list.origin.x, list.origin.y, list.size.width, list.size.height + 4.0),
                self.bg_color,
                RADIUS,
            );
            rects.push_rounded_border(
                Rect::from_xywh(list.origin.x, list.origin.y, list.size.width, list.size.height + 4.0),
                self.border_color,
                RADIUS,
                1.0,
            );

            // 选项行
            for i in 0..self.visible_count() {
                let opt_idx = i + self.scroll_offset;
                if opt_idx >= self.option_count() {
                    break;
                }
                let y = list.origin.y + i as f32 * OPTION_HEIGHT;
                let opt_rect = Rect::from_xywh(list.origin.x, y, list.size.width, OPTION_HEIGHT);

                // hover 高亮
                if opt_idx == self.hover_idx {
                    rects.push_rect(opt_rect, self.hover_color, 0.0);
                }
                // 选中标记
                if opt_idx == self.selected {
                    rects.push_rect(
                        Rect::from_xywh(opt_rect.origin.x, opt_rect.origin.y, 3.0, OPTION_HEIGHT),
                        self.selected_color,
                        0.0,
                    );
                }

                // 文本
                let baseline = y
                    + (OPTION_HEIGHT - texts.line_height(14.0)) / 2.0
                    + texts.ascent(14.0);
                texts.push_text(
                    &self.options[opt_idx],
                    opt_rect.origin.x + PADDING + 4.0,
                    baseline,
                    14,
                    self.text_color,
                );
            }
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let area = area.snap_to_pixels();
        let control = Rect::from_xywh(area.origin.x, area.origin.y, area.size.width, CONTROL_HEIGHT);

        match event {
            Event::CursorMoved(p) => {
                self.hovered = control.contains(*p);
                if self.expanded {
                    self.hover_idx = self.option_at(area, *p).unwrap_or(usize::MAX);
                }
                if self.hovered || self.expanded {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::CursorLeft => {
                self.hovered = false;
                self.hover_idx = usize::MAX;
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position,
            } => {
                if self.expanded {
                    if let Some(idx) = self.option_at(area, *position) {
                        self.selected = idx;
                        self.expanded = false;
                        self.hover_idx = usize::MAX;
                        if let Some(factory) = &self.on_select {
                            msgs.push(factory(idx));
                        }
                        return EventResult::Consumed;
                    }
                }
                if control.contains(*position) {
                    self.expanded = !self.expanded;
                    return EventResult::Consumed;
                }
                // 点击外部收起
                if self.expanded {
                    self.expanded = false;
                    self.hover_idx = usize::MAX;
                    return EventResult::Consumed;
                }
                EventResult::Ignored
            }
            Event::Key {
                key,
                pressed: true,
                ..
            } if self.focused => {
                match key {
                    Key::Named(NamedKey::Escape) => {
                        if self.expanded {
                            self.expanded = false;
                            self.hover_idx = usize::MAX;
                            EventResult::Consumed
                        } else {
                            EventResult::Ignored
                        }
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        if self.expanded {
                            // 导航到下一个选项
                            let next = if self.hover_idx == usize::MAX {
                                self.selected
                            } else {
                                self.hover_idx + 1
                            };
                            if next < self.option_count() {
                                self.hover_idx = next;
                                // 滚动跟随
                                if next >= self.scroll_offset + self.visible_count() {
                                    self.scroll_offset = next - self.visible_count() + 1;
                                }
                            }
                        } else {
                            self.expanded = true;
                            self.hover_idx = self.selected;
                        }
                        EventResult::Consumed
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        if self.expanded {
                            let prev = if self.hover_idx == usize::MAX {
                                self.selected
                            } else if self.hover_idx > 0 {
                                self.hover_idx - 1
                            } else {
                                0
                            };
                            self.hover_idx = prev;
                            // 滚动跟随
                            if prev < self.scroll_offset {
                                self.scroll_offset = prev;
                            }
                        }
                        EventResult::Consumed
                    }
                    Key::Named(NamedKey::Enter) => {
                        if self.expanded && self.hover_idx < self.option_count() {
                            self.selected = self.hover_idx;
                            self.expanded = false;
                            self.hover_idx = usize::MAX;
                            if let Some(factory) = &self.on_select {
                                msgs.push(factory(self.selected));
                            }
                        } else if !self.expanded {
                            self.expanded = true;
                            self.hover_idx = self.selected;
                        }
                        EventResult::Consumed
                    }
                    _ => EventResult::Ignored,
                }
            }
            Event::FocusIn => {
                self.focused = true;
                EventResult::Consumed
            }
            Event::FocusOut => {
                self.focused = false;
                self.expanded = false;
                self.hover_idx = usize::MAX;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}
