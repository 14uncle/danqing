//! @author 十四叔
//! @date 2026/07/17

//! 布局：基础值类型、约束传递与尺寸计算。
//!
//! 本模块为纯逻辑，不依赖任何平台或图形 API。
//! 值类型 (`Color`/`Point`/`Size`/`Rect`/`Edges`) 是渲染与布局两条车道的公共契约;
//! 布局算法 (`Constraints` 等) 在后续任务中补充。

/// RGBA 颜色，各分量取值 0.0~1.0(线性空间，提交 GPU 前不做伽马转换)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// 红色分量。
    pub r: f32,
    /// 绿色分量。
    pub g: f32,
    /// 蓝色分量。
    pub b: f32,
    /// 不透明度分量 (0.0 全透明，1.0 不透明)。
    pub a: f32,
}

impl Color {
    /// 不透明黑色。
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    /// 不透明白色。
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    /// 全透明。
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// 由三分量构造不透明颜色。
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// 由四分量构造颜色。
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// 由 sRGB 字节分量 (0~255) 构造不透明颜色。
    pub const fn from_srgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    /// 向另一颜色线性插值 (分量独立,`t` 夹到 0..1)。
    ///
    /// 用于主题/场景过渡动画;在存储空间 (sRGB 编码) 内插值,
    /// 与逐帧渲染的观感一致。
    pub fn lerp(self, other: Color, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self::rgba(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }
}

/// 二维点，逻辑像素坐标 (原点为窗口左上角，y 向下)。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Point {
    /// 横坐标。
    pub x: f32,
    /// 纵坐标。
    pub y: f32,
}

impl Point {
    /// 原点 (0, 0)。
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// 构造点。
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 二维尺寸，逻辑像素。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Size {
    /// 宽度。
    pub width: f32,
    /// 高度。
    pub height: f32,
}

impl Size {
    /// 零尺寸。
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// 构造尺寸。
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// 轴对齐矩形，由左上角原点与尺寸表示。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Rect {
    /// 左上角坐标。
    pub origin: Point,
    /// 矩形尺寸。
    pub size: Size,
}

impl Rect {
    /// 由原点与尺寸构造矩形。
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// 由坐标与宽高构造矩形。
    pub const fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(Point::new(x, y), Size::new(width, height))
    }

    /// 判断点是否落在矩形内 (含左/上边界，不含右/下边界)。
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.x < self.origin.x + self.size.width
            && point.y >= self.origin.y
            && point.y < self.origin.y + self.size.height
    }

    /// 平移矩形 (原点加偏移，尺寸不变)。
    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        Self::new(
            Point::new(self.origin.x + dx, self.origin.y + dy),
            self.size,
        )
    }

    /// 判断矩形是否为空 (宽或高非正)。
    pub fn is_empty(&self) -> bool {
        self.size.width <= 0.0 || self.size.height <= 0.0
    }

    /// 将矩形四边各内缩指定量。
    pub fn inset(&self, amount: f32) -> Self {
        Self::from_xywh(
            self.origin.x + amount,
            self.origin.y + amount,
            (self.size.width - amount * 2.0).max(0.0),
            (self.size.height - amount * 2.0).max(0.0),
        )
    }

    /// 求两个矩形的交集。
    ///
    /// 若不相交或仅边界接触，返回 `None`。
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let x0 = self.origin.x.max(other.origin.x);
        let y0 = self.origin.y.max(other.origin.y);
        let x1 = (self.origin.x + self.size.width).min(other.origin.x + other.size.width);
        let y1 = (self.origin.y + self.size.height).min(other.origin.y + other.size.height);
        let width = x1 - x0;
        let height = y1 - y0;
        if width > 0.0 && height > 0.0 {
            Some(Self::from_xywh(x0, y0, width, height))
        } else {
            None
        }
    }
}

/// 四边间距 (用于 Padding 等),逻辑像素。
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Edges {
    /// 上边距。
    pub top: f32,
    /// 右边距。
    pub right: f32,
    /// 下边距。
    pub bottom: f32,
    /// 左边距。
    pub left: f32,
}

impl Edges {
    /// 四边间距全为零。
    pub const ZERO: Self = Self::all(0.0);

    /// 四边相同间距。
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// 水平/垂直两个方向分别相同的间距。
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// 水平方向总间距 (left + right)。
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// 垂直方向总间距 (top + bottom)。
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// 布局约束：父组件传给子组件的尺寸范围 (逻辑像素)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    /// 最小宽度。
    pub min_width: f32,
    /// 最大宽度。
    pub max_width: f32,
    /// 最小高度。
    pub min_height: f32,
    /// 最大高度。
    pub max_height: f32,
}

impl Constraints {
    /// 固定约束：尺寸必须恰为给定值 (如根节点取窗口尺寸)。
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// 宽松约束：最小为零，只限上限。
    pub fn loose(max: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: max.width,
            min_height: 0.0,
            max_height: max.height,
        }
    }

    /// 把尺寸夹到约束范围内。
    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }

    /// 扣除四边间距后的子约束 (Padding 用)。
    pub fn deflate(&self, edges: Edges) -> Self {
        Self {
            min_width: (self.min_width - edges.horizontal()).max(0.0),
            max_width: (self.max_width - edges.horizontal()).max(0.0),
            min_height: (self.min_height - edges.vertical()).max(0.0),
            max_height: (self.max_height - edges.vertical()).max(0.0),
        }
    }

    /// 上限尺寸 (max_width, max_height)。
    pub fn max(&self) -> Size {
        Size::new(self.max_width, self.max_height)
    }
}

/// 流式子项：沿主轴排列的一个子组件的尺寸请求。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowChild {
    /// 主轴固有尺寸 (fill_weight 为 0 时使用)。
    pub main_fixed: f32,
    /// 填充权重:0 = 按固有尺寸;>0 = 按比例瓜分剩余主轴空间。
    pub fill_weight: u32,
}

/// 一维主轴分配结果：每项 (主轴偏移，分得的主轴尺寸)。
pub type FlowResult = Vec<(f32, f32)>;

/// 沿主轴为一组子项分配空间。
///
/// 规则：
/// - 先为所有 Fit(weight=0) 项分配固有尺寸;
/// - 剩余空间 (`main_max` 减去 Fit 占用与间距，不为负) 按权重分给 Fill 项;
/// - Fit 项溢出时不压缩 (M1 不做收缩，溢出部分由调用方裁剪)。
///
/// 返回每项的 (偏移，主轴尺寸),顺序与输入一致。
pub fn distribute(main_max: f32, gap: f32, children: &[FlowChild]) -> FlowResult {
    let n = children.len();
    if n == 0 {
        return Vec::new();
    }
    let gaps = gap * (n - 1) as f32;
    let fixed_sum: f32 = children
        .iter()
        .filter(|c| c.fill_weight == 0)
        .map(|c| c.main_fixed)
        .sum();
    let total_weight: u32 = children.iter().map(|c| c.fill_weight).sum();
    let remaining = (main_max - fixed_sum - gaps).max(0.0);

    let mut result = Vec::with_capacity(n);
    let mut offset = 0.0;
    for child in children {
        let size = if child.fill_weight == 0 {
            child.main_fixed
        } else {
            remaining * child.fill_weight as f32 / total_weight as f32
        };
        result.push((offset, size));
        offset += size + gap;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_edges() {
        let rect = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
        assert!(rect.contains(Point::new(10.0, 20.0))); // 左上角含
        assert!(rect.contains(Point::new(109.9, 69.9)));
        assert!(!rect.contains(Point::new(110.0, 70.0))); // 右下角不含
        assert!(!rect.contains(Point::new(9.9, 30.0)));
    }

    #[test]
    fn rect_intersection() {
        let a = Rect::from_xywh(0.0, 0.0, 100.0, 100.0);
        let b = Rect::from_xywh(50.0, 50.0, 100.0, 100.0);
        assert_eq!(
            a.intersect(&b),
            Some(Rect::from_xywh(50.0, 50.0, 50.0, 50.0))
        );

        // 不相交
        let c = Rect::from_xywh(200.0, 200.0, 10.0, 10.0);
        assert!(a.intersect(&c).is_none());

        // 边界接触 (视为不相交)
        let d = Rect::from_xywh(100.0, 0.0, 10.0, 10.0);
        assert!(a.intersect(&d).is_none());
    }

    #[test]
    fn rect_is_empty() {
        assert!(Rect::from_xywh(0.0, 0.0, 0.0, 10.0).is_empty());
        assert!(Rect::from_xywh(0.0, 0.0, 10.0, 0.0).is_empty());
        assert!(!Rect::from_xywh(0.0, 0.0, 10.0, 10.0).is_empty());
    }

    #[test]
    fn rect_inset() {
        let rect = Rect::from_xywh(10.0, 20.0, 100.0, 80.0);
        let inset = rect.inset(10.0);
        assert_eq!(inset, Rect::from_xywh(20.0, 30.0, 80.0, 60.0));

        // 内缩量过大时夹到零，避免负尺寸。
        let clamped = rect.inset(60.0);
        assert_eq!(clamped, Rect::from_xywh(70.0, 80.0, 0.0, 0.0));
    }

    #[test]
    fn edges_accumulation() {
        let e = Edges::symmetric(10.0, 20.0);
        assert_eq!(e.horizontal(), 20.0);
        assert_eq!(e.vertical(), 40.0);
        assert_eq!(Edges::all(5.0).horizontal(), 10.0);
    }

    #[test]
    fn color_from_srgb8() {
        let c = Color::from_srgb8(255, 0, 128);
        assert_eq!(c.r, 1.0);
        assert_eq!(c.g, 0.0);
        assert!((c.b - 128.0 / 255.0).abs() < f32::EPSILON);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn color_lerp_endpoints() {
        let a = Color::rgba(0.2, 0.4, 0.6, 0.8);
        let b = Color::rgba(0.8, 0.2, 0.4, 0.4);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
    }

    #[test]
    fn color_lerp_midpoint() {
        let a = Color::rgba(0.0, 0.2, 0.4, 0.0);
        let b = Color::rgba(1.0, 0.8, 0.0, 1.0);
        let mid = a.lerp(b, 0.5);
        assert!((mid.r - 0.5).abs() < f32::EPSILON);
        assert!((mid.g - 0.5).abs() < f32::EPSILON);
        assert!((mid.b - 0.2).abs() < f32::EPSILON);
        assert!((mid.a - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn color_lerp_clamps_t() {
        let a = Color::BLACK;
        let b = Color::WHITE;
        assert_eq!(a.lerp(b, -0.5), a);
        assert_eq!(a.lerp(b, 1.5), b);
    }

    #[test]
    fn constraints_tight_and_loose() {
        let tight = Constraints::tight(Size::new(100.0, 50.0));
        assert_eq!(
            tight.constrain(Size::new(10.0, 10.0)),
            Size::new(100.0, 50.0)
        );
        let loose = Constraints::loose(Size::new(100.0, 50.0));
        assert_eq!(
            loose.constrain(Size::new(200.0, 10.0)),
            Size::new(100.0, 10.0)
        );
        assert_eq!(
            loose.constrain(Size::new(20.0, 30.0)),
            Size::new(20.0, 30.0)
        );
    }

    #[test]
    fn constraints_deflate() {
        let c = Constraints::loose(Size::new(100.0, 100.0)).deflate(Edges::all(10.0));
        assert_eq!(c.max(), Size::new(80.0, 80.0));
        // 间距超过尺寸时夹到零
        let c2 = Constraints::loose(Size::new(10.0, 10.0)).deflate(Edges::all(20.0));
        assert_eq!(c2.max(), Size::ZERO);
    }

    #[test]
    fn distribute_all_fit() {
        let children = [
            FlowChild {
                main_fixed: 10.0,
                fill_weight: 0,
            },
            FlowChild {
                main_fixed: 20.0,
                fill_weight: 0,
            },
        ];
        let r = distribute(100.0, 5.0, &children);
        assert_eq!(r, vec![(0.0, 10.0), (15.0, 20.0)]);
    }

    #[test]
    fn distribute_fill_remaining() {
        let children = [
            FlowChild {
                main_fixed: 20.0,
                fill_weight: 0,
            },
            FlowChild {
                main_fixed: 0.0,
                fill_weight: 1,
            },
        ];
        let r = distribute(120.0, 10.0, &children);
        // 剩余 = 120 - 20 - 10 = 90，全给 fill
        assert_eq!(r, vec![(0.0, 20.0), (30.0, 90.0)]);
    }

    #[test]
    fn distribute_weighted_fills() {
        let children = [
            FlowChild {
                main_fixed: 0.0,
                fill_weight: 1,
            },
            FlowChild {
                main_fixed: 0.0,
                fill_weight: 3,
            },
        ];
        let r = distribute(90.0, 10.0, &children);
        // 剩余 80，按 1:3 分 → 20 / 60
        assert_eq!(r, vec![(0.0, 20.0), (30.0, 60.0)]);
    }

    #[test]
    fn distribute_overflow_keeps_fixed() {
        let children = [
            FlowChild {
                main_fixed: 80.0,
                fill_weight: 0,
            },
            FlowChild {
                main_fixed: 80.0,
                fill_weight: 0,
            },
            FlowChild {
                main_fixed: 0.0,
                fill_weight: 1,
            },
        ];
        let r = distribute(100.0, 0.0, &children);
        // 固定项不压缩;fill 项得到 max(0, 剩余)=0
        assert_eq!(r, vec![(0.0, 80.0), (80.0, 80.0), (160.0, 0.0)]);
    }

    #[test]
    fn distribute_empty_and_single() {
        assert!(distribute(100.0, 5.0, &[]).is_empty());
        let r = distribute(
            50.0,
            5.0,
            &[FlowChild {
                main_fixed: 0.0,
                fill_weight: 1,
            }],
        );
        assert_eq!(r, vec![(0.0, 50.0)]);
    }
}
