//! @author 十四叔
//! @date 2026/07/17

//! 集成测试:事件命中分发(hover/pressed/嵌套命中顺序)。

use danqing::event::{Event, MouseButton};
use danqing::widget::{Box as UiBox, EventResult, Widget};
use danqing::{Color, Constraints, Point, Rect, Size};

fn area() -> Rect {
    Rect::from_xywh(0.0, 0.0, 100.0, 50.0)
}

fn layout(box_: &mut UiBox) {
    let mut texts = danqing::TextBatch::new();
    box_.layout(Constraints::loose(Size::new(800.0, 600.0)), &mut texts);
}

#[test]
fn hover_tracks_cursor_position() {
    let mut b = UiBox::new(Color::BLACK).size(100.0, 50.0);
    layout(&mut b);
    assert!(!b.is_hovered());

    b.event(
        &Event::CursorMoved(Point::new(50.0, 25.0)),
        area(),
        &mut Vec::new(),
    );
    assert!(b.is_hovered(), "光标进入应 hover");

    b.event(
        &Event::CursorMoved(Point::new(500.0, 500.0)),
        area(),
        &mut Vec::new(),
    );
    assert!(!b.is_hovered(), "光标移出应取消 hover");
}

#[test]
fn press_and_release() {
    let mut b = UiBox::new(Color::BLACK).size(100.0, 50.0);
    layout(&mut b);

    let inside = Point::new(10.0, 10.0);
    let r = b.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: inside,
        },
        area(),
        &mut Vec::new(),
    );
    assert_eq!(r, EventResult::Consumed);
    assert!(b.is_pressed(), "按下应进入 pressed");

    let r = b.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position: inside,
        },
        area(),
        &mut Vec::new(),
    );
    assert_eq!(r, EventResult::Consumed);
    assert!(!b.is_pressed(), "抬起应退出 pressed");
}

#[test]
fn press_outside_ignored() {
    let mut b = UiBox::new(Color::BLACK).size(100.0, 50.0);
    layout(&mut b);
    let r = b.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(200.0, 200.0),
        },
        area(),
        &mut Vec::new(),
    );
    assert_eq!(r, EventResult::Ignored);
    assert!(!b.is_pressed());
}

#[test]
fn nested_child_consumes_before_parent() {
    let mut parent = UiBox::new(Color::BLACK)
        .size(100.0, 100.0)
        .child(UiBox::new(Color::WHITE).size(100.0, 100.0));
    let mut texts = danqing::TextBatch::new();
    parent.layout(Constraints::loose(Size::new(800.0, 600.0)), &mut texts);
    let parent_area = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);

    let r = parent.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(50.0, 50.0),
        },
        parent_area,
        &mut Vec::new(),
    );
    assert_eq!(r, EventResult::Consumed, "子组件应消费命中事件");
    assert!(!parent.is_pressed(), "子组件消费后父组件不得置 pressed");
}

#[test]
fn cursor_left_clears_state() {
    let mut b = UiBox::new(Color::BLACK).size(100.0, 50.0);
    layout(&mut b);
    b.event(
        &Event::CursorMoved(Point::new(10.0, 10.0)),
        area(),
        &mut Vec::new(),
    );
    assert!(b.is_hovered());
    b.event(&Event::CursorLeft, area(), &mut Vec::new());
    assert!(!b.is_hovered());
    assert!(!b.is_pressed());
}
