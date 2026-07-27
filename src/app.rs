//! @author 十四叔
//! @date 2026/07/17

//! 应用层:App trait —— 状态、消息、视图树。
//!
//! 数据流：组件事件 (如按钮点击) 产出消息 `Msg` → [`App::update`]
//! 修改状态 → 每帧 `sync` 把状态经绑定闭包同步进保留的组件树。

use std::any::Any;
use std::time::{Duration, Instant};

use crate::event::Event;
use crate::render::BackgroundFrame;
use crate::widget::Node;
use crate::window::WindowEventSender;
use tray_icon::menu::Menu;

/// 动画 / 时间上下文，由框架每帧传入 `Widget::animate`。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnimationCtx {
    /// 当前绝对时间。
    pub now: Instant,
    /// 自应用启动以来的累计时间。
    pub elapsed: Duration,
}

impl AnimationCtx {
    /// 构造动画上下文。
    pub fn new(now: Instant, elapsed: Duration) -> Self {
        Self { now, elapsed }
    }
}

/// 应用：状态容器 + 消息更新 + 视图树构建。
///
/// 实现者即为状态本体 (组件绑定闭包经 `Any` 向下转型读取状态)。
pub trait App: Any {
    /// 消息类型：组件事件产出，驱动状态变化。
    type Msg: 'static;

    /// 消息处理：修改应用状态。
    fn update(&mut self, msg: Self::Msg);

    /// 构建组件树 (启动时调用一次; 数据经绑定闭包每帧同步)。
    fn view(&self) -> Node;

    /// 原始事件钩子：键盘事件直送应用层 (M1 无焦点系统);
    /// 未被组件树消费的鼠标事件也会到达这里。
    fn event(&mut self, _event: &Event) {}

    /// 每帧心跳：在 `sync` 之前调用，驱动计时 / 过渡动画等时间相关状态。
    ///
    /// 默认空实现;需要逐帧推进状态的应用 (如番茄钟) 覆盖之。
    fn tick(&mut self, _ctx: &AnimationCtx) {}

    /// 每帧背景状态：场景选择 / 淡化进度 / 清屏色。
    ///
    /// 默认 `None` —— 保持 `BackgroundConfig` 初始化时的静态背景
    /// (showcase 等单背景应用无需实现)。窗口在每帧渲染前查询并写入渲染上下文。
    fn background_frame(&self) -> Option<BackgroundFrame> {
        None
    }

    /// 启动时 elapsed 偏移：默认 0 (全新会话), 持久化恢复场景下返回
    /// `effective_now` (= saved_elapsed + 跨重启 wall-clock 漂移)。
    /// 窗口在 `resumed` 时用 `Instant::now() - offset` 重置 `start`,
    /// 使得 `AnimationCtx::elapsed` 从恢复点继续，而不是 0 开始。
    fn boot_elapsed_offset(&self) -> Duration {
        Duration::ZERO
    }

    /// 注入窗口事件发送器 (App 主动控制窗口：显隐 / 全局热键退出等)。
    /// 默认空实现：不需要窗口控制的应用无需关心。
    /// `run_app` 启动时调用一次，在 `resumed` 之前。
    fn attach_window_sender(&mut self, _sender: WindowEventSender) {}

    /// 全局热键 ID -> 应用消息映射。返回 `None` 表示忽略该 ID。
    /// 默认空实现：应用不响应全局热键。
    /// 常量见 [`crate::window::hotkey_ids`]。
    fn hotkey(&mut self, _id: u8) -> Option<Self::Msg> {
        None
    }

    /// 系统托盘菜单项 ID -> 应用消息映射。返回 `None` 表示忽略该 ID。
    /// 默认空实现：应用不响应托盘菜单。
    /// 常量见 [`crate::window::tray_action_ids`], 与 `hotkey_ids` 编号独立但语义对齐。
    fn tray_action(&mut self, _id: u8) -> Option<Self::Msg> {
        None
    }

    /// 构建系统托盘菜单 (右键托盘图标弹出)。默认返回空菜单。
    /// `run_app` 启动时调用一次，移动到 TrayIcon 内部。返回的 Menu 由调用方构建，
    /// 不应在调用方持有引用 (TrayIcon 接管所有权)。
    fn tray_menu(&self) -> Menu {
        Menu::new()
    }
}
