//! @author 十四叔
//! @date 2026/07/23

//! 丹青番茄钟 POC —— 专注陪伴工具 × 场景沉浸。
//!
//! 最小番茄钟 (固定 25/5, 开始/暂停/重置) + 场景沉浸：
//! 场景大图为主角，中央大字倒计时，底部玻璃胶囊控件条，
//! 场景 前/后 切换带 800ms 交叉淡化，色调随场景调色板流动。

mod fader;
mod scenes;
mod timer;

#[path = "../common/log.rs"]
mod example_log;

use std::time::Duration;

use danqing::widget::{
    self, Box as UiBox, Button, Center, Column, Node, Padding, Row, Text, TitleBar,
};
use danqing::{
    AnimationCtx, App, BackgroundConfig, BackgroundFrame, Color, Easing, ScaleMode, ScenePalette,
    SceneTheme, Size, Theme, WindowAction, WindowConfig,
};
use fader::SceneFader;
use scenes::SCENES;
use timer::Pomodoro;

/// 全局噪声叠加 (防抖带颗粒，复用阶段 1 资产)。
const NOISE: &str = "assets/background/noise.png";
/// 噪声叠加不透明度。
const NOISE_OPACITY: f32 = 0.06;
/// 场景交叉淡化时长 (spec: 600~1000ms)。
const FADE_DURATION: Duration = Duration::from_millis(800);

/// 淡化缓动曲线 (淡入淡出两端柔和)。
const FADE_EASING: Easing = Easing::EaseInOut;

/// 番茄钟应用状态。
struct PomodoroApp {
    /// 计时状态机 (纯逻辑)。
    timer: Pomodoro,
    /// 注入时间轴：自应用启动的累计时间 (由 tick 心跳推进)。
    now: Duration,
    /// 场景交叉淡化器 (含当前场景索引)。
    fader: SceneFader,
}

/// 应用消息。
#[derive(Clone, Copy)]
enum Msg {
    /// 开始 / 暂停切换。
    StartPause,
    /// 重置回专注 25:00 停止态。
    Reset,
    /// 上一个场景。
    PrevScene,
    /// 下一个场景。
    NextScene,
}

impl PomodoroApp {
    /// 当前视觉调色板：淡化中为两端调色板的插值 (色调随画面同步流动)。
    fn palette(&self) -> ScenePalette {
        let (from, to, t) = self.fader.frame(self.now, |t| FADE_EASING.eval(t));
        SCENES[from].palette.lerp(SCENES[to].palette, t)
    }

    /// 当前场景主题 (颜色 token 随调色板流动)。
    fn theme(&self) -> SceneTheme {
        SceneTheme::new(self.palette())
    }
}

impl App for PomodoroApp {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        match msg {
            Msg::StartPause => self.timer.toggle(self.now),
            Msg::Reset => self.timer.reset(),
            Msg::PrevScene => {
                let target = (self.fader.current() + SCENES.len() - 1) % SCENES.len();
                self.fader.switch_to(target, self.now);
            }
            Msg::NextScene => {
                let target = (self.fader.current() + 1) % SCENES.len();
                self.fader.switch_to(target, self.now);
            }
        }
    }

    fn view(&self) -> Node {
        let t = self.theme();
        widget::node(
            Column::new()
                .cross_stretch()
                .child(
                    TitleBar::themed(&t, "丹青 · 番茄钟")
                        .bind_theme(|s: &PomodoroApp| s.theme())
                        .on_close(|| WindowAction::Close)
                        .on_minimize(|| WindowAction::Minimize)
                        .on_maximize(|| WindowAction::MaximizeOrRestore)
                        .on_drag(|| WindowAction::Drag),
                )
                .fill(Center::new(countdown_block(t)).fill_max(), 1)
                .child(Padding::all(t.spacing_xl(), Center::new(control_pill(t)))),
        )
    }

    fn tick(&mut self, ctx: &AnimationCtx) {
        self.now = ctx.elapsed;
        self.timer.tick(ctx.elapsed);
    }

    fn background_frame(&self) -> Option<BackgroundFrame> {
        let (from, to, fade) = self.fader.frame(self.now, |t| FADE_EASING.eval(t));
        Some(BackgroundFrame::new(from, to, fade, self.palette().base))
    }
}

/// 中央倒计时块：大字倒计时 + 阶段/场景标注。
fn countdown_block(t: SceneTheme) -> impl widget::Widget {
    Column::new()
        .cross_stretch()
        .child(Center::new(
            Text::bind(|s: &PomodoroApp| s.timer.display(s.now))
                .font_size(t.font_size_display())
                .bind_color(|s: &PomodoroApp| s.palette().text_primary),
        ))
        .child(Center::new(
            Text::bind(|s: &PomodoroApp| {
                format!(
                    "{} · {}",
                    s.timer.phase().label(),
                    SCENES[s.fader.current()].name
                )
            })
            .font_size(t.font_size_body())
            .bind_color(|s: &PomodoroApp| s.palette().text_secondary),
        ))
}

/// 主操作按钮 (开始/暂停): accent 底 + 场景基调色文字 (同场景色对，对比天然成立)。
fn primary_button(t: SceneTheme) -> Button {
    Button::themed(
        &t,
        Text::bind(|s: &PomodoroApp| {
            if s.timer.is_running() {
                "暂停".into()
            } else {
                "开始".into()
            }
        })
        .bind_color(|s: &PomodoroApp| s.palette().base),
    )
    .bind_color(|s: &PomodoroApp| s.palette().accent)
    .on_click(|| Msg::StartPause)
}

/// 幽灵按钮 (重置/场景切换): 透明底，悬停浮现玻璃，文字随场景。
fn ghost_button(t: SceneTheme, label: &'static str, msg: Msg) -> Button {
    Button::themed(
        &t,
        Text::new(label).bind_color(|s: &PomodoroApp| s.palette().text_primary),
    )
    .bind_color(|_: &PomodoroApp| Color::TRANSPARENT)
    .bind_hover_color(|s: &PomodoroApp| s.palette().surface)
    .bind_focus_color(|s: &PomodoroApp| s.palette().accent)
    .on_click(move || msg)
}

/// 底部玻璃胶囊控件条。
fn control_pill(t: SceneTheme) -> impl widget::Widget {
    UiBox::new(Color::TRANSPARENT)
        .bind_color(|s: &PomodoroApp| s.palette().surface)
        .radius(28.0)
        .child(Padding::new(
            danqing::Edges::symmetric(t.spacing_sm(), t.spacing_xs()),
            Row::new()
                .gap(t.spacing_xs())
                .child(ghost_button(t, "前", Msg::PrevScene))
                .child(primary_button(t))
                .child(ghost_button(t, "重置", Msg::Reset))
                .child(ghost_button(t, "后", Msg::NextScene)),
        ))
}

fn main() -> anyhow::Result<()> {
    example_log::init_log();

    let mut app = PomodoroApp {
        timer: Pomodoro::new(),
        now: Duration::ZERO,
        fader: SceneFader::new(0, FADE_DURATION),
    };

    let background = BackgroundConfig::with_scenes(SCENES.iter().map(|s| s.image))
        .scale(ScaleMode::Cover)
        .with_noise(NOISE, NOISE_OPACITY);
    let config = WindowConfig {
        title: "丹青 · 番茄钟".into(),
        size: Size::new(960.0, 640.0),
        clear_color: SCENES[0].palette.base,
        background,
        ..WindowConfig::default()
    };
    danqing::run_app(config, &mut app)?;
    Ok(())
}
