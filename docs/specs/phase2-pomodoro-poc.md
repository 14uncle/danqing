# Spec: 丹青阶段 2 — 专注陪伴 POC(番茄钟 × 场景沉浸)

> 对应战略文档:`docs/ideas/danqing-scene-immersion-pivot.md`(POC 范围已经采访确认)。
> 本 spec 待用户确认后进入 plan 阶段。

## Objective

落地丹青首个产品 POC:一个**最小番茄钟**,以此验证潮汐式场景沉浸美学——场景大图是界面主角、界面色调随场景流动、UI 退后极简。同时把阶段 1 的浅色单主题设计系统演进为**跨明暗的场景化设计系统**(每场景一套调色板,玻璃表面、文字、控件态在明暗两族下都成立)。

**用户故事:**

- 作为使用者,我打开番茄钟就看到一幅沉浸场景大图和中央大字倒计时,按一下开始即进入 25 分钟专注;结束自动流入 5 分钟休息,再自动流回专注。
- 作为使用者,我可以切换场景(篝火/海/雨/山/森林),画面在约 1 秒内交叉淡化过渡,文字与控件颜色随场景自然变化,暗场景(篝火)与亮场景(海)下都清晰可读。
- 作为维护者,场景资产全部由脚本程序化生成、零版权,每场景调色板随图一并产出;计时、调色等纯逻辑有单元测试护栏。

**明确排除(POC 不做):** 音频/声景、自定义时长、轮次统计、任务标签、摄影资产、实时全屏动态模糊。

## Tech Stack

- **语言**:Rust 2024 edition
- **窗口/事件**:`winit` 0.30;**自绘**:`wgpu` 30(均已就位)
- **字体**:沿用 `assets/fonts/` 现有字体(ZCOOL XiaoWei 回退),大字号呈现"细体"观感;若验证后观感不足,引入新字体属 Ask first
- **位图**:`image`(png)已在依赖中,无新增
- **场景生成**:Python + Pillow 脚本(演进 `tools/export-background.py`),产出 PNG + 调色板

## Commands

```bash
# 运行 POC(打开番茄钟窗口)
cargo run --example pomodoro

# 重新生成场景资产(改场景参数后)
python tools/export-scenes.py

# 纯逻辑测试(lib + 集成)
cargo test --lib --tests

# POC 内部纯逻辑测试(计时状态机等, 写在 example 的 #[cfg(test)] 中)
cargo test --example pomodoro

# 静态检查(必须零警告)
cargo clippy -- -D warnings

# 格式化
cargo fmt
```

提交门槛在原三件套(fmt + clippy + `cargo test --lib --tests`)之上**追加** `cargo test --example pomodoro`。

## Project Structure

```
src/
  theme.rs                  # 扩展: ScenePalette(场景调色板) + SceneTheme(由调色板构造的跨明暗 Theme 实现)
                            #        + display 级字号 token(大字倒计时)
  render/background.rs      # 扩展: 背景 pass 支持多场景纹理与交叉淡化(进度 uniform + easing)
  window.rs                 # 扩展: BackgroundConfig 场景化 API(按名选场景 / 设置淡化进度)
examples/
  pomodoro/
    main.rs                 # POC 应用: App impl + 组件树(场景 + 倒计时 + 控件条)
    timer.rs                # 番茄钟状态机(纯逻辑, 时间由外部注入)
    scenes.rs               # 由生成脚本产出的场景调色板常量(include! 进 main)
assets/
  scenes/                   # 程序化场景图, 提交版本控制
    bonfire.png             #   篝火(暗)
    sea.png                 #   海(亮)
    rain.png                #   雨(灰)
    mountain.png            #   山(中性)
    forest.png              #   森林(绿, 2026-07-23 用户增补)
tools/
  export-scenes.py          # 场景生成管线(演进自 export-background.py):
                            #   每场景产出 PNG + 调色板, 汇总写出 examples/pomodoro/scenes.rs
docs/
  specs/
    phase2-pomodoro-poc.md  # 本 spec
```

**职责划分:**

- **框架层(src/)**: 场景化主题(`ScenePalette`/`SceneTheme`)、多场景背景渲染与淡化——可复用、可单元测试的框架能力。
- **产品层(examples/pomodoro/)**: 番茄钟状态机与界面组装——POC 专属,不进框架公开 API。
- **资产管线(tools/ → assets/ → scenes.rs)**: 色调在生成时一并产出,不需要运行时取色算法。

## 功能规格

### 计时(最小番茄钟)

- 固定**专注 25:00 / 休息 5:00**,不接受自定义。
- 操作:**开始/暂停**(同一按钮两态)、**重置**。
- 阶段**自动流转**:专注结束自动进入休息并开始计时,休息结束自动回到专注并开始计时;**重置**回到专注 25:00 的停止态。
- 计时基于时间戳累计(外部注入 `now`),帧率波动不影响实际时长;倒计时显示 `mm:ss`。
- 状态机为纯逻辑:`Idle / Running / Paused` × `Focus / Break`,全部状态迁移有单元测试。

### 场景沉浸

- 场景大图**全屏 Cover** 铺满,是界面主角;无侧边栏、无页面导航。
- 5 个程序化场景:**篝火(暗)、海(亮)、雨(灰)、山(中性)、森林(绿,雾气针叶林)**,跨明暗两族。
- 场景切换(上一个/下一个)触发约 600~1000ms 的**交叉淡化**,缓动用 `easing_standard`;调色板 token 同步插值流动。
- 全局噪声叠加保留;光晕按场景可选(生成时配置)。
- 每场景调色板至少给出:背景基调(供 clear_color/fallback)、accent、文字主/次级色、玻璃表面色与透明度区间。

### 界面(UI 退后)

- **中央**:超大倒计时(display 级字号,约 96~144 逻辑像素档),其下小字标注当前阶段(专注/休息)与场景名。
- **底部居中**:一条半透明玻璃胶囊控件条——开始/暂停、重置、场景上一个/下一个;常显但低存在感(低不透明度),悬停增强。文字直接压在场景上时,以玻璃层保证可读性。
- **标题栏**:复用框架自绘 TitleBar(LOGO + 标题 + 窗控视觉)。
- 窗口默认约 960×640,可缩放,场景图随窗口 Cover 适配。

## Code Style

- 延续阶段 1:token 以 trait + 结构体表达,组件从 theme 读取,禁止魔法颜色/圆角/阴影值。
- `SceneTheme` 实现 `Theme` trait,与 `LightTheme` 平级;场景切换即换主题实例(过渡期内对两套 token 插值)。
- 纯逻辑(计时、调色、淡化进度、对比度计算)不依赖 `winit`/`wgpu`,按框架约定只出现在允许的位置;GPU 代码只在 `render/` 与 `window.rs`。
- 公开 API 经 `src/lib.rs` re-export,中文文档注释;新增 `.rs` 文件带 `@author 十四叔` 与 `@date` 头。
- 生成文件(`scenes.rs`)头部注明"由 tools/export-scenes.py 生成,勿手改"。

## Testing Strategy

- **lib 单元测试**(`src/theme.rs` 等):
  - `SceneTheme` 从调色板产出的 token 合法(alpha 区间、字号/间距/圆角有序等,沿用 LightTheme 既有护栏风格);
  - **对比度护栏**:对全部 5 个场景,大字倒计时文字 vs 场景背景极端色 ≥ 3:1(大字),控件文字 vs 玻璃表面 ≥ 4:1——明暗两族都成立,这是"跨明暗成立"的自动化证明;
  - 淡化进度/调色插值的纯逻辑(如落在 lib)。
- **example 单元测试**(`examples/pomodoro/timer.rs`,`cargo test --example pomodoro` 运行):
  - 开始/暂停/恢复/重置;25:00→5:00→25:00 自动流转;暂停期间不计时;时间戳注入,无 wall-clock 依赖。
- **人工视觉验证**(`cargo run --example pomodoro`):五场景观感、淡化动画、明暗下可读性、控件条悬停、计时准确性。
- **静态检查**:`cargo clippy -- -D warnings` 零警告;`cargo fmt --check` 通过。

## Boundaries

- **Always:**
  - 提交前:`cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` + `cargo test --example pomodoro` 全绿。
  - 场景资产一律由 `tools/export-scenes.py` 生成并提交 `assets/scenes/`,不引入任何外部版权素材。
  - 新增的框架级组件(若 POC 中沉淀出可复用组件)必须同时出现在 `examples/showcase.rs`。

- **Ask first:**
  - 新增外部依赖(如音频库、serde/图像新格式)。
  - 新增字体文件或改变字体回退策略。
  - 用摄影资产替换程序化场景。
  - 窗口行为变化:置顶、全局快捷键、托盘、自启动。
  - 改动 `examples/showcase.rs` 的结构(本阶段不动 showcase)。

- **Never:**
  - 任何形式的音频(无音频库依赖、无音频代码路径)。
  - 实时全屏动态模糊(静态预渲染路线不变)。
  - 自定义时长、轮次统计、任务标签(POC 排除项,留给后续)。
  - 在 `widget/`、`layout.rs`、`event.rs`、`text/` 引入 `winit`/`wgpu` 依赖。
  - 破坏既有组件(TextInput/TextArea 撤销重做、IME、焦点等)行为。

## Success Criteria

1. `cargo run --example pomodoro` 打开番茄钟:全屏场景大图 + 中央大字倒计时 + 底部玻璃控件条 + 自绘标题栏。
2. 计时正确:25:00 起计,暂停/恢复/重置行为符合规格,基于时间戳累计不受帧率影响。
3. 阶段自动流转:专注→休息→专注循环成立;重置回到专注 25:00 停止态。
4. 5 个场景可切换,切换有 600~1000ms 交叉淡化,文字/玻璃 token 随调色板流动;篝火(暗)与海(亮)两族下倒计时与控件均清晰可读。
5. 对比度护栏测试覆盖全部 5 场景(大字 ≥3:1、控件 ≥4:1),`cargo test --lib --tests` 与 `cargo test --example pomodoro` 全绿。
6. 场景资产 100% 程序化生成,`assets/scenes/` 与 `scenes.rs` 提交版本控制,无外部素材。
7. 无音频依赖与代码;`cargo clippy -- -D warnings` 零警告;`cargo fmt --check` 通过。

## Open Questions(默认已定,spec 评审时确认)

1. **场景选型**:采纳 pivot 文档建议的 4 个——篝火/海/雨/山;2026-07-23 用户增补第 5 个:森林(雾气针叶林,参照潮汐 App)。
2. **阶段流转**:自动连续流转(专注完自动开始休息),而非停下等用户确认——潮汐式"流动"观感的默认选择。
3. **"细体"实现**:先用现有字体大字号呈现;若人工验证观感不足,再申请引入细字重字体(Ask first)。
4. **控件条存在感**:常显低透明度 + 悬停增强(不做自动隐藏,POC 保持简单)。

## Related Documents

- `docs/ideas/danqing-scene-immersion-pivot.md` — 转向决策与 POC 范围采访确认
- `docs/specs/phase1-design-system.md` — 阶段 1 设计系统 spec(本 spec 的格式与 token 基础)
- `CLAUDE.md` — 项目约定与命令
