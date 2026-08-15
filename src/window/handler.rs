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
    dpi::{LogicalPosition, LogicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::ModifiersState,
    window::{Window as WinitWindow, WindowAttributes, WindowId},
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
    /// 图像纹理收集器 (每帧清空，paint 阶段填充)。
    images: ImageBatch,
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
            images: ImageBatch::new(),
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
        let screen = self.window.as_ref().map(|w| {
            let size = w.inner_size();
            Size::new(size.width as f32, size.height as f32)
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
            window.request_redraw();
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
        // 先持有窗口引用: 预渲染首帧需要查询 inner_size。
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
            // 窗口焦点变化时通知应用层 (用于失焦自动隐藏等行为)。
            if gained {
                self.app.focus_gained();
            } else {
                self.app.focus_lost();
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
                self.handle_close_request(event_loop, "收到关闭请求");
            }
            WindowEvent::Resized(size) => {
                log::info!("窗口尺寸变化：{}x{}", size.width, size.height);
                if let Some(context) = &mut self.context {
                    context.resize(size.width, size.height);
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
        if let Some(rx) = &self.hotkey_rx {
            while let Ok(id) = rx.try_recv() {
                if let Some(msg) = self.app.hotkey(id) {
                    self.app.update(msg);
                }
            }
        }
        // 托盘菜单事件轮询 (muda 内部维护的全局通道)。每个 MenuId 是字符串
        // 包装 (`MenuId(pub String)`); 我们约定菜单项 id 用 ASCII 数字 ("1"/"2"/"3"),
        // 解析成 u8 后转交 `app.tray_action`。非法 id 静默忽略。
        let tray_rx = tray_icon::menu::MenuEvent::receiver();
        while let Ok(event) = tray_rx.try_recv() {
            if let Ok(id) = event.id.0.parse::<u8>() {
                if let Some(msg) = self.app.tray_action(id) {
                    self.app.update(msg);
                }
            }
        }
        // 窗口事件通道轮询
        while let Ok(event) = self.window_event_rx.try_recv() {
            self.apply_window_event(event, event_loop);
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
        let control_flow = self.control_flow_for_current_state();
        if self.is_visible {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        event_loop.set_control_flow(control_flow);
    }
}

impl<A: App> Handler<'_, A> {
    /// 根据当前窗口模式和可见性决定控制流。
    ///
    /// - `OnDemand` + 隐藏 → `Wait` (零唤醒, 省电)
    /// - `OnDemand` + 可见 → `WaitUntil(16ms)` (事件驱动重绘, ~60fps)
    /// - `Continuous` (任何状态) → `WaitUntil(16ms)` (番茄钟等需持续 tick)
    fn control_flow_for_current_state(&self) -> ControlFlow {
        match self.config.mode {
            super::WindowMode::OnDemand if !self.is_visible => ControlFlow::Wait,
            _ => ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)),
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
        if let Some(window) = &self.window {
            window.set_visible(false);
        }
        self.is_visible = false;
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
            window.set_visible(true);
            window.request_redraw();
            // Windows: winit 的 focus_window() 走合成 Alt + SetForegroundWindow,
            // 对后台常驻进程受前台锁限制而静默失败 —— 窗口"已显示但被遮"。
            // 用 foreground 模块的 AttachThreadInput 方案硬抢 (先直调, 失败再挂接);
            // 其它平台保留 winit 原生 focus_window。
            #[cfg(target_os = "windows")]
            super::foreground::bring_to_foreground(window);
            #[cfg(not(target_os = "windows"))]
            window.focus_window();
        }
    }

    /// 处理 App 经 WindowEventSender 主动发来的事件。
    fn apply_window_event(&mut self, event: WindowAppEvent, event_loop: &ActiveEventLoop) {
        match event {
            WindowAppEvent::ToggleVisible => {
                // Handler 是 is_visible 唯一事实源：翻转后立即应用到 winit 窗口。
                self.is_visible = !self.is_visible;
                if self.is_visible {
                    self.show_window();
                } else {
                    self.hide_window();
                }
            }
            WindowAppEvent::Quit => event_loop.exit(),
            WindowAppEvent::PhaseAdvanced => {
                // 隐藏态时阶段流转 → 自动呼出 (用户可能没在电脑前，或在另一 app)，
                // 默认最大化呼出 (沉浸主界面)。手动 ToggleVisible 不强制最大化。
                if !self.is_visible {
                    self.is_visible = true;
                    log::info!("阶段流转，自动呼出窗口 (默认最大化)");
                    self.show_window();
                    self.maximize_window();
                }
            }
            WindowAppEvent::SetClearColor(color) => {
                self.config.clear_color = color;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::WindowMode;
    use super::tab_traverse_allowed;

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
