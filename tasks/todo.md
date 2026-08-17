# Task List: Text 省略号底边对齐

## Task 1: TextBatch 新增 descent 方法

**Description:** 在 TextBatch 上新增 `descent(px)` 方法，返回 baseline 到行底的距离。与 `ascent(px)` 对称。

**Acceptance criteria:**
- [x] `descent(px)` 方法存在且可编译
- [x] 返回值为字体 metrics 的 descent，若无则回退 `px * 0.2`

**Verification:**
- [ ] cargo clippy --all-targets -- -D warnings 零警告
- [ ] cargo test 全绿

**Dependencies:** None

**Files likely touched:**
- `src/render/text.rs`

**Estimated scope:** XS

---

## Task 2: Text::paint 检测 "..." 并拆分渲染

**Description:** 修改 `Text::paint`，当内容含 "..." 时拆分为两段渲染：前段按原 baseline，省略号按底边对齐。

**Acceptance criteria:**
- [x] 含 "..." 的文本，省略号 glyph 底边与行底对齐
- [x] 前段文字 baseline 不变
- [x] 不含 "..." 的文本行为完全不变

**Verification:**
- [ ] cargo clippy --all-targets -- -D warnings 零警告
- [ ] cargo test 全绿

**Dependencies:** Task 1

**Files likely touched:**
- `src/widget/base/text.rs`

**Estimated scope:** S

---

## Task 3: 新增单测覆盖两种情况

**Description:** 为 Text widget 新增单测：含 "..." 时省略号底边对齐，不含时 baseline 不变。

**Acceptance criteria:**
- [x] 测试 `text_with_ellipsis_uses_descent_baseline` 存在且通过
- [x] 测试 `text_without_ellipsis_uses_ascent_baseline` 存在且通过

**Verification:**
- [ ] cargo test 全绿

**Dependencies:** Task 2

**Files likely touched:**
- `src/widget/base/text.rs`

**Estimated scope:** XS
