---
name: scene-lru-pattern
description: "多场景(wgpu 纹理池)的 2 槽 LRU 懒加载模式 — PNG 字节预读 + 按需 create_texture, 适用于\"池大、同时只显示 from/to 跨淡化两端\"的场景"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 07cc6842-0680-4e27-9fdf-19c9c3c16744
  modified: 2026-07-24T08:27:29.757Z
---

# 场景纹理 2 槽 LRU 模式(danqing)

## 适用形态
- 有 N 个场景图(N 远大于 2)
- 同时只显示 1 对(`from` + `to` 跨淡化)
- N 张全驻 GPU 浪费巨大(N × 6.3MB 实测)

## 实现位置
`src/render/background.rs`(`BackgroundPipeline`),2026-07-24 落地。

## 关键设计
1. **预读** `Vec<Vec<u8>>` 全部 PNG 字节(`new` 阶段,~1MB 总)
2. **不**创建 wgpu 纹理,只持有 `device`/`queue`/`texture_layout` 的 clone(wgpu 30 都是内部 Arc,廉价)
3. `ensure_loaded(idx)`:
   - 命中 → 刷新 LRU 顺序
   - 未命中 → 从 `scene_bytes[idx]` decode + create_texture + write_texture
   - 满 → pop `VecDeque` 尾,drop `BackgroundTexture`(wgpu 通过 `Drop` 自动释放)
4. `set_frame` 对 `from`/`to` 各调一次 → `draw` 假设两端就绪
5. `has_background` 改判"已配置"而非"已加载",首帧语义正确
6. `draw` 加缺失分支优雅降级(单图无淡化,都缺则让 clear_color 透出)

## 容量选择
`SCENE_CACHE_CAPACITY = 2` 即可:`from` + `to` 跨场景淡化的两端,中断切换时 fader 自身按 dominant 吸附(参考 `examples/pomodoro/fader.rs`),不会同时需要 3 张。

## 跨场景淡化的时序保证
- 800ms 淡化 / 60fps ≈ 48 帧
- 一次 decode+upload 6MB PNG ~1ms
- 新场景纹理在 fade=0 之前已就绪,无清屏色闪帧
- 首帧(`from == to == 0`,fade 起步)会触发一次 6MB decode+upload,在 ≤1s 冷启门槛内可忽略

## 不可测部分
LRU 行为依赖 `wgpu::Device`,纯逻辑单测需要 `pollster::block_on(Context::new_async)`,性价比低。当前靠 `tools/benchmark.ps1` 复测 + 手动目测验证。

## 复用范围
任意"池大、同时活跃子集小、活跃子集动态变化"的多资源场景(纹理 / 音频缓冲 / 着色器变体等)。
