//! @author 十四叔
//! @date 2026/07/27

//! 场景环境音混音器 (纯逻辑)。
//!
//! 每帧把视觉淡化的 `(from, to, fade)` 与运行态转成两个音频槽的音量:
//! 音量 = 淡化权重 × 暂停沉降包络 × `AMBIENT_VOLUME`。
//! - 淡化权重: 静止 (from == to, fade = 1) 时全量落在 to 槽;
//!   切换中按 fade 在 from/to 间分配, 与画面 800ms 交叉淡化同源同步。
//! - 暂停沉降包络: running 边沿触发 300ms 线性 fade-in/fade-out,
//!   与视觉降饱和同条件 (`is_running`), 稳定态精确为 0.0 / 1.0。
//!
//! 时间由外部注入, 不读 wall-clock, 可完整单元测试。
//!
//! 下半部分为 rodio 输出适配层 (`AmbientPlayer`): 懒初始化输出流 +
//! from/to 双槽 `Player` (与视觉场景纹理 LRU 同构) + 静默降级。

use std::time::Duration;

/// 场景音源路径 (与 `scenes::SCENES` 索引对齐: 篝火/海/雨/山/森林)。
pub const SCENE_AUDIO: [&str; 5] = [
    "assets/audio/bonfire.ogg",
    "assets/audio/sea.ogg",
    "assets/audio/rain.ogg",
    "assets/audio/mountain.ogg",
    "assets/audio/forest.ogg",
];

/// 环境音目标音量 (固定, 无设置 UI)。
pub const AMBIENT_VOLUME: f32 = 0.6;

/// 暂停沉降包络时长 (淡入/淡出对称)。
const ENVELOPE_DURATION: Duration = Duration::from_millis(300);

/// 环境音混音器: 淡化权重 × 暂停沉降包络。
#[derive(Debug, Clone)]
pub struct AmbientMixer {
    /// 包络当前值 (0..1, 1 = 全量)。
    envelope: f32,
    /// 进行中的包络动画: (起始值, 目标值, 开始时刻)。
    anim: Option<(f32, f32, Duration)>,
    /// 上一帧见到的 running 状态 (边沿检测)。
    last_running: bool,
}

impl AmbientMixer {
    /// 创建混音器: 包络 0 (静音), 等待首次 running 边沿淡入。
    pub fn new() -> Self {
        Self {
            envelope: 0.0,
            anim: None,
            last_running: false,
        }
    }

    /// 计算两槽音量: `[(from, v_from), (to, v_to)]`。
    ///
    /// `fade` 为视觉淡化进度 (0..1, 经缓动); `running` 为计时运行态。
    /// running 边沿触发 300ms 包络动画; 动画进行中反向边沿从当前值续接 (无跳变)。
    pub fn frame_volumes(
        &mut self,
        from: usize,
        to: usize,
        fade: f32,
        running: bool,
        now: Duration,
    ) -> [(usize, f32); 2] {
        if running != self.last_running {
            let target = if running { 1.0 } else { 0.0 };
            self.anim = Some((self.envelope, target, now));
            self.last_running = running;
        }
        if let Some((start_v, target_v, start_t)) = self.anim {
            let t = (now.saturating_sub(start_t).as_secs_f32() / ENVELOPE_DURATION.as_secs_f32())
                .clamp(0.0, 1.0);
            self.envelope = start_v + (target_v - start_v) * t;
            if t >= 1.0 {
                self.anim = None;
            }
        }
        let gain = self.envelope * AMBIENT_VOLUME;
        [(from, (1.0 - fade) * gain), (to, fade * gain)]
    }
}

impl Default for AmbientMixer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// rodio 输出适配层
// ---------------------------------------------------------------------------

use std::fs::File;
use std::io::BufReader;

use rodio::Source;

/// 环境音播放器: 输出流 + from/to 双槽, 消费 [`AmbientMixer`] 的帧音量。
///
/// - 懒初始化: 首次出现非零音量才打开输出设备, 启动路径 (Idle 静音) 不触音频。
/// - 双槽: 槽位绑定场景, 与视觉 `(from, to)` 纹理 LRU 同构;
///   淡化结束后旧场景槽自动释放, 新场景槽按需重建 (`Decoder` 流式 + 无限循环)。
/// - 静默降级: 打开设备失败永久降级 (`disabled`); 单条音源打不开记入
///   `failed_scenes` 不再重试。所有失败仅 `log::warn`, 不 panic。
pub struct AmbientPlayer {
    /// 输出流 (懒初始化; None = 尚未打开设备)。
    stream: Option<rodio::MixerDeviceSink>,
    /// 双槽: (绑定场景索引, 播放器)。drop 即停播。
    slots: [Option<(usize, rodio::Player)>; 2],
    /// 永久降级旗标: 输出设备打开失败后置位, 之后每帧直接返回。
    disabled: bool,
    /// 打不开 (缺文件 / 解码失败) 的场景, 避免 60fps 重试刷日志。
    failed_scenes: [bool; SCENE_AUDIO.len()],
}

impl AmbientPlayer {
    /// 创建播放器: 未初始化, 未降级, 双槽为空。
    pub fn new() -> Self {
        Self {
            stream: None,
            slots: [None, None],
            disabled: false,
            failed_scenes: [false; SCENE_AUDIO.len()],
        }
    }

    /// 每帧应用混音结果: 对齐槽位与活跃场景, 设置两槽音量。
    ///
    /// 全静音且无活动槽时直接返回 (不开设备); 任一步失败仅 warn 不 panic。
    pub fn apply(&mut self, frame: [(usize, f32); 2]) {
        if self.disabled {
            return;
        }
        // 启动 Idle: 无音量且无槽, 不触碰音频设备。
        let idle = frame.iter().all(|(_, v)| *v <= 0.0) && self.slots.iter().all(Option::is_none);
        if idle {
            return;
        }
        if self.stream.is_none() {
            match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(stream) => self.stream = Some(stream),
                Err(err) => {
                    log::warn!("环境音输出设备打开失败, 永久降级: {err}");
                    self.disabled = true;
                    return;
                }
            }
        }
        let Some(stream) = self.stream.as_ref() else {
            return;
        };
        let active = [frame[0].0, frame[1].0];
        // 释放不再活跃的槽 (淡化完成后旧 from 退场)。
        for slot in &mut self.slots {
            if let Some((scene, _)) = slot {
                if !active.contains(scene) {
                    *slot = None;
                }
            }
        }
        // 活跃场景缺槽时绑定到空槽; 已知失败的场景跳过。
        for (scene, _) in frame.iter().copied() {
            if scene >= SCENE_AUDIO.len()
                || self.failed_scenes[scene]
                || self.slots.iter().flatten().any(|(s, _)| *s == scene)
            {
                continue;
            }
            let Some(slot) = self.slots.iter_mut().find(|slot| slot.is_none()) else {
                continue;
            };
            match Self::build_player(stream, scene) {
                Some(player) => *slot = Some((scene, player)),
                None => self.failed_scenes[scene] = true,
            }
        }
        // 音量每帧直写 (300ms 包络 / 800ms 淡化都由 mixer 算好)。
        for (scene, volume) in frame {
            if let Some((_, player)) = self.slots.iter().flatten().find(|(s, _)| *s == scene) {
                player.set_volume(volume);
            }
        }
    }

    /// 为场景构建循环播放槽: 打开文件 + 流式解码 + 无限循环。
    fn build_player(stream: &rodio::MixerDeviceSink, scene: usize) -> Option<rodio::Player> {
        let path = SCENE_AUDIO[scene];
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                log::warn!("环境音文件打开失败 ({path}): {err}");
                return None;
            }
        };
        let decoder = match rodio::Decoder::new(BufReader::new(file)) {
            Ok(decoder) => decoder,
            Err(err) => {
                log::warn!("环境音解码失败 ({path}): {err}");
                return None;
            }
        };
        let player = rodio::Player::connect_new(stream.mixer());
        player.append(decoder.repeat_infinite());
        Some(player)
    }
}

impl Default for AmbientPlayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl AmbientPlayer {
    /// 测试辅助: 强制永久降级, 避免 tick 路径触碰真实音频设备。
    pub fn disable_for_test(&mut self) {
        self.disabled = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenes::SCENES;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn scene_audio_array_aligns_with_scenes() {
        assert_eq!(SCENE_AUDIO.len(), SCENES.len(), "音源数组应与场景一一对应");
    }

    #[test]
    fn idle_stays_silent() {
        let mut m = AmbientMixer::new();
        for t in [0, 100, 10_000] {
            let v = m.frame_volumes(0, 0, 1.0, false, ms(t));
            assert_eq!(v, [(0, 0.0), (0, 0.0)], "Idle 应始终静音 (t={t})");
        }
    }

    #[test]
    fn running_fades_in_over_300ms() {
        let mut m = AmbientMixer::new();
        // 边沿帧: 包络从 0 起, 音量为 0。
        let v = m.frame_volumes(0, 0, 1.0, true, ms(1000));
        assert_eq!(v[1].1, 0.0);
        // 中点: 包络 0.5 → 音量 0.3。
        let v = m.frame_volumes(0, 0, 1.0, true, ms(1150));
        assert!((v[1].1 - 0.3).abs() < 1e-6, "中点应为半量: {}", v[1].1);
        // 终点及之后: 稳定全量 0.6。
        let v = m.frame_volumes(0, 0, 1.0, true, ms(1300));
        assert!((v[1].1 - AMBIENT_VOLUME).abs() < 1e-6);
        let v = m.frame_volumes(0, 0, 1.0, true, ms(999_999));
        assert!((v[1].1 - AMBIENT_VOLUME).abs() < 1e-6);
    }

    #[test]
    fn pause_fades_out_over_300ms() {
        let mut m = AmbientMixer::new();
        m.frame_volumes(0, 0, 1.0, true, ms(0));
        m.frame_volumes(0, 0, 1.0, true, ms(300)); // 淡入完成, 全量
        // 暂停边沿: 从全量起淡。
        let v = m.frame_volumes(0, 0, 1.0, false, ms(1000));
        assert!((v[1].1 - AMBIENT_VOLUME).abs() < 1e-6, "边沿帧应连续");
        let v = m.frame_volumes(0, 0, 1.0, false, ms(1150));
        assert!((v[1].1 - 0.3).abs() < 1e-6, "中点应为半量: {}", v[1].1);
        let v = m.frame_volumes(0, 0, 1.0, false, ms(1300));
        assert_eq!(v[1].1, 0.0);
    }

    #[test]
    fn fade_interpolation_splits_volume() {
        let mut m = AmbientMixer::new();
        m.frame_volumes(0, 0, 1.0, true, ms(0));
        m.frame_volumes(0, 0, 1.0, true, ms(300)); // 全量
        // 切换起点: 全量在 from。
        let v = m.frame_volumes(0, 1, 0.0, true, ms(400));
        assert!((v[0].1 - AMBIENT_VOLUME).abs() < 1e-6);
        assert_eq!(v[1].1, 0.0);
        // 中点: 两槽各半。
        let v = m.frame_volumes(0, 1, 0.5, true, ms(500));
        assert!((v[0].1 - 0.3).abs() < 1e-6);
        assert!((v[1].1 - 0.3).abs() < 1e-6);
        // 终点: 全量在 to。
        let v = m.frame_volumes(0, 1, 1.0, true, ms(600));
        assert_eq!(v[0].1, 0.0);
        assert!((v[1].1 - AMBIENT_VOLUME).abs() < 1e-6);
    }

    #[test]
    fn envelope_and_fade_are_independent() {
        let mut m = AmbientMixer::new();
        m.frame_volumes(0, 0, 1.0, true, ms(0));
        // 淡化中点 + 包络中点 (150ms): 音量 = 0.5 淡化 × 0.5 包络 × 0.6 = 0.15。
        let v = m.frame_volumes(0, 1, 0.5, true, ms(150));
        assert!((v[0].1 - 0.15).abs() < 1e-6, "from: {}", v[0].1);
        assert!((v[1].1 - 0.15).abs() < 1e-6, "to: {}", v[1].1);
    }

    #[test]
    fn retrigger_mid_envelope_continues_from_current_value() {
        let mut m = AmbientMixer::new();
        m.frame_volumes(0, 0, 1.0, true, ms(0));
        m.frame_volumes(0, 0, 1.0, true, ms(300)); // 全量
        m.frame_volumes(0, 0, 1.0, false, ms(1000)); // 开始淡出
        let v = m.frame_volumes(0, 0, 1.0, false, ms(1150)); // 淡出中点 0.3
        let mid = v[1].1;
        // 淡出中点恢复: 从当前包络值 (0.5) 续接淡入, 不跳变。
        let v = m.frame_volumes(0, 0, 1.0, true, ms(1200));
        assert!(
            (v[1].1 - mid).abs() < 1e-6,
            "反向边沿应连续: {mid} -> {}",
            v[1].1
        );
        // 固定 300ms 包络时长: 中点 (150ms) 走到 0.75 → 0.45; 终点 (300ms) 回全量。
        let v = m.frame_volumes(0, 0, 1.0, true, ms(1350));
        assert!((v[1].1 - 0.45).abs() < 1e-6, "中点: {}", v[1].1);
        let v = m.frame_volumes(0, 0, 1.0, true, ms(1500));
        assert!((v[1].1 - AMBIENT_VOLUME).abs() < 1e-6);
    }

    #[test]
    fn restored_running_session_fades_in_from_silence() {
        // 恢复 Running 会话: 首帧即 running=true, 从静音淡入而非爆音。
        let mut m = AmbientMixer::new();
        let v = m.frame_volumes(2, 2, 1.0, true, ms(0));
        assert_eq!(v[1].1, 0.0);
        let v = m.frame_volumes(2, 2, 1.0, true, ms(300));
        assert!((v[1].1 - AMBIENT_VOLUME).abs() < 1e-6);
    }

    #[test]
    fn player_idle_apply_does_not_touch_device() {
        // 全静音 + 空槽: apply 直接返回, 不开输出设备, 不降级。
        let mut player = AmbientPlayer::new();
        player.apply([(0, 0.0), (0, 0.0)]);
        assert!(player.stream.is_none(), "Idle 不应打开输出设备");
        assert!(!player.disabled);
    }

    #[test]
    fn player_failed_scene_is_skipped_on_apply() {
        // 已知失败的场景: apply 跳过绑定, 不建槽不降级。
        let mut player = AmbientPlayer::new();
        player.failed_scenes[4] = true;
        player.apply([(4, 0.0), (4, 0.0)]);
        assert!(player.slots.iter().all(Option::is_none));
        assert!(!player.disabled);
    }

    #[test]
    fn scene_audio_files_decode_as_ogg_vorbis() {
        // 解码冒烟: 验证 rodio 精简特性 (symphonia-ogg + symphonia-vorbis)
        // 足以解码 5 条资产; 不触输出设备, 纯解码路径。
        for path in SCENE_AUDIO {
            let file = std::fs::File::open(path).unwrap_or_else(|e| panic!("{path}: {e}"));
            let decoder = rodio::Decoder::new(std::io::BufReader::new(file))
                .unwrap_or_else(|e| panic!("{path} 解码失败: {e}"));
            let total = decoder.take(4096).count();
            assert!(total > 0, "{path} 应能解出采样");
        }
    }
}
