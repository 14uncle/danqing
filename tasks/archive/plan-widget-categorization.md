# Implementation Plan: widget/ 分类目录化 + Switcher 组件 + showcase 分类导航

> 依据已确认的目录划分意图（base/layout/form/view)+ "树不重建、选中分类才显示" 的 showcase 改造需求细化而来（interview-me 产出，2026-07-21 确认）。
> 本文档将工作拆分为 **9 个可验证任务**，按依赖顺序组织，全程保持 `danqing::widget::{...}` 平铺公开 API 零变化。

## Overview

1. 将 `src/widget/` 下 11 个组件文件按类型迁入 `base/`、`layout/`、`form/`、`view/` 四个子目录，`focus.rs`、`title_bar.rs` 留在根部；`widget/mod.rs` 继续平铺 re-export,tests/ 与 examples/ 零改动。
2. 新增 `src/widget/view/switcher.rs`:`Switcher` 容器保留全部子组件实例（sync/animate 全员传播），但 layout/paint/event/children 只作用 active 子组件，以"保留树 + 选择可见性"支撑分类导航。
3. showcase 改为 `Column[TitleBar, Row[侧边栏, Switcher[4 个分类面板]]]`，侧边栏选中态用 showcase 本地视觉方案，不改框架。

## Architecture Decisions

### 迁移层（已验证）

- **子目录 mod.rs 形态**：每个子目录一个 mod.rs，文件头 `//! @author 十四叔` / `//! @date 2026/07/21` + 中文模块文档 + 字母序 mod 声明 + 平铺 re-export。被移动的旧文件**保留原文件头日期**（纯移动，仅必要时改 import 行）。
- **widget/mod.rs 迁移后形态**（要点）:
  ```rust
  mod base;
  mod focus;
  mod form;
  mod layout;
  mod title_bar;
  mod view;

  pub use base::{Button, Text};
  pub use focus::FocusManager;
  pub use form::{TextArea, TextInput};
  pub use layout::{Box, Center, Column, Padding, Row};
  pub use title_bar::TitleBar;
  pub use view::{ScrollAxis, Scrollable, Switcher};
  ```
  子模块声明为**私有 mod + pub use 平铺**，与现状一致；`flow`、`text_editor` 保持各自子目录内的**私有 mod 声明**（不 re-export)，等效于当前可见性。
- **4 处深层 import 改为绝对路径**（项目生产代码惯例）:
  - `column.rs` / `row.rs`:`use crate::widget::flow::{Axis, Flow};` → `use crate::widget::layout::flow::{Axis, Flow};`
  - `text_area.rs`:`use crate::widget::text_editor::{TextEditor, char_to_byte};` → `use crate::widget::form::text_editor::{TextEditor, char_to_byte};`
  - `text_input.rs`:`use crate::widget::text_editor::TextEditor;` → `use crate::widget::form::text_editor::TextEditor;`
  可见性论证：`flow` 是 `layout` 的私有 mod，但 `column`/`row` 是 `layout` 的后代模块，绝对路径逐段可见，合法；`text_editor` 同理。
- **遗漏引用点排查结果**:
  - `CLAUDE.md` 测试命令 `cargo test widget::flow::tests::column_stacks_fit_children -- --exact` 迁移后失效，须改为 `widget::layout::flow::tests::...`。
  - `CLAUDE.md` 与 `README.md` 的目录结构描述需同步一句。
  - tests/ 7 个集成测试、examples/ 全部只走平铺路径，零改动（已逐条 grep 确认）。
  - 单元测试都在各文件内 `use super::*`，迁移后随文件移动继续有效。

### Switcher 设计验证结论

1. **children() 只返回 active 对焦点/事件/IME 的影响 —— 安全**:`FocusManager::rebuild` 与 `hit_focusable` 都只经 `children()` 遍历 → 隐藏面板内组件不进焦点链、不可点击聚焦。被隐藏面板中正焦点的 `TextInput`：切换后下一帧 rebuild 发现路径失效 → 焦点清除；旧路径发 `FocusOut` 时 `event_at_path` 索引越界返回 `Ignored`，该组件 `focused` 标志残留 true，但不被 paint 无视觉残留，切回不自动恢复焦点。**可接受**，用集成测试锁定并写入文档注释。
2. **隐藏子组件的陈旧几何无读者**:`ime_area_at_path`/`selected_text_at_path`/`wants_ime_at_path` 只以 rebuild 校验过的焦点路径为参数；鼠标命中经 Switcher.event 只转发 active。
3. **layout 语义**:active 子组件拿全约束,`Switcher 尺寸 = active 尺寸`；空 children 返回 `constraints.constrain(Size::ZERO)`。
4. **配套公开类型：不需要**;active 就是 `usize`，越界 **clamp** 不 panic（与 event_at_path 容错风格一致）。
5. **切片返回技巧**:`children()` 返回 `&self.children[self.active..self.active + 1]`，空时 `&[]`。

### Switcher API 形态

```rust
pub struct Switcher { children: Vec<Node>, active: usize, binding: Option<Box<dyn Fn(&dyn Any) -> usize>>, active_size: Size }

Switcher::new().child(node).child(node).active(1)
Switcher::bind<S: 'static>(f: impl Fn(&S) -> usize + 'static)  // 复刻 Text::bind
```

- `sync`：先对**所有**子组件递归 sync（状态保鲜），再求值 binding 并 clamp。
- `animate`：传播给**所有**子组件。
- `layout`/`paint`/`event`：只作用 active。
- `focusable` false;`children()/children_mut()` 只返回 active（或空）。

## Dependency Graph

```
Phase 1 目录迁移(行为不变)
Task 1 base/   ─┐
Task 2 view/   ─┤ (彼此独立, 但都改 widget/mod.rs, 串行)
Task 3 layout/ ─┤
Task 4 form/   ─┘
   └─ Checkpoint A: fmt + clippy -D warnings + 全测试绿 + showcase 冒烟

Phase 2 Switcher
Task 5 switcher.rs + 单元测试      (依赖 Task 4 的 view/ 目录)
Task 6 tests/switcher.rs 集成测试  (依赖 Task 5)
   └─ Checkpoint B: 全测试绿 + clippy

Phase 3 showcase 改造
Task 7 Showcase 状态/Msg + 四页归位  (依赖 Task 5)
Task 8 侧边栏 + Switcher 接线        (依赖 Task 7)
Task 9 文档同步 + 最终验收           (依赖 Task 8)
```

关键路径：4 → 5 → 6 → 7 → 8 → 9。迁移顺序按 import 依赖从干净到复杂：base → view → layout → form。

## Task List

### Phase 1: 目录迁移 — 纯移动，行为不变

每个任务统一手法：`git mv` 保留历史 → 新建子目录 mod.rs → 调整 `widget/mod.rs` → 修正深层 import → 验证。

- [ ] **Task 1: 迁移 base/(button.rs、text.rs)**
  - **Description:** `git mv src/widget/{button,text}.rs src/widget/base/`；新建 `src/widget/base/mod.rs`（文件头 + `//! 基础组件: 按钮与文本。` + `mod button; mod text;` + `pub use button::Button; pub use text::Text;`);`widget/mod.rs` 删除两条旧 mod 声明与 re-export，改挂 `mod base;` + `pub use base::{Button, Text};`。
  - **Acceptance criteria:**
    - [ ] `danqing::widget::{Button, Text}` 平铺路径不变。
    - [ ] 无任何 import 需要改动（两文件只走平铺路径）。
  - **Verification:** `cargo check` + `cargo test` 全绿。
  - **Dependencies:** None
  - **Files:** `src/widget/base/mod.rs`（新）, `src/widget/base/button.rs`, `src/widget/base/text.rs`, `src/widget/mod.rs`
  - **Scope:** S

- [ ] **Task 2: 迁移 view/(scrollable.rs)**
  - **Description:** 同上手法迁移 `scrollable.rs`;`view/mod.rs` re-export `{ScrollAxis, Scrollable}`。
  - **Acceptance criteria:** `danqing::widget::{ScrollAxis, Scrollable}` 不变。
  - **Verification:** `cargo check` + `cargo test` 全绿。
  - **Dependencies:** Task 1（共享 mod.rs，串行）
  - **Files:** `src/widget/view/mod.rs`（新）, `src/widget/view/scrollable.rs`, `src/widget/mod.rs`
  - **Scope:** S

- [ ] **Task 3: 迁移 layout/(box_、center、column、flow、padding、row)**
  - **Description:** 六文件迁入；`layout/mod.rs` 中 `mod flow;` 保持**私有**且不 re-export;`column.rs`、`row.rs` 的 import 改为 `crate::widget::layout::flow::{Axis, Flow}`；同步更新 CLAUDE.md 测试路径为 `widget::layout::flow::tests::...`。
  - **Acceptance criteria:**
    - [ ] 平铺 re-export `{Box, Center, Column, Padding, Row}` 不变；`Flow`/`Axis` 不进入公开 API。
    - [ ] CLAUDE.md 测试命令更新后实际可运行通过。
  - **Verification:** `cargo test widget::layout` 全绿；全量 `cargo test` 绿。
  - **Dependencies:** Task 2
  - **Files:** `src/widget/layout/mod.rs`（新）, 6 个迁移文件， `src/widget/mod.rs`, `CLAUDE.md`
  - **Scope:** M

- [ ] **Task 4: 迁移 form/(text_input、text_area、text_editor)**
  - **Description:** 三文件迁入；`form/mod.rs` 中 `mod text_editor;` 保持私有；`text_area.rs`、`text_input.rs` import 改为 `crate::widget::form::text_editor::...`;re-export `{TextArea, TextInput}`(`TextEditor` 不公开，维持现状）。
  - **Acceptance criteria:** 平铺路径不变；IME/选区相关单元测试全绿。
  - **Verification:** `cargo test widget::form` + 全量 `cargo test` 绿。
  - **Dependencies:** Task 3
  - **Files:** `src/widget/form/mod.rs`（新）, 3 个迁移文件， `src/widget/mod.rs`
  - **Scope:** M

- [ ] **Checkpoint A: 迁移收尾验收**
  - `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 全绿；`cargo run --example showcase` 人工冒烟；`git log --follow` 抽查一个迁移文件确认历史保留。

### Phase 2: Switcher 组件

- [ ] **Task 5: 实现 src/widget/view/switcher.rs + 模块内单元测试**
  - **Description:** 按"Switcher API 形态"实现；文件头 `@date 2026/07/21`；公开类型写中文文档注释，明确记录"隐藏面板内焦点在切换后被清除、切回不自动恢复"语义；`view/mod.rs` 挂 `mod switcher;` + `pub use switcher::Switcher;`;`widget/mod.rs` 的 view re-export 追加 `Switcher`。
  - **单元测试清单（模块内）:**
    - [ ] `active` 越界 clamp 到 `len-1`；空 children 时 layout 返回 ZERO 约束尺寸、children 为空。
    - [ ] layout 尺寸 == active 子组件尺寸；切换 active 后尺寸随之变化。
    - [ ] paint 只收集 active 子组件（RectBatch 条数对比）。
    - [ ] event 只到达 active（两个 Button 各挂计数消息，验证只有一个产出）。
    - [ ] sync 传播所有子组件（两个 `Text::bind` 内容都刷新）;binding 闭包驱动 active 切换并 clamp。
    - [ ] `children()/children_mut()` 非空时长度恒为 1。
  - **Acceptance criteria:** 上述测试全绿；`cargo clippy -- -D warnings` 零警告。
  - **Verification:** `cargo test widget::view::switcher`。
  - **Dependencies:** Task 4
  - **Files:** `src/widget/view/switcher.rs`（新）, `src/widget/view/mod.rs`, `src/widget/mod.rs`
  - **Scope:** M

- [ ] **Task 6: tests/switcher.rs 集成测试**
  - **Description:** 新建集成测试，只使用 `danqing::widget::{...}` 平铺路径。
  - **集成测试清单：**
    - [ ] FocusManager rebuild 后焦点链只含 active 面板内可聚焦组件；Tab 遍历不进入隐藏面板。
    - [ ] 焦点在面板 A 的 TextInput 时切换 active → rebuild 后焦点清除（锁定文档化行为）。
    - [ ] `event_at_path` 经 Switcher 路径（索引恒 0）到达 active 子组件并产出消息。
    - [ ] 点击隐藏面板区域不聚焦其内组件（需先 layout+paint)。
  - **Acceptance criteria:** 全部通过；文件头含 `@author 十四叔` / `@date 2026/07/21`。
  - **Verification:** `cargo test --test switcher` + 全量 `cargo test`。
  - **Dependencies:** Task 5
  - **Files:** `tests/switcher.rs`（新）
  - **Scope:** S

- [ ] **Checkpoint B: Switcher 验收**
  - fmt + clippy -D warnings + 全测试绿；`cargo doc` 无 broken intra-doc link。

### Phase 3: showcase 分类导航改造

- [ ] **Task 7: Showcase 状态扩展与四页归位**
  - **Description:** `Showcase` 增加 `selected: usize`;`Msg` 增加 `Select(usize)`;`update` 处理之。现有四个构造区归位为四个页面函数（均保留 `card()` 结构与 Scrollable 包装，树只建一次）:
    - `page_base`:**基础 base** ← 计数器 Button + Text 绑定回显（原"交互组件"卡的 Button 部分）。
    - `page_layout`:**布局 layout** ← 品牌色与圆角卡原样（UiBox 网格 + 圆角行 = Column/Row/gap/Box 演示）。
    - `page_form`:**表单 form** ← TextInput + TextArea 两卡。
    - `page_view`:**视图 view** ← 键盘响应卡（Positioned 自定义组件；页面说明 Switcher/Scrollable 亦属 view)。
  - **Acceptance criteria:** 四个页面函数各自返回 `impl Widget + 'static`；状态经绑定闭包每帧同步，无树重建。
  - **Verification:** `cargo check --example showcase`。
  - **Dependencies:** Task 5
  - **Files:** `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 8: 侧边栏 + Switcher 接线**
  - **Description:** `build_tree` 改为 `Column[TitleBar, Row[固定宽侧边栏, fill(Switcher)]]`；四个页面各包 `Scrollable::themed`。侧边栏四项：`基础 base` / `布局 layout` / `表单 form` / `视图 view`；**选中态高亮用 showcase 本地方案**：选中项文本前缀 `"▶ "`、未选中全角空格对齐，不改框架。Switcher 接 `.bind(|s: &Showcase| s.selected)`。
  - **Acceptance criteria:**
    - [ ] 点击侧边栏切换右侧面板；所有组件始终实例化。
    - [ ] 四页面内容与原四卡片一一对应，无功能回退（计数器、输入回显、字数统计、键盘方块全部可用）。
  - **Verification:** `cargo run --example showcase` 人工验收清单：切换四类 → 各页交互回归 → form 页 TextInput 聚焦中切走再切回，焦点清除行为符合预期 → Tab 只在可见面板内循环。
  - **Dependencies:** Task 7
  - **Files:** `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 9: 文档同步与最终验收**
  - **Description:** 更新 CLAUDE.md 与 README.md 的目录结构描述（四子目录一句）;showcase 文件头文档注释更新为"分类导航演示页"。
  - **Acceptance criteria:**
    - [ ] `cargo fmt --check` 通过。
    - [ ] `cargo clippy --all-targets -- -D warnings` 零警告。
    - [ ] `cargo test` 全量绿（含新集成测试）。
    - [ ] showcase 人工验收通过。
  - **Verification:** 上述四条。
  - **Dependencies:** Task 8
  - **Files:** `CLAUDE.md`, `README.md`, `examples/showcase.rs`
  - **Scope:** S

## Checkpoints

| 检查点 | 条件 | 验证命令 |
|---|---|---|
| A: 迁移收尾 | Phase 1 完成，行为不变 | fmt --check + clippy --all-targets -D warnings + cargo test + showcase 冒烟 + git log --follow 抽查 |
| B: Switcher 验收 | Phase 2 完成 | fmt + clippy -D warnings + cargo test + cargo doc 无 broken link |

## Risks and Mitigations

| 风险 | 等级 | 缓解 |
|---|---|---|
| 迁移与 mod 声明不同步导致编译断裂 | 低 | 每任务单步 `cargo check` + 全量 test;Phase 1 串行 |
| `flow`/`text_editor` 私有 mod 可见性判断失误 | 低 | 已按 Rust 逐段可见性规则论证；Task 3/4 各有编译验证兜底 |
| Switcher children() 只返回 active 引发焦点路径语义意外 | 中 | 已通读全部 children() 消费点；Task 6 用集成测试锁定"切走清焦、切回不恢复" |
| 隐藏子组件陈旧几何被其它路径读取 | 低 | 已确认 at_path 系列只走 rebuild 校验过的焦点路径 |
| 侧边栏选中态视觉过弱（仅文本前缀） | 低 | 用户明确"不改框架"；后续 Button 增选中态可平滑替换 |
| git mv 历史丢失（Windows) | 低 | 统一 `git mv`;Checkpoint A 用 `git log --follow` 抽查 |
| CLAUDE.md 测试路径失效被遗忘 | 低 | 显式列入 Task 3 验收标准 |

## Parallelization Notes

Phase 1 四个迁移任务内容独立但都改 `widget/mod.rs`，串行执行避免冲突。Phase 2/3 严格依赖 Phase 1 的 view/ 目录与 Switcher 组件，全程串行。无并行车道。
