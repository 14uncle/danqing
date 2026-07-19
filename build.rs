//! @author 十四叔
//! @date 2026/07/19

//! 构建脚本:
//! 1. 下载 M1 内嵌回退字体(OFL 许可)到 OUT_DIR。
//! 2. 生成阶段 1 视觉资产(LOGO、背景图)到 OUT_DIR;仓库不提交二进制。

use image::{ImageBuffer, Luma, Rgba, RgbaImage};
use std::{env, fs, io::Write, path::Path, path::PathBuf, process::Command};

/// 下载镜像(按序尝试)。
const FONT_URLS: &[&str] = &[
    "https://cdn.jsdelivr.net/gh/google/fonts@main/ofl/zcoolxiaowei/ZCOOLXiaoWei-Regular.ttf",
    "https://fastly.jsdelivr.net/gh/google/fonts@main/ofl/zcoolxiaowei/ZCOOLXiaoWei-Regular.ttf",
    "https://gcore.jsdelivr.net/gh/google/fonts@main/ofl/zcoolxiaowei/ZCOOLXiaoWei-Regular.ttf",
];

/// 期望字节数(2026-07-16 下载核验)。
const EXPECTED_SIZE: u64 = 6_313_808;

/// 品牌强调色 #3B82F6,与 `LightTheme::accent` 保持一致。
const BRAND_ACCENT: Rgba<u8> = Rgba([59, 130, 246, 255]);

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR 未设置"));
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR 未设置"));

    // 1. 回退字体
    let font_dest = out_dir.join("fallback-font.ttf");
    if !font_dest.exists() {
        download_font(&font_dest);
    }
    verify_font(&font_dest);

    // 2. 视觉资产(LOGO、背景图)
    generate_assets(&manifest_dir);

    println!("cargo:rerun-if-changed=build.rs");
}

// ---------------------------------------------------------------------------
// 字体下载与校验
// ---------------------------------------------------------------------------

fn download_font(dest: &Path) {
    for url in FONT_URLS {
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

fn verify_font(dest: &Path) {
    let data = fs::read(dest).expect("回退字体读取失败");
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
}

// ---------------------------------------------------------------------------
// 视觉资产生成
// ---------------------------------------------------------------------------

fn generate_assets(manifest_dir: &Path) {
    let logo_dir = manifest_dir.join("assets").join("logo");
    let bg_dir = manifest_dir.join("assets").join("background");
    fs::create_dir_all(&logo_dir).expect("创建 logo 目录失败");
    fs::create_dir_all(&bg_dir).expect("创建 background 目录失败");

    // LOGO PNG 多尺寸
    let sizes = [16_u32, 24, 32, 48, 256];
    let mut png_entries: Vec<(u32, Vec<u8>)> = Vec::with_capacity(sizes.len());
    for size in sizes {
        let img = draw_dq_logo(size);
        let mut buf = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .expect("写入 PNG 失败");
        let path = logo_dir.join(format!("logo_{size}.png"));
        fs::write(&path, &buf).expect("写入 logo PNG 失败");
        png_entries.push((size, buf));
    }

    // ICO:包含 16/24/32/48/256 多层
    let ico_path = logo_dir.join("logo.ico");
    let ico_data = build_ico(&png_entries);
    fs::write(&ico_path, &ico_data).expect("写入 logo.ico 失败");

    // 背景图
    generate_gradient(&bg_dir.join("gradient.png"), 512, 512);
    generate_noise(&bg_dir.join("noise.png"), 256, 256);
}

/// 绘制 "dq" 字母组合 LOGO。
///
/// 设计说明:
/// - 背景透明,便于不同场景叠加。
/// - "d" 位于左侧:左侧竖线 + 右侧半圆,开口向右。
/// - "q" 位于右侧:完整圆 + 右下竖线尾巴。
/// - 使用品牌强调色,几何风格,适配小尺寸 favicon。
fn draw_dq_logo(size: u32) -> RgbaImage {
    let mut img = ImageBuffer::from_pixel(size, size, Rgba([0, 0, 0, 0]));
    if size < 4 {
        return img;
    }

    let s = size as f32;
    let r = s * 0.24; // 字母半径
    let stroke = (s * 0.16).max(2.0); // 竖线粗细,至少 2px
    let gap = s * 0.10; // 两个字母之间的间距
    let cy = s * 0.5; // 垂直居中

    // d: 圆心偏左,右侧半圆 + 左侧竖线
    let cx_d = s * 0.5 - r - gap * 0.5;
    draw_filled_semicircle_right(&mut img, cx_d, cy, r, BRAND_ACCENT);
    draw_filled_rect(
        &mut img,
        cx_d - r - stroke * 0.5,
        cy - r,
        stroke,
        r * 2.0,
        BRAND_ACCENT,
    );

    // q: 圆心偏右,完整圆 + 右下竖线(从圆底向下延伸)
    let cx_q = s * 0.5 + r + gap * 0.5;
    draw_filled_circle(&mut img, cx_q, cy, r, BRAND_ACCENT);
    draw_filled_rect(
        &mut img,
        cx_q + r - stroke * 0.5,
        cy,
        stroke,
        r * 1.35,
        BRAND_ACCENT,
    );

    img
}

/// 绘制右半圆(圆心左侧被截断,用于字母 "d")。
fn draw_filled_semicircle_right(img: &mut RgbaImage, cx: f32, cy: f32, r: f32, color: Rgba<u8>) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    let r2 = r * r;
    let x0 = cx.floor().max(0.0) as i32;
    let x1 = (cx + r).ceil().min(w as f32) as i32;
    let y0 = (cy - r).floor().max(0.0) as i32;
    let y1 = (cy + r).ceil().min(h as f32) as i32;

    for y in y0..y1 {
        for x in x0..x1 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx >= 0.0 && dx * dx + dy * dy <= r2 {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn draw_filled_circle(img: &mut RgbaImage, cx: f32, cy: f32, r: f32, color: Rgba<u8>) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    let r2 = r * r;
    let x0 = (cx - r).floor().max(0.0) as i32;
    let x1 = (cx + r).ceil().min(w as f32 - 1.0) as i32;
    let y0 = (cy - r).floor().max(0.0) as i32;
    let y1 = (cy + r).ceil().min(h as f32 - 1.0) as i32;

    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r2 {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn draw_filled_rect(img: &mut RgbaImage, x: f32, y: f32, w: f32, h: f32, color: Rgba<u8>) {
    let (img_w, img_h) = (img.width() as i32, img.height() as i32);
    let x0 = x.floor().max(0.0) as i32;
    let x1 = (x + w).ceil().min(img_w as f32) as i32;
    let y0 = y.floor().max(0.0) as i32;
    let y1 = (y + h).ceil().min(img_h as f32) as i32;

    for y in y0..y1 {
        for x in x0..x1 {
            img.put_pixel(x as u32, y as u32, color);
        }
    }
}

/// 将多层 PNG 组装成 ICO 文件。
///
/// ICO 格式支持直接嵌入 PNG 数据(Vista+),每层目录指向对应 PNG 偏移。
fn build_ico(entries: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = entries.len() as u16;
    let header_size = 6 + entries.len() * 16;
    let mut data =
        Vec::with_capacity(header_size + entries.iter().map(|e| e.1.len()).sum::<usize>());

    // ICONDIR
    data.write_all(&[0, 0]).unwrap(); // 保留
    data.write_all(&[1, 0]).unwrap(); // 类型: 图标
    data.write_all(&count.to_le_bytes()).unwrap();

    let mut offset = header_size as u32;
    for (size, png) in entries {
        let width = if *size >= 256 { 0 } else { *size as u8 };
        let height = width;
        let color_count = 0;
        let reserved = 0;
        let planes = 1_u16;
        let bit_count = 32_u16;
        let data_size = png.len() as u32;

        data.push(width);
        data.push(height);
        data.push(color_count);
        data.push(reserved);
        data.write_all(&planes.to_le_bytes()).unwrap();
        data.write_all(&bit_count.to_le_bytes()).unwrap();
        data.write_all(&data_size.to_le_bytes()).unwrap();
        data.write_all(&offset.to_le_bytes()).unwrap();

        offset += data_size;
    }

    for (_, png) in entries {
        data.write_all(png).unwrap();
    }

    data
}

/// 生成浅色毛玻璃风格渐变背景。
fn generate_gradient(path: &Path, width: u32, height: u32) {
    let mut img = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));
    let top = (245.0, 247.0, 250.0);
    let bottom = (225.0, 234.0, 247.0);

    for y in 0..height {
        let t = y as f32 / (height as f32 - 1.0);
        let r = lerp(top.0, bottom.0, t) as u8;
        let g = lerp(top.1, bottom.1, t) as u8;
        let b = lerp(top.2, bottom.2, t) as u8;
        for x in 0..width {
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    img.save(path).expect("保存渐变背景失败");
}

/// 生成半透明灰度噪声纹理,用于叠加在背景上增加质感。
fn generate_noise(path: &Path, width: u32, height: u32) {
    let mut img = ImageBuffer::from_pixel(width, height, Luma([0_u8]));
    // 使用简单伪随机(不需要加密安全),种子固定保证可复现。
    let mut state: u32 = 0x1234_5678;
    for y in 0..height {
        for x in 0..width {
            state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let v = (state >> 24) as u8;
            img.put_pixel(x, y, Luma([v]));
        }
    }

    img.save(path).expect("保存噪声纹理失败");
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}
