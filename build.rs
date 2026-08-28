//! @author 十四叔
//! @date 2026/07/31
//!
//! 构建脚本: Windows 平台嵌入 exe 图标 (仅图标，不含 FileDescription/ProductName，
//! 因为库 crate 的 build.rs 会被所有产品继承)。

fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/logo/logo.ico");
        res.compile().unwrap();
    }
}
