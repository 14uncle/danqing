//! @author 十四叔
//! @date 2026/07/19

//! 集成测试:验证 build.rs 生成的视觉资产已就位。

use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("OUT_DIR"))
}

#[test]
fn logo_pngs_exist() {
    let logo_dir = out_dir().join("assets").join("logo");
    for size in [16_u32, 24, 32, 48, 256] {
        let path = logo_dir.join(format!("logo_{size}.png"));
        assert!(path.exists(), "LOGO PNG 应存在: {}", path.display());
        let meta = std::fs::metadata(&path).unwrap();
        assert!(meta.len() > 0, "LOGO PNG 不应为空: {}", path.display());
    }
}

#[test]
fn logo_ico_exists() {
    let path = out_dir().join("assets").join("logo").join("logo.ico");
    assert!(path.exists(), "logo.ico 应存在: {}", path.display());
    let meta = std::fs::metadata(&path).unwrap();
    assert!(meta.len() > 0, "logo.ico 不应为空");
}

#[test]
fn background_images_exist() {
    let bg_dir = out_dir().join("assets").join("background");
    for name in ["gradient.png", "noise.png"] {
        let path = bg_dir.join(name);
        assert!(path.exists(), "背景图应存在: {}", path.display());
        let meta = std::fs::metadata(&path).unwrap();
        assert!(meta.len() > 0, "背景图不应为空: {}", path.display());
    }
}
