//! @author 十四叔
//! @date 2026/07/17

//! 焦点 + 文本输入集成测试。
//!
//! 不依赖 winit/wgpu,直接操作组件树与 FocusManager。

use danqing::event::{Event, Key};
use danqing::widget::{Box as UiBox, Button, Column, FocusManager, TextInput, node};
use danqing::{Color, Constraints, Rect, Size};

fn build_tree() -> danqing::widget::Node {
    node(
        Column::new()
            .child(Button::new(danqing::widget::Text::new("A")))
            .child(
                TextInput::new()
                    .text("hello")
                    .on_change(|s: &str| s.to_string()),
            ),
    )
}

#[test]
fn tab_focuses_button_then_input() {
    let mut tree = build_tree();
    let mut texts = danqing::TextBatch::new();
    tree.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

    let mut focus = FocusManager::new();
    focus.rebuild(&tree);
    assert_eq!(focus.current(), Some(&vec![0]));

    focus.next();
    assert_eq!(focus.current(), Some(&vec![1]));
}

#[test]
fn typing_into_focused_input() {
    let mut tree = build_tree();
    let mut texts = danqing::TextBatch::new();
    tree.layout(Constraints::loose(Size::new(500.0, 500.0)), &mut texts);

    let mut focus = FocusManager::new();
    focus.rebuild(&tree);
    focus.set_focus(vec![1]);

    let path = focus.current().unwrap().clone();
    let mut msgs = Vec::new();
    danqing::widget::event_at_path(
        &mut tree,
        &path,
        &Event::FocusIn,
        Rect::default(),
        &mut msgs,
    );
    danqing::widget::event_at_path(
        &mut tree,
        &path,
        &Event::Key {
            key: Key::Character("!".to_string()),
            pressed: true,
            shift: false,
            ctrl: false,
        },
        Rect::default(),
        &mut msgs,
    );

    // 消息队列应包含变化消息 "hello!"
    let found = msgs.iter().any(|m| {
        m.downcast_ref::<String>()
            .map(|s| s == "hello!")
            .unwrap_or(false)
    });
    assert!(found, "输入后应产出变化消息");
}

#[test]
fn focus_manager_ignores_non_focusable_nodes() {
    let tree = node(UiBox::new(Color::BLACK));
    let mut focus = FocusManager::new();
    focus.rebuild(&tree);
    assert!(focus.current().is_none());
}
