//! @author 十四叔
//! @date 2026/07/18

//! TextInput / TextArea 共享的纯文本编辑状态与操作。
//!
//! 封装文本、光标、选区、插入删除以及撤销/重做逻辑,
//! 本身不关心单行/多行布局或渲染。

use std::any::Any;

use crate::widget::MsgQueue;

/// 撤销栈最大深度。
pub(crate) const MAX_UNDO: usize = 100;

/// 编辑前快照(用于撤销/重做)。
#[derive(Debug, Clone)]
struct Snapshot {
    text: String,
    cursor: usize,
    anchor: usize,
}

/// 文本变化回调:返回一条应用消息。
pub(crate) type ChangeFactory = Box<dyn Fn(&str) -> Box<dyn Any>>;

/// 通用文本编辑状态。
pub(crate) struct TextEditor {
    /// 当前文本内容。
    text: String,
    /// 光标位置(字符索引,0..=char_count)。
    cursor: usize,
    /// 选区锚点(字符索引);与 cursor 相等表示无选区。
    anchor: usize,
    /// 撤销栈。
    undo_stack: Vec<Snapshot>,
    /// 重做栈。
    redo_stack: Vec<Snapshot>,
    /// 正在执行撤销/重做,避免重复记录快照。
    is_undoing: bool,
    /// 文本变化时产出的应用消息。
    on_change: Option<ChangeFactory>,
}

impl TextEditor {
    /// 创建空编辑器。
    pub(crate) fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            anchor: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            is_undoing: false,
            on_change: None,
        }
    }

    /// 当前文本。
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// 设置文本,光标与锚点移到末尾,并清空撤销/重做栈。
    pub(crate) fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cursor = self.text.chars().count();
        self.anchor = self.cursor;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// 光标位置。
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    /// 设置光标位置。
    pub(crate) fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    /// 选区锚点(测试用)。
    #[cfg(test)]
    pub(crate) fn anchor(&self) -> usize {
        self.anchor
    }

    /// 设置选区锚点。
    pub(crate) fn set_anchor(&mut self, anchor: usize) {
        self.anchor = anchor;
    }

    /// 设置文本变化回调(每次编辑后触发)。
    pub(crate) fn on_change<M: 'static>(mut self, f: impl Fn(&str) -> M + 'static) -> Self {
        self.on_change = Some(Box::new(move |text| Box::new(f(text)) as Box<dyn Any>));
        self
    }

    /// 通知应用文本已变化。
    pub(crate) fn notify_change(&self, msgs: &mut MsgQueue) {
        if let Some(factory) = &self.on_change {
            msgs.push(factory(&self.text));
        }
    }

    /// 选区范围(起点,终点),保证 start <= end。
    pub(crate) fn selection_range(&self) -> (usize, usize) {
        if self.anchor <= self.cursor {
            (self.anchor, self.cursor)
        } else {
            (self.cursor, self.anchor)
        }
    }

    /// 当前选中的文本。
    pub(crate) fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection_range();
        if start == end {
            return None;
        }
        let start_byte = char_to_byte(&self.text, start);
        let end_byte = char_to_byte(&self.text, end);
        Some(self.text[start_byte..end_byte].to_string())
    }

    /// 全选文本。
    pub(crate) fn select_all(&mut self) {
        self.cursor = self.text.chars().count();
        self.anchor = 0;
    }

    /// 删除当前选区并记录撤销快照(用于 Cut)。
    pub(crate) fn cut_selection(&mut self) {
        let (start, end) = self.selection_range();
        if start != end {
            self.save_undo();
            self.delete_selection();
        }
    }

    /// 删除当前选区(若存在)。
    pub(crate) fn delete_selection(&mut self) {
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

    /// 在光标处插入文本,并记录撤销快照。
    pub(crate) fn insert(&mut self, text: &str) {
        self.save_undo();
        self.delete_selection();
        let byte_idx = char_to_byte(&self.text, self.cursor);
        self.text.insert_str(byte_idx, text);
        self.cursor += text.chars().count();
        self.anchor = self.cursor;
    }

    /// 删除光标前一个字符;返回是否发生了实际变更。
    pub(crate) fn backspace(&mut self) -> bool {
        let (start, end) = self.selection_range();
        if start != end {
            self.save_undo();
            self.delete_selection();
            return true;
        }
        if self.cursor == 0 {
            return false;
        }
        self.save_undo();
        let byte_idx = char_to_byte(&self.text, self.cursor);
        let prev = self.text[..byte_idx]
            .chars()
            .next_back()
            .map(|c| c.len_utf8())
            .unwrap_or(0);
        self.text.drain((byte_idx - prev)..byte_idx);
        self.cursor -= 1;
        self.anchor = self.cursor;
        true
    }

    /// 删除光标后一个字符;返回是否发生了实际变更。
    pub(crate) fn delete(&mut self) -> bool {
        let (start, end) = self.selection_range();
        if start != end {
            self.save_undo();
            self.delete_selection();
            return true;
        }
        if self.cursor >= self.text.chars().count() {
            return false;
        }
        self.save_undo();
        let byte_idx = char_to_byte(&self.text, self.cursor);
        let len = self.text[byte_idx..]
            .chars()
            .next()
            .map(|c| c.len_utf8())
            .unwrap_or(0);
        self.text.drain(byte_idx..(byte_idx + len));
        true
    }

    /// 水平移动光标。
    pub(crate) fn move_cursor(&mut self, delta: isize, extend_selection: bool) {
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

    /// 在变更前保存快照到撤销栈。
    fn save_undo(&mut self) {
        if self.is_undoing {
            return;
        }
        if self.undo_stack.len() >= MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(Snapshot {
            text: self.text.clone(),
            cursor: self.cursor,
            anchor: self.anchor,
        });
        self.redo_stack.clear();
    }

    /// 撤销上一次编辑;返回是否实际应用了快照。
    pub(crate) fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                text: self.text.clone(),
                cursor: self.cursor,
                anchor: self.anchor,
            });
            self.is_undoing = true;
            self.text = snapshot.text;
            self.cursor = snapshot.cursor;
            self.anchor = snapshot.anchor;
            self.is_undoing = false;
            true
        } else {
            false
        }
    }

    /// 重做上一次撤销;返回是否实际应用了快照。
    pub(crate) fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                text: self.text.clone(),
                cursor: self.cursor,
                anchor: self.anchor,
            });
            self.is_undoing = true;
            self.text = snapshot.text;
            self.cursor = snapshot.cursor;
            self.anchor = snapshot.anchor;
            self.is_undoing = false;
            true
        } else {
            false
        }
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// 字符索引转字节索引。
pub(crate) fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> TextEditor {
        TextEditor::new()
    }

    #[test]
    fn insert_adds_text_and_moves_cursor() {
        let mut e = editor();
        e.insert("Hello");
        assert_eq!(e.text(), "Hello");
        assert_eq!(e.cursor(), 5);
        assert_eq!(e.anchor(), 5);
    }

    #[test]
    fn insert_replaces_selection() {
        let mut e = TextEditor::new();
        e.set_text("Hello");
        e.set_cursor(1);
        e.set_anchor(4); // 选中 "ell"
        e.insert("?");
        assert_eq!(e.text(), "H?o");
        assert_eq!(e.cursor(), 2);
    }

    #[test]
    fn backspace_deletes_selection_or_previous_char() {
        let mut e = TextEditor::new();
        e.set_text("Hello");
        assert!(e.backspace());
        assert_eq!(e.text(), "Hell");
        assert_eq!(e.cursor(), 4);

        e.set_cursor(0);
        e.set_anchor(0);
        assert!(!e.backspace());
        assert_eq!(e.text(), "Hell");
    }

    #[test]
    fn delete_deletes_selection_or_next_char() {
        let mut e = TextEditor::new();
        e.set_text("Hello");
        e.set_cursor(0);
        e.set_anchor(0);
        assert!(e.delete());
        assert_eq!(e.text(), "ello");
        assert_eq!(e.cursor(), 0);

        e.set_cursor(3);
        e.set_text("ell");
        assert!(!e.delete());
    }

    #[test]
    fn undo_redo_cycle() {
        let mut e = editor();
        e.set_text("Hello");
        e.insert("!");
        assert_eq!(e.text(), "Hello!");

        e.undo();
        assert_eq!(e.text(), "Hello");

        e.redo();
        assert_eq!(e.text(), "Hello!");
    }

    #[test]
    fn undo_restores_cursor_and_anchor() {
        let mut e = TextEditor::new();
        e.set_text("Hello");
        e.set_cursor(1);
        e.set_anchor(4); // 选中 "ell"
        e.insert("?");
        assert_eq!(e.text(), "H?o");

        e.undo();
        assert_eq!(e.text(), "Hello");
        assert_eq!(e.cursor(), 1);
        assert_eq!(e.anchor(), 4);
    }

    #[test]
    fn new_edit_clears_redo_stack() {
        let mut e = editor();
        e.set_text("Hello");
        e.insert("!");
        e.undo();
        assert_eq!(e.text(), "Hello");

        e.insert("?");
        e.redo();
        assert_eq!(e.text(), "Hello?");
    }

    #[test]
    fn undo_stack_depth_limit() {
        let mut e = editor();
        for i in 0..=MAX_UNDO {
            e.insert(&i.to_string());
        }
        for _ in 0..MAX_UNDO {
            e.undo();
        }
        let after_undos = e.text().to_string();
        e.undo();
        assert_eq!(e.text(), after_undos);
    }

    #[test]
    fn move_cursor_clamps_to_bounds() {
        let mut e = TextEditor::new();
        e.set_text("ab");
        e.move_cursor(-1, false);
        assert_eq!(e.cursor(), 1);
        e.move_cursor(10, false);
        assert_eq!(e.cursor(), 2);
        e.set_cursor(0);
        e.set_anchor(0);
        e.move_cursor(1, true);
        assert_eq!(e.cursor(), 1);
        assert_eq!(e.anchor(), 0);
    }
}
