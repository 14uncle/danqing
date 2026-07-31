# Pomodoro POC — 专注陪伴工具 × 场景沉浸

> 此文件仅在处理 pomodoro 示例时加载。它覆盖了顶级 CLAUDE.md 中与 pomodoro 无关的内容。

## 设计契约

- **美学优先于功能密度**: 大图场景为主角,中央大字倒计时,底部玻璃胶囊控件条。优先保证视觉沉浸,不在计时功能上加复杂度。
- **潮汐式**: 计时运行时世界环绕(动效+环境音全量),暂停/空闲时世界退远(动效沉降、音频淡出)。视觉与听觉同源同步。
- **交叉淡化**: 场景切换不是瞬时跳变——旧/新场景 800ms 交叉淡化,UI token(文字/玻璃色)也按同一进度在调色板间插值。

## 模块导图

| 文件 | 职责 | 关键约定 |
|------|------|----------|
| `main.rs` | 入口、窗口、App trait、每帧 assemble | 常量定义: `FADE_DURATION`(800ms)、`FLASH_DURATION`(600ms)、`NOISE_OPACITY`(0.06)；`rain_clock` 暂停/恢复逻辑 |
| `timer.rs` | 番茄钟核心: Phase/Focus/Break, 25/5 固定 | 纯逻辑,与 UI 解耦；`Run` enum: Idle/Running/Paused |
| `state.rs` | 持久化: JSON → `%APPDATA%/danqing/pomodoro.json` | 跨重启恢复 deadline; `save_state` 节流 1s |
| `scenes.rs` | 5 场景资产声明: 图片路径 + ScenePalette | **由 tools/export-scenes.py 生成,勿手改**；索引顺序: 篝火/海/雨/山/森林 |
| `fader.rs` | 场景交叉淡化状态机(纯逻辑) | from/to + progress + easing; 中途打断按 dominant 侧吸附 |
| `motion.rs` | 场景动效强度策略(纯逻辑) | MotionEnvelope 500ms 线性 envelope；雨例外: 强度不含 envelope,仅雨钟受 envelope 推动 |
| `ambient.rs` | 环境音混音器 + rodio 输出适配层 | from/to 双槽(与场景纹理 LRU 同构)；有 LoopingDecoder 绕过 rodio 0.22 `repeat_infinite` bug |
| `audio.rs` | 完成反馈音效 | Windows 走 `MessageBeep`,其它平台 stub |
| `flash.rs` | 完成反馈视觉脉冲 | 头部满→尾部透明,600ms |
| `hint.rs` | 快捷键提示浮层 | Fade-in/out overlay |
| `today.rs` | "今日完成"计数 | 日期边界检测+自动复位 |
| `tray.rs` | 系统托盘菜单 | winit 0.30 tray API |

## 关键模式

### 场景切换流程
```
用户按 ◀/▶ (或快捷键 1-5)
  → SceneFader::switch_to(target, now)
  → 每帧 fader.frame(now, easing) → (from, to, fade)
  → BackgroundConfig { from_scene, to_scene, fade }
  → 文字/玻璃 token 也在 from/to palette 间按 fade 插值
  → 环境音 AmbientMixer 同步从 from 槽交叉淡化到 to 槽
```

### 动效 + 音频的潮汐契约
```
timer.is_running() == true  → 动效 envelope 目标=1, 音频增益=AMBIENT_VOLUME
timer.is_running() == false → 动效 envelope 目标=0, 音频增益=0 (休息期 duck=0.5)
```
雨例外: `rain_intensity` 不含 envelope,但 `rain_clock` 速度受 envelope 控制(暂停→减速冻结,恢复→加速续走)。

### 纹理 LRU 模式
场景大图不全部预载——维护 2 槽 LRU(from/to),切换方向时换出。详见 memory: `scene-lru-pattern`。

## 常见陷阱

1. **场景索引硬编码**: `motion.rs` 中 `RAIN_SCENE=2` 等的硬编码与 `SCENES` 数组顺序耦合。如果重新排序 `scenes.rs`,必须同步更新 `motion.rs` 中的常量。

2. **Rodio 循环**: rodio 0.22 的 `repeat_infinite()` 对 symphonia 解码器有 bug(循环后无声)。`ambient.rs` 中有自定义 `LoopingDecoder`——不要移除它改用 `repeat_infinite()`。

3. **状态持久化时序**: `save_state` 在 `tick()` 中按 1s 节流,不是每帧写盘。`SystemTime` 作为 deadline 基准,在 resume 时必须用保存的 `remaining` 反算新 deadline,不能简单恢复旧 deadline。

4. **跨淡化时间段**: 场景切换时旧场景的动效(如旧场景的雨)在 fade 过程中逐步减弱——通过 `motion.rs` 的 from/to 双输出实现,不是简单切换。

## 测试

```bash
# pomodoro 具体单元测试 (纯逻辑, 无 GPU)
cargo test --example pomodoro -- --exact

# 相关内存
cargo test fader -- --exact   # 场景淡化器
cargo test motion -- --exact  # 动效策略
cargo test ambient -- --exact # 环境音混音器
```
