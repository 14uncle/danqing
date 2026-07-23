//! @author 十四叔
//! @date 2026/07/23

//! 最小化窗口示例：空组件树 + 默认背景，用于隔离框架层内存基线。
//!
//! 带统计全局分配器：把 Rust 堆占用与原生层 (wgpu/驱动) 占用区分开。

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 统计当前存活 Rust 堆字节数的全局分配器。
struct CountingAllocator;

static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn main() {
    env_logger::init();
    // 后台线程周期性打印 Rust 堆存活字节, 与任务管理器的进程级数字对照。
    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let mb = LIVE_BYTES.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0;
            log::info!("Rust 堆存活: {mb:.1} MB");
        }
    });
    danqing::run(danqing::WindowConfig::default()).expect("运行窗口失败");
}
