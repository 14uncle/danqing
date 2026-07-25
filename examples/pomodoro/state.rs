//! @author 十四叔
//! @date 2026/07/25

//! 番茄钟状态持久化: 运行态 + 场景 + 计时快照 + 时间轴基准。
//!
//! JSON 写到 OS 配置目录 (`%APPDATA%/danqing/pomodoro.json` on Windows),
//! 启动时优先加载, 失败回退默认 25:00 Idle。Running 状态按 wall-clock
//! 偏移恢复 deadline, 允许跨重启不丢时间。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::timer::{Phase, Run};

/// 持有运行态的枚举镜像 (跨进程序列化)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    /// 静止 (未开始或被重置)。
    Idle,
    /// 计时中。
    Running,
    /// 暂停 (剩余时间已快照)。
    Paused,
}

impl From<Run> for RunState {
    fn from(r: Run) -> Self {
        match r {
            Run::Idle => Self::Idle,
            Run::Running => Self::Running,
            Run::Paused => Self::Paused,
        }
    }
}

impl From<RunState> for Run {
    fn from(s: RunState) -> Self {
        match s {
            RunState::Idle => Self::Idle,
            RunState::Running => Self::Running,
            RunState::Paused => Self::Paused,
        }
    }
}

/// 番茄钟持久化快照 (重启恢复的最小集)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PomodoroState {
    /// 当前阶段。
    pub phase: Phase,
    /// 当前运行态。
    pub run: RunState,
    /// 剩余秒数 (向下取整, 1s 误差可接受)。
    pub remaining_secs: u64,
    /// 当前场景索引。
    pub current_scene: usize,
    /// 保存时刻的 elapsed 时间 (注入时间轴基准)。
    pub saved_elapsed_secs: u64,
    /// 保存时刻的 wall-clock Unix 秒。
    pub saved_wall_secs: u64,
}

impl PomodoroState {
    /// 启动时计算 effective_now: 当前 wall-clock - saved_wall + saved_elapsed。
    /// 跨重启的 elapsed 偏移; 即 `AnimationCtx::elapsed` 应达到的值。
    pub fn effective_now_offset(&self) -> Duration {
        let now_wall = current_wall_secs();
        let delta = now_wall.saturating_sub(self.saved_wall_secs);
        Duration::from_secs(self.saved_elapsed_secs.saturating_add(delta))
    }
}

/// 当前 wall-clock Unix 秒 (失败时返回 0, 不影响持久化逻辑)。
fn current_wall_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 持久化文件路径 (OS 配置目录 + danqing/pomodoro.json)。
pub fn state_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("danqing").join("pomodoro.json"))
}

/// 写盘: 原子写 (临时文件 + rename)。失败不 panic, 记录错误。
pub fn save_state(state: &PomodoroState) -> io::Result<()> {
    let Some(path) = state_path() else {
        log::warn!("持久化路径不可用, 跳过保存");
        return Ok(());
    };
    save_to_path(&path, state)
}

/// 加载: 文件不存在 / 解析失败返回 None。
pub fn load_state() -> Option<PomodoroState> {
    let path = state_path()?;
    load_from_path(&path)
}

/// 写入指定路径 (测试与显式路径场景)。
pub fn save_to_path(path: &Path, state: &PomodoroState) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(state).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(tmp, path)?;
    Ok(())
}

/// 读取指定路径 (测试与显式路径场景)。
pub fn load_from_path(path: &Path) -> Option<PomodoroState> {
    let data = fs::read_to_string(path).ok()?;
    match serde_json::from_str(&data) {
        Ok(state) => Some(state),
        Err(err) => {
            log::warn!("解析持久化文件失败: {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_state_conversion_roundtrip() {
        for r in [Run::Idle, Run::Running, Run::Paused] {
            let s: RunState = r.into();
            let r2: Run = s.into();
            assert_eq!(r, r2);
        }
    }

    #[test]
    fn state_serialization_roundtrip() {
        let original = PomodoroState {
            phase: Phase::Focus,
            run: RunState::Running,
            remaining_secs: 1234,
            current_scene: 2,
            saved_elapsed_secs: 567,
            saved_wall_secs: 1_000_000,
        };
        let json = serde_json::to_string(&original).unwrap();
        let back: PomodoroState = serde_json::from_str(&json).unwrap();
        assert_eq!(original, back);
    }

    #[test]
    fn save_and_load_to_temp_path() {
        let dir = std::env::temp_dir().join("danqing-test-state-1");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("pomodoro.json");

        let original = PomodoroState {
            phase: Phase::Break,
            run: RunState::Paused,
            remaining_secs: 60,
            current_scene: 3,
            saved_elapsed_secs: 42,
            saved_wall_secs: 999_999,
        };
        save_to_path(&path, &original).unwrap();
        let loaded = load_from_path(&path).unwrap();
        assert_eq!(original, loaded);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_from_nonexistent_path_returns_none() {
        let path = std::env::temp_dir().join("danqing-test-nonexistent.json");
        let _ = fs::remove_file(&path);
        assert!(load_from_path(&path).is_none());
    }

    #[test]
    fn load_from_corrupted_json_returns_none() {
        let dir = std::env::temp_dir().join("danqing-test-corrupt");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("pomodoro.json");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, "{not json").unwrap();
        assert!(load_from_path(&path).is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_path_returns_pomodoro_json() {
        let path = state_path().unwrap();
        assert_eq!(
            path.file_name().and_then(|s| s.to_str()),
            Some("pomodoro.json")
        );
    }

    #[test]
    fn effective_now_offset_includes_wall_clock_delta() {
        let now_secs = current_wall_secs();
        let s = PomodoroState {
            phase: Phase::Focus,
            run: RunState::Idle,
            remaining_secs: 1500,
            current_scene: 0,
            // 假装保存于 100s 之前
            saved_elapsed_secs: 100,
            saved_wall_secs: now_secs.saturating_sub(100),
        };
        let offset = s.effective_now_offset().as_secs();
        // 期望 ≈ saved_elapsed + (now - saved_wall) = 100 + 100 = 200
        let tolerance = 2;
        assert!(
            (offset as i64 - 200).unsigned_abs() <= tolerance,
            "offset={offset}, expected ~200"
        );
    }
}
