# Implementation Plan: 背景图片渲染支持

## Overview

当前 `examples/showcase.rs` 已使用 `LightTheme` 与主题化组件呈现浅色毛玻璃风格,但窗口背景仍依赖 `clear_color` 的纯色填充,未能真正使用 `build.rs` 生成的 `gradient.png` / `noise.png`。本计划旨在增加最简的图片纹理渲染能力,使 showcase 能够将生成的背景图作为底层渲染,从而完整满足阶段 1 规格中“窗口背景使用固定渐变/噪声图营造半透明 + 模糊质感”的要求。

## Architecture Decisions

- **窗口级背景而非通用 Image Widget**: 阶段 1 的背景图需求明确为“窗口背景”,因此将配置放在 `WindowConfig` 中,由 `Context` 在 RectBatch 之前单独绘制。这比新增通用 `Image` Widget 更聚焦,避免改动 `Widget::paint` 签名与所有组件实现。
- **渲染层扩展**: 在 `src/render/` 下新增 `background.rs` 与 `background.wgsl`,作为第三路渲染 pass,与 `rect.rs` / `text.rs` 并列。避免改动现有矩形 SDF 管线,降低回归风险。
- **纹理生命周期**: 图片在 `Context` 初始化时解码为 RGBA,上传为 `wgpu::Texture`,由 `BackgroundPipeline` 持有。阶段 1 仅支持窗口背景图,不做图集或动态加载。
- **绘制顺序**: `BackgroundPipeline` 在 `Context::render` 中先于 `RectPipeline` / `TextPipeline` 绘制,确保背景在最底层。
- **失败回退**: 图片加载失败时记录警告并回退到 `WindowConfig.clear_color`,窗口不 panic。

## Task List

### Phase 1: 背景图渲染管线
- [x] **Task 1: 新增 `BackgroundPipeline`**
  - **Description:** 在 `src/render/background.rs` 实现 `BackgroundConfig`、`ScaleMode` 与 `BackgroundPipeline`(texture + sampler + bind group + render pipeline),支持 Stretch/Fit/Cover 三种缩放;在 `src/render/mod.rs` 中先于 `RectPipeline` 绘制。
  - **Acceptance criteria:**
    - [x] `BackgroundPipeline` 能正确采样纹理并输出到 surface。
    - [x] 无图片时 pass 不绘制,不崩溃。
    - [x] 图片加载失败时回退到清屏色。
  - **Verification:**
    - [x] `cargo test --lib` 通过。
    - [x] `cargo clippy -- -D warnings` 通过。
  - **Dependencies:** None
  - **Files touched:**
    - `src/render/background.rs`
    - `src/render/background.wgsl`
    - `src/render/mod.rs`
    - `src/window.rs`
    - `src/lib.rs`
  - **Estimated scope:** M

### Phase 2: Showcase 接入
- [x] **Task 2: 在 showcase 中使用背景图**
  - **Description:** 更新 `examples/showcase.rs`,通过 `WindowConfig.background` 加载 `OUT_DIR/assets/background/gradient.png` 与 `noise.png`,主图使用 Cover 缩放,噪声图以低透明度叠加。
  - **Acceptance criteria:**
    - [x] showcase 窗口背景显示渐变图。
    - [x] 背景之上正确绘制 `TitleBar` 与主题化组件。
    - [x] 图片缺失时窗口仍能启动并显示回退背景色。
  - **Verification:**
    - [x] `cargo build --example showcase` 通过。
    - [x] `cargo run --example showcase` 待人工确认视觉。
  - **Dependencies:** Task 1
  - **Files touched:**
    - `examples/showcase.rs`
  - **Estimated scope:** S

### Phase 3: 测试与验收
- [x] **Task 3: 补充设计系统测试**
  - **Description:** 在 `tests/design_system.rs` 增加背景图配置构造、showcase 组件树使用 theme 等断言,并补全 `theme.rs` / 各 widget 模块的单元测试。
  - **Acceptance criteria:**
    - [x] `tests/design_system.rs` 覆盖 theme token、组件应用 theme、TitleBar 命中、BackgroundConfig 构造。
    - [x] `cargo test` 全绿。
    - [x] `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo build --release` 通过。
  - **Verification:**
    - [x] `cargo test --lib --tests`
    - [x] `cargo clippy -- -D warnings`
    - [x] `cargo build --release`
  - **Dependencies:** Task 2
  - **Files likely touched:**
    - `tests/design_system.rs`
    - `src/theme.rs`
    - `src/widget/*.rs`
    - `src/render/background.rs`
  - **Estimated scope:** M

### Checkpoint: 阶段 1 背景图能力关闭
- [x] `cargo build --example showcase` 通过。
- [x] `cargo run --example showcase` 能看到统一毛玻璃视觉与背景图(人工确认)。
- [x] 全部 Commands 绿。
- [x] 人工终审,阶段 1 背景图相关验收通过。

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| 新增 wgpu texture/sampler 导致渲染管线回归 | 高 | 独立 `BackgroundPipeline`,不改 `RectPipeline`;已跑通全部测试。 |
| `image` crate 已在 dependencies 中,运行时复用 | 低 | 复用现有 `image` crate 的 `png` feature,未新增外部依赖。 |
| 背景图与半透明组件混合顺序错误 | 中 | `BackgroundPipeline` 先于 `RectPipeline` 绘制。 |
| 多平台纹理格式差异 | 低 | 使用 `Rgba8UnormSrgb`,为常见适配器支持格式。 |

## Open Questions (已确认)

1. ~~是否需要同时叠加 `gradient.png` 与 `noise.png`,还是二选一即可?~~ **同时叠加**: 最底层 `gradient.png`,之上以低透明度叠加 `noise.png`,再之上绘制主题化组件。
2. ~~是否需要支持缩放模式?~~ **支持缩放**: `ScaleMode::{Stretch, Fit, Cover}` 已实现在 `WindowConfig.background` 中。
3. ~~是否优先完成 Task 11(设计系统测试 + 最终验收),再追加背景图能力?~~ **否**: 背景图能力已落地,剩余测试工作并入 Task 11 或本计划 Task 3。

## Deviation Note

原计划在 `Widget` 层新增通用 `Image` 组件,实施中发现会波及 `Widget::paint` 签名与全部 11+ 个组件实现,回归面较大。由于阶段 1 需求明确为“窗口背景图”,改为在 `WindowConfig` / `Context` 层实现,同样满足“同时叠加”与“支持缩放”两个决策点,且风险更小。
