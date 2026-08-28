# Todo: 年度报告(旗舰版统计增强)

- [x] **S1** 聚合纯逻辑(stats.rs): `YearSummary` + `year_summary` + `month_trend` + `local_ym`/`local_ymd`
  - [x] 跨年边界测试
  - [x] 空数据测试
  - [x] 活跃天去重测试
  - [x] 场景分布测试
  - [x] 月趋势补零/跨年/末项测试
  - [x] CP1: `cargo test --example pomodoro` 全绿 (26 个 stats 测试)
- [x] **S2** 报告视图 + 切换(main.rs)
  - [x] `report_open` + `Msg::ToggleReport` + 三面板互斥(含焦点恢复 `report-button`)
  - [x] MultiPanel 第 4 子项 + bind 优先
  - [x] `report_panel`: 本年汇总 / 场景分布 / 近 12 月趋势
  - [x] 控制条「报告」按钮 + Esc 关闭
  - [x] toggle/互斥/escape/焦点恢复测试
  - [x] CP2: 全量门槛
- [x] **验证**: fmt + clippy 零警告 + `cargo test --example pomodoro` 196 + `cargo test --lib --tests` (239+58) 全绿
- [ ] **人工核验**: 运行 pomodoro → 积累会话 → 打开「报告」面板核对数字与 CSV 一致;验互斥/Esc
- [x] benchmark 双门槛复跑 (2026-08-01: startup 568.2 ms ≤1000ms + WS 185.1 MB ≤360MB, PASS)
