//! @author 十四叔
//! @date 2026/07/17

//! 窗口与托盘图标加载。
//!
//! - `load_window_icon`: 256x256 PNG, 窗口标题栏 / 任务栏缩略图
//! - `load_tray_icon`: 16x16 PNG, 系统托盘图标 (Windows 任务栏首选尺寸)
//! - `apply_windows_undecorated_style`: Windows 无边框窗口的圆角 / 阴影
//!
//! 加载失败时记录日志并返 `None`, 窗口创建不会因此 panic。

use winit::window::Icon;

/// 从 PNG 文件加载 winit 图标。
///
/// 将 PNG 解码为 RGBA 后,通过 [`Icon::from_rgba`] 创建图标。
/// 返回 `Err` 时调用方可选择回退到默认图标。
fn load_icon_from_png(path: &std::path::Path) -> Result<Icon, Box<dyn std::error::Error>> {
    let img = image::open(path)?.into_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).map_err(Into::into)
}

/// Windows 下为无边框窗口恢复圆角与阴影。
///
/// 使用 winit 公开的平台扩展 API, 避免手写 unsafe DWM 调用。
/// 若设置失败仅记录警告,不影响窗口功能。
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

/// 加载应用窗口图标。
///
/// 尝试读取 `assets/logo/logo_256.png`;
/// 失败时记录警告并返回 `None`, 避免窗口创建因图标问题而 panic。
pub(super) fn load_window_icon() -> Option<Icon> {
    let path = std::path::Path::new("assets")
        .join("logo")
        .join("logo_256.png");
    match load_icon_from_png(&path) {
        Ok(icon) => Some(icon),
        Err(err) => {
            log::warn!("加载窗口图标失败：{err}");
            None
        }
    }
}

/// 加载托盘图标 (16x16, Windows 任务栏首选尺寸)。
///
/// 读取 `assets/logo/logo_16.png`; 失败时记录警告并返回 `None`。
/// 返回 tray-icon 自身的 `Icon` 类型 (与 winit Icon 不通用)。
#[cfg(target_os = "windows")]
pub(super) fn load_tray_icon() -> Option<tray_icon::Icon> {
    let path = std::path::Path::new("assets")
        .join("logo")
        .join("logo_16.png");
    match image::open(&path) {
        Ok(img) => {
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            match tray_icon::Icon::from_rgba(rgba.into_raw(), width, height) {
                Ok(icon) => Some(icon),
                Err(err) => {
                    log::warn!("构建托盘 Icon 失败: {err}");
                    None
                }
            }
        }
        Err(err) => {
            log::warn!("加载托盘图标失败: {err}");
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
    fn load_icon_from_missing_path_returns_error() {
        let path = PathBuf::from("assets").join("logo").join("nonexistent.png");
        let icon = super::load_icon_from_png(&path);
        assert!(icon.is_err());
    }
}
