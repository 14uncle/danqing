// ! @author 十四叔
// ! @date 2026/07/17

// ! 组件: 保留模式 UI 树的节点抽象。
// !
// ! 本模块为纯逻辑: 组件只依赖布局类型与 CPU 收集器,
// ! 不接触任何平台 / 图形 API。
// !
// ! 每帧流程:
// ! 1. [`Widget::sync`] —— 从应用状态同步绑定属性;
// ! 2. [`Widget::layout`] —— 约束向下传、尺寸向上算;
// ! 3. [`Widget::paint`] —— 按缓存的几何收集绘制命令。

mod box_;
mod button;
mod center;
mod column;
mod flow;
mod focus;
mod padding;
mod row;
mod scrollable;
mod text;
mod text_area;
mod text_editor;
mod text_input;

pub use box_::Box;
pub use button::Button;
pub use center::Center;
pub use column::Column;
pub use focus::FocusManager;
pub use padding::Padding;
pub use row::Row;
pub use scrollable::{ScrollAxis, Scrollable};
pub use text::Text;
pub use text_area::TextArea;
pub use text_input::TextInput;

use std::any::Any;

use crate::app::AnimationCtx;
use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::{Constraints, Rect, Size};

/// 事件处理结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// 事件被消费, 停止分发。
    Consumed,
    /// 未消费, 继续向父级冒泡。
    Ignored,
}

/// 应用消息队列: 组件事件 (如按钮点击) 产出的类型擦除消息。
pub type MsgQueue = Vec<std::boxed::Box<dyn Any>>;

/// 组件: 保留模式 UI 树的一个节点。
pub trait Widget {
    /// 状态同步: 从应用状态更新绑定属性 (每帧布局前调用)。
    ///
    /// 默认实现无操作 (静态组件)。
    fn sync(&mut self, _state: &dyn Any) {}

    /// 动画更新: 由框架每帧在 `sync` 之后、`layout` 之前调用。
    ///
    /// 默认实现无操作。
    fn animate(&mut self, _ctx: &AnimationCtx) {}

    /// 布局: 在约束下计算自身尺寸。
    ///
    /// 容器组件在此递归子组件并缓存各自的几何, 供 paint 使用。
    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size;

    /// 绘制: 按 layout 缓存的几何收集绘制命令。
    ///
    /// `area` 为父组件摆放本组件的矩形 (布局结果)。
    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch);

    /// 事件处理 (鼠标事件经命中分发到达; 键盘 /IME 事件经焦点路由到达)。
    ///
    /// `area` 与 paint 收到的矩形一致; 组件可经 `msgs` 产出应用消息
    /// (如按钮点击)。返回 [`EventResult::Consumed`] 表示消费该事件。
    /// 默认忽略所有事件。
    fn event(&mut self, _event: &Event, _area: Rect, _msgs: &mut MsgQueue) -> EventResult {
        EventResult::Ignored
    }

    /// 当前组件是否可接收键盘焦点。
    ///
    /// 默认可聚焦组件 (如 Button/TextInput) 应返回 true。
    fn focusable(&self) -> bool {
        false
    }

    /// 子组件列表 (用于焦点遍历与命中测试)。
    ///
    /// 容器组件应返回其所有子节点; 叶子组件默认返回空。
    fn children(&self) -> &[Node] {
        &[]
    }

    /// 当前选中的文本 (用于 Copy/Cut)。
    ///
    /// 默认可编辑组件 (如 TextInput) 在选区非空时返回文本。
    fn selected_text(&self) -> Option<String> {
        None
    }

    /// 当前组件是否需要 IME 输入法服务。
    ///
    /// 默认可编辑文本组件 (如 TextInput) 返回 true。
    fn wants_ime(&self) -> bool {
        false
    }

    /// IME 候选框应吸附的矩形 (相对于窗口逻辑坐标)。
    ///
    /// 可输入组件返回光标或输入框区域; 无 IME 需求返回 None。
    fn ime_area(&self) -> Option<Rect> {
        None
    }

    /// 鼠标命中测试使用的矩形 (相对于窗口逻辑坐标)。
    ///
    /// 默认可点击/可聚焦组件 (如 Button/TextInput) 返回自身完整区域;
    /// 无命中需求返回 None。应与 `ime_area` 区分,后者可能只覆盖光标。
    fn hit_area(&self) -> Option<Rect> {
        None
    }

    /// 可变子组件列表 (用于事件分发与动画)。
    ///
    /// 容器组件应返回其所有子节点; 叶子组件默认返回空。
    fn children_mut(&mut self) -> &mut [Node] {
        &mut []
    }
}

/// 组件树节点: 盒装的组件对象。
pub type Node = std::boxed::Box<dyn Widget>;

/// 把组件装箱为节点。
pub fn node(widget: impl Widget + 'static) -> Node {
    std::boxed::Box::new(widget)
}

/// 沿路径向子组件分发事件 (用于焦点路由)。
///
/// `path` 为子索引序列; 返回处理结果。
pub fn event_at_path(
    root: &mut Node,
    path: &[usize],
    event: &Event,
    area: Rect,
    msgs: &mut MsgQueue,
) -> EventResult {
    if path.is_empty() {
        return root.event(event, area, msgs);
    }
    let (first, rest) = path.split_first().expect("path 非空");
    if let Some(child) = root.children_mut().get_mut(*first) {
        // 容器未缓存子区域时, 使用父区域作为近似; 焦点路由通常到达叶子组件。
        event_at_path(child, rest, event, area, msgs)
    } else {
        EventResult::Ignored
    }
}

/// 沿路径取组件的选中文本。
pub fn selected_text_at_path(root: &Node, path: &[usize]) -> Option<String> {
    if path.is_empty() {
        return root.selected_text();
    }
    let (first, rest) = path.split_first().expect("path 非空");
    root.children()
        .get(*first)
        .and_then(|child| selected_text_at_path(child, rest))
}

/// 沿路径取组件的 IME 吸附区域。
pub fn ime_area_at_path(root: &Node, path: &[usize]) -> Option<Rect> {
    if path.is_empty() {
        return root.ime_area();
    }
    let (first, rest) = path.split_first().expect("path 非空");
    root.children()
        .get(*first)
        .and_then(|child| ime_area_at_path(child, rest))
}

/// 沿路径判断组件是否需要 IME。
pub fn wants_ime_at_path(root: &Node, path: &[usize]) -> bool {
    if path.is_empty() {
        return root.wants_ime();
    }
    let (first, rest) = path.split_first().expect("path 非空");
    root.children()
        .get(*first)
        .is_some_and(|child| wants_ime_at_path(child, rest))
}

/// 沿路径触发动画更新。
pub fn animate_at_path(root: &mut Node, path: &[usize], ctx: &AnimationCtx) {
    root.animate(ctx);
    if path.is_empty() {
        return;
    }
    let (first, rest) = path.split_first().expect("path 非空");
    if let Some(child) = root.children_mut().get_mut(*first) {
        animate_at_path(child, rest, ctx);
    }
}
