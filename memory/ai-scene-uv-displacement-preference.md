---
name: ai-scene-uv-displacement-preference
description: "UV 位移适用于大幅运动(海浪/火焰),小幅氛围(additive雾)优先,需按场景选择"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8e52f597-93ad-4a7d-aa17-374eec8856af
  modified: 2026-08-06T08:45:46.768Z
---

UV 位移 vs additive 的选择原则:

- **UV 位移**: 适合大幅运动场景 — 海浪涌动(波涛翻滚)、火焰摇曳(纹理自身舞动)。
  效果: 底图像素本身位移,读作「世界在动」。
- **Additive 叠加**: 适合小幅氛围场景 — 山间雾气飘动、森林薄雾。
  原因: 雾气是独立于地形的半透明物质,叠加合理;且云海与山峰 y 重叠,
  UV 位移会牵连山峰导致整体扭曲。

反面案例:
- 篝火旧方案(余烬粒子叠加) → 割裂感 → 改为 UV 位移成功 ✅
- 山云海 UV 横移 → 山峰跟着扭 → 移除,保留 additive ✅

**Why:** UV 位移会让 mask 区域内所有像素位移;若目标区域与静态元素(山峰/地面)重叠,
无法只动云不动山。Additive 是独立层,不碰底图像素。

**How to apply:** 新场景先评估:运动幅度大且区域独立 → UV 位移;运动幅度小或与静态元素重叠 → additive。
参见 [[ai-scene-upgrade-workflow]]。
