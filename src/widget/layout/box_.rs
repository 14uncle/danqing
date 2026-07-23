//! @author 十四叔
//! @date 2026/07/17

//! Box 组件: 带背景色与圆角的矩形块, 可含一个子组件。
//!
//! 默认不可交互 (背景块语义);[`Box::hoverable`] 可开启
//! hover 变亮、pressed 变暗的交互效果。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Color, Constraints, Rect, Size, Theme};

/// 背景色块组件。
///
/// 无子组件时默认占满父组件给的最大尺寸; 有子组件时未显式指定的维度
/// 随子组件内容收缩 (卡片语义); 也可指定显式宽高强制固定尺寸。
/// 颜色绑定闭包: 每帧从类型擦除的应用状态产出背景色 (与 `Button::bind_color` 同构)。
type ColorBinding = std::boxed::Box<dyn Fn(&dyn std::any::Any) -> Color>;

pub struct Box {
    color: Color,
    color_binding: Option<ColorBinding>,
    radius: f32,
    width: Option<f32>,
    height: Option<f32>,
    child: Option<Node>,
    hoverable: bool,
    hovered: bool,
    pressed: bool,
    /// 边框颜色,`None` 表示不绘制边框。
    border_color: Option<Color>,
    /// 边框粗细。
    border_width: f32,
}

impl Box {
    /// 创建背景色块 (直角, 不可交互; 无子组件时占满父约束)。
    pub fn new(color: Color) -> Self {
        Self {
            color,
            color_binding: None,
            radius: 0.0,
            width: None,
            height: None,
            child: None,
            hoverable: false,
            hovered: false,
            pressed: false,
            border_color: None,
            border_width: 1.0,
        }
    }

    /// 使用主题默认值创建背景色块 (表面浮层色 + 中等圆角 + 细边框)。
    pub fn themed(theme: &impl Theme) -> Self {
        Self::new(theme.surface())
            .radius(theme.radius_md())
            .border_color(theme.border())
    }

    /// 设置边框颜色。
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// 绑定背景色: 每帧从应用状态读取 (场景色调流动等);设置后覆盖静态值。
    pub fn bind_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.color_binding = Some(std::boxed::Box::new(move |state: &dyn std::any::Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Box 颜色绑定的状态类型不匹配");
            f(state)
        }));
        self
    }

    /// 设置边框粗细。
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// 设置圆角半径 (逻辑像素)。
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// 设置显式宽高 (未设的维度仍按父约束)。
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// 仅设置显式宽度。
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// 仅设置显式高度。
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// 设置子组件 (占满 Box 内容区)。
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(std::boxed::Box::new(child));
        self
    }

    /// 开关 hover/pressed 交互效果 (默认关)。
    pub fn hoverable(mut self, hoverable: bool) -> Self {
        self.hoverable = hoverable;
        self
    }

    /// 当前是否 hover。
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// 当前是否按下。
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// 按交互状态调制后的实际绘制颜色。
    fn effective_color(&self) -> Color {
        if !self.hoverable {
            return self.color;
        }
        let scale = if self.pressed {
            0.7
        } else if self.hovered {
            1.25
        } else {
            1.0
        };
        Color::rgba(
            (self.color.r * scale).min(1.0),
            (self.color.g * scale).min(1.0),
            (self.color.b * scale).min(1.0),
            self.color.a,
        )
    }
}

impl Widget for Box {
    fn sync(&mut self, state: &dyn std::any::Any) {
        if let Some(binding) = &self.color_binding {
            self.color = binding(state);
        }
        if let Some(child) = &mut self.child {
            child.sync(state);
        }
    }

    fn animate(&mut self, ctx: &crate::app::AnimationCtx) {
        if let Some(child) = &mut self.child {
            child.animate(ctx);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        // 两个维度均显式指定: 尺寸固定, 子组件按 tight 占满内容区。
        if let (Some(width), Some(height)) = (self.width, self.height) {
            let size = constraints.constrain(Size::new(width, height));
            if let Some(child) = &mut self.child {
                child.layout(Constraints::tight(size), texts);
            }
            return size;
        }
        match &mut self.child {
            // 有子组件: 未显式指定的维度随子组件内容收缩,
            // 避免占满父约束上限、把 Flow 中的后续兄弟挤出屏幕。
            Some(child) => {
                let child_size = child.layout(constraints, texts);
                constraints.constrain(Size::new(
                    self.width.unwrap_or(child_size.width),
                    self.height.unwrap_or(child_size.height),
                ))
            }
            // 无子组件: 保持背景块语义, 未指定的维度占满父约束上限。
            None => constraints.constrain(Size::new(
                self.width.unwrap_or(constraints.max_width),
                self.height.unwrap_or(constraints.max_height),
            )),
        }
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        rects.push_rect(area, self.effective_color(), self.radius);
        if let Some(child) = &self.child {
            child.paint(area, rects, texts);
        }
        // 边框骑缝绘制(内外各半),最后画避免被不透明子组件盖住内半。
        if let Some(border) = self.border_color {
            rects.push_rounded_border(area, border, self.radius, self.border_width);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        // 先分发给子组件: 移动类全发, 其他类命中才发
        if let Some(child) = &mut self.child {
            let forward = match event {
                Event::CursorMoved(_) | Event::CursorLeft => true,
                e => e.position().is_some_and(|p| area.contains(p)),
            };
            if forward && child.event(event, area, msgs) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        if !self.hoverable {
            return EventResult::Ignored;
        }
        match event {
            Event::CursorMoved(p) => {
                self.hovered = area.contains(*p);
                if self.hovered {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::CursorLeft => {
                self.hovered = false;
                self.pressed = false;
                EventResult::Ignored
            }
            Event::MouseInput {
                pressed, position, ..
            } => {
                if *pressed {
                    if area.contains(*position) {
                        self.pressed = true;
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    let was_pressed = self.pressed;
                    self.pressed = false;
                    if was_pressed && area.contains(*position) {
                        EventResult::Consumed
                    } else {
                        EventResult::Ignored
                    }
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn children(&self) -> &[Node] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn children_mut(&mut self) -> &mut [Node] {
        match &mut self.child {
            Some(child) => std::slice::from_mut(child),
            None => &mut [],
        }
    }
}

#[cfg(test)]
impl Box {
    /// 当前背景色 (测试用)。
    pub(crate) fn color(&self) -> Color {
        self.color
    }

    /// 当前圆角半径 (测试用)。
    pub(crate) fn radius_value(&self) -> f32 {
        self.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LightTheme;

    #[test]
    fn themed_box_uses_theme_surface_and_medium_radius() {
        let theme = LightTheme;
        let box_ = Box::themed(&theme);
        assert_eq!(box_.color(), theme.surface());
        assert_eq!(box_.radius_value(), theme.radius_md());
    }

    #[test]
    fn new_box_preserves_explicit_color_and_zero_radius() {
        let color = Color::from_srgb8(255, 0, 0);
        let box_ = Box::new(color);
        assert_eq!(box_.color(), color);
        assert_eq!(box_.radius_value(), 0.0);
    }

    #[test]
    fn box_with_child_wraps_unspecified_dimensions() {
        // 有子组件时, 未显式指定的维度随内容收缩,
        // 而不是占满父约束上限 (showcase 卡片回归)。
        let mut texts = TextBatch::new();
        let mut box_ = Box::new(Color::WHITE).child(Box::new(Color::BLACK).size(120.0, 80.0));
        let size = box_.layout(Constraints::loose(Size::new(1280.0, 800.0)), &mut texts);
        assert_eq!(size, Size::new(120.0, 80.0));
    }

    #[test]
    fn box_without_child_fills_parent_max() {
        // 无子组件时保持背景块语义: 占满父约束上限。
        let mut texts = TextBatch::new();
        let mut box_ = Box::new(Color::WHITE);
        let size = box_.layout(Constraints::loose(Size::new(1280.0, 800.0)), &mut texts);
        assert_eq!(size, Size::new(1280.0, 800.0));
    }

    #[test]
    fn box_with_explicit_size_keeps_size_and_child_fills() {
        // 显式尺寸不受子组件影响, 子组件仍按 tight 占满。
        let mut texts = TextBatch::new();
        let mut box_ = Box::new(Color::WHITE)
            .size(400.0, 160.0)
            .child(Box::new(Color::BLACK));
        let size = box_.layout(Constraints::loose(Size::new(1280.0, 800.0)), &mut texts);
        assert_eq!(size, Size::new(400.0, 160.0));
    }
}
