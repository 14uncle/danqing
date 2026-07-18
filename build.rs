//! @author 十四叔
//! @date 2026/07/17

//! 构建脚本:下载 M1 内嵌回退字体(OFL 许可)到 OUT_DIR。
//!
//! 仓库不提交任何字体二进制(spec 边界);首次构建时下载,之后复用
//! OUT_DIR 缓存(cargo clean 后重新下载)。字体:ZCOOL XiaoWei,
//! SIL OFL 许可,来源 google/fonts(jsdelivr 镜像)。

use std::{env, fs, path::Path, path::PathBuf, process::Command};

/// 下载镜像(按序尝试)。
const URLS: &[&str] = &[
    "https://cdn.jsdelivr.net/gh/google/fonts@main/ofl/zcoolxiaowei/ZCOOLXiaoWei-Regular.ttf",
    "https://fastly.jsdelivr.net/gh/google/fonts@main/ofl/zcoolxiaowei/ZCOOLXiaoWei-Regular.ttf",
    "https://gcore.jsdelivr.net/gh/google/fonts@main/ofl/zcoolxiaowei/ZCOOLXiaoWei-Regular.ttf",
];

/// 期望字节数(2026-07-16 下载核验)。
const EXPECTED_SIZE: u64 = 6_313_808;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR 未设置"));
    let dest = out_dir.join("fallback-font.ttf");

    if !dest.exists() {
        download(&dest);
    }

    // 完整性校验:字节数 + TrueType 魔数
    let data = fs::read(&dest).expect("回退字体读取失败");
    assert_eq!(
        data.len() as u64,
        EXPECTED_SIZE,
        "回退字体大小不符(上游可能已更新,请同步 build.rs 的 EXPECTED_SIZE)"
    );
    assert_eq!(
        &data[..4],
        &[0x00, 0x01, 0x00, 0x00],
        "回退字体不是有效的 TrueType 文件"
    );
    println!("cargo:rerun-if-changed=build.rs");
}

/// 依次尝试镜像下载;curl.exe(Win10+ 自带)优先,PowerShell 兜底。
fn download(dest: &Path) {
    for url in URLS {
        if try_download("curl.exe", url, dest) || try_download("powershell", url, dest) {
            println!("cargo:warning=回退字体下载成功: {url}");
            return;
        }
        let _ = fs::remove_file(dest);
    }
    panic!(
        "回退字体下载失败:所有镜像均不可用。\n\
         请手动下载 ZCOOLXiaoWei-Regular.ttf(OFL,google/fonts)放到:\n  {}",
        dest.display()
    );
}

fn try_download(tool: &str, url: &str, dest: &Path) -> bool {
    let status = if tool == "curl.exe" {
        Command::new(tool)
            .args(["-sfL", "--max-time", "300", "-o"])
            .arg(dest)
            .arg(url)
            .status()
    } else {
        Command::new(tool)
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri '{url}' -OutFile '{}' -TimeoutSec 300",
                    dest.display()
                ),
            ])
            .status()
    };
    status.map(|s| s.success()).unwrap_or(false)
        && dest.metadata().map(|m| m.len()).unwrap_or(0) == EXPECTED_SIZE
}
