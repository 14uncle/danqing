---
name: ellipsis-ascii-dots-convention
description: 结尾省略号一律用 ASCII "..." 不用 U+2026 "…" —— Text 组件拆分渲染底边对齐; U+2026 在 CJK 字体居中渲染不靠底
metadata:
  type: project
---

丹青生态的省略号约定（2026-08-18 剪贴板占位文本 bug 定位后确立）:

- **结尾省略号**（按钮/占位文本，表"还有未完")：用 ASCII 三点 `"..."`,**不用** U+2026 `"…"`。danqing `Text` 组件检测 `"..."` 后缀自动拆分渲染：前段走 ascent baseline，省略号走 descent baseline 底边对齐（`src/widget/base/text.rs`)。TextInput 无拆分逻辑，但 ASCII 点号天然落 baseline = 文字底边。
- **中间截断省略号**（列表路径截断 `C:\Users\gwh…\xxx.png`)：用 U+2026 `"…"`, 居中渲染是正确视觉，不要改成 ASCII。
- TextInput 占位文字**不要加 placeholder_offset 与输入文字错位**: 实测 15px 下 ascent=15 / line_height=18，文本带 padding.top..+line_height 在框内居中，同 baseline 则占位→正文零跳变。排版估算别猜字体指标，用 TextBatch::ascent/descent/line_height 实测。

**Why:** U+2026 在 CJK 字体中字形垂直居中，结尾使用时永远浮在半空；旧代码曾用 placeholder_offset=6 按错误 ascent 估算（≈12, 实际 15）硬压，把占位文字压出文本带导致整体偏低。

**How to apply:** 写 UI 文案时结尾省略号直接打三个英文句点；需要垂直位置参考时读光标矩形 (area.y+padding.top, 高 line_height) 作为文本带基准。相关: [[danqing-visual-debug-tooling]]
