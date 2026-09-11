# Implementation Plan: OS 级肌肉链下沉 (E1+E3+E2)

> Spec: `docs/specs/SPEC-os.md`。来源: `docs/intent/framework-sinking.md` 簇E。
> 裁决 (2026-09-10): E4 驻地方案a (danqing::clipboard, 后续单独); 本轮 E1+E3+E2 先 (前台窗口域 + 纯逻辑)。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。

## Overview

三件下沉: E3 `to_crlf` (text 纯逻辑, 零依赖) + E1 `wait_for_focus_leave` + E2 `foreground_process_name` (window/foreground 平台域)。
E1/E2 需 `windows` 0.62 → `windows-sys` 0.59 移植。

## Architecture Decisions

1. **落点**: E3 落 `text/crlf.rs` (纯逻辑, 与 selection/encoding/fit 同层); E1/E2 落 `window/foreground.rs` (现平台驻地, 与 record/restore/simulate_paste 同居)。
2. **windows 移植** (0.62 → 0.59): `OpenProcess`/`QueryFullProcessImageNameW` 等 `Result` 返回 → `HANDLE`/`BOOL` 判空判 0; `GetWindowThreadProcessId` 的 `Option<&mut u32>` → `*mut u32` 裸指针 out 参数。
3. **非 Windows 门控**: E1/E2 随 `foreground` 模块级 `#[cfg(target_os = "windows")]` 门控 (与 record/restore/simulate_paste 一致), 非 Windows 上符号整体不存在, 无需单独 stub/降级。
4. **测试策略**: E3 3 测试直接搬 (纯逻辑); E1/E2 平台 API 依赖真实前台窗口, 只做「不 panic」冒烟 + 超时降级确定性测试 (`Duration::ZERO`) (仿 window/foreground 现有测试)。

## Task List

### Phase 1: 框架组件

- [x] **T1: E3 `to_crlf` 下沉** — `danqing/src/text/crlf.rs` (新建) + `text/mod.rs` + `lib.rs` ✅ 2026-09-10
  - 内容: `to_crlf` (裸 LF → CRLF, 已含 \r\n 先坍缩不重复加 \r)
  - 验收: 搬 clipboard `inject.rs` 3 测试零改动全绿; 三件套绿
  - 估时: XS | 依赖: 无

- [x] **T2: E1 `wait_for_focus_leave` 下沉** — `danqing/src/window/foreground.rs` (追加) ✅ 2026-09-10
  - 内容: 轮询等待前台窗口离开本进程 (10ms 间隔, 超时降级); windows-sys 0.59 移植 (随 foreground 模块级 cfg 门控)
  - 验收: 不 panic 冒烟测试 (无 GUI 环境返 true 或超时); 三件套绿
  - 估时: S | 依赖: 无

- [x] **T3: E2 `foreground_process_name` 下沉** — `danqing/src/window/foreground.rs` (追加) ✅ 2026-09-10
  - 内容: 获取前台进程名 (OpenProcess + QueryFullProcessImageNameW); windows-sys 0.59 移植 (随 foreground 模块级 cfg 门控)
  - 验收: 不 panic 冒烟测试 (无 GUI 环境返 None); 三件套绿
  - 估时: S-M | 依赖: 无

- [x] **T4: re-export 接线** — `danqing/src/lib.rs` (确认 foreground 已导出; text 加 to_crlf) ✅ 2026-09-10
  - 验收: `cargo check` 通过, `danqing::foreground::wait_for_focus_leave` / `danqing::to_crlf` 可达
  - 估时: XS | 依赖: T1-T3

### Checkpoint 1: 框架侧完成

- [x] 三件套绿; to_crlf 3 测试 + E1/E2 冒烟测试锁定 (505 passed) ✅ 2026-09-10
- [x] review (全模块) — 五轴 ✅ 2026-09-10 (Approve, 无 Critical/Required; 补超时降级测试 + 文档对齐)
- [x] 用户过目 ✅ 2026-09-10

### Phase 2: 产品迁移 (danqing 先 push → clipboard `cargo check` 重解 lock)

- [x] **T5: clipboard 删手写三函数** ✅ 2026-09-10
  - 文件: `danqing-clipboard/src/foreground.rs` 整个模块删除 (手写 wait_for_focus_leave / foreground_process_name + record/restore 纯 pass-through 薄委托一并清) + `inject.rs` (删 to_crlf) + `main.rs` 调用处改 `danqing::foreground::*` / `danqing::to_crlf`
  - 验收: clipboard 130 passed / 3 ignored (删 4 测试: 3 to_crlf + 1 foreground 冒烟, 已搬框架); 三件套绿; Cargo.lock 无变化 (patch path 覆盖, 发布时去 patch 才钉 rev)
  - 估时: S | 依赖: T1-T4 + danqing push

### Checkpoint 2: 全量验收

- [x] 各仓三件套绿; 分仓分别提交, message 注明关联 ✅ 2026-09-10 (danqing fee8860/99897a3, clipboard 9a4ca33/a3f1345)
- [x] 人工过目 (clipboard 粘贴注入行为不变) ✅ 2026-09-10 (130 passed 锁定)
- [x] 进 review 阶段 ✅ 2026-09-10 (Approve, code-simplify 判断跳过: 代码已最简)

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| windows-sys 0.59 API 签名差异 (HANDLE/BOOL vs Result) | 移植编译错误 | 逐函数核对签名; 判空/判 0 显式 |
| E1/E2 平台 API 无 GUI 环境行为 | 测试 flaky | 只做不 panic 冒烟, 断言放宽 (返降级值) |
| 非 Windows 编译 | 产品 Windows-only 但框架跨平台 | foreground 模块级 `#[cfg(windows)]` 门控, 非 Windows 符号整体不存在 |

## 开放问题 (已答)

- E4 驻地 → 方案a (danqing::clipboard), 后续单独
- 节奏 → E1+E3+E2 先
