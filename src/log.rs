//! @author 十四叔
//! @date 2026/07/24

//! 丹青框架日志初始化。
//!
//! 提供 `init_log`：本地时间戳 + level + target + message 格式，
//! 默认过滤级别 `info`（受 `RUST_LOG` 环境变量覆盖）。
//!
//! 日志同时写入 stderr 与日志目录，自动轮转（保留最近 10 个文件），
//! 并安装 panic hook 将 panic 信息写入日志文件。
//!
//! 日志目录按以下顺序取第一个**写得进去**的：
//! 1. exe 同级 `logs/`（便携包默认，解压即用）
//! 2. `%LOCALAPPDATA%\<exe 名>\logs`（**MSIX 商店版靠这一步**，见 [`user_logs_dir`]）
//! 3. 当前工作目录 `logs/`（兜底）
//!
//! # 用法
//!
//! ```rust,no_run
//! danqing::log::init_log();
//! log::info!("日志系统已就绪");
//! ```

use std::backtrace::Backtrace;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Once};
use std::time::SystemTime;

const LOG_RETENTION: usize = 10;
const MAX_LOG_FILE_COLLISIONS: u32 = 1_000;

static LOG_INIT: Once = Once::new();

#[derive(Debug)]
struct LogFile {
    path: PathBuf,
    modified: SystemTime,
}

fn logs_dir_for(executable: &Path) -> Option<PathBuf> {
    executable
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.join("logs"))
}

fn log_file_name(stem: &str, timestamp: &str, pid: u32, collision: u32) -> String {
    let collision = match collision {
        0 => String::new(),
        value => format!("-{value}"),
    };
    format!("{stem}-{timestamp}-p{pid}{collision}.log")
}

fn is_ascii_digits(value: &[u8]) -> bool {
    !value.is_empty() && value.iter().all(u8::is_ascii_digit)
}

fn is_decimal(value: &str) -> bool {
    is_ascii_digits(value.as_bytes())
}

fn is_log_timestamp(value: &str) -> bool {
    let timestamp = value.as_bytes();
    timestamp.len() == 19
        && is_ascii_digits(&timestamp[..8])
        && timestamp[8] == b'-'
        && is_ascii_digits(&timestamp[9..15])
        && timestamp[15] == b'.'
        && is_ascii_digits(&timestamp[16..])
}

fn is_log_file_for_stem(path: &Path, stem: &str) -> bool {
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    let prefix = format!("{stem}-");
    let Some(body) = name
        .strip_prefix(&prefix)
        .and_then(|name| name.strip_suffix(".log"))
    else {
        return false;
    };
    let Some((timestamp, process)) = body.split_once("-p") else {
        return false;
    };
    let mut process_parts = process.split('-');
    let valid_pid = process_parts.next().is_some_and(is_decimal);
    let valid_collision = process_parts.next().is_none_or(is_decimal);
    is_log_timestamp(timestamp) && valid_pid && valid_collision && process_parts.next().is_none()
}

fn select_logs_to_delete(
    entries: &[LogFile],
    current: &Path,
    stem: &str,
    keep: usize,
) -> Vec<PathBuf> {
    let mut history = entries
        .iter()
        .filter(|entry| entry.path != current)
        .filter(|entry| is_log_file_for_stem(&entry.path, stem))
        .collect::<Vec<_>>();
    history.sort_by(|left, right| {
        right
            .modified
            .cmp(&left.modified)
            .then_with(|| right.path.cmp(&left.path))
    });
    history
        .into_iter()
        .skip(keep.saturating_sub(1))
        .map(|entry| entry.path.clone())
        .collect()
}

fn format_panic_record(timestamp: &str, payload: &str, location: &str, backtrace: &str) -> String {
    format!("PANIC {timestamp}\npayload: {payload}\nlocation: {location}\n{backtrace}\n")
}

struct PreparedLog {
    path: PathBuf,
    stem: String,
    file: Arc<Mutex<File>>,
}

struct TeeWriter {
    file: Option<Arc<Mutex<File>>>,
}

impl Write for TeeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut stderr = io::stderr().lock();
        let _ = stderr.write_all(bytes);
        let _ = stderr.flush();

        if let Some(file) = &self.file {
            let mut file = file
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let _ = file.write_all(bytes);
            let _ = file.flush();
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let _ = io::stderr().flush();
        if let Some(file) = &self.file {
            let mut file = file
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let _ = file.flush();
        }
        Ok(())
    }
}

fn create_log_file_in(
    directory: &Path,
    stem: &str,
    timestamp: &str,
    pid: u32,
) -> io::Result<PreparedLog> {
    fs::create_dir_all(directory)?;

    for collision in 0..MAX_LOG_FILE_COLLISIONS {
        let path = directory.join(log_file_name(stem, timestamp, pid, collision));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                return Ok(PreparedLog {
                    path,
                    stem: stem.to_owned(),
                    file: Arc::new(Mutex::new(file)),
                });
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "本次启动无法分配唯一日志文件名",
    ))
}

fn prune_old_logs(log: &PreparedLog) -> io::Result<usize> {
    let directory = log
        .path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "日志文件路径没有父目录"))?;
    let mut entries = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            entries.push(LogFile {
                path: entry.path(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            });
        }
    }

    let stale = select_logs_to_delete(&entries, &log.path, &log.stem, LOG_RETENTION);
    for path in &stale {
        fs::remove_file(path)?;
    }
    Ok(stale.len())
}

fn install_panic_hook(file: Arc<Mutex<File>>) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut file) = file.try_lock() {
            let payload = info
                .payload()
                .downcast_ref::<&str>()
                .map(|payload| (*payload).to_owned())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| String::from("<非字符串 panic>"));
            let location = info
                .location()
                .map(ToString::to_string)
                .unwrap_or_else(|| String::from("<未知位置>"));
            let backtrace = Backtrace::force_capture().to_string();
            let timestamp = chrono::Local::now()
                .format("%Y-%m-%d %H:%M:%S%.3f")
                .to_string();
            let record = format_panic_record(&timestamp, &payload, &location, &backtrace);
            let _ = file.write_all(record.as_bytes());
            let _ = file.flush();
        }
        previous(info);
    }));
}

/// 用户数据目录下的 `logs/` —— Windows 上是 `%LOCALAPPDATA%\<exe 名>\logs`。
///
/// **为什么要有这一级 (2026-09-13)**: MSIX 商店版的 exe 装在
/// `C:\Program Files\WindowsApps\<包名>\`, 那里**只读**; 而 MSIX 应用的进程
/// 工作目录是 `C:\Windows\System32` (不是 exe 目录), 写进去同样失败。于是
/// 「exe 同级」与「CWD」**两级全都落空**, 而本框架的窗口是
/// `windows_subsystem = "windows"`、没有控制台 —— 商店用户**一条日志都没有**,
/// 出了 bug 无从向他要日志。这条由 danqing-pomodoro 在 2026-09-01 商店上架时撞上,
/// 当时记为「框架侧待办: 日志目录回退」, 一直挂着。
///
/// 取 **Local** 而非 Roaming: 日志是机器本地数据, 不该跟着漫游配置跑。
/// MSIX 下这个路径本身会被重定向到包的私有目录 (`...\Packages\<包名>\LocalCache\...`),
/// 那正是我们想要的落点。
///
/// **非 Windows 返回 `None`**: 这一级的动机是 Windows/MSIX 专有的, 其余平台
/// 维持「exe 同级 → CWD」两级不变, 不引入新的平台假设。
#[cfg(windows)]
fn user_logs_dir(stem: &str) -> Option<PathBuf> {
    let base = env::var_os("LOCALAPPDATA")?;
    if base.is_empty() {
        return None;
    }
    Some(Path::new(&base).join(stem).join("logs"))
}

#[cfg(not(windows))]
fn user_logs_dir(_stem: &str) -> Option<PathBuf> {
    None
}

fn try_create_log(timestamp: &str, pid: u32) -> Result<PreparedLog, String> {
    let exe_stem = env::current_exe().ok().and_then(|executable| {
        executable
            .file_stem()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string_lossy().into_owned())
    });
    // 1) 优先 exe 同级 logs/ 目录 —— 便携包的默认位置 (解压即用, 日志跟着程序走)。
    if let Ok(executable) = env::current_exe() {
        if let Some(directory) = logs_dir_for(&executable) {
            if let Some(stem) = &exe_stem {
                if let Ok(log) = create_log_file_in(&directory, stem, timestamp, pid) {
                    return Ok(log);
                }
            }
        }
    }
    // 2) 用户数据目录 —— **商店版靠这一步** (见 `user_logs_dir`)。
    if let Some(stem) = &exe_stem {
        if let Some(directory) = user_logs_dir(stem) {
            if let Ok(log) = create_log_file_in(&directory, stem, timestamp, pid) {
                return Ok(log);
            }
        }
    }
    // 3) 兜底：当前工作目录的 logs/ 子目录。
    let cwd = env::current_dir().map_err(|err| format!("无法确定当前目录：{err}"))?;
    let directory = cwd.join("logs");
    let stem = String::from("app");
    create_log_file_in(&directory, &stem, timestamp, pid)
        .map_err(|err| format!("CWD 日志创建失败：{err}"))
}

fn init_log_once() {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S%.3f").to_string();
    let pid = std::process::id();

    let prepared = try_create_log(&timestamp, pid);
    let (prepared, setup_error) = match prepared {
        Ok(prepared) => (Some(prepared), None),
        Err(err) => (None, Some(err)),
    };
    let file = prepared.as_ref().map(|log| Arc::clone(&log.file));

    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    builder
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
        .target(env_logger::Target::Pipe(Box::new(TeeWriter {
            file: file.clone(),
        })));

    if let Err(err) = builder.try_init() {
        eprintln!("无法安装日志记录器：{err}");
    }
    if let Some(err) = setup_error {
        log::warn!("{err}，将仅输出到 stderr");
    }
    if let Some(log) = &prepared {
        install_panic_hook(Arc::clone(&log.file));
        log::info!("日志文件：{}", log.path.display());
        if let Err(err) = prune_old_logs(log) {
            log::warn!("清理旧日志失败：{err}");
        }
    }
}

/// 初始化 `env_logger`，同时写入 stderr 与可执行文件同级的 `logs/`。
///
/// 仅需在应用 `main` 开头调用一次；幂等，重复调用无效。
/// 初始化失败时自动降级到 stderr。
pub fn init_log() {
    LOG_INIT.call_once(init_log_once);
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

    use super::{LogFile, log_file_name, logs_dir_for, select_logs_to_delete};

    /// 商店版那条兜底必须落在**用户数据目录**下 —— 既不是 exe 目录 (MSIX 下只读),
    /// 也不是 CWD (MSIX 下是 System32)。
    #[test]
    #[cfg(windows)]
    fn user_logs_dir_lands_under_local_app_data() {
        let dir = super::user_logs_dir("danqing-log").expect("Windows 上应能取到 LOCALAPPDATA");
        let base = std::env::var_os("LOCALAPPDATA").expect("测试环境应有 LOCALAPPDATA");
        assert!(
            dir.starts_with(Path::new(&base)),
            "应落在 %LOCALAPPDATA% 下, 实得 {dir:?}"
        );
        assert_eq!(
            dir,
            Path::new(&base).join("danqing-log").join("logs"),
            "落点必须是 <LOCALAPPDATA>\\<exe 名>\\logs"
        );
    }

    #[test]
    fn logs_dir_uses_executable_parent() {
        let executable = Path::new("/portable/pomodoro.exe");

        assert_eq!(
            logs_dir_for(executable),
            Some(PathBuf::from("/portable/logs"))
        );
    }

    #[test]
    fn logs_dir_rejects_path_without_parent() {
        assert_eq!(logs_dir_for(Path::new("/")), None);
    }

    #[test]
    fn log_file_name_is_deterministic() {
        assert_eq!(
            log_file_name("pomodoro", "20260724-153045.123", 12345, 0),
            "pomodoro-20260724-153045.123-p12345.log"
        );
        assert_eq!(
            log_file_name("pomodoro", "20260724-153045.123", 12345, 2),
            "pomodoro-20260724-153045.123-p12345-2.log"
        );
    }

    #[test]
    fn retention_keeps_current_and_nine_latest_matching_logs() {
        let current = PathBuf::from("logs/pomodoro-20260724-160000.000-p1.log");
        let mut entries = (0..12)
            .map(|index| LogFile {
                path: PathBuf::from(format!("logs/pomodoro-20260724-1500{index:02}.000-p1.log")),
                modified: SystemTime::UNIX_EPOCH + Duration::from_secs(index),
            })
            .collect::<Vec<_>>();
        entries.push(LogFile {
            path: current.clone(),
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(20),
        });
        entries.push(LogFile {
            path: PathBuf::from("logs/showcase-00.log"),
            modified: SystemTime::UNIX_EPOCH,
        });
        entries.push(LogFile {
            path: PathBuf::from("logs/notes.txt"),
            modified: SystemTime::UNIX_EPOCH,
        });

        let mut removed = select_logs_to_delete(&entries, &current, "pomodoro", 10);
        removed.sort();

        assert_eq!(
            removed,
            vec![
                PathBuf::from("logs/pomodoro-20260724-150000.000-p1.log"),
                PathBuf::from("logs/pomodoro-20260724-150001.000-p1.log"),
                PathBuf::from("logs/pomodoro-20260724-150002.000-p1.log"),
            ]
        );
        assert!(!removed.contains(&current));
    }

    #[test]
    fn retention_does_nothing_below_limit() {
        let current = PathBuf::from("logs/pomodoro-20260724-160000.000-p1.log");
        let entries = vec![
            LogFile {
                path: current.clone(),
                modified: SystemTime::UNIX_EPOCH + Duration::from_secs(2),
            },
            LogFile {
                path: PathBuf::from("logs/pomodoro-20260724-150000.000-p1.log"),
                modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1),
            },
        ];

        assert!(select_logs_to_delete(&entries, &current, "pomodoro", 10).is_empty());
    }

    #[test]
    fn retention_does_not_match_a_longer_executable_stem() {
        let current = PathBuf::from("logs/pomodoro-20260724-160000.000-p1.log");
        let entries = vec![
            LogFile {
                path: current.clone(),
                modified: SystemTime::UNIX_EPOCH + Duration::from_secs(20),
            },
            LogFile {
                path: PathBuf::from("logs/pomodoro-beta-20260724-150000.000-p2.log"),
                modified: SystemTime::UNIX_EPOCH,
            },
        ];

        assert!(select_logs_to_delete(&entries, &current, "pomodoro", 1).is_empty());
    }

    #[test]
    fn panic_record_contains_payload_location_and_backtrace() {
        let record = super::format_panic_record(
            "2026-07-24 15:30:45.123",
            "诊断 panic",
            "examples/pomodoro/main.rs:190:5",
            "stack backtrace:\n  0: pomodoro::main",
        );

        assert!(record.contains("PANIC 2026-07-24 15:30:45.123"));
        assert!(record.contains("payload: 诊断 panic"));
        assert!(record.contains("location: examples/pomodoro/main.rs:190:5"));
        assert!(record.contains("stack backtrace:\n  0: pomodoro::main"));
    }
}
