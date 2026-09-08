# Todo: anim 动画原语模块 (框架下沉·簇A)

> Plan: `tasks/plan-anim.md` (D1–D7 决策与依赖图)。Spec: `docs/specs/SPEC-anim.md` (已批准)。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。
> commit/push 待用户指示 (D7)。

## Phase 1: 框架纯加法

- [x] **T1: Easing 扩展 ✅ 2026-09-09** — `theme.rs` Easing 加 `EaseIn`(t³) / `EaseOut`(1-(1-t)³) 变体 + eval 分支
  - Acceptance: 端点精确 eval(0)=0 / eval(1)=1; 单调不减; EaseIn(0.5)<0.5 <EaseOut(0.5); 既有 Linear/EaseInOut 断言不动
  - Verify: `cargo test theme` + clippy 零警告
  - Files: `src/theme.rs` | S

- [x] **T2: `src/anim.rs` 建档 + `Tween` ✅ 2026-09-09** — 文件头 `@author 十四叔`/`@date 2026/09/09`; 模块头 doc (做什么/不做什么/为什么); Tween 采 ChannelEnvelope 目标追踪语义 (D1) + 全程续接 (D2) + 值域不夹; `lib.rs` 加 `mod anim;` + `pub use anim::{...}` (D5)
  - Acceptance: 移植 MotionEnvelope 3 条语义 (idle 恒零/500ms 改 duration 参数化淡入/反向续接无跳变) + 新增: easing 整形 (EaseInOut 中点=0.5 且 1/4 点<0.25)、目标逐帧微变平滑跟随 (时辰范式)、任意值域不夹取 (target=360 可达)、is_animating 真假两态
  - Verify: `cargo test anim::` 全绿; 三件套绿
  - Files: `src/anim.rs` (新), `src/lib.rs` | S

- [x] **T3: `Pulse` + `Cue`/`CueTiming` ✅ 2026-09-09** — 移植 flash.rs (进行中触发忽略/结束可再触发/线性 1→0) 与 hint.rs (四段时序) 语义; CueTiming::default = 1500/300/5000/500ms; 淡入 EaseOut 淡出 EaseIn 用 T1 变体
  - Acceptance: 移植 6+7 条全绿 (含时间轴平移/重复触发保护); 新增非默认 CueTiming 一条
  - Verify: `cargo test anim::` 全绿; 三件套绿
  - Files: `src/anim.rs`, `src/lib.rs` | S (依赖 T1)

- [x] **T4: `Crossfade` ✅ 2026-09-09** — 移植 fader.rs (usize 索引/current/switch_to ≥0.5 占优吸附/progress 饱和); `frame(now, easing: Easing) -> (usize, usize, f32)` 闭包改枚举
  - Acceptance: 移植 8 条全绿 (idle 静止/切换从 0/线性进度/终点精确/easing 整形/同目标 noop/打断早回弹/打断晚前吸)
  - Verify: `cargo test anim::` 全绿; 三件套绿
  - Files: `src/anim.rs`, `src/lib.rs` | S

### ★ Checkpoint 1: 纯加法完成
- [x] 三件套绿 (494 通过 / 0 失败, 2026-09-09)
- [x] 既有测试零改动全绿 (纯加法证明)

## Phase 2: 框架内消费者换芯

- [x] **T5: mixer 迁移 ✅ 2026-09-09** — 删私有 `ChannelEnvelope` (~30 行), `Mixer` 声道改持 `Tween::new(Duration::from_millis(300), Easing::Linear)`; 0..1 夹取留 `set_target`; `frame_gains` 内部 `env.gain(now, target)` 改 `tween.value(now, target)`
  - Acceptance: **mixer 既有 8 测试零改动全绿** (硬判据); `ENVELOPE_DURATION` 常量移作 Tween 构造参数
  - Verify: `cargo test audio::` 全绿; 三件套绿
  - Files: `src/audio/mixer.rs` | S

- [x] **T6: switch 迁移 ✅ 2026-09-09** — `animate()` 删 dt 步进 (`switch.rs:154-168`), 改 `anim_progress = tween.value(ctx.elapsed, anim_target)` (D3); 删 `last_time` 字段 + `ANIM_DURATION_S`; Tween 150ms/Linear
  - Acceptance: 三件套绿; 编译期 `last_time`/`ANIM_DURATION_S` 零残留引用
  - Verify: 三件套绿; 实机观感归 Checkpoint 2
  - Files: `src/widget/form/switch.rs` | S

### ★ Checkpoint 2: 消费者换芯完成
- [x] 三件套绿 (mixer 8 测试零改动全绿 = T5 硬判据; switch 既有插值测试零改动通过)
- [ ] 实机人工过目: showcase switch 拨动 / 快速双击翻转 观感不劣化

## Phase 3: 以用代测

- [x] **T7: showcase 动画原语演示卡 ✅ 2026-09-09** — 照既有卡范式加「动画原语」卡: 三按钮触发 Tween 目标翻转 (800ms EaseInOut)/Pulse (600ms)/Crossfade (800ms) + 数值实时回显 (review 对齐: 视觉消费实例 = 同在 showcase 的 switch/mixer)
  - Acceptance: 三原语实机可触发可观察; 三件套绿
  - Verify: `cargo run --example danqing-showcase` 实机人工过目
  - Files: `examples/showcase.rs` | S

## Phase 4: pomodoro 联动迁移 (danqing-pomodoro 仓)

- [ ] **T8: 调用点迁移** — 前提: danqing 已 push; lock 经 `cargo check` 驱动重解 (D6, 不用 cargo update -p)
  - main.rs: 删 `mod fader/flash/hint` + 三 use; 类型换 `danqing::{Crossfade, Pulse, Cue, CueTiming, Tween}`
  - `fader.frame(now, |t| FADE_EASING.eval(t))` ×3 → `frame(now, FADE_EASING)`; `current()/switch_to()` 直通
  - `motion_envelope.gain(running, now)` → `tween.value(now, target)` (参数序翻转, target = running?1:0); 字段/构造点同步
  - hint 两构造点 (`main.rs:201` `:273`): `triggered_at(ZERO)`→`Cue::new(CueTiming::default())`+`trigger(ZERO)`; `idle()`→`Cue::new(..)`
  - motion.rs: 删 MotionEnvelope + 其 3 条专属测试; 场景索引常量 + 9 强度函数保留不动
  - 删 flash.rs / hint.rs / fader.rs
  - Acceptance: 编译过; `MotionEnvelope`/`FlashOverlay`/`ShortcutHintOverlay`/`SceneFader` 全仓零残留 (grep 验证)
  - Verify: `cargo test` 全绿; 三件套绿
  - Files: `src/main.rs`, `src/motion.rs`, 删 3 文件 | M

- [ ] **T9: pomodoro 回归** — 既有测试全绿 (场景强度/索引锁不动); 实机四姿势: 场景切换交叉淡化 / 阶段完成脉冲 / 首启快捷键提示 (has_seen 两路径) / 暂停 500ms 视觉沉降
  - Acceptance: 四姿势观感与迁移前一致 (人工)
  - Verify: `cargo test` + 实机
  - Files: — | S

### ★ Checkpoint 3: 全量验收
- [ ] 两仓三件套绿
- [ ] 用户裁决 commit/push: danqing (T1–T7) 先提交并 push → pomodoro (T8–T9) 提交 lock, message 注明关联
