//! @author 十四叔
//! @date 2026/07/17

//! dlltool shim:把 GNU dlltool 调用翻译为 rust-lld 的导入库生成。
//!
//! 背景:windows-gnu 工具链下,rustc 为 `raw-dylib` 导入生成导入库时会
//! 调用 PATH 上的 `dlltool.exe`;rustup 自带的 dlltool 又依赖缺失的 GNU as。
//! 本 shim 用 rust-lld(lld-link 的 -def/-implib)直接产出等价 COFF 导入库。
//!
//! 用法:编译为 dlltool.exe 放到 PATH(如 ~/.cargo/bin)。rustc 调用形式:
//!   dlltool.exe -d <def> -D <dll> -l <out.lib> -m i386:x86-64 -f --64 \
//!               --no-leading-underscore --temp-prefix <p>

use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut def: Option<String> = None;
    let mut out: Option<String> = None;
    let mut dll: Option<String> = None;
    let mut machine = "x64".to_string();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--input-def" => {
                def = args.get(i + 1).cloned();
                i += 2;
            }
            "-l" | "--output-lib" => {
                out = args.get(i + 1).cloned();
                i += 2;
            }
            "-D" | "--dllname" => {
                dll = args.get(i + 1).cloned();
                i += 2;
            }
            "-m" => {
                machine = match args.get(i + 1).map(String::as_str) {
                    Some("i386") => "x86".into(),
                    Some("aarch64") | Some("arm64") => "arm64".into(),
                    _ => "x64".into(),
                };
                i += 2;
            }
            // 带值但忽略的参数
            "-f" | "--as" | "--as-flags" | "--temp-prefix" => {
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let (Some(def), Some(out)) = (def, out) else {
        eprintln!("dlltool-shim: 缺少 -d/-l 参数,原始参数: {args:?}");
        exit(2);
    };

    // 取证日志(默认关闭;设 DLLTOOL_SHIM_LOG=1 开启):记录调用参数并留存 def 文件
    if std::env::var_os("DLLTOOL_SHIM_LOG").is_some()
        && let Ok(mut log) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(r"F:\github\danqing\target\dlltool-calls.log")
    {
        use std::io::Write;
        let _ = writeln!(log, "def={def} out={out} dll={dll:?} machine={machine}");
        if let Ok(content) = std::fs::read_to_string(&def) {
            let dir = std::path::Path::new(r"F:\github\danqing\target\dlltool-defs");
            let _ = std::fs::create_dir_all(dir);
            let name = dll.clone().unwrap_or_else(|| "unknown.dll".into());
            let _ = std::fs::write(dir.join(format!("{name}.def")), content);
        }
    }

    // lld 会把 -out 的 DLL 名写进导入库;必须用 -D 给出的真实 DLL 名
    // (如 uxtheme.dll),否则生成的导入库会指向不存在的假 DLL。
    let dummy_dll = match dll {
        Some(name) => PathBuf::from(&out).with_file_name(name),
        None => PathBuf::from(out.replace(".lib", ".dll")),
    };

    let status = Command::new(find_rust_lld())
        .arg("-flavor")
        .arg("link")
        .arg(format!("-def:{def}"))
        .arg("-dll")
        .arg("-noentry")
        .arg(format!("-out:{}", dummy_dll.display()))
        .arg(format!("-implib:{out}"))
        .arg(format!("-machine:{machine}"))
        .status()
        .unwrap_or_else(|e| {
            eprintln!("dlltool-shim: 无法启动 rust-lld: {e}");
            exit(2);
        });

    exit(status.code().unwrap_or(2));
}

/// 定位 rust-lld:优先环境变量,其次查询 rustc sysroot,最后回退到已知路径。
fn find_rust_lld() -> PathBuf {
    if let Ok(p) = std::env::var("RUST_LLD") {
        return PathBuf::from(p);
    }
    if let Ok(output) = Command::new("rustc").arg("--print").arg("sysroot").output()
        && output.status.success()
    {
        let sysroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let candidate = PathBuf::from(sysroot)
            .join("lib")
            .join("rustlib")
            .join("x86_64-pc-windows-gnu")
            .join("bin")
            .join("rust-lld.exe");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from(r"C:\Users\gwhun\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\rust-lld.exe")
}
