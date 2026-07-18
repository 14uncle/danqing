//! @author 十四叔
//! @date 2026/07/17

//! 链接器包装器 v3:转发调用真实 mingw gcc 驱动,并把完整输出留存到文件。
//! 目的:捕获 ld 的符号解析过程(-y 跟踪),定位损坏导入库的来源。

use std::io::Write;
use std::process::{Command, Stdio, exit};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // 找到响应文件副本留存
    let mut tag = String::from("unknown");
    for a in &args {
        if let Some(rsp) = a.strip_prefix('@') {
            let _ = std::fs::copy(rsp, r"F:\github\danqing\target\linker-arguments.txt");
        }
        if a == "-o" {
            // -o 的下一个参数是输出名,用作日志标签
        }
    }
    if let Some(pos) = args.iter().position(|a| a == "-o")
        && let Some(out) = args.get(pos + 1)
    {
        if let Some(name) = std::path::Path::new(out).file_stem() {
            tag = name.to_string_lossy().into_owned();
        }
    }

    // 真实链接器;补 PATH 让 gcc 能找到 ld
    let self_contained = r"C:\Users\gwhun\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained";
    let real = format!(r"{self_contained}\x86_64-w64-mingw32-gcc.exe");
    let path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{self_contained};{path}");

    let output = Command::new(real)
        .args(&args)
        .env("PATH", new_path)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| {
            eprintln!("linkwrap: 无法启动真实链接器: {e}");
            exit(2);
        });

    // 留存输出
    let log_path = format!(r"F:\github\danqing\target\linker-out-{tag}.log");
    if let Ok(mut f) = std::fs::File::create(&log_path) {
        let _ = f.write_all(&output.stdout);
        let _ = f.write_all(&output.stderr);
    }
    // 同时转发,保持 cargo 可见
    use std::io::Write as _;
    let _ = std::io::stdout().write_all(&output.stdout);
    let _ = std::io::stderr().write_all(&output.stderr);

    exit(output.status.code().unwrap_or(2));
}
