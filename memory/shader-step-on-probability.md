---
name: shader-step-on-probability
description: "丹青场景 shader 密度门禁约定: step(threshold, hash) 的 on 概率 = 1-threshold;比例变量须写 step(1-ratio, h)。星野曾误写致实际 ~1280 颗、减量方向算反;星闪频率/相位取自 on 门槛同源 hash 会全体同步,须另起 hash"
metadata: 
  node_type: memory
  type: feedback
  modified: 2026-08-02T02:22:27.637Z
  originSessionId: 2a8b64b3-2ed6-4b6e-8c9e-5d8a1a23d8ae
---

丹青 `src/render/background.wgsl` 的动效密度门禁统一约定(2026-08-02 星夜星野 bug 落定):`step(threshold, hash)` → hash ≥ threshold 才 on,`P(on) = 1 − threshold`。雨列(0.70/0.72/0.85)、余烬、波光均按此。当配置量是「比例」而非门槛时(星野 SF_ON / SF_BIG / SF_WARM),必须写 `step(1.0 − ratio, h)` 才得 P(ratio)。

**星野 bug(2026-08-02 修复)**: `on = step(SF_ON=0.035, h)` 误写,实际 P(on)=0.965 → 渲染 ~1280 颗(注释以为 47)。更糟:「减四分之三」(SF_ON 0.14→0.035)在此约定下把 P(on) 从 0.86 提到 0.965 —— **减量方向算反**,用户连续多轮「还是太多」都无效。修复:`step(1.0 − SF_ON, h)` + SF_ON 0.035→0.074 → 实际 ~98 颗,对齐原静态图实测 ~80-170。星闪层(star_twinkle)同格一并修。

**动画参数 hash 耦合坑(2026-08-02, 星闪明暗呼吸)**: 星闪的 `k`(频率档)与相位原本取自 `on` 门槛同源 hash `h`——而 `on` 要求 h ≥ 1−SF_ON(高门槛),把 `floor(h*3)` 锁死在最大档、相位挤在圆末 ~8%,**所有 on 元素会以同一频率、近同步闪烁**(读作整体闪,而非独立呼吸,用户否掉全局同步后数据验证才揪出)。修复:频率 `freq_h`、相位 `phase_h` 用独立 hash seed(32/31/35 分散)。**规律: 从「可见性门槛」同源 hash 派生动画参数(频率/相位/方向)会让被选中的元素全部同频同步——可见性 hash 只该决定「有没有」,「怎么动」须另起 hash。**

**Why**: 注释里的推算与代码真实语义错位,且没按真实语义核对渲染结果——三轮调参全在错误的数上做文章;「独立呼吸」的意图被 hash 同源悄悄变成同步,肉眼一时难辨。
**How to apply**: 调 shader 密度/数量/动画参数前,先用 Python 按真实语义模拟核对(像 [[scene-motion-uv-displacement]] 用数据验证),再改代码;注释与代码语义必须一致;动画参数 hash 与可见性门槛 hash 分离。
