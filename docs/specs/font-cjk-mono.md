# Spec: 框架字体重构 —— 中文等宽 (Sarasa Mono SC SemiBold)

> @author 十四叔 · @date 2026/09/06（2026-09-06 复盘修订：SSAA 降级、字重 Regular→SemiBold）
> 前置：interview-me 确认（用户显式 yes）——log v1 闭环后，字体是最后观感差距（等宽对齐 + 小字不清晰）。框架级改动，danqing 全家（log/pomodoro/clipboard）受益。

## Objective

danqing 框架现嵌 `assets/fonts/ofl-sans.ttf`（Noto Sans SC / 思源黑体 GB2312 子集，`include_bytes!` 内嵌）。用户实测两大观感缺陷：

1. **非等宽**：日志正文用比例字体，缩进/字段无法对齐（日志场景核心心智缺失）。
2. **小字不清晰**：`fontdue::rasterize(ch, px)` 在目标 px 直接栅格、**无 hinting**；13px 级别中文细笔画发灰，观感软（Noto 350 偏细是主因）。

本 spec 解决两者（**复盘修正为字重路线，SSAA 降级不做，见 §Crispness 决策**）：
- **换字体**：主字改 **Sarasa Mono SC SemiBold**（iosevka 拉丁 + 源黑 CJK 的 CJK-mono），**子集化**为 GB2312 + ASCII + 常用标点；Noto Sans SC 子集**退役归档**。
- **锐化**：靠 **SemiBold 字重**（笔画更粗、小字对比更强）；**非超采样**（fontdue 解析式 AA 超出小字号锐化能力，见下）。
- 呈现范围：框架所有文本（UI + 正文）统一走新 mono（用户已批「mono 更好就干掉 Sans」）。

## Crispness 决策（2026-09-06 复盘，替代原 SSAA）

原计划用「图集 2× 超采样 + LANCZOS」锐化小字。实施前复盘 fontdue 行为纠偏：
fontdue 是**解析式抗锯齿**（逐像素解析覆盖率），栅格化最终位图**被封顶在目标 px**——26px 放大再缩回 13px 跟直接 13px 基本同结果，**变不出更多像素**，细笔画在 13px 下就是细、就是灰。故 SSAA 对小字锐化**不兑现承诺**，降级不做。真实杠杆：

| 杠杆 | 作用 | 结论 |
|------|------|------|
| **字重**（SemiBold） | 笔画粗、小字对比强，肉眼可见「锐」 | **采纳**（Sarasa 无 Medium 档，用 SemiBold） |
| 字号 | 13→14-15px 中文更稳 | 暂不动（现状达标，人工比对通过） |
| DPI（字体px×scale） | 若屏幕 >100% 需物理 px 栅格 | 实测截图达标，暂不动（留 v1.x） |
| hinting | 小字锐化正统 | fontdue 不支持，不引入新栅格器 |

## Tech Stack

- 字体：Sarasa Mono SC **SemiBold**（OFL，`sarasa-mono-sc-semibold`）
- 子集工具：fonttools `pyftsubset`（字体已随上游 OFL 授权；子集脚本落 `tools/`）
- 栅格：fontdue 0.9（`rasterize(ch, px)` 无内置 SSAA → 手动 2× + LANCZOS）
- 内嵌：`include_bytes!`（同现有机制）

## Commands

```bash
# 1. 生成子集字体 (需 fonttools; 范围见 §子集范围)
python tools/subset_font.py assets/fonts/src-sarasa-mono-sc.ttf assets/fonts/ofl-mono.ttf

# 2. 三件套 (danqing 仓)
cargo fmt
cargo clippy -- -D warnings
cargo test --lib --tests
```

## Project Structure

```
assets/fonts/
  ofl-sans.ttf        ← (退役) Noto Sans SC 子集，移入 assets/fonts/archive/ 或删除
  ofl-mono.ttf        ← Sarasa Mono SC 子集 (新, include_bytes 进框架)
  OFL.txt             ← 保留 (Sarasa 的 OFL 授权归属)
src/text/
  font.rs             ← 改内嵌源 (include_bytes 路径/常量名) + embedded_mono 更名
  atlas.rs            ← 仅 embedded_sans→embedded_mono 调用点 (无 SSAA 改动)
tools/
  subset-mono-font.py ← pyftsubset 包装 (固定字符范围, 可复跑; Sarasa 静态 TTF 免 instancer)
```

## 子集范围

覆盖：**ASCII 全可打印 + 常用西方标点 + digits + Latin**；**CJK 取 GB2312 一级/二级常用字**（~6763 字）+ 常用中文标点（，。「」《》等）。理由：a) UI 串（「丹青日志」「过滤:」「已点击 0 次」等）全在其中；b) 日志内容常见中文（错误词/时间）覆盖；c) 体积可控（完整 Sarasa 数 MB → 子集约 2-4MB）。任意超范围 CJK 回落 fontdue `.notdef`（诚实边界：日志多为 ASCII，中文子集是非目标池，不做全量覆盖）。

## 超采样——**不做**（见 §Crispness 决策，原 2× 方案已撤）

原 §超采样实现 撤销：fontdue 解析式 AA 使最终位图封顶于目标 px，SSAA 对小字锐化不兑现，改走字重。`atlas.rs::get_or_rasterize` **保持原生尺寸栅格不变**（无 SSAA 改动）。

**唯一下游修正**：`Font::embedded_sans` → 更名 `Font::embedded_mono`（memo/atlas/mem_probe/测试同步）；`render::text.rs` 的 `descent` 测试由硬编码回退值改为与加载字体真实 descent 比对（旧断言被 Noto 恰为 3.2 凑巧通过，换 mono 后露馅）。

## Testing Strategy

- `src/text/font.rs` 单测：子集字体能栅格化 CJK（「你」）与 ASCII（`a`），advance > 0；覆盖 `，。：「·+`。
- `render::text.rs` descent 测试：与加载字体真实 metrics 比对（font-agnostic）。
- 三件套全绿（fmt + clippy `-D warnings` + `cargo test`）。
- 人工：`cargo run --example danqing-showcase` base/form 页——小号中文锐利（SemiBold 对比强），正文/表格等宽对齐；旧 Noto 观感不存在。

## Boundaries

- **Always**：只改框架字体（换 mono、归档旧 Sans）；公开 API 除 `embedded_sans→embedded_mono` 更名外不动（`TextBatch::measure/push_text` 签名不变）；新资产提交 `assets/fonts/`；字重/子集走脚本可复跑。
- **Ask first**：引入第二个字体（等宽之外的 UI 字）；调字重档位；子集范围扩到非 GB2312 全量；做 DPI-aware 栅格（字体px×scale，v1.x 候选）。
- **Never**：引入非 OFL 授权字体（Sarasa 及构成均 SIL OFL 1.1）；把字体字节写死进 `.rs`（仍走 `include_bytes!`）；跳过子集直接内嵌完整 Sarasa（体积/内存不可控）。
- 跨产品：log/pomodoro/clipboard 不写字体代码——它们自动继承框架新 mono（呈现随之改变，layout 不改）。

## Success Criteria

- [x] `assets/fonts/ofl-mono.ttf` 为 Sarasa Mono SC **SemiBold** 子集，2.01MB（<3MB），`tools/subset-mono-font.py` 可复跑。
- [x] `include_bytes!` 指向新 mono；旧 `ofl-sans.ttf` 移入 `assets/fonts/archive/`（可回退）。
- [x] `Font::embedded_sans` → `Font::embedded_mono` 全链命名更新（内存/示例/测试）；公开度量 API 不变。
- [x] 三件套全绿（fmt + clippy `-D warnings` + 测试，386 lib 通过）。
- [x] 人工（用户「通过」）：小号中文锐利（SemiBold）、正文/表格等宽对齐；旧 Sans 从框架层移除。

## Open Questions（已决议，2026-09-06）

1. ~~字重~~ → **SemiBold**（Sarasa 无 Medium；SemiBold 笔画粗、对比强，人工过渡通过）。
2. ~~SSAA 倍率~~ → **不做超采样**（fontdue 解析式 AA 封顶目标 px，不锐化；改字重解决）——复盘纠偏，见 §Crispness 决策。
3. ~~旧字体去留~~ → **归档** `assets/fonts/archive/`（非删除，可回退）。
