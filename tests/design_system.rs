//! @author 十四叔
//! @date 2026/07/20

//! 设计系统集成测试。
//!
//! 验证阶段 1 设计系统对外 API 的行为:
//! - `LightTheme` token 自洽;
//! - 主题化组件(`Box`/`Button`/`TextInput`/`TextArea`/`Scrollable`)实际使用 theme token 绘制;
//! - `TitleBar` 提供命中区域，消费按钮区与拖拽区鼠标事件并产出 `WindowAction`;
//! - `BackgroundConfig` 与 `ScaleMode` 构造符合预期。

use danqing::widget::{
    Box as UiBox, Button, EventResult, MsgQueue, Scrollable, Text, TextArea, TextInput, TitleBar,
    Widget,
};
use danqing::{
    BackgroundConfig, Color, Constraints, Event, LightTheme, MouseButton, Point, Rect, ScaleMode,
    Size, Theme, WindowAction, WindowConfig,
};

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

fn color_eq(a: Color, b: Color) -> bool {
    approx_eq(a.r, b.r) && approx_eq(a.g, b.g) && approx_eq(a.b, b.b) && approx_eq(a.a, b.a)
}

fn color_from_array(c: [f32; 4]) -> Color {
    Color::rgba(c[0], c[1], c[2], c[3])
}

#[test]
fn theme_tokens_are_consistent() {
    let t = LightTheme;
    assert!(t.background().a > 0.0);
    assert!(t.surface().a > 0.0);
    assert!(t.surface_variant().a > 0.0);
    assert!(t.accent().a > 0.0);
    assert!(t.text_primary().a > 0.0);
    assert!(t.text_secondary().a > 0.0);
    assert!(t.divider().a > 0.0);
    assert!(t.border().a > 0.0);
    assert!(t.selection().a > 0.0);
    assert!(t.caret().a > 0.0);
    assert!(t.danger().a > 0.0);

    assert!(t.font_size_small() < t.font_size_body());
    assert!(t.font_size_body() < t.font_size_heading());

    assert!(t.spacing_xs() < t.spacing_sm());
    assert!(t.spacing_sm() < t.spacing_md());
    assert!(t.spacing_md() < t.spacing_lg());
    assert!(t.spacing_lg() < t.spacing_xl());

    assert!(t.radius_sm() < t.radius_md());
    assert!(t.radius_md() < t.radius_lg());

    assert!(t.shadow_sm().blur_radius >= 0.0);
    assert!(t.shadow_md().blur_radius >= 0.0);
    assert!(t.shadow_sm().color.a > 0.0);
    assert!(t.shadow_md().color.a > 0.0);
}

#[test]
fn box_paints_with_theme_surface() {
    let t = LightTheme;
    let mut widget = UiBox::themed(&t).size(100.0, 100.0);
    let mut texts = danqing::TextBatch::new();
    let mut rects = danqing::RectBatch::new();

    let size = widget.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
    widget.paint(
        Rect::from_xywh(0.0, 0.0, size.width, size.height),
        &mut rects,
        &mut texts,
    );

    assert_eq!(rects.len(), 1);
    assert!(color_eq(
        color_from_array(rects.instance_colors()[0]),
        t.surface()
    ));
}

#[test]
fn button_paints_with_theme_accent() {
    let t = LightTheme;
    let mut widget = Button::themed(&t, Text::new("OK"));
    let mut texts = danqing::TextBatch::new();
    let mut rects = danqing::RectBatch::new();

    let size = widget.layout(Constraints::loose(Size::new(200.0, 100.0)), &mut texts);
    widget.paint(
        Rect::from_xywh(0.0, 0.0, size.width, size.height),
        &mut rects,
        &mut texts,
    );

    assert!(!rects.is_empty());
    assert!(color_eq(
        color_from_array(rects.instance_colors()[0]),
        t.accent()
    ));
}

#[test]
fn text_input_paints_with_theme_surface() {
    let t = LightTheme;
    let mut widget = TextInput::themed(&t).width(200.0);
    let mut texts = danqing::TextBatch::new();
    let mut rects = danqing::RectBatch::new();

    let size = widget.layout(Constraints::loose(Size::new(300.0, 100.0)), &mut texts);
    widget.paint(
        Rect::from_xywh(0.0, 0.0, size.width, size.height),
        &mut rects,
        &mut texts,
    );

    assert!(!rects.is_empty());
    assert!(color_eq(
        color_from_array(rects.instance_colors()[0]),
        t.surface()
    ));
}

#[test]
fn text_area_paints_with_theme_surface() {
    let t = LightTheme;
    let mut widget = TextArea::themed(&t).width(200.0);
    let mut texts = danqing::TextBatch::new();
    let mut rects = danqing::RectBatch::new();

    let size = widget.layout(Constraints::loose(Size::new(300.0, 200.0)), &mut texts);
    widget.paint(
        Rect::from_xywh(0.0, 0.0, size.width, size.height),
        &mut rects,
        &mut texts,
    );

    assert!(!rects.is_empty());
    assert!(color_eq(
        color_from_array(rects.instance_colors()[0]),
        t.surface()
    ));
}

#[test]
fn scrollable_child_paints_with_theme_surface() {
    let t = LightTheme;
    let mut widget = Scrollable::themed(&t, UiBox::themed(&t).size(50.0, 50.0));
    let mut texts = danqing::TextBatch::new();
    let mut rects = danqing::RectBatch::new();

    let size = widget.layout(Constraints::tight(Size::new(100.0, 100.0)), &mut texts);
    widget.paint(
        Rect::from_xywh(0.0, 0.0, size.width, size.height),
        &mut rects,
        &mut texts,
    );

    assert!(!rects.is_empty());
    assert!(color_eq(
        color_from_array(rects.instance_colors()[0]),
        t.surface()
    ));
}

#[test]
fn title_bar_has_hit_area_after_layout() {
    let t = LightTheme;
    let mut bar = TitleBar::themed(&t, "丹青");
    let mut texts = danqing::TextBatch::new();

    let size = bar.layout(Constraints::tight(Size::new(400.0, 40.0)), &mut texts);
    let area = bar.hit_area().expect("TitleBar 应提供命中区域");

    assert_eq!(area.size.width, size.width);
    assert_eq!(area.size.height, size.height);
}

#[test]
fn title_bar_close_button_consumes_mouse_event() {
    let t = LightTheme;
    let mut bar = TitleBar::themed(&t, "丹青");
    let mut texts = danqing::TextBatch::new();
    bar.layout(Constraints::tight(Size::new(400.0, 40.0)), &mut texts);

    let area = bar.hit_area().unwrap();
    let close_center = Point::new(area.size.width - 20.0, area.size.height / 2.0);
    let mut msgs = MsgQueue::new();

    let result = bar.event(&Event::CursorMoved(close_center), area, &mut msgs);

    assert_eq!(result, EventResult::Consumed);
}

#[test]
fn title_bar_left_click_outside_buttons_emits_drag() {
    let t = LightTheme;
    let mut bar = TitleBar::themed(&t, "丹青").on_drag(|| WindowAction::Drag);
    let mut texts = danqing::TextBatch::new();
    bar.layout(Constraints::tight(Size::new(400.0, 40.0)), &mut texts);

    let area = bar.hit_area().unwrap();
    let left_side = Point::new(10.0, area.size.height / 2.0);
    let mut msgs = MsgQueue::new();

    let result = bar.event(
        &Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: left_side,
        },
        area,
        &mut msgs,
    );

    assert_eq!(result, EventResult::Consumed);
    assert_eq!(msgs.len(), 1);
    let action = msgs[0].downcast_ref::<WindowAction>().unwrap();
    assert_eq!(*action, WindowAction::Drag);
}

#[test]
fn background_config_default_is_empty() {
    let cfg = BackgroundConfig::default();
    assert!(cfg.image.is_none());
    assert!(cfg.noise.is_none());
    assert_eq!(cfg.scale, ScaleMode::Stretch);
    assert!(approx_eq(cfg.noise_opacity, 0.0));
}

#[test]
fn background_config_chaining() {
    let cfg = BackgroundConfig::with_image("gradient.png")
        .with_noise("noise.png", 0.08)
        .scale(ScaleMode::Cover);

    assert_eq!(cfg.image.as_ref().unwrap().as_os_str(), "gradient.png");
    assert_eq!(cfg.noise.as_ref().unwrap().as_os_str(), "noise.png");
    assert_eq!(cfg.scale, ScaleMode::Cover);
    assert!(approx_eq(cfg.noise_opacity, 0.08));
}

#[test]
fn background_config_noise_opacity_is_clamped() {
    let high = BackgroundConfig::with_image("bg.png").with_noise("noise.png", 1.5);
    assert!(approx_eq(high.noise_opacity, 1.0));

    let low = BackgroundConfig::with_image("bg.png").with_noise("noise.png", -0.5);
    assert!(approx_eq(low.noise_opacity, 0.0));
}

#[test]
fn scale_mode_default_is_stretch() {
    assert_eq!(ScaleMode::default(), ScaleMode::Stretch);
}

#[test]
fn window_config_can_use_theme_and_background() {
    let t = LightTheme;
    let bg = BackgroundConfig::with_image("gradient.png");
    let cfg = WindowConfig {
        title: "test".into(),
        size: Size::new(800.0, 600.0),
        clear_color: t.background(),
        background: bg,
    };

    assert!(color_eq(cfg.clear_color, t.background()));
    assert_eq!(
        cfg.background.image.as_ref().unwrap().as_os_str(),
        "gradient.png"
    );
}
