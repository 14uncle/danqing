//! @author 十四叔
//! @date 2026/07/17

//! TextInput 组件：单行可编辑文本。
//!
//! 支持光标、选区、键盘编辑、IME preedit 显示与 commit 插入。

use std::cell::Cell;

use crate::app::AnimationCtx;
use crate::event::{Event, ImeEvent, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::form::text_editor::TextEditor;
use crate::widget::{EventResult, MsgQueue, Widget};
use crate::{Color, Constraints, Edges, LightTheme, Rect, Size, Theme};

/// 光标闪烁周期 (秒)。
const BLINK_PERIOD: f32 = 0.5;

/// 单行文本输入组件。
pub struct TextInput {
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
    /// 背景圆角半径。
    radius: f32,
    /// 边框颜色。
    border_color: Color,
    /// 获得焦点时的边框颜色。
    focus_border_color: Color,
    /// 边框粗细。
    border_width: f32,
    /// 关闭自绘背景与边框: 由外层容器绘制外壳, 组件只管文本/光标/选区/IME。
    chromeless: bool,
    /// 显式宽度 (未指定则按约束上限)。
    width: Option<f32>,
    /// layout/paint 缓存：自身绝对矩形。
    area: Cell<Rect>,
    /// 每个字符右侧的 x 偏移 (用于鼠标点击定位光标与 IME 区域)。
    char_offsets: Vec<f32>,
    /// 行高 (用于 IME 区域与光标高度)。
    line_height: f32,
    /// 光标可见性 (由动画控制闪烁)。
    caret_visible: bool,
    /// IME 合成文本 (显示在光标处，带下划线)。
    preedit: Option<String>,
    /// 鼠标拖拽选区状态。
    dragging: bool,
    /// 占位文字 (空态显示, 居中, 可经 offset 微调垂直位置)。
    placeholder: Option<String>,
    /// 占位文字颜色。
    placeholder_color: Color,
    /// 占位文字相对正常 baseline 的垂直偏移 (正=下, 负=上)。
    /// 产品层可据此微调占位文字位置, 不影响光标/输入文字。
    placeholder_offset: f32,
}

impl TextInput {
    /// 创建文本输入框，使用默认浅色主题 token。
    pub fn new() -> Self {
        Self::themed(&LightTheme)
    }

    /// 使用指定主题创建文本输入框。
    pub fn themed(theme: &impl Theme) -> Self {
        Self {
            editor: TextEditor::new(),
            focused: false,
            font_size: theme.font_size_body(),
            color: theme.text_primary(),
            background: theme.surface_input(),
            selection_color: theme.selection(),
            caret_color: theme.caret(),
            padding: Edges::symmetric(theme.spacing_md(), theme.spacing_sm()),
            radius: theme.radius_sm(),
            border_color: theme.border(),
            focus_border_color: theme.accent(),
            border_width: 1.0,
            chromeless: false,
            width: None,
            area: Cell::new(Rect::default()),
            char_offsets: Vec::new(),
            line_height: 0.0,
            caret_visible: true,
            preedit: None,
            dragging: false,
            placeholder: None,
            placeholder_color: Color::from_srgb8(160, 160, 160),
            placeholder_offset: 0.0,
        }
    }

    /// 设置文本内容。
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

    /// 设置内边距 (文本起点与外框边缘的间距)。
    ///
    /// chromeless 场景下外层容器自绘占位文字/外壳时,
    /// 用同一份 padding 对齐两侧的文字起点, 避免光标与占位文字错位。
    pub fn padding(mut self, padding: Edges) -> Self {
        self.padding = padding;
        self
    }

    /// 设置占位文字与颜色 (文本为空且无 IME 合成时显示)。
    ///
    /// 占位文字默认与正常文字同 baseline (居中);
    /// 可经 [`Self::placeholder_offset`] 微调垂直位置, 不影响光标/输入文字。
    pub fn placeholder(mut self, text: impl Into<String>, color: Color) -> Self {
        self.placeholder = Some(text.into());
        self.placeholder_color = color;
        self
    }

    /// 占位文字垂直偏移 (逻辑像素, 正=下, 负=上)。
    ///
    /// 相对正常 baseline 偏移, 仅影响占位文字, 不影响光标与输入文字。
    /// 典型用法: 产品层需要占位文字靠下对齐时传正值。
    pub fn placeholder_offset(mut self, offset: f32) -> Self {
        self.placeholder_offset = offset;
        self
    }

    /// 关闭自绘背景与边框: 外壳 (底色/边框/焦点态描边) 交由外层容器绘制,
    /// 组件只负责文本、光标、选区与 IME preedit。
    ///
    /// 焦点态描边由外层经 [`Self::is_focused`] 查询后画在自己的外壳矩形上,
    /// 避免组件在内缩的文字区里再画一圈小边框 (双框)。
    pub fn chromeless(mut self) -> Self {
        self.chromeless = true;
        self
    }

    /// 是否持有键盘焦点 (供外层容器绘制焦点态外壳)。
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// 是否有进行中的 IME 合成 (preedit 未上屏)。
    ///
    /// 合成期间 `value()` 仍为空 —— 外层容器画占位文字时须连同此状态判断,
    /// 否则占位文字与拼音字母重叠, 直到候选词上屏才消失。
    pub fn is_composing(&self) -> bool {
        self.preedit.is_some()
    }

    /// 设置文本变化回调 (每次编辑后触发)。
    pub fn on_change<M: 'static>(mut self, f: impl Fn(&str) -> M + 'static) -> Self {
        self.editor = self.editor.on_change(f);
        self
    }

    /// 当前文本 (不含 preedit)。
    pub fn value(&self) -> &str {
        self.editor.text()
    }

    /// 清空文本 (可撤销)。
    ///
    /// 保留组件实例与焦点状态 —— 需要「清空输入」时不要用新建实例替代:
    /// FocusIn 只派发一次, 新实例永远等不到焦点态 (光标不显示)。
    ///
    /// 不产生 `on_change` 消息: 应用状态侧须自行同步清空 (下帧 bind 也会纠回)。
    pub fn clear(&mut self) {
        self.editor.select_all();
        self.editor.cut_selection();
        self.preedit = None;
    }

    /// 替换占位文字, 保留焦点与输入状态。
    pub fn set_placeholder(&mut self, text: impl Into<String>) {
        self.placeholder = Some(text.into());
    }

    /// 光标位置 (测试用)。
    #[cfg(test)]
    pub(crate) fn cursor(&self) -> usize {
        self.editor.cursor()
    }

    /// 设置光标位置 (测试用)。
    #[cfg(test)]
    pub(crate) fn set_cursor(&mut self, cursor: usize) {
        self.editor.set_cursor(cursor);
    }

    /// 选区锚点 (测试用)。
    #[cfg(test)]
    pub(crate) fn anchor(&self) -> usize {
        self.editor.anchor()
    }

    /// 设置选区锚点 (测试用)。
    #[cfg(test)]
    pub(crate) fn set_anchor(&mut self, anchor: usize) {
        self.editor.set_anchor(anchor);
    }

    /// 当前背景色 (测试用)。
    #[cfg(test)]
    pub(crate) fn background_color(&self) -> Color {
        self.background
    }

    /// 当前文本颜色 (测试用)。
    #[cfg(test)]
    pub(crate) fn text_color_value(&self) -> Color {
        self.color
    }

    /// 当前圆角半径 (测试用)。
    #[cfg(test)]
    pub(crate) fn radius_value(&self) -> f32 {
        self.radius
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

    /// 删除当前选区 (若存在，测试用)。
    #[cfg(test)]
    fn delete_selection(&mut self) {
        self.editor.delete_selection();
    }

    /// 撤销上一次编辑。
    fn undo(&mut self) -> bool {
        self.editor.undo()
    }

    /// 重做上一次撤销。
    fn redo(&mut self) -> bool {
        self.editor.redo()
    }

    /// 选区范围 (起点，终点),保证 start <= end。
    fn selection_range(&self) -> (usize, usize) {
        self.editor.selection_range()
    }

    /// 测量到给定字符索引的文本宽度。
    fn measure_to(&self, _texts: &mut TextBatch, char_idx: usize) -> f32 {
        if char_idx == 0 {
            return 0.0;
        }
        self.char_offsets.get(char_idx - 1).copied().unwrap_or(0.0)
    }

    /// 将本地 x 坐标 (相对于文本起点) 转换为字符索引。
    fn hit_to_index(&self, local_x: f32) -> usize {
        if self.char_offsets.is_empty() {
            return 0;
        }
        self.char_offsets
            .partition_point(|offset| *offset <= local_x)
            .min(self.char_offsets.len())
    }

    fn move_cursor(&mut self, delta: isize, extend_selection: bool) {
        self.editor.move_cursor(delta, extend_selection);
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
        let content_width = texts.measure(self.editor.text(), self.font_size);
        let line_height = texts.line_height(f32::from(self.font_size));
        let height = line_height + self.padding.vertical();
        let width = self
            .width
            .unwrap_or(constraints.max_width)
            .max(content_width + self.padding.horizontal());
        let size = constraints.constrain(Size::new(width, height));
        self.area.set(Rect::new(crate::Point::ZERO, size));
        self.line_height = line_height;

        // 缓存每个字符右侧的 x 偏移，用于鼠标点击定位光标。
        self.char_offsets.clear();
        let mut x = 0.0f32;
        for ch in self.editor.text().chars() {
            x += texts.measure(&ch.to_string(), self.font_size);
            self.char_offsets.push(x);
        }
        size
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        // 缓存绝对矩形，供 IME 区域与后续事件使用。
        self.area.set(area);

        // 背景与边框共用同一份像素对齐几何：轮廓精确重合 (贴合),
        // 且 1px 描边落在完整像素行上满强度渲染 (底边发虚的根因对策)。
        // chromeless 模式下外壳由外层容器绘制, 这里跳过。
        if !self.chromeless {
            let surface = area.snap_to_pixels();
            rects.push_rect(surface, self.background, self.radius);

            // 边框：聚焦时使用 accent，否则使用默认边框色。
            let border_color = if self.focused {
                self.focus_border_color
            } else {
                self.border_color
            };
            rects.push_rounded_border(surface, border_color, self.radius, self.border_width);
        }

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
        texts.push_text(
            self.editor.text(),
            text_x,
            baseline,
            self.font_size,
            self.color,
        );

        // 占位文字: 空态 + 无 IME 合成时显示, 可经 offset 微调垂直位置
        if self.editor.text().is_empty() && self.preedit.is_none() {
            if let Some(placeholder) = &self.placeholder {
                texts.push_text(
                    placeholder,
                    text_x,
                    baseline + self.placeholder_offset,
                    self.font_size,
                    self.placeholder_color,
                );
            }
        }

        // preedit 文本与下划线
        if let Some(preedit) = &self.preedit {
            let pre_x = text_x + self.measure_to(texts, self.editor.cursor());
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
            let caret_x = text_x + self.measure_to(texts, self.editor.cursor());
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
                    self.move_cursor(-1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::ArrowRight) => {
                    self.move_cursor(1, *shift);
                    EventResult::Consumed
                }
                Key::Named(NamedKey::Home) => {
                    self.editor.set_cursor(0);
                    if !shift {
                        self.editor.set_anchor(0);
                    }
                    EventResult::Consumed
                }
                Key::Named(NamedKey::End) => {
                    let end = self.editor.text().chars().count();
                    self.editor.set_cursor(end);
                    if !shift {
                        self.editor.set_anchor(end);
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
                    self.editor.cut_selection();
                    changed = true;
                }
                EventResult::Consumed
            }
            Event::Paste => EventResult::Consumed,
            Event::CursorMoved(p) => {
                if self.dragging {
                    let text_x = area.origin.x + self.padding.left;
                    let local_x = p.x - text_x;
                    self.editor.set_cursor(self.hit_to_index(local_x));
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
                self.editor.set_cursor(self.hit_to_index(local_x));
                self.editor.set_anchor(self.editor.cursor());
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
        let cursor = self.editor.cursor();
        let cursor_x = if cursor == 0 {
            0.0
        } else {
            self.char_offsets.get(cursor - 1).copied().unwrap_or(0.0)
        };
        let x = area.origin.x + self.padding.left + cursor_x;
        let y = area.origin.y + self.padding.top;
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

    #[test]
    fn text_input_uses_theme_defaults() {
        let input = TextInput::new();
        assert_eq!(input.text_color_value(), LightTheme.text_primary());
        assert_eq!(input.background_color(), LightTheme.surface_input());
        assert_eq!(input.radius_value(), LightTheme.radius_sm());
    }

    #[test]
    fn text_input_themed_uses_provided_theme() {
        let input = TextInput::themed(&LightTheme);
        assert_eq!(input.text_color_value(), LightTheme.text_primary());
        assert_eq!(input.background_color(), LightTheme.surface_input());
    }

    #[test]
    fn text_input_custom_overrides_theme() {
        let custom_bg = Color::from_srgb8(255, 0, 0);
        let input = TextInput::new().background(custom_bg).radius(8.0);
        assert_eq!(input.background_color(), custom_bg);
        assert_eq!(input.radius_value(), 8.0);
    }

    fn input() -> TextInput {
        TextInput::new().text("Hello")
    }

    #[test]
    fn chromeless_skips_background_and_border() {
        let area = Rect::from_xywh(0.0, 0.0, 200.0, 36.0);
        let mut texts = TextBatch::new();

        let normal = TextInput::new();
        let mut rects = RectBatch::new();
        normal.paint(area, &mut rects, &mut texts);
        assert!(!rects.is_empty(), "默认模式应自绘背景 + 边框");

        let chromeless = TextInput::new().chromeless();
        let mut rects = RectBatch::new();
        chromeless.paint(area, &mut rects, &mut texts);
        assert!(rects.is_empty(), "chromeless 不应自绘外壳 (背景/边框)");
    }

    #[test]
    fn composing_state_tracks_preedit_lifecycle() {
        let mut t = TextInput::new();
        let mut msgs = Vec::new();
        assert!(!t.is_composing());
        t.event(
            &Event::Ime(ImeEvent::Preedit {
                value: "ni".into(),
                cursor: None,
            }),
            Rect::default(),
            &mut msgs,
        );
        assert!(t.is_composing(), "preedit 期间应处于合成态");
        t.event(
            &Event::Ime(ImeEvent::Commit {
                value: "你".into()
            }),
            Rect::default(),
            &mut msgs,
        );
        assert!(!t.is_composing(), "上屏后合成态应结束");
        assert_eq!(t.value(), "你");
    }

    #[test]
    fn clear_empties_text_and_resets_cursor() {
        let mut t = TextInput::new().text("要清空的文本");
        t.clear();
        assert_eq!(t.value(), "");
        assert_eq!(t.cursor(), 0);
    }

    #[test]
    fn insert_moves_cursor() {
        let mut t = input();
        t.insert(" world");
        assert_eq!(t.value(), "Hello world");
        assert_eq!(t.cursor(), 11);
    }

    #[test]
    fn space_inserts_space() {
        let mut t = input();
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Space),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello ");
        assert_eq!(t.cursor(), 6);
    }

    #[test]
    fn undo_restores_deleted_text() {
        let mut t = input();
        // 删除 'o'
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Backspace),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hell");

        // Ctrl+Z 撤销
        t.event(
            &Event::Key {
                key: Key::Character("z".to_string()),
                pressed: true,
                shift: false,
                ctrl: true,
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello");
        assert_eq!(t.cursor(), 5);
    }

    #[test]
    fn undo_redo_cycle() {
        let mut t = input();
        t.event(
            &Event::Key {
                key: Key::Character("z".to_string()),
                pressed: true,
                shift: false,
                ctrl: true,
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello"); // 空撤销栈，无变化

        t.insert("!");
        assert_eq!(t.value(), "Hello!");

        t.undo();
        assert_eq!(t.value(), "Hello");

        t.redo();
        assert_eq!(t.value(), "Hello!");
    }

    #[test]
    fn new_edit_clears_redo_stack() {
        let mut t = input();
        t.insert("!");
        t.undo();
        assert_eq!(t.value(), "Hello");

        t.insert("?");
        t.redo(); // 重做栈已被清空
        assert_eq!(t.value(), "Hello?");
    }

    #[test]
    fn redo_via_keyboard_shortcuts() {
        let mut t = input();
        t.insert("!");
        t.undo();
        assert_eq!(t.value(), "Hello");

        // Ctrl+Shift+Z 重做
        t.event(
            &Event::Key {
                key: Key::Character("z".to_string()),
                pressed: true,
                shift: true,
                ctrl: true,
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello!");

        t.undo();
        assert_eq!(t.value(), "Hello");

        // Ctrl+Y 重做
        t.event(
            &Event::Key {
                key: Key::Character("y".to_string()),
                pressed: true,
                shift: false,
                ctrl: true,
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hello!");
    }

    #[test]
    fn undo_stack_depth_limit() {
        let mut t = TextInput::new();
        for i in 0..=crate::widget::form::text_editor::MAX_UNDO {
            t.insert(&i.to_string());
        }
        // 只能撤销最后 MAX_UNDO 次编辑
        for _ in 0..crate::widget::form::text_editor::MAX_UNDO {
            t.undo();
        }
        // 再撤销一次应无变化 (栈已空)
        let after_undos = t.value().to_string();
        t.undo();
        assert_eq!(t.value(), after_undos);
    }

    #[test]
    fn undo_restores_cursor_and_anchor() {
        let mut t = input();
        t.set_cursor(1);
        t.set_anchor(4); // 选中 "ell"
        t.insert("?"); // 替换为 "H?o", 光标在 2
        assert_eq!(t.value(), "H?o");
        assert_eq!(t.cursor(), 2);

        t.undo();
        assert_eq!(t.value(), "Hello");
        assert_eq!(t.cursor(), 1);
        assert_eq!(t.anchor(), 4);
    }

    #[test]
    fn backspace_deletes_previous_char() {
        let mut t = input();
        // cursor 在末尾，Backspace 删除 'o'
        assert_eq!(t.cursor(), 5);
        t.event(
            &Event::Key {
                key: Key::Named(NamedKey::Backspace),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.value(), "Hell");
        assert_eq!(t.cursor(), 4);
    }

    #[test]
    fn selection_and_delete() {
        let mut t = input();
        // 选中 "ell"
        t.set_cursor(1);
        t.set_anchor(4);
        t.delete_selection();
        assert_eq!(t.value(), "Ho");
        assert_eq!(t.cursor(), 1);
    }

    #[test]
    fn arrow_with_shift_extends_selection() {
        let mut t = input();
        t.set_cursor(1);
        t.set_anchor(1);
        t.move_cursor(2, true);
        assert_eq!(t.cursor(), 3);
        assert_eq!(t.anchor(), 1);
    }

    #[test]
    fn selected_text_returns_selection() {
        let mut t = input();
        t.set_cursor(1);
        t.set_anchor(4);
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
                alt: false,
            },
            Rect::default(),
            &mut Vec::new(),
        );
        assert_eq!(t.selected_text(), Some("Hello".to_string()));
        assert_eq!(t.cursor(), 5);
        assert_eq!(t.anchor(), 0);
    }

    #[test]
    fn cut_deletes_selection() {
        let mut t = input();
        t.set_cursor(1);
        t.set_anchor(4);
        t.event(&Event::Cut, Rect::default(), &mut Vec::new());
        assert_eq!(t.value(), "Ho");
        assert_eq!(t.cursor(), 1);
        assert_eq!(t.anchor(), 1);
        assert!(t.selected_text().is_none());

        // Cut 应记录撤销快照
        t.undo();
        assert_eq!(t.value(), "Hello");
    }

    #[test]
    fn mouse_click_positions_cursor() {
        let mut t = input();
        let mut texts = TextBatch::new();
        // 触发 layout 以计算 char_offsets
        t.layout(Constraints::loose(Size::new(500.0, 100.0)), &mut texts);

        // 点击文本起点左侧，光标应在 0
        t.event(
            &Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position: crate::Point::new(0.0, 0.0),
            },
            Rect::from_xywh(0.0, 0.0, 500.0, 100.0),
            &mut Vec::new(),
        );
        assert_eq!(t.cursor(), 0);

        // 点击文本末尾右侧，光标应在末尾
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
        assert_eq!(t.cursor(), 5);
    }

    #[test]
    fn mouse_drag_selects_text() {
        let mut t = input();
        let mut texts = TextBatch::new();
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
        // 将光标移到开头，避免光标偏移干扰原点判断。
        t.set_cursor(0);
        t.set_anchor(0);

        let mut texts = TextBatch::new();
        t.layout(Constraints::loose(Size::new(500.0, 100.0)), &mut texts);

        // paint 前 area 为本地原点，IME 区域应位于 (padding.left, padding.top)。
        let local = t.ime_area().unwrap();
        assert!((local.origin.x - t.padding.left).abs() < f32::EPSILON);
        assert!((local.origin.y - t.padding.top).abs() < f32::EPSILON);

        // paint 后缓存绝对矩形，IME 区域应跟随光标平移。
        let abs = Rect::from_xywh(20.0, 30.0, 500.0, 100.0);
        let mut rects = RectBatch::new();
        t.paint(abs, &mut rects, &mut texts);

        let area = t.ime_area().unwrap();
        let expected_x = abs.origin.x + t.padding.left;
        let expected_y = abs.origin.y + t.padding.top;
        assert!((area.origin.x - expected_x).abs() < f32::EPSILON);
        assert!((area.origin.y - expected_y).abs() < f32::EPSILON);
        assert_eq!(area.size.width, 0.0);
        assert_eq!(area.size.height, t.line_height);
    }

    #[test]
    fn placeholder_shown_when_empty_and_not_composing() {
        let color = Color::from_srgb8(160, 160, 160);
        let t = TextInput::new().placeholder("搜索历史...", color);
        let area = Rect::from_xywh(0.0, 0.0, 200.0, 36.0);
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        t.paint(area, &mut rects, &mut texts);
        // 无文本内容时应有占位文字字形
        assert!(!texts.is_empty(), "空输入应渲染占位文字");
    }

    #[test]
    fn placeholder_hidden_when_text_present() {
        let color = Color::from_srgb8(160, 160, 160);
        let t = TextInput::new()
            .text("Hello")
            .placeholder("搜索历史...", color);
        let area = Rect::from_xywh(0.0, 0.0, 200.0, 36.0);
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        t.paint(area, &mut rects, &mut texts);
        // 有文本内容时不应有占位文字字形 (只有 "Hello" 5 个)
        assert_eq!(texts.len(), 5, "有输入时不应渲染占位文字");
    }

    #[test]
    fn placeholder_hidden_during_composition() {
        let color = Color::from_srgb8(160, 160, 160);
        let mut t = TextInput::new().placeholder("搜索历史...", color);
        // 触发 IME 合成
        let mut msgs = Vec::new();
        t.event(
            &Event::Ime(ImeEvent::Preedit {
                value: "ni".into(),
                cursor: None,
            }),
            Rect::from_xywh(0.0, 0.0, 200.0, 36.0),
            &mut msgs,
        );
        let area = Rect::from_xywh(0.0, 0.0, 200.0, 36.0);
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        t.paint(area, &mut rects, &mut texts);
        // 合成中: 只有 preedit 字母, 没有占位文字
        assert_eq!(texts.len(), 2, "合成中不应渲染占位文字, 只有 preedit 字母");
    }
}
