# Todo: 丹青番茄钟 — 托盘菜单 + 首次启动 hint(2026-07 迭代)

> 阶段 2 补完之后,用户日常使用中暴露两个新需求:
> 1. 第一次用不知道全局热键存在 → 首次启动提示
> 2. 想随时参考热键 / 不开窗口也想暂停 → 托盘右键菜单
> 本迭代把这两件事交付了。详见 `docs/specs/phase2-pomodoro-poc.md`(已更新边界)。

## 交付物

- [x] **Slice 1** 首次启动快捷键 hint(`b410122`)
  - `examples/pomodoro/hint.rs`(新建):ShortcutHintOverlay 状态机, 7.3s 总时长
    (1.5s 静默 + 300ms ease-out 淡入 + 5s 停留 + 500ms ease-in 淡出)
  - `PomodoroState` 加 `#[serde(default)] has_seen_shortcut_hint: bool` 向后兼容
  - main.rs Stack 第三层 child 锚定窗口右下角(fill-spacer 推位 + Padding 内缩)
  - 6 个 hint 状态机单元测试 + 1 个旧 JSON 兼容测试
  - 验收:首次启动右下角淡入三行, 二次启动无提示, JSON 持久化

- [x] **Slice 2** 托盘框架层基础(`cd1796e`)
  - `Cargo.toml`:加 `tray-icon = "0.19"`
  - `src/window.rs`:加 `pub mod tray_action_ids` + `pub mod tray` 框架层
    (TrayHandle 持有 tray-icon::TrayIcon, install_tray 接收 Menu)
  - `load_tray_icon()` 从 `assets/logo/logo_16.png` 构建 tray-icon::Icon
  - Handler 加 `tray: Option<tray::TrayHandle>` 字段 (Drop 即清理)
  - Handler::about_to_wait drain `MenuEvent::receiver` (静态, 跨帧复用)
  - `App` trait 加 `tray_action` 默认方法
  - 验收:任务栏右下角出现 danqing 16x16 图标, 右键空菜单, 关闭消失

- [x] **Slice 3** 菜单结构 + 快捷键 label + UI 刷新修复(`e278904` + `0e00cbe`)
  - `src/window.rs::shortcut_for_id(id)` 单一来源 (双检 hotkey/tray id)
  - `examples/pomodoro/tray.rs`(新建):`build_menu()` 三条目 + Predefined 分隔符
  - `App::tray_menu()` 默认空, PomodoroApp 返 `build_menu()`
  - Handler::about_to_wait 解析 `MenuId.0` 为 u8, 转交 `app.tray_action`
  - **修复**:muda `TrackPopupMenu` 阻塞主线程, 模态循环可能丢 paint 消息;
    改 ControlFlow::Poll + 主动 `request_redraw()` 保证菜单 close 后下一帧
    立刻 paint 新值
  - **平台限制**:Windows 菜单打开期间读秒暂停 (TrackPopupMenu 阻塞无法绕过),
    关闭后立刻恢复。Linux/macOS 不受影响 (muda GTK/NSMenu 非阻塞)。

- [x] **Slice 4** shortcut_for_id 单一来源重构(`7f0a2bd`)
  - `shortcut_hint_overlay_widget` 三行硬编码 'Ctrl+Shift+P/S/Q' 切到
    `format!("{} {}", 动作, shortcut_for_id(id))`, 与 tray menu 同源
  - `src/window.rs::tests` 加两个契约测试:
    - `shortcut_for_id_returns_consistent_label_across_id_sets` (三对 id 同 label)
    - `shortcut_for_id_unknown_id_returns_empty` (非法 id 返空串不 panic)

## 全量验证

- [x] `cargo fmt --check` 无 diff
- [x] `cargo clippy -- -D warnings` 零警告
- [x] `cargo test --lib --tests`: 218 通过(新增 2 个 shortcut_for_id 测试)
- [x] `cargo test --example pomodoro`: 57 通过 (12 hint + 4 tray + 41 既有)
- [x] `cargo build --release --example pomodoro`: 干净
- [x] 用户手验: 托盘右键看到三菜单 + 正确 label, 点击切换, 菜单关闭后 UI 立刻更新,
      二次点击正常 toggle, 首次启动 hint 仍正常

## 边界变更

- `docs/specs/phase2-pomodoro-poc.md:139` 「Ask first」中:
  - 划掉「全局快捷键」(阶段 2 补完已落地)
  - 划掉「托盘」(本迭代落地, 注明 Windows 平台菜单 open 期间读秒暂停的限制)

## 不在本轮范围

- 托盘菜单项动态化(根据 state 切灰/勾选, 当前全静态)
- 菜单打开期间读秒继续(Windows 平台限制, 需换非阻塞菜单机制, 代价 3~5 天)
- 自启动(仍在 Ask first 边界)
- 统计 / 历史(全新方向, 跟「完善 pomodoro」是不同 POC)
