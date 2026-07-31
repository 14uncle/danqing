# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

丹青 (danqing) 是一个 Rust 跨平台自绘 UI 框架,使用 `winit` 0.30 处理窗口与事件,`wgpu` 30 自绘,保留模式组件树。基础里程碑 M1~M3 与阶段 1(设计系统 + 品牌视觉)、阶段 2(专注陪伴 POC:番茄钟 × 场景沉浸美学)及后续补完均已关闭并归档到 `tasks/archive/`。

- **当前分支**: `dev`(主分支 `master`)
- **下一步**: 山/森林场景动效等待用户人工终审。之后候选:第二 POC 剪贴板历史管理器(效率工具族,美学剂量低于专注陪伴族)。**未获用户指示时不要启动新 POC**。
- **性能门槛**: 启动 ≤1s、常驻内存 WS ≤360MB(核显记账);测量用 `tools/benchmark.ps1`。

> 详细架构见 `docs/CONTEXT/architecture.md`;场景动效开发范式见 `docs/CONTEXT/scenes-guidelines.md`。

## Common commands

```bash
# 运行阶段 1 演示页(会打开一个 GUI 窗口)
cargo run --example showcase

# 全部测试(纯逻辑,无需 GPU)
cargo test --lib --tests

# 性能基准(release 启动到可见 ≤1s、常驻内存 WS ≤360MB(核显记账);须先 cargo build --release --example showcase)
powershell -NoProfile -File tools/benchmark.ps1

# 运行单个测试
# 模块内单元测试
cargo test widget::layout::flow::tests::column_stacks_fit_children -- --exact
# 集成测试文件
cargo test --test event_dispatch press_and_release -- --exact

# 静态检查(必须零警告)
cargo clippy -- -D warnings

# 格式化与格式检查
cargo fmt
cargo fmt --check

# 发布构建
cargo build --release
```

## Architecture

数据流是单向的:

```
winit 事件(window/mod.rs)
    ↓ 转换为平台无关 Event(event.rs)
焦点路由 / 组件树命中分发
    ↓ 产出 Msg
App::update 修改状态
    ↓ 每帧
Widget::sync → Widget::animate → FocusManager::rebuild
Widget::layout 约束向下传、尺寸向上算
Widget::paint 收集 RectBatch / TextBatch
    ↓
render/mod.rs 提交 wgpu(矩形 SDF pass + 文本图集 pass)
```

依赖方向只允许向下: `widget/`、`layout.rs`、`event.rs`、`text/` 不得依赖 `winit`/`wgpu`。

## Build notes

- 字体、LOGO、背景图等二进制视觉资产统一放在仓库根目录 `assets/` 下并提交到版本控制。
- `build.rs` 不再下载字体或生成视觉资产;代码通过相对路径或 `include_bytes!` 直接从 `assets/` 加载。
- windows-gnu 工具链要求 PATH 上存在真正的 GNU binutils(`as.exe` + `dlltool.exe`),推荐从 MSYS2(清华 TUNA 镜像)安装。
- debug 构建默认关闭 wgpu 校验层以避免 1~2 秒启动/关闭延迟。需要校验层时请设置环境变量 `DANQING_WGPU_VALIDATION=1`(或 `WGPU_VALIDATION=1`)再运行 showcase。
- `.cargo/config.toml` 中已无额外 rustflags,msvc 环境下无副作用。

## Code conventions

- 公开 API 一律经 `src/lib.rs` re-export,不暴露深层模块路径给用户。
- 所有公共类型/函数写中文文档注释;内部实现用英文命名。
- 新增 `.rs` 文件头必须包含 `//! @author 十四叔` 与 `//! @date yyyy/MM/dd`。
- 提交前必须: `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` 全绿。
- 新增组件必须出现在 `examples/showcase.rs` 中(以用代测)。
- `widget/`、`layout.rs`、`event.rs`、`text/` 保持纯逻辑;平台/GPU 代码只出现在 `window/` 与 `render/`。
- 阶段 1 组件使用 `src/theme.rs` token,避免魔法颜色/圆角/阴影值。

## Documentation layout

- 规格(spec)文档统一放在 `docs/specs/`
- 实现计划(plan)与进度(todo)统一放在 `tasks/`;已关闭的里程碑 plan/todo 归档到 `tasks/archive/`
- 灵感/one-pager 等背景材料放在 `docs/ideas/`
- 分层上下文(按需加载)放在 `docs/CONTEXT/`

## Tests layout

- 单元测试写在对应模块的 `#[cfg(test)]` 中,覆盖纯逻辑:布局、事件命中、图集分配、文本排版、编辑状态。
- 集成测试在 `tests/` 中,端到端验证组件树构建 + 布局 + 模拟事件分发,无需 GPU。

## Exclusions

本项目明确不做以下方向:

- HIS / 医疗客户端(硬件集成与合规过重)
- 游戏专用组件(sprite、HUD、游戏内 UI 等)
- 试图成为覆盖所有场景的完整通用 UI 框架

优先服务 Rust 效率工具场景(剪贴板管理器、番茄钟、启动器、便签等)。
