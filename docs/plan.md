# Plan: 丹青 M1 实现计划

> 基于 `docs/spec.md`(已批准,2026-07-16)
> 目标: M1 最小闭环 —— 跨平台开窗 + 基础图元/文本绘制 + 键鼠事件 + showcase

## 架构分层

```
┌──────────────────────────────────────┐
│ examples/showcase.rs                 │ 应用示例
├──────────────────────────────────────┤
│ app.rs      App trait / run()        │ 应用层: 状态、消息、帧循环
├──────────────────────────────────────┤
│ widget/     Widget trait / 内建组件   │ UI 核心: 保留模式组件树
│ layout.rs   Constraints / Size       │
│ event.rs    事件类型 / 分发           │
├──────────────────────────────────────┤
│ text/       Font / Atlas             │ 文本层: 字体加载、字形图集
│ render/     Context / 管线×2          │ 渲染层 (wgpu)
├──────────────────────────────────────┤
│ window.rs   winit 封装               │ 平台适配层(唯一允许碰 OS API 的地方)
└──────────────────────────────────────┘
```

依赖方向只允许向下,不许反向;`widget/`、`layout.rs`、`event.rs` 为纯逻辑,不依赖 wgpu/winit。

## 执行模型(每帧)

1. winit 事件 → 内部 `Event` → 分发:鼠标事件经**命中测试**送达组件,键盘事件直送 `App`
2. 组件事件产生消息 `Msg` → `App::update(msg)` 修改应用状态
3. **layout**:约束向下传、尺寸向上算、父组件定子组件位置
4. **paint**:遍历组件树收集绘制命令(rect 实例 + 文本 run),两条管线分别绘制
5. M1 采用**持续渲染**(每帧 `request_redraw`,游戏式);按需渲染省电优化留待 M2

数据流约定:组件属性可绑定到状态的读取闭包(如 `Text` 绑定 `Fn(&S) -> String`),框架每帧同步 —— 树是保留的,数据是声明式的。

## 实施步骤(依赖顺序,每步一个验证点)

| 步骤 | 内容 | 涉及文件 | 验证 |
|---|---|---|---|
| 0 | 脚手架:依赖锁定、lib 骨架、空 example | Cargo.toml, src/lib.rs | `cargo build` 通过 |
| 1 | winit 开窗:ApplicationHandler 封装、事件打印 | src/window.rs | 窗口打开,关闭干净退出 |
| 2 | wgpu 上下文:device/queue/surface、清屏、resize | src/render/mod.rs | 窗口清屏为指定色,resize 无校验错误 |
| 3 | SDF 矩形管线:实例化 quad + fragment SDF 圆角/AA | src/render/rect.rs, rect.wgsl | 多个彩色圆角矩形,边缘平滑,resize 不变形 |
| 4 | 文本:font-kit 系统字体 + 内嵌回退、字形图集、文本管线 | src/text/*, src/render/text.rs, text.wgsl | 渲染 "Hello, 你好世界",中英文清晰 |
| 5 | 布局:Constraints/Size/Rect + column/row/padding/center | src/layout.rs | 单元测试(纯逻辑) |
| 6 | 组件树:Widget trait、Node、Box/Text/Column/Row/Padding/Center | src/widget/* | 单元/集成测试:建树 + 布局 |
| 7 | 事件分发:命中测试、hover/pressed 状态、键盘→App | src/event.rs | 单元测试 + 手动点击反馈 |
| 8 | App glue:App trait、Button(on_click → Msg)、run() | src/app.rs, src/widget/button.rs | 计数器 demo 可点击 |
| 9 | showcase:色板、圆角、中英文、按钮计数、键盘移方块 | examples/showcase.rs | **M1 验收 6 条全过** |
| 10 | 打磨:clippy 零警告、fmt、README、spec 收尾 | 全部 | 全部 Commands 绿 |

依赖锁定(2026-07-16 查询):winit 0.30.13 · wgpu 30 · fontdue 0.9.3 · font-kit 0.14.3 · etagere 0.3 · bytemuck 1 · pollster 1 · thiserror 2 · anyhow 1 · log 0.4 · env_logger 0.11

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| ~~wgpu/winit 版本兼容~~ | **已排除**:wgpu 30 要求 raw-window-handle ^0.6.2,winit 0.30.13 实现 rwh 0.6,兼容确认 |
| SDF shader 跨后端表现差异 | 成熟技术,参考多;备用方案:`lyon` tessellation |
| 内嵌回退字体选型/体积 | 选 SIL OFL 许可字体(首选得意黑 Smiley Sans,~2MB),Step 4 定;仅回退用,不影响常规体积 |
| winit 0.30 事件循环模型踩坑 | 严格按官方 ApplicationHandler + RedrawRequested 模式实现 |
| 无 GPU 环境无法自动测试 | 布局/事件/图集全部设计为纯逻辑可单测,与 GPU 隔离 |

## 并行性

Step 5(布局)与 Step 3-4(渲染管线)相互独立,可并行;Step 7 依赖 5+6。单人开发按表顺序推进,每步留验证点。

## 验证检查点

- Step 2/3/4 完成后:各跑一次确认 wgpu 校验层无错误
- Step 5/6/7 完成后:`cargo test` 全绿
- Step 9:对照 spec.md 验收标准 6 条逐项验证
- Step 10:`cargo fmt` / `cargo clippy -- -D warnings` / `cargo test` 全绿

## 前置条件(阻塞 Step 0)

- [ ] 用户安装 Rust 工具链(rustup + stable),装好后 `cargo --version` 可用
