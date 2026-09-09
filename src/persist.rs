//! @author 十四叔
//! @date 2026/09/09
//! 应用持久化核心工具集。
//!
//! 本模块提供五个 API，解决 pomodoro/clipboard/log 中重复出现的
//! 「配置目录定位 / 原子写入 / 缺省加载 / 脏标记 / 版本化文档」
//! 手写轮子问题。
//!
//! 通过 `persist` feature gate 控制编译。产品在 Cargo.toml 中
//! `features = ["persist"]` 即可使用。
//!
//! # 功能概览
//!
//! | API | 解决的问题 | 复杂度 |
//! |-----|-----------|--------|
//! | [`config_dir`] | 配置目录定位 (appdata/name) | 单函数 |
//! | [`atomic_save`] | 崩溃安全写入 (tmp+rename) | 单函数 |
//! | [`load_or_default`] | 缺省即正常 (missing → Default) | 单函数 |
//! | [`DirtyFlag`] | 脏标记+节流 (小窗隐藏/退出时 flush) | 结构体+方法 |
//! | [`VersionedDoc`] | 版本化文档 (拒绝降级覆盖) | 结构体+方法 |

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;

// ── 1. config_dir ────────────────────────────────────────────────────

/// 返回平台标准应用配置目录 `appdata/<name>`；目录不存在时自动创建。
///
/// # 平台映射
///
/// | 平台 | 路径 |
/// |------|------|
/// | Windows | `%APPDATA%/<name>` |
/// | macOS | `~/Library/Application Support/<name>` |
/// | Linux | `~/.config/<name>` |
///
/// # Panics
///
/// 当 `dirs::config_dir()` 返回 `None`（极罕见的桌面环境缺失）时 panic。
///
/// # Examples
///
/// ```ignore
/// let dir = danqing::persist::config_dir("my-app");
/// assert!(dir.ends_with("my-app"));
/// ```
pub fn config_dir(name: &str) -> PathBuf {
    let dir = dirs::config_dir().expect("config_dir 不可用").join(name);
    fs::create_dir_all(&dir).expect("create config dir");
    dir
}

// ── 2. atomic_save ───────────────────────────────────────────────────

/// 崩溃安全原子写入：写临时文件 → rename 覆盖目标。
///
/// 保证：写一半断电不会损坏已有文件。rename 在同一目录下是原子操作。
///
/// # Errors
///
/// - 临时文件创建/写入失败 → 返回 `io::Error`
/// - rename 失败 → 返回 `io::Error`
///
/// # Examples
///
/// ```ignore
/// danqing::persist::atomic_save(&path, b"hello")?;
/// ```
pub fn atomic_save(path: &Path, data: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

// ── 3. load_or_default ───────────────────────────────────────────────

/// 从 JSON 文件加载；文件不存在或解析失败时返回 `T::default()`。
///
/// 这是「缺省即正常」模式：首次启动/手动删配置都不会报错。
///
/// # Examples
///
/// ```ignore
/// #[derive(Deserialize, Default)]
/// struct Config { volume: f32 }
///
/// let cfg: Config = danqing::persist::load_or_default(&path);
/// assert_eq!(cfg.volume, 0.0); // 文件不存在 → 默认值
/// ```
pub fn load_or_default<T: Default + DeserializeOwned>(path: &Path) -> T {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

// ── 4. DirtyFlag ─────────────────────────────────────────────────────

/// 脏标记 + 节流刷新：数据变更时设脏，距上次 flush 超过 `interval`
/// 时自动 flush。退出时强制 flush（由调用方负责）。
///
/// 注入 `now_fn` 便于测试可控时间源。
///
/// # Examples
///
/// ```ignore
/// let mut dirty = DirtyFlag::new(Duration::from_secs(30), || {
///     atomic_save(&path, &serde_json::to_vec(&data)?)?;
///     Ok(())
/// });
///
/// // 数据变更时
/// dirty.mark();
///
/// // 事件循环末尾
/// dirty.try_flush(Instant::now())?;
///
/// // 退出时
/// dirty.force_flush()?;
/// ```
pub struct DirtyFlag<F: FnMut() -> io::Result<()>> {
    dirty: bool,
    last_flush: Instant,
    interval: Duration,
    flush_fn: F,
}

impl<F: FnMut() -> io::Result<()>> DirtyFlag<F> {
    /// 创建新的 DirtyFlag。`interval` 为两次自动 flush 的最小间隔。
    pub fn new(interval: Duration, flush_fn: F) -> Self {
        Self {
            dirty: false,
            last_flush: Instant::now(),
            interval,
            flush_fn,
        }
    }

    /// 标记数据已脏。
    pub fn mark(&mut self) {
        self.dirty = true;
    }

    /// 仅用于测试：设置 `last_flush` 为当前时刻。
    pub fn reset_timer(&mut self) {
        self.last_flush = Instant::now();
    }

    /// 若脏且距上次 flush 已超过 interval，执行 flush 并清除脏标记。
    ///
    /// # Errors
    ///
    /// flush 函数返回 Err 时透传；脏标记**不**清除（下次重试）。
    pub fn try_flush(&mut self, now: Instant) -> io::Result<bool> {
        if !self.dirty {
            return Ok(false);
        }
        if now.duration_since(self.last_flush) < self.interval {
            return Ok(false);
        }
        (self.flush_fn)()?;
        self.dirty = false;
        self.last_flush = now;
        Ok(true)
    }

    /// 强制 flush（退出时调用），无论是否脏。
    ///
    /// # Errors
    ///
    /// flush 函数返回 Err 时透传。
    pub fn force_flush(&mut self) -> io::Result<()> {
        if self.dirty {
            (self.flush_fn)()?;
            self.dirty = false;
        }
        Ok(())
    }
}

// ── 5. VersionedDoc ──────────────────────────────────────────────────

/// 版本化文档：写入时附带格式版本号，加载时拒绝版本更高的数据（防止降级覆盖）。
///
/// 内部维护 `version`（当前格式版本）和 `data`（文档内容）。
/// `save()` 写入 `{ version, data }` 包装；`load()` 校验版本号。
///
/// # Examples
///
/// ```ignore
/// let mut doc = VersionedDoc::new(1, stats);
/// doc.save(&path)?;
///
/// // 降级场景：旧版本程序加载新版本文件
/// let loaded = VersionedDoc::<Stats>::load(&path)?;
/// assert!(loaded.is_some()); // 版本匹配时返回 Some
/// ```
pub struct VersionedDoc<T> {
    version: u32,
    data: T,
}

/// 版本化文档的序列化/反序列化包装。
#[derive(Serialize, serde::Deserialize)]
struct Wrapper<T> {
    version: u32,
    data: T,
}

impl<T: Serialize + DeserializeOwned + Clone> VersionedDoc<T> {
    /// 创建新的版本化文档。`version` 为当前格式版本号。
    pub fn new(version: u32, data: T) -> Self {
        Self { version, data }
    }

    /// 从 JSON 文件加载。版本号高于当前版本时拒绝并返回 `None`；
    /// 文件不存在或格式错误时返回 `Some(VersionedDoc::new(version, Default))`。
    ///
    /// **注意**：`T` 需实现 `Default`。调用方需在 `T` 不满足 `Default` 时
    /// 使用其他加载方式。
    pub fn load(path: &Path, version: u32) -> Option<Self>
    where
        T: Default,
    {
        let bytes = fs::read(path).ok()?;
        let w: Wrapper<T> = serde_json::from_slice(&bytes).ok()?;
        if w.version > version {
            // 拒绝降级覆盖：版本号高于当前，说明是更新版本写的数据
            return None;
        }
        Some(Self {
            version,
            data: w.data,
        })
    }

    /// 将文档写入 JSON 文件（原子写入）。
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let w = Wrapper {
            version: self.version,
            data: &self.data,
        };
        let bytes = serde_json::to_vec(&w).map_err(io::Error::other)?;
        atomic_save(path, &bytes)
    }

    /// 返回文档数据的不可变引用。
    pub fn data(&self) -> &T {
        &self.data
    }

    /// 返回文档数据的可变引用。
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// 消费自身，返回文档数据。
    pub fn into_data(self) -> T {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn config_dir_returns_path() {
        let dir = config_dir("danqing-test-config");
        assert!(dir.exists());
        // 清理
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn atomic_save_writes_file() {
        let dir = config_dir("danqing-test-atomic");
        let path = dir.join("test.txt");
        atomic_save(&path, b"hello world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello world");
        assert!(!path.with_extension("tmp").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_or_default_existing_file() {
        let dir = config_dir("danqing-test-load");
        let path = dir.join("config.json");
        fs::write(&path, r#"{"key":"value"}"#).unwrap();
        let data: serde_json::Value = load_or_default(&path);
        assert_eq!(data["key"], "value");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_or_default_missing_file() {
        let path = PathBuf::from("/nonexistent/path/config.json");
        let data: serde_json::Value = load_or_default(&path);
        assert_eq!(data, serde_json::Value::Null);
    }

    #[test]
    fn load_or_default_corrupt_json() {
        let dir = config_dir("danqing-test-corrupt");
        let path = dir.join("bad.json");
        fs::write(&path, "not json!!!").unwrap();
        let data: serde_json::Value = load_or_default(&path);
        assert_eq!(data, serde_json::Value::Null);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dirty_flag_flushes_after_interval() {
        let counter = AtomicU32::new(0);
        let mut flag = DirtyFlag::new(Duration::from_secs(5), || {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        flag.mark();

        // 未到间隔 → 不 flush
        let early = Instant::now();
        assert!(!flag.try_flush(early).unwrap());
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        // 超过间隔 → flush
        let late = early + Duration::from_secs(6);
        assert!(flag.try_flush(late).unwrap());
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        // flush 后不再脏
        assert!(!flag.try_flush(late + Duration::from_secs(10)).unwrap());
    }

    #[test]
    fn dirty_flag_force_flush() {
        let counter = AtomicU32::new(0);
        let mut flag = DirtyFlag::new(Duration::from_secs(60), || {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        flag.mark();
        flag.force_flush().unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn dirty_flag_no_flush_when_clean() {
        let counter = AtomicU32::new(0);
        let mut flag = DirtyFlag::new(Duration::from_secs(0), || {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        // 不 mark → 不 flush
        flag.force_flush().unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn dirty_flag_error_keeps_dirty() {
        let mut flag = DirtyFlag::new(Duration::from_secs(0), || {
            Err(io::Error::new(io::ErrorKind::Other, "boom"))
        });
        flag.mark();
        let result = flag.try_flush(Instant::now() + Duration::from_secs(1));
        assert!(result.is_err());
        // 脏标记未清除
        assert!(flag.dirty);
    }

    #[test]
    fn versioned_doc_roundtrip() {
        let dir = config_dir("danqing-test-versioned");
        let path = dir.join("doc.json");

        let doc = VersionedDoc::new(1, vec![1, 2, 3]);
        doc.save(&path).unwrap();

        let loaded = VersionedDoc::<Vec<i32>>::load(&path, 1).unwrap();
        assert_eq!(loaded.data(), &vec![1, 2, 3]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn versioned_doc_rejects_higher_version() {
        let dir = config_dir("danqing-test-reject");
        let path = dir.join("doc.json");

        // 写入 version=2
        let doc = VersionedDoc::new(2, 42u32);
        doc.save(&path).unwrap();

        // 用 version=1 加载 → 拒绝
        let loaded = VersionedDoc::<u32>::load(&path, 1);
        assert!(loaded.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn versioned_doc_missing_file_returns_default() {
        let path = PathBuf::from("/nonexistent/versioned.json");
        let loaded = VersionedDoc::<u32>::load(&path, 1);
        assert!(loaded.is_none());
    }

    #[test]
    fn versioned_doc_corrupt_json_returns_default() {
        let dir = config_dir("danqing-test-vcorrupt");
        let path = dir.join("bad.json");
        fs::write(&path, "not json").unwrap();
        let loaded = VersionedDoc::<u32>::load(&path, 1);
        assert!(loaded.is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn versioned_doc_data_mut() {
        let mut doc = VersionedDoc::new(1, vec![1, 2]);
        doc.data_mut().push(3);
        assert_eq!(doc.data(), &vec![1, 2, 3]);
    }
}
