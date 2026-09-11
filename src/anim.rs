//! @author 十四叔
//! @date 2026/09/09

//! 动画原语 (纯逻辑): 补间包络 / 一次性脉冲 / 提示时序 / 离散交叉淡化。
//!
//! 做什么:
//! - [`Tween`]: 目标值变化的平滑跟随 (目标可边沿翻转、可逐帧微变),
//!   反向从当前值续接走全程时长 (无跳变), 稳定态精确到达 (无渐近漂移)。
//! - [`Pulse`]: 一次性 alpha 脉冲 (触发 → 线性衰减 → 可再触发)。
//! - [`Cue`]: 提示时序状态机 (静默延迟 → 淡入 → 停留 → 淡出 → 结束)。
//! - [`Crossfade`]: 离散状态 (索引) 间交叉淡化, 打断按进度占优侧吸附。
//!
//! 不做什么: 不读 wall-clock (时间一律外部注入 `Duration`, 可完整单测);
//! 不做泛型/多值补间 (v1 只 f32 标量); 不做弹簧/关键帧/时间线编排。
//!
//! 为什么: 这四个模式在农场被重复发明 4+ 次 (pomodoro motion/flash/hint/
//! fader 四文件 + 框架 audio/mixer.rs 私有包络 + switch.rs ad-hoc 步进),
//! 2026-09-09 下沉盘点 (docs/intent/framework-sinking.md 簇A) 裁定收编,
//! spec: docs/specs/SPEC-anim.md。

use std::time::Duration;

use crate::theme::Easing;

/// 通用补间包络: 目标值变化的平滑跟随器。
///
/// 目标变化 (边沿翻转 / 中途反向 / 逐帧微变) 时从**当前值**起走全程
/// `duration` 续接 (无跳变); 稳定态精确到达目标 (无渐近漂移)。
/// 值域不夹取 —— 0..1 是消费者语义, 本类型通吃任意 f32 (尺寸/位置/透明度)。
///
/// 时间由外部注入 (`AnimationCtx.elapsed` 或产品自维护的累计轴),
/// 不读 wall-clock, 可完整单元测试。
#[derive(Debug, Clone)]
pub struct Tween {
    /// 当前值 (上次推进的结果)。
    current: f32,
    /// 进行中的补间: (起始值, 目标值, 开始时刻)。
    anim: Option<(f32, f32, Duration)>,
    /// 补间时长。
    duration: Duration,
    /// 动效曲线。
    easing: Easing,
}

impl Tween {
    /// 创建补间: 初值 0, 静止 (目标为 0 时不触发补间)。
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self {
            current: 0.0,
            anim: None,
            duration,
            easing,
        }
    }

    /// 指定初值 (默认 0)。
    pub fn with_initial(mut self, initial: f32) -> Self {
        self.current = initial;
        self
    }

    /// 推进并返回当前值。
    ///
    /// 目标变化 (含进行中反向/微变) → 从当前值起重触发全程补间;
    /// 目标静止 → 补间播完后精确停在目标。零时长 = 直接到位。
    /// 目标须为有限 f32: NaN/inf 会让补间每帧重触发且污染当前值 (GIGO, 不加运行时检查)。
    pub fn value(&mut self, now: Duration, target: f32) -> f32 {
        if self.duration.is_zero() {
            self.anim = None;
            self.current = target;
            return self.current;
        }
        // 目标恰等于当前值: 已在终点, 取消在途补间 (消除「原地补间」的在途虚报)。
        if target == self.current {
            self.anim = None;
            return self.current;
        }
        let needs_anim = match self.anim {
            Some((_, target_v, _)) => target_v != target, // 目标变了 → 重触发
            None => self.current != target,               // 未到目标 → 触发
        };
        if needs_anim {
            self.anim = Some((self.current, target, now));
        }
        if let Some((start_v, target_v, start_t)) = self.anim {
            let t = (now.saturating_sub(start_t).as_secs_f32() / self.duration.as_secs_f32())
                .clamp(0.0, 1.0);
            self.current = start_v + (target_v - start_v) * self.easing.eval(t);
            if t >= 1.0 {
                self.anim = None;
                self.current = target_v;
            }
        }
        self.current
    }

    /// 上次推进后补间是否仍在途 (供 OnDemand 模式消费者决定 boost_frames)。
    pub fn is_animating(&self) -> bool {
        self.anim.is_some()
    }
}

/// 一次性脉冲: 触发 → 满 alpha 线性衰减 → 结束。
///
/// 进行中的触发被忽略 (避免视觉抖); 已结束可再次触发。
/// 消费形态: `progress()` 返回 Some(alpha) 时按 alpha 叠加画一层。
#[derive(Debug, Clone)]
pub struct Pulse {
    /// 脉冲起点 (注入时间轴); None = 未激活或已结束。
    started: Option<Duration>,
    /// 脉冲总时长。
    duration: Duration,
}

impl Pulse {
    /// 创建未激活的脉冲器。
    pub fn new(duration: Duration) -> Self {
        Self {
            started: None,
            duration,
        }
    }

    /// 触发一次脉冲。进行中触发被忽略; 已结束的可以再触发。
    pub fn trigger(&mut self, now: Duration) {
        if self.is_active(now) {
            return;
        }
        self.started = Some(now);
    }

    /// 当前脉冲进度: 1.0 (起点满) → 0.0 (终点), 线性衰减。
    /// 未激活或已结束返回 None。
    pub fn progress(&self, now: Duration) -> Option<f32> {
        let start = self.started?;
        let elapsed = now.saturating_sub(start);
        let total = self.duration.as_secs_f32();
        if total <= 0.0 {
            return None;
        }
        if elapsed >= self.duration {
            return None;
        }
        Some(1.0 - elapsed.as_secs_f32() / total)
    }

    /// 脉冲是否正在播放。
    pub fn is_active(&self, now: Duration) -> bool {
        self.progress(now).is_some()
    }
}

impl Default for Pulse {
    /// 默认 600ms 衰减 (pomodoro 完成反馈实测值)。
    fn default() -> Self {
        Self::new(Duration::from_millis(600))
    }
}

/// 提示时序: 静默延迟 → 淡入 → 停留 → 淡出 四段时长。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CueTiming {
    /// 淡入前的静默延迟 (避免启动瞬间抢戏)。
    pub delay: Duration,
    /// 淡入动画时长 (EaseOut 曲线)。
    pub fade_in: Duration,
    /// 满 alpha 停留时长。
    pub hold: Duration,
    /// 淡出动画时长 (EaseIn 曲线)。
    pub fade_out: Duration,
}

impl CueTiming {
    /// 四段总时长。
    fn total(&self) -> Duration {
        self.delay + self.fade_in + self.hold + self.fade_out
    }
}

impl Default for CueTiming {
    /// pomodoro 首启提示实测值: 1.5s 静默 / 300ms 淡入 / 5s 停留 / 500ms 淡出。
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(1500),
            fade_in: Duration::from_millis(300),
            hold: Duration::from_millis(5000),
            fade_out: Duration::from_millis(500),
        }
    }
}

/// 一次性提示状态机: 按 [`CueTiming`] 四段时序播放一条 alpha 曲线。
///
/// 消费形态: `progress()` 返回 Some(alpha) 时按 alpha 画提示, None 完全不可见。
/// 零时长的段自然跳过 (fade_in=0 立即满, fade_out=0 停留结束即 None)。
#[derive(Debug, Clone)]
pub struct Cue {
    /// 触发起点 (注入时间轴); None = 未激活。
    triggered_at: Option<Duration>,
    /// 四段时序配置。
    timing: CueTiming,
}

impl Cue {
    /// 创建未激活的提示。
    pub fn new(timing: CueTiming) -> Self {
        Self {
            triggered_at: None,
            timing,
        }
    }

    /// 在指定时刻触发提示 (可传启动偏移, 让动画从窗口启动那一刻起算)。
    pub fn trigger(&mut self, now: Duration) {
        self.triggered_at = Some(now);
    }

    /// 当前 alpha; 未激活或已结束返回 None。
    pub fn progress(&self, now: Duration) -> Option<f32> {
        let start = self.triggered_at?;
        let elapsed = now.saturating_sub(start);
        if elapsed >= self.timing.total() {
            return None;
        }
        let e = elapsed.as_secs_f32();
        let t = &self.timing;
        let delay = t.delay.as_secs_f32();
        let fade_in = t.fade_in.as_secs_f32();
        let hold = t.hold.as_secs_f32();
        let fade_out = t.fade_out.as_secs_f32();
        // 零时长段的除法不可达 (条件先判超时/越段), 与 hint.rs 源头结构一致。
        let alpha = if e < delay {
            0.0
        } else if e < delay + fade_in {
            Easing::EaseOut.eval((e - delay) / fade_in)
        } else if e < delay + fade_in + hold {
            1.0
        } else {
            1.0 - Easing::EaseIn.eval((e - delay - fade_in - hold) / fade_out)
        };
        Some(alpha)
    }
}

/// 离散状态交叉淡化器: 两个状态 (usize 索引) 间按进度插值。
///
/// 切换不是瞬时跳变: 旧状态与新状态在 `duration` 内交叉淡化。
/// 中途再切换 (打断): 按进度占优侧 (>=0.5) 吸附为新起点, 少数派贡献舍弃。
/// 消费形态与 `BackgroundFrame::new(from, to, fade)` 直通。
#[derive(Debug, Clone)]
pub struct Crossfade {
    /// 淡化起点状态 (静止时与 `to` 相同)。
    from: usize,
    /// 淡化终点状态 (当前目标)。
    to: usize,
    /// 本次淡化开始时刻 (注入时间轴)。
    start: Duration,
    /// 淡化时长。
    duration: Duration,
}

impl Crossfade {
    /// 创建静止于某状态的淡化器。
    pub fn new(initial: usize, duration: Duration) -> Self {
        Self {
            from: initial,
            to: initial,
            start: Duration::ZERO,
            duration,
        }
    }

    /// 当前目标状态 (淡化结束后的状态)。
    pub fn current(&self) -> usize {
        self.to
    }

    /// 切换到目标状态; 若正在淡化中, 按进度占优侧吸附为新起点。
    pub fn switch_to(&mut self, target: usize, now: Duration) {
        if target == self.to {
            return;
        }
        let dominant = if self.progress(now) >= 0.5 {
            self.to
        } else {
            self.from
        };
        self.from = dominant;
        self.to = target;
        self.start = now;
    }

    /// 原始进度 (0..=1, 饱和); 静止时恒为 1; 零时长淡化恒为 1 (不除零)。
    pub fn progress(&self, now: Duration) -> f32 {
        if self.from == self.to || self.duration.is_zero() {
            return 1.0;
        }
        (now.saturating_sub(self.start).as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// 帧三元组 (from, to, eased fade); 端点精确 (静止时 from == to, fade = 1)。
    pub fn frame(&self, now: Duration, easing: Easing) -> (usize, usize, f32) {
        (self.from, self.to, easing.eval(self.progress(now)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    // ---- Tween ----

    #[test]
    fn tween_idle_stays_at_initial() {
        let mut tw = Tween::new(ms(500), Easing::Linear);
        assert_eq!(tw.value(ms(0), 0.0), 0.0);
        assert_eq!(tw.value(ms(10_000), 0.0), 0.0);
        assert!(!tw.is_animating());
        // 非零初值: 目标即初值时不触发补间。
        let mut tw = Tween::new(ms(500), Easing::Linear).with_initial(0.3);
        assert_eq!(tw.value(ms(5), 0.3), 0.3);
        assert!(!tw.is_animating());
    }

    #[test]
    fn tween_fades_in_over_duration() {
        let mut tw = Tween::new(ms(500), Easing::Linear);
        assert_eq!(tw.value(ms(0), 1.0), 0.0, "边沿帧从当前值起");
        assert!((tw.value(ms(250), 1.0) - 0.5).abs() < 1e-6, "中点半量");
        assert_eq!(tw.value(ms(500), 1.0), 1.0, "终点精确到达");
        assert_eq!(tw.value(ms(9999), 1.0), 1.0, "稳定态不漂移");
        assert!(!tw.is_animating());
    }

    #[test]
    fn tween_reverse_continues_from_current() {
        // 移植 MotionEnvelope 语义: 反向边沿从当前值续接 (无跳变), 走全程时长。
        let mut tw = Tween::new(ms(500), Easing::Linear);
        tw.value(ms(0), 1.0);
        assert_eq!(tw.value(ms(500), 1.0), 1.0, "全量");
        // 目标归零: 边沿帧连续。
        assert_eq!(tw.value(ms(1000), 0.0), 1.0);
        let mid = tw.value(ms(1250), 0.0);
        assert!((mid - 0.5).abs() < 1e-6, "淡出中点 0.5");
        // 淡出中恢复: 从当前值续接, 500ms 全程后回全量。
        let v = tw.value(ms(1300), 1.0);
        assert!((v - mid).abs() < 1e-6, "反向边沿应连续: {mid} -> {v}");
        assert_eq!(tw.value(ms(1800), 1.0), 1.0);
    }

    #[test]
    fn tween_applies_easing_curve() {
        let mut tw = Tween::new(ms(500), Easing::EaseInOut);
        tw.value(ms(0), 1.0);
        // EaseInOut 中点仍为 0.5, 1/4 点 (t=0.25) 经曲线压低至 0.0625。
        assert!((tw.value(ms(250), 1.0) - 0.5).abs() < 1e-6);
        let mut tw = Tween::new(ms(500), Easing::EaseInOut);
        tw.value(ms(0), 1.0);
        assert!((tw.value(ms(125), 1.0) - 0.0625).abs() < 1e-6);
    }

    #[test]
    fn tween_follows_drifting_target() {
        // 时辰曲线范式: 目标逐帧微变, 包络平滑跟随, 每帧无跳变。
        let mut tw = Tween::new(ms(500), Easing::Linear);
        assert_eq!(tw.value(ms(0), 0.6), 0.0);
        // 补间 0 → 0.6 全程 500ms: ms(150) 处 t=0.3 → 0.18。
        assert!((tw.value(ms(150), 0.6) - 0.18).abs() < 1e-6);
        // 目标漂到 0.7: 从当前值 0.18 续接 (不重跳)。
        let v = tw.value(ms(160), 0.7);
        assert!((v - 0.18).abs() < 1e-6, "微变续接连续: {v}");
        // 新补间 0.18 → 0.7 全程 500ms: 中点 (ms 410) = 0.44, 终点 (ms 660) 精确到 0.7。
        assert!((tw.value(ms(410), 0.7) - 0.44).abs() < 1e-6);
        assert_eq!(tw.value(ms(660), 0.7), 0.7);
    }

    #[test]
    fn tween_does_not_clamp_range() {
        // 值域不夹: 0..1 是消费者语义, Tween 通吃任意 f32 (尺寸/位置/角度)。
        let mut tw = Tween::new(ms(100), Easing::Linear).with_initial(10.0);
        assert_eq!(tw.value(ms(0), 360.0), 10.0);
        assert_eq!(tw.value(ms(100), 360.0), 360.0);
        assert_eq!(tw.value(ms(999), 360.0), 360.0);
    }

    #[test]
    fn tween_zero_duration_jumps_to_target() {
        // 零时长防护: 不除零, 直接到位。
        let mut tw = Tween::new(Duration::ZERO, Easing::Linear);
        assert_eq!(tw.value(ms(0), 1.0), 1.0);
        assert!(!tw.is_animating());
    }

    #[test]
    fn tween_target_equal_current_cancels_in_place_anim() {
        // 补间途中目标漂回恰等于当前值: 取消补间, 不「原地补间」虚报在途
        // (OnDemand 消费者靠 is_animating 决定 boost_frames)。
        let mut tw = Tween::new(ms(500), Easing::Linear);
        tw.value(ms(0), 1.0);
        assert!((tw.value(ms(100), 1.0) - 0.2).abs() < 1e-6, "在途 0.2");
        assert!(tw.is_animating());
        assert_eq!(tw.value(ms(200), 0.2), 0.2, "目标==当前值: 无跳变");
        assert!(!tw.is_animating(), "不应虚报在途");
        assert_eq!(tw.value(ms(9999), 0.2), 0.2, "稳定在原值");
    }

    // ---- Pulse (移植 pomodoro flash.rs 语义) ----

    #[test]
    fn pulse_idle_returns_none() {
        let p = Pulse::default();
        assert!(p.progress(ms(0)).is_none());
        assert!(p.progress(ms(999_999)).is_none());
        assert!(!p.is_active(ms(0)));
    }

    #[test]
    fn pulse_trigger_starts_at_full_progress() {
        let mut p = Pulse::default();
        p.trigger(ms(1000));
        assert_eq!(p.progress(ms(1000)), Some(1.0));
    }

    #[test]
    fn pulse_progress_decays_linearly() {
        let mut p = Pulse::new(ms(1000));
        p.trigger(ms(0));
        assert_eq!(p.progress(ms(0)), Some(1.0));
        assert_eq!(p.progress(ms(500)), Some(0.5));
        assert_eq!(p.progress(ms(750)), Some(0.25));
    }

    #[test]
    fn pulse_returns_none_after_duration() {
        let mut p = Pulse::new(ms(600));
        p.trigger(ms(0));
        assert!(p.progress(ms(600)).is_none());
        assert!(p.progress(ms(999_999)).is_none());
    }

    #[test]
    fn pulse_trigger_during_active_is_ignored() {
        let mut p = Pulse::new(ms(1000));
        p.trigger(ms(0));
        // 100ms 后试图再次触发, 进度不被重置 (避免视觉抖)。
        p.trigger(ms(100));
        assert_eq!(p.progress(ms(100)), Some(0.9));
    }

    #[test]
    fn pulse_trigger_after_duration_is_accepted() {
        // 已结束的脉冲可以再次触发 (不允许首次后永久死锁)。
        let mut p = Pulse::new(ms(100));
        p.trigger(ms(0));
        assert!(p.progress(ms(100)).is_none());
        p.trigger(ms(200));
        assert_eq!(p.progress(ms(200)), Some(1.0));
    }

    // ---- Cue (移植 pomodoro hint.rs 语义, 四段时长参数化) ----

    #[test]
    fn cue_idle_returns_none() {
        let c = Cue::new(CueTiming::default());
        assert!(c.progress(ms(0)).is_none());
        assert!(c.progress(ms(999_999)).is_none());
    }

    #[test]
    fn cue_triggered_stays_zero_before_delay() {
        let mut c = Cue::new(CueTiming::default());
        c.trigger(ms(0));
        assert_eq!(c.progress(ms(0)), Some(0.0));
        assert_eq!(c.progress(ms(1499)), Some(0.0));
    }

    #[test]
    fn cue_fade_in_is_ease_out_and_reaches_full() {
        let mut c = Cue::new(CueTiming::default());
        c.trigger(ms(0));
        assert_eq!(c.progress(ms(1500)), Some(0.0));
        // 淡入中点 (1650ms) ease-out 应高于线性中点 0.5。
        let mid = c.progress(ms(1650)).unwrap();
        assert!(mid > 0.5 && mid < 0.9, "淡入中点应高于 0.5, 实际 {mid}");
        assert_eq!(c.progress(ms(1800)), Some(1.0));
    }

    #[test]
    fn cue_hold_phase_keeps_alpha_at_one() {
        let mut c = Cue::new(CueTiming::default());
        c.trigger(ms(0));
        assert_eq!(c.progress(ms(1800)), Some(1.0));
        assert_eq!(c.progress(ms(4000)), Some(1.0));
        assert_eq!(c.progress(ms(6799)), Some(1.0));
    }

    #[test]
    fn cue_fade_out_is_ease_in_and_ends() {
        let mut c = Cue::new(CueTiming::default());
        c.trigger(ms(0));
        assert_eq!(c.progress(ms(6800)), Some(1.0));
        // 淡出中点 (7050ms) ease-in 起手慢, alpha 应仍高 (≈0.875)。
        let mid = c.progress(ms(7050)).unwrap();
        assert!(mid > 0.7, "ease-in 淡出中点应高于线性, 实际 {mid}");
        assert!(c.progress(ms(7300)).is_none());
    }

    #[test]
    fn cue_returns_none_after_total_duration() {
        let mut c = Cue::new(CueTiming::default());
        c.trigger(ms(0));
        assert!(c.progress(ms(7300)).is_none());
        assert!(c.progress(ms(999_999)).is_none());
    }

    #[test]
    fn cue_trigger_at_nonzero_offset_shifts_curve() {
        // 触发点在 1000ms, 整条曲线向后平移 1s。
        let mut c = Cue::new(CueTiming::default());
        c.trigger(ms(1000));
        assert_eq!(c.progress(ms(1000)), Some(0.0));
        assert_eq!(c.progress(ms(2499)), Some(0.0));
        assert_eq!(c.progress(ms(2800)), Some(1.0));
        assert!(c.progress(ms(8300)).is_none());
    }

    #[test]
    fn cue_custom_timing() {
        // 非默认时序: 100ms 延迟 / 100ms 淡入 / 200ms 停留 / 100ms 淡出。
        let timing = CueTiming {
            delay: ms(100),
            fade_in: ms(100),
            hold: ms(200),
            fade_out: ms(100),
        };
        let mut c = Cue::new(timing);
        c.trigger(ms(0));
        assert_eq!(c.progress(ms(99)), Some(0.0));
        assert_eq!(c.progress(ms(200)), Some(1.0));
        assert_eq!(c.progress(ms(399)), Some(1.0));
        // 淡出中点 t=0.5: alpha = 1 - EaseIn(0.5) = 0.875 (缓入起手慢)。
        assert_eq!(c.progress(ms(450)), Some(0.875));
        assert!(c.progress(ms(500)).is_none());
    }

    // ---- Crossfade (移植 pomodoro fader.rs 语义) ----

    #[test]
    fn crossfade_idle_stays_on_initial() {
        let f = Crossfade::new(2, ms(800));
        assert_eq!(f.current(), 2);
        assert!(f.progress(ms(0)) >= 1.0);
        assert!(f.progress(ms(999_999)) >= 1.0);
        assert_eq!(f.frame(ms(123), Easing::Linear), (2, 2, 1.0));
    }

    #[test]
    fn crossfade_switch_starts_fade_from_zero() {
        let mut f = Crossfade::new(0, ms(800));
        f.switch_to(1, ms(1000));
        assert_eq!(f.current(), 1);
        assert_eq!(f.progress(ms(1000)), 0.0);
        assert_eq!(f.frame(ms(1000), Easing::Linear), (0, 1, 0.0));
    }

    #[test]
    fn crossfade_progresses_linearly_before_easing() {
        let mut f = Crossfade::new(0, ms(800));
        f.switch_to(1, ms(1000));
        assert!((f.progress(ms(1400)) - 0.5).abs() < 1e-6);
        assert_eq!(f.frame(ms(1400), Easing::Linear), (0, 1, 0.5));
    }

    #[test]
    fn crossfade_completes_exactly_at_duration() {
        let mut f = Crossfade::new(0, ms(800));
        f.switch_to(1, ms(1000));
        assert_eq!(f.progress(ms(1800)), 1.0);
        assert!(f.progress(ms(999_999)) >= 1.0);
    }

    #[test]
    fn crossfade_frame_applies_easing_curve() {
        let mut f = Crossfade::new(0, ms(800));
        f.switch_to(1, ms(1000));
        // ms(1200) 原始进度 0.25; EaseInOut 前半段 = 4t³ → 0.0625。
        let (_, _, fade) = f.frame(ms(1200), Easing::EaseInOut);
        assert!((fade - 0.0625).abs() < 1e-6);
    }

    #[test]
    fn crossfade_switch_to_same_is_noop() {
        let mut f = Crossfade::new(0, ms(800));
        f.switch_to(1, ms(1000));
        f.switch_to(1, ms(1200));
        assert!((f.progress(ms(1400)) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn crossfade_interrupt_early_snaps_back_to_origin() {
        let mut f = Crossfade::new(0, ms(800));
        f.switch_to(1, ms(1000));
        // 进度 0.25 (<0.5): 占优侧仍是 from=0, 新淡化从 0 起。
        f.switch_to(2, ms(1200));
        assert_eq!(f.frame(ms(1200), Easing::Linear), (0, 2, 0.0));
    }

    #[test]
    fn crossfade_interrupt_late_snaps_forward_to_target() {
        let mut f = Crossfade::new(0, ms(800));
        f.switch_to(1, ms(1000));
        // 进度 0.75 (>=0.5): 占优侧是 to=1, 新淡化从 1 起。
        f.switch_to(2, ms(1600));
        assert_eq!(f.frame(ms(1600), Easing::Linear), (1, 2, 0.0));
    }

    #[test]
    fn crossfade_zero_duration_is_instant() {
        // 零时长防护: 不除零, 切换即到位 (源头 fader.rs 无此防护, 下沉补齐)。
        let mut f = Crossfade::new(0, Duration::ZERO);
        f.switch_to(1, ms(1000));
        assert_eq!(f.progress(ms(1000)), 1.0);
        assert_eq!(f.frame(ms(1000), Easing::Linear), (0, 1, 1.0));
    }
}
