# Plan: 番茄钟场景动效试点(雨)

> Spec: `docs/specs/pomodoro-scene-motion.md`(2026-07-28 用户确认,Open Questions 已全部裁定)。
> 范围纪律: 只做雨场景;篝火是否跟进看本轮终审结果。

## 设计决策

1. **框架只提供能力,策略全在 example 纯逻辑。** uniform 是 `[opacity, fade, _pad0, _pad1]` 16B,两个 pad 浮点位恰好装下 `rain_intensity` 与 `time` —— buffer 尺寸不变、不需要 `SceneEffect` 枚举、`theme.rs`/`lib.rs` 零改动。场景权重、淡化合成、暂停沉降全部在 `motion.rs` 算成一个标量再下发。
2. **暗启动。** `BackgroundFrame` 新字段默认 0(`with_motion` builder 才设置),`intensity=0` 时 shader 贡献恒 0 —— 框架改动落地后 showcase 与 pomodoro 视觉逐像素不变,能力验证与策略上线分成两个可独立回滚的步骤。
3. **沉降包络复刻 `AmbientMixer` 范式。** 边沿检测(running 目标变化)→ 从当前值续接 500ms 滑动 → 到点停。时间注入 `AnimationCtx.elapsed`,不读 wall-clock,测试用例直接照搬 ambient.rs 的边沿/续接/反向范式。
4. **雨丝为 shader 内程序化 hash 叠加。** 2~3 层斜向条纹(远/近景深),additive 亮度叠加;密度/速度/倾角/亮度/层数全部集中在 `background.wgsl` 顶部 `const`,调参只动一处。
5. **f32 时间精度护栏。** Rust 侧上传前对雨效周期取模(如 64s),常驻数小时不出现相位抖动。
6. **场景索引防错位。** `motion.rs` 单测锁定 `SCENES[RAIN_SCENE].name == "雨"`,生成器重排顺序会在测试期爆炸而非静默错位。

## 依赖图

```
T1 框架雨效能力(暗启动)  background.rs + background.wgsl
    │   提供: BackgroundFrame.with_motion + uniform 接线 + 雨丝 shader
    │
T2 motion.rs 纯逻辑       examples/pomodoro/motion.rs (与 T1 无依赖,可并行)
    │   提供: 雨场景映射 + 500ms 包络 + 强度合成
    │
T3 接线 + 首轮调参        main.rs  (依赖 T1 + T2)
    │   产出: 雨场景可见动效、暂停沉降、淡化无跳变
    │
T4 门槛 + 人工终审        benchmark / 帧差佐证 / spec 8 条验收
```

T1 与 T2 之间无代码依赖(一个框架层一个 example 层),顺序执行即可;T3 是真正的汇合点。

## 任务切分

### T1: 框架雨效能力(暗启动)

`BackgroundFrame` 增加 `time: f32` / `rain_intensity: f32`(默认 0,`with_motion` builder,强度 clamp 0..1);`upload_quad` 层 0 写 `[opacity, fade, rain_intensity, time]`,叠加层恒 0;`background.wgsl` Uniforms 复用 pad 位,fs_main 在场景 mix 后叠加 `rain_overlay(uv, time) * rain_intensity`;Rust 侧上传前 time 对雨效周期取模。

- **Acceptance:** 新增单测(clamp/默认值)通过;`cargo test --lib --tests` 全绿;showcase 与 pomodoro 运行视觉与现状一致(intensity 恒 0);`DANQING_WGPU_VALIDATION=1` 运行 showcase 无校验错误。
- **Verify:** `cargo test --lib --tests` + `cargo clippy -- -D warnings` + 手动 `cargo run --example showcase` / `cargo run --example pomodoro` 目检无变化。
- **Files:** `src/render/background.rs`、`src/render/background.wgsl`

**Checkpoint 1:** 框架能力落地但零可见变化 —— 此处本身就是一个可提交点,回归风险已隔离。

### T2: motion.rs 纯逻辑

新增 `examples/pomodoro/motion.rs`(文件头 `@author 十四叔` + `@date 2026/07/28`):`RAIN_SCENE` 常量与名称锁定测试;`MotionEnvelope`(500ms,边沿检测 + 当前值续接);`rain_intensity(from, to, fade, envelope)` 权重合成。

- **Acceptance:** 单测覆盖 —— 场景名锁定、包络边沿/中点/续接/反向无跳变、from 雨/to 雨/双非雨 × fade 合成;`cargo test --example pomodoro` 全绿。
- **Verify:** `cargo test --example pomodoro motion`。
- **Files:** `examples/pomodoro/motion.rs`(新)、`examples/pomodoro/main.rs`(仅 `mod motion;`)

### T3: 接线 + 首轮剂量调参

`main.rs` 持有 `MotionEnvelope`,`tick` 以 `timer.is_running()` 驱动;`background_frame()` 产出 `.with_motion(now_secs, intensity)`。运行雨场景目检:雨丝一眼可见、余光不抢戏;暂停 500ms 淡出、恢复 500ms 淡入;雨 ↔ 其他场景切换全程无跳变。剂量不达标时只调 `background.wgsl` 顶部常量;超过两轮仍不达标触发 spec 撤回纪律。

- **Acceptance:** 雨场景动效肉眼确认;暂停/恢复沉降观感正确;切场景无跳变;`tools/print-window.ps1` 连拍两帧(间隔 ≥300ms)雨场景区域像素差 > 阈值、非雨场景 ≈ 0。
- **Verify:** `cargo run --example pomodoro` 人工目检 + print-window 连拍对比。
- **Files:** `examples/pomodoro/main.rs`(调参时含 `background.wgsl` 常量)

**Checkpoint 2:** 完整链路上线,用户在真实窗口里第一次判决剂量。

### T4: 门槛 + 人工终审

spec 验收 8 条逐条过:benchmark 双门槛(release)、三绿(fmt/clippy/test)、隐藏零渲染回归(隐藏期间日志无重绘、`tick` 行为不变)、帧差佐证归档、用户人工终审(剂量 + 文字可读性)。终审不通过 → `git revert` T1~T3,静态图保持现状。

- **Acceptance:** spec Success Criteria 1~8 全过;todo 收口提交。
- **Verify:** `powershell -NoProfile -File tools/benchmark.ps1 -Example pomodoro -Runs 1` + 全量测试 + 用户终审。
- **Files:** `tasks/todo-pomodoro-scene-motion.md`(勾选)、必要时 `CLAUDE.md` 现状同步

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| 雨丝剂量主观,调参发散 | 常量集中 shader 顶部一处;两轮不达触发撤回纪律(spec Non-functional) |
| f32 时间精度随常驻时长退化 | Rust 侧上传前取模(T1) |
| 生成器重排场景顺序导致雨效错位 | 单测锁定场景名(T2),测试期爆炸而非静默错 |
| 核显 fragment 开销 | 单次全屏叠加、无新 pass 无新纹理;benchmark 双门槛兜底(T4) |
| 淡化中雨效跳变 | 强度 = 场景权重 × 包络,单标量连续(T2 单测覆盖) |
