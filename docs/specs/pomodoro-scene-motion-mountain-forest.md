# Spec: 番茄钟场景动效 — 山与森林

## Objective

雨(2026-07-28)、篝火(2026-07-29)、海(2026-07-29)三场景动效已沿"潮汐式场景动效"范式落地:每效果一标量 uniform 并存、计时运行时世界环绕、暂停 500ms 沉降。本里程碑为第四、五场景——山(SCENES[3])与森林(SCENES[4])。

山与森林静态图自身的视觉语言:

- **山**: 暮色纵向多段渐变 + 暖色径向光(中心 (0.5, 0.66)) + 中央可读性 veil + 两层山脊(基线 y≈0.86 / 0.97)。整体为暗紫底亮暖光的暮色场景,调色板 base (86,80,115) 偏冷,accent (232,192,122) 暖金。
- **森林**: 雾灰绿→深绿纵向渐变 + 顶部穿雾天光(中心 (0.5, 0.10)) + 中央压暗 veil + 三层针叶林线(基线 y≈0.52 / 0.68 / 0.88) + 两道横向雾带(上层 y≈0.30,林间 y≈0.62)。整体为暗绿底亮顶光的冷色场景,调色板 base (50,72,59),accent (172,198,158) 嫩绿。

动效必须顺着各自静态图语言走,不发明新元素,剂量"一眼可见但不抢戏"——参照既有雨/火/海终审校准(同刻度、同形态、同包络语义)。

用户: 番茄钟使用者(单一用户即作者本人)。成功 = 山/森林场景在计时运行时"活了"(暖光呼吸 + 雾带漂移 / 顶光呼吸 + 雾带漂移一眼可见),暂停时 500ms 内沉降回静态图,且不抢余光、不破性能门槛。

## Tech Stack

- Rust + wgpu 30 + winit 0.30(同主仓);动效全部在 `src/render/background.wgsl` 程序化生成,**零新资产**(山/森林静态 PNG 已是终态)。
- 复用雨/篝火/海已建的通道:`BackgroundFrame.time` / uniform 动效槽位 / `motion.rs` 策略层 / `MotionEnvelope` 500ms 沉降(五效果共用同一包络实例,同涨同落)。

## Commands

```bash
cargo test --lib --tests                 # 框架纯逻辑测试
cargo test --example pomodoro            # 番茄钟纯逻辑测试
cargo clippy -- -D warnings              # 工作区静态检查(不覆盖 example)
cargo clippy --example pomodoro -- -D warnings  # example 静态检查(必须单独跑)
cargo fmt --check
powershell -NoProfile -File tools/benchmark.ps1 -Example pomodoro -Runs 3  # 性能门槛
# 运行观测: cargo run --release --example pomodoro
# 抓帧: tools/print-window.ps1 <hwnd> <out.png>(输出须 Windows 全路径)
# 帧差: Python PIL 裁框灰度 diff(mean abs diff + moved_ratio>8)
```

## Design

### 框架能力(uniform 扩容,第四次使用 → 8×f32 → 9×f32,36B)

雨/火/海三场落地后 uniform 32B 已满,本里程碑加 `mountain_intensity` 与 `forest_intensity` 两个新标量(山/森林不共享,各自独立标量),**布局 32B → 36B**:

- uniform: `[opacity, fade, rain_intensity, time, fire_intensity, sea_intensity, rain_time, mountain_intensity, forest_intensity]`,共 9 字段 f32。
- **删 `pad1`**,不再保留(总 36B,wgpu 4-byte 对齐,f32 自然合规,无需 padding)。
- **不引入 vec2,沿用全标量纪律**——每效果一标量语义清晰;后续第七个效果若需,继续扩 f32 槽位(标量模式在 12×f32 = 48B 之内仍 wgpu 友好)。
- `BackgroundFrame` 新增 `mountain_intensity: f32` + `forest_intensity: f32`(默认 0)+ `with_mountain()` / `with_forest()` 链式 builder,各自 clamp 到 `[0, 1]`。
- 五 effect intensity 均 `== 0` 时 shader 输出与静态逐像素一致(暗启动纪律,既有火/海段已保证,本里程碑扩展同结构)。
- 时间共享: 山/森林复用同一 `time` uniform 与 8s 取模(`MOTION_WRAP_SECS`);所有频率/速度取 1/8 Hz 整数倍,保 8s 公共周期不破。
- `motion: [f32; 5]` 数组扩 `[f32; 7]`,光晕/噪声层 `[0.0; 5]` 占位数组扩 `[0.0; 7]`(无新逻辑,只对位扩展)。

### 山效 shader(background.wgsl 新增段落,参数集中可调)

- **山脊云雾缭绕,随风而动**(用户 2026-07-30 终审反馈 — 弃"呼吸光晕"改"风吹云雾"): additive 雾色叠加。
  - 雾色 `MOUNTAIN_RIDGE_MIST_COLOR` (180, 175, 195) sRGB→linear: 冷暖中性,融入暮色调色不抢戏。
  - y mask 包裹两层山脊: `smoothstep(0.78, 0.86, y) * (1.0 - smoothstep(0.96, 1.0, y))` — 0.78 软入、0.86 满、0.96 软出,精确包住山脊基线 0.86 与 0.97。
  - 风驱 pattern: `mist_pattern(uv, t, 0.0625, 8.0, 0.0)` 主 + `mist_pattern(uv, t, -0.0625, 6.0, 2.1)` 副(反向慢漂),加权 0.65+0.35 叠加。`mist_pattern` 是 sum-of-sines 伪噪声(4 个不同频率 sin),x 累加 `t * speed` 偏移造"风吹过"。
  - 密度调制 1/4 Hz (2 × 1/8 Hz wrap-clean): `density = 0.6 + 0.4 sin(t * 2π * 0.25)`,造"雾淡雾浓"周期感。
  - alpha 上限 `MOUNTAIN_RIDGE_MIST_ALPHA = 0.22`,gate `intensity > 0` 与 `wrap_motion_time` 配合保证暂停回静态。
- **Wrap-clean 约束**: 8s 公共周期由 `wrap_motion_time` 强制回零,雾纹必须在整数周期数后回到原位。`8 * speed` 必须为整数(此处 0.0625、-0.0625 都是 1/16 = 0.5 周期/8s)。
- 采样坐标**完全不动** — 山脊 silhouette 静止,只有雾作为独立程序化层 additive 叠加。

### 森林效 shader(background.wgsl 新增段落,参数集中可调)

- **雾气缭绕,随风而动**(用户 2026-07-30 终审反馈 — 弃"飞机云横带"改"风吹云雾"): additive 全域云雾。
  - 雾色 `FOREST_MIST_COLOR` (190, 205, 195) sRGB→linear: 雾绿灰,融入森林色调。
  - y mask: `smoothstep(0.20, 0.45, y) * (1.0 - smoothstep(0.80, 0.95, y))` — 0.20 软入、0.45 满、0.80 软出、0.95 全出,避开最顶光与最底色,覆盖整片树冠到林下。
  - **3 层风驱 pattern 叠加**: `mist_pattern(uv, t, 0.0625, 5.0, 0.0)` + `mist_pattern(uv, t, -0.125, 8.0, 1.7)` + `mist_pattern(uv, t, 0.125, 11.0, 3.4)`,加权 0.5+0.3+0.2。3 层不同 speed (1/8、-1/4、1/4 周期/8s) + 不同 scale (5/8/11) + 不同 phase (0/1.7/3.4),造有机的、不规则密度分布 — 视觉上不再像"飞机云"那种规律横带,而是随风漂移的不规则雾气。
  - 密度调制 3/8 Hz wrap-clean: `density = 0.6 + 0.4 sin(t * 2π * 0.375 + 1.5)`,与山错开相位,造两场景不同步呼吸。
  - alpha 上限 `FOREST_MIST_ALPHA = 0.18`。
- **关键架构决策(再次强调)**: 雾的可见运动**只能通过程序化雾层 additive 叠加,不能通过采样坐标 UV 漂移**。两次翻车的教训:
  1. 初版"水平 UV 漂移" — 中林雾带 y=0.55-0.69 与中林线 y=0.68 重叠,UV 漂移让中林整片跟着横移读作"海草"。
  2. 二版"两道横带" — 用户称"太规则,像飞机飞过的痕迹"。即使是程序化层,固定 Y 中心 + 软入 mask 仍然读作"飞机云"。
  3. 当前:程序化层 + sum-of-sines pattern(4 个不同频率)+ 风驱偏移 + 密度调制,造真正的"雾气缭绕"。
- 树梢完全静止 — 雾是独立程序化层,采样坐标不动。

### 策略层(examples/pomodoro/motion.rs)

- `pub const MOUNTAIN_SCENE: usize = 3;` + `pub const FOREST_SCENE: usize = 4;`,单测锁定 `SCENES[3].name == "山"` 且唯一、`SCENES[4].name == "森林"` 且唯一(防生成器重排)。
- `mountain_intensity(from, to, fade, envelope)` 与 `forest_intensity(from, to, fade, envelope)` 与 `fire_intensity` / `sea_intensity` 同权重合成(共享私有 `scene_weight` helper,公开 API 按效果分列)。
- `MotionEnvelope` 原样复用(五效果共用同一包络实例——同涨同落,潮汐契约)。
- **不引入独立 clock**——山/森林沿用火/海"暂停回静态"语义,雨例外(定格可见),不向山/森林推广。
- 交叉淡化期间五效果两两并存(山↔森林、海↔山、海↔森林、雨↔山、雨↔森林、火↔山、火↔森林),标量模型天然覆盖,补并存单测。

### 接线(examples/pomodoro/main.rs)

`background_frame` 追加:

```rust
let mountain = motion::mountain_intensity(from, to, fade, self.motion_gain);
let forest = motion::forest_intensity(from, to, fade, self.motion_gain);
```

链式 builder 追加 `.with_mountain(mountain)` 与 `.with_forest(forest)`(插入 `.with_sea(sea)` 之后、`.with_rain_time(self.rain_clock)` 之前,与字段顺序一致);`tick` 包络推进逻辑不变。

## Boundaries

- **Always**: 提交前 `cargo fmt --check` + 两个 clippy + 全部测试绿 + 五轴评审;山/森林效参数集中在 wgsl 常量段;纯逻辑(policy)留在 example 侧。
- **Ask first**: 新增依赖;改性能门槛;动效推广到其他场景;改静态场景图资产;调 `FOREST_MIST_GAIN` 超出 0.008~0.012 区间(可能改变"克制剂量"契约)。
- **Never**: 改 `scenes.rs`(生成文件);山/森林效引入新资产文件;为动效改变重绘频率(可见 60fps / 隐藏零渲染的架构事实不动);在 widget/layout/event/text 引入平台依赖;在 `BackgroundFrame` 引入 vec2 uniform(坚持全标量纪律);给山/森林加独立 clock(雨例外不推广)。

## Success Criteria

1. 山/森林场景计时运行时,帧差证据:
   - **山** — 暖光区(y≈0.5-0.85)+ 山脊区(y>0.78)裁框两帧(≥1s 间隔)mean abs diff > 0.003、moved_ratio > 0.05;对照场景(雨/篝火/海/森林)同法 ≈ 0。
   - **森林** — 顶光区(y≈0-0.20)+ 中林雾带(y≈0.55-0.69)裁框两帧 mean abs diff > 0.005(雾带 UV 位移贡献);对照场景(雨/篝火/海/山)同法 ≈ 0。
2. 形态与静态图语言一致:
   - 山呼吸只调制已有暖色径向光 + 山脊 silhouette 亮度,不移边缘、不改色相、不发明新元素。
   - 森林顶光呼吸只调制已有顶光亮度;雾漂移只发生在两道横雾带 mask 范围内,**水平**方向,不改色相。
3. 暂停 → 500ms 内山/森林效沉降回静态(包络单元测试 + 运行时目测);暂停中恢复从当前值续接,无跳变。
4. 雨/火/海效行为零回归(雨/火/海段常量未动,既有 18 个测试全绿)。
5. 窗口隐藏时零渲染成本(架构事实,无新增 `request_redraw`)。
6. benchmark 门槛 PASS(暖机启动 ≤1s、常驻 WS ≤360MB)。
7. 提交门槛全绿(fmt / clippy×2 / test×2)+ 五轴评审通过。
8. 用户人工终审通过(山/森林运行+暂停帧差、目测节奏与剂量)。**✅ 2026-08-01 用户通过**(目测运行/暂停动效节奏与剂量; 修正: 无 1-5 场景快捷键, 场景切换仅 ◀/▶; 森林副层已去, 当前为单层 mist_pattern)。

> 终审通过 2026-08-01, plan/todo 已归档 `tasks/archive/`, 本 spec 关闭。
