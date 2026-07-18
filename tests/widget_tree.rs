//! @author 十四叔
//! @date 2026/07/17

//! 集成测试:组件树构建 + 布局 + 绘制命令收集(纯逻辑,无需 GPU)。

use danqing::widget::{self, Box as UiBox, Text, Widget};
use danqing::{Color, Constraints, Point, Rect, Size};

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
