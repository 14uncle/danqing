# Spec: Text 省略号底边对齐

## Objective

Text 组件自动检测内容中的 "..."，将其渲染在行底边而非与前段文字共用 baseline。前段文字位置不变，省略号作为装饰元素贴底。

**用途**: 按钮、标签等场景中，"清空…" 的省略号贴底比居中更自然。

**成功标准**:
- Text 内容含 "..." 时，省略号 glyph 底边与行底对齐
- 前段文字 baseline 不变
- 不含 "..." 的文本行为完全不变
- descent 值从字体 metrics 获取

## Tech Stack

- Rust 1.85+, edition 2024
- 框架: danqing (自绘渲染, ab_glyph 字体)

## Commands

```
Build: cargo build
Test:  cargo test
Lint:  cargo clippy --all-targets -- -D warnings
Fmt:   cargo fmt
```

## Project Structure

```
danqing/src/
├── render/text.rs        → TextBatch: push_text / measure / ascent / descent
├── widget/base/text.rs   → Text widget: layout / paint
```

## Code Style

中文注释; 文件头 `//! @author 十四叔` + `//! @date yyyy/MM/dd`

## Testing Strategy

- 单测: Text widget 的 paint 行为 (验证 "..." 存在时 baseline 偏移)
- 集成: 无 (纯渲染逻辑, 无状态交互)

## Boundaries

- Always: cargo fmt + clippy 零警告 + 测试全绿
- Ask first: API 变更
- Never: 破坏不含 "..." 的现有文本渲染

## Success Criteria

1. `TextBatch` 新增 `descent(px)` 方法，返回 baseline 到行底的距离
2. `Text::paint` 检测内容是否含 "..."
3. 若含: 前段按原 baseline 渲染，"..." 按 `area.origin.y + height - descent` 渲染
4. 若不含: 行为与现在完全一致
5. 新增单测覆盖两种情况

## Open Questions

- 无
