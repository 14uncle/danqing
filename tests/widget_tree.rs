//! @author 十四叔
//! @date 2026/07/17

//! 集成测试:组件树构建 + 布局 + 绘制命令收集(纯逻辑,无需 GPU)。

use danqing::widget::{self, Box as UiBox, Button, Center, Column, Row, Text, TextInput, Widget};
use danqing::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

struct AppState {
    count: u32,
}

fn build_tree() -> danqing::widget::Node {
    widget::node(
        UiBox::new(Color::from_srgb8(0x1A, 0x29, 0x3D)).child(
            UiBox::new(Color::WHITE)
                .size(100.0, 50.0)
                .radius(8.0)
                .child(Text::bind(|s: &AppState| format!("count {}", s.count))),
        ),
    )
}

#[test]
fn tree_layout_paint_collects_commands() {
    let mut texts = danqing::TextBatch::new();
    let mut tree = build_tree();

    tree.sync(&AppState { count: 7 });
    let screen = Size::new(800.0, 600.0);
    let size = tree.layout(Constraints::tight(screen), &mut texts);
    assert_eq!(size, screen, "根 Box 应占满窗口约束");

    let mut rects = danqing::RectBatch::new();
    tree.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);
    assert_eq!(rects.len(), 2, "两个 Box 各产生一条矩形命令");
    assert!(!texts.is_empty(), "绑定文本应产生字形命令");
}

#[test]
fn text_binding_updates_each_sync() {
    let mut texts = danqing::TextBatch::new();
    let mut text_widget = Text::bind(|s: &AppState| format!("count {}", s.count));

    text_widget.sync(&AppState { count: 1 });
    let loose = Constraints::loose(Size::new(800.0, 600.0));
    let w1 = text_widget.layout(loose, &mut texts).width;

    text_widget.sync(&AppState { count: 1000 });
    let w2 = text_widget.layout(loose, &mut texts).width;

    assert!(w2 > w1, "绑定更新后文本应变宽: {w1} -> {w2}");
}

#[test]
fn input_row_renders_text_input_background() {
    let mut texts = danqing::TextBatch::new();
    let mut tree = widget::node(
        Row::new()
            .gap(16.0)
            .child(Text::new("输入:").font_size(20).color(Color::WHITE))
            .child(
                TextInput::new()
                    .width(240.0)
                    .font_size(20)
                    .on_change(|s: &str| s.to_string()),
            )
            .fill(
                Center::new(Text::new("已输入: ").font_size(20).color(Color::WHITE)),
                1,
            ),
    );

    let size = tree.layout(Constraints::loose(Size::new(1280.0, 800.0)), &mut texts);
    assert!(size.height > 0.0, "输入行高度应大于零");

    let mut rects = danqing::RectBatch::new();
    tree.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);

    let bg = LightTheme.surface();
    let has_background = rects.instance_colors().iter().any(|c| {
        (c[0] - bg.r).abs() < 0.001
            && (c[1] - bg.g).abs() < 0.001
            && (c[2] - bg.b).abs() < 0.001
            && (c[3] - bg.a).abs() < 0.001
    });
    assert!(has_background, "应绘制出 TextInput 的背景");
}

#[test]
fn showcase_like_column_keeps_text_input_on_screen() {
    let mut texts = danqing::TextBatch::new();
    let screen = Size::new(1280.0, 800.0);
    let mut tree = widget::node(
        Column::new()
            .gap(16.0)
            .fill(
                Center::new(Text::new("danqing 丹青").font_size(20).color(Color::WHITE)),
                1,
            )
            .fill(UiBox::new(Color::from_srgb8(0xE6, 0x4C, 0x4C)), 6)
            .fill(UiBox::new(Color::from_srgb8(0x4C, 0xE6, 0xC3)), 2)
            .child(
                Row::new()
                    .gap(16.0)
                    .child(Button::new(
                        Text::new("点击 +1").font_size(20).color(Color::WHITE),
                    ))
                    .fill(
                        Center::new(Text::new("已点击 0 次").font_size(20).color(Color::WHITE)),
                        1,
                    ),
            )
            .child(
                Row::new()
                    .gap(16.0)
                    .child(Text::new("输入:").font_size(20).color(Color::WHITE))
                    .child(
                        TextInput::new()
                            .width(240.0)
                            .font_size(20)
                            .on_change(|s: &str| s.to_string()),
                    )
                    .fill(
                        Center::new(Text::new("已输入: ").font_size(20).color(Color::WHITE)),
                        1,
                    ),
            )
            .child(UiBox::new(Color::from_srgb8(0x1A, 0x29, 0x3D)).size(300.0, 180.0))
            .child(UiBox::new(Color::WHITE).size(100.0, 90.0)),
    );

    let size = tree.layout(Constraints::tight(screen), &mut texts);
    let mut rects = danqing::RectBatch::new();
    tree.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);

    let bg = LightTheme.surface();
    let input_rects: Vec<Rect> = rects
        .instance_rects()
        .into_iter()
        .zip(rects.instance_colors())
        .filter(|(_, c)| {
            (c[0] - bg.r).abs() < 0.001
                && (c[1] - bg.g).abs() < 0.001
                && (c[2] - bg.b).abs() < 0.001
                && (c[3] - bg.a).abs() < 0.001
        })
        .map(|(r, _)| r)
        .collect();

    assert!(
        input_rects
            .iter()
            .any(|r| r.size.width >= 200.0 && r.size.height >= 30.0),
        "Column 内应存在尺寸合理的 TextInput 背景;实际背景矩形 {:?}",
        input_rects
    );

    let input_y = input_rects
        .iter()
        .find(|r| r.size.width >= 200.0 && r.size.height >= 30.0)
        .map(|r| r.origin.y)
        .unwrap_or(f32::MAX);
    assert!(
        input_y < screen.height,
        "TextInput 顶边应位于屏幕内,实际 y={input_y}"
    );
    assert!(
        input_y > 100.0,
        "TextInput 不应被挤到标题上方,实际 y={input_y}"
    );
}
