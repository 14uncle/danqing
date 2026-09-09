# SPEC: anim 动画原语模块 (框架下沉·簇A)

- @author 十四叔
- @date 2026/09/09
- 状态: **已批准**（2026-09-09）；框架侧 T1–T7 已提交并 push（danqing fdd3d01，三件套绿 495 通过）；review 通过（APPROVE，Critical/Required 零发现；2 条 Optional 已处置：Tween 有限值 doc + 原地补间短路）；pomodoro 联动 T8 已落地（零 commit，184 测试全绿、四类型零残留），T9 实机四姿势 + pomodoro 提交待用户裁决
- 需求来源: `docs/intent/framework-sinking.md` 簇A（跨产品重复发明 4+ 处，成本 S，建议顺序第一）

## 目标

把「目标值滑动包络 / 一次性脉冲 / 提示时序 / 离散交叉淡化」四个动画原语从 pomodoro 产品侧下沉为 danqing 公共模块 `src/anim.rs`（纯逻辑，时间外部注入，不读 wall-clock，不依赖 winit/wgpu），并迁移框架内两个既有消费者（`audio/mixer.rs`、`widget/form/switch.rs`）与 pomodoro 侧四个文件。

**非目标**（明确不做）：
- pomodoro `ambient.rs` AmbientMixer → `danqing::audio::Mixer` 的迁移（收尾任务另起，需实机听感验收）
- 泛型/多值补间（Color/Point/数组）——v1 只做 f32 标量，等真实消费者再扩
- 物理弹簧/关键帧/时间线编排等高级动画设施
- xirang（已埋）/ clipboard 侧迁移
- 能力地图：本簇是单模块四类型，不拆子模块 spec，任务分解归 plan 阶段

## API 设计

### `Tween` —— 通用补间包络（本体，采 ChannelEnvelope 语义）

```rust
pub struct Tween { /* current, anim: Option<(f32, f32, Duration)>, duration, easing */ }
impl Tween {
    pub fn new(duration: Duration, easing: Easing) -> Self;
    pub fn with_initial(self, initial: f32) -> Self;
    /// 推进并返回当前值: 目标变化(含中途反向/持续微变)→从当前值起
    /// 走全程 duration 续接(无跳变); 稳定态精确到达目标(无渐近漂移)。
    pub fn value(&mut self, now: Duration, target: f32) -> f32;
    /// 动画是否进行中 (供 OnDemand 模式消费者决定 boost_frames)。
    pub fn is_animating(&self) -> bool;
}
```

语义决策（与源头的差异，逐条有意为之）：
1. **目标追踪式重触发**（采 `mixer.rs:44-51` 的 `needs_anim` 判定），而非 MotionEnvelope 的 bool 边沿检测——能平滑跟随逐帧微变的目标（时辰曲线范式），边沿场景行为等价。
2. **反向续接走全程 duration**（四处源头的共同语义），不采 switch 的「按剩余距离比例耗时」——统一后 switch 快速双击翻转观感不变坏（150ms 全程），行为更可预测。
3. **值域不 clamp**——0..1 夹取是消费者语义（Mixer 在 `set_target` 夹），Tween 本体允许任意 f32 值域（尺寸/位置/透明度通吃）。
4. **easing 在包络内整形**：`value = start + (target - start) * easing.eval(t)`，端点 t=0/1 精确（Easing::eval 自夹）。

### `Pulse` —— 一次性脉冲（pomodoro `flash.rs` 原样语义）

```rust
pub struct Pulse { /* started: Option<Duration>, duration */ }
impl Pulse {
    pub fn new(duration: Duration) -> Self;
    pub fn trigger(&mut self, now: Duration);          // 进行中触发被忽略; 已结束可再触发
    pub fn progress(&self, now: Duration) -> Option<f32>; // 1.0→0.0 线性衰减; 未激活/已结束 None
    pub fn is_active(&self, now: Duration) -> bool;
}
```

### `Cue` —— 提示时序（pomodoro `hint.rs` 参数化）

```rust
pub struct CueTiming { pub delay: Duration, pub fade_in: Duration, pub hold: Duration, pub fade_out: Duration }
// Default = pomodoro 现值: 1.5s / 300ms / 5s / 500ms
pub struct Cue { /* triggered_at: Option<Duration>, timing: CueTiming */ }
impl Cue {
    pub fn new(timing: CueTiming) -> Self;   // 未激活
    pub fn trigger(&mut self, now: Duration);
    pub fn progress(&self, now: Duration) -> Option<f32>;  // 静默 0 → EaseOut 淡入 → 满 → EaseIn 淡出 → None
}
```

### `Crossfade` —— 离散状态交叉淡化（pomodoro `fader.rs` 原样语义）

```rust
pub struct Crossfade { /* from: usize, to: usize, start, duration */ }
impl Crossfade {
    pub fn new(initial: usize, duration: Duration) -> Self;
    pub fn current(&self) -> usize;
    /// 切换目标; 淡化中打断按进度占优侧(>=0.5)吸附为新起点。
    pub fn switch_to(&mut self, target: usize, now: Duration);
    pub fn progress(&self, now: Duration) -> f32;                    // 0..=1, 静止恒 1
    pub fn frame(&self, now: Duration, easing: Easing) -> (usize, usize, f32);
}
```

离散状态用 `usize` 索引表示（任何枚举可映射），与 `BackgroundFrame::new(from, to, fade)` 消费形态直通。

### `Easing` 扩展（`src/theme.rs` 原地加变体，不搬家）

```rust
pub enum Easing { Linear, EaseInOut, EaseIn, EaseOut }  // 后两个为新增, 三次方曲线
```

Easing 留在 theme.rs（它是设计 token），anim.rs 引用之。`EaseIn = t³`、`EaseOut = 1-(1-t)³`（即 pomodoro hint.rs 被迫自带的两个私有函数，收编为 token）。

## 结构

- 新增 `src/anim.rs`（文件头 `@author 十四叔` / `@date 2026/09/09`；模块头中文 doc 写明做什么/不做什么/为什么——「做什么」四原语，「不做什么」非目标清单，「为什么」重复发明 4+ 处的盘点结论）
- `src/lib.rs` 加 `mod anim;` + `pub use anim::{Tween, Pulse, Cue, CueTiming, Crossfade};`（公开 API 一律经 lib.rs re-export）
- `src/theme.rs` Easing 加两变体
- 修改 `src/audio/mixer.rs`：私有 `ChannelEnvelope` 删除，`Mixer` 内部改用 `Tween`（300ms / Linear；0..1 夹取留在 `set_target`）
- 修改 `src/widget/form/switch.rs`：`animate()` 的 dt 步进改 `Tween`（150ms / Linear；语义差异见上 §2；时间源用 `AnimationCtx.elapsed`，无需自维护 epoch）
- `examples/showcase.rs` 加「动画原语」演示卡（以用代测：三按钮触发 Tween 目标翻转/Pulse/Crossfade + 数值实时回显；视觉消费实例为同在 showcase 的 switch(Tween)/mixer，小体量）

## 消费者迁移（pomodoro 仓，联动提交）

danqing 提交并 push 后，pomodoro `cargo update -p danqing`，两仓分别提交：

| pomodoro 文件 | 处置 |
|---|---|
| `motion.rs` | 删 `MotionEnvelope`（→ `Tween::new(500ms, Linear)`，调用点 `gain(running, now)` 改 `value(now, target)`）；场景强度函数/场景索引常量**保留**（产品语义） |
| `flash.rs` | 整文件删除 → `danqing::Pulse`（600ms） |
| `hint.rs` | 整文件删除 → `danqing::Cue`（默认 timing）；私有 ease_in/out_cubic 随之消灭 |
| `fader.rs` | 整文件删除 → `danqing::Crossfade`（800ms；`frame(now, \|t\| t)` 改 `frame(now, Easing::Linear)` 或实际用曲线） |
| `ambient.rs` | **不动**（非目标） |

## 测试策略

- **移植**：pomodoro 四文件的可搬测试语义全保（MotionEnvelope 3 + Flash 6 + Hint 7 + Fader 8 = 24 条），逐条改写为新 API 等价断言（边沿帧连续/中点/终点精确/反向续接/打断吸附双向/时间轴平移/重复触发保护）
- **新增**：Tween 的 easing 整形断言（EaseInOut 中点=0.5、1/4 点更低）、持续微变目标平滑跟随（mixer 时辰范式下沉为 Tween 直测）、任意值域不夹取；Easing 新变体端点精确 0/1 + 单调性 + 中点方向性
- **回归**：mixer 既有 8 测试**零改动全绿**（行为不变是迁移正确的证据）；switch 无单测 → showcase 实机观感对照
- 三件套：danqing `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests`；pomodoro 同三件套

## 验收标准

- [ ] `danqing::Tween/Pulse/Cue/CueTiming/Crossfade` 经 lib.rs 可用，Easing 四变体齐
- [ ] 24 条移植测试 + 新增测试全绿；mixer 8 测试零改动通过
- [ ] showcase 演示卡可实机触发三种原语；switch 拨动观感不劣化（人工）
- [ ] pomodoro 迁移后既有测试全绿；`MotionEnvelope`/`FlashOverlay`/`ShortcutHintOverlay`/`SceneFader` 零残留引用
- [ ] 两仓三件套绿；danqing 先 push，pomodoro 后提交 lock（注明关联）

## 边界

- 公开 API 一律经 `src/lib.rs` re-export；anim.rs 保持纯逻辑（依赖方向铁律：不得碰 winit/wgpu）
- 注释/文档一律中文；魔法数字提 const 附 doc
- 未获用户指示不 commit/push；两仓联动改动分别提交、message 注明关联
- 时间一律外部注入 `Duration`，任何原语不得读 wall-clock（可测性是本模块的存在理由）
