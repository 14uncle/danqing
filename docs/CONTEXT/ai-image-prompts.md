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

## 待生成

### 山 (待定)
- **诊断**: 云雾+山画面浑浊, 云雾多层叠加糊了灰阶
- **手术层级**: shader 参数修复 (不换图? 或换图?)
- **状态**: 待确认是否需要 AI 底图

### 火 (待定)
- **诊断**: "说不出哪好哪坏"= 画面没有主角: 只有余烬, 没有木材堆
- **手术**: 补主角: 余烬 → 完整篝火 + 爆裂声 + 火星
- **状态**: 待 AI 底图生成

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
