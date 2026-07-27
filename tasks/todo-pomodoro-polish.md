# Todo: 番茄钟打磨三件套(环境音 / 长休息+轮次 / 今日计数)

> 详见 `tasks/plan-pomodoro-polish.md`(验收标准、依赖、风险)。
> 依据 `docs/specs/pomodoro-polish.md`(2026-07-27 确认)。

## Phase 1: WS2 长休息 + 轮次
- [x] **Task 1** `timer.rs` — `Phase::LongBreak` (15min) + 轮次计数 `completed_focus` (0..4) + `tick` 返回值升级 `TickReport { advanced, focus_completions }`; skip 不推进轮次; 既有测试适配 + 新增 ≥6 测试
- [x] **Task 2** 持久化 `completed_focus` (`#[serde(default)]` 兼容旧 JSON) + 副标轮次显示 (`第 N/4 轮` 仅 Focus 相位; `长休息` label) (依赖 1)

### ⏸ Checkpoint 1: 长休息 + 轮次就绪
- [x] `cargo test --example pomodoro` 全绿; fmt + clippy 零警告
- [ ] 手动验: skip 连按 7 次走一遍 4 轮相位流转, 副标轮次正确
- [x] 提交 Phase 1

## Phase 2: WS3 今日完成计数
- [x] **Task 3** 今日计数纯逻辑 — `today_string()` (复用已有 dev-dep chrono `Local`) + `resolve_today_count` 跨日归零判定 + 单元测试
- [x] **Task 4** 持久化 `today_date`/`today_count` + tick 计数接线 (`focus_completions` 累加, skip 不计) + 副标「今日 N」(N≥1 才显示) (依赖 1, 3)

### ⏸ Checkpoint 2: 今日计数就绪
- [x] `cargo test --example pomodoro` 全绿; fmt + clippy 零警告
- [ ] 手动验: 完成一个 Focus (或构造 state), 副标显示「今日 1」
- [x] 提交 Phase 2

## Phase 3: WS1 场景环境音(重头戏)
- [x] **Task 5** 音源资产 — 5 场景 CC0 音源 (OpenGameArt CC0) → OGG ≤2MB × 5, 循环点 50ms 微 crossfade; `assets/audio/` + `ATTRIBUTION.md`; 资产护栏测试 (可与 Task 6 并行)
- [x] **Task 6** `ambient.rs` — `AmbientMixer` 纯逻辑 (淡化插值 × 暂停 300ms 包络, 目标音量 0.6) + `SCENE_AUDIO` 平行数组 + ≥6 单元测试 (可与 Task 5 并行)
- [x] **Task 7** rodio 接入 — dev-dep rodio (0.22: `DeviceSinkBuilder`/`Player`, 精简特性 ogg+vorbis); 2 槽 Player (from/to, 对齐视觉 LRU) + `repeat_infinite`; 懒初始化 + 静默降级; `tick` 一行接线 (依赖 5, 6)

### ⏸ Checkpoint 3: 环境音就绪
- [ ] 5 场景音景人工听感验收 (循环接缝 / 交叉淡化 / 暂停沉降)
- [ ] 降级路径 (设备占用 / 文件缺失) 不 panic
- [ ] 提交 Phase 3

## Phase 4: 终验与收口
- [ ] **Task 8** benchmark 双门槛 (启动 ≤1s, WS ≤360MB) + 人工终审 + spec 勾选 + 归档 + CLAUDE.md 更新 (依赖 1-7)

### ✅ Checkpoint Complete: 三件套封档
- [ ] 三个 WS 全部人工验收通过
- [ ] 全部命令绿: `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` + `cargo test --example pomodoro` + `cargo build --release`
- [ ] benchmark 双门槛达标
- [ ] 文档归档完成, CLAUDE.md 指向下一候选 (第二 POC 剪贴板历史管理器或用户指定)
