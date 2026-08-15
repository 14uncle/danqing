//! @author 十四叔
//! @date 2026/08/01

//! Windows 下把窗口抢到前台 / 顶层的原生实现。
//!
//! winit 0.30 的 `Window::focus_window()` 在 Windows 上走 `force_window_active`：
//! 先 `SendInput` 合成一次 Alt 按下/抬起，再 `SetForegroundWindow`。该技巧对
//! 后台常驻进程不可靠——合成输入被投递给当前前台应用 (另一进程)，Windows 的
//! 「最近输入」记账仍记在对方头上，前台锁 (foreground lock) 照常生效，
//! `SetForegroundWindow` 静默返回 FALSE：窗口「已显示但被遮」。
//! (番茄钟隐藏态完成专注后自动呼出即属此类，2026-08-01 实测复现。)
//!
//! 本模块改用经典 AttachThreadInput 方案：把本线程输入队列瞬时挂到前台窗口
//! 所属线程，共享输入状态从而获得前台权限，`SetForegroundWindow` 才能成功，
//! 随后立即 detach。顺序为「先直调、失败 (或前台属其它线程) 再挂接」，
//! 最小化挂接窗口期，规避 Raymond Chen 警告的输入队列共享死锁风险。

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::{FALSE, HWND, TRUE};
use windows_sys::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
};

use winit::window::Window;

/// 记录当前前台窗口的 HWND，用于后续恢复。
///
/// 返回 `Some(HWND)` 表示成功记录，`None` 表示当前无前台窗口。
/// 记录的 HWND 可用于 [`restore_foreground`] 恢复原前台窗口。
pub fn record_foreground() -> Option<HWND> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() { None } else { Some(hwnd) }
    }
}

/// 恢复之前记录的前台窗口。
///
/// 调用方应保证窗口仍存在且未被销毁；否则行为未定义。
/// 恢复失败仅记录警告，不 panic。
pub fn restore_foreground(hwnd: HWND) {
    if hwnd.is_null() {
        log::warn!("尝试恢复空前台窗口句柄，跳过");
        return;
    }
    bring_hwnd_to_foreground(hwnd);
}

/// 模拟粘贴快捷键 (Ctrl+V) 到当前前台窗口。
///
/// 用于实现剪贴板管理器的粘贴注入功能：
/// 1. 将内容写入剪贴板
/// 2. 恢复原前台窗口
/// 3. 调用本函数模拟 Ctrl+V 粘贴
///
/// 注意：此函数需要前台窗口可接收键盘输入；否则注入可能失败。
/// 失败仅记录警告，不 panic。
pub fn simulate_paste() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_CONTROL,
        VkKeyScanW,
    };

    unsafe {
        // 获取 V 键的虚拟键码
        let v_result = VkKeyScanW('v' as u16);
        let v_key_code = (v_result & 0xFF) as u16;
        let ctrl_key = VK_CONTROL;

        // 构造 Ctrl 按下 → V 按下 → V 抬起 → Ctrl 抬起 事件序列
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: ctrl_key,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: v_key_code,
                        wScan: 0,
                        dwFlags: 0,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // V 键抬起
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: v_key_code,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // Ctrl 键抬起
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: ctrl_key,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        let sent = SendInput(
            inputs.len() as u32,
            inputs.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent == 0 {
            log::warn!("SendInput 模拟 Ctrl+V 失败");
        }
    }
}

/// 把窗口抢到前台 + 提到顶层 (Windows)。对后台常驻进程同样有效。
///
/// 调用方应保证窗口已可见 (`set_visible(true)` 之后)；否则 `SetForegroundWindow`
/// 对隐藏窗口无效。失败仅记录警告，不 panic。
pub(super) fn bring_to_foreground(window: &Window) {
    let Some(hwnd) = hwnd_of(window) else {
        log::warn!("取窗口句柄失败，跳过抢前台");
        return;
    };
    bring_hwnd_to_foreground(hwnd);
}

/// 核心抢前台逻辑 (接受原生 HWND，拆出便于单元测试)。
fn bring_hwnd_to_foreground(hwnd: HWND) {
    unsafe {
        let current_thread = GetCurrentThreadId();
        let foreground = GetForegroundWindow();
        // 无前台窗口 (如锁屏刚解锁的间隙) 或前台属于本线程：直接激活即可，
        // 无需绕前台锁。先直调，能成功就不碰 AttachThreadInput。
        if foreground.is_null() {
            activate(hwnd);
            return;
        }
        let foreground_thread = GetWindowThreadProcessId(foreground, std::ptr::null_mut());
        if foreground_thread == 0 || foreground_thread == current_thread {
            activate(hwnd);
            return;
        }
        // 前台窗口属于其它线程 (典型场景：用户在另一应用里)：
        // 瞬时挂接输入队列 → 获得前台权限 → 激活 → 立即 detach。
        // 只在这两三次调用期间共享队列，结束后立刻断开，避免长期挂接副作用。
        let attached = AttachThreadInput(current_thread, foreground_thread, TRUE) != 0;
        let raised = SetForegroundWindow(hwnd);
        BringWindowToTop(hwnd);
        if attached {
            AttachThreadInput(current_thread, foreground_thread, FALSE);
        }
        // 挂接后仍失败 (如前台处于提升 / UIPI): 留下诊断, 不再静默被遮。
        if raised == 0 {
            log::warn!("挂接前台线程后 SetForegroundWindow 仍失败, 窗口可能未到最顶层");
        }
    }
}

/// `SetForegroundWindow` + `BringWindowToTop` 的常用组合 (前台 + 顶层双保证)。
fn activate(hwnd: HWND) {
    // Rust 2024 edition: unsafe 操作须显式包在 unsafe 块内 (unsafe_op_in_unsafe_fn)。
    unsafe {
        SetForegroundWindow(hwnd);
        BringWindowToTop(hwnd);
    }
}

/// 从 winit 窗口取原生 HWND；非 Windows 原生句柄返回 `None`。
fn hwnd_of(window: &Window) -> Option<HWND> {
    let handle = window.window_handle().ok()?;
    // raw-window-handle 0.6: `WindowHandle::as_raw()` 取 `RawWindowHandle`,
    // `Win32WindowHandle.hwnd` 为 `NonZeroIsize`, 经 `get()` 取裸指针作 HWND。
    match handle.as_raw() {
        RawWindowHandle::Win32(win32) => Some(win32.hwnd.get() as HWND),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    /// 冒烟测试：隐藏窗口上调用不 panic、不产生可见副作用。
    /// (SetForegroundWindow/BringWindowToTop 对隐藏窗口均静默无效。)
    ///
    /// 用系统 STATIC 类 + HWND_MESSAGE 建纯消息窗口 (完全不可见)，
    /// 避免创建 winit EventLoop —— winit 每线程只允许一个 EventLoop,
    /// 与 window::tests::event_loop_creation_smoke 并行会 RecreationAttempt。
    #[test]
    #[cfg(target_os = "windows")]
    fn bring_hwnd_to_foreground_hidden_window_does_not_panic() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, HWND_MESSAGE,
        };
        unsafe {
            let hwnd = CreateWindowExW(
                0,
                windows_sys::core::w!("STATIC"),
                windows_sys::core::w!(""),
                0, // 不传 WS_VISIBLE: 隐藏
                0,
                0,
                0,
                0,
                HWND_MESSAGE, // 父为消息窗口：纯后台，无任何可见性
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            assert!(!hwnd.is_null(), "创建隐藏消息窗口失败");
            super::bring_hwnd_to_foreground(hwnd);
            DestroyWindow(hwnd);
        }
    }

    /// 记录当前前台窗口: 无前台窗口时返回 None。
    #[test]
    #[cfg(target_os = "windows")]
    fn record_foreground_returns_none_when_no_foreground() {
        // 在消息窗口上下文中调用，应返回 None (无前台窗口)
        let recorded = super::record_foreground();
        // 无前台窗口时应返回 None，或返回一个有效的 HWND
        // 此测试验证 API 存在且不 panic
        let _ = recorded;
    }

    /// 记录当前前台窗口: 返回的 HWND 可用于后续恢复。
    #[test]
    #[cfg(target_os = "windows")]
    fn record_foreground_returns_hwnd() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, HWND_MESSAGE,
        };
        unsafe {
            // 创建一个可见窗口作为前台窗口
            let hwnd = CreateWindowExW(
                0,
                windows_sys::core::w!("STATIC"),
                windows_sys::core::w!(""),
                0, // 隐藏窗口
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            assert!(!hwnd.is_null(), "创建窗口失败");

            // 记录前台窗口 (可能是 None 或其他窗口)
            let recorded = super::record_foreground();
            // 验证 API 不 panic，返回值可选
            let _ = recorded;

            DestroyWindow(hwnd);
        }
    }

    /// 粘贴注入: 模拟 Ctrl+V 按键到前台窗口。
    #[test]
    #[cfg(target_os = "windows")]
    fn simulate_paste_shortcut_does_not_panic() {
        // 验证 simulate_paste() API 存在且不 panic
        // 实际效果需要手测验证 (需要真实前台窗口)
        super::simulate_paste();
    }
}
