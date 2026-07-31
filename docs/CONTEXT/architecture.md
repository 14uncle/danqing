# Architecture Detail

> 按需加载的架构细节。日常任务不需此文件;处理跨模块重构、新增渲染管线、或理解依赖方向时加载。

## Data Flow (full)

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

- `App` trait 提供 `tick()`(每帧心跳)与 `background_frame()`(场景→渲染通道)默认方法。
- 背景管线双纹理 mix 交叉淡化;SceneFader 纯逻辑在 example 侧。
- Center 逐轴 tight + `fill_max()` 显式占满;fill 子项交叉轴宽松是刻意的(定高色块案例)。

## Dependency Rules

```
[纯逻辑层]                    [平台/GPU 层]
widget/                       window/
layout.rs                     render/
event.rs                      ├── rect.rs + rect.wgsl
text/                         ├── text.rs + text.wgsl
theme.rs                      └── background.rs + background.wgsl
app.rs
```

- 纯逻辑层**不得**依赖 `winit`/`wgpu`。
- 公开 API 一律经 `src/lib.rs` re-export。

## Project Map

按职责分块的文件清单(处理某块时按文件名 recall):

### 平台适配层 (只允许接触 OS/GPU)

- `src/window/mod.rs` — winit 事件循环、IME/剪贴板、焦点路由、每帧 `request_redraw`
  - `src/window/event.rs` — 平台无关事件类型
  - `src/window/handler.rs` — 事件处理分发
  - `src/window/hotkey.rs` — OS 级全局热键
  - `src/window/icon.rs` — 窗口图标
  - `src/window/tray.rs` — 系统托盘
- `src/render/rect.rs` + `rect.wgsl` — 矩形 SDF 渲染管线
- `src/render/text.rs` + `text.wgsl` — 文本图集渲染管线
- `src/render/background.rs` + `background.wgsl` — 多场景背景渲染(含程序化动效)

### 纯逻辑核心 (不得依赖 winit/wgpu)

- `src/app.rs` — `App` trait + `tick()`/`background_frame()` 默认方法 + `AnimationCtx`
- `src/event.rs` — 平台无关事件类型(鼠标/键盘/IME/剪贴板)
- `src/layout.rs` — 值类型 + `Color::lerp`/`contrast_ratio`/`composite_over`
- `src/theme.rs` — `Theme` trait + `ScenePalette`/`SceneTheme`/`SceneSpec` + `LightTheme` + `Easing`
- `src/text/line_layout.rs` — 多行排版(显式换行 + soft-wrap)
- `src/text/atlas.rs` — 文本图集分配
- `src/text/font.rs` — 字体加载(运行时读取 `assets/`)

### 组件库

- `src/widget/base/` — Button、Text
- `src/widget/layout/` — Box、Column、Row、Padding、Center、Stack(共享 `flow.rs`)
- `src/widget/form/` — TextInput、TextArea(共享 `text_editor.rs`)
- `src/widget/view/` — Scrollable、Switcher
- `src/widget/focus.rs` — FocusManager 焦点链与 Tab 遍历
- `src/widget/title_bar.rs` — 框架层标题栏

### 示例

- `examples/showcase.rs` — 持续生长,以用代测
- `examples/pomodoro/` — 番茄钟 POC
  - `timer.rs` / `scenes.rs` / `fader.rs` / `flash.rs` / `audio.rs` / `state.rs` / `ambient.rs` / `motion.rs` / `today.rs` / `hint.rs` / `tray.rs` / `main.rs`
- `examples/common/log.rs` — 共享 `init_log`
- `examples/minimal.rs` — 最小骨架
- `examples/mem_probe.rs` — 内存探针

### 资产

- `assets/fonts/` — 内嵌 OFL 黑体(思源黑体 GB2312 子集)
- `assets/logo/` — 多尺寸 PNG / ICO
- `assets/background/` — 渐变背景图、噪声纹理
- `assets/scenes/` — 5 场景 PNG(篝火/海/雨/山/森林)
- `assets/audio/` — 5 场景 CC0 环境音 OGG + `ATTRIBUTION.md`

### 测试

- 单元测试:各模块 `#[cfg(test)]` 内
- 集成测试: `tests/event_dispatch.rs` / `focus_input.rs` / `widget_tree.rs` / `switcher.rs` / `title_bar_window.rs` / `assets.rs` / `design_system.rs` / `hover_debug.rs`

### 工具

- `tools/benchmark.ps1` — 性能基准
- `tools/minidbg.rs` / `tools/linkwrap.rs` / `tools/dlltool-shim.rs` — 诊断/排障
