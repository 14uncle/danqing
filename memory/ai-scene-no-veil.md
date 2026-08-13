---
name: ai-scene-no-veil
description: AI-generated scene backgrounds must not have veil/dark gauze overlays
metadata: 
  node_type: memory
  originSessionId: 8e52f597-93ad-4a7d-aa17-374eec8856af
  modified: 2026-08-06T01:27:39.538Z
---

AI 生成的场景底图 (bonfire, mountain, forest, rain 等) 不加暗纱 (veil)。
用户明确要求保留 AI 图的自然亮度, 不要用 veil 压暗。

**Why:** 暗纱会杀死 AI 图的自然光影质感 (落日/余烬/云层亮度), 用户多次要求去掉。
**Why:** contrast guard 失败是可接受的——技术约束不应牺牲视觉质量。

**How to apply:** export-scenes.py 中 AI 场景 (有 `ai_base` 字段的) 不配置 `veil`。
contrast guard 失败时通知用户即可, 不要自动加 veil 修复。

Related: [[danqing-assets-directory-convention]]
