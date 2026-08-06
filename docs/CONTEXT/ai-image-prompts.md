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

### 星夜 ✅ (2026-08-06 底图升级)
- **工具**: 元宝 AI
- **提示词**: `Deep dark night sky with prominent milky way band stretching diagonally from lower-left to upper-right, rich galactic dust lanes and dark nebulae within the band, warm yellowish core glow in upper third, deep indigo/navy background (22,26,52), layered dark mountain silhouettes at bottom edge, no visible individual stars, no text, no people, no watermarks, atmospheric depth, cinematic composition, 1536x1024`
- **结果**: `assets/scenes/starry_ai_1.png` → `starry_clean.png` → `starry.png`
- **动效适配**: 无需改动 shader (星野/星闪/流星/星雾均为程序化, 不依赖底图几何)
- **迭代笔记**: v1 银河层次丰富, 尘埃暗隙可辨; export-scenes.py 需改为 ai_base 模式; 水印在山脊+天空交界处, 纹理合成效果差, 改用周围暗色直接覆盖
- **export-scenes.py 改动**: 星夜配置从 stops+ridges+veil+milkyway 改为 ai_base: "starry_clean.png"
- **水印矩形参考**: 推荐尺寸 ≤350×120 (右下角) 或 ≤300×100 (底部中间); bonfire/forest 曾用 400×224 矩形偏大
- **去水印方法**: 暗色场景 (starry/bonfire/forest) 用纯色覆盖比纹理合成更干净; `remove_watermark.py` 默认纹理合成适用于亮色/纹理复杂区域

### 雪原 ✅ (2026-08-06)
- **工具**: 元宝 AI
- **提示词**: `Vast pristine snow field stretching to distant mountain silhouettes, soft blue-white atmosphere, gentle snowfall particles in air, cold indigo twilight sky, no text, no people, no watermarks, atmospheric depth, cinematic composition, 1536x1024`
- **结果**: `assets/scenes/snow_ai_2.png` → `snow.png`
- **动效预留**: 飘雪粒子 (程序化) + 雪面微光 (乘性提亮)
- **调色板**: base (200, 210, 225) 冷蓝白

### 沙漠 ✅ (2026-08-06)
- **工具**: 元宝 AI
- **提示词**: `Rolling sand dunes with smooth curves, warm golden sunset glow at horizon, long shadows across sand ripples, amber and deep orange palette, heat haze atmosphere, no text, no people, no watermarks, vast open landscape, cinematic composition, 1536x1024`
- **结果**: `assets/scenes/sand_ai_2.png` → `sand.png`
- **动效预留**: 热浪空气扭曲 (UV 位移) + 沙尘微粒 (additive)
- **调色板**: base (180, 130, 80) 暖沙色

### 竹林 ✅ (2026-08-06)
- **工具**: 元宝 AI
- **提示词**: `Tall thin bamboo stalks in misty grove, soft diffused light filtering through canopy, teal and emerald green palette, atmospheric fog between stalks, serene Eastern aesthetic, no text, no people, no watermarks, cinematic composition, 1536x1024`
- **结果**: `assets/scenes/bamboo_ai_2.png` → `bamboo.png`
- **动效预留**: 竹叶摇曳 (UV 位移) + 光斑闪烁 (additive) + 薄雾漂移 (additive)
- **调色板**: base (30, 60, 50) 深翠绿

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
- **2026-08-06 UV 位移改造**: fire_breath(径向光晕) + ember_layer(粒子叠加) → fire_sway(UV 位移, 火焰纹理自身横向摇曳); FIRE_CENTER (0.42,0.38) 对齐火焰尖, FIRE_MASK_RADIUS 0.06 只包火焰; 余烬粒子保留为微量点缀

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
| 星夜 | `milky way, galactic dust lanes, deep indigo night sky, mountain silhouettes` |
| 雪原 | `snow field, pristine white snow, distant mountain silhouettes, cold blue atmosphere` |
| 沙漠 | `sand dunes, golden hour sunset, warm amber light, vast desert landscape` |
| 竹林 | `bamboo grove, thin stalks, soft mist, dappled light, green teal palette` |

## 待生成 (2026-08-06)

### 雪原
- **提示词**: `Vast pristine snow field stretching to distant mountain silhouettes, soft blue-white atmosphere, gentle snowfall particles in air, cold indigo twilight sky, no text, no people, no watermarks, atmospheric depth, cinematic composition, 1536x1024`
- **动效预留**: 飘雪粒子 (程序化, 类雨丝范式) + 雪面微光 (乘性提亮)
- **调色板**: base (200, 210, 225) 冷蓝白

### 沙漠
- **提示词**: `Rolling sand dunes with smooth curves, warm golden sunset glow at horizon, long shadows across sand ripples, amber and deep orange palette, heat haze atmosphere, no text, no people, no watermarks, vast open landscape, cinematic composition, 1536x1024`
- **动效预留**: 热浪空气扭曲 (UV 位移, 类海浪范式) + 沙尘微粒 (additive)
- **调色板**: base (180, 130, 80) 暖沙色

### 竹林
- **提示词**: `Tall thin bamboo stalks in misty grove, soft diffused light filtering through canopy, teal and emerald green palette, atmospheric fog between stalks, serene Eastern aesthetic, no text, no people, no watermarks, cinematic composition, 1536x1024`
- **动效预留**: 竹叶摇曳 (UV 位移, 轻柔横摆) + 光斑闪烁 (additive) + 薄雾漂移 (additive, 类森林范式)
- **调色板**: base (30, 60, 50) 深翠绿

## 迭代记录格式

每次迭代记录:
1. 提示词版本
2. 使用工具
3. 生成图路径
4. 去水印方式
5. 动效适配参数
6. 用户反馈
