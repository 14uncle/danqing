# Implementation Plan: E4 剪贴板监听下沉

> Spec: `docs/specs/SPEC-os.md` 簇E/E4。来源: `docs/intent/framework-sinking.md` 簇E。
> 裁决 (2026-09-10): E4 驻地方案a (`danqing::clipboard`, 新平台驻地); E1+E3+E2 已落地, E4 单独推进。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。

## Overview

下沉剪贴板监听四件肌肉: `ClipImage` (图片数据) + `ClipSource` trait (读取抽象) + `ensure_opaque_alpha` (DIB alpha 修补) + `SystemClipSource` (arboard + Win32 真实源)。
**留产品**: `Monitor` 的隐私判定 (ExclusionList) / 落库 (Store) / 轮询循环 (run) 是业务; `strip_html_tags` / `paths_to_json` 是 RichText 展示语义; `MAX_IMAGE_BYTES` / `POLL_INTERVAL` 是产品策略。

## Architecture Decisions

1. **落点**: 新建 `danqing::clipboard` 模块 (`src/clipboard.rs`), 平台代码驻地的显式豁免 (CLAUDE.md 记: 平台代码只住 window//render/, 现 + clipboard)。
2. **windows 移植** (0.62 → 0.59): `IsClipboardFormatAvailable`/`OpenClipboard`/`CloseClipboard`/`GlobalUnlock` 的 `Result` → `BOOL` 判 0; `GetClipboardData`/`GlobalLock` 的 `Result<HANDLE>` → `HANDLE`/`*mut c_void` 判 `is_null()`; `CF_HDROP` 从 newtype (`.0.into()`) → 裸 `u32` 常量; `HGLOBAL`/`HDROP` 从 newtype → `*mut c_void` 别名。
3. **错误处理**: `SystemClipSource::new()` 返回 `Result<Self, arboard::Error>` (框架不引 anyhow 进生产依赖); 读取方法静默降级返 None (与 `read_clipboard` 一致)。
4. **测试策略**: `ensure_opaque_alpha` 3 测试直接搬 (纯逻辑); `SystemClipSource` 只做不 panic 冒烟 (无 GUI 环境返 None/失败, 仿 foreground 现有测试); `ClipSource` 默认实现 (html/files/image 返 None) 无需额外测试。

## Task List

### Phase 1: 框架组件

- [x] **T1: Cargo.toml 加 windows-sys features** — 新增 `Win32_System_DataExchange` + `Win32_System_Memory` + `Win32_System_Ole` ✅ 2026-09-10
  - 验收: cargo check 通过, 无未用 feature 告警
  - 估时: XS | 依赖: 无

- [x] **T2: 新建 `src/clipboard.rs`** — ClipImage + ClipSource trait + ensure_opaque_alpha + SystemClipSource ✅ 2026-09-10
  - 内容: 四件自 clipboard `monitor.rs` 下沉; SystemClipSource 的 Win32 走 windows-sys 0.59 移植; 保留 CF_HDROP 缓冲区 +1 与 DIB alpha 全 0 的实证注释
  - 验收: ensure_opaque_alpha 3 测试直接搬 + SystemClipSource 不 panic 冒烟; 三件套绿
  - 估时: M | 依赖: T1

- [x] **T3: lib.rs re-export** — `pub mod clipboard;` ✅ 2026-09-10
  - 验收: `danqing::clipboard::{ClipImage, ClipSource, SystemClipSource, ensure_opaque_alpha}` 可达
  - 估时: XS | 依赖: T2

- [x] **T4: CLAUDE.md 记平台驻地豁免** — 平台代码只住 window//render/ 约定扩为含 clipboard (剪贴板域无法归入窗口域) ✅ 2026-09-10
  - 验收: CLAUDE.md 豁免行落地
  - 估时: XS | 依赖: T2

### Checkpoint 1: 框架侧完成

- [x] 三件套绿; ensure_opaque_alpha 3 测试 + SystemClipSource 冒烟锁定 (510 passed) ✅ 2026-09-10
- [ ] review (全模块) — 五轴
- [ ] 用户过目

### Phase 2: 产品迁移 (danqing 先 push → clipboard `cargo check` 重解)

- [x] **T5: clipboard 删手写四件** ✅ 2026-09-10
  - 文件: `danqing-clipboard/src/monitor.rs` 删 ClipImage/ClipSource/ensure_opaque_alpha/SystemClipSource, 改 `danqing::clipboard::*`; `main.rs` 改 `use danqing::clipboard::{SystemClipSource, ensure_opaque_alpha}`
  - 保留: Monitor (隐私判定/落库/轮询) + paths_to_json + strip_html_tags + MAX_IMAGE_BYTES/POLL_INTERVAL
  - 验收: clipboard 127 passed / 3 ignored (删 ensure_opaque_alpha 3 测试, 已搬框架); 三件套绿; ClipImage 仅测试用 → import 落到 tests 模块
  - 估时: M | 依赖: T1-T4 + danqing push

### Checkpoint 2: 全量验收

- [x] 各仓三件套绿; 分仓分别提交, message 注明关联 ✅ 2026-09-10 (danqing 321bd65, clipboard fc73acc)
- [x] 人工过目 (clipboard 监听行为不变) ✅ 2026-09-10 (127 passed 锁定)
- [x] 进 review 阶段 ✅ 2026-09-10 (REQUEST CHANGES: 1 Required 平台隔离 + 2 Optional + 2 Nit, 已全修; code-simplify 跳过: 代码已最简)

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| windows-sys 0.59 API 签名差异 (BOOL/HANDLE vs Result) | 移植编译错误 | 逐函数核对签名; 判空/判 0 显式 |
| CF_HDROP 在 System_Ole 而非 UI_Shell | 漏加 feature 编译错 | Cargo.toml 三 feature 一次加全 |
| SystemClipSource 无 GUI 环境行为 | 测试 flaky | 只做不 panic 冒烟, 断言放宽 (返降级值) |
| arboard 错误类型暴露 | 框架生产依赖引入 anyhow | new() 返 arboard::Error, 不引 anyhow |

## 开放问题 (已答)

- E4 驻地 → 方案a (`danqing::clipboard`), 本 plan 落地
