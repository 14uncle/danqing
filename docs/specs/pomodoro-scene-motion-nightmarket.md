# 夜市场景动效 Spec

> 状态: 已移除 (2026-08-09 用户要求)
> 创建: 2026-08-09

## 场景概述

夜市场景展现热闹的中国传统夜市，有灯笼、摊位、蒸汽、人群。AI 底图已包含静态蒸汽，运行时动效需让场景"活"起来。

## 动效清单

### 1. 强化蒸汽动效 (已有，需调优)

**目标**: 让摊位蒸汽更明显地飘动升腾

**当前问题**:
- 蒸汽柱位置 (x=0.20/0.45/0.70) 与底图摊位不匹配
- alpha=0.25 太弱，被底图静态蒸汽掩盖

**调整方案**:
- 蒸汽柱位置改为: x=0.32 (左摊位), x=0.52 (中摊位), x=0.68 (右摊位)
- alpha 提升至 0.35
- 蒸汽高度增加至 0.40 (到 y=0.35)
- 上升速度微调至 0.10

**实现位置**: `src/render/background.wgsl` NM_STEAM_* 常量

---

### 2. 灯笼摇摆 (新增)

**目标**: 灯笼随风轻微摆动，增加生动感

**视觉设计**:
- 灯笼位于画面上方 (y < 0.45)
- 摆动幅度极小 (±1-2px)，读作"微风轻拂"
- 摆动频率: 0.5-1.0 Hz (慢，悠闲感)
- UV 位移方案: 以灯笼中心为原点，横向正弦位移

**参数设计**:
```wgsl
const LANTERN_Y_BAND: f32 = 0.45;      // 灯笼带下缘
const LANTERN_SWAY_AMP: f32 = 0.002;   // 摆动幅度 (uv, ≈2px @960px)
const LANTERN_SWAY_FREQ: f32 = 0.125;   // 摆动频率 (1/8 Hz, 8s 周期)
```

**实现方案**:
- 在 `fs_main` 的 `sample_uv` 计算阶段，对 y < LANTERN_Y_BAND 的区域添加横向 UV 位移
- 位移量: `sin(time * LANTERN_SWAY_FREQ * 2π) * LANTERN_SWAY_AMP`
- 位移随 y 增加而衰减 (顶部摆动大，根部不动)

**实现位置**: `src/render/background.wgsl` 新增 `lantern_sway()` 函数

---

### 3. 人群微动 (新增)

**目标**: 人群轻微晃动，营造"人潮涌动"的氛围

**视觉设计**:
- 人群位于画面下方 (y > 0.65)
- 动效极其微妙，几乎不可察觉
- 方案: additive 亮度闪烁 + 微小 UV 位移

**参数设计**:
```wgsl
const CROWD_Y_TOP: f32 = 0.65;         // 人群带上缘
const CROWD_FLICKER_ALPHA: f32 = 0.08; // 闪烁强度 (极弱)
const CROWD_DISP_AMP: f32 = 0.001;     // UV 位移幅度 (≈1px)
```

**实现方案**:
- 分列 hash 生成伪随机闪烁 (频率 {1,2}/8 Hz)
- 亮度闪烁: additive 暖白色，alpha 极低
- UV 位移: 微小横向摆动，模拟人群呼吸感
- 使用 `rain_time` (非 wrap) 保证连续性

**实现位置**: `src/render/background.wgsl` 新增 `crowd_flicker()` 函数

---

### 4. 招牌闪烁 (新增)

**目标**: 霓虹灯/招牌灯光明灭闪烁，增加夜市氛围

**视觉设计**:
- 招牌位于画面两侧 (x < 0.15 或 x > 0.85)
- 不同招牌不同闪烁频率 (差异化)
- 明灭闪烁 (on/off 或明暗交替)

**参数设计**:
```wgsl
const SIGN_LEFT_X: f32 = 0.10;         // 左侧招牌带中心
const SIGN_RIGHT_X: f32 = 0.90;        // 右侧招牌带中心
const SIGN_WIDTH: f32 = 0.08;          // 招牌带宽度
const SIGN_Y_TOP: f32 = 0.30;          // 招牌带上缘
const SIGN_Y_BOT: f32 = 0.70;          // 招牌带下缘
const SIGN_FLICKER_ALPHA: f32 = 0.15;  // 闪烁强度
```

**实现方案**:
- 分列 hash 生成不同闪烁频率 ({2,3,4,5}/8 Hz)
- 使用 `sin()` 或 `step()` 生成明灭模式
- 颜色: 暖黄色 (匹配灯笼色调)
- 只在招牌带内生效

**实现位置**: `src/render/background.wgsl` 新增 `sign_flicker()` 函数

---

## Uniform 扩展

当前 uniform 已有 `nightmarket_intensity` 字段，4 种动效共享此强度值。

无需新增 uniform 字段。

---

## 实现顺序

1. ✅ **强化蒸汽** - 调整现有常量 (位置/alpha/速度)
2. ✅ **灯笼摇摆** - 新增 `lantern_sway()` UV 位移函数
3. ✅ **人群微动** - 新增 `crowd_flicker()` additive 层
4. ✅ **招牌闪烁** - 新增 `sign_flicker()` additive 层

## 验证标准

- [x] 蒸汽明显飘动，位置匹配底图摊位
- [x] 灯笼微摆，幅度自然 (不超过 ±3px)
- [x] 人群微动几乎不可见，但能感受到"活"的气息
- [x] 招牌闪烁有节奏感，不同招牌频率不同
- [x] 所有动效在 nightmarket_intensity=0 时完全消失
- [x] 切换场景时动效平滑过渡 (500ms envelope)
- [x] `cargo clippy` 零警告
- [x] `cargo test --lib --tests` 全绿

## 相关文件

- `src/render/background.wgsl` - Shader 动效实现 ✅ 已修改
- `src/render/background.rs` - Uniform 传递 (无需修改，复用 nightmarket_intensity)
- `examples/pomodoro/motion.rs` - 强度计算 (无需修改)
- `examples/pomodoro/main.rs` - 主程序连接 (无需修改)

## 实现细节

### 修改的文件

1. **`src/render/background.wgsl`**:
   - 调整 `NM_STEAM_*` 常量 (位置/alpha/速度)
   - 新增 `lantern_sway()` 函数
   - 新增 `crowd_flicker()` 函数
   - 新增 `sign_flicker()` 函数
   - 在 `fs_main` 中应用灯笼摇摆 (UV 位移阶段)
   - 在 `fs_main` 中叠加人群微动和招牌闪烁 (additive 阶段)

### 复用的机制

- **MotionEnvelope**: 所有动效共享 `nightmarket_intensity`，500ms 线性过渡
- **rain_time**: 人群微动使用 `u.rain_time` (非 wrap)，保证连续性
- **u.time**: 灯笼摇摆和招牌闪烁使用 `u.time` (8s wrap)，保公共周期

### 参数选择理由

- **灯笼摇摆**: 0.125 Hz (8s 周期)，幅度 0.002 uv (≈2px)，读作"微风轻拂"
- **人群微动**: α=0.08 极弱，几乎不可察觉但能感受到"活"的气息
- **招牌闪烁**: {2,3,4,5}/8 Hz 多频率，差异化避免单调
