//! @author 十四叔
//! @date 2026/07/19

//! 丹青 showcase —— 阶段 1 设计系统组件图鉴。
//!
//! 本示例是唯一且持续生长的演示程序：框架每落地一项能力，
//! 就在这里展示一项 (以用代测)。左侧按 widget/ 目录分类导航
//! (基础 / 布局 / 表单 / 视图), 右侧经 MultiPanel 切换分类面板;
//! 所有面板常驻实例化，切换不重建组件树。

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use danqing::widget::{
    self, Box as UiBox, Button, CloseButton, Column, DragArea, EventResult, IconInput, MsgQueue,
    MultiPanel, Node, Padding, ReachArea, Row, Scrollable, Switch, Tabs, Text, TextArea, TextInput,
    TitleBar, Widget,
};
use danqing::{
    App, BackgroundConfig, Color, Event, GlobalHotkey, Key, LightTheme, NamedKey, Point, Rect,
    ScaleMode, Size, Theme, WindowAction, WindowEventSender,
};
/// 键盘移动方块的区域尺寸。
const KEYBOARD_AREA: Size = Size::new(300.0, 180.0);
/// 方块尺寸。
const SQUARE_SIZE: f32 = 40.0;
/// 每次按键移动步长 (逻辑像素)。
const MOVE_STEP: f32 = 20.0;
/// 点击穿透演示的全局热键 id (showcase 自有, 不用引擎 pomodoro 残留常量)。
const HOTKEY_CLICK_THROUGH: u8 = 1;
/// 点击穿透演示的热键主键: K (Virtual-Key 码)。
const HOTKEY_CLICK_THROUGH_VK: u32 = 0x4B;
/// 位置记忆演示的落点文件 (target/ 下, gitignore; 演示从简未做防抖)。
const POSITION_FILE: &str = "target/tmp/showcase-position.txt";

/// 分类导航：与 src/widget/ 子目录一一对应。
const CATEGORIES: [&str; 5] = [
    "基础 base",
    "布局 layout",
    "表单 form",
    "导航 nav",
    "视图 view",
];

/// showcase 应用 (状态容器 + 消息更新 + 视图树)。
struct Showcase {
    count: u32,
    square_pos: Point,
    last_key: String,
    input_value: String,
    icon_input_value: String,
    textarea_value: String,
    /// 当前选中的分类索引 (驱动 MultiPanel)。
    selected: usize,
    /// 窗口是否已最大化 (决定标题栏按钮图标 □/□□)。
    is_maximized: bool,
    /// 当前显示的图像 (RGBA 数据，宽，高)。
    image_data: Option<(Vec<u8>, u32, u32)>,
    /// Tabs 演示：当前选中的 tab 索引。
    selected_tab: usize,
    /// Switch 演示：是否启用通知。
    switch_enabled: bool,
    /// 点击穿透演示：当前是否处于穿透态。
    click_through: bool,
    /// 窗口置顶演示：当前是否置顶 (env DANQING_SHOWCASE_TOPMOST=1 出生即置顶,
    /// 供脚本验证创建路径的 WS_EX_TOPMOST 落位)。
    topmost: bool,
    /// 窗口事件发送器 (点击穿透演示用; run_app 启动时注入)。
    sender: Option<WindowEventSender>,
    /// 启动后待开穿透标记 (env DANQING_SHOWCASE_CLICK_THROUGH=1 触发,
    /// 首次显示回调里生效 —— 供截图验证 LAYERED×wgpu 呈现兼容性)。
    pending_click_through: bool,
    /// 时辰调色演示: None = 渐变背景原样; Some(hour) = 切到时辰演示场景
    /// 并按小时调色 (env DANQING_TOD_DEMO=19.0 可预置, 供脚本截图验证)。
    tod_hour: Option<f32>,
    /// 音频演示: 懒初始化播放器 (首次点播放才开设备; 无音频设备环境
    /// 静默降级不崩)。
    audio_player: Option<danqing::audio::AudioPlayer>,
    /// 微事件演示: 萤火虫/闪电包络的触发时刻 (世界时钟 elapsed; None = 未触发)。
    demo_firefly_at: Option<std::time::Duration>,
    /// 闪电演示触发时刻。
    demo_flash_at: Option<std::time::Duration>,
    /// 最近 tick 的世界时钟 (包络计算基准)。
    last_elapsed: std::time::Duration,
    /// 伸手仲裁演示: 最近一次手势协议消息 ("未按" / "已按住" / "已撤防")。
    reach_state: String,
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
    /// 图标输入框内容变化。
    IconInputChanged(String),
    /// 图标输入框图标点击。
    IconInputSearch,
    /// 多行文本域内容变化。
    TextareaChanged(String),
    /// 切换分类面板。
    Select(usize),
    /// Tabs 演示：切换 tab。
    TabChanged(usize),
    /// 打开本地图片。
    OpenImage,
    /// Switch 演示：切换开关状态。
    SwitchToggle,
    /// 点击穿透演示：切换穿透态 (Switch 与全局热键 Ctrl+Shift+K 双入口)。
    ClickThroughToggle,
    /// 窗口置顶演示：切换置顶层级。
    TopmostToggle,
    /// 时辰调色演示: 设置演示小时 (None = 复位渐变背景)。
    SetTod(Option<f32>),
    /// 音频演示: 播放 440Hz 正弦测试音 (2s)。
    PlayTestTone,
    /// 微事件演示: 萤火虫 (8s 包络, 自动切黄昏演示场景)。
    DemoFirefly,
    /// 微事件演示: 闪电 (1.6s 双闪脉冲)。
    DemoFlash,
    /// 伸手仲裁演示: 按下登记。
    ReachArm,
    /// 伸手仲裁演示: 撤防 (转拖拽/早抬起)。
    ReachCancel,
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
            Msg::IconInputChanged(s) => self.icon_input_value = s,
            Msg::IconInputSearch => {
                log::info!("搜索：{}", self.icon_input_value);
            }
            Msg::TextareaChanged(s) => self.textarea_value = s,
            Msg::Select(i) => self.selected = i,
            Msg::TabChanged(i) => self.selected_tab = i,
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
            Msg::SwitchToggle => self.switch_enabled = !self.switch_enabled,
            Msg::ClickThroughToggle => {
                self.click_through = !self.click_through;
                if let Some(sender) = &self.sender {
                    sender.set_click_through(self.click_through);
                }
            }
            Msg::TopmostToggle => {
                self.topmost = !self.topmost;
                if let Some(sender) = &self.sender {
                    sender.set_topmost(self.topmost);
                }
            }
            Msg::SetTod(hour) => self.tod_hour = hour,
            Msg::PlayTestTone => {
                use rodio::Source;
                let player = self.audio_player.get_or_insert_with(Default::default);
                let tone = rodio::source::SineWave::new(440.0)
                    .take_duration(std::time::Duration::from_secs(2));
                player.play_source(tone, 0.5);
            }
            Msg::DemoFirefly => {
                self.tod_hour = Some(19.0); // 萤火虫在黄昏演示场景上才可见
                self.demo_firefly_at = Some(self.last_elapsed);
            }
            Msg::DemoFlash => {
                self.tod_hour = Some(19.0);
                self.demo_flash_at = Some(self.last_elapsed);
            }
            Msg::ReachArm => self.reach_state = "已按住 (待产品长按判定)".into(),
            Msg::ReachCancel => self.reach_state = "已撤防 (转拖拽/早抬起)".into(),
        }
    }

    fn view(&self) -> Node {
        build_tree()
    }

    /// 时辰调色演示: 每帧产出背景状态 (场景 0 渐变 / 场景 1 时辰演示图)。
    /// tod_hour 为 None 时输出恒等帧 (与不设 background_frame 视觉一致)。
    fn background_frame(&self) -> Option<danqing::BackgroundFrame> {
        let frame = danqing::BackgroundFrame::new(0, 0, 0.0, theme().background());
        let Some(hour) = self.tod_hour else {
            return Some(frame);
        };
        let (tint, brightness, saturation, sky, glow) = tod_params(hour);
        let mut frame = danqing::BackgroundFrame::new(1, 1, 0.0, theme().background())
            .with_time_of_day(tint, brightness, saturation)
            .with_sky_amount(sky)
            .with_glow_amount(glow);
        // 微事件演示包络 (萤火虫 8s 淡入淡出 / 闪电双闪脉冲)。
        if let Some(t0) = self.demo_firefly_at {
            let dt = self.last_elapsed.saturating_sub(t0).as_secs_f32();
            frame = frame.with_event_firefly(firefly_envelope(dt));
        }
        if let Some(t0) = self.demo_flash_at {
            let dt = self.last_elapsed.saturating_sub(t0).as_secs_f32();
            frame = frame.with_flash(flash_pulse(dt));
        }
        Some(frame)
    }

    /// 微事件演示包络到期自清 (tick 驱动; background_frame 是 &self 不可变)。
    fn tick(&mut self, ctx: &danqing::AnimationCtx) {
        self.last_elapsed = ctx.elapsed;
        if let Some(t0) = self.demo_firefly_at {
            if ctx.elapsed.saturating_sub(t0).as_secs_f32() > 8.0 {
                self.demo_firefly_at = None;
            }
        }
        if let Some(t0) = self.demo_flash_at {
            if ctx.elapsed.saturating_sub(t0).as_secs_f32() > 1.6 {
                self.demo_flash_at = None;
            }
        }
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

    fn attach_window_sender(&mut self, sender: WindowEventSender) {
        self.sender = Some(sender);
    }

    fn visibility_changed(&mut self, visible: bool) {
        // env 触发 (DANQING_SHOWCASE_CLICK_THROUGH=1): 首次显示后自动开穿透。
        // 窗口创建前发送会被丢弃, 故挂在首次可见回调上。
        if visible && self.pending_click_through {
            self.pending_click_through = false;
            self.update(Msg::ClickThroughToggle);
        }
    }

    fn hotkey(&mut self, id: u8) -> Option<Msg> {
        (id == HOTKEY_CLICK_THROUGH).then_some(Msg::ClickThroughToggle)
    }

    /// 位置记忆演示 (ShowPlacement::Remember): 从落点文件恢复。
    fn load_window_position(&self) -> Option<(i32, i32)> {
        let text = std::fs::read_to_string(POSITION_FILE).ok()?;
        let (x, y) = text.trim().split_once(',')?;
        Some((x.parse().ok()?, y.parse().ok()?))
    }

    /// 位置记忆演示: 拖动即写文件 (演示从简未防抖; 产品侧应防抖落盘)。
    fn save_window_position(&mut self, x: i32, y: i32) {
        let _ = std::fs::create_dir_all("target/tmp");
        let _ = std::fs::write(POSITION_FILE, format!("{x},{y}"));
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
        .cross_center()
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
        // CloseButton: 矢量 × 按钮 (点击清零计数，hover 出底色)。
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
        .cross_center()
        .child(
            Row::new()
                .gap(2.0)
                .cross_center()
                .child(
                    Text::new("输入：")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    TextInput::themed(t)
                        .width(240.0)
                        .on_change(|s: &str| Msg::InputChanged(s.to_string())),
                ),
        )
        .child(
            Text::bind(|s: &Showcase| format!("已输入：{}", s.input_value))
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
}

/// 图标输入区:IconInput + 实时回显 + 图标点击搜索。
fn icon_input_row(t: &LightTheme) -> impl Widget + 'static {
    Row::new()
        .gap(t.spacing_lg())
        .cross_center()
        .child(
            Row::new()
                .gap(2.0)
                .cross_center()
                .child(
                    Text::new("搜索：")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    IconInput::themed(t)
                        .width(280.0)
                        .placeholder("输入关键词...", t.text_secondary())
                        .on_change(|s: &str| Msg::IconInputChanged(s.to_string()))
                        .on_icon_click(|| Msg::IconInputSearch),
                ),
        )
        .child(
            Text::bind(|s: &Showcase| {
                if s.icon_input_value.is_empty() {
                    "点击右侧图标搜索".to_string()
                } else {
                    format!("搜索：{}", s.icon_input_value)
                }
            })
            .font_size(t.font_size_body())
            .color(t.text_primary()),
        )
}

/// 多行输入区:Scrollable + TextArea + 实时回显字数 / 行数。
fn textarea_card(t: &LightTheme) -> impl Widget + 'static {
    // 与单行输入框等高
    let label_height = t.control_height();
    Row::new()
        .gap(t.spacing_lg())
        .child(
            // label + TextArea 紧凑排列 (3px 间距)
            Row::new()
                .gap(2.0)
                .child(
                    // label 固定为单行输入框高度，内部 Center 使文本在其自身高度内垂直居中;
                    // 不用 cross_center —— 那会把 label 居中到多行 TextArea 的完整高度。
                    UiBox::new(Color::TRANSPARENT)
                        .height(label_height)
                        .child(widget::Center::new(
                            Text::new("多行：")
                                .font_size(t.font_size_body())
                                .color(t.text_primary()),
                        )),
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
                ),
        )
        .child(
            // 与 label 等高，文本在 control_height 内垂直居中
            UiBox::new(Color::TRANSPARENT)
                .height(label_height)
                .child(widget::Center::new(
                    Text::bind(|s: &Showcase| {
                        let chars = s.textarea_value.chars().count();
                        let lines = s.textarea_value.lines().count();
                        format!("字数：{} 行数：{}", chars, lines)
                    })
                    .font_size(t.font_size_body())
                    .color(t.text_primary()),
                )),
        )
}

/// 滑动开关区：Switch 组件演示。
fn switch_card(t: &LightTheme) -> impl Widget + 'static {
    Row::new()
        .gap(t.spacing_lg())
        .cross_center()
        .child(
            Row::new()
                .gap(2.0)
                .cross_center()
                .child(
                    Text::new("通知：")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    Switch::new()
                        .bind(|s: &Showcase| s.switch_enabled)
                        .on_toggle(|| Msg::SwitchToggle),
                ),
        )
        .child(
            Text::bind(|s: &Showcase| {
                if s.switch_enabled {
                    "已开启".to_string()
                } else {
                    "已关闭".to_string()
                }
            })
            .font_size(t.font_size_body())
            .color(t.text_primary()),
        )
}

/// 点击穿透区：窗口行为演示 (desk-window 模块)。
/// 开启后鼠标事件直达下层窗口, 点本窗口无效 —— 切回用全局热键 Ctrl+Shift+K。
fn passthrough_card(t: &LightTheme) -> impl Widget + 'static {
    Row::new()
        .gap(t.spacing_lg())
        .cross_center()
        .child(
            Row::new()
                .gap(2.0)
                .cross_center()
                .child(
                    Text::new("点击穿透：")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    Switch::new()
                        .bind(|s: &Showcase| s.click_through)
                        .on_toggle(|| Msg::ClickThroughToggle),
                ),
        )
        .child(
            Text::bind(|s: &Showcase| {
                if s.click_through {
                    "已开启 —— 点我无效, 按 Ctrl+Shift+K 切回".to_string()
                } else {
                    "已关闭 (或按 Ctrl+Shift+K 开启)".to_string()
                }
            })
            .font_size(t.font_size_body())
            .color(t.text_primary()),
        )
}

/// 窗口置顶区：窗口行为演示 (desk-window 模块)。
/// 开启后窗口恒在普通窗口之上; 关闭后回到普通层级。
fn topmost_card(t: &LightTheme) -> impl Widget + 'static {
    Row::new()
        .gap(t.spacing_lg())
        .cross_center()
        .child(
            Row::new()
                .gap(2.0)
                .cross_center()
                .child(
                    Text::new("窗口置顶：")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    Switch::new()
                        .bind(|s: &Showcase| s.topmost)
                        .on_toggle(|| Msg::TopmostToggle),
                ),
        )
        .child(
            Text::bind(|s: &Showcase| {
                if s.topmost {
                    "已置顶 —— 普通窗口压不住我".to_string()
                } else {
                    "普通层级".to_string()
                }
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
///
/// 按钮 + 信息文本由 Row 框架管理; Image 因动态替换仍手动布局。
struct ImageDemo {
    /// 第一行：按钮 + 图片尺寸信息 (框架管理 sync/animate/layout/paint/event)。
    header: Node,
    /// 图片组件 (动态数据，手动管理)。
    image: widget::Image,
    /// header 布局尺寸缓存。
    header_size: Size,
}

impl ImageDemo {
    fn new() -> Self {
        let t = theme();
        let (data, width, height) = load_logo();
        let header = Row::new()
            .gap(t.spacing_sm())
            .cross_center()
            .child(
                Button::themed(
                    &t,
                    Text::new("打开图片")
                        .font_size(t.font_size_body())
                        .color(Color::WHITE),
                )
                .on_click(|| Msg::OpenImage),
            )
            .child(
                Text::bind(|s: &Showcase| match &s.image_data {
                    Some((_, w, h)) => format!("{w}×{h} px"),
                    None => String::new(),
                })
                .font_size(t.font_size_small())
                .color(Color::rgba(0.6, 0.6, 0.6, 1.0)),
            );
        Self {
            header: Box::new(header),
            image: widget::Image::new(data, width, height),
            header_size: Size::ZERO,
        }
    }
}

impl Widget for ImageDemo {
    fn sync(&mut self, state: &dyn std::any::Any) {
        self.header.sync(state);
        let state = state
            .downcast_ref::<Showcase>()
            .expect("ImageDemo 绑定状态类型不匹配");
        if let Some((data, w, h)) = &state.image_data {
            self.image = widget::Image::new(data.clone(), *w, *h);
        }
    }

    fn animate(&mut self, ctx: &danqing::AnimationCtx) {
        self.header.animate(ctx);
    }

    fn layout(
        &mut self,
        constraints: danqing::Constraints,
        texts: &mut danqing::TextBatch,
    ) -> Size {
        self.header_size = self.header.layout(constraints, texts);
        let gap = 8.0;
        let image_size = self.image.layout(
            danqing::Constraints::loose(Size::new(
                constraints.max_width,
                constraints.max_height - self.header_size.height - gap,
            )),
            texts,
        );
        Size::new(
            image_size.width.max(self.header_size.width),
            self.header_size.height + gap + image_size.height,
        )
    }

    fn paint(&self, area: Rect, rects: &mut danqing::RectBatch, texts: &mut danqing::TextBatch) {
        self.header.paint(area, rects, texts);
        let gap = 8.0;
        let image_area = Rect::from_xywh(
            area.origin.x,
            area.origin.y + self.header_size.height + gap,
            area.size.width,
            area.size.height - self.header_size.height - gap,
        );
        self.image.paint(image_area, rects, texts);
    }

    fn paint_image(&self, area: Rect, images: &mut danqing::ImageBatch) {
        let gap = 8.0;
        let image_area = Rect::from_xywh(
            area.origin.x,
            area.origin.y + self.header_size.height + gap,
            area.size.width,
            area.size.height - self.header_size.height - gap,
        );
        self.image.paint_image(image_area, images);
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.header.event(event, area, msgs)
    }

    fn children(&self) -> &[Node] {
        self.header.children()
    }

    fn children_mut(&mut self) -> &mut [Node] {
        self.header.children_mut()
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
        Column::new()
            .gap(t.spacing_lg())
            .cross_stretch()
            .child(card(t, "品牌色与圆角", palette_and_rounded_card(t)))
            .child(card(t, "DragArea 拖拽层", drag_area_card(t)))
            .child(card(t, "时辰调色 + 双蒙版", tod_card(t)))
            .child(card(t, "音频 (audio)", audio_card(t)))
            .child(card(t, "伸手仲裁 ReachArea", reach_area_card(t))),
    )
}

/// 音频演示: danqing::audio 输出路径端到端 (440Hz 正弦 2s, 免资产)。
fn audio_card(t: &LightTheme) -> impl Widget + 'static {
    Button::themed(
        t,
        Text::new("播放测试音 (440Hz × 2s)")
            .font_size(t.font_size_body())
            .color(Color::WHITE),
    )
    .on_click(|| Msg::PlayTestTone)
}

/// 萤火虫演示包络 (8s): 1.5s 淡入 → 保持 → 1.5s 淡出。
fn firefly_envelope(t: f32) -> f32 {
    let ss = |x: f32| {
        let t = x.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    ss(t / 1.5) * (1.0 - ss((t - 6.5) / 1.5))
}

/// 闪电双闪脉冲 (1.6s): 主闪快衰 → 间隙微光 → 次闪较弱 → 灭。
/// 形状是产品口味 (桌景闪电同款语义), 引擎只收强度。
fn flash_pulse(t: f32) -> f32 {
    if t < 0.0 {
        0.0
    } else if t < 0.12 {
        1.0 - t / 0.12 * 0.55
    } else if t < 0.22 {
        0.15
    } else if t < 0.5 {
        0.65 * (1.0 - (t - 0.22) / 0.28)
    } else {
        0.0
    }
}

/// 时辰演示迷你曲线 (演示级 6 帧线性插值; 产品级 8 帧 smoothstep
/// 曲线归 danqing-deskscape scene-world Task 5)。
/// 返回 (色调 RGB, 亮度, 饱和度, 天空蒙版量, 发光蒙版量)。
fn tod_params(hour: f32) -> ([f32; 3], f32, f32, f32, f32) {
    // (时辰, 色调, 亮度, 饱和度, 天空量, 发光量)
    const KEYS: [(f32, [f32; 3], f32, f32, f32, f32); 6] = [
        (0.0, [0.70, 0.78, 1.00], 0.50, 0.72, 0.95, 1.00), // 深夜
        (6.0, [1.00, 0.88, 0.72], 0.92, 0.90, 0.25, 0.20), // 清晨
        (12.0, [1.00, 1.00, 1.00], 1.08, 1.02, 0.00, 0.00), // 正午
        (17.0, [1.00, 0.90, 0.70], 1.00, 0.98, 0.05, 0.05), // 金时
        (19.0, [0.98, 0.72, 0.52], 0.82, 0.90, 0.45, 0.60), // 黄昏
        (21.0, [0.80, 0.82, 1.00], 0.62, 0.80, 0.80, 1.00), // 入夜
    ];
    let h = hour.rem_euclid(24.0);
    // 找 h 所在的关键帧区间 (环形: 21 点之后回绕到次日 0 点)。
    let mut i = KEYS.len() - 1;
    for (k, key) in KEYS.iter().enumerate() {
        if h >= key.0 {
            i = k;
        }
    }
    let a = KEYS[i];
    let b = KEYS[(i + 1) % KEYS.len()];
    let span = (b.0 - a.0).rem_euclid(24.0).max(0.001);
    let t = ((h - a.0).rem_euclid(24.0) / span).clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    (
        [
            lerp(a.1[0], b.1[0]),
            lerp(a.1[1], b.1[1]),
            lerp(a.1[2], b.1[2]),
        ],
        lerp(a.2, b.2),
        lerp(a.3, b.3),
        lerp(a.4, b.4),
        lerp(a.5, b.5),
    )
}

/// 时辰调色 + 双蒙版演示卡: 按钮切演示小时 (窗口背景即画布;
/// env DANQING_TOD_DEMO=19.0 可预置, 供脚本截图验证)。
fn tod_card(t: &LightTheme) -> impl Widget + 'static {
    let btn = |label: &'static str, hour: Option<f32>| {
        Button::themed(
            t,
            Text::new(label)
                .font_size(t.font_size_body())
                .color(Color::WHITE),
        )
        .on_click(move || Msg::SetTod(hour))
    };
    Row::new()
        .gap(t.spacing_sm())
        .cross_center()
        .child(
            Text::new("背景时辰 →")
                .font_size(t.font_size_body())
                .color(t.text_secondary()),
        )
        .child(btn("清晨", Some(7.0)))
        .child(btn("正午", Some(12.0)))
        .child(btn("黄昏", Some(19.0)))
        .child(btn("深夜", Some(23.0)))
        .child(btn("复位", None))
        .child(
            Text::new("微事件 →")
                .font_size(t.font_size_body())
                .color(t.text_secondary()),
        )
        .child(
            Button::themed(
                t,
                Text::new("萤火虫")
                    .font_size(t.font_size_body())
                    .color(Color::WHITE),
            )
            .on_click(|| Msg::DemoFirefly),
        )
        .child(
            Button::themed(
                t,
                Text::new("闪电")
                    .font_size(t.font_size_body())
                    .color(Color::WHITE),
            )
            .on_click(|| Msg::DemoFlash),
        )
}

/// ReachArea 演示: 伸手手势的空间仲裁协议 (arm/cancel) ——
/// 长按 600ms 的时间判定在产品 tick (引擎 widget 无周期消息通道)。
fn reach_area_card(t: &LightTheme) -> impl Widget + 'static {
    Column::new()
        .gap(t.spacing_sm())
        .child(
            Text::bind(|s: &Showcase| format!("手势状态: {}", s.reach_state))
                .font_size(t.font_size_body())
                .color(t.text_primary()),
        )
        .child(
            ReachArea::new(
                Text::new("按住我: 微抖=保持, 拖动=转拖拽移窗, 早抬=撤防")
                    .font_size(t.font_size_body())
                    .color(t.text_secondary()),
            )
            .on_arm(|_| Msg::ReachArm)
            .on_cancel(|| Msg::ReachCancel),
        )
}

/// DragArea 演示: 无边框窗口的背景拖拽层 —— 按住卡片内容区空白
/// 左键拖动即移动整个窗口 (消息经 WindowAction::Drag 到 Handler)。
fn drag_area_card(t: &LightTheme) -> impl Widget + 'static {
    DragArea::new(Padding::all(
        t.spacing_md(),
        Text::new("按住本卡片空白拖动 → 移动整个窗口")
            .font_size(t.font_size_body())
            .color(t.text_secondary()),
    ))
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
            .child(card(t, "图标输入", icon_input_row(t)))
            .child(card(t, "多行输入", textarea_card(t)))
            .child(card(t, "滑动开关", switch_card(t))),
    )
}

/// 导航页：Tabs 多面板切换。
fn page_nav(t: &LightTheme) -> impl Widget + 'static {
    page(
        t,
        "导航 nav — Tabs 多面板切换",
        Column::new()
            .gap(t.spacing_lg())
            .cross_stretch()
            .child(card(t, "Tabs 组件 (多面板切换)", tabs_card(t))),
    )
}

/// 视图页：视口与可见性; 自定义组件演示。
fn page_view(t: &LightTheme) -> impl Widget + 'static {
    page(
        t,
        "视图 view — 自定义组件演示",
        Column::new()
            .gap(t.spacing_lg())
            .cross_stretch()
            .child(card(
                t,
                "键盘响应 (自定义 Positioned 组件)",
                keyboard_card(t),
            ))
            .child(card(
                t,
                "点击穿透 (窗口行为, 热键 Ctrl+Shift+K)",
                passthrough_card(t),
            ))
            .child(card(t, "窗口置顶 (窗口行为)", topmost_card(t))),
    )
}

/// Tabs 演示：三个 tab 切换不同内容。
fn tabs_card(t: &LightTheme) -> impl Widget + 'static {
    Tabs::new(t)
        .tab("概览")
        .tab("设置")
        .tab("关于")
        .child(
            Column::new()
                .gap(t.spacing_md())
                .child(
                    Text::new("这是概览面板")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    Text::new("Tabs 组件演示：点击上方 tab 切换面板内容")
                        .font_size(t.font_size_small())
                        .color(t.text_secondary()),
                ),
        )
        .child(
            Column::new()
                .gap(t.spacing_md())
                .child(
                    Text::new("设置面板")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    Text::new("每个面板独立持有组件状态")
                        .font_size(t.font_size_small())
                        .color(t.text_secondary()),
                ),
        )
        .child(
            Column::new()
                .gap(t.spacing_md())
                .child(
                    Text::new("关于")
                        .font_size(t.font_size_body())
                        .color(t.text_primary()),
                )
                .child(
                    Text::new("danqing 丹青 UI 框架")
                        .font_size(t.font_size_small())
                        .color(t.text_secondary()),
                ),
        )
        .on_change(Msg::TabChanged)
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
                    // 分类面板：四个页面常驻实例化，MultiPanel 只切换可见性。
                    .fill(
                        MultiPanel::new()
                            .child(page_base(&t))
                            .child(page_layout(&t))
                            .child(page_form(&t))
                            .child(page_nav(&t))
                            .child(page_view(&t))
                            .bind(|s: &Showcase| s.selected),
                        1,
                    ),
                1,
            ),
    )
}

fn main() -> anyhow::Result<()> {
    danqing::log::init_log();

    // env DANQING_SHOWCASE_TOPMOST=1: 出生即置顶 (验证创建路径 WS_EX_TOPMOST 落位)。
    let topmost_at_boot = std::env::var_os("DANQING_SHOWCASE_TOPMOST").is_some();
    let mut app = Showcase {
        count: 0,
        square_pos: Point::ZERO,
        last_key: String::from("-"),
        input_value: String::new(),
        icon_input_value: String::new(),
        textarea_value: String::new(),
        selected: 0,
        is_maximized: false,
        image_data: None,
        selected_tab: 0,
        switch_enabled: false,
        click_through: false,
        topmost: topmost_at_boot,
        sender: None,
        pending_click_through: std::env::var_os("DANQING_SHOWCASE_CLICK_THROUGH").is_some(),
        // env DANQING_TOD_DEMO=19.0: 预置时辰演示小时 (脚本截图验证用,
        // 合成鼠标点击不到达组件 —— 见 danqing-visual-debug-tooling 记忆)。
        tod_hour: std::env::var("DANQING_TOD_DEMO")
            .ok()
            .and_then(|v| v.parse::<f32>().ok()),
        audio_player: None,
        // 微事件演示: env DANQING_EVENT_DEMO=firefly|flash 启动即触发
        // (脚本截图验证用, 同 DANQING_TOD_DEMO 的注入理由)。
        demo_firefly_at: match std::env::var("DANQING_EVENT_DEMO").as_deref() {
            Ok("firefly") => Some(std::time::Duration::ZERO),
            _ => None,
        },
        demo_flash_at: match std::env::var("DANQING_EVENT_DEMO").as_deref() {
            Ok("flash") => Some(std::time::Duration::ZERO),
            _ => None,
        },
        last_elapsed: std::time::Duration::ZERO,
        reach_state: "未按".into(),
    };

    let t = theme();
    // 场景 0 = 渐变 (默认原样) / 场景 1 = 时辰演示图 (tod-demo, 中性日光底
    // + 天空/发光双蒙版); 蒙版未点亮时 (amount=0) 对渐变场景零影响。
    let background = BackgroundConfig::with_scenes([
        "assets/background/gradient.png",
        "assets/background/tod-demo.png",
    ])
    .scale(ScaleMode::Cover)
    .with_glow("assets/background/glow.png", 0.25)
    .with_noise("assets/background/noise.png", 0.06)
    .with_sky_mask("assets/background/tod-demo-sky.png")
    .with_glow_mask("assets/background/tod-demo-glow.png");
    let config = danqing::WindowConfig {
        title: "danqing showcase".into(),
        clear_color: t.background(),
        background,
        topmost: topmost_at_boot,
        // 位置记忆演示: 记住上次拖到的位置, 重启复原 (落点文件 target/tmp/)。
        placement: danqing::ShowPlacement::Remember,
        // 点击穿透演示的热键 (覆盖默认的番茄钟语义热键 —— showcase 本就不用它们,
        // 覆盖后不再白白全局吞掉 Ctrl+Shift+P/S/Q)。
        hotkeys: vec![GlobalHotkey::ctrl_shift(
            HOTKEY_CLICK_THROUGH,
            HOTKEY_CLICK_THROUGH_VK,
        )],
        ..danqing::WindowConfig::default()
    };
    danqing::run_app(config, &mut app)?;
    Ok(())
}
