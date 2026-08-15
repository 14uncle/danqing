# Widget 开发范式

本文档是新增/修改组件时的开发指南，与 `architecture.md` 配合阅读。

---

## 1. 组件分类

| 类型 | 位置 | 特征 | 示例 |
|------|------|------|------|
| **Base (叶子)** | `widget/base/` | 无子组件或单子组件；直接产出绘制命令 | `Text`, `Button`, `Image` |
| **Layout (容器)** | `widget/layout/` | 管理子组件排列；委托 `Flow` 或自行计算 | `Row`, `Column`, `Box`, `Stack`, `Padding`, `Center` |
| **View (视图)** | `widget/view/` | 滚动、切换等视口行为 | `Scrollable`, `Switcher` |
| **Form (表单)** | `widget/form/` | 文本输入/编辑 | `TextInput`, `TextArea` |

新增组件时，先判断属于哪一类，放入对应子目录。

---

## 2. 每帧生命周期

```
sync(state)  →  animate(ctx)  →  layout(constraints)  →  paint(area)
```

| 方法 | 必须? | 职责 |
|------|-------|------|
| `sync` | 否 | 从 `&dyn Any` 状态读取绑定值，更新本地字段；递归 sync 子组件 |
| `animate` | 否 | 动画 tick；通常只递归子组件 |
| `layout` | **是** | 约束向下传、尺寸向上算；缓存几何到 struct 字段 |
| `paint` | **是** | 从缓存几何收集 `RectBatch` / `TextBatch`；不可变 |
| `paint_image` | 否 | 收集 `ImageBatch`（纹理组件用） |
| `event` | 否 | 处理交互事件；返回 `Consumed` / `Ignored` |

**关键约束**: `layout` 和 `paint` 是仅有的必须方法。其余均有合理默认值。

---

## 3. 最小组件模板

以 `Box`（背景色块）为范例，展示一个完整但精简的组件结构：

```rust
//! @author 十四叔
//! @date 2026/MM/dd

//! Box 组件: 带背景色与圆角的矩形块。

use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Color, Constraints, Rect, Size, Theme};

/// 颜色绑定闭包：从类型擦除的应用状态产出颜色。
type ColorBinding = Box<dyn Fn(&dyn std::any::Any) -> Color>;

pub struct Box {
    color: Color,
    color_binding: Option<ColorBinding>,
    radius: f32,
    child: Option<Node>,
    // layout 缓存
    area: Rect,
}

impl Box {
    /// 创建背景色块。
    pub fn new(color: Color) -> Self {
        Self {
            color,
            color_binding: None,
            radius: 0.0,
            child: None,
            area: Rect::default(),
        }
    }

    /// 使用主题默认值创建。
    pub fn themed(theme: &impl Theme) -> Self {
        Self::new(theme.surface()).radius(theme.radius_md())
    }

    /// 绑定颜色：每帧从应用状态读取。
    pub fn bind_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
        self.color_binding = Some(Box::new(move |state: &dyn std::any::Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Box 颜色绑定的状态类型不匹配");
            f(state)
        }));
        self
    }

    /// 设置圆角半径。
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// 设置子组件。
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

impl Widget for Box {
    fn sync(&mut self, state: &dyn std::any::Any) {
        // 1. 读取绑定
        if let Some(binding) = &self.color_binding {
            self.color = binding(state);
        }
        // 2. 递归 sync 子组件
        if let Some(child) = &mut self.child {
            child.sync(state);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        match &mut self.child {
            Some(child) => {
                // 有子组件: 未指定维度随内容收缩
                let child_size = child.layout(constraints, texts);
                constraints.constrain(child_size)
            }
            // 无子组件: 占满父约束上限
            None => constraints.constrain(Size::new(
                constraints.max_width,
                constraints.max_height,
            )),
        }
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        // 1. 绘制自身背景
        rects.push_rect(area, self.color, self.radius);
        // 2. 绘制子组件 (偏移到内容区)
        if let Some(child) = &self.child {
            child.paint(area, rects, texts);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        self.area = area; // 缓存绝对矩形
        // 先分发给子组件
        if let Some(child) = &mut self.child {
            if child.event(event, area, msgs) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        EventResult::Ignored
    }

    fn children(&self) -> &[Node] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn children_mut(&mut self) -> &mut [Node] {
        match &mut self.child {
            Some(child) => std::slice::from_mut(child),
            None => &mut [],
        }
    }
}
```

---

## 4. 核心模式

### 4.1 构造器模式

每个组件提供两个构造器：

```rust
/// 便捷构造器：使用 LightTheme 默认值。
pub fn new(/* 必要参数 */) -> Self {
    Self::themed(&LightTheme, /* ... */)
}

/// 主题构造器：接受任意 Theme 实现。
pub fn themed(theme: &impl Theme, /* 必要参数 */) -> Self {
    Self {
        color: theme.accent(),
        radius: theme.radius_md(),
        // ...
    }
}
```

**规则**: 不在 `new()` 里硬编码颜色/圆角/间距。一律从 `Theme` token 读取。

### 4.2 Builder 模式

可选配置通过返回 `Self` 的 builder 方法链式调用：

```rust
Button::new(Text::new("OK"))
    .on_click(|| Msg::Submit)
    .color(Color::RED)
    .id("submit-btn")
```

### 4.3 状态绑定 (Bind)

组件不直接持有应用状态。通过闭包从 `&dyn Any` 下载值：

```rust
// 定义
type ColorBinding = Box<dyn Fn(&dyn Any) -> Color>;

// 绑定
pub fn bind_color<S: 'static>(mut self, f: impl Fn(&S) -> Color + 'static) -> Self {
    self.color_binding = Some(Box::new(move |state: &dyn Any| {
        let state = state.downcast_ref::<S>().expect("状态类型不匹配");
        f(state)
    }));
    self
}

// 每帧 sync 时调用
fn sync(&mut self, state: &dyn Any) {
    if let Some(binding) = &self.color_binding {
        self.color = binding(state);
    }
}
```

**要点**:
- `S: 'static` 是具体状态类型，编译期确定
- 运行时通过 `downcast_ref` 从 `&dyn Any` 提取
- `expect` 消息应标明组件名和绑定用途，便于调试

### 4.4 Layout 缓存

`layout()` 计算的几何结果缓存到 struct 字段，供 `paint()` 和 `event()` 读取：

```rust
// struct 字段
child_size: Size,  // 子组件尺寸
area: Rect,        // 自身绝对矩形

// layout 中写入
fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
    self.child_size = self.child.layout(constraints, texts);
    self.area = Rect::new(Point::ZERO, size);
    size
}

// paint / event 中读取
fn paint(&self, area: Rect, ...) {
    rects.push_rect(area, self.color, self.radius);
    let inner = Rect::new(
        Point::new(area.origin.x + padding, area.origin.y + padding),
        self.child_size,
    );
    self.child.paint(inner, rects, texts);
}
```

**注意**: `area` 参数是父组件传入的绝对坐标；`self.area` 缓存的是自身在 `layout` 阶段计算的相对矩形。事件处理时需要缓存绝对 `area` 到 `self.area` 供命中测试。

### 4.5 事件分发

容器组件的事件分发模式：

```rust
fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
    self.area = area;
    // 移动类事件全发 (所有子组件需跟踪 hover)
    // 其他事件只发给命中的子组件
    if let Some(child) = &mut self.child {
        let forward = match event {
            Event::CursorMoved(_) | Event::CursorLeft => true,
            e => e.position().is_some_and(|p| area.contains(p)),
        };
        if forward && child.event(event, area, msgs) == EventResult::Consumed {
            return EventResult::Consumed;
        }
    }
    EventResult::Ignored
}
```

**事件类型分发规则**:
| 事件 | 分发方式 |
|------|----------|
| `CursorMoved`, `CursorLeft` | **广播** — 所有子组件都收到 (hover 跟踪) |
| 其他 (`MouseInput`, `Key`, `FocusIn/Out`) | **命中** — 只发给 position 命中的子组件 |

### 4.6 Focus 集成

叶子组件需要 focus 时：

```rust
fn focusable(&self) -> bool { true }
fn focus_id(&self) -> Option<&'static str> { self.id }
fn reset_focus(&mut self) { self.focused = false; }
fn hit_area(&self) -> Option<Rect> { Some(self.area) }
fn ime_area(&self) -> Option<Rect> { Some(self.area) }
```

**规则**:
- `focusable()` — 叶子组件返回 `true`；容器返回 `false`（默认值）
- `focus_id()` — 按名聚焦标识，用于面板关闭后焦点恢复
- `hit_area()` — 点击聚焦的命中区域；返回 `Some(self.area)` 表示整个组件可点击聚焦
- `reset_focus()` — 清除焦点视觉状态，被 `Switcher` 等容器在面板隐藏时调用

---

## 5. Theme Token 使用规范

### 必须

- 所有颜色、圆角、间距、字体大小一律通过 `Theme` trait 方法获取
- `new()` 构造器使用 `LightTheme` 默认值
- `themed()` 构造器接受 `&impl Theme` 参数

### 禁止

- 在 `paint()` 或 `event()` 中使用魔法颜色字面量 (`Color::rgba(0.2, 0.2, 0.2, 1.0)`)
- 在 `layout()` 中使用硬编码间距值

### 例外

- 几何常量（焦点环内缩 `3.0`、虚线参数 `4.0, 2.0, 1.0`）允许在 `paint()` 中直接使用
- 动态调制系数（`pressed: 0.7`, `hovered: 1.25`）允许在 `effective_color()` 中使用

---

## 6. 子组件模式

### 单子组件 (Button, Box)

```rust
child: Option<Node>,  // 或 Node (无 Option 时用 Box::new 占位)

fn children(&self) -> &[Node] {
    match &self.child {
        Some(child) => std::slice::from_ref(child),
        None => &[],
    }
}
```

### 多子组件 (Row, Column)

委托 `Flow` 结构体处理：

```rust
// Row/Column 内部持有 Flow
flow: Flow,

fn children(&self) -> &[Node] { self.flow.children() }
fn children_mut(&mut self) -> &mut [Node] { self.flow.children_mut() }
```

### 无子组件 (Text, TextInput)

```rust
fn children(&self) -> &[Node] { &[] }
fn children_mut(&mut self) -> &mut [Node] { &mut [] }
```

---

## 7. 测试规范

### 单元测试

写在组件文件末尾的 `#[cfg(test)] mod tests {}` 中：

```rust
#[cfg(test)]
impl MyWidget {
    /// 当前值 (测试用)。
    pub(crate) fn value(&self) -> f32 { self.value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_uses_theme_defaults() {
        let w = MyWidget::new();
        assert_eq!(w.value(), LightTheme.some_token());
    }
}
```

**测试覆盖点**:
- 构造器使用正确的 theme token 默认值
- Builder 方法正确覆盖默认值
- Bind 闭包在 `sync` 后更新本地字段
- Layout 在不同约束下的尺寸计算
- 有/无子组件时的边界行为

### 集成测试

放在 `tests/` 目录，验证组件树构建 + 布局 + 模拟事件分发的端到端行为，无需 GPU。

---

## 8. 新增组件 Checklist

1. [ ] 放入正确的子目录 (`base/`, `layout/`, `view/`, `form/`)
2. [ ] 文件头 `//! @author 十四叔` + `//! @date yyyy/MM/dd`
3. [ ] 中文模块文档注释
4. [ ] 公开 API 经 `src/lib.rs` re-export
5. [ ] 两个构造器: `new()` (LightTheme) + `themed()` (&impl Theme)
6. [ ] Builder 方法链式调用
7. [ ] `layout()` 和 `paint()` 实现
8. [ ] 如需交互: `event()` + `focusable()` + `hit_area()`
9. [ ] 如需状态驱动: `bind_*` 方法 + `sync()` 闭包调用
10. [ ] `children()` / `children_mut()` 正确返回子组件切片
11. [ ] 单元测试覆盖构造器、builder、bind、layout
12. [ ] `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` 全绿
13. [ ] 在 `examples/showcase.rs` 中添加用法示例