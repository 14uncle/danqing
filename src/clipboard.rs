//! @author 十四叔
//! @date 2026/09/10
//!
//! 剪贴板读取原语: 抽象 trait + 真实系统源。
//!
//! 下沉自 danqing-clipboard `monitor.rs` (2026-09-10, 簇E/E4)。
//! 监听器 (隐私判定/落库/轮询循环) 是产品业务, 本模块只提供读取原语:
//! `ClipSource` trait (序列号 + 多格式读取) + `SystemClipSource` (arboard + Win32)。
//! 平台代码驻地豁免见 CLAUDE.md (剪贴板域无法归入窗口域)。

/// 剪贴板图片 (RGBA 原始字节)。
#[derive(Clone)]
pub struct ClipImage {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

/// 剪贴板读取抽象: 真实实现走 arboard + Win32 序列号; 测试用假源。
pub trait ClipSource {
    /// 剪贴板序列号, 变化即内容变了。
    fn sequence(&mut self) -> u32;
    /// 读纯文本; 剪贴板非文本或读失败返回 None。
    fn text(&mut self) -> Option<String>;
    /// 读 HTML 富文本; 非 HTML 或读失败返回 None。
    fn html(&mut self) -> Option<String> {
        None
    }
    /// 读文件路径列表 (CF_HDROP); 非文件剪贴板或读失败返回 None。
    fn files(&mut self) -> Option<Vec<String>> {
        None
    }
    /// 读剪贴板图片 (RGBA); 非图片或读失败返回 None。
    fn image(&mut self) -> Option<ClipImage> {
        None
    }
}

/// DIB 来源的 RGBA 常带全 0 alpha (通道未定义), ALPHA_BLENDING 下整图隐形。
/// 全 0 则视为不透明通道缺失 → 全部置 255; 存在非零 alpha 则尊重原通道。
pub fn ensure_opaque_alpha(rgba: &mut [u8]) {
    if !rgba.is_empty() && rgba.chunks_exact(4).all(|px| px[3] == 0) {
        for px in rgba.chunks_exact_mut(4) {
            px[3] = 255;
        }
    }
}

/// 真实剪贴板源 (arboard 读文本, Win32 读序列号/文件列表)。
pub struct SystemClipSource {
    clipboard: arboard::Clipboard,
}

impl SystemClipSource {
    /// 失败返回 arboard 错误 (框架不引 anyhow 进生产依赖)。
    pub fn new() -> Result<Self, arboard::Error> {
        Ok(Self {
            clipboard: arboard::Clipboard::new()?,
        })
    }
}

impl ClipSource for SystemClipSource {
    #[cfg(target_os = "windows")]
    fn sequence(&mut self) -> u32 {
        // SAFETY: 无参只读系统调用, 无线程安全风险。
        unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() }
    }

    /// 非 Windows 无剪贴板序列号概念 (本模块的 Win32 监听仅 Windows 产品使用)。
    #[cfg(not(target_os = "windows"))]
    fn sequence(&mut self) -> u32 {
        0
    }

    fn text(&mut self) -> Option<String> {
        self.clipboard.get_text().ok()
    }

    fn html(&mut self) -> Option<String> {
        self.clipboard.get().html().ok()
    }

    fn image(&mut self) -> Option<ClipImage> {
        let img = self.clipboard.get().image().ok()?;
        let mut rgba = img.bytes.into_owned();
        ensure_opaque_alpha(&mut rgba);
        Some(ClipImage {
            width: img.width,
            height: img.height,
            rgba,
        })
    }

    /// Win32 读取 CF_HDROP 文件列表; 非 Windows 走 trait 默认 (返 None)。
    #[cfg(target_os = "windows")]
    fn files(&mut self) -> Option<Vec<String>> {
        use windows_sys::Win32::System::DataExchange::{
            CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        };
        use windows_sys::Win32::System::Memory::{GlobalLock, GlobalUnlock};
        use windows_sys::Win32::System::Ole::CF_HDROP;
        use windows_sys::Win32::UI::Shell::DragQueryFileW;

        unsafe {
            // 检查剪贴板是否有 CF_HDROP 数据
            if IsClipboardFormatAvailable(CF_HDROP as u32) == 0 {
                return None;
            }

            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return None;
            }
            let result = (|| -> Option<Vec<String>> {
                let h_data = GetClipboardData(CF_HDROP as u32);
                if h_data.is_null() {
                    return None;
                }
                let h_drop = GlobalLock(h_data);
                if h_drop.is_null() {
                    return None;
                }
                let mut paths = Vec::new();
                let mut i = 0u32;
                loop {
                    // DragQueryFileW(cch=0) 返回所需字符数, **不含** null 终止符,
                    // 缓冲区必须 +1, 否则写入时末位字符被 null 挤掉 (路径丢尾巴:
                    // 「.png」入库成「.pn」, 2026-08-16 查库实证)
                    let len = DragQueryFileW(h_drop, i, std::ptr::null_mut(), 0);
                    if len == 0 {
                        break;
                    }
                    let mut buf = vec![0u16; len as usize + 1];
                    let written = DragQueryFileW(h_drop, i, buf.as_mut_ptr(), buf.len() as u32);
                    if written > 0 {
                        // 去掉末尾 null
                        let end = (written as usize).min(buf.len());
                        if let Ok(s) = String::from_utf16(&buf[..end]) {
                            paths.push(s);
                        }
                    }
                    i += 1;
                }
                let _ = GlobalUnlock(h_data);
                Some(paths)
            })();
            let _ = CloseClipboard();
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_opaque_alpha_fixes_all_zero() {
        let mut rgba = vec![100, 150, 200, 0, 50, 60, 70, 0];
        ensure_opaque_alpha(&mut rgba);
        assert_eq!(
            rgba,
            vec![100, 150, 200, 255, 50, 60, 70, 255],
            "全 0 alpha 置 255"
        );
    }

    #[test]
    fn ensure_opaque_alpha_respects_real_alpha() {
        let mut rgba = vec![100, 150, 200, 0, 50, 60, 70, 128];
        let orig = rgba.clone();
        ensure_opaque_alpha(&mut rgba);
        assert_eq!(rgba, orig, "存在非零 alpha 则不动");
    }

    #[test]
    fn ensure_opaque_alpha_edge_cases() {
        let mut empty: Vec<u8> = vec![];
        ensure_opaque_alpha(&mut empty); // 不 panic
        let mut odd = vec![1u8, 2, 3];
        ensure_opaque_alpha(&mut odd); // 非 4 倍数长度不 panic
        assert_eq!(odd, vec![1, 2, 3]);
    }

    /// 真实系统源: 无 GUI 环境 (CI) 下 new 可能失败或读失败, 只验证不 panic。
    /// 覆盖全部读取方法: files 无 CF_HDROP 时走 IsClipboardFormatAvailable 早退,
    /// 至少锁定「不 panic、不泄漏」。
    #[test]
    #[cfg(target_os = "windows")]
    fn system_clip_source_does_not_panic() {
        if let Ok(mut src) = SystemClipSource::new() {
            let _ = src.sequence();
            let _ = src.text();
            let _ = src.html();
            let _ = src.files();
            let _ = src.image();
        }
    }
}
