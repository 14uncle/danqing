//! @author 十四叔
//! @date 2026/07/18

//! TextArea 组件:多行可编辑文本。
//!
//! 支持显式换行、按字符 soft-wrap、光标/选区、键盘编辑、
//! IME preedit、剪贴板复制/剪切/粘贴以及鼠标点击定位光标。

use std::any::Any;
use std::cell::Cell;

use crate::app::AnimationCtx;
use crate::event::{Event, ImeEvent, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::text::{Line, break_lines};
use crate::widget::form::text_editor::{TextEditor, char_to_byte};
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, Edges, LightTheme, Point, Rect, Size, Theme};

/// 光标闪烁周期(秒)。
const BLINK_PERIOD: f32 = 0.5;

/// 主题绑定：每帧从应用状态产出随主题流动的颜色。
type ThemeBinding = Box<dyn Fn(&dyn Any) -> TextAreaColors>;

/// 随主题流动的多行输入框颜色子集 (构建后仍可经 [`TextArea::bind_theme`] 每帧刷新)。
///
/// **为什么需要它**: 视图树只在启动时构建一次 (`danqing/src/window/mod.rs` 的
/// `let tree = app.view();`, 之后整棵交给 Handler 不再重建), 所以 `themed(&theme)`
/// 烘进去的颜色不会跟随运行时切主题。`TextArea` 的主题色**没有任何每帧通路**。
///
/// 形状与 [`crate::widget::TitleBar::bind_theme`] 一致 —— 后续组件要补时请沿用。
#[derive(Debug, Clone, Copy, PartialEq)]
struct TextAreaColors {
    text: Color,
    background: Color,
    selection: Color,
    caret: Color,
    border: Color,
    focus_border: Color,
}

impl TextAreaColors {
    /// 从主题解析 —— `themed` 与 `bind_theme` **共用这一份**, 免得两处各写一套口径。
    fn from_theme(theme: &impl Theme) -> Self {
        Self {
            text: theme.text_primary(),
            background: theme.surface_input(),
            selection: theme.selection(),
            caret: theme.caret(),
            border: theme.border(),
            focus_border: theme.accent(),
        }
    }
}

/// 多行文本输入组件。
pub struct TextArea {
    /// 共享文本编辑状态。
    editor: TextEditor,
    /// 主题绑定 (每帧刷新随主题流动的颜色; 见 [`TextArea::bind_theme`])。
    theme_binding: Option<ThemeBinding>,
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
    /// 背景圆角半径。
    radius: f32,
    /// 边框颜色。
    border_color: Color,
    /// 获得焦点时的边框颜色。
    focus_border_color: Color,
    /// 边框粗细。
    border_width: f32,
    /// 显式宽度(未指定则按约束上限)。
    width: Option<f32>,
    /// 最小高度(未指定则只随内容增长;内容超高时仍随内容,供 Scrollable 滚动)。
    height: Option<f32>,
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
    /// 创建多行文本输入框,使用默认浅色主题 token。
    pub fn new() -> Self {
        Self::themed(&LightTheme)
    }

    /// 使用指定主题创建多行文本输入框。
    pub fn themed(theme: &impl Theme) -> Self {
        let colors = TextAreaColors::from_theme(theme);
        Self {
            editor: TextEditor::new(),
            theme_binding: None,
            focused: false,
            font_size: theme.font_size_body(),
            color: colors.text,
            background: colors.background,
            selection_color: colors.selection,
            caret_color: colors.caret,
            padding: Edges::symmetric(theme.spacing_md(), theme.spacing_sm()),
            radius: theme.radius_sm(),
            border_color: colors.border,
            focus_border_color: colors.focus_border,
            border_width: 1.0,
            width: None,
            height: None,
            area: Cell::new(Rect::default()),
            lines: vec![Line::empty()],
            char_offsets: vec![Vec::new()],
            line_height: 0.0,
            caret_visible: true,
            preedit: None,
            dragging: false,
        }
    }

    /// 绑定主题：每帧从应用状态重取主题，刷新随主题流动的颜色
    /// (正文/底色/选区/光标/边框/焦点边框); 其余规格 (字号、内边距、圆角、
    /// 边框粗细) 保持构建时的值。
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
                .expect("TextArea 主题绑定的状态类型不匹配");
            TextAreaColors::from_theme(&f(state))
        }));
        self
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

    /// 设置背景圆角半径。
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// 设置显式宽度。
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// 设置最小高度。
    ///
    /// 内容不足时保持该高度 (如填满 Scrollable 视口,避免背景接缝);
    /// 内容超高时仍随内容增长, 由外层 Scrollable 滚动。
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
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

    /// 当前背景色(测试用)。
    #[cfg(test)]
    pub(crate) fn background_color(&self) -> Color {
        self.background
    }

    /// 当前文本颜色(测试用)。
    #[cfg(test)]
    pub(crate) fn text_color_value(&self) -> Color {
        self.color
    }

    /// 当前光标所在行索引(测试用)。
    #[cfg(test)]
    pub(crate) fn cursor_line_index(&self) -> usize {
        self.cursor_line()
    }

    /// 当前圆角半径(测试用)。
    #[cfg(test)]
    pub(crate) fn radius_value(&self) -> f32 {
        self.radius
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
    fn sync(&mut self, state: &dyn Any) {
        if let Some(binding) = &self.theme_binding {
            let c = binding(state);
            self.color = c.text;
            self.background = c.background;
            self.selection_color = c.selection;
            self.caret_color = c.caret;
            self.border_color = c.border;
            self.focus_border_color = c.focus_border;
        }
    }

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
        let content_height = self.lines.len() as f32 * self.line_height + self.padding.vertical();
        let height = content_height.max(self.height.unwrap_or(0.0));
        let size = constraints.constrain(Size::new(width, height));
        self.area.set(Rect::new(Point::ZERO, size));
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        self.area.set(area);

        // 背景与边框共用同一份像素对齐几何: 轮廓精确重合 (贴合),
        // 且 1px 描边落在完整像素行上满强度渲染 (底边发虚的根因对策)。
        let surface = area.snap_to_pixels();
        rects.push_rect(surface, self.background, self.radius);

        // 边框: 聚焦时使用 accent,否则使用默认边框色。
        let border_color = if self.focused {
            self.focus_border_color
        } else {
            self.border_color
        };
        rects.push_rounded_border(surface, border_color, self.radius, self.border_width);

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
                ..
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

    /// 重置焦点视觉: 与 FocusOut 同语义 (清聚焦/IME 合成/拖拽/光标),
    /// 但隐藏面板收不到 FocusOut, 须主动清除 (面板隐藏时被容器调用)。
    fn reset_focus(&mut self) {
        self.focused = false;
        self.preedit = None;
        self.dragging = false;
        self.caret_visible = false;
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
    use crate::LightTheme;

    /// `bind_theme` 必须**每帧重取**主题 —— 视图树只在启动时构建一次,
    /// 所以 `themed(&theme)` 烘进去的颜色不会跟着运行时切主题走。
    ///
    /// 用 `SceneTheme` 作测试主题: `LightTheme` 与 `DarkTheme` 是**两个类型**,
    /// 而 `bind_theme<S, T>` 要求 `T` 单一, 没法用它们构造「同一类型、两个主题」。
    /// `SceneTheme` 是「一个类型 + 可换调色板」, 正好。
    #[test]
    fn bind_theme_refreshes_colors_each_frame() {
        use crate::theme::{ScenePalette, SceneTheme};

        fn scene(text_primary: Color, accent: Color) -> SceneTheme {
            SceneTheme::new(ScenePalette {
                base: Color::WHITE,
                accent,
                text_primary,
                text_secondary: Color::WHITE,
                surface: Color::WHITE,
                surface_input: Color::WHITE,
                backdrop_light: Color::WHITE,
                backdrop_dark: Color::WHITE,
            })
        }
        struct S(SceneTheme);

        let mut area = TextArea::themed(&LightTheme).bind_theme(|s: &S| s.0);
        area.sync(&S(scene(
            Color::from_srgb8(1, 2, 3),
            Color::from_srgb8(4, 5, 6),
        )));

        assert_eq!(area.color, Color::from_srgb8(1, 2, 3), "正文色应来自新主题");
        assert_eq!(
            area.focus_border_color,
            Color::from_srgb8(4, 5, 6),
            "焦点边框色 = 新主题的 accent"
        );
    }

    #[test]
    fn text_area_uses_theme_defaults() {
        let area = TextArea::new();
        assert_eq!(area.text_color_value(), LightTheme.text_primary());
        assert_eq!(area.background_color(), LightTheme.surface_input());
        assert_eq!(area.radius_value(), LightTheme.radius_sm());
    }

    #[test]
    fn text_area_themed_uses_provided_theme() {
        let area = TextArea::themed(&LightTheme);
        assert_eq!(area.text_color_value(), LightTheme.text_primary());
        assert_eq!(area.background_color(), LightTheme.surface_input());
    }

    #[test]
    fn text_area_custom_overrides_theme() {
        let custom_bg = Color::from_srgb8(255, 0, 0);
        let area = TextArea::new().background(custom_bg).radius(8.0);
        assert_eq!(area.background_color(), custom_bg);
        assert_eq!(area.radius_value(), 8.0);
    }

    fn area() -> Rect {
        Rect::from_xywh(0.0, 0.0, 500.0, 500.0)
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

    /// 外框**上边**那条描边段的颜色。
    ///
    /// 按几何定位而非「批次里出现过 accent」: accent 同时用于光标与选区, 后者在
    /// 焦点取色失效时照样存在 —— 那种写法会让本测试因错误的原因变绿。
    fn top_border_color(ta: &mut TextArea) -> [f32; 4] {
        let a = area();
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        ta.paint(a, &mut rects, &mut texts);
        let colors = rects.instance_colors();
        let idx = rects
            .instance_rects()
            .iter()
            .position(|r| r.origin.y == a.origin.y && r.size.height == 1.0 && r.size.width > 1.0)
            .expect("外框上边应被绘制");
        colors[idx]
    }

    #[test]
    fn focus_in_turns_the_border_accent_and_focus_out_restores_it() {
        let mut ta = TextArea::new();
        let resting = rgba_of(LightTheme.border());
        let focused = rgba_of(LightTheme.accent());
        assert_ne!(
            resting, focused,
            "前提: 两个 token 必须不同, 否则本测试无法证伪"
        );

        assert_eq!(top_border_color(&mut ta), resting, "静默态: 常规边框色");

        let mut msgs = MsgQueue::new();
        assert_eq!(
            ta.event(&Event::FocusIn, area(), &mut msgs),
            EventResult::Consumed,
            "FocusIn 应被消费"
        );
        assert_eq!(top_border_color(&mut ta), focused, "焦点态: 边框转 accent");

        ta.event(&Event::FocusOut, area(), &mut msgs);
        assert_eq!(top_border_color(&mut ta), resting, "失焦后: 复原常规边框色");
    }

    #[test]
    fn height_builder_sets_min_height() {
        let mut texts = TextBatch::new();
        let mut t = TextArea::new().width(400.0).height(160.0);
        let size = t.layout(Constraints::loose(Size::new(800.0, 800.0)), &mut texts);
        assert!(
            (size.height - 160.0).abs() < 0.01,
            "空内容时应保持显式高度 160, 实际 {}",
            size.height
        );
    }

    #[test]
    fn height_builder_does_not_cap_content_growth() {
        let mut texts = TextBatch::new();
        let long = "行\n".repeat(30);
        let mut t = TextArea::new().width(400.0).height(160.0).text(long);
        let size = t.layout(Constraints::loose(Size::new(800.0, 800.0)), &mut texts);
        assert!(
            size.height > 160.0,
            "内容超高时应随内容增长以支持滚动, 实际 {}",
            size.height
        );
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
    fn enter_after_first_line_advances_caret_to_second_line() {
        // 回归测试:第一行输入字符后按 Enter,
        // 光标必须落在第二行首列(而非 fallback 回第一行行首)。
        // 根因:`break_lines` 未为末尾 '\n' 产生占位空行。
        let mut t = TextArea::new();
        let mut texts = crate::TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        t.insert("ab");
        assert_eq!(t.cursor(), 2);

        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Enter),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            area(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "ab\n");
        assert_eq!(t.cursor(), 3);

        // 下一帧 layout 才会重建行;测试中显式触发。
        t.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

        // 光标应在新行(索引 1)而非第一行(索引 0)。
        // paint() 通过同一个 cursor_line() 推 caret_y,
        // 因此本断言同时覆盖布局与渲染两个面。
        assert_eq!(t.cursor_line_index(), 1);
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
                alt: false,
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
        for i in 0..=crate::widget::form::text_editor::MAX_UNDO {
            t.insert(&i.to_string());
        }
        for _ in 0..crate::widget::form::text_editor::MAX_UNDO {
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
