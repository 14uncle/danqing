# Implementation Plan: 小件包下沉 (S1/S2/S3)

> Spec: `docs/specs/SPEC-small-items.md`。来源: `docs/intent/framework-sinking.md` 小件包。
> 2026-09-10 审查: 6 件中 3 件不够格 (open_feedback/CSV BOM/MSIX), 砍至 3 件; 打包一个 spec, 统一五阶段推进。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。

## Overview

三件下沉: S1 `reveal_in_file_manager`(fs,跨平台) + S2 相对时间+历法+时区(time,改chrono) + S3 RGBA 降采样+等比缩进(image,纯逻辑)。

## Task List

### Phase 1: 框架组件

- [x] **T1: S1 `reveal_in_file_manager`** — `danqing/src/fs.rs`(新建) + `lib.rs` ✅ 2026-09-10
  - 内容: Explorer / Finder / xdg-open 三平台 cfg 分流
  - 验收: 三件套绿 (无测试, Command::spawn 不可测)
  - 估时: XS | 依赖: 无

- [x] **T2: S2 相对时间+历法+时区** — `danqing/src/time.rs`(新建) + `lib.rs` ✅ 2026-09-10
  - 内容: `relative_time` + `civil_from_days` + `local_tz_offset_seconds`(改chrono)
  - 验收: 8 测试直接搬 (刚/分/时/昨/更早/闰日/跨天/纪元); 三件套绿
  - 估时: S | 依赖: 无

- [x] **T3: S3 缩略图降采样** — `danqing/src/image.rs`(新建) + `lib.rs` ✅ 2026-09-10
  - 内容: `downscale_rgba` + `aspect_fit`
  - 验收: 4 测试 (aspect_fit 横/竖/不放大 + downscale_rgba 恒等/2×2→1×1); 三件套绿
  - 估时: XS | 依赖: 无

### Checkpoint 1: 框架侧完成

- [x] 三件套绿; S2 8 测试 + S3 4 测试锁定 (522 passed) ✅ 2026-09-10
- [ ] review (全模块) — 五轴
- [ ] 用户过目

### Phase 2: 产品迁移 (danqing 先 push → 产品重解)

- [x] **T4: pomodoro 删 S1 手写** — `danqing-pomodoro/src/main.rs` 删 `reveal_in_file_manager`+`reveal_attempt`, 改 `danqing::fs::reveal_in_file_manager` ✅ 2026-09-10
  - 验收: pomodoro 185 测试全绿; 三件套绿; net -35 行
  - 估时: XS | 依赖: T1 + danqing push

- [x] **T5: clipboard 删 S2 手写** — `danqing-clipboard/src/ui/history_list.rs` 删 `relative_time`+`civil_from_days`+`local_tz_offset_seconds`, 改 `danqing::time::*`; 测试 import 改框架; 删 `windows` 依赖的 `Win32_System_Time` feature ✅ 2026-09-10
  - 验收: clipboard 127 测试全绿 (8 测试零改动全绿, 改框架前缀); 三件套绿
  - 估时: S | 依赖: T2 + danqing push

- [x] **T6: clipboard 删 S3 手写** — `danqing-clipboard/src/ui/history_list.rs` 删 `downscale_rgba`+`aspect_fit`, 改 `danqing::image::*` ✅ 2026-09-10
  - 验收: clipboard 127 测试全绿; 三件套绿; 与 T5 同仓同 commit
  - 估时: XS | 依赖: T3 + danqing push

### Checkpoint 2: 全量验收

- [x] 各仓三件套绿; 分仓分别提交, message 注明关联 ✅ 2026-09-10 (danqing 82ff49e/00fa98a, clipboard ea706d3/e308f5f, pomodoro 54522f1)
- [x] 人工过目 ✅ 2026-09-10 (pomodoro 185 + clipboard 127 全绿)
- [x] 进 review 阶段 ✅ 2026-09-10 (APPROVE + 2 Important: 删 clipboard windows 死依赖 + downscale_rgba 补非均匀插值测试; code-simplify 跳过: 代码已最简)

## Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| S2 chrono vs Win32 FFI 时区差异 | 显示偏差 | chrono 内部调同系 API, 实测一致 |
| S3 降采样精度 (f32 舍入) | 缩略图微差异 | 与原版逐像素一致 (原版已用 f32 + round) |
