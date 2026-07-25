# Implementation Plan: 番茄钟 POC 补完(收口再开下一个 POC)

> 依据与 `dev` 分支当前会话对齐:番茄钟 POC 收口 5 个真缺(完成反馈 / 手动跳阶段 / 暂停视觉 / OS 级全局热键 / 状态持久化),完成后封档,启动下一个 POC(剪贴板历史管理器)。
> 本文档将工作拆为 **6 个可验证任务**,按依赖顺序组织。

## Overview

阶段 2 POC 已落地"最小番茄钟 + 场景沉浸",但其作为可日常使用的工具,缺失 5 处关键闭环:阶段结束无感知、无法手动跳阶段、暂停态视觉模糊、关闭后状态丢失、无 OS 级控制通道。本计划逐项补完,**目标是把番茄钟从"美学验证 POC"升级为"作者自己也能挂在任务栏日用的工具"**,完成后该 POC 封档,转向下一个 POC。

收口原则: 改动集中在 `examples/pomodoro/` 与 `src/window.rs`(OS 级热键必须触达 OS API,CLAUDE.md 明确允许);框架核心(`src/widget/`、`src/theme.rs`、`src/render/`、状态机)尽量不动。完成反馈与持久化采用 OS 标准能力(Win32 `MessageBeep` + `dirs::config_dir()`),不引入额外资产。

## Architecture Decisions

- **持久化用 JSON,平台标准配置目录**: `serde` + `serde_json`(都已隐式随 `anyhow` 引入附近;如未引入,在 danqing 顶层 `Cargo.toml` 加 `serde = { version = "1", features = ["derive"] }` + `serde_json = "1"`);文件路径 `dirs::config_dir()?.join("danqing/pomodoro.json")`(Windows 下解析为 `%APPDATA%/danqing/pomodoro.json`);每次 `update` 写盘但每秒最多 1 次(去抖,避免 60fps 写 4 次 JSON);关闭时强制 flush。**为何不每帧写**: 计时精度损失 ≤ 1 秒,与 25 分钟量级相比无感,且 IO 与磁盘压力不可接受。
- **持久化字段 = 重启恢复的最小集**: `PomodoroState { phase, run, remaining, current_scene, saved_elapsed, saved_wall_clock }`。Running 状态下,启动时按 `now = saved_elapsed + (current_wall - saved_wall)` 重新注入,`deadline = now + remaining`,允许 running 跨重启不丢时间。Paused / Idle 状态下 `remaining` 原样恢复,运行时长重置。
- **OS 级全局热键用 `windows-sys`,不走 raw-dylib**: `windows-sys = { version = "0.59", features = ["Win32_UI_Input_KeyboardAndPoint", "Win32_UI_WindowsAndMessaging"] }`,`RegisterHotKey` / `UnregisterHotKey` 静态绑定,规避 CLAUDE.md 强调的 windows-gnu raw-dylib 风险。Mac/Linux 暂 stub(`#[cfg(not(target_os = "windows"))]` 路径返回 `Ok(())` 不注册,日志一行 `global hotkeys unsupported on this platform`)。
- **关闭窗口 = 隐藏到任务栏,不是退出**: 与 OS 级热键配套。热键的作用域是"应用活着、窗口不一定可见"。关闭按钮 → `window.set_visible(false)`;热键 `Ctrl+Shift+P` → 在 visible/invisible 之间切换。退出仅在最后一个窗口真正销毁时(进程级)发生,winit 默认行为即可。
- **完成反馈默认走 `MessageBeep` 零资产方案**: `windows-sys` 的 `MessageBeep(0x00000040 /* MB_ICONASTERISK */)`(或 `MB_OK`);如用户后续提供 `assets/sound/complete.wav`,可一键切换。无 WAV 资产、无音频管线负担,与"声音只是提示"语义一致。
- **完成反馈视觉用全屏调色板脉冲**: 不引入新组件,加一个 `feedback::flash_overlay(t, &palette)` 工具函数,返回 `Node`(全屏 `Box` + `palette.accent` 底色 + 透明度随 `t` 衰减),在 widget 树根叠加进 `view()`;`flash` 状态由 `PomodoroApp` 持有(`flash_started: Option<Duration>`),`tick` 检测 `timer.tick()` 返回 true 时启动,600ms 后自动清空。
- **手动跳阶段 = `Pomodoro::skip(now)` 加方法**: 语义:"立即结束当前阶段,开始下一个";Running 状态下更新 `deadline = now + next_phase.duration`,Paused 状态下更新 `remaining = next_phase.duration`;不动 run 状态;测试覆盖 Running/Paused/Idle 三态。
- **暂停视觉 = bind 调色板降饱和 + 倒计时降透明度**: 暂停时 `palette = base_palette.desaturate(0.5)`(或 `Color::gray_blend(palette, 0.5)` 通用工具),倒计时文本加 `opacity: 0.6`;不引入新组件,不引入新调色板字段;所有走 `bind_color` 的 widget 自动跟随。
- **持久化与热键的 Cargo 依赖**: `serde` + `serde_json`(仅 example 需要,但放顶层 `Cargo.toml` 也合理——选顶层以便将来其它 POC 复用);`windows-sys`(target_os = "windows" 条件依赖);`dirs = "5"`(仅 example 依赖或顶层;选顶层)。总增加 3 个依赖,均轻量、广泛使用、零 native 二进制。
- **持久化路径与目录约定**: 沿用 CLAUDE.md 的"资产统一放 `assets/`"原则,但**状态文件不是资产**(运行时生成、用户特定、不进版本控制),放 OS 配置目录。不在 `dirs` 不可用时崩溃(`Option<PathBuf>` + silent skip + 日志 warn 一行)。

## Dependency Graph

```
Task 1  持久化 (PomodoroState + save/load + 去抖 + 路径)
 ├─ Task 2  手动跳阶段 (Pomodoro::skip + Skip 按钮)
 ├─ Task 3  暂停视觉 (palette desat + 倒计时 opacity)
 ├─ Task 4  完成反馈 (visual flash + MessageBeep)
 └─ Task 5  OS 级全局热键 + 窗口显隐 (windows-sys + WM_HOTKEY)
     └─ Task 6  终验与封档
```

关键路径: 1 → 5 → 6。
并行车道: Task 2 / Task 3 / Task 4 互相独立,可在 Task 1 完成后并行实施。

## Task List

### Phase 1: 持久化基础

- [ ] **Task 1: 持久化 — PodomoroState + save/load + 路径 + 去抖**
  - **Description:** 新建 `examples/pomodoro/state.rs`:`PomodoroState` 结构体(`phase: Phase`、`run: RunState`、`remaining: Duration`、`current_scene: usize`、`saved_elapsed: Duration`、`saved_wall_clock: SystemTime`),派生 `Serialize`/`Deserialize`;`RunState` 复制 `timer.rs` 的 `Run` 但要 `pub` + `Serialize`/`Deserialize`(提取枚举到 state.rs,或保持私有但在 state.rs 平行定义一份 `enum RunState`,转换函数);函数 `state_path() -> Option<PathBuf>`(走 `dirs::config_dir()` + `danqing/pomodoro.json`,失败 warn + skip);`save_state(state: &PomodoroState) -> anyhow::Result<()>`(原子写:写临时文件 + rename);`load_state() -> Option<PomodoroState>`(文件不存在或解析失败 → `None` + warn)。`PomodoroApp` 集成: 增加 `state_dirty: bool` 旗标 + `last_save_at: Duration`;`update` 在状态变更后置 `state_dirty = true`;`tick` 检查 `state_dirty && (now - last_save_at) >= 1s` 时 save;`run()` 退出前 flush 一次。Load: `PomodoroApp::new()` 优先用持久化状态构造 timer,无则新建 25:00 Idle;恢复时若 `run == Running`,计算 `effective_now = saved_elapsed + (now_wall - saved_wall_clock)`,按 `effective_now` 注入恢复 deadline。
  - **Acceptance criteria:**
    - [ ] `PomodoroState` 字段齐全,JSON 格式可读(serde_json `pretty = false` 但键名清晰)。
    - [ ] 正常路径:启动 → 走 5 秒 → 暂停 → 关闭 → 重新启动,状态完整恢复(剩余时间误差 ≤ 1s)。
    - [ ] Running 状态跨重启:启动 → 走 5 秒 → 关闭 → 3 秒后启动,剩余时间 = 25:00 - 5 - 3 = 24:52(允许 ±1s 误差)。
    - [ ] 配置文件不存在 / 解析失败 → 退回默认 25:00 Idle,不 panic。
    - [ ] `dirs::config_dir()` 不可用 → warn 日志一行 + 跳过持久化,内存模式仍可用。
    - [ ] 写盘每秒最多 1 次(grep 测试或节流代码注释可见)。
    - [ ] 单元测试: `PomodoroState` 序列化往返;Run state 转换函数正确;`state_path` 在 mock 目录下返回预期路径。
  - **Verification:** `cargo test --lib --tests` + `cargo test --example pomodoro` 全绿;`cargo run --example pomodoro` 手动跑"启动→5s→关闭→重启"流程,日志 + 实际状态文件确认。
  - **Dependencies:** None
  - **Files:** `examples/pomodoro/state.rs`(新), `examples/pomodoro/timer.rs`(`Run` 提取或平行), `examples/pomodoro/main.rs`, `src/window.rs`(退出时 flush 钩子), `Cargo.toml`(`serde`, `serde_json`, `dirs`)
  - **Scope:** M

### ⏸ Checkpoint 1: 持久化就绪
- [ ] `cargo test --example pomodoro` 绿(含 state 单元测试)
- [ ] 手动验: 启动 → 暂停 → 关闭 → 重启,状态完整恢复

### Phase 2: 体力小件 + 完成反馈(可并行)

- [ ] **Task 2: 手动跳阶段 — `Pomodoro::skip` + Skip 按钮**
  - **Description:** `timer.rs` 增加方法 `pub fn skip(&mut self, now: Duration) -> bool`: 若 `run == Running`,推进 `phase = next()`,`deadline = now + next.duration`;若 `run == Paused`,推进 `phase = next()`,`remaining = next.duration`;若 `run == Idle`,仅切换 `phase` 与 `remaining`(语义:开始新阶段但仍停);返回 `phase` 是否发生变更。`main.rs` 增加 `Msg::Skip` 与"跳"按钮(放底部胶囊,介于"开始/暂停"与"重置"之间,或独立一行——设计决定;默认与重置并列,主按钮用 ghost 样式);场景化主题同样接入(`&t` 入参 + `bind_color`)。
  - **Acceptance criteria:**
    - [ ] `skip` 在 Running/Paused/Idle 三态下语义正确,`phase` 切换。
    - [ ] Running 下 skip 后续时刻计时从新阶段满量开始,不继承旧阶段剩余。
    - [ ] Skip 按钮样式与 ghost 按钮一致,主题色跟随场景。
    - [ ] 单元测试: 三态 skip 各自覆盖;连续 skip 多次正常流转。
  - **Verification:** `cargo test --example pomodoro` 绿;手动按 Skip 按钮,场景跳转 + 计时从新阶段满量开始。
  - **Dependencies:** Task 1(Skip 也是状态变更,需持久化)
  - **Files:** `examples/pomodoro/timer.rs`, `examples/pomodoro/main.rs`
  - **Scope:** S

- [ ] **Task 3: 暂停视觉 — palette 降饱和 + 倒计时 opacity**
  - **Description:** `ScenePalette` 不增加字段;`PomodoroApp::palette()` 在 `timer.is_running() == false` 时返回 `palette.desaturate(0.5)`(或现有工具 `Color::gray_blend`);`Color::desaturate(factor)` 通用工具: `mix(gray, factor)`,`factor=0` 保留原色,`factor=1` 全灰;放在 `src/theme.rs` 或 `src/layout.rs`(选 `layout.rs` 与 `Color::lerp` 同居)。倒计时 `Text::bind(...)` 加 `.bind_alpha(|s: &PomodoroApp| if s.timer.is_running() { 1.0 } else { 0.6 })`(若 `Text` 不支持 --alpha,降级方案: 暂停时改倒计时颜色为 `palette.text_secondary`——优先 bind_alpha,失败再降级,记录决策)。TitleBar 主题绑定已存在,因 `palette()` 改变,标题栏会自动降饱和,无需额外改动。
  - **Acceptance criteria:**
    - [ ] 暂停时,场景图、控件、所有 bind 颜色同步降饱和,UI 整体灰度变深。
    - [ ] 倒计时文字透明度降低,在 5 场景下肉眼可见暂停态。
    - [ ] Resume 立即恢复满饱和 + 满透明度,无中间态残留。
    - [ ] `Color::desaturate` 端点正确: factor=0 恒等、factor=1 全灰,clamp 0..1。
  - **Verification:** `cargo test --lib layout` 绿;手动按暂停,所有 UI 灰度加深;按开始,完全恢复。
  - **Dependencies:** None(独立于 Task 1)
  - **Files:** `src/layout.rs`(`Color::desaturate`), `examples/pomodoro/main.rs`(palette 与 alpha 绑定)
  - **Scope:** S

- [ ] **Task 4: 完成反馈 — 视觉 flash + 音效**
  - **Description:** 新建 `examples/pomodoro/feedback.rs`:`FlashOverlay` 结构(`started: Option<Duration>`, `duration: Duration = 600ms`),方法 `trigger(&mut self, now: Duration)` / `frame(&self, now: Duration) -> Option<f32>`(返回 0..1 衰减进度,`None` 表示未激活);`flash_overlay_widget(t: SceneTheme, progress: f32) -> impl Widget`,全屏 `Box` + `palette.accent` 底色 + alpha = progress(头部满 alpha,尾部 linear 衰减到 0)。`PomodoroApp` 持有 `flash: FlashOverlay`;`tick` 监听 `timer.tick()` 返回 true(阶段流转)时调 `flash.trigger(now)`;若 `flash` 触发,调 `audio::beep()`。`audio::beep()` 模块: Windows 下 `windows-sys` `MessageBeep(0x00000040)`,其它平台 stub(返回 `Ok(())`);`view()` 末尾条件追加 `flash_overlay_widget(t, frame)` 节点(若 `frame.is_some()`)。Audio 与 visual 共享同一触发点,无 race。
  - **Acceptance criteria:**
    - [ ] 阶段跨过终点时,屏幕有 ~600ms 全屏脉冲(accent 色),与场景色调和谐。
    - [ ] 同时触发系统蜂鸣,Windows 下可闻。
    - [ ] 连续多次跨过(如 long overshoot)不会重叠触发新脉冲;`FlashOverlay` 在进行中不会被新触发覆盖(避免视觉抖)。
    - [ ] Flash 期间不影响其它 UI 交互(底色透明叠在控件之上,点击穿透)。
    - [ ] 单元测试: `FlashOverlay` 进度端点、trigger 行为。
  - **Verification:** `cargo test --example pomodoro` 绿;手动拨快 25:00 触发跳过(用 `tick(secs(25*60+1))` 调一次或单元测试模拟),看到 flash + 听到 beep。
  - **Dependencies:** None(独立于 Task 1)
  - **Files:** `examples/pomodoro/feedback.rs`(新), `examples/pomodoro/audio.rs`(新), `examples/pomodoro/main.rs`
  - **Scope:** S

### ⏸ Checkpoint 2: 三件小修完成
- [ ] `cargo test --example pomodoro` 全绿
- [ ] 手动验: Skip 跳转、暂停灰度、阶段结束 flash + beep 全部就绪

### Phase 3: OS 级全局热键(重头戏)

- [ ] **Task 5: OS 级全局热键 + 窗口显隐**
  - **Description:** `Cargo.toml` 加 `windows-sys = { version = "0.59", features = ["Win32_UI_Input_KeyboardAndPoint", "Win32_UI_WindowsAndMessaging"] }`(target_os = "windows" 条件依赖)。`src/window.rs` 增加模块: `register_hotkeys(hwnd) -> HotkeyHandle` / `unregister_hotkeys(handle)`;`HotkeyId` 枚举 `{ ToggleVisible = 1, StartPause = 2 }`;Windows 实现: `RegisterHotKey(hwnd, 1, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 0x50 /* P */)` + `RegisterHotKey(hwnd, 2, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 0x53 /* S */)`;`unregister_hotkeys` 调 `UnregisterHotKey`。`src/window.rs` 在 `EventLoopWindowTarget` 消息循环中加 `WM_HOTKEY` 处理: 解析 `wparam` → `Msg::ToggleVisible` 或 `Msg::StartPause`,注入到 App 消息队列。`App` trait 增加默认方法 `fn window_hotkey(&mut self, id: u8) -> Option<Msg>`(默认 `None`);`PomodoroApp` 实现: id=1 → `Some(Msg::ToggleVisible)`,id=2 → `Some(Msg::StartPause)`。`Msg::ToggleVisible` 在 `update` 中: 切换 `is_visible: bool` 旗标,调 `window.set_visible(flag)`(需要 window.rs 暴露给 App 一个句柄或 channel,见下);`Msg::StartPause` 复用现有 `toggle` 逻辑。window.rs 暴露的 channel: `App` trait 现有 `update(msg: Msg)` 不接受 `&Window`,新增 `fn on_window_event(&mut self, event: WindowAppEvent)` 默认 `()`,`WindowAppEvent::SetVisible(bool)` —— 即 App 不直接调 window,而是通过消息发;window.rs 持有 `Sender<WindowAppEvent>` 通过 `event_loop.run` 闭包注入 App。窗口显隐 = `is_visible` 旗标 + 每帧检查,隐藏时跳过 `request_redraw`。
  - **Acceptance criteria:**
    - [ ] 应用启动时 `RegisterHotKey` 双热键成功(handle 非零);退出时 `UnregisterHotKey` 清理,无 handle 泄漏。
    - [ ] `Ctrl+Shift+P`: 应用已隐藏 → 显示并置顶;已显示 → 隐藏到任务栏;不影响焦点窗口(全局)。
    - [ ] `Ctrl+Shift+S`: 在任何窗口聚焦时都能开始/暂停番茄钟,无需切到番茄钟窗口。
    - [ ] 关闭按钮 = 隐藏(不退出进程);Ctrl+Q 或 Cmd+Q 仍走系统默认退出。
    - [ ] Mac/Linux 编译通过,日志一行 `global hotkeys unsupported on this platform`,运行不 panic。
    - [ ] 嵌套启动 2 个番茄钟: 第二个实例检测到 `RegisterHotKey` 失败 → 不注册、warn、报告用户(单实例行为,可选;MVP 不做但要测不致命)。
    - [ ] 单元测试: `HotkeyId` 映射正确;Mac/Linux stub 不 panic。
  - **Verification:** `cargo build --release --example pomodoro` 绿;`cargo clippy -- -D warnings` 零警告;手动: 启动 → 最小化 → 按 Ctrl+Shift+P 弹出 → 按 Ctrl+Shift+S 暂停(此时焦点在浏览器)→ 切到番茄钟看状态。
  - **Dependencies:** Task 1(windows-sys 引入为持久化同期,本任务大量使用)
  - **Files:** `Cargo.toml`, `src/window.rs`, `src/app.rs`, `src/lib.rs`(re-export), `examples/pomodoro/main.rs`
  - **Scope:** L

### ⏸ Checkpoint 3: 全局热键就绪
- [ ] `cargo build --release` 绿;`cargo clippy -- -D warnings` 零警告
- [ ] 手动验: 全局 Ctrl+Shift+P / Ctrl+Shift+S 在多个应用聚焦时生效
- [ ] 关闭按钮 = 隐藏到任务栏,进程不退出

### Phase 4: 终验与封档

- [ ] **Task 6: 终验、文档收口与封档**
  - **Description:** 跑全部命令: `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` + `cargo test --example pomodoro` + `DANQING_WGPU_VALIDATION=1 cargo run --example pomodoro`(验证无 GPU 错误);按 spec 验收清单逐条核: 25/5 / 阶段自动流转 / 场景切换 / 持久化恢复 / 全局热键 / 完成反馈 / 暂停视觉 / 跳过阶段;`tasks/todo-pomodoro-completion.md` 全部勾选;`CLAUDE.md` "下一步"改写为"启动下一个 POC: 剪贴板历史管理器"并 link to `docs/ideas/danqing-efficiency-tool-glassmorphism.md`;把 `plan-pomodoro-completion.md` + `todo-pomodoro-completion.md` 移入 `tasks/archive/`(按现有归档约定);新建 `docs/specs/pomodoro-completion.md` 记录封档事实(成功标准 + 实施链接);最后 git commit + push(若用户授权)。
  - **Acceptance criteria:**
    - [ ] 全部命令绿(0 format 错误,0 clippy warning,全部测试 pass)。
    - [ ] 5 个真缺全部修复,人工验收 5/5 通过。
    - [ ] `CLAUDE.md` 反映封档与下一步。
    - [ ] 文档归档完成,仓库结构整洁。
  - **Verification:** 上述验收 + 人工终审。
  - **Dependencies:** Task 1, 2, 3, 4, 5
  - **Files:** `CLAUDE.md`, `tasks/archive/plan-pomodoro-completion.md`, `tasks/archive/todo-pomodoro-completion.md`, `docs/specs/pomodoro-completion.md`(新)
  - **Scope:** S

### ✅ Checkpoint Complete: 番茄钟 POC 封档
- [ ] 5 个真缺全部修复并人工验收
- [ ] 全部 Commands 绿
- [ ] 文档归档完成,CLAUDE.md 指向下一个 POC
- [ ] 准备启动剪贴板历史管理器 POC(需用户另行指示开始)

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| `serde` / `serde_json` / `dirs` 引入与 windows-gnu 工具链冲突 | 编译失败或 raw-dylib 错误 | 三者均为纯 Rust crate,无 native 依赖;windows-gnu 风险仅来自 `windows-sys`(`-sys` 后缀代表纯绑定,无链接问题);Task 1 早期编译验证 |
| `MessageBeep` 在某些 Windows 版本无声音 | 反馈失效 | 视觉脉冲仍是可靠反馈,音频只是叠加;另写一行日志告知用户触发;提供后续替换为 WAV 的路径 |
| `RegisterHotKey` 与系统/其它应用热键冲突 | 启动注册失败 → 无全局热键 | 失败不 panic,降级为仅 in-app 快捷键,日志 warn;若用户报告,后续提供热键自定义入口 |
| 持久化 1Hz 节流下,关电脑/强杀进程丢失 ≤ 1s 计时 | 体验不完美 | 25 分钟量级下可接受;关闭/暂停/切场景等关键节点前必 flush;若用户在意,后续可加 ctrl-c handler |
| 番茄钟与 OS 共享热键误触 | 误操作 | 选 `Ctrl+Shift+P` / `Ctrl+Shift+S` 是冷僻组合;后续可暴露设置页热键自定义 |
| `窗口显隐` + `request_redraw` 死循环 | CPU 100% | 显隐状态变化时才 `request_redraw`,持续 visible 走既有每帧节流;测试时监控 CPU |
| WM_HOTKEY 在 winit 0.30 event loop 中的正确接收 | 静默不触发 | 走 `EventLoopWindowTarget::pump_message` 或 `with_os_proc` 自定义,Task 5 优先验证机制;若 winit 屏蔽,可能需要 `winapi` 直接 `GetMessage` 旁路 |
| Mac/Linux 用户看到"global hotkeys unsupported"但功能缺一半 | 体验不一致 | 任务范围明确 Windows 优先;文档标注;Mac/Linux 后续单独规划 |
| `Text::bind_alpha` 不存在 | 暂停视觉降级方案 | 提前 grep `bind_alpha` 确认;若无,改 `bind_color` 配 `Color::with_alpha` 或 SceneTheme 派生字段 |
| 持久化路径在某些 sandbox/portable 模式下不可写 | 持久化静默失效 | `state_path()` 返回 `Option`,`save` 失败时降级为内存模式 + warn 日志,POC 仍可用 |

## Open Questions

1. **`WM_HOTKEY` 在 winit 0.30 中的接收机制**: Task 5 第一件事,验证不成立 → 退路是 `event_loop.run` 用 `with_os_proc` 注入 `TranslateMessage` / `DispatchMessage` 拦截。若两条都失败,退路是 in-app 快捷键 + 用户手动切窗口(放弃 OS 级,POC 范围降一档)。
2. **`Text::bind_alpha` 是否存在**: Task 3 第一件事,降级是 `bind_color` + `Color::with_alpha`。
3. **关闭按钮保留 vs 替换为"隐藏"按钮**: 当前 X 按钮 = 关闭进程。Task 5 改造后行为变更,用户是否要在 UI 上加文字提示(如 tooltip "关闭窗口会隐藏到任务栏")?
4. **持久化文件名 / 目录**: `danqing/pomodoro.json` 在 `%APPDATA%/danqing/` 下,符合约定;若用户希望放 `LOCALAPPDATA`(local, 不漫游)还是 `APPDATA`(roaming, 跨设备),后者更稳,选 APPDATA。
5. **WAV 资产 vs MessageBeep 决策**: 默认 MessageBeep,前提是用户接受"Windows 默认 beep 声"作为完成提示;若用户想自定义,后续可在 `assets/sound/` 加 WAV 并改 `audio::beep()` 实现。
