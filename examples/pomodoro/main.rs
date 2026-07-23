//! @author 十四叔
//! @date 2026/07/23

//! 丹青番茄钟 POC —— 专注陪伴工具 × 场景沉浸。
//!
//! 当前为骨架 (阶段 2 Task 3): 窗口 + 标题栏 + 倒计时占位;
//! 完整组装 (控件条 / 场景切换 / 过渡动画) 见 Task 6/7。

mod timer;

use std::time::Duration;

use danqing::widget::{self, Center, Column, Node, Text, TitleBar};
use danqing::{App, LightTheme, Size, Theme, WindowAction, WindowConfig};
use timer::Pomodoro;

/// 番茄钟应用状态。
struct PomodoroApp {
    /// 计时状态机 (纯逻辑)。
    timer: Pomodoro,
    /// 注入时间轴: 自应用启动的累计时间 (Task 6 由 tick 心跳推进)。
    now: Duration,
}

impl App for PomodoroApp {
    type Msg = ();

    fn update(&mut self, _msg: ()) {}

    fn view(&self) -> Node {
        let t = LightTheme;
        widget::node(
            Column::new()
                .child(
                    TitleBar::themed(&t, "丹青 · 番茄钟")
                        .on_close(|| WindowAction::Close)
                        .on_minimize(|| WindowAction::Minimize)
                        .on_maximize(|| WindowAction::MaximizeOrRestore)
                        .on_drag(|| WindowAction::Drag),
                )
                .fill(
                    Center::new(
                        Text::bind(|s: &PomodoroApp| s.timer.display(s.now))
                            .font_size(t.font_size_display())
                            .color(t.text_primary()),
                    ),
                    1,
                ),
        )
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = PomodoroApp {
        timer: Pomodoro::new(),
        now: Duration::ZERO,
    };
    let t = LightTheme;
    let config = WindowConfig {
        title: "丹青 · 番茄钟".into(),
        size: Size::new(960.0, 640.0),
        clear_color: t.background(),
        ..WindowConfig::default()
    };
    danqing::run_app(config, &mut app)?;
    Ok(())
}
