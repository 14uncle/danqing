//! @author 十四叔
//! @date 2026/07/17

//! 文本层:字体加载与字形图集。
//!
//! 纯逻辑(CPU)层,不接触 GPU;渲染层负责把图集上传为纹理。

mod atlas;
mod font;
pub mod line_layout;

pub use atlas::{AtlasError, GlyphAtlas, GlyphInfo};
pub use font::{Font, FontError};
pub use line_layout::{Line, break_lines};
