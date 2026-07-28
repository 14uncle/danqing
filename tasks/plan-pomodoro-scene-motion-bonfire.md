# Plan: 番茄钟场景动效 — 篝火场景

Spec: `docs/specs/pomodoro-scene-motion-bonfire.md`(2026-07-28)

依赖图: T1(框架火效能力)→ T2(策略层)→ T3(接线)→ T4(调参+门槛+终审)。T2 只依赖 spec 不依赖 T1 代码,但按雨试点节奏串行推进、每任务一提交。

## T1: 框架篝火动效能力(暗启动)

- `BackgroundFrame` 新增 `fire_intensity`(默认 0)+ `with_fire()` builder(clamp 0..1)。
- uniform 16B → 32B:`[opacity, fade, rain_intensity, time, fire_intensity, pad×3]`;`draw` 把 `fire_intensity` 随场景层下发,叠加层恒 0。
- `RAIN_WRAP_SECS` → `MOTION_WRAP_SECS`(注释同步为"场景动效公共周期",雨/火共用)。
- `background.wgsl` 新增火效段:呼吸(3 正弦叠加 flicker × 径向 mask,乘性)+ 余烬(分列 hash 上浮圆点,暖色 additive),全部门控 `u.fire_intensity > 0.0`;参数集中常量段。
- Acceptance: `fire_intensity == 0` 输出与静态逐像素一致(既有测试全绿即证);新增单测:`fire_intensity` 默认 0、`with_fire` clamp、wrap 更名后行为不变。
- Files: `src/render/background.rs`, `src/render/background.wgsl`

## T2: motion.rs 篝火策略

- `pub const BONFIRE_SCENE: usize = 0;` + 单测锁定 `SCENES[0].name == "篝火"` 唯一。
- 提取私有 `scene_weight(scene, from, to, fade)`;`rain_intensity` 改走 helper(行为不变);新增 `fire_intensity(from, to, fade, envelope)`。
- Acceptance: 单测覆盖篝火索引锁名、双非火恒 0、from/to 淡化权重、包络缩放;雨效测试原样全绿。
- Files: `examples/pomodoro/motion.rs`(过渡期 dead_code 允许在 T3 移除)

## T3: main.rs 接线

- `background_frame` 追加 `.with_fire(motion::fire_intensity(from, to, fade, self.motion_gain))`。
- 移除 T2 过渡期 `#![allow(dead_code)]`。
- Acceptance: 新增单测——篝火场景运行中火强度 1.0、暂停 500ms 沉降(边沿 1.0 / 中点 0.5 / 消失 0.0)、非火场景恒 0;既有雨测试全绿。
- Files: `examples/pomodoro/main.rs`, `examples/pomodoro/motion.rs`

## T4: 调参 + 门槛 + 终审

- release 跑起来,PrintWindow 抓篝火场景两帧(≥300ms 间隔),PIL 裁框帧差取证;对照山场景 ≈ 0。
- 用户目测调参(剂量纪律:≤2 轮,超则按撤退条款 revert)。
- benchmark `-Runs 3`(暖机)双门槛;暂停沉降运行时目测。
- 门槛:fmt / clippy×2 / test×2 全绿 + 五轴评审 + 紧贴 commit 复验。
- CLAUDE.md 现状行 + project map 同步;plan/todo 归档 `tasks/archive/`;memory 更新。
- Files: `background.wgsl`(仅常量段调参)、`CLAUDE.md`、`tasks/`、memory

## Checkpoints

- T1 后:火效能力存在但不可见(暗启动),全量测试绿。
- T3 后:篝火场景可见动效,进入用户目测调参。
- T4 后:spec 8 条验收逐条勾选,用户终审"过了"才算关闭。
