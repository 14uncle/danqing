//! @author 十四叔
//! @date 2026/07/21

//! 表单组件: 单行输入框、多行文本域与滑动开关。
//!
//! `text_editor` 为内部编辑状态机, 经 [`TextInput`] / [`TextArea`] 复用,
//! 不进入公开 API。

mod icon_input;
mod switch;
mod text_area;
mod text_editor;
mod text_input;

pub use icon_input::IconInput;
pub use switch::Switch;
pub use text_area::TextArea;
pub use text_input::TextInput;
