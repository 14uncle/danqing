# Plan: 番茄钟场景动效 — 雨场景 UV 位移化改造

> Spec: `docs/specs/pomodoro-scene-motion-rain-uv.md`(2026-07-29 起草,概念待用户确认)

## 任务分解

| # | 任务 | 产出 | 验证 |
|---|------|------|------|
| T1 | wgsl 雨段新增 `rain_shear` 位移场 + fs_main 采样分支追加雨项 | `src/render/background.wgsl` 雨段常量区 + 位移函数 + 采样分支 | `cargo build --release --example pomodoro` 通过;`rain_intensity=0` 逐像素一致(暗启动纪律,由既有暂停/非雨场景测试与帧差佐证) |
| T2 | 客观佐证 + 性能门槛 | 帧差对照(改造前基线 vs 改造后、山场景对照、暂停沉降)+ benchmark | spec Success Criteria 1/3/6 |
| T3 | 终审调参 + 收口 | 用户目测终审;调参只动雨段常量;门槛全绿 + 五轴评审;归档 | spec Success Criteria 2/7/8 |

## 顺序与依赖

T1 → T2 → T3 严格串行(单文件改动,无并行价值)。

## 预期变更面

- `src/render/background.wgsl` — 唯一预期变更文件(雨段新增常量 + `rain_shear` 函数 + fs_main 采样分支加雨位移项)。
- `src/render/background.rs` / `examples/pomodoro/motion.rs` / `main.rs` — 预期零变更(rain_intensity 通道已就位);若实现中发现必须动,先问用户(spec Ask first)。

## 风险与撤退

- 阵风位移在天空读出亮带 → 定量预判 <0.5% 不可读;若目测可读,降 `RAIN_SHEAR_GAIN` 或给位移场加纵向 mask(调参轮内解决)。
- 触发 spec Retreat Discipline 任一条件 → 整里程碑 `git revert`。
