---
name: winit-030-focus-window-foreground-lock
description: "winit 0.30 focus_window 在 Windows 后台进程下受前台锁静默失败, 须 AttachThreadInput 绕锁"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 4302791f-1c53-4e48-a3b5-d82663d20e39
  modified: 2026-08-01T10:06:41.235Z
---

winit 0.30 的 `Window::focus_window()` 在 Windows 上走 `force_window_active`：先 `SendInput` 合成 Alt 按下/抬起，再 `SetForegroundWindow`。对后台常驻进程不可靠——合成输入被投递给当前前台应用（另一进程），Windows 的「最近输入」记账仍记在对方头上，前台锁（foreground lock）照常生效，`SetForegroundWindow` 静默返回 FALSE：窗口「已显示但被遮」。

danqing 修复在 `src/window/foreground.rs` 的 `bring_hwnd_to_foreground`：先直调 `SetForegroundWindow`+`BringWindowToTop`；前台属其它线程时 `AttachThreadInput` 瞬时挂接→激活→立即 detach。接入点 `handler.rs::show_window`（Windows cfg 分支），覆盖番茄钟隐藏态完成专注自动呼出 + Ctrl+Shift+P 手动呼出两条路径。

相关：[[danqing-visual-debug-tooling]]（窗口类 bug 须物理复现取证）。
