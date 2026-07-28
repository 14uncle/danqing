# 丹青 番茄钟 POC 补完 — 实施封档记录

> 阶段 2 已交付"最小番茄钟 + 场景沉浸美学" (见 `docs/specs/phase2-pomodoro-poc.md`), 但作为可日常使用的工具尚缺 5 处关键闭环。
> 本次的补完把番茄钟从"美学验证 POC"升级为"作者可日常挂在任务栏的工具"。

## 5 个真缺修复对应

| # | 缺口 | 修复方案 |
|---|------|----------|
| 1 | 完成反馈缺失 | `flash::FlashOverlay` 全屏 accent 脉冲 + `audio::MessageBeep` 蜂鸣;阶段流转触发 |
| 2 | 无手动跳阶段 | `Pomodoro::skip` + Skip 按钮;Running/Paused/Idle 三态语义 |
| 3 | 暂停视觉不明显 | `Color::desaturate` + `ScenePalette::desaturate` 整体降饱和 + 倒计时切 `text_secondary` |
| 4 | 无 OS 级控制 | `windows-sys` `RegisterHotKey` 独立线程 + `WindowEventSender` 通道;`Ctrl+Shift+P/S/Q` 默认热键 |
| 5 | 关闭后状态丢失 | `PomodoroState` JSON 持久化 + Running 跨重启 wall-clock 偏移 + 1Hz 节流 + 退出 flush |

## 关键架构决定

### 持久化 (Task 1)
- **JSON 写盘到 OS 标准配置目录**: `dirs::config_dir() + danqing/pomodoro.json` (Windows 解析为 `%APPDATA%/danqing/pomodoro.json`)
- **1Hz 节流 + 退出 flush**: 25 分钟量级下 1s 误差无感, 避免 60fps 重复 IO
- **Running 状态跨重启**: `effective_now = saved_elapsed + (current_wall - saved_wall)`, `start = Instant::now() - effective_now` 在 `Handler::resumed` 应用
- **依赖**: `serde` + `serde_json` + `dirs` (dev-deps 因仅 example 使用)

### 跳阶段 (Task 2)
- `Pomodoro::skip(now)` 纯逻辑方法, 三态各自动作
- Skip 按钮放底部胶囊 主按钮与重置之间, ghost 样式

### 暂停视觉 (Task 3)
- `Color::desaturate(factor)`: 向 RGB 均值线性插值, clamp 0..1, alpha 保留
- `ScenePalette::desaturate(factor)`: 逐字段调用, 用于暂停时整体降饱和
- `bind_alpha` 在 `Text` 上不存在, 降级为 `bind_color` 切换 `text_secondary`
- 全部走 `palette()` 的 `bind_color` 自动跟随降饱和;标题栏、控件、控件文字同步灰度

### OS 级全局热键 (Task 5)
- **新加 `Stack` widget**: 多子组件层叠, 后添加者绘制在上层 (flash 落地)
- **独立线程 + 消息钩入**: `RegisterHotKey` 配合 `HWND = NULL` 把热键关联到当前线程消息队列, `GetMessageW` 循环监听 `WM_HOTKEY`, 通过 mpsc 通道发到主线程
- **主线程 `about_to_wait` 钩子**: winit ApplicationHandler 提供的"待命"回调, 轮询热键 + 窗口事件通道
- **`App::hotkey(id)` 默认方法**: 应用决定热键 ID -> Msg 的映射, 框架不硬编码
- **`WindowEventSender` 通道**: App 主动控制窗口的官方通路 (显隐 / 退出), 不直接持有 Window
- **关闭 = 隐藏**: `CloseRequested` 改为 `set_visible(false)`, 退出由 `Ctrl+Shift+Q` 显式触发
- **Mac/Linux stub**: `hotkeys::spawn()` 返回 `None`, 日志一行 "global hotkeys unsupported", 不影响应用运行

### 完成反馈 (Task 4)
- `FlashOverlay` 纯逻辑进度状态机, 1.0 -> 0.0 线性衰减, 进行中触发被忽略
- `MessageBeep(MB_ICONASTERISK)` 零资产方案, 未来可替换为 WAV
- 全屏 flash 通过 `Stack` widget 叠加在 root 上

## 验收标准 (已全部通过)

- [x] 启动后场景循环淡化 / 计时显示 / 控件交互全功能
- [x] 关闭后 5 秒暂停, 重启状态恢复 (剩余时间误差 ≤ 1s)
- [x] Running 状态跨重启: deadline 按 wall-clock 偏移正确
- [x] 手动跳过阶段: 三态语义正确, 单元测试覆盖
- [x] 暂停视觉: 全 UI 降饱和, 倒计时切 `text_secondary`, 立即回满
- [x] 阶段结束: ~600ms 全屏 accent 脉冲 + Windows 蜂鸣
- [x] 全局热键: `Ctrl+Shift+P` 显隐跨应用聚焦生效, `Ctrl+Shift+S` 开始/暂停, `Ctrl+Shift+Q` 退出
- [x] 关闭按钮 = 隐藏到任务栏, 进程不退出
- [x] 配置文件不存在 / 解析失败 / 路径不可用均不 panic, 降级内存模式
- [x] `cargo fmt --check` + `cargo clippy -- -D warnings` 零警告
- [x] `cargo test --lib --tests` 全绿 (270+ 测试)
- [x] `cargo test --example pomodoro` 全绿 (47 测试, 含 9 个新测试)
- [x] `cargo build --release` 绿

## 已关闭开放问题

1. **`WM_HOTKEY` 在 winit 0.30 中的接收机制** — 解决: 不拦截 winit 事件循环, 走独立线程 + 消息队列, 主线程 `about_to_wait` 轮询
2. **`Text::bind_alpha` 是否存在** — 否, 降级 `bind_color` 切 `text_secondary`
3. **关闭按钮 tooltip** — 当前未加, 用户体验可观察
4. **APPDATA vs LOCALAPPDATA** — 选 APPDATA (漫游)
5. **WAV vs MessageBeep** — 选 MessageBeep 零资产, 留替换接口

## 不在本计划范围

- 任务列表 / 长休息 / 统计 / 设置页 / 自定义时长 / 多 profile / 托盘图标
- 全局热键 Mac/Linux 完整实现
- 持久化的多设备同步

## 下一步

番茄钟 POC 封档。下一个 POC 候选: 剪贴板历史管理器 (效率工具族, 美学剂量低于专注陪伴族), 详见 `docs/ideas/danqing-efficiency-tool-glassmorphism.md`。**未获用户指示时不要启动新 POC。**
