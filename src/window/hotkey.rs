//! @author 十四叔
//! @date 2026/07/17

//! 全局热键子系统。
//!
//! - `hotkey_ids` 常量: 注册到 winit 平台层 + App 消费时按 ID 映射到 `Msg`
//! - `hotkeys::spawn()`: 启 Windows 线程做 RegisterHotKey + GetMessage 循环
//!   (非 Windows 平台返 None)

/// 全局热键 ID 常量 (PomodoroApp 消费时按 ID 映射到 `Msg`)。
pub mod hotkey_ids {
    /// 显隐窗口 (Ctrl+Shift+P)。
    pub const TOGGLE_VISIBLE: u8 = 1;
    /// 开始/暂停番茄钟 (Ctrl+Shift+S)。
    pub const START_PAUSE: u8 = 2;
    /// 退出应用 (Ctrl+Shift+Q)。
    pub const QUIT: u8 = 3;
}

#[cfg(target_os = "windows")]
pub(super) mod hotkeys {
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread::{self, JoinHandle};

    /// Windows 启动全局热键监听线程:
    /// 1. `RegisterHotKey(NULL, ...)` 关联到当前线程消息队列
    /// 2. 标准 `GetMessage/DispatchMessage` 循环
    /// 3. `WM_HOTKEY` 时通过 `tx` 把热键 ID 发送给主线程
    /// 4. 主线程 `about_to_wait` 轮询, 转 `Msg`
    pub fn spawn() -> Option<(Receiver<u8>, JoinHandle<()>)> {
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = thread::Builder::new()
            .name("danqing-hotkey".into())
            .spawn(move || unsafe {
                run(tx);
            });
        match handle {
            Ok(h) => Some((rx, h)),
            Err(err) => {
                log::warn!("hotkey 线程启动失败: {err}");
                None
            }
        }
    }

    unsafe fn run(tx: Sender<u8>) {
        use crate::window::hotkey_ids;
        use windows_sys::Win32::Foundation::HWND;
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, RegisterHotKey, UnregisterHotKey,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MSG, PM_NOREMOVE, PeekMessageW, TranslateMessage,
            WM_HOTKEY,
        };

        // 虚拟键码: P=0x50, S=0x53, Q=0x51
        const VK_P: u32 = 0x50;
        const VK_S: u32 = 0x53;
        const VK_Q: u32 = 0x51;
        const MODS: u32 = MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT;

        let toggle_id = hotkey_ids::TOGGLE_VISIBLE as i32;
        let start_pause_id = hotkey_ids::START_PAUSE as i32;
        let quit_id = hotkey_ids::QUIT as i32;

        let hwnd: HWND = std::ptr::null_mut();

        // 关键: 线程必须有消息队列 `RegisterHotKey` 才会把 WM_HOTKEY 派进来。
        // std::thread::spawn 出来的线程默认**没有**消息队列, 必须先用 PeekMessageW
        // 触发一次队列创建 (PM_NOREMOVE 不取走消息, 安全)。
        let mut peek_msg: MSG = unsafe { std::mem::zeroed() };
        unsafe {
            PeekMessageW(&mut peek_msg, hwnd, 0, 0, PM_NOREMOVE);
        }
        log::info!("[hotkey thread] 消息队列已创建");

        let mut ok = true;
        if unsafe { RegisterHotKey(hwnd, toggle_id, MODS, VK_P) } == 0 {
            log::warn!("RegisterHotKey Ctrl+Shift+P 失败");
            ok = false;
        }
        if ok && unsafe { RegisterHotKey(hwnd, start_pause_id, MODS, VK_S) } == 0 {
            log::warn!("RegisterHotKey Ctrl+Shift+S 失败");
            unsafe {
                UnregisterHotKey(hwnd, toggle_id);
            }
            ok = false;
        }
        if ok && unsafe { RegisterHotKey(hwnd, quit_id, MODS, VK_Q) } == 0 {
            log::warn!("RegisterHotKey Ctrl+Shift+Q 失败");
            unsafe {
                UnregisterHotKey(hwnd, toggle_id);
                UnregisterHotKey(hwnd, start_pause_id);
            }
            ok = false;
        }
        if !ok {
            return;
        }
        log::info!("全局热键已注册: Ctrl+Shift+P/S/Q");

        let mut msg: MSG = unsafe { std::mem::zeroed() };
        loop {
            // GetMessage 阻塞直到有消息; 返回 0 表示收到 WM_QUIT (退出)
            if unsafe { GetMessageW(&mut msg, hwnd, 0, 0) } <= 0 {
                break;
            }
            if msg.message == WM_HOTKEY {
                let id = (msg.wParam as u32) & 0xFF;
                log::debug!("[hotkey thread] WM_HOTKEY id={id}");
                let _ = tx.send(id as u8);
            }
            log::debug!(
                "[hotkey thread] msg=0x{:x} wparam={}",
                msg.message,
                msg.wParam
            );
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }

        unsafe {
            UnregisterHotKey(hwnd, toggle_id);
            UnregisterHotKey(hwnd, start_pause_id);
            UnregisterHotKey(hwnd, quit_id);
        }
        log::info!("全局热键已注销");
    }
}

#[cfg(not(target_os = "windows"))]
pub(super) mod hotkeys {
    use std::sync::mpsc::Receiver;
    use std::thread::JoinHandle;

    /// 非 Windows 平台: 全局热键 unavailable, 返回 None。
    pub fn spawn() -> Option<(Receiver<u8>, JoinHandle<()>)> {
        log::info!("global hotkeys unsupported on this platform");
        None
    }
}
