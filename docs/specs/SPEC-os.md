# SPEC: OS 级肌肉链下沉 —— 前台窗口 / 剪贴板

> 来源: `docs/intent/framework-sinking.md` 簇E。
> 形态: 分阶下沉。E1/E2 落 `window/foreground` (现平台驻地); E4 需新平台驻地 (开放问题)。
> 共同摩擦: clipboard 用 `windows` 0.62, danqing 用 `windows-sys` 0.59 —— E1/E2/E4 需移植。E1/E2 随 `foreground` 模块级 `#[cfg(windows)]` 门控 (与 record/restore/simulate_paste 一致), 非 Windows 上符号整体不存在, 无需单独 stub。

## 产品一句话

把 clipboard 日用实战验证的 OS 肌肉 (前台窗口定位 / 焦点等待 / 剪贴板监听) 下沉进 danqing,补齐 `simulate_paste` 文档配方缺的最后一环,给启动器/专注工具/下一件产品备料。

## 能力地图

| # | 能力 | 职责 | 来源 | 成本 | 现状 |
|---|------|------|------|------|------|
| E1 | `wait_for_focus_leave` | 轮询等待前台窗口离开本进程 (隐藏后焦点异步转移, 固定延时的真实 bug 捶打出来) | clipboard `foreground.rs:75-100` | **S** | danqing 无, 是 simulate_paste 配方缺的一环 |
| E2 | `foreground_process_name` | 获取前台进程名 (隐私三件套比对排除名单) | clipboard `foreground.rs:12-53` | S-M | danqing 零 |
| E3 | `to_crlf` 文本归一 | 裸 LF → CRLF (Windows 剪贴板惯例, 旧版记事本换行丢失) | clipboard `inject.rs:15-20` | **S** | 3 测试纯逻辑, 零依赖 |
| E4 | 剪贴板监听 (`ClipSource`/`SystemClipSource`/`ensure_opaque_alpha`) | 序列号门控 + CF_HDROP 文件读取 + DIB alpha 修补 | clipboard `monitor.rs:37-164,58-64` | M-L | 20+ 测试全场最厚; 需新平台驻地 |
| E5 | 粘贴注入编排配方 | 防重入 + 写剪贴板 + 隐藏 + 等焦点离开 + 恢复 + Ctrl+V | clipboard `main.rs:295-387` | M | 依赖 E1; 编排含产品状态, 不全下沉 |

**已下沉 (勿再提议)**: `record_foreground` / `restore_foreground` / `simulate_paste` 已在 `danqing::foreground` (window/foreground.rs, 2026-08-01 AttachThreadInput 方案)。

## 为什么

- **E1 是「粘贴注入配方」的最后一块**: danqing 已有 `simulate_paste` + `restore_foreground`, 但「隐藏窗口后等焦点自然回落」这一步还留在产品侧手写。固定延时不可靠 (过短 Ctrl+V 打自己窗口, 过长用户感知延迟) —— clipboard 用 10ms 轮询 + 超时降级解决了, 这正是该下沉的通用机制。
- **E2 复用面最宽**: 前台进程名是启动器/专注工具/隐私过滤的通用需求, danqing 零实现。
- **E3 是纯函数**: 3 测试、零依赖, 「任何写剪贴板文本的产品必遇」。
- **E4 是肌肉本体**: 序列号门控 (500ms 轮询不漏不重)、CF_HDROP 缓冲区 +1 (路径丢尾巴实证)、DIB alpha 全 0 修补 (整图隐形实证), 都是 bug 换来的内化知识, 20+ 测试全场最厚。

## 形态裁决

1. **E1/E2 落 `window/foreground.rs`** (现平台驻地, 无裁决): 前台窗口定位/焦点等待属窗口域, 与 `record_foreground`/`restore_foreground` 同居, 复用 windows-sys 0.59 的 `Win32_UI_WindowsAndMessaging` + `Win32_System_Threading` (均已声明, 无新增 feature)。
2. **E3 落 `text/`** (纯逻辑): `to_crlf` 是文本归一, 与 `text::fit` 同层; 零依赖无门控。
3. **E4 需新平台驻地** (开放问题 ①②): 剪贴板监听 (GetClipboardSequenceNumber/OpenClipboard/CF_HDROP) 属剪贴板域, 不属窗口域。现约定「平台代码只住 window/ 与 render/」需开新驻地 `danqing::clipboard`。

## API 设计

### E1 + E2 (`window/foreground.rs` 追加)

```rust
/// 轮询等待前台窗口不再是本进程 (焦点已转移), 最多等 timeout。
/// 隐藏窗口后焦点转移是异步的, 固定延时不可靠; 每 10ms 检测一次。
/// 超时返回 false (调用方仍应注入, 降级为旧行为)。无前台窗口视为已离开。
pub fn wait_for_focus_leave(timeout: std::time::Duration) -> bool;

/// 获取当前前台窗口的进程名 (如 "notepad.exe")。
/// 失败返回 None (无前台 / 权限不足 / API 失败)。不取本进程自己。
pub fn foreground_process_name() -> Option<String>;
```

移植 (windows 0.62 → windows-sys 0.59): `GetForegroundWindow() -> HWND` / `GetWindowThreadProcessId(hwnd, *mut u32) -> u32` / `OpenProcess(...) -> HANDLE` (0.59 返 HANDLE 非 Result, 判 is_null) / `QueryFullProcessImageNameW(...) -> BOOL` (判 0)。

### E3 (`text/mod.rs` 追加 `to_crlf`)

```rust
/// Windows 剪贴板文本惯例为 CRLF; 裸 LF 在部分应用换行丢失。
/// 已含 \r\n 的片段先坍缩再统一, 不重复加 \r (Ditto 同款)。
pub fn to_crlf(text: &str) -> String;
```

### E4 (`src/clipboard.rs` 新模块, 依赖开放问题裁决)

```rust
pub trait ClipSource {
    fn sequence(&mut self) -> u32;
    fn text(&mut self) -> Option<String>;
    fn html(&mut self) -> Option<String> { None }
    fn files(&mut self) -> Option<Vec<String>> { None }
    fn image(&mut self) -> Option<ClipImage> { None }
}

pub struct SystemClipSource { /* arboard + Win32 */ }

/// DIB 来源 RGBA 常带全 0 alpha → 全置 255; 存在非零则尊重原通道。
pub fn ensure_opaque_alpha(rgba: &mut [u8]);
```

**不下沉**: `Monitor` (监听器) 的隐私判定 (ExclusionList) / 落库 (Store) / 轮询循环 (run) 是产品业务; `strip_html_tags` / `paths_to_json` 是 RichText 展示语义, 暂留产品 (等第二消费者)。

## 依赖

- E1/E2: 无新增 (windows-sys `Win32_UI_WindowsAndMessaging` + `Win32_System_Threading` 已声明)。
- E3: 无 (纯 std)。
- E4: 新增 windows-sys features —— `Win32_System_DataExchange` (序列号/剪贴板句柄) + `Win32_System_Memory` (GlobalLock/Unlock) + CF_HDROP (Win32_System_Ole 或 Win32_UI_Shell)。arboard 已依赖。

## 验收标准

1. **单元测试直接搬** (行为保持):
   - E1: clipboard `foreground.rs` 现有测试不 panic (无 GUI 环境); 新增超时降级纯逻辑可测 (注入时间源? 或保留轮询只测不 panic)。
   - E2: 现有 `foreground_process_name_does_not_panic` 搬移。
   - E3: 3 测试直接搬, 零改动 (裸 LF / 已含 CRLF 各一, 空串 + 无换行 + 孤立 \r 折进 `edge_cases`)。
   - E4: `ensure_opaque_alpha` 3 测试 (全 0 置 255 / 尊重非零 / 空与奇数长度) 直接搬; `SystemClipSource::files` 的 CF_HDROP +1 教训随代码内化。
2. **三件套**: `cargo fmt` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib --tests` 全绿。
3. **产品迁移验证** (clipboard):
   - E1: `foreground.rs` 删手写 `wait_for_focus_leave`, 改 `danqing::foreground::wait_for_focus_leave`; `main.rs` paste_into_previous 相应简化。
   - E2: 删手写 `foreground_process_name`, 改框架。
   - E3: `inject.rs` 删手写 `to_crlf`, 改 `danqing::to_crlf`。
   - 既有测试零改动全绿 (行为保持)。

## Boundaries

- **Always**: 失败静默降级不 panic (前台/剪贴板 API 在无 GUI/权限不足时返回 None/降级); 非 Windows 上平台符号随模块级 cfg 门控整体不存在 (产品边界 Windows-only); 隐私判定与落库是产品业务, 框架只提供读取原语。
- **Ask first**: 剪贴板监听轮询间隔/隐私策略 (产品决策); 新增平台代码驻地 (开放问题)。
- **Never**: 向真实前台注入按键进默认测试 (SendInput/写剪贴板在自动化时打进用户前台, 已有 `#[ignore]` 封口); 猜剪贴板内容编码。

## 开放问题 (已裁决 2026-09-10)

1. **E4 新平台驻地 → 方案 (a)**: 新建 `danqing::clipboard` 模块 + CLAUDE.md 记豁免 (平台代码住 window//render/ + clipboard, 剪贴板域无法归入窗口域)。
2. **E 簇推进节奏 → E1+E3+E2 先**: 前台窗口域零摩擦一个 spec 收; E4 后续单独 (剪贴板域 + windows-sys 移植 + 新 feature)。
