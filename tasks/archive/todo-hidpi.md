# Todo: HiDPI 缩放支持

> Plan: `tasks/plan-hidpi.md`
> Spec: `docs/specs/hidpi-scale-factor.md`

## Phase 1: 边界改造

- [x] T1: TextBatch scale 化
  - Acceptance: `TextBatch` 新增 `set_scale_factor(f32)`（默认 1.0）；`push_text` 按 `round(px×s)` 物理栅格化、坐标 ×s 后物理格吸附；`measure/line_height/ascent/descent` 返回逻辑值（物理指标 ÷s）；`push_clip` 收逻辑 rect 内部 ×s；scale 变更清图集缓存；s=1.0 行为与现状逐位一致
  - Verify: 新增单测 s=2.0（落点/尺寸 ×2、measure 返回 ÷2、栅格 px=30）+ s=1.5（取整一致）+ s=1.0（恒等）；`cargo test render::text` 全绿
  - Files: `src/render/text.rs`
  - 检查点: **CP1**

- [x] T2: 输入边界换算
  - Acceptance: `convert_event` 增加 scale 参数；`CursorMoved` 坐标 ÷s；`MouseWheel` PixelDelta ÷s（LineDelta 不动）；handler 存 `self.cursor` 处 ÷s；handler 现传 1.0（T3 才接线），行为不变
  - Verify: 新增单测：物理 (200,100) @s=2 → 逻辑 (100,50)；PixelDelta ÷s、LineDelta 原样；`cargo test window` 全绿
  - Files: `src/window/event.rs`, `src/window/handler.rs`
  - 检查点: **CP1**

## Phase 2: 翻牌接线

- [x] T3: 布局视口逻辑化 + scale 接线
  - Acceptance: handler 新增 `scale: f64` 字段，窗口创建时取 `window.scale_factor()` 初始化并下发 `texts.set_scale_factor` + `context.set_scale_factor`；`render_frame` 布局视口 = `last_real_size ÷ s`（逻辑）；`Context::render` 内 rect/image pass 喂逻辑尺寸、text pass 喂物理尺寸（DrawTarget 分域）；s=1.0 本机行为与改造前一致
  - Verify: 新增单测（视口换算助手：3200×2000 @s=2 → 1600×1000）；`cargo test --lib --tests` 全绿；showcase 本机 100% 肉眼无漂移
  - Files: `src/window/handler.rs`, `src/render/mod.rs`
  - 检查点: **CP1**

## Checkpoint CP1: s=1.0 回归锁

- [x] `cargo test --lib --tests` 现有测试**零修改**全绿 + 新增单测绿（616 lib + 58 集成）
- [x] `cargo clippy -- -D warnings` 零警告 + `cargo fmt --check`
- [x] showcase 100% 人工比对无漂移（截图 target/tmp/showcase-cp1.png，文字/图像/分层全部正常）

## Phase 3: 动态切换与验收

- [x] T4: ScaleFactorChanged 动态切换
  - Acceptance: `window_event` 新增 `ScaleFactorChanged` arm：更新 `self.scale` → `texts.set_scale_factor`（清图集）→ `context.set_scale_factor` → 请求 redraw；尺寸/表面走既有 Resized 单路径不双写；隐藏态 scale 照常更新（无幻影问题）
  - Verify: arm 内各 setter 的单测已在 T1/T3 落地（清图集/非法值拒绝）；arm 本体为纯接线，由 T5「运行中改系统缩放」人工验证；`cargo test window` 全绿
  - Files: `src/window/handler.rs`
  - 检查点: **CP2**

- [x] T5: 人工视觉验收 (**2026-09-22 通过**)
  - Acceptance: 本机 1080p 屏 Windows 缩放调 200%：showcase 与 100% 基准逐区域等比、文字清晰度不劣化（S1）；125%/150% 各抽验一次无破版（S2）；运行中改系统缩放 UI 等比跟随（S3/T4 实测）；日志无「栅格化失败」warn（R1 观察；图集满时 err 含「字形图集已满」，触发则停工请示）
  - 验收记录: showcase 全套通过（S1 等比 / S2 125%·150% 抽验 / S3 动态跟随）；**pomodoro + danqing-log 源码复验通过**（RustRover debug，200%，danqing-log 走 `.cargo/config.toml` patch）。R1: 当日全部运行日志零「栅格化失败」warn（grep 口径「图集|字形」，覆盖 measure/paint 两路径）。**注**: 原扳机写「图集满」是死字符串（实际日志为「栅格化失败」/「字形图集已满」），2026-09-22 review 修正。
  - **事故与收口**: 首轮产品「不通过」实为跑了旧框架 —— danqing-log 的 patch 默认关、lock 钉修复前 rev `21e03952`；pomodoro 点到商店版（包私有日志 09:30 实证，源码 exe 未重建）。与修复本身无关。为此框架新增启动 scale 日志（`2ac0900`），「跑的是哪个框架/几倍缩放」一贴日志可辨。
  - Files: 无代码改动（诊断日志行见 `2ac0900`）
  - 检查点: **CP2**

## Checkpoint CP2: spec S1–S4 全满足 (**2026-09-22 达成**)

- [x] S1/S2/S3 人工验收通过（见 T5 验收记录）
- [x] S4 全部测试绿（617 lib + 58 集成，现有测试零修改）

## Phase 4: 联动与收口

- [x] T6: danqing-log 联动验证 (**2026-09-22 完成**)
  - Acceptance: danqing-log 内 `cargo update -p danqing` 后 253 测试全绿 + 本机启动冒烟无漂移 = spec S5「产品零代码改动」成立；lock 是否提交由用户定夺
  - 验收记录: danqing push `d8133ea` 后关 patch、`cargo check` 驱动重解一步钉到 `danqing#d8133ea1`；**253 测试全绿**（88 lib + 154 main + 8 genlog + 3 keygen）；启动冒烟通过 —— 日志实证「DPI 缩放接线：scale=1」（新框架在跑 + 用户已恢复 100%）、零「栅格化失败」warn。产品零代码改动。**lock 用户裁决提交**（danqing-log `08365a0`）。
  - Verify: `cargo test`（danqing-log）+ 人工启动
  - Files: `danqing-log/Cargo.lock`（已提交，`08365a0`）

- [x] T7: 记忆/文档回填 (2026-09-22 完成, 本批一并落地)
  - Acceptance: `memory/font-cjk-mono.md` 的「DPI 栅格未处理」条目改写为已支持（含 A3 取整决策）；spec 状态翻「已完成」；`MEMORY.md` 索引同步；本 plan/todo 归档 `tasks/archive/`
  - Verify: 文件核查
  - Files: `memory/font-cjk-mono.md`, `MEMORY.md`, `docs/specs/hidpi-scale-factor.md`
