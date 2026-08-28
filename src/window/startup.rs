//! @author 十四叔
//! @date 2026/08/25
//!
//! 开机启动：通过 Windows 注册表 Run 键管理自启动。
//!
//! 注册表路径：`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
//! 非 Windows 平台编译为空 (cfg gate), 与其他平台能力 (热键/托盘/前台) 一致。

use std::path::Path;

/// 注册表 Run 键路径。
#[cfg(target_os = "windows")]
const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// 开机启动操作错误。
#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    /// 注册表 I/O 错误。
    #[error("注册表操作失败：{0}")]
    Registry(#[from] std::io::Error),
    /// 路径校验失败 (非绝对路径)。
    #[error("无效路径：{0}")]
    InvalidPath(String),
}

/// 设置开机启动状态。
///
/// - `app_name`: 注册表值名 (如 "danqing-clipboard")
/// - `exe_path`: 可执行文件完整路径
/// - `enabled`: true = 写入注册表，false = 删除注册表项
///
/// 返回 `Ok(())` 表示操作成功，`Err` 表示注册表操作失败。
#[cfg(target_os = "windows")]
pub fn set_enabled(app_name: &str, exe_path: &str, enabled: bool) -> Result<(), StartupError> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(RUN_KEY_PATH, winreg::enums::KEY_WRITE)?;

    if enabled {
        // 校验路径：必须是绝对路径
        let path = Path::new(exe_path);
        if !path.is_absolute() {
            return Err(StartupError::InvalidPath(exe_path.to_string()));
        }
        run_key.set_value(app_name, &exe_path)?;
        log::info!("已设置开机启动：{app_name} = {exe_path}");
    } else {
        // 删除：忽略 "值不存在" 的错误
        match run_key.delete_value(app_name) {
            Ok(()) => {
                log::info!("已取消开机启动：{app_name}");
            }
            Err(e) => {
                // ERROR_FILE_NOT_FOUND (2) 表示值本就不存在，不算失败
                if e.raw_os_error() == Some(2) {
                    log::info!("开机启动本就未开启，无需取消：{app_name}");
                } else {
                    return Err(StartupError::Registry(e));
                }
            }
        }
    }
    Ok(())
}

/// 查询当前是否已设置开机启动。
///
/// - `app_name`: 注册表值名
///
/// 返回 `Ok(true)` 如果注册表项存在且值非空，`Ok(false)` 如果不存在。
#[cfg(target_os = "windows")]
pub fn is_enabled(app_name: &str) -> Result<bool, StartupError> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(RUN_KEY_PATH, winreg::enums::KEY_READ)?;

    match run_key.get_value::<String, _>(app_name) {
        Ok(val) => Ok(!val.is_empty()),
        Err(e) => {
            // ERROR_FILE_NOT_FOUND (2) 表示值不存在
            if e.raw_os_error() == Some(2) {
                Ok(false)
            } else {
                Err(StartupError::Registry(e))
            }
        }
    }
}

// ── 非 Windows 平台：no-op ──

/// 非 Windows 平台：无操作，始终返回成功。
#[cfg(not(target_os = "windows"))]
pub fn set_enabled(_app_name: &str, _exe_path: &str, _enabled: bool) -> Result<(), StartupError> {
    Ok(())
}

/// 非 Windows 平台：不支持开机启动，始终返回 false。
#[cfg(not(target_os = "windows"))]
pub fn is_enabled(_app_name: &str) -> Result<bool, StartupError> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用注册表值名，避免污染生产值。
    const TEST_APP_NAME: &str = "danqing_test_startup";

    #[test]
    fn set_and_query_enabled() {
        // 初始状态：应该是未启用
        let initial = is_enabled(TEST_APP_NAME).expect("查询失败");
        assert!(!initial, "测试前应无注册表项");

        // 设置启用 (用一个存在的 exe 路径)
        let exe = std::env::current_exe().expect("获取当前 exe 路径失败");
        let exe_str = exe.to_string_lossy().to_string();
        set_enabled(TEST_APP_NAME, &exe_str, true).expect("设置启用失败");

        // 查询：应该是已启用
        let enabled = is_enabled(TEST_APP_NAME).expect("查询失败");
        assert!(enabled, "设置后应为已启用");

        // 设置禁用
        set_enabled(TEST_APP_NAME, &exe_str, false).expect("设置禁用失败");

        // 查询：应该是未启用
        let disabled = is_enabled(TEST_APP_NAME).expect("查询失败");
        assert!(!disabled, "禁用后应为未启用");
    }

    #[test]
    fn disable_nonexistent_is_ok() {
        // 对一个不存在的值执行禁用，应该不报错
        let result = set_enabled("danqing_nonexistent_key_12345", "C:\\fake.exe", false);
        assert!(result.is_ok(), "禁用不存在的值不应报错：{result:?}");
    }

    #[test]
    fn reject_relative_path() {
        let result = set_enabled(TEST_APP_NAME, "relative\\path.exe", true);
        assert!(result.is_err(), "相对路径应被拒绝");
    }
}
