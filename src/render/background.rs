//! @author 十四叔
//! @date 2026/07/19

//! 窗口背景图渲染: 将 `build.rs` 生成的渐变/噪声图绘制在组件树之下。
//!
//! 当前支持一张主背景图与一张可选噪声叠加图,并提供 Stretch/Fit/Cover
//! 三种缩放模式。图片在 `Context` 初始化时解码并上传为 wgpu 纹理,
//! 每帧按窗口尺寸重新计算顶点坐标与 UV。

use std::path::{Path, PathBuf};

use crate::render::DrawTarget;

/// 背景图缩放模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    /// 拉伸填满整个窗口(可能改变宽高比)。
    #[default]
    Stretch,
    /// 完整显示图片,留白处显示清屏色。
    Fit,
    /// 等比缩放并裁切,不留黑边。
    Cover,
}

/// 窗口背景配置。
///
/// 由 `WindowConfig` 持有;`Context` 在初始化时读取并上传纹理。
#[derive(Debug, Clone, Default)]
pub struct BackgroundConfig {
    /// 主背景图路径(通常为 `OUT_DIR/assets/background/gradient.png`)。
    pub image: Option<PathBuf>,
    /// 可选噪声叠加图路径(通常为 `OUT_DIR/assets/background/noise.png`)。
    pub noise: Option<PathBuf>,
    /// 主背景图缩放模式。
    pub scale: ScaleMode,
    /// 噪声图叠加不透明度 (0.0 ..= 1.0)。
    pub noise_opacity: f32,
}

impl BackgroundConfig {
    /// 使用指定主背景图创建配置,其余为默认值。
    pub fn with_image(path: impl Into<PathBuf>) -> Self {
        Self {
            image: Some(path.into()),
            ..Self::default()
        }
    }

    /// 叠加噪声图。
    pub fn with_noise(mut self, path: impl Into<PathBuf>, opacity: f32) -> Self {
        self.noise = Some(path.into());
        self.noise_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// 设置主背景图缩放模式。
    pub fn scale(mut self, scale: ScaleMode) -> Self {
        self.scale = scale;
        self
    }
}

/// 单个已上传的背景纹理。
#[allow(dead_code)]
struct BackgroundTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

/// 背景渲染管线。
pub struct BackgroundPipeline {
    pipeline: wgpu::RenderPipeline,
    uniform_buf: wgpu::Buffer,
    uniform_bind: wgpu::BindGroup,
    vertex_buf: wgpu::Buffer,
    background: Option<BackgroundTexture>,
    noise: Option<BackgroundTexture>,
    scale: ScaleMode,
    noise_opacity: f32,
}

/// 单个顶点:归一化位置 (0..1) + UV。
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl BackgroundPipeline {
    /// 创建管线并按配置加载纹理;加载失败时记录警告并继续(回退到清屏色)。
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        config: &BackgroundConfig,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("background shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("background.wgsl").into()),
        });

        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("background uniforms"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("background uniform buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("background uniform bind group"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("background texture layout"),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("background pipeline layout"),
            bind_group_layouts: &[Some(&uniform_layout), Some(&texture_layout)],
            ..Default::default()
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("background pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2,
                        1 => Float32x2,
                    ],
                })],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("background vertex buffer"),
            size: (6 * size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let background = config
            .image
            .as_deref()
            .and_then(|p| load_texture(device, queue, &texture_layout, p, "background"));
        let noise = config
            .noise
            .as_deref()
            .and_then(|p| load_texture(device, queue, &texture_layout, p, "noise"));

        Self {
            pipeline,
            uniform_buf,
            uniform_bind,
            vertex_buf,
            background,
            noise,
            scale: config.scale,
            noise_opacity: config.noise_opacity.clamp(0.0, 1.0),
        }
    }

    /// 是否存在可绘制的背景。
    pub fn has_background(&self) -> bool {
        self.background.is_some()
    }

    /// 绘制背景层(在 RectBatch 之前调用)。
    pub fn draw(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &DrawTarget,
    ) {
        let Some(bg) = &self.background else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("background pass"),
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

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));

        // 主背景图
        self.upload_quad(queue, target, bg.width, bg.height, self.scale, 1.0);
        pass.set_bind_group(1, &bg.bind_group, &[]);
        pass.draw(0..6, 0..1);

        // 噪声叠加图
        if let Some(noise) = &self.noise {
            self.upload_quad(
                queue,
                target,
                noise.width,
                noise.height,
                ScaleMode::Stretch,
                self.noise_opacity,
            );
            pass.set_bind_group(1, &noise.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }
    }

    /// 按缩放模式计算顶点与 UV,上传到 vertex buffer。
    fn upload_quad(
        &self,
        queue: &wgpu::Queue,
        target: &DrawTarget,
        img_w: u32,
        img_h: u32,
        scale: ScaleMode,
        opacity: f32,
    ) {
        let screen_w = target.width;
        let screen_h = target.height;
        let img_wf = img_w as f32;
        let img_hf = img_h as f32;

        let (x0, y0, x1, y1, u0, v0, u1, v1) = match scale {
            ScaleMode::Stretch => (
                0.0f32, 0.0f32, screen_w, screen_h, 0.0f32, 0.0f32, 1.0f32, 1.0f32,
            ),
            ScaleMode::Fit => {
                let scale = (screen_w / img_wf).min(screen_h / img_hf);
                let dw = img_wf * scale;
                let dh = img_hf * scale;
                let ox = (screen_w - dw) * 0.5;
                let oy = (screen_h - dh) * 0.5;
                (ox, oy, ox + dw, oy + dh, 0.0, 0.0, 1.0, 1.0)
            }
            ScaleMode::Cover => {
                let scale = (screen_w / img_wf).max(screen_h / img_hf);
                let dw = img_wf * scale;
                let dh = img_hf * scale;
                let ox = (screen_w - dw) * 0.5;
                let oy = (screen_h - dh) * 0.5;
                let u0 = (-ox / dw).clamp(0.0, 1.0);
                let v0 = (-oy / dh).clamp(0.0, 1.0);
                let u1 = ((screen_w - ox) / dw).clamp(0.0, 1.0);
                let v1 = ((screen_h - oy) / dh).clamp(0.0, 1.0);
                (0.0, 0.0, screen_w, screen_h, u0, v0, u1, v1)
            }
        };

        let verts = [
            Vertex {
                pos: [x0 / screen_w, y0 / screen_h],
                uv: [u0, v0],
            },
            Vertex {
                pos: [x1 / screen_w, y0 / screen_h],
                uv: [u1, v0],
            },
            Vertex {
                pos: [x1 / screen_w, y1 / screen_h],
                uv: [u1, v1],
            },
            Vertex {
                pos: [x0 / screen_w, y0 / screen_h],
                uv: [u0, v0],
            },
            Vertex {
                pos: [x1 / screen_w, y1 / screen_h],
                uv: [u1, v1],
            },
            Vertex {
                pos: [x0 / screen_w, y1 / screen_h],
                uv: [u0, v1],
            },
        ];
        queue.write_buffer(&self.vertex_buf, 0, bytemuck::cast_slice(&verts));
        queue.write_buffer(
            &self.uniform_buf,
            0,
            bytemuck::cast_slice(&[opacity, 0.0, 0.0, 0.0f32]),
        );
    }
}

/// 加载 PNG 并创建纹理与 bind group;失败时返回 None 并记录警告。
fn load_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    path: &Path,
    label: &str,
) -> Option<BackgroundTexture> {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(err) => {
            log::warn!("背景图加载失败 {}: {err}", path.display());
            return None;
        }
    };
    let img = match image::load_from_memory(&data) {
        Ok(img) => img.into_rgba8(),
        Err(err) => {
            log::warn!("背景图解码失败 {}: {err}", path.display());
            return None;
        }
    };
    let (width, height) = img.dimensions();
    let size = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("{label} texture")),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(&format!("{label} sampler")),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!("{label} bind group")),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        size,
    );

    Some(BackgroundTexture {
        texture,
        view,
        bind_group,
        width,
        height,
    })
}
