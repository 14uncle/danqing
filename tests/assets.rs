//! @author 十四叔
//! @date 2026/07/19

//! 集成测试:验证仓库 `assets/` 下的视觉资产已就位。

use std::path::PathBuf;

fn assets_dir() -> PathBuf {
    PathBuf::from("assets")
}

#[test]
fn logo_svg_exists() {
    let path = assets_dir().join("logo").join("logo.svg");
    assert!(path.exists(), "LOGO SVG 源文件应存在: {}", path.display());
    let meta = std::fs::metadata(&path).unwrap();
    assert!(
        meta.len() > 0,
        "LOGO SVG 源文件不应为空: {}",
        path.display()
    );
}

#[test]
fn logo_pngs_exist() {
    let logo_dir = assets_dir().join("logo");
    for size in [16_u32, 24, 32, 48, 256] {
        let path = logo_dir.join(format!("logo_{size}.png"));
        assert!(path.exists(), "LOGO PNG 应存在: {}", path.display());
        let meta = std::fs::metadata(&path).unwrap();
        assert!(meta.len() > 0, "LOGO PNG 不应为空: {}", path.display());
    }
}

#[test]
fn logo_ico_exists() {
    let path = assets_dir().join("logo").join("logo.ico");
    assert!(path.exists(), "logo.ico 应存在: {}", path.display());
    let meta = std::fs::metadata(&path).unwrap();
    assert!(meta.len() > 0, "logo.ico 不应为空");
}

#[test]
fn background_images_exist() {
    let bg_dir = assets_dir().join("background");
    for name in ["gradient.png", "glow.png", "noise.png"] {
        let path = bg_dir.join(name);
        assert!(path.exists(), "背景图应存在: {}", path.display());
        let meta = std::fs::metadata(&path).unwrap();
        assert!(meta.len() > 0, "背景图不应为空: {}", path.display());
    }
}

#[test]
fn ofl_sans_font_exists() {
    let path = assets_dir().join("fonts").join("ofl-sans.ttf");
    assert!(path.exists(), "内嵌黑体应存在: {}", path.display());
    let meta = std::fs::metadata(&path).unwrap();
    assert!(meta.len() > 0, "内嵌黑体不应为空");
    assert!(
        meta.len() <= 3 * 1024 * 1024,
        "内嵌黑体应控制在 3 MB 以内, 实际 {} 字节",
        meta.len()
    );
}

#[test]
fn ofl_license_exists() {
    let path = assets_dir().join("fonts").join("OFL.txt");
    assert!(path.exists(), "OFL 许可文件应存在: {}", path.display());
    let meta = std::fs::metadata(&path).unwrap();
    assert!(meta.len() > 0, "OFL 许可文件不应为空");
}
