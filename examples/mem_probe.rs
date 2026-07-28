//! @author 十四叔
//! @date 2026/07/23

//! 内存探针：逐步构造框架对象，打印 Rust 堆存活字节变化。

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

fn mb() -> f64 {
    LIVE_BYTES.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0
}

fn main() {
    println!("启动:            {:.1} MB", mb());
    let font = danqing::Font::load();
    println!("Font::load 后:   {:.1} MB (来源: {})", mb(), font.source());
    drop(font);
    println!("drop 字体后:     {:.1} MB", mb());
    let sans = danqing::Font::embedded_sans();
    println!("内嵌黑体加载后:  {:.1} MB", mb());
    drop(sans);
    println!("drop 内嵌黑体后: {:.1} MB", mb());
}
