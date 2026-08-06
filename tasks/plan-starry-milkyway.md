# Implementation Plan: 星夜银河升级(深邃银河 · 三层合成)

- @author 十四叔
- @date 2026/08/03
- 状态: 待评审
- Spec: `docs/specs/pomodoro-scene-starry-milkyway.md`
- 上位意图: `docs/intent/scene-atmosphere-upgrade.md`

## Overview

为星夜场景加"深邃银河"主体,三层合成: AI 重画底图(光带+尘埃暗隙) + Yale BSC 星表亮星(银道坐标投影、CPU 启动烘焙成纹理) + 程序化暗星雾(银纬 mask 聚集)。uniform 布局与 `motion.rs` 策略层零改动;渲染改动集中在 `src/render/background.rs` / `background.wgsl` 与 example 侧新星表模块。

## Architecture Decisions

- **银道坐标系投影**: 银经 l→UV.x、银纬 b→UV.y + 一个旋转常量。亮星天然沿 b≈0 聚集,底图光带/星表/暗星雾三层共用同一坐标系,对齐只需调一个角——不做真实地点/时刻地平坐标换算。
- **CPU 启动烘焙,shader 纯采样**: 9,110 星 splat 成单通道星野纹理(1280×720),fragment 1~2 次采样,零逐星循环;烘焙耗时目标 <100ms,计入 ≤1s 启动门槛实测。
- **星闪改调制采样**: 星闪脉冲逻辑不变(1/8 Hz 档位、双极 ±0.42),改为细网格脉冲场调制采样到的星野纹理;网格密度保证 ≤1 亮星/格即读作逐星星闪,无需相位通道。
- **星野纹理走既有上传机制**: `background.rs` 纹理 bind group 扩一槽,example 侧按字节上传(与场景大图 `create_scene_texture` 同机制),启动烘焙、常驻(~4MB)。
- **暗星雾解析 mask**: 密度 = f(银纬),带宽/角度为 wgsl 常量,底图生成后人工测量光带中心线回填——回填是显式落地步骤(Task 8),不是隐式期望。
- **原始星表数据不入库**: `tools/export-stars.py` 内置来源 URL 人工触发下载,仅产物 `assets/stars.bin`(~53KB)入库。

## Task List

### Phase 0: 授权地基(fail-fast)

- [x] Task 1: 星表授权尽调 + BSC 数据源落地 (2026-08-03: 234944d)

### Checkpoint: Phase 0
- [x] 授权结论落字(spec Open Questions 关闭一项;不干净 → 启动降级方案重新评审)

### Phase 1: 数据管线

- [x] Task 2: `tools/export-stars.py` + `assets/stars.bin`(银道投影预处理) (2026-08-03: 4171ceb)
- [x] Task 3: Rust 星表解析模块 `starfield.rs` + 纯逻辑单测 (2026-08-03: 1aadecc)

### Checkpoint: Phase 1
- [x] `cargo test --example pomodoro starfield` 全绿;锚点星(织女/牛郎)投影坐标人工核对通过

### Phase 2: 渲染通路

- [x] Task 4: 星野纹理烘焙 + 绑定点接线(`background.rs` + example 上传) (2026-08-03: e77d639)
- [x] Task 5: shader 重写 `star_field`/`star_twinkle`(采样化,旧 hash 网格常量退役) (2026-08-03: 9c958b3)
- [x] Task 6: `star_haze` 暗星雾 + 银纬解析 mask (2026-08-03: 2709373)

### Checkpoint: Phase 2(中途目测)
- [x] 旧底图 + 新星野运行观测: 星点密度分级可辨、星闪落在亮星上、暂停 500ms 沉降语义不变
- [x] 五场景零回归(既有测试全绿)

### Phase 3: 资产与对齐(资产线 Task 7 可与 Phase 1~2 并行)

- [x] Task 7: AI 底图重生 + 挑片(构图契约: 光带左下→右上斜跨、最亮段上 1/3、山脊保留、无可分辨亮星) (2026-08-03: 63860d3)
- [x] Task 8: 光带角度/带宽常量回填 + 三层对齐目测迭代(受 5-commit 预算约束) (2026-08-03: f42a588)

### Checkpoint: Phase 3
- [x] 银河光带一眼可辨且与星表亮星带、暗星雾带三层重合;与倒计时大字区无遮挡冲突

### Phase 4: 收口

- [x] Task 9: benchmark + 全量回归 + 提交三件套 (2026-08-03: f42a588)
- [x] Task 10: 用户终审(与海/雨并排盲切 + spec Success Criteria 逐条过) (2026-08-04: 用户确认通过)

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| Yale BSC 授权不干净 | 高 | Task 1 前置 fail-fast;降级方案: 仅用星座结构自绘位置(血统不破,损失"真实密度"叙事的一半) |
| 启动烘焙超 100ms / 破 1s 门槛 | 中 | Task 4 验收含实测;超限则降烘焙分辨率(640×360 上采样,星点本来就是软点) |
| AI 底图压不住可分辨亮星 | 高 | 多轮挑片是预期成本;兜底: 脚本阈值后期压制;"可分辨"阈值裁定权在用户(spec 灰度防线) |
| 星闪调制新机制观感退化 | 中 | 视觉迭代 5-commit 预算;超限提级,退回"脉冲网格+纹理仅定点"的保守方案 |
| 底图光带角度与解析 mask 错位 | 低 | 银道坐标统一 + Task 8 显式回填步骤 |
| 星野纹理常驻 4MB 破内存门槛 | 低 | 预算 360MB 内两个数量级余量;benchmark Task 9 实测兜底 |

## Open Questions

- 外部 3~5 人第一反应(意图层决定项)——Task 10 终审前由用户定。
- "可分辨亮星"像素/亮度阈值——Task 7 挑片时由用户裁定并回填 spec。
