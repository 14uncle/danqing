//! @author 十四叔
//! @date 2026/07/17

//! 矩形族渲染管线 (SDF 圆角，实例化 quad)。
//!
//! 每帧用法：先经 [`RectBatch`] 收集矩形，再由管线一次绘制。
//! 绘制同时负责以清屏色开始 render pass(每帧第一个 pass)。

use crate::{Color, Rect};

/// 无裁剪时使用的极大安全矩形 (像素坐标)。
const NO_CLIP_MIN: [f32; 2] = [-1_000_000.0; 2];
const NO_CLIP_MAX: [f32; 2] = [1_000_000.0; 2];

/// 单个矩形实例的 GPU 数据布局。
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RectInstance {
    /// 左上角像素坐标。
    pos: [f32; 2],
    /// 像素尺寸。
    size: [f32; 2],
    /// RGBA 颜色。
    color: [f32; 4],
    /// 四角圆角半径 (像素),顺序：左上、右上、右下、左下。
    radii: [f32; 4],
    /// 旋转角度 (弧度), 绕矩形中心顺时针。
    rotation: f32,
    /// 裁剪矩形左上角。
    clip_min: [f32; 2],
    /// 裁剪矩形右下角 (不含)。
    clip_max: [f32; 2],
}

/// 矩形收集器：一帧内待绘制的矩形列表。
///
/// 组件树 paint 阶段向其中 push 矩形; 目前由应用层直接使用，
/// Task 8 起由组件树驱动。
#[derive(Debug, Default)]
pub struct RectBatch {
    instances: Vec<RectInstance>,
    /// 裁剪矩形栈;`None` 表示当前裁剪区为空 (完全裁剪)。
    clip_stack: Vec<Option<Rect>>,
}

impl RectBatch {
    /// 新建空收集器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 压入一个裁剪矩形。
    ///
    /// 后续 push 的矩形会被裁剪到该矩形与所有祖先裁剪矩形的交集。
    /// 必须在子组件 paint 前调用，并在 paint 后调用 [`Self::pop_clip`]。
    pub fn push_clip(&mut self, rect: Rect) {
        let next = match self.current_clip() {
            Some(parent) => parent.intersect(&rect),
            None => Some(rect),
        };
        self.clip_stack.push(next);
    }

    /// 弹出当前裁剪矩形，恢复上一层裁剪状态。
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn current_clip(&self) -> Option<Rect> {
        self.clip_stack.iter().rev().find_map(|r| *r)
    }

    /// 添加一个矩形 (颜色与统一圆角半径，半径 0 为直角)。
    pub fn push_rect(&mut self, rect: Rect, color: Color, radius: f32) {
        self.push_rounded_rect(rect, color, [radius; 4]);
    }

    /// 添加一个矩形 (颜色与逐角圆角半径)。
    ///
    /// `radii` 顺序：左上、右上、右下、左下。
    pub fn push_rounded_rect(&mut self, rect: Rect, color: Color, radii: [f32; 4]) {
        let (clip_min, clip_max) = match self.current_clip() {
            Some(clip) => match clip.intersect(&rect) {
                Some(intersection) => (
                    [intersection.origin.x, intersection.origin.y],
                    [
                        intersection.origin.x + intersection.size.width,
                        intersection.origin.y + intersection.size.height,
                    ],
                ),
                None => return,
            },
            None => (NO_CLIP_MIN, NO_CLIP_MAX),
        };
        self.instances.push(RectInstance {
            pos: [rect.origin.x, rect.origin.y],
            size: [rect.size.width, rect.size.height],
            color: [color.r, color.g, color.b, color.a],
            radii,
            rotation: 0.0,
            clip_min,
            clip_max,
        });
    }

    /// 添加一条沿圆角矩形边框的虚线 (划线 - 空隙式)。
    ///
    /// 从顶边左端出发顺时针走一整圈，四条直边与四个圆角共享同一
    /// 划线 - 空隙相位 (按周长弧长推进),虚线节奏绕角连续、首尾相接。
    /// 直边为长条形 dash，圆角处用半步重叠的小圆点融成平滑弧段。
    /// 描边路径内缩半线宽，划线整体落在 `rect` 内侧,外缘与 `rect`
    /// 边缘对齐，四边留白严格一致。
    /// `dash` 为每段划线长度，`gap` 为空隙长度，`thickness` 为线宽。
    pub fn push_dashed_border(
        &mut self,
        rect: Rect,
        color: Color,
        radius: f32,
        dash: f32,
        gap: f32,
        thickness: f32,
    ) {
        let step = dash + gap;
        if step <= 0.0 || dash <= 0.0 || thickness <= 0.0 {
            return;
        }
        let r = radius
            .max(0.0)
            .min(rect.size.width * 0.5)
            .min(rect.size.height * 0.5);
        let straight_w = (rect.size.width - 2.0 * r).max(0.0);
        let straight_h = (rect.size.height - 2.0 * r).max(0.0);

        // 路径内缩半线宽:直边内移、弧半径收缩 (圆心不变),
        // 使划线外缘与 rect 边缘对齐而非骑跨边缘。
        let half = thickness * 0.5;
        let ir = (r - half).max(0.0);
        let arc_len = std::f32::consts::FRAC_PI_2 * ir;

        let x0 = rect.origin.x;
        let y0 = rect.origin.y;
        let x1 = x0 + rect.size.width;
        let y1 = y0 + rect.size.height;
        let ix0 = x0 + half;
        let iy0 = y0 + half;
        let ix1 = x1 - half;
        let iy1 = y1 - half;

        // 已走过的周长弧长，作为各段虚线的相位逐段传递。
        let mut phase = 0.0f32;

        // 顶边 (左→右) 与右上弧 (θ: -π/2 → 0)。
        push_dashed_hline(
            self,
            x0 + r,
            iy0,
            straight_w,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
        phase += straight_w;
        push_dashed_arc(
            self,
            x1 - r,
            y0 + r,
            ir,
            -std::f32::consts::FRAC_PI_2,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
        phase += arc_len;

        // 右边 (上→下) 与右下弧 (θ: 0 → π/2)。
        push_dashed_vline(
            self,
            ix1,
            y0 + r,
            straight_h,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
        phase += straight_h;
        push_dashed_arc(
            self,
            x1 - r,
            y1 - r,
            ir,
            0.0,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
        phase += arc_len;

        // 底边 (右→左) 与左下弧 (θ: π/2 → π)。
        push_dashed_hline(
            self,
            x1 - r,
            iy1,
            -straight_w,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
        phase += straight_w;
        push_dashed_arc(
            self,
            x0 + r,
            y1 - r,
            ir,
            std::f32::consts::FRAC_PI_2,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
        phase += arc_len;

        // 左边 (下→上) 与左上弧 (θ: π → 3π/2)。
        push_dashed_vline(
            self,
            ix0,
            y1 - r,
            -straight_h,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
        phase += straight_h;
        push_dashed_arc(
            self,
            x0 + r,
            y0 + r,
            ir,
            std::f32::consts::PI,
            color,
            dash,
            gap,
            thickness,
            phase,
        );
    }

    /// 添加一条沿圆角矩形边框的实线描边。
    ///
    /// 四条直边为矩形长条，四个圆角用沿圆弧排列的小矩形衔接，
    /// 半径取 `thickness/2` 以自然融合成平滑弧线，从而跟随组件圆角。
    /// `thickness` 为线宽。
    ///
    /// 描边矩形先四舍五入对齐到最近的整数像素边界: 1px 细线落在分数
    /// 坐标时覆盖率被拆到两行像素 (输入框底边发虚的根因), 对齐后落在
    /// 完整像素行上满强度渲染。绘制填充 + 描边的表面组件应把同一个
    /// [`Rect::snap_to_pixels`] 结果传给两者 (见 `Box::paint`),
    /// 使描边外缘与填充轮廓精确重合 (border-box 贴合)。
    pub fn push_rounded_border(&mut self, rect: Rect, color: Color, radius: f32, thickness: f32) {
        if thickness <= 0.0 {
            return;
        }
        let rect = rect.snap_to_pixels();
        let r = radius
            .max(0.0)
            .min(rect.size.width * 0.5)
            .min(rect.size.height * 0.5);
        let straight_w = (rect.size.width - 2.0 * r).max(0.0);
        let straight_h = (rect.size.height - 2.0 * r).max(0.0);
        let half = thickness * 0.5;

        // 四条直边: 整体内缩在矩形边界之内 (描边不跨边界),
        // 否则 Scrollable 等裁剪边界会削掉外凸的半线宽, 边线发虚甚至消失。
        self.push_rect(
            Rect::from_xywh(rect.origin.x + r, rect.origin.y, straight_w, thickness),
            color,
            0.0,
        );
        self.push_rect(
            Rect::from_xywh(
                rect.origin.x + r,
                rect.origin.y + rect.size.height - thickness,
                straight_w,
                thickness,
            ),
            color,
            0.0,
        );
        self.push_rect(
            Rect::from_xywh(rect.origin.x, rect.origin.y + r, thickness, straight_h),
            color,
            0.0,
        );
        self.push_rect(
            Rect::from_xywh(
                rect.origin.x + rect.size.width - thickness,
                rect.origin.y + r,
                thickness,
                straight_h,
            ),
            color,
            0.0,
        );

        // 四个圆角：沿 90° 圆弧等距放置小矩形，步长为 half 使弧线更平滑。
        // 弧半径内缩 half (r - half), 使圆点外缘恰好贴合矩形边界。
        // 顺序：左上、右上、右下、左下，每段从一条直边过渡到相邻直边。
        if r > half {
            let arc_r = r - half;
            let corner_len = std::f32::consts::FRAC_PI_2 * arc_r;
            let segments = (corner_len / half).ceil().max(2.0) as usize;
            let angle_step = std::f32::consts::FRAC_PI_2 / segments as f32;
            for corner_idx in 0..4 {
                let (cx, cy, start_theta) = match corner_idx {
                    0 => (rect.origin.x + r, rect.origin.y + r, std::f32::consts::PI),
                    1 => (
                        rect.origin.x + rect.size.width - r,
                        rect.origin.y + r,
                        std::f32::consts::PI + std::f32::consts::FRAC_PI_2,
                    ),
                    2 => (
                        rect.origin.x + rect.size.width - r,
                        rect.origin.y + rect.size.height - r,
                        0.0,
                    ),
                    3 => (
                        rect.origin.x + r,
                        rect.origin.y + rect.size.height - r,
                        std::f32::consts::FRAC_PI_2,
                    ),
                    _ => unreachable!(),
                };
                for i in 0..=segments {
                    let theta = start_theta + i as f32 * angle_step;
                    let px = cx + arc_r * theta.cos();
                    let py = cy + arc_r * theta.sin();
                    self.push_rect(
                        Rect::from_xywh(px - half, py - half, thickness, thickness),
                        color,
                        half,
                    );
                }
            }
        }
    }

    /// 添加一条线段 (以旋转的细圆角矩形表示)。
    ///
    /// 从 `p1` 绘制到 `p2`, 线宽为 `thickness`; 端点带圆角，过渡自然。
    /// 利用实例的 `rotation` 字段让细矩形沿线段方向摆放，因此可绘制
    /// 任意角度的直线，用于标题栏按钮符号等几何图形。
    pub fn push_line(&mut self, p1: crate::Point, p2: crate::Point, thickness: f32, color: Color) {
        if thickness <= 0.0 {
            return;
        }
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let length_sq = dx * dx + dy * dy;
        if length_sq < 1e-12 {
            return;
        }
        let length = length_sq.sqrt();
        let angle = dy.atan2(dx);
        let half = thickness * 0.5;

        // 线段的轴对齐包围盒 (含端点半径), 用于与裁剪区求交。
        let min_x = p1.x.min(p2.x) - half;
        let max_x = p1.x.max(p2.x) + half;
        let min_y = p1.y.min(p2.y) - half;
        let max_y = p1.y.max(p2.y) + half;
        let bbox = Rect::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y);

        let (clip_min, clip_max) = match self.current_clip() {
            Some(clip) => match clip.intersect(&bbox) {
                Some(intersection) => (
                    [intersection.origin.x, intersection.origin.y],
                    [
                        intersection.origin.x + intersection.size.width,
                        intersection.origin.y + intersection.size.height,
                    ],
                ),
                None => return,
            },
            None => (NO_CLIP_MIN, NO_CLIP_MAX),
        };

        // 细矩形中心与线段中心重合，尺寸为 (length + thickness) × thickness,
        // 旋转后两端自然形成半圆端点。
        let size = crate::Size::new(length + thickness, thickness);
        let center = crate::Point::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5);
        let pos = crate::Point::new(center.x - size.width * 0.5, center.y - size.height * 0.5);

        self.instances.push(RectInstance {
            pos: [pos.x, pos.y],
            size: [size.width, size.height],
            color: [color.r, color.g, color.b, color.a],
            radii: [half; 4],
            rotation: angle,
            clip_min,
            clip_max,
        });
    }

    /// 矩形数量。
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// 测试用：读取所有实例的颜色 (不参与公开 API 契约)。
    #[doc(hidden)]
    pub fn instance_colors(&self) -> Vec<[f32; 4]> {
        self.instances.iter().map(|i| i.color).collect()
    }

    /// 测试用：读取所有实例的逐角圆角半径 (不参与公开 API 契约)。
    #[doc(hidden)]
    pub fn instance_radii(&self) -> Vec<[f32; 4]> {
        self.instances.iter().map(|i| i.radii).collect()
    }

    /// 测试用：读取所有实例的矩形 (不参与公开 API 契约)。
    #[doc(hidden)]
    pub fn instance_rects(&self) -> Vec<Rect> {
        self.instances
            .iter()
            .map(|i| Rect::from_xywh(i.pos[0], i.pos[1], i.size[0], i.size[1]))
            .collect()
    }
}

/// 沿水平方向绘制一段划线 - 空隙虚线 (长度可为负，表示从右向左)。
///
/// `phase` 为本段起点之前已走过的弧长，虚线节奏据此延续而非重新开始。
#[allow(clippy::too_many_arguments)]
fn push_dashed_hline(
    rects: &mut RectBatch,
    x0: f32,
    y: f32,
    len: f32,
    color: Color,
    dash: f32,
    gap: f32,
    thickness: f32,
    phase: f32,
) {
    let step = dash + gap;
    if step <= 0.0 || dash <= 0.0 || thickness <= 0.0 {
        return;
    }
    let abs_len = len.abs();
    let dir = len.signum();
    for (a, b) in dash_on_intervals(phase, abs_len, dash, step) {
        let lo = x0 + dir * a;
        let hi = x0 + dir * b;
        rects.push_rect(
            Rect::from_xywh(lo.min(hi), y - thickness * 0.5, (hi - lo).abs(), thickness),
            color,
            0.0,
        );
    }
}

/// 沿垂直方向绘制一段划线 - 空隙虚线 (长度可为负，表示从下向上)。
///
/// `phase` 含义同 [`push_dashed_hline`]。
#[allow(clippy::too_many_arguments)]
fn push_dashed_vline(
    rects: &mut RectBatch,
    x: f32,
    y0: f32,
    len: f32,
    color: Color,
    dash: f32,
    gap: f32,
    thickness: f32,
    phase: f32,
) {
    let step = dash + gap;
    if step <= 0.0 || dash <= 0.0 || thickness <= 0.0 {
        return;
    }
    let abs_len = len.abs();
    let dir = len.signum();
    for (a, b) in dash_on_intervals(phase, abs_len, dash, step) {
        let lo = y0 + dir * a;
        let hi = y0 + dir * b;
        rects.push_rect(
            Rect::from_xywh(x - thickness * 0.5, lo.min(hi), thickness, (hi - lo).abs()),
            color,
            0.0,
        );
    }
}

/// 沿 90° 圆弧绘制一段划线 - 空隙虚线 (θ 随弧长递增，屏幕坐标下即顺时针)。
///
/// 以小圆点沿弧半步重叠排列，仅在划线相位内落点，融成平滑弧段。
/// 与直边按区间精确填充不同，弧上按固定步长采样,dash 端点精度为
/// ±`thickness/2`，极端参数 (`dash` 小于步长) 下节奏可能与直边错位。
/// `phase` 含义同 [`push_dashed_hline`]。
#[allow(clippy::too_many_arguments)]
fn push_dashed_arc(
    rects: &mut RectBatch,
    cx: f32,
    cy: f32,
    r: f32,
    start_theta: f32,
    color: Color,
    dash: f32,
    gap: f32,
    thickness: f32,
    phase: f32,
) {
    let step = dash + gap;
    let arc_len = std::f32::consts::FRAC_PI_2 * r;
    if step <= 0.0 || dash <= 0.0 || thickness <= 0.0 || r <= 0.0 {
        return;
    }
    let half = thickness * 0.5;
    let march = half.max(0.25);
    let mut d = 0.0f32;
    while d < arc_len {
        if (phase + d).rem_euclid(step) < dash {
            let theta = start_theta + d / r;
            let px = cx + r * theta.cos();
            let py = cy + r * theta.sin();
            rects.push_rect(
                Rect::from_xywh(px - half, py - half, thickness, thickness),
                color,
                half,
            );
        }
        d += march;
    }
}

/// 列出 `[0, len]` 内处于划线段 (on) 的局部距离区间，已按 `phase` 相位偏移。
fn dash_on_intervals(phase: f32, len: f32, dash: f32, step: f32) -> Vec<(f32, f32)> {
    let mut intervals = Vec::new();
    // 当前相位所处划线段的局部起点 (可能为负，表示 phase 落在划线中段)。
    let mut seg_start = -phase.rem_euclid(step);
    if seg_start + dash <= 0.0 {
        seg_start += step;
    }
    while seg_start < len {
        let a = seg_start.max(0.0);
        let b = (seg_start + dash).min(len);
        if b > a {
            intervals.push((a, b));
        }
        seg_start += step;
    }
    intervals
}

/// 一帧的绘制目标与参数。
pub struct DrawTarget<'a> {
    /// 帧纹理视图。
    pub view: &'a wgpu::TextureView,
    /// 目标宽度 (像素)。
    pub width: f32,
    /// 目标高度 (像素)。
    pub height: f32,
    /// 清屏颜色。
    pub clear_color: Color,
}

/// 矩形渲染管线。持有 GPU 管线、uniform 与实例缓冲。
pub struct RectPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instance_buf: wgpu::Buffer,
    /// 实例缓冲当前容量 (实例个数)。
    capacity: usize,
}

impl RectPipeline {
    const INITIAL_CAPACITY: usize = 256;

    /// 创建管线，target 为 surface 颜色格式。
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rect shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("rect.wgsl").into()),
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rect uniforms"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect uniform buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rect bind group"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rect pipeline layout"),
            bind_group_layouts: &[Some(&bind_layout)],
            immediate_size: 0,
        });
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<RectInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // pos
                1 => Float32x2, // size
                2 => Float32x4, // color
                3 => Float32x4, // radii
                4 => Float32,   // rotation (弧度，绕矩形中心)
                5 => Float32x2, // clip_min
                6 => Float32x2, // clip_max
            ],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(instance_layout)],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect instance buffer"),
            size: (Self::INITIAL_CAPACITY * size_of::<RectInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_buf,
            bind_group,
            instance_buf,
            capacity: Self::INITIAL_CAPACITY,
        }
    }

    /// 上传屏幕尺寸 uniform(像素 → clip 的缩放依据)。
    fn write_screen_uniform(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        let data = [width, height, 0.0, 0.0];
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&data));
    }

    /// 确保实例缓冲容量足够 (不足则扩容到下一个 2 的幂)。
    fn ensure_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.capacity {
            return;
        }
        let new_capacity = needed.next_power_of_two();
        self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rect instance buffer"),
            size: (new_capacity * size_of::<RectInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.capacity = new_capacity;
    }

    /// 开始 render pass 并绘制收集到的全部矩形。
    ///
    /// `clear` 为 true 时以 `target.clear_color` 清屏; 为 false 时保留已有内容，
    /// 用于背景图已绘制的情况。
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &DrawTarget,
        batch: &RectBatch,
        clear: bool,
    ) {
        self.write_screen_uniform(queue, target.width, target.height);
        self.ensure_capacity(device, batch.len());
        if !batch.is_empty() {
            queue.write_buffer(
                &self.instance_buf,
                0,
                bytemuck::cast_slice(&batch.instances),
            );
        }

        let load = if clear {
            wgpu::LoadOp::Clear(wgpu::Color {
                r: f64::from(target.clear_color.r),
                g: f64::from(target.clear_color.g),
                b: f64::from(target.clear_color.b),
                a: f64::from(target.clear_color.a),
            })
        } else {
            wgpu::LoadOp::Load
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("rect pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if !batch.is_empty() {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.instance_buf.slice(..));
            pass.draw(0..6, 0..batch.len() as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Rect};

    #[test]
    fn push_rounded_rect_keeps_per_corner_radii() {
        let mut batch = RectBatch::new();
        batch.push_rounded_rect(
            Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
            Color::BLACK,
            [0.0, 4.0, 0.0, 0.0],
        );
        assert_eq!(batch.instance_radii(), vec![[0.0, 4.0, 0.0, 0.0]]);
    }

    #[test]
    fn push_rect_expands_single_radius_to_all_corners() {
        let mut batch = RectBatch::new();
        batch.push_rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0), Color::BLACK, 3.0);
        assert_eq!(batch.instance_radii(), vec![[3.0; 4]]);
    }

    #[test]
    fn clip_stack_skips_fully_clipped_rects() {
        let mut batch = RectBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 10.0, 10.0));
        batch.push_rect(Rect::from_xywh(20.0, 20.0, 10.0, 10.0), Color::BLACK, 0.0);
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn clip_stack_keeps_partially_visible_rects() {
        let mut batch = RectBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 15.0, 10.0));
        batch.push_rect(Rect::from_xywh(10.0, 0.0, 10.0, 10.0), Color::BLACK, 0.0);
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn nested_clip_intersects() {
        let mut batch = RectBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 100.0, 100.0));
        batch.push_clip(Rect::from_xywh(50.0, 50.0, 100.0, 100.0));
        batch.push_rect(Rect::from_xywh(0.0, 0.0, 60.0, 60.0), Color::BLACK, 0.0);
        assert_eq!(batch.len(), 1);
        batch.pop_clip();
        batch.pop_clip();
        batch.push_rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0), Color::BLACK, 0.0);
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn empty_clip_skips_all() {
        let mut batch = RectBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 0.0, 10.0));
        batch.push_rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0), Color::BLACK, 0.0);
        assert_eq!(batch.len(), 0);
        batch.pop_clip();
        batch.push_rect(Rect::from_xywh(0.0, 0.0, 10.0, 10.0), Color::BLACK, 0.0);
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn dash_on_intervals_offsets_by_phase() {
        // 无相位：从 0 开始的常规节奏。
        assert_eq!(
            dash_on_intervals(0.0, 10.0, 4.0, 6.0),
            vec![(0.0, 4.0), (6.0, 10.0)]
        );
        // 相位落在空隙：整段无划线。
        assert!(dash_on_intervals(4.0, 2.0, 4.0, 6.0).is_empty());
        // 相位落在空隙尾部：空隙结束后开始划线并截断到段尾。
        assert_eq!(dash_on_intervals(5.0, 4.0, 4.0, 6.0), vec![(1.0, 4.0)]);
        // 相位落在划线中段：段首直接续画。
        assert_eq!(dash_on_intervals(2.0, 2.0, 4.0, 6.0), vec![(0.0, 2.0)]);
    }

    #[test]
    fn dashed_border_carries_phase_across_edges() {
        // 直角 10x10, 周长 40, 节奏 4+2: 顶边画完相位 10,
        // 右边须空 2px 后续画, 底边从划线中段开始, 左边恰好重新对齐。
        let mut batch = RectBatch::new();
        batch.push_dashed_border(
            Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
            Color::WHITE,
            0.0,
            4.0,
            2.0,
            1.0,
        );
        let rects = batch.instance_rects();
        let expected = [
            (0.0, 0.0, 4.0, 1.0),
            (6.0, 0.0, 4.0, 1.0),
            (9.0, 2.0, 1.0, 4.0),
            (9.0, 8.0, 1.0, 2.0),
            (8.0, 9.0, 2.0, 1.0),
            (2.0, 9.0, 4.0, 1.0),
            (0.0, 6.0, 1.0, 4.0),
            (0.0, 0.0, 1.0, 4.0),
        ];
        assert_eq!(rects.len(), expected.len());
        for e in expected {
            assert!(
                rects.iter().any(|r| {
                    (r.origin.x - e.0).abs() < 1e-4
                        && (r.origin.y - e.1).abs() < 1e-4
                        && (r.size.width - e.2).abs() < 1e-4
                        && (r.size.height - e.3).abs() < 1e-4
                }),
                "缺少划线 {e:?}"
            );
        }
    }

    #[test]
    fn dashed_border_arc_dots_follow_shared_phase() {
        // 顶边恰好一段划线 (4px), 随后右上弧整段落在空隙内, 不应有圆点。
        let mut batch = RectBatch::new();
        batch.push_dashed_border(
            Rect::from_xywh(0.0, 0.0, 6.0, 4.0),
            Color::WHITE,
            1.0,
            4.0,
            2.0,
            1.0,
        );
        let dots: Vec<Rect> = batch
            .instance_rects()
            .into_iter()
            .zip(batch.instance_radii())
            .filter(|(_, radii)| radii[0] > 0.0)
            .map(|(r, _)| r)
            .collect();
        assert_eq!(dots.len(), 5);
        // 右上弧区域 (圆心 (5,1) 附近) 不应有圆点。
        assert!(dots.iter().all(|r| {
            let cx = r.origin.x + 0.5;
            let cy = r.origin.y + 0.5;
            !(cx > 4.0 && cy < 1.5)
        }));
    }

    #[test]
    fn dashed_border_degenerate_inputs_are_safe() {
        let rect = Rect::from_xywh(0.0, 0.0, 10.0, 10.0);

        // dash / thickness 为 0 以及零尺寸 rect: 整体跳过, 无实例。
        let mut batch = RectBatch::new();
        batch.push_dashed_border(rect, Color::WHITE, 4.0, 0.0, 2.0, 1.0);
        batch.push_dashed_border(rect, Color::WHITE, 4.0, 4.0, 2.0, 0.0);
        batch.push_dashed_border(
            Rect::from_xywh(0.0, 0.0, 0.0, 0.0),
            Color::WHITE,
            4.0,
            4.0,
            2.0,
            1.0,
        );
        assert!(batch.is_empty());

        // 负半径按 0 钳制, 行为与 radius=0 一致。
        let mut neg = RectBatch::new();
        neg.push_dashed_border(rect, Color::WHITE, -5.0, 4.0, 2.0, 1.0);
        let mut zero = RectBatch::new();
        zero.push_dashed_border(rect, Color::WHITE, 0.0, 4.0, 2.0, 1.0);
        assert_eq!(neg.len(), zero.len());

        // 超大半径 (钳到 min(w,h)/2)、半径小于半线宽 (弧被跳过)、gap 为 0:
        // 均不 panic 且实例数有界。
        let mut batch = RectBatch::new();
        batch.push_dashed_border(rect, Color::WHITE, 99.0, 4.0, 2.0, 1.0);
        batch.push_dashed_border(rect, Color::WHITE, 1.0, 4.0, 2.0, 4.0);
        batch.push_dashed_border(rect, Color::WHITE, 4.0, 4.0, 0.0, 1.0);
        assert!(batch.len() < 256);
    }

    #[test]
    fn rounded_border_negative_radius_behaves_like_zero() {
        let rect = Rect::from_xywh(0.0, 0.0, 10.0, 10.0);
        let mut neg = RectBatch::new();
        neg.push_rounded_border(rect, Color::WHITE, -5.0, 1.0);
        let mut zero = RectBatch::new();
        zero.push_rounded_border(rect, Color::WHITE, 0.0, 1.0);
        assert_eq!(neg.instance_rects(), zero.instance_rects());
    }

    #[test]
    fn rounded_border_stays_inside_rect() {
        // 描边内缩: 所有实例 (直边 + 圆角点) 必须完全落在矩形内部,
        // 否则在 Scrollable 裁剪边界处外凸的半线宽会被裁掉, 导致边线发虚或消失。
        let rect = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
        let mut batch = RectBatch::new();
        batch.push_rounded_border(rect, Color::WHITE, 6.0, 1.0);
        assert!(!batch.instance_rects().is_empty());
        for inst in batch.instance_rects() {
            let eps = 1e-4;
            assert!(
                inst.origin.x >= rect.origin.x - eps
                    && inst.origin.y >= rect.origin.y - eps
                    && inst.origin.x + inst.size.width <= rect.origin.x + rect.size.width + eps
                    && inst.origin.y + inst.size.height <= rect.origin.y + rect.size.height + eps,
                "描边实例应完全落在矩形内部: {inst:?}"
            );
        }
    }

    #[test]
    fn rounded_border_keeps_full_thickness_under_clip() {
        // 描边与裁剪边界重合时仍保持完整线宽 (内缩前会被裁掉一半)。
        let rect = Rect::from_xywh(0.0, 0.0, 100.0, 50.0);
        let mut batch = RectBatch::new();
        batch.push_clip(rect);
        batch.push_rounded_border(rect, Color::WHITE, 6.0, 2.0);
        batch.pop_clip();
        let top_edge = batch.instance_rects().iter().any(|r| {
            (r.origin.y - rect.origin.y).abs() < 1e-4 && (r.size.height - 2.0).abs() < 1e-4
        });
        assert!(top_edge, "顶边应保持完整 2px 线宽且与矩形顶对齐");
    }

    #[test]
    fn rounded_border_aligns_to_nearest_pixel_grid() {
        // 细线满强度 + 贴合的组合不变式:
        // 1. 描边外缘与 snap_to_pixels 后的矩形四边精确重合 (与填充共用轮廓);
        // 2. 每边相对原矩形偏移不超过 0.5px (四舍五入, 而非单向内缩——
        //    单向内缩会让填充在描边外侧露出一圈底色, 卡片边框不贴合的回归);
        // 3. 1px 直边落在完整像素行上 (整数坐标满强度渲染, 底边发虚的回归)。
        let rect = Rect::from_xywh(285.3, 142.553, 240.0, 35.951);
        let snapped = rect.snap_to_pixels();
        let mut batch = RectBatch::new();
        batch.push_rounded_border(rect, Color::WHITE, 6.0, 1.0);
        let rects = batch.instance_rects();
        assert!(!rects.is_empty());
        let eps = 1e-4;
        let (sx1, sy1) = (
            snapped.origin.x + snapped.size.width,
            snapped.origin.y + snapped.size.height,
        );
        let touches = |pred: &dyn Fn(&Rect) -> bool| rects.iter().any(pred);
        assert!(
            touches(&|r: &Rect| (r.origin.y - snapped.origin.y).abs() < eps),
            "顶缘应与对齐矩形顶重合"
        );
        assert!(
            touches(&|r: &Rect| (r.origin.y + r.size.height - sy1).abs() < eps),
            "底缘应与对齐矩形底重合"
        );
        assert!(
            touches(&|r: &Rect| (r.origin.x - snapped.origin.x).abs() < eps),
            "左缘应与对齐矩形左重合"
        );
        assert!(
            touches(&|r: &Rect| (r.origin.x + r.size.width - sx1).abs() < eps),
            "右缘应与对齐矩形右重合"
        );
        // 四舍五入: 每边偏移 ≤ 0.5px。
        assert!((snapped.origin.x - rect.origin.x).abs() <= 0.5 + eps);
        assert!((snapped.origin.y - rect.origin.y).abs() <= 0.5 + eps);
        assert!((sx1 - (rect.origin.x + rect.size.width)).abs() <= 0.5 + eps);
        assert!((sy1 - (rect.origin.y + rect.size.height)).abs() <= 0.5 + eps);
    }
}
