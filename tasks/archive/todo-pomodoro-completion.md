# Todo: 番茄钟 POC 补完(收口再开下一个 POC) ✅ 已关闭

> 详见 `tasks/archive/plan-pomodoro-completion.md`(验收标准、依赖、风险)。
> 实施完毕, 全部阶段已通过, 关闭封档。

## Phase 1: 持久化基础
- [x] **Task 1** 持久化 `PomodoroState` + save/load + 路径 + 1Hz 去抖 — `Cargo.toml` 加 `serde` + `serde_json` + `dirs`;`state.rs` 序列化/反序列化 + 配置目录 + 原子写;Running 状态跨重启恢复

### ⏸ Checkpoint 1: 持久化就绪 ✅
- [x] `cargo test --example pomodoro` 绿(含 state 单元测试)
- [x] 手动验: 启动 → 5s → 暂停 → 关闭 → 重启, 状态完整恢复

## Phase 2: 体力小件 + 完成反馈(可并行)
- [x] **Task 2** 手动跳阶段 — `Pomodoro::skip` + Skip 按钮(依赖 1)
- [x] **Task 3** 暂停视觉 — `Color::desaturate` + palette 降饱和 + 倒计时切 `text_secondary` (`bind_alpha` 不存在, 降级 `bind_color`)
- [x] **Task 4** 完成反馈 — 视觉 flash(`FlashOverlay` + `Stack` widget)+ 音效(`MessageBeep`)

### ⏸ Checkpoint 2: 三件小修完成 ✅
- [x] `cargo test --example pomodoro` 全绿
- [x] 手动验: Skip 跳转、暂停灰度、阶段结束 flash + beep 全部就绪

## Phase 3: OS 级全局热键(重头戏)
- [x] **Task 5** OS 级全局热键 + 窗口显隐(依赖 1)— `windows-sys` 绑定 `RegisterHotKey`;`Ctrl+Shift+P` 显隐 / `Ctrl+Shift+S` 开始暂停 / `Ctrl+Shift+Q` 退出;关闭按钮 = 隐藏;Mac/Linux stub

### ⏸ Checkpoint 3: 全局热键就绪 ✅
- [x] `cargo build --release` 绿;`cargo clippy -- -D warnings` 零警告
- [x] 手动验: 全局热键在多个应用聚焦时生效
- [x] 关闭按钮 = 隐藏到任务栏, 进程不退出

## Phase 4: 终验与封档
- [x] **Task 6** 终验、文档收口与封档(依赖 1-5)— 全部命令绿;CLAUDE.md 改"下一步";plan/todo 归档 `tasks/archive/`;新建 `docs/specs/pomodoro-completion.md`

### ✅ Checkpoint Complete: 番茄钟 POC 封档
- [x] 5 个真缺全部修复并人工验收
- [x] 全部 Commands 绿: `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` + `cargo test --example pomodoro` + `cargo build --release`
- [x] 文档归档完成, CLAUDE.md 指向下一个 POC
- [x] 准备启动剪贴板历史管理器 POC(需用户另行指示开始)

## 五项真缺修复一览

| # | 缺口 | 修复 | 文件 |
|---|------|------|------|
| 1 | 完成反馈缺失 | `FlashOverlay` 全屏 accent 脉冲 + `MessageBeep` 蜂鸣 | `examples/pomodoro/{flash,audio}.rs` 新建 |
| 2 | 无手动跳阶段 | `Pomodoro::skip` + Skip 按钮 | `examples/pomodoro/{timer,main}.rs` |
| 3 | 暂停视觉不明显 | `Color::desaturate` + `ScenePalette::desaturate` 整体降饱和 + 倒计时切 `text_secondary` | `src/{layout,theme}.rs` + `examples/pomodoro/main.rs` |
| 4 | 无 OS 级控制 | `windows-sys` `RegisterHotKey` 线程 + `WindowEventSender` 通道 + `Ctrl+Shift+P/S/Q` | `src/{window,app,lib}.rs` + `examples/pomodoro/main.rs` |
| 5 | 关闭后状态丢失 | `PomodoroState` JSON 持久化 + Running 跨重启按 wall-clock 偏移恢复 + 1Hz 节流 | `examples/pomodoro/{state,main}.rs` + `src/{app,window}.rs` |

## 新增 / 变更文件

- **新增**:
  - `examples/pomodoro/state.rs` — 持久化快照 + 路径 + 原子写
  - `examples/pomodoro/flash.rs` — 视觉脉冲状态机
  - `examples/pomodoro/audio.rs` — Windows beep 包装
  - `src/widget/layout/stack.rs` — 层叠容器 (flash 落地)
- **变更**:
  - `examples/pomodoro/timer.rs` — `Run` pub + `Phase` serde 派生 + `restore` + `skip`
  - `examples/pomodoro/main.rs` — 集成 5 项 (skip 按钮, palette 降饱和, flash + beep, WindowEventSender, state 加载/退出 flush)
  - `src/window.rs` — `WindowAppEvent` + `WindowEventSender` + `hotkeys` 子模块 + `about_to_wait` 钩子
  - `src/app.rs` — `boot_elapsed_offset` + `attach_window_sender` + `hotkey` 默认实现
  - `src/layout.rs` — `Color::desaturate`
  - `src/theme.rs` — `ScenePalette::desaturate`
  - `src/lib.rs` — re-export `WindowAppEvent` / `WindowEventSender` / `hotkey_ids`
  - `Cargo.toml` — `windows-sys` 主依赖 (条件 feature)
