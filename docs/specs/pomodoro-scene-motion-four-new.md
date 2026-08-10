# Spec: 四新场景动效

## Objective

为铁匠铺、洞穴、夜市、火车 4 个新场景添加 shader 动效,使静态底图在计时运行时"活起来"。
基础设施 (SceneSpec / intensity 函数 / BackgroundFrame / uniform upload) 已就绪,
本 spec 只覆盖 background.wgsl 中的效果函数 + fs_main 调度。

## 动效设计

### 铁匠铺 (blacksmith)

**视觉**: 铁砧上火花飞溅 + 炉火脉冲。

- **blacksmith_sparks**: 程序化火花粒子,从铁砧区域 (uv ~0.50, 0.58) 向上飞溅。
  分列 hash,每列一颗,纵向上升 + 横向微摆,橙黄色 additive。
  密度低于篝火余烬 (forge 场景较小),明亮但不抢眼。
- **blacksmith_forge_glow**: 炉火脉冲,在锻造闪光周期内 (1.5Hz) 径向亮度调制。
  作用于铁砧/炉火区域 (uv ~0.45-0.55, 0.55-0.65),模拟锤击时的亮度爆发。
  乘性调制底图 (不是 additive),读作"铁被击打时发亮"。

### 洞穴 (cave)

**视觉**: 水面微波 + 生物荧光呼吸。

- **cave_water_ripple**: 水面 UV 纵向位移 (复用 sea_swell 范式但幅度极小)。
  位移区 y=0.60-1.00 (水池区域),模拟地下水面的微波荡漾。
  两层正弦叠加,速度慢于海 (~60%),幅度为海的 ~30%。
- **cave_bioluminescence**: 钟乳石/石笋区域的生物荧光脉冲。
  散点分布在 y=0.20-0.65 (洞穴上半部),明灭频率 0.25-0.5Hz (比星闪慢一倍,呼吸感)。
  色调匹配图中青蓝色 (0.4, 0.85, 0.80),additive 叠加。

### 夜市 (nightmarket)

**视觉**: 灯笼暖光闪烁 + 升腾蒸汽。

- **nightmarket_lanterns**: 灯笼暖光脉冲散点。
  分列 hash,散布在 y=0.15-0.50 (灯笼带),明灭频率 {1,2}/8 Hz。
  色调暖黄 (1.0, 0.7, 0.3),additive,模拟灯笼内烛光摇曳。
  密度高于星闪 (灯笼很多),单点更小。
- **nightmarket_steam**: 食摊升腾蒸汽 (additive 雾气)。
  用 mist_pattern 生成,从 y=0.55-0.75 (摊位区) 向上漂浮。
  色调暖白 (1.0, 0.85, 0.7),低 alpha,与图中已有蒸汽融合增强。

### 火车 (train)

**视觉**: 车窗雨滴 + 车厢内光呼吸。

- **train_window_drops**: 车窗玻璃上的雨滴。
  散点分布在 x=0.40-0.90, y=0.05-0.85 (车窗区域),
  微弱的 UV 位移 (模拟雨滴折射) + 亮度微调。
  雨滴位置基本不动 (已凝结在玻璃上),只有亮度微弱明灭。
- **train_interior_glow**: 车厢内暖光呼吸。
  径向渐变,中心在 x=0.15, y=0.30 (顶部灯具区域),
  慢呼吸 (0.125Hz, 8s 周期),暖黄色 additive。
  覆盖车厢内壁区域,模拟灯光在绿色车厢内的暖色反射。

## Uniform 布局

无变化。现有 15×f32 = 60B 有效数据 + 4B padding = 64B buffer。
新效果通过 `blacksmith_intensity`/`cave_intensity`/`nightmarket_intensity`/`train_intensity`
控制,复用 `rain_time` 作为非 wrap 时间轴。

## 时间轴

- `u.time` (取模 8s): 用于周期性闪烁/脉冲 (forge_glow, lanterns, drops)
- `u.rain_time` (非 wrap): 用于持续漂移 (water_ripple, steam, interior_glow)

## Code Style

- Shader 函数命名: `{scene}_{element}(uv, t)` 或 `{scene}_{element}(uv, rt)`
- 常量命名: `SCENE_ELEMENT` 全大写下划线
- 中文注释描述视觉意图,英文命名

## Success Criteria

1. `cargo clippy -- -D warnings` 零警告
2. 4 个场景各有独立且可辨识的动效
3. 动效不与其他场景视觉重复
4. 暂停 500ms 后动效沉降回静态图
5. `cargo test --lib --tests` 全绿
