//! @author 十四叔
//! @date 2026/08/30
//!
//! rodio 输出适配层: N 声道循环槽 + 一次性事件音。
//!
//! 移植自 danqing-pomodoro ambient.rs 输出层并推广 (双槽 → N 声道):
//! - 懒初始化: 首次有声音需求 (ensure_loop/play_once) 才打开输出设备。
//! - 静默降级: 设备打开失败永久降级 (disabled); 单条音源打不开记入
//!   failed_paths 不再重试 (防 60fps 刷日志)。所有失败仅 log::warn。
//! - 零音量声道暂停不卸载 (不空转解码/重采样), 增益回正续播 (位置保持)。
//!
//! 混音策略 (增益包络/声道生灭) 在 [`super::Mixer`], 本层只忠实执行。

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::Source;

/// N 声道音频播放器: 消费 [`super::Mixer`] 的帧增益, 管理循环声道与事件音。
pub struct AudioPlayer {
    /// 输出流 (懒初始化; None = 尚未打开设备)。
    stream: Option<rodio::MixerDeviceSink>,
    /// 循环声道: 声道号 → (音源路径, 播放器)。drop 即停播。
    loops: HashMap<u32, (PathBuf, rodio::Player)>,
    /// 进行中的一次性事件音 (播完清场)。
    oneshots: Vec<rodio::Player>,
    /// 永久降级旗标: 输出设备打开失败后置位, 之后直接返回。
    disabled: bool,
    /// 打不开的音源路径 (缺文件/解码失败), 不再重试。
    failed_paths: HashSet<PathBuf>,
}

impl AudioPlayer {
    /// 创建播放器: 未初始化, 未降级, 零声道。
    pub fn new() -> Self {
        Self {
            stream: None,
            loops: HashMap::new(),
            oneshots: Vec::new(),
            disabled: false,
            failed_paths: HashSet::new(),
        }
    }

    /// 确保声道绑定到循环音源 (幂等; 已绑同路径则零动作, 路径变化重绑)。
    /// 首次绑定时打开输出设备 (懒初始化); 失败仅 warn (静默降级)。
    pub fn ensure_loop(&mut self, channel: u32, path: &Path) {
        if self.disabled || self.failed_paths.contains(path) {
            return;
        }
        if let Some((bound, _)) = self.loops.get(&channel) {
            if bound == path {
                return;
            }
            self.loops.remove(&channel); // 路径变化 → 旧槽退场 (drop 停播)
        }
        let Some(stream) = self.stream() else {
            return;
        };
        let Some(source) = LoopingDecoder::new(path) else {
            log::warn!("循环音源打开/解码失败 ({}), 不再重试", path.display());
            self.failed_paths.insert(path.to_path_buf());
            return;
        };
        let player = rodio::Player::connect_new(stream.mixer());
        player.append(source);
        player.pause(); // 建槽即暂停 —— 增益由 apply 驱动, 零增益不发声
        self.loops.insert(channel, (path.to_path_buf(), player));
    }

    /// 应用每帧增益: 音量直写; 零音量声道暂停 (常驻应用的支配态是安静),
    /// 增益回正续播。全零且无声道时零动作 (不碰设备)。
    pub fn apply(&mut self, gains: &[(u32, f32)]) {
        if self.disabled {
            return;
        }
        if gains.iter().all(|(_, g)| *g <= 0.0) && self.loops.is_empty() {
            return; // 静默 idle: 不触碰音频设备
        }
        for &(channel, gain) in gains {
            if let Some((_, player)) = self.loops.get(&channel) {
                player.set_volume(gain.max(0.0));
                if gain > 0.0 {
                    player.play();
                } else {
                    player.pause();
                }
            }
        }
        // 事件音清场: 播完的退场 (不积累)。
        self.oneshots.retain(|p| !p.empty());
    }

    /// 一次性事件音 (雷声/点缀; 音量夹到 0..1)。播完自动退场。
    /// 失败仅 warn (静默降级), 不阻塞世界。
    pub fn play_once(&mut self, path: &Path, volume: f32) {
        if self.disabled || self.failed_paths.contains(path) {
            return;
        }
        let Some(stream) = self.stream() else {
            return;
        };
        let Some(source) = Self::decode(path) else {
            log::warn!("事件音打开/解码失败 ({}), 不再重试", path.display());
            self.failed_paths.insert(path.to_path_buf());
            return;
        };
        let player = Self::spawn(stream, source, volume);
        self.oneshots.push(player);
    }

    /// 播放程序化音源 (正弦测试音等, 免资产; showcase 演示用)。
    /// 一次性: 源耗尽自动退场。
    pub fn play_source<S>(&mut self, source: S, volume: f32)
    where
        S: Source<Item = f32> + Send + 'static,
    {
        if self.disabled {
            return;
        }
        let Some(stream) = self.stream() else {
            return;
        };
        let player = Self::spawn(stream, source, volume);
        self.oneshots.push(player);
    }

    /// 建一次性播放槽: 挂流 + 定音量 + 开播 (播放完由 apply 清场)。
    fn spawn<S>(stream: &rodio::MixerDeviceSink, source: S, volume: f32) -> rodio::Player
    where
        S: Source<Item = f32> + Send + 'static,
    {
        let player = rodio::Player::connect_new(stream.mixer());
        player.append(source);
        player.set_volume(volume.clamp(0.0, 1.0));
        player.play();
        player
    }

    /// 懒打开输出流; 失败永久降级并返回 None。
    fn stream(&mut self) -> Option<&rodio::MixerDeviceSink> {
        if self.stream.is_none() {
            match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(stream) => self.stream = Some(stream),
                Err(err) => {
                    log::warn!("音频输出设备打开失败, 永久降级 (无声运行): {err}");
                    self.disabled = true;
                    return None;
                }
            }
        }
        self.stream.as_ref()
    }

    /// 打开文件并创建一次性解码器; 任一步失败返回 None。
    fn decode(path: &Path) -> Option<rodio::Decoder<BufReader<File>>> {
        let file = File::open(path).ok()?;
        rodio::Decoder::new(BufReader::new(file)).ok()
    }

    /// 是否已永久降级 (无音频设备环境; 测试与诊断用)。
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// 无限循环的流式解码源: 当前解码器耗尽时重开文件从头解码续播。
///
/// 存在理由 (pomodoro 2026-07 血泪, 逐字移植): rodio 0.22 的
/// `repeat_infinite` 内部走 `buffered()`, 建缓冲时把 symphonia 解码器
/// 初始空包 (`current_span_len() == Some(0)`) 误判为流结束, 追加后整源
/// 秒空、无声。此处自实现循环绕开该环节。
///
/// 回卷不用 `try_seek`: symphonia 粗粒度 seek 回 0 会跳过首个 Vorbis 包
/// (实测少 1156 采样 ≈ 24ms), 每循环一次接缝就爆音一声; 重开文件从头
/// 解码才是逐位一致的真回卷。音源文件首尾应做微 crossfade (资产侧纪律)。
struct LoopingDecoder {
    /// 音源路径 (重开文件回卷用)。
    path: PathBuf,
    /// 当前解码器; None = 已永久失败 (静默降级, 后续一律 None)。
    current: Option<rodio::Decoder<BufReader<File>>>,
    /// 声道数 (自首帧捕获, 循环不变)。
    channels: rodio::ChannelCount,
    /// 采样率 (自首帧捕获, 循环不变)。
    sample_rate: rodio::SampleRate,
}

impl LoopingDecoder {
    /// 打开并解码首轮; 失败返回 None (调用方记 failed_paths)。
    fn new(path: &Path) -> Option<Self> {
        let decoder = Self::decode(path)?;
        Some(Self {
            path: path.to_path_buf(),
            channels: decoder.channels(),
            sample_rate: decoder.sample_rate(),
            current: Some(decoder),
        })
    }

    /// 打开文件并创建解码器; 任一步失败返回 None。
    fn decode(path: &Path) -> Option<rodio::Decoder<BufReader<File>>> {
        let file = File::open(path).ok()?;
        rodio::Decoder::new(BufReader::new(file)).ok()
    }
}

impl Iterator for LoopingDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        // 最多两轮: 当前解码器取流 → 耗尽则重开文件回卷再取;
        // 仍无采样视为永久失败 (防音频线程空转)。
        for _ in 0..2 {
            let mut decoder = self.current.take()?;
            if let Some(sample) = decoder.next() {
                self.current = Some(decoder);
                return Some(sample);
            }
            self.current = Self::decode(&self.path);
        }
        log::warn!("循环音源永久关闭 ({})", self.path.display());
        self.current = None;
        None
    }
}

impl Source for LoopingDecoder {
    fn current_span_len(&self) -> Option<usize> {
        None // 无限流
    }

    fn channels(&self) -> rodio::ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None // 无限循环
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 静默 idle: 全零增益且零声道 → 不碰输出设备, 不降级。
    #[test]
    fn idle_apply_does_not_touch_device() {
        let mut player = AudioPlayer::new();
        player.apply(&[(0, 0.0), (1, 0.0)]);
        assert!(player.stream.is_none(), "idle 不应打开输出设备");
        assert!(!player.is_disabled());
    }

    /// 缺失音源: ensure_loop 记名不重试, 不 panic, 不降级整机。
    #[test]
    fn missing_loop_source_recorded_not_retried() {
        let mut player = AudioPlayer::new();
        let missing = Path::new("assets/audio/definitely-not-exists.ogg");
        player.ensure_loop(0, missing);
        assert!(player.failed_paths.contains(missing), "失败应记名");
        player.ensure_loop(0, missing); // 第二次: 不重试 (无重复日志路径)
        assert!(!player.is_disabled(), "单文件失败不降级整机");
    }

    /// 缺失事件音: play_once 静默降级 (warn), 不 panic。
    #[test]
    fn missing_oneshot_degrades_silently() {
        let mut player = AudioPlayer::new();
        player.play_once(Path::new("assets/audio/definitely-not-exists.ogg"), 0.8);
        assert!(player.oneshots.is_empty());
        assert!(
            player
                .failed_paths
                .contains(Path::new("assets/audio/definitely-not-exists.ogg"))
        );
    }

    /// 循环解码器对缺失文件返回 None (解码冒烟与回卷精度测试需要真实
    /// 音源资产 —— 归 danqing-deskscape Task 7 用 rain.ogg 跑)。
    #[test]
    fn looping_decoder_missing_file_is_none() {
        assert!(LoopingDecoder::new(Path::new("definitely-not-exists.ogg")).is_none());
    }
}
