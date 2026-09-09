# Todo: Overlay 模态浮层组件 (框架下沉·簇C)

> Plan: `tasks/plan-overlay.md` (D1–D8 决策与依赖图)。Spec: `docs/specs/SPEC-overlay.md` (已批准 2026-09-09)。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。
> commit/push 待用户指示 (D7)。

## Phase 1: 框架组件

- [x] **T1: `widget/view/overlay.rs` 建档 + Overlay 骨架与 open 门控 ✅ 2026-09-09** — 文件头/模块头 doc 齐; `new`/`themed` + `bind_open<S>`; scrim 直接画矩形不做子组件 (事件由 Overlay 自体模态消费); 关态四件 + 开态透传 + sync/animate 门控; view/mod.rs 登记 (lib.rs 既有 `pub mod widget` 链条直达, 无新增 re-export)
  - Acceptance: 关态四件 + 开态透传 + 关态 sync 不触达内容 ✅ (4 绿)
  - Verify: `cargo test widget::view::overlay` 全绿; 三件套绿 ✅

- [x] **T2: 开态交互与渲染层 ✅ 2026-09-09** — 卡片 area 由 `card_rect()` 纯函数计算 (layout 缓存 content_size + area 参数, 无额外 Cell); 遮罩点击 Consumed + opt-in `on_scrim_click`; 卡片内透传; paint 开态 push_layer ×2 (关态不开)
  - Acceptance: 五条单测全锁 (带工厂发消息/无工厂静默/移动滚轮也吞/键盘透传/层数 +1) ✅ (9 绿累计)
  - Verify: `cargo test widget::view::overlay` 9 全绿; 三件套绿 (clippy --all-targets 净) ✅

### ★ Checkpoint 1: 组件纯逻辑完成
- [x] 三件套绿; 六语义各有单测锁定 ✅ 2026-09-09
- [x] review 修订 ✅ 2026-09-09 (REQUEST CHANGES → 全处置): C1 卡内 Ignored 穿透底层 (log Scrim 全窗兜底语义下沉时丢失) → 定位事件恒 Consumed; C2 hit_area 覆写抢焦 (下沉新增回归, log 源头没有) → 删覆写 + focusable 改恒 false (Tabs 范式); R1 关层边沿 reset_focus (MultiPanel 判例); R2 paint_image 透传; O1 hover 卫生 / O2 event.position() / O3 验收姿势提示 / N1-N3。12 测试绿 (新增 C1 回归 + 关层边沿 + paint_image 三条)
- 衍生立案 (不在本簇): focus.rs visit() 末写者胜语义对重叠兄弟存在潜在反转, 值得钉板测试单独立项 (评审 C2 附带建议)

## Phase 2: 以用代测

- [x] **T3: showcase 模态浮层演示卡 ✅ 2026-09-09** — 「模态浮层 (Overlay)」卡: 打开按钮 (`.id("overlay-demo-open")` 焦点锚); 浮层本体盖根 Stack 顶 (build_tree 根由 Column 改包 Stack); 玻璃卡 (surface+radius_lg+CloseButton); `bind_open` + `on_scrim_click`; Showcase 实现 focus_request/focus_restored 演示焦点回归协议
  - Acceptance: 三姿势实机可触发; 三件套绿 (clippy --all-targets 净) ✅。踩坑记录: Row::fill 需权重参数; showcase 此前未 import Stack
  - Verify: `cargo run --example danqing-showcase` 实机人工过目 (待人工)
  - Files: `examples/showcase.rs` | S

### ★ Checkpoint 2: 框架侧完成
- [ ] 实机人工过目 (三姿势 + 焦点回归; 注: focus_request 仅在焦点为空时应用 —— 若焦点环未回归, 先查关闭点击点是否压到底层可聚焦组件, 非演示 bug)
- [ ] 用户裁决 commit/push (danqing 先行)

## Phase 3: 产品迁移 (分仓, danqing push 后)

- [ ] **T4: pomodoro 三面板迁移** — settings/stats/report 面板函数改为返回内容卡; 树装配处包 `Overlay::themed(..).bind_open(..)`; 互斥与焦点回归逻辑 (main.rs:461-506) 逐行保留; 手写 `Stack{scrim,Center}` 三处删除
  - Acceptance: 既有测试全绿 (面板互斥/焦点回归零改动 = 行为保持判据); 三处手写机构零残留
  - 人工四姿势: 开合 / 互斥 / 焦点回归 / 点面板外不穿透底层 (D2 观感变化点)
  - Files: `src/main.rs` | M | 前提: danqing push

- [ ] **T5: log 迁移** — `SettingsOverlay` 整删 + 本地 `Scrim` 组件删除 → `danqing::Overlay`; Esc 前置关卡 (main.rs:999-1002) 保留; settings_open 绑定直通
  - Acceptance: 既有测试全绿; SettingsOverlay/Scrim 零残留
  - 人工两姿势: 设置卡盖表格文本清晰 (push_layer 效应) / Esc 两阶段 (先清焦后关层)
  - Files: `src/settings.rs`, `src/main.rs` | M | **前提闸: log 在途批次 (async-open+text-selection) 已提交**

- [ ] **T6: clipboard 迁移** — 设置面板 + 清空确认两处换 Overlay (双层叠加 = z 序验证场)
  - Acceptance: 既有测试全绿; 手写 scrim 两处零残留
  - 人工姿势: 确认层盖设置层 z 序正确 / 关任一层
  - Files: `src/ui/settings.rs` | S-M | **前提: 用户裁决动不动** (clipboard 日用在用)

### ★ Checkpoint 3: 全量验收
- [ ] 各仓三件套绿; 分仓分别提交, message 注明关联 danqing 提交
