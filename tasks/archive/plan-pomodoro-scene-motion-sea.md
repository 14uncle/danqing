# Plan: 番茄钟场景动效 — 海场景

Spec: `docs/specs/pomodoro-scene-motion-sea.md`(2026-07-29,概念已获用户 "yes" 裁定)

依赖图: T1(框架海效能力)→ T2(策略层)→ T3(接线)→ T4(调参+门槛+终审)。同雨/篝火节奏串行推进、每任务一提交。

## T1: 框架海动效能力(暗启动)

- `BackgroundFrame` 新增 `sea_intensity`(默认 0)+ `with_sea()` builder(clamp 0..1)。
- uniform 保持 32B 不扩容:`[opacity, fade, rain_intensity, time, fire_intensity, sea_intensity, pad×2]`;`draw` 把 `sea_intensity` 随场景层下发,叠加层恒 0。
- `background.wgsl` 新增海效段:涌动(2 层水平行进正弦亮度波 × 波带纵向 mask,乘性)+ 碎点(分列 hash 原地明灭软圆点,乘性提亮),全部门控 `u.sea_intensity > 0.0`;参数集中常量段;所有频率取 1/8 Hz 整数倍保 8s 公共周期。
- Acceptance: `sea_intensity == 0` 输出与静态逐像素一致(既有测试全绿即证);新增单测:`sea_intensity` 默认 0、`with_sea` clamp、与 `with_motion`/`with_fire` 链式互不覆盖。
- Files: `src/render/background.rs`, `src/render/background.wgsl`

## T2: motion.rs 海策略

- `pub const SEA_SCENE: usize = 1;` + 单测锁定 `SCENES[1].name == "海"` 唯一。
- 新增 `sea_intensity(from, to, fade, envelope)`,走既有 `scene_weight` helper。
- Acceptance: 单测覆盖海索引锁名、双非海恒 0、from/to 淡化权重、包络缩放、交叉淡化三效果两两并存;雨/火测试原样全绿。
- Files: `examples/pomodoro/motion.rs`(过渡期 dead_code 允许在 T3 移除)

## T3: main.rs 接线

- `background_frame` 追加 `.with_sea(motion::sea_intensity(from, to, fade, self.motion_gain))`。
- 移除 T2 过渡期 `#![allow(dead_code)]`。
- Acceptance: 新增单测——海场景运行中海强度 1.0、暂停 500ms 沉降(边沿 1.0 / 中点 0.5 / 消失 0.0)、非海场景恒 0;既有雨/火测试全绿。
- Files: `examples/pomodoro/main.rs`, `examples/pomodoro/motion.rs`

## T4: 调参 + 门槛 + 终审

- release 跑起来,PrintWindow 抓海场景两帧(≥300ms 间隔),PIL 裁框帧差取证(波带区);对照山场景 ≈ 0。
- 用户目测调参(剂量纪律:≤2 轮,超则按撤退条款 revert)。
- benchmark `-Runs 3`(暖机)双门槛;暂停沉降运行时目测。
- 门槛:fmt / clippy×2 / test×2 全绿 + 五轴评审 + 紧贴 commit 复验。
- spec 8 条验收逐条勾选 + Open Questions 裁决记录;CLAUDE.md 现状行 + project map 同步;plan/todo 归档 `tasks/archive/`;memory 更新。
- Files: `background.wgsl`(仅常量段调参)、`docs/specs/pomodoro-scene-motion-sea.md`、`CLAUDE.md`、`tasks/`、memory

## Checkpoints

- T1 后:海效能力存在但不可见(暗启动),全量测试绿。
- T3 后:海场景可见动效,进入用户目测调参。
- T4 后:spec 8 条验收逐条勾选,用户终审"过了"才算关闭。
