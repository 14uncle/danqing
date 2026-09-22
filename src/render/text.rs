//! @author 十四叔
//! @date 2026/07/17

//! 文本渲染管线：图集纹理 + 实例化字形 quad。
//!
//! [`TextBatch`] 是 CPU 侧：持有字体与图集，负责按字排版并收集实例;
//! [`TextPipeline`] 是 GPU 侧：负责把图集脏区域上传纹理并绘制实例。

use std::ops::Range;

use crate::Color;
use crate::render::DrawTarget;
use crate::render::LinearRgba;
use crate::text::{Font, GlyphAtlas};

/// 无裁剪时使用的极大安全矩形 (像素坐标)。
const NO_CLIP_MIN: [f32; 2] = [-1_000_000.0; 2];
const NO_CLIP_MAX: [f32; 2] = [1_000_000.0; 2];

/// 单个字形实例的 GPU 数据布局。
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlyphInstance {
    /// 目标左上角像素坐标。
    dst_pos: [f32; 2],
    /// 目标像素尺寸。
    dst_size: [f32; 2],
    /// 图集 uv 左上角 (0..1)。
    uv_min: [f32; 2],
    /// 图集 uv 右下角 (0..1)。
    uv_max: [f32; 2],
    /// RGBA 颜色 (线性空间 —— 由 [`LinearRgba`] 保证, 见 `render/linear.rs`)。
    color: LinearRgba,
    /// 裁剪矩形左上角。
    clip_min: [f32; 2],
    /// 裁剪矩形右下角 (不含)。
    clip_max: [f32; 2],
}

/// 文本收集器：字体 + 图集 + 一帧内的字形实例。
///
/// 持久存在 (字体与图集跨帧复用); 每帧 [`Self::clear`] 清空实例列表。
pub struct TextBatch {
    font: Font,
    atlas: GlyphAtlas,
    instances: Vec<GlyphInstance>,
    /// 裁剪矩形栈;`None` 表示当前裁剪区为空 (完全裁剪)。
    clip_stack: Vec<Option<crate::Rect>>,
    /// 层边界: 与 RectBatch::push_layer 配套; 渲染按层分段交替,
    /// 本层的矩形先画 (可盖低层文本), 本层文本后画。
    layer_marks: Vec<usize>,
    /// DPI 缩放因子 (逻辑 → 物理像素; 1.0 = 100% 缩放)。
    ///
    /// 框架内部坐标统一为逻辑像素 (spec: docs/specs/hidpi-scale-factor.md):
    /// 文字按 `round(px×scale)` 物理字号栅格化以保住物理像素格锐度
    /// (memory/font-cjk-mono 的清晰字方案), 测量/行高返回逻辑值 (物理指标 ÷scale)。
    /// 实例 (落点/尺寸/裁剪) 全部是物理域, 与 text.wgsl 的 screen_size 同域。
    scale: f32,
}

impl TextBatch {
    /// 新建：按策略加载字体 (内嵌黑体优先，系统兜底), 建默认图集。
    pub fn new() -> Self {
        Self {
            font: Font::load(),
            atlas: GlyphAtlas::new(),
            instances: Vec::new(),
            clip_stack: Vec::new(),
            layer_marks: Vec::new(),
            scale: 1.0,
        }
    }

    /// 设置 DPI 缩放因子 (窗口创建时取 `Window::scale_factor`, `ScaleFactorChanged` 时更新)。
    ///
    /// 变化时清空字形缓存: 缓存键是物理像素字号, scale 一变全部失效,
    /// 留着只白占图集空间 (高分屏图集压力本就 ×s², 见 spec R1)。
    /// 非正数/非有限值拒绝: 平台层不该给出, 给了也不能炸排版。
    pub fn set_scale_factor(&mut self, scale: f32) {
        if !scale.is_finite() || scale <= 0.0 {
            log::warn!("拒绝非法缩放因子: {scale}");
            return;
        }
        if (self.scale - scale).abs() <= f32::EPSILON {
            return;
        }
        self.scale = scale;
        self.atlas = GlyphAtlas::new();
    }

    /// 当前缩放因子。
    pub fn scale_factor(&self) -> f32 {
        self.scale
    }

    /// 测试观测面: 图集已缓存字形数 (scale 变更清缓存的断言点;
    /// 也可供 R1 图集容量诊断用)。
    #[doc(hidden)]
    pub fn atlas_glyph_count(&self) -> usize {
        self.atlas.glyph_count()
    }

    /// 逻辑字号 → 物理像素字号 (栅格化与行指标共用同一值, 保证内部自洽)。
    fn phys_px(&self, px: f32) -> f32 {
        (px * self.scale).round().max(1.0)
    }

    /// 压入一个裁剪矩形 (**逻辑像素**, 与组件坐标同域)。
    ///
    /// 内部 ×scale 转物理域存储 (字形实例是物理域, text.wgsl 同域剔除)。
    /// 后续 push 的字形会被裁剪到该矩形与所有祖先裁剪矩形的交集。
    /// 必须在子组件 paint 前调用，并在 paint 后调用 [`Self::pop_clip`]。
    pub fn push_clip(&mut self, rect: crate::Rect) {
        let rect = crate::Rect::from_xywh(
            rect.origin.x * self.scale,
            rect.origin.y * self.scale,
            rect.size.width * self.scale,
            rect.size.height * self.scale,
        );
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

    /// 测试观测面: 每个字形实例的 (落点, 裁剪下界, 裁剪上界)。
    ///
    /// `#[doc(hidden)] pub` 而非 `cfg(test)`: 产品侧回归锁也要用
    /// (如 danqing-log 断言「侧栏直方图真的画出来了」—— 那是宽度折叠
    /// 判定失效时会静默消失的特征)。与 `TextInput::text_color` 同一处置。
    #[doc(hidden)]
    pub fn glyph_clips(&self) -> impl Iterator<Item = ([f32; 2], [f32; 2], [f32; 2])> + '_ {
        self.instances
            .iter()
            .map(|g| (g.dst_pos, g.clip_min, g.clip_max))
    }

    fn current_clip(&self) -> Option<crate::Rect> {
        self.clip_stack.iter().rev().find_map(|r| *r)
    }

    /// 字体来源描述 (诊断用)。
    pub fn font_source(&self) -> &str {
        self.font.source()
    }

    /// 建议行高 (**逻辑像素**; `px` 为逻辑字号, 预期整数值 —— 主题字号档皆整数;
    /// 分数值在 s=1.0 下会经 phys_px 取整, 与引入前行为不等价)。
    pub fn line_height(&self, px: f32) -> f32 {
        self.font.line_height(self.phys_px(px)) / self.scale
    }

    /// 指定逻辑字号下的 ascent(基线到行顶的距离, **逻辑像素**; `px` 预期整数值, 同 [`Self::line_height`])。
    pub fn ascent(&self, px: f32) -> f32 {
        let phys = self.phys_px(px);
        self.font
            .inner()
            .horizontal_line_metrics(phys)
            .map(|m| m.ascent)
            .unwrap_or(phys * 0.8)
            / self.scale
    }

    /// 指定逻辑字号下的 descent(基线到行底的距离, **逻辑像素**; `px` 预期整数值, 同 [`Self::line_height`])。
    pub fn descent(&self, px: f32) -> f32 {
        let phys = self.phys_px(px);
        self.font
            .inner()
            .horizontal_line_metrics(phys)
            .map(|m| m.descent.abs())
            .unwrap_or(phys * 0.2)
            / self.scale
    }

    /// 测量单行文本宽度 (**逻辑像素**; 逐字前进宽度之和 ÷scale; 顺带预热图集缓存)。
    pub fn measure(&mut self, text: &str, px: u16) -> f32 {
        let phys = self.phys_px(f32::from(px)) as u16;
        let mut width = 0.0;
        for ch in text.chars() {
            match self.atlas.get_or_rasterize(self.font.inner(), ch, phys) {
                Ok(info) => width += info.advance,
                Err(err) => log::warn!("测量时栅格化失败，按 0 宽计：{err}"),
            }
        }
        width / self.scale
    }

    /// 按字排版一段单行文本：从 (x, baseline) 起逐字放置 (**逻辑像素**坐标)。
    ///
    /// 内部 ×scale 转物理域: 按 `round(px×scale)` 物理字号栅格化、
    /// 落点吸附物理像素格。排版失败的字形 (如图集已满) 记录日志并跳过，不中断整行。
    pub fn push_text(&mut self, text: &str, x: f32, baseline: f32, px: u16, color: Color) {
        let phys = self.phys_px(f32::from(px)) as u16;
        let mut pen_x = x * self.scale;
        let baseline = baseline * self.scale;
        let atlas_size = self.atlas.size() as f32;
        for ch in text.chars() {
            let info = match self.atlas.get_or_rasterize(self.font.inner(), ch, phys) {
                Ok(info) => info,
                Err(err) => {
                    // err 必须带上: 图集已满 (AtlasError::Full) 与 fontdue 栅格化失败
                    // 在日志里靠它区分, 与 measure 路径对称 (R1 预警线, 2026-09-22 评审)。
                    log::warn!("字形栅格化失败，跳过：{ch:?} ({phys}px 物理)：{err}");
                    continue;
                }
            };
            if info.width > 0 {
                // 字形落点吸附整数像素: dst 矩形与物理像素格对齐后, 线性采样
                // 退化为逐纹素取值 —— 分数位置会让每个字形向邻像素渗色,
                // 小字号正文整片发灰发虚 (与竞品 ClearType 观感的差距主因)。
                // pen_x 仍按真实 advance 累加, 只吸附落点, 行间/词间距离不变。
                let gx = (pen_x + info.bearing_x as f32).round();
                let gy = (baseline - info.bearing_y as f32).round();
                let glyph_rect =
                    crate::Rect::from_xywh(gx, gy, info.width as f32, info.height as f32);
                let (clip_min, clip_max) = match self.current_clip() {
                    Some(clip) => match clip.intersect(&glyph_rect) {
                        Some(intersection) => (
                            [intersection.origin.x, intersection.origin.y],
                            [
                                intersection.origin.x + intersection.size.width,
                                intersection.origin.y + intersection.size.height,
                            ],
                        ),
                        None => {
                            pen_x += info.advance;
                            continue;
                        }
                    },
                    None => (NO_CLIP_MIN, NO_CLIP_MAX),
                };
                self.instances.push(GlyphInstance {
                    dst_pos: [gx, gy],
                    dst_size: [info.width as f32, info.height as f32],
                    uv_min: [
                        info.uv_min.0 as f32 / atlas_size,
                        info.uv_min.1 as f32 / atlas_size,
                    ],
                    uv_max: [
                        info.uv_max.0 as f32 / atlas_size,
                        info.uv_max.1 as f32 / atlas_size,
                    ],
                    // sRGB → linear: 字形覆盖率是 alpha, 颜色分量同样会被再编码。
                    color: LinearRgba::from(color),
                    clip_min,
                    clip_max,
                });
            }
            pen_x += info.advance;
        }
    }

    /// 清空本帧实例与层标记 (字体与图集保留)。
    pub fn clear(&mut self) {
        self.instances.clear();
        self.layer_marks.clear();
    }

    /// 开新层：之后 push 的文本属于新层 (与 RectBatch::push_layer 配对调用)。
    pub fn push_layer(&mut self) {
        self.layer_marks.push(self.instances.len());
    }

    /// 逐层实例区间 (恒 ≥1 段; 未 push_layer 时为单层全量)。
    pub fn layer_spans(&self) -> Vec<Range<usize>> {
        let mut spans = Vec::with_capacity(self.layer_marks.len() + 1);
        let mut start = 0;
        for &mark in &self.layer_marks {
            spans.push(start..mark);
            start = mark;
        }
        spans.push(start..self.instances.len());
        spans
    }

    /// 实例数量。
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// 测试用：读取所有字形实例的颜色 —— **GPU 实际收到的分量, 已是线性空间**
    /// (不参与公开 API 契约)。
    ///
    /// 与 `RectBatch::instance_colors` 同一约定: 想与主题 token 比对,
    /// 必须先把 token 解码 (`LinearRgba::from`)。
    #[doc(hidden)]
    pub fn instance_colors(&self) -> Vec<LinearRgba> {
        self.instances.iter().map(|i| i.color).collect()
    }

    /// 测试用：读取所有字形实例的**目标矩形** (`dst_pos` + `dst_size`) ——
    /// 即 GPU 实际收到的落点与尺寸 (不参与公开 API 契约)。
    ///
    /// 与 `RectBatch::instance_rects` 同一约定、同一形状 —— 两边对称。
    ///
    /// **为什么需要它**: `measure` 给的是**前进宽度之和 (advance)**, 而眼睛看的是
    /// **字形实际着墨的范围 (ink)**。两者不相等 —— 字形有左右侧边距, 而且
    /// `push_text` 是按 `round(pen_x + bearing_x)` 落点、按 `info.width` 定宽的。
    /// 于是「按 advance 居中」在屏上**不等于**「看着居中」。
    /// 2026-09-15 的实例: 一个不在内嵌字体子集里的字符 (`✕` U+2715, 0×0 空字形)
    /// 照样占着 6px 的 advance, 把整串文本顶偏 —— 那时没有任何一把尺能量到它,
    /// 只能靠人眼在手写的基准里比。有了这个访问器, 「画出来居中不居中」
    /// 才第一次成为**可断言**的事。
    #[doc(hidden)]
    pub fn instance_rects(&self) -> Vec<crate::Rect> {
        self.instances
            .iter()
            .map(|i| {
                crate::Rect::from_xywh(i.dst_pos[0], i.dst_pos[1], i.dst_size[0], i.dst_size[1])
            })
            .collect()
    }
}

impl Default for TextBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// 文本渲染管线：图集纹理 + 实例缓冲 + 采样渲染。
pub struct TextPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    uniform_bind: wgpu::BindGroup,
    atlas_tex: wgpu::Texture,
    atlas_bind: wgpu::BindGroup,
    instance_buf: wgpu::Buffer,
    capacity: usize,
}

impl TextPipeline {
    const INITIAL_CAPACITY: usize = 512;

    /// 创建管线，图集纹理按 atlas_size 建 (u8 alpha → R8Unorm)。
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat, atlas_size: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("text.wgsl").into()),
        });

        // group(0): 屏幕尺寸 uniform
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text uniforms"),
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
            label: Some("text uniform buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text uniform bind group"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        // group(1): 图集纹理 + 采样器
        let atlas_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text atlas layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph atlas"),
            size: wgpu::Extent3d {
                width: atlas_size,
                height: atlas_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let atlas_view = atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("atlas sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let atlas_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text atlas bind group"),
            layout: &atlas_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text pipeline layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&atlas_layout)],
            immediate_size: 0,
        });
        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<GlyphInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // dst_pos
                1 => Float32x2, // dst_size
                2 => Float32x2, // uv_min
                3 => Float32x2, // uv_max
                4 => Float32x4, // color
                5 => Float32x2, // clip_min
                6 => Float32x2, // clip_max
            ],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text pipeline"),
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
            label: Some("glyph instance buffer"),
            size: (Self::INITIAL_CAPACITY * size_of::<GlyphInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_buf,
            uniform_bind,
            atlas_tex,
            atlas_bind,
            instance_buf,
            capacity: Self::INITIAL_CAPACITY,
        }
    }

    /// 把图集脏区域上传 GPU(增量)。
    fn sync_atlas(&mut self, queue: &wgpu::Queue, batch: &mut TextBatch) {
        let Some((min_x, min_y, max_x, max_y)) = batch.atlas.take_dirty() else {
            return;
        };
        let width = max_x - min_x;
        let height = max_y - min_y;
        // 按行拷贝脏矩形为紧凑缓冲
        let mut data = Vec::with_capacity((width * height) as usize);
        for row in min_y..max_y {
            let start = (row * batch.atlas.size() + min_x) as usize;
            data.extend_from_slice(&batch.atlas.pixels()[start..start + width as usize]);
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas_tex,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: min_x,
                    y: min_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.capacity {
            return;
        }
        let new_capacity = needed.next_power_of_two();
        self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("glyph instance buffer"),
            size: (new_capacity * size_of::<GlyphInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.capacity = new_capacity;
    }

    /// 上传图集脏区 + 整批字形实例 + 屏幕 uniform (每帧一次, 须在 [`Self::draw_span`] 之前)。
    ///
    /// 与 RectPipeline::upload 同理: wgpu 的 write_buffer 统一在 submit 的全部
    /// pass 之前执行, 分层渲染必须整批一次上传、各层只按区间绘制。
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        batch: &mut TextBatch,
        target: &DrawTarget,
    ) {
        self.sync_atlas(queue, batch);
        let data = [target.width, target.height, 0.0, 0.0];
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&data));
        self.ensure_capacity(device, batch.instances.len());
        if !batch.instances.is_empty() {
            queue.write_buffer(
                &self.instance_buf,
                0,
                bytemuck::cast_slice(&batch.instances),
            );
        }
    }

    /// 在已有内容的画面上叠加绘制批次中的一个字形区间 (LoadOp::Load, 不清屏),
    /// 须先调用 [`Self::upload`]。空区间直接跳过, 不开 pass。
    pub fn draw_span(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &DrawTarget,
        span: Range<usize>,
    ) {
        if span.is_empty() {
            return;
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("text pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind, &[]);
        pass.set_bind_group(1, &self.atlas_bind, &[]);
        pass.set_vertex_buffer(0, self.instance_buf.slice(..));
        // 实例顶点属性按 first_instance 偏移取值: 以区间端点为实例范围。
        pass.draw(0..6, span.start as u32..span.end as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

    #[test]
    fn glyph_layout_is_unchanged_by_linear_color() {
        // 与 rect 侧同一条守卫: 换了颜色类型后, 实例尺寸与字段偏移必须逐字节不变,
        // 否则顶点属性偏移错位、文字会花。
        assert_eq!(
            size_of::<LinearRgba>(),
            size_of::<[f32; 4]>(),
            "线性色必须与 [f32; 4] 同尺寸"
        );
        // dst_pos(8)+dst_size(8)+uv_min(8)+uv_max(8)+color(16)+clip_min(8)+clip_max(8)
        assert_eq!(size_of::<GlyphInstance>(), 64, "实例总布局不得变");
    }

    #[test]
    fn push_text_decodes_color_to_linear() {
        let mut batch = TextBatch::new();
        batch.push_text("A", 0.0, 20.0, 16, Color::rgb(0.5, 0.5, 0.5));
        assert_eq!(batch.len(), 1, "先确认字形实例确实产生了");
        let c = batch.instance_colors()[0];
        assert!((c.r - 0.21404).abs() < 1e-4, "r 实得 {} (未解码?)", c.r);
        assert_eq!(c.a, 1.0, "alpha 不参与色彩空间转换");
    }

    #[test]
    fn clip_stack_skips_fully_clipped_glyphs() {
        let mut batch = TextBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 10.0, 10.0));
        // 文本在 (20,0) 开始，完全在裁剪区外
        batch.push_text("A", 20.0, 20.0, 16, Color::BLACK);
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn clip_stack_keeps_visible_glyphs() {
        let mut batch = TextBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 100.0, 100.0));
        batch.push_text("A", 0.0, 20.0, 16, Color::BLACK);
        assert!(!batch.is_empty());
    }

    #[test]
    fn layer_spans_and_clear_resets() {
        let mut batch = TextBatch::new();
        batch.push_text("A", 0.0, 20.0, 16, Color::BLACK);
        let n1 = batch.len();
        batch.push_layer();
        batch.push_text("B", 0.0, 20.0, 16, Color::BLACK);
        let n2 = batch.len();
        assert_eq!(batch.layer_spans(), vec![0..n1, n1..n2], "两层分段");
        batch.clear();
        assert_eq!(batch.layer_spans(), vec![0..0], "clear 后回归单层空段");
    }

    #[test]
    fn nested_clip_intersects_for_text() {
        let mut batch = TextBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 100.0, 100.0));
        batch.push_clip(Rect::from_xywh(50.0, 0.0, 100.0, 100.0));
        batch.push_text("A", 0.0, 20.0, 16, Color::BLACK);
        assert_eq!(batch.len(), 0);
        batch.pop_clip();
        batch.pop_clip();
        batch.push_text("A", 0.0, 20.0, 16, Color::BLACK);
        assert!(!batch.is_empty());
    }

    #[test]
    fn descent_returns_loaded_font_descent() {
        // TextBatch::new() 用 Font::load()，总带真实 metrics；descent 应返回加载字体的
        // descent（px*0.2 只是无 metrics 时的防御回退，正常字体走不到）。
        // 旧版断言硬编码 0.2*px=3.2，仅因 Noto 的 descent 恰为 3.2 而通过；换 mono 后
        // 露馅(4.56)——改成与加载字体的真实 descent 比对，不再绑定字体巧合值。
        let batch = TextBatch::new();
        let d = batch.descent(16.0);
        let expected = batch
            .font
            .inner()
            .horizontal_line_metrics(16.0)
            .unwrap()
            .descent
            .abs();
        assert!(
            (d - expected).abs() < 0.01,
            "descent 应为加载字体的 descent {expected}, 实际 {d}"
        );
    }

    // ---- HiDPI scale 支持 (spec: docs/specs/hidpi-scale-factor.md) ----

    #[test]
    fn scale_factor_defaults_to_one() {
        let batch = TextBatch::new();
        assert_eq!(batch.scale_factor(), 1.0, "默认 scale 必须为 1.0 (恒等)");
    }

    #[test]
    fn set_scale_factor_rejects_invalid() {
        let mut batch = TextBatch::new();
        for bad in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            batch.set_scale_factor(bad);
        }
        assert_eq!(batch.scale_factor(), 1.0, "非法 scale 必须被拒绝");
    }

    #[test]
    fn measure_is_scale_invariant_logical_width() {
        // 核心设计不变量: measure 返回逻辑宽度, 跨 scale 一致 (物理宽度 ÷s)。
        let mut one = TextBatch::new();
        let mut two = TextBatch::new();
        two.set_scale_factor(2.0);
        let w1 = one.measure("日志 ABC", 15);
        let w2 = two.measure("日志 ABC", 15);
        assert!(
            (w1 - w2).abs() < 0.05,
            "逻辑宽度跨 scale 必须一致: s=1 得 {w1}, s=2 得 {w2}"
        );
    }

    #[test]
    fn line_metrics_are_scale_invariant() {
        let one = TextBatch::new();
        let mut two = TextBatch::new();
        two.set_scale_factor(2.0);
        for px in [12.0, 15.0, 20.0] {
            assert!(
                (one.line_height(px) - two.line_height(px)).abs() < 0.1,
                "line_height({px}) 跨 scale 不一致"
            );
            assert!(
                (one.ascent(px) - two.ascent(px)).abs() < 0.1,
                "ascent({px}) 跨 scale 不一致"
            );
        }
    }

    #[test]
    fn glyph_geometry_scales_to_physical() {
        // 同一逻辑坐标: s=2 的落点/位图约为 s=1 的两倍 (实例是物理域)。
        let mut one = TextBatch::new();
        one.push_text("A", 10.0, 40.0, 15, Color::BLACK);
        let r1 = one.instance_rects()[0];
        let mut two = TextBatch::new();
        two.set_scale_factor(2.0);
        two.push_text("A", 10.0, 40.0, 15, Color::BLACK);
        let r2 = two.instance_rects()[0];
        let ratio = r2.size.width / r1.size.width;
        assert!(
            (1.8..=2.2).contains(&ratio),
            "位图宽度比应≈2 (物理栅格化), 实得 {ratio}"
        );
        // 落点 ×2 (±2px 容差: 不同物理 px 下 bearing 取整不同)。
        assert!(
            (r2.origin.x - r1.origin.x * 2.0).abs() <= 2.0,
            "落点 x 应≈×2: {} vs {}",
            r1.origin.x,
            r2.origin.x
        );
        assert!(
            (r2.origin.y - r1.origin.y * 2.0).abs() <= 2.0,
            "落点 y 应≈×2: {} vs {}",
            r1.origin.y,
            r2.origin.y
        );
    }

    #[test]
    fn scale_change_clears_glyph_cache() {
        let mut batch = TextBatch::new();
        batch.push_text("你", 0.0, 40.0, 15, Color::BLACK);
        assert!(batch.atlas_glyph_count() > 0);
        batch.set_scale_factor(2.0);
        assert_eq!(
            batch.atlas_glyph_count(),
            0,
            "scale 变更必须清图集缓存 (缓存键是物理字号)"
        );
        // 同值重复设置不得清 (拖拽经过同 scale 屏不抖)。
        batch.push_text("你", 0.0, 40.0, 15, Color::BLACK);
        let n = batch.atlas_glyph_count();
        batch.set_scale_factor(2.0);
        assert_eq!(batch.atlas_glyph_count(), n, "同值重复设置不得清缓存");
    }

    #[test]
    fn clip_rect_applies_in_logical_space() {
        let mut batch = TextBatch::new();
        batch.set_scale_factor(2.0);
        // 逻辑裁剪宽 10 = 物理 20; clip 若未 ×s, 物理 16 会被误裁。
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 10.0, 100.0));
        batch.push_text("A", 8.0, 40.0, 15, Color::BLACK); // 物理 x=16 < 20: 保留
        assert_eq!(batch.len(), 1, "逻辑裁剪区内的字形必须保留");
        batch.push_text("B", 12.0, 40.0, 15, Color::BLACK); // 物理 x=24 > 20: 剔除
        assert_eq!(batch.len(), 1, "逻辑裁剪区外的字形必须剔除");
    }

    #[test]
    fn scale_one_is_identical_to_unscaled() {
        // s=1.0 回归锁: 显式设 1.0 与默认行为逐位一致。
        let mut plain = TextBatch::new();
        let mut explicit = TextBatch::new();
        explicit.set_scale_factor(1.0);
        for text in ["日志 viewer", "ERROR 42"] {
            plain.push_text(text, 7.0, 33.0, 15, Color::BLACK);
            explicit.push_text(text, 7.0, 33.0, 15, Color::BLACK);
        }
        assert_eq!(plain.instance_rects(), explicit.instance_rects());
        let mut m1 = TextBatch::new();
        let mut m2 = TextBatch::new();
        m2.set_scale_factor(1.0);
        assert_eq!(m1.measure("日志 viewer", 15), m2.measure("日志 viewer", 15));
    }

    #[test]
    fn fractional_scale_rounding_is_pinned() {
        // 取整策略钉死: phys_px(15)@1.5 必须 == 23 (round half away from zero,
        // 22.5→23)。s=2.0 下 round 是恒等操作锁不住这条 —— 用跨尺度等价把
        // 物理字号变成可断言量: s=1.5 逻辑 15px 与 s=1.0 直接 23px 必须逐位同形。
        let mut scaled = TextBatch::new();
        scaled.set_scale_factor(1.5);
        scaled.push_text("A", 0.0, 40.0, 15, Color::BLACK);
        let mut direct = TextBatch::new();
        direct.push_text("A", 0.0, 60.0, 23, Color::BLACK); // 40×1.5=60, 同基线
        assert_eq!(
            scaled.instance_rects()[0],
            direct.instance_rects()[0],
            "同物理字号同落点, 实例必须逐位相等"
        );
        // 行指标同一把物理尺: line_height(15)@1.5 == line_height(23)@1.0 ÷ 1.5
        assert_eq!(
            scaled.line_height(15.0),
            direct.line_height(23.0) / 1.5,
            "line_height 与栅格化必须共用同一物理字号"
        );
        // measure 同一把物理尺: measure(15)@1.5 == measure(23)@1.0 ÷ 1.5
        assert_eq!(
            scaled.measure("日志", 15),
            direct.measure("日志", 23) / 1.5,
            "measure 与栅格化必须共用同一物理字号"
        );
    }
}
