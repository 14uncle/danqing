//! @author 十四叔
//! @date 2026/07/24

//! 丹青示例共享的初始化辅助。
//!
//! 当前提供 `init_log`：本地时间戳 + level + target + message 格式，
//! 默认过滤级别 `info`（受 `RUST_LOG` 环境变量覆盖）。
//!
//! 各 example 通过 `#[path = ...]` 引入本模块，避免相互依赖。

use std::io::Write;

/// 初始化 `env_logger`，使用丹青示例统一的时间戳格式。
///
/// 仅需在每个 example 的 `main` 开头调用一次。
pub fn init_log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let now = chrono::Local::now();
            writeln!(
                buf,
                "{} {} [{}] {}",
                now.format("%H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}
