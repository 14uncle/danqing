//! @author 十四叔
//! @date 2026/07/17

//! 迷你调试器:以 DEBUG_ONLY_THIS_PROCESS 启动目标进程,
//! 打印加载的模块与首/二次异常(异常码、地址、所属模块)。
//!
//! 用于在没有 WinDbg 的环境里定位启动即崩(access violation)的模块。
//!
//! 用法: minidbg.exe <目标exe> [参数...]

#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::process::exit;

type BOOL = i32;
type DWORD = u32;
type WORD = u16;
type HANDLE = *mut core::ffi::c_void;
type LPVOID = *mut core::ffi::c_void;
type LPCWSTR = *const u16;
type LPWSTR = *mut u16;

const DEBUG_ONLY_THIS_PROCESS: DWORD = 0x2;
const INFINITE: DWORD = 0xFFFFFFFF;
const DBG_CONTINUE: DWORD = 0x0001_0002;
const DBG_EXCEPTION_NOT_HANDLED: DWORD = 0x8001_0001;

const EXCEPTION_DEBUG_EVENT: DWORD = 1;
const CREATE_THREAD_DEBUG_EVENT: DWORD = 2;
const CREATE_PROCESS_DEBUG_EVENT: DWORD = 3;
const EXIT_THREAD_DEBUG_EVENT: DWORD = 4;
const EXIT_PROCESS_DEBUG_EVENT: DWORD = 5;
const LOAD_DLL_DEBUG_EVENT: DWORD = 6;
#[allow(dead_code)]
const UNLOAD_DLL_DEBUG_EVENT: DWORD = 7;
const OUTPUT_DEBUG_STRING_EVENT: DWORD = 8;

#[repr(C)]
struct STARTUPINFOW {
    cb: DWORD,
    lpReserved: LPWSTR,
    lpDesktop: LPWSTR,
    lpTitle: LPWSTR,
    dwX: DWORD,
    dwY: DWORD,
    dwXSize: DWORD,
    dwYSize: DWORD,
    dwXCountChars: DWORD,
    dwYCountChars: DWORD,
    dwFillAttribute: DWORD,
    dwFlags: DWORD,
    wShowWindow: WORD,
    cbReserved2: WORD,
    lpReserved2: *mut u8,
    hStdInput: HANDLE,
    hStdOutput: HANDLE,
    hStdError: HANDLE,
}

#[repr(C)]
struct PROCESS_INFORMATION {
    hProcess: HANDLE,
    hThread: HANDLE,
    dwProcessId: DWORD,
    dwThreadId: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EXCEPTION_RECORD {
    ExceptionCode: DWORD,
    ExceptionFlags: DWORD,
    ExceptionRecord: *mut core::ffi::c_void,
    ExceptionAddress: LPVOID,
    NumberParameters: DWORD,
    __pad: DWORD,
    ExceptionInformation: [usize; 15],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EXCEPTION_DEBUG_INFO {
    ExceptionRecord: EXCEPTION_RECORD,
    dwFirstChance: DWORD,
    __pad: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CREATE_PROCESS_DEBUG_INFO {
    hFile: HANDLE,
    hProcess: HANDLE,
    hThread: HANDLE,
    lpBaseOfImage: LPVOID,
    dwDebugInfoFileOffset: DWORD,
    nDebugInfoSize: DWORD,
    lpThreadLocalBase: LPVOID,
    lpStartAddress: LPVOID,
    lpImageName: LPVOID,
    fUnicode: WORD,
    __pad: WORD,
    __pad2: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LOAD_DLL_DEBUG_INFO {
    hFile: HANDLE,
    lpBaseOfDll: LPVOID,
    dwDebugInfoFileOffset: DWORD,
    nDebugInfoSize: DWORD,
    lpImageName: LPVOID,
    fUnicode: WORD,
    __pad: WORD,
    __pad2: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EXIT_PROCESS_DEBUG_INFO {
    dwExitCode: DWORD,
    __pad: DWORD,
}

#[repr(C)]
union DEBUG_EVENT_UNION {
    Exception: EXCEPTION_DEBUG_INFO,
    CreateProcessInfo: CREATE_PROCESS_DEBUG_INFO,
    LoadDll: LOAD_DLL_DEBUG_INFO,
    ExitProcess: EXIT_PROCESS_DEBUG_INFO,
    __align: [u64; 20],
}

#[repr(C)]
struct DEBUG_EVENT {
    dwDebugEventCode: DWORD,
    dwProcessId: DWORD,
    dwThreadId: DWORD,
    __pad: DWORD,
    u: DEBUG_EVENT_UNION,
}

unsafe extern "system" {
    fn CreateProcessW(
        app: LPCWSTR,
        cmd: LPWSTR,
        proc_attr: *mut core::ffi::c_void,
        thread_attr: *mut core::ffi::c_void,
        inherit: BOOL,
        flags: DWORD,
        env: *mut core::ffi::c_void,
        cwd: LPCWSTR,
        si: *mut STARTUPINFOW,
        pi: *mut PROCESS_INFORMATION,
    ) -> BOOL;
    fn WaitForDebugEvent(event: *mut DEBUG_EVENT, timeout: DWORD) -> BOOL;
    fn ContinueDebugEvent(pid: DWORD, tid: DWORD, status: DWORD) -> BOOL;
    fn ReadProcessMemory(
        process: HANDLE,
        base: *const core::ffi::c_void,
        buf: *mut core::ffi::c_void,
        size: usize,
        read: *mut usize,
    ) -> BOOL;
    fn TerminateProcess(process: HANDLE, code: u32) -> BOOL;
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn GetThreadContext(thread: HANDLE, ctx: *mut CONTEXT) -> BOOL;
}

const CONTEXT_AMD64: u32 = 0x0010_0000;
const CONTEXT_FULL: u32 = CONTEXT_AMD64 | 0x1 | 0x2 | 0x4;

/// x64 CONTEXT(对齐 16,布局到 Rip 为止,其余填充)。
#[repr(C, align(16))]
struct CONTEXT {
    p_home: [u64; 6],
    context_flags: u32,
    mx_csr: u32,
    seg: [u16; 6],
    eflags: u32,
    dr: [u64; 6],
    rax: u64,
    rcx: u64,
    rdx: u64,
    rbx: u64,
    rsp: u64,
    rbp: u64,
    rsi: u64,
    rdi: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rip: u64,
    _rest: [u8; 1024],
}

/// 打印线程寄存器与栈上的候选返回地址。
unsafe fn dump_context(process: HANDLE, thread: HANDLE, modules: &[(usize, usize, String)]) {
    let mut ctx: CONTEXT = unsafe { std::mem::zeroed() };
    ctx.context_flags = CONTEXT_FULL;
    if unsafe { GetThreadContext(thread, &mut ctx) } == 0 {
        println!("  GetThreadContext 失败");
        return;
    }
    println!("  RIP={:#x} RSP={:#x} RBP={:#x}", ctx.rip, ctx.rsp, ctx.rbp);
    println!(
        "  RAX={:#x} RCX={:#x} RDX={:#x} RBX={:#x}",
        ctx.rax, ctx.rcx, ctx.rdx, ctx.rbx
    );
    let mut stack = [0u8; 512];
    if unsafe { read_mem(process, ctx.rsp as usize, &mut stack) } {
        println!("  栈上候选返回地址:");
        for (i, q) in stack.chunks_exact(8).enumerate() {
            let val = u64::from_le_bytes(q.try_into().unwrap()) as usize;
            if let Some((base, _, name)) = modules
                .iter()
                .find(|(base, size, _)| val >= *base && val < *base + *size)
            {
                println!(
                    "    [rsp+{:#04x}] {val:#014x} -> {} (+{:#x})",
                    i * 8,
                    if name.is_empty() { "?" } else { name },
                    val - base
                );
            }
        }
    }
}

/// 读取被调试进程内存。
unsafe fn read_mem(process: HANDLE, addr: usize, buf: &mut [u8]) -> bool {
    let mut read = 0usize;
    unsafe {
        ReadProcessMemory(
            process,
            addr as *const core::ffi::c_void,
            buf.as_mut_ptr() as *mut core::ffi::c_void,
            buf.len(),
            &mut read,
        ) != 0
            && read == buf.len()
    }
}

/// 读取被调试进程中的模块名。lpImageName 实际指向一个
/// 类似 UNICODE_STRING 的结构(偏移 8 处为字符串缓冲区指针)。
unsafe fn read_string(process: HANDLE, addr: usize, _unicode: bool) -> String {
    if addr == 0 {
        return String::new();
    }
    let mut hdr = [0u8; 16];
    if unsafe { read_mem(process, addr, &mut hdr) } {
        let buf_ptr = u64::from_le_bytes(hdr[8..16].try_into().unwrap()) as usize;
        if buf_ptr != 0 {
            let mut buf = [0u8; 512];
            if unsafe { read_mem(process, buf_ptr, &mut buf) } {
                let wide: Vec<u16> = buf
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .take_while(|&c| c != 0)
                    .collect();
                let s = String::from_utf16_lossy(&wide);
                if !s.is_empty() {
                    return s;
                }
            }
        }
    }
    String::new()
}

/// 读取模块的 SizeOfImage(从 PE 头)。
unsafe fn module_size(process: HANDLE, base: usize) -> usize {
    let mut buf = [0u8; 4];
    // e_lfanew 在 0x3c
    if !unsafe { read_mem(process, base + 0x3c, &mut buf) } {
        return 0;
    }
    let pe = u32::from_le_bytes(buf) as usize;
    // SizeOfImage: NT头 + 4(签名) + 20(文件头) + 56(可选头偏移)
    if !unsafe { read_mem(process, base + pe + 4 + 20 + 56, &mut buf) } {
        return 0;
    }
    u32::from_le_bytes(buf) as usize
}

fn main() {
    let mut argv = std::env::args().skip(1);
    let Some(target) = argv.next() else {
        eprintln!("用法: minidbg <目标exe> [--read <模块内RVA,十六进制>] [参数...]");
        exit(2);
    };
    // 可选:崩溃时读取主模块 base+RVA 处的 32 字节
    let mut read_rva: Option<usize> = None;
    let rest: Vec<String> = argv.collect();
    let mut i = 0;
    let mut child_args: Vec<String> = Vec::new();
    while i < rest.len() {
        if rest[i] == "--read"
            && let Some(v) = rest.get(i + 1)
        {
            read_rva = usize::from_str_radix(v.trim_start_matches("0x"), 16).ok();
            i += 2;
            continue;
        }
        child_args.push(rest[i].clone());
        i += 1;
    }
    let cmdline = format!("\"{target}\" {}", child_args.join(" "));
    let mut wide: Vec<u16> = OsStr::new(&cmdline)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut si: STARTUPINFOW = unsafe { std::mem::zeroed() };
    si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };

    let ok = unsafe {
        CreateProcessW(
            std::ptr::null(),
            wide.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            DEBUG_ONLY_THIS_PROCESS,
            std::ptr::null_mut(),
            std::ptr::null(),
            &mut si,
            &mut pi,
        )
    };
    if ok == 0 {
        eprintln!("CreateProcessW 失败: {}", std::io::Error::last_os_error());
        exit(1);
    }

    let mut modules: Vec<(usize, usize, String)> = Vec::new(); // (base, size, name)
    let mut process_handle: HANDLE = std::ptr::null_mut();
    let mut main_thread: HANDLE = std::ptr::null_mut();
    let mut done = false;

    while !done {
        let mut ev: DEBUG_EVENT = unsafe { std::mem::zeroed() };
        if unsafe { WaitForDebugEvent(&mut ev, INFINITE) } == 0 {
            eprintln!("WaitForDebugEvent 失败");
            break;
        }
        let mut status = DBG_CONTINUE;
        match ev.dwDebugEventCode {
            CREATE_PROCESS_DEBUG_EVENT => {
                let info = unsafe { ev.u.CreateProcessInfo };
                process_handle = info.hProcess;
                main_thread = info.hThread;
                let base = info.lpBaseOfImage as usize;
                let size = unsafe { module_size(info.hProcess, base) };
                let name = unsafe {
                    read_string(
                        info.hProcess,
                        info.lpImageName as usize,
                        info.fUnicode != 0,
                    )
                };
                println!("[module] {base:#014x} +{size:#x}  {name} (主模块)");
                modules.push((base, size, name));
                unsafe {
                    CloseHandle(info.hFile);
                }
            }
            LOAD_DLL_DEBUG_EVENT => {
                let info = unsafe { ev.u.LoadDll };
                let base = info.lpBaseOfDll as usize;
                let size = unsafe { module_size(process_handle, base) };
                let name = unsafe {
                    read_string(process_handle, info.lpImageName as usize, info.fUnicode != 0)
                };
                println!("[module] {base:#014x} +{size:#x}  {name}");
                modules.push((base, size, name));
                unsafe {
                    CloseHandle(info.hFile);
                }
            }
            EXCEPTION_DEBUG_EVENT => {
                let info = unsafe { ev.u.Exception };
                let rec = &info.ExceptionRecord;
                let addr = rec.ExceptionAddress as usize;
                let owner = modules
                    .iter()
                    .find(|(base, size, _)| addr >= *base && addr < *base + *size);
                let owner_desc = owner
                    .map(|(base, _, name)| format!("{} (+{:#x})", name, addr - base))
                    .unwrap_or_else(|| "未知模块".into());
                println!(
                    "[exception] code={:#010x} addr={addr:#014x} {} chance  in {}",
                    rec.ExceptionCode,
                    if info.dwFirstChance != 0 { "1st" } else { "2nd" },
                    owner_desc
                );
                if info.dwFirstChance != 0 {
                    if rec.ExceptionCode == 0xC000_0005 {
                        unsafe {
                            dump_context(process_handle, main_thread, &modules);
                        }
                        if let (Some(rva), Some(base)) =
                            (read_rva, modules.first().map(|m| m.0))
                        {
                            let mut buf = [0u8; 32];
                            if unsafe { read_mem(process_handle, base + rva, &mut buf) } {
                                println!(
                                    "  内存[{:#x}+rva]: {}",
                                    base,
                                    buf.iter()
                                        .map(|b| format!("{b:02x}"))
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                );
                            }
                        }
                    }
                    status = DBG_EXCEPTION_NOT_HANDLED;
                } else {
                    // 二次机会:进程必死,终止之
                    unsafe {
                        TerminateProcess(process_handle, rec.ExceptionCode);
                    }
                }
            }
            EXIT_PROCESS_DEBUG_EVENT => {
                let info = unsafe { ev.u.ExitProcess };
                println!("[exit] code={:#x}", info.dwExitCode);
                done = true;
            }
            OUTPUT_DEBUG_STRING_EVENT => {}
            CREATE_THREAD_DEBUG_EVENT | EXIT_THREAD_DEBUG_EVENT => {}
            _ => {}
        }
        unsafe {
            ContinueDebugEvent(ev.dwProcessId, ev.dwThreadId, status);
        }
    }
}
