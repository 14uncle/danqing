# Plan: TitleBar 可选内嵌栏槽 (titlebar-embed)

> spec: `docs/specs/SPEC-titlebar-embed.md`; 框架级, danqing-log app-chrome 依赖此。
> 目标: 给 TitleBar 加一个「标题与三键之间」的托管槽, 产品把搜索/过滤 Bar 塞进去。
> 向后兼容: 未 embed 时零变化(仍是叶子行为)。

## Overview

`TitleBar` 现为叶子(无 `children`)。加一个可选 child(槽), 置于标题文字右侧、三键左侧的中间余宽,
经框架 child-node 机制参与布局/事件/焦点。产品端把 `Bar`(双 TextInput)塞入, 标题栏 = `logo+标题+输入+三键`。

## Architecture Decisions

1. **单槽而非多槽**: `embed(impl Widget)` 存一个 `Option<Node>`; 多个输入由产品包一个 Bar 呈现(单节点),
   槽只托管一个。多槽位留待有真实需求再扩(不现在做)。
2. **child-node 机制 vs BottomBar 手动转发**: 采用 child-node(children/children_mut)让框架接管布局/事件/焦点;
   IME/hit 若 child-node 链路覆盖不全, 按实际实测定 → 可能需在 TitleBar 补转发(同 BottomBar 纪律)。
   **Open Question → T3 实测定**, 不预设。
3. **几何守恒**: 现有 `logo_rect`/`button_rect`/三键/拖拽逻辑不动; 只插中间。title 文字起点跟随 logo、
   终点让位给槽与按钮(槽不存在时 title 可延长)。

## Task List

### T1: TitleBar 容器化 + embed 槽
- **Acceptance**: `TitleBar::embed(impl Widget)` builder 存 `Option<Node>`; 新增 `children()/children_mut()`
  (默认空, embed 后恰 1); 未 embed 时整机行为不变(既有单测全绿)。
- **Verify**: `cargo test widget::title_bar::tests` — 新增 embed 结构断言 + 既有全过。
- **Files**: `src/widget/title_bar.rs` | M | `Node`/`children` 接入

### T2: 布局 — 槽占标题与三键之间中间余宽
- **Acceptance**: `layout` 先算 logo/title 宽与三键区起点; 槽取其间余宽、竖直撑满栏高, `layout` 子节点;
  无槽时 title 可延展到按钮左缘(现状); 按钮区几何不变。
- **Verify**: 单测 — 有槽时按钮仍最右(`button_rect` 断言), 槽占中间; 无槽时 title 区与现状一致。
- **Files**: `src/widget/title_bar.rs` | M

### T3: 事件/焦点/IME 路由（实测定, 见 Open Question）
- **Acceptance**: 命中中间槽→转发(落焦/打字/IME); 命中按钮→触发按钮; 命中标题区→拖拽/双击最大化
  (现有 `handle_drag_or_double_click` 不回归)。若 child-node 分发已覆盖, 则零转发; 否则补
  `wants_ime/ime_area/selected_text/hit_area/reset_focus` 于 TitleBar(同 BottomBar)。
- **Verify**: 单测 — embed TextInput 后: 点击槽落焦、输入事件达输入框、按钮不误触; 拖拽仍起。
- **Files**: `src/widget/title_bar.rs` | M

### T4: showcase demo + 回归
- **Acceptance**: `examples/showcase.rs` 标题栏卡片增加「嵌入输入槽」demo(以用代测); 全部既有
  TitleBar 单测(按钮/红绿灯/最大化图标/hover/透明)仍绿。
- **Verify**: `cargo run --example danqing-showcase` 人工 + `cargo test --lib --tests`。
- **Files**: `examples/showcase.rs`, `src/widget/title_bar.rs` | S

### Checkpoint: 模块验收 (T4 后)
- [ ] 三件套绿; 既有 TitleBar 测试全过(向后兼容); showcase 演示人工验收; 进 review 阶段。

## Risks and Mitigations
| 风险 | 影响 | 缓解 |
|---|---|---|
| 容器化破坏无 embed 产品渲染 | 高 | 默认 children 空 = 行为不变; 既有全量单测锁 |
| 槽事件与按钮/拖拽命中冲突 | 中 | T3 明确路由(槽→转发/按钮→触发/标题→拖拽); 单测 |
| child-node 不覆盖 IME/命中(需手动转发) | 中 | Open Question→T3 实测定; 若需转发照 BottomBar |

## Open Questions
- child-node 事件分发是否已覆盖嵌入子节点的 IME/命中/焦点? → T3 实测; 不足则 TitleBar 补转发。
- embed 槽在 Bar 为 Hidden(原始模式无搜索)时高度为 0 —— 标题栏是否因此变矮/仍固定高? → 跟随 Bar
  `layout` 高度(0 时标题栏仅标题+三键), plan 按「标题栏高 = max(自身高, 槽高)」实现, 实测定。
