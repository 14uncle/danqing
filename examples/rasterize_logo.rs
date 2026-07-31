//! @author 十四叔
//! @date 2026/07/31
//!
//! SVG → PNG / ICO 栅格化工具。
//!
//! ```bash
//! cargo run --example rasterize_logo -- assets/logo/pomodoro.svg
//! ```
//!
//! 在 SVG 所在目录生成 256/48/32/24/16 px PNG + .ico。

use std::io::Write;
use std::path::PathBuf;

use resvg::tiny_skia::Pixmap;
use resvg::usvg::{Options, Tree};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run --example rasterize_logo -- <path/to/logo.svg>");

    let svg_data = std::fs::read_to_string(&svg_path)?;
    let tree = Tree::from_str(&svg_data, &Options::default())?;
    let svg_size = tree.size().width(); // square

    let sizes: &[u32] = &[256, 48, 32, 24, 16];
    let stem = svg_path.file_stem().unwrap().to_str().unwrap();
    let dir = svg_path.parent().unwrap();

    // — PNG per size —
    for &size in sizes {
        let mut pixmap = Pixmap::new(size, size).expect("create pixmap");
        let s = size as f32 / svg_size;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(s, s),
            &mut pixmap.as_mut(),
        );
        let png_path = dir.join(format!("{stem}_{size}.png"));
        pixmap.save_png(&png_path)?;
        println!("  → {}", png_path.display());
    }

    // — ICO: container 内嵌各尺寸 PNG —
    let ico_path = dir.join(format!("{stem}.ico"));
    let mut ico = IcoWriter::new();
    for &size in sizes {
        let png_bytes = std::fs::read(dir.join(format!("{stem}_{size}.png")))?;
        ico.add(size, png_bytes);
    }
    std::fs::write(&ico_path, ico.finish())?;
    println!("  → {}", ico_path.display());
    println!("done.");
    Ok(())
}

// ── minimal ICO writer (PNG-in-ICO) ──

struct IcoWriter {
    entries: Vec<(u32, Vec<u8>)>,
}

impl IcoWriter {
    fn new() -> Self {
        Self { entries: vec![] }
    }
    fn add(&mut self, size: u32, png: Vec<u8>) {
        self.entries.push((size, png));
    }
    fn finish(self) -> Vec<u8> {
        let n = self.entries.len() as u16;
        let header_sz = 6usize;
        let dir_sz = n as usize * 16;
        let mut offset = (header_sz + dir_sz) as u32;
        let mut out = Vec::new();

        // header
        out.write_all(&0u16.to_le_bytes()).unwrap();
        out.write_all(&1u16.to_le_bytes()).unwrap();
        out.write_all(&n.to_le_bytes()).unwrap();

        // directory
        for (size, png) in &self.entries {
            let w = if *size >= 256 { 0u8 } else { *size as u8 };
            out.write_all(&[w]).unwrap();
            out.write_all(&[w]).unwrap();
            out.write_all(&[0u8]).unwrap();
            out.write_all(&[0u8]).unwrap();
            out.write_all(&1u16.to_le_bytes()).unwrap();
            out.write_all(&32u16.to_le_bytes()).unwrap();
            let sz = png.len() as u32;
            out.write_all(&sz.to_le_bytes()).unwrap();
            out.write_all(&offset.to_le_bytes()).unwrap();
            offset += sz;
        }

        for (_, png) in self.entries {
            out.write_all(&png).unwrap();
        }
        out
    }
}
