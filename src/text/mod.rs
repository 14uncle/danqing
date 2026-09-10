//! @author 十四叔
//! @date 2026/07/17

//! 文本层: 字体加载/字形图集 + 选区模型/编码检测/宽度适配。
//!
//! 纯逻辑(CPU)层,不接触 GPU;渲染层负责把图集上传为纹理。

mod atlas;
mod crlf;
pub mod encoding;
pub mod fit;
mod font;
pub mod line_layout;
pub mod selection;

pub use atlas::{AtlasError, GlyphAtlas, GlyphInfo};
pub use crlf::to_crlf;
pub use font::{Font, FontError};
pub use line_layout::{Line, break_lines};
