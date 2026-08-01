# Spec: 番茄钟场景动效 — 星夜

## Objective

雨、篝火、海、山、森林五场景动效已沿「潮汐式场景动效」范式落地(每效果一标量 uniform 并存、计时运行时世界环绕、暂停 500ms 沉降)。本里程碑为第六场景——**星夜**(SCENES[5])。

星夜静态图语言: 深靛蓝夜空渐变 + 底部两层暗山脊,**静态图无星点**(2026-08-01 用户裁定, 雨场景范式 — 星野运行时程序化渲染)。整体为暗底白字的夜晚场景, 调色板 base (22,26,52)。

动效顺着静态语言走, 不发明新元素, 剂量「一眼可见但不抢戏」——基础星野常驻、星闪点缀、流星偶发, 三者是「夜」语义的自然延伸。**暂停时星野定格可见**(按场景权重常驻), 星闪/流星随包络 500ms 沉降。

用户: 番茄钟使用者(单一用户即作者本人)。成功 = 星夜场景在计时运行时「活了」(星野 + 星闪 + 偶发流星一眼可见), 暂停时星闪/流星 500ms 内沉降回静态星野 (星野本身保持), 不抢余光、不破性能门槛。

## Tech Stack

- Rust + wgpu 30 + winit 0.30(同主仓);动效全部在 `src/render/background.wgsl` 程序化生成, **零新资产**。
- 复用既有通道: `BackgroundFrame.time` / uniform 动效槽位 / `motion.rs` 策略层 / `MotionEnvelope` 500ms 沉降(六效果共用同一包络实例, 同涨同落)。

## Commands

```bash
cargo test --example pomodoro starry      # 星夜动效策略 + 接线测试
cargo test --lib uniform_buffer_size      # uniform 布局护栏
cargo clippy --example pomodoro -- -D warnings
cargo fmt --check
cargo run --release --example pomodoro    # 运行观测 (◀/▶ 切到星夜)
```

## Design

### 框架能力(uniform 扩容, 9×f32 → 11×f32, 44B)

雨/火/海/山/森林五场落地后 uniform 36B, 本里程碑加 `starry_intensity` 与 `starry_base`:

- uniform: `[opacity, fade, rain_intensity, time, fire_intensity, sea_intensity, rain_time, mountain_intensity, forest_intensity, starry_intensity, starry_base]`, 共 11 字段 f32, **44B 有效**。
- **buffer 保持 48B**(11×4=44B, 16B 对齐上取到 48B, 尾部 4B padding)——`UNIFORM_BUFFER_BYTES` 不变, `UNIFORM_FIELDS` 护栏 10→11。
- `BackgroundFrame` 新增 `starry_intensity: f32`(+ `with_starry()`, 包络驱动)与 `starry_base: f32`(+ `with_starry_base()`, 场景权重常驻), 均 clamp 到 `[0, 1]`。
- 星闪频率取 1/8 Hz 整数倍, 与 8s 公共周期对齐(用 `u.time` 取模); 流星用 `u.rain_time`(非 wrap 连续)避免 8s 重置跳变。

### 基础星野 shader(background.wgsl `star_field`)

- **雨场景范式: 静态图去星, 星野运行时程序化渲染**, 按 `starry_base`(场景权重)**常驻** — 暂停星野定格可见, 不含包络。
- 密网格 hash(96×40 格), 格内随机偏移, 亮度/尺寸按 hash 分布; 星带 y 0.80 上(山脊上方)。
- additive 星点(暗夜适用), 亮度 0.08-0.38。

### 星闪 shader(background.wgsl `star_twinkle`)

- **与基础星野同格**的明灭增量, 叠在星野之上读作「星星在闪」。
- 明灭频率取档位 {1,2,3}/8 Hz(整数倍, 保 8s 公共周期, `u.time`), smoothstep 缓起缓落(海碎点范式)。
- 随 `starry_intensity`(包络×权重)沉降 — 暂停 500ms 星闪归零, 星野保持。
- 明灭增量克制 `0.14`(基星亮度之上再加, 不喧宾夺主)。

### 流星 shader(background.wgsl `meteor`)

- **偶发斜向流星**, 用 `u.rain_time`(连续累加)触发: 24s 一颗(`METEOR_PERIOD`), 存续 1.4s。
- 头部从右上斜向左下(位置由 `rain_hash(idx)` 确定性决定), 尾迹朝右上(头部后方)指数衰减, 头部亮核。
- **压暗 + 淡入**(2026-08-01 用户反馈「太亮像爆闪灯」): 头部 0.5(原 0.9)、总乘数 0.9(原 1.6)、smoothstep 0.25 淡入 — 柔和出现非突发闪光。
- 存续期 `life` 内按 `(1-life)` 淡出; 随 `starry_intensity`(包络)沉降, 暂停/非星夜无流星。

### 策略层(examples/pomodoro/motion.rs)

- `pub const STAR_SCENE: usize = 5;` + 单测锁定 `SCENES[5].name == "星夜"` 且唯一(防生成器重排)。
- `starry_intensity(from, to, fade, envelope)` 与火/海/山/森林同权重合成(共享 `scene_weight`)。
- `starry_base(from, to, fade)` **仅场景权重**(不含包络, 与 `rain_intensity` 同构)——基础星野常驻, 暂停定格可见。
- `MotionEnvelope` 原样复用(六效果共用同一包络实例——同涨同落, 潮汐契约)。
- **不引入独立 clock**——星闪/流星沿用火/海/山/森林「暂停沉降」语义; 星野按权重常驻(雨范式)。
- 交叉淡化期间六效果两两并存(标量模型天然覆盖), 补并存单测。

### 接线(examples/pomodoro/main.rs)

`background_frame` 追加 `let starry = motion::starry_intensity(from, to, fade, self.motion_gain);` 与 `.with_starry(starry)`。

## Boundaries

- **Always**: 提交前 `cargo fmt --check` + 两个 clippy + 全部测试绿; 星夜效参数集中在 wgsl 常量段; 纯逻辑(policy)留在 example 侧。
- **Ask first**: 新增依赖; 改性能门槛; 改 `SF_BRIGHT` 超出 0.25~0.5 区间 / `METEOR_HEAD` 超出 0.3~0.7 / `METEOR_PERIOD` 超出 15~40s(剂量契约)。
- **Never**: 改 `scenes.rs`(生成文件); 星夜效引入新资产文件; 为动效改变重绘频率; 在 widget/layout/event/text 引入平台依赖; 给星夜加独立 clock(雨例外不推广)。

## Success Criteria

1. 星夜场景计时运行时, 星闪一眼可见(暗夜星点明灭), 偶发流星(≤24s 出现一次)。
2. 暂停 → 500ms 内星闪/流星沉降回静态星野, **星野本身保持可见**(定格, 包络单元测试 + 运行时目测); 暂停中恢复从当前值续接, 无跳变。
3. 雨/火/海/山/森林效行为零回归(常量未动, 既有测试全绿)。
4. 流星不引入 8s wrap 跳变(rain_time 连续, 无重置); 流星出现柔和(淡入), 非突发闪光。
5. 窗口隐藏时零渲染成本(架构事实, 无新增 `request_redraw`)。
6. benchmark 门槛 PASS(暖机启动 ≤1s、常驻 WS ≤360MB)。
7. 提交门槛全绿(fmt / clippy×2 / test×2)。
8. 用户人工终审通过(星夜运行+暂停目测节奏与剂量)。
