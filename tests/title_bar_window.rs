//! @author 十四叔
//! @date 2026/07/20

//! 标题栏窗口控制集成测试。
//!
//! 模拟点击自绘标题栏的关闭/最小化/最大化按钮与拖拽区域,
//! 验证产出的 `WindowAction` 消息类型正确。

use danqing::event::{Event, MouseButton, WindowAction};
use danqing::widget::{EventResult, MsgQueue, TitleBar, TitleBarStyle, Widget};
use danqing::{Constraints, LightTheme, Point, Rect, Theme};

/// 400px 宽标题栏区域。
fn title_area() -> Rect {
    Rect::from_xywh(0.0, 0.0, 400.0, 40.0)
}

/// 根据主题计算第 i 个按钮的中心(0=关闭,1=最大化,2=最小化,从右往左)。
fn button_center(theme: &impl Theme, width: f32, index: usize) -> Point {
    let height = theme.spacing_xl() + theme.spacing_lg();
    let button_gap = 1.0;

    let right = width;
    let x = right - (index as f32 + 0.5) * height - index as f32 * button_gap;
    Point::new(x, height / 2.0)
}

/// 构造带全部窗口动作回调的标题栏。
fn titled_bar() -> TitleBar {
    TitleBar::themed(&LightTheme, "丹青")
        .on_close(|| WindowAction::Close)
        .on_minimize(|| WindowAction::Minimize)
        .on_maximize(|| WindowAction::MaximizeOrRestore)
        .on_drag(|| WindowAction::Drag)
}

/// 构造红绿灯布局、带全部窗口动作回调的标题栏。
fn traffic_bar() -> TitleBar {
    titled_bar().style(TitleBarStyle::TrafficLights)
}

/// 红绿灯布局下第 i 个按钮中心 (0=关闭,1=最小化,2=最大化,从左往右)。
///
/// 几何与 `TitleBar` 的 token 取值保持一致:
/// 直径 / 前导边距 = `spacing_md`, 间隙 = `spacing_sm`。
fn traffic_button_center(theme: &impl Theme, index: usize) -> Point {
    let height = theme.spacing_xl() + theme.spacing_lg();
    let diameter = theme.spacing_md();
    let gap = theme.spacing_sm();
    let leading = theme.spacing_md();
    let x = leading + index as f32 * (diameter + gap) + diameter / 2.0;
    Point::new(x, height / 2.0)
}

/// 在指定位置完成一次 hover + 按下 + 释放, 返回累积的消息队列。
fn click(bar: &mut TitleBar, position: Point) -> MsgQueue {
    let mut msgs = MsgQueue::new();
    bar.event(&Event::CursorMoved(position), title_area(), &mut msgs);
    bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position,
        },
        title_area(),
        &mut msgs,
    );
    bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position,
        },
        title_area(),
        &mut msgs,
    );
    msgs
}

fn layout(bar: &mut TitleBar) {
    let area = title_area();
    let mut texts = danqing::TextBatch::new();
    bar.layout(Constraints::tight(area.size), &mut texts);
}

#[test]
fn traffic_close_button_emits_close_action() {
    let mut bar = traffic_bar();
    layout(&mut bar);

    let msgs = click(&mut bar, traffic_button_center(&LightTheme, 0));

    assert_eq!(msgs.len(), 1);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::Close
    );
}

#[test]
fn traffic_minimize_button_emits_minimize_action() {
    let mut bar = traffic_bar();
    layout(&mut bar);

    let msgs = click(&mut bar, traffic_button_center(&LightTheme, 1));

    assert_eq!(msgs.len(), 1);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::Minimize
    );
}

#[test]
fn traffic_maximize_button_emits_maximize_action() {
    let mut bar = traffic_bar();
    layout(&mut bar);

    let msgs = click(&mut bar, traffic_button_center(&LightTheme, 2));

    assert_eq!(msgs.len(), 1);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::MaximizeOrRestore
    );
}

#[test]
fn traffic_drag_area_emits_drag_then_double_click_maximizes() {
    let mut bar = traffic_bar();
    layout(&mut bar);

    // 红绿灯组右侧的空白区仍是拖拽区。
    let position = Point::new(200.0, title_area().size.height / 2.0);
    let mut msgs = MsgQueue::new();
    let press = |pressed: bool| Event::MouseInput {
        button: MouseButton::Left,
        pressed,
        position,
    };

    bar.event(&press(true), title_area(), &mut msgs);
    // 快速在同一位置第二次按下, 应识别为双击最大化。
    bar.event(&press(true), title_area(), &mut msgs);

    assert_eq!(msgs.len(), 2);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::Drag
    );
    assert_eq!(
        *msgs[1].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::MaximizeOrRestore
    );
}

#[test]
fn title_bar_style_exported_at_widget_root() {
    // 构造成功即证明 TitleBarStyle 平铺导出与 .style() builder 可用。
    let _bar = TitleBar::themed(&LightTheme, "丹青").style(TitleBarStyle::Standard);
}

#[test]
fn close_button_emits_close_action() {
    let mut bar = titled_bar();
    layout(&mut bar);

    let center = button_center(&LightTheme, title_area().size.width, 0);
    let mut msgs = MsgQueue::new();

    bar.event(&Event::CursorMoved(center), title_area(), &mut msgs);
    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: center,
        },
        title_area(),
        &mut msgs,
    );
    bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position: center,
        },
        title_area(),
        &mut msgs,
    );

    assert_eq!(result, EventResult::Consumed);
    assert_eq!(msgs.len(), 1);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::Close
    );
}

#[test]
fn maximize_button_emits_maximize_action() {
    let mut bar = titled_bar();
    layout(&mut bar);

    let center = button_center(&LightTheme, title_area().size.width, 1);
    let mut msgs = MsgQueue::new();

    bar.event(&Event::CursorMoved(center), title_area(), &mut msgs);
    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: center,
        },
        title_area(),
        &mut msgs,
    );
    bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position: center,
        },
        title_area(),
        &mut msgs,
    );

    assert_eq!(result, EventResult::Consumed);
    assert_eq!(msgs.len(), 1);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::MaximizeOrRestore
    );
}

#[test]
fn minimize_button_emits_minimize_action() {
    let mut bar = titled_bar();
    layout(&mut bar);

    let center = button_center(&LightTheme, title_area().size.width, 2);
    let mut msgs = MsgQueue::new();

    bar.event(&Event::CursorMoved(center), title_area(), &mut msgs);
    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: center,
        },
        title_area(),
        &mut msgs,
    );
    bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position: center,
        },
        title_area(),
        &mut msgs,
    );

    assert_eq!(result, EventResult::Consumed);
    assert_eq!(msgs.len(), 1);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::Minimize
    );
}

#[test]
fn non_button_area_emits_drag_action() {
    let mut bar = titled_bar();
    layout(&mut bar);

    let position = Point::new(20.0, title_area().size.height / 2.0);
    let mut msgs = MsgQueue::new();

    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position,
        },
        title_area(),
        &mut msgs,
    );

    assert_eq!(result, EventResult::Consumed);
    assert_eq!(msgs.len(), 1);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::Drag
    );
}

#[test]
fn double_click_in_drag_area_emits_maximize_action() {
    let mut bar = titled_bar();
    layout(&mut bar);

    let position = Point::new(20.0, title_area().size.height / 2.0);
    let mut msgs = MsgQueue::new();

    // 第一次按下
    bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position,
        },
        title_area(),
        &mut msgs,
    );
    // 快速在同一位置第二次按下,应识别为双击最大化
    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position,
        },
        title_area(),
        &mut msgs,
    );

    assert_eq!(result, EventResult::Consumed);
    assert_eq!(msgs.len(), 2);
    assert_eq!(
        *msgs[0].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::Drag
    );
    assert_eq!(
        *msgs[1].downcast_ref::<WindowAction>().unwrap(),
        WindowAction::MaximizeOrRestore
    );
}

#[test]
fn right_click_is_ignored() {
    let mut bar = titled_bar();
    layout(&mut bar);

    let center = button_center(&LightTheme, title_area().size.width, 0);
    let mut msgs = MsgQueue::new();

    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Right,
            pressed: true,
            position: center,
        },
        title_area(),
        &mut msgs,
    );

    assert_eq!(result, EventResult::Ignored);
    assert!(msgs.is_empty());
}

#[test]
fn no_callback_produces_no_message() {
    let mut bar = TitleBar::themed(&LightTheme, "丹青");
    layout(&mut bar);

    let center = button_center(&LightTheme, title_area().size.width, 0);
    let mut msgs = MsgQueue::new();

    bar.event(&Event::CursorMoved(center), title_area(), &mut msgs);
    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: center,
        },
        title_area(),
        &mut msgs,
    );
    bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: false,
            position: center,
        },
        title_area(),
        &mut msgs,
    );

    assert_eq!(result, EventResult::Consumed);
    assert!(msgs.is_empty());
}
