// ! @author 十四叔
// ! @date 2026/07/17

// ! 丹青 showcase —— M1 演示页。
// !
// ! 本示例是唯一且持续生长的演示程序: 框架每落地一项能力,
// ! 就在这里展示一项 (以用代测)。

use danqing::widget::{
    self, Box as UiBox, Button, Center, Column, EventResult, MsgQueue, Node, Padding, Row, Text,
    TextInput, Widget,
};
use danqing::{App, Color, Event, Key, NamedKey, Point, Size};
use std::io::Write;
use std::time::SystemTime;

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

const PALETTE: [Color; 6] = [
    Color::from_srgb8(0xE6, 0x4C, 0x4C),
    Color::from_srgb8(0xE6, 0x9F, 0x4C),
    Color::from_srgb8(0xE0, 0xD5, 0x4F),
    Color::from_srgb8(0x5B, 0xB8, 0x5F),
    Color::from_srgb8(0x4C, 0x9F, 0xE6),
    Color::from_srgb8(0x8A, 0x6F, 0xD6),
];

/// 色板区:6×6 网格, 等分填充。
fn palette_grid() -> Column {
    let mut grid = Column::new().gap(8.0);
    for color in PALETTE {
        let mut row = Row::new().gap(8.0);
        for _ in 0..6 {
            row = row.fill(UiBox::new(color), 1);
        }
        grid = grid.fill(row, 1);
    }
    grid
}

/// 圆角区: 同一颜色、递增圆角半径。
fn rounded_row() -> Row {
    let teal = Color::from_srgb8(0x4C, 0xE6, 0xC3);
    let mut row = Row::new().gap(8.0);
    for radius in [0.0f32, 8.0, 16.0, 24.0, 36.0, 48.0] {
        row = row.fill(UiBox::new(teal).radius(radius), 1);
    }
    row
}

/// 交互区: 按钮 + 计数文本。
fn counter_row() -> Row {
    Row::new()
        .gap(16.0)
        .child(
            Button::new(Text::new("点击 +1").font_size(20).color(Color::WHITE))
                .on_click(|| Msg::Increment)
                .radius(12.0),
        )
        .fill(
            Center::new(
                Text::bind(|s: &Showcase| format!("已点击 {} 次", s.count))
                    .font_size(20)
                    .color(Color::WHITE),
            ),
            1,
        )
}

/// 键盘区: 方向键 /WASD 移动方块, 并回显最后按下的字符键。
fn keyboard_row() -> Row {
    Row::new()
        .gap(16.0)
        .fill(
            UiBox::new(Color::from_srgb8(0x1A, 0x29, 0x3D))
                .size(KEYBOARD_AREA.width, KEYBOARD_AREA.height)
                .radius(12.0)
                .child(
                    Positioned::bind(
                        |s: &Showcase| s.square_pos,
                        UiBox::new(Color::from_srgb8(0xE6, 0x4C, 0x9F))
                            .size(SQUARE_SIZE, SQUARE_SIZE)
                            .radius(8.0),
                    )
                    .hoverable(false),
                ),
            2,
        )
        .fill(
            Center::new(
                Text::bind(|s: &Showcase| format!("最后按键: {}", s.last_key))
                    .font_size(20)
                    .color(Color::WHITE),
            ),
            1,
        )
}

/// 输入区:TextInput + 实时回显。
fn input_row() -> Row {
    Row::new()
        .gap(16.0)
        .child(Text::new("输入:").font_size(20).color(Color::WHITE))
        .child(
            TextInput::new()
                .width(240.0)
                .font_size(20)
                .on_change(|s: &str| Msg::InputChanged(s.to_string())),
        )
        .fill(
            Center::new(
                Text::bind(|s: &Showcase| format!("已输入: {}", s.input_value))
                    .font_size(20)
                    .color(Color::WHITE),
            ),
            1,
        )
}

/// 绝对 / 相对定位容器: 把子组件按状态绑定的偏移量平移。
///
/// 本组件为 showcase 键盘演示专用, 放在示例文件中以保持框架核心精简。
struct Positioned {
    child: Node,
    offset: Point,
    binding: Option<Box<dyn Fn(&Showcase) -> Point>>,
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
        // 当前实现本身无交互效果, 保留接口以兼容调用链。
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

/// 构建组件树 (保留模式: 树只建一次, 数据每帧同步)。
fn build_tree() -> Node {
    widget::node(Padding::all(
        24.0,
        Column::new()
            .gap(16.0)
            .fill(
                Center::new(
                    Text::new("danqing 丹青 — 跨平台自绘 UI 框架")
                        .font_size(20)
                        .color(Color::WHITE),
                ),
                1,
            )
            .fill(palette_grid(), 6)
            .fill(rounded_row(), 2)
            .child(counter_row())
            .child(input_row())
            .child(keyboard_row())
            .child(
                Row::new()
                    .gap(12.0)
                    .fill(
                        UiBox::new(Color::from_srgb8(0xE6, 0x4C, 0x9F))
                            .height(90.0)
                            .radius(20.0),
                        2,
                    )
                    .fill(
                        UiBox::new(Color::from_srgb8(0x9F, 0x4C, 0xE6))
                            .height(90.0)
                            .radius(40.0),
                        1,
                    )
                    .fill(UiBox::new(Color::WHITE).height(90.0).radius(20.0), 1),
            ),
    ))
}

fn main() -> anyhow::Result<()> {
    // env_logger::init();
    init_log();

    let mut app = Showcase {
        count: 0,
        square_pos: Point::ZERO,
        last_key: String::from("-"),
        input_value: String::new(),
    };
    danqing::run_app(danqing::WindowConfig::default(), &mut app)?;
    Ok(())
}

fn init_log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = now.as_secs() % 86_400;
            let ms = now.subsec_millis();
            let hh = secs / 3600;
            let mm = (secs % 3600) / 60;
            let ss = secs % 60;
            writeln!(
                buf,
                "{:02}:{:02}:{:02}.{:03} {} [{}] {}",
                hh,
                mm,
                ss,
                ms,
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}
