//! @author 十四叔
//! @date 2026/07/18

//! TextArea 组件:多行可编辑文本。
//!
//! 支持显式换行、按字符 soft-wrap、光标/选区、键盘编辑、
//! IME preedit、剪贴板复制/剪切/粘贴以及鼠标点击定位光标。

use std::cell::Cell;

use crate::app::AnimationCtx;
use crate::event::{Event, ImeEvent, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::text::{Line, break_lines};
use crate::widget::text_editor::{TextEditor, char_to_byte};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, Edges, Point, Rect, Size};

/// 光标闪烁周期(秒)。
const BLINK_PERIOD: f32 = 0.5;

/// 多行文本输入组件。
pub struct TextArea {
    /// 共享文本编辑状态。
    editor: TextEditor,
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
    /// 每行文本的字符区间与宽度。
    lines: Vec<Line>,
    /// 每行每个字符右侧的 x 偏移,用于命中测试。
    char_offsets: Vec<Vec<f32>>,
    /// 行高。
    line_height: f32,
    /// 光标可见性(由动画控制闪烁)。
    caret_visible: bool,
    /// IME 合成文本。
    preedit: Option<String>,
    /// 鼠标拖拽选区状态。
    dragging: bool,
}

impl TextArea {
    /// 创建多行文本输入框(默认空文本、字号 16、深色文本、浅色背景)。
    pub fn new() -> Self {
        Self {
            editor: TextEditor::new(),
            focused: false,
            font_size: 16,
            color: Color::from_srgb8(0x22, 0x22, 0x22),
            background: Color::WHITE,
            selection_color: Color::from_srgb8(0xB3, 0xD7, 0xFF),
            caret_color: Color::from_srgb8(0x1E, 0x90, 0xFF),
            padding: Edges::symmetric(12.0, 8.0),
            width: None,
            area: Cell::new(Rect::default()),
            lines: vec![Line::empty()],
            char_offsets: vec![Vec::new()],
            line_height: 0.0,
            caret_visible: true,
            preedit: None,
            dragging: false,
        }
    }

    /// 设置初始文本。
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.editor.set_text(text);
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
        self.editor = self.editor.on_change(f);
        self
    }

    /// 当前文本(不含 preedit)。
    pub fn value(&self) -> &str {
        self.editor.text()
    }

    /// 光标位置(测试用)。
    #[cfg(test)]
    pub(crate) fn cursor(&self) -> usize {
        self.editor.cursor()
    }

    /// 设置光标位置(测试用)。
    #[cfg(test)]
    pub(crate) fn set_cursor(&mut self, cursor: usize) {
        self.editor.set_cursor(cursor);
    }

    /// 选区锚点(测试用)。
    #[cfg(test)]
    pub(crate) fn anchor(&self) -> usize {
        self.editor.anchor()
    }

    /// 设置选区锚点(测试用)。
    #[cfg(test)]
    pub(crate) fn set_anchor(&mut self, anchor: usize) {
        self.editor.set_anchor(anchor);
    }

    /// 重新计算行排版与每行字符偏移。
    fn rebuild_lines(&mut self, texts: &mut TextBatch, content_width: f32) {
        self.lines = break_lines(self.editor.text(), content_width, &mut |ch| {
            texts.measure(&ch.to_string(), self.font_size)
        });

        self.char_offsets.clear();
        for line in &self.lines {
            let mut offsets = Vec::with_capacity(line.len());
            let start_byte = char_to_byte(self.editor.text(), line.start);
            let end_byte = char_to_byte(self.editor.text(), line.end);
            let mut x = 0.0f32;
            for ch in self.editor.text()[start_byte..end_byte].chars() {
                x += texts.measure(&ch.to_string(), self.font_size);
                offsets.push(x);
            }
            self.char_offsets.push(offsets);
        }
    }

    /// 返回光标所在行的索引。
    fn cursor_line(&self) -> usize {
        self.line_index_of_char(self.editor.cursor())
    }

    /// 返回字符索引所在行的索引。
    ///
    /// 字符索引等于某行 end 时,归该行所有(表示行尾插入位置)。
    fn line_index_of_char(&self, char_idx: usize) -> usize {
        self.lines
            .iter()
            .position(|line| char_idx <= line.end)
            .unwrap_or(self.lines.len().saturating_sub(1))
    }

    /// 测量到指定行指定列的文本宽度。
    fn measure_to(&self, line_idx: usize, col: usize) -> f32 {
        if col == 0 {
            return 0.0;
        }
        let offsets = &self.char_offsets[line_idx];
        offsets.get(col - 1).copied().unwrap_or(0.0)
    }

    /// 将本地 x 坐标(相对于文本起点)转换为列数。
    fn hit_to_col(&self, line_idx: usize, local_x: f32) -> usize {
        let offsets = &self.char_offsets[line_idx];
        if offsets.is_empty() {
            return 0;
        }
        offsets
            .partition_point(|offset| *offset <= local_x)
            .min(offsets.len())
    }

    /// 将本地 (x, y) 转换为字符索引。
    fn hit_to_index(&self, local_x: f32, local_y: f32) -> usize {
        let line_idx = if local_y <= self.padding.top {
            0
        } else {
            let idx = ((local_y - self.padding.top) / self.line_height).floor() as usize;
            idx.min(self.lines.len().saturating_sub(1))
        };
        let line = self.lines[line_idx];
        let col = self.hit_to_col(line_idx, local_x - self.padding.left);
        (line.start + col).min(line.end)
    }

    /// 在光标处插入文本。
    fn insert(&mut self, text: &str) {
        self.editor.insert(text);
    }

    /// 通知应用文本已变化。
    fn notify_change(&self, msgs: &mut MsgQueue) {
        self.editor.notify_change(msgs);
    }

    /// 全选文本。
    fn select_all(&mut self) {
        self.editor.select_all();
    }

    /// 选区范围(起点,终点),保证 start <= end。
    fn selection_range(&self) -> (usize, usize) {
        self.editor.selection_range()
    }

    /// 水平移动光标。
    fn move_cursor_horizontal(&mut self, delta: isize, extend_selection: bool) {
        self.editor.move_cursor(delta, extend_selection);
    }

    /// 垂直移动光标(按行)。
    fn move_cursor_vertical(&mut self, delta: isize, extend_selection: bool) {
        let line_idx = self.cursor_line();
        let col = self.editor.cursor() - self.lines[line_idx].start;
        let target_idx = if delta < 0 {
            line_idx.saturating_sub(delta.unsigned_abs())
        } else {
            (line_idx + delta as usize).min(self.lines.len() - 1)
        };
        let target_line = self.lines[target_idx];
        let target_col = col.min(target_line.len());
        self.editor.set_cursor(target_line.start + target_col);
        if !extend_selection {
            self.editor.set_anchor(self.editor.cursor());
        }
    }

    /// 撤销上一次编辑。
    fn undo(&mut self) -> bool {
        self.editor.undo()
    }

    /// 重做上一次撤销。
    fn redo(&mut self) -> bool {
        self.editor.redo()
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextArea {
    fn animate(&mut self, ctx: &AnimationCtx) {
        if self.focused {
            let t = ctx.elapsed.as_secs_f32();
            self.caret_visible = (t % (BLINK_PERIOD * 2.0)) < BLINK_PERIOD;
        } else {
            self.caret_visible = false;
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.line_height = texts.line_height(f32::from(self.font_size));
        let content_width =
            self.width.unwrap_or(constraints.max_width).max(0.0) - self.padding.horizontal();
        self.rebuild_lines(texts, content_width.max(0.0));

        let max_line_width = self.lines.iter().map(|l| l.width).fold(0.0, f32::max);
        let width = self
            .width
            .unwrap_or(constraints.max_width)
            .max(max_line_width + self.padding.horizontal());
        let height = self.lines.len() as f32 * self.line_height + self.padding.vertical();
        let size = constraints.constrain(Size::new(width, height));
        self.area.set(Rect::new(Point::ZERO, size));
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        self.area.set(area);

        // 背景
        rects.push_rect(area, self.background, 4.0);

        let text_x = area.origin.x + self.padding.left;
        let ascent = texts.ascent(f32::from(self.font_size));
        let (sel_start, sel_end) = self.selection_range();

        for (line_idx, line) in self.lines.iter().enumerate() {
            let line_y = area.origin.y + self.padding.top + line_idx as f32 * self.line_height;
            let baseline = line_y + ascent;

            // 选区高亮
            if sel_start < sel_end && self.focused {
                let line_sel_start = sel_start.max(line.start).min(line.end);
                let line_sel_end = sel_end.max(line.start).min(line.end);
                if line_sel_start < line_sel_end {
                    let x0 = text_x + self.measure_to(line_idx, line_sel_start - line.start);
                    let x1 = text_x + self.measure_to(line_idx, line_sel_end - line.start);
                    rects.push_rect(
                        Rect::from_xywh(x0, line_y, x1 - x0, self.line_height),
                        self.selection_color,
                        0.0,
                    );
                }
            }

            // 基础文本
            let start_byte = char_to_byte(self.editor.text(), line.start);
            let end_byte = char_to_byte(self.editor.text(), line.end);
            texts.push_text(
                &self.editor.text()[start_byte..end_byte],
                text_x,
                baseline,
                self.font_size,
                self.color,
            );
        }

        // preedit 文本与下划线(显示在光标处)。
        if let Some(preedit) = &self.preedit {
            let line_idx = self.cursor_line();
            let line = self.lines[line_idx];
            let col = self.editor.cursor() - line.start;
            let pre_x = text_x + self.measure_to(line_idx, col);
            let pre_y = area.origin.y + self.padding.top + line_idx as f32 * self.line_height;
            let baseline = pre_y + ascent;
            texts.push_text(preedit, pre_x, baseline, self.font_size, self.color);
            let pre_width = texts.measure(preedit, self.font_size);
            let underline_y = baseline + self.line_height * 0.15;
            rects.push_rect(
                Rect::from_xywh(pre_x, underline_y, pre_width, 1.0),
                self.color,
                0.0,
            );
        }

        // 光标
        if self.focused && self.caret_visible {
            let line_idx = self.cursor_line();
            let line = self.lines[line_idx];
            let col = self.editor.cursor() - line.start;
            let caret_x = text_x + self.measure_to(line_idx, col);
            let caret_y = area.origin.y + self.padding.top + line_idx as f32 * self.line_height;
            rects.push_rect(
                Rect::from_xywh(caret_x, caret_y, 2.0, self.line_height),
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
                Key::Named(NamedKey::Space) => {
                    self.insert(" ");
                    changed = true;
                    EventResult::Consumed
                }
                Key::Character(s) if *ctrl && s == "a" => {
                    self.select_all();
                    EventResult::Consumed
                }
                Key::Character(s) if *ctrl && s == "z" => {
                    if *shift {
                        changed = self.redo();
                    } else {
                        changed = self.undo();
                    }
                    EventResult::Consumed
                }
                Key::Character(s) if *ctrl && s == "y" => {
                    changed = self.redo();
                    EventResult::Consumed
                }
                Key::Named(NamedKey::ArrowLeft) => {
                    self.move_cursor_horizontal(-1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::ArrowRight) => {
                    self.move_cursor_horizontal(1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::ArrowUp) => {
                    self.move_cursor_vertical(-1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::ArrowDown) => {
                    self.move_cursor_vertical(1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Home) => {
                    let line = self.lines[self.cursor_line()];
                    self.editor.set_cursor(line.start);
                    if !shift {
                        self.editor.set_anchor(line.start);
                    }
                    EventResult::Consumed
                }
                Key::Named(NamedKey::End) => {
                    let line = self.lines[self.cursor_line()];
                    self.editor.set_cursor(line.end);
                    if !shift {
                        self.editor.set_anchor(line.end);
                    }
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Backspace) => {
                    changed = self.editor.backspace();
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Delete) => {
                    changed = self.editor.delete();
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Enter) => {
                    self.insert("\n");
                    changed = true;
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
                    self.editor.cut_selection();
                    changed = true;
                }
                EventResult::Consumed
            }
            Event::Paste => EventResult::Consumed,
            Event::CursorMoved(p) => {
                if self.dragging {
                    let local_x = p.x - area.origin.x;
                    let local_y = p.y - area.origin.y;
                    self.editor.set_cursor(self.hit_to_index(local_x, local_y));
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
                if area.contains(*position) {
                    let local_x = position.x - area.origin.x;
                    let local_y = position.y - area.origin.y;
                    self.editor.set_cursor(self.hit_to_index(local_x, local_y));
                    self.editor.set_anchor(self.editor.cursor());
                    self.dragging = true;
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
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
        self.editor.selected_text()
    }

    fn wants_ime(&self) -> bool {
        true
    }

    fn ime_area(&self) -> Option<Rect> {
        let area = self.area.get();
        let line_idx = self.cursor_line();
        let line = self.lines[line_idx];
        let col = self.editor.cursor() - line.start;
        let cursor_x = self.measure_to(line_idx, col);
        let x = area.origin.x + self.padding.left + cursor_x;
        let y = area.origin.y + self.padding.top + line_idx as f32 * self.line_height;
        Some(Rect::from_xywh(x, y, 0.0, self.line_height))
    }

    fn hit_area(&self) -> Option<Rect> {
        Some(self.area.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area() -> Rect {
        Rect::from_xywh(0.0, 0.0, 500.0, 500.0)
    }

    #[test]
    fn insert_and_newline() {
        let mut t = TextArea::new();
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        t.insert("ab");
        t.insert("\n");
        t.insert("c");
        assert_eq!(t.value(), "ab\nc");
        assert_eq!(t.cursor(), 4);
    }

    #[test]
    fn space_inserts_space() {
        let mut t = TextArea::new();
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        t.insert("ab");
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Space),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "ab ");
        assert_eq!(t.cursor(), 3);
    }

    #[test]
    fn arrow_up_down_moves_between_lines() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        // 光标在末尾(第 1 行第 2 列之后)。
        assert_eq!(t.cursor(), 5);

        t.move_cursor_vertical(-1, false);
        // 移到上一行,列数 clamp 到上一行长度 2
        assert_eq!(t.cursor(), 2);

        t.move_cursor_vertical(1, false);
        assert_eq!(t.cursor(), 5);
    }

    #[test]
    fn home_end_keys_move_per_line() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        t.set_cursor(5); // 第 1 行末尾
        t.set_anchor(5);

        // End 应跳到行尾
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::End),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.cursor(), 5);

        // Home 应跳到行首
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Home),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.cursor(), 3);
    }

    #[test]
    fn backspace_across_newline() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        t.set_cursor(3); // 第 1 行开头
        t.set_anchor(3);
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Backspace),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "abcd");
        assert_eq!(t.cursor(), 2);
    }

    #[test]
    fn delete_across_newline() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        t.set_cursor(2); // 第 0 行末尾
        t.set_anchor(2);
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Delete),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "abcd");
        assert_eq!(t.cursor(), 2);
    }

    #[test]
    fn ctrl_a_selects_all() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        t.event(&Event::FocusIn, area(), &mut Vec::new());
        t.event(
            &Event::Key {
                key: Key::Character("a".to_string()),
                pressed: true,
                shift: false,
                ctrl: true,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.selected_text(), Some("ab\ncd".to_string()));
    }

    #[test]
    fn undo_restores_deleted_text() {
        let mut t = TextArea::new().text("Hello");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Backspace),
                pressed: true,
                shift: false,
                ctrl: false,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hell");

        t.event(
            &Event::Key {
                key: Key::Character("z".to_string()),
                pressed: true,
                shift: false,
                ctrl: true,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello");
        assert_eq!(t.cursor(), 5);
    }

    #[test]
    fn undo_redo_cycle() {
        let mut t = TextArea::new().text("Hello");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        t.insert("!");
        assert_eq!(t.value(), "Hello!");

        t.undo();
        assert_eq!(t.value(), "Hello");

        t.redo();
        assert_eq!(t.value(), "Hello!");
    }

    #[test]
    fn redo_via_keyboard_shortcuts() {
        let mut t = TextArea::new().text("Hello");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        t.insert("!");
        t.undo();
        assert_eq!(t.value(), "Hello");

        // Ctrl+Shift+Z
        t.event(
            &Event::Key {
                key: Key::Character("z".to_string()),
                pressed: true,
                shift: true,
                ctrl: true,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello!");

        t.undo();
        // Ctrl+Y
        t.event(
            &Event::Key {
                key: Key::Character("y".to_string()),
                pressed: true,
                shift: false,
                ctrl: true,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello!");
    }

    #[test]
    fn undo_restores_multiline_cursor_and_anchor() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        // 选中 "b\nc"
        t.set_cursor(1);
        t.set_anchor(4);
        t.insert("?");
        assert_eq!(t.value(), "a?d");

        t.undo();
        assert_eq!(t.value(), "ab\ncd");
        assert_eq!(t.cursor(), 1);
        assert_eq!(t.anchor(), 4);
    }

    #[test]
    fn new_edit_clears_redo_stack() {
        let mut t = TextArea::new().text("Hello");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        t.insert("!");
        t.undo();
        assert_eq!(t.value(), "Hello");

        t.insert("?");
        t.redo();
        assert_eq!(t.value(), "Hello?");
    }

    #[test]
    fn undo_stack_depth_limit() {
        let mut t = TextArea::new();
        for i in 0..=crate::widget::text_editor::MAX_UNDO {
            t.insert(&i.to_string());
        }
        for _ in 0..crate::widget::text_editor::MAX_UNDO {
            t.undo();
        }
        let after_undos = t.value().to_string();
        t.undo();
        assert_eq!(t.value(), after_undos);
    }

    #[test]
    fn mouse_click_positions_cursor() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        // 点击第一行靠左,光标应接近行首。
        t.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: Point::new(0.0, 10.0),
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.cursor(), 0);
    }

    #[test]
    fn ime_area_follows_caret() {
        let mut t = TextArea::new().text("ab\ncd");
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);
        t.set_cursor(3);
        t.set_anchor(3);

        let mut rects = crate::RectBatch::new();
        t.paint(area(), &mut rects, &mut texts);
        let ime = t.ime_area().unwrap();
        assert!(ime.origin.y > area().origin.y + t.padding.top);
    }
}
