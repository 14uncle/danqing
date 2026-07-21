//! @author 十四叔
//! @date 2026/07/21

//! Switcher 组件: 多面板切换容器。

use std::any::Any;

use crate::app::AnimationCtx;
use crate::event::Event;
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Node, Widget};
use crate::{Constraints, Rect, Size};

/// active 索引绑定闭包: 每帧从应用状态读取。
type ActiveBinding = Box<dyn Fn(&dyn Any) -> usize>;

/// 切换容器: 保留全部子组件实例, 只让 active 子组件参与布局 / 绘制 / 事件。
///
/// 与"销毁-重建"的切换不同, Switcher 始终持有所有子组件:
/// `sync` / `animate` 传播给全部子组件 (状态保鲜、动画存活),
/// `layout` / `paint` / `event` 只作用于 active 子组件,
/// [`Switcher::children`] 只暴露 active 子组件。
///
/// 焦点语义: 隐藏面板内的组件不进焦点链; 若焦点恰在隐藏面板内,
/// 下一帧焦点重建会将其清除, 切回后不自动恢复 (组件内残留的
/// focused 标志不可见, 无副作用)。注意焦点路径按索引解析: 若切换后
/// 新面板在相同路径上恰好也是可聚焦组件, 焦点会静默落在该组件上
/// (不派发 FocusIn), 应用若介意可在切换消息里一并处理。
///
/// `active` 越界时钳制到末尾索引, 不 panic。
pub struct Switcher {
    children: Vec<Node>,
    active: usize,
    binding: Option<ActiveBinding>,
    /// layout 缓存: active 子组件尺寸。
    active_size: Size,
}

impl Switcher {
    /// 创建空切换容器。
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            active: 0,
            binding: None,
            active_size: Size::ZERO,
        }
    }

    /// 追加一个面板 (子组件), 返回自身以链式调用。
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(std::boxed::Box::new(widget));
        self
    }

    /// 设置初始 active 索引 (越界时在布局 / 同步时钳制)。
    pub fn active(mut self, active: usize) -> Self {
        self.active = active;
        self
    }

    /// 绑定应用状态: 每帧 `sync` 时经闭包读取 active 索引。
    ///
    /// 状态类型 `S` 须与 [`App`](crate::App) 实现者一致。
    pub fn bind<S: 'static>(mut self, f: impl Fn(&S) -> usize + 'static) -> Self {
        self.binding = Some(std::boxed::Box::new(move |state: &dyn Any| {
            let state = state
                .downcast_ref::<S>()
                .expect("Switcher 绑定的状态类型不匹配");
            f(state)
        }));
        self
    }

    /// 把越界的 active 钳制到末尾索引。
    fn clamp_active(&mut self) {
        if !self.children.is_empty() && self.active >= self.children.len() {
            self.active = self.children.len() - 1;
        }
    }

    /// active 子组件的可见切片 (空容器返回空切片)。
    fn active_range(&self) -> std::ops::Range<usize> {
        if self.children.is_empty() {
            return 0..0;
        }
        let active = self.active.min(self.children.len() - 1);
        active..active + 1
    }
}

impl Default for Switcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Switcher {
    fn sync(&mut self, state: &dyn Any) {
        for child in &mut self.children {
            child.sync(state);
        }
        if let Some(binding) = &self.binding {
            self.active = binding(state);
        }
        self.clamp_active();
    }

    fn animate(&mut self, ctx: &AnimationCtx) {
        for child in &mut self.children {
            child.animate(ctx);
        }
    }

    fn layout(&mut self, constraints: Constraints, texts: &mut TextBatch) -> Size {
        self.clamp_active();
        match self.children.get_mut(self.active) {
            Some(child) => {
                self.active_size = child.layout(constraints, texts);
                constraints.constrain(self.active_size)
            }
            None => {
                self.active_size = Size::ZERO;
                constraints.constrain(Size::ZERO)
            }
        }
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        if let Some(child) = self.children.get(self.active) {
            child.paint(Rect::new(area.origin, self.active_size), rects, texts);
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        match self.children.get_mut(self.active) {
            Some(child) => child.event(event, Rect::new(area.origin, self.active_size), msgs),
            None => EventResult::Ignored,
        }
    }

    fn children(&self) -> &[Node] {
        &self.children[self.active_range()]
    }

    fn children_mut(&mut self) -> &mut [Node] {
        let range = self.active_range();
        &mut self.children[range]
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::Point;

    /// 测试桩: 固定尺寸, 记录 sync / paint / event 调用。
    struct Stub {
        id: &'static str,
        size: Size,
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Stub {
        fn new(
            id: &'static str,
            width: f32,
            height: f32,
            log: &Rc<RefCell<Vec<&'static str>>>,
        ) -> Self {
            Self {
                id,
                size: Size::new(width, height),
                log: Rc::clone(log),
            }
        }
    }

    impl Widget for Stub {
        fn sync(&mut self, _state: &dyn Any) {
            self.log.borrow_mut().push(self.id);
        }

        fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
            constraints.constrain(self.size)
        }

        fn paint(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {
            self.log.borrow_mut().push(self.id);
        }

        fn event(&mut self, _event: &Event, _area: Rect, msgs: &mut MsgQueue) -> EventResult {
            msgs.push(std::boxed::Box::new(self.id));
            EventResult::Consumed
        }
    }

    fn loose() -> Constraints {
        Constraints::loose(Size::new(400.0, 300.0))
    }

    #[test]
    fn empty_switcher_lays_out_to_zero_and_has_no_children() {
        let mut switcher = Switcher::new();
        let mut texts = TextBatch::default();
        let size = switcher.layout(loose(), &mut texts);
        assert_eq!(size, Size::ZERO);
        assert!(switcher.children().is_empty());
        assert!(switcher.children_mut().is_empty());
    }

    #[test]
    fn out_of_range_active_clamps_to_last() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut switcher = Switcher::new()
            .child(Stub::new("a", 10.0, 10.0, &log))
            .child(Stub::new("b", 20.0, 20.0, &log))
            .active(9);
        let mut texts = TextBatch::default();
        let size = switcher.layout(loose(), &mut texts);
        assert_eq!(size, Size::new(20.0, 20.0));
        assert_eq!(switcher.children().len(), 1);
    }

    #[test]
    fn layout_size_matches_active_child() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut switcher = Switcher::new()
            .child(Stub::new("a", 10.0, 10.0, &log))
            .child(Stub::new("b", 20.0, 15.0, &log));
        let mut texts = TextBatch::default();
        assert_eq!(switcher.layout(loose(), &mut texts), Size::new(10.0, 10.0));

        switcher = switcher.active(1);
        assert_eq!(switcher.layout(loose(), &mut texts), Size::new(20.0, 15.0));
    }

    #[test]
    fn paint_collects_only_active_child() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut switcher = Switcher::new()
            .child(Stub::new("hidden", 10.0, 10.0, &log))
            .child(Stub::new("shown", 10.0, 10.0, &log))
            .active(1);
        let mut texts = TextBatch::default();
        let size = switcher.layout(loose(), &mut texts);
        let mut rects = RectBatch::default();
        switcher.paint(Rect::new(Point::ZERO, size), &mut rects, &mut texts);
        assert_eq!(log.take(), vec!["shown"]);
    }

    #[test]
    fn event_reaches_only_active_child() {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut switcher = Switcher::new()
            .child(Stub::new("hidden", 10.0, 10.0, &log))
            .child(Stub::new("shown", 10.0, 10.0, &log))
            .active(1);
        let mut texts = TextBatch::default();
        let size = switcher.layout(loose(), &mut texts);
        let mut msgs = MsgQueue::new();
        let event = Event::CursorMoved(Point::ZERO);
        let result = switcher.event(&event, Rect::new(Point::ZERO, size), &mut msgs);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(msgs.len(), 1);
        assert!(
            msgs[0]
                .downcast_ref::<&'static str>()
                .is_some_and(|id| *id == "shown")
        );
    }

    #[test]
    fn sync_reaches_all_children_and_binding_drives_active() {
        struct State {
            active: usize,
        }
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut switcher = Switcher::new()
            .child(Stub::new("a", 10.0, 10.0, &log))
            .child(Stub::new("b", 10.0, 10.0, &log))
            .bind(|s: &State| s.active);

        switcher.sync(&State { active: 1 });
        // sync 传播给全部子组件。
        assert_eq!(log.take(), vec!["a", "b"]);

        // binding 驱动 active 切换, children() 只暴露 active。
        let mut texts = TextBatch::default();
        switcher.layout(loose(), &mut texts);
        assert_eq!(switcher.children().len(), 1);

        // binding 越界同样钳制。
        switcher.sync(&State { active: 42 });
        assert_eq!(switcher.children().len(), 1);
    }
}
