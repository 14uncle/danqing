# Todo: anim 动画原语模块 (框架下沉·簇A)

> Plan: `tasks/plan-anim.md` (D1–D7 决策与依赖图)。Spec: `docs/specs/SPEC-anim.md` (已批准)。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。
> commit/push 待用户指示 (D7)。

## 明日交接 (2026-09-09 收工盘点)

- danqing: T1–T7 **已提交并 push (fdd3d01)**; 工作树余 2 个 doc 台账改动 (本文件 + SPEC-anim.md 状态行), 随下次提交带走
- pomodoro: T8 **已迁移未提交** (M main.rs / M motion.rs / M CLAUDE.md / D fader.rs+flash.rs+hint.rs), 184 测试绿、仓库惯例 clippy 净、四类型零残留; Cargo.lock 无变动 (patch 段已提交, lock 无 git rev)
- 待用户四件事: ① pomodoro commit/push 裁决 ② CP2 实机 (danqing showcase: switch 拨动/快速双击 + 动画原语卡三按钮) ③ T9 四姿势实机 (场景切换淡化/完成脉冲/首启提示 has_seen 两路径/暂停 500ms 沉降) ④ pomodoro license.rs:65 `#[expect(dead_code)]` 在 --all-targets 下预存炸 (HEAD 同样炸), 修不修
- 之后: 簇B (AsyncJob) 待用户发令; 下沉三政策门裁决仍挂账 (见 docs/intent/framework-sinking.md)

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

- [x] **T8: 调用点迁移 ✅ 2026-09-09** — 前提: danqing 已 push (fdd3d01); lock 经 `cargo check` 验证**无需变动** ([patch] 已提交在 Cargo.toml, lock 中 danqing 条目无 git rev 钉, 发布去 patch 时再钉) (D6)
  - main.rs: 删 `mod fader/flash/hint` + 三 use; 类型换 `danqing::{Crossfade, Pulse, Cue, CueTiming, Tween}` ✅
  - `fader.frame(now, |t| FADE_EASING.eval(t))` ×3 → `frame(now, FADE_EASING)`; `current()/switch_to()` 直通 ✅
  - `motion_envelope.gain(running, now)` → `value(now, target)` (参数序翻转, target = running?1:0); 字段名保留, 构造点 ×2 换 `Tween::new(motion::SETTLE_DURATION, Easing::Linear)` ✅
  - hint 两构造点: 新助手 `triggered_cue(at)` (Cue::new+trigger); `idle()` → `Cue::new(CueTiming::default())` ✅
  - motion.rs: 删 MotionEnvelope + 其 3 条专属测试; 2 条暂停测试改由 danqing::Tween 驱动 (同值断言); 场景索引常量 + 9 强度函数保留 ✅; CLAUDE.md 同步 (Patterns/Source Layout)
  - 删 flash.rs / hint.rs / fader.rs ✅
  - Acceptance: 编译过; 四类型全仓零残留 (历史 spec 文档除外) ✅
  - Verify: `cargo test` 184 全绿; 仓库惯例 clippy (`-- -D warnings`) 零警告。注: `--all-targets` 暴露 license.rs:65 `#[expect(dead_code)]` 预存失败 (仅测试引用所致, HEAD 上同样炸, 非本次引入)
  - Files: `src/main.rs`, `src/motion.rs`, `CLAUDE.md`, 删 3 文件 | M

- [ ] **T9: pomodoro 回归** — 既有测试全绿 (184 通过 2026-09-09, 场景强度/索引锁不动 ✅ 机器部分); 实机四姿势: 场景切换交叉淡化 / 阶段完成脉冲 / 首启快捷键提示 (has_seen 两路径) / 暂停 500ms 视觉沉降
  - Acceptance: 四姿势观感与迁移前一致 (人工)
  - Verify: `cargo test` ✅ + 实机 (待人工)
  - Files: — | S

### ★ Checkpoint 3: 全量验收
- [x] 两仓三件套绿 (danqing 495 / pomodoro 184, 2026-09-09)
- [ ] 用户裁决 commit/push: danqing (T1–T7) ✅ 已提交并 push (fdd3d01) → pomodoro (T8–T9) 待裁决提交 (lock 无变动), message 注明关联
