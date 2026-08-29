//! @author 十四叔
//! @date 2026/08/30
//!
//! N 声道混音器 (纯逻辑): 每声道独立增益包络 (300ms 线性, 目标变化触发,
//! 反向边沿从当前值续接无跳变 —— pomodoro AmbientMixer 范式的 N 声道推广)。
//!
//! 桌景声景模型: 底 loop (雨声) + 时辰点缀层 (鸟叫/蝉鸣/虫声) 各占一声道,
//! 时辰 crossfade = 各声道目标增益随时间推移; 一次性事件音 (雷声) 不走
//! 包络, 直接经输出层播放。
//!
//! 时间由外部注入, 不读 wall-clock, 可完整单元测试。

use std::collections::BTreeMap;
use std::time::Duration;

/// 增益包络时长 (淡入/淡出对称; pomodoro 同款 300ms)。
const ENVELOPE_DURATION: Duration = Duration::from_millis(300);

/// 单声道增益包络: 目标变化触发线性包络, 反向边沿从当前值续接。
///
/// 目标可持续微变 (时辰点缀层增益随时辰曲线逐帧滑动): 每帧微变 =
/// 300ms 时间常数的平滑跟随器, 无跳变; 目标静止时包络精确到达。
#[derive(Debug, Clone)]
struct ChannelEnvelope {
    /// 包络当前值 (0..1)。
    current: f32,
    /// 产品登记的目标增益 (全局关闭时帧内按 0 覆写, 不改本值)。
    target: f32,
    /// 进行中的包络动画: (起始值, 目标值, 开始时刻)。
    anim: Option<(f32, f32, Duration)>,
}

impl ChannelEnvelope {
    fn new() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            anim: None,
        }
    }

    /// 推进包络并返回当前增益。目标变化 (含中途反向) → 从当前值起
    /// 300ms 线性包络续接; 稳定态精确到达 (无渐近漂移)。
    fn gain(&mut self, now: Duration, effective_target: f32) -> f32 {
        let needs_anim = match self.anim {
            Some((_, tv, _)) => tv != effective_target, // 目标变了 → 重触发
            None => self.current != effective_target,   // 未到目标 → 触发
        };
        if needs_anim {
            self.anim = Some((self.current, effective_target, now));
        }
        if let Some((start_v, target_v, start_t)) = self.anim {
            let t = (now.saturating_sub(start_t).as_secs_f32() / ENVELOPE_DURATION.as_secs_f32())
                .clamp(0.0, 1.0);
            self.current = start_v + (target_v - start_v) * t;
            if t >= 1.0 {
                self.anim = None;
                self.current = target_v;
            }
        }
        self.current
    }
}

/// N 声道混音器 (纯逻辑): 每帧输出各声道当前增益。
///
/// 声道经 `set_target` 隐式注册 (首次出现即建档, 从 0 淡入); 帧输出按
/// 声道号升序 (确定性, 回放/日志可比对)。全局开关 `set_enabled(false)`
/// = 所有声道目标强制 0 (走包络淡出, 非硬切 —— 桌面应用不可有爆音路径)。
pub struct Mixer {
    channels: BTreeMap<u32, ChannelEnvelope>,
    enabled: bool,
}

impl Mixer {
    /// 创建混音器: 零声道, 全局开。
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
            enabled: true,
        }
    }

    /// 设置全局开关 (false = 全部声道包络淡出至静音)。
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 设置声道目标增益 (夹到 0..1; 声道首次出现即注册, 从 0 淡入)。
    /// 目标可每帧重设 (时辰曲线滑动) —— 包络自动平滑跟随。
    pub fn set_target(&mut self, channel: u32, gain: f32) {
        self.channels
            .entry(channel)
            .or_insert_with(ChannelEnvelope::new)
            .target = gain.clamp(0.0, 1.0);
    }

    /// 每帧取所有声道当前增益 (推进包络; 按声道号升序)。
    pub fn frame_gains(&mut self, now: Duration) -> Vec<(u32, f32)> {
        self.channels
            .iter_mut()
            .map(|(&id, env)| {
                let target = if self.enabled { env.target } else { 0.0 };
                (id, env.gain(now, target))
            })
            .collect()
    }
}

impl Default for Mixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// 帧输出中某声道的增益 (缺失 = 0)。
    fn gain_of(gains: &[(u32, f32)], channel: u32) -> f32 {
        gains
            .iter()
            .find(|(id, _)| *id == channel)
            .map(|(_, g)| *g)
            .unwrap_or(0.0)
    }

    /// 新声道从 0 淡入: 边沿帧连续 (从 0 起), 300ms 到目标, 之后精确稳定。
    #[test]
    fn new_channel_fades_in_over_300ms() {
        let mut m = Mixer::new();
        m.set_target(1, 0.8);
        let g = m.frame_gains(ms(0));
        assert_eq!(gain_of(&g, 1), 0.0, "边沿帧从 0 起");
        let g = m.frame_gains(ms(150));
        assert!((gain_of(&g, 1) - 0.4).abs() < 1e-6, "中点半量");
        let g = m.frame_gains(ms(300));
        assert_eq!(gain_of(&g, 1), 0.8, "终点精确到达");
        let g = m.frame_gains(ms(999_999));
        assert_eq!(gain_of(&g, 1), 0.8, "稳定态不漂移");
    }

    /// 淡出到 0: 目标归零, 300ms 平滑无声。
    #[test]
    fn fade_out_to_zero() {
        let mut m = Mixer::new();
        m.set_target(1, 0.8);
        m.frame_gains(ms(0));
        m.frame_gains(ms(300)); // 全量
        m.set_target(1, 0.0);
        let g = m.frame_gains(ms(1000));
        assert_eq!(gain_of(&g, 1), 0.8, "边沿帧连续 (从全量起淡)");
        let g = m.frame_gains(ms(1300));
        assert_eq!(gain_of(&g, 1), 0.0, "终点静音");
    }

    /// 包络中途反向: 从当前值续接 (无跳变), 固定 300ms 重新走。
    #[test]
    fn retrigger_mid_envelope_continues_from_current() {
        let mut m = Mixer::new();
        m.set_target(1, 1.0);
        m.frame_gains(ms(0));
        m.frame_gains(ms(300)); // 全量 1.0
        m.set_target(1, 0.0);
        let g = m.frame_gains(ms(1000));
        assert_eq!(gain_of(&g, 1), 1.0);
        let g = m.frame_gains(ms(1150)); // 淡出中点 0.5
        let mid = gain_of(&g, 1);
        assert!((mid - 0.5).abs() < 1e-6);
        m.set_target(1, 1.0); // 恢复
        let g = m.frame_gains(ms(1200));
        assert!(
            (gain_of(&g, 1) - mid).abs() < 1e-6,
            "反向边沿应连续: {mid} -> {}",
            gain_of(&g, 1)
        );
    }

    /// 声道独立: A 淡出中 B 不受影响, 各有包络。
    #[test]
    fn channels_have_independent_envelopes() {
        let mut m = Mixer::new();
        m.set_target(1, 1.0);
        m.set_target(2, 0.6);
        m.frame_gains(ms(0));
        m.frame_gains(ms(300)); // 双双全量
        m.set_target(1, 0.0); // A 开始淡出
        let g = m.frame_gains(ms(400));
        assert!((gain_of(&g, 2) - 0.6).abs() < 1e-6, "B 不受 A 影响");
        assert_eq!(gain_of(&g, 1), 1.0, "A 边沿帧连续");
        let g = m.frame_gains(ms(700));
        assert_eq!(gain_of(&g, 1), 0.0, "A 淡完");
        assert!((gain_of(&g, 2) - 0.6).abs() < 1e-6, "B 仍全量");
    }

    /// 全局开关: 关闭 = 所有声道目标强制 0 (包络淡出, 非硬切);
    /// 重开从当前值淡入。静音是桌面应用的生存伦理 —— 不可有爆音路径。
    #[test]
    fn master_disable_fades_all_out() {
        let mut m = Mixer::new();
        m.set_target(1, 1.0);
        m.set_target(2, 0.5);
        m.frame_gains(ms(0));
        m.frame_gains(ms(300)); // 全量
        m.set_enabled(false);
        let g = m.frame_gains(ms(400));
        assert_eq!(gain_of(&g, 1), 1.0, "关闭边沿连续");
        let g = m.frame_gains(ms(700));
        assert_eq!(gain_of(&g, 1), 0.0);
        assert_eq!(gain_of(&g, 2), 0.0, "全部淡出");
        m.set_enabled(true);
        let g = m.frame_gains(ms(1000));
        assert_eq!(gain_of(&g, 1), 0.0, "重开边沿从 0 续接");
        let g = m.frame_gains(ms(1300));
        assert_eq!(gain_of(&g, 1), 1.0, "淡回目标");
        assert!((gain_of(&g, 2) - 0.5).abs() < 1e-6);
    }

    /// 帧输出确定性排序 (声道号升序) —— 回放/日志可比对。
    #[test]
    fn frame_gains_sorted_by_channel() {
        let mut m = Mixer::new();
        m.set_target(7, 0.3);
        m.set_target(1, 0.5);
        m.set_target(3, 0.9);
        let g = m.frame_gains(ms(0));
        let ids: Vec<u32> = g.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec![1, 3, 7]);
    }

    /// 目标值夹到 0..1 (负/超界防护)。
    #[test]
    fn target_gain_clamped() {
        let mut m = Mixer::new();
        m.set_target(1, 1.7);
        m.frame_gains(ms(0));
        let g = m.frame_gains(ms(300));
        assert_eq!(gain_of(&g, 1), 1.0, "超界夹到 1");
        m.set_target(1, -0.5);
        m.frame_gains(ms(1000));
        let g = m.frame_gains(ms(1300));
        assert_eq!(gain_of(&g, 1), 0.0, "负值夹到 0");
    }

    /// 未注册的声道从不出现在帧输出 (零声道 = 空帧)。
    #[test]
    fn unregistered_channel_absent() {
        let mut m = Mixer::new();
        assert!(m.frame_gains(ms(0)).is_empty(), "零声道 = 空帧");
    }
}
