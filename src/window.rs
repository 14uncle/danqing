// ! @author 十四叔
// ! @date 2026/07/17

// ! 窗口与事件循环封装 (winit 平台适配层)。
// !
// ! 本模块是唯一允许接触 OS 窗口 API 的地方: 负责窗口创建、
// ! 事件循环驱动, 并把 winit 事件转换为平台无关的内部事件。

use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Ime as WinitIme, MouseButton as WinitMouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey as WinitNamedKey},
    window::{Window as WinitWindow, WindowAttributes, WindowId},
};

use crate::app::{AnimationCtx, App};
use crate::event::{Event, ImeEvent, Key, MouseButton, NamedKey};
use crate::render::{Context, RectBatch, TextBatch};
use crate::widget::{
    FocusManager, MsgQueue, Node, event_at_path, ime_area_at_path, selected_text_at_path,
    wants_ime_at_path,
};
use crate::{Color, Point, Rect, Size};

/// 窗口 / 事件循环相关错误。
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    /// 事件循环创建或运行失败。
    #[error("事件循环错误: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    /// 窗口创建失败。
    #[error("创建窗口失败: {0}")]
    Os(#[from] winit::error::OsError),
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
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "danqing showcase".into(),
            size: Size::new(1280.0, 800.0),
            // 深蓝灰: 非常量黑 / 白, 用于验证颜色参数通路
            clear_color: Color::rgb(0.10, 0.16, 0.24),
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

/// winit 应用处理器, 驱动窗口生命周期与事件分发。
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
    /// 根矩形 (事件命中用, 每帧布局后更新)。
    root_area: Rect,
    /// 焦点管理器。
    focus: FocusManager,
    /// 应用启动时间 (用于动画)。
    start: Instant,
    /// 系统剪贴板 (懒加载)。
    clipboard: Option<arboard::Clipboard>,
    /// 是否已完成首帧渲染(用于一次性诊断计时)。
    first_frame_done: bool,
}

impl<A: App> Handler<'_, A> {
    /// 记录一条窗口事件到日志。
    fn log_event(event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                log::debug!("鼠标移动: ({:.0}, {:.0})", position.x, position.y);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                log::info!("鼠标按键: {button:?} {state:?}");
            }
            WindowEvent::MouseWheel { delta, .. } => {
                log::info!("滚轮: {delta:?}");
            }
            WindowEvent::KeyboardInput { event, .. } => {
                log::info!("键盘: {:?} {:?}", event.logical_key, event.state);
            }
            WindowEvent::Ime(ime) => {
                log::info!("IME: {ime:?}");
            }
            WindowEvent::ModifiersChanged(mods) => {
                log::debug!("修饰键: {mods:?}");
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
        let Some(path) = self.focus.current().map(|p| p.to_vec()) else {
            // 无焦点时回退到应用层
            self.app.event(event);
            return;
        };

        match event {
            Event::Key { key, pressed, .. } if *pressed => {
                match key {
                    Key::Named(NamedKey::Tab) => {
                        if self.modifiers.shift_key() {
                            self.focus.prev();
                        } else {
                            self.focus.next();
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
                Err(err) => log::warn!("初始化剪贴板失败: {err}"),
            }
        }
        self.clipboard.as_mut()
    }

    fn set_clipboard(&mut self, text: String) {
        if let Some(cb) = self.clipboard() {
            if let Err(err) = cb.set_text(text) {
                log::warn!("写入剪贴板失败: {err}");
            }
        }
    }

    fn get_clipboard(&mut self) -> Option<String> {
        self.clipboard().map(|cb| cb.get_text().unwrap_or_default())
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
        let attrs = WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(
                f64::from(self.config.size.width),
                f64::from(self.config.size.height),
            ));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                log::error!("创建窗口失败: {err}");
                event_loop.exit();
                return;
            }
        };
        log::info!("窗口已创建: {}", self.config.title);

        let ctx_start = Instant::now();
        match Context::new(Arc::clone(&window), self.config.clear_color) {
            Ok(context) => {
                self.context = Some(context);
                log::info!("渲染上下文初始化耗时: {:?}", ctx_start.elapsed());
            }
            Err(err) => {
                log::error!("初始化渲染上下文失败: {err}");
                event_loop.exit();
                return;
            }
        }
        self.window = Some(window);
        // 持续渲染模式: 请求首帧, 之后每帧结束再请求下一帧
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

        // 鼠标事件经组件树命中分发, 分发后可能更新焦点
        if matches!(
            event,
            WindowEvent::CursorMoved { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::MouseWheel { .. }
        ) {
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
        for boxed in self.msgs.drain(..) {
            match boxed.downcast::<A::Msg>() {
                Ok(msg) => self.app.update(*msg),
                Err(_) => log::warn!("丢弃类型不匹配的消息"),
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                log::info!("收到关闭请求,退出事件循环");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                log::info!("窗口尺寸变化: {}x{}", size.width, size.height);
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
                    self.tree.sync(self.app);
                    let ctx = AnimationCtx::new(Instant::now(), self.start.elapsed());
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
                    self.update_ime();
                }
                if let Some(context) = &mut self.context
                    && !context.render(&rects, &mut self.texts)
                {
                    event_loop.exit();
                    return;
                }
                if !self.first_frame_done {
                    self.first_frame_done = true;
                    log::info!("首帧渲染耗时: {:?}", frame_start.elapsed());
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// 打开窗口并运行应用: 事件分发、消息驱动、每帧重绘, 直到窗口关闭。
pub fn run_app<A: App>(config: WindowConfig, app: &mut A) -> Result<(), WindowError> {
    let event_loop = EventLoop::new()?;
    let texts_start = Instant::now();
    let texts = TextBatch::new();
    log::info!(
        "文本批次初始化(含字体加载)耗时: {:?}",
        texts_start.elapsed()
    );
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
    };
    let run_start = Instant::now();
    event_loop.run_app(&mut handler)?;
    log::info!("事件循环运行耗时: {:?}", run_start.elapsed());
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
    use super::*;

    /// 冒烟测试: 仅创建事件循环 (链接触发 shim 生成的导入库)。
    /// 若导入库损坏, 本测试会以访问违规崩溃。
    #[test]
    fn event_loop_creation_smoke() {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        let event_loop = EventLoop::builder().with_any_thread(true).build();
        drop(event_loop.expect("创建事件循环失败"));
    }
}
