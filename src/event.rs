//! @author 十四叔
//! @date 2026/07/17

//! 事件：平台无关的内部事件类型与分发语义。
//!
//! 本模块为纯逻辑，不依赖 winit;winit 事件到内部事件的
//! 转换发生在平台适配层 (window.rs)。
//!
//! 分发语义 (M1):
//! - `CursorMoved`: 广播全树，各组件自行判定 hover;
//! - `MouseInput`/`MouseWheel`: 沿命中路径分发 (后绘制者优先);
//! - 键盘：不送组件树，直送应用层 (M1 无焦点系统)。

use crate::Point;

/// 鼠标按键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// 左键。
    Left,
    /// 右键。
    Right,
    /// 中键 (滚轮按下)。
    Middle,
    /// 侧键 (后退)。
    Back,
    /// 侧键 (前进)。
    Forward,
    /// 其他按键。
    Other(u16),
}

/// 逻辑按键 (M1: 字符 + 常用具名键)。
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    /// 字符键 (已按布局解析的文本)。
    Character(String),
    /// 具名功能键。
    Named(NamedKey),
}

/// 具名功能键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    /// 方向键。
    ArrowUp,
    /// 方向键。
    ArrowDown,
    /// 方向键。
    ArrowLeft,
    /// 方向键。
    ArrowRight,
    /// 空格。
    Space,
    /// 回车。
    Enter,
    /// Esc。
    Escape,
    /// Tab。
    Tab,
    /// 退格。
    Backspace,
    /// 删除。
    Delete,
    /// Home。
    Home,
    /// End。
    End,
    /// Shift。
    Shift,
    /// Ctrl。
    Control,
    /// Alt。
    Alt,
}

/// IME 事件。
#[derive(Debug, Clone, PartialEq)]
pub enum ImeEvent {
    /// IME 已启用 (开始合成)。
    Enabled,
    /// IME 已禁用 (合成结束)。
    Disabled,
    /// 合成中文本。
    Preedit {
        /// 合成字符串。
        value: String,
        /// 光标在合成字符串中的位置 (起始，结束),None 表示无特定位置。
        cursor: Option<(usize, usize)>,
    },
    /// 合成提交 (最终文本)。
    Commit {
        /// 提交的字符串。
        value: String,
    },
}

/// 内部事件 (平台无关)。
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// 鼠标移动 (窗口逻辑坐标)。
    CursorMoved(Point),
    /// 鼠标离开窗口。
    CursorLeft,
    /// 鼠标按键按下 / 抬起。
    MouseInput {
        /// 按键。
        button: MouseButton,
        /// true = 按下，false = 抬起。
        pressed: bool,
        /// 事件发生时光标位置。
        position: Point,
    },
    /// 滚轮滚动。
    MouseWheel {
        /// 滚动量 (行或像素，平台归一后)。
        delta: (f32, f32),
        /// 事件发生时光标位置。
        position: Point,
    },
    /// 键盘按下 / 抬起。
    Key {
        /// 逻辑键。
        key: Key,
        /// true = 按下，false = 抬起。
        pressed: bool,
        /// Shift 是否按下。
        shift: bool,
        /// Ctrl 是否按下。
        ctrl: bool,
        /// Alt 是否按下。
        alt: bool,
    },
    /// IME 合成事件。
    Ime(ImeEvent),
    /// 复制请求 (焦点组件应通过 `Widget::selected_text` 提供文本)。
    Copy,
    /// 剪切请求 (焦点组件应通过 `Widget::selected_text` 提供文本，然后删除选区)。
    Cut,
    /// 粘贴请求 (系统剪贴板文本将通过 `Event::Ime(Commit)` 送达 )。
    Paste,
    /// 当前组件获得焦点。
    FocusIn,
    /// 当前组件失去焦点。
    FocusOut,
}

impl Event {
    /// 鼠标类事件的光标位置 (非鼠标事件返回 None)。
    pub fn position(&self) -> Option<Point> {
        match self {
            Event::CursorMoved(p) => Some(*p),
            Event::MouseInput { position, .. } => Some(*position),
            Event::MouseWheel { position, .. } => Some(*position),
            _ => None,
        }
    }
}

/// 窗口控制动作。
///
/// 由自绘标题栏等组件产出，经 `window.rs` 的 `Handler` 识别后调用 OS 窗口 API。
/// 保持纯逻辑，不依赖 `winit`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAction {
    /// 关闭窗口。
    Close,
    /// 最小化窗口。
    Minimize,
    /// 最大化或还原窗口。
    MaximizeOrRestore,
    /// 开始拖拽移动窗口。
    Drag,
}
