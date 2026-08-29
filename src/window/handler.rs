//! @author 十四叔
//! @date 2026/07/17

//! winit 应用处理器：驱动窗口生命周期与事件分发。
//!
//! 由 `run_app` 构造，通过 `event_loop.run_app(&mut handler)` 启动。
//! 持有应用本体 (`&mut A`)、组件树、GPU 上下文、事件通道，协调：
//! - 窗口创建 (resumed)
//! - 事件分发 (window_event)
//! - 消息消费 (downcast 到 `WindowAction` 或 `App::Msg`)
//! - 每帧渲染 (RedrawRequested)
//! - 心跳驱动 (about_to_wait, 隐藏态推进 app.tick + 60fps 主动重绘)

use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{Window as WinitWindow, WindowAttributes, WindowId, WindowLevel},
};

use crate::app::{AnimationCtx, App};
use crate::event::{Event, ImeEvent, Key, NamedKey, WindowAction};
use crate::render::{Context, ImageBatch, RectBatch, TextBatch};
use crate::widget::{
    FocusManager, MsgQueue, Node, event_at_path, ime_area_at_path, selected_text_at_path,
    wants_ime_at_path,
};
use crate::{Point, Rect, Size};

use super::event::{WindowAppEvent, convert_event};
#[cfg(target_os = "windows")]
use super::icon::apply_windows_undecorated_style;
use super::icon::{window_icons, with_taskbar_icon};

/// winit 应用处理器，驱动窗口生命周期与事件分发。
///
/// 字段全部私有：外部通过 [`Handler::new`] 构造，通过 trait 方法 (resumed /
/// window_event / about_to_wait) 操作。
pub(super) struct Handler<'a, A: App> {
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
    focus: FocusManager,
    /// 应用启动时间 (用于动画)。
    start: Instant,
    /// 系统剪贴板 (懒加载)。
    clipboard: Option<arboard::Clipboard>,
    /// 是否已完成首帧渲染 (用于一次性诊断计时)。
    first_frame_done: bool,
    /// 进程入口时间 (run_app 起点，用于启动总耗时基准)。
    boot: Instant,
    /// 全局热键接收器 (来自热键线程，`None` 表示未启用或平台不支持)。
    hotkey_rx: Option<Receiver<u8>>,
    /// 托盘生命周期句柄 (持有期间托盘图标可见; Drop 时移除)。
    /// 仅靠 Drop 副作用保活，字段本身不读; `dead_code` 抑制。
    #[allow(dead_code)]
    tray: Option<super::tray::TrayHandle>,
    /// 窗口事件接收器 (App 主动发出：显隐 / 退出)。
    window_event_rx: Receiver<WindowAppEvent>,
    /// 当前窗口可见性 (热键 ToggleVisible 状态记录，与 Handler 同步)。
    is_visible: bool,
    /// 窗口当前是否持有 OS 焦点 (由 WindowEvent::Focused 维护)。
    ///
    /// Alt+Tab 激活本窗口时，若用户先松 Alt、后松 Tab, Windows 会把迟到的
    /// Tab 投递进队列——实测它排在 Focused(true) 之前 (同毫秒、序在前),
    /// 且此时 winit 已清零修饰键状态，无法靠 Alt 识别。唯一可靠的判据是
    /// 到达时窗口尚未持有 OS 焦点 (见 [`dispatch_focused_event`] 的 Tab 守卫)。
    has_os_focus: bool,
    /// 最后一次可信的客户区尺寸 (物理像素)，窗口创建时取真实 inner_size 初始化。
    ///
    /// Windows 坑 (2026-08-16 定位「剪贴板唤起列表不全」根因):
    /// set_visible(false) 隐藏窗口后，系统补发一个 160x28 的幻影 WM_SIZE
    /// (历史 minimized 占位尺寸), 而再次 set_visible(true) 时并不补发真实
    /// 尺寸 —— winit 缓存的 inner_size 就此卡死在幻影值：布局视口归零
    /// (Scrollable 纠偏被无限推迟)、surface 尺寸与窗口失配导致
    /// get_current_texture 永久 Outdated, 窗口定格在隐藏前的最后一帧。
    /// 对策：隐藏态的 Resized 一律不信 (见 window_event), 每帧布局尺寸以
    /// 本字段为准而非再问 winit, 显示时用 request_inner_size 自愈 winit 缓存。
    /// 已知取舍：隐藏期间的 DPI 变化 (拖到别的显示器) 同样被忽略, 唤起时
    /// 会以旧物理像素强制回尺寸 —— 对固定尺寸弹窗影响小, 下次真实尺寸
    /// 变化即自愈。
    last_real_size: PhysicalSize<u32>,
    /// 热键主键吞键守卫 (热键触发时置入, 主键抬起 / 失焦时清除)。
    /// 详见 [`hotkey_swallow_filter`]。
    swallow_hotkey_key: Option<KeyCode>,
    /// 图像纹理收集器 (每帧清空，paint 阶段填充)。
    images: ImageBatch,
    /// ── Adaptive 帧率治理状态 (仅 WindowMode::Adaptive 使用) ──
    /// 当前帧率档缓存 (decide 输出, 变化时记日志; render_frame 尾据此门控续链)。
    frame_rate: super::frame_budget::FrameRate,
    /// 上次活动时刻 (窗口输入/托盘热键动作/应用事件; Adaptive 帧率判定输入)。
    last_activity: Instant,
    /// 事件升帧截止时刻 (产品经 boost_frames 请求; None = 未升帧)。
    boost_until: Option<Instant>,
    /// 上次全屏检测轮询时刻 (500ms 一探, 纯查询系统调用不每帧跑)。
    last_fullscreen_poll: Instant,
    /// 前台全屏应用检测缓存 (true = 渲染暂停, 仅低频轮询)。
    fullscreen_suspended: bool,
}

impl<'a, A: App> Handler<'a, A> {
    /// 构造 Handler。仅接收调用方真正提供的值，其他字段用合理默认值
    /// (None / empty / new / Instant::now()) 就地初始化。
    /// `tree` 应为 `app.view()` 的结果 (在调用方先求值，以满足借用检查)。
    ///
    /// 9 个参数是必要的 (每个子系统一个入口：配置 / App / 组件树 / 文本 /
    /// 热键通道 / 托盘 / 窗口事件 / 启动基准 / GPU 线程), 单点构造不接受
    /// 拆 sub-config (各子系统生命周期独立，强耦合反而失真)。
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        config: WindowConfig,
        app: &'a mut A,
        tree: Node,
        texts: TextBatch,
        hotkey_rx: Option<Receiver<u8>>,
        tray: Option<super::tray::TrayHandle>,
        window_event_rx: Receiver<WindowAppEvent>,
        boot: Instant,
    ) -> Self {
        Self {
            config,
            window: None,
            context: None,
            texts,
            cursor: Point::ZERO,
            modifiers: ModifiersState::empty(),
            app,
            tree,
            msgs: MsgQueue::new(),
            root_area: Rect::default(),
            focus: FocusManager::new(),
            start: Instant::now(),
            clipboard: None,
            first_frame_done: false,
            boot,
            hotkey_rx,
            tray,
            window_event_rx,
            is_visible: true,
            has_os_focus: false,
            last_real_size: PhysicalSize::new(0, 0),
            swallow_hotkey_key: None,
            images: ImageBatch::new(),
            frame_rate: super::frame_budget::FrameRate::Full,
            last_activity: boot,
            boost_until: None,
            // 首次轮询立即执行 (减 1s 使首个 about_to_wait 即检测全屏态)。
            last_fullscreen_poll: boot - Duration::from_secs(1),
            fullscreen_suspended: false,
        }
    }
}

use super::{CloseBehavior, WindowConfig};

/// Tab 焦点遍历是否放行：仅当窗口持有 OS 焦点。
///
/// Alt+Tab 激活本窗口时泄漏的 Tab 排在 Focused(true) 之前到达
/// (此时 `has_os_focus == false`); 用户主动遍历只发生在持有 OS 焦点期间。
fn tab_traverse_allowed(has_os_focus: bool) -> bool {
    has_os_focus
}

/// Windows 虚拟键码 → winit KeyCode (仅覆盖字母与数字键; 其他键不吞, 返回 None)。
///
/// 字母键的 vk 即 ASCII 大写码 (0x41..=0x5A), 主键盘数字键同理 (0x30..=0x39)。
fn vk_to_key_code(vk: u32) -> Option<KeyCode> {
    let code = match vk {
        0x41 => KeyCode::KeyA,
        0x42 => KeyCode::KeyB,
        0x43 => KeyCode::KeyC,
        0x44 => KeyCode::KeyD,
        0x45 => KeyCode::KeyE,
        0x46 => KeyCode::KeyF,
        0x47 => KeyCode::KeyG,
        0x48 => KeyCode::KeyH,
        0x49 => KeyCode::KeyI,
        0x4A => KeyCode::KeyJ,
        0x4B => KeyCode::KeyK,
        0x4C => KeyCode::KeyL,
        0x4D => KeyCode::KeyM,
        0x4E => KeyCode::KeyN,
        0x4F => KeyCode::KeyO,
        0x50 => KeyCode::KeyP,
        0x51 => KeyCode::KeyQ,
        0x52 => KeyCode::KeyR,
        0x53 => KeyCode::KeyS,
        0x54 => KeyCode::KeyT,
        0x55 => KeyCode::KeyU,
        0x56 => KeyCode::KeyV,
        0x57 => KeyCode::KeyW,
        0x58 => KeyCode::KeyX,
        0x59 => KeyCode::KeyY,
        0x5A => KeyCode::KeyZ,
        0x30 => KeyCode::Digit0,
        0x31 => KeyCode::Digit1,
        0x32 => KeyCode::Digit2,
        0x33 => KeyCode::Digit3,
        0x34 => KeyCode::Digit4,
        0x35 => KeyCode::Digit5,
        0x36 => KeyCode::Digit6,
        0x37 => KeyCode::Digit7,
        0x38 => KeyCode::Digit8,
        0x39 => KeyCode::Digit9,
        _ => return None,
    };
    Some(code)
}

/// 热键主键此刻是否仍被物理按住 (武装吞键守卫的前置检查)。
///
/// 热键经线程消息队列 + 100ms 心跳轮询到达主循环, 天然滞后于物理按键;
/// 快速点按时主键在武装前早已抬起。焦点合成只覆盖「按住中的键」,
/// 已抬起就意味着不会有漏键, 无需守卫 —— 此时若强行武装, 守卫将因永远
/// 收不到主键抬起而卡死, 误吞用户本次会话里后续的主动输入。
#[cfg(target_os = "windows")]
fn vk_still_held(vk: u32) -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
    // 返回值的最高位 = 当前按住 (最低位是"自上次调用以来按过", 不看)。
    unsafe { (GetAsyncKeyState(vk as i32) as u16) & 0x8000 != 0 }
}

/// 非 Windows 平台无全局热键线程, 守卫永不武装。
#[cfg(not(target_os = "windows"))]
fn vk_still_held(_vk: u32) -> bool {
    false
}

/// 热键主键吞键判定 (2026-08-16 定位「剪贴板唤起列表不全」真根因)。
///
/// 热键触发后，主键的按下事件仍可能漏进刚唤起并抢到焦点的窗口，有两条路径
/// (实测均发生):
/// 1. winit 在 WM_SETFOCUS 时给所有物理按住中的键合成 Pressed 事件
///    (合成事件 `repeat=false`, 顺序: 字母键在前修饰键在后 —— 日志实锤);
/// 2. 主键按住不放时 Windows 键盘自动重复继续投递 (`repeat=true`,
///    RegisterHotKey 的 MOD_NOREPEAT 只挡 WM_HOTKEY 重发, 不挡按键本身)。
///
/// 漏进的 "v" 落进检索框, 历史列表被静默过滤 (表现: 行数变少、缺最新条目,
/// 清空检索框即恢复)。
///
/// `swallow` 为热键触发时记下的主键; 返回 true 表示该键盘事件应被丢弃。
/// 武装前置条件 (vk_still_held): 主键此刻仍被物理按住 —— 快速点按 (抬起
/// 早于武装) 不会武装, 也就不会形成「等不到抬起事件」的卡死守卫。
///
/// 已知残余 (可接受): 若主键恰在武装检查与焦点窃取之间的毫秒级空窗抬起,
/// 守卫仍会卡死至下次失焦 (Focused(false) 兜底解除); 期间用户第一次主动
/// 按下主键会被吞一次 (含 Ctrl+V 粘贴按键流), 该次的 Released 到达即自愈。
/// 相比被修掉的静默过滤 (无感知、须清检索框才恢复), 这是数量级改善。
fn hotkey_swallow_filter(
    swallow: &mut Option<KeyCode>,
    physical: PhysicalKey,
    state: ElementState,
) -> bool {
    let Some(code) = *swallow else {
        return false;
    };
    if physical != PhysicalKey::Code(code) {
        return false;
    }
    match state {
        // 主键仍按住期间的按下 (焦点合成 / 自动重复): 吞, 守卫保持。
        ElementState::Pressed => true,
        // 主键抬起: 解除守卫, 事件放行 (对文本输入无副作用)。
        ElementState::Released => {
            *swallow = None;
            false
        }
    }
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
            WindowEvent::Focused(gained) => {
                log::info!("OS 焦点：{gained}");
            }
            _ => {}
        }
    }

    /// 发送焦点进 / 出事件。
    fn dispatch_focus_changes(&mut self, previous: Option<&[usize]>, current: Option<&[usize]>) {
        // 焦点迁移是稀有且高价值的诊断信号 (Alt+Tab 泄漏类排查), 值得常驻。
        if previous != current {
            log::info!("焦点变化：{previous:?} -> {current:?}");
        }
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
            // Alt+Tab 切回本窗口时，先松 Alt、后松 Tab 的指法会让 Windows 把
            // 迟到的 Tab 投递进队列——实测它排在 Focused(true) 之前到达，
            // 且修饰键已被清零，无法靠 Alt 状态识别。唯一可靠的判据：
            // 窗口未持有 OS 焦点时到达的 Tab 必是泄漏，不做焦点遍历。
            // 合法 Tab 遍历只会发生在持有 OS 焦点期间，零误伤。
            if tab_traverse_allowed(self.has_os_focus) {
                if self.modifiers.shift_key() {
                    self.focus.prev();
                } else {
                    self.focus.next();
                }
            }
            return;
        }

        let Some(path) = self.focus.current().map(|p| p.to_vec()) else {
            // 无焦点时回退到应用层
            self.app.event(event);
            return;
        };

        // 键盘前置过滤: 应用层在焦点分发前拦截 (如无修饰字母键触发收藏)
        if let Event::Key { pressed: true, .. } = event {
            if let Some(msg) = self.app.app_key_filter(event) {
                self.msgs.push(Box::new(msg));
                return;
            }
        }

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
    fn handle_window_action(&mut self, action: WindowAction, event_loop: &ActiveEventLoop) {
        // 关闭按钮遵循 close_behavior 策略 (隐藏 / 退出), 与 Alt+F4 一致。
        if let WindowAction::Close = action {
            self.handle_close_request(event_loop, "标题栏关闭窗口");
            return;
        }
        let Some(window) = self.window.as_ref() else {
            return;
        };
        match action {
            WindowAction::Close => {} // 已在上面处理
            WindowAction::Minimize => {
                log::info!("标题栏最小化窗口");
                window.set_minimized(true);
            }
            WindowAction::MaximizeOrRestore => {
                let maximized = window.is_maximized();
                log::info!("标题栏最大化/还原窗口：{}", !maximized);
                window.set_maximized(!maximized);
                // 通知应用层窗口最大化状态已切换 (TitleBar 据此 □↔□□)。
                self.app.maximized_changed(!maximized);
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

    /// 渲染一帧并 present (RedrawRequested 与启动预渲染共用)。
    ///
    /// 每帧心跳 → sync 绑定 → 焦点重建 → 布局 → 绘制 → 提交 wgpu。
    /// 渲染失败时退出事件循环 (防御; Context::render 目前恒成功)。
    fn render_frame(&mut self, event_loop: &ActiveEventLoop) {
        let frame_start = Instant::now();
        let mut rects = RectBatch::new();
        self.texts.clear();
        self.images = ImageBatch::new();
        let screen = self.window.as_ref().map(|_| {
            // 尺寸取自 last_real_size 而非 winit inner_size: 后者在 Windows 上
            // 被隐藏态的幻影 WM_SIZE 污染后不再自愈 (见 last_real_size 字段注释)。
            Size::new(
                self.last_real_size.width as f32,
                self.last_real_size.height as f32,
            )
        });
        if let Some(screen) = screen {
            let ctx = AnimationCtx::new(Instant::now(), self.start.elapsed());
            // 每帧心跳先行：计时 / 过渡动画推进后，绑定闭包在 sync 中读到新状态。
            self.app.tick(&ctx);
            self.tree.sync(self.app);
            self.tree.animate(&ctx);
            self.focus.rebuild(&self.tree);
            // 应用请求恢复焦点 (如面板关闭后回到打开面板的按钮)。请求一次性:
            // 每帧存在即消费 (focus_restored 清除), 仅在焦点为空时应用 —
            // 若关闭面板时焦点仍被占用 (如点击面板内组件关闭), 请求静默丢弃,
            // 避免残留请求在用户稍后清焦 (Esc/点空白) 时误把焦点拉回按钮。
            if let Some(id) = self.app.focus_request() {
                self.app.focus_restored();
                if self.focus.current().is_none() {
                    self.focus.set_focus_by_id(id);
                }
            }
            let prev = self.focus.previous().map(|p| p.to_vec());
            let curr = self.focus.current().map(|p| p.to_vec());
            self.dispatch_focus_changes(prev.as_deref(), curr.as_deref());
            self.focus.acknowledge();
            let size = self
                .tree
                .layout(crate::Constraints::tight(screen), &mut self.texts);
            self.root_area = Rect::new(Point::ZERO, size);
            self.tree.paint(self.root_area, &mut rects, &mut self.texts);
            self.tree.paint_image(self.root_area, &mut self.images);
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
            // Context::render 目前恒返回 true; 失败路径为防御 (退出事件循环)。
            if !context.render(&rects, &mut self.texts, &mut self.images) {
                event_loop.exit();
                return;
            }
        }
        if !self.first_frame_done {
            self.first_frame_done = true;
            // 首次调用必为显示前的预渲染 (resume_window 先调再 set_visible)。
            log::info!("预渲染首帧耗时：{:?}", frame_start.elapsed());
        }
        if let Some(window) = &self.window {
            // Adaptive 降帧/暂停态不续渲染链 (由 about_to_wait 按档单驱),
            // 否则自续链会把降帧架空回全速。
            if self.config.mode != super::WindowMode::Adaptive
                || super::frame_budget::should_continue_render_chain(self.frame_rate)
            {
                window.request_redraw();
            }
        }
    }
}

impl<A: App> ApplicationHandler for Handler<'_, A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        // 持久化恢复：用 app 的 boot_elapsed_offset 重置 start, 使得
        // AnimationCtx::elapsed 从 effective_now 起算，而不是 0。
        self.start = Instant::now() - self.app.boot_elapsed_offset();
        let window_width = f64::from(self.config.size.width);
        let window_height = f64::from(self.config.size.height);
        // 窗口居中：取主显示器尺寸计算偏移，确保窗口显示在屏幕正中央。
        let position = event_loop.available_monitors().next().map(|monitor| {
            let m_size = monitor.size();
            let m_pos = monitor.position();
            LogicalPosition::new(
                (m_pos.x as f64 + (m_size.width as f64 - window_width) / 2.0).max(0.0),
                (m_pos.y as f64 + (m_size.height as f64 - window_height) / 2.0).max(0.0),
            )
        });
        let (window_icon, taskbar_icon) = window_icons(&self.config.logo_name);
        let mut attrs = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_visible(false)
            .with_inner_size(LogicalSize::new(window_width, window_height));
        // 任务栏按钮图标取 ICON_BIG; winit 0.30 的 with_window_icon 只设 ICON_SMALL
        // (标题栏), 不补 ICON_BIG 时任务栏偶发显示系统缺省图标 (见 icon::with_taskbar_icon)。
        attrs = with_taskbar_icon(attrs, taskbar_icon);
        attrs = attrs.with_window_icon(window_icon);
        // 显式居中位置 (同时是最大化窗口的还原位置: 取消最大化后回到屏幕中央)。
        // 注意: 不设 with_maximized —— winit 的 create_window 对 maximized 属性会调
        // set_maximized → ShowWindow(SW_MAXIMIZE), 无视 with_visible(false) 直接让
        // 窗口在 GPU 初始化期间全屏可见 (空表面白屏一闪)。见下方「先显示再最大化」。
        if let Some(pos) = position {
            attrs = attrs.with_position(pos);
        }
        // 全平台使用自绘标题栏 (按钮布局样式由 TitleBar 按平台适配，
        // 参见 docs/specs/title-bar-cross-platform.md)。
        let attrs = attrs.with_decorations(false);
        // 置顶层级: 常驻陪伴形态 (桌景) 置顶于普通窗口之上; 默认 Normal,
        // 既有产品 (番茄钟/剪贴板) 层级行为不变。
        let attrs = attrs.with_window_level(if self.config.topmost {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        });
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                super::log_error_chain("创建窗口失败", &err);
                event_loop.exit();
                return;
            }
        };
        log::info!("窗口已创建：{}", self.config.title);

        #[cfg(target_os = "windows")]
        apply_windows_undecorated_style(&window);

        // 位置记忆: 创建后 (显示前) 恢复到上次位置, 钳进所在/最近显示器
        // 工作区; 最大化由系统管理位置, 跳过。无存储 (默认钩子) 时保持居中。
        #[cfg(target_os = "windows")]
        if self.config.placement == super::ShowPlacement::Remember && !self.config.maximized {
            if let Some(saved) = self.app.load_window_position() {
                super::placement::restore_position(&window, saved, window.inner_size());
            }
        }

        // 同步 inline 初始化 GPU 上下文 (实例 + surface + 适配器 + 设备 + 管线)。
        // request_adapter 传 `compatible_surface: Some(&surface)` 让 DX12 后端
        // 一步优化 device / presentation engine 创建，比传 None 省 ~200ms。
        // 实例预建后台线程试过又撤回 (2026-07): 实例时间确实藏住了, 但
        // request_adapter 等额变贵 (+250ms), 净收益为零且方差更大。
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
                super::log_error_chain("初始化渲染上下文失败", &err);
                event_loop.exit();
                return;
            }
        }
        // 先持有窗口引用并记下真实客户区尺寸: 预渲染首帧需要布局基准;
        // 此后每帧布局以 last_real_size 为准 (隐藏后 winit inner_size 会卡幻影值)。
        self.last_real_size = window.inner_size();
        self.window = Some(Arc::clone(&window));
        // 预渲染首帧: 隐藏时渲染 + present, 显示时直接见内容 — 避免首帧就绪前白屏。
        // 平台注: 已在 Windows/DX12 验证。Wayland 上隐藏表面未映射, get_current_texture
        // 返回 Outdated/Lost 导致预渲染帧被跳过 — 优雅退化为旧行为 (无白屏增益, 无崩溃)。
        self.render_frame(event_loop);
        window.set_visible(true);
        // 先以普通尺寸显示 (预渲染内容已在屏), 再最大化 — 而非在 create_window 设
        // maximized (那会让窗口在 GPU 初始化期间全屏白屏)。最大化过程有内容, 无白屏。
        if self.config.maximized {
            window.set_maximized(true);
        }
        log::info!("窗口已显示");
        // 初始可见性同步: visibility_changed 的契约是「可见性变化后必回调」,
        // 启动首次显示也是一次变化 —— 漏掉则应用层的可见性镜像 (热键/Esc 的
        // toggle 方向判断) 与真相相反 (2026-08-27 剪贴板首轮启动 Esc 与
        // 点走双双失效的根因: 首轮 ToggleVisible 误判方向走了显示路径)。
        self.app.visibility_changed(self.is_visible);
        // 机器可读启动基准 (ASCII, 供 tools/benchmark.ps1 解析)。
        log::info!("perf startup_to_visible {:?}", self.boot.elapsed());
        // 持续渲染模式: render_frame 末尾已请求首帧, 之后每帧结束再请求下一帧。
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
        if let WindowEvent::Focused(gained) = event {
            self.has_os_focus = gained;
            if !gained {
                // 失焦即解除吞键守卫 (主键抬起事件可能随焦点易手而错过)。
                self.swallow_hotkey_key = None;
            }
            // 窗口焦点变化时通知应用层 (用于失焦自动隐藏等行为)。
            if gained {
                self.app.focus_gained();
            } else {
                self.app.focus_lost();
            }
        }

        // 热键主键吞键: 唤起抢到焦点后, 主键仍按住期间的按下事件 (焦点合成 /
        // 自动重复) 会漏进本窗口 (见 hotkey_swallow_filter), 必须吞掉。
        if let WindowEvent::KeyboardInput {
            event: key_event, ..
        } = &event
        {
            if hotkey_swallow_filter(
                &mut self.swallow_hotkey_key,
                key_event.physical_key,
                key_event.state,
            ) {
                log::debug!("吞掉热键主键按下：{:?}", key_event.physical_key);
                return;
            }
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
                Self::note_activity(self.config.mode, &mut self.last_activity);
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
            Self::note_activity(self.config.mode, &mut self.last_activity);
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
                self.handle_close_request(event_loop, "收到关闭请求");
            }
            WindowEvent::Resized(size) => {
                log::info!("窗口尺寸变化：{}x{}", size.width, size.height);
                // 隐藏态的 Resized 不可信 (Windows 幻影 160x28, 见 last_real_size
                // 字段注释): 不接受、不重建 surface, 待显示时以 last_real_size 为准。
                if !self.is_visible {
                    log::info!("隐藏态忽略尺寸变化：{}x{}", size.width, size.height);
                    return;
                }
                self.last_real_size = size;
                if let Some(context) = &mut self.context {
                    context.resize(size.width, size.height);
                }
            }
            WindowEvent::Moved(position) => {
                // 位置记忆: 拖动后回报物理坐标 (产品侧防抖落盘)。
                // 最大化/最小化位置由系统管理, 不记 —— 最大化位污染还原位;
                // 最小化窗口被挪到幻影坐标 (-32000,-32000), 记忆会被冲掉。
                if self.config.placement == super::ShowPlacement::Remember
                    && self.window.as_ref().is_some_and(|w| {
                        super::placement::should_remember_position(
                            w.is_maximized(),
                            w.is_minimized().unwrap_or(false),
                        )
                    })
                {
                    self.app.save_window_position(position.x, position.y);
                }
            }
            WindowEvent::RedrawRequested => {
                self.render_frame(event_loop);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        log::debug!("[about_to_wait] tick");
        // 隐藏态时主动 tick: RedrawRequested 不会发火 (窗口隐藏), 必须自己驱动
        // app.tick, 否则计时器冻结，阶段流转 / 持久化 / flash / beep 全停。
        // ControlFlow::Poll 已保证循环持续转，此处 tick 每帧推进。
        if !self.is_visible {
            let ctx = AnimationCtx::new(Instant::now(), self.start.elapsed());
            self.app.tick(&ctx);
        }
        // 全局热键通道轮询
        // 热键与托盘动作都可能改变勾选态 (穿透/置顶等): 任一通道有派发就在
        // 本帧末尾重建托盘菜单, 保持勾选项与 App 状态一致。
        let mut tray_menu_dirty = false;
        if let Some(rx) = &self.hotkey_rx {
            while let Ok(id) = rx.try_recv() {
                Self::note_activity(self.config.mode, &mut self.last_activity);
                // 记下热键主键: 按住热键时主键的按下事件会漏进刚唤起的窗口
                // (见 hotkey_swallow_filter), 抬起前这些按下必须吞掉。
                // 仅当主键此刻仍被物理按住才武装守卫 (见 vk_still_held)。
                self.swallow_hotkey_key = self
                    .config
                    .hotkeys
                    .iter()
                    .find(|h| h.id == id)
                    .filter(|h| vk_still_held(h.vk))
                    .and_then(|h| vk_to_key_code(h.vk));
                if let Some(msg) = self.app.hotkey(id) {
                    self.app.update(msg);
                }
                tray_menu_dirty = true;
            }
        }
        // 托盘菜单事件轮询 (muda 内部维护的全局通道)。每个 MenuId 是字符串
        // 包装 (`MenuId(pub String)`); 我们约定菜单项 id 用 ASCII 数字 ("1"/"2"/"3"),
        // 解析成 u8 后转交 `app.tray_action`。非法 id 静默忽略。
        let tray_rx = tray_icon::menu::MenuEvent::receiver();
        while let Ok(event) = tray_rx.try_recv() {
            if let Ok(id) = event.id.0.parse::<u8>() {
                Self::note_activity(self.config.mode, &mut self.last_activity);
                if let Some(msg) = self.app.tray_action(id) {
                    self.app.update(msg);
                }
                tray_menu_dirty = true;
            }
        }
        // 窗口事件通道轮询
        while let Ok(event) = self.window_event_rx.try_recv() {
            Self::note_activity(self.config.mode, &mut self.last_activity);
            if self.apply_window_event(event, event_loop) {
                tray_menu_dirty = true;
            }
        }
        // 动作 (托盘点击/全局热键/经 sender 的状态变更) 可能改了勾选态:
        // 重建托盘菜单 (tray_menu 从 App 状态现查, 幂等)。
        if tray_menu_dirty {
            if let Some(tray) = &self.tray {
                tray.set_menu(self.app.tray_menu());
            }
        }
        // 焦点事件流对账 (Windows): AttachThreadInput 抢前台后, Windows 的
        // 焦点消息投递可能整体失真 (WM_SETFOCUS / WM_NCACTIVATE / WM_KILLFOCUS
        // 任一缺席 —— 2026-08-25 日志实锤两种形态), winit 的 Focused 事件流
        // 随之卡死。此处每帧拿 OS 前台真相对照 winit 报到状态, 缺什么补什么
        // (详见 foreground::reconcile_focus_state); 一两帧内收敛, 幂等自限。
        #[cfg(target_os = "windows")]
        if self.is_visible {
            if let Some(window) = &self.window {
                super::foreground::reconcile_focus_state(window, self.has_os_focus);
            }
        }
        // 控制流策略：根据窗口模式决定 ControlFlow。
        //   OnDemand (默认): 隐藏态 → Wait (零唤醒, 省电); 可见态 → WaitUntil(16ms)。
        //   Continuous (番茄钟等): 无论可见与否都用 WaitUntil(16ms), 保持 tick 推进。
        //
        //   WaitUntil 比 Poll 省 CPU (空载时 Poll 一秒跑几千次，WaitUntil 仅 ~60 次),
        //   同时 OS 调度超时 / 外部事件 / 模态菜单 close 仍能及时唤醒 winit。
        //   关键：muda 托盘菜单的 TrackPopupMenu 是 Windows 阻塞 API, 会在主线程
        //     跑模态消息循环，期间 winit 事件循环被冻结; 菜单关闭后必须主动
        //     重发 RedrawRequested, 否则 pending 的 paint 消息可能被模态循环
        //     过滤/丢弃, UI 卡在旧值不更新 (读秒停止、按钮 label 不切)。
        //   Adaptive (桌景等常驻氛围应用): 帧率治理接管 —— 全屏检测轮询 +
        //     活动/升帧判定, 按帧率档驱动重绘与轮询间隔 (见 adaptive_frame_pacing)。
        let control_flow = if self.config.mode == super::WindowMode::Adaptive {
            self.adaptive_frame_pacing()
        } else {
            if self.is_visible {
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            self.control_flow_for_current_state()
        };
        event_loop.set_control_flow(control_flow);
    }
}

impl<A: App> Handler<'_, A> {
    /// 根据当前窗口模式和可见性决定控制流。
    ///
    /// - `OnDemand` + 隐藏 → `WaitUntil(100ms)` (低频轮询, 确保热键及时响应)
    /// - `OnDemand` + 可见 → `WaitUntil(16ms)` (事件驱动重绘, ~60fps)
    /// - `Continuous` (任何状态) → `WaitUntil(16ms)` (番茄钟等需持续 tick)
    fn control_flow_for_current_state(&self) -> ControlFlow {
        match self.config.mode {
            // 隐藏态使用 100ms 轮询: 既保证热键及时响应, 又避免零唤醒导致热键失效
            // (ControlFlow::Wait 无法被标准库 mpsc 通道唤醒)
            super::WindowMode::OnDemand if !self.is_visible => {
                ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(100))
            }
            _ => ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)),
        }
    }

    /// Adaptive 模式帧率驱动: 全屏检测低频轮询 → 帧率决策 → 按档发重绘。
    /// 降帧/暂停态 render_frame 尾不续链 (门控见 render_frame), 由本函数
    /// 按轮询间隔单驱 —— 降帧生效的关键。
    fn adaptive_frame_pacing(&mut self) -> ControlFlow {
        let now = Instant::now();
        if !self.is_visible {
            // 隐藏态: 与 OnDemand 隐藏同款低频轮询 (保热键响应),
            // 零渲染零检测 (隐藏本就不渲染, 全屏检测无意义)。
            return ControlFlow::WaitUntil(now + Duration::from_millis(100));
        }
        // 全屏检测 500ms 一探 (纯查询式系统调用, 不每帧跑)。
        if now.duration_since(self.last_fullscreen_poll) >= Duration::from_millis(500) {
            self.last_fullscreen_poll = now;
            let fullscreen = super::fullscreen::fullscreen_app_foreground();
            if fullscreen != self.fullscreen_suspended {
                self.fullscreen_suspended = fullscreen;
                log::info!(
                    "前台全屏应用: {} → 渲染{}",
                    if fullscreen { "检出" } else { "退出" },
                    if fullscreen { "暂停" } else { "恢复" }
                );
            }
        }
        let rate = super::frame_budget::decide(
            now.duration_since(self.last_activity),
            self.boost_until
                .map(|until| until.saturating_duration_since(now))
                .unwrap_or(Duration::ZERO),
            self.fullscreen_suspended,
        );
        if rate != self.frame_rate {
            log::info!("帧率档 {:?} → {:?}", self.frame_rate, rate);
            self.frame_rate = rate;
        }
        if rate != super::frame_budget::FrameRate::Suspended {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        ControlFlow::WaitUntil(now + super::frame_budget::poll_interval(rate))
    }

    /// 活动戳记 (Adaptive 帧率判定输入): 窗口输入/托盘热键动作/应用事件
    /// 都算活动。仅 Adaptive 模式戳 (其它模式零开销, 不读时钟)。
    /// 关联函数形态 (不借整个 self): 调用点可能正持有其它字段的借用
    /// (如 hotkey_rx 轮询循环), 字段级错位借用才能编译。
    fn note_activity(mode: super::WindowMode, last_activity: &mut Instant) {
        if mode == super::WindowMode::Adaptive {
            *last_activity = Instant::now();
        }
    }

    /// 关闭请求 (Alt+F4 / 标题栏关闭按钮) 的统一策略：
    /// `CloseBehavior::Hide` 常驻型应用只隐藏窗口; `CloseBehavior::Exit` 退出进程。
    fn handle_close_request(&mut self, event_loop: &ActiveEventLoop, source: &str) {
        match self.config.close_behavior {
            CloseBehavior::Hide => {
                log::info!("{source} → 隐藏");
                self.hide_window();
            }
            CloseBehavior::Exit => {
                log::info!("{source} → 退出进程");
                event_loop.exit();
            }
        }
    }

    /// 隐藏窗口：应用层 `is_visible` 与 OS 状态同步翻转。
    /// 状态切换的"动作"统一收口在此，减少 toggle / close / min 等路径的复制。
    fn hide_window(&mut self) {
        // is_visible 必须先于 set_visible 置位: Windows 隐藏后补发的幻影
        // WM_SIZE (160x28, 见 last_real_size 注释) 若以同步 SendMessage 语义
        // 派发, 会在 set_visible 调用内重入窗口过程 —— 此时 is_visible 必须
        // 已为 false, 隐藏态守卫才能拦住它 (CloseBehavior::Hide 路径依赖此序)。
        self.is_visible = false;
        // 隐藏即非前台: SW_HIDE 后 OS 焦点必然易手, 不等 winit 的 Focused(false)
        // 报到 —— 焦点消息失真的环境下 (AttachThreadInput 抢前台后遗症) 它可能
        // 永远不到, has_os_focus 残留 true 会让下次唤起时的对账补发被跳过
        // (2026-08-25 日志实锤: toggle 隐藏 → 残留 → 唤起无 Focused(true))。
        // 同步补发 WM_KILLFOCUS: 把 winit 的 is_focused 一并洗回 false,
        // 否则它残留 true 时下次对账补获得无法构成跳变 (同日志, 连按轮死锁)。
        // 若真实 Focused(false) 随后补到, 重复置 false 无副作用。
        self.has_os_focus = false;
        #[cfg(target_os = "windows")]
        if let Some(window) = &self.window {
            super::foreground::inject_focus_loss(window);
        }
        if let Some(window) = &self.window {
            window.set_visible(false);
        }
    }

    /// 最大化窗口并同步应用层最大化状态 (标题栏图标据此切换 □/□□)。
    /// 用于隐藏态阶段流转自动呼出时的"默认最大化"；手动 ToggleVisible 不走此路径。
    fn maximize_window(&mut self) {
        if let Some(window) = &self.window {
            window.set_maximized(true);
        }
        self.app.maximized_changed(true);
    }

    /// 显示窗口 + 抢焦点 + 重绘。winit 的 SW_SHOW 默认不抢焦点，
    /// 显式抢前台防止"已显示但被遮" (尤其在另一 app 后台时)。
    fn show_window(&self) {
        if let Some(window) = &self.window {
            // 显示落位: Cursor 策略下先把窗口挪到鼠标光标处 (热键唤起的面板
            // 贴手边, 不再恒居中)。最大化窗口不挪 (位置由系统管理)。
            if self.config.placement == super::ShowPlacement::Cursor && !window.is_maximized() {
                super::placement::move_to_cursor(window, self.last_real_size);
            }
            window.set_visible(true);
            // 自愈 winit 尺寸缓存: 隐藏期间若被幻影尺寸污染, 显示时强制回到
            // 最后可信尺寸, 触发真实 WM_SIZE 让 winit 内部状态恢复健康。
            // 最大化窗口不可经 SetWindowPos 改尺寸 (request_inner_size 会清
            // 最大化标志), 跳过 —— 其恢复依赖 SW_SHOW 还原最大化时系统补发
            // 真实 WM_SIZE (彼时 is_visible=true, 会被正常接受)。
            // 尺寸本就一致时 Windows 不会补发 WM_SIZE, 无副作用。
            if !window.is_maximized() && window.inner_size() != self.last_real_size {
                let _ = window.request_inner_size(self.last_real_size);
            }
            window.request_redraw();
            // Windows: winit 的 focus_window() 走合成 Alt + SetForegroundWindow,
            // 对后台常驻进程受前台锁限制而静默失败 —— 窗口"已显示但被遮"。
            // 用 foreground 模块的 AttachThreadInput 方案硬抢 (先直调, 失败再挂接);
            // 其它平台保留 winit 原生 focus_window。
            // 抢前台造成的 winit 焦点事件流失真由 about_to_wait 的每帧对账
            // (reconcile_focus_state) 统一修复, 此处不做即时补发。
            #[cfg(target_os = "windows")]
            super::foreground::bring_to_foreground(window);
            #[cfg(not(target_os = "windows"))]
            window.focus_window();
        }
    }

    /// 处理 App 经 WindowEventSender 主动发来的事件。
    /// 返回 true = 该事件改了托盘菜单勾选项相关状态 (调用方据此刻意重建菜单)。
    fn apply_window_event(&mut self, event: WindowAppEvent, event_loop: &ActiveEventLoop) -> bool {
        match event {
            WindowAppEvent::ToggleVisible => {
                // Handler 是 is_visible 唯一事实源：翻转后立即应用到 winit 窗口。
                self.is_visible = !self.is_visible;
                if self.is_visible {
                    self.show_window();
                } else {
                    self.hide_window();
                }
                self.app.visibility_changed(self.is_visible);
                false
            }
            WindowAppEvent::ShowWindow => {
                // 仅显示窗口 (不切换)。用于 focus_lost 等场景。
                if !self.is_visible {
                    self.is_visible = true;
                    self.show_window();
                    self.app.visibility_changed(true);
                }
                false
            }
            WindowAppEvent::HideWindow => {
                // 仅隐藏窗口 (不切换)。用于 focus_lost 和关闭按钮。
                if self.is_visible {
                    self.is_visible = false;
                    self.hide_window();
                    self.app.visibility_changed(false);
                }
                false
            }
            WindowAppEvent::Quit => {
                event_loop.exit();
                false
            }
            WindowAppEvent::PhaseAdvanced => {
                // 隐藏态时阶段流转 → 自动呼出 (用户可能没在电脑前，或在另一 app)，
                // 默认最大化呼出 (沉浸主界面)。手动 ToggleVisible 不强制最大化。
                if !self.is_visible {
                    self.is_visible = true;
                    log::info!("阶段流转，自动呼出窗口 (默认最大化)");
                    self.show_window();
                    self.maximize_window();
                }
                false
            }
            WindowAppEvent::SetClearColor(color) => {
                self.config.clear_color = color;
                if let Some(ctx) = self.context.as_mut() {
                    ctx.set_clear_color(color);
                }
                false
            }
            WindowAppEvent::BoostFrames(secs) => {
                // 事件升帧: 微事件播放期临时全帧率 (后发覆盖先到)。
                // 帧率判定在 adaptive_frame_pacing 消费 boost_until, 到期自动回落。
                self.boost_until = Some(Instant::now() + Duration::from_secs_f32(secs.max(0.0)));
                false
            }
            WindowAppEvent::SetClickThrough(enabled) => {
                // 穿透只改命中测试, 不动可见性与焦点; 底层实现幂等。
                // 窗口未创建时 (resumed 前) 丢弃 —— 调用方应经
                // visibility_changed 等回调确认窗口就绪后再发。
                if let Some(window) = &self.window {
                    super::passthrough::set_click_through(window, enabled);
                }
                true
            }
            WindowAppEvent::SetTopmost(topmost) => {
                self.config.topmost = topmost;
                if let Some(window) = &self.window {
                    window.set_window_level(if topmost {
                        WindowLevel::AlwaysOnTop
                    } else {
                        WindowLevel::Normal
                    });
                }
                true
            }
            WindowAppEvent::SetInnerSize(size) => {
                let Some(window) = &self.window else {
                    return true; // 窗口未创建时丢弃 (resumed 前)
                };
                if window.is_maximized() {
                    // request_inner_size 会清最大化标志 (见 show_window 注释),
                    // 最大化态的尺寸请求静默丢弃。
                    log::info!("最大化态忽略尺寸请求 {}x{}", size.width, size.height);
                    return true;
                }
                // 与创建路径同约定: 逻辑像素。
                let logical = LogicalSize::new(f64::from(size.width), f64::from(size.height));
                if self.is_visible {
                    // winit 异步生效 (Windows 上通常返回 None), 实际尺寸以随后的
                    // Resized 事件为准; 渲染表面经既有 Resized 流程自动跟随。
                    let _ = window.request_inner_size(logical);
                } else {
                    // 隐藏态不直接改窗口: Windows 会对隐藏窗口补发 WM_SIZE,
                    // 但隐藏态 Resized 一律不信 (幻影 160x28 防护), 显示时
                    // show_window 又以 last_real_size 自愈回滚 —— 窗口会弹回
                    // 旧尺寸而产品配置已变更 (三态不一致, desk-window review
                    // 实证)。只更新信标: 显示时自愈路径把窗口做到新尺寸并
                    // 触发真实 WM_SIZE (表面经 Resized 流程跟随)。
                    let physical: PhysicalSize<f64> = logical.to_physical(window.scale_factor());
                    self.last_real_size =
                        PhysicalSize::new(physical.width as u32, physical.height as u32);
                    log::info!(
                        "隐藏态尺寸请求记入信标 {}x{}, 显示时落位",
                        self.last_real_size.width,
                        self.last_real_size.height
                    );
                }
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::WindowMode;
    use super::{hotkey_swallow_filter, tab_traverse_allowed, vk_to_key_code};
    use winit::event::ElementState;
    use winit::keyboard::{KeyCode, PhysicalKey};

    /// 热键主键在守卫期间的按下被吞, 且守卫保持 (继续吞后续按下)。
    /// 焦点合成 (repeat=false) 与自动重复 (repeat=true) 两种来源在
    /// 过滤器入口处不作区分 —— 守卫期间的按下都不可能是主动输入。
    /// 回归: 「唤起后 v 漏进检索框, 列表被静默过滤」(2026-08-16 定位)。
    #[test]
    fn hotkey_key_press_is_swallowed_while_held() {
        let mut swallow = Some(KeyCode::KeyV);
        let v = PhysicalKey::Code(KeyCode::KeyV);
        assert!(hotkey_swallow_filter(
            &mut swallow,
            v,
            ElementState::Pressed
        ));
        assert!(swallow.is_some());
    }

    /// 主键抬起解除守卫, 之后的主动输入正常放行。
    #[test]
    fn hotkey_key_release_lifts_swallow() {
        let mut swallow = Some(KeyCode::KeyV);
        let v = PhysicalKey::Code(KeyCode::KeyV);
        assert!(!hotkey_swallow_filter(
            &mut swallow,
            v,
            ElementState::Released
        ));
        assert!(swallow.is_none());
        assert!(!hotkey_swallow_filter(
            &mut swallow,
            v,
            ElementState::Pressed
        ));
    }

    /// 非热键主键的按下/重复 (如长按 Backspace 连删) 不受影响。
    #[test]
    fn other_key_press_is_not_swallowed() {
        let mut swallow = Some(KeyCode::KeyV);
        let backspace = PhysicalKey::Code(KeyCode::Backspace);
        assert!(!hotkey_swallow_filter(
            &mut swallow,
            backspace,
            ElementState::Pressed
        ));
        assert!(swallow.is_some());
    }

    /// 字母/数字 vk 映射; 未覆盖的 vk 不吞。
    #[test]
    fn vk_maps_letters_and_digits() {
        assert_eq!(vk_to_key_code(0x56), Some(KeyCode::KeyV));
        assert_eq!(vk_to_key_code(0x41), Some(KeyCode::KeyA));
        assert_eq!(vk_to_key_code(0x30), Some(KeyCode::Digit0));
        assert_eq!(vk_to_key_code(0x39), Some(KeyCode::Digit9));
        assert_eq!(vk_to_key_code(0x01), None);
    }

    #[test]
    fn tab_without_os_focus_is_suppressed() {
        // Alt+Tab 泄漏的 Tab 在 Focused(true) 之前到达 (has_os_focus=false),
        // 必须拦截, 否则焦点被切到下一个组件 (2026-07-29 实测回归)。
        assert!(!tab_traverse_allowed(false));
    }

    #[test]
    fn tab_with_os_focus_traverses() {
        // 持有 OS 焦点期间的 Tab 是用户主动遍历, 必须放行。
        assert!(tab_traverse_allowed(true));
    }

    /// 按需渲染模式: 隐藏态应使用 Wait (零唤醒)。
    #[test]
    fn on_demand_hidden_uses_wait() {
        let control_flow = control_flow_for_mode(WindowMode::OnDemand, false, false);
        assert!(
            matches!(control_flow, winit::event_loop::ControlFlow::Wait),
            "OnDemand 隐藏态应使用 ControlFlow::Wait"
        );
    }

    /// 按需渲染模式: 可见且无动画应使用 WaitUntil (事件驱动重绘)。
    #[test]
    fn on_demand_visible_no_animation_uses_wait_until() {
        let control_flow = control_flow_for_mode(WindowMode::OnDemand, true, false);
        assert!(
            matches!(control_flow, winit::event_loop::ControlFlow::WaitUntil(_)),
            "OnDemand 可见无动画应使用 ControlFlow::WaitUntil"
        );
    }

    /// 按需渲染模式: 可见且有动画应使用 WaitUntil (持续渲染)。
    #[test]
    fn on_demand_visible_with_animation_uses_wait_until() {
        let control_flow = control_flow_for_mode(WindowMode::OnDemand, true, true);
        assert!(
            matches!(control_flow, winit::event_loop::ControlFlow::WaitUntil(_)),
            "OnDemand 可见有动画应使用 ControlFlow::WaitUntil"
        );
    }

    /// 持续渲染模式: 隐藏态仍使用 WaitUntil (番茄钟等需要持续 tick)。
    #[test]
    fn continuous_hidden_uses_wait_until() {
        let control_flow = control_flow_for_mode(WindowMode::Continuous, false, false);
        assert!(
            matches!(control_flow, winit::event_loop::ControlFlow::WaitUntil(_)),
            "Continuous 隐藏态应使用 ControlFlow::WaitUntil (保持 tick)"
        );
    }

    /// 持续渲染模式: 可见态使用 WaitUntil (原有行为)。
    #[test]
    fn continuous_visible_uses_wait_until() {
        let control_flow = control_flow_for_mode(WindowMode::Continuous, true, false);
        assert!(
            matches!(control_flow, winit::event_loop::ControlFlow::WaitUntil(_)),
            "Continuous 可见态应使用 ControlFlow::WaitUntil"
        );
    }

    /// 根据窗口模式和可见性决定控制流。
    ///
    /// - `OnDemand` + 隐藏 → `Wait` (零唤醒, 省电)
    /// - `OnDemand` + 可见 → `WaitUntil(16ms)` (事件驱动重绘, ~60fps)
    /// - `Continuous` (任何状态) → `WaitUntil(16ms)` (番茄钟等需持续 tick)
    fn control_flow_for_mode(
        mode: WindowMode,
        is_visible: bool,
        has_animation: bool,
    ) -> winit::event_loop::ControlFlow {
        use std::time::{Duration, Instant};
        use winit::event_loop::ControlFlow;

        match mode {
            WindowMode::OnDemand if !is_visible && !has_animation => ControlFlow::Wait,
            _ => ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)),
        }
    }
}
