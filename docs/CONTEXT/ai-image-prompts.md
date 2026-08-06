# AI 场景底图生成提示词

> 按需加载。记录各场景 AI 生图提示词与迭代历史,支持渐进式优化。
> 工作流: Claude 提示词 → 用户用 AI 工具生成 → Claude 适配动效。

## 已完成

### 森林 (2026-08-04)
- **工具**: 元宝 AI
- **提示词**: `Misty pine forest at twilight, layered depth with atmospheric fog, silhouetted conifers with detailed branches, warm sunset glow through clouds at top, dark teal/green palette, no text, no people, cinematic composition, 1536x1024`
- **结果**: `assets/scenes/forest_yuanbao_clean.png` → `forest.png`
- **动效适配**: 暗纱加强 (peak 95), forest_mist shader 无需改动
- **迭代笔记**: 即梦版本水印去除困难,元宝更干净

## 已完成 (升级)

### 海 ✅ (2026-08-06 通过)
- **工具**: 元宝 AI
- **提示词**: `Dark dramatic ocean at dusk, rolling waves with foam crests, deep navy/teal water, moody overcast sky with warm sunset glow at horizon, atmospheric mist over water surface, no text, no people, cinematic composition, 1536x1024`
- **结果**: `assets/scenes/ocean_ai_3.png` → `sea.png`
- **动效适配**: GLINT_BAND_TOP 0.72→0.48 对齐 AI 底图浪花带; SEA_MASK_TOP 0.55 不变
- **迭代笔记**: v1 偏平无层次; v2 偏暗落日偏左; v3 居中落日+浪花层次最佳

### 雨 ✅ (2026-08-05 通过)
- **工具**: 元宝 AI
- **提示词**: `Dark moody rainstorm sky, heavy gray-blue clouds with atmospheric depth, rain-soaked atmosphere with misty haze, no rain streaks in image, silhouette of distant treeline at bottom edge, dark teal/slate palette, cinematic composition, no text, no people, 1536x1024`
- **结果**: `assets/scenes/rain_ai_2.png` → `rain_ai_clean.png` → `rain.png`
- **动效适配**: 无需改动 shader (雨丝全部程序化渲染); 加暗纱 (peak 75) 压中心亮度保对比度
- **迭代笔记**: v1 有雨丝(与 shader 冲突)且无地面元素; v2 暴风云+树线剪影更佳

## 待重新生成

### 山 ✅ (2026-08-04 通过)
- **工具**: 元宝 AI
- **提示词**: `Layered mountain ridgelines at sunset, cloud fog flowing between peaks, warm orange sunset glow at horizon, dark purple/indigo palette, misty atmosphere with depth, no text, no people, cinematic composition, 1536x1024`
- **结果**: `assets/scenes/mountain_ai_clean.png` → `mountain.png`
- **动效适配**: 双层云雾动效 (ec85f78): 主云海 Y=0.25-0.55 + 薄雾 Y=0.45-0.65
- **迭代笔记**: v1 云海在上半部, 动效在下半部不匹配; 改为动效适配图 (双层云雾)

### 火 ✅ (2026-08-04 通过)
- **工具**: 元宝 AI
- **提示词**: `Small campfire with stacked birch logs, centered composition, warm orange flames, glowing embers, dark forest background at night, lots of dark space around the fire, no text, no people, cinematic composition, 1536x1024`
- **关键**: 火堆小一点、居中、四周留暗保对比度
- **迭代笔记**: v1 裁剪破坏构图; v2 明火太大; v3 改为余烬/火炭概念 (无明火), 动效改为火星从炭堆升起
- **结果**: `assets/scenes/bonfire_ai_3.png` → `bonfire.png`
- **动效适配**: FIRE_CENTER.y 0.65→0.50, EMBER_SPAN 0.52→0.38, veil peak 65→45
- **余烬聚焦**: EMBER_DENSITY 160→60, EMBER_SWAY 0.006→0.002, band收窄至x=0.25-0.50, EMBER_SPAN 0.38→0.15
- **火星优化**: EMBER_SPEED 0.25→0.40 (加快), EMBER_RADIUS 0.006→0.004 (缩小), EMBER_COLOR→橙红色 (1.0,0.45,0.15)

## 提示词模板

### 通用约束
```
no text, no people, no watermarks, cinematic composition, 1536x1024
```

### 风格关键词
```
atmospheric, moody, dark palette, depth of field, layered composition
```

### 场景特定关键词
| 场景 | 关键词 |
|------|--------|
| 森林 | `misty pine forest, conifers, atmospheric fog, twilight` |
| 山 | `mountain ridgelines, cloud fog, dusk, layered peaks` |
| 火 | `bonfire campfire, embers, warm glow, dark forest background` |

## 迭代记录格式

每次迭代记录:
1. 提示词版本
2. 使用工具
3. 生成图路径
4. 去水印方式
5. 动效适配参数
6. 用户反馈
