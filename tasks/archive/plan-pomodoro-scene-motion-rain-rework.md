# Plan: 番茄钟场景动效 — 雨场景改造(去丝 + 雨幕独挑 + 暂停定格)

> Spec: `docs/specs/pomodoro-scene-motion-rain-rework.md`(2026-07-29 关闭,验收 8/8)

## 执行记录

| # | 任务 | 产出 | 结果 |
|---|------|------|------|
| T1 | UV 位移化试错: 摆动式剪切 → 双相位流图 | `background.wgsl` 两版机制 | 目测均不达 ("底层静态背景图的雨没有动"),用户裁定换方向,撤出 |
| T2 | 静态图去丝 + 雨幕独挑 | `export-scenes.py` 雨配置去 streaks 重生成;wgsl 雨段门槛 0.70/0.72/0.85 | 终审 "棒,效果对了" |
| T3 | 暂停定格 (用户追加) | `BackgroundFrame.rain_time` + uniform 第 7 槽 + `rain_clock` 包络推进 | 单测锁定冻结/续走;运行时验证 0.000 + 可见 |
| T4 | 收口 | 门槛全绿 + benchmark PASS + 五轴 Approve + 归档 | 本文件 |

## 备注

- 计划起草时按 UV 位移范式分解(wgsl 位移场/佐证/终审 三任务);执行中经用户两次裁定转向,上表为实际路径决算。
- 预期变更面("只改 wgsl")因方向修正扩大到: 生成器 + 资产 + wgsl + background.rs(uniform 第 7 槽)+ motion.rs/main.rs(雨钟策略)。`scenes.rs` 重出无 diff,widget/layout/window 零变更。
