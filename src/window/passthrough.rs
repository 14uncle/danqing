//! @author 十四叔
//! @date 2026/08/28
//!
//! 点击穿透 (桌面常驻陪伴形态)。
//!
//! 穿透态下窗口不收鼠标事件 —— 命中测试直达下层窗口, 小世界纯观赏;
//! 切回后恢复正常交互。Windows 实现: `WS_EX_LAYERED | WS_EX_TRANSPARENT`
//! 扩展样式切换 (`SetWindowLongPtrW(GWL_EXSTYLE)`); 关闭时两标志一并摘除,
//! 回到创建时的呈现路径 (danqing 窗口创建时不带 LAYERED)。
//!
//! 已知风险 (plan-desk-window 已记, 预授权 fallback): LAYERED 与 wgpu
//! swapchain 的呈现兼容性需 showcase 实测; 备用路线为 SetWindowSubclass
//! 拦 `WM_NCHITTEST` 返回 `HTTRANSPARENT`。
//!
//! 非 Windows 平台为 no-op stub, 与 hotkey/startup 的平台门一致。

/// 计算切换点击穿透后的扩展样式 (纯函数, 便于单测):
/// 开 = 置位 `WS_EX_LAYERED | WS_EX_TRANSPARENT`; 关 = 两标志一并摘除,
/// 其余既有标志原样保留。
#[cfg(target_os = "windows")]
fn exstyle_for_click_through(style: u32, enabled: bool) -> u32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::{WS_EX_LAYERED, WS_EX_TRANSPARENT};

    if enabled {
        style | WS_EX_LAYERED | WS_EX_TRANSPARENT
    } else {
        style & !WS_EX_LAYERED & !WS_EX_TRANSPARENT
    }
}

/// 切换窗口点击穿透 (Windows)。幂等: 目标样式已是现状时不动窗口。
#[cfg(target_os = "windows")]
pub(crate) fn set_click_through(window: &winit::window::Window, enabled: bool) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, SetWindowLongPtrW,
    };

    let Some(hwnd) = super::foreground::hwnd_of(window) else {
        log::warn!("点击穿透切换失败: 非 Win32 窗口句柄");
        return;
    };
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        // MSDN: 返回 0 表示失败 (真实顶层窗口 exstyle 恒非零,
        // 无需 SetLastError 区分)。
        if style == 0 {
            log::warn!("点击穿透切换失败: GetWindowLongPtrW 返回 0");
            return;
        }
        let next = exstyle_for_click_through(style as u32, enabled);
        if next == style as u32 {
            return;
        }
        if SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next as isize) == 0 {
            log::warn!("点击穿透切换失败: SetWindowLongPtrW 返回 0");
        }
    }
}

/// 非 Windows 平台: 点击穿透未实现, 静默 no-op。
#[cfg(not(target_os = "windows"))]
pub(crate) fn set_click_through(_window: &winit::window::Window, _enabled: bool) {}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::exstyle_for_click_through;
    use windows_sys::Win32::UI::WindowsAndMessaging::{WS_EX_LAYERED, WS_EX_TRANSPARENT};

    #[test]
    fn enable_adds_layered_and_transparent() {
        let next = exstyle_for_click_through(0, true);
        assert_ne!(next & WS_EX_LAYERED, 0, "开穿透必须置位 LAYERED");
        assert_ne!(next & WS_EX_TRANSPARENT, 0, "开穿透必须置位 TRANSPARENT");
    }

    #[test]
    fn disable_removes_both_flags_but_keeps_others() {
        // 0x08 模拟一个既有的无关标志 (如 WS_EX_TOPMOST)。
        let style = WS_EX_LAYERED | WS_EX_TRANSPARENT | 0x08;
        let next = exstyle_for_click_through(style, false);
        assert_eq!(next & WS_EX_LAYERED, 0, "关穿透必须摘掉 LAYERED");
        assert_eq!(next & WS_EX_TRANSPARENT, 0, "关穿透必须摘掉 TRANSPARENT");
        assert_ne!(next & 0x08, 0, "既有无关标志必须保留");
    }

    #[test]
    fn disable_on_plain_style_is_noop() {
        assert_eq!(exstyle_for_click_through(0x08, false), 0x08);
    }

    #[test]
    fn enable_is_idempotent() {
        let once = exstyle_for_click_through(0, true);
        assert_eq!(exstyle_for_click_through(once, true), once);
    }
}
