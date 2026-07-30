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

- **暮色径向光呼吸(双轨: 乘性 + additive)**: 暖色径向光 mask(中心 (0.5, 0.66), 半径 0.45)× 双正弦 flicker(1/8 + 2/8 Hz,错相)。
  - **乘性** `MOUNTAIN_BREATH_GAIN = 0.12`: `color.rgb *= 1 + breath(uv, t) × intensity`(径向光调制)。最初设计 0.07 太小(暮色暗背景 + sRGB→linear 衰减),用户实测无可见动效,上调到 0.12。
  - **additive** `MOUNTAIN_GLOW_GAIN = 0.06`: 暖色径向光 (240, 200, 170) sRGB→linear 颜色 × mask × flicker(±1)× 增益,直接 additive 叠加,绕开 sRGB 视觉衰减,保证主观可见剂量。
  - 双轨目的:乘性保持色相不偏(暖光更暖不泛白),additive 保证动态明显;终审可单关一轨观察效果。
- **山脊 silhouette 整体亮度慢呼吸(新增,靠轮廓而非光)**: mask `smoothstep(0.78, 0.86, uv.y)`(覆盖静态图山脊区 y≈0.86 / 0.97,远山层次感)× 双正弦 flicker(频率 1/8 + 2/8 Hz,相位与径向光错开)× 幅度 `MOUNTAIN_RIDGE_GAIN = 0.07`(原 0.04 略小,上调)。
- 两个 mask 区域在 y 上**不重叠**(径向光中心 y=0.66 半径 0.45, 山脊 y>0.78),无视觉撞车。
- 参数全部集中在 wgsl 常量段,调参只动该段(与雨/火/海段并列,互不改名改值)。

### 森林效 shader(background.wgsl 新增段落,参数集中可调)

- **顶光呼吸(对位篝火 `fire_breath`,中心偏上)**: 顶光 mask(中心 (0.5, 0.10), 半径 0.42)× 双正弦 flicker(1/8 + 2/8 Hz 错相)× 幅度 `FOREST_TOP_GAIN = 0.10`(原 0.06,上调以保证可见)。
- **两道横雾程序化密度调制(替代 UV 漂移)**: 两道雾带 mask(y 中心 0.30 / 0.62,半高 0.09 / 0.07)× 静态底雾颜色 (206,220,206) / (188,205,189) sRGB→linear × 静态 alpha (0.16 / 0.12)× **density 调制 (1.0 ± 0.20 sin, 1/16 Hz 反相, 16s 周期)**。
  - **关键架构决策**: 雾的可见运动**不能通过采样坐标 UV 漂移实现**。初版"水平 UV 漂移"在用户实测时翻车:中林雾带 y=0.55-0.69 与中林线 y=0.68 直接重叠,水平 UV 位移让中林整片跟着横移,读作"海草摇摆"(雨场景试错的"沿轴均匀"陷阱扩展版:离散元素 + 沿轴 UV 位移 = 整片跟着动)。
  - 改为**程序化雾色叠加 + density 调制**: 静态 PNG 已有底雾,运行时仅按密度起伏再叠一层薄薄的颜色,采样坐标**完全不动**,树梢静止,只有雾整体淡浓。
  - 雾带 mask 形状与 export-scenes.py:439-444 静态底雾一致,保证两个版本视觉融合;静态底雾提供基础外观,程序化层提供呼吸感。
  - 频率 1/16 Hz 是 8s 公共周期的整数倍(2×8=16),不破既有约束。
- 形态:`color.rgb *= 1 + forest_top_breath(uv, t) × intensity + forest_mist_overlay(uv, t) × intensity`(顶光乘性 + 雾 additive 双层叠加,不动采样)。
- 参数全部集中在 wgsl 常量段,调参只动该段。

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
8. 用户人工终审通过(山/森林运行+暂停帧差、目测节奏与剂量)。
