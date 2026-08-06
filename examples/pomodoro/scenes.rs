//! @author 十四叔
//! @date 2026/07/23

//! 场景资产与调色板常量 —— 由 tools/export-scenes.py 生成, 勿手改。

use danqing::{Color, ScenePalette, SceneSpec};

/// POC 场景清单 (数组顺序即 ◀/▶ 切换顺序)。
pub const SCENES: [SceneSpec; 9] = [
    SceneSpec {
        name: "篝火",
        image: "assets/scenes/bonfire.png",
        palette: ScenePalette {
            base: Color::from_srgb8(26, 15, 10),
            accent: Color::from_srgb8(255, 159, 67),
            text_primary: Color::from_srgb8(240, 230, 215),
            text_secondary: Color::from_srgb8(195, 180, 165),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(208, 217, 228),
            backdrop_dark: Color::from_srgb8(8, 14, 12),
        },
    },
    SceneSpec {
        name: "海",
        image: "assets/scenes/sea.png",
        palette: ScenePalette {
            base: Color::from_srgb8(168, 221, 232),
            accent: Color::from_srgb8(15, 55, 75),
            text_primary: Color::from_srgb8(255, 255, 255),
            text_secondary: Color::from_srgb8(200, 210, 220),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(255, 235, 204),
            backdrop_dark: Color::from_srgb8(0, 16, 16),
        },
    },
    SceneSpec {
        name: "雨",
        image: "assets/scenes/rain.png",
        palette: ScenePalette {
            base: Color::from_srgb8(82, 95, 107),
            accent: Color::from_srgb8(127, 179, 217),
            text_primary: Color::from_srgb8(242, 246, 249),
            text_secondary: Color::from_srgb8(195, 205, 213),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(193, 207, 210),
            backdrop_dark: Color::from_srgb8(25, 40, 37),
        },
    },
    SceneSpec {
        name: "山",
        image: "assets/scenes/mountain.png",
        palette: ScenePalette {
            base: Color::from_srgb8(86, 80, 115),
            accent: Color::from_srgb8(232, 192, 122),
            text_primary: Color::from_srgb8(245, 241, 250),
            text_secondary: Color::from_srgb8(205, 198, 218),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(170, 130, 147),
            backdrop_dark: Color::from_srgb8(1, 19, 27),
        },
    },
    SceneSpec {
        name: "森林",
        image: "assets/scenes/forest.png",
        palette: ScenePalette {
            base: Color::from_srgb8(50, 72, 59),
            accent: Color::from_srgb8(172, 198, 158),
            text_primary: Color::from_srgb8(240, 246, 240),
            text_secondary: Color::from_srgb8(186, 201, 187),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(169, 202, 193),
            backdrop_dark: Color::from_srgb8(0, 9, 4),
        },
    },
    SceneSpec {
        name: "星夜",
        image: "assets/scenes/starry.png",
        palette: ScenePalette {
            base: Color::from_srgb8(22, 26, 52),
            accent: Color::from_srgb8(255, 224, 160),
            text_primary: Color::from_srgb8(246, 247, 255),
            text_secondary: Color::from_srgb8(185, 192, 220),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(231, 242, 247),
            backdrop_dark: Color::from_srgb8(13, 21, 30),
        },
    },
    SceneSpec {
        name: "雪原",
        image: "assets/scenes/snow.png",
        palette: ScenePalette {
            base: Color::from_srgb8(200, 210, 225),
            accent: Color::from_srgb8(80, 120, 160),
            text_primary: Color::from_srgb8(240, 245, 255),
            text_secondary: Color::from_srgb8(180, 195, 215),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(206, 222, 245),
            backdrop_dark: Color::from_srgb8(10, 52, 98),
        },
    },
    SceneSpec {
        name: "沙漠",
        image: "assets/scenes/sand.png",
        palette: ScenePalette {
            base: Color::from_srgb8(180, 130, 80),
            accent: Color::from_srgb8(220, 180, 100),
            text_primary: Color::from_srgb8(255, 245, 230),
            text_secondary: Color::from_srgb8(200, 170, 130),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(255, 207, 141),
            backdrop_dark: Color::from_srgb8(52, 20, 0),
        },
    },
    SceneSpec {
        name: "竹林",
        image: "assets/scenes/bamboo.png",
        palette: ScenePalette {
            base: Color::from_srgb8(30, 60, 50),
            accent: Color::from_srgb8(100, 180, 130),
            text_primary: Color::from_srgb8(220, 240, 230),
            text_secondary: Color::from_srgb8(150, 180, 165),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(238, 255, 250),
            backdrop_dark: Color::from_srgb8(5, 38, 30),
        },
    },
];

#[cfg(test)]
mod tests {
    //! 对比度护栏: 与 tools/export-scenes.py 生成期护栏同规则,
    //! 防止 scenes.rs 被手改后护栏静默失效 (spec: 大字 ≥3:1, 控件 ≥4:1)。
    use super::*;
    use danqing::{composite_over, contrast_ratio};

    /// 大字 (倒计时) 对场景背景极值的最低对比度。
    const DISPLAY_MIN: f32 = 3.0;
    /// 控件文字对玻璃合成底的最低对比度。
    const CONTROL_MIN: f32 = 4.0;

    #[test]
    fn all_scenes_pass_contrast_guards() {
        assert_eq!(SCENES.len(), 6, "沉浸世界应有 6 个场景");
        for spec in &SCENES {
            let p = &spec.palette;
            for (label, backdrop) in [
                ("backdrop_light", p.backdrop_light),
                ("backdrop_dark", p.backdrop_dark),
            ] {
                let display = contrast_ratio(p.text_primary, backdrop);
                assert!(
                    display >= DISPLAY_MIN,
                    "{}: 大字 vs {label} = {display:.2} < {DISPLAY_MIN}",
                    spec.name
                );
                let glass = composite_over(p.surface, backdrop);
                let control = contrast_ratio(p.text_primary, glass);
                assert!(
                    control >= CONTROL_MIN,
                    "{}: 控件文字 vs 玻璃({label}) = {control:.2} < {CONTROL_MIN}",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn scene_images_are_unique_and_named() {
        for (i, a) in SCENES.iter().enumerate() {
            for b in &SCENES[i + 1..] {
                assert_ne!(a.image, b.image, "场景图路径不应重复");
                assert_ne!(a.name, b.name, "场景名不应重复");
            }
        }
    }
}
