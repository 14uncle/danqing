# Spec: TitleBar 可选内嵌栏槽 (titlebar-embed)

> @author 十四叔 · @date 2026/09/06
> 前置: interview-me + 用户裁决 A(框架侧)。danqing-log 窗件 v1: 把过滤/搜索输入嵌进标题栏,
> 标题区成为「标题 + 输入 + 三窗键」一条。框架级改动, 全家(log/pomodoro/clipboard)可复用。

## Objective

现有 `TitleBar` 是**叶子组件**(无 `children()`), 只能自绘 logo/标题/按钮。danqing-log 要让输入框
进标题栏, 若产品自建就会复制一份窗口按钮几何/消息逻辑(偏离「产品优先复用框架」)。

本 spec 给 TitleBar 加一个**可选内嵌栏槽**: 一个托管在「标题与窗口按钮之间」的子节点,
产品把搜索/过滤 Bar 塞进去, 标题栏即成为 `logo + 标题 + 内嵌输入 + 三键` 的一条。
**向后兼容**: 未 `embed` 时行为与现状完全一致(仍是叶子行为)。

呈现场景: danqing-log 把过滤+搜索 Bar 放入槽位 ⇒ klogg 式「搜索在标题区」; 后续产品(启动器/便签)
同款。

## Tech Stack

- 框架自身不变(wgpu/winit/等), 不改依赖。
- 复用现有 child-node 组件树机制(`Node`/`children`/`children_mut`)。
- 参考范式: `src/widget/form/text_input.rs`(输入焦点/IME) + base/button.rs(命中), 以及
  danqing-log 的 Bar 的转发纪律(见 `../danqing-log/docs/specs/SPEC-app-chrome.md`)。

## Commands

```bash
# 三件套 + showcase demo
cargo fmt
cargo clippy -- -D warnings
cargo test --lib --tests
cargo run --example danqing-showcase   # 标题栏卡片新增 embed demo
```

## Project Structure

```
src/widget/title_bar.rs  ← 主改动: 加 embed 槽 + 容器化(children) + 布局/事件/焦点路由
src/widget/mod.rs        ← re-export TitleBar 现有 API 不变(embed 是 builder, 无需新导出名)
examples/showcase.rs     ← 标题栏卡片加「嵌入输入槽」demo (以用代测)
src/widget/title_bar.rs #[cfg(test)] ← 新单测 + 既有 TitleBar 测试须全绿(向后兼容)
```

## Code Style

沿用现有 `TitleBar` 的中文 doc + builder 方法返回 `Self`。加:

```rust
/// 内嵌栏槽: 标题与窗口按钮之间托管一个子节点(如搜索/过滤输入), 可选。
/// 未设置时 TitleBar 保持叶子行为(children 为空), 完全向后兼容。
pub fn embed(mut self, widget: impl Widget) -> Self
```

布局规则: logo/标题/按钮区几何维持现状(现有 `button_rect`/`logo_rect` 不改);
`embed` 槽取「标题文字右侧至最左窗口按钮左侧」的**中间余宽**, 竖直撑满栏高。
`layout` 内对该子节点调 `layout` 并把其 area 记在中间; `paint` 在标题文字后、按钮前画它。

## Testing Strategy

- 单测: ①`children()`/`children_mut()` 默认空、embed 后恰 1 元素; ②embed 槽占中间、按钮仍最右
  (复用 `button_rect` 既有断言思路); ③事件命中: 命中中间槽→转发子节点, 命中按钮→仍触发按钮,
  命中标题区→拖拽(现有 `handle_drag_or_double_click` 不回归); ④focus/hit_area 转发到嵌入子节点
  (点击输入落焦, 参考 BottomBar 的 `focus_id`/`hit_area` 转发)。
- 回归: 全部既有 TitleBar 单测(按钮几何/红绿灯/最大化图标/hover/透明背景)必须仍绿。
- 人工: showcase 标题栏卡片, 嵌入一个 TextInput, 验证点击落焦 + 打字 + 在标题区拖拽不冲突。

## Boundaries

- **Always**: embed 保留在框架(全家受益); 既有 TitleBar 全部公开 API/测试不破坏(向后兼容);
  新增 showcase demo; 新 `.rs`(如需)带 @author/@date 头。
- **Ask first**: 同时嵌**两个**或更多子节点(槽设计为单节点, 多输入由产品包一个 Bar 呈现 —— 若产品
  坚持多槽位需重议布局); 改变三窗键/logo/title 的既有几何(不动, 只插中间);
  IME 完整链路若 child-node 机制覆盖不全需手动转发(那会引入 BottomBar 式转发, 属加强项, 需说明)。
- **Never**: 不因 embed 破坏无 embed 产品的渲染(默认零变化); 不复制窗口按钮逻辑到产品。

## Success Criteria

- [ ] `TitleBar::embed` 单节点槽, 未 embed 时完全向后兼容(既有测试全绿)。
- [ ] embed 后: `logo + 标题 + 槽 + 三窗键` 布局正确, 按钮仍最右且可点。
- [ ] 嵌入的输入子节点能落焦 + 收到事件/IME(点击进框打字), 与标题区拖拽不冲突。
- [ ] showcase 有入口; 三件套绿。

## Open Questions

- 嵌入子节点的事件是经框架 child-node 自动分发, 还是 TitleBar 需像 BottomBar 那样手动转发
  (IME/hit/reset_focus)? → plan 阶段按实际 child-node 事件链路实测定, spec 不做预设。
