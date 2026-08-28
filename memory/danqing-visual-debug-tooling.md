---
name: danqing-visual-debug-tooling
description: 丹青视觉排障工具链的实测结论与坑 (PrintWindow +1 行偏移、DWM 截屏伪影、线性混合、探针标定法)
metadata: 
  node_type: memory
  type: reference
  originSessionId: 8434501f-a77a-4974-afda-2d8ce51b81a8
  modified: 2026-07-29T06:46:06.699Z
---

丹青渲染排障实测结论 (2026-07-23, 细线描边根因排查沉淀; 同日番茄钟 POC 脚本化验收增补):

- `tools/print-window.ps1` (PrintWindow API) 可在窗口被遮挡/位于其他虚拟桌面时抓到窗口内容, 是默认验证手段; 但**截图存在 +1 像素行偏移假象** (水平细线显示位置比几何低一行), 分析像素时须排除, 不要误判为渲染 bug。单次调用耗时 ~0.7s, 想抓"点击后 N 毫秒"的瞬态帧要减去这个延迟。
- `tools/capture-screen.ps1` 全屏截图只在 showcase 与当前虚拟桌面一致时有效; SetForegroundWindow 无法跨虚拟桌面前置窗口。
- **PrintWindow 对"位于非当前虚拟桌面且经历过分辨率变化 (如最大化)"的 wgpu 窗口会抓到陈旧画面**: 返回旧尺寸的白色矩形贴在新尺寸黑底上 (2026-07-23 run9/run10 实测, 同代码 run8 窗口在当前桌面时抓图完美)。这是 DWM/DirectComposition 不为不可见桌面重组窗口视觉所致, 不是渲染 bug——判据: 应用日志 resize 已处理、render::Context::resize 有 debug 级 "surface 重建" 日志 (INFO 级别不可见, 别把"只有启动时一条 surface 已配置"误判为没重建)。验证最大化渲染要么让用户肉眼看, 要么先把窗口切回当前虚拟桌面再抓。
- 全屏截图/DWM 合成会产生"鬼影"(其他窗口残影叠在截图里), PrintWindow 直抓的像素里没有——判渲染 bug 前先排除截屏伪影。
- **点击注入: 优先 `tools/click-post.ps1`** (PostMessage WM_MOUSEMOVE/LBUTTONDOWN/UP 直投窗口队列, 不动物理鼠标, 用户在机操作也不干扰); `tools/click-at.ps1` (SetCursorPos+mouse_event) 与物理鼠标实时抢光标, 仅作备用。两个脚本都必须用 ClientToScreen 取真实客户区原点——自绘无边框窗口没有 +8/+31 标准边框偏移; mouse_event 只发 DOWN 不发 UP 会让逻辑按键卡死。
- **PowerShell .ps1 文件在 GBK 系统上只能写 ASCII**: 无 BOM 的 UTF-8 中文注释会被按 ANSI 误读, 多字节字符吃掉换行后变量"不存在"等诡异报错。
- wgpu surface 为 sRGB 格式时**混合在线性空间进行**: 半透明黑线 (如 border alpha 0.18) 叠白底呈现 ~233 灰而非直觉的 209; 估像素覆盖率必须先做 sRGB→linear 转换再算 alpha。
- 细线/像素级根因标定法: 在 window.rs paint 后临时 push 全宽彩色探针线 (整数 + 不同小数位 y), 配合 `DANQING_DUMP_RECTS` 式实例几何转储与着色器 alpha=1 实验, 可把"几何/光栅化/SDF 覆盖率"三层问题分离定位。
- 细线描边"贴合 vs 发虚"两难终局方案 (2026-07-23): `Rect::snap_to_pixels()` 四舍五入对齐像素格, 表面组件 (Box/TextInput/TextArea) 的**填充与描边共用同一份对齐几何**——轮廓精确重合 (贴合, 偏移 ≤0.5px) 且 1px 细线落完整像素行 (满强度)。单向 ceil/floor 内缩会露填充底色边 (卡片不贴合), 完全不对齐则细线覆盖率拆两行 (底边发虚), 两条路都走过且都失败; SDF 过渡带收窄 (d164349, w ≤ 半尺寸) 保留。护栏测试: `rounded_border_aligns_to_nearest_pixel_grid` (rect.rs)、`paint_snaps_fill_and_border_to_same_pixel_grid` (box_.rs)。项目状态见 [[danqing-project-state]]。
- **输入时序类 bug (如 Alt+Tab 泄漏按键) 合成输入复现不可靠** (2026-07-29): keybd_event/SendKeys 发的 Alt+Tab 与真人指法规程不同——真人"先松 Alt 后松 Tab"时 Windows 会把迟发 Tab 排在 `Focused(true)` **之前**投递给激活窗口, 合成序列要么不泄漏、要么连切换器都不提交。此类排查的取证手段 = 事件日志埋点 (info 级键盘 / OS 焦点 / 焦点变化, 已在 handler.rs 常驻) + 请用户物理复现一轮, 不要在合成时序上空转。Windows 侧结论 (winit 失焦即合成修饰键释放致 Alt 状态不可信、判据只能是 has_os_focus 时序) 已写在 `src/window/handler.rs` 注释与 `tab_traverse_allowed` 护栏测试里。
- **跨进程 SetWindowPos 在本机 Claude bash→PowerShell 链路返回 TRUE 但不生效** (2026-08-28 位置记忆验收实测): 同一交互桌面下 (读类 API GetWindowRect/PrintWindow 全通), 对 notepad 与 showcase 的 SetWindowPos 均返回真却零位移, GetLastError 只剩无意义残留——疑似 harness 的 Job 对象 UI 限制, 根因未查实。**对策: 验收需要"窗口被挪动"时优先走引擎自有路径 (位置记忆 = 改落点文件再重启; 拖拽 = 留给用户物理过), 别在外部挪窗脚本上空耗**; winit `Moved`/set_outer_position 坐标约定含 DWM 不可见边距, 与 GetWindowRect 读数自洽但与"肉眼期望坐标"差 ~18px, 跨约定手填坐标值会误判为漂移。
