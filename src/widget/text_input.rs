//! @author 十四叔
//! @date 2026/07/17

//! TextInput 组件:单行可编辑文本。
//!
//! 支持光标、选区、键盘编辑、IME preedit 显示与 commit 插入。

use std::cell::Cell;

use crate::app::AnimationCtx;
use crate::event::{Event, ImeEvent, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, Edges, Rect, Size};

/// 文本变化回调:返回一条应用消息。
type ChangeFactory = Box<dyn Fn(&str) -> Box<dyn std::any::Any>>;

/// 光标闪烁周期(秒)。
const BLINK_PERIOD: f32 = 0.5;

/// 单行文本输入组件。
pub struct TextInput {
    /// 当前文本内容。
    text: String,
    /// 光标位置(字符索引,0..=char_count)。
    cursor: usize,
    /// 选区锚点(字符索引);与 cursor 相等表示无选区。
    anchor: usize,
    /// 是否获得焦点。
    focused: bool,
    /// 字体大小。
    font_size: u16,
    /// 文本颜色。
    color: Color,
    /// 背景色。
    background: Color,
    /// 选中背景色。
    selection_color: Color,
    /// 光标颜色。
    caret_color: Color,
    /// 内边距。
    padding: Edges,
    /// 显式宽度(未指定则按约束上限)。
    width: Option<f32>,
    /// layout/paint 缓存:自身绝对矩形。
    area: Cell<Rect>,
    /// 每个字符右侧的 x 偏移(用于鼠标点击定位光标与 IME 区域)。
    char_offsets: Vec<f32>,
    /// 行高(用于 IME 区域与光标高度)。
    line_height: f32,
    /// 光标可见性(由动画控制闪烁)。
    caret_visible: bool,
    /// IME 合成文本(显示在光标处,带下划线)。
    preedit: Option<String>,
    /// 文本变化时产出的应用消息。
    on_change: Option<ChangeFactory>,
    /// 鼠标拖拽选区状态。
    dragging: bool,
}

impl TextInput {
    /// 创建文本输入框(默认空文本、字号 16、深色文本、浅色背景)。
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            anchor: 0,
            focused: false,
            font_size: 16,
            color: Color::from_srgb8(0x22, 0x22, 0x22),
            background: Color::WHITE,
            selection_color: Color::from_srgb8(0xB3, 0xD7, 0xFF),
            caret_color: Color::from_srgb8(0x1E, 0x90, 0xFF),
            padding: Edges::symmetric(12.0, 8.0),
            width: None,
            area: Cell::new(Rect::default()),
            char_offsets: Vec::new(),
            line_height: 0.0,
            caret_visible: true,
            preedit: None,
            on_change: None,
            dragging: false,
        }
    }

    /// 设置文本变化回调。
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor = self.text.chars().count();
        self.anchor = self.cursor;
        self
    }

    /// 设置字号。
    pub fn font_size(mut self, size: u16) -> Self {
        self.font_size = size;
        self
    }

    /// 设置文本颜色。
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 设置背景色。
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// 设置显式宽度。
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// 设置文本变化回调(每次编辑后触发)。
    pub fn on_change<M: 'static>(mut self, f: impl Fn(&str) -> M + 'static) -> Self {
        self.on_change = Some(Box::new(move |text| {
            Box::new(f(text)) as Box<dyn std::any::Any>
        }));
        self
    }

    /// 当前文本(不含 preedit)。
    pub fn value(&self) -> &str {
        &self.text
    }

    /// 测量到给定字符索引的文本宽度。
    fn measure_to(&self, texts: &mut TextBatch, char_idx: usize) -> f32 {
        let prefix: String = self.text.chars().take(char_idx).collect();
        texts.measure(&prefix, self.font_size)
    }

    /// 在光标处插入文本。
    fn insert(&mut self, text: &str) {
        self.delete_selection_pre_edit();
        let byte_idx = char_to_byte(&self.text, self.cursor);
        self.text.insert_str(byte_idx, text);
        self.cursor += text.chars().count();
        self.anchor = self.cursor;
    }

    /// 通知应用文本已变化。
    fn notify_change(&self, msgs: &mut MsgQueue) {
        if let Some(factory) = &self.on_change {
            msgs.push(factory(&self.text));
        }
    }

    fn delete_selection_pre_edit(&mut self) {
        let (start, end) = self.selection_range();
        if start == end {
            return;
        }
        let start_byte = char_to_byte(&self.text, start);
        let end_byte = char_to_byte(&self.text, end);
        self.text.drain(start_byte..end_byte);
        self.cursor = start;
        self.anchor = self.cursor;
    }

    /// 全选文本。
    fn select_all(&mut self) {
        self.cursor = self.text.chars().count();
        self.anchor = 0;
    }

    /// 删除当前选区(若存在)。
    fn delete_selection(&mut self) {
        let (start, end) = self.selection_range();
        if start != end {
            let start_byte = char_to_byte(&self.text, start);
            let end_byte = char_to_byte(&self.text, end);
            self.text.drain(start_byte..end_byte);
            self.cursor = start;
            self.anchor = self.cursor;
        }
    }

    /// 将本地 x 坐标(相对于文本起点)转换为字符索引。
    fn hit_to_index(&self, local_x: f32) -> usize {
        if self.char_offsets.is_empty() {
            return 0;
        }
        self.char_offsets
            .partition_point(|offset| *offset <= local_x)
            .min(self.char_offsets.len())
    }
    fn move_cursor(&mut self, delta: isize, extend_selection: bool) {
        let len = self.text.chars().count();
        let new_cursor = if delta < 0 {
            self.cursor.saturating_sub(delta.unsigned_abs())
        } else {
            (self.cursor + delta as usize).min(len)
        };
        self.cursor = new_cursor;
        if !extend_selection {
            self.anchor = self.cursor;
        }
    }

    /// 选区范围(起点,终点),保证 start <= end。
    fn selection_range(&self) -> (usize, usize) {
        if self.anchor <= self.cursor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextInput {
    fn animate(&mut self, ctx: &AnimationCtx) {
        if self.focused {
            let t = ctx.elapsed.as_secs_f32();
            self.caret_visible = (t % (BLINK_PERIOD * 2.0)) < BLINK_PERIOD;
        } else {
            self.caret_visible = false;
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        let content_width = texts.measure(&self.text, self.font_size);
        let line_height = texts.line_height(f32::from(self.font_size));
        let height = line_height + self.padding.vertical();
        let width = self
            .width
            .unwrap_or(constraints.max_width)
            .max(content_width + self.padding.horizontal());
        let size = constraints.constrain(Size::new(width, height));
        self.area.set(Rect::new(crate::Point::ZERO, size));
        self.line_height = line_height;

        // 缓存每个字符右侧的 x 偏移,用于鼠标点击定位光标。
        self.char_offsets.clear();
        let mut x = 0.0f32;
        for ch in self.text.chars() {
            x += texts.measure(&ch.to_string(), self.font_size);
            self.char_offsets.push(x);
        }
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        // 缓存绝对矩形,供 IME 区域与后续事件使用。
        self.area.set(area);

        // 背景
        rects.push_rect(area, self.background, 4.0);

        // 文本起点
        let text_x = area.origin.x + self.padding.left;
        let baseline = area.origin.y + self.padding.top + texts.ascent(f32::from(self.font_size));

        // 选区高亮
        let (sel_start, sel_end) = self.selection_range();
        if sel_start < sel_end && self.focused {
            let x0 = text_x + self.measure_to(texts, sel_start);
            let x1 = text_x + self.measure_to(texts, sel_end);
            rects.push_rect(
                Rect::from_xywh(
                    x0,
                    area.origin.y + self.padding.top,
                    x1 - x0,
                    area.size.height - self.padding.vertical(),
                ),
                self.selection_color,
                0.0,
            );
        }

        // 基础文本
        texts.push_text(&self.text, text_x, baseline, self.font_size, self.color);

        // preedit 文本与下划线
        if let Some(preedit) = &self.preedit {
            let pre_x = text_x + self.measure_to(texts, self.cursor);
            texts.push_text(preedit, pre_x, baseline, self.font_size, self.color);
            let pre_width = texts.measure(preedit, self.font_size);
            let underline_y = baseline + texts.line_height(f32::from(self.font_size)) * 0.15;
            rects.push_rect(
                Rect::from_xywh(pre_x, underline_y, pre_width, 1.0),
                self.color,
                0.0,
            );
        }

        // 光标
        if self.focused && self.caret_visible {
            let caret_x = text_x + self.measure_to(texts, self.cursor);
            let caret_height = texts.line_height(f32::from(self.font_size));
            let caret_y = area.origin.y + self.padding.top;
            rects.push_rect(
                Rect::from_xywh(caret_x, caret_y, 2.0, caret_height),
                self.caret_color,
                0.0,
            );
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let mut changed = false;
        let result = match event {
            Event::FocusIn => {
                self.focused = true;
                self.caret_visible = true;
                EventResult::Consumed
            }
            Event::FocusOut => {
                self.focused = false;
                self.preedit = None;
                self.dragging = false;
                EventResult::Consumed
            }
            Event::Key {
                key,
                pressed: true,
                shift,
                ctrl,
            } => match key {
                Key::Character(s) if !ctrl => {
                    self.insert(s);
                    changed = true;
                    EventResult::Consumed
                }
                Key::Character(s) if *ctrl && s == "a" => {
                    self.select_all();
                    EventResult::Consumed
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    self.move_cursor(-1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::ArrowRight) => {
                    self.move_cursor(1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Home) => {
                    self.cursor = 0;
                    if !shift {
                        self.anchor = 0;
                    }
                    EventResult::Consumed
                }
                Key::Named(NamedKey::End) => {
                    self.cursor = self.text.chars().count();
                    if !shift {
                        self.anchor = self.cursor;
                    }
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Backspace) => {
                    let (start, end) = self.selection_range();
                    if start != end {
                        self.delete_selection_pre_edit();
                        changed = true;
                    } else if self.cursor > 0 {
                        let byte_idx = char_to_byte(&self.text, self.cursor);
                        let prev = self.text[..byte_idx]
                            .chars()
                            .next_back()
                            .map(|c| c.len_utf8())
                            .unwrap_or(0);
                        self.text.drain((byte_idx - prev)..byte_idx);
                        self.cursor -= 1;
                        self.anchor = self.cursor;
                        changed = true;
                    }
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Delete) => {
                    let (start, end) = self.selection_range();
                    if start != end {
                        self.delete_selection_pre_edit();
                        changed = true;
                    } else if self.cursor < self.text.chars().count() {
                        let byte_idx = char_to_byte(&self.text, self.cursor);
                        let len = self.text[byte_idx..]
                            .chars()
                            .next()
                            .map(|c| c.len_utf8())
                            .unwrap_or(0);
                        self.text.drain(byte_idx..(byte_idx + len));
                        changed = true;
                    }
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Enter) => {
                    // 单行输入框忽略回车
                    EventResult::Consumed
                }
                _ => EventResult::Ignored,
            },
            Event::Ime(ImeEvent::Preedit { value, .. }) => {
                self.preedit = if value.is_empty() {
                    None
                } else {
                    Some(value.clone())
                };
                EventResult::Consumed
            }
            Event::Ime(ImeEvent::Commit { value }) => {
                self.preedit = None;
                self.insert(value);
                changed = true;
                EventResult::Consumed
            }
            Event::Ime(ImeEvent::Disabled) => {
                self.preedit = None;
                EventResult::Consumed
            }
            Event::Copy => EventResult::Consumed,
            Event::Cut => {
                if self.selected_text().is_some() {
                    self.delete_selection();
                    changed = true;
                }
                EventResult::Consumed
            }
            Event::Paste => EventResult::Consumed,
            Event::CursorMoved(p) => {
                if self.dragging {
                    let text_x = area.origin.x + self.padding.left;
                    let local_x = p.x - text_x;
                    self.cursor = self.hit_to_index(local_x);
                }
                EventResult::Ignored
            }
            Event::CursorLeft => {
                self.dragging = false;
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position,
            } => {
                let text_x = area.origin.x + self.padding.left;
                let local_x = position.x - text_x;
                self.cursor = self.hit_to_index(local_x);
                self.anchor = self.cursor;
                self.dragging = true;
                EventResult::Consumed
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                ..
            } => {
                self.dragging = false;
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        };
        if changed {
            self.notify_change(msgs);
        }
        result
    }

    fn focusable(&self) -> bool {
        true
    }

    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range();
        if start == end {
            return None;
        }
        let start_byte = char_to_byte(&self.text, start);
        let end_byte = char_to_byte(&self.text, end);
        Some(self.text[start_byte..end_byte].to_string())
    }

    fn wants_ime(&self) -> bool {
        true
    }

    fn ime_area(&self) -> Option<Rect> {
        let area = self.area.get();
        let cursor_x = if self.cursor == 0 {
            0.0
        } else {
            self.char_offsets
                .get(self.cursor - 1)
                .copied()
                .unwrap_or(0.0)
        };
        let x = area.origin.x + self.padding.left + cursor_x;
        let y = area.origin.y + self.padding.top;
        Some(Rect::from_xywh(x, y, 0.0, self.line_height))
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}

/// 字符索引转字节索引。
fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> TextInput {
        TextInput::new().text("Hello")
    }

    #[test]
    fn insert_moves_cursor() {
        let mut t = input();
        t.insert(" world");
        assert_eq!(t.value(), "Hello world");
        assert_eq!(t.cursor, 11);
    }

    #[test]
    fn backspace_deletes_previous_char() {
        let mut t = input();
        // cursor 在末尾,Backspace 删除 'o'
        assert_eq!(t.cursor, 5);
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Backspace),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hell");
        assert_eq!(t.cursor, 4);
    }

    #[test]
    fn selection_and_delete() {
        let mut t = input();
        // 选中 "ell"
        t.cursor = 1;
        t.anchor = 4;
        t.delete_selection_pre_edit();
        assert_eq!(t.value(), "Ho");
        assert_eq!(t.cursor, 1);
    }

    #[test]
    fn arrow_with_shift_extends_selection() {
        let mut t = input();
        t.cursor = 1;
        t.anchor = 1;
        t.move_cursor(2, true);
        assert_eq!(t.cursor, 3);
        assert_eq!(t.anchor, 1);
    }

    #[test]
    fn selected_text_returns_selection() {
        let mut t = input();
        t.cursor = 1;
        t.anchor = 4;
        assert_eq!(t.selected_text(), Some("ell".to_string()));
    }

    #[test]
    fn ctrl_a_selects_all_text() {
        let mut t = input();
        t.event(&Event::FocusIn, Rect::default(), &mut Vec::new());
        t.event(
            &Event::Key {
                key: Key::Character("a".to_string()),
                pressed: true,
                shift: false,
                ctrl: true,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.selected_text(), Some("Hello".to_string()));
        assert_eq!(t.cursor, 5);
        assert_eq!(t.anchor, 0);
    }

    #[test]
    fn cut_deletes_selection() {
        let mut t = input();
        t.cursor = 1;
        t.anchor = 4;
        t.event(&Event::Cut, Rect::default(), &mut Vec::new());
        assert_eq!(t.value(), "Ho");
        assert_eq!(t.cursor, 1);
        assert_eq!(t.anchor, 1);
        assert!(t.selected_text().is_none());
    }

    #[test]
    fn mouse_click_positions_cursor() {
        let mut t = input();
        let mut texts = crate::TextBatch::new();
        // 触发 layout 以计算 char_offsets
        t.layout(Constraints::loose(Size::new(500.0, 100.0)), &mut texts);

        // 点击文本起点左侧,光标应在 0
        t.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: crate::Point::new(0.0, 0.0),
            },
            Rect::from_xywh(0.0, 0.0, 500.0, 100.0),
            &mut Vec::new(),
        );
        assert_eq!(t.cursor, 0);

        // 点击文本末尾右侧,光标应在末尾
        let end_x = t.char_offsets.last().copied().unwrap_or(0.0) + 100.0;
        t.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: crate::Point::new(end_x, 0.0),
            },
            Rect::from_xywh(0.0, 0.0, 500.0, 100.0),
            &mut Vec::new(),
        );
        assert_eq!(t.cursor, 5);
    }

    #[test]
    fn mouse_drag_selects_text() {
        let mut t = input();
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 100.0)), &mut texts);

        let area = Rect::from_xywh(0.0, 0.0, 500.0, 100.0);
        // 按下并拖到末尾
        t.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: crate::Point::new(0.0, 0.0),
            },
            area,
            &mut Vec::new(),
        );
        let end_x = t.char_offsets.last().copied().unwrap_or(0.0) + 100.0;
        t.event(
            &Event::CursorMoved(crate::Point::new(end_x, 0.0)),
            area,
            &mut Vec::new(),
        );
        t.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: false,
                position: crate::Point::new(end_x, 0.0),
            },
            area,
            &mut Vec::new(),
        );
        assert_eq!(t.selected_text(), Some("Hello".to_string()));
        assert!(!t.dragging);
    }

    #[test]
    fn ime_area_follows_caret_after_paint() {
        let mut t = input();
        // 将光标移到开头,避免光标偏移干扰原点判断。
        t.cursor = 0;
        t.anchor = 0;

        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 100.0)), &mut texts);

        // paint 前 area 为本地原点,IME 区域应位于 (padding.left, padding.top)。
        let local = t.ime_area().unwrap();
        assert!((local.origin.x - t.padding.left).abs() < f32::EPSILON);
        assert!((local.origin.y - t.padding.top).abs() < f32::EPSILON);

        // paint 后缓存绝对矩形,IME 区域应跟随光标平移。
        let abs = Rect::from_xywh(20.0, 30.0, 500.0, 100.0);
        let mut rects = crate::RectBatch::new();
        t.paint(abs, &mut rects, &mut texts);

        let area = t.ime_area().unwrap();
        let expected_x = abs.origin.x + t.padding.left;
        let expected_y = abs.origin.y + t.padding.top;
        assert!((area.origin.x - expected_x).abs() < f32::EPSILON);
        assert!((area.origin.y - expected_y).abs() < f32::EPSILON);
        assert_eq!(area.size.width, 0.0);
        assert_eq!(area.size.height, t.line_height);
    }
}
