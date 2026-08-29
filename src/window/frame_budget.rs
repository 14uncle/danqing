//! @author 十四叔
//! @date 2026/08/30
//!
//! 帧率预算 (纯逻辑): 常驻氛围应用的自适应帧率决策 —— [`WindowMode::Adaptive`]
//! 的核心判定。活动时全帧率, 空闲降帧, 前台有全屏应用 (游戏/视频) 时暂停
//! 渲染 (Wallpaper Engine 同款生存策略: 用户在游戏, 世界别抢 GPU)。
//!
//! 全部输入显式注入 (不读 wall-clock / 不碰 OS), 可完整单测;
//! OS 侧全屏检测见 `super::fullscreen`, 接线见 `Handler`。

use std::time::Duration;

/// 帧率档 (决策输出)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameRate {
    /// 全帧率 (活动期/事件升帧期): ~60fps。
    Full,
    /// 降帧 (空闲期): 5fps —— 灯火呼吸仍有生气, 电费税大降。
    Throttled,
    /// 暂停 (前台有其它应用的全屏窗口): 零渲染, 仅低频轮询全屏态。
    Suspended,
}

/// 空闲判定阈值: 无输入/消息/升帧满此时长即降帧
/// (spec 验收条款 1: 无事件无交互 30s 后 ≤5fps)。
pub(crate) const IDLE_ENTER: Duration = Duration::from_secs(30);

/// 帧率决策 (纯函数)。
///
/// - `since_activity`: 距上次活动 (窗口输入/托盘热键动作/应用消息) 的时长
/// - `boost_remaining`: 事件升帧剩余时长 (零 = 未升帧; 微事件播放期由产品请求)
/// - `fullscreen_app_foreground`: 前台存在其它应用的全屏窗口
///
/// 优先级: 全屏暂停 > 升帧 > 活动期 > 降帧。空闲再久也只降帧不暂停
/// —— 世界仍在过日子 (只是过得安静), 暂停只留给「用户明显没空看」。
pub(crate) fn decide(
    since_activity: Duration,
    boost_remaining: Duration,
    fullscreen_app_foreground: bool,
) -> FrameRate {
    if fullscreen_app_foreground {
        return FrameRate::Suspended;
    }
    if boost_remaining > Duration::ZERO || since_activity < IDLE_ENTER {
        return FrameRate::Full;
    }
    FrameRate::Throttled
}

/// 各帧率档的事件循环轮询间隔 (ControlFlow::WaitUntil 的步长)。
pub(crate) fn poll_interval(rate: FrameRate) -> Duration {
    match rate {
        FrameRate::Full => Duration::from_millis(16),
        FrameRate::Throttled => Duration::from_millis(200),
        // 暂停态只轮询全屏检测, 间隔无需短 —— 全屏退出半秒内感知即可。
        FrameRate::Suspended => Duration::from_millis(500),
    }
}

/// render_frame 尾的自续渲染链 (request_redraw) 是否放行:
/// 仅全帧率档续链; 降帧/暂停态由 about_to_wait 按轮询间隔单驱,
/// 否则自续链会把降帧架空回 60fps。
pub(crate) fn should_continue_render_chain(rate: FrameRate) -> bool {
    rate == FrameRate::Full
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 活动期内 (输入/消息/动作 < 30s) → 全帧率。
    #[test]
    fn active_period_renders_full_rate() {
        assert_eq!(
            decide(Duration::ZERO, Duration::ZERO, false),
            FrameRate::Full
        );
        assert_eq!(
            decide(Duration::from_secs(29), Duration::ZERO, false),
            FrameRate::Full,
            "29s 仍在活动期"
        );
    }

    /// 空闲 30s 后降帧; 边界 30.0s 整即降 (spec: 无事件无交互 30s 后 ≤5fps)。
    #[test]
    fn idle_after_30s_throttles() {
        assert_eq!(
            decide(Duration::from_secs(30), Duration::ZERO, false),
            FrameRate::Throttled
        );
        assert_eq!(
            decide(Duration::from_secs(3600), Duration::ZERO, false),
            FrameRate::Throttled,
            "闲置再久也只是降帧, 不暂停 (世界仍在过日子)"
        );
    }

    /// 事件升帧: 空闲中遇升帧期 → 全帧率播放 (流星 10fps = 魔法破产)。
    #[test]
    fn boost_overrides_idle() {
        assert_eq!(
            decide(Duration::from_secs(600), Duration::from_secs(20), false),
            FrameRate::Full,
            "升帧剩余 20s → 全帧率"
        );
    }

    /// 升帧耗尽 → 回落到活动/空闲判定 (不残留)。
    #[test]
    fn expired_boost_falls_back() {
        assert_eq!(
            decide(Duration::from_secs(600), Duration::ZERO, false),
            FrameRate::Throttled
        );
    }

    /// 前台全屏应用 → 暂停渲染 (WE 同款生存策略), 优先级最高:
    /// 活动期/升帧期都压不住 (用户在游戏, 世界别抢 GPU)。
    #[test]
    fn fullscreen_app_suspends_everything() {
        assert_eq!(
            decide(Duration::ZERO, Duration::ZERO, true),
            FrameRate::Suspended
        );
        assert_eq!(
            decide(Duration::from_secs(600), Duration::from_secs(20), true),
            FrameRate::Suspended,
            "升帧期遇全屏也暂停"
        );
    }

    /// 全屏退出 → 回到活动判定 (世界回来继续过日子)。
    #[test]
    fn fullscreen_exit_resumes_activity_rules() {
        assert_eq!(
            decide(Duration::from_secs(600), Duration::ZERO, false),
            FrameRate::Throttled
        );
    }

    /// 轮询间隔映射: 全帧 16ms / 降帧 200ms (5fps) / 暂停 500ms
    /// (只轮询全屏态, 零渲染)。
    #[test]
    fn poll_interval_mapping() {
        assert_eq!(poll_interval(FrameRate::Full), Duration::from_millis(16));
        assert_eq!(
            poll_interval(FrameRate::Throttled),
            Duration::from_millis(200)
        );
        assert_eq!(
            poll_interval(FrameRate::Suspended),
            Duration::from_millis(500)
        );
    }

    /// 降帧/暂停态不续渲染链 (render_frame 尾的自续 request_redraw 要按此门控)。
    #[test]
    fn only_full_rate_continues_render_chain() {
        assert!(should_continue_render_chain(FrameRate::Full));
        assert!(!should_continue_render_chain(FrameRate::Throttled));
        assert!(!should_continue_render_chain(FrameRate::Suspended));
    }
}
