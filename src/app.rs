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

    /// 每帧背景状态: 场景选择 / 淡化进度 / 清屏色。
    ///
    /// 默认 `None` —— 保持 `BackgroundConfig` 初始化时的静态背景
    /// (showcase 等单背景应用无需实现)。窗口在每帧渲染前查询并写入渲染上下文。
    fn background_frame(&self) -> Option<BackgroundFrame> {
        None
    }
}
