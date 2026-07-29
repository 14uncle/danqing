//! @author 十四叔
//! @date 2026/07/23

//! 丹青番茄钟 POC —— 专注陪伴工具 × 场景沉浸。
//!
//! 最小番茄钟 (固定 25/5, 开始/暂停/重置) + 场景沉浸：
//! 场景大图为主角，中央大字倒计时，底部玻璃胶囊控件条，
//! 场景 前/后 切换带 800ms 交叉淡化，色调随场景调色板流动。

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod ambient;
mod audio;
mod fader;
mod flash;
mod hint;
mod motion;
mod scenes;
mod state;
mod timer;
mod today;
mod tray;

#[path = "../common/log.rs"]
mod example_log;

use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use danqing::widget::{
    self, Box as UiBox, Button, Center, Column, Node, Padding, Row, Stack, Text, TitleBar,
};
use danqing::{
    AnimationCtx, App, BackgroundConfig, BackgroundFrame, Color, Easing, Edges, LightTheme,
    ScaleMode, ScenePalette, SceneTheme, Size, Theme, WindowAction, WindowConfig,
    WindowEventSender, hotkey_ids, shortcut_for_id, tray_action_ids,
};
use fader::SceneFader;
use flash::FlashOverlay;
use hint::ShortcutHintOverlay;
use scenes::SCENES;
use state::{PomodoroState, RunState, load_state, save_state};
use timer::{Phase, Pomodoro, Run};
use tray::build_menu;

/// 完成反馈视觉脉冲时长 (头部满 → 尾部透明)。
const FLASH_DURATION: Duration = Duration::from_millis(600);

/// 全局噪声叠加 (防抖带颗粒，复用阶段 1 资产)。
const NOISE: &str = "assets/background/noise.png";
/// 噪声叠加不透明度。
const NOISE_OPACITY: f32 = 0.06;
/// 场景交叉淡化时长 (spec: 600~1000ms)。
const FADE_DURATION: Duration = Duration::from_millis(800);
/// 持久化节流间隔：state_dirty 为 true 时，距上次保存超过此间隔才落盘。
const SAVE_THROTTLE: Duration = Duration::from_secs(1);

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
    /// 启动时 elapsed 偏移 (持久化恢复); 0 表示全新会话。
    now_offset: Duration,
    /// 状态脏旗标：update 触发，tick 节流落盘后清零。
    state_dirty: bool,
    /// 最近一次成功落盘的 now 值 (节流基准)。
    last_save_at: Duration,
    /// 最近一次跨日检查的 now 值 (1Hz 节流基准)。
    last_date_check: Duration,
    /// 完成反馈视觉脉冲 (阶段流转触发)。
    flash: FlashOverlay,
    /// 首次启动快捷键提示 (一过性 fade-in/hold/fade-out 状态机)。
    hint: ShortcutHintOverlay,
    /// 当前持久化的「已见过快捷键提示」旗标 (snapshot_state 直接读)。
    has_seen_shortcut_hint: bool,
    /// 今日计数所属日期 (YYYY-MM-DD, 与 today_count 配对)。
    today_date: String,
    /// 今日已自然完成的专注数 (skip 不计，跨日归零)。
    today_count: u32,
    /// 环境音混音器 (纯逻辑: 淡化权重 × 暂停沉降包络)。
    ambient_mixer: ambient::AmbientMixer,
    /// 环境音播放器 (rodio 适配层: 懒初始化 + 双槽 + 静默降级)。
    ambient_player: ambient::AmbientPlayer,
    /// 场景动效沉降包络 (纯逻辑: 暂停 500ms 淡出 / 恢复淡入)。
    motion_envelope: motion::MotionEnvelope,
    /// 最近 tick 算出的动效包络值 (`background_frame` 只读)。
    motion_gain: f32,
    /// 雨钟 (秒): 雨丝下落时间轴。暂停时定格可见 (不随包络沉降),
    /// 包络只推进本钟 — 暂停 500ms 减速冻结, 恢复 500ms 加速续走。
    rain_clock: f32,
    /// 窗口事件发送器 (run_app 启动时注入，App 借此控制窗口显隐 / 退出)。
    window_sender: Option<WindowEventSender>,
}

/// 应用消息。
#[derive(Clone, Copy)]
enum Msg {
    /// 开始 / 暂停切换。
    StartPause,
    /// 跳过当前阶段，进入下一阶段。
    Skip,
    /// 重置回专注 25:00 停止态。
    Reset,
    /// 上一个场景。
    PrevScene,
    /// 下一个场景。
    NextScene,
    /// 切换窗口可见性 (全局热键 Ctrl+Shift+P)。
    ToggleVisible,
    /// 退出应用 (全局热键 Ctrl+Shift+Q)。
    Quit,
}

impl PomodoroApp {
    /// 默认会话构造：25:00 Focus Idle, 场景 0, 全部偏移为 0。
    fn new_default() -> Self {
        Self {
            timer: Pomodoro::new(),
            now: Duration::ZERO,
            fader: SceneFader::new(0, FADE_DURATION),
            now_offset: Duration::ZERO,
            state_dirty: true,
            last_save_at: Duration::ZERO,
            last_date_check: Duration::ZERO,
            flash: FlashOverlay::new(FLASH_DURATION),
            // 全新会话：触发一次性快捷键提示，同时标记为已见 (节流落盘后 JSON 持久化)。
            hint: ShortcutHintOverlay::triggered_at(Duration::ZERO),
            has_seen_shortcut_hint: true,
            today_date: today::today_string(),
            today_count: 0,
            ambient_mixer: ambient::AmbientMixer::new(),
            ambient_player: ambient::AmbientPlayer::new(),
            motion_envelope: motion::MotionEnvelope::new(),
            motion_gain: 0.0,
            rain_clock: 0.0,
            window_sender: None,
        }
    }

    /// 从持久化状态恢复：设置 timer / 场景 / now_offset,
    /// 状态保持 dirty 以确保一次重写。
    fn from_state(state: PomodoroState) -> Self {
        let now_offset = state.effective_now_offset();
        let run: Run = state.run.into();
        let remaining = Duration::from_secs(state.remaining_secs);
        let deadline = if matches!(run, Run::Running) {
            Some(now_offset + remaining)
        } else {
            None
        };
        let timer = Pomodoro::restore(state.phase, run, remaining, deadline, state.completed_focus);
        let fader = if state.current_scene < SCENES.len() {
            SceneFader::new(state.current_scene, FADE_DURATION)
        } else {
            SceneFader::new(0, FADE_DURATION)
        };
        // 一次性快捷键提示：没看过就触发一次，触发即标记为已见。
        let should_show_hint = !state.has_seen_shortcut_hint;
        // 今日计数：跨日归零恢复 (空串/过期日期一律归零)。
        let today = today::today_string();
        let today_count = today::resolve_today_count(&state.today_date, state.today_count, &today);
        Self {
            timer,
            now: now_offset,
            fader,
            now_offset,
            state_dirty: true,
            last_save_at: now_offset,
            last_date_check: now_offset,
            flash: FlashOverlay::new(FLASH_DURATION),
            hint: if should_show_hint {
                ShortcutHintOverlay::triggered_at(now_offset)
            } else {
                ShortcutHintOverlay::idle()
            },
            has_seen_shortcut_hint: true,
            today_date: today,
            today_count,
            ambient_mixer: ambient::AmbientMixer::new(),
            ambient_player: ambient::AmbientPlayer::new(),
            motion_envelope: motion::MotionEnvelope::new(),
            motion_gain: 0.0,
            rain_clock: 0.0,
            window_sender: None,
        }
    }

    /// 立即落盘 (退出/异常时调用，不走节流)。
    /// 失败不 panic: 进程即将退出，错误仅供日志，重试窗口已无。
    fn flush(&mut self) {
        match save_state(&self.snapshot_state()) {
            Ok(()) => {
                self.state_dirty = false;
                self.last_save_at = self.now;
            }
            Err(err) => log::warn!("flush 状态失败：{err}"),
        }
    }

    /// 应用当前状态为快照 (供 save_state 调用)。
    fn snapshot_state(&self) -> PomodoroState {
        PomodoroState {
            phase: self.timer.phase(),
            run: RunState::from(self.timer.run()),
            remaining_secs: self.timer.remaining(self.now).as_secs(),
            current_scene: self.fader.current(),
            saved_elapsed_secs: self.now.as_secs(),
            saved_wall_secs: current_wall_secs(),
            has_seen_shortcut_hint: self.has_seen_shortcut_hint,
            completed_focus: self.timer.completed_focus(),
            today_date: self.today_date.clone(),
            today_count: self.today_count,
        }
    }

    /// 当前视觉调色板：淡化中为两端调色板的插值 (色调随画面同步流动);
    /// 暂停时整体降饱和 70% (含控件底色与文字色), 视觉上明显区分。
    fn palette(&self) -> ScenePalette {
        let (from, to, t) = self.fader.frame(self.now, |t| FADE_EASING.eval(t));
        let base = SCENES[from].palette.lerp(SCENES[to].palette, t);
        if self.timer.is_running() {
            base
        } else {
            base.desaturate(0.7)
        }
    }

    /// 当前场景主题 (颜色 token 随调色板流动)。
    fn theme(&self) -> SceneTheme {
        SceneTheme::new(self.palette())
    }
}

/// 当前 wall-clock Unix 秒 (失败时回落到 0, 不影响持久化逻辑)。
fn current_wall_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl App for PomodoroApp {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) {
        self.state_dirty = true;
        match msg {
            Msg::StartPause => self.timer.toggle(self.now),
            Msg::Skip => {
                self.timer.skip(self.now);
            }
            Msg::Reset => self.timer.reset(),
            Msg::PrevScene => {
                let target = (self.fader.current() + SCENES.len() - 1) % SCENES.len();
                self.fader.switch_to(target, self.now);
            }
            Msg::NextScene => {
                let target = (self.fader.current() + 1) % SCENES.len();
                self.fader.switch_to(target, self.now);
            }
            Msg::ToggleVisible => {
                if let Some(sender) = &self.window_sender {
                    sender.toggle_visible();
                }
            }
            Msg::Quit => {
                if let Some(sender) = &self.window_sender {
                    sender.quit();
                }
            }
        }
    }

    fn view(&self) -> Node {
        let t = self.theme();
        widget::node(
            Stack::new()
                .child(content_column(t))
                .child(flash_overlay_widget())
                .child(shortcut_hint_overlay_widget()),
        )
    }

    fn tick(&mut self, ctx: &AnimationCtx) {
        let dt = ctx.elapsed.saturating_sub(self.now);
        self.now = ctx.elapsed;
        let report = self.timer.tick(ctx.elapsed);
        if report.advanced {
            // 阶段流转触发视觉脉冲 + 系统提示音
            self.flash.trigger(self.now);
            audio::beep();
            // 通知 Handler: 阶段流转 (用于隐藏态时自动呼出窗口)
            if let Some(sender) = &self.window_sender {
                sender.phase_advanced();
            }
        }
        // 今日计数：自然完成的专注才计 (skip 不产生 focus_completions);
        // 跨日先归零再累加，并标脏触发 1Hz 节流持久化。
        if report.focus_completions > 0 {
            let today = today::today_string();
            self.today_count =
                today::resolve_today_count(&self.today_date, self.today_count, &today)
                    + u32::from(report.focus_completions);
            self.today_date = today;
            self.state_dirty = true;
        }
        // 跨日归零 (1Hz 节流): 常驻应用过午夜后, 不等下次完成即刷新副标「今日 N」。
        if self.now.saturating_sub(self.last_date_check) >= SAVE_THROTTLE {
            self.last_date_check = self.now;
            let today = today::today_string();
            if today != self.today_date {
                self.today_date = today;
                self.today_count = 0;
                self.state_dirty = true;
            }
        }
        // 1Hz 节流落盘：状态变更后，距上次保存 ≥ 1s 才写。
        if self.state_dirty && self.now.saturating_sub(self.last_save_at) >= SAVE_THROTTLE {
            match save_state(&self.snapshot_state()) {
                Ok(()) => {
                    self.last_save_at = self.now;
                    self.state_dirty = false;
                }
                Err(err) => {
                    log::warn!("保存状态失败：{err}");
                    self.last_save_at = self.now; // 节流，避免 60fps 重复刷写
                }
            }
        }
        // 环境音：与视觉淡化同源 (from/to/fade), 300ms 增益包络;
        // 休息期 duck 沉降 (世界退远一步), 懒初始化 + 静默降级。
        let (from, to, fade) = self.fader.frame(self.now, |t| FADE_EASING.eval(t));
        let duck = match self.timer.phase() {
            Phase::Focus => 1.0,
            Phase::Break | Phase::LongBreak => ambient::BREAK_DUCK,
        };
        let frame = self.ambient_mixer.frame_volumes(
            from,
            to,
            fade,
            self.timer.is_running(),
            duck,
            self.now,
        );
        self.ambient_player.apply(frame);
        // 场景动效: 与音频同潮汐契约 — 运行全量, 暂停 500ms 沉降 (视觉独立时长)。
        self.motion_gain = self.motion_envelope.gain(self.timer.is_running(), self.now);
        // 雨钟: 雨丝定格可见 (2026-07-29 用户裁定: 暂停显示雨丝, 不随包络沉降);
        // 包络只推进下落时间 — 暂停 500ms 减速冻结, 恢复 500ms 加速续走, 无跳变。
        self.rain_clock += dt.as_secs_f32() * self.motion_gain;
    }

    fn background_frame(&self) -> Option<BackgroundFrame> {
        let (from, to, fade) = self.fader.frame(self.now, |t| FADE_EASING.eval(t));
        let rain = motion::rain_intensity(from, to, fade);
        let fire = motion::fire_intensity(from, to, fade, self.motion_gain);
        let sea = motion::sea_intensity(from, to, fade, self.motion_gain);
        Some(
            BackgroundFrame::new(from, to, fade, self.palette().base)
                .with_motion(self.now.as_secs_f32(), rain)
                .with_fire(fire)
                .with_sea(sea)
                .with_rain_time(self.rain_clock),
        )
    }

    fn boot_elapsed_offset(&self) -> Duration {
        self.now_offset
    }

    fn attach_window_sender(&mut self, sender: WindowEventSender) {
        self.window_sender = Some(sender);
    }

    fn hotkey(&mut self, id: u8) -> Option<Msg> {
        match id {
            hotkey_ids::TOGGLE_VISIBLE => Some(Msg::ToggleVisible),
            hotkey_ids::START_PAUSE => Some(Msg::StartPause),
            hotkey_ids::QUIT => Some(Msg::Quit),
            _ => None,
        }
    }

    fn tray_action(&mut self, id: u8) -> Option<Msg> {
        match id {
            tray_action_ids::TOGGLE_VISIBLE => Some(Msg::ToggleVisible),
            tray_action_ids::START_PAUSE => Some(Msg::StartPause),
            tray_action_ids::QUIT => Some(Msg::Quit),
            _ => None,
        }
    }

    fn tray_menu(&self) -> danqing::tray_icon::menu::Menu {
        build_menu()
    }
}

/// 内容列：标题栏 + 中央倒计时 + 底部控件条 (无 flash 叠加，flash 由 Stack 在根上盖)。
fn content_column(t: SceneTheme) -> impl widget::Widget {
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
        .child(Padding::all(t.spacing_xl(), Center::new(control_pill(t))))
}

/// 全屏 flash 叠加层：阶段流转时 accent 色脉冲衰减。
/// 未激活时 alpha = 0, 完全透明 (无视觉影响); 激活时由 `progress()` 驱动 alpha。
fn flash_overlay_widget() -> impl widget::Widget {
    UiBox::new(Color::TRANSPARENT).bind_color(|s: &PomodoroApp| {
        let alpha = s.flash.progress(s.now).unwrap_or(0.0);
        let c = s.palette().accent;
        Color::rgba(c.r, c.g, c.b, alpha)
    })
}

/// 首次启动快捷键提示叠加层：窗口右下角三行快捷键说明，由 `hint.progress()` 驱动 alpha。
/// 不激活时完全透明 (无视觉影响); 激活时按 ease-out 淡入 → 停留 → ease-in 淡出。
/// 布局策略：外层 Column 用 fill spacer 把内容推到下方; 内层 Row 用 fill spacer 把内容推到右;
/// 最后 Padding 加 `spacing_lg` 的右/下内边距, 等价于把文本锚定在窗口右下角内缩 16px。
fn shortcut_hint_overlay_widget() -> impl widget::Widget {
    let line_painter = |s: &PomodoroApp| {
        let alpha = s.hint.progress(s.now).unwrap_or(0.0);
        let c = s.palette().text_secondary;
        Color::rgba(c.r, c.g, c.b, c.a * alpha)
    };
    let t = LightTheme;
    let line_a = Text::new(format!(
        "显示/隐藏  {}",
        shortcut_for_id(hotkey_ids::TOGGLE_VISIBLE)
    ))
    .font_size(t.font_size_small())
    .bind_color(line_painter);
    let line_b = Text::new(format!(
        "暂停/开始  {}",
        shortcut_for_id(hotkey_ids::START_PAUSE)
    ))
    .font_size(t.font_size_small())
    .bind_color(line_painter);
    let line_c = Text::new(format!("退出  {}", shortcut_for_id(hotkey_ids::QUIT)))
        .font_size(t.font_size_small())
        .bind_color(line_painter);
    let text_column = Column::new().child(line_a).child(line_b).child(line_c);
    let edge = t.spacing_lg();
    let padded = Padding::new(
        Edges {
            top: 0.0,
            right: edge,
            bottom: edge,
            left: 0.0,
        },
        text_column,
    );
    Column::new().fill(UiBox::new(Color::TRANSPARENT), 1).child(
        Row::new()
            .fill(UiBox::new(Color::TRANSPARENT), 1)
            .child(padded),
    )
}

/// 副标文案 (纯逻辑，可测):
/// - Running + Focus: `专注 · 场景 · 第 N/4 轮` (轮次 = completed_focus + 1);
/// - Running + Break/LongBreak: `休息 · 场景` / `长休息 · 场景` (不带轮次);
/// - 暂停/停止: `⏸ 已暂停 · 场景`;
/// - 今日计数 ≥ 1 时所有形态追加 ` · 今日 N`。
fn subtitle_text(
    running: bool,
    phase: Phase,
    scene_name: &str,
    completed_focus: u8,
    today_count: u32,
) -> String {
    let base = if !running {
        format!("⏸ 已暂停 · {scene_name}")
    } else {
        match phase {
            Phase::Focus => format!(
                "专注 · {scene_name} · 第 {}/{} 轮",
                completed_focus + 1,
                timer::CYCLE_LENGTH
            ),
            Phase::Break | Phase::LongBreak => format!("{} · {scene_name}", phase.label()),
        }
    };
    if today_count >= 1 {
        format!("{base} · 今日 {today_count}")
    } else {
        base
    }
}

/// 中央倒计时块：大字倒计时 + 阶段/场景标注。
/// 暂停时：倒计时切 `text_secondary` + 整体降饱和 + 副标加 "已暂停" 文字。
/// 三重信号确保暂停态视觉明显，用户无需猜测。
fn countdown_block(t: SceneTheme) -> impl widget::Widget {
    Column::new()
        .cross_stretch()
        .child(Center::new(
            Text::bind(|s: &PomodoroApp| s.timer.display(s.now))
                .font_size(t.font_size_display())
                .bind_color(|s: &PomodoroApp| {
                    if s.timer.is_running() {
                        s.palette().text_primary
                    } else {
                        s.palette().text_secondary
                    }
                }),
        ))
        .child(Center::new(
            Text::bind(|s: &PomodoroApp| {
                let scene_name = SCENES[s.fader.current()].name;
                subtitle_text(
                    s.timer.is_running(),
                    s.timer.phase(),
                    scene_name,
                    s.timer.completed_focus(),
                    s.today_count,
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
            Edges::symmetric(t.spacing_sm(), t.spacing_xs()),
            Row::new()
                .gap(t.spacing_xs())
                .child(ghost_button(t, "前", Msg::PrevScene))
                .child(primary_button(t))
                .child(ghost_button(t, "跳", Msg::Skip))
                .child(ghost_button(t, "重置", Msg::Reset))
                .child(ghost_button(t, "后", Msg::NextScene)),
        ))
}

fn main() -> ExitCode {
    example_log::init_log();

    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("应用启动失败：{err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> anyhow::Result<()> {
    // 优先加载持久化状态; 失败/不存在则新建默认会话。
    let mut app = match load_state() {
        Some(state) => {
            log::info!(
                "从持久化恢复：phase={:?} run={:?} remaining={}s scene={} now_offset={}s",
                state.phase,
                state.run,
                state.remaining_secs,
                state.current_scene,
                state.effective_now_offset().as_secs(),
            );
            PomodoroApp::from_state(state)
        }
        None => PomodoroApp::new_default(),
    };

    let background = BackgroundConfig::with_scenes(SCENES.iter().map(|s| s.image))
        .scale(ScaleMode::Cover)
        .with_noise(NOISE, NOISE_OPACITY);
    let config = WindowConfig {
        title: "丹青 · 番茄钟".into(),
        size: Size::new(960.0, 640.0),
        clear_color: SCENES[0].palette.base,
        background,
        // 常驻型应用：关闭按钮 / Alt+F4 只隐藏窗口，进程由托盘 / 全局热键退出。
        close_behavior: danqing::CloseBehavior::Hide,
        ..WindowConfig::default()
    };
    danqing::run_app(config, &mut app)?;
    // 退出 flush: 立即落盘一次，不走节流。
    app.flush();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subtitle_running_focus_shows_round() {
        assert_eq!(
            subtitle_text(true, Phase::Focus, "篝火", 1, 0),
            "专注 · 篝火 · 第 2/4 轮"
        );
        assert_eq!(
            subtitle_text(true, Phase::Focus, "海", 0, 0),
            "专注 · 海 · 第 1/4 轮"
        );
    }

    #[test]
    fn subtitle_running_break_and_long_break_hide_round() {
        assert_eq!(subtitle_text(true, Phase::Break, "海", 2, 0), "休息 · 海");
        assert_eq!(
            subtitle_text(true, Phase::LongBreak, "山", 0, 0),
            "长休息 · 山"
        );
    }

    #[test]
    fn subtitle_paused_keeps_paused_wording() {
        assert_eq!(
            subtitle_text(false, Phase::Focus, "雨", 3, 0),
            "⏸ 已暂停 · 雨"
        );
        assert_eq!(
            subtitle_text(false, Phase::LongBreak, "森林", 0, 0),
            "⏸ 已暂停 · 森林"
        );
    }

    #[test]
    fn subtitle_appends_today_count_when_positive() {
        assert_eq!(
            subtitle_text(true, Phase::Focus, "篝火", 1, 3),
            "专注 · 篝火 · 第 2/4 轮 · 今日 3"
        );
        assert_eq!(
            subtitle_text(true, Phase::Break, "海", 2, 1),
            "休息 · 海 · 今日 1"
        );
        assert_eq!(
            subtitle_text(false, Phase::Focus, "雨", 3, 2),
            "⏸ 已暂停 · 雨 · 今日 2"
        );
    }

    #[test]
    fn focus_completion_bumps_today_count() {
        let mut app = PomodoroApp::new_default();
        assert_eq!(app.today_count, 0);
        app.last_save_at = Duration::from_secs(25 * 60); // 防测试触发真实落盘
        app.ambient_player.disable_for_test(); // 防测试触碰音频设备
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_secs(25 * 60));
        app.tick(&ctx);
        assert_eq!(app.today_count, 1);
        assert!(app.state_dirty, "计数变更应标脏以触发持久化");
    }

    #[test]
    fn completion_on_new_day_resets_before_bump() {
        let mut app = PomodoroApp::new_default();
        app.today_date = "2020-01-01".into();
        app.today_count = 7;
        app.last_save_at = Duration::from_secs(25 * 60); // 防测试触发真实落盘
        app.ambient_player.disable_for_test(); // 防测试触碰音频设备
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_secs(25 * 60));
        app.tick(&ctx);
        assert_eq!(app.today_count, 1, "跨日应先归零再 +1");
        assert_eq!(app.today_date, today::today_string());
    }

    #[test]
    fn skip_does_not_bump_today_count() {
        let mut app = PomodoroApp::new_default();
        app.timer.toggle(app.now);
        app.update(Msg::Skip);
        assert_eq!(app.today_count, 0);
    }

    #[test]
    fn today_count_survives_state_roundtrip() {
        let mut app = PomodoroApp::new_default();
        app.today_count = 3;
        app.today_date = today::today_string();
        let state = app.snapshot_state();
        let restored = PomodoroApp::from_state(state);
        assert_eq!(restored.today_count, 3);
    }

    #[test]
    fn stale_date_resets_on_restore() {
        let mut app = PomodoroApp::new_default();
        app.today_count = 9;
        app.today_date = "2020-01-01".into();
        let state = app.snapshot_state();
        let restored = PomodoroApp::from_state(state);
        assert_eq!(restored.today_count, 0, "过期日期恢复时应归零");
    }

    #[test]
    fn background_frame_carries_rain_motion_when_running_on_rain_scene() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60); // 防测试触发真实落盘
        app.ambient_player.disable_for_test(); // 防测试触碰音频设备
        app.fader.switch_to(motion::RAIN_SCENE, app.now);
        app.timer.toggle(app.now); // 开始计时
        // 场景淡化 (800ms) 完成后包络才开始走 (首次 tick 边沿), 再走满 500ms。
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.rain_intensity - 1.0).abs() < 1e-6,
            "雨场景运行中雨效应全量: {}",
            frame.rain_intensity
        );
        assert!(
            (frame.time - 1.4).abs() < 1e-6,
            "动效时间应注入: {}",
            frame.time
        );
        assert!(
            frame.rain_time > 0.0,
            "运行中雨钟应推进: {}",
            frame.rain_time
        );
    }

    #[test]
    fn background_frame_rain_freezes_visible_on_pause() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60);
        app.ambient_player.disable_for_test();
        app.fader.switch_to(motion::RAIN_SCENE, app.now);
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        // 暂停 (2026-07-29 用户裁定): 雨丝定格可见 — 强度不沉降, 雨钟 500ms 内减速冻结。
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1650));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.rain_intensity - 1.0).abs() < 1e-6,
            "暂停边沿雨丝应全量可见: {}",
            frame.rain_intensity
        );
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1900));
        app.tick(&ctx);
        let frozen = app.background_frame().expect("应有背景帧").rain_time;
        assert!(frozen > 0.0, "雨钟应已推进过: {frozen}");
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.rain_intensity - 1.0).abs() < 1e-6,
            "暂停 500ms 后雨丝仍全量可见: {}",
            frame.rain_intensity
        );
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(2400));
        app.tick(&ctx);
        let later = app.background_frame().expect("应有背景帧").rain_time;
        assert!(
            (later - frozen).abs() < 1e-6,
            "暂停后雨钟应冻结: {frozen} -> {later}"
        );
        // 恢复: 雨钟从冻结点续走, 无跳变 (边沿帧包络为 0, 次帧起升)。
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(2900));
        app.tick(&ctx);
        let edge = app.background_frame().expect("应有背景帧").rain_time;
        assert!(
            (edge - frozen).abs() < 1e-6,
            "恢复边沿帧应连续: {frozen} -> {edge}"
        );
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(3400));
        app.tick(&ctx);
        let resumed = app.background_frame().expect("应有背景帧").rain_time;
        assert!(
            resumed > frozen,
            "恢复后雨钟应从冻结点续走: {frozen} -> {resumed}"
        );
    }

    #[test]
    fn background_frame_rain_stays_zero_on_non_rain_scene() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60);
        app.ambient_player.disable_for_test();
        app.timer.toggle(app.now); // 运行中, 但场景是篝火 (非雨)
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert_eq!(frame.rain_intensity, 0.0, "非雨场景雨效恒 0");
    }

    #[test]
    fn background_frame_carries_fire_motion_when_running_on_bonfire_scene() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60); // 防测试触发真实落盘
        app.ambient_player.disable_for_test(); // 防测试触碰音频设备
        app.fader.switch_to(motion::BONFIRE_SCENE, app.now); // 默认场景即篝火, 显式锁定
        app.timer.toggle(app.now); // 开始计时
        // 场景淡化 (800ms) 完成后包络才开始走 (首次 tick 边沿), 再走满 500ms。
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.fire_intensity - 1.0).abs() < 1e-6,
            "篝火场景运行中火效应全量: {}",
            frame.fire_intensity
        );
        assert_eq!(frame.rain_intensity, 0.0, "篝火场景雨效恒 0");
    }

    #[test]
    fn background_frame_fire_settles_on_pause() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60);
        app.ambient_player.disable_for_test();
        app.fader.switch_to(motion::BONFIRE_SCENE, app.now);
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        // 暂停: 边沿帧连续 (仍全量), +250ms 沉降中点 0.5, +500ms 消失。
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1650));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.fire_intensity - 1.0).abs() < 1e-6,
            "暂停边沿帧应连续: {}",
            frame.fire_intensity
        );
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1900));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.fire_intensity - 0.5).abs() < 1e-6,
            "暂停沉降中点: {}",
            frame.fire_intensity
        );
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(2150));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            frame.fire_intensity.abs() < 1e-6,
            "暂停 500ms 后火效应消失: {}",
            frame.fire_intensity
        );
    }

    #[test]
    fn background_frame_fire_stays_zero_on_non_bonfire_scene() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60);
        app.ambient_player.disable_for_test();
        app.fader.switch_to(motion::RAIN_SCENE, app.now); // 运行中, 但场景是雨 (非篝火)
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert_eq!(frame.fire_intensity, 0.0, "非篝火场景火效恒 0");
    }

    #[test]
    fn background_frame_carries_sea_motion_when_running_on_sea_scene() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60); // 防测试触发真实落盘
        app.ambient_player.disable_for_test(); // 防测试触碰音频设备
        app.fader.switch_to(motion::SEA_SCENE, app.now);
        app.timer.toggle(app.now); // 开始计时
        // 场景淡化 (800ms) 完成后包络才开始走 (首次 tick 边沿), 再走满 500ms。
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.sea_intensity - 1.0).abs() < 1e-6,
            "海场景运行中海效应全量: {}",
            frame.sea_intensity
        );
        assert_eq!(frame.rain_intensity, 0.0, "海场景雨效恒 0");
        assert_eq!(frame.fire_intensity, 0.0, "海场景火效恒 0");
    }

    #[test]
    fn background_frame_sea_settles_on_pause() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60);
        app.ambient_player.disable_for_test();
        app.fader.switch_to(motion::SEA_SCENE, app.now);
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        // 暂停: 边沿帧连续 (仍全量), +250ms 沉降中点 0.5, +500ms 消失。
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1650));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.sea_intensity - 1.0).abs() < 1e-6,
            "暂停边沿帧应连续: {}",
            frame.sea_intensity
        );
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1900));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            (frame.sea_intensity - 0.5).abs() < 1e-6,
            "暂停沉降中点: {}",
            frame.sea_intensity
        );
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(2150));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert!(
            frame.sea_intensity.abs() < 1e-6,
            "暂停 500ms 后海效应消失: {}",
            frame.sea_intensity
        );
    }

    #[test]
    fn background_frame_sea_stays_zero_on_non_sea_scene() {
        let mut app = PomodoroApp::new_default();
        app.last_save_at = Duration::from_secs(25 * 60);
        app.ambient_player.disable_for_test(); // 默认场景即篝火 (非海)
        app.timer.toggle(app.now);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(900));
        app.tick(&ctx);
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_millis(1400));
        app.tick(&ctx);
        let frame = app.background_frame().expect("应有背景帧");
        assert_eq!(frame.sea_intensity, 0.0, "非海场景海效恒 0");
    }

    #[test]
    fn midnight_rollover_resets_today_count_without_completion() {
        // 不等下次自然完成 (评审发现: 副标曾会显示昨天的「今日 N」)。
        let mut app = PomodoroApp::new_default();
        app.today_date = "2020-01-01".into();
        app.today_count = 5;
        app.last_save_at = Duration::from_secs(25 * 60); // 防测试触发真实落盘
        app.ambient_player.disable_for_test(); // 防测试触碰音频设备
        // 首次 tick (now=0) 距 last_date_check=0 不足 1s, 不触发检查。
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::ZERO);
        app.tick(&ctx);
        assert_eq!(app.today_count, 5, "1s 节流未到, 不应检查");
        // 1s 后: 触发跨日归零。
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_secs(1));
        app.tick(&ctx);
        assert_eq!(app.today_count, 0, "跨午夜应主动归零");
        assert_eq!(app.today_date, today::today_string());
        // 同日不再误清: 有计数后保持。
        app.today_count = 2;
        let ctx = AnimationCtx::new(std::time::Instant::now(), Duration::from_secs(2));
        app.tick(&ctx);
        assert_eq!(app.today_count, 2, "同日不得误清");
    }
}
