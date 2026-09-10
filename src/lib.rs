//! @author 十四叔
//! @date 2026/07/17

//! 丹青 (danqing) —— 跨平台自绘 UI 框架。
//!
//! 保留模式组件树 + wgpu 自绘管线。M1 最小闭环：
//! 跨平台开窗、基础图元/文本绘制、键鼠事件响应。
//!
//! 公开 API 一律经本模块 re-export，不允许使用者路径深穿。

mod anim;
mod app;
pub mod asset;
pub mod audio;
pub mod event;
mod job;
pub mod layout;
pub mod log;
/// 应用持久化核心 (config_dir/atomic_save/load_or_default/DirtyFlag/VersionedDoc)。
/// 仅 `persist` feature 启用: 默认关闭不拉序列化栈, 产品显式开启。
#[cfg(feature = "persist")]
pub mod persist;
mod render;
mod text;
pub mod theme;
/// 应用内更新检查核心 (版本对比/缓存/后台检查/GitHub 运输)。
/// 仅 `update` feature 启用: 默认关闭不拉网络栈, 产品显式开启。
#[cfg(feature = "update")]
pub mod update;
pub mod widget;
mod window;

pub use anim::{Crossfade, Cue, CueTiming, Pulse, Tween};
pub use app::{AnimationCtx, App};
pub use event::{Event, ImeEvent, Key, MouseButton, NamedKey, WindowAction};
pub use job::{AsyncJob, CancelFlag, CancelToken, SearchNav};
pub use layout::{Color, Constraints, Edges, FlowChild, Point, Rect, Size, distribute};
pub use render::{
    BackgroundConfig, BackgroundFrame, Context as RenderContext, ImageBatch, RectBatch,
    RenderError, ScaleMode, TextBatch,
};
pub use text::{AtlasError, Font, FontError, GlyphAtlas, GlyphInfo, Line, break_lines, to_crlf};
pub use text::{encoding, fit, selection};
pub use theme::{
    Easing, LightTheme, ScenePalette, SceneSpec, SceneTheme, Shadow, Theme, composite_over,
    contrast_ratio, relative_luminance,
};
pub use window::tray::TrayHandle;
pub use window::{
    CloseBehavior, GlobalHotkey, ShowPlacement, WindowAppEvent, WindowConfig, WindowError,
    WindowEventSender, WindowMode, foreground, hotkey_ids, run, run_app, shortcut_for_id, startup,
    tray, tray_action_ids,
};
// 托盘子模块：re-export tray-icon (含 menu), 供例子构建菜单使用。
pub use ::tray_icon;
