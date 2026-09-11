//! @author 十四叔
//! @date 2026/09/09

//! 异步作业与命中导航 (纯逻辑): worker 线程 + tick 拾取 + 代次防乱序。
//!
//! 做什么:
//! - [`AsyncJob`]: launch 起工作线程, poll 心跳零成本拾取, 代次防旧轮晚到覆盖新轮;
//!   invalidate 外部失效 (换文件等在途作废由持有方显式调用)。
//! - [`AsyncJob::launch_catched`]: panic 护栏 —— worker panic 转 Err 交付, 不卡 Loading
//!   (仅 panic=unwind 构建生效; 农场 release 标准 profile 为 panic="abort", 该 profile
//!   下 worker panic 直接终止进程 —— 护栏守护 dev/test 与未设 abort 的消费者)。
//! - [`CancelToken`]/[`CancelFlag`]: drop 即取消的令牌对, worker 轮询早退。
//! - [`SearchNav`]: 升序命中表 + partition_point 定位 + 环绕跳转。
//!
//! 不做什么: 不含线程池/任务调度/async 运行时; 进度计数不封装 (`Arc<AtomicU64>`
//! 直用, worker 累加 / UI 读); 无时间语义 (不读 wall-clock); 只用 std
//! (thread/Mutex/Atomic), 不碰 winit/wgpu。
//!
//! 为什么: 该模式在农场重复发明 4 处 (danqing-log search.rs/open.rs 两消费者 +
//! clipboard revision 堂兄弟 + danqing update.rs 裸线程无代次防护), 2026-09-09
//! 下沉盘点 (docs/intent/framework-sinking.md 簇B) 裁定收编, spec: docs/specs/SPEC-job.md。
//! 两条教训内化为设施: panic 护栏 (async-open 评审 FYI→Required: detached 线程 panic
//! 曾让应用层永久 Loading) 与 drop 即取消 (OpenJob::drop 语义上提为类型)。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// 异步作业: launch 起工作线程, poll 心跳拾取, 代次防乱序覆盖
/// (两次快速发起: 旧轮晚到的结果被丢弃, 不覆盖新轮)。
pub struct AsyncJob<T> {
    done: Arc<Mutex<Option<(u64, T)>>>,
    rev: Arc<AtomicU64>,
    /// 发起代次 (单调递增)。
    generation: u64,
    seen_rev: u64,
}

impl<T: Send + 'static> Default for AsyncJob<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + 'static> AsyncJob<T> {
    pub fn new() -> Self {
        Self {
            done: Arc::new(Mutex::new(None)),
            rev: Arc::new(AtomicU64::new(0)),
            generation: 0,
            seen_rev: 0,
        }
    }

    /// 发起新一轮 (代次 +1)。在途旧轮不取消, 其晚到结果按乱序丢弃。
    /// `work` 可能 panic 的场景用 `launch_catched` 护栏 (Result 类型专用)。
    pub fn launch<F>(&mut self, work: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        self.generation += 1;
        let generation = self.generation;
        let done = Arc::clone(&self.done);
        let rev = Arc::clone(&self.rev);
        std::thread::spawn(move || {
            let out = work();
            *done.lock().unwrap() = Some((generation, out));
            rev.fetch_add(1, Ordering::Release);
        });
    }

    /// 拾取完成结果 (仅当属于最新一轮; 每帧调用, 无结果零成本)。
    ///
    /// 结果槽为覆盖式单槽: poll 须每帧调用 —— 若长期不 poll (或两轮完成落在
    /// 同一 poll 间隔内且旧轮后写槽), 旧轮晚到可能覆盖未拾取的新轮结果
    /// (代价: 该次结果丢失, 代次机制保证状态不错乱)。
    pub fn poll(&mut self) -> Option<T> {
        let r = self.rev.load(Ordering::Acquire);
        if r == self.seen_rev {
            return None;
        }
        self.seen_rev = r;
        let (generation, out) = self.done.lock().unwrap().take()?;
        if generation != self.generation {
            return None; // 乱序完成的旧轮
        }
        Some(out)
    }

    /// 使在途作业失效 (代次 +1, 不发起新工作): 旧轮晚到的结果按乱序丢弃。
    /// 跨作业失效场景: async-open 换入新文件后, 旧文件上的在途 filter/search
    /// 结果不得贴到新文件 (review C1); AsyncJob 自身的代次只覆盖「同 job 连续
    /// launch」, 换文件这种外部失效须由持有方显式调用。
    pub fn invalidate(&mut self) {
        self.generation += 1;
    }
}

impl<T: Send + 'static, E: Send + 'static> AsyncJob<Result<T, E>> {
    /// 同 [`launch`](AsyncJob::launch), 但 worker panic 经 catch_unwind 转为
    /// `Err(on_panic())` 交付: 应用层 poll 照常收到结果, 不卡 Loading。
    /// (教训内化: async-open 评审 FYI→Required —— detached 线程 panic 曾致永久 Loading。)
    ///
    /// 仅 panic=unwind 构建生效: 农场 release 标准 profile 为 `panic = "abort"`,
    /// 该 profile 下 worker panic 直接终止进程 —— 护栏守护 dev/test 与未设 abort 的消费者。
    /// `on_panic` 自身必须不 panic (建议预构廉价错误值): recovery 路径上二次 panic
    /// 会让 done 永不写入, 恰是护栏要消灭的「永久 Loading」。
    pub fn launch_catched<F, P>(&mut self, work: F, on_panic: P)
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
        P: FnOnce() -> E + Send + 'static,
    {
        self.launch(move || {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
                .unwrap_or_else(|_| Err(on_panic()))
        });
    }
}

/// 取消令牌 (持有端): drop 或 [`CancelToken::cancel`] 置位;
/// worker 侧持 [`CancelFlag`] 轮询早退。令牌不 Clone (单一持有方)。
#[derive(Debug)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    /// 创建令牌对: (持有端, worker 端)。
    pub fn pair() -> (Self, CancelFlag) {
        let flag = Arc::new(AtomicBool::new(false));
        (
            Self {
                flag: Arc::clone(&flag),
            },
            CancelFlag { flag },
        )
    }

    /// 显式提前取消 (drop 同样取消)。
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

impl Drop for CancelToken {
    /// drop 即取消: 旗标置位, worker 扫描循环早退 (OpenJob::drop 语义上提为类型)。
    fn drop(&mut self) {
        self.cancel();
    }
}

/// 取消旗标 (worker 端): 可 Clone 进多个工作单元, 共享同一旗标。
#[derive(Debug, Clone)]
pub struct CancelFlag {
    flag: Arc<AtomicBool>,
}

impl CancelFlag {
    /// worker 循环内的早退查询。
    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    /// 引擎钩子互操作: 取出内部 Arc (如 logfile `IndexHooks.cancel: Arc<AtomicBool>`)。
    pub fn arc_cloned(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.flag)
    }
}

/// 命中导航: 升序命中表 (文件行号) + 环绕跳转。纯逻辑, 与渲染解耦供单测。
///
/// 生于服务异步搜索作业 (搜索栏 Enter 起 [`AsyncJob`], F3/Shift+F3 走本导航),
/// 与消费者同处一模块。
#[derive(Debug)]
pub struct SearchNav {
    hits: Arc<Vec<u64>>,
    /// 总命中数 (含 cap 外未收集部分, 如实展示)。
    total: u64,
    /// 当前命中下标 (未定位 = None)。
    current: Option<usize>,
}

impl SearchNav {
    pub fn new(hits: Arc<Vec<u64>>, total: u64) -> Self {
        Self {
            hits,
            total,
            current: None,
        }
    }

    pub fn hits(&self) -> &Arc<Vec<u64>> {
        &self.hits
    }

    pub fn total(&self) -> u64 {
        self.total
    }

    /// 当前命中行号。
    pub fn current_line(&self) -> Option<u64> {
        self.current.map(|i| self.hits[i])
    }

    /// 应用后首跳: 第一个 >= from 的命中; 全部小于 from 则环绕回首条。
    pub fn jump_first_from(&mut self, from: u64) -> Option<u64> {
        if self.hits.is_empty() {
            return None;
        }
        let idx = self.hits.partition_point(|&h| h < from);
        let idx = if idx >= self.hits.len() { 0 } else { idx };
        self.current = Some(idx);
        Some(self.hits[idx])
    }

    /// 下一命中 (环绕)。
    pub fn jump_next(&mut self) -> Option<u64> {
        if self.hits.is_empty() {
            return None;
        }
        let i = match self.current {
            None => 0,
            Some(i) => (i + 1) % self.hits.len(),
        };
        self.current = Some(i);
        Some(self.hits[i])
    }

    /// 上一命中 (环绕)。
    pub fn jump_prev(&mut self) -> Option<u64> {
        if self.hits.is_empty() {
            return None;
        }
        let i = match self.current {
            None => self.hits.len() - 1,
            Some(0) => self.hits.len() - 1,
            Some(i) => i - 1,
        };
        self.current = Some(i);
        Some(self.hits[i])
    }

    /// (第 k 条, 收集到 n 条) 1-based, 供状态栏; 未定位 = None。
    pub fn position(&self) -> Option<(usize, usize)> {
        self.current.map(|i| (i + 1, self.hits.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// 轮询拾取 helper: 1s 上限 (源头 search.rs 同款时序纪律)。
    fn poll_until<T: Send + 'static>(job: &mut AsyncJob<T>) -> Option<T> {
        for _ in 0..1000 {
            if let Some(v) = job.poll() {
                return Some(v);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        None
    }

    #[test]
    fn async_job_delivers_latest_generation() {
        let mut job: AsyncJob<u64> = AsyncJob::new();
        job.launch(|| 42);
        assert_eq!(poll_until(&mut job), Some(42), "1s 内必交付");
        assert_eq!(job.poll(), None, "结果只取一次");
    }

    #[test]
    fn async_job_invalidate_discards_inflight_result() {
        // review C1 机制钉: launch 后不 poll, invalidate, 结果完成后 poll 必须 None。
        let mut job: AsyncJob<u64> = AsyncJob::new();
        job.launch(|| 7);
        job.invalidate();
        assert_eq!(poll_until(&mut job), None, "失效轮的结果必须被丢弃");
        // invalidate 不影响后续新一轮交付。
        job.launch(|| 8);
        assert_eq!(poll_until(&mut job), Some(8), "新一轮照常交付");
    }

    #[test]
    fn async_job_rapid_relaunch_discards_older_round() {
        // 两连开 (open.rs 语义上提): 慢旧轮晚到不得覆盖新轮。
        let mut job: AsyncJob<u64> = AsyncJob::new();
        job.launch(|| {
            std::thread::sleep(Duration::from_millis(50));
            1 // 旧轮: 慢
        });
        job.launch(|| 2); // 新轮: 快, 先完成
        assert_eq!(poll_until(&mut job), Some(2), "新轮先交付");
        // 旧轮晚到: 代次不符, 丢弃; 不得有第二个结果。
        std::thread::sleep(Duration::from_millis(120));
        assert_eq!(job.poll(), None, "旧轮晚到必须丢弃");
    }

    #[test]
    fn launch_catched_delivers_panic_as_err() {
        // panic 护栏: worker panic → Err 交付, poll 可达 (不卡 Loading)。
        let mut job: AsyncJob<Result<u64, String>> = AsyncJob::new();
        job.launch_catched(|| panic!("boom"), || "panicked".to_string());
        assert_eq!(
            poll_until(&mut job),
            Some(Err("panicked".to_string())),
            "panic 须转为 Err 交付"
        );
    }

    #[test]
    fn launch_catched_passes_ok_through() {
        let mut job: AsyncJob<Result<u64, String>> = AsyncJob::new();
        job.launch_catched(|| Ok(9), || "unused".to_string());
        assert_eq!(poll_until(&mut job), Some(Ok(9)), "正常作业不受护栏影响");
    }

    #[test]
    fn cancel_token_drop_sets_flag() {
        // drop 即取消 (OpenJob::drop 语义上提为类型)。
        let (token, flag) = CancelToken::pair();
        assert!(!flag.is_cancelled(), "初始未取消");
        drop(token);
        assert!(flag.is_cancelled(), "drop 后须置位");
    }

    #[test]
    fn cancel_token_explicit_cancel_shares_one_flag() {
        let (token, flag) = CancelToken::pair();
        token.cancel();
        assert!(flag.is_cancelled());
        // flag 可 Clone 进多个工作单元; arc_cloned 互操作同一旗标。
        assert!(flag.clone().is_cancelled());
        assert!(flag.arc_cloned().load(Ordering::Relaxed));
    }

    // ---- SearchNav (移植 danqing-log search.rs, 语义零漂移) ----

    #[test]
    fn nav_first_from_positions() {
        let hits = Arc::new(vec![10, 20, 30]);
        let mut nav = SearchNav::new(hits, 3);
        assert_eq!(nav.jump_first_from(0), Some(10), "从头");
        assert_eq!(nav.jump_first_from(10), Some(10), "恰在命中");
        assert_eq!(nav.jump_first_from(15), Some(20), "中间取后");
        assert_eq!(nav.jump_first_from(31), Some(10), "越过末尾环绕回首");
    }

    #[test]
    fn nav_next_prev_wrap() {
        let mut nav = SearchNav::new(Arc::new(vec![5, 9]), 2);
        assert_eq!(nav.jump_next(), Some(5), "未定位先跳首条");
        assert_eq!(nav.jump_next(), Some(9));
        assert_eq!(nav.jump_next(), Some(5), "末尾环绕");
        assert_eq!(nav.jump_prev(), Some(9), "反向环绕");
        assert_eq!(nav.position(), Some((2, 2)));
    }

    #[test]
    fn nav_empty_hits() {
        let mut nav = SearchNav::new(Arc::new(vec![]), 0);
        assert_eq!(nav.jump_first_from(0), None);
        assert_eq!(nav.jump_next(), None);
        assert_eq!(nav.jump_prev(), None);
        assert_eq!(nav.position(), None);
    }
}
