//! @author 十四叔
//! @date 2026/07/21

//! 布局组件: 盒模型容器与流式排布。
//!
//! `flow` 为内部排布引擎, 经 [`Column`] / [`Row`] 复用, 不进入公开 API。

mod box_;
mod center;
mod column;
mod drag_area;
mod flow;
mod padding;
mod reach_area;
mod row;
mod stack;

pub use box_::Box;
pub use center::Center;
pub use column::Column;
pub use drag_area::DragArea;
pub use flow::CrossAlign;
pub use padding::Padding;
pub use reach_area::ReachArea;
pub use row::Row;
pub use stack::Stack;
