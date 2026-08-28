# Spec: Tabs 组件 (danqing 引擎)

## Objective

在 danqing 框架中新增 `Tabs` 容器组件，提供带可视化 tab 栏的多面板切换能力。产品侧只需一个 `Tabs::new().tab("标签").child(content).bind(...)` 即可获得完整的 tab 交互。

## 设计决策

- **自包含**：tab 栏渲染 + 面板切换在一个组件内完成，不需要产品侧手动组合 Row + MultiPanel
- **面板切换逻辑复用 MultiPanel 的 sync/显示机制**：sync 传播给全部子面板（状态保鲜），只有 active 面板参与 layout/paint/event
- **Tab 栏为自绘叶子**：不用 Button 子组件拼接，直接在 paint 中绘制文字 + 指示线，保持视觉紧凑
- **Theme 驱动**：构造函数接受 `&impl Theme`，颜色/间距/字号走 theme token

## 范围

### 本次交付 (MVP)
- 水平顶部 tab 栏
- 字符串 label
- 点击切换 + bind 状态绑定
- 选中态指示线 (accent 色)
- hover 反馈
- Theme 定制

### 未来 (todo)
- 垂直 tab 栏 (左侧)
- icon + text 混合 label

## API 设计

```rust
// 产品侧用法
Tabs::new(&theme)
    .tab("常规")
    .tab("快捷键")
    .tab("关于")
    .child常规内容)
    .child(hotkey_content)
    .child(about_content)
    .bind(|app: &MyApp| app.settings_tab)
```

### Builder 方法

| 方法 | 说明 |
|------|------|
| `Tabs::new(theme)` | 创建空 Tabs |
| `.tab(label: &str)` | 添加一个 tab 标签 (返回 self) |
| `.child(widget)` | 添加一个面板内容 (与 tab 一一对应) |
| `.bind(f: Fn(&S) -> usize)` | 绑定 active tab 索引到应用状态 |
| `.active(index)` | 设置初始 active tab (默认 0) |

### 约束

- `tab()` 和 `child()` 数量必须一致 (运行时 panic 检查)
- active 索引越界时钳制到最后一个 tab (不 panic)
- tab 栏高度固定 (由 theme 字号 + padding 决定)

## 结构

```
Tabs
  ├── tab_bar (自绘: 文字 + 指示线)
  └── children[active] (面板, 复用 MultiPanel 的显隐逻辑)
```

## 文件位置

- 实现: `danqing/src/widget/view/tabs.rs`
- 模块注册: `danqing/src/widget/view/mod.rs` (pub mod + pub use)
- re-export: `danqing/src/widget/mod.rs` (pub use view::Tabs)

## 主题 Token

| Token | 用途 |
|-------|------|
| `text_primary` | 选中 tab 文字色 |
| `text_secondary` | 未选中 tab 文字色 |
| `accent` | 选中指示线颜色 |
| `hover_bg` | tab hover 背景 (可选) |
| `spacing_md` | tab 间距 |
| `font_size_small` | tab 文字字号 |

## 测试策略

- 单元测试: tab/child 数量一致性检查、active 钳制、bind 驱动切换
- 视觉测试: 与 MultiPanel 测试模式一致 (Stub 子组件验证 sync/layout/paint/event 传播)

## Success Criteria

- [ ] `Tabs` 组件可在 danqing 内编译通过
- [ ] `cargo clippy` 零警告 (在 danqing 仓库内)
- [ ] 单元测试全绿
- [ ] 产品侧 (danqing-clipboard) 可用 `danqing::widget::Tabs` 替换当前手写 TabBar
- [ ] 产品侧 `cargo clippy` + `cargo test` 全绿
