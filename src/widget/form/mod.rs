//! @author 十四叔
//! @date 2026/07/21

//! 表单组件: 单行输入框与多行文本域。
//!
//! `text_editor` 为内部编辑状态机, 经 [`TextInput`] / [`TextArea`] 复用,
//! 不进入公开 API。

mod text_area;
mod text_editor;
mod text_input;

pub use text_area::TextArea;
pub use text_input::TextInput;
