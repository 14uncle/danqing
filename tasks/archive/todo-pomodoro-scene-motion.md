# Todo: 番茄钟场景动效试点(雨)

> Plan: `tasks/plan-pomodoro-scene-motion.md` | Spec: `docs/specs/pomodoro-scene-motion.md`
> 提交门槛(每次提交前): `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` 全绿。

- [x] **T1: 框架雨效能力(暗启动)**
  - Acceptance: `BackgroundFrame.with_motion(time, rain_intensity)`(默认 0、强度 clamp);uniform 复用 16B pad 位;`background.wgsl` 雨丝叠加(intensity=0 输出不变);time Rust 侧取模;showcase/pomodoro 视觉零变化;`DANQING_WGPU_VALIDATION=1` 无校验错误
  - Verify: `cargo test --lib --tests` + clippy + 手动 showcase/pomodoro 目检无变化
  - Files: `src/render/background.rs`、`src/render/background.wgsl`

- [x] **T2: motion.rs 纯逻辑**
  - Acceptance: `RAIN_SCENE` 场景名锁定测试;`MotionEnvelope` 500ms 边沿/续接/反向无跳变;`rain_intensity` from 雨/to 雨/双非雨 × fade 合成;`cargo test --example pomodoro` 全绿
  - Verify: `cargo test --example pomodoro motion`
  - Files: `examples/pomodoro/motion.rs`(新,头注释 @author/@date)、`examples/pomodoro/main.rs`(仅 mod 声明)

- [x] **T3: 接线 + 首轮剂量调参**
  - Acceptance: 雨场景雨丝一眼可见、余光不抢戏;暂停 500ms 淡出/恢复 500ms 淡入;场景切换雨效无跳变;print-window 连拍两帧(≥300ms)雨场景像素差 > 阈值、非雨场景 ≈ 0
  - Verify: `cargo run --example pomodoro` 人工目检 + `tools/print-window.ps1` 连拍对比
  - Files: `examples/pomodoro/main.rs`(调参含 `src/render/background.wgsl` 常量);同时移除 `motion.rs` 顶部的 `#![allow(dead_code)]` 过渡行

- [x] **T4: 门槛 + 人工终审**
  - Acceptance: spec Success Criteria 1~8 全过(benchmark 双门槛、三绿、隐藏零渲染回归、帧差佐证、用户终审);终审不过 → `git revert` T1~T3
  - Verify: `tools/benchmark.ps1 -Example pomodoro` + 全量测试 + 用户人工终审
  - Files: `tasks/todo-pomodoro-scene-motion.md`、必要时 `CLAUDE.md`
