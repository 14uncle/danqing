//! @author 十四叔
//! @date 2026/11/26
//!
//! 无激活微提示窗: 粘贴后数秒内告诉用户「发生了什么、还能做什么」。
//!
//! 为什么独立成窗: 主面板是 480×640 的可聚焦窗口, 拿它当提示面会抢焦点、
//! 扰动面板落位与关闭行为; 提示必须是 `WS_EX_NOACTIVATE` 的独立弹出窗。
//!
//! Windows 实现: 模块自有线程 + 消息循环持有唯一窗口; 分层窗口
//! (`WS_EX_LAYERED | WS_EX_TRANSPARENT`) 配 `SetLayeredWindowAttributes(LWA_ALPHA)`,
//! GDI 直接画字; `ShowWindow(SW_SHOWNOACTIVATE)` 显示、`WS_EX_TOOLWINDOW` 不进
//! 任务栏与 Alt+Tab。定位优先读前台线程插入符 (`GetGUIThreadInfo`), 取不到回退
//! 鼠标光标, 再钳进显示器工作区。
//!
//! 不做什么: 不抢焦点、不接收鼠标、不进入任务栏、不持有剪贴板、不联网、不
//! 读取文档正文; 文本由调用方提供, 本模块只负责显示与到期隐藏。
//!
//! 非 Windows 平台为 `Unsupported` 错误 + no-op, 与 hotkey/startup 的平台门一致。

/// 提示窗建不起来的原因 (不携带窗口标题或文本内容)。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HintError {
    /// 当前平台没有提示窗实现。
    #[error("提示窗仅支持 Windows")]
    Unsupported,
    /// 提示窗线程启动或初始化失败。
    #[error("提示窗线程启动失败")]
    ThreadStart,
    /// 注册窗口类或创建窗口失败。
    #[error("提示窗创建失败")]
    WindowCreate,
}

/// 提示文本按行切分: 去 `\r`, 最多保留 4 行 (产品固定两行)。
///
/// 纯函数; 空文本保留一个空行, 由绘制侧按空行测量。
fn split_hint_lines(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = text
        .split('\n')
        .take(4)
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// 96 DPI 基准缩放因子, 最小 1 倍。
fn scale_for_dpi(dpi: i32) -> i32 {
    ((dpi as f32 / 96.0).round() as i32).max(1)
}

/// 锚点 + 偏移 → 提示窗左上角, 钳进工作区。
///
/// 纯函数; 窗口比工作区还大时贴工作区左上, 不遮挡锚点由偏移量保证。
fn position_with_work(
    anchor: (i32, i32),
    offset: (i32, i32),
    size: (i32, i32),
    work: (i32, i32, i32, i32),
) -> (i32, i32) {
    super::placement::clamp_into_work_area((anchor.0 + offset.0, anchor.1 + offset.1), size, work)
}

#[cfg(target_os = "windows")]
mod win {
    use std::mem::size_of;
    use std::sync::mpsc::{Receiver, Sender};
    use std::time::Duration;

    use windows_sys::Win32::Foundation::{
        COLORREF, ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM,
    };
    use windows_sys::Win32::Graphics::Gdi::{
        BeginPaint, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, CreateRoundRectRgn,
        CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_PITCH, DT_CALCRECT, DT_LEFT, DT_NOPREFIX,
        DT_SINGLELINE, DT_TOP, DeleteObject, DrawTextW, EndPaint, FF_DONTCARE, FW_NORMAL, FillRect,
        GetDC, GetDeviceCaps, GetMonitorInfoW, InvalidateRect, LOGPIXELSY,
        MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
        ReleaseDC, SelectObject, SetBkMode, SetTextColor, SetWindowRgn, TRANSPARENT, UpdateWindow,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GUITHREADINFO,
        GWLP_USERDATA, GetClientRect, GetCursorPos, GetForegroundWindow, GetGUIThreadInfo,
        GetMessageW, GetWindowLongPtrW, GetWindowThreadProcessId, HWND_TOPMOST, KillTimer,
        LWA_ALPHA, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, RegisterClassW, SW_HIDE,
        SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SetLayeredWindowAttributes, SetTimer, SetWindowLongPtrW,
        SetWindowPos, ShowWindow, TranslateMessage, WM_APP, WM_DESTROY, WM_PAINT, WM_TIMER,
        WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
        WS_EX_TRANSPARENT, WS_POPUP,
    };

    use super::{HintError, position_with_work, scale_for_dpi, split_hint_lines};

    /// 线程消息: 唤醒提示窗线程并抽干命令队列。
    const WM_HINT_COMMAND: u32 = WM_APP + 1;
    /// TTL 定时器 ID。
    const TIMER_ID: usize = 1;
    /// 背景 alpha: 约 93% 不透明, 保留轻微透视。
    const HINT_ALPHA: u8 = 238;

    fn rgb(r: u32, g: u32, b: u32) -> COLORREF {
        r | (g << 8) | (b << 16)
    }

    enum Command {
        Show { text: String, ttl: Duration },
        Hide,
        Shutdown,
    }

    /// 绘制状态; 以 `GWLP_USERDATA` 挂在窗口上, 仅提示窗线程访问。
    struct PaintState {
        lines: Vec<String>,
        font: windows_sys::Win32::Graphics::Gdi::HFONT,
        scale: i32,
        line_height: i32,
    }

    /// 提示窗句柄: 命令通道 + 线程 id + 线程句柄。
    pub struct HintWindow {
        tx: Sender<Command>,
        thread_id: u32,
        join: Option<std::thread::JoinHandle<()>>,
    }

    impl HintWindow {
        /// 创建提示窗线程与隐藏窗口; 失败返回错误供调用方失败关闭。
        pub fn new() -> Result<Self, HintError> {
            let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
            let (ready_tx, ready_rx) = std::sync::mpsc::channel();
            let join = std::thread::Builder::new()
                .name("danqing-hint".into())
                .spawn(move || hint_thread(cmd_rx, ready_tx))
                .map_err(|_| HintError::ThreadStart)?;
            match ready_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(Ok(thread_id)) => Ok(Self {
                    tx: cmd_tx,
                    thread_id,
                    join: Some(join),
                }),
                Ok(Err(error)) => {
                    drop(cmd_tx);
                    let _ = join.join();
                    Err(error)
                }
                Err(_) => {
                    drop(cmd_tx);
                    Err(HintError::ThreadStart)
                }
            }
        }

        /// 显示/更新提示文本, 并把 TTL 重置为 `ttl`。
        pub fn show(&self, text: &str, ttl: Duration) {
            if self
                .tx
                .send(Command::Show {
                    text: text.to_string(),
                    ttl,
                })
                .is_ok()
            {
                self.notify();
            }
        }

        /// 立即隐藏; 幂等。
        pub fn hide(&self) {
            if self.tx.send(Command::Hide).is_ok() {
                self.notify();
            }
        }

        fn notify(&self) {
            // SAFETY: 纯线程消息投递; 线程已就绪 (消息队列在 new 返回前建立)。
            unsafe {
                PostThreadMessageW(self.thread_id, WM_HINT_COMMAND, 0, 0);
            }
        }
    }

    impl Drop for HintWindow {
        fn drop(&mut self) {
            let _ = self.tx.send(Command::Shutdown);
            self.notify();
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    /// 提示窗线程: 注册类、建窗、建字体, 然后泵消息直到 Shutdown。
    fn hint_thread(cmd_rx: Receiver<Command>, ready: Sender<Result<u32, HintError>>) {
        // SAFETY: 线程独占窗口与 GDI 资源; 所有返回值都在使用前判空。
        unsafe {
            let hmod = GetModuleHandleW(std::ptr::null());
            let class_name = windows_sys::core::w!("DanqingHintWindow");
            let class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(hint_wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hmod,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name,
            };
            let atom = RegisterClassW(&class);
            if atom == 0 {
                let last = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                if last != ERROR_CLASS_ALREADY_EXISTS as i32 {
                    let _ = ready.send(Err(HintError::WindowCreate));
                    return;
                }
            }
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED
                    | WS_EX_TRANSPARENT
                    | WS_EX_TOPMOST
                    | WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE,
                class_name,
                windows_sys::core::w!("danqing hint"),
                WS_POPUP,
                0,
                0,
                10,
                10,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hmod,
                std::ptr::null(),
            );
            if hwnd.is_null() {
                let _ = ready.send(Err(HintError::WindowCreate));
                return;
            }

            let screen = GetDC(std::ptr::null_mut());
            let dpi = if screen.is_null() {
                96
            } else {
                GetDeviceCaps(screen, LOGPIXELSY as i32).max(96)
            };
            if !screen.is_null() {
                ReleaseDC(std::ptr::null_mut(), screen);
            }
            let scale = scale_for_dpi(dpi);
            let font = CreateFontW(
                -(13 * scale),
                0,
                0,
                0,
                FW_NORMAL as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY as u32,
                DEFAULT_PITCH as u32 | FF_DONTCARE as u32,
                windows_sys::core::w!("Microsoft YaHei UI"),
            );
            if font.is_null() {
                DestroyWindow(hwnd);
                let _ = ready.send(Err(HintError::WindowCreate));
                return;
            }

            let state = Box::new(PaintState {
                lines: vec![String::new()],
                font,
                scale,
                line_height: 0,
            });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

            // 强制建立本线程消息队列, 否则极早的 PostThreadMessage 会失败。
            let mut msg: MSG = std::mem::zeroed();
            PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_NOREMOVE);
            let _ = ready.send(Ok(GetCurrentThreadId()));

            let mut shutdown = false;
            while !shutdown {
                let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
                if ret <= 0 {
                    break;
                }
                if msg.message == WM_HINT_COMMAND {
                    while let Ok(command) = cmd_rx.try_recv() {
                        match command {
                            Command::Show { text, ttl } => {
                                let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PaintState;
                                if !ptr.is_null() {
                                    apply_show(hwnd, &mut *ptr, &text, ttl);
                                }
                            }
                            Command::Hide => hide(hwnd),
                            Command::Shutdown => {
                                shutdown = true;
                                break;
                            }
                        }
                    }
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut PaintState;
            if !ptr.is_null() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                let state = Box::from_raw(ptr);
                DeleteObject(state.font);
            }
            DestroyWindow(hwnd);
        }
    }

    unsafe extern "system" fn hint_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // SAFETY: 只在提示窗线程回调; 状态指针由本线程写入并在销毁前清空。
        unsafe {
            match msg {
                WM_PAINT => {
                    let mut ps: PAINTSTRUCT = std::mem::zeroed();
                    let hdc = BeginPaint(hwnd, &mut ps);
                    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const PaintState;
                    if !ptr.is_null() && !hdc.is_null() {
                        paint(hdc, hwnd, &*ptr);
                    }
                    EndPaint(hwnd, &ps);
                    0
                }
                WM_TIMER => {
                    if wparam == TIMER_ID {
                        KillTimer(hwnd, TIMER_ID);
                        ShowWindow(hwnd, SW_HIDE);
                    }
                    0
                }
                WM_DESTROY => {
                    windows_sys::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                    0
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    /// 计算行高与窗口尺寸; 选中字体后逐行 `DrawTextW(DT_CALCRECT)`。
    unsafe fn measure(
        hdc: windows_sys::Win32::Graphics::Gdi::HDC,
        state: &mut PaintState,
    ) -> (i32, i32) {
        unsafe {
            let pad = 10 * state.scale;
            let gap = 2 * state.scale;
            let old = SelectObject(hdc, state.font);
            let mut max_width = 0;
            let mut line_height = 0;
            for line in &state.lines {
                let wide: Vec<u16> = line.encode_utf16().collect();
                let mut rect = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    wide.len() as i32,
                    &mut rect,
                    DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX,
                );
                max_width = max_width.max(rect.right - rect.left);
                line_height = line_height.max(rect.bottom - rect.top);
            }
            SelectObject(hdc, old);
            state.line_height = line_height.max(1);
            let count = state.lines.len() as i32;
            let width = max_width + pad * 2;
            let height = state.line_height * count + gap * (count - 1) + pad * 2;
            (width.max(1), height.max(1))
        }
    }

    /// 显示或更新提示: 量尺寸、定位、圆角、alpha、无激活显示、重置 TTL。
    unsafe fn apply_show(hwnd: HWND, state: &mut PaintState, text: &str, ttl: Duration) {
        unsafe {
            state.lines = split_hint_lines(text);
            let hdc = GetDC(hwnd);
            let (width, height) = if hdc.is_null() {
                (200, 40)
            } else {
                let size = measure(hdc, state);
                ReleaseDC(hwnd, hdc);
                size
            };
            let anchor = anchor_point();
            let offset = (6 * state.scale, 10 * state.scale);
            let (x, y) = match work_area_at(anchor) {
                Some(work) => position_with_work(anchor, offset, (width, height), work),
                None => (anchor.0 + offset.0, anchor.1 + offset.1),
            };
            SetWindowPos(hwnd, HWND_TOPMOST, x, y, width, height, SWP_NOACTIVATE);
            let radius = 8 * state.scale;
            let region = CreateRoundRectRgn(0, 0, width + 1, height + 1, radius, radius);
            if !region.is_null() {
                SetWindowRgn(hwnd, region, 1);
            }
            SetLayeredWindowAttributes(hwnd, 0, HINT_ALPHA, LWA_ALPHA);
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            InvalidateRect(hwnd, std::ptr::null(), 0);
            UpdateWindow(hwnd);
            SetTimer(hwnd, TIMER_ID, ttl.as_millis().max(100) as u32, None);
        }
    }

    /// 隐藏提示并停表。
    unsafe fn hide(hwnd: HWND) {
        unsafe {
            KillTimer(hwnd, TIMER_ID);
            ShowWindow(hwnd, SW_HIDE);
        }
    }

    /// 绘制: 实底 + 两行左对齐白字。
    unsafe fn paint(hdc: windows_sys::Win32::Graphics::Gdi::HDC, hwnd: HWND, state: &PaintState) {
        unsafe {
            let mut client: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client);
            let brush = CreateSolidBrush(rgb(28, 28, 30));
            if !brush.is_null() {
                FillRect(hdc, &client, brush);
                DeleteObject(brush);
            }
            SetBkMode(hdc, TRANSPARENT as i32);
            SetTextColor(hdc, rgb(236, 236, 238));
            let old = SelectObject(hdc, state.font);
            let pad = 10 * state.scale;
            let gap = 2 * state.scale;
            for (index, line) in state.lines.iter().enumerate() {
                let wide: Vec<u16> = line.encode_utf16().collect();
                let mut rect = RECT {
                    left: pad,
                    top: pad + index as i32 * (state.line_height + gap),
                    right: client.right - pad,
                    bottom: client.bottom - pad,
                };
                DrawTextW(
                    hdc,
                    wide.as_ptr(),
                    wide.len() as i32,
                    &mut rect,
                    DT_LEFT | DT_TOP | DT_SINGLELINE | DT_NOPREFIX,
                );
            }
            SelectObject(hdc, old);
        }
    }

    /// 锚点: 前台窗口插入符右下角; 取不到回退鼠标光标; 都失败用 (0,0)。
    unsafe fn anchor_point() -> (i32, i32) {
        unsafe {
            let foreground = GetForegroundWindow();
            if !foreground.is_null() {
                let mut pid = 0u32;
                let thread_id = GetWindowThreadProcessId(foreground, &mut pid);
                if thread_id != 0 {
                    let mut info: GUITHREADINFO = std::mem::zeroed();
                    info.cbSize = size_of::<GUITHREADINFO>() as u32;
                    if GetGUIThreadInfo(thread_id, &mut info) != 0
                        && !info.hwndCaret.is_null()
                        && info.rcCaret.right >= info.rcCaret.left
                        && info.rcCaret.bottom >= info.rcCaret.top
                    {
                        return (info.rcCaret.right, info.rcCaret.bottom);
                    }
                }
            }
            let mut cursor = POINT { x: 0, y: 0 };
            if GetCursorPos(&mut cursor) != 0 {
                (cursor.x, cursor.y)
            } else {
                (0, 0)
            }
        }
    }

    /// 锚点所在 (或最近) 显示器的工作区; 查询失败返回 None。
    unsafe fn work_area_at(anchor: (i32, i32)) -> Option<(i32, i32, i32, i32)> {
        unsafe {
            let monitor = MonitorFromPoint(
                POINT {
                    x: anchor.0,
                    y: anchor.1,
                },
                MONITOR_DEFAULTTONEAREST,
            );
            let mut info: MONITORINFO = std::mem::zeroed();
            info.cbSize = size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut info) == 0 {
                return None;
            }
            Some((
                info.rcWork.left,
                info.rcWork.top,
                info.rcWork.right,
                info.rcWork.bottom,
            ))
        }
    }
}

#[cfg(target_os = "windows")]
pub use win::HintWindow;

/// 非 Windows 平台: 无提示窗实现, 创建即返回错误 (产品按失败关闭处理)。
#[cfg(not(target_os = "windows"))]
pub struct HintWindow;

#[cfg(not(target_os = "windows"))]
use std::time::Duration;

#[cfg(not(target_os = "windows"))]
impl HintWindow {
    /// 非 Windows 不支持提示窗。
    pub fn new() -> Result<Self, HintError> {
        Err(HintError::Unsupported)
    }

    /// no-op。
    pub fn show(&self, _text: &str, _ttl: Duration) {}

    /// no-op。
    pub fn hide(&self) {}
}

#[cfg(test)]
mod tests {
    use super::{HintError, position_with_work, scale_for_dpi, split_hint_lines};

    #[test]
    fn split_lines_keeps_order_and_strips_cr() {
        assert_eq!(
            split_hint_lines("已转换：Markdown 链接\r\nEsc 原始 · Tab 换形"),
            vec!["已转换：Markdown 链接", "Esc 原始 · Tab 换形"]
        );
    }

    #[test]
    fn split_lines_caps_at_four_and_keeps_single_empty() {
        assert_eq!(split_hint_lines("").len(), 1);
        assert_eq!(split_hint_lines("1\n2\n3\n4\n5").len(), 4);
        assert_eq!(split_hint_lines("1\n2\n3\n4\n5")[3], "4");
    }

    #[test]
    fn scale_rounds_to_nearest_and_floors_at_one() {
        assert_eq!(scale_for_dpi(96), 1);
        assert_eq!(scale_for_dpi(120), 1); // 1.25 → 1 (四舍五入到 1)
        assert_eq!(scale_for_dpi(144), 2);
        assert_eq!(scale_for_dpi(48), 1);
    }

    #[test]
    fn position_applies_offset_then_clamps_into_work_area() {
        assert_eq!(
            position_with_work((500, 400), (6, 10), (200, 40), (0, 0, 1920, 1040)),
            (506, 410)
        );
        // 贴右下缘: 钳回工作区, 不越过任务栏。
        assert_eq!(
            position_with_work((1900, 1030), (6, 10), (200, 40), (0, 0, 1920, 1040)),
            (1720, 1000)
        );
        // 左侧副屏负坐标。
        assert_eq!(
            position_with_work((-1900, 100), (6, 10), (200, 40), (-1920, 0, 0, 1080)),
            (-1894, 110)
        );
    }

    #[test]
    fn hint_window_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<super::HintWindow>();
    }

    #[test]
    fn error_display_has_no_content() {
        assert_eq!(HintError::WindowCreate.to_string(), "提示窗创建失败");
        assert_eq!(HintError::Unsupported.to_string(), "提示窗仅支持 Windows");
    }
}
