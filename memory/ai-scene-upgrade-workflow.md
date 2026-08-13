---
name: ai-scene-upgrade-workflow
description: "AI scene image upgrade workflow - strict step-by-step, watermark handling, export-scenes.py pitfalls"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8e52f597-93ad-4a7d-aa17-374eec8856af
  modified: 2026-08-10T01:46:34.861Z
---

## AI 场景底图升级工作流 (严格步骤)

> ⚠️ 必须按顺序执行，不可跳步。

### 步骤

1. **Claude 出 prompt** → 写到 `docs/CONTEXT/ai-image-prompts.md`
2. **用户用 AI 工具生图** → 用户自行放置到桌面 `AI 生图/` 目录
3. **复制到 scenes 目录** → `cp` 到 `assets/scenes/` (如 `starry_ai_1.png`)
4. **去水印** → `python tools/remove_watermark.py <input> <output> [x0 y0 x1 y1]`
   - 先扫描水印精确坐标（`brightness > threshold`），用最小区域覆盖
   - 纹理合成在复杂背景（山脊+天空交界）效果差 → 手动 Pillow 用周围暗色覆盖
5. **更新 export-scenes.py** → `ai_base` 指向去水印后的文件（如 `starry_clean.png`）
6. **运行 export-scenes.py** → 生成 `starry.png` + `scenes.rs`
7. **适配 shader 动效**（如需要）→ 调 `background.wgsl` 参数
8. **测试** → `cargo clippy` + `cargo test --lib --tests` + `cargo test --example pomodoro`

### 关键教训 (2026-08-06 血泪)

- **export-scenes.py 会覆盖图片**: `build_scene()` 读 `ai_base` 指向的文件，处理后 `img.save()` 写回 `assets/scenes/{key}.png`。如果 `ai_base` 指向 `starry.png` 本身，会形成读旧图→写旧图的死循环。**必须指向独立的 AI 源文件**（如 `starry_clean.png`）。
- **export-scenes.py 必须用 ai_base 模式**: 程序化配置（`stops`+`ridges`+`veil`+`milkyway`）会完全覆盖 AI 图。新场景必须改为 `"ai_base": "xxx.png"`。
- **去水印不能跳过**: 用户终审会检查水印。水印区域坐标用 Python 精确扫描，不要猜。
- **水印区域要精确**: 默认 `(w-350, h-120, w, h)` 太大，留下的补丁明显。用 `brightness > threshold` 扫描文字边界，用最小区域覆盖。
- **pomodoro 不要用 noise 叠加层**: `noise.png` 是亮灰纹理（mean 227），在暗场景上造成灰雾。pomodoro 每个场景已有自己的背景图，不需要全局噪声。showcase 可保留。

### 通用规则

- **AI 底图不加暗纱** → 见 [[ai-scene-no-veil]]
- contrast guard 失败可接受, 不要自动加 veil 修复
- shader 参数灵活调整, 不预设"调图适配 shader"或反过来
- `scenes.rs` 是生成文件, 勿手改
- prompt 风格统一: 暗色调电影感、大气雾气、无人无字 1536x1024
- 雨场景底图不含雨丝 (雨由 shader 程序化渲染)

### 动效选择原则 (2026-08-06)

- UV 位移: 大幅运动、区域独立 → 海浪涌动、火焰摇曳 ✅
- Additive 叠加: 小幅氛围、与静态元素重叠 → 山间雾气、森林薄雾 ✅
- 详见 [[ai-scene-uv-displacement-preference]]

### 当前进度 (2026-08-09)

- ✅ Rain — 已完成 (纯程序化雨丝)
- ✅ Sea — 已完成 (UV 涌动 + 碎点 + 水汽)
- ✅ Bonfire — 已完成 (UV 火焰摇曳 + 余烬点缀)
- ✅ Mountain — additive 雾效, 保持现状
- ✅ Forest — additive 雾效, 保持现状
- ✅ Blacksmith — 已完成 (铁匠铺 shader 动效)
- ✅ Cave — 已完成 (洞穴滴水动效)
- ✅ Nightmarket — 已完成 (夜市灯笼动效)
- ✅ Train — 已完成 (火车车厢动效)

Related: [[ai-scene-no-veil]], [[danqing-assets-directory-convention]], [[ai-scene-uv-displacement-preference]]
