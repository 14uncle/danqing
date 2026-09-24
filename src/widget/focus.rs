//! @author 十四叔
//! @date 2026/07/17

//! 焦点管理：纯逻辑的焦点链、Tab 遍历与点击聚焦。
//!
//! 本模块不依赖 winit/wgpu;它通过 `Widget::children()` 遍历组件树，
//! 通过 `Widget::focusable()` 判断节点是否可聚焦。

use crate::Point;
use crate::Rect;
use crate::event::CursorIcon;
use crate::widget::Node;

/// 组件树中的节点路径：从根到目标节点的子索引序列。
pub type FocusPath = Vec<usize>;

/// 焦点管理器。
///
/// 每帧根据当前组件树重建焦点链，维护当前焦点路径。
/// 首次重建时自动聚焦焦点链第一个节点 (启动便利);
/// 之后用户主动清焦 (点击空白 / Escape) 不会被每帧重建抢回。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FocusManager {
    /// 当前焦点路径。
    current: Option<FocusPath>,
    /// 上一帧焦点路径 (用于触发 FocusIn/FocusOut)。
    previous: Option<FocusPath>,
    /// 按深度优先顺序收集的可聚焦节点路径。
    chain: Vec<FocusPath>,
    /// 与 `chain` 平行的稳定焦点标识 (无标识为 `None`)。
    chain_ids: Vec<Option<&'static str>>,
    /// 是否已完成首次自动聚焦。
    did_initial_focus: bool,
}

impl FocusManager {
    /// 创建空的焦点管理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 根据组件树重建焦点链，并保留仍有效的当前焦点。
    pub fn rebuild(&mut self, root: &Node) {
        self.chain.clear();
        self.chain_ids.clear();
        self.collect(root, &mut Vec::new());

        // 若当前焦点路径在新树中不再有效，则重置为 None
        if let Some(path) = &self.current {
            if !self.is_valid_path(root, path) {
                self.current = None;
            }
        }

        // 仅在首次重建时自动聚焦焦点链第一个节点;
        // 之后用户主动清焦 (点击空白 / Escape) 不再抢回。
        if !self.did_initial_focus && self.current.is_none() && !self.chain.is_empty() {
            self.current = Some(self.chain[0].clone());
            self.did_initial_focus = true;
        }
    }

    /// 当前焦点路径。
    pub fn current(&self) -> Option<&FocusPath> {
        self.current.as_ref()
    }

    /// 上一帧焦点路径 (用于检测焦点变化)。
    pub fn previous(&self) -> Option<&FocusPath> {
        self.previous.as_ref()
    }

    /// 确认当前焦点变化已处理，将 previous 同步为 current。
    ///
    /// 由窗口层在发送完 FocusIn/FocusOut 后调用，防止同一变化被重复分发。
    pub fn acknowledge(&mut self) {
        self.previous = self.current.clone();
    }

    /// 当前焦点是否刚变化 (用于在 window.rs 发送 FocusIn/FocusOut)。
    pub fn changed(&self) -> bool {
        self.current != self.previous
    }

    /// 切换到下一个可聚焦节点 (Tab)。
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

    /// 切换到上一个可聚焦节点 (Shift+Tab)。
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

    /// 设置焦点为点击位置最上层的可聚焦节点 (后绘制者优先)。
    ///
    /// 点击未命中任何可聚焦节点时清除焦点 (点击空白 = 取消聚焦),
    /// 之后键盘事件由窗口层回退到应用层处理。
    pub fn set_by_click(&mut self, root: &Node, pos: Point) {
        self.previous = self.current.clone();
        self.current = hit_focusable(root, pos);
    }

    /// 显式设置焦点路径。
    ///
    /// 设到当前路径 = no-op: 不得把 previous 抹成 current —— rebuild 自动聚焦 /
    /// 点击定焦留下的 pending 跳变要靠 `changed()` 供窗口层派发 FocusIn/Out,
    /// 抹掉会让组件视觉焦点永远起不来 (clipboard 2026-09-24「唤起无焦点态」根因)。
    pub fn set_focus(&mut self, path: FocusPath) {
        if self.current.as_ref() == Some(&path) {
            return;
        }
        self.previous = self.current.clone();
        self.current = Some(path);
    }

    /// 按稳定标识聚焦 (见 [`crate::widget::Widget::focus_id`])。
    ///
    /// 供 `App::focus_request` 使用: 弹层面板关闭后焦点回到打开面板的按钮。
    /// 未找到匹配标识时不变更焦点并返回 `false`; 目标已是当前路径时返回 `true`
    /// 且不动 `previous` (同 [`Self::set_focus`] 的 no-op 语义)。
    pub fn set_focus_by_id(&mut self, id: &str) -> bool {
        let Some(idx) = self.chain_ids.iter().position(|i| *i == Some(id)) else {
            return false;
        };
        self.set_focus(self.chain[idx].clone());
        true
    }

    /// 清除当前焦点。
    pub fn clear_focus(&mut self) {
        self.previous = self.current.clone();
        self.current = None;
    }

    fn collect(&mut self, node: &Node, prefix: &mut FocusPath) {
        if node.focusable() {
            if let Some(id) = node.focus_id() {
                // 重复 id 会让按名聚焦 (set_focus_by_id) 静默取第一个, 应用侧难以察觉。
                debug_assert!(
                    !self
                        .chain_ids
                        .iter()
                        .flatten()
                        .any(|existing| *existing == id),
                    "焦点标识重复: {id} — 按名聚焦会静默取第一个, 请用唯一 id"
                );
            }
            self.chain.push(prefix.clone());
            self.chain_ids.push(node.focus_id());
        }
        let children = node.children();
        // 开态模态屏障: 与命中遍历 ([`visit_hits`]) **同一规则** —— 只深入最上层
        // 那个屏障子树, 屏障外兄弟连同其子树对 Tab 遍历同样不可见。
        //
        // 缺了这条, 设置卡开着时 Tab 会走到卡后被盖住的组件上, 按 Tab 看不到
        // 任何事发生 (三问第 1 问)。这与 `modal_barrier_confines_click_focus_to_its_subtree`
        // 是**同一个 bug 的两条通道** —— 那条走点击定焦, 这条走 Tab 遍历;
        // 规则只该有一套, 故在此复用同一判据而非另写。
        //
        // 注意: 这里**不改遍历顺序** (本函数正序 = Tab 顺序, 与 `visit_hits`
        // 的逆序是两种用途), 只加屏障过滤。
        if let Some(i) = children.iter().rposition(|c| c.modal_barrier()) {
            prefix.push(i);
            self.collect(&children[i], prefix);
            prefix.pop();
            return;
        }
        for (i, child) in children.iter().enumerate() {
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

/// 命中测试：返回点击位置的可聚焦节点路径。
///
/// 兄弟重叠时低索引赢 (逆序遍历但逐次覆写 result, 最终低索引覆盖高索引)。
/// 需要「最深 / z 序最上优先」的查询走 [`visit_hits`] 的 `first_wins`
/// (指针形状即此) —— 两条规则都在同一趟遍历里, 见 [`visit_hits`]。
///
/// **例外: 模态屏障** (开态 `Overlay`, 见 [`crate::widget::Widget::modal_barrier`])。
/// 子级中若有开态屏障, 只深入最上层 (z 序最后 = 最高索引) 的那个,
/// 屏障外兄弟连同其子树对点击定焦不可见 —— 否则浮在底层的模态卡
/// 会被被盖住的可聚焦组件抢焦 (danqing-log 2026-09-14: 设置卡里的
/// 下拉点击后, 焦点落到底层全 rect 的 LogView, 键盘导航漏成滚动)。
/// 与弹层通道 `popup_scope`/`find_barrier` 同一语义 (兄弟倒序、嵌套取更靠内)。
fn hit_focusable(root: &Node, pos: Point) -> Option<FocusPath> {
    let mut result = None;
    let mut path = Vec::new();
    visit_hits(
        root,
        &mut path,
        pos,
        None,
        &mut result,
        &mut |node, path| node.focusable().then(|| path.clone()),
        false,
    );
    result
}

/// 指针形状查询：返回该位置**最深且 z 序最上**的表态组件所说的形状。
///
/// 与 [`hit_focusable`] **共用同一趟遍历** ([`visit_hits`]), 故模态屏障与祖先
/// 裁剪语义同焦点命中 —— 模态卡之后的组件抢不到光标。
/// 无组件表态返回 `None`, 由调用方回退到默认箭头。
pub(crate) fn cursor_at(root: &Node, pos: Point) -> Option<CursorIcon> {
    let mut result = None;
    let mut path = Vec::new();
    visit_hits(
        root,
        &mut path,
        pos,
        None,
        &mut result,
        &mut |node, _| node.cursor_icon(),
        true,
    );
    result
}

/// 命中路径遍历 —— [`hit_focusable`] 与 [`cursor_at`] **共用同一趟**。
///
/// 语义 (两处查询共享, 改一处即改两处):
/// - 深度优先, 兄弟**逆序** (z 序靠上者先访问), 自身在其子级之后;
/// - 开态模态屏障 ([`Widget::modal_barrier`](crate::widget::Widget::modal_barrier))
///   收束进屏障子树, 屏障外兄弟整支跳过 (成因见 [`hit_focusable`] 文档);
/// - 祖先 `hit_area` 作为后代可见区域的**裁剪**, 越界子树跳过; 同一矩形也是
///   节点自身的命中判定。
///
/// `probe` 决定「这个节点算不算候选、候选值是什么」; `first_wins` 决定多个候选
/// 谁胜 —— `true` 取**首次** (最深且 z 序最上, 指针形状用), `false` 取**末次**
/// (祖先覆盖后代, 焦点沿用此语义, **勿改**: 有 `overlapping_siblings_low_index_wins`
/// 钉着)。
fn visit_hits<T>(
    node: &Node,
    path: &mut FocusPath,
    pos: Point,
    clip: Option<Rect>,
    result: &mut Option<T>,
    probe: &mut impl FnMut(&Node, &FocusPath) -> Option<T>,
    first_wins: bool,
) {
    // 祖先的 hit_area 作为后代可见区域的裁剪。
    let child_clip = match (clip, node.hit_area()) {
        (Some(parent), Some(hit)) => parent.intersect(&hit),
        (Some(parent), None) => Some(parent),
        (None, Some(hit)) => Some(hit),
        (None, None) => None,
    };

    let children = node.children();
    if let Some(i) = children.iter().rposition(|c| c.modal_barrier()) {
        // 开态模态屏障: 收束进屏障子树, 屏障外兄弟跳过 (见 hit_focusable 文档)。
        path.push(i);
        visit_hits(
            &children[i],
            path,
            pos,
            child_clip,
            result,
            probe,
            first_wins,
        );
        path.pop();
    } else {
        // 逆序遍历子节点，再检查自身 (兄弟重叠时低索引赢：result 逐次覆写)
        for (i, child) in children.iter().enumerate().rev() {
            path.push(i);
            visit_hits(child, path, pos, child_clip, result, probe, first_wins);
            path.pop();
        }
    }
    if let Some(area) = node.hit_area() {
        let area = match child_clip {
            Some(c) => match c.intersect(&area) {
                Some(intersection) => intersection,
                None => return,
            },
            None => area,
        };
        if area.contains(pos) {
            if let Some(v) = probe(node, path) {
                // first_wins: 已定则不再覆写 (首个候选 = 最深且 z 序最上)。
                if !first_wins || result.is_none() {
                    *result = Some(v);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{Box as UiBox, Button, Column, Stack, Text, Widget, node};
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
        // 深度优先：A(0), B(1,0)
        assert_eq!(mgr.chain, vec![vec![0], vec![1, 0]]);
        assert_eq!(mgr.current(), Some(&vec![0]));
    }

    #[test]
    fn tab_chain_excludes_nodes_behind_an_open_modal_barrier() {
        // 与点击定焦**同一条规则** (见 `modal_barrier_confines_click_focus_to_its_subtree`):
        // 屏障开时屏障外的可聚焦节点**不进焦点链** —— 否则 Tab 会走到卡后被盖住
        // 的组件上, 按 Tab 看不到任何事发生。
        //
        // 注: 断言里出现 `vec![]` (空路径) 是因为根 Stack 的 `focusable()` 为
        // 「有任一可聚焦后代」(`stack.rs:95`), 于是它自己是链首。这是**既有行为**,
        // 本测试**锁的是屏障规则**, 不是认可链首那个不可见节点 (P9 另议)。
        let make = |open: bool| {
            node(
                Stack::new()
                    .child(Button::new(Text::new("底层")))
                    .child(Barrier {
                        open,
                        child: node(Button::new(Text::new("卡内"))),
                    }),
            )
        };
        let mut texts = dummy_texts();

        // 开态: 屏障外的「底层」按钮不得进链, 链上只有屏障内那个按钮。
        let mut open_tree = make(true);
        open_tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut open_mgr = FocusManager::new();
        open_mgr.rebuild(&open_tree);
        assert_eq!(
            open_mgr.chain,
            vec![vec![], vec![1, 0]],
            "开态屏障: 屏障外节点不得进 Tab 链"
        );

        // 关态 = 既有语义不动 (屏障不存在, 它的子树整个不暴露)。
        let mut closed_tree = make(false);
        closed_tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut closed_mgr = FocusManager::new();
        closed_mgr.rebuild(&closed_tree);
        assert_eq!(
            closed_mgr.chain,
            vec![vec![], vec![0]],
            "关态: 只有屏障外那个按钮"
        );
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
        // 点击空白 / 不可聚焦区域 = 取消聚焦 (标准失焦行为),
        // 之后键盘事件才能回退到应用层 (showcase 键盘方块回归)。
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

        // 点击不可聚焦的 Box 区域 (Button 下方)。
        mgr.set_by_click(&tree, crate::Point::new(50.0, 500.0));
        assert!(mgr.current().is_none(), "点击空白应清除焦点");
    }

    #[test]
    fn set_focus_by_id_targets_matching_button() {
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")).id("alpha"))
                .child(Button::new(Text::new("B"))),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        assert_eq!(mgr.current(), Some(&vec![0]), "初始自动聚焦第一个可聚焦");

        assert!(mgr.set_focus_by_id("alpha"), "应能按稳定 id 聚焦");
        assert_eq!(mgr.current(), Some(&vec![0]));

        assert!(
            !mgr.set_focus_by_id("missing"),
            "未知 id 应返回 false 且不变更焦点"
        );
        assert_eq!(mgr.current(), Some(&vec![0]));
    }

    #[test]
    fn rebuild_does_not_refocus_after_explicit_clear() {
        // 用户主动清焦 (点击空白 / Escape) 后，每帧 rebuild 不应抢回焦点。
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
        // 启动便利保留：首次重建时自动聚焦焦点链第一个节点。
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
        // 无焦点时按 Tab 应聚焦第一个节点 (Tab 遍历不依赖自动聚焦)。
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

        // 必须 paint 一次，让子组件缓存绝对矩形。
        let mut rects = crate::RectBatch::new();
        tree.paint(
            Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0),
            &mut rects,
            &mut texts,
        );

        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        assert_eq!(mgr.current(), Some(&vec![0])); // 初始焦点在 Button

        // 点击 TextInput 内部但远离光标的位置，应聚焦到 TextInput([1])。
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
    fn set_focus_to_current_target_preserves_pending_transition() {
        // clipboard 实机回归 (2026-09-24): 热键唤起录入框无焦点态 (边框不亮/光标不闪),
        // 但键盘仍可达 —— 首帧 rebuild 自动聚焦链首留下 None→path 待派发跳变,
        // 同帧 focus_request 的 set_focus(_by_id) 把 previous 抹成 current,
        // 跳变被吞 → FocusIn 永不派发, 组件视觉焦点 (focused 标志) 永远起不来。
        // 规则: 设到当前路径 = no-op, 不得伪造/吞掉 pending 跳变。
        let mut texts = dummy_texts();
        let mut tree = node(Column::new().child(Button::new(Text::new("A")).id("alpha")));
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree); // 自动聚焦 [0], previous=None
        assert_eq!(mgr.current(), Some(&vec![0]));
        assert_eq!(mgr.previous(), None, "rebuild 自动聚焦留下 pending 跳变");

        assert!(mgr.set_focus_by_id("alpha"), "目标 == 当前路径");
        assert_eq!(mgr.previous(), None, "pending 跳变不得被抹掉");
        assert!(mgr.changed(), "None→[0] 的跳变应保留供 FocusIn 派发");

        // set_focus 同语义
        let mut mgr2 = FocusManager::new();
        mgr2.rebuild(&tree);
        mgr2.set_focus(vec![0]);
        assert_eq!(mgr2.previous(), None, "set_focus 同路径同样不得抹 pending");
        assert!(mgr2.changed());
    }

    #[test]
    fn set_focus_by_id_across_paths_still_dispatches_normally() {
        // danqing-log Ctrl+F 语义不受同路径 no-op 影响: 跨路径仍产生正常跳变。
        let mut texts = dummy_texts();
        let mut tree = node(
            Column::new()
                .child(Button::new(Text::new("A")).id("alpha"))
                .child(Button::new(Text::new("B")).id("beta")),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let mut mgr = FocusManager::new();
        mgr.rebuild(&tree);
        mgr.acknowledge(); // 收掉自动聚焦跳变
        assert!(mgr.set_focus_by_id("beta"));
        assert_eq!(mgr.previous(), Some(&vec![0]));
        assert_eq!(mgr.current(), Some(&vec![1]));
        assert!(mgr.changed(), "跨路径切换照常产生跳变");
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
        // Scrollable 自身不可聚焦，但内部的 TextInput 是焦点链唯一成员。
        assert_eq!(mgr.current(), Some(&vec![0]));

        // 点击 TextInput 内部 (视口内) 应聚焦到 TextInput([0])。
        mgr.set_by_click(&tree, Point::new(50.0, 18.0));
        assert_eq!(mgr.current(), Some(&vec![0]));

        // 点击视口内但在 TextInput 外：清除焦点 (点击空白 = 取消聚焦)。
        mgr.set_by_click(&tree, Point::new(50.0, 80.0));
        assert!(mgr.current().is_none());

        // 点击视口外不应聚焦。
        mgr.set_by_click(&tree, Point::new(50.0, 150.0));
        assert!(mgr.current().is_none());
    }

    #[test]
    fn overlapping_siblings_low_index_wins() {
        // 末写者胜反转：visit 逆序遍历但逐次覆写 result,
        // 最终低索引覆盖高索引 = 先绘制者赢 (非注释声称的后绘制者优先)。
        // Overlay 簇C 评审 C2 发现此语义，钉板测试锁定行为。
        let mut texts = dummy_texts();
        let mut tree = node(
            Stack::new()
                .child(
                    UiBox::new(Color::TRANSPARENT)
                        .size(200.0, 200.0)
                        .child(Button::new(Text::new("A"))),
                )
                .child(
                    UiBox::new(Color::TRANSPARENT)
                        .size(200.0, 200.0)
                        .child(Button::new(Text::new("B"))),
                ),
        );
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);

        // 两个按钮完全重叠，点击中心应命中 A（索引 0，低索引赢）。
        let hit = hit_focusable(&tree, Point::new(50.0, 50.0));
        assert_eq!(hit, Some(vec![0, 0]), "重叠兄弟应命中先绘制者 (A)");

        // 反向验证：单独只有 B 时命中 B。
        let mut tree_b_only = node(
            Stack::new().child(
                UiBox::new(Color::TRANSPARENT)
                    .size(200.0, 200.0)
                    .child(Button::new(Text::new("B"))),
            ),
        );
        tree_b_only.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let hit_b = hit_focusable(&tree_b_only, Point::new(50.0, 50.0));
        assert_eq!(hit_b, Some(vec![0, 0]), "单个按钮应命中自身");
    }

    /// 走一趟 layout + paint —— 与真实帧序一致, 组件才有**绝对坐标**的命中矩形
    /// (Button 把矩形缓存在 `layout`(局部坐标) 与 `event` 里, 只有 TextInput 那款
    /// 在 `paint` 里缓存绝对坐标; 本探针取后者, 否则嵌套偏移下验不出「最深者胜」)。
    fn layout_and_paint(tree: &mut Node, texts: &mut crate::TextBatch) {
        let size = tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), texts);
        let mut rects = crate::RectBatch::new();
        tree.paint(Rect::new(Point::ZERO, size), &mut rects, texts);
    }

    /// 光标探针: 布局/绘制透传子树, 自身在命中矩形上表态一个形状 (`None` = 不表态)。
    struct CursorProbe {
        icon: Option<CursorIcon>,
        area: std::cell::Cell<Rect>,
        child: Option<Node>,
    }

    impl CursorProbe {
        fn new(icon: Option<CursorIcon>) -> Self {
            Self {
                icon,
                area: std::cell::Cell::new(Rect::default()),
                child: None,
            }
        }

        fn child(mut self, child: Node) -> Self {
            self.child = Some(child);
            self
        }
    }

    impl Widget for CursorProbe {
        fn layout(&mut self, constraints: Constraints, texts: &mut crate::TextBatch) -> Size {
            match &mut self.child {
                Some(c) => c.layout(constraints, texts),
                None => constraints.constrain(Size::new(100.0, 100.0)),
            }
        }

        fn paint(&self, area: Rect, rects: &mut crate::RectBatch, texts: &mut crate::TextBatch) {
            self.area.set(area);
            if let Some(c) = &self.child {
                c.paint(area, rects, texts);
            }
        }

        fn hit_area(&self) -> Option<Rect> {
            Some(self.area.get())
        }

        fn cursor_icon(&self) -> Option<CursorIcon> {
            self.icon
        }

        fn children(&self) -> &[Node] {
            self.child.as_slice()
        }

        fn children_mut(&mut self) -> &mut [Node] {
            self.child.as_mut_slice()
        }
    }

    #[test]
    fn cursor_takes_the_deepest_opinion_not_the_ancestor() {
        // 外层 Pointer, 内层 Text, 两层命中矩形完全重合 —— 指针落在重合处,
        // **更深**的内层胜。这条与焦点相反 (焦点是祖先覆盖后代, 见
        // `overlapping_siblings_low_index_wins`), 故必须分别钉住。
        let mut texts = dummy_texts();
        let mut tree = node(
            CursorProbe::new(Some(CursorIcon::Pointer))
                .child(node(CursorProbe::new(Some(CursorIcon::Text)))),
        );
        layout_and_paint(&mut tree, &mut texts);
        assert_eq!(
            cursor_at(&tree, Point::new(50.0, 50.0)),
            Some(CursorIcon::Text),
            "深层表态应盖过祖先"
        );
    }

    #[test]
    fn cursor_does_not_reach_past_an_open_modal_barrier() {
        // 对照组: 同一棵树, 屏障开 / 关给出**不同**答案 —— 否则这条测试是空的。
        // 屏障在索引 0; 无屏障时逆序遍历先访问索引 1 (z 序靠上), 答案是 Pointer;
        // 屏障开时收束进索引 0 子树, 答案翻成 Text。
        let mut texts = dummy_texts();
        let build = |open: bool| {
            node(
                Stack::new()
                    .child(Barrier {
                        open,
                        child: node(CursorProbe::new(Some(CursorIcon::Text))),
                    })
                    .child(CursorProbe::new(Some(CursorIcon::Pointer))),
            )
        };
        let mut with = build(true);
        layout_and_paint(&mut with, &mut texts);
        assert_eq!(
            cursor_at(&with, Point::new(50.0, 50.0)),
            Some(CursorIcon::Text),
            "开态屏障: 屏障外兄弟不得抢到光标"
        );

        let mut without = build(false);
        layout_and_paint(&mut without, &mut texts);
        assert_eq!(
            cursor_at(&without, Point::new(50.0, 50.0)),
            Some(CursorIcon::Pointer),
            "关态: 屏障不存在, 逆序遍历先访问上层兄弟"
        );
    }

    #[test]
    fn nodes_without_an_opinion_yield_none() {
        // 不表态 = 默认实现返回 None —— 悬停其上不改变形状, 由调用方回退默认箭头。
        let mut texts = dummy_texts();
        let mut tree = node(CursorProbe::new(None));
        layout_and_paint(&mut tree, &mut texts);
        assert_eq!(cursor_at(&tree, Point::new(50.0, 50.0)), None);
    }

    #[test]
    fn cursor_outside_every_hit_area_is_none() {
        let mut texts = dummy_texts();
        let mut tree = node(CursorProbe::new(Some(CursorIcon::Pointer)));
        layout_and_paint(&mut tree, &mut texts);
        assert_eq!(cursor_at(&tree, Point::new(500.0, 500.0)), None);
    }

    /// 屏障探针: 可开关的模态屏障, layout/paint 透传子树 (焦点命中要求子树
    /// 真实出矩形; mod.rs 弹层测试的屏障探针不出矩形, 锁的是另一条语义)。
    struct Barrier {
        open: bool,
        child: Node,
    }

    impl Widget for Barrier {
        fn layout(&mut self, constraints: Constraints, texts: &mut crate::TextBatch) -> Size {
            self.child.layout(constraints, texts)
        }

        fn paint(&self, area: Rect, rects: &mut crate::RectBatch, texts: &mut crate::TextBatch) {
            self.child.paint(area, rects, texts);
        }

        fn modal_barrier(&self) -> bool {
            self.open
        }

        fn children(&self) -> &[Node] {
            if self.open {
                std::slice::from_ref(&self.child)
            } else {
                &[]
            }
        }

        fn children_mut(&mut self) -> &mut [Node] {
            if self.open {
                std::slice::from_mut(&mut self.child)
            } else {
                &mut []
            }
        }
    }

    #[test]
    fn modal_barrier_confines_click_focus_to_its_subtree() {
        // danqing-log 实机回归 (2026-09-14): 设置卡 (Overlay, 高索引兄弟) 里的
        // 主题下拉点击展开后按 ↑↓, 选中项不动、底下的日志区反而滚动 ——
        // 点击定焦被被盖住的 LogView 抢走: 它 focusable + 全 rect hit_area,
        // 又占更低兄弟索引, 「低索引赢」对开着的模态不成立。
        // 屏障打开时屏障外的子树对点击定焦必须不可见 —— 与弹层通道
        // `popup_scope`/`find_barrier` 同一语义 (兄弟倒序、嵌套取更靠内)。
        let make = |open: bool| {
            node(
                Stack::new()
                    .child(
                        UiBox::new(Color::TRANSPARENT)
                            .size(200.0, 200.0)
                            .child(Button::new(Text::new("底层"))),
                    )
                    .child(Barrier {
                        open,
                        child: node(
                            UiBox::new(Color::TRANSPARENT)
                                .size(200.0, 200.0)
                                .child(Button::new(Text::new("卡内"))),
                        ),
                    }),
            )
        };
        let mut texts = dummy_texts();
        let mut tree = make(true);
        tree.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);

        // 点击两棵子树的重叠处: 屏障开 → 焦点必须落在屏障内 [1,0,0] (卡内按钮),
        // 不得是屏障外低索引的 [0,0] (底层按钮)。
        let hit = hit_focusable(&tree, Point::new(50.0, 50.0));
        assert_eq!(hit, Some(vec![1, 0, 0]), "开态屏障外不得抢焦");

        // 屏障关闭 = 既有语义不动 (低索引赢, 锁于 overlapping_siblings_low_index_wins)。
        let mut closed = make(false);
        closed.layout(Constraints::loose(Size::new(1000.0, 1000.0)), &mut texts);
        let hit_closed = hit_focusable(&closed, Point::new(50.0, 50.0));
        assert_eq!(hit_closed, Some(vec![0, 0]), "关态屏障不改变既有语义");
    }
}
