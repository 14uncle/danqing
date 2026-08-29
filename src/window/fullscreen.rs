//! @author 十四叔
//! @date 2026/08/30
//!
//! 全屏应用检测 (Windows): 前台是否存在其它应用的全屏窗口 —— [`WindowMode::
//! Adaptive`](super::WindowMode) 的「全屏自动暂停」传感器 (秒级留存层:
//! 用户在游戏/看全屏视频, 世界不抢 GPU, Wallpaper Engine 同款生存策略)。
//!
//! 双路线互补 (spike 实证见 danqing-deskscape Task 2):
//! - 路线 A `SHQueryUserNotificationState`: D3D 独占全屏 (游戏) 由系统报到;
//!   对无边框窗口化全屏 (浏览器 F11 / 现代游戏主流形态) 不报。
//! - 路线 B 前台矩形覆盖判定: 无边框全屏可靠检出; 须排除 Shell 窗口
//!   (Progman/WorkerW/任务栏 —— 用户看桌面时前台是 Shell 且铺满全屏,
//!   不排除会把世界误暂停在最常发生的场景)。
//!
//! 两路都纯查询无副作用; 失败一律返回 false (宁可不暂停, 不可误暂停)。
//! 非 Windows 平台恒 false (无检测, 永不暂停)。

/// 前台是否有其它应用的全屏窗口 (纯查询, 建议低频轮询 ~500ms)。
#[cfg(target_os = "windows")]
pub(crate) fn fullscreen_app_foreground() -> bool {
    quns_reports_fullscreen() || foreground_window_covers_monitor()
}

/// 非 Windows 平台无检测: 恒 false (永不暂停)。
#[cfg(not(target_os = "windows"))]
pub(crate) fn fullscreen_app_foreground() -> bool {
    false
}

/// 路线 A: 系统用户通知状态报 D3D 独占全屏。
#[cfg(target_os = "windows")]
fn quns_reports_fullscreen() -> bool {
    use windows_sys::Win32::UI::Shell::{
        QUNS_RUNNING_D3D_FULL_SCREEN, SHQueryUserNotificationState,
    };

    // SAFETY: 纯查询式系统调用, 输出参数为栈上 POD; 失败 (非 0) 返回 false。
    unsafe {
        let mut state = 0;
        if SHQueryUserNotificationState(&mut state) != 0 {
            return false;
        }
        state == QUNS_RUNNING_D3D_FULL_SCREEN
    }
}

/// 路线 B: 前台窗口矩形完全覆盖其所在显示器 (无边框全屏判定)。
#[cfg(target_os = "windows")]
fn foreground_window_covers_monitor() -> bool {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetShellWindow, GetWindowRect,
    };

    // SAFETY: 全部为纯查询式系统调用, 句柄/矩形均为借来的只读数据。
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() || hwnd == GetShellWindow() {
            return false;
        }
        // Shell 类排除: 桌面 (Progman) / 壁纸宿主 (WorkerW) / 任务栏铺满全屏,
        // 但它们是桌面本身, 不是「用户开了全屏应用」。
        let mut class = [0u16; 64];
        let len = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
        if len > 0 {
            let name = String::from_utf16_lossy(&class[..len as usize]);
            if matches!(
                name.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
            ) {
                return false;
            }
        }
        let mut rect: RECT = std::mem::zeroed();
        if GetWindowRect(hwnd, &raw mut rect) == 0 {
            return false;
        }
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..std::mem::zeroed()
        };
        if GetMonitorInfoW(monitor, &raw mut info) == 0 {
            return false;
        }
        let m = info.rcMonitor;
        // 覆盖判定: 窗口矩形四边均不内缩于显示器矩形 (最大化窗口只覆盖
        // 工作区, 任务栏边外露, 不会误判)。
        rect.left <= m.left && rect.top <= m.top && rect.right >= m.right && rect.bottom >= m.bottom
    }
}
