---
name: winit-030-window-icon-two-tier
description: "winit 0.30 Windows 窗口图标分两档, with_window_icon 只设 ICON_SMALL, 任务栏需 with_taskbar_icon 补 ICON_BIG"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 9428be43-8220-4913-9f09-c354a5bef812
  modified: 2026-08-02T03:16:11.891Z
---

winit 0.30.13 在 Windows 上把窗口图标拆成 ICON_SMALL 与 ICON_BIG 两档,缺一不可:

- `WindowAttributes::with_window_icon` **只发 `WM_SETICON(ICON_SMALL)`**(window.rs `set_window_icon` 只调 Small)
- ICON_BIG(任务栏按钮/Alt+Tab 首选)只由 `WindowAttributesExtWindows::with_taskbar_icon(Option<Icon>)` 驱动,默认空
- 窗口类图标 winit 也注册为 0(`WNDCLASSEXW.hIcon=0`),别指望类图标兜底
- 不补 ICON_BIG 时任务栏沿回退链(ICON_SMALL → 无内嵌图标的 exe 缺省图标)行为不定 → **"偶发"显示系统缺省图标**

danqing 2026-08-02 落地: `src/window/icon.rs` 新增 `window_icons()`(同一 PNG 出两档)+ `with_taskbar_icon()`(cfg Windows 下用扩展 trait),`handler.rs` `resumed()` 窗口创建时两档分别挂。exe 内嵌图标是另一条防线(`tools/patch_icon.py`,手动 post-build,默认只打 release exe)。相关: [[danqing-assets-directory-convention]]
