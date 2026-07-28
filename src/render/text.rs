//! @author 十四叔
//! @date 2026/07/17

//! 文本渲染管线：图集纹理 + 实例化字形 quad。
//!
//! [`TextBatch`] 是 CPU 侧：持有字体与图集，负责按字排版并收集实例;
//! [`TextPipeline`] 是 GPU 侧：负责把图集脏区域上传纹理并绘制实例。

use crate::Color;
use crate::render::DrawTarget;
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
    /// RGBA 颜色。
    color: [f32; 4],
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
}

impl TextBatch {
    /// 新建：按策略加载字体 (内嵌黑体优先，系统兜底), 建默认图集。
    pub fn new() -> Self {
        Self {
            font: Font::load(),
            atlas: GlyphAtlas::new(),
            instances: Vec::new(),
            clip_stack: Vec::new(),
        }
    }

    /// 压入一个裁剪矩形。
    ///
    /// 后续 push 的字形会被裁剪到该矩形与所有祖先裁剪矩形的交集。
    /// 必须在子组件 paint 前调用，并在 paint 后调用 [`Self::pop_clip`]。
    pub fn push_clip(&mut self, rect: crate::Rect) {
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

    fn current_clip(&self) -> Option<crate::Rect> {
        self.clip_stack.iter().rev().find_map(|r| *r)
    }

    /// 字体来源描述 (诊断用)。
    pub fn font_source(&self) -> &str {
        self.font.source()
    }

    /// 建议行高。
    pub fn line_height(&self, px: f32) -> f32 {
        self.font.line_height(px)
    }

    /// 指定字号下的 ascent(基线到行顶的距离)。
    pub fn ascent(&self, px: f32) -> f32 {
        self.font
            .inner()
            .horizontal_line_metrics(px)
            .map(|m| m.ascent)
            .unwrap_or(px * 0.8)
    }

    /// 测量单行文本宽度 (逐字前进宽度之和; 顺带预热图集缓存)。
    pub fn measure(&mut self, text: &str, px: u16) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            match self.atlas.get_or_rasterize(self.font.inner(), ch, px) {
                Ok(info) => width += info.advance,
                Err(err) => log::warn!("测量时栅格化失败，按 0 宽计：{err}"),
            }
        }
        width
    }

    /// 按字排版一段单行文本：从 (x, baseline) 起逐字放置。
    ///
    /// 排版失败的字形 (如图集满) 记录日志并跳过，不中断整行。
    pub fn push_text(&mut self, text: &str, x: f32, baseline: f32, px: u16, color: Color) {
        let mut pen_x = x;
        let atlas_size = self.atlas.size() as f32;
        for ch in text.chars() {
            let Ok(info) = self.atlas.get_or_rasterize(self.font.inner(), ch, px) else {
                log::warn!("字形栅格化失败，跳过：{ch:?} ({px}px)");
                continue;
            };
            if info.width > 0 {
                let gx = pen_x + info.bearing_x as f32;
                let gy = baseline - info.bearing_y as f32;
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
                    color: [color.r, color.g, color.b, color.a],
                    clip_min,
                    clip_max,
                });
            }
            pen_x += info.advance;
        }
    }

    /// 清空本帧实例 (字体与图集保留)。
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    /// 实例数量。
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
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

    /// 在已有内容的画面上叠加绘制文本 (LoadOp::Load, 不清屏)。
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &DrawTarget,
        batch: &mut TextBatch,
    ) {
        self.sync_atlas(queue, batch);
        if batch.is_empty() {
            return;
        }
        let data = [target.width, target.height, 0.0, 0.0];
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&data));
        self.ensure_capacity(device, batch.len());
        queue.write_buffer(
            &self.instance_buf,
            0,
            bytemuck::cast_slice(&batch.instances),
        );

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
        pass.draw(0..6, 0..batch.len() as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rect;

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
}
