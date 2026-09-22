# Plan: HiDPI 缩放支持（scale factor 贯通框架）

> Spec: `docs/specs/hidpi-scale-factor.md`（2026-09-21 用户批准）
> Todo: `tasks/todo-hidpi.md`

## Overview

框架内部坐标系统一为逻辑像素（= 100% 缩放下的现有全部数字，产品零改动），scale factor 只落在渲染边界（TextBatch ×s 物理栅格化 + pass 分域喂尺寸）与输入边界（光标/滚轮 ÷s），并补 `ScaleFactorChanged` 动态切换。s=1.0 数学恒等，现有测试零修改全绿 = 回归锁。

## Architecture Decisions

- **A1 逻辑坐标语义**（spec D1）：widget/layout/event/text 纯逻辑层零改动；现有数字即逻辑值。
- **A2 渲染边界分域**（spec D2）：rect/image pass 喂**逻辑**视口尺寸（shader 除法中 s 约掉，零改动）；text pass 保持喂**物理**尺寸，`TextBatch` 内部按 px×s 栅格化、落点物理格吸附 —— 字形位图 1:1 上屏，现有清晰字方案（memory/font-cjk-mono）完整保住。
- **A3 R2 定案 — atlas 零改动**：`GlyphAtlas::get_or_rasterize` 维持 u16 px 签名；TextBatch 内部 `round(px×s)` 得物理 px。栅格/测量/行高共用同一取整值保证内部一致；绝对尺寸误差 <1px，远小于肉眼阈值。**atlas.rs 与其测试逐字节不动**，s=1.0 回归锁最硬形式成立。
- **A4 D4 核查结果 — 无 scissor**：全框架无 `set_scissor_rect`，裁剪全部 shader 内（text.wgsl/image.wgsl 按 `in.px` 剔除），clip 随实例同域旅行。TextBatch::push_clip 收逻辑 rect 内部 ×s；ImageBatch 收逻辑 rect 直通。render 提交侧零换算。
- **A5 R4 核查结果 — 背景 shader 零改动**：background.wgsl 中 `screen_w/screen_h` 仅用于长宽比（s 约掉不变），噪声为 UV 域连续 value noise，无物理像素依赖。spec D5 豁免无现存对象，保留为未来 shader 纪律。
- **A6 IME 光标区零改动**：`update_ime` 本就按 Logical* 上报 widget 域 rect（handler.rs:630）—— 改造后 widget 域 = 逻辑域，自动转正（现网 HiDPI 上 IME 候选窗位置其实是错的，顺带修好）。
- **A7 动态切换单路径**：`ScaleFactorChanged` 只更新 scale + 清图集 + redraw；尺寸/表面一律走既有 `Resized` 单路径（winit 随后必发），避免双源竞态。

## Task List

### Phase 1: 边界改造（s=1.0 恒等，各自落地不翻牌）

- [ ] T1: TextBatch scale 化（render/text.rs）
- [ ] T2: 输入边界换算（window/event.rs + handler.rs 光标存点）

### Phase 2: 翻牌接线

- [ ] T3: 布局视口逻辑化 + scale 接线（handler.rs + render/mod.rs）

### Checkpoint CP1: s=1.0 回归锁

- [ ] `cargo test --lib --tests` 现有测试**零修改**全绿 + 新增单测绿
- [ ] `cargo clippy -- -D warnings` + `cargo fmt --check`
- [ ] showcase 本机 100% 人工快速比对：与改造前无视觉漂移

### Phase 3: 动态切换与验收

- [ ] T4: ScaleFactorChanged 动态切换（handler.rs）
- [ ] T5: 人工视觉验收（本机调 200%/150%/125% + 运行中改缩放 + R1 图集观察）

### Checkpoint CP2: spec S1–S4 全满足

- [ ] S1 200% 等比 + 清晰度不劣化；S2 125%/150% 无破版；S3 动态切换正确；S4 测试全绿

### Phase 4: 联动与收口

- [ ] T6: danqing-log 联动验证（**阻塞于用户 push danqing**）
- [ ] T7: 记忆/文档回填（font-cjk-mono DPI 条目改写、spec 状态、MEMORY.md）

依赖：T1 ∥ T2 → T3 → CP1 → T4 → T5 → CP2 → T6/T7。
**全程不 commit/push**，CP2 后由用户指示提交节奏（联动链路：danqing 先 push → danqing-log 再 update lock，顺序不能反）。

## Risks and Mitigations

| 风险 | 级别 | 对策 |
|------|------|------|
| T3 翻牌引入回归 | 高 | s=1.0 数学恒等（×1.0/÷1.0 逐位不变）+ CP1 测试零修改绿 + showcase 人工比对 |
| R1 图集容量 @s=2（字形面积 ×4，1024² 或不足） | 中 | T5 验收盯日志「栅格化失败」warn（图集满 err 含「字形图集已满」）；触发则请示扩容/驱逐（Boundaries: ask first） |
| fractional scale 行高/测量不一致 | 低 | A3 同一 rounded px 贯穿栅格与测量，内部自洽 |
| ScaleFactorChanged 与 Resized 时序竞态 | 中 | A7 单路径；T5 运行中改缩放实测 |
| 隐藏态改系统缩放后落位错误 | 低 | scale 更新无幻影尺寸问题；显示时自愈路径按新 scale 换算（handler.rs:1410 已用 scale_factor） |

## Open Questions

- R3 placement 跨 DPI 屏恢复：维持现状（物理域语义不变），T5 验收若见错位另立 follow-up。
- R1 图集容量：T5 观察后定，如需变更走「Ask first」请示。
