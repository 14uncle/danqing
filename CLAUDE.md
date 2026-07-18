# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

丹青 (danqing) 是一个 Rust 跨平台自绘 UI 框架,处于 **M2** 里程碑(M1 已关闭)。它使用 `winit` 0.30 处理窗口与事件,`wgpu` 30 自绘,保留模式组件树;M2 已加入焦点系统、单行 `TextInput`、剪贴板与 IME 支持。

## Common commands

```bash
# 运行 M1 演示页(会打开一个 GUI 窗口)
cargo run --example showcase

# 全部测试(纯逻辑,无需 GPU)
cargo test

# 运行单个测试
# 模块内单元测试
cargo test widget::flow::tests::column_stacks_fit_children -- --exact
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
winit 事件(window.rs)
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

- `src/app.rs`: `App` trait(`update`/`view`/`event`) 与 `run_app()` 入口;`AnimationCtx` 用于每帧动画(如光标闪烁)。
- `src/window.rs`: 唯一允许接触 OS 窗口 API 的适配层;winit 事件循环、焦点路由、IME/剪贴板封装、消息消费、每帧 `request_redraw` 驱动。
- `src/event.rs`: 平台无关事件类型(鼠标/键盘/IME/剪贴板)与分发语义;`Event::Key` 携带 shift/ctrl 修饰键。
- `src/layout.rs`: 纯逻辑值类型与布局分配算法。
- `src/widget/`: 纯逻辑组件。`Widget` trait 含 `sync`/`animate`/`layout`/`paint`/`event` 及焦点相关默认方法(`focusable`/`children`/`ime_area`/`wants_ime`/`selected_text`);容器复用 `src/widget/flow.rs`;`FocusManager` 负责焦点链与 Tab 遍历;`TextInput` 是单行可编辑文本组件。
- `src/render/`/`src/text/`: 同 M1。

依赖方向只允许向下: `widget/`、`layout.rs`、`event.rs` 不得依赖 `winit`/`wgpu`。

## Build notes

- `build.rs` 首次构建时会从 jsdelivr 下载 OFL 回退字体(ZCOOL XiaoWei)到 `OUT_DIR`,仓库内不提交字体二进制。若下载失败,检查网络或更新 `EXPECTED_SIZE`。
- `.cargo/config.toml` 记录本机 windows-gnu 工具链排障结论;msvc 环境下无副作用。
- debug 构建默认关闭 wgpu 校验层以避免 1~2 秒启动/关闭延迟。需要校验层时请设置环境变量 `DANQING_WGPU_VALIDATION=1`(或 `WGPU_VALIDATION=1`)再运行 showcase,用于人工验证无校验错误。

## Code conventions

- 公开 API 一律经 `src/lib.rs` re-export,不暴露深层模块路径给用户。
- 所有公共类型/函数写中文文档注释;内部实现用英文命名。
- 提交前必须: `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test` 全绿。
- 新增组件必须出现在 `examples/showcase.rs` 中(以用代测)。
- `widget/`、`layout.rs`、`event.rs` 保持纯逻辑;平台/GPU 代码只出现在 `window.rs` 与 `render/`。

## Tests layout

- 单元测试写在对应模块的 `#[cfg(test)]` 中,覆盖纯逻辑:布局、事件命中、图集分配。
- 集成测试在 `tests/` 中,端到端验证组件树构建 + 布局 + 模拟事件分发,无需 GPU。
