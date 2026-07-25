//! @author 十四叔
//! @date 2026/07/17

//! 窗口与事件循环封装 (winit 平台适配层)。
//!
//! 本模块是唯一允许接触 OS 窗口 API 的地方：负责窗口创建、
//! 事件循环驱动，并把 winit 事件转换为平台无关的内部事件。

use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Ime as WinitIme, MouseButton as WinitMouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey as WinitNamedKey},
    window::{Icon, Window as WinitWindow, WindowAttributes, WindowId},
};

use crate::app::{AnimationCtx, App};
use crate::event::{Event, ImeEvent, Key, MouseButton, NamedKey, WindowAction};
use crate::render::{BackgroundConfig, Context, RectBatch, TextBatch};
use crate::widget::{
    FocusManager, MsgQueue, Node, event_at_path, ime_area_at_path, selected_text_at_path,
    wants_ime_at_path,
};
use crate::{Color, Point, Rect, Size};

/// 窗口 / 事件循环相关错误。
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    /// 事件循环创建或运行失败。
    #[error("事件循环错误：{0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    /// 窗口创建失败。
    #[error("创建窗口失败：{0}")]
    Os(#[from] winit::error::OsError),
}

fn error_chain_messages(label: &str, error: &(dyn std::error::Error + 'static)) -> Vec<String> {
    let mut messages = vec![format!("{label}：{error}")];
    let mut source = error.source();
    let mut depth = 1;
    while let Some(error) = source {
        messages.push(format!("  原因 {depth}：{error}"));
        source = error.source();
        depth += 1;
    }
    messages
}

fn log_error_chain(label: &str, error: &(dyn std::error::Error + 'static)) {
    for message in error_chain_messages(label, error) {
        log::error!("{message}");
    }
}

/// 窗口初始配置。
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// 窗口标题。
    pub title: String,
    /// 初始逻辑尺寸。
    pub size: Size,
    /// 清屏颜色。
    pub clear_color: Color,
    /// 背景图配置。
    pub background: BackgroundConfig,
    /// 窗口边框颜色 (无边框窗口时自绘)。
    pub border_color: Color,
    /// 窗口边框圆角半径 (配合自绘边框与 DWM 圆角)。
    pub border_radius: f32,
    /// 窗口边框粗细。
    pub border_thickness: f32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "danqing showcase".into(),
            size: Size::new(1280.0, 800.0),
            // 深蓝灰：非常量黑 / 白，用于验证颜色参数通路
            clear_color: Color::rgb(0.10, 0.16, 0.24),
            background: BackgroundConfig::default(),
            // 浅灰边框，与浅色毛玻璃主题协调
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.12),
            border_radius: 12.0,
            border_thickness: 1.0,
        }
    }
}

/// 把 winit 窗口事件转换为内部事件; 无关事件返回 None。
fn convert_event(event: &WindowEvent, cursor: Point, modifiers: ModifiersState) -> Option<Event> {
    match event {
        WindowEvent::CursorMoved { position, .. } => Some(Event::CursorMoved(Point::new(
            position.x as f32,
            position.y as f32,
        ))),
        WindowEvent::CursorLeft { .. } => Some(Event::CursorLeft),
        WindowEvent::MouseInput { state, button, .. } => {
            let button = match button {
                WinitMouseButton::Left => MouseButton::Left,
                WinitMouseButton::Right => MouseButton::Right,
                WinitMouseButton::Middle => MouseButton::Middle,
                WinitMouseButton::Back => MouseButton::Back,
                WinitMouseButton::Forward => MouseButton::Forward,
                WinitMouseButton::Other(v) => MouseButton::Other(*v),
            };
            Some(Event::MouseInput {
                button,
                pressed: *state == ElementState::Pressed,
                position: cursor,
            })
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let d = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
            };
            Some(Event::MouseWheel {
                delta: d,
                position: cursor,
            })
        }
        WindowEvent::KeyboardInput { event, .. } => {
            let key = match &event.logical_key {
                WinitKey::Character(s) => Key::Character(s.to_string()),
                WinitKey::Named(named) => {
                    let named = match named {
                        WinitNamedKey::ArrowUp => NamedKey::ArrowUp,
                        WinitNamedKey::ArrowDown => NamedKey::ArrowDown,
                        WinitNamedKey::ArrowLeft => NamedKey::ArrowLeft,
                        WinitNamedKey::ArrowRight => NamedKey::ArrowRight,
                        WinitNamedKey::Space => NamedKey::Space,
                        WinitNamedKey::Enter => NamedKey::Enter,
                        WinitNamedKey::Escape => NamedKey::Escape,
                        WinitNamedKey::Tab => NamedKey::Tab,
                        WinitNamedKey::Backspace => NamedKey::Backspace,
                        WinitNamedKey::Delete => NamedKey::Delete,
                        WinitNamedKey::Home => NamedKey::Home,
                        WinitNamedKey::End => NamedKey::End,
                        WinitNamedKey::Shift => NamedKey::Shift,
                        WinitNamedKey::Control => NamedKey::Control,
                        WinitNamedKey::Alt => NamedKey::Alt,
                        _ => return None,
                    };
                    Key::Named(named)
                }
                _ => return None,
            };
            Some(Event::Key {
                key,
                pressed: event.state == ElementState::Pressed,
                shift: modifiers.shift_key(),
                ctrl: modifiers.control_key(),
            })
        }
        WindowEvent::Ime(ime) => match ime {
            WinitIme::Enabled => Some(Event::Ime(ImeEvent::Enabled)),
            WinitIme::Disabled => Some(Event::Ime(ImeEvent::Disabled)),
            WinitIme::Preedit(value, cursor) => Some(Event::Ime(ImeEvent::Preedit {
                value: value.clone(),
                cursor: *cursor,
            })),
            WinitIme::Commit(value) => Some(Event::Ime(ImeEvent::Commit {
                value: value.clone(),
            })),
        },
        _ => None,
    }
}

/// 从 PNG 文件加载 winit 图标。
///
/// 将 PNG 解码为 RGBA 后，通过 [`Icon::from_rgba`] 创建图标。
/// 返回 `Err` 时调用方可选择回退到默认图标。
fn load_icon_from_png(path: &std::path::Path) -> Result<Icon, Box<dyn std::error::Error>> {
    let img = image::open(path)?.into_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).map_err(Into::into)
}

/// Windows 下为无边框窗口恢复圆角与阴影。
///
/// 使用 winit 公开的平台扩展 API, 避免手写 unsafe DWM 调用。
/// 若设置失败仅记录警告，不影响窗口功能。
#[cfg(target_os = "windows")]
fn apply_windows_undecorated_style(window: &WinitWindow) {
    use winit::platform::windows::{CornerPreference, WindowExtWindows};

    if let Err(err) = std::panic::catch_unwind(|| {
        window.set_undecorated_shadow(true);
        window.set_corner_preference(CornerPreference::Round);
    }) {
        log::warn!("设置 Windows 无边框窗口样式失败：{err:?}");
    }
}

/// 加载应用窗口图标。
///
/// 尝试读取 `assets/logo/logo_256.png`;
/// 失败时记录警告并返回 `None`, 避免窗口创建因图标问题而 panic。
fn load_window_icon() -> Option<Icon> {
    let path = std::path::Path::new("assets")
        .join("logo")
        .join("logo_256.png");
    match load_icon_from_png(&path) {
        Ok(icon) => Some(icon),
        Err(err) => {
            log::warn!("加载窗口图标失败：{err}");
            None
        }
    }
}

/// 应用主动发给窗口的事件 (用于全局热键配套: 显隐 / 退出等)。
#[derive(Debug, Clone, Copy)]
pub enum WindowAppEvent {
    /// 切换窗口可见性 (Handler 翻转内部状态后应用到 winit)。
    /// 单一事实源在 Handler, App 不持有副本以避免失同步。
    ToggleVisible,
    /// 退出应用 (事件循环收到后 `event_loop.exit()`)。
    Quit,
    /// 阶段流转通知: 隐藏态时 Handler 自动呼出窗口 + 抢焦点。
    PhaseAdvanced,
}

/// 应用持有的窗口事件发送器 (轻量 clone, 内部是 mpsc Sender)。
#[derive(Clone)]
pub struct WindowEventSender {
    sender: Sender<WindowAppEvent>,
}

impl WindowEventSender {
    /// 请求 Handler 翻转窗口可见性。
    pub fn toggle_visible(&self) {
        let _ = self.sender.send(WindowAppEvent::ToggleVisible);
    }

    /// 退出应用。
    pub fn quit(&self) {
        let _ = self.sender.send(WindowAppEvent::Quit);
    }

    /// 通知 Handler 阶段已流转 (隐藏态时 Handler 决定是否自动呼出)。
    pub fn phase_advanced(&self) {
        let _ = self.sender.send(WindowAppEvent::PhaseAdvanced);
    }
}

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
mod hotkeys {
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
mod hotkeys {
    use std::sync::mpsc::Receiver;
    use std::thread::JoinHandle;

    /// 非 Windows 平台: 全局热键 unavailable, 返回 None。
    pub fn spawn() -> Option<(Receiver<u8>, JoinHandle<()>)> {
        log::info!("global hotkeys unsupported on this platform");
        None
    }
}

/// winit 应用处理器，驱动窗口生命周期与事件分发。
struct Handler<'a, A: App> {
    config: WindowConfig,
    window: Option<Arc<WinitWindow>>,
    context: Option<Context>,
    /// 文本收集器 (持久持有字体与图集)。
    texts: TextBatch,
    /// 当前光标位置 (鼠标输入事件的位置来源)。
    cursor: Point,
    /// 当前修饰键状态。
    modifiers: ModifiersState,
    /// 应用本体 (状态容器)。
    app: &'a mut A,
    /// 组件树 (启动时由 App::view 构建一次)。
    tree: Node,
    /// 组件产出的消息队列。
    msgs: MsgQueue,
    /// 根矩形 (事件命中用，每帧布局后更新)。
    root_area: Rect,
    /// 焦点管理器。
    focus: FocusManager,
    /// 应用启动时间 (用于动画)。
    start: Instant,
    /// 系统剪贴板 (懒加载)。
    clipboard: Option<arboard::Clipboard>,
    /// 是否已完成首帧渲染 (用于一次性诊断计时)。
    first_frame_done: bool,
    /// 进程入口时间 (run_app 起点, 用于启动总耗时基准)。
    boot: Instant,
    /// 全局热键接收器 (来自热键线程, `None` 表示未启用或平台不支持)。
    hotkey_rx: Option<Receiver<u8>>,
    /// 窗口事件接收器 (App 主动发出: 显隐 / 退出)。
    window_event_rx: Receiver<WindowAppEvent>,
    /// 当前窗口可见性 (热键 ToggleVisible 状态记录, 与 Handler 同步)。
    is_visible: bool,
}

impl<A: App> Handler<'_, A> {
    /// 记录一条窗口事件到日志。
    fn log_event(event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                log::debug!("鼠标移动：({:.0}, {:.0})", position.x, position.y);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                log::info!("鼠标按键：{button:?} {state:?}");
            }
            WindowEvent::MouseWheel { delta, .. } => {
                log::info!("滚轮：{delta:?}");
            }
            WindowEvent::KeyboardInput { event, .. } => {
                log::info!("键盘：{:?} {:?}", event.logical_key, event.state);
            }
            WindowEvent::Ime(ime) => {
                log::info!("IME: {ime:?}");
            }
            WindowEvent::ModifiersChanged(mods) => {
                log::debug!("修饰键：{mods:?}");
            }
            _ => {}
        }
    }

    /// 发送焦点进 / 出事件。
    fn dispatch_focus_changes(&mut self, previous: Option<&[usize]>, current: Option<&[usize]>) {
        if let Some(path) = previous {
            if current.map(|c| c != path).unwrap_or(true) {
                event_at_path(
                    &mut self.tree,
                    path,
                    &Event::FocusOut,
                    self.root_area,
                    &mut self.msgs,
                );
            }
        }
        if let Some(path) = current {
            if previous.map(|p| p != path).unwrap_or(true) {
                event_at_path(
                    &mut self.tree,
                    path,
                    &Event::FocusIn,
                    self.root_area,
                    &mut self.msgs,
                );
            }
        }
    }

    /// 将键盘 /IME/ 剪贴板事件路由到当前焦点组件。
    fn dispatch_focused_event(&mut self, event: &Event) {
        // Tab 遍历与当前焦点状态无关，必须最先处理：
        // 清焦 (点击空白/Escape) 后键盘仍能借此重回焦点链。
        if let Event::Key {
            key: Key::Named(NamedKey::Tab),
            pressed: true,
            ..
        } = event
        {
            if self.modifiers.shift_key() {
                self.focus.prev();
            } else {
                self.focus.next();
            }
            return;
        }

        let Some(path) = self.focus.current().map(|p| p.to_vec()) else {
            // 无焦点时回退到应用层
            self.app.event(event);
            return;
        };

        match event {
            Event::Key { key, pressed, .. } if *pressed => {
                match key {
                    Key::Named(NamedKey::Escape) => {
                        // 焦点组件未消费 Escape 时清除焦点，
                        // 键盘事件随后回退到应用层。
                        let consumed = event_at_path(
                            &mut self.tree,
                            &path,
                            event,
                            self.root_area,
                            &mut self.msgs,
                        ) == crate::widget::EventResult::Consumed;
                        if !consumed {
                            self.focus.clear_focus();
                            self.dispatch_focus_changes(Some(&path), None);
                            self.focus.acknowledge();
                        }
                        return;
                    }
                    Key::Character(c)
                        if self.modifiers.control_key()
                            && matches!(c.as_str(), "c" | "x" | "v") =>
                    {
                        self.handle_clipboard(c.as_str());
                        return;
                    }
                    _ => {}
                }
                event_at_path(&mut self.tree, &path, event, self.root_area, &mut self.msgs);
            }
            Event::Ime(_) => {
                event_at_path(&mut self.tree, &path, event, self.root_area, &mut self.msgs);
            }
            _ => {
                self.app.event(event);
            }
        }
    }

    /// 处理 Ctrl+C/X/V 剪贴板快捷键。
    fn handle_clipboard(&mut self, key: &str) {
        let Some(path) = self.focus.current().map(|p| p.to_vec()) else {
            return;
        };

        match key {
            "c" | "x" => {
                let cut = key == "x";
                // 剪切前先快照选中文本
                let snapshot = if cut {
                    selected_text_at_path(&self.tree, &path)
                } else {
                    None
                };
                let consumed = event_at_path(
                    &mut self.tree,
                    &path,
                    &if cut { Event::Cut } else { Event::Copy },
                    self.root_area,
                    &mut self.msgs,
                ) == crate::widget::EventResult::Consumed;
                if consumed {
                    let text = if cut {
                        snapshot
                    } else {
                        selected_text_at_path(&self.tree, &path)
                    };
                    if let Some(text) = text {
                        self.set_clipboard(text);
                    }
                }
            }
            "v" => {
                let consumed = event_at_path(
                    &mut self.tree,
                    &path,
                    &Event::Paste,
                    self.root_area,
                    &mut self.msgs,
                ) == crate::widget::EventResult::Consumed;
                if consumed {
                    if let Some(text) = self.get_clipboard() {
                        event_at_path(
                            &mut self.tree,
                            &path,
                            &Event::Ime(ImeEvent::Commit { value: text }),
                            self.root_area,
                            &mut self.msgs,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn clipboard(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.clipboard.is_none() {
            match arboard::Clipboard::new() {
                Ok(cb) => self.clipboard = Some(cb),
                Err(err) => log::warn!("初始化剪贴板失败：{err}"),
            }
        }
        self.clipboard.as_mut()
    }

    fn set_clipboard(&mut self, text: String) {
        if let Some(cb) = self.clipboard() {
            if let Err(err) = cb.set_text(text) {
                log::warn!("写入剪贴板失败：{err}");
            }
        }
    }

    fn get_clipboard(&mut self) -> Option<String> {
        self.clipboard().map(|cb| cb.get_text().unwrap_or_default())
    }

    /// 处理自绘标题栏等组件产出的窗口控制动作。
    fn handle_window_action(&mut self, action: WindowAction, _event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        match action {
            WindowAction::Close => {
                // 标题栏关闭按钮 = 隐藏 (与 Alt+F4 一致); 进程由 Ctrl+Shift+Q 退出
                log::info!("标题栏关闭窗口 → 隐藏");
                // set_visible(false) 由 self.hide_window 负责; 这里只改 self.is_visible
                if let Some(window) = self.window.as_ref() {
                    window.set_visible(false);
                }
                self.is_visible = false;
            }
            WindowAction::Minimize => {
                log::info!("标题栏最小化窗口");
                window.set_minimized(true);
            }
            WindowAction::MaximizeOrRestore => {
                let maximized = window.is_maximized();
                log::info!("标题栏最大化/还原窗口：{}", !maximized);
                window.set_maximized(!maximized);
            }
            WindowAction::Drag => {
                if let Err(err) = window.drag_window() {
                    log::warn!("拖拽窗口失败：{err}");
                }
            }
        }
    }

    /// 根据当前焦点更新 IME 状态与光标区域。
    fn update_ime(&self) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let path = match self.focus.current() {
            Some(p) => p,
            None => {
                window.set_ime_allowed(false);
                return;
            }
        };
        let wants_ime = wants_ime_at_path(&self.tree, path);
        window.set_ime_allowed(wants_ime);
        if wants_ime {
            if let Some(area) = ime_area_at_path(&self.tree, path) {
                window.set_ime_cursor_area(
                    LogicalPosition::new(f64::from(area.origin.x), f64::from(area.origin.y)),
                    LogicalSize::new(f64::from(area.size.width), f64::from(area.size.height)),
                );
            }
        }
    }
}

impl<A: App> ApplicationHandler for Handler<'_, A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        // 持久化恢复: 用 app 的 boot_elapsed_offset 重置 start, 使得
        // AnimationCtx::elapsed 从 effective_now 起算, 而不是 0。
        self.start = Instant::now() - self.app.boot_elapsed_offset();
        let attrs = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_visible(false)
            .with_window_icon(load_window_icon())
            .with_inner_size(LogicalSize::new(
                f64::from(self.config.size.width),
                f64::from(self.config.size.height),
            ));
        // 全平台使用自绘标题栏 (按钮布局样式由 TitleBar 按平台适配，
        // 参见 docs/specs/title-bar-cross-platform.md)。
        let attrs = attrs.with_decorations(false);
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                log_error_chain("创建窗口失败", &err);
                event_loop.exit();
                return;
            }
        };
        log::info!("窗口已创建：{}", self.config.title);

        #[cfg(target_os = "windows")]
        apply_windows_undecorated_style(&window);

        let ctx_start = Instant::now();
        match Context::new(
            Arc::clone(&window),
            self.config.clear_color,
            &self.config.background,
        ) {
            Ok(context) => {
                self.context = Some(context);
                log::info!("渲染上下文初始化耗时：{:?}", ctx_start.elapsed());
            }
            Err(err) => {
                log_error_chain("初始化渲染上下文失败", &err);
                event_loop.exit();
                return;
            }
        }
        window.set_visible(true);
        log::info!("窗口已显示");
        // 机器可读启动基准 (ASCII, 供 tools/benchmark.ps1 解析)。
        log::info!("perf startup_to_visible {:?}", self.boot.elapsed());
        self.window = Some(window);
        // 持续渲染模式：请求首帧，之后每帧结束再请求下一帧
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        Self::log_event(&event);
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.cursor = Point::new(position.x as f32, position.y as f32);
        }
        if let WindowEvent::ModifiersChanged(mods) = event {
            self.modifiers = mods.state();
            return;
        }

        // 鼠标事件经组件树命中分发，分发后可能更新焦点
        if matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
        ) {
            // 鼠标事件
            if let Some(internal) = convert_event(&event, self.cursor, self.modifiers) {
                let result = self.tree.event(&internal, self.root_area, &mut self.msgs);
                if let Event::MouseInput {
                    pressed: true,
                    position,
                    ..
                } = &internal
                {
                    let prev = self.focus.current().map(|p| p.to_vec());
                    self.focus.set_by_click(&self.tree, *position);
                    let curr = self.focus.current().map(|p| p.to_vec());
                    self.dispatch_focus_changes(prev.as_deref(), curr.as_deref());
                    self.focus.acknowledge();
                }
                if result == crate::widget::EventResult::Ignored {
                    self.app.event(&internal);
                }
            }
        } else if let Some(internal) = convert_event(&event, self.cursor, self.modifiers) {
            // 键盘 /IME 事件经焦点路由
            self.dispatch_focused_event(&internal);
        }

        // 消费组件产出的消息
        let msgs: Vec<_> = self.msgs.drain(..).collect();
        for boxed in msgs {
            // 先尝试识别窗口控制动作 (如自绘标题栏发出的 Close/Minimize/Maximize/Drag)
            let boxed = match boxed.downcast::<WindowAction>() {
                Ok(action) => {
                    self.handle_window_action(*action, event_loop);
                    continue;
                }
                Err(b) => b,
            };
            match boxed.downcast::<A::Msg>() {
                Ok(msg) => self.app.update(*msg),
                Err(_) => log::warn!("丢弃类型不匹配的消息"),
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                // 关闭按钮 = 隐藏 (不退出进程); 进程由 Quit 显式退出。
                log::info!("收到关闭请求，隐藏");
                self.hide_window();
            }
            WindowEvent::Resized(size) => {
                log::info!("窗口尺寸变化：{}x{}", size.width, size.height);
                if let Some(context) = &mut self.context {
                    context.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let frame_start = Instant::now();
                let mut rects = RectBatch::new();
                self.texts.clear();
                let screen = self.window.as_ref().map(|w| {
                    let size = w.inner_size();
                    Size::new(size.width as f32, size.height as f32)
                });
                if let Some(screen) = screen {
                    let ctx = AnimationCtx::new(Instant::now(), self.start.elapsed());
                    // 每帧心跳先行: 计时 / 过渡动画推进后, 绑定闭包在 sync 中读到新状态。
                    self.app.tick(&ctx);
                    self.tree.sync(self.app);
                    self.tree.animate(&ctx);
                    self.focus.rebuild(&self.tree);
                    let prev = self.focus.previous().map(|p| p.to_vec());
                    let curr = self.focus.current().map(|p| p.to_vec());
                    self.dispatch_focus_changes(prev.as_deref(), curr.as_deref());
                    self.focus.acknowledge();
                    let size = self
                        .tree
                        .layout(crate::Constraints::tight(screen), &mut self.texts);
                    self.root_area = Rect::new(Point::ZERO, size);
                    self.tree.paint(self.root_area, &mut rects, &mut self.texts);
                    // 无边框窗口下自绘边框与圆角。
                    if self.config.border_thickness > 0.0 {
                        rects.push_rounded_border(
                            self.root_area,
                            self.config.border_color,
                            self.config.border_radius,
                            self.config.border_thickness,
                        );
                    }
                    self.update_ime();
                }
                if let Some(context) = &mut self.context {
                    // 应用层提供的每帧背景状态 (场景选择 / 淡化 / 清屏色)。
                    if let Some(frame) = self.app.background_frame() {
                        context.set_background_frame(frame);
                    }
                    if !context.render(&rects, &mut self.texts) {
                        event_loop.exit();
                        return;
                    }
                }
                if !self.first_frame_done {
                    self.first_frame_done = true;
                    log::info!("首帧渲染耗时：{:?}", frame_start.elapsed());
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("[about_to_wait] tick");
        // 隐藏态时主动 tick: RedrawRequested 不会发火 (窗口隐藏), 必须自己驱动
        // app.tick, 否则计时器冻结, 阶段流转 / 持久化 / flash / beep 全停。
        // ControlFlow::Poll 已保证循环持续转, 此处 tick 每帧推进。
        if !self.is_visible {
            let ctx = AnimationCtx::new(Instant::now(), self.start.elapsed());
            self.app.tick(&ctx);
        }
        // 全局热键通道轮询
        if let Some(rx) = &self.hotkey_rx {
            while let Ok(id) = rx.try_recv() {
                if let Some(msg) = self.app.hotkey(id) {
                    self.app.update(msg);
                }
            }
        }
        // 窗口事件通道轮询
        while let Ok(event) = self.window_event_rx.try_recv() {
            self.apply_window_event(event, event_loop);
        }
        // 控制流: 隐藏时 Poll (循环持续转), 显示时 Wait (节能, 等事件)
        if self.is_visible {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }
}

impl<A: App> Handler<'_, A> {
    /// 隐藏窗口: 应用层 `is_visible` 与 OS 状态同步翻转。
    /// 状态切换的"动作"统一收口在此, 减少 toggle / close / min 等路径的复制。
    fn hide_window(&mut self) {
        if let Some(window) = &self.window {
            window.set_visible(false);
        }
        self.is_visible = false;
    }

    /// 显示窗口 + 抢焦点 + 重绘。winit 的 SW_SHOW 默认不抢焦点,
    /// 显式 focus_window 防止"已显示但被遮" (尤其在另一 app 后台时)。
    fn show_window(&self) {
        if let Some(window) = &self.window {
            window.set_visible(true);
            window.request_redraw();
            window.focus_window();
        }
    }

    /// 处理 App 经 WindowEventSender 主动发来的事件。
    fn apply_window_event(&mut self, event: WindowAppEvent, event_loop: &ActiveEventLoop) {
        match event {
            WindowAppEvent::ToggleVisible => {
                // Handler 是 is_visible 唯一事实源: 翻转后立即应用到 winit 窗口。
                self.is_visible = !self.is_visible;
                if self.is_visible {
                    self.show_window();
                } else {
                    self.hide_window();
                }
            }
            WindowAppEvent::Quit => event_loop.exit(),
            WindowAppEvent::PhaseAdvanced => {
                // 隐藏态时阶段流转 → 自动呼出 (用户可能没在电脑前, 或在另一 app)
                if !self.is_visible {
                    self.is_visible = true;
                    log::info!("阶段流转, 自动呼出窗口");
                    self.show_window();
                }
            }
        }
    }
}

/// 打开窗口并运行应用：事件分发、消息驱动、每帧重绘，直到窗口关闭。
pub fn run_app<A: App>(config: WindowConfig, app: &mut A) -> Result<(), WindowError> {
    let boot = Instant::now();
    let event_loop = EventLoop::new()?;
    let texts_start = Instant::now();
    let texts = TextBatch::new();
    log::info!(
        "文本批次初始化 (含字体加载) 耗时：{:?}",
        texts_start.elapsed()
    );
    // 注入窗口事件发送器 (App 主动控制窗口: 显隐 / 退出)
    let (window_event_tx, window_event_rx) = channel();
    app.attach_window_sender(WindowEventSender {
        sender: window_event_tx,
    });
    // 启动全局热键监听线程 (None 表示平台不支持)
    let hotkey_rx = hotkeys::spawn().map(|(rx, _handle)| rx);
    let mut handler = Handler {
        tree: app.view(),
        config,
        window: None,
        context: None,
        texts,
        cursor: Point::ZERO,
        modifiers: ModifiersState::empty(),
        app,
        msgs: MsgQueue::new(),
        root_area: Rect::default(),
        focus: FocusManager::new(),
        start: Instant::now(),
        clipboard: None,
        first_frame_done: false,
        boot,
        hotkey_rx,
        window_event_rx,
        is_visible: true,
    };
    let run_start = Instant::now();
    event_loop.run_app(&mut handler)?;
    log::info!("事件循环运行耗时：{:?}", run_start.elapsed());
    log::info!("事件循环已退出");
    Ok(())
}

/// 打开窗口并运行事件循环 (无应用), 直到用户关闭窗口。
pub fn run(config: WindowConfig) -> Result<(), WindowError> {
    struct NoopApp;
    impl App for NoopApp {
        type Msg = ();
        fn update(&mut self, _msg: ()) {}
        fn view(&self) -> Node {
            crate::widget::node(crate::widget::Text::new(""))
        }
    }
    run_app(config, &mut NoopApp)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// 冒烟测试：仅创建事件循环 (链接触发 shim 生成的导入库)。
    /// 若导入库损坏，本测试会以访问违规崩溃。
    #[test]
    fn event_loop_creation_smoke() {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        let event_loop = EventLoop::builder().with_any_thread(true).build();
        drop(event_loop.expect("创建事件循环失败"));
    }

    #[test]
    fn load_icon_from_valid_png_succeeds() {
        let path = PathBuf::from("assets").join("logo").join("logo_256.png");
        let icon = load_icon_from_png(&path);
        assert!(icon.is_ok(), "应能加载有效 PNG 图标：{icon:?}");
    }

    #[test]
    fn load_icon_from_missing_path_returns_error() {
        let path = PathBuf::from("assets").join("logo").join("nonexistent.png");
        let icon = load_icon_from_png(&path);
        assert!(icon.is_err());
    }

    #[test]
    fn error_chain_messages_include_all_sources() {
        #[derive(Debug)]
        struct TestError {
            message: &'static str,
            source: Option<Box<TestError>>,
        }

        impl std::fmt::Display for TestError {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.message)
            }
        }

        impl std::error::Error for TestError {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.source
                    .as_deref()
                    .map(|source| source as &(dyn std::error::Error + 'static))
            }
        }

        let error = TestError {
            message: "outer",
            source: Some(Box::new(TestError {
                message: "middle",
                source: Some(Box::new(TestError {
                    message: "leaf",
                    source: None,
                })),
            })),
        };

        assert_eq!(
            error_chain_messages("初始化渲染上下文失败", &error),
            vec![
                String::from("初始化渲染上下文失败：outer"),
                String::from("  原因 1：middle"),
                String::from("  原因 2：leaf"),
            ]
        );
    }
}
