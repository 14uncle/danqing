# Implementation Plan: E5 粘贴注入编排配方下沉

> Spec: `docs/specs/SPEC-os.md` 簇E/E5。来源: `docs/intent/framework-sinking.md` 簇E。
> 裁决 (2026-09-10): E1+E3+E2+E4 已落地, E5 下沉「等焦点离开 → 恢复原前台 → Ctrl+V」三连组合函数; 防重入/写剪贴板/隐藏窗口留产品。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。

## Overview

下沉 `paste_into_previous(prev_foreground, timeout)` —— 把「隐藏窗口后注入粘贴」的三步顺序固化进框架。三步每步顺序都是 bug 换来的 (等焦点离开 / 恢复不杀 TSF / 注入), 组合成一个配方函数, 产品不再手写这三连。

## Architecture Decisions

1. **落点**: `window/foreground.rs` 追加, 与 `wait_for_focus_leave`/`restore_foreground`/`simulate_paste` 同居 (组合这三个已下沉函数)。
2. **签名**: `pub fn paste_into_previous(prev_foreground: Option<HWND>, timeout: std::time::Duration)` —— 无返回值, 各步失败静默降级 (与三子函数一致)。
3. **测试策略**: 无新测试 —— `simulate_paste` 注入真实 Ctrl+V (SendInput), 进默认测试打进用户前台 (违反「测试严禁真实桌面副作用」铁律); 三子函数已有各自验证, 组合无需额外测试。
4. **留产品**: 防重入 (`is_pasting`) / 找条目 / 写剪贴板 / 隐藏窗口 / 重置防重入。

## Task List

### Phase 1: 框架组件

- [x] **T1: `window/foreground.rs` 追加 `paste_into_previous`** ✅ 2026-09-10
  - 内容: 组合 wait_for_focus_leave → restore_foreground → simulate_paste 三连, 文档注释固化顺序与降级语义
  - 验收: 三件套绿 (无新测试)
  - 估时: XS | 依赖: 无 (E1 已落)

### Checkpoint 1: 框架侧完成

- [x] 三件套绿 ✅ 2026-09-10
- [x] 用户过目 ✅ 2026-09-10

### Phase 2: 产品迁移 (danqing 先 push → clipboard 重解)

- [x] **T2: clipboard 两处 paste 函数改用 `paste_into_previous`** ✅ 2026-09-10
  - 文件: `danqing-clipboard/src/main.rs` `paste_item` / `paste_item_plain_text` 末尾三连段 → `danqing::foreground::paste_into_previous(prev, PASTE_INJECT_TIMEOUT)`
  - 验收: clipboard 既有测试零改动全绿 (127); 三件套绿
  - 估时: XS | 依赖: T1 + danqing push

### Checkpoint 2: 全量验收

- [x] 各仓三件套绿; 分仓分别提交, message 注明关联 ✅ 2026-09-10
- [ ] 人工过目 (clipboard 粘贴注入行为不变)
- [ ] 进 review 阶段

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| prev 指针跨线程 move (usize→*mut c_void) | 类型不匹配编译错 | 调用处 `prev.map(\|h\| h as *mut std::ffi::c_void)` |
| simulate_paste 副作用进测试 | 注入真实按键 | 无新测试, 三子函数已各自验证 |
