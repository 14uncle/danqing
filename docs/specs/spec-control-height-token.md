# Spec: 统一控件高度 Token

> 状态: Draft  
> 日期: 2026-08-27  
> 作者: 十四叔

## Objective

在 `Theme` trait 中新增 `control_height()` token，让所有单行表单控件（Button、TextInput、IconInput）默认高度统一为 **36px**，消除并排时的视觉错位。

**用户场景：** 效率工具表单页——搜索框旁边放按钮、设置页输入框与确认按钮同行——默认就对齐，不需要产品层手动调 padding。

## Tech Stack

- Rust, 丹青 UI 框架
- `Theme` trait (`src/theme.rs`)
- 受影响组件：`Button`, `TextInput`, `IconInput`
- **不受影响：** `TextArea`（多行，高度随内容）

## 现状分析

### 当前高度

| 组件 | padding 垂直 | 内容行高 | 实际高度 |
|------|-------------|---------|---------|
| Button | 24px (`spacing_lg` 16 + `spacing_md` 12) | ~19px | ~43px |
| TextInput | 16px (`spacing_md` 12 + `spacing_sm` 8) | ~19px | ~35px |
| IconInput | 同 TextInput | ~19px | ~35px |

**差值：~8px。** 按钮比输入框高出一截，并排放置时肉眼可见。

### 根因

`Theme` 没有统一的控件高度约束，各组件各自组合 padding + 内容高度。

## Commands

```bash
cargo test --lib --tests
cargo clippy -- -D warnings
cargo fmt --check
cargo run --example showcase   # 视觉验证
```

## 改动范围

| 文件 | 改动 |
|------|------|
| `src/theme.rs` | `Theme` trait 新增 `fn control_height(&self) -> f32` 带默认实现 `36.0`；`LightTheme` 显式实现 |
| `src/widget/base/button.rs` | 新增 `control_height` 字段；`layout` 中 `natural_height.max(control_height)` |
| `src/widget/form/text_input.rs` | 同上 |
| `src/widget/form/icon_input.rs` | 复用内部 TextInput 的 control_height，不新增字段；`layout` 确保总高对齐 |
| `examples/showcase.rs` | 确认视觉对齐（无代码改动预期，仅验证） |

## Code Style

```rust
// Theme trait 新增（带默认实现，SceneTheme 自动继承）
/// 标准控件高度 (按钮、输入框等单行表单控件)。
fn control_height(&self) -> f32 {
    36.0
}

// Button / TextInput layout 中使用
let natural_height = content_height + self.padding.vertical();
let height = natural_height.max(self.control_height);
```

遵循项目约定：中文文档注释、内部实现英文命名、token 带默认实现。

## Testing Strategy

| 测试类型 | 内容 |
|----------|------|
| 单元测试 | `theme` 模块：`control_height` 返回 36.0，区间断言 32~44 |
| 单元测试 | `button` / `text_input`：layout 后高度 == 32（空内容场景） |
| 现有测试 | 更新因高度变化而失败的断言 |
| 视觉验证 | `showcase` 示例中按钮与输入框并排对齐 |

## Boundaries

- **Always:** `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` 全绿
- **Ask first:** 改变默认值（36px）；扩展到其他组件（Switcher、Tabs 等）
- **Never:** 不动 TextArea（多行组件）；不改现有 padding 语义

## Success Criteria

1. `LightTheme.control_height()` 返回 `36.0`
2. `Button::new(Text::new("OK")).layout(...)` 输出高度 == 36.0
3. `TextInput::new().layout(...)` 输出高度 == 36.0
4. `showcase` 中按钮与输入框并排时顶部/底部精确对齐
5. 全量测试通过，clippy 零警告

## Open Questions

无。值已确认 36px，范围已确认 3 个组件。
