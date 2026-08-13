---
name: wgpu-instance-prebuild-no-gain
description: "后台线程预建 wgpu 实例对启动时间净收益为零(实例省下的时间 request_adapter 等额变贵),已撤回,勿再试"
metadata: 
  node_type: memory
  type: project
  originSessionId: 0112d934-8ade-4fe0-ac65-6d9ff4e7edb8
  modified: 2026-07-28T04:16:08.359Z
---

2026-07-28 性能调优时试过:启动时在后台线程预建 `wgpu::Instance`(与字体加载/窗口创建并行),`resumed` 里 join。

**Why:** 实例创建确实被完全藏住(instance+surface 从 ~281ms 降到 ~µs),但 `request_adapter` 从 ~250ms 涨到 380~1270ms,GPU 路径总耗时不变(前后都 ~550-590ms),且方差更大。与项目历史一致(handler.rs 注释:此前预建 GpuDevice 也是 inline 更快)。已撤回,代码注释留有记录。

**How to apply:** 不要再尝试把 wgpu 初始化拆到后台线程;DX12 后端加载/适配器枚举的总成本是守恒的。启动调优的剩余空间不在 GPU 初始化拆分上。

相关测量事实:benchmark 首轮恒定 ~1.4s(驱动冷启动,与代码无关),热轮才代表真实水平;WS 测量有 ±7MB 驱动记账噪声。[[wgpu-30-memory-lever]]
