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
    text.split('\n')
        .take(4)
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_string())
        .collect()
}

/// 客户区插入符矩形 → 屏幕锚点 (右下角)。
///
/// `GUITHREADINFO.rcCaret` 是相对 `hwndCaret` 的**客户区**坐标, 必须先换算到屏幕;
/// 换算函数由调用方注入, 便于纯逻辑测试锁定换算方向。
fn caret_anchor_point(
    rect: (i32, i32, i32, i32),
    to_screen: impl FnOnce((i32, i32)) -> Option<(i32, i32)>,
) -> Option<(i32, i32)> {
    let (left, top, right, bottom) = rect;
    if right < left || bottom < top {
        return None;
    }
    to_screen((right, bottom))
}

/// 96 DPI 基准缩放因子, 最小 1 倍。
fn scale_for_dpi(dpi: i32) -> i32 {
    ((dpi as f32 / 96.0).round() as i32).max(1)
}

/// 锚点 + 偏移 → 提示窗左上角, 钳进工作区。
///
/// 默认放在锚点右下方; 钳制后若窗口会压住锚点, 改放锚点上方。
/// 上下都放不下 (工作区太小或锚点贴满) 时保留先钳制结果并接受遮挡。
fn position_with_work(
    anchor: (i32, i32),
    offset: (i32, i32),
    size: (i32, i32),
    work: (i32, i32, i32, i32),
) -> (i32, i32) {
    let below = super::placement::clamp_into_work_area(
        (anchor.0 + offset.0, anchor.1 + offset.1),
        size,
        work,
    );
    if !covers_anchor(below, size, anchor) {
        return below;
    }
    let above = super::placement::clamp_into_work_area(
        (anchor.0 + offset.0, anchor.1 - offset.1 - size.1),
        size,
        work,
    );
    if covers_anchor(above, size, anchor) {
        below
    } else {
        above
    }
}

/// 窗口矩形是否覆盖锚点 (右/下边界为开区间)。
fn covers_anchor(pos: (i32, i32), size: (i32, i32), anchor: (i32, i32)) -> bool {
    pos.0 <= anchor.0 && anchor.0 < pos.0 + size.0 && pos.1 <= anchor.1 && anchor.1 < pos.1 + size.1
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
        BeginPaint, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, ClientToScreen, CreateFontW,
        CreateRoundRectRgn, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_PITCH, DT_CALCRECT, DT_LEFT,
        DT_NOPREFIX, DT_SINGLELINE, DT_TOP, DeleteObject, DrawTextW, EndPaint, FF_DONTCARE,
        FW_NORMAL, FillRect, GetDC, GetDeviceCaps, GetMonitorInfoW, InvalidateRect, LOGPIXELSY,
        MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
        ReleaseDC, SelectObject, SetBkMode, SetTextColor, SetWindowRgn, TRANSPARENT, UpdateWindow,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GUI_CARETBLINKING,
        GUITHREADINFO, GWLP_USERDATA, GetClientRect, GetCursorPos, GetForegroundWindow,
        GetGUIThreadInfo, GetMessageW, GetWindowLongPtrW, GetWindowThreadProcessId, HWND_TOPMOST,
        KillTimer, LWA_ALPHA, MSG, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, RegisterClassW,
        SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SetLayeredWindowAttributes, SetTimer,
        SetWindowLongPtrW, SetWindowPos, ShowWindow, TranslateMessage, WM_APP, WM_DESTROY,
        WM_PAINT, WM_QUIT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };

    use super::{
        HintError, caret_anchor_point, position_with_work, scale_for_dpi, split_hint_lines,
    };

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

    /// 初始化握手: 线程先报自身 id, 再报初始化结果。
    ///
    /// 先报 id 是为了让父线程在初始化超时时能投递 `WM_QUIT` 回收线程,
    /// 而不是丢掉线程句柄留下永驻线程与窗口。
    enum Handshake {
        Started(u32),
        Initialized,
        Failed(HintError),
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
            // 第一阶段: 线程 id 报到。失败即线程已自行退出, 只需 join。
            let thread_id = match ready_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(Handshake::Started(thread_id)) => thread_id,
                Ok(_) => {
                    let _ = join.join();
                    return Err(HintError::ThreadStart);
                }
                Err(_) => {
                    // 连报到都超时: 丢弃命令端, 线程首个发送失败即退出。
                    drop(cmd_tx);
                    return Err(HintError::ThreadStart);
                }
            };
            // 第二阶段: 初始化结果。超时则投递 WM_QUIT 让线程退出并回收。
            match ready_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(Handshake::Initialized) => Ok(Self {
                    tx: cmd_tx,
                    thread_id,
                    join: Some(join),
                }),
                Ok(Handshake::Failed(error)) => {
                    drop(cmd_tx);
                    let _ = join.join();
                    Err(error)
                }
                Ok(Handshake::Started(_)) | Err(_) => {
                    drop(cmd_tx);
                    // SAFETY: 线程 id 已报到, 消息队列已建立; 投递 WM_QUIT 结束其循环。
                    unsafe {
                        PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
                    }
                    let _ = join.join();
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
            if self.tx.send(Command::Shutdown).is_ok() {
                self.notify();
            }
            if let Some(join) = self.join.take() {
                let _ = join.join();
            }
        }
    }

    /// 提示窗线程: 报文线程 id、注册类、建窗、建字体, 然后泵消息直到 Shutdown。
    fn hint_thread(cmd_rx: Receiver<Command>, ready: Sender<Handshake>) {
        // 先报到线程 id; 父线程已放弃等待时直接退出, 不留永驻线程。
        // SAFETY: 纯查询当前线程 id, 无内存安全风险。
        let thread_id = unsafe { GetCurrentThreadId() };
        if ready.send(Handshake::Started(thread_id)).is_err() {
            return;
        }
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
                    let _ = ready.send(Handshake::Failed(HintError::WindowCreate));
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
                let _ = ready.send(Handshake::Failed(HintError::WindowCreate));
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
                let _ = ready.send(Handshake::Failed(HintError::WindowCreate));
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
            let _ = ready.send(Handshake::Initialized);

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
            if !region.is_null() && SetWindowRgn(hwnd, region, 1) == 0 {
                // 设置失败时区域所有权未转移, 必须自行释放, 否则每次 show 漏一个 GDI 区域。
                DeleteObject(region);
            }
            SetLayeredWindowAttributes(hwnd, 0, HINT_ALPHA, LWA_ALPHA);
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            InvalidateRect(hwnd, std::ptr::null(), 0);
            UpdateWindow(hwnd);
            let ttl_ms = ttl.as_millis().min(u32::MAX as u128) as u32;
            if SetTimer(hwnd, TIMER_ID, ttl_ms.max(100), None) == 0 {
                log::warn!("提示窗 TTL 定时器创建失败, 本次提示不会自动隐藏");
            }
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

    /// 锚点: 前台窗口插入符右下角 (客户区坐标已换算到屏幕);
    /// 取不到回退鼠标光标; 都失败用 (0,0)。
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
                        && info.flags & GUI_CARETBLINKING != 0
                        && !info.hwndCaret.is_null()
                    {
                        let rect = (
                            info.rcCaret.left,
                            info.rcCaret.top,
                            info.rcCaret.right,
                            info.rcCaret.bottom,
                        );
                        let hwnd_caret = info.hwndCaret;
                        // rcCaret 是相对 hwndCaret 的客户区坐标, 必须换算到屏幕。
                        if let Some(anchor) = caret_anchor_point(rect, |point| {
                            let mut pt = POINT {
                                x: point.0,
                                y: point.1,
                            };
                            // SAFETY: 只读坐标换算, 句柄来自刚查询到的插入符窗口。
                            if ClientToScreen(hwnd_caret, &mut pt) != 0 {
                                Some((pt.x, pt.y))
                            } else {
                                None
                            }
                        }) {
                            return anchor;
                        }
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
    use super::{
        HintError, caret_anchor_point, position_with_work, scale_for_dpi, split_hint_lines,
    };

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
        // 贴右下缘: 先钳回工作区, 但下方放置会压住锚点 → 翻到锚点上方。
        assert_eq!(
            position_with_work((1900, 1030), (6, 10), (200, 40), (0, 0, 1920, 1040)),
            (1720, 980)
        );
        // 左侧副屏负坐标。
        assert_eq!(
            position_with_work((-1900, 100), (6, 10), (200, 40), (-1920, 0, 0, 1080)),
            (-1894, 110)
        );
        // 上下都放不下: 接受遮挡, 保留钳制结果。
        assert_eq!(
            position_with_work((100, 20), (6, 10), (200, 40), (0, 0, 200, 40)),
            (0, 0)
        );
    }

    #[test]
    fn caret_anchor_maps_client_corner_through_screen_conversion() {
        let seen = std::cell::Cell::new((0, 0));
        let anchor = caret_anchor_point((10, 20, 40, 60), |point| {
            seen.set(point);
            Some((point.0 + 300, point.1 + 200))
        });
        assert_eq!(seen.get(), (40, 60), "必须换算插入符右下角, 而非左上角");
        assert_eq!(anchor, Some((340, 260)));
        assert_eq!(
            caret_anchor_point((40, 60, 10, 20), |_| Some((0, 0))),
            None,
            "反向矩形视为无锚点"
        );
        assert_eq!(
            caret_anchor_point((0, 0, 1, 1), |_| None),
            None,
            "换算失败回退光标路径"
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
