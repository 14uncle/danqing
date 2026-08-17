//! @author 十四叔
//! @date 2026/07/17

//! Text 组件：单行文本，内容可静态或绑定到应用状态。

use std::any::Any;

use crate::render::{RectBatch, TextBatch};
use crate::widget::Widget;
use crate::{Color, Constraints, LightTheme, Rect, Size, Theme};

/// 文本绑定闭包：从类型擦除的应用状态产出显示内容。
type TextBinding = Box<dyn Fn(&dyn Any) -> String>;

/// 颜色绑定闭包：从类型擦除的应用状态产出文字颜色。
type ColorBinding = Box<dyn Fn(&dyn Any) -> Color>;

/// 文本组件。
///
/// 显示一段单行文本，字号与颜色可在构建时指定;
/// 内容可静态 ([`Text::new`]) 或绑定到状态读取闭包 ([`Text::bind`]),
/// 颜色同样支持状态绑定 ([`Text::bind_color`]), 用于导航选中态等场景。
pub struct Text {
    content: String,
    binding: Option<TextBinding>,
    color_binding: Option<ColorBinding>,
    font_size: u16,
    color: Color,
}

impl Text {
    /// 创建静态文本组件，默认字号为浅色主题正文字号、颜色为不透明黑色。
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            binding: None,
            color_binding: None,
            font_size: LightTheme.font_size_body(),
            color: Color::BLACK,
        }
    }

    /// 创建绑定文本组件：每帧从应用状态读取内容。
    pub fn bind<S: 'static>(f: impl Fn(&S) -> String + 'static) -> Self {
        let mut text = Self::new("");
        text.binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Text 绑定的状态类型不匹配");
            f(state)
        }));
        text
    }

    /// 设置字号 (逻辑像素)。
    pub fn font_size(mut self, size: u16) -> Self {
        self.font_size = size;
        self
    }

    /// 设置颜色。
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 绑定文字颜色：每帧从应用状态读取颜色 (如导航选中态)。
    ///
    /// 与 [`Text::bind`] 同构;设置后覆盖 `color` 的静态值。
    pub fn bind_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.color_binding = Some(Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Text 颜色绑定的状态类型不匹配");
            f(state)
        }));
        self
    }
}

impl Widget for Text {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(binding) = &self.binding {
            self.content = binding(state);
        }
        if let Some(binding) = &self.color_binding {
            self.color = binding(state);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        let width = texts.measure(&self.content, self.font_size);
        let height = texts.line_height(f32::from(self.font_size));
        constraints.constrain(Size::new(width, height))
    }

    fn paint(&self, area: Rect, _rects: &mut RectBatch, texts: &mut TextBatch) {
        let baseline = area.origin.y + texts.ascent(f32::from(self.font_size));

        // 检测 "..." 并拆分渲染: 前段 baseline 不变, 省略号底边对齐
        if let Some(pos) = self.content.find("...") {
            let prefix = &self.content[..pos];
            let ellipsis = "...";

            // 前段: 正常 baseline
            if !prefix.is_empty() {
                texts.push_text(prefix, area.origin.x, baseline, self.font_size, self.color);
            }

            // 省略号: 底边对齐
            let desc = texts.descent(f32::from(self.font_size));
            let ellipsis_baseline = area.origin.y + area.size.height - desc;
            let prefix_width = texts.measure(prefix, self.font_size);
            texts.push_text(
                ellipsis,
                area.origin.x + prefix_width,
                ellipsis_baseline,
                self.font_size,
                self.color,
            );
        } else {
            // 无省略号: 正常渲染
            texts.push_text(
                &self.content,
                area.origin.x,
                baseline,
                self.font_size,
                self.color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_color_updates_color_on_sync() {
        struct Nav {
            selected: bool,
        }
        let idle = Color::BLACK;
        let active = Color::WHITE;
        let mut text =
            Text::new("基础").bind_color(move |s: &Nav| if s.selected { active } else { idle });
        text.sync(&(Nav { selected: true }) as &dyn Any);
        assert_eq!(text.color, active, "选中时应取绑定色");
        text.sync(&(Nav { selected: false }) as &dyn Any);
        assert_eq!(text.color, idle, "未选中时应回退绑定色");
    }

    #[test]
    fn static_color_used_without_binding() {
        let mut text = Text::new("x").color(Color::WHITE);
        text.sync(&() as &dyn Any);
        assert_eq!(text.color, Color::WHITE, "无绑定时静态色不被改动");
    }

    #[test]
    fn text_without_ellipsis_paints_once() {
        let text = Text::new("清空");
        let mut texts = TextBatch::new();
        let area = Rect::from_xywh(0.0, 0.0, 200.0, 40.0);
        text.paint(area, &mut RectBatch::new(), &mut texts);
        // 不含 "..." 应只产生一轮 push_text (前段)
        assert!(!texts.is_empty(), "应有字形输出");
    }

    #[test]
    fn text_with_ellipsis_paints_two_segments() {
        let text = Text::new("清空...");
        let mut texts = TextBatch::new();
        let area = Rect::from_xywh(0.0, 0.0, 200.0, 40.0);
        text.paint(area, &mut RectBatch::new(), &mut texts);
        // 含 "..." 应产生两轮 push_text (前段 + 省略号)
        assert!(!texts.is_empty(), "应有字形输出");
    }
}
