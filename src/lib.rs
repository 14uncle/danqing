//! @author 十四叔
//! @date 2026/07/17

//! 丹青 (danqing) —— 跨平台自绘 UI 框架。
//!
//! 保留模式组件树 + wgpu 自绘管线。M1 最小闭环：
//! 跨平台开窗、基础图元/文本绘制、键鼠事件响应。
//!
//! 公开 API 一律经本模块 re-export，不允许使用者路径深穿。

mod app;
pub mod event;
pub mod layout;
mod render;
mod text;
pub mod theme;
pub mod widget;
mod window;

pub use app::{AnimationCtx, App};
pub use event::{Event, ImeEvent, Key, MouseButton, NamedKey, WindowAction};
pub use layout::{Color, Constraints, Edges, FlowChild, Point, Rect, Size, distribute};
pub use render::{
    BackgroundConfig, BackgroundFrame, Context as RenderContext, RectBatch, RenderError, ScaleMode,
    TextBatch,
};
pub use text::{AtlasError, Font, FontError, GlyphAtlas, GlyphInfo, Line, break_lines};
pub use theme::{
    Easing, LightTheme, ScenePalette, SceneSpec, SceneTheme, Shadow, Theme, composite_over,
    contrast_ratio, relative_luminance,
};
pub use window::tray::TrayHandle;
pub use window::{
    WindowAppEvent, WindowConfig, WindowError, WindowEventSender, hotkey_ids, run, run_app, tray,
    tray_action_ids,
};
