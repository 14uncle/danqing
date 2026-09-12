//! @author 十四叔
//! @date 2026/09/12
//!
//! 下拉选择器：自足组件 (选择框 + 弹出选项列表 + 键盘导航 + 点外收起)。
//!
//! 做什么：
//! - 展开态**组件自管** (非绑定): 开合由控件点击、键盘、点外收起驱动，应用
//!   没有第二个开合入口; `is_expanded()` 只读暴露。
//! - 弹出层走框架的弹层通道 ([`Widget::popup_area`] / [`Widget::paint_popup`]):
//!   矩形用**窗口绝对坐标**; 绘制由 `paint_popups` 在主树之后统一开层，命中由
//!   `dispatch_popup_event` 优先投递。因此弹层可超出控件自身矩形、跨越任意多级
//!   祖先，不受容器裁剪与兄弟覆盖影响。组件**不得**自行 `push_layer`
//!   (配对收口在 `paint_popups`)。
//! - `hit_area` 展开时取「控件 ∪ 弹层」并集 —— 否则点选项时焦点会掉，
//!   键盘导航随之失效。
//! - 键盘 (聚焦后经焦点路由直达): ↑↓ 移动 hover / Enter·Space 选中 /
//!   Esc 收起; 收起态 ↑↓ 与 Enter·Space 均展开。Esc 在收起态返回 `Ignored`,
//!   留给 app 级 Esc 协议 (与 Overlay 同一约定)。
//! - 焦点既驱动边框 (转 `theme.accent()`，与其他表单组件 TextInput / TextArea /
//!   Switch / IconInput 同一约定), 也驱动展开态: **失焦即收起**, 故 Tab 移焦后
//!   弹层不会留在屏上无人认领。
//!
//! 不做什么：
//! - 长列表滚动 (v1; 超长列表请自行控制选项数)
//! - 空间不足向上翻转 (v1 恒在控件下方展开)
//! - 打字过滤 / 分组 / 多选 / 禁用态
//! - 开合动画; 弹层阴影 (框架无阴影绘制原语，现有组件一律不画)
//!
//! 已知限制 (2026-09-12 裁决，见 `docs/specs/SPEC-dropdown.md`):
//! 鼠标点选选项后, `FocusManager::set_by_click` 会用**已收起**的 `hit_area` 复判
//! (`focus.rs` 的命中遍历只读 `hit_area`, 从不读 `popup_area`), 且它在事件分发
//! **之后**才运行。于是点击那一刻, 焦点会落到**弹层下方那个被遮住的控件**上;
//! 只有落点下方没有可聚焦控件时才会清空。两种结果都需重新点击才能继续键盘导航。
//! 纯键盘路径不受影响 (全程无点击)。
//!
//! 为什么自足：此前两版实现均被证伪 —— 画在自身区域内会被后续绘制者覆盖、
//! 并撑坏父容器布局; 推给应用侧 `Overlay` 组装则让每个产品重复手写弹层、
//! 键盘导航与点外关闭 (showcase 曾为此背 4 个 Msg + 3 个状态字段 + 20 行导航)。
//! 根因是框架当时没有弹层通道，通道见 `widget/mod.rs`。

use std::any::Any;
use std::cell::Cell;

use crate::event::{Event, Key, MouseButton, NamedKey};
use crate::render::{RectBatch, TextBatch};
use crate::widget::{EventResult, MsgQueue, Widget, push_diagonal};
use crate::{Color, Constraints, LightTheme, Point, Rect, Size, Theme};

/// 选中绑定：从应用状态读取当前选中索引。
type SelectedBinding = Box<dyn Fn(&dyn Any) -> usize>;
/// 选中消息工厂：选中某项时产出应用消息 (索引)。
type SelectFactory = Box<dyn Fn(usize) -> Box<dyn Any>>;

/// 无 hover 行。
const NO_HOVER: usize = usize::MAX;
/// 箭头区宽度 (组件固有几何，非主题 token)。
const ARROW_W: f32 = 24.0;
/// 折线箭头的半宽 (总宽 = 2×)。
const ARROW_HALF_W: f32 = 3.5;
/// 折线箭头的半高 (总高 = 2×)。
const ARROW_HALF_H: f32 = 2.0;
/// 折线箭头的笔画粗细。
const ARROW_STROKE: f32 = 1.6;

/// 下拉选择器。
pub struct Dropdown {
    /// 选项文本。
    options: Vec<String>,
    /// 当前选中索引。
    selected: usize,
    /// 选中绑定。
    selected_binding: Option<SelectedBinding>,
    /// 选中消息工厂。
    on_select: Option<SelectFactory>,
    /// 展开态 (组件自管)。
    expanded: bool,
    /// 弹层内 hover 行 (`NO_HOVER` = 无)。
    hover_idx: usize,
    /// 固定宽度 (None = 填满可用宽度)。
    width: Option<f32>,
    /// 控件矩形缓存 (绝对坐标; paint 与常规事件路径刷新)。
    area: Cell<Rect>,
    /// 控件高度 (主题 token, 构造时取定)。
    control_height: f32,
    /// 列表行高。
    row_height: f32,
    /// 圆角。
    radius: f32,
    /// 行高亮圆角。
    row_radius: f32,
    /// 文本左右内边距。
    padding: f32,
    /// 弹层与控件间距，兼作列表上下内边距。
    list_pad: f32,
    /// 字号。
    font_size: u16,
    /// 控件底色。
    bg_color: Color,
    /// 弹层底色 (比控件更实，须遮住底下内容)。
    popup_bg_color: Color,
    /// 边框色。
    border_color: Color,
    /// 焦点态边框色 (accent)。与 TextInput / TextArea / Switch / IconInput 同一
    /// 约定: 取 `theme.accent()`，焦点态由框架的 `FocusIn` / `FocusOut` 驱动。
    focus_border_color: Color,
    /// 是否持有焦点。
    focused: bool,
    /// 正文色。
    text_color: Color,
    /// hover 行底色。
    hover_color: Color,
    /// 选中行底色。
    selected_color: Color,
    /// 箭头色。
    arrow_color: Color,
}

impl Dropdown {
    /// 创建下拉选择器 (默认浅色主题)。
    pub fn new(options: Vec<String>) -> Self {
        Self::themed(&LightTheme, options)
    }

    /// 使用指定主题创建。
    pub fn themed(theme: &impl Theme, options: Vec<String>) -> Self {
        let control_height = theme.control_height();
        Self {
            options,
            selected: 0,
            selected_binding: None,
            on_select: None,
            expanded: false,
            hover_idx: NO_HOVER,
            width: None,
            area: Cell::new(Rect::default()),
            control_height,
            row_height: control_height,
            radius: theme.radius_sm(),
            row_radius: theme.radius_sm(),
            padding: theme.spacing_md(),
            list_pad: theme.spacing_xs(),
            font_size: theme.font_size_body(),
            bg_color: theme.surface_input(),
            popup_bg_color: theme.surface_input(),
            border_color: theme.border(),
            focus_border_color: theme.accent(),
            focused: false,
            text_color: theme.text_primary(),
            hover_color: theme.surface_variant(),
            selected_color: theme.selection(),
            arrow_color: theme.accent(),
        }
    }

    /// 绑定选中索引 (从应用状态读取)。
    pub fn bind_selected<S: 'static>(mut self, f: impl Fn(&S) -> usize + 'static) -> Self {
        self.selected_binding = Some(Box::new(move |state: &dyn Any| {
            f(state
                .downcast_ref::<S>()
                .expect("Dropdown 选中绑定的状态类型不匹配"))
        }));
        self
    }

    /// 选中回调：选中某项时产出应用消息 (消息内容 = 选项索引)。
    pub fn on_select<M: 'static>(mut self, f: impl Fn(usize) -> M + 'static) -> Self {
        self.on_select = Some(Box::new(move |idx| Box::new(f(idx)) as Box<dyn Any>));
        self
    }

    /// 设置固定宽度 (None = 填满可用宽度)。
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// 当前是否展开 (只读; 展开态由组件自管)。
    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    /// 展开弹层并把 hover 落在当前选中项 (空选项列表不展开)。
    fn open(&mut self) {
        if self.options.is_empty() {
            return;
        }
        self.expanded = true;
        self.hover_idx = self.selected.min(self.options.len() - 1);
    }

    /// 收起弹层并清 hover。
    fn collapse(&mut self) {
        self.expanded = false;
        self.hover_idx = NO_HOVER;
    }

    /// 切换展开态。
    fn toggle(&mut self) {
        if self.expanded {
            self.collapse();
        } else {
            self.open();
        }
    }

    /// 选中指定项并发消息，随后收起。
    fn select(&mut self, idx: usize, msgs: &mut MsgQueue) {
        if idx < self.options.len() {
            // 先本地生效：即使没有绑定也能当帧显示正确选中项。
            self.selected = idx;
        }
        self.collapse();
        if let Some(factory) = &self.on_select {
            msgs.push(factory(idx));
        }
    }

    /// 按增量移动 hover 行 (不越界)。
    fn move_hover(&mut self, delta: isize) {
        let len = self.options.len();
        if len == 0 {
            return;
        }
        let cur = if self.hover_idx == NO_HOVER {
            self.selected
        } else {
            self.hover_idx
        };
        self.hover_idx = (cur as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    /// 选中当前 hover 行 (无 hover 时选当前选中项)。
    fn select_hover(&mut self, msgs: &mut MsgQueue) {
        let idx = if self.hover_idx == NO_HOVER {
            self.selected
        } else {
            self.hover_idx
        };
        self.select(idx, msgs);
    }

    /// 由控件矩形推出的弹层矩形 (控件正下方，同宽)。
    fn popup_rect_for(&self, control: Rect) -> Rect {
        Rect::from_xywh(
            control.origin.x,
            control.origin.y + control.size.height + self.list_pad,
            control.size.width,
            self.row_height * self.options.len() as f32 + self.list_pad * 2.0,
        )
        .snap_to_pixels()
    }

    /// 弹层内第 `i` 行的矩形。
    fn row_rect_in(&self, popup: Rect, i: usize) -> Rect {
        Rect::from_xywh(
            popup.origin.x + self.list_pad,
            popup.origin.y + self.list_pad + self.row_height * i as f32,
            popup.size.width - self.list_pad * 2.0,
            self.row_height,
        )
    }

    /// 弹层内第 `i` 行的高亮底色 (`None` = 无高亮)。
    ///
    /// **选中态优先于 hover**: 选中行本身就是强调底色, hover 再换一次色会让人
    /// 误以为选中态被取消了 (2026-09-12 用户要求)。故选中行 hover 时底色不变,
    /// 代价是它没有额外的悬停反馈 —— 选中态本身已是足够强的标记。
    fn row_fill(&self, i: usize) -> Option<Color> {
        if i == self.selected {
            Some(self.selected_color)
        } else if i == self.hover_idx {
            Some(self.hover_color)
        } else {
            None
        }
    }

    /// 弹层内某坐标落在第几行 (夹取到合法范围)。
    fn row_index_at(&self, control: Rect, p: Point) -> usize {
        let popup = self.popup_rect_for(control);
        let rel = p.y - (popup.origin.y + self.list_pad);
        let idx = (rel / self.row_height).floor().max(0.0) as usize;
        idx.min(self.options.len().saturating_sub(1))
    }

    /// 键盘处理 (聚焦态经焦点路由到达)。
    fn handle_key(&mut self, key: &Key, msgs: &mut MsgQueue) -> EventResult {
        match key {
            Key::Named(NamedKey::ArrowDown) => {
                if self.expanded {
                    self.move_hover(1);
                } else {
                    self.open();
                }
                EventResult::Consumed
            }
            Key::Named(NamedKey::ArrowUp) => {
                if self.expanded {
                    self.move_hover(-1);
                } else {
                    self.open();
                }
                EventResult::Consumed
            }
            Key::Named(NamedKey::Enter) | Key::Named(NamedKey::Space) => {
                if self.expanded {
                    self.select_hover(msgs);
                } else {
                    self.open();
                }
                EventResult::Consumed
            }
            Key::Named(NamedKey::Escape) if self.expanded => {
                self.collapse();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

/// 两矩形的并集外接矩形 (`Rect` 无 union, 不为单个消费者新增公共 API)。
fn union_rect(a: Rect, b: Rect) -> Rect {
    let x0 = a.origin.x.min(b.origin.x);
    let y0 = a.origin.y.min(b.origin.y);
    let x1 = (a.origin.x + a.size.width).max(b.origin.x + b.size.width);
    let y1 = (a.origin.y + a.size.height).max(b.origin.y + b.size.height);
    Rect::from_xywh(x0, y0, x1 - x0, y1 - y0)
}

impl Widget for Dropdown {
    fn sync(&mut self, state: &dyn Any) {
        if let Some(bind) = &self.selected_binding {
            self.selected = bind(state);
        }
    }

    fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
        let w = self.width.unwrap_or(constraints.max().width);
        // 弹层不参与布局：高度恒为控件高度，不撑高父容器、不挤走兄弟
        // (第一版画在自身区域内，正是死在撑坏布局上)。
        constraints.constrain(Size::new(w, self.control_height))
    }

    fn paint(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        let control = Rect::from_xywh(
            area.origin.x,
            area.origin.y,
            area.size.width,
            self.control_height,
        );
        self.area.set(control);

        rects.push_rect(control, self.bg_color, self.radius);
        let border = if self.focused {
            self.focus_border_color
        } else {
            self.border_color
        };
        rects.push_rounded_border(control, border, self.radius, 1.0);

        let px = self.font_size as f32;
        if let Some(text) = self.options.get(self.selected) {
            let baseline = control.origin.y
                + (self.control_height - texts.line_height(px)) / 2.0
                + texts.ascent(px);
            texts.push_text(
                text,
                control.origin.x + self.padding,
                baseline,
                self.font_size,
                self.text_color,
            );
        }

        // 箭头区：分隔线 + 折线 (展开时朝上)。
        let arrow_x = control.origin.x + (control.size.width - ARROW_W).max(0.0);
        rects.push_rect(
            Rect::from_xywh(
                arrow_x,
                control.origin.y + self.list_pad,
                1.0,
                self.control_height - self.list_pad * 2.0,
            ),
            border,
            0.0,
        );
        let cx = arrow_x + ARROW_W / 2.0;
        let cy = control.origin.y + self.control_height / 2.0;
        let (s, h) = (ARROW_HALF_W, ARROW_HALF_H);
        let c = self.arrow_color;
        // 折线用 `push_diagonal` (圆点队列逼近对角线), 与 TitleBar 的 ×、
        // CloseButton 的 ×、时钟指针同一算法 —— 好处是不依赖
        // `RectInstance.rotation`。原先的 `RectBatch::push_line` 正是靠 rotation
        // 摆放斜线, 实测渲染不出来 (斜线躺平成一根横杠), 已于 2026-09-12 删除;
        // 未查明的 rotation 根因见 `tasks/todo-dropdown.md` T7。
        if self.expanded {
            push_diagonal(
                rects,
                Point::new(cx - s, cy + h),
                Point::new(cx, cy - h),
                ARROW_STROKE,
                c,
            );
            push_diagonal(
                rects,
                Point::new(cx, cy - h),
                Point::new(cx + s, cy + h),
                ARROW_STROKE,
                c,
            );
        } else {
            push_diagonal(
                rects,
                Point::new(cx - s, cy - h),
                Point::new(cx, cy + h),
                ARROW_STROKE,
                c,
            );
            push_diagonal(
                rects,
                Point::new(cx, cy + h),
                Point::new(cx + s, cy - h),
                ARROW_STROKE,
                c,
            );
        }
    }

    fn event(&mut self, event: &Event, area: Rect, msgs: &mut MsgQueue) -> EventResult {
        let area = area.snap_to_pixels();
        let pos = event.position();
        // 弹层内的按下经 `dispatch_popup_event` 直达 (area = 弹层矩形); 其余走常规
        // 分发 (area = 控件布局矩形)。只有常规**定位**事件才能刷新控件矩形缓存 ——
        // 键盘事件经焦点路由到达时 area 是整个根区域，入缓存会毁掉命中几何。
        let on_popup = matches!((self.popup_area(), pos), (Some(p), Some(pt)) if p.contains(pt));
        if !on_popup && pos.is_some() {
            self.area.set(Rect::from_xywh(
                area.origin.x,
                area.origin.y,
                area.size.width,
                self.control_height,
            ));
        }
        let control = self.area.get();

        match event {
            Event::CursorMoved(p) => {
                if self.expanded && self.popup_rect_for(control).contains(*p) {
                    self.hover_idx = self.row_index_at(control, *p);
                    EventResult::Consumed
                } else if control.contains(*p) {
                    self.hover_idx = NO_HOVER;
                    EventResult::Consumed
                } else {
                    self.hover_idx = NO_HOVER;
                    EventResult::Ignored
                }
            }
            Event::CursorLeft => {
                self.hover_idx = NO_HOVER;
                EventResult::Ignored
            }
            Event::MouseInput {
                button: MouseButton::Left,
                pressed: true,
                position,
            } => {
                if self.expanded && self.popup_rect_for(control).contains(*position) {
                    let idx = self.row_index_at(control, *position);
                    self.select(idx, msgs);
                    EventResult::Consumed
                } else if control.contains(*position) {
                    self.toggle();
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            Event::Key {
                key, pressed: true, ..
            } => self.handle_key(key, msgs),
            // 焦点既驱动边框色, 也驱动展开态: 失焦即收起 (2026-09-12 裁决, 与
            // `reset_focus` 同语义)。否则 Tab 移焦后弹层会留在屏上无人认领 ——
            // 点外收起只覆盖鼠标按下, 键盘移焦不产生按下事件。
            Event::FocusIn => {
                self.focused = true;
                EventResult::Consumed
            }
            Event::FocusOut => {
                self.focused = false;
                self.collapse();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn focusable(&self) -> bool {
        true
    }

    /// 容器隐藏本子树时由框架调用：收起弹层，别把展开态带进下一次显示。
    ///
    /// 同时清焦点标记 —— 子树隐藏时框架未必补发 `FocusOut` (Switch 同一处理)，
    /// 不清则下次显示会带着上一轮的焦点边框。
    fn reset_focus(&mut self) {
        self.collapse();
        self.focused = false;
    }

    /// 控件矩形; 展开时并入弹层 —— 否则点选项时焦点复判落空，键盘导航中断。
    fn hit_area(&self) -> Option<Rect> {
        let control = self.area.get();
        Some(match self.popup_area() {
            Some(popup) => union_rect(control, popup),
            None => control,
        })
    }

    fn popup_area(&self) -> Option<Rect> {
        (self.expanded && !self.options.is_empty()).then(|| self.popup_rect_for(self.area.get()))
    }

    /// 弹层由 `paint_popups` 在主树绘制之后调用，渲染层已由框架开好
    /// (组件不得自行 `push_layer`)。
    fn paint_popup(&self, area: Rect, rects: &mut RectBatch, texts: &mut TextBatch) {
        let area = area.snap_to_pixels();
        rects.push_rect(area, self.popup_bg_color, self.radius);
        rects.push_rounded_border(area, self.border_color, self.radius, 1.0);

        let px = self.font_size as f32;
        for (i, text) in self.options.iter().enumerate() {
            let row = self.row_rect_in(area, i);
            if let Some(fill) = self.row_fill(i) {
                rects.push_rect(row, fill, self.row_radius);
            }
            let baseline =
                row.origin.y + (self.row_height - texts.line_height(px)) / 2.0 + texts.ascent(px);
            texts.push_text(
                text,
                row.origin.x + self.padding,
                baseline,
                self.font_size,
                self.text_color,
            );
        }
    }

    /// 点外收起 (框架在按下落于弹层与其持有者控件之外时调用)。
    fn on_popup_dismiss(&mut self) {
        self.collapse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::{
        Column, FocusManager, dismiss_popup_at, dispatch_popup_event, event_at_path, node,
    };
    use std::cell::Cell;
    use std::rc::Rc;

    const OPTIONS: [&str; 3] = ["甲", "乙", "丙"];

    /// 绑定源状态。
    struct Demo {
        selected: usize,
    }

    fn opts() -> Vec<String> {
        OPTIONS.iter().map(|s| s.to_string()).collect()
    }

    /// 默认：绑定 selected 到 `Demo`, 选中回调产出裸索引。
    fn dropdown() -> Dropdown {
        Dropdown::new(opts())
            .bind_selected(|s: &Demo| s.selected)
            .on_select(|idx| idx)
    }

    fn theme() -> LightTheme {
        LightTheme
    }

    /// 以固定控件矩形 (0,0,200,control_height) 画一帧，填充几何缓存。
    fn layout_at(dd: &mut Dropdown) {
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        dd.sync(&Demo { selected: 0 });
        dd.paint(control(), &mut rects, &mut texts);
    }

    /// 控件矩形。
    fn control() -> Rect {
        Rect::from_xywh(0.0, 0.0, 200.0, theme().control_height())
    }

    /// 弹层顶边的 y (控件底边 + 间距)。
    fn popup_top() -> f32 {
        control().origin.y + theme().control_height() + theme().spacing_xs()
    }

    /// 弹层矩形 (全由主题 token 推出)。
    fn expected_popup() -> Rect {
        Rect::from_xywh(
            control().origin.x,
            popup_top(),
            control().size.width,
            theme().control_height() * OPTIONS.len() as f32 + theme().spacing_xs() * 2.0,
        )
    }

    /// 弹层第 `i` 行中心的屏幕坐标。
    ///
    /// **刻意不经 `popup_area()`** —— 否则实现错了测试会跟着错 (自证): 弹层纵向
    /// 几何的任何偏差都会同时移动「期望点」与被测的命中区, 永远自洽。
    fn option_point(i: usize) -> Point {
        Point::new(
            control().origin.x + 5.0,
            popup_top() + theme().spacing_xs() + theme().control_height() * (i as f32 + 0.5),
        )
    }

    fn left_press(x: f32, y: f32) -> Event {
        Event::MouseInput {
            button: MouseButton::Left,
            pressed: true,
            position: Point::new(x, y),
        }
    }

    /// 控件内按下 (常规路径：area = 控件布局矩形)。
    fn click_control(dd: &mut Dropdown, msgs: &mut MsgQueue) -> EventResult {
        let c = control();
        dd.event(&left_press(c.origin.x + 10.0, c.origin.y + 10.0), c, msgs)
    }

    /// 弹层第 `i` 行按下 (弹层通道路径：area = 弹层矩形)。
    ///
    /// 按下坐标由主题 token 算出; 若组件自身的弹层几何有偏差, `event` 认不出这个
    /// 点落在弹层内, 测试即失败 —— 这正是它抓得住实现错误的原因。
    fn click_option(dd: &mut Dropdown, i: usize, msgs: &mut MsgQueue) -> EventResult {
        let p = option_point(i);
        let popup = dd.popup_area().expect("需先展开");
        dd.event(&left_press(p.x, p.y), popup, msgs)
    }

    /// 光标移到弹层内第 `i` 行 (广播路径：area = 控件布局矩形)。
    fn cursor_over_popup(dd: &mut Dropdown, i: f32, msgs: &mut MsgQueue) -> EventResult {
        let p = option_point(i as usize);
        dd.event(&Event::CursorMoved(p), control(), msgs)
    }

    /// 新开一个已展开的 Dropdown, 在弹层内 `y` 处按下; 返回选中的行索引。
    fn select_at_y(y: f32) -> Option<usize> {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        click_control(&mut dd, &mut msgs); // 展开
        msgs.clear();
        let popup = dd.popup_area().expect("应展开");
        dd.event(&left_press(popup.origin.x + 5.0, y), popup, &mut msgs);
        msgs.first()
            .and_then(|m| m.downcast_ref::<usize>().copied())
    }

    /// 键盘按下 (焦点路由路径：area 为根区域，组件不得据此刷新几何)。
    fn key_press(dd: &mut Dropdown, key: NamedKey, msgs: &mut MsgQueue) -> EventResult {
        dd.event(
            &Event::Key {
                key: Key::Named(key),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            Rect::default(),
            msgs,
        )
    }

    /// 主题 token 的原始 RGBA —— 与 `RectBatch::instance_colors` 同一表示。
    fn rgba_of(c: Color) -> [f32; 4] {
        [c.r, c.g, c.b, c.a]
    }

    /// 批次中所有**横向描边段**的颜色 (1px 厚、横向成段)。
    ///
    /// 按几何筛取而非「批次里有没有出现过 accent」: 箭头的折线色同为
    /// `theme.accent()`, 后者在把焦点判断整个删掉之后依然全绿 —— 属于自证。
    /// 1px 厚这一条同时排除了折线的 1.6×1.6 圆点、1px 宽的箭头分隔线与左右竖边。
    fn horizontal_border_colors(rects: &RectBatch) -> Vec<[f32; 4]> {
        let colors = rects.instance_colors();
        rects
            .instance_rects()
            .iter()
            .enumerate()
            .filter(|(_, r)| r.size.height == 1.0 && r.size.width > 1.0)
            .map(|(i, _)| colors[i])
            .collect()
    }

    /// 单画一帧控件, 取横向描边段的颜色。
    fn control_border_colors(dd: &Dropdown) -> Vec<[f32; 4]> {
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        dd.paint(control(), &mut rects, &mut texts);
        horizontal_border_colors(&rects)
    }

    /// 焦点事件 (焦点路由路径: area 为根区域)。
    fn focus_event(dd: &mut Dropdown, event: Event) -> EventResult {
        let mut msgs = MsgQueue::new();
        dd.event(&event, Rect::default(), &mut msgs)
    }

    /// 右侧区**分隔线**的颜色: 1px 宽、纵向成段、且落在 `x + w − ARROW_W` 处。
    ///
    /// 按 x 定位而非「纵向描边里有没有 accent」—— 控件自身的左右竖边本来就是
    /// 焦点色, 后者分不清是哪一条, 会给「分隔线漏了焦点色」放行。
    fn zone_divider_color(dd: &Dropdown) -> [f32; 4] {
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        dd.paint(control(), &mut rects, &mut texts);
        let divider_x = control().origin.x + control().size.width - ARROW_W;
        let rects_of_batch = rects.instance_rects();
        let idx = rects_of_batch
            .iter()
            .position(|r| r.size.width == 1.0 && r.size.height > 1.0 && r.origin.x == divider_x)
            .expect("右侧区分隔线应被绘制");
        rects.instance_colors()[idx]
    }

    #[test]
    fn layout_height_is_control_height_regardless_of_option_count() {
        // 第一版老毛病回归锁：弹层不参与布局，不得撑高父容器。
        let mut texts = TextBatch::new();
        let c = Constraints::loose(Size::new(400.0, 400.0));
        let mut one = Dropdown::new(vec!["甲".to_string()]).width(200.0);
        let mut many = Dropdown::new((0..50).map(|i| format!("选项 {i}")).collect()).width(200.0);

        let s1 = one.layout(c, &mut texts);
        let s2 = many.layout(c, &mut texts);

        assert_eq!(s1.height, s2.height, "布局高度不得随选项数变化");
        // 验收项的另一半: 不只是「不随选项数变」, 而是**就是**控件高度。
        // 少了这一条, 让 layout 恒返回高度 0 也能全绿。
        assert_eq!(
            s2.height,
            theme().control_height(),
            "高度必须等于 theme.control_height()"
        );
        assert_eq!(s2.width, 200.0, "宽度取显式设定值");
    }

    #[test]
    fn popup_area_none_collapsed_and_below_control_when_expanded() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        assert_eq!(dd.popup_area(), None, "收起态无弹层");

        click_control(&mut dd, &mut msgs);
        // 字面期望 (全由主题 token 推出) —— 原先只断言「origin.y >= 控件底边」,
        // 只有下界, 把间距改成 100 也照样通过。
        assert_eq!(
            dd.popup_area(),
            Some(expected_popup()),
            "弹层须紧贴控件下方, 间距 = spacing_xs, 高 = 行高 × 项数 + 上下内边距"
        );
    }

    #[test]
    fn click_control_toggles_expanded() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        assert_eq!(click_control(&mut dd, &mut msgs), EventResult::Consumed);
        assert!(dd.is_expanded(), "首次点击展开");
        click_control(&mut dd, &mut msgs);
        assert!(!dd.is_expanded(), "再次点击收起");
        assert!(msgs.is_empty(), "开合不产出消息");
    }

    #[test]
    fn click_option_selects_collapses_and_emits_index() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        click_control(&mut dd, &mut msgs);
        msgs.clear();

        assert_eq!(click_option(&mut dd, 2, &mut msgs), EventResult::Consumed);
        assert!(!dd.is_expanded(), "选中后收起");
        assert_eq!(msgs.len(), 1, "产出一条消息");
        assert_eq!(
            msgs[0].downcast_ref::<usize>(),
            Some(&2),
            "消息内容 = 行索引"
        );
        assert_eq!(dd.selected, 2, "本地立即生效");
    }

    #[test]
    fn cursor_over_popup_tracks_hover_row_and_clears_outside() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        click_control(&mut dd, &mut msgs);

        assert_eq!(
            cursor_over_popup(&mut dd, 1.0, &mut msgs),
            EventResult::Consumed
        );
        assert_eq!(dd.hover_idx, 1, "移动高亮跟随光标行");

        dd.event(
            &Event::CursorMoved(Point::new(900.0, 900.0)),
            control(),
            &mut msgs,
        );
        assert_eq!(dd.hover_idx, NO_HOVER, "移出弹层后清除 hover");
    }

    #[test]
    fn selected_row_keeps_its_fill_under_hover() {
        // 选中态优先于 hover (2026-09-12 用户要求): 选中行 hover 时底色保持
        // 选中色, 不得被换成 hover 色 —— 否则看起来像选中态被取消了。
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        click_control(&mut dd, &mut msgs);

        // 展开时 hover 默认落在当前选中项, 正是最容易看出问题的时刻。
        assert_eq!(dd.hover_idx, 0, "前提: 展开时 hover 落在选中行");
        assert_eq!(
            dd.row_fill(0),
            Some(dd.selected_color),
            "选中行带 hover 也必须用选中底色"
        );

        // hover 移到非选中行 → 该行用 hover 底色; 选中行移开 hover 后仍是选中色。
        cursor_over_popup(&mut dd, 1.0, &mut msgs);
        assert_eq!(dd.hover_idx, 1, "前提: hover 已移到第 1 行");
        assert_eq!(
            dd.row_fill(1),
            Some(dd.hover_color),
            "非选中行用 hover 底色"
        );
        assert_eq!(
            dd.row_fill(0),
            Some(dd.selected_color),
            "选中行未被 hover 时仍是选中底色"
        );
        assert_eq!(dd.row_fill(2), None, "既非选中也非 hover 的行无高亮");
    }

    #[test]
    fn keyboard_arrows_open_then_move_within_bounds() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        key_press(&mut dd, NamedKey::ArrowDown, &mut msgs);
        assert!(dd.is_expanded(), "收起态 ArrowDown 展开");
        assert_eq!(dd.hover_idx, 0, "展开时 hover 落在当前选中项");

        for _ in 0..10 {
            key_press(&mut dd, NamedKey::ArrowDown, &mut msgs);
        }
        assert_eq!(dd.hover_idx, OPTIONS.len() - 1, "下移不越界");

        for _ in 0..10 {
            key_press(&mut dd, NamedKey::ArrowUp, &mut msgs);
        }
        assert_eq!(dd.hover_idx, 0, "上移不越界");
    }

    #[test]
    fn keyboard_enter_selects_hover_and_escape_leaves_selection_untouched() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        key_press(&mut dd, NamedKey::ArrowDown, &mut msgs); // 展开，hover = 0
        key_press(&mut dd, NamedKey::ArrowDown, &mut msgs); // hover = 1
        key_press(&mut dd, NamedKey::Enter, &mut msgs);

        assert!(!dd.is_expanded(), "Enter 选中后收起");
        assert_eq!(msgs.len(), 1, "Enter 产出一条消息");
        assert_eq!(msgs[0].downcast_ref::<usize>(), Some(&1), "选中 hover 行");
        assert_eq!(dd.selected, 1);

        msgs.clear();
        dd.sync(&Demo { selected: 2 });
        key_press(&mut dd, NamedKey::ArrowDown, &mut msgs);
        key_press(&mut dd, NamedKey::Escape, &mut msgs);

        assert!(!dd.is_expanded(), "Esc 收起");
        assert!(msgs.is_empty(), "Esc 不产出消息");
        assert_eq!(dd.selected, 2, "Esc 不改变选中值");
    }

    #[test]
    fn escape_when_collapsed_is_ignored_for_app_level_protocol() {
        // 收起态 Esc 必须 Ignored: app 级 Esc 协议 (清焦/关面板) 依赖它冒泡。
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        assert_eq!(
            key_press(&mut dd, NamedKey::Escape, &mut msgs),
            EventResult::Ignored
        );
    }

    #[test]
    fn dismiss_and_reset_focus_both_collapse() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        click_control(&mut dd, &mut msgs);
        assert!(dd.is_expanded());
        dd.on_popup_dismiss();
        assert!(!dd.is_expanded(), "点外收起");

        click_control(&mut dd, &mut msgs);
        assert!(dd.is_expanded());
        dd.reset_focus();
        assert!(
            !dd.is_expanded(),
            "reset_focus 收起 (容器隐藏子树时由框架调用)"
        );
    }

    #[test]
    fn hit_area_covers_popup_only_when_expanded() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        assert_eq!(dd.hit_area(), Some(control()), "收起态仅控件矩形");

        click_control(&mut dd, &mut msgs);
        let hit = dd.hit_area().expect("展开态应有 hit_area");
        let popup = dd.popup_area().expect("应展开");
        assert!(
            hit.contains(Point::new(popup.origin.x + 1.0, popup.origin.y + 1.0)),
            "hit_area 须覆盖弹层"
        );
        assert!(
            hit.size.height >= control().size.height + popup.size.height,
            "hit_area 须为控件与弹层的并集"
        );
    }

    #[test]
    fn empty_options_never_expand() {
        let mut dd = Dropdown::new(Vec::new());
        let mut rects = RectBatch::new();
        let mut texts = TextBatch::new();
        dd.paint(control(), &mut rects, &mut texts);
        let mut msgs = MsgQueue::new();

        click_control(&mut dd, &mut msgs);
        assert!(!dd.is_expanded(), "空列表不展开 (无内容可弹)");
        assert_eq!(dd.popup_area(), None);
    }

    #[test]
    fn click_option_selects_the_row_under_the_cursor_for_every_row() {
        // 首/中/末行**逐行**验证。原先只点末行, 而末行恰好是 `row_index_at` 的
        // clamp 边界 —— off-by-one 会被 `min(len-1)` 吃掉, 测试却全绿。
        for i in 0..OPTIONS.len() {
            let mut dd = dropdown();
            layout_at(&mut dd);
            let mut msgs = MsgQueue::new();
            click_control(&mut dd, &mut msgs);
            msgs.clear();

            assert_eq!(click_option(&mut dd, i, &mut msgs), EventResult::Consumed);
            assert_eq!(
                msgs.first()
                    .and_then(|m| m.downcast_ref::<usize>().copied()),
                Some(i),
                "第 {i} 行必须选中自己"
            );
        }
    }

    #[test]
    fn row_hit_testing_respects_list_padding_at_row_boundaries() {
        // 行命中的 off-by-one 只在**行边界附近**露头: 行中心的点击被
        // 「内边距(4) < 行高(36)」掩盖, 丢掉内边距偏移也照样落在同一行。
        let row1_top = popup_top() + theme().spacing_xs() + theme().control_height();

        assert_eq!(select_at_y(row1_top + 1.0), Some(1), "边界下方归本行");
        assert_eq!(select_at_y(row1_top - 1.0), Some(0), "边界上方归上一行");
    }

    #[test]
    fn keyboard_opens_from_collapsed_with_arrow_up_enter_and_space() {
        // 决策表原先的空格: 收起态只验过 ArrowDown, ArrowUp / Enter / Space 未覆盖。
        for key in [NamedKey::ArrowUp, NamedKey::Enter, NamedKey::Space] {
            let mut dd = dropdown();
            layout_at(&mut dd);
            let mut msgs = MsgQueue::new();

            assert_eq!(
                key_press(&mut dd, key, &mut msgs),
                EventResult::Consumed,
                "{key:?} 应被消费"
            );
            assert!(dd.is_expanded(), "收起态 {key:?} 应展开");
            assert_eq!(dd.hover_idx, 0, "{key:?} 展开后 hover 落在当前选中项");
            assert!(msgs.is_empty(), "仅展开不产出消息");
        }
    }

    #[test]
    fn keyboard_space_selects_hover_like_enter() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        key_press(&mut dd, NamedKey::ArrowDown, &mut msgs); // 展开, hover = 0
        key_press(&mut dd, NamedKey::ArrowDown, &mut msgs); // hover = 1
        msgs.clear();

        assert_eq!(
            key_press(&mut dd, NamedKey::Space, &mut msgs),
            EventResult::Consumed
        );
        assert!(!dd.is_expanded(), "Space 选中后收起");
        assert_eq!(
            msgs.first()
                .and_then(|m| m.downcast_ref::<usize>().copied()),
            Some(1),
            "Space 选中 hover 行 (与 Enter 同格)"
        );
    }

    #[test]
    fn key_events_do_not_pollute_the_geometry_cache() {
        // 键盘事件经焦点路由到达时 `area` 是整个根区域。若 `event` 无差别地拿它
        // 刷新控件矩形缓存, 之后的弹层几何会全盘错位 —— 这条锁住那个守卫。
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        dd.event(
            &Event::Key {
                key: Key::Named(NamedKey::ArrowDown),
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
            },
            Rect::from_xywh(0.0, 0.0, 1200.0, 900.0), // 焦点路由给的根区域
            &mut msgs,
        );

        assert!(dd.is_expanded(), "前提: 已展开");
        assert_eq!(
            dd.popup_area(),
            Some(expected_popup()),
            "按键不得污染控件矩形缓存 (否则弹层几何全错)"
        );
    }

    #[test]
    fn collapse_clears_hover_row() {
        // 收起必须清 hover, 否则重开时会残留上一次的高亮行。
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        click_control(&mut dd, &mut msgs);
        cursor_over_popup(&mut dd, 1.0, &mut msgs);
        assert_eq!(dd.hover_idx, 1, "前提: 已有 hover 行");

        dd.on_popup_dismiss();
        assert_eq!(dd.hover_idx, NO_HOVER, "收起必须清 hover");
    }

    /// 同层兄弟探针: 只要有定位事件落在它身上的矩形就算「收到」。
    struct Sibling {
        seen: Rc<Cell<usize>>,
    }

    impl Widget for Sibling {
        fn layout(&mut self, constraints: Constraints, _texts: &mut TextBatch) -> Size {
            constraints.constrain(Size::new(200.0, 300.0))
        }

        fn paint(&self, _area: Rect, _rects: &mut RectBatch, _texts: &mut TextBatch) {}

        fn event(&mut self, event: &Event, _area: Rect, _msgs: &mut MsgQueue) -> EventResult {
            if event.position().is_some() {
                self.seen.set(self.seen.get() + 1);
            }
            EventResult::Consumed
        }
    }

    #[test]
    fn popup_owner_wins_over_a_sibling_covering_the_same_area() {
        // **端到端**: 真实容器 (Column) + 真实 Dropdown + 一个正好压在弹层区上的
        // 同层兄弟, 复刻 `handler.rs` 的按下路径 (点外收起 → 弹层优先 →
        // 常规分发)。这是本批的头号卖点 —— 也恰是两次翻车都出在的那条链路,
        // 所以必须有一条不绕过真实分发的测试。
        let sib_seen = Rc::new(Cell::new(0usize));
        let col = Column::new()
            .gap(0.0)
            .child(dropdown().width(200.0))
            .child(Sibling {
                seen: Rc::clone(&sib_seen),
            });
        let mut root = node(col);
        let viewport = Rect::from_xywh(0.0, 0.0, 200.0, 600.0);
        let mut texts = TextBatch::new();
        root.layout(Constraints::tight(viewport.size), &mut texts);
        root.paint(viewport, &mut RectBatch::new(), &mut texts);

        let mut msgs = MsgQueue::new();
        // 1) 常规分发: 点控件展开 (Flow 按命中把事件派给 Dropdown)
        assert_eq!(
            root.event(&left_press(10.0, 10.0), viewport, &mut msgs),
            EventResult::Consumed,
            "控件应接收点击并展开"
        );
        msgs.clear();

        // 2) 驱动顺序: 点外收起 → 弹层优先分发。落点在弹层第 1 行, 同时也在
        //    兄弟矩形 (y 36..336) 之内 —— 兄弟绝不该拿到它。
        let p = option_point(1);
        assert!(
            Rect::from_xywh(0.0, 36.0, 200.0, 300.0).contains(p),
            "前提: 落点同时落在被盖住的兄弟矩形内"
        );
        dismiss_popup_at(&mut root, p);
        assert_eq!(
            dispatch_popup_event(&mut root, &left_press(p.x, p.y), &mut msgs),
            Some(EventResult::Consumed),
            "弹层持有者必须先于同层兄弟拿到这次按下"
        );
        assert_eq!(sib_seen.get(), 0, "被弹层盖住的兄弟不得收到事件");
        assert_eq!(
            msgs.first()
                .and_then(|m| m.downcast_ref::<usize>().copied()),
            Some(1),
            "选中弹层第 1 行"
        );
    }

    #[test]
    fn focus_in_turns_the_control_border_accent_and_focus_out_restores_it() {
        let mut dd = dropdown();
        layout_at(&mut dd);

        let resting = rgba_of(theme().border());
        let focused = rgba_of(theme().accent());
        assert_ne!(
            resting, focused,
            "前提: 两个 token 必须不同, 否则本测试无法证伪"
        );

        let before = control_border_colors(&dd);
        assert!(!before.is_empty(), "控件横向描边应被绘制");
        assert!(before.iter().all(|c| *c == resting), "静默态: 常规边框色");

        assert_eq!(
            focus_event(&mut dd, Event::FocusIn),
            EventResult::Consumed,
            "FocusIn 应被消费"
        );
        assert!(
            control_border_colors(&dd).iter().all(|c| *c == focused),
            "焦点态: 边框须转 accent"
        );

        assert_eq!(
            focus_event(&mut dd, Event::FocusOut),
            EventResult::Consumed,
            "FocusOut 应被消费"
        );
        assert!(
            control_border_colors(&dd).iter().all(|c| *c == resting),
            "失焦后: 须复原常规边框色"
        );
    }

    #[test]
    fn focus_also_recolors_the_right_zone_divider() {
        // 与 IconInput 对齐 (icon_input.rs:308 起): 焦点时右侧区那条竖分隔线
        // 也跟随外框转 accent。原先它固定用 `border_color`, 聚焦后外框变色而
        // 紧挨着的这条线不变 —— 两个组件同一位置的观感不一致。
        let mut dd = dropdown();
        layout_at(&mut dd);

        let resting = rgba_of(theme().border());
        let focused = rgba_of(theme().accent());
        assert_ne!(
            resting, focused,
            "前提: 两个 token 必须不同, 否则本测试无法证伪"
        );

        assert_eq!(
            zone_divider_color(&dd),
            resting,
            "静默态: 分隔线用常规边框色"
        );

        focus_event(&mut dd, Event::FocusIn);
        assert_eq!(
            zone_divider_color(&dd),
            focused,
            "焦点态: 分隔线须跟随外框转 accent"
        );

        focus_event(&mut dd, Event::FocusOut);
        assert_eq!(zone_divider_color(&dd), resting, "失焦后: 分隔线复原");
    }

    #[test]
    fn focus_out_collapses_an_open_popup() {
        // 键盘展开后 Tab 移焦 —— 失焦即收起 (2026-09-12 裁决)。这条必须自己
        // 覆盖: 点外收起走的是鼠标按下, 而键盘移焦**不产生按下事件**, 所以
        // 没有别的机制会把留在屏上的弹层收掉。
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();

        focus_event(&mut dd, Event::FocusIn);
        key_press(&mut dd, NamedKey::ArrowDown, &mut msgs);
        assert!(dd.is_expanded(), "前提: 收起态按 ↓ 应展开");

        assert_eq!(
            focus_event(&mut dd, Event::FocusOut),
            EventResult::Consumed,
            "FocusOut 应被消费"
        );
        assert!(!dd.is_expanded(), "失焦须收起弹层");
        assert_eq!(dd.popup_area(), None, "收起后不应再有弹层区域");
    }

    #[test]
    fn focus_out_after_a_mouse_selection_keeps_it_and_does_not_reopen() {
        // 时序复刻鼠标点选项的真实链路: 弹层内按下 → 选中并收起 → 框架随后
        // `set_by_click` 用**已收起**的命中区复判, 焦点落到别处 → FocusOut 到达。
        //
        // 这条**不**负责锁「失焦即收起」: 它跑在已收起的组件上, 把 FocusOut 里的
        // `collapse()` 删掉它照样全绿 (实测过)。那一半由
        // `focus_out_collapses_an_open_popup` 锁。本条锁的是另一件事 —— 这次
        // FocusOut 是幂等的: 既不重开弹层, 也不把刚选中的项弄丢。
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        click_control(&mut dd, &mut msgs);
        msgs.clear();
        click_option(&mut dd, 2, &mut msgs);
        assert_eq!(
            msgs.first()
                .and_then(|m| m.downcast_ref::<usize>().copied()),
            Some(2),
            "前提: 已选中第 2 项"
        );

        focus_event(&mut dd, Event::FocusOut);

        assert!(!dd.is_expanded(), "不得因失焦而重新展开");
        assert_eq!(dd.popup_area(), None, "不得因失焦而重新展开");
        assert_eq!(
            zone_divider_color(&dd),
            rgba_of(theme().border()),
            "边框回到静默态"
        );

        // 选中项是否仍在, 用行为读回而非读私有字段: 收起态 Enter 会展开并把
        // hover 落到选中项, 再 Enter 即选中该 hover 项。若 `selected` 被
        // FocusOut 重置为 0, 这里会读到 0 而不是 2。
        msgs.clear();
        key_press(&mut dd, NamedKey::Enter, &mut msgs);
        key_press(&mut dd, NamedKey::Enter, &mut msgs);
        assert_eq!(
            msgs.last().and_then(|m| m.downcast_ref::<usize>().copied()),
            Some(2),
            "选中项须仍为第 2 项"
        );
    }

    #[test]
    fn reset_focus_clears_the_focus_border_alongside_collapsing() {
        let mut dd = dropdown();
        layout_at(&mut dd);
        let mut msgs = MsgQueue::new();
        click_control(&mut dd, &mut msgs);
        assert!(dd.is_expanded(), "前提: 已展开");
        focus_event(&mut dd, Event::FocusIn);
        // 前提: 焦点真的生效了。少了这一条, 断言「重置后是常规色」在 paint 永远
        // 画常规色的情况下也成立 —— 测试会因为错误的原因变绿。
        assert!(
            control_border_colors(&dd)
                .iter()
                .all(|c| *c == rgba_of(theme().accent())),
            "前提: FocusIn 后边框已转 accent"
        );

        dd.reset_focus();

        assert!(!dd.is_expanded(), "沿用原有的收起语义");
        assert!(
            control_border_colors(&dd)
                .iter()
                .all(|c| *c == rgba_of(theme().border())),
            "reset_focus 后边框须复原 (子树隐藏未必补发 FocusOut)"
        );
    }

    #[test]
    fn clicking_the_control_grants_focus_and_the_border_turns_accent() {
        // **端到端**: 复刻 handler.rs 的点击路径 (常规树分发 → FocusManager::
        // set_by_click → 把 FocusIn 投到焦点路径)。只单测 `event` 的 FocusIn 分支
        // 不够 —— 组件若 `focusable()` 返 false 或 `hit_area` 错位, 焦点压根到不了
        // 它, 那种测试照样全绿, 而界面上边框永远不会变绿 (正是本次上报的现象)。
        let mut root = node(Column::new().gap(0.0).child(dropdown().width(200.0)));
        let viewport = Rect::from_xywh(0.0, 0.0, 200.0, 600.0);
        let mut texts = TextBatch::new();
        root.layout(Constraints::tight(viewport.size), &mut texts);
        // 必须先画一帧: `hit_area` 读的是 paint/event 填的几何缓存, 没画过就还是
        // `Rect::default()`, `set_by_click` 命中不到任何东西 (窗口里 paint 恒在
        // 点击之前, 故这不是测试的额外要求)。
        root.paint(viewport, &mut RectBatch::new(), &mut texts);

        let mut focus = FocusManager::new();
        focus.rebuild(&root);
        // 先清焦: `rebuild` 首次会**自动**把焦点落在链首 (`focus.rs:54` 的
        // `did_initial_focus` 只生效一次)。不清的话焦点本来就已是 [0], 点击成了
        // 旁观者, 这条测试就证明不了「点击把焦点给了 Dropdown」。
        focus.clear_focus();
        assert!(focus.current().is_none(), "前提: 焦点已清空");

        focus.set_by_click(&root, Point::new(10.0, 10.0));
        let path = focus
            .current()
            .cloned()
            .expect("点击控件后焦点应落在 Dropdown 上");
        assert_eq!(path, vec![0], "路径 = Column 的第 0 个孩子");

        // 焦点变化后框架就地投递 FocusIn (`handler.rs::dispatch_focus_changes`)
        let mut msgs = MsgQueue::new();
        event_at_path(&mut root, &path, &Event::FocusIn, viewport, &mut msgs);

        let mut rects = RectBatch::new();
        root.paint(viewport, &mut rects, &mut texts);
        let colors = horizontal_border_colors(&rects);
        assert!(!colors.is_empty(), "控件横向描边应被绘制");
        assert!(
            colors.iter().all(|c| *c == rgba_of(theme().accent())),
            "走完真实焦点链路后, 控件边框应为 accent"
        );
    }
}
