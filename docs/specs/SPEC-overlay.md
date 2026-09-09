# SPEC: Overlay 模态浮层组件 (框架下沉·簇C)

- @author 十四叔
- @date 2026/09/09
- 状态: **已批准**（2026-09-09，用户发起 plan 阶段即批准）；框架侧 T1–T3 已落地（/build auto 零 commit）；review REQUEST CHANGES 已处置（C1 卡内未消费事件穿透底层 + C2 hit_area 覆写抢焦，均修；R1 关层边沿 reset_focus + R2 paint_image 透传，均修；语义清单已按修后行为更新）
- 需求来源: `docs/intent/framework-sinking.md` 簇C（重复发明 6 份，全场最多；成本 M，建议顺序第三；簇A anim / 簇B job 已落地）

## 目标

把「模态浮层」从三产品的 6 份手写拷贝下沉为 danqing 组件 `widget/view/overlay.rs` 的 `Overlay`：open 态绑定 + scrim 遮罩 + 居中内容卡槽 + 关态零尺寸不拦事件 + 开态自开渲染新层 + 可选点遮罩关闭。一次删除 pomodoro ~350 行 / log ~200 行 / clipboard 两处手写。

**形态说明**：本簇是 widget 组件，落 `widget/view/`（MultiPanel/Tabs 同族），不涉三政策门。框架既有配套全部就位，本组件是它们的组装者而非新设施：`theme.scrim()` token、`RectBatch/TextBatch::push_layer()`、`App::focus_request`/`focus_restored` 协议（focus.rs 注释本就写着「弹层面板关闭后焦点回到打开面板的按钮」）、`Button::id()` 稳定焦点标识。

**非目标**（明确不做）：
- **Esc 关闭不进组件**：产品语义分裂——pomodoro 无 Esc 关闭；log 有且须先清 TextInput 焦点再关层（S3 回归教训，两层 Escape 竞争）。Esc 留在产品侧，组件 doc 写明范式
- **焦点回归编排不进组件**：框架 `App::focus_request`/`focus_restored` 协议已在，产品侧两行调用即可（pomodoro 现网即如此）
- **面板互斥状态机**（pomodoro 三面板互开互关）是产品状态，不进组件
- **开合动画** v1 不做（三产品现状即无动画；未来可叠簇A Cue/Tween）
- log 的独立 `Scrim` 组件不单独下沉（被 Overlay 内化）
- xirang（已埋）

## API 设计

```rust
/// 模态浮层: open 态绑定的 scrim 遮罩 + 居中内容卡。
pub struct Overlay { /* open 绑定闭包 / 内容子树 / 点遮罩消息工厂 / 卡片 area 缓存 */ }
impl Overlay {
    /// 默认浅色主题 token 创建 (scrim 色走 theme.scrim())。
    pub fn new(content: impl Widget + 'static) -> Self;
    pub fn themed(theme: &impl Theme, content: impl Widget + 'static) -> Self;
    /// 绑定 open 态: 每帧从应用状态读取 (关态零尺寸不拦事件, 见语义清单)。
    pub fn bind_open<S: 'static>(self, f: impl Fn(&S) -> bool + 'static) -> Self;
    /// 点遮罩关闭消息 (opt-in; 不设则遮罩只吞事件不发消息)。
    pub fn on_scrim_click<M: 'static>(self, f: impl Fn() -> M + 'static) -> Self;
}
```

**语义清单**（从三产品现网归纳，经首评 C1/C2/R1/R2 修订；前 5 条对应 log `settings.rs:56-139` 的最完整形态）：
1. sync/animate/layout/paint/event/children 全线 open 门控——关态子树完全休眠
2. 关态 layout 返回 `(constraints.max().width, 0)`（log 实证形态；勿置 Row 内）
3. 开态 paint 前 `rects.push_layer()` + `texts.push_layer()`（log 教训内化：同层文本恒在矩形之上，卡片须开新层才能盖底层表格文本）
4. 开态定位事件一律 Consumed：卡内透传内容但结果不冒泡（内容 Ignored ≠ 放行底层——log Scrim 全窗兜底语义）；卡外吞掉，左键点遮罩有 `on_scrim_click` 时发消息；遮罩上 CursorMoved 透传内容清 hover 残留；键盘等非定位事件透传且结果原样返回（Esc→清焦协议依赖 Ignored 传播）
5. 关态 children 为空（焦点系统跳过整棵子树）；容器恒不可聚焦 + 不覆写 hit_area（Tabs 范式：FocusManager collect 递归直达卡内组件——容器进焦点命中会遮蔽卡内组件，首评 C2 教训）
6. 开→关边沿对内容子树调 `reset_focus`（关态后 FocusOut 送不进空 children；MultiPanel 判例：藏子树的容器负责清焦点视觉）
7. 开态转发 `paint_image`（卡内 Image 可渲染）；已知限制：底层页面 Image 恒穿 scrim（ImageBatch 无分层）

## 结构

- 新增 `src/widget/view/overlay.rs`（文件头 @author/@date；模块头 doc 做什么/不做什么/为什么——「为什么」锚 6 份拷贝的盘点结论 + push_layer 教训出处）
- `src/widget/view/mod.rs` 登记 + `src/lib.rs` re-export `Overlay`（公开 API 一律经 lib.rs）
- `examples/showcase.rs` 加「模态浮层 (Overlay)」演示卡（以用代测，新组件铁规）：打开按钮 → scrim + 玻璃卡（关闭按钮 + 说明文字）；点遮罩关闭演示 opt-in 消息；关态零尺寸可经底层按钮仍可点来反证

## 消费者迁移（三仓分批，danqing push 后）

| 仓 | 处置 | 前提 |
|---|---|---|
| pomodoro | settings/stats/report 三面板内容保留，浮层机构换 Overlay（~350 行删除）；互斥与焦点回归逻辑留产品侧 | 无 |
| danqing-log | `SettingsOverlay` 整删 → `danqing::Overlay`（~200 行）；产品 Esc 前置关卡保留 | **log 在途批次先提交**（与簇B T6 同闸） |
| danqing-clipboard | 设置面板 + 清空确认（双层叠加）两处换 Overlay；日用在用，迁移即引擎复用再验证 | 用户裁决是否动 |

- 各仓迁移验收：既有测试全绿 + 人工姿势（pomodoro：三面板开合/互斥/焦点回归/点面板外不穿透；log：设置卡盖表格文本清晰/Esc 两阶段；clipboard：双层叠加 z 序）
- 迁移顺序建议 pomodoro → log → clipboard，一仓一提交，message 注明关联

## 测试策略

- 框架纯逻辑单测（新写，零可搬——三产品的测试都测产品状态逻辑，留各仓）：
  关态 layout 零尺寸 / 关态 event Ignored / 关态 children 空 + focusable false；
  开态遮罩点击发消息且消费 / 卡片内点击透传子树 / 无 on_scrim_click 时遮罩点击仅消费；
  开态 paint 后 RectBatch/TextBatch 层数 +1（push_layer 断言）
- showcase 演示卡人工过目（以用代测）
- 三件套：`cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests`

## 验收标准

- [ ] `danqing::Overlay` 经 lib.rs 可用，六条语义各有单测锁定
- [ ] showcase 演示卡可实机开/关/点遮罩关闭（人工）
- [ ] 三仓迁移后既有测试全绿，三产品手写浮层机构零残留
- [ ] 迁移按仓分批提交，message 注明关联 danqing 提交；danqing 先 push

## 边界

- Overlay 是纯逻辑 widget（不碰 winit/wgpu），依赖方向铁律不变
- 颜色/间距/圆角一律走 Theme token（scrim/surface/radius_lg/spacing_xl），不自定义
- 注释/文档一律中文；公开 API 经 lib.rs re-export
- 未获用户指示不 commit/push；联动改动分仓分别提交
