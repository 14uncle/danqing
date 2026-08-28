---
name: widget-showcase-demo-rule
description: 新增 widget 必须在 showcase.rs 添加演示，人工验证
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b3403604-a1b9-4c16-a1af-1ec53b0885ca
  modified: 2026-08-18T14:44:25.912Z
---

新增 widget 组件时，必须在 `examples/showcase.rs` 中添加对应的演示卡片。

**Why:** 用户需要人工验证组件的视觉效果和交互行为，showcase 是唯一的演示程序。

**How to apply:**
1. 在 `page_view`（或对应分类页）中添加 `card(t, "组件名", demo_card(t))`
2. demo_card 展示组件的核心功能（状态切换、交互、绑定等）
3. 使用 `on_change` 或 `bind` 与应用状态联动
4. 提交前确保 `cargo clippy --example danqing-showcase -- -D warnings` 零警告

**Reference:**
- Tabs demo: `examples/showcase.rs` 的 `tabs_card` 函数
- showcase 结构: 左侧导航 + 右侧 MultiPanel 切换分类面板
