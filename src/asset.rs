//! @author 十四叔
//! @date 2026/09/01

//! 运行时资产路径解析: exe 目录优先, CWD 回退。
//!
//! MSIX 包启动时进程 CWD 是 System32 而非包目录, 便携包从非 exe 目录启动时
//! 同样如此 — 裸相对路径 (`assets/...`) 会静默读空 (灰底背景/无环境音)。
//! 所有运行时资产读取统一经 [`resolve`] 解析。

use std::path::{Path, PathBuf};

/// 把相对路径解析到可执行文件所在目录; 拼接结果不存在时原样返回
/// (CWD 相对), 保证 `cargo test` 等开发场景正常工作。
pub fn resolve(path: impl AsRef<Path>) -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(path.as_ref());
            if candidate.exists() {
                return candidate;
            }
        }
    }
    path.as_ref().to_path_buf()
}
