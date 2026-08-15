//! @author 十四叔
//! @date 2026/08/14

//! 图像纹理渲染管线：将 RGBA 像素数据上传为 GPU 纹理并绘制 quad。
//!
//! 每帧用法：先经 [`ImageBatch`] 收集图像实例，再由管线一次绘制。
//! 纹理按需创建并缓存，避免重复上传。

use crate::Rect;

use super::DrawTarget;

/// 无裁剪时使用的极大安全矩形 (像素坐标)。
const NO_CLIP_MIN: [f32; 2] = [-1_000_000.0; 2];
const NO_CLIP_MAX: [f32; 2] = [1_000_000.0; 2];

/// 单个图像实例的 GPU 数据布局。
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ImageInstance {
    /// 目标左上角像素坐标。
    dst_pos: [f32; 2],
    /// 目标像素尺寸。
    dst_size: [f32; 2],
    /// 纹理 UV 左上角 (0..1)。
    uv_min: [f32; 2],
    /// 纹理 UV 右下角 (0..1)。
    uv_max: [f32; 2],
    /// 裁剪矩形左上角。
    clip_min: [f32; 2],
    /// 裁剪矩形右下角 (不含)。
    clip_max: [f32; 2],
}

/// 图像实例数据 (包含纹理键)。
#[derive(Debug)]
struct ImageEntry {
    instance: ImageInstance,
    texture_key: TextureKey,
}

/// 纹理缓存键：按图像尺寸和数据哈希标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextureKey {
    pub width: u32,
    pub height: u32,
    /// 数据哈希，用于区分相同尺寸的不同图片。
    pub data_hash: u64,
}

/// 图像收集器：一帧内待绘制的图像列表。
///
/// 组件树 paint 阶段向其中 push 图像实例。
#[derive(Debug, Default)]
pub struct ImageBatch {
    entries: Vec<ImageEntry>,
    /// 裁剪矩形栈;`None` 表示当前裁剪区为空 (完全裁剪)。
    clip_stack: Vec<Option<Rect>>,
    /// 待上传的纹理数据 (key, RGBA 数据)。
    pending_uploads: Vec<(TextureKey, Vec<u8>)>,
}

impl ImageBatch {
    /// 新建空收集器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 压入一个裁剪矩形。
    pub fn push_clip(&mut self, rect: Rect) {
        let next = match self.current_clip() {
            Some(parent) => parent.intersect(&rect),
            None => Some(rect),
        };
        self.clip_stack.push(next);
    }

    /// 弹出当前裁剪矩形。
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    fn current_clip(&self) -> Option<Rect> {
        self.clip_stack.iter().rev().find_map(|r| *r)
    }

    /// 添加一个图像实例。
    ///
    /// `data` 为 RGBA8 像素数据，`width`/`height` 为图像尺寸。
    /// `dst_rect` 为绘制目标区域。
    pub fn push_image(&mut self, data: &[u8], width: u32, height: u32, dst_rect: Rect) {
        let (clip_min, clip_max) = match self.current_clip() {
            Some(clip) => match clip.intersect(&dst_rect) {
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

        // 计算数据哈希，用于区分相同尺寸的不同图片
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hasher);
        let data_hash = hasher.finish();

        let key = TextureKey {
            width,
            height,
            data_hash,
        };
        self.entries.push(ImageEntry {
            instance: ImageInstance {
                dst_pos: [dst_rect.origin.x, dst_rect.origin.y],
                dst_size: [dst_rect.size.width, dst_rect.size.height],
                uv_min: [0.0, 0.0],
                uv_max: [1.0, 1.0],
                clip_min,
                clip_max,
            },
            texture_key: key.clone(),
        });

        // 记录待上传纹理 (全量上传，后续可优化为脏区域)
        if !self.pending_uploads.iter().any(|(k, _)| *k == key) {
            self.pending_uploads.push((key, data.to_vec()));
        }
    }

    /// 实例数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取待上传的纹理列表。
    pub fn pending_uploads(&self) -> &[(TextureKey, Vec<u8>)] {
        &self.pending_uploads
    }

    /// 清空待上传队列 (上传完成后调用)。
    pub fn clear_pending_uploads(&mut self) {
        self.pending_uploads.clear();
    }

    /// 测试用：读取所有实例的目标矩形。
    #[doc(hidden)]
    pub fn instance_rects(&self) -> Vec<Rect> {
        self.entries
            .iter()
            .map(|e| {
                let i = &e.instance;
                Rect::from_xywh(i.dst_pos[0], i.dst_pos[1], i.dst_size[0], i.dst_size[1])
            })
            .collect()
    }
}

/// 图像纹理缓存项。
struct CachedTexture {
    #[allow(dead_code)]
    texture: wgpu::Texture,
    #[allow(dead_code)]
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
}

/// 纹理缓存最大数量。
const MAX_TEXTURE_CACHE: usize = 32;

/// 图像渲染管线。持有 GPU 管线、uniform 与纹理缓存。
pub struct ImagePipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    instance_buf: wgpu::Buffer,
    capacity: usize,
    /// 纹理缓存：按尺寸键存储已创建的纹理。
    cache: std::collections::HashMap<TextureKey, CachedTexture>,
    /// 缓存访问顺序（用于 LRU 淘汰）。
    cache_order: Vec<TextureKey>,
}

impl ImagePipeline {
    const INITIAL_CAPACITY: usize = 64;

    /// 创建管线，target 为 surface 颜色格式。
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("image.wgsl").into()),
        });

        // Group 0: uniforms
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("image uniforms"),
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
            label: Some("image uniform buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("image uniform bind group"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        // Group 1: texture + sampler (per-image, 动态绑定)
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("image texture bind group layout"),
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("image sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("image pipeline layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });

        let instance_layout = wgpu::VertexBufferLayout {
            array_stride: size_of::<ImageInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // dst_pos
                1 => Float32x2, // dst_size
                2 => Float32x2, // uv_min
                3 => Float32x2, // uv_max
                4 => Float32x2, // clip_min
                5 => Float32x2, // clip_max
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("image pipeline"),
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
            label: Some("image instance buffer"),
            size: (Self::INITIAL_CAPACITY * size_of::<ImageInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_buf,
            uniform_bind_group,
            sampler,
            texture_bind_group_layout,
            instance_buf,
            capacity: Self::INITIAL_CAPACITY,
            cache: std::collections::HashMap::new(),
            cache_order: Vec::new(),
        }
    }

    /// 上传屏幕尺寸 uniform。
    fn write_screen_uniform(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        let data = [width, height, 0.0, 0.0];
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&data));
    }

    /// 确保实例缓冲容量足够。
    fn ensure_capacity(&mut self, device: &wgpu::Device, needed: usize) {
        if needed <= self.capacity {
            return;
        }
        let new_capacity = needed.next_power_of_two();
        self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("image instance buffer"),
            size: (new_capacity * size_of::<ImageInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.capacity = new_capacity;
    }

    /// 获取或创建纹理缓存，可选上传像素数据。
    fn get_or_create_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: TextureKey,
        data: Option<&[u8]>,
    ) -> &CachedTexture {
        if !self.cache.contains_key(&key) {
            // 缓存满时淘汰最久未使用的
            if self.cache.len() >= MAX_TEXTURE_CACHE {
                if let Some(oldest) = self.cache_order.first().cloned() {
                    self.cache.remove(&oldest);
                    self.cache_order.remove(0);
                }
            }

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("image texture"),
                size: wgpu::Extent3d {
                    width: key.width,
                    height: key.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            // 上传像素数据
            if let Some(data) = data {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    data,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * key.width),
                        rows_per_image: Some(key.height),
                    },
                    wgpu::Extent3d {
                        width: key.width,
                        height: key.height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("image texture bind group"),
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            self.cache.insert(
                key.clone(),
                CachedTexture {
                    texture,
                    view,
                    bind_group,
                },
            );
            self.cache_order.push(key.clone());
        } else {
            // 更新访问顺序（移到最后）
            if let Some(pos) = self.cache_order.iter().position(|k| k == &key) {
                self.cache_order.remove(pos);
                self.cache_order.push(key.clone());
            }
        }
        &self.cache[&key]
    }

    /// 绘制收集到的全部图像。
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &DrawTarget,
        batch: &mut ImageBatch,
    ) {
        if batch.is_empty() {
            return;
        }

        // 上传待处理的纹理
        for (key, data) in batch.pending_uploads() {
            self.get_or_create_texture(device, queue, key.clone(), Some(data));
        }
        batch.clear_pending_uploads();

        self.write_screen_uniform(queue, target.width, target.height);
        self.ensure_capacity(device, batch.len());

        // 按纹理分批绘制
        for entry in &batch.entries {
            // 确保纹理已缓存
            if let Some(cached) = self.cache.get(&entry.texture_key) {
                queue.write_buffer(
                    &self.instance_buf,
                    0,
                    bytemuck::cast_slice(&[entry.instance]),
                );

                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("image pass"),
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
                pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                pass.set_bind_group(1, &cached.bind_group, &[]);
                pass.set_vertex_buffer(0, self.instance_buf.slice(..));
                pass.draw(0..6, 0..1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_batch_push_and_len() {
        let mut batch = ImageBatch::new();
        assert!(batch.is_empty());

        let data = vec![255u8; 4 * 10 * 10]; // 10x10 RGBA
        batch.push_image(&data, 10, 10, Rect::from_xywh(0.0, 0.0, 100.0, 100.0));
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn image_batch_clip_skips_fully_clipped() {
        let mut batch = ImageBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 10.0, 10.0));

        let data = vec![255u8; 4 * 10 * 10];
        batch.push_image(&data, 10, 10, Rect::from_xywh(20.0, 20.0, 10.0, 10.0));
        assert!(batch.is_empty());
    }

    #[test]
    fn image_batch_clip_keeps_partially_visible() {
        let mut batch = ImageBatch::new();
        batch.push_clip(Rect::from_xywh(0.0, 0.0, 15.0, 10.0));

        let data = vec![255u8; 4 * 10 * 10];
        batch.push_image(&data, 10, 10, Rect::from_xywh(10.0, 0.0, 10.0, 10.0));
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn image_batch_pending_uploads() {
        let mut batch = ImageBatch::new();
        let data = vec![255u8; 4 * 10 * 10];
        batch.push_image(&data, 10, 10, Rect::from_xywh(0.0, 0.0, 100.0, 100.0));

        assert_eq!(batch.pending_uploads().len(), 1);
        assert_eq!(batch.pending_uploads()[0].0.width, 10);
        assert_eq!(batch.pending_uploads()[0].0.height, 10);

        batch.clear_pending_uploads();
        assert!(batch.pending_uploads().is_empty());
    }
}
