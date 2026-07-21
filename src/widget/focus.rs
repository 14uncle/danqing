//! @author 十四叔
//! @date 2026/07/17

//! 焦点管理:纯逻辑的焦点链、Tab 遍历与点击聚焦。
//!
//! 本模块不依赖 winit/wgpu;它通过 `Widget::children()` 遍历组件树,
//! 通过 `Widget::focusable()` 判断节点是否可聚焦。

use crate::Point;
use crate::Rect;
use crate::widget::Node;

/// 组件树中的节点路径:从根到目标节点的子索引序列。
pub type FocusPath = Vec<usize>;

/// 焦点管理器。
///
/// 每帧根据当前组件树重建焦点链,维护当前焦点路径。
/// 首次重建时自动聚焦焦点链第一个节点(启动便利);
/// 之后用户主动清焦(点击空白 / Escape)不会被每帧重建抢回。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FocusManager {
    /// 当前焦点路径。
    current: Option<FocusPath>,
    /// 上一帧焦点路径(用于触发 FocusIn/FocusOut)。
    previous: Option<FocusPath>,
    /// 按深度优先顺序收集的可聚焦节点路径。
    chain: Vec<FocusPath>,
    /// 是否已完成首次自动聚焦。
    did_initial_focus: bool,
}

impl FocusManager {
    /// 创建空的焦点管理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 根据组件树重建焦点链,并保留仍有效的当前焦点。
    pub fn rebuild(&mut self, root: &Node) {
        self.chain.clear();
        self.collect(root, &mut Vec::new());

        // 若当前焦点路径在新树中不再有效,则重置为 None
        if let Some(path) = &self.current {
            if !self.is_valid_path(root, path) {
                self.current = None;
            }
        }

        // 仅在首次重建时自动聚焦焦点链第一个节点;
        // 之后用户主动清焦(点击空白 / Escape)不再抢回。
        if !self.did_initial_focus && self.current.is_none() && !self.chain.is_empty() {
            self.current = Some(self.chain[0].clone());
            self.did_initial_focus = true;
        }
    }

    /// 当前焦点路径。
    pub fn current(&self) -> Option<&FocusPath> {
        self.current.as_ref()
    }

    /// 上一帧焦点路径(用于检测焦点变化)。
    pub fn previous(&self) -> Option<&FocusPath> {
        self.previous.as_ref()
    }

    /// 确认当前焦点变化已处理,将 previous 同步为 current。
    ///
    /// 由窗口层在发送完 FocusIn/FocusOut 后调用,防止同一变化被重复分发。
    pub fn acknowledge(&mut self) {
        self.previous = self.current.clone();
    }

    /// 当前焦点是否刚变化(用于在 window.rs 发送 FocusIn/FocusOut)。
    pub fn changed(&self) -> bool {
        self.current != self.previous
    }

    /// 切换到下一个可聚焦节点(Tab)。
    pub fn next(&mut self) {
        if self.chain.is_empty() {
            self.current = None;
            return;
        }
        let idx = self
            .current_index()
            .map(|i| (i + 1) % self.chain.len())
            .unwrap_or(0);
        self.previous = self.current.clone();
        self.current = Some(self.chain[idx].clone());
    }

    /// 切换到上一个可聚焦节点(Shift+Tab)。
    pub fn prev(&mut self) {
        if self.chain.is_empty() {
            self.current = None;
            return;
        }
        let n = self.chain.len();
        let idx = self
            .current_index()
            .map(|i| (i + n - 1) % n)
            .unwrap_or(n - 1);
        self.previous = self.current.clone();
        self.current = Some(self.chain[idx].clone());
    }

    /// 设置焦点为点击位置最上层的可聚焦节点(后绘制者优先)。
    ///
    /// 点击未命中任何可聚焦节点时清除焦点(点击空白 = 取消聚焦),
    /// 之后键盘事件由窗口层回退到应用层处理。
    pub fn set_by_click(&mut self, root: &Node, pos: Point) {
        self.previous = self.current.clone();
        self.current = hit_focusable(root, pos);
    }

    /// 显式设置焦点路径。
    pub fn set_focus(&mut self, path: FocusPath) {
        self.previous = self.current.clone();
        self.current = Some(path);
    }

    /// 清除当前焦点。
    pub fn clear_focus(&mut self) {
        self.previous = self.current.clone();
        self.current = None;
    }

    fn collect(&mut self, node: &Node, prefix: &mut FocusPath) {
        if node.focusable() {
            self.chain.push(prefix.clone());
        }
        for (i, child) in node.children().iter().enumerate() {
            prefix.push(i);
            self.collect(child, prefix);
            prefix.pop();
        }
    }

    fn current_index(&self) -> Option<usize> {
        let current = self.current.as_ref()?;
        self.chain.iter().position(|p| p == current)
    }

    fn is_valid_path(&self, node: &Node, path: &FocusPath) -> bool {
        let mut current = node;
        for &idx in path {
            match current.children().get(idx) {
                Some(child) => current = child,
                None => return false,
            }
        }
        current.focusable()
    }
}

/// 命中测试:返回点击位置最上层(z 序靠后绘制者)的可聚焦节点路径。
fn hit_focusable(root: &Node, pos: Point) -> Option<FocusPath> {
    let mut result = None;
    let mut path = Vec::new();
    visit(root, &mut path, pos, None, &mut result);
    result
}

fn visit(
    node: &Node,
    path: &mut FocusPath,
    pos: Point,
    clip: Option<Rect>,
    result: &mut Option<FocusPath>,
) {
    // 祖先的 hit_area 作为后代可见区域的裁剪。
    let child_clip = match (clip, node.hit_area()) {
        (Some(parent), Some(hit)) => parent.intersect(&hit),
        (Some(parent), None) => Some(parent),
        (None, Some(hit)) => Some(hit),
        (None, None) => None,
    };

    // 后绘制者优先:先遍历子节点,再检查自身
    for (i, child) in node.children().iter().enumerate().rev() {
        path.push(i);
        visit(child, path, pos, child_clip, result);
        path.pop();
    }
    if node.focusable() {
        if let Some(area) = node.hit_area() {
            let area = match child_clip {
                Some(c) => match c.intersect(&area) {
                    Some(intersection) => intersection,
                    None => return,
                },
                None => area,
            };
            if area.contains(pos) {
                *result = Some(path.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Box as UiBox, Button, Column, Text, node};
    use crate::{Color, Constraints, Rect, Size};

    fn dummy_texts() -> crate::TextBatch {
        crate::TextBatch::new()
    }

    #[test]
    fn chain_collects_focusables_in_order() {
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new().child(Button::new(Text::new("A"))).child(
                Column::new()
                    .child(Button::new(Text::new("B")))
                    .child(UiBox::new(Color::BLACK)),
            ),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        // 深度优先: A(0), B(1,0)
        assert_eq!(mgr.chain, vec![vec![0], vec![1, 0]]);
        assert_eq!(mgr.current(), Some(&vec![0]));
    }

    #[test]
    fn tab_cycles_forward() {
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(Button::new(Text::new("B"))),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        mgr.next();
        assert_eq!(mgr.current(), Some(&vec![1]));
        mgr.next();
        assert_eq!(mgr.current(), Some(&vec![0]));
    }

    #[test]
    fn shift_tab_cycles_backward() {
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(Button::new(Text::new("B"))),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        mgr.prev();
        assert_eq!(mgr.current(), Some(&vec![1]));
    }

    #[test]
    fn click_non_focusable_area_clears_focus() {
        // 点击空白 / 不可聚焦区域 = 取消聚焦(标准失焦行为),
        // 之后键盘事件才能回退到应用层(showcase 键盘方块回归)。
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(UiBox::new(Color::BLACK).size(100.0, 100.0)),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        assert_eq!(mgr.current(), Some(&vec![0]), "初始焦点应在 Button");

        // 点击不可聚焦的 Box 区域(Button 下方)。
        mgr.set_by_click(&tree, crate::Point::new(50.0, 500.0));
        assert!(mgr.current().is_none(), "点击空白应清除焦点");
    }

    #[test]
    fn rebuild_does_not_refocus_after_explicit_clear() {
        // 用户主动清焦(点击空白 / Escape)后,每帧 rebuild 不应抢回焦点。
        let mut texts = dummy_texts();
        let mut tree = node(Column::new().child(Button::new(Text::new("A"))));
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        assert!(mgr.current().is_some(), "首次重建应自动聚焦");

        mgr.clear_focus();
        mgr.rebuild(&tree);
        assert!(mgr.current().is_none(), "清焦后 rebuild 不应重新聚焦");
    }

    #[test]
    fn first_rebuild_auto_focuses_first_focusable() {
        // 启动便利保留:首次重建时自动聚焦焦点链第一个节点。
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(Button::new(Text::new("B"))),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        assert_eq!(mgr.current(), Some(&vec![0]));
    }

    #[test]
    fn tab_from_none_focuses_first() {
        // 无焦点时按 Tab 应聚焦第一个节点(Tab 遍历不依赖自动聚焦)。
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(Button::new(Text::new("B"))),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        mgr.clear_focus();
        mgr.next();
        assert_eq!(mgr.current(), Some(&vec![0]));
    }

    #[test]
    fn click_focuses_topmost_focusable() {
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(UiBox::new(Color::BLACK).size(100.0, 100.0)),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        mgr.set_by_click(&tree, crate::Point::new(0.0, 0.0));
        assert_eq!(mgr.current(), Some(&vec![0]));
    }

    #[test]
    fn click_text_input_uses_hit_area_not_ime_cursor() {
        use crate::widget::TextInput;

        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(TextInput::new().text("hello")),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);

        // 必须 paint 一次,让子组件缓存绝对矩形。
        let mut rects = crate::RectBatch::new();
        tree.paint(
            Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0),
            &mut rects,
            &mut texts,
        );

        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        assert_eq!(mgr.current(), Some(&vec![0])); // 初始焦点在 Button

        // 点击 TextInput 内部但远离光标的位置,应聚焦到 TextInput([1])。
        mgr.set_by_click(&tree, crate::Point::new(10.0, 60.0));
        assert_eq!(mgr.current(), Some(&vec![1]));
    }

    #[test]
    fn invalid_path_is_cleared() {
        let mut mgr = FocusManager::new();
        mgr.set_focus(vec![0, 5]);
        let tree = node(UiBox::new(Color::BLACK));
        mgr.rebuild(&tree);
        assert!(mgr.current().is_none());
    }

    #[test]
    fn rebuild_preserves_previous_after_next() {
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(Button::new(Text::new("B"))),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree); // current=A, previous=A
        mgr.next(); // current=B, previous=A
        mgr.rebuild(&tree); // 不应覆盖 previous
        assert_eq!(mgr.previous(), Some(&vec![0]));
        assert_eq!(mgr.current(), Some(&vec![1]));
    }

    #[test]
    fn rebuild_preserves_previous_after_set_focus() {
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")))
                .child(Button::new(Text::new("B"))),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree); // current=A, previous=None
        mgr.set_focus(vec![1]); // current=B, previous=A
        mgr.rebuild(&tree); // 不应覆盖 previous
        assert_eq!(mgr.previous(), Some(&vec![0]));
        assert_eq!(mgr.current(), Some(&vec![1]));
    }

    #[test]
    fn click_outside_scrollable_viewport_does_not_focus_child() {
        use crate::widget::{Scrollable, TextInput};

        let mut texts = dummy_texts();
        let mut tree = node(Scrollable::new(TextInput::new().text("hello").width(200.0)));
        tree.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
        let mut rects = crate::RectBatch::new();
        tree.paint(
            Rect::from_xywh(0.0, 0.0, 100.0, 100.0),
            &mut rects,
            &mut texts,
        );

        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        // Scrollable 自身不可聚焦,但内部的 TextInput 是焦点链唯一成员。
        assert_eq!(mgr.current(), Some(&vec![0]));

        // 点击 TextInput 内部(视口内)应聚焦到 TextInput([0])。
        mgr.set_by_click(&tree, Point::new(50.0, 18.0));
        assert_eq!(mgr.current(), Some(&vec![0]));

        // 点击视口内但在 TextInput 外:清除焦点(点击空白 = 取消聚焦)。
        mgr.set_by_click(&tree, Point::new(50.0, 80.0));
        assert!(mgr.current().is_none());

        // 点击视口外不应聚焦。
        mgr.set_by_click(&tree, Point::new(50.0, 150.0));
        assert!(mgr.current().is_none());
    }
}
