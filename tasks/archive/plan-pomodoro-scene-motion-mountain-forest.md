# 山 + 森林场景动效补完

## Context

丹青番茄钟 POC 已闭环五场景静态视觉与音频(场景沉浸美学),雨/篝火/海三场的运行时动效也按"静态图去烘焙 + 程序化 shader"范式落地并通过终审。山(SCENES[3])与森林(SCENES[4])目前是仅有的两个**纯静态**场景——静止时 `mountain_intensity = forest_intensity = 0`,shader 端无对应分支,主要景深感缺失。

本改动沿用既有雨/火/海的范式为这两个场景补动效,与既有五场景轮转节奏一致,生成"潮汐起"时整个世界都环绕、暂停/归零时逐场景独立沉降的视觉语言。**不动** pause/音频/镜头/资产——本次纯做视觉 shader-only 增量。

---

## 1. Uniform 缓冲扩展

既有 8 字段 = 32B(f32 × 8,全标量无 vec)。本次扩为 9 字段 = 36B:

| 槽 | 字段 | 类型 | 来源 |
|---:|---|---|---|
| 0 | `opacity` | f32 | 既有 |
| 1 | `fade` | f32 | 既有 |
| 2 | `rain_intensity` | f32 | 既有 |
| 3 | `time` | f32 | 既有 (篝火/海/山/森林共用,雨不用) |
| 4 | `fire_intensity` | f32 | 既有 |
| 5 | `sea_intensity` | f32 | 既有 |
| 6 | `rain_time` | f32 | 既有 |
| 7 | `mountain_intensity` | f32 | **新增** |
| 8 | `forest_intensity` | f32 | **新增** |

**删 `pad1`,不再保留**(总 36B,wgpu 4-byte 对齐,f32 自然合规,无需 padding)。

Rust 侧 `BackgroundFrame` 同步扩两个字段,`motion: [f32; 5]` 数组扩 `[f32; 7]`(把 mountain/forest 加到尾部),`bytemuck::cast_slice` 字面量数组从 8 项扩 9 项。

WGSL 同步扩 `Uniforms` struct。

---

## 2. 山动效设计 (mountain)

**两元素,克制**——参照"篝火光晕呼吸"范式做乘性起伏。两个 mask 区域在 y 上不重叠,无视觉撞车。

### 2.1 暖色径向光呼吸 (对位 `fire_breath`)

- 中心 `(0.5, 0.66)`,对齐静态图暖光(`export-scenes.py:399`)
- 半径 `0.45`
- 频率 `1/8 + 2/8 Hz`,公共周期 8s 兼容
- 幅度 `MOUNTAIN_BREATH_GAIN = 0.07`(比篝火 0.08 略低;暮色对调亮敏感,克制)
- 形态:`color.rgb *= 1 + mountain_breath(uv, t) × intensity`

### 2.2 山脊 silhouette 整体亮度慢呼吸 (新增,靠轮廓而非光)

- mask:`smoothstep(0.78, 0.86, uv.y)`(覆盖静态图山脊区 y≈0.86 / 0.97,远山层次感)
- 幅度 `MOUNTAIN_RIDGE_GAIN = 0.04`(明显低于径向光,做层次)
- 频率 `1/8 + 2/8 Hz`,相位与径向光错开

### 2.3 不需要新资产 / export-scenes.py 不动

### 2.4 暂停降级

`mountain_intensity` 随既有 `MotionEnvelope` 500ms 归零,贡献乘子=0,逐像素回静态图。

---

## 3. 森林动效设计 (forest)

**两元素**——顶光呼吸 + 两道横雾 UV 漂移。漂移必须**水平**(森林有横雾带沿水平延展;若沿垂直 UV 位移会读作"雾降下",破坏"林间穿雾"语义)。

### 3.1 顶光呼吸 (对位 `fire_breath`,中心偏上)

- 中心 `(0.5, 0.10)`,对齐静态图顶部穿雾天光(`export-scenes.py:426`)
- 半径 `0.42`
- 频率 `1/8 + 2/8 Hz` 错相(同山范式)
- 幅度 `FOREST_TOP_GAIN = 0.06`(略低于山,因为顶光区不与中央倒计时冲突,可稍弱)

### 3.2 两道横雾水平 UV 漂移 (与海同构机制,但轴向相反)

- 雾带位置(静态 `export-scenes.py:440-443`):
  - 上层雾: y 中心 0.30,半高 0.09
  - 林间雾: y 中心 0.62,半高 0.07
- 漂移方向:上层左漂,林间右漂,造"穿林风"反向感
- 漂移幅度:`FOREST_MIST_GAIN = 0.10`(uv 单位,峰值,见 §3.4 论证)
- 形态:`sample_uv.x += forest_mist_drift(uv, t) × forest_intensity`(作用于采样坐标,**在 `textureSample` 之前**叠加,与海段同构)

### 3.3 不需要新资产 / export-scenes.py 不动

### 3.4 漂移幅度校正

§3.2 给出 `FOREST_MIST_GAIN = 0.10` 是**核心调参点**。论证:

- 海段 `SEA_SWELL_GAIN = 0.015`(纵向位移,实测 ±9.6px @960x640 窗)
- 用户要求"雾比海慢、幅度更小"
- 雾是柔带不是硬波带,位移幅度可低,但必须肉眼可读
- 若直接套海的 `0.015`,在 960px 窗仅 ±9.6px,即便雾带 mask 把幅度限制在雾内,**远小于海段因深度 mask(0.4→1.0)动态增益**
- 雾带本身已限定可视区,需补偿 `0.015` 之下变不可读
- **推荐 `0.010`**,实测开始窗口裁屏验证后再定终值(备选 0.008~0.012 区间,终审可调;`export-scenes.py` 不动)
- 在 960px 窗下 0.010 uv = ±9.6px,雾带本身宽度内 ±5px 是合理视觉剂量

**Phase 4 验证阶段会调,本常量在终审前可改。**

### 3.5 暂停降级

`forest_intensity` 随既有 `MotionEnvelope` 500ms 归零,顶光乘子=0、UV 漂移=0,逐像素回静态图。

---

## 4. CPU 端 intensity 计算

完全沿用既有模式:

```rust
// motion.rs 新增 (与 rain_intensity / fire_intensity / sea_intensity 完全同构)

pub const MOUNTAIN_SCENE: usize = 3;
pub const FOREST_SCENE: usize = 4;

pub fn mountain_intensity(from: usize, to: usize, fade: f32, envelope: f32) -> f32 {
    envelope * scene_weight(MOUNTAIN_SCENE, from, to, fade)
}

pub fn forest_intensity(from: usize, to: usize, fade: f32, envelope: f32) -> f32 {
    envelope * scene_weight(FOREST_SCENE, from, to, fade)
}
```

- 复用既有 `scene_weight`(`motion.rs:84-87`)
- 复用既有 `envelope`(`motion_gain`,即 `motion_envelope.gain(timer.is_running(), now)`)
- 不需要新独立 clock(雨例外,山/森林对齐火/海"暂停回静态"语义)
- `intensity` 经 `with_mountain()` / `with_forest()` builder clamp 到 `[0, 1]`,无负值

---

## 5. 文件改动清单

按"加一个新场景动效"模板 8 步落到具体文件 + 行号:

### 步骤 1:资产
**不动**。山/森林 PNG 已是终态;`tools/export-scenes.py` 不动;`assets/scenes/{mountain,forest}.png` 不动。

### 步骤 2:`examples/pomodoro/motion.rs`

| 行号 | 改动 |
|------|------|
| 21 后 | 加 `pub const MOUNTAIN_SCENE: usize = 3;`(锁名"山",单测唯一) |
| 27 后 | 加 `pub const FOREST_SCENE: usize = 4;`(锁名"森林",单测唯一) |
| 104 后 | 加 `pub fn mountain_intensity(...)` 与 `pub fn forest_intensity(...)` |
| 测试模块 | 加 `mountain_scene_index_points_at_mountain` / `forest_scene_index_points_at_forest` / `mountain_intensity_weights_by_scene_and_fade` / `forest_intensity_weights_by_scene_and_fade` / `mountain_coexists_with_rain_fire_sea_on_crossfade` / `forest_coexists_with_rain_fire_sea_on_crossfade` / `mountain_pauses_fall_to_zero_in_500ms` / `forest_pauses_fall_to_zero_in_500ms` |

### 步骤 3:`examples/pomodoro/scenes.rs`
**不动**(生成文件,5 SCENES 已是终态)。

### 步骤 4:`examples/pomodoro/main.rs`

| 行号 | 改动 |
|------|------|
| 367 后 | `background_frame` 接线:加 `let mountain = motion::mountain_intensity(from, to, fade, self.motion_gain);` 与 `let forest = motion::forest_intensity(from, to, fade, self.motion_gain);` |
| 373 后 | 链式 builder 追加 `.with_mountain(mountain)` 与 `.with_forest(forest)` |
| 测试模块 | 加对应 `background_frame_carries_*` / `settles_on_pause` / `stays_zero_on_non_*` 测试 6 个 |

### 步骤 5:`src/render/background.rs`

| 行号 | 改动 |
|------|------|
| 53-74 (struct) | `BackgroundFrame` 加 `pub mountain_intensity: f32` 与 `pub forest_intensity: f32`(默认 0) |
| 78-90 (`new`) | 初始化两个新字段为 0.0 |
| 105-109 后 | 加 `with_mountain(mountain_intensity) -> Self` 与 `with_forest(forest_intensity) -> Self` 两个 builder,各自 clamp 0..1 |
| 559-569 (默认 frame) | 默认 frame 加 `mountain_intensity: 0.0, forest_intensity: 0.0` |
| 574-580 (motion 数组) | 改 `[f32; 7]`,追加 `frame.mountain_intensity, frame.forest_intensity` 到尾部 |
| 633, 647 (`[0.0; 5]`) | 改 `[0.0; 7]`(两处,光晕/噪声层) |
| 666 (draw_layer 签名) | 改 `motion: [f32; 5]` → `[f32; 7]` |
| 696 (注释) | 改注释:`uniform 布局 (36B): [opacity, fade, 雨丝强度, 动效时间, 篝火强度, 海强度, 雨钟, 山强度, 森林强度]` |
| 708 (upload_quad 签名) | 改 `motion: [f32; 5]` → `[f32; 7]` |
| 775-777 (bytemuck cast_slice) | 改 `[opacity, fade, motion[0..6]]` 共 9 项 = 36B |
| 测试模块 | 加 `with_mountain_sets_and_clamps_intensity` / `with_forest_sets_and_clamps_intensity` / `with_mountain_is_independent_of_*` 三类,既有测试零回归 |

**注:** 这块构采用 §1 决策的最小扩容方案——删 pad、加 mountain_intensity + forest_intensity,共 9 字段 36B。**不引入 vec2,沿用全标量纪律**(每效果一标量语义清晰)。详细取舍见 Phase 3 review note(原 Plan agent 在 32B/36B/40B/44B/48B 间反复摇摆,最终取最小 36B 增量)。

### 步骤 6:`src/render/background.wgsl`

| 行号 | 改动 |
|------|------|
| 11-20 (Uniforms) | 删 `pad1: f32`,加 `mountain_intensity: f32,` 与 `forest_intensity: f32,` 共 9 字段 |
| 142 后(海段与 fs_main 之间) | 新增"山动效"段:`MOUNTAIN_*` 常量 + `mountain_flicker(t)` + `mountain_breath(uv, t)` + `mountain_ridge_breath(uv, t)` |
| 197 后(海段尾,fs_main 之前) | 新增"森林动效"段:`FOREST_*` 常量 + `forest_top_flicker(t)` + `forest_top_breath(uv, t)` + `forest_mist_mask(uv, y, half)` + `forest_mist_drift(uv, t)` |
| 241-244(`sample_uv` 初始化) | 海段 `if (u.sea_intensity > 0.0)` 之后,**追加** 森林段的 `sample_uv` 偏移块(在 `textureSample` 之前) |
| 257-271(雨/火/海 post-blend 块) | (a) 在海 `sea_glints` 段之后加山 `mountain_breath` 乘性块;(b) 加森林 `forest_top_breath` 乘性块 |

### 步骤 7:测试

```
examples/pomodoro/motion.rs::tests
├── mountain_scene_index_points_at_mountain          (锁名 SCENES[3].name == "山")
├── forest_scene_index_points_at_forest             (锁名 SCENES[4].name == "森林")
├── mountain_intensity_weights_by_scene_and_fade    (from/to/双非/静止)
├── forest_intensity_weights_by_scene_and_fade      (同上)
├── mountain_coexists_with_rain_fire_sea_on_crossfade
├── forest_coexists_with_rain_fire_sea_on_crossfade
├── mountain_pauses_fall_to_zero_in_500ms           (envelope × 1 → tick 500ms → 0)
├── forest_pauses_fall_to_zero_in_500ms
└── (既有雨/火/海 14 个测试零回归)

src/render/background.rs::tests
├── with_mountain_sets_and_clamps_intensity          (-0.1→0, 0.5→0.5, 1.5→1.0)
├── with_forest_sets_and_clamps_intensity            (同上)
├── with_mountain_does_not_touch_rain_fire_sea_forest_time
├── with_forest_does_not_touch_rain_fire_sea_mountain_time
└── (既有 11+ 个测试零回归)

examples/pomodoro/main.rs::tests
├── background_frame_carries_mountain_motion_when_running_on_mountain_scene
├── background_frame_carries_forest_motion_when_running_on_forest_scene
├── background_frame_mountain_settles_to_zero_on_pause
├── background_frame_forest_settles_to_zero_on_pause
├── background_frame_mountain_stays_zero_on_non_mountain_scene
├── background_frame_forest_stays_zero_on_non_forest_scene
└── (既有测试零回归)
```

### 步骤 8:`tools/export-scenes.py`
**不动**。详见 §2.3 / §3.3。

---

## 6. 关键文件(实施核心路径)

| 路径 | 角色 |
|------|------|
| `F:\github\danqing\src\render\background.rs` | BackgroundFrame 字段 + builder + 36B cast_slice + 默认 frame |
| `F:\github\danqing\src\render\background.wgsl` | Uniforms 9 字段 + 山段 + 森林段 + fs_main sample_uv 水平偏移 |
| `F:\github\danqing\examples\pomodoro\motion.rs` | MOUNTAIN_SCENE / FOREST_SCENE + intensity 函数 + 8 个新单测 |
| `F:\github\danqing\examples\pomodoro\main.rs` | background_frame 接线 + 6 个新单测 |
| `F:\github\danqing\examples\pomodoro\scenes.rs` | **不修改,只读取**(确认 5 SCENES 索引 0..4 已对) |
| `F:\github\danqing\tools\export-scenes.py` | **不修改** |
| `F:\github\danqing\tests\assets.rs` | **不修改** |

---

## 7. 验证

### 7.1 门槛(全绿)
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo clippy --example pomodoro -- -D warnings
cargo test --lib --tests
cargo test --example pomodoro
cargo build --release --example pomodoro
powershell -NoProfile -File tools\benchmark.ps1 -Example pomodoro -Runs 3
```

### 7.2 帧差客观佐证

用既有的 PS1 + click-post + PrintWindow 抓帧脚本(见 `tools/` 与 `CLAUDE.md`「视觉排障工具链」):

- **山运行 vs 山暂停**: 裁框 `mountain.png` 中段 y≈0.5-0.85 区(径向光+山脊覆盖区),两帧(≥1s 间隔)mean abs diff > 0.003、moved_ratio > 0.05
- **森林运行 vs 森林暂停**: 裁框顶光区 y≈0-0.20 + 中林雾带 y≈0.55-0.69,两帧 mean abs diff > 0.005(雾带 UV 位移贡献)
- **对照场景(雨/篝火/海/山-or-森林)**: 在非山(or 森林)场景运行时,对应 intensity 标量恒 0,既有动效输出与现状逐像素一致(雨/篝火/海零回归)
- **山↔森林过渡中点**: 两个 intensity 各 ≈ 0.5,叠加输出无明显鬼影(山 mask 集中 y>0.5,森林 mask 集中 y<0.5 + 雾带)

### 7.3 人工终审场景
1. 五场景轮转(雨→篝火→海→山→森林→雨):逐场景确认动效"一眼可见但不抢戏"
2. 山暂停中点 250ms(沉降中段):观察径向光缩暗程度
3. 山↔森林过渡中点:无跳变、无叠加鬼影
4. 暂停→恢复:沿用既有 `MotionEnvelope` reverse edge,无跳变
5. 窗口隐藏无 `request_redraw`(架构不变,自动继承)
6. 内存:WS ≤360MB(既有 3 场景+音频已 PASS,加 2 场景 shader-only 几乎无变化)
7. 启动:≤1s(无新资产/无新 pass,自动继承)

### 7.4 帧差法无法保证的部分

- 静止帧差只证明"动了",不证明"动得对";动效剂量(亮度、幅度、节奏)由人工终审定
- 暂停回静态:`intensity=0` 严格门控在 shader 内保证,但需要确认静态图渲染本身正确(继承既有 4 场景验证)
- 雾漂移幅度(§3.4):通过人工目测 + 裁屏像素差定终值,不预先优化

---

## 8. 风险与边界

### 8.1 暂停"强度地板" — 不需要
既有 `MotionEnvelope` `envelope_pause_fades_out_and_resume_continues_from_current` 测试已锁定"暂停 500ms 归 0,反向边沿续接无跳变"。`intensity > 0.0` 严格门控确保零贡献。不需要 `max(intensity, ε)` 之类地板——会破坏"暂停=静态"语义。

### 8.2 五场景轮转鬼影
山径向光 mask 中心 (0.5, 0.66),半径 0.45。在 y=0.62(森林林间雾带中心)处,山 mask 值 ≈ 1-smoothstep(0.18, 0.45, 0.04) ≈ 1.0。森林林间雾带 mask ≈ 1.0。两者 mask 在此区域重叠,但**作用域不同**: 山段乘性调亮度(±0.07),森林段改 sample_uv.x(±0.010 uv)。不同 GPU 域,合成时是亮度叠加 + 采样偏移,**无像素级鬼影**。

Crossfade 中点(山↔森林)两个 intensity 各 ≈ 0.5,叠加结果:
- 山段:亮度 × (1 + 0.07 × 0.5 × sin) = × (1 ± 0.0175) — 在视觉阈值边缘
- 森林段:sample_uv.x 偏 ±0.005 uv(960 窗下 ±4.8px) — 可见但不突兀
- 加起来 = 山呼吸略亮 + 雾漂 4.8px — 各自的视觉剂量都极小,叠加不冲突

### 8.3 性能
山+森林每像素 shader 增量 ≈ 18 ALU,既有雨火海 ≈ 75 ALU,总增加 ≈ +24%。暖机启动 ≤1s、WS ≤360MB 既有 3 场景已 PASS,加 2 场景 GPU 碎片叠加,大概率 PASS。若老核显 60fps 余量不足:
- 降 `MOUNTAIN_BREATH_GAIN` 之外的双正弦为单正弦(山段 ALU -1)
- 缩 `FOREST_MIST_MASK` smoothstep 区间(森林段 ALU -1)
- 但**先等 benchmark 数据再调**,不预先优化

### 8.4 视觉一致性
- 山/森林与既有三效果共用 `MotionEnvelope` 实例:**沿用同涨同落潮汐契约**
- 山/森林没有"独立 clock"(雨例外):与火/海对齐,既有 500ms 归 0 语义自然适配
- 视觉降级无新增:既有 `palette.desaturate(0.7)` 与"⏸ 已暂停"次级色已统一处理暂停视觉降级,山/森林零侵入

---

## 9. 与既有范式的偏离(均无)

| 维度 | 既有 | 本次 | 偏离 |
|------|------|------|------|
| uniform | 8×f32 = 32B 全标量 | 9×f32 = 36B 全标量 | **不引入 vec2**,每效果一标量语义保留 |
| 强度合成 | `envelope × scene_weight × fade`(火/海),`scene_weight × fade`(雨) | `envelope × scene_weight × fade` | **同火/海**,无独立 clock |
| 暂停时长 | 500ms `MotionEnvelope` | 500ms `MotionEnvelope` | 完全复用 |
| 频率基频 | 1/8 Hz 整数倍,公共周期 8s | 1/8 + 2/8 Hz,公共周期 8s | **不引入新基频** |
| 资产 | export-scenes 重生成(如雨去丝) | 不动 export-scenes | 完全 shader-only 增量 |
| 镜头 | 统一 Cover | 统一 Cover | 零变更 |
| 测试节奏 | T1 框架 / T2 策略 / T3 主程序 / T4 调参 | 同上 | 完全复用 |

---

## 10. 验收检查表

- [ ] `cargo fmt --check` 通过
- [ ] `cargo clippy -- -D warnings` 通过(workspace + example 双重)
- [ ] `cargo test --lib --tests` 全绿,新增 8 + 4 + 6 = 18 个测试,既有 ~45 个零回归
- [ ] `cargo test --example pomodoro` 全绿
- [ ] `cargo build --release --example pomodoro` 通过
- [ ] benchmark 启动 ≤1s、WS ≤360MB
- [ ] 五场景轮转:雨→篝火→海→山→森林→雨,逐场景目测动效"一眼可见但不抢戏"
- [ ] 山/森林运行帧 vs 暂停帧:裁屏有可读差异(参考 7.2 阈值)
- [ ] 雨/篝火/海运行帧:与改前逐像素一致(零回归)
- [ ] 山↔森林过渡中点 500ms:无跳变、无叠加鬼影
- [ ] 暂停 500ms 后逐像素回静态图
- [ ] 隐藏窗口无 `request_redraw`(架构不变自动继承)
- [ ] 文档同步:`CLAUDE.md` 当前状态更新、`tasks/todo-*.md` 与 `docs/specs/pomodoro-scene-motion-{mountain,forest}.md` 写、运行+暂停帧差截图归档到 `tasks/archive/`
- [ ] `tools/export-scenes.py` 不动,`scenes.rs` 不动,音频段不动
