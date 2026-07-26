//! @author 十四叔
//! @date 2026/07/17

//! 窗口与事件循环封装 (winit 平台适配层)。
//!
//! 本模块是唯一允许接触 OS 窗口 API 的地方:负责窗口创建、事件循环驱动,
//! 并把 winit 事件转换为平台无关的内部事件。
//!
//! 子模块:
//! - `event`   应用 → Handler 事件通道 + winit → 内部事件适配
//! - `icon`    窗口 / 托盘图标加载 + Windows 无边框样式
//! - `hotkey`  全局热键 ID 常量 + Windows 注册线程
//! - `tray`    托盘菜单项 ID + 快捷键 label 单一来源 + 跨平台托盘
//! - `handler` ApplicationHandler 实现(本模块最大, 单独拆出)

mod event;
mod handler;
mod hotkey;
mod icon;
pub mod tray;

use std::sync::mpsc::channel;
use std::time::Instant;

use winit::{event_loop::EventLoop, keyboard::ModifiersState};

use crate::app::App;
use crate::render::{BackgroundConfig, GpuDevice, TextBatch};
use crate::widget::{FocusManager, MsgQueue, Node};
use crate::{Color, Point, Rect, Size};

pub use event::{WindowAppEvent, WindowEventSender};
pub use hotkey::hotkey_ids;
pub use tray::tray_action_ids;
#[allow(unused_imports)]
pub use tray::{TrayHandle, shortcut_for_id};

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
            // 深蓝灰:非常量黑 / 白,用于验证颜色参数通路
            clear_color: Color::rgb(0.10, 0.16, 0.24),
            background: BackgroundConfig::default(),
            // 浅灰边框,与浅色毛玻璃主题协调
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.12),
            border_radius: 12.0,
            border_thickness: 1.0,
        }
    }
}

/// 打开窗口并运行应用:事件分发、消息驱动、每帧重绘,直到窗口关闭。
pub fn run_app<A: App>(config: WindowConfig, app: &mut A) -> Result<(), WindowError> {
    use handler::Handler;
    use hotkey::hotkeys;
    use icon::load_tray_icon;

    let boot = Instant::now();
    let event_loop = EventLoop::new()?;
    // 提前在后台线程创建 GPU 设备 (实例 + 适配器 + 逻辑设备):该过程不依赖
    // 窗口,与随后的字体加载 / 建窗串行工作重叠,缩短启动到可见的耗时。
    let gpu_handle = std::thread::spawn(GpuDevice::new);
    let texts_start = Instant::now();
    let texts = TextBatch::new();
    log::info!(
        "文本批次初始化 (含字体加载) 耗时：{:?}",
        texts_start.elapsed()
    );
    // 注入窗口事件发送器 (App 主动控制窗口:显隐 / 退出)
    let (window_event_tx, window_event_rx) = channel();
    app.attach_window_sender(WindowEventSender {
        sender: window_event_tx,
    });
    // 启动全局热键监听线程 (None 表示平台不支持)
    let hotkey_rx = hotkeys::spawn().map(|(rx, _handle)| rx);
    // 安装系统托盘 (图标 + 菜单)。load_tray_icon 失败则降级到无托盘。
    let tray = load_tray_icon().and_then(|icon| {
        let menu = app.tray_menu();
        tray::install_tray(icon, menu)
    });
    let tree = app.view();
    let mut handler = Handler {
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
        gpu_handle: Some(gpu_handle),
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
    use super::*;

    /// 冒烟测试:仅创建事件循环 (链接触发 shim 生成的导入库)。
    /// 若导入库损坏,本测试会以访问违规崩溃。
    #[test]
    fn event_loop_creation_smoke() {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        let event_loop = EventLoop::builder().with_any_thread(true).build();
        drop(event_loop.expect("创建事件循环失败"));
    }

    /// 单一来源契约:三个共享 ID 在 hotkey_ids / tray_action_ids 两套下
    /// 都必须返回同一个 label,任何一组漏改都会立刻被这条测试发现。
    #[test]
    fn shortcut_for_id_returns_consistent_label_across_id_sets() {
        assert_eq!(shortcut_for_id(hotkey_ids::TOGGLE_VISIBLE), "Ctrl+Shift+P");
        assert_eq!(
            shortcut_for_id(tray_action_ids::TOGGLE_VISIBLE),
            "Ctrl+Shift+P"
        );
        assert_eq!(shortcut_for_id(hotkey_ids::START_PAUSE), "Ctrl+Shift+S");
        assert_eq!(
            shortcut_for_id(tray_action_ids::START_PAUSE),
            "Ctrl+Shift+S"
        );
        assert_eq!(shortcut_for_id(hotkey_ids::QUIT), "Ctrl+Shift+Q");
        assert_eq!(shortcut_for_id(tray_action_ids::QUIT), "Ctrl+Shift+Q");
    }

    /// Release build 下未知 id 静默返空 (供调用方容错);
    /// debug build 下 `shortcut_for_id` 内的 `debug_assert!` 会先 panic, 提示加新 ID 时漏改。
    #[test]
    #[cfg(not(debug_assertions))]
    fn shortcut_for_id_unknown_id_returns_empty() {
        assert_eq!(shortcut_for_id(0), "");
        assert_eq!(shortcut_for_id(99), "");
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
