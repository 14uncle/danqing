# Spec: 沉浸世界九场景

## Objective

将番茄钟「沉浸世界」从5个场景扩展到9个，核心标准：**窗口隐藏后3秒内通过环境音辨识场景**。每个场景需有独立的 AI 底图、shader 动效、程序化环境音。

## 假设

1. 遵循现有场景架构（SceneSpec + shader + procedural audio）
2. AI 底图由用户用元宝 AI 生成，Claude 适配动效
3. 环境音用 export-ambient.py 程序化生成（FFT shaped noise + tonal elements）
4. Uniform buffer 保持64B（16×f32），不新增字段（动效通过 from/to 淡化复用现有字段）
5. 新场景的 shader 动效写在 background.wgsl 中，通过现有 uniform 字段控制

## 九场景清单

| # | 场景 key | 中文名 | 声音签名 | 元素类型 |
|---|---------|--------|---------|---------|
| 0 | bonfire | 篝火 | 木柴噼啪 | 火 |
| 1 | sea | 海浪 | 水浪涌动 | 水 |
| 2 | rain | 雨夜窗边 | 雨滴敲击 | 水 |
| 3 | mountain | 山风 | 持续风声 | 风/地 |
| 4 | forest | 森林 | 鸟鸣叶响 | 植被 |
| 5 | blacksmith | 铁匠铺 | 金属锤击 | 人造/金属 |
| 6 | cave | 洞穴 | 滴水回声 | 石/地下 |
| 7 | nightmarket | 夜市 | 人声喧嚣 | 人造/人声 |
| 8 | train | 火车车厢 | 铁轨节奏 | 人造/机械 |

## 声音辨识设计

每个场景的环境音必须有**不可替代的标志性音素**：

| 场景 | 标志性音素 | 频率特征 | 排除 |
|------|-----------|---------|------|
| 篝火 | 木柴爆裂 (pop/crackle) | 2-8kHz 瞬态脉冲 | — |
| 海浪 | 周期浪涌 (swash/backwash) | 0.1-0.5Hz 调制 | — |
| 雨夜窗边 | 雨滴敲击玻璃 (tap) | 4-12kHz 瞬态 | — |
| 山风 | 持续宽频风声 | 100-800Hz 为主 | — |
| 森林 | 鸟鸣 (chirp) | 2-6kHz 间歇 | — |
| 铁匠铺 | 锤击铁砧 (clang) | 1-4kHz 金属共鸣 + 节奏 | 不能像篝火噼啪 |
| 洞穴 | 滴水回声 (drip+echo) | 1-3kHz + 混响尾 | 不能像雨滴 |
| 夜市 | 人声嘈杂 (chatter) | 300-3000Hz 语音频段 | 不能像鸟鸣 |
| 火车车厢 | 铁轨节拍 (clack-clack) | 2-4Hz 周期 + 低频轰鸣 | 不能像海浪周期 |

## Tech Stack

- Shader: WGSL (background.wgsl)
- 渲染: wgpu 30, BackgroundFrame uniform buffer
- 音频: rodio 0.22 + 自实现 LoopingDecoder
- 音频生成: Python export-ambient.py (numpy + scipy + ffmpeg)
- 图片生成: 用户手动用元宝 AI

## Commands

```bash
# 运行番茄钟
cargo run --example pomodoro

# 生成环境音
python3 tools/export-ambient.py

# 导出场景图
python3 tools/export-scenes.py

# 测试
cargo test --lib --tests
cargo clippy -- -D warnings
cargo fmt --check
```

## Project Structure

```
examples/pomodoro/
  scenes.rs       → SceneSpec 定义 (9条)
  motion.rs       → 场景动效策略 (9组)
  ambient.rs      → SCENE_AUDIO 音频映射 (9条)
  main.rs         → 场景强度接入
src/render/
  background.wgsl → shader 动效 (9组)
  background.rs   → BackgroundFrame + uniform buffer
tools/
  export-ambient.py → 程序化音频生成
  export-scenes.py  → 场景图导出
assets/
  scenes/         → 9张场景底图
  audio/          → 9个环境音 OGG
```

## Code Style

遵循项目约定：中文文档注释、英文命名、`@author 十四叔` 文件头。

Shader 函数命名：`{scene}_effect(uv, rt, intensity)` 或 `{scene}_layer(uv, rt)`。

环境音函数命名：`_tonal_elements_{scene}(rng, N)` 返回 (left, right) stereo。

## Testing Strategy

- 单元测试：motion.rs 场景权重计算、ambient.rs 音频数组对齐
- 集成测试：assets.rs 检查9个 OGG +9个 PNG 存在
- 手动测试：运行 pomodoro，逐个切换场景，窗口隐藏后听音辨识

## Boundaries

- **Always**: 每个场景必须有独立的标志性音素；shader 动效不能与其他场景重复
- **Ask first**: 替换现有场景的底图；修改 uniform buffer 结构
- **Never**: 删除现有场景的动效代码；在 shader 中硬编码颜色值

## Success Criteria

1. `cargo test --lib --tests` 全绿
2. `cargo clippy -- -D warnings` 零警告
3. 9个场景各有独立的 AI 底图（assets/scenes/*.png）
4. 9个场景各有独立的环境音（assets/audio/*.ogg）
5. 窗口隐藏后，每个场景的环境音可在3秒内被辨识
6. 新场景的 shader 动效与现有场景视觉风格一致

## Open Questions

1. 铁匠铺的锤击节奏用什么频率？（建议1-2Hz，模拟人工打铁节奏）
2. 洞穴的混响时长？（建议0.5-1s，模拟中等大小洞穴）
3. 夜市的人声是中文还是多语言混合？（建议模糊化处理，不做语义区分）
4. 火车的铁轨节拍用什么节奏？（建议2-4Hz，模拟普通列车匀速行驶）
