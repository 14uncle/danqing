//! @author 十四叔
//! @date 2026/07/26

//! 系统托盘子系统。
//!
//! - `tray_action_ids`: 托盘菜单项 ID 常量(语义与 `hotkey_ids` 一一对应,
//!   独立编号便于菜单项与热键解耦后调整)
//! - `shortcut_for_id`: 单一来源, 供 UI 展示快捷键 label(托盘菜单右侧 +
//!   首次启动 hint), 双检 hotkey/tray id 防漂移
//! - `TrayHandle` + `install_tray`: 跨平台托盘安装(Windows / macOS 走
//!   `tray-icon`; 其他平台 stub)

use crate::window::hotkey_ids;

/// 系统托盘菜单项 ID 常量 (语义与 `hotkey_ids` 一一对应, 独立编号便于
/// 菜单项与热键解耦后调整)。
pub mod tray_action_ids {
    /// 显隐窗口 (托盘菜单项)。
    pub const TOGGLE_VISIBLE: u8 = 1;
    /// 开始/暂停番茄钟 (托盘菜单项)。
    pub const START_PAUSE: u8 = 2;
    /// 退出应用 (托盘菜单项)。
    pub const QUIT: u8 = 3;
}

/// 快捷键 label 字符串 (供 UI 展示: 托盘菜单右侧 + 首次启动 hint)。
///
/// 单一来源: 注册用了 P/S/Q, 展示也用 P/S/Q, 杜绝字符串漂移。
/// 与 [`hotkey_ids`] / [`tray_action_ids`] 同居本模块, 框架保证三者同步。
/// 即使两组 ID 当前数值一致, 仍显式双检以防未来解耦时漏改。
///
/// debug build 下未知 id 触发 `debug_assert!`, release 下静默返 `""`。
/// 加新 ID 时必须同时更新本表 + hotkey/tray id 两侧常量 + 本表断言条件,
/// 否则测试会立刻爆。
pub fn shortcut_for_id(id: u8) -> &'static str {
    debug_assert!(
        id == hotkey_ids::TOGGLE_VISIBLE
            || id == tray_action_ids::TOGGLE_VISIBLE
            || id == hotkey_ids::START_PAUSE
            || id == tray_action_ids::START_PAUSE
            || id == hotkey_ids::QUIT
            || id == tray_action_ids::QUIT,
        "shortcut_for_id: id {id} 未在映射表中 (新加 ID 必须同时更新此函数)"
    );
    if id == hotkey_ids::TOGGLE_VISIBLE || id == tray_action_ids::TOGGLE_VISIBLE {
        "Ctrl+Shift+P"
    } else if id == hotkey_ids::START_PAUSE || id == tray_action_ids::START_PAUSE {
        "Ctrl+Shift+S"
    } else if id == hotkey_ids::QUIT || id == tray_action_ids::QUIT {
        "Ctrl+Shift+Q"
    } else {
        ""
    }
}

// TrayHandle + install_tray: 跨平台托盘实现。`mod tray` 提到 tray.rs 顶层避免
// 跟父模块同名冲突。

/// 托盘生命周期句柄。持有底层 `tray-icon::TrayIcon`, drop 时 tray-icon 内部清理
/// 并移除系统托盘图标。Handler 在 `run_app` 期间持有 Handle, 退出时随 Handler
/// 一起 drop, 保证托盘与进程生命周期严格同步。
#[cfg(target_os = "windows")]
pub struct TrayHandle {
    // TrayIcon 不实现 Send/Sync (内部持有平台特定句柄), 但 Handler 不跨线程,
    // 存为字段即可。
    tray: tray_icon::TrayIcon,
}

#[cfg(not(target_os = "windows"))]
pub struct TrayHandle;

impl TrayHandle {
    /// 整体替换托盘菜单。动作 (托盘点击/全局热键) 改变勾选态后由 Handler
    /// 重建, 保持勾选项与 App 状态一致。
    pub fn set_menu(&self, _menu: tray_icon::menu::Menu) {
        #[cfg(target_os = "windows")]
        self.tray.set_menu(Some(Box::new(_menu)));
    }
}

/// 安装系统托盘 (图标 + 菜单)。
///
/// `icon` 通常来自 [`crate::window::icon::load_tray_icon`] 或窗口图标;
/// `menu` 由调用方构建 (例: `examples/pomodoro/tray.rs::build_menu`),
/// 一旦传入即归 TrayIcon 所有, 调用方不应再持有。
///
/// 返回 `None` 表示安装失败 (日志已记录)。
#[cfg(target_os = "windows")]
pub fn install_tray(icon: tray_icon::Icon, menu: tray_icon::menu::Menu) -> Option<TrayHandle> {
    use tray_icon::TrayIconBuilder;
    match TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .build()
    {
        Ok(tray) => {
            log::info!("托盘图标已安装 (Windows)");
            Some(TrayHandle { tray })
        }
        Err(err) => {
            log::warn!("托盘图标安装失败: {err}");
            None
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn install_tray(_icon: tray_icon::Icon, _menu: tray_icon::menu::Menu) -> Option<TrayHandle> {
    log::info!("托盘在当前平台未启用");
    None
}
