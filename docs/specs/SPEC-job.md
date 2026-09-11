# SPEC: job 异步作业模块 (框架下沉·簇B)

- @author 十四叔
- @date 2026/09/09
- 状态: **已批准**（2026-09-09）；框架侧 T1–T5 已落地（/build auto 零 commit，三件套绿：默认 447 lib + 8 二进制 / update 特性 461）；review 通过（APPROVE + 1 Required 已处置：panic="abort" release 下护栏是死代码，文档守护域已声明）；code-simplify 零改动（逐字节保真移植 + 评审零结构性发现，已在正确高度）；T6 log 迁移待前提闸（log 在途批次先提交）
- 需求来源: `docs/intent/framework-sinking.md` 簇B（重复发明 4 处，成本 M，建议顺序第二；簇A anim 已于 2026-09-09 落地：danqing fdd3d01 + pomodoro 3c74dc4）

## 目标

把「异步作业（worker 线程 + tick 拾取 + 代次防乱序）」从 danqing-log 下沉为 danqing 公共模块 `src/job.rs`，并把两条实战教训内化为框架设施：worker panic 护栏（async-open 评审 FYI→Required 升档的教训）与 drop 即取消令牌（OpenJob::drop 语义上提为类型）。命中导航 SearchNav 搭车同沉（生于服务异步搜索作业，与消费者同处）。

**政策门②裁定（随本 spec 批准生效）**：非 UI 纯工具模块进框架合法——update.rs（网络栈）与 anim.rs（动画原语）已两先例；job 零依赖纯 std（thread/Mutex/Atomic），**不设 feature 门控**（update 的门控为 ureq/open 依赖负载而设，job 无此负载；anim 亦未门控）。

**非目标**（明确不做）：
- danqing-log 消费者迁移有前提：log 在途批次（async-open + text-selection，未 commit）落地之前不动 log（改动同文件，不可并行）——迁移是第二阶段，见 §消费者迁移
- clipboard 的 revision 监听计数器（`main.rs:63-68`）不下沉：它是「变更计数 + tick 比对」的 watcher 范式，不是作业；单一消费者，避免投机泛化
- update.rs 的 AsyncJob 全量迁移不做：fire-and-forget 全局发布与 poll 范式不同构，迁移 = 产品侧 API 变化；只做最小加固（见 §结构）
- 通用线程池/任务调度/async 运行时；进度百分比的通用封装（`Arc<AtomicU64>` 直用即可，包装无行为增量）
- pomodoro / xirang 侧改动（pomodoro 无异步作业；xirang 已埋）

## API 设计

### `AsyncJob<T>` —— 异步作业本体（移植 danqing-log `search.rs:16-77`，语义零漂移）

```rust
pub struct AsyncJob<T> { /* done: Arc<Mutex<Option<(u64, T)>>>, rev: Arc<AtomicU64>, generation, seen_rev */ }
impl<T: Send + 'static> AsyncJob<T> {
    pub fn new() -> Self;
    /// 发起新一轮 (代次+1): 在途旧轮不取消, 其晚到结果按乱序丢弃。
    pub fn launch(&mut self, work: impl FnOnce() -> T + Send + 'static);
    /// 拾取完成结果 (仅最新一轮; 每帧调用, 无结果零成本)。
    pub fn poll(&mut self) -> Option<T>;
    /// 使在途作业失效 (代次+1, 不发起新工作): 换文件等外部失效由持有方显式调用
    /// (async-open review C1: 旧文件在途结果不得贴到新文件)。
    pub fn invalidate(&mut self);
}
```

### panic 护栏 —— `launch_catched`（教训内化：detached 线程 panic 不得让应用层永久 Loading）

```rust
impl<T: Send + 'static, E: Send + 'static> AsyncJob<Result<T, E>> {
    /// 同 launch, 但 worker panic 经 catch_unwind 转为 Err(on_panic()) 交付,
    /// 应用层 poll 照常收到结果 (按各类失败语义处理, 不卡 Loading)。
    pub fn launch_catched(
        &mut self,
        work: impl FnOnce() -> Result<T, E> + Send + 'static,
        on_panic: impl FnOnce() -> E + Send + 'static,
    );
}
```

### 取消令牌 —— `CancelToken` / `CancelFlag`（教训内化：drop 即取消，`OpenJob::drop` 语义上提为类型）

```rust
/// 取消令牌 (持有端): drop 或 cancel() 置位; worker 侧持 CancelFlag 轮询早退。
pub struct CancelToken { /* Arc<AtomicBool> */ }
impl CancelToken {
    /// 创建令牌对: (持有端, worker 端)。
    pub fn pair() -> (CancelToken, CancelFlag);
    /// 显式提前取消 (drop 同样取消)。
    pub fn cancel(&self);
}
#[derive(Clone)]
pub struct CancelFlag { /* Arc<AtomicBool> */ }
impl CancelFlag {
    /// worker 循环内的早退查询。
    pub fn is_cancelled(&self) -> bool;
    /// 引擎钩子互操作: 取出内部 Arc (如 logfile IndexHooks.cancel: Arc<AtomicBool>)。
    pub fn arc_cloned(&self) -> std::sync::Arc<std::sync::atomic::AtomicBool>;
}
```

进度计数不封装：`Arc<AtomicU64>` 直用（worker 累加 / UI 读），模块 doc 写明该范式。

### `SearchNav` —— 命中导航搭车（移植 `search.rs:79-` 起，语义零漂移）

升序命中表 + `partition_point` 定位 + 环绕跳转。公开面以 search.rs 现网为准（new/hits/total/current_line/jump_first_from/jump_next/jump_prev）。与作业同模块的理由：生于服务异步搜索作业，消费者同处（搜索栏 Enter 起作业，F3/Shift+F3 走导航）。

## 结构

- 新增 `src/job.rs`（文件头 `@author 十四叔` / `@date 2026/09/09`；模块头中文 doc 做什么/不做什么/为什么——「为什么」锚盘点簇B 结论 + 两条教训出处）
- `src/lib.rs`：`mod job;` + `pub use job::{AsyncJob, CancelFlag, CancelToken, SearchNav};`
- `src/update.rs` 最小加固（无 API 变化）：`CHECK_CACHE` 发布加代次戳，后发先至的旧检查不得覆盖新检查（盘点标记的潜在缺口）；全量迁移是**非目标**
- `examples/showcase.rs` 加「异步作业 (job)」演示卡（以用代测，小体量）：按钮起 ~300ms 假作业显示在途/完成；快速连点演示旧轮丢弃；另一按钮演示 panic 护栏（panic 作业 → Err 文案，不卡死）

## 消费者迁移（danqing-log 仓，前提满足才开工）

前提：danqing 已 push 且 **log 在途批次（async-open + text-selection）已提交**（其改动覆盖 open.rs/selection.rs，与本迁移同文件，不可并行）。

| log 文件 | 处置 |
|---|---|
| `search.rs` | 删 AsyncJob + SearchNav 本地实现 → `danqing::{AsyncJob, SearchNav}`；字段过滤/正则搜索两消费者调用点直通 |
| `open.rs` | OpenJob 保留（OpenOutcome/OpenKind/path 是产品语义）：内部 AsyncJob 换框架版；`run_catched` 删除 → `launch_catched`；cancel 原子量换 CancelToken（drop 即取消由令牌承载，OpenJob 手写 Drop 删除）；IndexHooks 互操作走 `CancelFlag::arc_cloned` |

- 硬判据：log 既有测试**零改动或近零改动全绿**（search 6 + open 7 的行为保持是迁移正确的证据，同簇A mixer 8 测试先例）
- pomodoro / clipboard / xirang：不动

## 测试策略

- 移植：search.rs 的 AsyncJob/SearchNav 测试全搬（代次防乱序/invalidate/环绕跳转/首跳定位），改写为新 API 等价断言
- 新增：`launch_catched` panic → Err 交付且 poll 可达（不卡 Loading）；CancelToken drop 置位 + cancel() 显式置位 + `arc_cloned` 互操作同一旗标；两连 launch 旧轮丢弃（open.rs 两连开语义上提）
- update.rs 加固：代次戳测试（旧代次 publish 被拒）
- 三件套：`cargo fmt` + `cargo clippy -- -D warnings` + `cargo test --lib --tests`；log 迁移后同三件套

## 验收标准

- [ ] `danqing::{AsyncJob, CancelToken, CancelFlag, SearchNav}` 经 lib.rs 可用
- [ ] 移植 + 新增测试全绿；三件套绿
- [ ] showcase 演示卡可实机触发三姿势（完成/旧轮丢弃/panic 护栏）（人工）
- [ ] （前提满足后）log 迁移：既有测试全绿，`AsyncJob`/`SearchNav`/`run_catched` 本地实现零残留
- [ ] 两仓分别提交，message 注明关联；danqing 先 push

## 边界

- job.rs 纯逻辑：只用 std（thread/Mutex/Atomic），不碰 winit/wgpu，不读 wall-clock
- 公开 API 一律经 `src/lib.rs` re-export；注释/文档一律中文
- 未获用户指示不 commit/push；联动改动两仓分别提交、message 注明关联
- 不做投机泛化：clipboard watcher 范式 / 通用进度封装 / 线程池，明确不收
