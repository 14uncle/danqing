# 意图文档：Windows 自动平铺窗口管理器

> 由 interview-me 技能产出，2026-09-11 用户确认。
> 2026-09-11 深度调研更新：4 agent 并行深挖竞品 issues + 定价 + 需求验证。

## 核心意图

- **Outcome：** 一款 Windows 自动平铺窗口管理器——窗口打开即自动归位，用户无需手动拖拽或理解"平铺"概念。提供可视化 GUI 配置界面给想调布局的人。
- **User：** 普通多窗口 Windows 用户（开发者/设计师/分析师/办公族），不是 i3 式极客。
- **Why now：** FancyZones 自动平铺需求 6 年未满足（371+ 反应），所有开源方案都是 CLI/YAML 配置，macOS Sequoia 2024 已原生支持自动平铺拉高预期。"自动平铺 + 现代 GUI"在 Windows 上是空白。
- **Success：** 用户装上后"窗口自动排好"的体感；MS Store/GitHub 有自然下载；首单外检通过。
- **Constraint：** 必须长在 danqing 上（winit 窗口事件监听 + wgpu GUI 渲染）；单人带宽。
- **Out of scope：** Linux/macOS；i3 式手动平铺/power user 极客功能；窗口堆叠/tabbing（可后续迭代）；多显示器高级特性（可后续迭代）；Explorer 级别的完整窗口管理器替换。

## 产品定位一句话

> Windows 上第一个"装上就自动排好窗口"的工具——不需要配置、不需要学 YAML、不需要理解"平铺"。

## 差异化三角

| 维度 | FancyZones (PowerToys) | GlazeWM/Komorebi | 本产品 |
|------|----------------------|-----------------|--------|
| 自动平铺 | ❌ 需手动拖拽（6年未满足，371+ 反应） | ✅ 但 YAML 配置 | ✅ 开箱自动 |
| GUI 配置 | ✅ 但笨重 | ❌ CLI/YAML | ✅ 现代自绘 GUI |
| 目标用户 | 通用 | 极客 | 通用 |
| 学习成本 | 低 | 高 | 极低 |
| 便携免安装 | N/A | ❌ V3 要管理员 | ✅ portable |

## 必须实现的功能（入场券）

| 功能 | 理由 |
|------|------|
| 窗口打开自动归位 | 核心卖点 |
| 窗口关闭自动重排 | FancyZones #2694 用户最大痛点 |
| 应用排除列表 | Electron 对话框、游戏会被错误平铺 |
| 键盘驱动全部操作 | 用户明确不想用鼠标管理窗口 |
| 崩溃后窗口恢复 | "卸载级"痛点——丢未保存进度用户直接卸载 |
| 首次启动不重排已开窗口 | 新用户入门最大障碍 |
| 多显示器基础支持 | 多屏用户占比高 |
| Excel/Office 兼容性测试 | 最常用应用，ribbon 消失是不可用级 bug |

## 差异化功能（竞品都没做好）

| 功能 | 竞品现状 | 机会 |
|------|---------|------|
| 堆叠/标签页布局 | GlazeWM 最想要但没做（89+58+45 反应） | 头号差异化 |
| 可视化 GUI 配置 | 全部 CLI/YAML | danqing 结构性优势 |
| 便携免安装 | GlazeWM V3 要管理员 | 企业/受限环境蓝海 |
| 多显示器稳定性 | 休眠/热插拔必出问题 | 做好即赢 |
| 多种布局（Master/Dwindle/Grid） | GlazeWM 只有 BSP | 布局丰富度 |
| 鼠标拖拽移动/调整 | 纯键盘门槛高 | 降低入门门槛 |

## 技术路径

- 监听窗口事件：WinEventHook（窗口创建/销毁/移动）
- 布局引擎：窗口树 + BSP/Dwindle/Master/Columns 多种算法
- 状态持久化：每次变动写 JSON 到磁盘，崩溃后可恢复
- 排除机制：应用白名单/黑名单，通配符匹配窗口类名/标题
- GUI 配置：danqing wgpu 自绘，可视化拖拽编辑布局
- 系统托盘常驻

## 最大技术风险

1. 应用兼容性（Electron、Java、游戏）——需要大量测试 + 排除机制
2. 多显示器热插拔——需要监听显示器连接/断开事件
3. 与 Windows 原生 Snap 的共存——快捷键冲突需要可配置
4. PowerToys 团队说的 "much harder than it sounds"——应用对窗口布局有控制权

## 定价策略

| 维度 | 决策 |
|------|------|
| 定价模式 | 买断制（品类共识） |
| 价格 | $15-20 / ¥45-60（对标 AquaSnap $18） |
| 发行 | GitHub 免费版 + MS Store 付费版（freemium） |
| 付费版差异化 | 自定义布局、多显示器配置、快捷键链、配置同步 |
| 产品线阶梯 | pomodoro ¥18 → 窗口管理器 ¥45-60 → log $45/$95 |

## 调研数据来源

- 6 agent 并行初扫（桌面工具/开发者/数据处理/内容创作/系统监控/趋势预测）
- 4 agent 并行深挖（GlazeWM+Komorebi issues / FancyZones issues / 自动平铺需求验证 / 定价）
- 交叉验证 GitHub issues 反应数 + 用户原话 + 竞品现状

## 裁决（2026-09-15 用户指令）

**埋掉**：不立项、不推进；仓库 `danqing-tile/` 与代码/spec 产物原样保留作档案（同 Xirang / disk 处置）。代码完成（3207 行 / 37 commits / 63 测试全绿），**从未人工验收、从未发布**。

**依据（2026-09-15 体检，四 agent 查证）**：

1. **免费在位者封死（闸③直接命中）**：PowerToys/FancyZones 微软第一方免费（138,664★ / 单版本 581 万下载 / MS Store 521 评分级）；GlazeWM GPL-3.0 免费 12,769★；FancyWM MS Store 免费 4.5★；**Seelen UI（17,807★ / 350 万下载 / MS Store 免费 4.5★·687 评分 / 内置自动平铺 BSP·columns·stacks + 每应用规则 + floating 例外 / 周更）** —— 本卷对比三角中完全没有它。
2. **「FancyZones 明确不做」不成立**：canonical issue #2694 六年 384 反应**至今 open、从未标 not_planned**，2020 年官方原话是「auto-tile 不适合 FancyZones 架构，**应做成独立模块**」。是「从未做」，不是「承诺不做」，不可当护城河。
3. **需求衰减**：2019-2020 为峰值（#37 👍116 / #2694 👍384），此后逐年归零——2025 全年新请求最高 👍2，2026 至今最高 👍4、中位数 0。r/Windows11 平铺话题最高票（408）**是在夸系统自带功能**；Super User 2024-2026 相关提问全是「怎么关掉自动吸附」。
4. **目标人群错位**：本卷定位「面向普通用户、不需要理解平铺」；查证显示想要自动平铺的人**几乎全是先在 Linux 养成平铺习惯、被工作逼回 Windows 的人**，而 Windows 原生用户中「没用过 Linux 平铺却想要它」的证据为零。**「不用学 YAML」对唯一想要这功能的人群没有价值**——他们要的就是配置权。
5. **付费天花板实测**：品类头部 komorebi（15,176★ / 35 万下载）2025 全年总收入 **$12,070**，其中 $7,877 来自**商业使用许可（合规费）**，为体验付费 **$0**（作者靠 MDM 企业设备检测「逼」出付费，并写道「企业不会自愿为改善员工工作条件的软件掏钱」）；下载→付费转化 0.03–0.09%。本产品 v1 全免费，直接收入为 0。
6. **渠道实测挂科**：MS Store「tiling window manager」货架上真应用仅 1 个且免费；HN 头部发布（komorebi 229 分）来自同一几千人池子反复刷脸，头部产品付费用户总数 118 人。

**技术负债（结构性）**：Windows 不强制窗口契约，自动平铺要长期与 app 开发者打游击——komorebi 作者原话：「Applications behaving badly is a big problem for any window management project on Windows... especially since the rise of Electron, [developers] are increasingly throwing established Win32 application development guidelines to the wind.」这是「重投入」在本产品上的具体形状。

**未查证项**：Reddit 三站评论正文（网络阻断）；Google 搜索量硬数据。
