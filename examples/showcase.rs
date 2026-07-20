//! @author 十四叔
//! @date 2026/07/19

//! 丹青 showcase —— 阶段 1 毛玻璃演示页。
//!
//! 本示例是唯一且持续生长的演示程序: 框架每落地一项能力,
//! 就在这里展示一项 (以用代测)。当前使用 LightTheme 与主题化组件,
//! 呈现统一的浅色毛玻璃视觉。

use danqing::widget::{
    self, Box as UiBox, Button, Center, Column, EventResult, MsgQueue, Node, Padding, Row,
    Scrollable, Text, TextArea, TextInput, TitleBar, Widget,
};
use danqing::{
    App, BackgroundConfig, Event, Key, LightTheme, NamedKey, Point, ScaleMode, Size, Theme,
};
use std::io::Write;

/// 键盘移动方块的区域尺寸。
const KEYBOARD_AREA: Size = Size::new(300.0, 180.0);
/// 方块尺寸。
const SQUARE_SIZE: f32 = 40.0;
/// 每次按键移动步长 (逻辑像素)。
const MOVE_STEP: f32 = 20.0;

/// showcase 应用 (状态容器 + 消息更新 + 视图树)。
struct Showcase {
    count: u32,
    square_pos: Point,
    last_key: String,
    input_value: String,
    textarea_value: String,
}

/// 应用消息。
enum Msg {
    /// 计数器 +1。
    Increment,
    /// 移动键盘方块。
    MoveSquare { dx: f32, dy: f32 },
    /// 字符键输入。
    KeyChar(String),
    /// 文本输入框内容变化。
    InputChanged(String),
    /// 多行文本域内容变化。
    TextareaChanged(String),
}

impl App for Showcase {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::Increment => self.count += 1,
            Msg::MoveSquare { dx, dy } => {
                self.square_pos.x =
                    (self.square_pos.x + dx).clamp(0.0, KEYBOARD_AREA.width - SQUARE_SIZE);
                self.square_pos.y =
                    (self.square_pos.y + dy).clamp(0.0, KEYBOARD_AREA.height - SQUARE_SIZE);
            }
            Msg::KeyChar(c) => self.last_key = c,
            Msg::InputChanged(s) => self.input_value = s,
            Msg::TextareaChanged(s) => self.textarea_value = s,
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
}

/// 阶段 1 浅色主题。
fn theme() -> LightTheme {
    LightTheme
}

/// 品牌强调色样例区:6×6 网格,等分填充。
fn palette_grid() -> Column {
    let t = theme();
    let colors = [
        t.accent(),
        t.danger(),
        t.text_primary(),
        t.text_secondary(),
        t.divider(),
        t.border(),
    ];
    let mut grid = Column::new().gap(t.spacing_sm());
    for color in colors {
        let mut row = Row::new().gap(t.spacing_sm());
        for _ in 0..6 {
            row = row.fill(UiBox::new(color).radius(t.radius_sm()), 1);
        }
        grid = grid.fill(row, 1);
    }
    grid
}

/// 圆角区:同一颜色、递增圆角半径。
fn rounded_row() -> Row {
    let t = theme();
    let mut row = Row::new().gap(t.spacing_sm());
    for radius in [
        0.0f32,
        t.radius_sm(),
        t.radius_md(),
        t.radius_lg(),
        24.0,
        36.0,
    ] {
        row = row.fill(UiBox::themed(&t).radius(radius), 1);
    }
    row
}

/// 交互区:按钮 + 计数文本。
fn counter_row() -> Row {
    let t = theme();
    Row::new()
        .gap(t.spacing_lg())
        .child(
            Button::themed(
                &t,
                Text::new("点击 +1")
                    .font_size(t.font_size_body())
                    .color(t.surface()),
            )
            .on_click(|| Msg::Increment),
        )
        .fill(
            Center::new(
                Text::bind(|s: &Showcase| format!("已点击 {} 次", s.count))
                    .font_size(t.font_size_body())
                    .color(t.text_primary()),
            ),
            1,
        )
}

/// 键盘区:方向键 /WASD 移动方块,并回显最后按下的字符键。
fn keyboard_row() -> Row {
    let t = theme();
    Row::new()
        .gap(t.spacing_lg())
        .fill(
            UiBox::themed(&t)
                .size(KEYBOARD_AREA.width, KEYBOARD_AREA.height)
                .child(
                    Positioned::bind(
                        |s: &Showcase| s.square_pos,
                        UiBox::new(t.accent())
                            .size(SQUARE_SIZE, SQUARE_SIZE)
                            .radius(t.radius_md()),
                    )
                    .hoverable(false),
                ),
            2,
        )
        .fill(
            Center::new(
                Text::bind(|s: &Showcase| format!("最后按键: {}", s.last_key))
                    .font_size(t.font_size_body())
                    .color(t.text_primary()),
            ),
            1,
        )
}

/// 输入区:TextInput + 实时回显。
fn input_row() -> Row {
    let t = theme();
    Row::new()
        .gap(t.spacing_lg())
        .child(
            Text::new("输入:")
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
        .child(
            TextInput::themed(&t)
                .width(240.0)
                .on_change(|s: &str| Msg::InputChanged(s.to_string())),
        )
        .fill(
            Center::new(
                Text::bind(|s: &Showcase| format!("已输入: {}", s.input_value))
                    .font_size(t.font_size_body())
                    .color(t.text_primary()),
            ),
            1,
        )
}

/// 多行输入区:Scrollable + TextArea + 实时回显字数/行数。
fn textarea_row() -> Row {
    let t = theme();
    Row::new()
        .gap(t.spacing_lg())
        .child(
            Text::new("多行:")
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
        .child(
            UiBox::themed(&t)
                .size(400.0, 160.0)
                .child(Scrollable::themed(
                    &t,
                    TextArea::themed(&t)
                        .width(400.0)
                        .on_change(|s: &str| Msg::TextareaChanged(s.to_string())),
                )),
        )
        .fill(
            Center::new(
                Text::bind(|s: &Showcase| {
                    let chars = s.textarea_value.chars().count();
                    let lines = s.textarea_value.lines().count();
                    format!("字数:{} 行数:{}", chars, lines)
                })
                .font_size(t.font_size_body())
                .color(t.text_primary()),
            ),
            1,
        )
}

/// 定位绑定函数类型。
type PositionBinding = Box<dyn Fn(&Showcase) -> Point>;

/// 绝对 / 相对定位容器:把子组件按状态绑定的偏移量平移。
///
/// 本组件为 showcase 键盘演示专用,放在示例文件中以保持框架核心精简。
struct Positioned {
    child: Node,
    offset: Point,
    binding: Option<PositionBinding>,
    child_size: Size,
}

impl Positioned {
    /// 按应用状态绑定偏移量。
    fn bind(f: impl Fn(&Showcase) -> Point + 'static, child: impl Widget + 'static) -> Self {
        Self {
            child: Box::new(child),
            offset: Point::ZERO,
            binding: Some(Box::new(f)),
            child_size: Size::ZERO,
        }
    }

    /// 关闭 hover/pressed 交互效果 (键盘区方块不需要)。
    fn hoverable(self, hoverable: bool) -> Self {
        // 当前实现本身无交互效果,保留接口以兼容调用链。
        let _ = hoverable;
        self
    }
}

impl Widget for Positioned {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.child.sync(state);
        if let Some(binding) = &self.binding {
            let state = state
                .downcast_ref::<Showcase>()
                .expect("Positioned 绑定状态类型不匹配");
            self.offset = binding(state);
        }
    }

    fn layout(
        &mut self,
        constraints: danqing::Constraints,
        texts: &mut danqing::TextBatch,
    ) -> Size {
        self.child_size = self.child.layout(constraints, texts);
        constraints.constrain(self.child_size)
    }

    fn paint(
        &self,
        area: danqing::Rect,
        rects: &mut danqing::RectBatch,
        texts: &mut danqing::TextBatch,
    ) {
        let origin = Point::new(area.origin.x + self.offset.x, area.origin.y + self.offset.y);
        self.child
            .paint(danqing::Rect::new(origin, self.child_size), rects, texts);
    }

    fn event(&mut self, event: &Event, area: danqing::Rect, msgs: &mut MsgQueue) -> EventResult {
        let origin = Point::new(area.origin.x + self.offset.x, area.origin.y + self.offset.y);
        self.child
            .event(event, danqing::Rect::new(origin, self.child_size), msgs)
    }
}

/// 构建组件树 (保留模式:树只建一次,数据每帧同步)。
fn build_tree() -> Node {
    let t = theme();
    widget::node(
        Column::new()
            .child(TitleBar::themed(&t, "danqing 丹青"))
            .fill(
                Padding::all(
                    t.spacing_lg(),
                    Column::new()
                        .gap(t.spacing_md())
                        .fill(
                            Center::new(
                                Text::new("跨平台自绘 UI 框架 — 阶段 1 设计系统")
                                    .font_size(t.font_size_heading())
                                    .color(t.text_primary()),
                            ),
                            1,
                        )
                        .fill(palette_grid(), 6)
                        .fill(rounded_row(), 2)
                        .child(counter_row())
                        .child(input_row())
                        .child(textarea_row())
                        .child(keyboard_row())
                        .child(
                            Row::new()
                                .gap(t.spacing_md())
                                .fill(UiBox::themed(&t).height(90.0).radius(t.radius_lg()), 2)
                                .fill(UiBox::themed(&t).height(90.0).radius(t.radius_lg()), 1)
                                .fill(
                                    UiBox::new(t.surface_variant())
                                        .height(90.0)
                                        .radius(t.radius_lg()),
                                    1,
                                ),
                        ),
                ),
                1,
            ),
    )
}

fn main() -> anyhow::Result<()> {
    // env_logger::init();
    init_log();

    let mut app = Showcase {
        count: 0,
        square_pos: Point::ZERO,
        last_key: String::from("-"),
        input_value: String::new(),
        textarea_value: String::new(),
    };

    let t = theme();
    let out_dir = std::path::PathBuf::from(env!("OUT_DIR"));
    let background = BackgroundConfig::with_image(
        out_dir
            .join("assets")
            .join("background")
            .join("gradient.png"),
    )
    .with_noise(
        out_dir.join("assets").join("background").join("noise.png"),
        0.08,
    )
    .scale(ScaleMode::Cover);
    let config = danqing::WindowConfig {
        clear_color: t.background(),
        background,
        ..danqing::WindowConfig::default()
    };
    danqing::run_app(config, &mut app)?;
    Ok(())
}

fn init_log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let now = chrono::Local::now();
            writeln!(
                buf,
                "{} {} [{}] {}",
                now.format("%H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}
