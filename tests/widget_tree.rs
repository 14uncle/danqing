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

#[test]
fn click_empty_card_clears_focus_for_keyboard_fallback() {
    // showcase 键盘演示回归:树内存在可聚焦组件(Button)时,
    // 点击不可聚焦的卡片(键盘区)应清除焦点,后续按键才能回退到应用层。
    let mut texts = danqing::TextBatch::new();
    let mut tree = widget::node(
        Column::new()
            .gap(16.0)
            .child(Button::new(
                Text::new("点击 +1").font_size(20).color(Color::WHITE),
            ))
            .child(UiBox::new(Color::from_srgb8(0x1A, 0x29, 0x3D)).size(300.0, 180.0)),
    );
    let size = tree.layout(Constraints::tight(Size::new(1280.0, 800.0)), &mut texts);
    let mut rects = danqing::RectBatch::new();
    tree.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);

    let mut focus = danqing::widget::FocusManager::new();
    focus.rebuild(&tree);
    assert!(focus.current().is_some(), "初始应自动聚焦 Button");

    // 从绘制批次中找到键盘区卡片的实际位置,点击其中心(不可聚焦的 Box)。
    let dark = Color::from_srgb8(0x1A, 0x29, 0x3D);
    let card = rects
        .instance_rects()
        .into_iter()
        .zip(rects.instance_colors())
        .find(|(_, c)| {
            (c[0] - dark.r).abs() < 0.001
                && (c[1] - dark.g).abs() < 0.001
                && (c[2] - dark.b).abs() < 0.001
        })
        .map(|(r, _)| r)
        .expect("应找到键盘区卡片");
    focus.set_by_click(
        &tree,
        Point::new(
            card.origin.x + card.size.width / 2.0,
            card.origin.y + card.size.height / 2.0,
        ),
    );
    assert!(
        focus.current().is_none(),
        "点击不可聚焦卡片应清除焦点,实际 {:?}",
        focus.current()
    );
}

#[test]
fn stretched_column_gives_cards_uniform_width() {
    // 卡片式布局:内容列开启交叉轴拉伸后,自然宽度不同的卡片
    // 应统一为最宽卡片的宽度(showcase 卡片等宽需求)。
    let mut texts = danqing::TextBatch::new();
    let screen = Size::new(1280.0, 800.0);
    let mut tree = widget::node(
        Column::new()
            .gap(16.0)
            .cross_stretch()
            .child(
                UiBox::new(Color::WHITE)
                    .child(UiBox::new(Color::from_srgb8(0xE6, 0x4C, 0x4C)).size(200.0, 100.0)),
            )
            .child(
                UiBox::new(Color::WHITE)
                    .child(UiBox::new(Color::from_srgb8(0x4C, 0xE6, 0xC3)).size(320.0, 100.0)),
            ),
    );

    let size = tree.layout(Constraints::loose(screen), &mut texts);
    let mut rects = danqing::RectBatch::new();
    tree.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);

    let white_cards: Vec<Rect> = rects
        .instance_rects()
        .into_iter()
        .zip(rects.instance_colors())
        .filter(|(_, c)| {
            (c[0] - 1.0).abs() < 0.001 && (c[1] - 1.0).abs() < 0.001 && (c[2] - 1.0).abs() < 0.001
        })
        .map(|(r, _)| r)
        .collect();

    assert_eq!(white_cards.len(), 2, "应绘制两张白色卡片");
    assert!(
        white_cards
            .iter()
            .all(|r| (r.size.width - 320.0).abs() < 0.001),
        "两张卡片应等宽 320,实际 {:?}",
        white_cards.iter().map(|r| r.size.width).collect::<Vec<_>>()
    );
}

#[test]
fn fit_box_with_child_wraps_content_height() {
    // Box 带子组件但未显式指定尺寸时,未指定的维度应随子组件内容收缩;
    // 否则会占满父约束上限,把 Column 后续兄弟挤出屏幕
    // (showcase 卡片化后按钮/输入框不可见的回归)。
    let mut texts = danqing::TextBatch::new();
    let screen = Size::new(1280.0, 800.0);
    let green = Color::from_srgb8(0x4C, 0xE6, 0xC3);
    let mut tree = widget::node(
        Column::new()
            .gap(16.0)
            .child(
                UiBox::new(Color::WHITE)
                    .child(UiBox::new(Color::from_srgb8(0xE6, 0x4C, 0x4C)).size(200.0, 100.0)),
            )
            .child(UiBox::new(green).size(200.0, 100.0)),
    );

    let size = tree.layout(Constraints::tight(screen), &mut texts);
    let mut rects = danqing::RectBatch::new();
    tree.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);

    let second = rects
        .instance_rects()
        .into_iter()
        .zip(rects.instance_colors())
        .find(|(_, c)| {
            (c[0] - green.r).abs() < 0.001
                && (c[1] - green.g).abs() < 0.001
                && (c[2] - green.b).abs() < 0.001
        })
        .map(|(r, _)| r)
        .expect("应找到第二张卡片");

    // 第一张卡片包裹内容后高 100,第二张应位于 y = 100 + gap 16 = 116。
    assert!(
        (second.origin.y - 116.0).abs() < 0.001,
        "第二张卡片应紧跟第一张内容之后,实际 y={}",
        second.origin.y
    );
}

#[test]
fn fit_center_in_column_does_not_push_later_children_off_screen() {
    // Center 在 Flow 中作为 Fit 子项时会占满父约束上限;
    // 若把它当作普通 child 放在 Column 里,会独占全部剩余高度,
    // 导致后续卡片被挤出屏幕。本测试验证这种误用不会导致后续子项
    // 的绘制区域跑到屏幕外(修复前第一个后续子项的 y 会接近屏幕高度)。
    let mut texts = danqing::TextBatch::new();
    let screen = Size::new(1280.0, 800.0);
    let mut tree = widget::node(
        Column::new()
            .gap(16.0)
            .child(Center::new(
                Text::new("标题").font_size(20).color(Color::WHITE),
            ))
            .child(UiBox::new(Color::from_srgb8(0xE6, 0x4C, 0x4C)).size(200.0, 100.0))
            .child(UiBox::new(Color::from_srgb8(0x4C, 0xE6, 0xC3)).size(200.0, 100.0)),
    );

    let size = tree.layout(Constraints::tight(screen), &mut texts);
    let mut rects = danqing::RectBatch::new();
    tree.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);

    let red = Color::from_srgb8(0xE6, 0x4C, 0x4C);
    let first_card = rects
        .instance_rects()
        .into_iter()
        .zip(rects.instance_colors())
        .find(|(_, c)| {
            (c[0] - red.r).abs() < 0.001
                && (c[1] - red.g).abs() < 0.001
                && (c[2] - red.b).abs() < 0.001
        })
        .map(|(r, _)| r)
        .expect("应找到第一个红色卡片");

    assert!(
        first_card.origin.y < screen.height * 0.5,
        "第一个卡片不应被标题挤到屏幕下半部分,实际 y={}",
        first_card.origin.y
    );
}
