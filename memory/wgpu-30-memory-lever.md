---
name: wgpu-30-memory-lever
description: "wgpu 30 上 Backends::PRIMARY 在 Windows 同时拉起 Vulkan + DX12 两套后端加载器,默认 MemoryHints::Performance 留 slack; 两者一起改能砍 100+ MB"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 07cc6842-0680-4e27-9fdf-19c9c3c16744
  modified: 2026-07-24T08:27:21.531Z
---

# wgpu 30 内存双杠杆(常驻进程的隐形大头)

## 现象(2026-07-24 实测)
- danqing showcase release: WS 345 MB → 186 MB(-46%),**只改两行**
- danqing pomodoro release: 280 MB → 230 MB(基础上),再叠加场景 LRU → 210 MB

## 改法(`src/render/mod.rs`)
1. `Backends::PRIMARY`(Windows 默认)→ `Backends::DX12`
   - 注释原说"选择单一主 backend",但 PRIMARY 在 Windows = Vulkan + DX12 + WebGPU,实际拉起两套驱动
   - Windows 10+ 全平台支持 DX12 且核显驱动最稳
2. `request_device(...memory_hints: wgpu::MemoryHints::MemoryUsage, ..)`
   - 默认 `Performance` 让 wgpu 分配器保留大块 slack
   - `MemoryUsage` 减少预留,本框架渲染负载轻,性能损失可忽略

## 为什么容易踩
- `Backends::PRIMARY` 字面看像"primary 那个",直觉以为是单选
- `MemoryHints` 没显式设时是 `Performance` 默认,wgpu 文档不显眼
- 共享驱动 DLL 在 WS 中占比小,**私有提交**(private bytes)才是大头,容易误判

## 验证命令
```bash
powershell -NoProfile -File tools/benchmark.ps1 -Example showcase -Runs 3
# 看 WS 与 private 都应同步下降
```

## 相关
[[danqing-visual-debug-tooling]] —— 视觉排障的 GPU 端参考
