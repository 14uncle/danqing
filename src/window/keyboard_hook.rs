//! @author 十四叔
//! @date 2026/11/19
//!
//! 低级键盘钩子平台原语 (WH_KEYBOARD_LL)。
//!
//! 做什么: 封装钩子线程、消息循环、安装/注销生命周期; 把键事件 (vk/down/injected)
//! 投递给调用方裁决「吞键还是透传」。不做: 不做 IO、不做剪贴板、不做阻塞等待——
//! 钩子回调必须极快, 重活由调用方的工作线程消化。
//! 为什么: 场景级「条件接管 Ctrl+V」需要可条件透传的钩子, `RegisterHotKey`
//! 无法限定仅终端生效 (2026-11-19, danqing-clipboard 场景 A)。

// ===== 纯决策 (零 Win32 依赖, 全部可测) =====

// Win32 VK_ 虚拟键码同值常量; 独立声明保持纯逻辑跨平台可测。
const VK_SHIFT: u32 = 0x10;
const VK_CONTROL: u32 = 0x11;
const VK_MENU: u32 = 0x12;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const VK_V: u32 = 0x56;
const VK_LSHIFT: u32 = 0xA0;
const VK_RSHIFT: u32 = 0xA1;
const VK_LCONTROL: u32 = 0xA2;
const VK_RCONTROL: u32 = 0xA3;
const VK_LMENU: u32 = 0xA4;
const VK_RMENU: u32 = 0xA5;

/// 低级键盘钩子回调产出的最小键事件。
#[derive(Debug, Clone, Copy)]
pub struct HookKey {
    /// 虚拟键码 (Win32 `VK_*` 同值)
    pub vk: u32,
    /// `true` = 按下, `false` = 抬起
    pub down: bool,
    /// `LLKHF_INJECTED`: 事件由 `SendInput` 注入 (自身补发的粘贴即此类)
    pub injected: bool,
}

/// 调用方对单次键事件的裁决。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookDecision {
    /// 透传 (`CallNextHookEx`)
    Pass,
    /// 吞键 (返回非零, 目标应用收不到该键)
    Swallow,
}

/// [`CtrlVCombo::feed`] 的匹配结果 (纯决策, 不含副作用)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComboEvent {
    /// 非目标组合: 应透传
    None,
    /// 命中「物理 Ctrl+V 按下」(每次物理触发仅首次): 应吞键并触发处理
    Matched,
    /// 命中后的自动重复或 V 抬起: 应吞键但不重复触发
    SwallowOnly,
}

/// 一组修饰键的逐键按下态 (泛码 + 左 + 右独立跟踪)。
///
/// 为什么逐键: 左右同按后放开一侧时, 组布尔会被误清——双 Shift 场景会把
/// Ctrl+Shift+V 误判成 Ctrl+V 吞键 (评审 O-2)。
#[derive(Debug, Default)]
struct ModState {
    generic: bool,
    left: bool,
    right: bool,
}

impl ModState {
    fn down(&self) -> bool {
        self.generic || self.left || self.right
    }
}

/// 物理 `Ctrl+V` (不含 Shift/Alt/Win) 匹配器。
///
/// 做什么: 消费 [`HookKey`] 流, 输出 [`ComboEvent`]。
/// 不做什么: 不做修饰键之外的组合键、不管焦点与前台进程 (调用方职责)。
/// 注入事件 (`injected`) 永不命中且不参与状态跟踪——自身补发的 `Ctrl+V`
/// 必须原样透传, 否则递归触发。
#[derive(Debug, Default)]
pub struct CtrlVCombo {
    ctrl: ModState,
    shift: ModState,
    alt: ModState,
    win: ModState,
    /// 已吞下 V 按下, 正在等自动重复/抬起
    armed: bool,
}

impl CtrlVCombo {
    /// 新建空状态匹配器 (无修饰键按下)。
    pub fn new() -> Self {
        Self::default()
    }

    /// 消费一个键事件, 返回裁决建议。
    pub fn feed(&mut self, key: &HookKey) -> ComboEvent {
        if key.injected {
            return ComboEvent::None;
        }
        match key.vk {
            VK_CONTROL => {
                self.ctrl.generic = key.down;
                ComboEvent::None
            }
            VK_LCONTROL => {
                self.ctrl.left = key.down;
                ComboEvent::None
            }
            VK_RCONTROL => {
                self.ctrl.right = key.down;
                ComboEvent::None
            }
            VK_SHIFT => {
                self.shift.generic = key.down;
                ComboEvent::None
            }
            VK_LSHIFT => {
                self.shift.left = key.down;
                ComboEvent::None
            }
            VK_RSHIFT => {
                self.shift.right = key.down;
                ComboEvent::None
            }
            VK_MENU => {
                self.alt.generic = key.down;
                ComboEvent::None
            }
            VK_LMENU => {
                self.alt.left = key.down;
                ComboEvent::None
            }
            VK_RMENU => {
                self.alt.right = key.down;
                ComboEvent::None
            }
            VK_LWIN => {
                self.win.left = key.down;
                ComboEvent::None
            }
            VK_RWIN => {
                self.win.right = key.down;
                ComboEvent::None
            }
            VK_V => {
                if key.down {
                    if self.armed {
                        // 按住 V 的自动重复: 吞键, 不重复触发
                        ComboEvent::SwallowOnly
                    } else if self.ctrl.down()
                        && !self.shift.down()
                        && !self.alt.down()
                        && !self.win.down()
                    {
                        self.armed = true;
                        ComboEvent::Matched
                    } else {
                        ComboEvent::None
                    }
                } else if self.armed {
                    self.armed = false;
                    ComboEvent::SwallowOnly
                } else {
                    ComboEvent::None
                }
            }
            _ => ComboEvent::None,
        }
    }
}

// ===== Win32 安装层 (副作用集中在本段) =====

use std::sync::Mutex;

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, PM_NOREMOVE, PeekMessageW,
    PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN,
    WM_QUIT, WM_SYSKEYDOWN,
};

/// 钩子事件处理器: 同步裁决吞键/透传。
///
/// 必须极快 (纯决策 + 无阻塞投递); 重活交给调用方的工作线程。
type HookHandler = Box<dyn Fn(HookKey) -> HookDecision + Send>;

static HANDLER: Mutex<Option<HookHandler>> = Mutex::new(None);

/// 键盘钩子句柄: drop 即注销钩子并回收钩子线程。
pub struct KeyboardHookHandle {
    thread_id: u32,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl KeyboardHookHandle {
    /// 显式注销 (drop 亦可)。
    pub fn uninstall(mut self) {
        self.stop();
    }

    fn stop(&mut self) {
        if let Some(t) = self.thread.take() {
            unsafe {
                // 消息队列已由 hook_thread 提前创建 (PeekMessageW), 此处失败极罕见;
                // 有限重试防「投递丢失 → join 永久挂起」(评审 O-3)
                let mut posted = false;
                for _ in 0..20 {
                    if PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0) != 0 {
                        posted = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                if !posted {
                    log::warn!("键盘钩子 WM_QUIT 投递失败, 钩子线程可能残留");
                }
            }
            let _ = t.join();
        }
        // 锁中毒也必须清 handler, 否则后续安装永远报「已安装」(评审 O-4)
        let mut guard = HANDLER.lock().unwrap_or_else(|p| p.into_inner());
        *guard = None;
    }
}

impl Drop for KeyboardHookHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 安装低级键盘钩子 (仅支持单实例, 重复安装返回错误)。
///
/// 做什么: 起钩子线程装 `WH_KEYBOARD_LL`, 每个键事件先经 `handler` 裁决,
/// [`HookDecision::Swallow`] 吞键, 其余 `CallNextHookEx` 透传。
/// 不做什么: 不吞默认透传的键、不在回调里做 IO; 安装失败只报错不 panic。
/// 为什么: 钩子回调与消息泵必须同线程, 由专用线程承载, 不占用调用方线程。
pub fn install_keyboard_hook<F>(handler: F) -> std::io::Result<KeyboardHookHandle>
where
    F: Fn(HookKey) -> HookDecision + Send + 'static,
{
    {
        let mut guard = HANDLER
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if guard.is_some() {
            return Err(std::io::Error::other("键盘钩子已安装 (仅支持单实例)"));
        }
        *guard = Some(Box::new(handler));
    }

    let (tx, rx) = std::sync::mpsc::channel::<Result<u32, String>>();
    let thread = std::thread::Builder::new()
        .name("danqing-keyboard-hook".into())
        .spawn(move || hook_thread(tx))?;

    match rx.recv() {
        Ok(Ok(thread_id)) => Ok(KeyboardHookHandle {
            thread_id,
            thread: Some(thread),
        }),
        Ok(Err(e)) => {
            let _ = thread.join();
            let mut guard = HANDLER.lock().unwrap_or_else(|p| p.into_inner());
            *guard = None;
            Err(std::io::Error::other(e))
        }
        Err(_) => {
            let _ = thread.join();
            let mut guard = HANDLER.lock().unwrap_or_else(|p| p.into_inner());
            *guard = None;
            Err(std::io::Error::other("钩子线程提前退出"))
        }
    }
}

fn hook_thread(tx: std::sync::mpsc::Sender<Result<u32, String>>) {
    // edition 2024: unsafe 操作收进显式块, 函数本身保持安全签名
    unsafe {
        let thread_id = GetCurrentThreadId();
        let hmod = GetModuleHandleW(std::ptr::null());
        if hmod.is_null() {
            let _ = tx.send(Err(format!(
                "GetModuleHandleW 失败: {}",
                std::io::Error::last_os_error()
            )));
            return;
        }
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0);
        if hook.is_null() {
            let _ = tx.send(Err(format!(
                "SetWindowsHookExW 失败: {}",
                std::io::Error::last_os_error()
            )));
            return;
        }
        // 强制创建本线程消息队列后再报到, 否则极早的 WM_QUIT 可能投递失败 (评审 O-3)
        let mut msg: MSG = std::mem::zeroed();
        PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
        let _ = tx.send(Ok(thread_id));
        // 消息泵: 低级钩子事件在本线程消息检索期间被投递
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            // LL 钩子无窗口消息需要 Dispatch, 泵本身即投递机制
        }
        UnhookWindowsHookEx(hook);
    }
}

unsafe extern "system" fn hook_proc(n_code: i32, w_param: usize, l_param: isize) -> isize {
    unsafe {
        if n_code >= 0 {
            let kb = &*(l_param as *const KBDLLHOOKSTRUCT);
            let down = matches!(w_param as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
            let key = HookKey {
                vk: kb.vkCode,
                down,
                injected: kb.flags & LLKHF_INJECTED != 0,
            };
            let decision = HANDLER.lock().ok().and_then(|guard| {
                guard.as_ref().and_then(|handler| {
                    // handler panic 不得穿越 extern "system" 边界 (会 abort 进程),
                    // 捕获后按透传处理 (评审 O-4)
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(key))).ok()
                })
            });
            if matches!(decision, Some(HookDecision::Swallow)) {
                return 1;
            }
        }
        CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(vk: u32, down: bool) -> HookKey {
        HookKey {
            vk,
            down,
            injected: false,
        }
    }

    #[test]
    fn ctrl_v_down_matches() {
        let mut combo = CtrlVCombo::new();
        assert!(matches!(
            combo.feed(&key(VK_CONTROL, true)),
            ComboEvent::None
        ));
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::Matched));
    }

    #[test]
    fn side_modifier_keys_match_independently() {
        // 真实 LL 钩子报 VK_LCONTROL/VK_RCONTROL, 泛码只在非 LL 场景出现
        for vk in [VK_LCONTROL, VK_RCONTROL] {
            let mut combo = CtrlVCombo::new();
            combo.feed(&key(vk, true));
            assert!(
                matches!(combo.feed(&key(VK_V, true)), ComboEvent::Matched),
                "vk={vk:#x} 应命中"
            );
        }
    }

    #[test]
    fn dual_shift_release_one_keeps_exclusion() {
        // 双 Shift 同按后放开一侧: 组仍视为 Shift 按下, 不得误判成 Ctrl+V (评审 O-2)
        let mut combo = CtrlVCombo::new();
        combo.feed(&key(VK_CONTROL, true));
        combo.feed(&key(VK_LSHIFT, true));
        combo.feed(&key(VK_RSHIFT, true));
        combo.feed(&key(VK_RSHIFT, false));
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::None));
    }

    #[test]
    fn plain_v_passes() {
        let mut combo = CtrlVCombo::new();
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::None));
        assert!(matches!(combo.feed(&key(VK_V, false)), ComboEvent::None));
    }

    #[test]
    fn shift_alt_win_excluded() {
        // Ctrl+Shift+V
        let mut combo = CtrlVCombo::new();
        combo.feed(&key(VK_CONTROL, true));
        combo.feed(&key(VK_SHIFT, true));
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::None));
        // Ctrl+Alt+V
        let mut combo = CtrlVCombo::new();
        combo.feed(&key(VK_CONTROL, true));
        combo.feed(&key(VK_MENU, true));
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::None));
        // Ctrl+Win+V
        let mut combo = CtrlVCombo::new();
        combo.feed(&key(VK_CONTROL, true));
        combo.feed(&key(VK_LWIN, true));
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::None));
    }

    #[test]
    fn auto_repeat_swallows_without_retrigger() {
        let mut combo = CtrlVCombo::new();
        combo.feed(&key(VK_CONTROL, true));
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::Matched));
        // 按住 V 的自动重复: 吞键但不重复触发
        assert!(matches!(
            combo.feed(&key(VK_V, true)),
            ComboEvent::SwallowOnly
        ));
        assert!(matches!(
            combo.feed(&key(VK_V, true)),
            ComboEvent::SwallowOnly
        ));
        // 抬起: 吞掉防残留, 随后解除武装
        assert!(matches!(
            combo.feed(&key(VK_V, false)),
            ComboEvent::SwallowOnly
        ));
        // 抬起后再次按下 = 新一次物理触发
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::Matched));
    }

    #[test]
    fn v_up_without_match_passes() {
        let mut combo = CtrlVCombo::new();
        assert!(matches!(combo.feed(&key(VK_V, false)), ComboEvent::None));
    }

    #[test]
    fn injected_never_matches_nor_tracks() {
        let mut combo = CtrlVCombo::new();
        // 真实 Ctrl 按下
        combo.feed(&key(VK_CONTROL, true));
        // 注入的 V: 永不命中 (simulate_paste 补发的 Ctrl+V 必须透传防递归)
        let injected_v = HookKey {
            vk: VK_V,
            down: true,
            injected: true,
        };
        assert!(matches!(combo.feed(&injected_v), ComboEvent::None));
        // 注入的 Ctrl 抬起: 不污染真实修饰键状态
        let injected_ctrl_up = HookKey {
            vk: VK_CONTROL,
            down: false,
            injected: true,
        };
        assert!(matches!(combo.feed(&injected_ctrl_up), ComboEvent::None));
        // 真实 Ctrl 仍在按下态, V 仍应命中
        assert!(matches!(combo.feed(&key(VK_V, true)), ComboEvent::Matched));
    }
}
