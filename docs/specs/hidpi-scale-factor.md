# Spec: HiDPI 缩放支持（scale factor 贯通框架）

- 日期: 2026-09-21
- 状态: **已完成**（2026-09-22；S1–S5 全满足，CP1/CP2 达成，T6 联动 lock 复钉 danqing#d8133ea1）
- 触发: LogLens 在 3200×2000 @ 200% 缩放屏上全 UI 等比缩小约 50%（用户上报，截图存 Desktop\danqing\）

## Objective

**现象**：高 DPI 屏（scale factor > 1.0）上，danqing 应用窗口物理尺寸正确，但全部内容（文字/行高/间距/控件）按 1/s 缩小。200% 屏上缩小一半。

**根因**（调试已闭环）：框架全链路按物理像素工作，`scale_factor` 从未应用于渲染路径 —— 全框架仅 `handler.rs:1410` 一处用于隐藏态尺寸换算；`ScaleFactorChanged` 事件未处理。属记录在案的已知遗留（`memory/font-cjk-mono.md`：「未处理，v1.x 候选」），因 LogLens 上架商店、真实高分屏用户踩中而从遗留升级为缺陷。

**目标**：任意 scale factor（1.25 / 1.5 / 1.75 / 2.0 及更高）下，框架渲染的视觉尺寸与 100% 基准等比一致，文字清晰度不劣于现状，**产品侧零改动**。

**用户**：高分屏 Windows 用户。现代笔电 3K/4K 屏 + 150%/200% 系统是主流配置，LogLens 已上架 MS Store 与 GitHub，每个高分屏下载者都会踩中。

## 设计决策（审查点；任一可否决）

- **D1 逻辑坐标语义**：框架内部全部坐标与字号统一为**逻辑像素**（= 100% 缩放下的像素）。现有所有数字（theme token、widget 内常量、产品代码里的尺寸）语义不变 —— 这是「产品零改动」的根基。
- **D2 scale 只落在两个边界**：
  - *渲染边界*：各 pass 的 `screen_size` uniform 改喂逻辑视口（rect/image/background pass 的 shader 里 `px/screen_size` 除法中 s 自然约掉，shader 零改动）；`TextBatch` 持有 scale —— 文字按 `px×s` 物理栅格化 + 物理像素格吸附落点（保住现有文字清晰度方案，见 memory/font-cjk-mono），`measure/line_height/ascent/descent` 返回逻辑值，clip 矩形换算。
  - *输入边界*：光标坐标 ÷s、滚轮 PixelDelta ÷s、IME 光标区 ×s（`handler.rs:630`），均在 `window/` 域完成。
  - 布局视口 = 物理尺寸 ÷ s。
- **D3 含 ScaleFactorChanged 动态切换**（多屏拖拽 / 运行中改系统缩放）：更新 s → 清字形图集 → 按新逻辑视口重排。不修此条等于修一半（拖到另一块屏还是错的）。
- **D4 scissor 矩形**：若 RectBatch/TextBatch 的裁剪走 `set_scissor_rect`（物理像素域），render 提交时 ×s。plan 阶段逐处核查确认。
- **D5 背景场景 shader 豁免**：若背景/场景 shader 内有按物理 frag coord 计算的程序纹理（颗粒密度等），高分屏下视觉密度变细属**可接受行为变化**，本次不调整。plan 阶段核查是否存在此类用法并记录。
- **D6 s=1.0 回归锁**：s=1.0 时行为与改造前逐位一致；全部现有测试**零修改**通过即为证。

## Tech Stack

现有栈，无新依赖：Rust + winit 0.30 + wgpu 30 + fontdue。

## Commands

```bash
# 全部测试（纯逻辑，无需 GPU）
cargo test --lib --tests
# 静态检查（必须零警告）
cargo clippy -- -D warnings
cargo fmt
# 人工视觉验收
cargo run --example danqing-showcase
# 单个测试
cargo test <name> -- --exact
```

## 涉及模块

| 文件 | 改动 |
|------|------|
| `src/window/handler.rs` | scale 跟踪、布局视口换算（物理÷s）、`ScaleFactorChanged` 处理、IME 光标区 ×s、隐藏态尺寸路径核查 |
| `src/window/event.rs` | 光标坐标 ÷s、滚轮 PixelDelta ÷s |
| `src/render/mod.rs` | `screen_size` uniform 喂逻辑视口、scissor ×s（若存在） |
| `src/render/text.rs` | `TextBatch` scale 化：px×s 物理栅格化、物理格吸附落点、测量返回逻辑值、clip 换算、scale 变更清图集 |
| `src/text/atlas.rs` | 栅格 px 粒度支持 fractional scale（u16 → f32 或取整策略，plan 定）；图集容量评估（R1） |
| `src/widget/`、`src/layout.rs`、`src/event.rs`、`src/text/` 其余 | **零改动**（纯逻辑层纪律） |

## Code Style

沿用 `danqing/CLAUDE.md` 约定：中文文档注释、新文件头 `//! @author 十四叔` + `//! @date`、公开 API 一律经 `src/lib.rs` re-export、纯逻辑层不碰 winit/wgpu。

## Testing Strategy

**纯逻辑单测（无 GPU，必须）**：
- `TextBatch` 在 s=2.0 / 1.5 下：栅格化 px、落点物理格吸附、`measure`/`line_height` 返回逻辑值正确
- 事件换算：物理光标坐标 → 逻辑坐标（含 fractional 取整策略）
- 视口换算：物理尺寸 ÷ s = 布局视口
- **s=1.0 回归锁**：全部现有测试零修改通过

**人工视觉验收（必须，文字渲染质量是主观判定）**：
- 本机 1080p 屏 Windows 缩放调 200% → `danqing-showcase` 与 100% 基准截图逐区域等比，文字清晰度不劣化
- 125% / 150% 各抽验一次（布局不破、文字不糊）
- 运行中改系统缩放 = S3 动态切换测试（与上一步同一操作覆盖）
- 验收完毕恢复 100%

**高分屏实机**（3200×2000 @ 200%）最终确认：随 LogLens 下一版发布验证，不阻塞本 spec 关闭。

## Boundaries

- **Always**：提交前三件套（fmt + clippy -D warnings + 测试全绿）；s=1.0 回归锁不破；纯逻辑层零改动纪律；产品零改动目标
- **Ask first**：新增依赖；改动 widget/text 纯逻辑层公共 API 语义；图集容量策略变更（扩容/驱逐）
- **Never**：产品侧为适应框架而改任何数字；引入 hinting 或更换栅格器（memory/font-cjk-mono 明令）；采用「DPI 不感知 + 系统位图拉伸」的偷懒方案（文字全糊，本框架的清晰字方案作废）

## Success Criteria

- **S1** 200% 缩放下 showcase 全组件与 100% 基准等比（截图对比 + 文字清晰度人工判定）
- **S2** 125% / 150% 抽验无破版（布局不错位、文字不糊）
- **S3** ScaleFactorChanged 动态切换正确（运行中改系统缩放后 UI 等比跟随、无残留错位）
- **S4** 全部现有测试零修改通过 + 新增 scale 相关单测绿
- **S5** 产品侧（danqing-log）仅 `cargo update -p danqing` 即获得修复，零代码改动（联动改动另行提交，发布节奏用户另定）

## Risks / Open Questions

- **R1 图集容量**：1024² 图集在 s=2.0 下字形面积 ×4，LogLens 长会话大字号档位是否触发「图集已满跳过字形」（日志扳机「栅格化失败」）—— plan 阶段评估（选项：扩容 2048² / LRU 驱逐 / 按 scale 分档）
- **R2 fractional scale 栅格粒度**：atlas 现按 u16 px 键控，fontdue 原生接受 f32 —— plan 定（f32 直传 vs 就近取整，取整方向影响行高一致性）
- **R3 placement 持久化语义**：产品保存/恢复的窗口位置尺寸跨 DPI 屏行为 —— 现状不变（物理域），验收发现错位再立 follow-up
- **R4 背景 shader 物理像素程序纹理核查**：是否存在按 frag coord 计算的效果（D5 豁免是否真有对象）
