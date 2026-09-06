---
name: font-cjk-mono
description: 框架内嵌字体 = Sarasa Mono SC SemiBold(GB2312 子集),Noto Sans SC 已退役;为何 mono/为何字重而非超采样/DPI 遗留
metadata:
  type: project
---

2026-09-06 框架字体重构（`commit`，spec `docs/specs/font-cjk-mono.md`，plan `tasks/plan-font-cjk-mono.md`）。

**核心改动**：`assets/fonts/ofl-mono.ttf` = Sarasa Mono SC **SemiBold**（GB2312 子集，2.01MB，`tools/subset-mono-font.py` 可复跑，源 `_font_tmp/SarasaMonoSC-SemiBold.ttf` 需重下载）；旧 `ofl-sans.ttf` 移入 `assets/fonts/archive/`；`Font::embedded_sans → embedded_mono`。danqing-log/pomodoro/clipboard **不写字体代码**，自动继承。

**Why（非显而易见的推理）**：
- **mono 而非 Sans**：日志正文要等宽对齐（缩进/字段一眼分出），比例字体做不到——这是 vs klogg 观感差距之一。
- **字重（SemiBold）而非超采样**：fontdue 是**解析式抗锯齿**（逐像素解析覆盖率），最终位图**封顶于目标 px**——26px 栅格再缩回 13px 跟直接 13px 基本同结果，**变不出更多像素**，13px 细笔画就是细、就是灰。所以「2× 超采样锐化小字」**不兑现承诺**，复盘撤销；换 **SemiBold**（笔画粗、对比强）才是 13px 的可见锐化杠杆。Sarasa 无 Medium 档，用 SemiBold。
- **子集范围**：GB2312 常用字(~6763) + ASCII + 常见中文标点；超范围 CJK 回落 fontdue `.notdef`（诚实边界，日志多为 ASCII）。

**How to apply**：
- 换字重/改子集 → `python tools/subset-mono-font.py [源.ttf]`（需 fonttools）。
- 「小字仍软」排查顺序：**① 字重档位** → **② 字号**（13→14-15px）→ **③ DPI-aware 栅格**（字体 px × scale_factor；danqing 现按逻辑 px 栅格，屏幕 >100% 时文字在物理域偏小，**未处理，v1.x 候选**）→ ④ hinting（fontdue 不支持，勿引入新栅格器）。
- 除非重现「小字不清晰」投诉，勿回退到 SSAA 路线（已被证据否掉）。
