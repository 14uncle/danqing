# Spec: 番茄钟场景动效试点(雨)

> 适用于阶段 2 番茄钟 POC 的场景沉浸美学增强。意图经 interview-me 访谈收敛,于 2026-07-28 获用户显式确认(yes)。

## Objective

环境音落地后出现视听落差:耳朵听到活的雨,眼睛看到的雨却不动。为雨场景增加"一眼可见但不抢戏"的程序化雨丝动效,让画面与声音重新匹配。

**用户故事:**

- 作为使用者,我在雨场景下能看到雨丝持续下落,画面与耳边的雨声是一致的"活"的场景;余光扫过窗口时不会被动效持续勾走注意力——注意力的主角仍然是计时与工作本身。
- 作为维护者,动效是纯增量的:不引入新资产、不提高重绘频率、隐藏窗口时零额外开销;效果不好时可以一次 `git revert` 干净回到静态图。

**本轮明确排除:** 篝火/海/山/森林四场景动效(篝火是第二候选,看雨试点终审结果再议)、动效的用户可调配置项、新视觉资产(帧序列/视频)、showcase 改动、第二 POC。

## Tech Stack

- **渲染:** 现有 `src/render/background.rs` + `background.wgsl` 场景层,扩展 uniform 携带时间与效果参数;雨丝为 fragment shader 内程序化 hash 叠加,无纹理资产。
- **时间:** `AnimationCtx.elapsed` 经 `App::background_frame()` 注入(与 `flash.rs`/`hint.rs` 同纪律,不读 wall-clock)。
- **约束:** 不新增第三方依赖;`widget/`、`layout.rs`、`event.rs`、`text/` 纯逻辑不动。

## Commands

```bash
# 开发运行
cargo run --example pomodoro

# 纯逻辑与集成测试
cargo test --lib --tests
cargo test --example pomodoro

# 格式与静态检查
cargo fmt --check
cargo clippy -- -D warnings

# 发布构建与 benchmark 双门槛(启动 ≤1s、常驻 WS ≤360MB)
cargo build --release --example pomodoro
powershell -NoProfile -File tools/benchmark.ps1 -Example pomodoro -Runs 1

# 帧差客观佐证(PrintWindow 连拍,见 tools/ 视觉排障工具链)
```

提交门槛: `cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test --lib --tests`、`cargo test --example pomodoro` 全绿;外加 release build 与 benchmark 双门槛 PASS。

## Project Structure

```text
src/
  render/
    background.rs       # 扩展: BackgroundFrame 增加 time 与 rain_intensity (with_motion builder);
                        #        uniform 保持 16B, 复用两个 pad 浮点位, 无需 SceneEffect 枚举
    background.wgsl     # 扩展: 场景层 fs_main 叠加程序化雨丝 (intensity=0 时输出与现状一致)
examples/
  pomodoro/
    motion.rs           # 新增: 雨场景映射 + 500ms 暂停沉降包络 + 强度权重合成 (纯逻辑)
    main.rs             # 扩展: background_frame 注入 time 与 intensity
docs/specs/
  pomodoro-scene-motion.md
```

设计要点(plan 阶段收敛): 框架只提供"雨丝叠加"渲染能力(两个 f32 参数),全部策略(哪个场景下雨、淡化权重、暂停沉降)留在 example 侧纯逻辑;`theme.rs`/`lib.rs` 不动。

不动的文件: `examples/pomodoro/scenes.rs`(生成文件,勿手改)、`tools/export-scenes.py`、`src/widget/`、`src/window/`(重绘语义不变)、`src/theme.rs`。

## Functional Requirements

1. **雨丝动效**: 雨场景可见时,场景图上层叠加程序化雨丝(斜向、分层密度、随时间持续下落)。剂量目标: 正视一眼即知"雨在落";余光不被持续吸引。
2. **效果随场景淡化**: 场景跨淡化期间,雨效强度随场景权重缩放(雨为 `from` 时按 `1-fade`、为 `to` 时按 `fade`),不产生跳变。
3. **时间注入**: `BackgroundFrame` 增加时间(秒)与雨效强度(0..1)字段(默认 0,`with_motion` builder 设置);`App::background_frame()` 每帧产出;强度合成(场景权重 × 沉降包络)为 example 侧纯逻辑,可单元测试。
4. **非雨场景零影响**: 强度为 0 时 shader 输出与现状视觉一致;showcase 与单图背景路径不受影响(默认参数不触新代码路径的可见效果)。
5. **暂停沉降**(2026-07-28 用户裁定): 计时暂停时雨效做 500ms 淡出(视觉独立时长,不复用音频 300ms),恢复时 500ms 淡入;视听沉降不必等长。
6. **隐藏零开销**: 不引入任何额外 `request_redraw`;窗口隐藏期间无渲染(现有架构事实,回归验证)。
7. **参数为代码常量**: 雨丝密度/速度/亮度/倾角为命名常量,集中在 shader 或 Rust 侧一处,不调不暴露。

## Non-functional

- **性能门槛**: benchmark 双门槛 PASS(启动 ≤1s、常驻 WS ≤360MB,核显记账);GPU 增量限定为场景层 fragment 叠加,无新纹理、无新 pass。
- **可撤回**: 满足以下任一即 `git revert` 回静态图,不留半成品开关: benchmark 退化、人工终审不过、实现复杂度失控(例如 shader 调参超过两轮仍达不到剂量目标)。

## Testing Strategy

- **纯逻辑单测**(框架层,`src/render/background.rs`): `BackgroundFrame` 新字段的 clamp(强度夹到 0..1)与默认值(不调用 `with_motion` 时 time/intensity 恒 0)。
- **纯逻辑单测**(example 侧,`motion.rs`): 雨场景索引映射(锁定 `SCENES[RAIN_SCENE].name == "雨"`,防生成器重排)、500ms 沉降包络(边沿触发/中点续接/反向边沿无跳变,同 AmbientMixer 用例范式)、强度权重合成(from 雨 / to 雨 / 双非雨 × fade)。
- **shader 逻辑不可测部分最小化**: 凡能在 Rust 侧算的量(权重、沉降系数)不进 shader;shader 只消费 time 与 intensity 两个标量。
- **客观佐证**: PrintWindow 连拍两帧(间隔 ≥300ms),雨场景区域平均像素差 > 阈值(证明"动了");同窗口非雨场景两帧场景区域像素差 ≈ 0(证明"没误伤")。
- **人工终审**: 雨场景 1080p 下肉眼确认剂量(一眼可见、余光不抢戏)、文字可读性不降;暂停/恢复的沉降观感(若 Open Question 1 成立)。

## Boundaries

- **Always:** 提交门槛三绿 + benchmark 双门槛;时间注入不读 wall-clock;动效强度计算留 Rust 纯逻辑;新 `.rs` 文件头 `@author 十四叔` + `@date`。
- **Ask first:** 新增依赖(预期零)、改动 wgpu 后端/设备策略、把动效推广到第二个场景、改 `scenes.rs` 或生成器。
- **Never:** 为动效引入帧序列/视频等新资产、提高重绘频率或改变隐藏零渲染语义、在 shader 里写死不可调的魔法参数而不留 Rust 侧常量、为保留失败方案加特性开关(撤回走 git)。

## Success Criteria

1. 雨场景可见时雨丝持续下落,PrintWindow 连拍两帧(间隔 ≥300ms)场景区域像素差超过阈值;非雨场景同法测量场景区域像素差 ≈ 0。
2. 场景跨淡化(雨 ↔ 其他)全程无雨效跳变:雨效随淡化权重平滑进出。
3. 暂停时雨效 500ms 淡出至不可见,恢复时 500ms 淡入(2026-07-28 用户裁定)。
4. 隐藏窗口期间无任何重绘(日志/架构回归确认),`app.tick` 行为不变。
5. benchmark 双门槛 PASS(启动 ≤1s、常驻 WS ≤360MB)。
6. `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test --lib --tests` / `cargo test --example pomodoro` 全绿。
7. 人工终审: 用户确认"一眼活了、余光不抢戏、文字可读",剂量判决同以往终审惯例。
8. 终审不通过或门槛退化时,`git revert` 干净撤回,静态图行为与现状逐字节一致。

## Open Questions

(无遗留。原两问已于 2026-07-28 裁定: 暂停时雨效做沉降淡出/淡入;视觉沉降时长 500ms,不复用音频 300ms。)
