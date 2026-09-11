//! @author 十四叔
//! @date 2026/07/31
//!
//! 构建脚本: 本仓库为库 crate, 不再 embed 图标。子 crate (库 build.rs 会被所有
//! 依赖它的产品继承) 若 embed 自己的图标会产生重复资源叶子 (.rsrc merge failure:
//! GROUP_ICON/ICON/VERSION), 导致 exe 内嵌图标二选一未定——产品据此各自
//! 通过各自 build.rs 嵌入本产品图标 (danqing-pomodoro / danqing-clipboard /
//! danqing-log 均已自定义)。

fn main() {
    // 库 crate 不注入任何 Windows 资源; 图标与版本信息由产品 build.rs 各自设置。
}
