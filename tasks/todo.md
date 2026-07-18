# Todo: 丹青 (danqing) M1

> 详见 `tasks/plan.md`(验收标准、依赖、风险)。每任务完成后勾选,检查点需人工确认。
> 前置条件 ✅ Rust 工具链已装(cargo 1.97.0)。

## Phase 1: Foundation
- [x] **Task 1** 脚手架 + 基础值类型 — `cargo build`/`test`/`clippy` 绿 ✅ 2026-07-16
- [x] **Task 2** 跨平台开窗(依赖 1)— 开窗/事件日志/干净退出 ✅ 2026-07-16(冒烟运行通过;干净退出待人工确认)
- [x] **Task 3** wgpu 渲染上下文(依赖 2)— 指定色清屏,resize 校验零错误 ✅ 2026-07-16(Checkpoint 1 人工验收通过:深蓝灰清屏/resize/干净退出)

### ⏸ Checkpoint 1 ✅ 通过(2026-07-16):开窗+深蓝灰清屏+resize+干净退出,校验零错误

## Phase 2: Drawing
- [x] **Task 4** SDF 矩形管线(依赖 3)— 多色圆角矩形,边缘 AA,resize 不变形 ✅ 2026-07-16(PrintWindow 截图自检通过,校验零错误)
- [x] **Task 5** 字体加载+字形图集·纯逻辑(依赖 1,⟂ 可并行)✅ 2026-07-16(OQ1 裁决:build.rs 经 jsdelivr 下载 ZCOOL XiaoWei OFL;12 项测试全绿)
- [x] **Task 6** 文本渲染管线(依赖 3+5)— 渲染 `Hello, 你好世界` 中英文清晰 ✅ 2026-07-16(截图自检:基线对齐、两字号两色;系统字体走 Microsoft YaHei)

### ⏸ Checkpoint 2 ✅ 通过(2026-07-16):矩形+中英文文本同屏,resize 稳定,校验零错误,`cargo test` 12/12

## Phase 3: UI Core
- [x] **Task 7** 布局算法·纯逻辑(依赖 1,⟂ 可并行)— `cargo test layout::` 绿 ✅
- [x] **Task 8** 组件树 + Box/Text(依赖 4+6+7)— showcase 改为声明式树,文本绑定状态闭包 ✅
- [x] **Task 9** 容器组件 Column/Row/Padding/Center(依赖 7+8)— showcase 容器排版 ✅(修复:fill 子项交叉轴改宽松约束;fill 槽放 Center)

### ⏸ Checkpoint 3 ✅ 通过(2026-07-16):showcase 完全组件树驱动(截图自检),24 项测试全绿,clippy 0

## Phase 4: Interactivity
- [x] **Task 10** 事件类型+命中分发(依赖 8+9)— 命中单测绿,Box hover/pressed 变色 ✅ 2026-07-16(`tests/event_dispatch.rs` 5 项,`tests/hover_debug.rs` 1 项全绿)
- [x] **Task 11** App glue + Button(依赖 10)— 点击计数当帧更新,`run()` 一行启动 ✅ 2026-07-16
- [x] **Task 12** 键盘交互(依赖 11)— 方向键/WASD 移方块,字符键可达 App ✅ 2026-07-16(showcase 新增键盘区,`cargo test`/`clippy`/`fmt --check`/`build --release` 全绿)

### ⏸ Checkpoint 4 ✅ 通过(2026-07-16):计数+键盘可用,spec 验收 #3 已对照,review 后进 Phase 5

## Phase 5: Showcase & Polish
- [x] **Task 13** Showcase 集成 + M1 验收(依赖 9+11+12)— spec Success Criteria 6 条逐条过 ✅ 2026-07-16(运行无 wgpu 校验错误,五个展示区全部呈现)
- [x] **Task 14** 打磨收尾(依赖 13)— fmt/clippy/test/release 全绿,README + 中文文档注释 ✅ 2026-07-16

### ⏸ Checkpoint Complete ✅ 通过(2026-07-16):6/6 验收通过,`cargo test`/`clippy -D warnings`/`fmt --check`/`build --release` 全绿,M1 关闭

---

## Post-M3 增量优化

- [x] **Task 16** 提取 `TextEditor` 公共编辑层,`TextArea` 支持撤销/重做 — `Ctrl+Z` / `Ctrl+Shift+Z` / `Ctrl+Y`,单元测试覆盖 ✅ 2026-07-18
  - 验证:`cargo test` 100 项全绿,`cargo clippy -- -D warnings`,`cargo fmt --check`,`cargo build --release` 全通过。
**并行车道**:渲染车道 2→3→4→6 ∥ 字体车道 5 ∥ 布局车道 7(契约 = Task 1 值类型)。
**阻塞项**:Open Question 1(内嵌字体进仓库方式)阻塞 Task 5,需先裁决。
