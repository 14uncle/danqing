//! @author 十四叔
//! @date 2026/10/10
//!
//! 窗口整体不透明度 (alpha): 拖出期间半透明反馈等瞬时效果
//! (Drop Shelf T6 缺口肌肉, 造物寄生)。
//!
//! Windows 实现: `WS_EX_LAYERED` + `SetLayeredWindowAttributes(LWA_ALPHA)`。
//! 与 [`super::passthrough`] 共享 LAYERED 标志 —— 关穿透会连 LAYERED 一并摘除,
//! 把不透明度效果废掉; 两者同窗同时使用会互踩, 同一时刻只用一种。
//! 非 Windows 平台为 no-op stub, 与 hotkey/startup 的平台门一致。

/// 不透明度 → alpha 字节 (0.0..=1.0 → 0..=255; 纯逻辑, 可单测)。
fn alpha_byte(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// 计算设置不透明度后的扩展样式 (纯函数, 便于单测):
/// 半透明 = 置位 `WS_EX_LAYERED`; 全不透明 = 摘除它, 回到创建时的呈现路径。
#[cfg(target_os = "windows")]
fn exstyle_for_opacity(style: u32, opacity: f32) -> u32 {
    use windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED;

    if opacity < 1.0 {
        style | WS_EX_LAYERED
    } else {
        style & !WS_EX_LAYERED
    }
}

/// 主窗口 HWND 槽 (danqing 单窗架构; 0 = 未就绪)。
///
/// 存在理由 (评审 R4-Optional): 拖出期间的半透明反馈必须在 `DoDragDrop`
/// **阻塞之前**生效, 而 `WindowEventSender` 走 mpsc 通道 —— 通道要等事件循环
/// 回到 `about_to_wait` 才被抽干, 那时拖拽早结束了。同步直调绕开通道。
static MAIN_HWND: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);

/// 窗口就绪时登记主窗口 HWND (handler 创建窗口时调一次)。
pub(crate) fn register_main_hwnd(hwnd: isize) {
    MAIN_HWND.store(hwnd, std::sync::atomic::Ordering::Relaxed);
}

/// 取主窗口 HWND (None = 未就绪)。
pub(crate) fn main_hwnd() -> Option<isize> {
    match MAIN_HWND.load(std::sync::atomic::Ordering::Relaxed) {
        0 => None,
        h => Some(h),
    }
}

/// 按 HWND 设置整体不透明度 (同步直调, 立即生效)。`opacity` 0.0~1.0。
#[cfg(target_os = "windows")]
pub(crate) fn set_opacity_hwnd(hwnd: isize, opacity: f32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, LWA_ALPHA, SetLayeredWindowAttributes, SetWindowLongPtrW,
    };

    let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        // MSDN: 返回 0 表示失败 (真实顶层窗口 exstyle 恒非零, 同 passthrough)。
        if style == 0 {
            log::warn!("设置不透明度失败: GetWindowLongPtrW 返回 0");
            return;
        }
        let next = exstyle_for_opacity(style as u32, opacity);
        if next != style as u32 && SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next as isize) == 0 {
            log::warn!("设置不透明度失败: SetWindowLongPtrW 返回 0");
            return;
        }
        if opacity < 1.0 {
            // LAYERED 已就位才设 alpha; 全不透明时 LAYERED 已摘, 呈现天然不透明。
            if SetLayeredWindowAttributes(hwnd, 0, alpha_byte(opacity), LWA_ALPHA) == 0 {
                log::warn!("设置不透明度失败: SetLayeredWindowAttributes 返回 0");
            }
        }
    }
}

/// 设置窗口整体不透明度 (Windows)。`opacity` 0.0 全透明 ~ 1.0 全不透明。
#[cfg(target_os = "windows")]
pub(crate) fn set_opacity(window: &winit::window::Window, opacity: f32) {
    let Some(hwnd) = super::foreground::hwnd_of(window) else {
        log::warn!("设置不透明度失败: 非 Win32 窗口句柄");
        return;
    };
    set_opacity_hwnd(hwnd as isize, opacity);
}

/// 非 Windows 平台: 不透明度未实现, 静默 no-op。
#[cfg(not(target_os = "windows"))]
pub(crate) fn set_opacity(_window: &winit::window::Window, _opacity: f32) {}

#[cfg(not(target_os = "windows"))]
pub(crate) fn set_opacity_hwnd(_hwnd: isize, _opacity: f32) {}

#[cfg(test)]
mod tests {
    use super::alpha_byte;

    #[test]
    fn alpha_byte_maps_unit_interval() {
        assert_eq!(alpha_byte(0.0), 0);
        assert_eq!(alpha_byte(0.5), 128);
        assert_eq!(alpha_byte(1.0), 255);
    }

    #[test]
    fn alpha_byte_clamps_out_of_range() {
        assert_eq!(alpha_byte(-1.0), 0);
        assert_eq!(alpha_byte(2.0), 255);
    }
}

#[cfg(all(test, target_os = "windows"))]
mod win_tests {
    use super::exstyle_for_opacity;
    use windows_sys::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED;

    #[test]
    fn translucent_adds_layered() {
        let next = exstyle_for_opacity(0x08, 0.6);
        assert_ne!(next & WS_EX_LAYERED, 0, "半透明必须置位 LAYERED");
        assert_ne!(next & 0x08, 0, "既有无关标志必须保留");
    }

    #[test]
    fn opaque_removes_layered_keeps_others() {
        let style = WS_EX_LAYERED | 0x08;
        let next = exstyle_for_opacity(style, 1.0);
        assert_eq!(next & WS_EX_LAYERED, 0, "全不透明摘掉 LAYERED 回创建路径");
        assert_eq!(next & 0x08, 0x08);
    }
}
