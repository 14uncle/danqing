---
name: windows-gnu-toolchain-lld-fix
description: "本机 Rust 构建必须用 windows-gnu + ld.lld + dlltool shim,否则链接失败或运行即崩"
metadata: 
  node_type: memory
  type: project
  originSessionId: 4cad717e-4f66-4be2-bc93-bc02c8f26405
---

danqing 项目所在机器(Win11,无 Visual Studio Build Tools,GitHub 直连被墙)的 Rust 环境要点:

1. **工具链**:`stable-x86_64-pc-windows-gnu`(rustup override 设在 F:\github\danqing)。msvc 不可用(无 link.exe;Git Bash 的 `/usr/bin/link` 是 GNU coreutils,会 shadow MSVC linker)。
2. **PATH 需要真正的 GNU binutils**(`~/.cargo/bin/as.exe` + `dlltool.exe`,2.46.1):rustc 为 raw-dylib 生成导入库时调用 PATH 上的 `dlltool.exe`;rustup 自带的 dlltool 依赖缺失的 GNU `as` 无法工作。**自研 shim(lld-link 格式)是错误路线**:其导入对象与 GNU 格式混排时,重复 DLL 的 `__IMPORT_DESCRIPTOR_*` 对象无法被链接器合并 → 部分导入 IAT 未填充 → **首次 stdio 输出(GetConsoleMode)/堆分配(GetProcessHeap)就访问违规,且毫无日志**(测试二进制正常与否取决于链接时符号竞争顺序,具随机性)。真 binutils 取自 MSYS2 清华 TUNA 镜像(GitHub 被墙):`mirrors.tuna.tsinghua.edu.cn/msys2/mingw/mingw64/mingw-w64-x86_64-binutils-*.pkg.tar.zst`;Git Bash tar 不支持 zstd,用 `python -m pip install zstandard` 解压提取 as.exe/dlltool.exe(无其他 DLL 依赖)。
3. **不需要 `-fuse-ld=lld`**(`.cargo/config.toml` 现已清空 rustflags):统一 GNU 格式导入对象后 GNU ld 正确处理;lld 也不合并 GNU 预编译导入库与 rustc 导入对象的重复描述符。
4. **症状回忆**:exe 启动零输出即 segfault,或仅测试二进制正常 → 查导入表(llvm-objdump -p)里目标函数是否在某描述符内 + 其 `__imp_` 槽是否被覆盖。

诊断工具在仓库 `tools/`:`minidbg.rs`(迷你调试器:打印模块加载+异常上下文+寄存器/栈/内存读取)、`linkwrap.rs`(链接器包装器:留存完整链接 argv/输出)、`dlltool-shim.rs`(历史遗留,已被真 binutils 取代,仅备查)。
