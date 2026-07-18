# Implementation Plan: 丹青 (danqing) M1 — 最小闭环

> 依据 `docs/spec.md`(已批准,2026-07-16)与 `docs/plan.md` 细化而来。
> 本文档将 plan.md 的 11 个步骤重排为 **14 个可验证任务**,按垂直切片组织:
> 每个任务交付一条"从代码到像素/行为"的完整路径,而非水平分层。
>
> **前置条件已满足**:Rust 工具链已安装(cargo 1.97.0 ≥ MSRV 1.85),Step 0 不再阻塞。

## Overview

构建 Rust 跨平台自绘 UI 框架的 M1:打通"winit 事件 → 保留模式组件树 → wgpu 像素"完整链路。
最终交付 `cargo run --example showcase`,展示彩色/圆角矩形、中英文文本、可点击按钮计数、键盘移动方块。

showcase 从 Task 2 起就是**唯一且持续生长的演示程序** —— 每个任务把当步能力加进 showcase(以用代测,spec Boundaries 要求),不建一次性示例文件。

## Architecture Decisions

- **基础值类型前置**:`Color` / `Point` / `Size` / `Rect` / `Edges` 是纯数据结构,放在 `src/layout.rs`(spec 结构中最接近的纯逻辑模块),于 Task 1 随脚手架定义。渲染管线(Task 4/6)与布局算法(Task 7)都依赖它们 —— 前置定义使 Phase 2 与布局车道可并行。
- **依赖方向只允许向下**:`widget/`、`layout.rs`、`event.rs` 为纯逻辑,不依赖 wgpu/winit;`render/*` 可引用 layout 的值类型,反向禁止(spec 约束)。
- **showcase 即验收**:M1 没有独立 GUI 测试,showcase 是唯一渲染验证手段;每个渲染任务完成后人工跑一次,要求 debug 构建的 wgpu 校验层零错误。
- **单 crate**:按 spec,`danqing` 单 crate,`Cargo.toml` 设 `rust-version = "1.85"`、edition 2024。
- **持续渲染**:每帧 `request_redraw`(游戏式),按需渲染留 M2(docs/plan.md 已定)。

## Dependency Graph

```
Task 1  脚手架 + 基础值类型 (Cargo.toml, lib.rs, layout.rs 值类型)
 ├─ Task 2  开窗 (window.rs)
 │    └─ Task 3  wgpu 上下文 (render/mod.rs)
 │         ├─ Task 4  矩形管线 (render/rect.rs)
 │         └─ Task 6  文本管线 (render/text.rs) ◄── Task 5
 ├─ Task 5  字体+图集·纯逻辑 (text/font.rs, text/atlas.rs)   ⟂ 与 2/3/4 并行
 └─ Task 7  布局算法·纯逻辑 (layout.rs)                      ⟂ 与 2~6 并行
      └─ Task 8  组件树+叶子组件 (widget/mod.rs, Box, Text) ◄── 需要 4+6
           └─ Task 9  容器组件 (Column/Row/Padding/Center)
                └─ Task 10 事件分发+命中测试 (event.rs)
                     └─ Task 11 App glue + Button (app.rs, button.rs)
                          └─ Task 12 键盘交互
                               └─ Task 13 Showcase 集成 + M1 验收
                                    └─ Task 14 打磨 (clippy/fmt/README)
```

关键路径:1 → 2 → 3 → 4 → 8 → 9 → 10 → 11 → 12 → 13 → 14。
并行车道:渲染车道(2→3→4→6)∥ 字体车道(5)∥ 布局车道(7)。车道间契约 = Task 1 的值类型。

## Task List

### Phase 1: Foundation —— 从进程到 GPU 像素

- [ ] **Task 1: 项目脚手架 + 基础值类型**
  - **Description:** 建立可构建的单 crate 骨架,锁依赖版本,定义全部下游任务共用的纯数据值类型。
  - **Acceptance criteria:**
    - [ ] `Cargo.toml`:包名 `danqing`,edition 2024,`rust-version = "1.85"`;依赖锁定:winit 0.30.13 / wgpu 30 / fontdue 0.9 / etagere 0.3 / font-kit 0.14 / bytemuck 1 / pollster 1 / thiserror 2 / anyhow 1 / log 0.4 / env_logger 0.11
    - [ ] `src/lib.rs` 骨架存在,显式 re-export;`src/layout.rs` 含 `Color`(含 `Color::BLACK` 等常量)、`Point`、`Size`、`Rect`、`Edges`,带中文文档注释
    - [ ] `examples/showcase.rs` 空 main 存在
  - **Verification:** `cargo build` 通过;`cargo test` 通过(空);`cargo clippy -- -D warnings` 零警告
  - **Dependencies:** None
  - **Files:** `Cargo.toml`, `src/lib.rs`, `src/layout.rs`, `examples/showcase.rs`
  - **Scope:** S

- [ ] **Task 2: 跨平台开窗**
  - **Description:** 用 winit 0.30 `ApplicationHandler` 模式封装窗口创建与事件循环,事件先打印到日志;关闭窗口干净退出。
  - **Acceptance criteria:**
    - [ ] showcase 打开一个窗口(标题含 "danqing")
    - [ ] 鼠标/键盘事件经 `log` 输出可见
    - [ ] 点关闭按钮:进程退出,无 panic、无挂起
    - [ ] 平台 API 只出现在 `src/window.rs`(spec 约束)
  - **Verification:** `cargo run --example showcase` 人工确认三项;`env_logger` 输出事件
  - **Dependencies:** Task 1
  - **Files:** `src/window.rs`, `src/lib.rs`, `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 3: wgpu 渲染上下文**
  - **Description:** 创建 instance/device/queue/surface,实现按指定色清屏与 resize 重建 surface;debug 构建启用 wgpu 校验层。
  - **Acceptance criteria:**
    - [ ] 窗口持续清屏为指定颜色(非常量黑/白,验证参数通路)
    - [ ] 拖拽改变窗口大小:清屏正常,校验层零错误
    - [ ] 关闭干净退出,无校验错误
  - **Verification:** `cargo run --example showcase` 人工确认;resize 全程日志无 wgpu error
  - **Dependencies:** Task 2
  - **Files:** `src/render/mod.rs`, `src/lib.rs`, `src/window.rs`, `examples/showcase.rs`
  - **Scope:** M

### Checkpoint 1: Foundation
- [ ] `cargo run --example showcase`:开窗 → 指定色清屏 → resize → 干净退出,全程校验层零错误
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] **人工 review 后进入 Phase 2**

### Phase 2: Drawing —— 两条渲染管线

- [ ] **Task 4: SDF 矩形管线**
  - **Description:** 实例化 quad + fragment shader SDF 圆角/抗锯齿;支持每实例位置、尺寸、颜色、圆角半径;投影矩阵随窗口尺寸更新。
  - **Acceptance criteria:**
    - [ ] showcase 同屏展示多个不同颜色、不同圆角半径(含 0)的矩形
    - [ ] 圆角边缘平滑抗锯齿(肉眼无阶梯)
    - [ ] resize 后矩形不变形、不闪烁,校验层零错误
  - **Verification:** `cargo run --example showcase` 人工确认;resize 测试
  - **Dependencies:** Task 3
  - **Files:** `src/render/rect.rs`, `src/render/rect.wgsl`, `src/render/mod.rs`, `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 5: 字体加载与字形图集(纯逻辑)** ⟂ 可与 2/3/4 并行
  - **Description:** `font-kit` 查找系统中文字体(如微软雅黑),内嵌 OFL 字体兜底;`fontdue` 栅格化字形;`etagere` shelf-packing 管理图集页。全部为 CPU 纯逻辑,与 GPU 隔离。
  - **Acceptance criteria:**
    - [ ] 系统字体查找成功则用系统字体,失败回退内嵌字体(两条路径都有测试,系统路径可注入 mock)
    - [ ] 单元测试:栅格化 "你"/"A" 产出非空位图与正确度量;图集分配不重叠、满时正确扩容/报错
    - [ ] 字形缓存:同一 (char, size) 只栅格化一次
  - **Verification:** `cargo test text::` 全绿(无需 GPU)
  - **Dependencies:** Task 1
  - **Files:** `src/text/font.rs`, `src/text/atlas.rs`, `src/lib.rs`
  - **Scope:** M

- [ ] **Task 6: 文本渲染管线**
  - **Description:** 图集上传 GPU 纹理,文本管线采样图集绘制字形 quad;按字排版(CJK 无需整形,spec 已定);图集脏区域增量上传。
  - **Acceptance criteria:**
    - [ ] showcase 渲染 `Hello, 你好世界`,中英文均清晰、基线对齐
    - [ ] 不同字号、颜色文本同屏正确
    - [ ] resize 无失真,校验层零错误;回退字体路径人工断网/改名验证一次(或注入测试)
  - **Verification:** `cargo run --example showcase` 人工确认;`cargo test` 全绿
  - **Dependencies:** Task 3, Task 5
  - **Files:** `src/render/text.rs`, `src/render/text.wgsl`, `src/render/mod.rs`, `examples/showcase.rs`
  - **Scope:** M

### Checkpoint 2: Drawing
- [ ] showcase 同屏:圆角矩形 + `Hello, 你好世界`,resize 稳定,校验层零错误
- [ ] `cargo test` 全绿(图集/字体单测)
- [ ] **人工 review 后进入 Phase 3**

### Phase 3: UI Core —— 保留模式组件树到像素

- [ ] **Task 7: 布局算法(纯逻辑)** ⟂ 可与 2~6 并行
  - **Description:** 在 Task 1 值类型之上实现 `Constraints` 与布局算法:固定/填充约束传递、Column/Row 主轴分配、Padding/Center 计算。
  - **Acceptance criteria:**
    - [ ] `Constraints` 能表达:固定尺寸、最大值约束、fill 剩余空间
    - [ ] 单元测试覆盖:嵌套 Column、Row+Padding 组合、Center 居中、空 children、溢出收缩
  - **Verification:** `cargo test layout::` 全绿
  - **Dependencies:** Task 1
  - **Files:** `src/layout.rs`
  - **Scope:** M

- [ ] **Task 8: 组件树与叶子组件(Box/Text)**
  - **Description:** 定义 `Widget` trait 与持久 `Node` 树;实现叶子组件 `Box`(背景色/圆角)与 `Text`(绑定 `Fn(&S) -> String` 读取闭包,框架每帧同步);帧循环集成:遍历树 → layout → 收集绘制命令(rect 实例 + 文本 run)→ 两条管线绘制。
  - **Acceptance criteria:**
    - [ ] showcase 改为**声明一棵树**并渲染:有色 Box + 一行 Text(替换 Task 4/6 的手写绘制调用)
    - [ ] `Text` 内容来自状态闭包,状态改变后下一帧文本更新
    - [ ] 集成测试(无 GPU):建树 → 布局 → 收集绘制命令,断言命令数量与几何正确
    - [ ] 组件属性经 lib.rs re-export,不深穿路径
  - **Verification:** `cargo run --example showcase` 视觉确认;`cargo test` 全绿
  - **Dependencies:** Task 4, Task 6, Task 7
  - **Files:** `src/widget/mod.rs`, `src/widget/box_.rs`, `src/widget/text.rs`, `src/app.rs`(帧循环初版), `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 9: 容器组件(Column/Row/Padding/Center)**
  - **Description:** 实现四个容器组件,内部用 Task 7 布局算法;showcase 改为容器排版的页面。
  - **Acceptance criteria:**
    - [ ] showcase 页面由 `Column`/`Row`/`Padding`/`Center` 嵌套排布,视觉正确
    - [ ] 单元测试:容器布局结果与 Task 7 算法预期一致
    - [ ] 新增组件全部出现在 showcase(spec "以用代测")
  - **Verification:** `cargo run --example showcase` 视觉确认;`cargo test` 全绿
  - **Dependencies:** Task 7, Task 8
  - **Files:** `src/widget/column.rs`, `src/widget/row.rs`, `src/widget/padding.rs`, `src/widget/center.rs`, `examples/showcase.rs`
  - **Scope:** M

### Checkpoint 3: UI Core
- [ ] showcase 完全由组件树驱动(无手写绘制调用残留)
- [ ] `cargo test` 全绿(布局 + 组件树集成)
- [ ] `cargo clippy -- -D warnings` 零警告
- [ ] **人工 review 后进入 Phase 4**

### Phase 4: Interactivity —— 事件、消息、按钮、键盘

- [ ] **Task 10: 事件类型与命中分发**
  - **Description:** 定义内部 `Event`(鼠标移动/按下/抬起/滚轮、键盘);winit 事件转换;沿组件树做命中测试(后绘制者优先),把鼠标事件送达目标组件;维护 hover/pressed 状态并触发重绘。
  - **Acceptance criteria:**
    - [ ] 单元测试:嵌套树命中顺序正确(顶层子树优先)、边界点击、移出后 hover 清除
    - [ ] showcase 中 Box hover 变色、pressed 再变色(状态由事件驱动,非轮询)
    - [ ] 滚轮事件送达命中组件(暂默认可不消费)
  - **Verification:** `cargo test event::` 全绿;showcase 人工 hover/点击确认
  - **Dependencies:** Task 8, Task 9
  - **Files:** `src/event.rs`, `src/window.rs`, `src/widget/mod.rs`, `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 11: App glue 与 Button 组件**
  - **Description:** 定义 `App` trait(`update(msg)` / `view()` 或等价结构)、框架入口 `run()`;`Button` 组件:on_click 产生消息 `Msg` → `App::update` 改状态 → 每帧同步到树。
  - **Acceptance criteria:**
    - [ ] showcase 含按钮:点击 → 计数 +1,计数文本经状态闭包当帧更新
    - [ ] 按钮有 hover/pressed 视觉反馈(复用 Task 10 状态)
    - [ ] `run()` 一行启动应用,窗口关闭即干净退出
  - **Verification:** `cargo run --example showcase` 人工点击确认计数;`cargo test` 全绿
  - **Dependencies:** Task 10
  - **Files:** `src/app.rs`, `src/widget/button.rs`, `src/lib.rs`, `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 12: 键盘交互**
  - **Description:** 键盘事件(字符键 + 方向键等功能键)直送 `App`(M1 无焦点系统,spec 已定);showcase 增加键盘区:方向键/WASD 移动一个方块。
  - **Acceptance criteria:**
    - [ ] 方向键/WASD 移动方块,位置持续平滑更新
    - [ ] 字符输入能被 App 收到(日志或界面回显)
    - [ ] 键盘事件与鼠标事件互不干扰
  - **Verification:** `cargo run --example showcase` 人工按键确认
  - **Dependencies:** Task 11
  - **Files:** `src/event.rs`, `src/app.rs`, `examples/showcase.rs`
  - **Scope:** S

### Checkpoint 4: Interactivity
- [ ] 按钮计数 + 键盘移方块均工作;事件单测全绿
- [ ] 对照 spec 验收标准 #3(事件分发)逐项人工确认
- [ ] **人工 review 后进入 Phase 5**

### Phase 5: Showcase & Polish —— 验收与收尾

- [ ] **Task 13: Showcase 集成与 M1 验收**
  - **Description:** 组织最终 showcase 页面:色板(彩色矩形)、圆角展示、中英文文本、按钮计数、键盘移方块五个区域;对照 spec Success Criteria 逐条验收。
  - **Acceptance criteria(spec 6 条全过):**
    - [ ] #1 窗口稳定 ~60 FPS(vsync,持续渲染下观察无掉帧)
    - [ ] #2 五个展示区全部呈现且正确
    - [ ] #3 鼠标/键盘事件分发正确(人工 + 单测)
    - [ ] #4 关闭干净退出,校验层零错误
    - [ ] #5 `cargo test` 全绿、`cargo clippy -- -D warnings` 通过
    - [ ] #6 适配层之外无平台专有 API(代码评审 `widget/`、`layout.rs`、`event.rs`)
  - **Verification:** 逐条人工验收并记录结果
  - **Dependencies:** Task 9, Task 11, Task 12
  - **Files:** `examples/showcase.rs`
  - **Scope:** M

- [ ] **Task 14: 打磨收尾**
  - **Description:** 零警告收尾、格式化、README(构建/运行/架构图)、公开 API 中文文档注释补全、spec/plan 状态更新。
  - **Acceptance criteria:**
    - [ ] `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test` 全绿
    - [ ] `cargo build --release` 成功
    - [ ] README 含运行方式与分层架构说明;所有公开类型有中文文档注释
  - **Verification:** spec Commands 全部执行通过
  - **Dependencies:** Task 13
  - **Files:** `README.md`, 全源码(注释/格式收尾)
  - **Scope:** S

### Checkpoint: Complete(M1 完成)
- [ ] spec Success Criteria 6/6 通过,记录在案
- [ ] 全部 Commands 绿;`tasks/todo.md` 全部勾选
- [ ] **人工终审,M1 关闭**

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| 内嵌回退字体与 spec "Never 提交二进制" 边界冲突 | Med | **见 Open Question 1**,Task 5 开工前必须裁决 |
| SDF shader 跨后端表现差异 | Med | 成熟技术参考多;备用 `lyon` tessellation(演进点已预留) |
| winit 0.30 事件循环模型踩坑 | Med | 严格按官方 `ApplicationHandler` + `RedrawRequested` 模式;Task 2 单独验证干净退出 |
| wgpu 30 新大版本 API 变动 | Med | Task 3 最先验证 surface/resize 全路径,失败早暴露(fail fast) |
| 无 GPU 环境无法自动测试 | Low | 布局/事件/图集全部纯逻辑单测,与 GPU trait 隔离(spec 已定) |
| 文本度量(基线/行高)细节耗时 | Low | M1 只按字排版,行高取 fontdue 水平度量,不追求完美排版 |

## Open Questions(阻塞性已标注)

1. **[阻塞 Task 5] 内嵌回退字体如何进仓库?** spec Boundaries 写"Never 提交字体等二进制大文件",但已决方案要求"内嵌 OFL 字体兜底",二者冲突。选项:
   - (a) **build.rs 构建期下载**(固定 URL + sha256 校验),仓库零二进制 —— 推荐,不违反边界
   - (b) 提交得意黑(Smiley Sans,OFL,~2MB)作为**书面豁免**的唯一例外
   - (c) 不内嵌,回退=用户自带路径(放弃 OQ2 的兜底决策)
2. [不阻塞,Task 1 定] 值类型放 `src/layout.rs` 是否符合你对 spec 项目结构的理解?还是想单开 `src/geom.rs`?
3. [不阻塞,Task 11 定] `App` trait 形态偏好:`update(msg)+view()` Elm 风格,还是 `on_event()/build()` 回调风格?docs/plan.md 的消息模型倾向前者。

## Parallelization Notes

- **可并行**:Task 5(字体/图集)∥ Task 7(布局)∥ Task 2→3→4(渲染车道);车道契约是 Task 1 的值类型,必须先落地。
- **必须串行**:Task 8 起的 UI 核心链(8→9→10→11→12→13→14),每一步都依赖前一步的 showcase 状态。
- **多会话协作时**:Task 5 与 Task 7 适合分配给独立会话,两者均为纯逻辑、自带单测、不碰 GPU。