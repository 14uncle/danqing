# Plan: 丹青 M2 — 焦点系统与文本输入

> 依据 `docs/specs/spec-m2.md`(已确认,2026-07-16)
> 目标:在 M1 保留模式组件树上实现焦点管理、单行 `TextInput`、剪贴板与 IME。

## 架构变化

M1 的事件流:

```
winit → window.rs → Event::Key 直送 App
                → 鼠标事件经 tree.event 命中分发
```

M2 事件流:

```
winit → window.rs → 内部 Event
                → Tab/Shift+Tab → FocusManager 切换焦点
                → 其他 Key/IME/剪贴板 → 路由到当前焦点组件
                → 鼠标事件 → tree.event 命中分发 → 同步更新焦点
                → 消息队列 → App::update
```

新增组件:

- `src/widget/focus.rs`:纯逻辑焦点管理器(焦点链、Tab 遍历、点击聚焦)。
- `src/widget/text_input.rs`:单行可编辑文本组件。

`Widget` trait 新增默认方法(不破坏现有实现,`showcase.rs` 的 `Positioned` 需要补默认无关实现):

- `fn focusable(&self) -> bool { false }`
- `fn children(&self) -> &[Node] { &[] }`
- `fn selected_text(&self) -> Option<String> { None }`
- `fn ime_area(&self) -> Option<Rect> { None }`
- `fn animate(&mut self, _ctx: &AnimationCtx) {}`

新增 `EventCtx` 或扩展事件签名以携带剪贴板;最终方案在 Task 1 确定。

## 依赖顺序

```
Task 1  事件/Focus 基础设施
 ├─ 扩展 Event 枚举(Ime/Copy/Cut/Paste/Tab 等)
 ├─ Widget trait 新增 focusable/children/selected_text/ime_area/animate
 ├─ FocusManager(纯逻辑)
 └─ window.rs 集成:路由键盘/IME/剪贴板到焦点组件

Task 2  TextInput 组件
 ├─ 编辑状态:字符串 + 光标/选区(char 索引)
 ├─ 键盘处理:输入/删除/方向/Home/End/Ctrl+A
 ├─ 渲染:背景、选区高亮、文本、光标(animate 闪烁)
 └─ 鼠标点击聚焦(可选:点击定位光标)

Task 3  剪贴板
 ├─ 引入 arboard(ask-first 已确认)
 ├─ Ctrl+C/X/V 路由
 └─ TextInput 响应 Copy/Cut/Paste 事件

Task 4  IME 集成
 ├─ winit Ime 事件转换
 ├─ 启用/禁用 IME(set_ime_allowed)
 ├─ 设置 IME 光标区域(set_ime_cursor_area)
 └─ TextInput preedit 显示 + commit 插入

Task 5  showcase 更新 + Button 焦点
 ├─ Button 支持 focusable 与空格/回车触发
 ├─ showcase 增加 TextInput 与焦点演示
 └─ 视觉焦点环

Task 6  打磨验收
 ├─ 新增单元/集成测试
 ├─ cargo fmt/clippy/test/release 全绿
 └─ 人工运行 showcase 验证焦点/输入/IME
```

关键路径:1 → 2 → 3 → 4 → 5 → 6。

## 关键设计决策

### 1. 焦点路由

`FocusManager` 通过 `Widget::children()` 对组件树做 DFS,收集 `focusable()==true` 的节点路径(`Vec<usize>`)。

- **Tab**:window.rs 捕获 `NamedKey::Tab`,根据 Shift 调用 `focus_next()`/`focus_prev()`。
- **键盘事件**:除 Tab 和全局快捷键(Ctrl+C/X/V)外,都通过 `event_at_path(focus_path, event)` 发送给焦点组件。
- **鼠标点击**:命中分发后,`FocusManager.hit_test(root, pos)` 找到点击位置最上层 focusable 节点并设为焦点。

### 2. 剪贴板抽象

为避免 `widget/` 依赖平台,剪贴板操作由 `window.rs` 适配层封装:

- `Event::Copy`/`Event::Cut`/`Event::Paste` 作为平台无关事件送达焦点组件。
- `Copy`/`Cut` 消费后,`window.rs` 调用 `tree.selected_text(path)` 取文本,再写入系统剪贴板。
- `Paste` 消费前,`window.rs` 先读剪贴板,再向焦点组件发送 `Event::Ime(ImeEvent::Commit { text })` 或 `Event::Input { text }`。

### 3. IME

winit 0.30 IME 事件:

- `WindowEvent::Ime::Enabled` / `Disabled`:控制是否接收合成。
- `WindowEvent::Ime::Preedit(value, cursor)`:显示下划线合成文本。
- `WindowEvent::Ime::Commit(value)`:将最终文本提交给输入框。

内部表示:

```rust
pub enum ImeEvent {
    Enabled,
    Disabled,
    Preedit { value: String, cursor: Option<usize> },
    Commit { value: String },
}
```

`TextInput` 维护 `preedit: Option<String>`;paint 时先绘制基础文本,再叠加 preedit 文本与下划线背景/下划线。

### 4. 光标闪烁

引入 `AnimationCtx { now: Instant, elapsed: Duration }`,由 `window.rs` 每帧构造并调用 `tree.animate(ctx)`。
`TextInput` 根据 elapsed 周期切换 `caret_visible`。

### 5. `Widget::children()` 存储改造

当前 `Flow` 用 `Vec<(Node, u32)>`,无法直接返回 `&[Node]`。改为:

```rust
pub struct Flow {
    children: Vec<Node>,
    weights: Vec<u32>,
    gap: f32,
    areas: Vec<Rect>,
}
```

`children()` 返回 `self.children.as_slice()`。`Column`/`Row` 其他逻辑不变。

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| `Widget` trait 新增方法导致 `showcase.rs` 的 `Positioned` 编译失败 | 低 | 默认方法实现,只需确认无显式 `impl Widget for Positioned` 冲突;如有则补默认方法 |
| `arboard` 在 windows-gnu 下链接失败 | 中 | 先 `cargo build` 验证;失败则换 `copypasta` 或改用命令行 `clip`/`powershell` 兜底 |
| winit IME Windows 行为差异 | 中 | Task 4 最先验证 `set_ime_allowed` + `set_ime_cursor_area` 能收到事件 |
| 光标/选区 char↔byte 索引在 CJK 多字节下出错 | 中 | 编辑逻辑全用 char 边界,字符串修改时按 char 迭代;写边界单测 |
| 焦点链构建与树变化不同步 | 低 | 每帧 `sync` 后重建焦点链,并校验当前路径仍有效 |

## 并行车道

- **事件/焦点车道**(Task 1) 与 **TextInput 编辑逻辑车道**(Task 2 纯逻辑部分) 可并行;契约 = `Event` 枚举与 `Widget` trait 新增方法。
- **剪贴板**(Task 3) 与 **IME**(Task 4) 都依赖 Task 1 的路由,但彼此独立。

## 验收检查点

1. Task 1 后:`cargo test`/`clippy` 绿,焦点管理器单测通过。
2. Task 2 后:`TextInput` 单测覆盖插入/删除/光标移动/选区。
3. Task 3 后:剪贴板集成测试通过(可 mock 剪贴板 trait)。
4. Task 4 后:showcase 能调起输入法并显示 preedit/commit。
5. Task 6 后:对照 `docs/specs/spec-m2.md` Success Criteria 全过,全部 Commands 绿。

## 开放问题(进入 Task 前确认)

1. `Widget::event` 是否改为接收 `EventCtx`(含剪贴板)还是保持当前签名用 `Event` + 后续 `selected_text()` 回调?推荐后者,对现有 API 侵入最小。
2. 是否允许在 M2 引入 `AnimationCtx` 默认方法,还是把光标闪烁推迟到 M3?
3. 是否需要鼠标拖拽选区?M2 规格只写"如能实现选区更佳",建议 M2 只做点击聚焦+键盘选区,拖拽留 M3。
