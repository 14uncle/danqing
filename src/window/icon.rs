//! @author 十四叔
//! @date 2026/07/17

//! 窗口与托盘图标加载。
//!
//! - `load_window_icon`: 256×256 PNG, 窗口标题栏 / 任务栏缩略图
//! - `load_tray_icon`: 16×16 PNG, 系统托盘图标 (Windows 任务栏首选尺寸)
//!
//! 两个函数接受 logo 名称 (如 `"logo"` 或 `"pomodoro"`), 从
//! `assets/logo/{name}_{size}.png` 加载对应尺寸的 PNG。
//! - `apply_windows_undecorated_style`: Windows 无边框窗口的圆角 / 阴影
//!
//! 加载失败时记录日志并返 `None`, 窗口创建不会因此 panic。

use winit::window::Icon;

/// 从 PNG 文件加载 winit 图标。
///
/// 将 PNG 解码为 RGBA 后，通过 [`Icon::from_rgba`] 创建图标。
/// 返回 `Err` 时调用方可选择回退到默认图标。
fn load_icon_from_png(path: &std::path::Path) -> Result<Icon, Box<dyn std::error::Error>> {
    let img = image::open(path)?.into_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).map_err(Into::into)
}

/// Windows 下为无边框窗口恢复圆角与阴影。
///
/// 使用 winit 公开的平台扩展 API, 避免手写 unsafe DWM 调用。
/// 若设置失败仅记录警告，不影响窗口功能。
#[cfg(target_os = "windows")]
pub(super) fn apply_windows_undecorated_style(window: &winit::window::Window) {
    use winit::platform::windows::{CornerPreference, WindowExtWindows};

    if let Err(err) = std::panic::catch_unwind(|| {
        window.set_undecorated_shadow(true);
        window.set_corner_preference(CornerPreference::Round);
    }) {
        log::warn!("设置 Windows 无边框窗口样式失败：{err:?}");
    }
}

/// 基于可执行文件所在目录构建资源路径。
///
/// 打包便携版从非 exe 目录启动时，CWD 不一定是 exe 目录。
/// 优先用 `current_exe()` 的 parent() 拼接; 若该路径不存在则回退到 CWD,
/// 保证 `cargo test` 等开发场景也能正常工作。
fn exe_relative(path: &str) -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(path);
            if candidate.exists() {
                return candidate;
            }
        }
    }
    std::path::PathBuf::from(path)
}

/// 加载应用窗口图标。
///
/// 读取 `assets/logo/{name}_256.png`（相对于 exe 所在目录）;
/// 失败时记录警告并返回 `None`, 避免窗口创建因图标问题而 panic。
pub(super) fn load_window_icon(name: &str) -> Option<Icon> {
    let path = exe_relative(&format!("assets/logo/{name}_256.png"));
    match load_icon_from_png(&path) {
        Ok(icon) => Some(icon),
        Err(err) => {
            log::warn!("加载窗口图标失败：{err}");
            None
        }
    }
}

/// 窗口图标对 (标题栏小图标，任务栏大图标)。
///
/// Windows 上任务栏按钮图标首选 ICON_BIG, 而 winit 0.30 的 `with_window_icon` 只发
/// `WM_SETICON(ICON_SMALL)` (标题栏/小图标); 不补 ICON_BIG 时任务栏偶发回退到
/// 无内嵌图标的 exe 缺省图标 (2026-08-02 排查锁定)。两档用同一 PNG,
/// 由调用方分别设到 `with_window_icon` / [`with_taskbar_icon`]。
pub(super) fn window_icons(name: &str) -> (Option<Icon>, Option<Icon>) {
    let icon = load_window_icon(name);
    (icon.clone(), icon)
}

/// Windows 下把任务栏图标 (ICON_BIG) 挂到窗口属性上; 非 Windows 平台为 no-op。
pub(super) fn with_taskbar_icon(
    attrs: winit::window::WindowAttributes,
    _icon: Option<Icon>,
) -> winit::window::WindowAttributes {
    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowAttributesExtWindows;
        if let Some(icon) = _icon {
            return attrs.with_taskbar_icon(Some(icon));
        }
    }
    attrs
}

/// 加载托盘图标 (16x16, Windows 任务栏首选尺寸)。
///
/// 读取 `assets/logo/{name}_16.png`（相对于 exe 所在目录）;
/// 失败时记录警告并返回 `None`。
/// 返回 tray-icon 自身的 `Icon` 类型 (与 winit Icon 不通用)。
#[cfg(target_os = "windows")]
pub(super) fn load_tray_icon(name: &str) -> Option<tray_icon::Icon> {
    let path = exe_relative(&format!("assets/logo/{name}_16.png"));
    match image::open(&path) {
        Ok(img) => {
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            match tray_icon::Icon::from_rgba(rgba.into_raw(), width, height) {
                Ok(icon) => Some(icon),
                Err(err) => {
                    log::warn!("构建托盘 Icon 失败：{err}");
                    None
                }
            }
        }
        Err(err) => {
            log::warn!("加载托盘图标失败：{err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn load_icon_from_valid_png_succeeds() {
        let path = PathBuf::from("assets").join("logo").join("logo_256.png");
        let icon = super::load_icon_from_png(&path);
        assert!(icon.is_ok(), "应能加载有效 PNG 图标：{icon:?}");
    }

    #[test]
    fn load_window_icon_with_logo_name() {
        let icon = super::load_window_icon("logo");
        assert!(icon.is_some(), "应能通过名称 'logo' 加载窗口图标");
    }

    #[test]
    fn window_icons_loads_both_slots_for_pomodoro() {
        let (window_icon, taskbar_icon) = super::window_icons("pomodoro");
        assert!(window_icon.is_some(), "窗口图标应可加载 (pomodoro)");
        assert!(
            taskbar_icon.is_some(),
            "任务栏图标应可加载 (pomodoro): 缺失会回退到系统缺省图标"
        );
    }

    #[test]
    fn load_icon_from_missing_path_returns_error() {
        let path = PathBuf::from("assets").join("logo").join("nonexistent.png");
        let icon = super::load_icon_from_png(&path);
        assert!(icon.is_err());
    }
}
