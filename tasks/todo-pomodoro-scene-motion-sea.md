# Todo: 番茄钟场景动效 — 海场景

Spec: `docs/specs/pomodoro-scene-motion-sea.md` / Plan: `tasks/plan-pomodoro-scene-motion-sea.md`

- [ ] T1: 框架海动效能力(暗启动)
  - Acceptance: uniform 32B 填 pad + `sea_intensity` 通道 + wgsl 海效段(涌动+碎点,门控 sea_intensity>0);`sea_intensity==0` 输出与静态逐像素一致;新增单测全绿
  - Verify: `cargo test --lib --tests` + `cargo clippy -- -D warnings` + `cargo fmt --check`
  - Files: `src/render/background.rs`, `src/render/background.wgsl`
- [ ] T2: motion.rs 海策略
  - Acceptance: `SEA_SCENE = 1` 名称锁定;`sea_intensity` 权重合成;交叉淡化并存单测;雨/火测试零回归
  - Verify: `cargo test --example pomodoro` + `cargo clippy --example pomodoro -- -D warnings`
  - Files: `examples/pomodoro/motion.rs`
- [ ] T3: main.rs 接线
  - Acceptance: `background_frame` 携带海强度;运行 1.0 / 暂停 500ms 沉降 / 非海恒 0 单测;T2 过渡期 allow(dead_code) 移除
  - Verify: `cargo test --example pomodoro` + `cargo clippy --example pomodoro -- -D warnings`
  - Files: `examples/pomodoro/main.rs`, `examples/pomodoro/motion.rs`
- [ ] T4: 调参 + 门槛 + 终审
  - Acceptance: 帧差证据(海 >0 / 山 ≈0);用户目测 ≤2 轮;benchmark 双门槛 PASS;提交门槛全绿 + 五轴评审;spec 8 条验收勾选 + Open Questions 裁决;CLAUDE.md 同步;plan/todo 归档;memory 更新
  - Verify: `tools/benchmark.ps1 -Example pomodoro -Runs 3` + 全部提交门槛 + 用户终审
  - Files: `background.wgsl`(常量段)、spec、`CLAUDE.md`、`tasks/`、memory
