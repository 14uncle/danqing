# Plan: anim 动画原语模块 (框架下沉·簇A)

> Spec: `docs/specs/SPEC-anim.md` (已批准, 2026-09-09)。Todo: `tasks/todo-anim.md`。
> 需求来源: `docs/intent/framework-sinking.md` 簇A。

## 决策记录

- **D1 · Tween 语义采 ChannelEnvelope 目标追踪式**：`needs_anim` 判定（目标≠在途动画目标 或 当前值未达目标）同时覆盖边沿触发与逐帧微变跟随（时辰曲线范式）；MotionEnvelope 的 bool 边沿检测是其子集。
- **D2 · 续接统一走全程 duration**：舍弃 switch 的剩余距离比例耗时。switch 场景目标只有 0/1，中途翻转最差情况 = 从 0.5 走全程 150ms，观感无差。
- **D3 · switch 时间源用 `AnimationCtx.elapsed`（Duration）**：`ctx.now: Instant` 与 Tween 的注入 Duration 时间轴不同型；`elapsed` 字段现成（`app.rs:24`），无需自维护 epoch。
- **D4 · Easing 原地扩展不搬家**：`theme.rs` 加 `EaseIn = t³` / `EaseOut = 1-(1-t)³`；Easing 是设计 token，住在 theme 正确。pomodoro hint.rs 的两个私有三次方函数随之消灭。
- **D5 · 模块形态仿 `app`**：`mod anim;` 私有 + `lib.rs` `pub use anim::{Tween, Pulse, Cue, CueTiming, Crossfade};`，不许路径深穿。
- **D6 · pomodoro lock 重解走 `cargo check` 驱动**：不用 `cargo update -p danqing`（2026-09-08 实测会把 windows 边错配炸 wgpu-hal，见用户级记忆 danqing-dep-lock-minimal-reresolve）。
- **D7 · commit/push 等用户指示**：plan 中标注联动点（T8 前 danqing 必须先 push），默认全程不 commit（/build auto 先例）。

## 依赖图

```
T1 Easing 扩展 ──┐
                 ├─→ T3 Pulse+Cue ─┐
T2 Tween ────────┼─────────────────┼─→ T5 mixer 迁移 ─┐
                 └─→ T4 Crossfade ─┘                  ├─→ T7 showcase 卡 ──→ ★CP2
                       (T4 独立)     T6 switch 迁移 ───┘        │
                                                                ↓ (danqing push 后)
                                            T8 pomodoro 迁移 ──→ T9 回归 ──→ ★CP3
```

T1–T4 为纯加法（框架多一个模块，零既有行为变化）；T5/T6 是框架内消费者换芯；T8/T9 跨仓联动。

## 任务清单

### Phase 1: 框架纯加法（danqing）

**T1: Easing 扩展** — theme.rs 加 `EaseIn`/`EaseOut` 三次方变体 + eval 分支；测试：端点精确 0/1、单调、中点方向性（EaseIn 中点 <0.5，EaseOut 中点 >0.5，与 hint.rs 现语义一致）。| S

**T2: `Tween`** — anim.rs 建档（文件头+模块头 doc：做什么/不做什么/为什么）+ Tween 实现 + lib.rs re-export。测试：移植 MotionEnvelope 3 条（idle 恒零/淡入中点终点/暂停续接）改写为新 API + 新增 easing 整形（EaseInOut 中点 0.5、1/4 点更低）、持续微变平滑跟随、任意值域不夹取、is_animating。| S

**T3: `Pulse` + `Cue`** — 移植 flash.rs/hint.rs 语义；Cue 四段时长入 `CueTiming`（Default = 1.5s/300ms/5s/500ms），淡入 EaseOut/淡出 EaseIn 经 T1 变体。测试：移植 6+7 条，Cue 加非默认 timing 一条。| S（依赖 T1）

**T4: `Crossfade`** — 移植 fader.rs 语义（usize 索引、≥0.5 占优吸附）；`frame(now, easing: Easing)` 签名从闭包改枚举。测试：移植 8 条。| S

**★ Checkpoint 1**: 三件套绿（`cargo fmt` / `clippy -- -D warnings` / `cargo test --lib --tests`）；纯加法，既有测试零改动全绿。

### Phase 2: 框架内消费者换芯（danqing）

**T5: mixer 迁移** — 删私有 `ChannelEnvelope`，`Mixer` 内部改 `Tween::new(300ms, Linear)`；0..1 夹取留在 `set_target`。**验收硬判据：mixer 既有 8 测试零改动全绿**（行为不变 = 迁移正确）。| S

**T6: switch 迁移** — `animate()` dt 步进（`switch.rs:154-168`）改 Tween（150ms / Linear，时间源 `ctx.elapsed`）；删 `last_time` 字段与 `ANIM_DURATION_S`；`anim_progress` 读值改 `tween.value(elapsed, anim_target)`。| S

**★ Checkpoint 2**: 三件套绿；showcase 实机人工过目 switch 观感（拨动/快速双击翻转）。

### Phase 3: 以用代测（danqing）

**T7: showcase 动画原语演示卡** — 按钮触发 Pulse（卡片闪脉冲）+ Tween 驱动进度条（目标随机/交替）+ Crossfade 两色块切换；小体量，照既有卡范式。| S → 实机人工过目

### Phase 4: pomodoro 联动迁移（danqing-pomodoro 仓）

**T8: 调用点迁移**（前提：danqing 已 push；lock 经 `cargo check` 重解，D6）—
- `main.rs`：删 `mod fader/flash/hint` 与三个 `use`；类型换 `danqing::{Crossfade, Pulse, Cue, CueTiming, Tween}`
- `fader.frame(self.now, \|t\| FADE_EASING.eval(t))` ×3 处 → `frame(self.now, FADE_EASING)`；`fader.current()/switch_to()` 同名直通
- `motion_envelope.gain(running, now)` → `tween.value(now, if running {1.0} else {0.0})`（注意参数序翻转）；字段类型同步
- hint 两构造点（`main.rs:201` 与 `:273`）：`triggered_at(ZERO)` → `Cue::new(CueTiming::default())` + `trigger(ZERO)`；`idle()` → `Cue::new(...)`
- `motion.rs`：删 `MotionEnvelope` 及其 3 条专属测试（语义已由 danqing Tween 测试兜底）；场景索引常量与 9 个强度函数**保留**，强度函数签名不变
- 删 `flash.rs` / `hint.rs` / `fader.rs` 三文件 | M

**T9: pomodoro 回归** — 既有测试全绿（场景强度/索引锁测试不动）；实机四姿势人工验收：场景切换交叉淡化 / 阶段完成脉冲 / 首启快捷键提示 / 暂停 500ms 视觉沉降。| S

**★ Checkpoint 3**: 两仓三件套绿；用户裁决 commit/push（danqing 先，pomodoro 后，message 注明关联）。

## 风险与对策

| 风险 | 对策 |
|---|---|
| Tween 与 switch 旧行为差异（全程 vs 比例时长）破坏观感 | CP2 实机对照；spec 已裁统一语义 |
| pomodoro `hint` 构造点语义差（首启 vs 已见过） | T8 验收列明两构造点逐一核对 |
| easing 整形改变 MotionEnvelope 线性沉降观感 | Tween 迁移处一律 `Easing::Linear`，行为等价由移植测试锁死 |
| mixer 迁移隐性改行为 | T5 硬判据 = 既有 8 测试零改动全绿 |
