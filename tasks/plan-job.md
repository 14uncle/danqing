# Plan: job 异步作业模块 (框架下沉·簇B)

> Spec: `docs/specs/SPEC-job.md` (已批准 2026-09-09)。来源: `docs/intent/framework-sinking.md` 簇B。
> 簇A 先例 (plan-anim.md) 的结构与纪律全部沿袭。

## 决策 (D1–D9)

- **D1 无 feature 门控** (spec 已裁定): job 零依赖纯 std; update 的门控为 ureq/open 依赖而设, anim 亦未门控
- **D2 命名保源**: AsyncJob/SearchNav 沿用 log 原名 (血统可 grep, 移植零认知税); CancelToken/CancelFlag 为新设施命名
- **D3 CancelToken::cancel(&self)**: 内部可变性 (AtomicBool), 与 drop 置位同语义; token 单持有 (不 Clone), flag 可 Clone 进 worker
- **D4 SearchNav 公开面 = 现网全量**: new/hits/total/current_line/jump_first_from/jump_next/jump_prev/position (spec 措辞「以现网为准」落到这 8 个)
- **D5 bytes_as_literal_regex 留 log 产品侧**: 它拖 regex::bytes 依赖, 不进框架 (spec 非目标未列, 此处钉死)
- **D6 update.rs 最小加固手法**: 加 `CHECK_GEN: AtomicU64` 代次戳, spawn_check 入口取号, publish 内部仅在 gen ≥ 已存代次时应用 (同代次二次发布须放行); `publish` 公开签名不变; 验证走 `cargo test --features update` (update 是 feature 门控, 默认三件套不含)。落地命名对账: 静态 `GEN` + 局部 `generation` (裸 `gen` 是 edition 2024 保留字)
- **D7 零 commit 纪律**: 未获用户指示不 commit/push; log 联动两仓分别提交、message 注明关联 (同簇A D7)
- **D8 log 迁移前提闸**: log 在途批次 (async-open+text-selection 未 commit) 落地前不动 log (open.rs/selection.rs 同文件冲突)
- **D9 进度计数不封装**: Arc<AtomicU64> 直用 (worker 累加/UI 读), 模块 doc 写明范式; 包装无行为增量不收

## 依赖图

```
T1 (job.rs + AsyncJob) ──┬── T2 (launch_catched + CancelToken) ──┐
                         └── T3 (SearchNav) ────────────────────┤
T4 (update.rs 代次戳, 独立文件) ────────────────────────────────┤
                                                                ▼
                                                  CP1 (三件套绿, 纯加法)
                                                                │
                                                                ▼
                                              T5 (showcase 演示卡, 依赖 T1-T3 API)
                                                                │
                                                                ▼
                                       CP2 (三件套绿 + 实机人工过目 + commit 裁决)
                                                                │
                                          [前提闸 D8: log 在途批次已提交]
                                                                ▼
                                              T6 (log 迁移 search.rs/open.rs)
                                                                │
                                                                ▼
                                              CP3 (两仓绿 + log 提交裁决)
```

## 风险

| 风险 | 缓解 |
|---|---|
| 线程测试时序脆弱 (sleep 轮询) | 沿用源头 1s 轮询上限模式 (实测稳定); 不收紧时序断言 |
| update.rs 代次戳改变多线程行为 | 签名不变 + 只加拒绝路径; 既有 update 测试零改动为判据 |
| panic 护栏在 release (panic="abort") 下是死代码 (worker panic 直接 abort 进程) | 源头 run_catched 同款继承性限制; 文档显式声明守护域 = dev/test/非 abort 消费者 (job.rs 模块头 + launch_catched doc + showcase 卡注释) |
| log 迁移与在途批次撞车 | D8 前提闸: 不提交不动工, spec 已写明 |
| showcase panic 演示误伤应用 | catch_unwind 隔离在 worker 线程; UI 线程不接触 |
