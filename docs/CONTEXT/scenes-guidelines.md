# 场景动效开发范式

> 按需加载。仅在开发/修改场景动效(shader、uniform、MotionEnvelope)时加载。
> 各场景的详细 spec 见 `docs/specs/pomodoro-scene-motion*.md`。

## 核心范式

**"静态图去烘焙 + 运行时程序化"** — 动效在 fragment shader 内程序化生成,零新资产。静态场景图仅作底图,动效元素(雨丝/余烬/波光/雾带)全由 shader 计算。

## 机制选型决策树

```
元素几何形状?
├── 有剪影边缘可动(波带/山脊/树冠)
│   → UV 位移采样坐标
│   → 位移须随效果强度缩放,暂停沉降回静态
│   → 位移场设计:天空区 mask=0、近地面位移大、相位含小 y 项破直线
│   → 两层同向行进(反向叠加成驻波,原地脉动不可读)
│   → 频率全落 1/8 Hz 整数倍,保 8s 公共周期
│
└── 沿轴均匀线条(雨丝) 或 点状粒子(余烬/波光碎点)
    → 去烘焙 + 程序化渲染
    → 适用场景:雨(丝)、篝火(余烬点+光晕)、海(波光碎点)
```

**关键教训**: 亮度/颜色调制(additive 或乘性)作用于静态图之上,人眼读作"明暗对象沿静态背景移动"——像车在波形路上开,路本身没动。要让背景图本身动,必须动采样坐标(UV 位移)或去烘焙程序化重绘。亮场景(近白底)additive 被底吃掉,细碎提亮走乘性。

> 详见 memory: [[scene-motion-uv-displacement]]

## Uniform 布局

当前 48B (9×f32 有效 36B + WGSL 16B 对齐 padding 12B),每效果一标量(非互斥选择子,交叉淡化可同时非零):

```wgsl
struct Uniforms {
    opacity: f32,            // 图层不透明度
    fade: f32,               // 场景交叉淡化系数
    rain_intensity: f32,     // 雨效强度
    time: f32,               // 动效时间(秒,上传前取模 8s)
    fire_intensity: f32,     // 篝火效强度
    sea_intensity: f32,      // 海效强度
    rain_time: f32,          // 雨钟(秒,非 wrap,持续累加,f32 25min 安全)
    mountain_intensity: f32, // 山效强度
    forest_intensity: f32,   // 森林效强度
}
```

演进历史: 16B → 32B(雨+火) → 36B(山) → 48B(9 字段,加 mountain/forest intensity)。`rain_time` 用于所有"持续漂移"类动效(雾、波),非 wrap-clean 数学(避免 sin 跳变)。Rust 侧 `UNIFORM_BUFFER_BYTES` 常量 + `uniform_buffer_size_covers_wgsl_struct` 回归测试护栏。

## MotionEnvelope

500ms 线性 envelope,控制动效强度的启停过渡:

- **启动**: 0→1 over 500ms,动效从静态渐入
- **暂停**: 1→0 over 500ms,动效沉降回静态
- **视觉独立时长**: 不复用音频 300ms,独立可调

## 视觉调参经验

### Alpha 阈值 (融入型效果,如雾、光晕)

| alpha | 效果 |
|-------|------|
| 0.10-0.15 | 不可见 |
| 0.20-0.30 | 微妙,用户可能嫌"看不清楚" |
| 0.30-0.45 | 明显可见(推荐目标区间) |
| 0.45-0.55 | 强势,易读作独立元素 |
| ≥0.55 | 读作独立云团/光斑,破"融入"语义 |

### 遮罩宽度 (融入型效果)

- 屏占比 55% (如 0.40-0.95) → 太宽弥漫
- 屏占比 38% (如 0.50-0.88) → 聚焦融入
- 屏占比 ≤40% 为宜

### sum-of-sines vs value noise

- **sum-of-sines**: 流畅波纹,适合大尺度连续视觉(雾);但有周期性(LCM 重复+传送带感)
- **value noise**: 无周期,适合雪/小颗粒;但 cell 边界有方块感,大尺度雾会读作"马赛克"
- **破周期**: 速度 ±30% 准周期起伏、多层错 LCM、反向漂移、错相

### wrap-clean 数学陷阱

`speed=0.0625` 看似 wrap-clean (8×0.0625×k=k/2 整数),但 sin(k/2·π)≠0(π 无理数),仍有 ~5% 跳变。**正确修法:用 `u.rain_time`**(非 wrap,持续累加,f32 25min 安全)。

## 视觉迭代预算

- 同一区域超过 **5 commit** 同主题还无收敛 → **提级反思根本问题**(换函数/换范式,非调参)
- 用户主导的方向修正(指定数值/裁定换机制)不计入撤退预算
- 每轮调参后自检是否在解决根因还是症状

## 已有 Spec 索引

| 场景 | Spec | 状态 |
|------|------|------|
| 雨(试点) | `docs/specs/pomodoro-scene-motion.md` | ✅ 已关闭 |
| 篝火 | `docs/specs/pomodoro-scene-motion-bonfire.md` | ✅ 已关闭 |
| 海 | `docs/specs/pomodoro-scene-motion-sea.md` | ✅ 已关闭 |
| 雨改造 | `docs/specs/pomodoro-scene-motion-rain-rework.md` | ✅ 已关闭 |
| 山/森林 | `docs/specs/pomodoro-scene-motion-mountain-forest.md` | ✅ 已关闭 (2026-08-01 人工终审通过) |
| 星夜 | `docs/specs/pomodoro-scene-motion-starry.md` | ⏳ 待用户终审 (2026-08-01 实现完成) |

## 相关 Memory

- [[scene-motion-uv-displacement]] — UV 位移 vs 程序化渲染的选型边界
- [[scene-lru-pattern]] — 多场景纹理 2 槽 LRU 懒加载
- [[danqing-project-state]] — 各场景动效关闭状态与 lessons learned
