# Intent: 第六件产品 —— 磁盘空间分析器 (Windows)

> 2026-09-11 选型流程：用户点选候选 C（磁盘空间分析器）→ 痛点验证通过 → 闸门1通过（¥19/$2.99 买断）→ 深度调研完成。
> 需求驱动方针第二次实战，见用户级记忆 `product-selection-demand-first`。

- @author 十四叔
- @date 2026/09/11
- 状态: **plan 已批准，待 /build**（2026-09-11）
- Spec 产物: [disk-analyzer-capability-map.md](../spec/disk-analyzer-capability-map.md) + [SPEC-scanner.md](../spec/SPEC-scanner.md) + [SPEC-visualizer.md](../spec/SPEC-visualizer.md) + [SPEC-analyzer.md](../spec/SPEC-analyzer.md) + [SPEC-ui.md](../spec/SPEC-ui.md) + [SPEC-pro.md](../spec/SPEC-pro.md)
- Plan 产物: [plan.md](../tasks/plan.md) + [todo.md](../tasks/todo.md) — 22 tasks, 5 phases

## 一句话

Windows 上的「MFT 极速 + 现代 UI + 开源 + 智能清理建议」磁盘空间分析器——三大竞品各有一个致命短板，我们同时满足全部条件且价格最低。

## 已确认的意图

- **结局 (Outcome)**：秒开 TB 级磁盘扫描、treemap + 饼图 + 扩展名分布可视化、SSD 性能悬崖警告、开发者缓存识别、智能清理建议的桌面工具，开源免费版 + Pro 付费版
- **用户 (User)**：开发者 / 系统管理员 / 普通 Windows 用户——造物主本人是 SSD 用户+开发者（在场质检满格）
- **为什么是现在 (Why now)**：danqing-tile 放一放腾出带宽；市场空白明确（WizTree 闭源+无深色模式，WinDirStat 极慢+UI 过时，TreeSize 付费墙+已转订阅）；现代需求（SSD 悬崖/开发者缓存/WSL2 VHDX）未被满足
- **成功 (Success)**：POC 阶段 = 速度对比可截图碾压 WinDirStat + 功能对比碾压 WizTree；发布阶段 = 首单外检
- **约束 (Constraint)**：单人带宽；长在 danqing 上（wgpu treemap 渲染）；买断制否决订阅；MFT 解析需管理员权限
- **不做 (Out of scope)**：磁盘清理执行（只建议不执行）、文件恢复、磁盘碎片整理、网络 NAS 远程扫描、订阅制

## 调研证据摘要（深潜 2026-09-11）

### 竞品格局

| 工具 | 价格 | 速度 | UI | 核心短板 |
|------|------|------|-----|----------|
| **WizTree** | 个人免费，商业 $25+ | ⚡ MFT 极快 | Win32 风格 | 闭源 + 非NTFS退化 + 无深色模式 |
| **WinDirStat** | 免费开源 GPL | 🐢 极慢（分钟级） | XP 时代 | 速度 + UI + 危险操作无确认 |
| **TreeSize** | Free 阉割 / Pro 20.40 EUR/年订阅 | 中等 | 中等 | 付费墙严重 + 已转订阅制 |
| **SpaceSniffer** | 免费 | 中等 | 独特交互 | **已废弃 8+ 年** |
| **FolderSizes** | $30 个人 / $60 Pro 买断 | 快 | 企业风 | 价格偏高 |

### 四大核心卖点（铁证）

| 卖点 | 证据强度 | 核心数据 |
|------|----------|----------|
| **开发者缓存膨胀** | **极强** | 15条GitHub Issue：Rust target/ 62-79GB, node_modules 99GB, Docker VHDX 395GB, Xcode DerivedData 143GB, Codex checkpoint 102GB |
| **虚拟磁盘容器** | **极强** | WSL#4699 **1418票**（7年未解决）; Docker #244 **221票**（10年未解决） |
| **SSD性能断崖** | **强** | Tom's Hardware: QLC缓存耗尽写入降85-95%; 厂商建议保持<80%; Intel白皮书: >75%占用SLC缓存显著缩小 |
| **智能清理建议** | 中 | 方向正确（"is it safe to delete X" 高频搜索），需产品内验证 |

### 竞品用户痛点（GitHub Issues 铁证）

**WinDirStat**：
- #106 暗色模式 **29反应（全仓库最高）**——7年未实现
- #151 速度问题 37评论——"3分钟 vs WizTree 2秒"
- #604 删除操作无确认 27评论——用户删坏系统引导文件
- #50 OneDrive 扫描触发文件下载 17评论

**WizTree**：
- 闭源隐私顾虑——每个 "alternative" 讨论必提
- 非NTFS退化——USB/exFAT/NAS 失去核心卖点
- 商业环境需付费——IT管理员转向 TreeSize Free

**TreeSize**：
- Free版功能阉割严重——无网络盘/无重复检测/无导出/无CLI
- 2026年7月永久许可证政策变更——Ars Technica 报道，HN 热议

### 用户流失路径

```
WinDirStat 用户 ──(慢+过时)──→ WizTree（最常见）
                  ──(慢+过时)──→ TreeSize
                  ──(想要更好可视化)──→ SpaceSniffer

WizTree 用户 ──(闭源/隐私)──→ WinDirStat
              ──(非NTFS退化)──→ TreeSize Pro
              ──(商业付费)──→ ncdu/gdu

TreeSize 用户 ──(付费墙/推销)──→ WizTree
               ──(订阅政策变更)──→ WinDirStat 或 WizTree
```

## 产品定位

**一句话**：开源 + MFT极速 + 现代UI + 免费网络盘 + 云文件正确处理 + ¥19买断

**现有三个竞品各有一个致命短板，我们同时满足全部条件。**

| 条件 | WizTree | WinDirStat | TreeSize | **我们** |
|------|---------|------------|----------|----------|
| MFT 极速 | ✅ | ❌ | ✅ | ✅ |
| 现代 UI + 深色模式 | ❌ | ❌ | 中等 | ✅ |
| 开源 | ❌ | ✅ | ❌ | ✅ |
| 免费网络盘 | ❌ | ✅ | ❌(Free) | ✅ |
| 云文件正确处理 | ? | ❌(触发下载) | ? | ✅ |
| 买断制 | ✅ | N/A | ❌(已转订阅) | ✅ |

## MVP 边界

**In**：
1. MFT 直读极速扫描（管理员权限）+ 标准遍历降级（非管理员/非NTFS）
2. Treemap + 饼图 + 扩展名分布可视化
3. 深色/浅色主题
4. 文件夹大小排序 + 最大文件 Top N
5. SSD 性能悬崖警告（>85% 占用时弹窗提醒）
6. 开发者缓存识别（node_modules, .cargo/target, .git, DerivedData, __pycache__）
7. OneDrive/Dropbox 占位符正确识别（不触发下载）
8. 扫描结果导出 CSV/JSON

**Out**：
- 磁盘清理执行（只建议不执行，用户自行操作）
- 文件恢复
- 磁盘碎片整理
- 网络 NAS 远程扫描
- 重复文件检测（Pro 版）
- 历史趋势对比（Pro 版）
- WSL2 VHDX 内部分析（Pro 版）
- CLI 命令行（Pro 版）

## 开枪前提（POC 入口判据）

1. MFT 直读扫描速度对 WinDirStat 形成**可截图的碾压**（目标：1TB 磁盘 < 5秒）
2. Treemap 渲染性能达标（100万文件节点流畅交互）
3. 非管理员模式 + ReFS 降级扫描功能完整
4. 发布后首单外检

## 定价锚与渠道

| 版本 | 内容 | 价格 |
|------|------|------|
| **开源免费版** | MFT扫描 + treemap + 深色模式 + 基础分析 + SSD警告 + 开发者缓存识别 | $0 / GitHub |
| **Pro 版** | 历史趋势对比 + 重复文件检测 + WSL2 VHDX分析 + CLI + 导出PDF/Excel | ¥19 / $2.99 买断 |

**渠道**：
- GitHub 主（开源免费版引流 + Pro 版转化）
- MS Store 辅（搜索「disk space analyzer」「磁盘清理」流量大）

**冷启动**：
- Show HN 打「开源版 WizTree + 现代 UI + 深色模式」
- r/software、r/Windows10、r/techsupport
- WizTree/WinDirStat 相关老帖下接客

## 产品线一致性

| 产品 | 定价 | 授权 |
|------|------|------|
| pomodoro | ¥18 | 买断 |
| log (LogLens) | $45 / $95 | 买断 |
| **磁盘分析器** | **¥19 / $2.99** | **买断** |

¥19 与 pomodoro ¥18 同档位，形成产品线统一价格锚点。

## 技术风险

| 风险 | 严重性 | 缓解 |
|------|--------|------|
| MFT 解析需管理员权限 | 中 | 强制提权 + 非管理员降级为目录遍历 |
| ReFS 无 MFT 可读 | 中 | 降级为标准扫描，功能完整 |
| WizTree 速度壁垒 | 高 | 聚焦功能差异化而非速度竞赛 |
| 云文件占位符识别 | 中 | 调用 Windows API 检测 FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS |
| treemap 100万节点渲染性能 | 高 | wgpu 实例化渲染 + 层级LOD |

## 与框架的复用关系

| 引擎 | 复用场景 |
|------|----------|
| **danqing（UI 框架）** | treemap 渲染（wgpu）、深色主题、窗口管理、打包发布流程 |
| **danqing-encoding** | 文件名编码检测（Windows 非 Unicode 路径） |
| **danqing-logfile** | 不直接复用（日志特化），但 mmap 模式可参考 |

## 候选队列（未选，归档备查）

- **A「danqing-tile 窗口平铺管理器」**：已启动但放一放。Wayland/Sway 生态成熟但 Windows 侧 PowerToys FancyZones 免费+微软背书，差异化空间窄。
- **B「Hazel for Windows 文件自动整理」**：候选头名，等 log 发布后用户重启。
- 枪毙归档：泛截图工具（红海）、AI 听写（空窗已关）、快速记录（入口被掐）、抓包（团队赛道）、数字健康阻断器（MSIX 风险）。
