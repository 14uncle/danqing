//! @author 十四叔
//! @date 2026/09/10
//!
//! 文件系统交互原语。
//!
//! 下沉自 danqing-pomodoro `main.rs:1386-1416` (2026-09-10, 小件包 S1)。
//! 2026-10-10 补 `recycle`/`open_path` (Drop Shelf T23 缺口肌肉, 造物寄生)。

use std::path::{Path, PathBuf};

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

/// OS 载荷路径卫生 (评审 R1): 空路径会提前终结双零表 (把同批后续路径静默
/// 截断), 内嵌 NUL 会把一条路径裂成多条 —— 这两种路径不许进任何 OS 载荷
/// (HDROP / SHFileOperation pFrom)。入库层 (shelves.json) 另有拒收, 这里是
/// 引擎边界的最后一道闸。
pub(crate) fn is_clean_os_path(path: &Path) -> bool {
    !path.as_os_str().is_empty() && !path.to_string_lossy().contains('\0')
}

/// 用系统默认程序打开文件/目录 (Shell "open" 动词)。
///
/// Drop Shelf「Space 快速预览」的简化实现 (真预览窗 v1.x); 失败返回错误由
/// 调用方提示, 不静默。
pub fn open_path(path: &Path) -> std::io::Result<()> {
    open_attempt(path)
}

/// 批量送回收站 (可撤销删除)。全成返回 Ok; 有中止返回 Err (调用方逐条留档)。
///
/// Drop Shelf F13「删除(回收站)」用; 确认弹层在产品侧 (显式确认是产品语义,
/// 引擎只执行)。Windows 走 `SHFileOperationW(FO_DELETE + FOF_ALLOWUNDO)`;
/// 非 Windows 平台暂不支持, 明示错误。
pub fn recycle(paths: &[PathBuf]) -> std::io::Result<()> {
    recycle_attempt(paths)
}

/// 路径表 → 双零终结宽字符列表 (`SHFileOperationW.pFrom` 载荷; 纯逻辑, 可单测)。
/// 每路径零终结, 表尾再补一个零; 空表 = 双零 (补足, 别只给一个零)。
#[cfg(target_os = "windows")]
fn double_null_paths(paths: &[PathBuf]) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    let mut out = Vec::new();
    for p in paths {
        out.extend(p.as_os_str().encode_wide().chain(std::iter::once(0u16)));
    }
    out.push(0u16);
    if paths.is_empty() {
        out.push(0u16); // 空表: 无路径自带的终结零, 双零要补足
    }
    out
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

#[cfg(target_os = "windows")]
fn open_attempt(path: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::core::{PCWSTR, w};

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: wide 是双零终结宽串; ShellExecuteW 不持有指针。
    let inst = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW 返回值 > 32 为成功 (MSDN 既定契约)。
    if (inst.0 as isize) > 32 {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "ShellExecuteW 失败, code={}",
            inst.0 as isize
        )))
    }
}

#[cfg(not(target_os = "windows"))]
fn open_attempt(path: &Path) -> std::io::Result<()> {
    // macOS/Linux: 各自的 open/xdg-open。
    #[cfg(target_os = "macos")]
    let cmd = ("open", path.to_path_buf());
    #[cfg(not(target_os = "macos"))]
    let cmd = ("xdg-open", path.to_path_buf());
    std::process::Command::new(cmd.0)
        .arg(cmd.1)
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "windows")]
fn recycle_attempt(paths: &[PathBuf]) -> std::io::Result<()> {
    use windows::Win32::UI::Shell::{
        FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_SILENT, SHFILEOPSTRUCTW, SHFileOperationW,
    };

    if paths.is_empty() {
        return Ok(());
    }
    // 路径卫生闸 (评审 R1): 含 NUL/为空的路径会裂批/截批, 剔除并留痕。
    let clean: Vec<PathBuf> = paths
        .iter()
        .filter(|p| is_clean_os_path(p))
        .cloned()
        .collect();
    if clean.len() != paths.len() {
        log::warn!(
            "回收站: {} 条路径含 NUL 或为空, 已剔除",
            paths.len() - clean.len()
        );
    }
    if clean.is_empty() {
        return Ok(());
    }
    let from = double_null_paths(&clean);
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: windows::core::PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT).0 as u16,
        ..Default::default()
    };
    // SAFETY: from 是双零终结列表, 挂到函数返回前存活; op 只在本调用内存活。
    let code = unsafe { SHFileOperationW(&mut op) };
    if code == 0 && !op.fAnyOperationsAborted.as_bool() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "回收站操作未完成 (code={code}, aborted={})",
            op.fAnyOperationsAborted.as_bool()
        )))
    }
}

#[cfg(not(target_os = "windows"))]
fn recycle_attempt(_paths: &[PathBuf]) -> std::io::Result<()> {
    Err(std::io::Error::other("当前平台暂不支持回收站操作"))
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::{double_null_paths, is_clean_os_path};
    use std::path::PathBuf;

    /// 双零终结: 每路径零终结, 表尾再补一个零。
    #[test]
    fn double_null_paths_terminates_each_and_the_table() {
        let paths = vec!["C:\\a.txt".into(), "C:\\中文".into()];
        let blob = double_null_paths(&paths);
        // 逐单元: a . t x t 零 中 文 零 表尾零
        let expect: Vec<u16> = "C:\\a.txt"
            .encode_utf16()
            .chain(std::iter::once(0))
            .chain("C:\\中文".encode_utf16())
            .chain(std::iter::once(0))
            .chain(std::iter::once(0))
            .collect();
        assert_eq!(blob, expect);
    }

    /// 空表 = 单个双零 (2 个 u16)。
    #[test]
    fn double_null_paths_empty_is_lone_double_null() {
        let blob = double_null_paths(&[]);
        assert_eq!(blob, vec![0u16, 0]);
    }

    /// 路径卫生闸 (评审 R1): 空路径/含 NUL 路径不许进 OS 载荷。
    #[test]
    fn clean_os_path_rejects_empty_and_nul() {
        assert!(is_clean_os_path(std::path::Path::new(r"C:\a.txt")));
        assert!(!is_clean_os_path(std::path::Path::new("")));
        let dirty = PathBuf::from("C:\\junk.txt\0C:\\secret.docx");
        assert!(!is_clean_os_path(&dirty), "内嵌 NUL 会把一条裂成多条");
    }
}
