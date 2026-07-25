//! @author 十四叔
//! @date 2026/07/23

//! 番茄钟状态机 (纯逻辑)。
//!
//! 时间由外部注入 (`Duration` 累计值，通常来自 `AnimationCtx::elapsed`),
//! 不读 wall-clock, 可完整单元测试。语义：
//! - 固定专注 25:00 / 休息 5:00, 阶段结束自动流转并自动开始下一阶段;
//! - `toggle` 在开始 / 暂停间切换 (开始即恢复);
//! - `reset` 回到专注 25:00 停止态;
//! - tick 越过终点时余量带入下一阶段 (晚到的帧不吃时间)。

use std::time::Duration;

use serde::{Deserialize, Serialize};

// === 阶段时长常量 ===
const FOCUS_DURATION_SECS: u64 = 25 * 60; // 25 分钟
const BREAK_DURATION_SECS: u64 = 5 * 60; // 5 分钟

/// 计时阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    /// 专注 (25 分钟)。
    Focus,
    /// 休息 (5 分钟)。
    Break,
}

impl Phase {
    /// 阶段时长。
    pub fn duration(self) -> Duration {
        match self {
            Self::Focus => Duration::from_secs(FOCUS_DURATION_SECS),
            Self::Break => Duration::from_secs(BREAK_DURATION_SECS),
        }
    }

    /// 下一阶段 (专注 → 休息 → 专注)。
    pub fn next(self) -> Self {
        match self {
            Self::Focus => Self::Break,
            Self::Break => Self::Focus,
        }
    }

    /// 中文标注。
    pub fn label(self) -> &'static str {
        match self {
            Self::Focus => "专注",
            Self::Break => "休息",
        }
    }
}

/// 运行状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    /// 停止 (未开始或被重置)。
    Idle,
    /// 计时中。
    Running,
    /// 暂停 (剩余时间已快照)。
    Paused,
}

/// 番茄钟状态机。
#[derive(Debug, Clone)]
pub struct Pomodoro {
    phase: Phase,
    run: Run,
    /// 非 Running 时的剩余时间快照; Running 时由 deadline 推算。
    remaining: Duration,
    /// Running 时的截止点 (注入时间轴上的绝对位置)。
    deadline: Option<Duration>,
}

impl Pomodoro {
    /// 创建番茄钟：专注 25:00 停止态。
    pub fn new() -> Self {
        Self {
            phase: Phase::Focus,
            run: Run::Idle,
            remaining: Phase::Focus.duration(),
            deadline: None,
        }
    }

    /// 从持久化恢复任意状态 (用于跨重启恢复)。
    pub fn restore(
        phase: Phase,
        run: Run,
        remaining: Duration,
        deadline: Option<Duration>,
    ) -> Self {
        Self {
            phase,
            run,
            remaining,
            deadline,
        }
    }

    /// 当前阶段。
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// 当前运行状态。
    pub fn run(&self) -> Run {
        self.run
    }

    /// 是否计时中。
    pub fn is_running(&self) -> bool {
        self.run == Run::Running
    }

    /// 开始 / 暂停切换：Idle 或 Paused 进入计时，Running 快照剩余并暂停。
    pub fn toggle(&mut self, now: Duration) {
        match self.run {
            Run::Idle | Run::Paused => {
                self.deadline = Some(now + self.remaining);
                self.run = Run::Running;
            }
            Run::Running => {
                self.remaining = self.remaining_at(now);
                self.deadline = None;
                self.run = Run::Paused;
            }
        }
    }

    /// 重置：回到专注 25:00 停止态。
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// 立即跳过当前阶段剩余时间, 进入下一阶段。
    /// - Running: `deadline = now + next_phase.duration` (从满量重新开始)
    /// - Paused / Idle: `remaining = next_phase.duration`, `deadline = None`
    ///
    /// 返回是否发生阶段切换 (始终为 true, 排除自身相等)。
    pub fn skip(&mut self, now: Duration) -> bool {
        self.phase = self.phase.next();
        let next_duration = self.phase.duration();
        match self.run {
            Run::Running => {
                self.deadline = Some(now + next_duration);
            }
            Run::Paused | Run::Idle => {
                self.remaining = next_duration;
                self.deadline = None;
            }
        }
        true
    }

    /// 推进计时; 越过阶段终点时自动流转并自动开始下一阶段。
    ///
    /// 返回是否发生了阶段流转。余量带入下一阶段 (deadline 顺延),
    /// 连续越过多个终点时循环处理。
    pub fn tick(&mut self, now: Duration) -> bool {
        let mut advanced = false;
        while self.run == Run::Running && now >= self.deadline.unwrap_or(now) {
            let deadline = self.deadline.unwrap_or(now);
            self.phase = self.phase.next();
            self.deadline = Some(deadline + self.phase.duration());
            advanced = true;
        }
        if advanced {
            self.remaining = self.remaining_at(now);
        }
        advanced
    }

    /// 当前剩余时间。
    pub fn remaining(&self, now: Duration) -> Duration {
        match self.run {
            Run::Running => self.remaining_at(now),
            _ => self.remaining,
        }
    }

    /// `mm:ss` 显示 (剩余秒数向下取整)。
    pub fn display(&self, now: Duration) -> String {
        let secs = self.remaining(now).as_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    /// Running 状态下由 deadline 推算剩余 (饱和减法)。
    fn remaining_at(&self, now: Duration) -> Duration {
        self.deadline.unwrap_or(now).saturating_sub(now)
    }
}

impl Default for Pomodoro {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(s: u64) -> Duration {
        Duration::from_secs(s)
    }

    #[test]
    fn new_is_focus_idle() {
        let p = Pomodoro::new();
        assert_eq!(p.phase(), Phase::Focus);
        assert!(!p.is_running());
        assert_eq!(p.remaining(secs(0)), secs(FOCUS_DURATION_SECS));
        assert_eq!(p.display(secs(0)), "25:00");
    }

    #[test]
    fn toggle_starts_then_pauses() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        assert!(p.is_running());
        let pause_at = FOCUS_DURATION_SECS / 2;
        p.toggle(secs(pause_at));
        assert!(!p.is_running());
        assert_eq!(
            p.remaining(secs(pause_at)),
            secs(FOCUS_DURATION_SECS - pause_at)
        );
    }

    #[test]
    fn paused_remaining_is_frozen() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        p.toggle(secs(FOCUS_DURATION_SECS / 2));
        // 暂停后时间推移不改变剩余。
        let remaining_at_pause = FOCUS_DURATION_SECS - FOCUS_DURATION_SECS / 2;
        assert_eq!(p.remaining(secs(999)), secs(remaining_at_pause));
    }

    #[test]
    fn resume_continues_from_paused_remaining() {
        let mut p = Pomodoro::new();
        let pause_at = FOCUS_DURATION_SECS / 2;
        p.toggle(secs(0));
        p.toggle(secs(pause_at));
        p.toggle(secs(999)); // 很久后恢复
        assert!(p.is_running());
        // 恢复后 1s 内剩余 = 暂停时剩余 - 1
        assert_eq!(
            p.remaining(secs(1000)),
            secs(FOCUS_DURATION_SECS - pause_at - 1)
        );
    }

    #[test]
    fn tick_before_deadline_does_not_advance() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        assert!(!p.tick(secs(FOCUS_DURATION_SECS - 1)));
        assert_eq!(p.phase(), Phase::Focus);
    }

    #[test]
    fn tick_past_deadline_auto_advances_and_keeps_running() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        assert!(p.tick(secs(FOCUS_DURATION_SECS)));
        assert_eq!(p.phase(), Phase::Break);
        assert!(p.is_running());
        assert_eq!(
            p.remaining(secs(FOCUS_DURATION_SECS)),
            secs(BREAK_DURATION_SECS)
        );
    }

    #[test]
    fn overshoot_carries_into_next_phase() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        // 帧晚到 3 秒: 下一阶段从原终点顺延, 余量不亏。
        let overshoot = 3u64;
        let tick_at = FOCUS_DURATION_SECS + overshoot;
        assert!(p.tick(secs(tick_at)));
        assert_eq!(
            p.remaining(secs(tick_at)),
            secs(BREAK_DURATION_SECS - overshoot)
        );
    }

    #[test]
    fn break_completion_returns_to_focus() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        p.tick(secs(FOCUS_DURATION_SECS));
        let cycle = FOCUS_DURATION_SECS + BREAK_DURATION_SECS;
        assert!(p.tick(secs(cycle)));
        assert_eq!(p.phase(), Phase::Focus);
        assert!(p.is_running());
        assert_eq!(p.remaining(secs(cycle)), secs(FOCUS_DURATION_SECS));
    }

    #[test]
    fn huge_overshoot_rolls_multiple_phases() {
        // 在第二轮 break, 剩 2 分钟 (focus 2: 30-55, break 2: 55-60, 60-58=2)
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        let cycle = FOCUS_DURATION_SECS + BREAK_DURATION_SECS;
        let tick_at = 2 * cycle - 2 * 60; // 2 个完整 cycle 减 2 分钟 = 58 分钟 = 3480s
        assert!(p.tick(secs(tick_at)));
        assert_eq!(p.phase(), Phase::Break);
        assert_eq!(p.remaining(secs(tick_at)), secs(2 * 60));
    }

    #[test]
    fn reset_returns_to_focus_idle() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        p.tick(secs(FOCUS_DURATION_SECS));
        p.reset();
        assert_eq!(p.phase(), Phase::Focus);
        assert!(!p.is_running());
        assert_eq!(p.remaining(secs(999)), secs(FOCUS_DURATION_SECS));
    }

    #[test]
    fn display_formats_mm_ss() {
        let mut p = Pomodoro::new();
        assert_eq!(p.display(secs(0)), "25:00");
        p.toggle(secs(0));
        assert_eq!(p.display(secs(1)), "24:59");
    }

    #[test]
    fn phase_labels_are_chinese() {
        assert_eq!(Phase::Focus.label(), "专注");
        assert_eq!(Phase::Break.label(), "休息");
    }

    #[test]
    fn restore_preserves_all_fields() {
        let p = Pomodoro::restore(Phase::Break, Run::Paused, Duration::from_secs(120), None);
        assert_eq!(p.phase(), Phase::Break);
        assert_eq!(p.run(), Run::Paused);
        assert!(!p.is_running());
        assert_eq!(
            p.remaining(Duration::from_secs(9999)),
            Duration::from_secs(120)
        );
    }

    #[test]
    fn restore_running_with_deadline_resumes_correctly() {
        let now = Duration::from_secs(1000);
        let p = Pomodoro::restore(
            Phase::Focus,
            Run::Running,
            Duration::from_secs(600),
            Some(now + Duration::from_secs(600)),
        );
        assert!(p.is_running());
        assert_eq!(p.remaining(now), Duration::from_secs(600));
    }

    #[test]
    fn skip_in_running_advances_deadline() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        let skip_at = FOCUS_DURATION_SECS / 2;
        assert!(p.skip(secs(skip_at)));
        assert_eq!(p.phase(), Phase::Break);
        assert!(p.is_running());
        // 新阶段从 now 起算, 满量 BREAK_DURATION_SECS
        assert_eq!(p.remaining(secs(skip_at)), secs(BREAK_DURATION_SECS));
    }

    #[test]
    fn skip_in_paused_advances_remaining() {
        let mut p = Pomodoro::new();
        let skip_at = FOCUS_DURATION_SECS / 2;
        p.toggle(secs(0));
        p.toggle(secs(skip_at)); // 暂停
        assert!(p.skip(secs(skip_at)));
        assert_eq!(p.phase(), Phase::Break);
        assert!(!p.is_running());
        // 暂停态下 remaining = 下一阶段满量
        assert_eq!(p.remaining(secs(999)), secs(BREAK_DURATION_SECS));
    }

    #[test]
    fn skip_in_idle_advances_phase_and_remaining() {
        let mut p = Pomodoro::new();
        assert!(p.skip(secs(0)));
        assert_eq!(p.phase(), Phase::Break);
        assert!(!p.is_running());
        assert_eq!(p.remaining(secs(999)), secs(BREAK_DURATION_SECS));
    }

    #[test]
    fn skip_consecutive_cycles_through_phases() {
        let mut p = Pomodoro::new();
        assert!(p.skip(secs(0)));
        assert_eq!(p.phase(), Phase::Break);
        assert!(p.skip(secs(0)));
        assert_eq!(p.phase(), Phase::Focus);
        assert!(p.skip(secs(0)));
        assert_eq!(p.phase(), Phase::Break);
    }
}
