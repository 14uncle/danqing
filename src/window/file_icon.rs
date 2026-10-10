//! @author 十四叔
//! @date 2026/10/10
//!
//! Shell 文件图标抽取 (Win32): 给产品列表提供条目图标 (RGBA8 像素,
//! 直接喂 `ImageBatch`) —— Drop Shelf T5 缺口肌肉 (造物寄生)。
//!
//! 路线: `SHGetFileInfoW` 取 HICON → `DrawIconEx` 画进 32bpp DIB 段 →
//! BGRA 转 RGBA。不用 `GetDIBits` 抠 HICON 自带位图 (那条路的 alpha 通道
//! 历史上不可靠); DIB 段路线对带 alpha 的现代图标保真。
//!
//! 线程语义: 可在任意线程调用 (纯 GDI, 无窗口依赖); 图标句柄用完即毁。

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDC, HGDIOBJ, ReleaseDC, SelectObject,
};
use windows::Win32::UI::Shell::{
    SHFILEINFOW, SHGFI_FLAGS, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_SMALLICON, SHGetFileInfoW,
};
use windows::Win32::UI::WindowsAndMessaging::{DI_NORMAL, DestroyIcon, DrawIconEx, HICON};
use windows::core::PCWSTR;

/// Shell 图标位图 (RGBA8, 行优先, 上到下, 非预乘)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellIcon {
    /// RGBA8 像素 (长度 = width * height * 4)。
    pub rgba: Vec<u8>,
    /// 宽 (像素)。
    pub width: u32,
    /// 高 (像素)。
    pub height: u32,
}

/// 取文件/文件夹的 Shell 图标。
///
/// `size` = 期望边长: ≤16 取系统小图标 (16), 否则大图标 (32);
/// 系统给不出时返回 `None` (调用方自备占位图, 不在此代为涂鸦)。
pub fn file_icon(path: &Path, size: u32) -> Option<ShellIcon> {
    let icon = shell_hicon(path, size)?;
    // SAFETY: icon 是 SHGetFileInfoW 交出的有效句柄; 本函数内全权负责销毁。
    let result = unsafe { hicon_to_rgba(icon, size) };
    let _ = unsafe { DestroyIcon(icon) };
    result
}

/// 取 Shell 图标句柄 (小/大按 size 挑)。
fn shell_hicon(path: &Path, size: u32) -> Option<HICON> {
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut shfi = SHFILEINFOW::default();
    let flags: SHGFI_FLAGS = SHGFI_ICON
        | if size <= 16 {
            SHGFI_SMALLICON
        } else {
            SHGFI_LARGEICON
        };
    // SAFETY: wide 是双零终结宽串; shfi 由本帧持有, 函数只按 cb 填充。
    let ok = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            Default::default(),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            flags,
        )
    };
    if ok == 0 || shfi.hIcon.is_invalid() {
        None
    } else {
        Some(shfi.hIcon)
    }
}

/// HICON → RGBA8 (画进 32bpp DIB 段, 再翻通道)。
///
/// # Safety
/// `icon` 必须是有效图标句柄 (本函数不负责销毁)。
unsafe fn hicon_to_rgba(icon: HICON, want: u32) -> Option<ShellIcon> {
    // 系统图标实际边长: 小 16 / 大 32; want 只决定取哪档, 不强改系统尺寸。
    let (w, h): (u32, u32) = if want <= 16 { (16, 16) } else { (32, 32) };

    // SAFETY: 全程配对 —— GetDC/ReleaseDC, CreateCompatibleDC/DeleteDC,
    // CreateDIBSection/SelectObject(还原)/DeleteObject, 任一步失败走早退不留句柄。
    unsafe {
        let screen = GetDC(None);
        if screen.is_invalid() {
            return None;
        }
        let mem = CreateCompatibleDC(Some(screen));
        if mem.is_invalid() {
            let _ = ReleaseDC(None, screen);
            return None;
        }
        // 32bpp 自顶向下 DIB (biHeight 为负 = 上到下), 预置全零 = 未盖处 alpha 0。
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w as i32,
                biHeight: -(h as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let dib = CreateDIBSection(Some(screen), &bmi, DIB_RGB_COLORS, &mut bits, None, 0);
        let Ok(dib) = dib else {
            let _ = DeleteDC(mem);
            let _ = ReleaseDC(None, screen);
            return None;
        };
        if bits.is_null() {
            let _ = DeleteObject(HGDIOBJ(dib.0));
            let _ = DeleteDC(mem);
            let _ = ReleaseDC(None, screen);
            return None;
        }
        let old = SelectObject(mem, HGDIOBJ(dib.0));
        let drawn = DrawIconEx(mem, 0, 0, icon, w as i32, h as i32, 0, None, DI_NORMAL);
        let _ = SelectObject(mem, old);

        let out = if drawn.is_ok() {
            let bgra = std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize);
            Some(ShellIcon {
                rgba: bgra_to_rgba(bgra),
                width: w,
                height: h,
            })
        } else {
            None
        };

        let _ = DeleteObject(HGDIOBJ(dib.0));
        let _ = DeleteDC(mem);
        let _ = ReleaseDC(None, screen);
        out
    }
}

/// BGRA → RGBA 逐像素翻通道 (纯逻辑, 可单测; 长度 4 的倍数, 不足尾块原样丢弃)。
fn bgra_to_rgba(bgra: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bgra.len());
    for px in bgra.chunks_exact(4) {
        out.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 通道序: BGRA → RGBA, alpha 不动。
    #[test]
    fn bgra_to_rgba_swaps_red_blue_keeps_alpha() {
        // 两个像素: (B=1,G=2,R=3,A=4) (B=10,G=20,R=30,A=40)
        let bgra = [1u8, 2, 3, 4, 10, 20, 30, 40];
        assert_eq!(bgra_to_rgba(&bgra), vec![3, 2, 1, 4, 30, 20, 10, 40]);
    }

    /// 尺寸守恒: 每 4 字节进 4 字节出。
    #[test]
    fn bgra_to_rgba_preserves_length() {
        let bgra = vec![7u8; 16 * 16 * 4];
        assert_eq!(bgra_to_rgba(&bgra).len(), bgra.len());
    }

    /// 尾部残块 (不足 4 字节) 静默丢弃, 不 panic。
    #[test]
    fn bgra_to_rgba_drops_partial_tail() {
        assert_eq!(bgra_to_rgba(&[1, 2, 3, 4, 5]), vec![3, 2, 1, 4]);
        assert!(bgra_to_rgba(&[]).is_empty());
    }
}
