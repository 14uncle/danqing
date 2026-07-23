# Todo: 丹青阶段 2 — 专注陪伴 POC(番茄钟 × 场景沉浸)

> 详见 `tasks/plan-phase2.md`(验收标准、依赖、风险)。每任务完成后勾选,检查点需人工确认。

## Phase 1: 框架纯逻辑
- [x] **Task 1** 颜色插值、对比度与动效求值纯逻辑 — `cargo test --lib theme/layout` 绿 ✅ f73b373
- [ ] **Task 2** ScenePalette + SceneTheme + SceneSpec(依赖 1)— token 护栏 + lerp + re-export

### ⏸ Checkpoint 1: 框架 token 就绪
- [ ] `cargo test --lib --tests` 全绿;clippy 零警告
- [ ] 人工确认 ScenePalette 字段集(生成管线输入契约)

## Phase 2: 状态机 / 资产管线 / 渲染管线(可并行)
- [ ] **Task 3** 番茄钟状态机 + example 骨架 — `cargo test --example pomodoro` 机制验证 + 状态机测试绿
- [ ] **Task 4** 场景生成管线 export-scenes.py(依赖 2)— 4 张场景 PNG + scenes.rs,人工过目击调
- [ ] **Task 5** 背景管线多场景 + 交叉淡化 + App 通道 — showcase 单图路径回归无异常

### ⏸ Checkpoint 2: 状态机 / 资产 / 管线就绪
- [ ] `cargo test --lib --tests` + `cargo test --example pomodoro` 全绿;clippy 零警告
- [ ] showcase 人工回归;4 张场景图人工 review

## Phase 3: POC 组装与终验
- [ ] **Task 6** POC 界面组装(依赖 2/3/4/5)— 计时全功能 + 场景即时切换,wgpu 校验层无错误
- [ ] **Task 7** 过渡动画 + 色调流动 + 4 场景对比度护栏 + spec 终验(依赖 6)

### ✅ Checkpoint Complete: 阶段 2 POC 关闭
- [ ] spec Success Criteria 7/7 通过
- [ ] 全部 Commands 绿;本 todo 全部勾选
- [ ] 人工终审(重点:场景沉浸观感是否成立)
