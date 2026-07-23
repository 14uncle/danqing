# Plan: 丹青 M3 — 滚动容器与多行文本域

> 依据 `docs/specs/spec-m3.md`（已批准，2026-07-18）  
> 状态：**已实现**（2026-07-18）

## 架构变化

M2 事件流：

```
winit → window.rs → 内部 Event
                → Tab/Shift+Tab → FocusManager 切换焦点
                → 其他 Key/IME/剪贴板 → 路由到当前焦点组件
                → 鼠标事件 → tree.event 命中分发 → 同步更新焦点
                → 消息队列 → App::update
```

M3 新增：

- **渲染裁剪基础**：`RectBatch` / `TextBatch` 增加 clip stack，`RectInstance` / `GlyphInstance` 增加 `clip_min` / `clip_max`，WGSL 片元着色器按 clip discard。
- **`Scrollable`**：滚动容器，维护滚动偏移、视口裁剪、滚轮事件、滚动条视觉。
- **`TextArea`**：多行文本域，基于 `break_lines` 排版，支持光标/选区、键盘编辑、IME、剪贴板、鼠标点击与拖拽选区。
- **焦点命中裁剪**：`FocusManager::hit_focusable` 让后代 `hit_area` 与祖先 `hit_area` 求交，避免滚动到视口外的子组件被点击聚焦。

## 任务清单

- [x] **Task 0** M1/M2 文档清理
- [x] **Task 1** 渲染裁剪基础
- [x] **Task 2** `Scrollable` 容器
- [x] **Task 3** 多行文本排版 `src/text/line_layout.rs`
- [x] **Task 4** `TextArea` 组件
- [x] **Task 5** 拖拽选区（`TextInput` + `TextArea`）
- [x] **Task 6** showcase 更新
- [x] **Task 7** M3 文档、规格、验收

## 关键设计决策

1. **渲染裁剪**：Batch 级 clip stack。容器在 `paint` 前 `push_clip(viewport)`，绘制子组件后 `pop_clip`。默认无裁剪时使用极大安全矩形。
2. **`Scrollable` + `TextArea` 组合**：`TextArea` 报告自然内容尺寸，`Scrollable` 负责滚动偏移、裁剪与滚轮。
3. **按字符 soft-wrap**：M3 先做字符级换行，避免引入 Unicode 换行库；按词换行列为后续增强。
4. **拖拽选区**：在 `TextInput` / `TextArea` 中分别维护 `dragging` 状态，左键按下开始，拖动时更新 `cursor` 并保持 `anchor`，释放结束。

## 新增/修改文件

- `src/layout.rs`：`Rect::intersect` / `is_empty`
- `src/render/rect.rs`、`src/render/rect.wgsl`
- `src/render/text.rs`、`src/render/text.wgsl`
- `src/text/line_layout.rs`
- `src/text/mod.rs`
- `src/widget/scrollable.rs`
- `src/widget/text_area.rs`
- `src/widget/text_input.rs`（拖拽选区）
- `src/widget/focus.rs`（命中裁剪）
- `src/widget/mod.rs`
- `src/lib.rs`
- `examples/showcase.rs`
- `docs/specs/spec-m3.md`、`tasks/todo-m3.md`、`tasks/plan-m3.md`
- `README.md`

## 验证命令

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --lib --tests
cargo build --release
cargo run --example showcase
```

全部通过为 M3 关闭条件。

## 风险与缓解（已实现中确认）

| 风险 | 缓解 |
|---|---|
| shader 增加 clip discard 后边缘异常 | 默认 clip 为极大矩形;未裁剪矩形行为与 M2 一致 |
| Scrollable 子组件 Fill 权重无法真正滚动 | 文档说明 Scrollable 用于自然尺寸内容 |
| 多行光标上下移动视觉列不完美 | 先用列数 clamp 实现,符合 M3 范围 |
| IME 候选框在 Scrollable 内位置异常 | TextArea IME 区域返回光标矩形,验收时中文输入正常 |

## 验收结果

- `cargo test --lib --tests` 全绿。
- `cargo clippy -- -D warnings` 通过。
- `cargo fmt --check` 通过。
- `cargo build --release` 成功。
- showcase 新增“多行”区域,支持输入、换行、滚动与字数/行数回显。
