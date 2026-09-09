# Todo: job 异步作业模块 (框架下沉·簇B)

> Plan: `tasks/plan-job.md` (D1–D9 决策与依赖图)。Spec: `docs/specs/SPEC-job.md` (已批准 2026-09-09)。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。
> commit/push 待用户指示 (D7)。

## Phase 1: 框架纯加法

- [x] **T1: `src/job.rs` 建档 + `AsyncJob<T>` 移植 ✅ 2026-09-09** — 文件头 @author/@date; 模块头 doc (做什么/不做什么/为什么, 锚盘点簇B+两条教训出处); 移植 search.rs:16-77 语义零漂移 (launch 代次+1/poll 仅最新轮/invalidate 外部失效/Default); `lib.rs` 加 `mod job;` + re-export (D2)
  - Acceptance: 移植 2 条 (delivers_latest_generation/invalidate_discards_inflight_result 等价断言, 含「结果只取一次」「invalidate 后新一轮照常交付」) + 新增 1 条: 两连 launch 旧轮晚到丢弃 (open.rs 两连开语义上提) ✅ (3 绿)
  - Verify: `cargo test job::` 全绿; 三件套绿 ✅
  - Files: `src/job.rs` (新), `src/lib.rs` | S

- [x] **T2: `launch_catched` + `CancelToken`/`CancelFlag` ✅ 2026-09-09** — panic 护栏 (catch_unwind → Err(on_panic()), 复用 launch); 令牌对 (pair()/cancel()/is_cancelled()/arc_cloned(); Drop 置位); re-export 补齐 (D3)
  - Acceptance: panic 作业 → poll 收到 Err 不卡死; token drop 置位 + 显式 cancel() + arc_cloned 互操作同一旗标; Ok 直通 ✅ (4 绿)
  - Verify: `cargo test job::` 7 全绿; 三件套绿 ✅
  - Files: `src/job.rs`, `src/lib.rs` | S (依赖 T1)

- [x] **T3: `SearchNav` 移植 ✅ 2026-09-09** — 现网全量公开面 (D4: new/hits/total/current_line/jump_first_from/jump_next/jump_prev/position); partition_point + 环绕语义零漂移
  - Acceptance: 移植 3 条 (nav_first_from_positions 四姿态/next_prev_wrap 含 position/empty_hits 全 None) ✅ (3 绿)
  - Verify: `cargo test job::` 10 全绿; 三件套绿 ✅
  - Files: `src/job.rs`, `src/lib.rs` | S (依赖 T1)

- [x] **T4: update.rs 代次戳加固 ✅ 2026-09-09** — `GEN: AtomicU64` 发行器 + CHECK_CACHE 折代次进 Mutex `(u64, Option<CheckCache>)`; spawn_check 入口取号, 代次闸门 `accept_gen` 拒绝乱序旧代次; 公开 `publish` 签名不变 (产品直调恒取最新号)。注: `gen` 是 edition 2024 保留字, 标识符用 `generation`
  - Acceptance: 新增 `generation_gate_rejects_stale_publish` (纯函数闸门: 新代次应用/同代次放行/旧代次拒绝 —— 全局态测试会与既有 `current_hint_reflects_published_cache` 并行互踩, 闸门策略提纯测); 既有 update 测试零改动全绿 ✅
  - Verify: `cargo test --features update` 461 全绿; 默认三件套绿 ✅
  - Files: `src/update.rs` | S (独立)

### ★ Checkpoint 1: 纯加法完成
- [x] 三件套绿 (含 --features update) ✅ 2026-09-09
- [x] 既有测试零改动全绿 (纯加法证明: 437 lib + update 特性 461 均绿) ✅

## Phase 2: 以用代测

- [x] **T5: showcase 异步作业演示卡 ✅ 2026-09-09** — 照既有卡范式: 按钮 A 起 300ms 假作业 (返回轮次号, 快速连点演示旧轮丢弃 —— 旧轮「第 N 轮完成」永不上屏); 按钮 B 起 panic 作业 (launch_catched → Err 文案, 不卡死); tick 内 poll
  - Acceptance: 三姿势实机可触发可观察; 三件套绿 (clippy --all-targets 净) ✅
  - Verify: `cargo run --example danqing-showcase` 实机人工过目 (待人工)
  - Files: `examples/showcase.rs` | S

### ★ Checkpoint 2: 框架侧完成
- [ ] 实机人工过目: 演示卡三姿势
- [ ] 用户裁决 commit/push (danqing 先行)

## Phase 3: log 联动迁移 (danqing-log 仓, 前提闸 D8)

- [ ] **T6: log 调用点迁移** — 前提: danqing 已 push 且 log 在途批次 (async-open+text-selection) 已提交
  - search.rs: 删 AsyncJob + SearchNav 本地实现 → `danqing::{AsyncJob, SearchNav}`; `bytes_as_literal_regex` 保留 (D5); 字段过滤/正则搜索调用点直通
  - open.rs: OpenJob 保留产品语义 (OpenOutcome/OpenKind/path); 内部 AsyncJob 换框架版; run_catched 删 → launch_catched; cancel 原子量换 CancelToken (手写 Drop 删除, 语义由令牌承载); IndexHooks 互操作走 CancelFlag::arc_cloned
  - Acceptance: 编译过; log 侧 `AsyncJob`/`SearchNav`/`run_catched` 本地实现零残留 (grep 验证)
  - 硬判据: log 既有测试零改动或近零改动全绿 (search 6 + open 7; 同簇A mixer 8 先例)
  - Verify: `cargo test` 全绿; 三件套绿
  - Files: `src/search.rs`, `src/open.rs` | M

### ★ Checkpoint 3: 全量验收
- [ ] 两仓三件套绿
- [ ] 用户裁决 log commit/push (message 注明关联 danqing 提交)
