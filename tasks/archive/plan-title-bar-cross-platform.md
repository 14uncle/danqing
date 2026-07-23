# Implementation Plan: 全平台统一自绘标题栏

> 依据 `docs/specs/title-bar-cross-platform.md`(interview-me 产出，2026-07-22 确认）细化。
> 拆分为 **6 个可验证任务**,2 个检查点；全程 `WindowAction` 枚举不变，Windows 视觉零回归。

## Overview

1. `TitleBar` 按钮从「硬编码从右往左三键」重构为「角色 × 样式」二维模型：
   `TitleBarStyle::Standard`（右置 Windows 风，现状）与 `TitleBarStyle::TrafficLights`（左置红绿灯）。
   样式默认值由 `cfg!(target_os = "macos")` 解析，但显式可构造 —— Windows 宿主机即可测红绿灯布局。
2. macOS 红绿灯：直径 12px、间隙 8px、前导边距 12px、垂直居中；顺序 红(close)/黄(minimize)/绿(maximize);
   hover 显示深色 × / − / + 符号；颜色经 `Theme` 新增 token 落地。
3. `window.rs` 移除 `with_decorations(false)` 的 `#[cfg(target_os = "windows")]` 门控，三平台统一去装饰；
   Windows 的 DWM 圆角/阴影设置保留不动。

## Architecture Decisions

### 角色化按钮模型（Task 1 重构核心）

- 新增私有 `enum ButtonRole { Close, Maximize, Minimize }`;`buttons: [TitleButton; 3]` 改为按角色索引
  （定义 `ButtonRole::ALL: [ButtonRole; 3] = [Close, Maximize, Minimize]`，数组下标即角色序）。
- `TitleBarStyle` 决定**有序角色列表与放置方向**:
  - `Standard`:`[Close, Maximize, Minimize]` 从**右**往左排（现状，像素级不变）。
  - `TrafficLights`:`[Close, Minimize, Maximize]` 从**左**往右排，圆形按钮。
- `button_rect(area, role)` / `hit_button` / `paint` / `emit_button_action` 全部改按角色寻址；
  对外行为（回调、拖拽、双击最大化）与样式无关。
- `TitleBarStyle::platform_default()`:`cfg!(target_os = "macos")` → `TrafficLights`，否则 `Standard`。
  `TitleBar::new/themed` 用之；新增 `.style(TitleBarStyle)` builder 供测试与特殊场景覆盖。
- `TitleBarStyle` 为公开枚举（中文文档注释），经 `widget/mod.rs` 平铺 re-export。

### 红绿灯绘制（Task 2)

- 圆形按钮：`push_rect(rect, color, diameter/2)` 即得圆（SDF 圆角矩形半径 = 半边）。
- 符号只在 hover 时绘制，颜色 `Color::rgba(0,0,0,0.55)`；复用 `paint_button_symbol` 的几何参数，
  符号种类映射：Close→×（复用）、Minimize→−（复用）、Maximize→+(**新增变体**：水平 + 垂直两条矩形）。
- 非 hover 状态不画任何符号（macOS 原生行为）。
- hover/pressed 复用现有 `TitleButton` 状态机，红绿灯无矩形 hover 背景（按钮本身是实心圆）。

### Theme token(Task 2 前置）

`Theme` trait 新增三个方法（增量，不破坏既有实现者 —— trait 目前仅 `LightTheme` 一个实现）:

```rust
fn traffic_close(&self) -> Color;    // LightTheme: #FF5F57
fn traffic_minimize(&self) -> Color; // LightTheme: #FEBC2E
fn traffic_maximize(&self) -> Color; // LightTheme: #28C840
```

### macOS 布局顺序（Task 2)

`[12px 边距][红绿灯组 52px][logo_gap][LOGO][logo_gap][标题]······(其余为拖拽区)`。
标题不居中，LOGO + 标题顺排在红绿灯之后（访谈确认）。

### window.rs(Task 4)

- 删除 `#[cfg(target_os = "windows")]` 门控与「其他平台保留原生标题栏作为降级」注释，
  `with_decorations(false)` 全平台生效。
- `apply_windows_undecorated_style` 维持 Windows 专属；自绘边框（`border_thickness > 0`）本就不分平台，不动。

## Dependency Graph

```
Phase 1 组件层(纯逻辑)
Task 1 角色化重构 + TitleBarStyle 骨架   (行为不变)
Task 2 Theme token + TrafficLights 实现  (依赖 Task 1)
Task 3 红绿灯集成测试                    (依赖 Task 2)
   └─ Checkpoint A: fmt + clippy + 全测试绿 + showcase Windows 冒烟

Phase 2 平台层
Task 4 window.rs 全平台去装饰            (逻辑上独立于 1-3,单开发者串行)
Task 5 跨平台编译检查 (linux/macos)      (依赖 Task 1-4 的全部 cfg 分支)
   └─ Checkpoint B: 三平台 cargo check 通过

Phase 3 收尾
Task 6 文档同步 + 最终验收               (依赖 Checkpoint B)
```

关键路径：1 → 2 → 3 → 5 → 6。

## Task List

### Phase 1: 组件层 — TitleBar 样式化

- [ ] **Task 1: 按钮角色化重构 + `TitleBarStyle` 骨架（Standard 行为像素级不变）**
  - **Description:** 新增公开枚举 `TitleBarStyle { Standard, TrafficLights }`（仅 `Standard` 有实现）与
    `platform_default()`;`TitleBar` 增加 `style` 字段与 `.style()` builder；引入私有 `ButtonRole`,
    `button_rect`/`hit_button`/`button_symbol_color`/`button_background_color`/`emit_button_action`/
    `paint_button_symbol` 全部改按角色寻址，Standard 分支布局公式保持原样。
    `widget/mod.rs` re-export 追加 `TitleBarStyle`。
  - **Acceptance criteria:**
    - [ ] 现有 `title_bar.rs` 全部单测与 `tests/title_bar_window.rs` 零修改通过（布局像素级不变的证据）。
    - [ ] `TitleBar::themed(&LightTheme, "x").style(TitleBarStyle::Standard)` 可构造。
    - [ ] `danqing::widget::TitleBarStyle` 平铺路径可用。
  - **Verification:** `cargo test --lib --tests` 全绿；`cargo clippy -- -D warnings` 零警告。
  - **Dependencies:** None
  - **Files:** `src/widget/title_bar.rs`, `src/widget/mod.rs`
  - **Scope:** M

- [ ] **Task 2: Theme 红绿灯 token + TrafficLights 布局/绘制/hover 符号**
  - **Description:** `Theme` trait 新增 `traffic_close/minimize/maximize`,`LightTheme` 实现为
    `#FF5F57`/`#FEBC2E`/`#28C840`;`TitleBar` 实现 TrafficLights 分支：左置圆形按钮
    （直径 12、间隙 8、前导 12、垂直居中，尺寸走 `theme.spacing_*` 近似取 token),
    LOGO + 标题顺排其后；hover 画深色符号（×/− 复用，+ 新增变体）；非 hover 无符号。
  - **单元测试清单（模块内）:**
    - [ ] TrafficLights 三个按钮 rect 位于左侧、顺序 close→minimize→maximize、y 垂直居中。
    - [ ] Standard 与 TrafficLights 的 `hit_button` 命中互不串位。
    - [ ] hover close 时 paint 产出红色圆 + × 符号矩形；非 hover 时只有圆无符号。
    - [ ] 绿灯点击产出注册在 `on_maximize` 上的消息。
    - [ ] `platform_default()` 在当前宿主机上返回预期值（cfg 分支冒烟）。
  - **Acceptance criteria:** 上述测试全绿；Theme token 被实际使用（无魔法颜色残留）。
  - **Verification:** `cargo test widget::title_bar` + 全量 `cargo test`。
  - **Dependencies:** Task 1
  - **Files:** `src/theme.rs`, `src/widget/title_bar.rs`
  - **Scope:** M

- [ ] **Task 3: tests/title_bar_window.rs 红绿灯集成测试**
  - **Description:** 扩展集成测试，显式 `.style(TitleBarStyle::TrafficLights)` 构造（绕开平台默认值，
    Windows 宿主机可跑）:左置三按钮分别产出 `Close`/`Minimize`/`MaximizeOrRestore`;
    按钮右侧空白区仍是拖拽区，双击仍产出 `MaximizeOrRestore`。
  - **Acceptance criteria:** 新增 4 条测试全绿；只使用 `danqing::widget::{...}` 平铺路径。
  - **Verification:** `cargo test --test title_bar_window` + 全量 `cargo test`。
  - **Dependencies:** Task 2
  - **Files:** `tests/title_bar_window.rs`
  - **Scope:** S

- [ ] **Checkpoint A: 组件层验收**
  - `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 全绿；
  - `cargo run --example showcase` Windows 人工冒烟：标题栏视觉/交互与现状完全一致。

### Phase 2: 平台层

- [ ] **Task 4: window.rs 全平台去装饰**
  - **Description:** 移除 `with_decorations(false)` 上的 `#[cfg(target_os = "windows")]` 门控，
    更新注释（去掉「其他平台保留原生标题栏作为降级」表述）;`apply_windows_undecorated_style`
    与其调用点保持 Windows 专属。
  - **Acceptance criteria:**
    - [ ] Windows 上 showcase 行为零回归（人工冒烟）。
    - [ ] `window.rs` 中不再存在「仅 Windows 去装饰」的 cfg 与注释。
  - **Verification:** `cargo test --lib --tests` 全绿 + showcase 冒烟。
  - **Dependencies:** None（与 Phase 1 无文件交集；单开发者串行排在 Checkpoint A 后）
  - **Files:** `src/window.rs`
  - **Scope:** S

- [ ] **Task 5: 跨平台编译检查**
  - **Description:** `rustup target add x86_64-unknown-linux-gnu x86_64-apple-darwin` 后分别
    `cargo check --target ...`;macOS 分支（TrafficLights 默认解析）首次真正被编译，
    修复暴露的 cfg 分支问题。若 apple-darwin std 安装失败（网络），降级为
    `cargo check --target x86_64-unknown-linux-gnu` + 人工审查 macOS cfg 分支，并在 todo 记录。
  - **Acceptance criteria:**
    - [ ] Linux target check 通过。
    - [ ] macOS target check 通过（或记录降级理由）。
  - **Verification:** 上述命令。
  - **Dependencies:** Task 2, Task 4（需全部 cfg 分支就绪）
  - **Files:** 视修复情况
  - **Scope:** S

- [ ] **Checkpoint B: 平台层验收**
  - 三平台编译检查结论明确；fmt + clippy + 全测试绿。

### Phase 3: 收尾

- [ ] **Task 6: 文档同步与最终验收**
  - **Description:** `docs/specs/title-bar-window-controls.md` 顶部加一行「Open Question 3 与
    Success Criteria 6 已被 `title-bar-cross-platform.md` 取代」;CLAUDE.md 中标题栏相关描述
    （如「Windows 使用自绘标题栏」类表述，若有）同步；README 涉及跨平台标题栏的描述同步。
  - **Acceptance criteria:**
    - [ ] `cargo fmt --check` / `cargo clippy --all-targets -- -D warnings` / `cargo test` 全绿。
    - [ ] 文档无「其他平台保留原生标题栏降级」残留表述。
  - **Verification:** 上述三条 + `grep -rn "降级" docs/ CLAUDE.md README.md` 无过期结论。
  - **Dependencies:** Checkpoint B
  - **Files:** `docs/specs/title-bar-window-controls.md`, `CLAUDE.md`, `README.md`
  - **Scope:** S

## Checkpoints

| 检查点 | 条件 | 验证命令 |
|---|---|---|
| A: 组件层验收 | Phase 1 完成，Standard 零回归 | fmt --check + clippy --all-targets -D warnings + cargo test + showcase 冒烟 |
| B: 平台层验收 | Phase 2 完成，三平台可编译 | cargo check --target {linux,macos} + fmt + clippy + cargo test |

## Risks and Mitigations

| 风险 | 等级 | 缓解 |
|---|---|---|
| 角色化重构引入 Standard 布局回归 | 中 | 现有单测（hover 背景位置、按钮命中）即回归锁，Task 1 要求零修改通过 |
| apple-darwin std 安装失败（网络） | 中 | Task 5 已定义降级路径并强制记录 |
| macOS/Linux 无真机，行为缺陷漏出 | 中 | 显式验收口径：编译 + 单测；真机问题按后续缺陷处理（spec 已声明） |
| macOS 无边框窗口阴影丢失（平台行为） | 低 | spec Open Question 记录，本次不处理 |
| 红绿灯尺寸写死像素与 theme 间距 token 冲突 | 低 | 优先取 `spacing_*` 近似值，像素规格仅作注释参考 |

## Parallelization Notes

Task 4(window.rs）与 Phase 1(title_bar.rs/theme.rs）文件无交集，理论可并行；
单开发者场景建议串行，按 1→2→3→4→5→6 推进。无其他并行车道。
