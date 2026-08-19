# Memory Index

> 按任务类型分组;同一 memory 可能在多组中出现(交叉引用)。

## 场景/动效 (场景 shader、动效包络、纹理管理)

- [场景动效 UV 位移机制](scene-motion-uv-displacement.md) — 亮度调制读作"光在车上跑路没动",世界本身动须 UV 位移采样坐标;但对沿轴均匀元素(雨丝)不成立,须去烘焙+程序化;亮场景 additive 被近白底吃掉走乘性
- [场景纹理 2 槽 LRU](scene-lru-pattern.md) — 多场景池按 from/to 懒加载的 wgpu 纹理模式,danqing 2026-07-24 落地
- [rodio 0.22 repeat_infinite bug](rodio-022-repeat-infinite-bug.md) — symphonia 解码器循环秒空无声,须自实现 LoopingDecoder (ambient.rs)
- [shader step 门禁约定](shader-step-on-probability.md) — step(threshold,hash) 的 on 概率=1-threshold;比例须写 step(1-ratio,h);星野曾误写致实际 ~1280 颗、减量方向算反
- [AI 场景 UV 位移选择偏好](ai-scene-uv-displacement-preference.md) — UV 位移适合大幅运动(海浪/火焰),additive 适合小幅氛围(雾气);与静态元素重叠时走 additive
- [shader 常量重复定义检查](shader-duplicate-constant-check.md) — 修改 WGSL 时必须先 grep 同名常量,旧定义被替代须删除;shader 编译错误只在运行时暴露

## 性能/内存

- [wgpu 30 内存双杠杆](wgpu-30-memory-lever.md) — Backends::PRIMARY Windows 同时拉起 Vulkan+DX12、MemoryHints::Performance 默认留 slack,改 DX12 + MemoryUsage 砍 100+ MB
- [minidbg 符号保留偏好](minidbg-symbol-preference.md) — strip=debuginfo 而非 symbols,~1MB 换崩溃诊断可用性,体积优化不可牺牲
- [wgpu 实例预建无收益](wgpu-instance-prebuild-no-gain.md) — 后台线程预建 Instance 省下的时间 request_adapter 等额变贵,已撤回勿再试

## 视觉排障

- [丹青视觉排障工具链](danqing-visual-debug-tooling.md) — PrintWindow +1 行偏移与非当前虚拟桌面抓旧帧、click-post.ps1 点击注入首选、ps1 须纯 ASCII、sRGB 线性混合、输入时序类 bug 须物理复现取证(合成 Alt+Tab 不可靠)

## 构建/工具链

- [windows-gnu 工具链 binutils 修复](windows-gnu-toolchain-lld-fix.md) — windows-gnu + 真 binutils(MSYS2 dlltool),lld shim 走不通(部分 IAT 填充不全)

## 窗口/平台

- [winit 0.30 抢前台受前台锁](winit-030-focus-window-foreground-lock.md) — focus_window 合成 Alt 对后台进程静默失败,须 AttachThreadInput 绕锁 (foreground.rs)
- [winit 0.30 窗口图标两档](winit-030-window-icon-two-tier.md) — with_window_icon 只设 ICON_SMALL,任务栏须 with_taskbar_icon 补 ICON_BIG,否则偶发缺省图标
- [Poll 空转致环境音呲啦](poll-control-flow-audio-crackling.md) — 隐藏态用 ControlFlow::Poll 致 tick 数千 fps,hammer rodio player → buffer underrun;统一 WaitUntil(16ms)

## 流程/规范

- [双 JetBrains IDE MCP 去重](jetbrains-dual-mcp-dedup.md) — idea/rustrover 双份 schema 白占 ~30k;deniedMcpServers 非托管设置不生效,只能 claude mcp remove

- [丹青资产目录约定](danqing-assets-directory-convention.md) — 字体、LOGO、背景图统一放 assets/ 并提交,build.rs 不再生成资产
- [AI 场景底图不加暗纱](ai-scene-no-veil.md) — AI 生成的场景底图禁用 veil,保留自然亮度;contrast guard 失败可接受
- [AI 场景升级工作流](ai-scene-upgrade-workflow.md) — 严格8步:prompt→生图→复制→去水印→更新ai_base→export-scenes.py→shader适配→测试;export-scenes.py会覆盖图片须指向独立源文件;pomodoro禁用noise叠加层
- [Rust 文件头注释规则](danqing-rs-header.md) — 新建 .rs 文件须加 @author 十四叔 与 @date yyyy/MM/dd
- [省略号 ASCII 三点约定](ellipsis-ascii-dots-convention.md) — 结尾省略号用 "..." (Text 组件拆分渲染底边对齐), U+2026 居中只用于中间截断; 占位文字勿加 offset 错位, 字体指标须实测勿估算
- [文档目录约定](danqing-document-locations.md) — spec 放 docs/specs，plan/todo 放 tasks
- [提交前重新验证](verify-immediately-before-commit.md) — IDE 自动保存可能在验证与提交之间改脏文件,门槛要紧贴 commit
- [提交前必评审](review-before-commit.md) — git commit 前必须执行一遍 /agent-skills:code-review-and-quality 五轴评审
- [截图流程与路径](screenshot-rules.md) — 先问用户要截图;自行截图放 target/tmp,不放根目录
- [新 widget 必须加 showcase demo](widget-showcase-demo-rule.md) — 新增 widget 组件必须在 showcase.rs 添加演示卡片，人工验证视觉和交互

## 项目状态/战略

- [丹青项目状态](danqing-project-state.md) — M1~M3 + 阶段 1/2 + 打磨三件套 + 五场景动效 全检关闭;2026-08-01 战略升级著作型旗舰, 里程碑 0 完成 + 里程碑 1 Task E 年度报告完成(07657e6)+ Box 布局 bug 修复(548322c), 剪贴板顺延
- [丹青战略定位](danqing-strategic-positioning-efficiency-tools.md) — 专注陪伴工具+效率工具两族、潮汐式场景沉浸美学、首个 POC 番茄钟已关闭
- [丹青旗舰十年战略](danqing-flagship-strategy.md) — 2026-08-01 确认著作型旗舰(专注陪伴系统×十年建造史);2026-08-10 pomodoro 全部免费发布(练手),付费部分废弃;变现留给下一个产品
