//! @author 十四叔
//! @date 2026/08/30
//!
//! 音频子系统: N 声道混音器 (mixer, 纯逻辑) + rodio 输出适配 (player)。
//! 移植自 danqing-pomodoro ambient.rs 范式并推广 (双槽 → N 声道 + 一次性事件音)。

mod mixer;
mod player;

pub use mixer::Mixer;
pub use player::AudioPlayer;
