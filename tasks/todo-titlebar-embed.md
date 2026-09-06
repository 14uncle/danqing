# TODO: TitleBar 可选内嵌栏槽

> plan: `tasks/plan-titlebar-embed.md` | spec: `docs/specs/SPEC-titlebar-embed.md`
> 逐条勾选; 每任务后跑三件套 (fmt + clippy -D warnings + test)。向后兼容: 未 embed 零变化。

- [x] **T1: TitleBar 容器化 + embed 槽** ✅ 2026-09-06
  - Acceptance: `embed(impl Widget)` 存 `Option<Node>`; `children()/children_mut()` 默认空/embed 后 1;
    未 embed 行为不变(既有单测全绿)
  - Verify: `cargo test widget::title_bar::tests`
  - Files: `src/widget/title_bar.rs`
  - 实测: 照 box_.rs 范式(embed 字段 + `embed()` builder + children/children_mut);
    title_bar 26 测试绿(24 既有 + 2 新 embed), 全量 388 lib + 集成, clippy 0

- [x] **T2: 布局 — 槽占标题与三键之间中间余宽** ✅ 2026-09-06
  - Acceptance: logo/title 宽与三键区起点; 槽取其间余宽竖直撑满, `layout` 子节点; 无槽时 title 可延展;
    按钮区几何不变(仍最右)
  - Verify: `cargo test`(槽占中间 + 按钮最右断言)
  - Files: `src/widget/title_bar.rs`
  - 实测: embed_slot_span/slot_right 算槽界(标题右+留白..按钮左/窗右缘); child 按 loose 自然高度竖直居中;
    sync/animate 转发 child; embed_area 字段; 27 测试绿 + 集成, clippy 0

- [x] **T3: 事件/焦点/IME 路由(实测定)** ✅ 2026-09-06
  - Acceptance: 命中槽→转发落焦/打字/IME; 命中按钮→触发; 命中标题区→拖拽/双击最大化; 若 child-node
    已覆盖则零转发, 否则补 wants_ime/ime_area/selected_text/hit_area/reset_focus
  - Verify: `cargo test`(embed TextInput 点击落焦/输入/按钮不误触)
  - Files: `src/widget/title_bar.rs`
  - 实测: 键盘/IME/剪贴板走焦点路径 event_at_path(children_mut 自动钻到 child) → 零转发;
    鼠标才需 TitleBar.event 转发(槽内→child, 按钮/标题区不转). hit_area 已由 paint 缓存 absolute 坐标
    (focus.rs「必须 paint 缓存绝对矩形」), 点击落焦自动. 2 新单测 + 29 title_bar 绿

- [x] **T4: showcase demo + 回归** ✅ 2026-09-06
  - Acceptance: showcase 标题栏卡片加「嵌入输入槽」demo; 全部既有 TitleBar 单测仍绿
  - Verify: `cargo run --example danqing-showcase` + `cargo test --lib --tests`
  - Files: `examples/showcase.rs`, `src/widget/title_bar.rs`
  - 实测: page_base 加「TitleBar 内嵌输入槽」卡(独立 TitleBar + LogoKind::Log + embed TextInput,
    不动真实窗口 chrome); showcase 编译过 + 全量测试绿 + clippy 0

- [ ] **Checkpoint: 模块验收**
  - [ ] 三件套绿; 既有 TitleBar 测试全过; showcase 人工通过; 进 review(`/agent-skills:code-review-and-quality`)
