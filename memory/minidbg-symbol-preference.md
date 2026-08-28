---
name: minidbg-symbol-preference
description: "danqing 显式偏好保留二进制符号名以服务 minidbg 崩溃诊断, ~1MB 包体积换不去, profile 用 strip = \"debuginfo\" 而非 \"symbols\""
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 07cc6842-0680-4e27-9fdf-19c9c3c16744
  modified: 2026-07-24T08:27:34.027Z
---

# minidbg 符号保留偏好(2026-07-24 优化对话中确认)

## 决策
`Cargo.toml [profile.release]` 用 `strip = "debuginfo"` 而非 `strip = "symbols"`。**这是有意识的选择**,不是疏忽。

## 为什么
- 诊断/排障工具链里有 `tools/minidbg.rs`(见 CLAUDE.md)
- 崩溃诊断时需要从二进制符号名反推栈
- 代价:release exe 大约多 1MB(12MB → 8.1MB 用其他杠杆后,再加 1MB = 7MB)

## What this means
- 后续做体积优化时,**不要**默认推荐 `strip = "symbols"`
- 如果某轮优化只能通过 `strip = "symbols"` 才能达成目标,先停手告知,让用户拍板
- 用 `lto = "fat"` + `opt-level = "z"` + `panic = "abort"` 已经能压到 ~8MB,这是当前的甜区

## How to apply
任何给 danqing 做性能/体积建议时,把"minidbg 诊断可用"列为不可牺牲约束。

## 相关
[[danqing-visual-debug-tooling]] —— 排障工具链入口
