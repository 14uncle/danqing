//! @author 十四叔
//! @date 2026/08/17
//!
//! 窗口显示落位 ([`ShowPlacement::Cursor`]): 每次显示前把窗口挪到鼠标光标处。
//!
//! 热键唤起的工具面板 (剪贴板管理器等) 不应恒在屏幕中央 —— 用户的视线
//! 与手都在光标附近。Windows 经 `GetCursorPos` 取全局光标物理坐标, 窗口
//! 左上角贴光标、整体钳进光标所在显示器的工作区 (避开任务栏)。

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::Window;

/// 窗口重新显示时的落位策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShowPlacement {
    /// 原位显示 (默认): 首次创建居中, 之后显隐不挪窝。
    #[default]
    Center,
    /// 跟随鼠标光标: 每次显示前挪到光标处 (左上角贴光标), 钳进光标所在
    /// 显示器的工作区。适用于热键唤起的工具面板 (剪贴板管理器等)。
    Cursor,
}

/// 把窗口挪到鼠标光标处 (物理像素): 左上角贴光标, 钳进所在显示器工作区。
///
/// `win_size` 用调用方持有的最后可信客户区尺寸 —— 隐藏态 winit 缓存可能是
/// 幻影值 (见 `Handler::last_real_size`); 无边框窗口客户区 ≈ 外框, 直接可用。
/// 取光标 / 显示器信息失败时保持原位 (静默降级, 居中总比不显示强)。
#[cfg(target_os = "windows")]
pub fn move_to_cursor(window: &Window, win_size: PhysicalSize<u32>) {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    // SAFETY: 纯查询式系统调用 + set_outer_position; 无内存/线程安全风险。
    unsafe {
        let mut cursor = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut cursor) == 0 {
            return;
        }
        // 光标所在显示器 (DEFAULTTONEAREST 纯防御: 理论上光标必在某屏)
        let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..std::mem::zeroed()
        };
        if GetMonitorInfoW(monitor, &raw mut info) == 0 {
            return;
        }
        let work = info.rcWork;
        let (x, y) = clamp_into_work_area(
            (cursor.x, cursor.y),
            (win_size.width as i32, win_size.height as i32),
            (work.left, work.top, work.right, work.bottom),
        );
        window.set_outer_position(PhysicalPosition::new(x, y));
    }
}

/// 非 Windows 平台暂无全局光标位置 API, 保持原位 (居中)。
#[cfg(not(target_os = "windows"))]
pub fn move_to_cursor(_window: &Window, _win_size: PhysicalSize<u32>) {}

/// 落位钳制 (纯函数): 窗口左上角贴光标, 但不许越出工作区;
/// 窗口比工作区还大时贴工作区左上。
pub(crate) fn clamp_into_work_area(
    cursor: (i32, i32),
    win: (i32, i32),
    work: (i32, i32, i32, i32),
) -> (i32, i32) {
    let (left, top, right, bottom) = work;
    // clamp 上界先托底到下界: 窗口比工作区大时 max < min, 直接 clamp 会 panic
    let x = cursor.0.clamp(left, (right - win.0).max(left));
    let y = cursor.1.clamp(top, (bottom - win.1).max(top));
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::clamp_into_work_area;

    /// 光标在工作区中部: 窗口左上角即光标。
    #[test]
    fn cursor_in_middle_stays() {
        assert_eq!(
            clamp_into_work_area((500, 400), (480, 640), (0, 0, 1920, 1040)),
            (500, 400)
        );
    }

    /// 光标贴右缘: 窗口右缘钳回工作区右缘。
    #[test]
    fn right_edge_clamped() {
        assert_eq!(
            clamp_into_work_area((1900, 400), (480, 640), (0, 0, 1920, 1040)),
            (1440, 400)
        );
    }

    /// 光标贴底缘 (任务栏上方): 窗口底缘钳回工作区底缘, 不压任务栏。
    #[test]
    fn bottom_edge_clamped_above_taskbar() {
        assert_eq!(
            clamp_into_work_area((500, 1030), (480, 640), (0, 0, 1920, 1040)),
            (500, 400)
        );
    }

    /// 主屏左侧副屏 (负坐标): 贴光标与钳右缘都正确。
    #[test]
    fn negative_coord_monitor() {
        let work = (-1920, 0, 0, 1080);
        assert_eq!(
            clamp_into_work_area((-1900, 100), (480, 640), work),
            (-1900, 100)
        );
        assert_eq!(
            clamp_into_work_area((-10, 100), (480, 640), work),
            (-480, 100)
        );
    }

    /// 窗口比工作区还大: 贴工作区左上, clamp 不 panic。
    #[test]
    fn oversized_window_pins_to_work_origin() {
        assert_eq!(
            clamp_into_work_area((500, 500), (2000, 1200), (0, 0, 1920, 1040)),
            (0, 0)
        );
    }
}
