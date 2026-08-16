//! @author 十四叔
//! @date 2026/07/19

//! 丹青 showcase —— 阶段 1 设计系统组件图鉴。
//!
//! 本示例是唯一且持续生长的演示程序：框架每落地一项能力，
//! 就在这里展示一项 (以用代测)。左侧按 widget/ 目录分类导航
//! (基础 / 布局 / 表单 / 视图), 右侧经 Switcher 切换分类面板;
//! 所有面板常驻实例化，切换不重建组件树。

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use danqing::widget::{
    self, Box as UiBox, Button, CloseButton, Column, EventResult, MsgQueue, Node, Padding, Row,
    Scrollable, Switcher, Text, TextArea, TextInput, TitleBar, Widget,
};
use danqing::{
    App, BackgroundConfig, Color, Event, Key, LightTheme, NamedKey, Point, Rect, ScaleMode, Size,
    Theme, WindowAction,
};
#[path = "common/log.rs"]
mod example_log;

/// 键盘移动方块的区域尺寸。
const KEYBOARD_AREA: Size = Size::new(300.0, 180.0);
/// 方块尺寸。
const SQUARE_SIZE: f32 = 40.0;
/// 每次按键移动步长 (逻辑像素)。
const MOVE_STEP: f32 = 20.0;

/// 分类导航：与 src/widget/ 子目录一一对应。
const CATEGORIES: [&str; 4] = ["基础 base", "布局 layout", "表单 form", "视图 view"];

/// showcase 应用 (状态容器 + 消息更新 + 视图树)。
struct Showcase {
    count: u32,
    square_pos: Point,
    last_key: String,
    input_value: String,
    textarea_value: String,
    /// 当前选中的分类索引 (驱动 Switcher)。
    selected: usize,
    /// 窗口是否已最大化 (决定标题栏按钮图标 □/□□)。
    is_maximized: bool,
    /// 当前显示的图像 (RGBA 数据，宽，高)。
    image_data: Option<(Vec<u8>, u32, u32)>,
}

/// 应用消息。
enum Msg {
    /// 计数器 +1。
    Increment,
    /// 计数器清零 (CloseButton 演示)。
    ResetCount,
    /// 移动键盘方块。
    MoveSquare { dx: f32, dy: f32 },
    /// 字符键输入。
    KeyChar(String),
    /// 文本输入框内容变化。
    InputChanged(String),
    /// 多行文本域内容变化。
    TextareaChanged(String),
    /// 切换分类面板。
    Select(usize),
    /// 打开本地图片。
    OpenImage,
}

impl App for Showcase {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::ResetCount => self.count = 0,
            Msg::MoveSquare { dx, dy } => {
                self.square_pos.x =
                    (self.square_pos.x + dx).clamp(0.0, KEYBOARD_AREA.width - SQUARE_SIZE);
                self.square_pos.y =
                    (self.square_pos.y + dy).clamp(0.0, KEYBOARD_AREA.height - SQUARE_SIZE);
            }
            Msg::KeyChar(c) => self.last_key = c,
            Msg::InputChanged(s) => self.input_value = s,
            Msg::TextareaChanged(s) => self.textarea_value = s,
            Msg::Select(i) => self.selected = i,
            Msg::OpenImage => {
                // 打开文件对话框选择图片
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("图片", &["png", "jpg", "jpeg", "gif", "bmp"])
                    .pick_file()
                {
                    match image::open(&path) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let (w, h) = rgba.dimensions();
                            self.image_data = Some((rgba.into_raw(), w, h));
                        }
                        Err(e) => log::warn!("加载图片失败：{e}"),
                    }
                }
            }
        }
    }

    fn view(&self) -> Node {
        build_tree()
    }

    fn event(&mut self, event: &Event) {
        let Event::Key { key, pressed, .. } = event else {
            return;
        };
        if !pressed {
            return;
        }
        let (dx, dy) = match key {
            Key::Named(NamedKey::ArrowLeft) => (-MOVE_STEP, 0.0),
            Key::Named(NamedKey::ArrowRight) => (MOVE_STEP, 0.0),
            Key::Named(NamedKey::ArrowUp) => (0.0, -MOVE_STEP),
            Key::Named(NamedKey::ArrowDown) => (0.0, MOVE_STEP),
            Key::Character(c) => {
                let lower = c.to_ascii_lowercase();
                match lower.as_str() {
                    "a" => (-MOVE_STEP, 0.0),
                    "d" => (MOVE_STEP, 0.0),
                    "w" => (0.0, -MOVE_STEP),
                    "s" => (0.0, MOVE_STEP),
                    _ => {
                        self.update(Msg::KeyChar(c.clone()));
                        return;
                    }
                }
            }
            _ => return,
        };
        self.update(Msg::MoveSquare { dx, dy });
    }

    fn maximized_changed(&mut self, is_maximized: bool) {
        self.is_maximized = is_maximized;
    }
}

/// 阶段 1 浅色主题。
fn theme() -> LightTheme {
    LightTheme
}

/// 卡片容器：带标题标签与主题化边框 / 圆角。
fn card(t: &LightTheme, title: &str, content: impl Widget + 'static) -> impl Widget + 'static {
    Column::new()
        .gap(t.spacing_sm())
        // 拉伸交叉轴：白底卡片与标题标签同宽，由外层内容列统一卡片宽度。
        .cross_stretch()
        .child(
            Text::new(title)
                .font_size(t.font_size_body())
                .color(t.text_secondary()),
        )
        .child(
            UiBox::themed(t)
                .radius(t.radius_lg())
                .child(Padding::all(t.spacing_lg(), content)),
        )
}

/// 品牌色样例区:6 行 × 6 列固定色块。
fn palette_grid(t: &LightTheme) -> impl Widget + 'static {
    let colors = [
        t.accent(),
        t.danger(),
        t.text_primary(),
        t.text_secondary(),
        t.divider(),
        t.border(),
    ];
    let mut col = Column::new().gap(t.spacing_sm());
    for color in colors {
        let mut row = Row::new().gap(t.spacing_sm());
        for _ in 0..6 {
            row = row.child(UiBox::new(color).size(40.0, 40.0).radius(t.radius_sm()));
        }
        col = col.child(row);
    }
    col
}

/// 圆角区：同一颜色、递增圆角半径。
fn rounded_row(t: &LightTheme) -> impl Widget + 'static {
    let mut row = Row::new().gap(t.spacing_sm());
    for radius in [
        0.0f32,
        t.radius_sm(),
        t.radius_md(),
        t.radius_lg(),
        24.0,
        36.0,
    ] {
        row = row.child(UiBox::themed(t).size(40.0, 40.0).radius(radius));
    }
    row
}

/// 品牌色与圆角卡片。
fn palette_and_rounded_card(t: &LightTheme) -> impl Widget + 'static {
    Column::new()
        .gap(t.spacing_md())
        .child(palette_grid(t))
        .child(rounded_row(t))
}

/// 交互区：按钮 + 计数文本。
fn counter_row(t: &LightTheme) -> impl Widget + 'static {
    let symbol_color = t.text_secondary();
    let hover_color = t.surface_variant();
    Row::new()
        .gap(t.spacing_lg())
        .child(
            Button::themed(
                t,
                Text::new("点击 +1")
                    .font_size(t.font_size_body())
                    .color(Color::WHITE),
            )
            .on_click(|| Msg::Increment),
        )
        .child(
            Text::bind(|s: &Showcase| format!("已点击 {} 次", s.count))
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
        // CloseButton: 矢量 × 按钮 (点击清零计数, hover 出底色)。
        .child(
            CloseButton::new()
                .on_click(|| Msg::ResetCount)
                .bind_color(move |_: &Showcase| symbol_color)
                .bind_hover_color(move |_: &Showcase| hover_color),
        )
}

/// 输入区:TextInput + 实时回显。
fn input_row(t: &LightTheme) -> impl Widget + 'static {
    Row::new()
        .gap(t.spacing_lg())
        .child(
            Text::new("输入：")
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
        .child(
            TextInput::themed(t)
                .width(240.0)
                .on_change(|s: &str| Msg::InputChanged(s.to_string())),
        )
        .child(
            Text::bind(|s: &Showcase| format!("已输入：{}", s.input_value))
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
}

/// 多行输入区:Scrollable + TextArea + 实时回显字数 / 行数。
fn textarea_card(t: &LightTheme) -> impl Widget + 'static {
    Row::new()
        .gap(t.spacing_lg())
        .child(
            Text::new("多行：")
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
        .child(
            // 透明尺寸壳：只为 Scrollable 提供 400×160 视口，背景职责归 TextArea,
            // 避免外层 UiBox 与 TextArea 双层 surface 叠出接缝。
            UiBox::new(Color::TRANSPARENT)
                .size(400.0, 160.0)
                .child(Scrollable::themed(
                    t,
                    TextArea::themed(t)
                        .width(400.0)
                        .height(160.0)
                        .on_change(|s: &str| Msg::TextareaChanged(s.to_string())),
                )),
        )
        .child(
            Text::bind(|s: &Showcase| {
                let chars = s.textarea_value.chars().count();
                let lines = s.textarea_value.lines().count();
                format!("字数：{} 行数：{}", chars, lines)
            })
            .font_size(t.font_size_body())
            .color(t.text_primary()),
        )
}

/// 键盘区：方向键 /WASD 移动方块，并回显最后按下的字符键。
fn keyboard_card(t: &LightTheme) -> impl Widget + 'static {
    Row::new()
        .gap(t.spacing_lg())
        .child(
            UiBox::themed(t)
                .size(KEYBOARD_AREA.width, KEYBOARD_AREA.height)
                .child(Positioned::bind(
                    |s: &Showcase| s.square_pos,
                    UiBox::new(t.accent())
                        .size(SQUARE_SIZE, SQUARE_SIZE)
                        .radius(t.radius_md()),
                )),
        )
        .child(
            Text::bind(|s: &Showcase| format!("最后按键：{}", s.last_key))
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
}

/// 绝对 / 相对定位容器：把子组件按状态绑定的偏移量平移。
///
/// 本组件为 showcase 键盘演示专用，放在示例文件中以保持框架核心精简。
///
/// 注意：本组件的绘制与事件区域随偏移量平移，可能超出自身布局矩形;
/// 父容器不得按布局矩形对其裁剪。
struct Positioned {
    child: Node,
    offset: Point,
    binding: Box<dyn Fn(&Showcase) -> Point>,
    child_size: Size,
}

impl Positioned {
    /// 按应用状态绑定偏移量。
    fn bind(f: impl Fn(&Showcase) -> Point + 'static, child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            offset: Point::ZERO,
            binding: Box::new(f),
            child_size: Size::ZERO,
        }
    }
}

impl Widget for Positioned {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.child.sync(state);
        let state = state
            .downcast_ref::<Showcase>()
            .expect("Positioned 绑定状态类型不匹配");
        self.offset = (self.binding)(state);
    }

    fn layout(
        &mut self,
        constraints: danqing::Constraints,
        texts: &mut danqing::TextBatch,
    ) -> Size {
        self.child_size = self.child.layout(constraints, texts);
        constraints.constrain(self.child_size)
    }

    fn paint(&self, area: Rect, rects: &mut danqing::RectBatch, texts: &mut danqing::TextBatch) {
        let origin = Point::new(area.origin.x + self.offset.x, area.origin.y + self.offset.y);
        self.child
            .paint(Rect::new(origin, self.child_size), rects, texts);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let origin = Point::new(area.origin.x + self.offset.x, area.origin.y + self.offset.y);
        self.child
            .event(event, Rect::new(origin, self.child_size), msgs)
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.child)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.child)
    }
}

/// 页面包装：Scrollable + Padding + 页标题 + 内容。
fn page(t: &LightTheme, heading: &str, content: impl Widget + 'static) -> impl Widget + 'static {
    Scrollable::themed(
        t,
        Padding::all(
            t.spacing_lg(),
            Column::new()
                .gap(t.spacing_lg())
                // 卡片统一为内容列宽，右边缘对齐。
                .cross_stretch()
                .child(
                    Text::new(heading)
                        .font_size(t.font_size_heading())
                        .color(t.text_primary()),
                )
                .child(content),
        ),
    )
}

/// 基础页：按钮与文本。
fn page_base(t: &LightTheme) -> impl Widget + 'static {
    page(
        t,
        "基础 base — 按钮与文本",
        Column::new()
            .gap(t.spacing_lg())
            .cross_stretch()
            .child(card(t, "按钮与计数", counter_row(t)))
            .child(card(t, "Image 组件", ImageDemo::new())),
    )
}

/// Image 组件演示：显示 LOGO 图片，支持打开本地图片。
struct ImageDemo {
    image: widget::Image,
    open_button: Node,
    /// 当前加载的图片尺寸 (用于显示文字信息)。
    image_info: Option<(u32, u32)>,
}

impl ImageDemo {
    fn new() -> Self {
        let t = theme();
        // 默认显示 LOGO 图片
        let (data, width, height) = load_logo();
        Self {
            image: widget::Image::new(data, width, height),
            open_button: Box::new(
                Button::themed(
                    &t,
                    Text::new("打开图片")
                        .font_size(t.font_size_body())
                        .color(Color::WHITE),
                )
                .on_click(|| Msg::OpenImage),
            ),
            image_info: Some((width, height)),
        }
    }

    /// 获取按钮实际尺寸 (包含 padding)。
    fn open_button_size(&self) -> Size {
        // Button 的 padding: Edges::symmetric(spacing_lg, spacing_md)
        // 竖直方向 padding = spacing_md * 2
        // 文字高度 ≈ font_size * 1.2
        // 总高度 = font_size * 1.2 + spacing_md * 2
        let t = theme();
        let text_height = t.font_size_body() as f32 * 1.2;
        let height = text_height + t.spacing_md() * 2.0;
        Size::new(100.0, height)
    }
}

impl Widget for ImageDemo {
    fn sync(&mut self, state: &dyn std::any::Any) {
        let state = state
            .downcast_ref::<Showcase>()
            .expect("ImageDemo 绑定状态类型不匹配");
        self.open_button.sync(state);
        // 如果有加载的图片，更新 image
        if let Some((data, w, h)) = &state.image_data {
            self.image = widget::Image::new(data.clone(), *w, *h);
            self.image_info = Some((*w, *h));
        }
    }

    fn layout(
        &mut self,
        constraints: danqing::Constraints,
        texts: &mut danqing::TextBatch,
    ) -> Size {
        // 第一行：按钮 + 图片信息 (横向排列)
        let button_size = self.open_button_size();
        // 让按钮进行 layout 以缓存其内部状态
        self.open_button
            .layout(danqing::Constraints::tight(button_size), texts);
        let row_height = button_size.height.max(20.0);

        // 图片区域
        let image_size = self.image.layout(
            danqing::Constraints::loose(Size::new(
                constraints.max_width,
                constraints.max_height - row_height - 8.0,
            )),
            texts,
        );
        Size::new(image_size.width, image_size.height + row_height + 8.0)
    }

    fn paint(&self, area: Rect, rects: &mut danqing::RectBatch, texts: &mut danqing::TextBatch) {
        // 第一行：按钮在左，图片信息在右
        // 按钮使用其自身计算的尺寸
        let button_size = self.open_button_size();
        let button_area = Rect::from_xywh(
            area.origin.x,
            area.origin.y,
            button_size.width,
            button_size.height,
        );
        self.open_button.paint(button_area, rects, texts);

        // 图片信息在按钮后面
        if let Some((w, h)) = &self.image_info {
            let info = format!("{}×{} px", w, h);
            let baseline =
                button_area.origin.y + button_area.size.height * 0.5 + texts.ascent(12.0) * 0.3;
            texts.push_text(
                &info,
                button_area.origin.x + button_area.size.width + 8.0,
                baseline,
                12,
                crate::Color::rgba(0.6, 0.6, 0.6, 1.0),
            );
        }

        // 图片在下方
        let image_area = Rect::from_xywh(
            area.origin.x,
            button_area.origin.y + button_size.height + 8.0,
            area.size.width,
            area.size.height - button_size.height - 8.0,
        );
        self.image.paint(image_area, rects, texts);
    }

    fn paint_image(&self, area: Rect, images: &mut danqing::ImageBatch) {
        // 图片在下方
        let button_size = self.open_button_size();
        let image_area = Rect::from_xywh(
            area.origin.x,
            area.origin.y + button_size.height + 8.0,
            area.size.width,
            area.size.height - button_size.height - 8.0,
        );
        self.image.paint_image(image_area, images);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let button_height = 36.0;
        let button_area =
            Rect::from_xywh(area.origin.x, area.origin.y, area.size.width, button_height);
        self.open_button.event(event, button_area, msgs)
    }
}

/// 从 assets/logo/logo_256.png 加载 LOGO 图片。
fn load_logo() -> (Vec<u8>, u32, u32) {
    let path = std::path::Path::new("assets/logo/logo_256.png");
    match image::open(path) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            (rgba.into_raw(), w, h)
        }
        Err(_) => {
            // 回退：创建一个 2x2 的默认图片
            let data = vec![
                100, 150, 180, 255, // 浅蓝
                120, 160, 170, 255, // 蓝绿
                90, 140, 160, 255, // 深蓝
                110, 150, 170, 255, // 中间色
            ];
            (data, 2, 2)
        }
    }
}

/// 布局页：盒模型与流式排布。
fn page_layout(t: &LightTheme) -> impl Widget + 'static {
    page(
        t,
        "布局 layout — 盒模型与流式排布",
        card(t, "品牌色与圆角", palette_and_rounded_card(t)),
    )
}

/// 表单页：单行与多行文本输入。
fn page_form(t: &LightTheme) -> impl Widget + 'static {
    page(
        t,
        "表单 form — 文本输入",
        Column::new()
            .gap(t.spacing_lg())
            .cross_stretch()
            .child(card(t, "单行输入", input_row(t)))
            .child(card(t, "多行输入", textarea_card(t))),
    )
}

/// 视图页：视口与可见性; 自定义组件演示。
fn page_view(t: &LightTheme) -> impl Widget + 'static {
    page(
        t,
        "视图 view — 每个面板都是 Scrollable, 分类切换由 Switcher 驱动",
        card(t, "键盘响应 (自定义 Positioned 组件)", keyboard_card(t)),
    )
}

/// 侧边栏导航项：选中时实心 accent (Button::bind_color 等状态绑定) 并在左缘绘制竖条。
///
/// 本组件为 showcase 导航专用，放在示例文件中以保持框架核心精简。
struct NavItem {
    button: Node,
    index: usize,
    selected: bool,
    marker_color: Color,
}

impl NavItem {
    /// 包装导航按钮，index 对应 Showcase.selected 的分类序号。
    fn new(index: usize, marker_color: Color, button: impl Widget + 'static) -> Self {
        Self {
            button: Box::new(button),
            index,
            selected: false,
            marker_color,
        }
    }
}

impl Widget for NavItem {
    fn sync(&mut self, state: &dyn std::any::Any) {
        let state = state
            .downcast_ref::<Showcase>()
            .expect("NavItem 绑定状态类型不匹配");
        self.selected = state.selected == self.index;
        self.button.sync(state);
    }

    fn layout(
        &mut self,
        constraints: danqing::Constraints,
        texts: &mut danqing::TextBatch,
    ) -> Size {
        self.button.layout(constraints, texts)
    }

    fn paint(&self, area: Rect, rects: &mut danqing::RectBatch, texts: &mut danqing::TextBatch) {
        self.button.paint(area, rects, texts);
        if self.selected {
            // 选中竖条：左缘内缩 4px，宽 3px，高为按钮的一半，圆角拉满成胶囊。
            let bar_w = 3.0;
            let bar_h = area.size.height * 0.5;
            let bar = Rect::from_xywh(
                area.origin.x + 4.0,
                area.origin.y + (area.size.height - bar_h) / 2.0,
                bar_w,
                bar_h,
            );
            rects.push_rect(bar, self.marker_color, bar_w / 2.0);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.button.event(event, area, msgs)
    }

    fn children(&self) -> &[Node] {
        std::slice::from_ref(&self.button)
    }

    fn children_mut(&mut self) -> &mut [Node] {
        std::slice::from_mut(&mut self.button)
    }
}

/// 比 accent 深一档的品牌色 (派生而非魔法值:RGB 缩 0.8)。
fn accent_strong(t: &LightTheme) -> Color {
    let a = t.accent();
    Color::rgba(a.r * 0.8, a.g * 0.8, a.b * 0.8, a.a)
}

/// accent 的低透明淡染 (派生而非魔法值): ghost 导航项的 hover 底色。
///
/// 白玻璃侧栏上 `surface_variant` 与合成底色几乎同值，亮度型 hover 不可辨;
/// 带色相偏移的 accent wash 才能在玻璃上读出悬停态。
fn accent_wash(t: &LightTheme, alpha: f32) -> Color {
    let a = t.accent();
    Color::rgba(a.r, a.g, a.b, alpha)
}

/// 侧边栏：分类导航;未选中项 ghost (透明底 + 深色字),选中项实心 accent + 白字 + 左缘竖条。
fn sidebar(t: &LightTheme) -> impl Widget + 'static {
    let mut col = Column::new().gap(t.spacing_sm()).cross_stretch();
    for (i, name) in CATEGORIES.iter().enumerate() {
        let name = *name;
        let accent = t.accent();
        let strong = accent_strong(t);
        let hover_bg = accent_wash(t, 0.12);
        let idle_text = t.text_primary();
        let marker = t.surface();
        col = col.child(NavItem::new(
            i,
            marker,
            Button::themed(
                t,
                Text::new(name)
                    .font_size(t.font_size_body())
                    .bind_color(move |s: &Showcase| {
                        if s.selected == i {
                            Color::WHITE
                        } else {
                            idle_text
                        }
                    }),
            )
            .bind_color(move |s: &Showcase| {
                if s.selected == i {
                    accent
                } else {
                    Color::TRANSPARENT
                }
            })
            .bind_hover_color(
                move |s: &Showcase| {
                    if s.selected == i { strong } else { hover_bg }
                },
            )
            .bind_focus_color(move |s: &Showcase| {
                if s.selected == i {
                    Color::WHITE
                } else {
                    accent
                }
            })
            .on_click(move || Msg::Select(i)),
        ));
    }
    UiBox::themed(t)
        .radius(t.radius_lg())
        .width(160.0)
        .child(Padding::all(t.spacing_md(), col))
}

/// 构建组件树 (保留模式：树只建一次，数据每帧同步)。
fn build_tree() -> Node {
    let t = theme();
    widget::node(
        Column::new()
            .child(
                TitleBar::themed(&t, "danqing 丹青")
                    .bind_maximized(|s: &Showcase| s.is_maximized)
                    .on_close(|| WindowAction::Close)
                    .on_minimize(|| WindowAction::Minimize)
                    .on_maximize(|| WindowAction::MaximizeOrRestore)
                    .on_drag(|| WindowAction::Drag),
            )
            .fill(
                Row::new()
                    .child(Padding::all(t.spacing_lg(), sidebar(&t)))
                    // 分类面板：四个页面常驻实例化，Switcher 只切换可见性。
                    .fill(
                        Switcher::new()
                            .child(page_base(&t))
                            .child(page_layout(&t))
                            .child(page_form(&t))
                            .child(page_view(&t))
                            .bind(|s: &Showcase| s.selected),
                        1,
                    ),
                1,
            ),
    )
}

fn main() -> anyhow::Result<()> {
    example_log::init_log();

    let mut app = Showcase {
        count: 0,
        square_pos: Point::ZERO,
        last_key: String::from("-"),
        input_value: String::new(),
        textarea_value: String::new(),
        selected: 0,
        is_maximized: false,
        image_data: None,
    };

    let t = theme();
    let background = BackgroundConfig::with_image("assets/background/gradient.png")
        .scale(ScaleMode::Cover)
        .with_glow("assets/background/glow.png", 0.25)
        .with_noise("assets/background/noise.png", 0.06);
    let config = danqing::WindowConfig {
        clear_color: t.background(),
        background,
        ..danqing::WindowConfig::default()
    };
    danqing::run_app(config, &mut app)?;
    Ok(())
}
