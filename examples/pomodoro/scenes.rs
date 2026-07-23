//! @author 十四叔
//! @date 2026/07/23

//! 场景资产与调色板常量 —— 由 tools/export-scenes.py 生成, 勿手改。

use danqing::{Color, ScenePalette, SceneSpec};

/// POC 场景清单 (数组顺序即 ◀/▶ 切换顺序)。
pub const SCENES: [SceneSpec; 4] = [
    SceneSpec {
        name: "篝火",
        image: "assets/scenes/bonfire.png",
        palette: ScenePalette {
            base: Color::from_srgb8(26, 15, 10),
            accent: Color::from_srgb8(255, 159, 67),
            text_primary: Color::from_srgb8(250, 244, 235),
            text_secondary: Color::from_srgb8(199, 184, 166),
            surface: Color::rgba(0.0, 0.0, 0.0, 0.25),
            surface_input: Color::rgba(0.0, 0.0, 0.0, 0.38),
            backdrop_light: Color::from_srgb8(127, 86, 44),
            backdrop_dark: Color::from_srgb8(64, 39, 19),
        },
    },
    SceneSpec {
        name: "海",
        image: "assets/scenes/sea.png",
        palette: ScenePalette {
            base: Color::from_srgb8(168, 221, 232),
            accent: Color::from_srgb8(12, 74, 110),
            text_primary: Color::from_srgb8(8, 32, 48),
            text_secondary: Color::from_srgb8(60, 90, 105),
            surface: Color::rgba(1.0, 1.0, 1.0, 0.55),
            surface_input: Color::rgba(1.0, 1.0, 1.0, 0.85),
            backdrop_light: Color::from_srgb8(226, 245, 247),
            backdrop_dark: Color::from_srgb8(182, 225, 233),
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
            backdrop_light: Color::from_srgb8(102, 114, 125),
            backdrop_dark: Color::from_srgb8(66, 77, 88),
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
            backdrop_light: Color::from_srgb8(149, 130, 146),
            backdrop_dark: Color::from_srgb8(85, 77, 104),
        },
    },
];
