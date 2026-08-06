# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

丹青 (danqing) 是一个 Rust 跨平台自绘 UI 框架,使用 `winit` 0.30 处理窗口与事件,`wgpu` 30 自绘,保留模式组件树。基础里程碑 M1~M3 与阶段 1(设计系统 + 品牌视觉)、阶段 2(专注陪伴 POC:番茄钟 × 场景沉浸美学)及后续补完均已关闭并归档到 `tasks/archive/`。

- **当前分支**: `dev`(主分支 `master`)
- **战略**: 2026-08-01 interview-me 确认「著作型旗舰」十年战略(专注陪伴系统 × 十年建造史),见 `docs/intent/companion-flagship.md` + `tasks/plan-flagship-roadmap.md`。里程碑 0「旗舰化第一刀」已完成:山/森林动效终审通过(2026-08-01)、付费边界 spec 确认、数据层 MVP、建造实录三篇草稿。剪贴板降级为引擎复用验证顺延。**未获用户指示时不要启动新 POC**。
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
- 提交前必须: `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` 全绿。可用 `Workflow({name: "pre-commit"})` 自动化三件套。
- 新增组件必须出现在 `examples/showcase.rs` 中(以用代测)。
- `widget/`、`layout.rs`、`event.rs`、`text/` 保持纯逻辑;平台/GPU 代码只出现在 `window/` 与 `render/`。
- 阶段 1 组件使用 `src/theme.rs` token,避免魔法颜色/圆角/阴影值。

## Context loading

按任务类型加载上下文,避免一次加载所有文档——聚焦上下文胜过大量上下文。

| 任务类型 | 必须加载 | 可选(深入时加载) | 相关 Memory |
|----------|----------|-------------------|-------------|
| **旗舰/十年战略**(数据层/建造实录/付费边界) | `docs/intent/companion-flagship.md` + `tasks/plan-flagship-roadmap.md` | `docs/specs/companion-flagship-pricing.md` | `danqing-flagship-strategy`, `danqing-project-state` |
| **场景动效**(shader/uniform/动效) | `docs/CONTEXT/scenes-guidelines.md` | 对应场景 spec(`docs/specs/pomodoro-scene-motion*.md`) | `scene-motion-uv-displacement`, `scene-lru-pattern` |
| **AI 场景底图升级**(生图/去水印/适配) | `memory/ai-scene-upgrade-workflow.md` + `docs/CONTEXT/ai-image-prompts.md` | `tools/remove_watermark.py` + `tools/export-scenes.py` | `ai-scene-uv-displacement-preference`, `ai-scene-no-veil` |
| **跨模块重构**(依赖/渲染/事件) | `docs/CONTEXT/architecture.md` | 相关模块源码 + 测试 | — |
| **窗口/平台**(winit/IME/托盘/热键) | `src/window/mod.rs` | `docs/CONTEXT/architecture.md` §平台适配层 | `danqing-visual-debug-tooling` |
| **新增组件**(widget) | 一个现有同族组件(照模式) | `src/theme.rs` | — |
| **性能/内存** | `tools/benchmark.ps1` | `docs/CONTEXT/architecture.md` | `wgpu-30-memory-lever`, `minidbg-symbol-preference` |
| **构建/工具链** | — | `build.rs` + `.cargo/config.toml` | `windows-gnu-toolchain-lld-fix` |
| **Pomodoro POC** | `examples/pomodoro/CLAUDE.md` | `docs/specs/phase2-pomodoro-poc.md` | — |
| **Bug 修复** | 最小复现 + `cargo test` 输出 | 相关模块源码 | — |

Agent 启动时的默认加载: `CLAUDE.md` + `MEMORY.md`(已自动加载)。CONTEXT 文档**不在默认加载之列**——仅在上述任务类型触发时按需加载。

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
