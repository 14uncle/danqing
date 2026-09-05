---
name: ambient-audio-workflow
description: "环境音供应链与工作流: Freesound CC0-only + 合成混音 + 许可台账; 资产库 Desktop\\danqing\\环境音; rodio 先例在 pomodoro; xirang(息壤) 需分层 stems + 混音上引擎"
metadata:
  type: project
---

## 环境音资产库与工作流 (pomodoro 已跑通, 2026-08-29 用户口述确认)

- **资产库**: `C:\Users\gwhun\Desktop\danqing\环境音` (2026-08-07 建, ~20 mp3, 按场景组织: 夜市/洞穴/火车/铁匠铺, 含候选与最终合成)
- **工作流**: 按场景下候选 (Freesound.org) → 试听筛选 → 合成混音 (交叉淡化底噪 + 定点混入点缀) → 许可台账 (`来源与许可.md` 逐文件记录作者/URL/时长/许可, 含下载日期)
- **许可红线**: 只收 **CC0** (公有领域, 可商用/可修改/无需署名) —— 对付费产品与 DLC 同样安全; 「关于」页列作者更体面
- **播放先例**: danqing-pomodoro 用 rodio 0.22 (default-features = false + playback + symphonia-ogg/vorbis)

**Why:** xirang(息壤) 的声景是留存主力 (隐性钩子), 时辰 crossfade 需要分层 stems + 运行时混音 —— 与 pomodoro 的预合成单 loop 不同; 混音能力留在产品侧会拉低引擎复用率 (产品线核心指标)。

**How to apply:** xirang scene-world/音频任务: 新素材沿用 CC0-only + 台账; 新场景音轨按分层 stems 制作 (底噪/时辰层/点缀层分开), 不做预混单文件; 混音与播放能力提上 danqing 引擎并经 lib.rs re-export (已入 docs/intent/third-product-desk-scene.md 引擎新肌肉复核); 最终选定音轨随场景入仓库资产 (沿用 [[danqing-assets-directory-convention]]), 不从 Desktop 散养目录引用。相关: [[ai-scene-upgrade-workflow]]
