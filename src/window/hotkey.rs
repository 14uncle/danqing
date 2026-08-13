//! @author 十四叔
//! @date 2026/07/17

//! 全局热键子系统。
//!
//! - `GlobalHotkey`: 一枚热键的声明 (修饰键 + 虚拟键码), 产品经
//!   `WindowConfig::hotkeys` 显式声明自己的热键集合
//! - `hotkey_ids` 常量: 注册到 winit 平台层 + App 消费时按 ID 映射到 `Msg`
//!   (遗留: 常量语义仍属首个消费者番茄钟, 彻底泛化留作后续)
//! - `hotkeys::spawn()`: 启 Windows 线程做 RegisterHotKey + GetMessage 循环
//!   (非 Windows 平台或空声明返 None)

/// 全局热键 ID 常量 (PomodoroApp 消费时按 ID 映射到 `Msg`)。
pub mod hotkey_ids {
    /// 显隐窗口 (Ctrl+Shift+P)。
    pub const TOGGLE_VISIBLE: u8 = 1;
    /// 开始/暂停番茄钟 (Ctrl+Shift+S)。
    pub const START_PAUSE: u8 = 2;
    /// 退出应用 (Ctrl+Shift+Q)。
    pub const QUIT: u8 = 3;
}

/// 一枚全局热键的声明: ID + 修饰键 + 虚拟键码。
///
/// `vk` 为 Windows 虚拟键码 (如 V 键 = 0x56); 非 Windows 平台忽略整份声明。
/// MOD_NOREPEAT 恒带 (长按不连发)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalHotkey {
    /// 热键 ID: WM_HOTKEY 的 wParam, 经 `App::hotkey` 路由为应用消息。
    pub id: u8,
    /// 虚拟键码 (字母键即 ASCII 大写码, 如 0x56 = V)。
    pub vk: u32,
    /// Ctrl 修饰。
    pub ctrl: bool,
    /// Shift 修饰。
    pub shift: bool,
    /// Alt 修饰。
    pub alt: bool,
}

impl GlobalHotkey {
    /// Ctrl+Shift+某键 (常驻工具唤起键的常用组合)。
    pub const fn ctrl_shift(id: u8, vk: u32) -> Self {
        Self {
            id,
            vk,
            ctrl: true,
            shift: true,
            alt: false,
        }
    }
}

#[cfg(target_os = "windows")]
pub(super) mod hotkeys {
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread::{self, JoinHandle};

    use super::GlobalHotkey;

    /// Windows 启动全局热键监听线程:
    /// 1. `RegisterHotKey(NULL, ...)` 关联到当前线程消息队列
    /// 2. 标准 `GetMessage/DispatchMessage` 循环
    /// 3. `WM_HOTKEY` 时通过 `tx` 把热键 ID 发送给主线程
    /// 4. 主线程 `about_to_wait` 轮询, 转 `Msg`
    ///
    /// 空声明不启动线程 (产品可借此完全关闭全局热键)。
    pub fn spawn(specs: &[GlobalHotkey]) -> Option<(Receiver<u8>, JoinHandle<()>)> {
        if specs.is_empty() {
            log::info!("未声明全局热键, 热键线程不启动");
            return None;
        }
        let specs = specs.to_vec();
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = thread::Builder::new()
            .name("danqing-hotkey".into())
            .spawn(move || unsafe {
                run(tx, &specs);
            });
        match handle {
            Ok(h) => Some((rx, h)),
            Err(err) => {
                log::warn!("hotkey 线程启动失败: {err}");
                None
            }
        }
    }

    /// 修饰键位组装 (MOD_NOREPEAT 恒带)。
    fn mods_of(spec: &GlobalHotkey) -> u32 {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
        };
        let mut mods = MOD_NOREPEAT;
        if spec.ctrl {
            mods |= MOD_CONTROL;
        }
        if spec.shift {
            mods |= MOD_SHIFT;
        }
        if spec.alt {
            mods |= MOD_ALT;
        }
        mods
    }

    unsafe fn run(tx: Sender<u8>, specs: &[GlobalHotkey]) {
        use windows_sys::Win32::Foundation::HWND;
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, MSG, PM_NOREMOVE, PeekMessageW, TranslateMessage,
            WM_HOTKEY,
        };

        let hwnd: HWND = std::ptr::null_mut();

        // 关键: 线程必须有消息队列 `RegisterHotKey` 才会把 WM_HOTKEY 派进来。
        // std::thread::spawn 出来的线程默认**没有**消息队列, 必须先用 PeekMessageW
        // 触发一次队列创建 (PM_NOREMOVE 不取走消息, 安全)。
        let mut peek_msg: MSG = unsafe { std::mem::zeroed() };
        unsafe {
            PeekMessageW(&mut peek_msg, hwnd, 0, 0, PM_NOREMOVE);
        }
        log::info!("[hotkey thread] 消息队列已创建");

        let mut registered: Vec<i32> = Vec::with_capacity(specs.len());
        for spec in specs {
            if unsafe { RegisterHotKey(hwnd, spec.id as i32, mods_of(spec), spec.vk) } == 0 {
                log::warn!(
                    "RegisterHotKey 失败: id={} vk=0x{:02X} (与其它应用冲突?)",
                    spec.id,
                    spec.vk
                );
                for id in &registered {
                    unsafe {
                        UnregisterHotKey(hwnd, *id);
                    }
                }
                return;
            }
            registered.push(spec.id as i32);
        }
        log::info!("全局热键已注册: {specs:?}");

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

        for id in &registered {
            unsafe {
                UnregisterHotKey(hwnd, *id);
            }
        }
        log::info!("全局热键已注销");
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ctrl_shift_mods_include_norepeat() {
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
            };
            let spec = GlobalHotkey::ctrl_shift(1, 0x56);
            assert_eq!(mods_of(&spec), MOD_NOREPEAT | MOD_CONTROL | MOD_SHIFT);
        }

        #[test]
        fn no_mods_means_only_norepeat() {
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::MOD_NOREPEAT;
            let spec = GlobalHotkey {
                id: 1,
                vk: 0x56,
                ctrl: false,
                shift: false,
                alt: false,
            };
            assert_eq!(mods_of(&spec), MOD_NOREPEAT);
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub(super) mod hotkeys {
    use std::sync::mpsc::Receiver;
    use std::thread::JoinHandle;

    use super::GlobalHotkey;

    /// 非 Windows 平台: 全局热键 unavailable, 返回 None。
    pub fn spawn(_specs: &[GlobalHotkey]) -> Option<(Receiver<u8>, JoinHandle<()>)> {
        log::info!("global hotkeys unsupported on this platform");
        None
    }
}
