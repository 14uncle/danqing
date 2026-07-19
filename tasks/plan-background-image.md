# Implementation Plan: 背景图片渲染支持

## Overview

当前 `examples/showcase.rs` 已使用 `LightTheme` 与主题化组件呈现浅色毛玻璃风格,但窗口背景仍依赖 `clear_color` 的纯色填充,未能真正使用 `build.rs` 生成的 `gradient.png` / `noise.png`。本计划旨在增加最简的图片纹理渲染能力,使 showcase 能够将生成的背景图作为底层渲染,从而完整满足阶段 1 规格中“窗口背景使用固定渐变/噪声图营造半透明 + 模糊质感”的要求。

## Architecture Decisions

- **渲染层扩展**: 在 `src/render/` 下新增 `image.rs` 与 `image.wgsl`,与 `rect.rs` / `text.rs` 并列,作为第三路渲染 pass。避免改动现有矩形 SDF 管线,降低回归风险。
- **纹理生命周期**: 图片在 CPU 侧解码为 RGBA,上传为 `wgpu::Texture`,由渲染上下文持有。阶段 1 仅支持单张全屏背景图,不做图集或动态加载。
- **Widget 层抽象**: 新增 `src/widget/image.rs` 的 `Image` 组件,接收文件路径,在 `paint` 时向 `ImageBatch` 提交一个带 UV 的 quad。保持 widget/ 纯逻辑、render/ 管平台的边界。
- **与 RectBatch 的协作**: `ImageBatch` 在 `Context::render` 中先于 `RectBatch` 绘制,确保背景在最底层。clip stack 对图片同样生效。
- **失败回退**: 图片加载失败时记录警告并回退到 `LightTheme.background()` 纯色,窗口不 panic。

## Task List

### Phase 1: 图片解码与上传
- [ ] **Task 1: 新增 `ImageBatch` 与 `ImagePipeline`**
  - **Description:** 在 `src/render/image.rs` 实现 `ImageBatch`(收集带 UV 的 quad 实例)与 `ImagePipeline`(texture + sampler + bind group + render pipeline)。
  - **Acceptance criteria:**
    - [ ] `ImageBatch` 可添加一个全屏或指定区域的 textured quad。
    - [ ] `ImagePipeline::draw` 能正确采样纹理并输出到 surface。
    - [ ] 无图片时 pass 不绘制,不崩溃。
  - **Verification:**
    - [ ] `cargo test --lib` 通过。
    - [ ] `cargo clippy -- -D warnings` 通过。
  - **Dependencies:** None
  - **Files likely touched:**
    - `src/render/image.rs`
    - `src/render/image.wgsl`
    - `src/render/mod.rs`
  - **Estimated scope:** M

- [ ] **Task 2: 实现 `Image` Widget**
  - **Description:** 在 `src/widget/image.rs` 实现 `Image` 组件,支持从文件路径加载 PNG,失败时回退到指定颜色。提供 `Image::new(path)` 与 `Image::themed(&theme, path)` 构造器。
  - **Acceptance criteria:**
    - [ ] `Image` 在 `paint` 时向 `ImageBatch` 提交自身区域。
    - [ ] 文件不存在时回退到 `theme.background()` 并记录 `log::warn`。
    - [ ] 公开类型经 `src/lib.rs` 与 `src/widget/mod.rs` re-export。
  - **Verification:**
    - [ ] 单元测试: 有效路径产生非空 batch,缺失路径使用回退颜色。
    - [ ] `cargo test widget::image` 通过。
  - **Dependencies:** Task 1
  - **Files likely touched:**
    - `src/widget/image.rs`
    - `src/widget/mod.rs`
    - `src/lib.rs`
  - **Estimated scope:** M

### Checkpoint: 渲染层与 Widget 就绪
- [ ] `cargo test --lib --tests` 全绿。
- [ ] `cargo clippy -- -D warnings` 零警告。
- [ ] 人工 review `Image` API 设计。

### Phase 2: Showcase 与验收
- [ ] **Task 3: 在 showcase 中使用背景图**
  - **Description:** 更新 `examples/showcase.rs`,在组件树最底层放置 `Image::themed(&theme, OUT_DIR/assets/background/gradient.png)`,并在其上叠加半透明 `noise.png` 或直接使用 `theme.background()` 色调。
  - **Acceptance criteria:**
    - [ ] showcase 窗口背景显示渐变图。
    - [ ] 背景之上正确绘制 `TitleBar` 与主题化组件。
    - [ ] 图片缺失时窗口仍能启动并显示回退背景色。
  - **Verification:**
    - [ ] `cargo build --example showcase` 通过。
    - [ ] `cargo run --example showcase` 人工确认视觉。
  - **Dependencies:** Task 1, Task 2
  - **Files likely touched:**
    - `examples/showcase.rs`
  - **Estimated scope:** S

- [ ] **Task 4: 补充设计系统测试**
  - **Description:** 在 `tests/design_system.rs` 增加背景图加载、Image widget 回退、showcase 组件树使用 theme 的断言,并补全 `theme.rs` / 各 widget 模块的单元测试。
  - **Acceptance criteria:**
    - [ ] `tests/design_system.rs` 覆盖 theme token、组件应用 theme、TitleBar 命中、Image 回退。
    - [ ] `cargo test` 全绿。
    - [ ] `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo build --release` 通过。
  - **Verification:**
    - [ ] `cargo test --lib --tests`
    - [ ] `cargo clippy -- -D warnings`
    - [ ] `cargo build --release`
  - **Dependencies:** Task 3
  - **Files likely touched:**
    - `tests/design_system.rs`
    - `src/theme.rs`
    - `src/widget/*.rs`
  - **Estimated scope:** M

### Checkpoint: 阶段 1 背景图能力关闭
- [ ] `cargo run --example showcase` 能看到统一毛玻璃视觉与背景图。
- [ ] 全部 Commands 绿。
- [ ] 人工终审,阶段 1 背景图相关验收通过。

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| 新增 wgpu texture/sampler 导致渲染管线回归 | 高 | 独立 `ImagePipeline`,不改 `RectPipeline`;增加渲染层单元测试。 |
| `image` crate 已在 build-dependencies,运行时依赖需谨慎 | 中 | 复用现有 `image` dependencies 中的 `png` feature,不新增外部依赖。 |
| 背景图与半透明组件混合顺序错误 | 中 | `ImageBatch` 先于 `RectBatch` 绘制;clip stack 复用现有逻辑。 |
| 多平台纹理格式差异 | 低 | 使用 `Rgba8UnormSrgb` 或 adapter 支持的格式,上传时做格式适配。 |

## Open Questions (已确认)

1. ~~是否需要同时叠加 `gradient.png` 与 `noise.png`,还是二选一即可?~~ **同时叠加**: 最底层 `gradient.png`,之上以低透明度叠加 `noise.png`,再之上绘制主题化组件。
2. ~~`Image` widget 是否需要支持显式尺寸 / 缩放模式(stretch/fit/cover)?阶段 1 是否只要全屏 stretch?~~ **支持缩放**: `Image` 提供 `ScaleMode::{Stretch, Fit, Cover}`。
3. ~~是否优先完成 Task 11(设计系统测试 + 最终验收),再追加背景图能力?~~ **否**: 先落地背景图能力,再进入 Task 11 做最终验收。