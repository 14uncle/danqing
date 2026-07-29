//! @author 十四叔
//! @date 2026/07/28

//! 场景动效策略 (纯逻辑): 哪个场景下雨/烧火/涌动、暂停沉降包络、强度权重合成。
//!
//! 与环境音同一美学契约 (潮汐式): 计时运行时世界环绕, 暂停/空闲时
//! 世界退远 —— 雨/火/海效包络以 `timer.is_running()` 为目标, 500ms 滑动
//! (2026-07-28 spec 裁定: 视觉沉降独立时长, 不复用音频 300ms)。
//! 时间由外部注入 (`AnimationCtx.elapsed` 累计值), 不读 wall-clock,
//! 可完整单元测试。
//!
//! 雨、火、海是并存标量而非互斥选择子: 交叉淡化期间两端可同时非零
//! (spec: docs/specs/pomodoro-scene-motion-bonfire.md,
//!  docs/specs/pomodoro-scene-motion-sea.md)。

use std::time::Duration;

/// 雨场景在 `SCENES` 中的索引 (单测锁定名称, 防生成器重排静默错位)。
pub const RAIN_SCENE: usize = 2;

/// 篝火场景在 `SCENES` 中的索引 (单测锁定名称, 防生成器重排静默错位)。
pub const BONFIRE_SCENE: usize = 0;

/// 海场景在 `SCENES` 中的索引 (单测锁定名称, 防生成器重排静默错位)。
pub const SEA_SCENE: usize = 1;

/// 暂停沉降时长 (视觉 500ms; 音频包络 300ms 见 ambient.rs, 两者独立)。
pub const SETTLE_DURATION: Duration = Duration::from_millis(500);

/// 动效沉降包络: 计时运行 = 全量 (1), 暂停/空闲 = 0;
/// 目标变化触发 500ms 滑动, 反向边沿从当前值续接 (无跳变)。
/// 与 `ambient::AmbientMixer` 的包络段同范式。
#[derive(Debug, Clone)]
pub struct MotionEnvelope {
    /// 包络当前值 (0..1, 1 = 全量雨效)。
    value: f32,
    /// 进行中的包络动画: (起始值, 目标值, 开始时刻)。
    anim: Option<(f32, f32, Duration)>,
    /// 上一帧见到的目标值 (边沿检测)。
    last_target: f32,
}

impl MotionEnvelope {
    /// 创建包络: 初始 0 (无雨效), 等待首次 running 边沿淡入。
    pub fn new() -> Self {
        Self {
            value: 0.0,
            anim: None,
            last_target: 0.0,
        }
    }

    /// 推进包络并返回当前值 (0..=1)。
    ///
    /// 目标 = running ? 1 : 0; 目标变化触发 500ms 滑动动画,
    /// 动画进行中反向边沿从当前值续接 (无跳变)。
    pub fn gain(&mut self, running: bool, now: Duration) -> f32 {
        let target = if running { 1.0 } else { 0.0 };
        if target != self.last_target {
            self.anim = Some((self.value, target, now));
            self.last_target = target;
        }
        if let Some((start_v, target_v, start_t)) = self.anim {
            let t = (now.saturating_sub(start_t).as_secs_f32() / SETTLE_DURATION.as_secs_f32())
                .clamp(0.0, 1.0);
            self.value = start_v + (target_v - start_v) * t;
            if t >= 1.0 {
                self.anim = None;
            }
        }
        self.value
    }
}

impl Default for MotionEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

/// 场景在交叉淡化中的权重: from 随 fade 淡出, to 随 fade 淡入, 静止时恒 1。
fn scene_weight(scene: usize, from: usize, to: usize, fade: f32) -> f32 {
    let w = |idx: usize| if idx == scene { 1.0 } else { 0.0 };
    w(from) * (1.0 - fade) + w(to) * fade
}

/// 雨效强度合成: 包络 × 雨场景淡化权重。
pub fn rain_intensity(from: usize, to: usize, fade: f32, envelope: f32) -> f32 {
    envelope * scene_weight(RAIN_SCENE, from, to, fade)
}

/// 火效强度合成: 包络 × 篝火场景淡化权重。
pub fn fire_intensity(from: usize, to: usize, fade: f32, envelope: f32) -> f32 {
    envelope * scene_weight(BONFIRE_SCENE, from, to, fade)
}

/// 海效强度合成: 包络 × 海场景淡化权重。
pub fn sea_intensity(from: usize, to: usize, fade: f32, envelope: f32) -> f32 {
    envelope * scene_weight(SEA_SCENE, from, to, fade)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenes::SCENES;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn rain_scene_index_points_at_rain() {
        assert_eq!(SCENES[RAIN_SCENE].name, "雨");
        // 雨场景唯一: 其余场景不会被误判。
        assert_eq!(SCENES.iter().filter(|s| s.name == "雨").count(), 1);
    }

    #[test]
    fn bonfire_scene_index_points_at_bonfire() {
        assert_eq!(SCENES[BONFIRE_SCENE].name, "篝火");
        // 篝火场景唯一: 其余场景不会被误判。
        assert_eq!(SCENES.iter().filter(|s| s.name == "篝火").count(), 1);
    }

    #[test]
    fn sea_scene_index_points_at_sea() {
        assert_eq!(SCENES[SEA_SCENE].name, "海");
        // 海场景唯一: 其余场景不会被误判。
        assert_eq!(SCENES.iter().filter(|s| s.name == "海").count(), 1);
    }

    #[test]
    fn sea_intensity_weights_by_scene_and_fade() {
        // 海为 from: 随 fade 淡出。
        assert!((sea_intensity(SEA_SCENE, 0, 0.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((sea_intensity(SEA_SCENE, 0, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!(sea_intensity(SEA_SCENE, 0, 1.0, 1.0).abs() < 1e-6);
        // 海为 to: 随 fade 淡入。
        assert!((sea_intensity(0, SEA_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
        // 双非海: 恒 0。
        assert_eq!(sea_intensity(0, 3, 0.5, 1.0), 0.0);
        // 静止于海 (from == to): 权重恒 1, 只随包络缩放。
        assert!((sea_intensity(SEA_SCENE, SEA_SCENE, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn sea_coexists_with_rain_and_fire_on_crossfade() {
        // 海↔雨、海↔火交叉淡化中点: 两效果各 0.5 并存 (标量并存, 非互斥选择子)。
        assert!((sea_intensity(SEA_SCENE, RAIN_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((rain_intensity(SEA_SCENE, RAIN_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((sea_intensity(BONFIRE_SCENE, SEA_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((fire_intensity(BONFIRE_SCENE, SEA_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn fire_intensity_weights_by_scene_and_fade() {
        // 篝火为 from: 随 fade 淡出。
        assert!((fire_intensity(BONFIRE_SCENE, 1, 0.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((fire_intensity(BONFIRE_SCENE, 1, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!(fire_intensity(BONFIRE_SCENE, 1, 1.0, 1.0).abs() < 1e-6);
        // 篝火为 to: 随 fade 淡入。
        assert!((fire_intensity(1, BONFIRE_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
        // 双非篝火: 恒 0。
        assert_eq!(fire_intensity(1, 3, 0.5, 1.0), 0.0);
        // 静止于篝火 (from == to): 权重恒 1, 只随包络缩放。
        assert!((fire_intensity(BONFIRE_SCENE, BONFIRE_SCENE, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn rain_and_fire_coexist_on_crossfade() {
        // 雨→篝火交叉淡化中点: 两效果各 0.5 并存 (spec: 标量并存, 非互斥选择子)。
        assert!((rain_intensity(RAIN_SCENE, BONFIRE_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!((fire_intensity(RAIN_SCENE, BONFIRE_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn envelope_idle_stays_zero() {
        let mut e = MotionEnvelope::new();
        assert_eq!(e.gain(false, ms(0)), 0.0);
        assert_eq!(e.gain(false, ms(10_000)), 0.0);
    }

    #[test]
    fn envelope_fades_in_over_500ms() {
        let mut e = MotionEnvelope::new();
        assert_eq!(e.gain(true, ms(0)), 0.0); // 边沿帧从 0 起
        assert!((e.gain(true, ms(250)) - 0.5).abs() < 1e-6);
        assert!((e.gain(true, ms(500)) - 1.0).abs() < 1e-6);
        assert!((e.gain(true, ms(9999)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn envelope_pause_fades_out_and_resume_continues_from_current() {
        let mut e = MotionEnvelope::new();
        e.gain(true, ms(0));
        e.gain(true, ms(500)); // 全量
        // 暂停边沿: 从 1 续接, 不跳变。
        assert!((e.gain(false, ms(1000))) - 1.0 < 1e-6);
        // 淡出中点 (250ms) = 0.5。
        let mid = e.gain(false, ms(1250));
        assert!((mid - 0.5).abs() < 1e-6);
        // 淡出中恢复: 从当前值续接淡入, 不跳变; 500ms 后回全量。
        let v = e.gain(true, ms(1300));
        assert!((v - mid).abs() < 1e-6, "反向边沿应连续: {mid} -> {v}");
        assert!((e.gain(true, ms(1800)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn rain_intensity_weights_by_scene_and_fade() {
        // 雨为 from: 随 fade 淡出。
        assert!((rain_intensity(RAIN_SCENE, 0, 0.0, 1.0) - 1.0).abs() < 1e-6);
        assert!((rain_intensity(RAIN_SCENE, 0, 0.5, 1.0) - 0.5).abs() < 1e-6);
        assert!(rain_intensity(RAIN_SCENE, 0, 1.0, 1.0).abs() < 1e-6);
        // 雨为 to: 随 fade 淡入。
        assert!((rain_intensity(0, RAIN_SCENE, 0.5, 1.0) - 0.5).abs() < 1e-6);
        // 双非雨: 恒 0。
        assert_eq!(rain_intensity(0, 1, 0.5, 1.0), 0.0);
        // 静止于雨 (from == to): 权重恒 1, 只随包络缩放。
        assert!((rain_intensity(RAIN_SCENE, RAIN_SCENE, 1.0, 0.5) - 0.5).abs() < 1e-6);
    }
}
