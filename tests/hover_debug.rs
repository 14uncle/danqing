//! @author 十四叔
//! @date 2026/07/17

//! 集成测试:深层组件树的 hover 分发与绘制颜色联动(回归)。

use danqing::event::Event;
use danqing::widget::{self, Box as UiBox, Column, Padding, Row};
use danqing::{Color, Constraints, Point, Rect, Size};

#[test]
fn deep_hover_changes_paint_color() {
    let pink = Color::from_srgb8(0xE6, 0x4C, 0x9F);
    let mut tree = widget::node(Padding::all(
        24.0,
        Column::new()
            .gap(16.0)
            .fill(
                Column::new()
                    .gap(8.0)
                    .fill(Row::new().gap(8.0).fill(UiBox::new(Color::BLACK), 1), 1),
                6,
            )
            .child(
                Row::new()
                    .gap(12.0)
                    .fill(UiBox::new(pink).height(90.0).hoverable(true), 2)
                    .fill(UiBox::new(Color::WHITE).height(90.0), 1),
            ),
    ));
    let mut texts = danqing::TextBatch::new();
    let screen = Size::new(400.0, 300.0);
    let size = tree.layout(Constraints::tight(screen), &mut texts);
    let root = Rect::new(Point::ZERO, size);

    // 悬停前:pink 实例颜色 = 原色
    let mut rects = danqing::RectBatch::new();
    tree.paint(root, &mut rects, &mut texts);
    let before = rects.instance_colors();
    assert!(
        before.iter().any(|c| (c[0] - pink.r).abs() < 0.01),
        "应有粉色实例: {before:?}"
    );

    // 悬停在粉色块上(底部区域)
    let specials_y = 300.0 - 24.0 - 45.0; // 底 padding 24,半高 45
    let r = tree.event(
        &Event::CursorMoved(Point::new(100.0, specials_y)),
        root,
        &mut Vec::new(),
    );
    println!("hover result: {r:?}");

    let mut rects2 = danqing::RectBatch::new();
    tree.paint(root, &mut rects2, &mut texts);
    let after = rects2.instance_colors();
    let hovered = after
        .iter()
        .any(|c| c[1] > pink.g * 1.15 && (c[0] - pink.r).abs() < 0.3);
    println!("before: {before:?}");
    println!("after:  {after:?}");
    assert!(hovered, "悬停后粉色块应变亮");
}
