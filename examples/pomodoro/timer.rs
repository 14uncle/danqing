//! @author 十四叔
//! @date 2026/07/23

//! 番茄钟状态机 (纯逻辑)。
//!
//! 时间由外部注入 (`Duration` 累计值, 通常来自 `AnimationCtx::elapsed`),
//! 不读 wall-clock, 可完整单元测试。语义:
//! - 固定专注 25:00 / 休息 5:00, 阶段结束自动流转并自动开始下一阶段;
//! - `toggle` 在开始 / 暂停间切换 (开始即恢复);
//! - `reset` 回到专注 25:00 停止态;
//! - tick 越过终点时余量带入下一阶段 (晚到的帧不吃时间)。
//!
//! NOTE: Task 3 骨架只消费 `new`/`display`, 完整 API 由 Task 6 界面组装接入;
//! 届时移除本模块的 `allow(dead_code)`。
#![allow(dead_code)]

use std::time::Duration;

/// 计时阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Self::Focus => Duration::from_secs(25 * 60),
            Self::Break => Duration::from_secs(5 * 60),
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
enum Run {
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
    /// 创建番茄钟: 专注 25:00 停止态。
    pub fn new() -> Self {
        Self {
            phase: Phase::Focus,
            run: Run::Idle,
            remaining: Phase::Focus.duration(),
            deadline: None,
        }
    }

    /// 当前阶段。
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// 是否计时中。
    pub fn is_running(&self) -> bool {
        self.run == Run::Running
    }

    /// 开始 / 暂停切换: Idle 或 Paused 进入计时, Running 快照剩余并暂停。
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

    /// 重置: 回到专注 25:00 停止态。
    pub fn reset(&mut self) {
        *self = Self::new();
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
    fn new_is_focus_25_min_idle() {
        let p = Pomodoro::new();
        assert_eq!(p.phase(), Phase::Focus);
        assert!(!p.is_running());
        assert_eq!(p.remaining(secs(0)), secs(25 * 60));
        assert_eq!(p.display(secs(0)), "25:00");
    }

    #[test]
    fn toggle_starts_then_pauses() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        assert!(p.is_running());
        p.toggle(secs(10));
        assert!(!p.is_running());
        assert_eq!(p.remaining(secs(10)), secs(25 * 60 - 10));
    }

    #[test]
    fn paused_remaining_is_frozen() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        p.toggle(secs(60));
        // 暂停后时间推移不改变剩余。
        assert_eq!(p.remaining(secs(600)), secs(24 * 60));
    }

    #[test]
    fn resume_continues_from_paused_remaining() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        p.toggle(secs(60));
        p.toggle(secs(600)); // 600s 处恢复
        assert!(p.is_running());
        assert_eq!(p.remaining(secs(610)), secs(24 * 60 - 10));
    }

    #[test]
    fn tick_before_deadline_does_not_advance() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        assert!(!p.tick(secs(25 * 60 - 1)));
        assert_eq!(p.phase(), Phase::Focus);
    }

    #[test]
    fn tick_past_deadline_auto_advances_and_keeps_running() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        assert!(p.tick(secs(25 * 60)));
        assert_eq!(p.phase(), Phase::Break);
        assert!(p.is_running());
        assert_eq!(p.remaining(secs(25 * 60)), secs(5 * 60));
    }

    #[test]
    fn overshoot_carries_into_next_phase() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        // 帧晚到 3 秒: 下一阶段从原终点顺延, 余量不亏。
        assert!(p.tick(secs(25 * 60 + 3)));
        assert_eq!(p.remaining(secs(25 * 60 + 3)), secs(5 * 60 - 3));
    }

    #[test]
    fn break_completion_returns_to_focus() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        p.tick(secs(25 * 60));
        assert!(p.tick(secs(30 * 60)));
        assert_eq!(p.phase(), Phase::Focus);
        assert!(p.is_running());
        assert_eq!(p.remaining(secs(30 * 60)), secs(25 * 60));
    }

    #[test]
    fn huge_overshoot_rolls_multiple_phases() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        // 58 分钟后回来: 专注(0-25) → 休息(25-30) → 专注(30-55) → 休息(55-60),
        // 当前处于第二段休息, 剩 2 分钟。
        assert!(p.tick(secs(58 * 60)));
        assert_eq!(p.phase(), Phase::Break);
        assert_eq!(p.remaining(secs(58 * 60)), secs(2 * 60));
    }

    #[test]
    fn reset_returns_to_focus_idle() {
        let mut p = Pomodoro::new();
        p.toggle(secs(0));
        p.tick(secs(25 * 60));
        p.reset();
        assert_eq!(p.phase(), Phase::Focus);
        assert!(!p.is_running());
        assert_eq!(p.remaining(secs(999)), secs(25 * 60));
    }

    #[test]
    fn display_formats_mm_ss() {
        let mut p = Pomodoro::new();
        assert_eq!(p.display(secs(0)), "25:00");
        p.toggle(secs(0));
        assert_eq!(p.display(secs(1)), "24:59");
        assert_eq!(p.display(secs(20 * 60)), "05:00");
    }

    #[test]
    fn phase_labels_are_chinese() {
        assert_eq!(Phase::Focus.label(), "专注");
        assert_eq!(Phase::Break.label(), "休息");
    }
}
