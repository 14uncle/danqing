# Implementation Plan: 丹青阶段 1 视觉 Remediation R2 —— 让玻璃成立

> 依据：2026-07-22 阶段 1 视觉复盘（showcase 截图对照 `docs/specs/phase1-design-system.md` 十条验收）。
> 前一轮：`tasks/plan-phase1-visual-remediation.md`（✅ 已完成）解决了"苍白/无层次"，但 surface 提到 0.95 后玻璃感丢失，且遗留字体、选中态、TextArea 接缝问题。
> **状态：✅ 已完成（2026-07-23 阶段 1 正式关闭）**

## Overview

本轮 5 项修复源自复盘结论：

1. **玻璃不成立**——`surface()` alpha 0.95 近不透明，渐变背景透不上来（spec 原文示例为 0.60)。
2. **TitleBar 是一条纯白带子**——`bg: theme.surface()`(`src/widget/title_bar.rs:174`)，与背景断裂，视觉上误以为原生栏还在。
3. **导航选中标记豆腐块**——showcase 用 `"▶ "` 前缀（`examples/showcase.rs:410`)，当前字体无此字形；且选中态除前缀外无任何区分。
4. **TextArea 卡片接缝**——`TextArea` 自画 surface 背景（`src/widget/form/text_area.rs:79`),showcase 外层又套 `UiBox::themed`，两层白不同圆角叠出缝。
5. **正文发灰**——字体为单字体无逐字形回退（`src/text/font.rs`)，系统黑体查找疑似静默失败后兜底 XiaoWei（笔画细的艺术字体），小字号站不住。✅ 已拍板：**内嵌 OFL 黑体**。

已核实的有利事实：背景/矩形/文本三个 pass 均为 `ALPHA_BLENDING`(`src/render/background.rs:213`、`rect.rs:666`、`text.rs:326`)，降 alpha 不动渲染管线；`RectBatch` 支持圆角矩形/圆形，选中标记可以画出来，不依赖字形。

## Architecture Decisions

- **字体策略（已确认）**：新增一款 OFL 黑体（思源黑体 / Noto Sans SC，子集化）提交到 `assets/fonts/`，加载链为 系统黑体 → **内嵌 OFL 黑体** → XiaoWei（末位兜底）。XiaoWei 保留作品牌资产，本轮不引入多字体混排（标题用品牌字体留待后续）。
- **选中态图形化**：不赌字体字形覆盖率，选中标记用 `UiBox` 画（accent 圆点/竖条）+ 按钮背景区分，`Button::color()` builder 已存在可直接用。
- **玻璃感用 token 回调实现**：surface alpha 降到 0.70 一档，配合既有阴影/边框拉开层次；不做运行时 backdrop blur（同前轮决策）。
- **接缝在框架层修**:TextArea 在 Scrollable 视口内应填满高度，而不是内容多高画多高；showcase 包装同步简化。

## Task List

### Phase 1: 字体与选中态

- [x] **Task 1: 内嵌 OFL 黑体并诊断字体加载** ✅ 2026-07-22
  - **Description:** 子集化思源黑体 / Noto Sans SC（覆盖 GB2312 常用字 + ASCII + 常用标点，目标 ≤ 3 MB）为 `assets/fonts/ofl-sans.ttf`，同步提交 OFL LICENSE;`src/text/font.rs` 加载链改为 系统黑体 → 内嵌 OFL 黑体 → XiaoWei；把字体实际来源（`Font::load` 已知的 source 描述）提升为 info 级日志，确认本机此前是否一直兜底 XiaoWei。
  - **完成记录：** `tools/subset-font.py` 一键重制（instancer 非 inplace 返回值坑已修）;ofl-sans.ttf 2.21 MB(7553 字）;**诊断结论：本机实际使用 system Microsoft YaHei，并未兜底 XiaoWei** —— 正文发灰与 ▶ 豆腐块另有根因（YaHei 缺字形 + 小字号渲染）,T2 图形化选中标记因此是正确方向。
  - **Acceptance:**
    - `assets/fonts/ofl-sans.ttf` 与许可文件存在，体积 ≤ 3 MB。
    - `tests/assets.rs` 新增存在性检查并 pass。
    - `font.rs` 单元测试更新：回退链顺序正确、新字体可解析。
    - showcase 日志可见实际字体来源；正文视觉不再发灰（人工）。
  - **Verify:** `cargo test --lib text` + `cargo test --test assets`
  - **Dependencies:** None
  - **Files:** `assets/fonts/ofl-sans.ttf`, `assets/fonts/OFL.txt`, `src/text/font.rs`, `tests/assets.rs`，可能新增 `tools/subset-font.py`
  - **Scope:** M

- [x] **Task 2: 导航选中态图形化** ✅ 2026-07-22
  - **Description:** 移除 showcase 侧边栏的 `"▶ "/全角空格` 文本前缀；改为按钮内 Row 布局：选中项前置 accent 小圆点（`UiBox` 圆形，直径约 6px）或左侧竖条，未选中项占同等宽度保持对齐；同时选中按钮背景用 `Button::color()` 区分为更深的品牌蓝。若发现框架缺"按状态绑定样式"的能力，在 showcase 本地解决，不改框架。
  - **完成记录：** 框架缺颜色绑定能力，经作者拍板偏离"不改框架"一句：`Button::bind_color`（与 `Text::bind` 同构，附单元测试）;showcase 本地 `NavItem` 画左缘白色竖条，选中 = 深背景（accent RGB ×0.8 派生）+ 竖条 + 白字。
  - **Acceptance:**
    - 四个导航按钮选中项有图形标记 + 背景区分，未选中项整齐对齐。
    - 全窗口不再出现豆腐块字形。
  - **Verify:** 人工运行 `cargo run --example showcase` 切换四个分类确认
  - **Dependencies:** None（与 Task 1 独立；即使字体修好也不依赖 ▶ 字形）
  - **Files:** `examples/showcase.rs`
  - **Scope:** S

### Checkpoint 1: 文字与导航可用

- [x] `cargo test --lib --tests` 全绿；`cargo fmt --check` / `cargo clippy -- -D warnings` 通过。
- [x] 人工确认：正文不发灰、无豆腐块、当前页导航可辨。

### Phase 2: 玻璃感

- [x] **Task 3: surface 降 alpha，让背景透上来** ✅ 2026-07-22
  - **Description:** `LightTheme::surface()` 从 0.95 降到 0.70 一档（0.70–0.75 区间目视迭代取最优）;`surface_variant`、`divider`、`border` 在半透明表面上的可见性同步校准；评估输入框是否需要比卡片更实的背景（效率工具输入区可读性优先，允许 `TextInput`/`TextArea` 背景保持更高 alpha，通过各自 token 而非全局 surface)。同步更新 `theme.rs`、`tests/design_system.rs` 及受影响组件测试中的颜色断言。
  - **完成记录：** surface 取 0.72;Theme 新增 `surface_input()` token(0.95),TextInput/TextArea 改用；showcase 按钮白字从 `t.surface()` 改 `Color::WHITE` 防洗白；新增玻璃区间（0.6~0.8）与输入区更实两条护栏测试。**alpha 档位待 T7 人工目视复核。**
  - **Acceptance:**
    - 卡片下能透出渐变背景的蓝色倾向与光晕，层次不靠纯阴影。
    - 输入框文字可读性不劣于现状。
    - 全部测试绿，无新魔法值。
  - **Verify:** `cargo test --lib --tests` + 人工 showcase
  - **Dependencies:** None
  - **Files:** `src/theme.rs`, `src/widget/form/text_input.rs`, `src/widget/form/text_area.rs`, `tests/design_system.rs`
  - **Scope:** S

- [x] **Task 4: TitleBar 融入背景** ✅ 2026-07-22
  - **Description:** `TitleBar` 不再画不透明背景条：背景条改为透明（不 push 背景 rect）或极低 alpha，让窗口背景图直接贯通到顶；按钮 hover 背景、关闭按钮 danger 反馈保留；确认与去装饰窗口圆角/阴影协调。同步更新 `title_bar.rs` 中断言背景色的测试。
  - **完成记录：** `bg` 改 `Color::TRANSPARENT` 且 paint 跳过透明背景（附"不产不可见矩形"护栏测试）;LOGO 内填充改用 `surface_input` 保持辨识度；按钮 hover/关闭 danger 不变。
  - **Acceptance:**
    - 标题栏区域可见渐变背景，不再是一条白带；LOGO、标题、三按钮在其上清晰可辨。
    - 拖拽、双击最大化、三按钮功能不变（现有命中测试全绿）。
  - **Verify:** `cargo test --lib title_bar` + 人工 showcase
  - **Dependencies:** Task 3（一起目视迭代 alpha 档位）
  - **Files:** `src/widget/title_bar.rs`
  - **Scope:** S

- [x] **Task 5: 修 TextArea 卡片接缝** ✅ 2026-07-22
  - **Description:** 框架层：TextArea 在 Scrollable 视口内高度应填满可用空间（视口高于内容时背景铺满视口，内容超高时随内容增长），而非只画内容高度；showcase 层：简化 `textarea_card` 包装，去掉外层 `UiBox` 与 TextArea 重复绘制的 surface（二选一保留背景职责）。注意不要破坏 M3 的滚动与撤销/重做行为。
  - **完成记录：** TextArea 新增 `height()` 最小高度 builder（空内容铺满视口、内容超高仍增长，两条回归测试）;showcase 外层 UiBox 改透明尺寸壳，背景职责归 TextArea。滚动/编辑行为人工复核留给 T7。
  - **Acceptance:**
    - 表单页多行卡片为一个完整圆角白框，顶部无接缝、无双层框。
    - `cargo test --lib` 中 TextArea/Scrollable 测试全绿；滚动行为人工确认不变。
  - **Verify:** `cargo test --lib` + 人工 showcase
  - **Dependencies:** None
  - **Files:** `src/widget/form/text_area.rs`, `src/widget/form/text_editor.rs`（视实现）, `examples/showcase.rs`
  - **Scope:** M

### Checkpoint 2: 玻璃成立

- [x] `cargo test --lib --tests` 全绿；`cargo clippy -- -D warnings` 零警告。
- [x] 人工确认：卡片透出背景、标题栏融入顶部、多行卡片无接缝。

- [x] **Task 6.5: 重制背景渐变让玻璃可读** ✅ 2026-07-22（视觉终审后补入）
  - **Description:** T3 完成后 surface 半透明已生效，但 gradient.png 顶部 (247,249,254) → 底部 (228,236,251) 整体过淡，72% 白叠 98% 白肉眼无法分辨玻璃。重制背景资产提高饱和度与起伏。
  - **完成记录：** `tools/export-background.py` 顶部改 (240,245,253)、底部改 (186,208,244)，光晕峰值 alpha 26→64;showcase glow 叠加 0.15→0.25;`theme.background()` 清屏色同步新顶部色。截屏自验（`tools/capture-screen.ps1`）：层次与品牌蓝氛围可读，深色文字在全幅背景上仍清晰。

- [x] **Task 6.6: 实线描边内缩，修复输入框边线发虚/消失** ✅ 2026-07-23（终审指认后补入）
  - **Description:** 用户终审指认：单行框左右边发虚；多行框焦点态左边、下边没画上。根因：`push_rounded_border` 把 1px 描边骑跨在几何边缘上（内外各 0.5px),Scrollable 裁剪边界削掉外凸半线宽后只剩 0.5px(125% DPI 下 0.625 物理像素），能否看见全凭亚像素相位。
  - **完成记录：** 四条直边与圆角弧（弧半径 r - half）整体内缩，与既有的虚线描边内缩行为一致；新增"实例全部落在矩形内"与"裁剪下保持完整线宽"两条几何回归测试。PrintWindow 直接抓窗口验证：单/多行框四边边线均匀可见；"背景鬼影"同步证实为 DWM 截屏伪影（直接窗口像素里不存在）。

- [x] **Task 6.7: 细线像素对齐 + SDF 过渡带收窄，根治底边发虚** ✅ 2026-07-23（终审再指认后补入）
  - **Description:** T6.6 后用户再指认"底边还是有点发虚/有点淡/没画上"。像素级诊断（PrintWindow + 实例几何转储 + alpha=1 光栅实验）定位两层根因：① 1px 矩形只光栅化到一行像素，SDF smoothstep(-w,w) 在 2px 过渡带下单行覆盖率封顶 84%、随亚像素相位最低 50%(sRGB surface 线性混合进一步把 0.18 边框洗淡）;② 输入框高度由字体行高算出必为分数（如 35.951)，底边永远落在坏相位。
  - **完成记录：** `push_rounded_border` 描边矩形先向内对齐整数像素（顶/左 ceil、底/右 floor，不越出原矩形）;rect.wgsl 过渡带 `w = min(fwidth(d), 半尺寸)`，细线中心行满覆盖、大矩形 AA 不变。新增"直边对齐整数像素"回归测试（先 RED 后 GREEN)。抓图验证：单/多行框四边全部满强度 (233,234,234) 均匀可见。

### Phase 3: 收尾

- [x] **Task 6: 排版清理** ✅ 2026-07-22
  - **Description:** showcase 中 `"字数:{} 行数:{}"` 等半角冒号改全角并补空格；全文件扫一遍同类问题（如 `"最后按键:{}"`)。
  - **完成记录：** `"字数:{} 行数:{}"` 改全角冒号；其余文案（`输入：`、`已输入：{}`、`最后按键：{}`、`多行：`）原本即为全角，扫描确认无遗漏。
  - **Acceptance:** 界面文案标点统一全角。
  - **Verify:** 人工 showcase
  - **Dependencies:** None
  - **Files:** `examples/showcase.rs`
  - **Scope:** S

- [x] **Task 7: 回归验证与阶段 1 终审** ✅ 2026-07-23
  - **Description:** 全量验证 + 人工终审，对照 `docs/specs/phase1-design-system.md` 十条验收逐条过；通过后回补 `tasks/todo-phase1.md` 的 Checkpoint Complete 三项勾选，阶段 1 正式关闭。
  - **完成记录：** 十条验收逐条核实（含 `window.rs` 窗口图标已用 `logo_256.png`)；程序化 LOGO 与新 logo.svg 比例一致（16.4%/18%/10.2%/78.1% 破框朱砂滴）;`cargo fmt` / `clippy -D warnings` / 230 测试 / `cargo build --release` 全绿；作者人工终审通过（边框四边满强度确认为最后放行项）,Checkpoint Complete 三项已勾。
  - **Acceptance:**
    - `cargo fmt` / `cargo clippy -- -D warnings` / `cargo test --lib --tests` / `cargo build --release` 全绿。
    - spec 验收 4（毛玻璃整体效果）与 10（人工视觉验证）通过。
  - **Verify:** 上述命令全部执行 + 人工 showcase
  - **Dependencies:** Task 1–6
  - **Files:** `tasks/todo-phase1.md`（勾选）
  - **Scope:** S

### Checkpoint 3: 阶段 1 关闭

- [x] spec Success Criteria 10/10 通过。
- [x] `tasks/todo-phase1.md` 全部勾选。
- [ ] 准备进入阶段 2 POC（剪贴板历史管理器）。

## Risks and Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| 半透明叠加在噪声背景上显脏（前轮已触发过一次，才把 surface 提到 0.95) | 高 | Task 3/4 一起目视迭代 alpha 档位；必要时把 `noise.png` 不透明度从 0.06 再降；输入框保留更实背景 |
| 字体子集化缺字（生僻字/符号变豆腐） | 中 | 子集字符集按 GB2312 + 常用标点 + showcase 实际用字核对；选中态已图形化不依赖字形 |
| TextArea 填高改动影响 M3 滚动/IME 光标定位 | 中 | 只改背景绘制范围与 layout 高度协商，不动编辑状态机；`cargo test --lib` 全量回归 + 人工滚动验证 |
| 内嵌字体使 exe 体积明显增大 | 低 | 子集化控制在 ≤ 3 MB；发布构建确认体积增量 |

## Open Questions — 已确认

1. **正文字体**：✅ 内嵌 OFL 黑体（思源黑体 / Noto Sans SC 子集），系统黑体仍在链首，XiaoWei 末位兜底并保留作品牌资产。
2. **选中标记**：✅ 图形化（圆点/竖条 + 按钮背景区分），不依赖字体字形。
3. **surface alpha 目标值**:0.70–0.75 区间，实现时目视定档，以"透出渐变且文字可读"为准。

## Related Documents

- `docs/specs/phase1-design-system.md` — 阶段 1 原始规格（十条验收）
- `docs/specs/phase1-visual-remediation.md` — 前轮视觉 spec
- `tasks/plan-phase1-visual-remediation.md` — 前轮计划（已完成）
- `tasks/todo-phase1.md` — 阶段 1 总待办（Checkpoint Complete 待勾）
- `CLAUDE.md` — 项目约定与命令
