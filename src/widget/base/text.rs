//! @author 十四叔
//! @date 2026/07/17

//! Text 组件：单行文本，内容可静态或绑定到应用状态。

use std::any::Any;

use crate::render::{RectBatch, TextBatch};
use crate::widget::Widget;
use crate::{Color, Constraints, LightTheme, Rect, Size, Theme};

/// 文本绑定闭包：从类型擦除的应用状态产出显示内容。
type TextBinding = Box<dyn Fn(&dyn Any) -> String>;

/// 文本组件。
///
/// 显示一段单行文本，字号与颜色可在构建时指定;
/// 内容可静态 ([`Text::new`]) 或绑定到状态读取闭包 ([`Text::bind`])。
pub struct Text {
    content: String,
    binding: Option<TextBinding>,
    font_size: u16,
    color: Color,
}

impl Text {
    /// 创建静态文本组件，默认字号为浅色主题正文字号、颜色为不透明黑色。
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            binding: None,
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
}

impl Widget for Text {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(binding) = &self.binding {
            self.content = binding(state);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        let width = texts.measure(&self.content, self.font_size);
        let height = texts.line_height(f32::from(self.font_size));
        constraints.constrain(Size::new(width, height))
    }

    fn paint(&self, area: Rect, _rects: &mut RectBatch, texts: &mut TextBatch) {
        let baseline = area.origin.y + texts.ascent(f32::from(self.font_size));
        texts.push_text(
            &self.content,
            area.origin.x,
            baseline,
            self.font_size,
            self.color,
        );
    }
}
