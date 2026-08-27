# Plan: 年度报告(旗舰版统计增强)

- @author 十四叔
- @date 2026/08/01
- 状态: 里程碑 1 Task E; 2026-08-01 计划已批准
- 依据: `tasks/plan-flagship-roadmap.md` Task E + `docs/specs/companion-flagship-pricing.md` + `docs/ideas/pomodoro-competitor-memo.md`

## 目标

数据层 MVP(今日/本周/累计)→ 当前年汇总 + 近 12 月趋势的深度洞察。旗舰版统计增强兑现;数据格式零改动(纯读聚合)。

## 已定决策

- 年度报告 = 旗舰版(2026-08-01 裁定);本期不实现真实付费门禁(语义先行,「旗舰版」角标作语义标记)。
- 范围: 当前年汇总 + 近 12 月趋势,无年份导航。
- 入口: 控制条「报告」按钮;Esc 关闭;三面板(设置/统计/报告)互斥。

## S1: 聚合纯逻辑(stats.rs)

- `YearSummary { total_secs, session_count, active_days, scene_secs: Vec<u64> }`
- `FocusHistory::year_summary(&self, year: u32) -> YearSummary`
- `FocusHistory::month_trend(&self, now_wall: u64, months: u32) -> Vec<(u32, u32, u64)>`
- 私有 `local_ym` / `local_ymd`(epoch 秒 → 本地日期,chrono `Local`;出界回退)。
- 测试: 跨年边界、空数据、活跃天去重、场景分布、月趋势补零/跨年/末项为 now 月。
- **CP1 验收**: `cargo test --example pomodoro stats -- --exact` 全绿。

## S2: 报告视图 + 切换(main.rs)

- `report_open: bool` + `Msg::ToggleReport` + 三面板互斥(update 双向关)。
- MultiPanel 加第 4 子项, bind 优先 `report_open → 3`。
- `report_panel(t)`: 遮罩 + 玻璃卡片(宽 ~360); 标题 +「旗舰版」角标 + CloseButton; 本年汇总行 / 场景分布行 / 近 12 月趋势行。
- 控制条「报告」按钮; Esc 关闭。
- 测试: toggle/互斥/escape/面板数据一致。
- **CP2 验收**: 面板端到端可见 + 全量门槛。

## 验证(总)

1. `cargo test --example pomodoro` 全绿
2. `cargo test --lib --tests` 全绿
3. `cargo fmt` + `cargo clippy -- -D warnings` 零警告
4. 人工: 积累会话 → 打开「报告」核对数字与 CSV 一致;验互斥/Esc。
5. benchmark 双门槛不破(不碰框架 src/)。
