//! @author 十四叔
//! @date 2026/07/21

//! Switcher 集成测试: 焦点链、事件路径与点击命中的可见性语义。
//!
//! 不依赖 winit/wgpu,直接操作组件树与 FocusManager。

use danqing::event::Event;
use danqing::widget::{
    Box as UiBox, Button, Column, FocusManager, Switcher, Text, TextInput, event_at_path, node,
};
use danqing::{Color, Constraints, Point, Rect, Size};

/// 双面板树: 面板 0 一个按钮, 面板 1 一个输入框, 外层 Column 便于验证路径。
fn build_tree(active: usize) -> danqing::widget::Node {
    node(
        Column::new().child(
            Switcher::new()
                .child(Button::new(Text::new("A")).on_click(|| "clicked-a"))
                .child(
                    TextInput::new()
                        .text("hello")
                        .on_change(|s: &str| s.to_string()),
                )
                .active(active),
        ),
    )
}

#[test]
fn focus_chain_contains_only_active_panel() {
    let mut tree = build_tree(0);
    let mut texts = danqing::TextBatch::new();
    tree.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

    let mut focus = FocusManager::new();
    focus.rebuild(&tree);
    // 面板 0 激活: 焦点链只有按钮, 路径经 Switcher 可见切片 (索引恒 0)。
    assert_eq!(focus.current(), Some(&vec![0, 0]));
    focus.next();
    // 只有一个可聚焦组件, Tab 循环回自身, 不进入隐藏面板的 TextInput。
    assert_eq!(focus.current(), Some(&vec![0, 0]));
}

#[test]
fn switching_panels_swaps_focus_chain() {
    let mut tree = build_tree(1);
    let mut texts = danqing::TextBatch::new();
    tree.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

    let mut focus = FocusManager::new();
    focus.rebuild(&tree);
    // 面板 1 激活: 焦点链换成 TextInput。
    assert_eq!(focus.current(), Some(&vec![0, 0]));
    focus.next();
    assert_eq!(focus.current(), Some(&vec![0, 0]));
}

/// 分类切换状态: 驱动 Switcher::bind。
struct Nav {
    active: usize,
}

#[test]
fn focus_in_hidden_panel_is_cleared_after_switch() {
    // 文档化语义: 焦点所在面板被隐藏后, 下一帧 rebuild 清除焦点
    // (新面板同路径不可聚焦时); 切回不自动恢复。
    let mut tree = node(
        Column::new().child(
            Switcher::new()
                .child(
                    TextInput::new()
                        .text("a")
                        .on_change(|s: &str| s.to_string()),
                )
                .child(UiBox::new(Color::BLACK))
                .bind(|s: &Nav| s.active),
        ),
    );
    let mut texts = danqing::TextBatch::new();
    tree.sync(&Nav { active: 0 });
    tree.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

    let mut focus = FocusManager::new();
    focus.rebuild(&tree);
    focus.set_focus(vec![0, 0]);
    assert_eq!(focus.current(), Some(&vec![0, 0]));

    // 切换到面板 1 (不可聚焦): rebuild 发现旧路径失效, 焦点清除。
    tree.sync(&Nav { active: 1 });
    focus.rebuild(&tree);
    assert!(focus.current().is_none());

    // 切回面板 0: 焦点不自动恢复 (首次自动聚焦已消耗)。
    tree.sync(&Nav { active: 0 });
    focus.rebuild(&tree);
    assert!(focus.current().is_none());
}

/// 渲染的矩形实例数: 聚焦环 (虚线边框) 由大量小矩形组成, 计数可区分有/无环。
fn ring_count(
    tree: &mut danqing::widget::Node,
    area: Rect,
    texts: &mut danqing::TextBatch,
) -> usize {
    let mut rects = danqing::RectBatch::new();
    tree.paint(area, &mut rects, texts);
    rects.len()
}

#[test]
fn switching_panel_clears_previous_focus_ring() {
    // 回归: 面板切换后旧面板按钮的焦点环必须清除。FocusOut 经 Switcher 的
    // 可见切片无法送达隐藏面板, 故 active 变化时 Switcher 主动 reset_focus;
    // 否则重开面板会残留上一会话的焦点环 (渲染矩形数不回归基线)。
    let mut tree = node(
        Column::new().child(
            Switcher::new()
                .child(Button::new(Text::new("A")).on_click(|| "a"))
                .child(Button::new(Text::new("B")).on_click(|| "b"))
                .bind(|s: &Nav| s.active),
        ),
    );
    let mut texts = danqing::TextBatch::new();
    let area = Rect::new(Point::ZERO, Size::new(300.0, 80.0));
    tree.sync(&Nav { active: 0 });
    tree.layout(Constraints::tight(area.size), &mut texts);

    // 面板 0 激活、未聚焦: 基线 = 按钮 A 的填充矩形数。
    let baseline = ring_count(&mut tree, area, &mut texts);

    // 聚焦按钮 A (模拟 handler 派发 FocusIn): 出现焦点环 → 矩形数增加。
    let mut msgs = Vec::new();
    event_at_path(&mut tree, &[0, 0], &Event::FocusIn, area, &mut msgs);
    let focused = ring_count(&mut tree, area, &mut texts);
    assert!(focused > baseline, "聚焦后应多出焦点环矩形");

    // 切到面板 1 再切回面板 0: 按钮 A 的焦点环必须已被清除。
    tree.sync(&Nav { active: 1 });
    tree.layout(Constraints::tight(area.size), &mut texts);
    tree.sync(&Nav { active: 0 });
    tree.layout(Constraints::tight(area.size), &mut texts);
    let after = ring_count(&mut tree, area, &mut texts);
    assert_eq!(after, baseline, "面板切走再切回后不应残留焦点环");
}

#[test]
fn event_at_path_reaches_active_child_through_visible_slice() {
    let mut tree = build_tree(0);
    let mut texts = danqing::TextBatch::new();
    tree.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

    // Switcher 层路径索引恒为 0 (可见切片), 事件到达 active 面板的按钮。
    let mut msgs = Vec::new();
    let result = event_at_path(
        &mut tree,
        &[0, 0],
        &Event::Key {
            key: danqing::event::Key::Named(danqing::event::NamedKey::Enter),
            pressed: true,
            shift: false,
            ctrl: false,
        },
        Rect::new(Point::ZERO, Size::new(500.0, 500.0)),
        &mut msgs,
    );
    assert_eq!(result, danqing::widget::EventResult::Consumed);
    assert!(
        msgs.iter().any(|m| m
            .downcast_ref::<&'static str>()
            .is_some_and(|s| *s == "clicked-a")),
        "active 面板按钮应产出点击消息"
    );
}

#[test]
fn click_on_hidden_panel_area_does_not_focus_its_children() {
    let mut tree = build_tree(0);
    let mut texts = danqing::TextBatch::new();
    let area = Rect::new(Point::ZERO, Size::new(500.0, 500.0));
    tree.layout(Constraints::loose(area.size), &mut texts);
    tree.paint(area, &mut danqing::RectBatch::default(), &mut texts);

    let mut focus = FocusManager::new();
    focus.rebuild(&tree);

    // 面板 0 激活时, TextInput (面板 1) 不可见: 点击面板区域任何位置
    // 都只能命中按钮或落空, 不会聚焦隐藏面板的 TextInput。
    focus.set_by_click(&tree, Point::new(400.0, 400.0));
    assert_ne!(focus.current(), Some(&vec![0, 1]));
}
