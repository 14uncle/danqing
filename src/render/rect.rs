//! @author 十四叔
//! @date 2026/07/17

//! 矩形族渲染管线(SDF 圆角,实例化 quad)。
//!
//! 每帧用法:先经 [`RectBatch`] 收集矩形,再由管线一次绘制。
//! 绘制同时负责以清屏色开始 render pass(每帧第一个 pass)。

use crate::{Color, Rect};

/// 无裁剪时使用的极大安全矩形(像素坐标)。
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
    /// 圆角半径(像素)。
    radius: f32,
    /// 对齐填充。
    _pad: f32,
    /// 裁剪矩形左上角。
    clip_min: [f32; 2],
    /// 裁剪矩形右下角(不含)。
    clip_max: [f32; 2],
}

/// 矩形收集器:一帧内待绘制的矩形列表。
///
/// 组件树 paint 阶段向其中 push 矩形;目前由应用层直接使用,
/// Task 8 起由组件树驱动。
#[derive(Debug, Default)]
pub struct RectBatch {
    instances: Vec<RectInstance>,
    /// 裁剪矩形栈;`None` 表示当前裁剪区为空(完全裁剪)。
    clip_stack: Vec<Option<crate::Rect>>,
}

impl RectBatch {
    /// 新建空收集器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 压入一个裁剪矩形。
    ///
    /// 后续 push 的矩形会被裁剪到该矩形与所有祖先裁剪矩形的交集。
    /// 必须在子组件 paint 前调用,并在 paint 后调用 [`Self::pop_clip`]。
    pub fn push_clip(&mut self, rect: crate::Rect) {
        let next = match self.current_clip() {
            Some(parent) => parent.intersect(&rect),
            None => Some(rect),
        };
        self.clip_stack.push(next);
    }

    /// 弹出当前裁剪矩形,恢复上一层裁剪状态。
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn current_clip(&self) -> Option<crate::Rect> {
        self.clip_stack.iter().rev().find_map(|r| *r)
    }

    /// 添加一个矩形(颜色与圆角半径,半径 0 为直角)。
    pub fn push_rect(&mut self, rect: Rect, color: Color, radius: f32) {
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
            radius,
            _pad: 0.0,
            clip_min,
            clip_max,
        });
    }

    /// 添加一条沿圆角矩形边框的虚线(划线-空隙式)。
    ///
    /// 四条直边为长条形 dash,四个圆角用等距小圆点衔接,
    /// 从而跟随组件圆角。`dash` 为每段划线长度,`gap` 为空隙长度,
    /// `thickness` 为线宽。
    pub fn push_dashed_border(
        &mut self,
        rect: Rect,
        color: Color,
        radius: f32,
        dash: f32,
        gap: f32,
        thickness: f32,
    ) {
        let r = radius
            .min(rect.size.width * 0.5)
            .min(rect.size.height * 0.5);
        let straight_w = (rect.size.width - 2.0 * r).max(0.0);
        let straight_h = (rect.size.height - 2.0 * r).max(0.0);

        // 四条直边
        push_dashed_hline(
            self,
            rect.origin.x + r,
            rect.origin.y,
            straight_w,
            color,
            dash,
            gap,
            thickness,
        );
        push_dashed_hline(
            self,
            rect.origin.x + rect.size.width - r,
            rect.origin.y + rect.size.height,
            -straight_w,
            color,
            dash,
            gap,
            thickness,
        );
        push_dashed_vline(
            self,
            rect.origin.x + rect.size.width,
            rect.origin.y + r,
            straight_h,
            color,
            dash,
            gap,
            thickness,
        );
        push_dashed_vline(
            self,
            rect.origin.x,
            rect.origin.y + rect.size.height - r,
            -straight_h,
            color,
            dash,
            gap,
            thickness,
        );

        // 四个圆角:用与线宽等大的小圆点近似,顺序左上、右上、右下、左下。
        if r > 0.0 && thickness > 0.0 {
            let half = thickness * 0.5;
            let corner_step = thickness + gap;
            let corner_len = std::f32::consts::FRAC_PI_2 * r;
            if corner_step > 0.0 {
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
                    let mut d = 0.0f32;
                    while d < corner_len {
                        let t = d / corner_len;
                        let theta = start_theta + t * std::f32::consts::FRAC_PI_2;
                        let px = cx + r * theta.cos();
                        let py = cy + r * theta.sin();
                        self.push_rect(
                            Rect::from_xywh(px - half, py - half, thickness, thickness),
                            color,
                            half,
                        );
                        d += corner_step;
                    }
                }
            }
        }
    }

    /// 添加一条沿圆角矩形边框的实线描边。
    ///
    /// 四条直边为矩形长条,四个圆角用沿圆弧排列的小矩形衔接,
    /// 半径取 `thickness/2` 以自然融合成平滑弧线,从而跟随组件圆角。
    /// `thickness` 为线宽。
    pub fn push_rounded_border(&mut self, rect: Rect, color: Color, radius: f32, thickness: f32) {
        if thickness <= 0.0 {
            return;
        }
        let r = radius
            .min(rect.size.width * 0.5)
            .min(rect.size.height * 0.5);
        let straight_w = (rect.size.width - 2.0 * r).max(0.0);
        let straight_h = (rect.size.height - 2.0 * r).max(0.0);
        let half = thickness * 0.5;

        // 四条直边
        self.push_rect(
            Rect::from_xywh(
                rect.origin.x + r,
                rect.origin.y - half,
                straight_w,
                thickness,
            ),
            color,
            0.0,
        );
        self.push_rect(
            Rect::from_xywh(
                rect.origin.x + r,
                rect.origin.y + rect.size.height - half,
                straight_w,
                thickness,
            ),
            color,
            0.0,
        );
        self.push_rect(
            Rect::from_xywh(
                rect.origin.x - half,
                rect.origin.y + r,
                thickness,
                straight_h,
            ),
            color,
            0.0,
        );
        self.push_rect(
            Rect::from_xywh(
                rect.origin.x + rect.size.width - half,
                rect.origin.y + r,
                thickness,
                straight_h,
            ),
            color,
            0.0,
        );

        // 四个圆角:沿 90° 圆弧等距放置小矩形,步长为 half 使弧线更平滑。
        // 顺序:左上、右上、右下、左下,每段从一条直边过渡到相邻直边。
        if r > 0.0 {
            let corner_len = std::f32::consts::FRAC_PI_2 * r;
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
                    let px = cx + r * theta.cos();
                    let py = cy + r * theta.sin();
                    self.push_rect(
                        Rect::from_xywh(px - half, py - half, thickness, thickness),
                        color,
                        half,
                    );
                }
            }
        }
    }

    /// 矩形数量。
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// 测试用:读取所有实例的颜色(不参与公开 API 契约)。
    #[doc(hidden)]
    pub fn instance_colors(&self) -> Vec<[f32; 4]> {
        self.instances.iter().map(|i| i.color).collect()
    }

    /// 测试用:读取所有实例的矩形(不参与公开 API 契约)。
    #[doc(hidden)]
    pub fn instance_rects(&self) -> Vec<Rect> {
        self.instances
            .iter()
            .map(|i| Rect::from_xywh(i.pos[0], i.pos[1], i.size[0], i.size[1]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Rect};

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
}

/// 沿水平方向绘制一段划线-空隙虚线(长度可为负,表示从右向左)。
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
) {
    let step = dash + gap;
    if step <= 0.0 || thickness <= 0.0 {
        return;
    }
    let abs_len = len.abs();
    let dir = len.signum();
    let mut dist = 0.0f32;
    while dist < abs_len {
        let seg = dash.min(abs_len - dist);
        let start_x = x0 + dir * dist;
        rects.push_rect(
            Rect::from_xywh(start_x, y - thickness * 0.5, seg, thickness),
            color,
            0.0,
        );
        dist += step;
    }
}

/// 沿垂直方向绘制一段划线-空隙虚线(长度可为负,表示从下向上)。
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
) {
    let step = dash + gap;
    if step <= 0.0 || thickness <= 0.0 {
        return;
    }
    let abs_len = len.abs();
    let dir = len.signum();
    let mut dist = 0.0f32;
    while dist < abs_len {
        let seg = dash.min(abs_len - dist);
        let start_y = y0 + dir * dist;
        rects.push_rect(
            Rect::from_xywh(x - thickness * 0.5, start_y, thickness, seg),
            color,
            0.0,
        );
        dist += step;
    }
}

/// 一帧的绘制目标与参数。
pub struct DrawTarget<'a> {
    /// 帧纹理视图。
    pub view: &'a wgpu::TextureView,
    /// 目标宽度(像素)。
    pub width: f32,
    /// 目标高度(像素)。
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
    /// 实例缓冲当前容量(实例个数)。
    capacity: usize,
}

impl RectPipeline {
    const INITIAL_CAPACITY: usize = 256;

    /// 创建管线,target 为 surface 颜色格式。
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
                3 => Float32,   // radius
                4 => Float32,   // _pad (对齐占位,shader 中忽略)
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

    /// 确保实例缓冲容量足够(不足则扩容到下一个 2 的幂)。
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

    /// 开始 render pass(以 clear_color 清屏)并绘制收集到的全部矩形。
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &DrawTarget,
        batch: &RectBatch,
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

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("rect pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(target.clear_color.r),
                        g: f64::from(target.clear_color.g),
                        b: f64::from(target.clear_color.b),
                        a: f64::from(target.clear_color.a),
                    }),
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
