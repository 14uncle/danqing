# Plan: Dropdown 下拉选择器重写 + 框架弹层通道

> Spec: `docs/specs/SPEC-dropdown.md` (状态: 待批准)。来源: 用户 2026-09-12 指令「下拉选择器推倒重新写」+ 同日两项裁决 (架构 = 延迟弹层通道; v1 范围 = 键盘 + 点外关闭)。
> `plan-overlay.md` (框架下沉·簇C) 的 D/依赖图/风险三段结构与纪律沿袭。文档落 `tasks/plan-dropdown.md` + `tasks/todo-dropdown.md`, 与 `plan-overlay.md`/`todo-overlay.md` 同族 (`docs/tasks/` 是磁盘分析器按自身 CLAUDE.md 占用的位置, 与本计划无关, 一个字节不动)。

## 决策 (D1–D9)

- **D1 弹层走「延迟绘制 + 根级命中」通道**, 不采应用侧 Overlay 组装 (两版旧实现均被证伪: 自绘被后续兄弟覆盖; 推给应用则样板外溢), 也不采「逐级放宽容器命中门」(弹层可跨任意多级祖先矩形, 逐级放宽不可行)
- **D2 弹层优先只作用于鼠标按下**, `CursorMoved` 保持既有广播 (`flow.rs:272`)——若让弹层截走广播, 其余组件的 hover 残留将无法清除 (Overlay 的 hover 卫生同理)。滚轮不入 v1 (无滚动)
- **D3 `push_layer` 配对收口在 `paint_popups` 内唯一一处**, 组件不得自行 push_layer——规避 `render/mod.rs:291`「漏压一侧静默错乱」的坑
- **D4 三个纯函数落 `widget/mod.rs` 用 `pub(crate)`**, 不落 `window/`: 它们是纯树遍历, 守住「`widget/` 不得依赖 winit/wgpu」的依赖方向铁律; `pub(crate)` 不污染用户 API (公开 API 一律经 lib.rs 的原则)
- **D5 Dropdown 自管展开态** (非绑定式): 应用没有第二个开合入口, 开/合全由控件点击、键盘、点外收起驱动; 框架既有 `TextInput`/`Button` 同样自管临时 UI 态 (光标/hover/pressed), 非首例。**待确认: 若日后要多实例互斥或程序化开合, 须改绑定式** (见开放问题 1)
- **D6 `popup_area()` 保留并接线**, 不删——它是本设施的第一块砖, 架构裁决 (方案 A) 已定
- **D7 v1 范围锁死**: 无滚动 / 无空间不足翻转 / 无打字过滤 / 无分组多选禁用态 / 无开合动画 / 无嵌套弹层。元数为「基础: 键盘 + 点外关闭」
- **D8 零 commit 纪律**: 未获用户指示不 commit/push; CP2 处用户裁决
- **D9 在飞的 `scrollable.rs` 坐标系修复**是本次之外的独立 bug fix (子组件 `area` 须随 scroll offset 平移), **单独提交、不并入本次方向**; 同批在飞的 `overlay.rs` 定位绑定 (D6 之外) 见 T5

## 依赖图

```
T1 (widget/mod.rs: trait 三方法 + 三个纯函数 + 探针单测)
      │
      ├──────────────→ T2 (handler.rs 接线: 绘制趟 + 事件趟)
      │                       │
      │                       ▼
      │                 CP1 (三件套绿 + 无弹层路径零回归)
      │
      └──────────────→ T3 (dropdown.rs 全量重写, 仅需 T1 的 trait 方法)
                              │
                              ▼
                        T4 (showcase 换自足用法 + 删旧样板)   ← 需 T2 + T3
                              │
                              ▼
                        CP2 (实机人工验收 + commit/push 裁决)

T5 (回退 overlay.rs 定位绑定) ← 独立, 待用户裁决
T6 (scrollable.rs 修复单独提交) ← 独立, 待用户确认
```

T1 是唯一的关键路径瓶颈: 框架设施不落地, T2/T3 都无从开工。T2 与 T3 在 T1 之后**可并行** (T3 只需 trait 方法存在, 不需要驱动接线即可完成纯逻辑单测)。

## 任务清单

### Phase 1: 弹层通道 (框架设施)
- T1: `widget/mod.rs` — trait 三方法 + 三纯函数 + 单测
- T2: `window/handler.rs` — 绘制趟 + 事件趟接线

### Checkpoint 1: 通道语义锁死
- 三件套绿; 探针单测全绿; **无弹层路径零回归** (既有全量测试绿为判据)
- 人工: 既有交互 (Overlay/Tabs/焦点/Scrollable) 无可见变化

### Phase 2: Dropdown 重写
- T3: `widget/form/dropdown.rs` 全量重写
- T4: `examples/showcase.rs` 换自足用法 + 删旧样板

### Checkpoint 2: 实机验收
- 人工姿势全过; 三件套绿; 用户裁决 commit/push

### Phase 3: 收尾 (待裁决)
- T5: 回退 `overlay.rs` 定位绑定
- T6: `scrollable.rs` 修复单独提交

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| 动根渲染/事件主流程, 回归面大 | 高 | 三函数无弹层时是**完全空操作**; 单测锁「无弹层 = 与既有行为逐位等价」; CP1 以既有全量测试绿为判据 |
| `CursorMoved` 被弹层截走 → 其余组件 hover 残留 | 中 | D2 限定只对鼠标按下做优先; 单测锁「`CursorMoved` 仍走广播」 |
| `push_layer` 漏压一侧 → 静默错乱 | 中 | D3 收口单点; 单测断言 `rects`/`texts` 层数**相等** |
| **鼠标选中选项后焦点掉落** (见下方专述) | 中 | T3/T4 验收含「选中后键盘仍可继续导航」人工姿势; 失败则启用备选方案 (见下) |
| `hit_area` 不含弹层 → 点选项时焦点掉 | 高 | Dropdown 单测锁「展开态 `hit_area` 含弹层」 |
| 弹层超出窗口底部 (v1 不翻转) | 低 | 非目标明写; showcase 演示卡置于页面中部, 不误导 |
| 键盘导航与应用级 `on_key` 抢键 | 低 | 焦点路由先于 app 兜底 (`handler.rs:201 key_falls_back_to_app`); T4 同时删除 showcase 现有 app 级导航段 |
| Dropdown 未聚焦时键盘不达 | 低 | 预期行为 (与 TextInput 同); 组件 doc 写明需先聚焦 |

### 专述: 鼠标选中后焦点掉落 (需你在 CP2 前知晓)

`handler.rs:908` 分发完树事件后**紧跟着**执行:

```rust
self.focus.set_by_click(&self.tree, *position);   // 按 hit_area 判定焦点归属
```

时序陷阱: 用户点击弹层内的选项 → Dropdown 在事件处理中**立即**选中并收起 → `popup_area()` 变 `None`、`hit_area` 缩回控件矩形 → 随后 `set_by_click` 用缩回后的 `hit_area` 判定, 发现该点无命中 → **焦点被清空**。后果: 鼠标选完选项后, 键盘 ↑↓ 失效, 须重新点击控件。

纯键盘路径 (Tab 聚焦 → Enter 展开 → ↑↓ → Enter 选中) **不受影响**, 因为全程无点击。

两个处置选项, 待你裁决:
- **(a) 接受为 v1 已知限制**, 写进组件 doc。零额外改动。
- **(b) 扩展 `focus.rs`**, 让 `set_by_click` 的命中遍历与 `dispatch_popup_event` 同样读 `popup_area()`。改动小但多碰一个子系统 (焦点系统)。

我倾向 (a): 受影响的只是「先鼠标选、再键盘导航」这一条次要流; (b) 把焦点系统也拖进本次改动面, 与 CP1「零回归」的判据冲突。但若你认为下拉的键鼠混用是高频姿势, 就选 (b), 我会把它并进 T2。

## 开放问题 (2026-09-12 全部落定)

用户「按你推荐开工」一次性批准全部推荐项:

1. **展开态自管** (D5) —— 采纳。T3 按自管实现, 不增 `bind_expanded`。多实例互斥由「点外收起」自然达成 (点 A 之外任何位置先收起 A)。
2. **回退 `overlay.rs` 定位绑定** —— 采纳, T5 可动工。
3. **焦点掉落选 (a)** —— 采纳 (接受为 v1 已知限制, 写进组件 doc)。不扩展 `focus.rs`, 改动面收在本次范围内。
4. **`scrollable.rs` 单独提交** (T6) —— 采纳。
