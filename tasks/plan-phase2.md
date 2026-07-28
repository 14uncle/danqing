# Implementation Plan: 丹青阶段 2 — 专注陪伴 POC(番茄钟 × 场景沉浸)

> 依据 `docs/specs/phase2-pomodoro-poc.md` 细化而来。
> 本文档将阶段 2 拆分为 **7 个可验证任务**,按依赖顺序组织。

## Overview

落地首个 POC:最小番茄钟(固定 25/5、开始/暂停/重置、场景切换、大字倒计时),
同时把框架演进为场景沉浸形态——`ScenePalette`/`SceneTheme` 跨明暗主题、
背景管线多场景交叉淡化、程序化场景资产管线(图 + 调色板一并产出)。

## Architecture Decisions

- **框架层 / 产品层分离**: 场景化主题(`ScenePalette`/`SceneTheme`/`SceneSpec`)与多场景背景渲染进 `src/`(可复用、可测);番茄钟状态机与界面组装留在 `examples/pomodoro/`(POC 专属,不进公开 API)。
- **App → 渲染的场景通道**: `App` trait 增加默认方法 `fn background_frame(&self) -> Option<BackgroundFrame>`(默认 `None`,showcase 不受影响);window 每帧查询并驱动 `Context`/`BackgroundPipeline` 的场景选择与淡化进度,清屏色随场景流动。
- **App 每帧心跳**: `App` trait 增加默认方法 `fn tick(&mut self, ctx: &AnimationCtx)`(默认空实现),在 `RedrawRequested` 中 `sync` 之前调用——计时推进与淡化推进都挂在它上面。这是框架此前缺失的能力(此前只有 widget 有 `animate`)。
- **淡化双纹理绑定**: 背景 pass 的纹理 bind group layout 不变(纹理 + sampler),pipeline layout 改为 [uniform, tex_from, tex_to] 两个纹理槽,逐场景预建 bind group,每帧绑定 from/to 两组 + 淡化 uniform,shader `mix` 输出;淡化结束后 from==to 走同一绑定,无分支。
- **光晕烘焙进场景图**: 不做逐场景 glow 叠加层,场景 PNG 生成时直接把光晕与中央可读性晕影烘进去;噪声叠加层全局保留(复用 `assets/background/noise.png`)。
- **对比度护栏数据随调色板产出**: `ScenePalette` 携带 `backdrop_light`/`backdrop_dark`(场景最亮/最暗区域色,生成时产出),护栏测试用它们验证文字可读性。**对 spec 的一点修正**:4 个具体场景的护栏断言放在 example 测试里(调色板常量由脚本生成进 example),lib 里放护栏函数与合成调色板测试。
- **TitleBar 主题为已知限制**: TitleBar 在构建时取主题快照,本 POC 不随场景流动(倒计时、控件等经 bind 闭包每帧读状态的元素会流动);如需流动另起任务。

## Dependency Graph

```
Task 1  颜色/动效纯逻辑 (Color::lerp, 对比度, Easing::eval, display 字号)
 └─ Task 2  ScenePalette + SceneTheme + SceneSpec
     └─ Task 4  场景生成管线 (export-scenes.py → PNG + scenes.rs)

Task 3  番茄钟状态机 + example 骨架 (独立; 率先验证 cargo test --example 机制)
Task 5  背景管线多场景 + 交叉淡化 + App 通道 (独立, render/window/app)

Task 6  POC 界面组装 (依赖 2/3/4/5; 含 App::tick 心跳)
 └─ Task 7  过渡动画 + 色调流动 + 4 场景对比度护栏 + 终验
```

关键路径:1 → 2 → 4 → 6 → 7。
并行车道:Task 3(状态机)∥ Task 5(渲染管线)∥ Task 1/2(主题)。

## Task List

### Phase 1: 框架纯逻辑 — 颜色与场景主题

- [ ] **Task 1: 颜色插值、对比度与动效求值纯逻辑**
  - **Description:** `layout.rs` 的 `Color` 增加 `lerp`(分量线性插值);`theme.rs` 增加 `relative_luminance`(输入视为 sRGB 编码,先解码为线性再加权,与 `from_srgb8` 语义一致)、`contrast_ratio`(WCAG 公式)、`composite_over`(半透明色合成到不透明底色);`Easing` 增加 `eval(t)`(Linear / EaseInOut cubic,clamp 0..1);`Theme` trait 增加 `font_size_display()`(默认 120,大字倒计时档)。
  - **Acceptance criteria:**
    - [ ] `Color::lerp` 端点与中点正确,t clamp 到 0..1。
    - [ ] 黑/白 luminance 为 0/1(误差 < 0.01),黑白对比度 ≈ 21:1。
    - [ ] `composite_over` 半透明合成结果正确(alpha=1 退化为顶层色)。
    - [ ] `Easing::eval` 端点恒等、单调、clamp。
    - [ ] `font_size_display()` ≥ `font_size_heading()`,LightTheme 编译不受影响。
  - **Verification:** `cargo test --lib theme` 与 `cargo test --lib layout` 绿;`cargo clippy -- -D warnings` 零警告。
  - **Dependencies:** None
  - **Files:** `src/layout.rs`, `src/theme.rs`
  - **Scope:** S

- [ ] **Task 2: ScenePalette + SceneTheme + SceneSpec**
  - **Description:** `theme.rs` 新增 `ScenePalette`(base 背景基调、accent、text_primary、text_secondary、surface、surface_input、backdrop_light、backdrop_dark)与 `SceneTheme`(实现 `Theme`:颜色 token 取自调色板,selection/caret 派生自 accent,divider/border 派生自文字色,字号/间距/圆角/阴影/动效沿用 LightTheme 档位);`ScenePalette::lerp`(过渡插值);`SceneSpec { name, image, palette }`(生成文件的目标形状)。全部经 `lib.rs` re-export。
  - **Acceptance criteria:**
    - [ ] `SceneTheme` 实现 `Theme` 且全部 token 合法(沿用 LightTheme 护栏风格:alpha 可见、字号/间距/圆角有序、玻璃区间)。
    - [ ] `ScenePalette::lerp` 端点恒等、中点插值正确。
    - [ ] 合成调色板的对比度护栏函数可对暗/亮两族给出正确断言(合成样例:深底白字 ≥3:1,浅底深字 ≥3:1)。
    - [ ] 公开类型经 `src/lib.rs` re-export,中文文档注释齐全。
  - **Verification:** `cargo test --lib theme` 绿;`cargo clippy -- -D warnings` 零警告。
  - **Dependencies:** Task 1
  - **Files:** `src/theme.rs`, `src/lib.rs`
  - **Scope:** M

### ⏸ Checkpoint 1: 框架 token 就绪
- [ ] `cargo test --lib --tests` 全绿
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] 人工确认 ScenePalette 字段集(生成管线的输入契约)后进入 Phase 2

### Phase 2: 状态机 / 资产管线 / 渲染管线(可并行)

- [ ] **Task 3: 番茄钟状态机 + example 骨架**
  - **Description:** 新建 `examples/pomodoro/` 目录型 example:`main.rs` 骨架(App impl,TitleBar + 占位文本,可开窗口);`timer.rs` 纯逻辑状态机,TDD 先行——`Phase { Focus(25:00), Break(5:00) }` × `Idle/Running/Paused`,时间由外部注入(`Duration` 累计值,非 wall-clock):`start/pause/resume/reset/tick/remaining/display`;阶段结束自动流转并自动开始下一阶段;重置回专注 25:00 停止态;`display()` 输出 `mm:ss`。
  - **Acceptance criteria:**
    - [ ] 开始/暂停/恢复/重置语义正确,暂停期间不计时。
    - [ ] tick 越过终点自动 Focus→Break、Break→Focus 并继续计时(余量不累亏)。
    - [ ] `display()` 格式 `mm:ss`(如 `24:59`、`05:00`)。
    - [ ] **`cargo test --example pomodoro` 确实运行这些测试**(机制验证是本任务的隐藏验收点;若机制不成立,调整测试落点并在 todo 备注)。
    - [ ] `cargo run --example pomodoro` 能打开带 TitleBar 的窗口。
  - **Verification:** `cargo test --example pomodoro` 绿;`cargo clippy -- -D warnings` 零警告。
  - **Dependencies:** None
  - **Files:** `examples/pomodoro/main.rs`, `examples/pomodoro/timer.rs`
  - **Scope:** M

- [ ] **Task 4: 场景生成管线 export-scenes.py**
  - **Description:** 演进 `tools/export-background.py` 为 `tools/export-scenes.py`:4 个场景配置(篝火-暗暖、海-亮青、雨-灰蓝、山-中性),每场景程序化生成多 stop 竖向渐变 + 烘焙径向光晕 + 中央可读性晕影,输出 `assets/scenes/{bonfire,sea,rain,mountain}.png`;同时产出 `examples/pomodoro/scenes.rs`——`SCENES: [SceneSpec; 4]` 常量(名称、图路径、完整 `ScenePalette` 含 backdrop 极端色),文件头注明"勿手改"。噪声复用现有 `assets/background/noise.png`。
  - **Acceptance criteria:**
    - [ ] `python tools/export-scenes.py` 一次跑通,4 张 PNG 落盘且非平凡(尺寸 ≥1024,非纯色)。
    - [ ] `scenes.rs` 常量形状与 lib 的 `SceneSpec`/`ScenePalette` 一致(Task 6 编译验证)。
    - [ ] 调色板人工过目:篝火暗、海亮、雨灰、山中性,明暗两族分明。
    - [ ] 资产提交 `assets/scenes/`,零外部素材。
  - **Verification:** 脚本运行成功;人工查看 4 张 PNG 观感。
  - **Dependencies:** Task 2(ScenePalette/SceneSpec 形状)
  - **Files:** `tools/export-scenes.py`, `assets/scenes/*.png`, `examples/pomodoro/scenes.rs`
  - **Scope:** M

- [ ] **Task 5: 背景管线多场景 + 交叉淡化 + App 通道**
  - **Description:** `BackgroundConfig` 增加 `with_scenes(Vec<PathBuf>)`(与既有 `image` 路径并存,showcase 不受影响);`render/background.rs` 预加载全部场景纹理,pipeline layout 改 [uniform, tex_from, tex_to],shader 双采样 `mix`,淡化 uniform;新增 `BackgroundFrame { from, to, fade, clear_color }`(lib re-export);`App` trait 增加默认方法 `background_frame()`(默认 `None`);`window.rs` 在 `RedrawRequested` 查询并写入 `Context`,`Context::render` 传给背景 pass;清屏色随 `BackgroundFrame` 流动。
  - **Acceptance criteria:**
    - [ ] 配置 scenes 后 `has_background()` 为真;未配置时行为与现状完全一致(showcase 回归)。
    - [ ] `background_frame()` 为 `None` 时走既有单图路径。
    - [ ] fade=0 显示 from 场景,fade=1 显示 to 场景(代码审查 + Task 6/7 人工目验)。
    - [ ] 纯逻辑部分(配置链式构造、fade clamp)有单元测试。
    - [ ] `DANQING_WGPU_VALIDATION=1` 下无校验错误(Task 6 首次运行 POC 时验证)。
  - **Verification:** `cargo test --lib` 绿;`cargo run --example showcase` 人工回归(渐变背景与毛玻璃不变)。
  - **Dependencies:** None
  - **Files:** `src/render/background.rs`, `src/render/background.wgsl`, `src/render/mod.rs`, `src/app.rs`, `src/window.rs`, `src/lib.rs`
  - **Scope:** M

### ⏸ Checkpoint 2: 状态机 / 资产 / 管线就绪
- [ ] `cargo test --lib --tests` + `cargo test --example pomodoro` 全绿
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] showcase 人工回归无异常
- [ ] 人工 review 4 张场景图后进入 Phase 3

### Phase 3: POC 组装与终验

- [ ] **Task 6: POC 界面组装(计时全功能 + 场景即时切换)**
  - **Description:** `App` trait 增加默认方法 `tick(&AnimationCtx)`(window 在 `sync` 前调用);`examples/pomodoro/main.rs` 完整组装——状态 `{ timer, scene_index, fader }`,Msg `{ StartPause, Reset, PrevScene, NextScene }`;视图:Column[TitleBar, 中央大字倒计时(display 档,bind 文本与颜色)+ 阶段/场景名小字, 底部玻璃胶囊控件条(开始/暂停 bind 标签、重置、场景 ◀/▶)];`background_frame()` 输出当前场景(fade 先恒 1.0,即时切换);文字/控件颜色经 bind 每帧取当前 `SceneTheme`;`WindowConfig` 960×640、`with_scenes` + 噪声。
  - **Acceptance criteria:**
    - [ ] 倒计时每秒跳动,25:00 起计;开始/暂停/重置/自动流转全部可用。
    - [ ] 场景 ◀/▶ 切换背景图与全部颜色 token(即时切换版)。
    - [ ] `scenes.rs` 被实际消费,编译通过。
    - [ ] `DANQING_WGPU_VALIDATION=1 cargo run --example pomodoro` 无校验错误。
    - [ ] `cargo test --lib --tests` + `cargo test --example pomodoro` + clippy 全绿。
  - **Verification:** 上述命令 + 人工运行 POC 走完 开始→暂停→重置→切场景 全流程。
  - **Dependencies:** Task 2, 3, 4, 5
  - **Files:** `src/app.rs`, `src/window.rs`, `examples/pomodoro/main.rs`, `examples/pomodoro/scenes.rs`(消费方)
  - **Scope:** M

- [ ] **Task 7: 过渡动画、色调流动与终验**
  - **Description:** example 内实现 `SceneFader`(from/to/起始时间/时长 ~800ms,`tick` 推进,`Easing::eval` 缓动):切场景时 `background_frame()` 输出 from/to/fade,清屏色同步插值;当前有效调色板 = `ScenePalette::lerp(from, to, eased_t)`,全部 bind 颜色随之流动;在 example 测试中对 4 个真实场景做对比度护栏(倒计时文字 vs backdrop 两极端 ≥3:1;控件文字 vs surface 合成色 ≥4:1);按 spec Success Criteria 逐条终验。
  - **Acceptance criteria:**
    - [ ] 切场景有 600~1000ms 交叉淡化,淡化期间颜色平滑流动,结束后稳定。
    - [ ] 4 场景对比度护栏测试全绿(明暗两族)。
    - [ ] spec Success Criteria 7 条逐条核对通过。
    - [ ] `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test --lib --tests` + `cargo test --example pomodoro` 全绿。
  - **Verification:** 上述命令 + 人工视觉验收清单(四场景观感、淡化动画、明暗可读性、控件悬停、计时准确)。
  - **Dependencies:** Task 6
  - **Files:** `examples/pomodoro/main.rs`, `examples/pomodoro/timer.rs`(如需), `tasks/todo-phase2.md`
  - **Scope:** M

### ✅ Checkpoint Complete: 阶段 2 POC 关闭
- [ ] spec Success Criteria 7/7 通过
- [ ] 全部 Commands 绿;`tasks/todo-phase2.md` 全部勾选
- [ ] 人工终审(重点:场景沉浸观感是否成立)

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| `cargo test --example` 不运行 example 内测试 | 计时状态机失去测试护栏 | Task 3 首件事就是验证机制;不成立则把 timer 挪入 lib 或 tests/ 并记录决策 |
| WGSL 双纹理改动破坏背景 pass | 背景黑屏/校验错误 | showcase 单图路径保留回归;Task 6 用 `DANQING_WGPU_VALIDATION=1` 验证;fade 端点有代码审查锚点 |
| 程序化场景观感不达标("像渐变壁纸") | POC 美学验证失败 | Task 4 脚本参数集中可调;Checkpoint 2 人工 review 图;中央晕影 + 光晕 + 噪声三层保底 |
| TitleBar 颜色不随场景流动 | 暗场景下标题栏违和 | 已知限制(构建时快照);若终审不可接受,另起任务给 TitleBar 加 bind |
| 大字号(120px)图集分配失败/模糊 | 倒计时渲染异常 | Task 6 人工目验;数字+冒号仅 11 个字形,图集压力小 |

## Open Questions

1. `cargo test --example pomodoro` 机制(Task 3 验证,失败有备用落点)。
2. 场景美术的具体参数(渐变 stop、光晕位置)——Task 4 出图后人工定夺,可迭代。
3. TitleBar 场景流动是否进 POC——默认不进,终审时确认。
