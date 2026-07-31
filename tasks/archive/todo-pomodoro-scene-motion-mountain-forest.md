# Todo: 番茄钟场景动效 — 山与森林

Spec: `docs/specs/pomodoro-scene-motion-mountain-forest.md` / Plan: `C:\Users\gwhun\.claude\plans\immutable-exploring-brook.md`(等价 plan-pomodoro-scene-motion-mountain-forest.md)

- [x] T1: 框架山/森林动效能力(暗启动)
  - Acceptance: uniform 32B → 36B(删 pad1,加 mountain_intensity + forest_intensity,共 9 字段 f32,全标量无 vec2);`BackgroundFrame` 加两个字段 + with_mountain/with_forest builders(各自 clamp 0..1);`motion: [f32;5]` 扩 `[f32;7]`;bytemuck cast_slice 9 项 36B;wgsl `Uniforms` struct 同步扩 + 山/森林段(径向光呼吸+山脊呼吸 / 顶光呼吸+雾带水平 UV 漂移);门控 intensity>0;intensity=0 输出与静态逐像素一致;新增单测全绿
  - Verify: `cargo test --lib --tests` + `cargo clippy -- -D warnings` + `cargo fmt --check`
  - Files: `src/render/background.rs`, `src/render/background.wgsl`
- [x] T2: motion.rs 山/森林策略
  - Acceptance: `MOUNTAIN_SCENE = 3` + `FOREST_SCENE = 4` 名称锁 + 唯一;`mountain_intensity` / `forest_intensity` 权重合成(共享 scene_weight);交叉淡化并存单测;包络 500ms 暂停归零单测;雨/火/海测试零回归
  - Verify: `cargo test --example pomodoro` + `cargo clippy --example pomodoro -- -D warnings`
  - Files: `examples/pomodoro/motion.rs`
- [x] T3: main.rs 接线
  - Acceptance: `background_frame` 携带山/森林强度;运行 1.0 / 暂停 500ms 沉降 / 非目标场景恒 0 单测;既有 9 个雨/火/海 background_frame 测试零回归
  - Verify: `cargo test --example pomodoro` + `cargo clippy --example pomodoro -- -D warnings`
  - Files: `examples/pomodoro/main.rs`
- [x] T4: 调参 + 门槛 + 归档
  - Acceptance: fmt + clippy×2 + test×2 全绿;benchmark PASS(启动 ≤1s、WS ≤360MB);`FOREST_MIST_GAIN = 0.010` 起始值(终审视觉调参点,可在 0.008~0.012 区间调);spec 8 条验收勾选;CLAUDE.md 同步;plan/todo 归档
  - Verify: `tools/benchmark.ps1 -Example pomodoro -Runs 3` + 全部提交门槛
  - Files: `background.wgsl`(常量段)、spec、`CLAUDE.md`、`tasks/`
- [ ] T5: 人工终审 + 帧差截图归档
  - Acceptance: GUI 跑 showcase → ◀/▶ 切换到山、森林观察动效"一眼可见但不抢戏";PrintWindow 抓山运行+暂停帧、森林运行+暂停帧裁框帧差(mean abs diff > 阈值、moved_ratio > 0.05);用户终审通过
  - Verify: 截图归档到 `tasks/archive/`(运行端 + 暂停端 PNG);`docs/specs/pomodoro-scene-motion-mountain-forest.md` 验收 1/2/3 勾选
  - Status: 代码与门槛全绿,真实 GUI 验证由用户在 showcase 上完成(本次会话未运行 showcase)
