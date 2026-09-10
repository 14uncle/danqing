//! @author 十四叔
//! @date 2026/09/10
//!
//! 文件系统交互原语。
//!
//! 下沉自 danqing-pomodoro `main.rs:1386-1416` (2026-09-10, 小件包 S1)。

use std::path::Path;

/// 在系统文件管理器中定位并高亮文件。
///
/// - Windows: `explorer /select,<path>`
/// - macOS: `open -R <path>`
/// - Linux/其它: `xdg-open <dir>` (打开所在目录)
///
/// 失败仅记录日志, 不 panic —— 导出成功后显示文件, 失败不影响导出结果。
pub fn reveal_in_file_manager(path: &Path) {
    if let Err(err) = reveal_attempt(path) {
        log::warn!("在文件管理器中显示文件失败：{err}");
    }
}

#[cfg(target_os = "windows")]
fn reveal_attempt(path: &Path) -> std::io::Result<std::process::Child> {
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn()
}

#[cfg(target_os = "macos")]
fn reveal_attempt(path: &Path) -> std::io::Result<std::process::Child> {
    std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn reveal_attempt(path: &Path) -> std::io::Result<std::process::Child> {
    let Some(dir) = path.parent() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "文件无父目录",
        ));
    };
    std::process::Command::new("xdg-open").arg(dir).spawn()
}
