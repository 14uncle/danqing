//! @author 十四叔
//! @date 2026/09/13

//! 平台能力查询。
//!
//! 目前只有一件: 当前进程是不是跑在**已打包** (MSIX) 环境里。

/// 当前进程是否运行在**已打包**环境里 (MSIX / 商店版)。
///
/// 判据是 Win32 的 `GetCurrentPackageFullName`: 未打包时它返回
/// `APPMODEL_ERROR_NO_PACKAGE` (15700); 已打包时返回
/// `ERROR_INSUFFICIENT_BUFFER` (122) 并回填名字长度 ——
/// 传空缓冲区一问即知, 不必真把那个名字取出来。
///
/// **为什么是运行时判断而不是编译期开关**: 商店版与便携版是**同一个二进制**。
/// 加 feature 分区会立刻产生「手上这个包是哪个构建」的混淆 (danqing-pomodoro 的
/// freemium 三版本构建就是这种复杂度的前车之鉴)。而包标识本来就是个**运行时事实**,
/// 那就按运行时问。
///
/// **典型用途**: 已打包的应用由平台代管更新, 不应再去查 GitHub Releases —— 那会把
/// 商店用户导向站外获取安装包, 还可能让他装成便携版, 反而更乱。见
/// `danqing-log` 的 `src/app_update.rs`。
///
/// 非 Windows 平台恒为 `false` —— 这一问只对 MSIX 有意义。
#[cfg(windows)]
pub fn is_packaged() -> bool {
    use windows_sys::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;

    /// 进程没有包标识。
    const APPMODEL_ERROR_NO_PACKAGE: u32 = 15_700;

    let mut length: u32 = 0;
    // SAFETY: 只传一个待回填的长度指针与空缓冲区 —— 这正是该 API 约定的
    // 「问需要多大」用法, 它不写入任何缓冲区。
    let rc = unsafe { GetCurrentPackageFullName(&mut length, std::ptr::null_mut()) };
    rc != APPMODEL_ERROR_NO_PACKAGE
}

#[cfg(not(windows))]
pub fn is_packaged() -> bool {
    false
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(windows)]
    fn a_plain_test_binary_is_not_packaged() {
        // `cargo test` 跑的是裸 exe, 没有包标识 —— 这是可断言的事实。
        // 它同时把最大的风险挡在外面: 判据取反 (把「没打包」认成「打包了」
        // 会让便携版静默不再检查更新)。
        assert!(!super::is_packaged(), "裸 exe 不该被判为已打包");
    }

    #[test]
    #[cfg(not(windows))]
    fn is_packaged_is_always_false_off_windows() {
        assert!(!super::is_packaged());
    }
}
