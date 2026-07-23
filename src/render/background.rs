//! @author 十四叔
//! @date 2026/07/19

//! 窗口背景图渲染: 将 `assets/background/` 下的渐变 / 光晕 / 噪声图
//! 绘制在组件树之下。
//!
//! 当前支持一张主背景图与可选的光晕、噪声叠加图, 并提供 Stretch/Fit/Cover
//! 三种缩放模式。图片在 `Context` 初始化时解码并上传为 wgpu 纹理,
//! 每帧按窗口尺寸重新计算顶点坐标与 UV。

use std::path::{Path, PathBuf};

use crate::render::DrawTarget;

/// 背景图缩放模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    /// 拉伸填满整个窗口 (可能改变宽高比)。
    #[default]
    Stretch,
    /// 完整显示图片, 留白处显示清屏色。
    Fit,
    /// 等比缩放并裁切, 不留黑边。
    Cover,
}

/// 窗口背景配置。
///
/// 由 `WindowConfig` 持有;`Context` 在初始化时读取并上传纹理。
/// 多场景模式经 [`BackgroundConfig::with_scenes`] 配置,
/// 未配置场景时回退到 `image` 单图路径 (行为与阶段 1 一致)。
#[derive(Debug, Clone, Default)]
pub struct BackgroundConfig {
    /// 主背景图路径 (通常为 `assets/background/gradient.png`)。
    pub image: Option<PathBuf>,
    /// 场景图路径列表 (多场景模式; 非空时优先于 `image`)。
    pub scenes: Vec<PathBuf>,
    /// 可选光晕叠加图路径 (通常为 `assets/background/glow.png`)。
    pub glow: Option<PathBuf>,
    /// 可选噪声叠加图路径 (通常为 `assets/background/noise.png`)。
    pub noise: Option<PathBuf>,
    /// 主背景图缩放模式。
    pub scale: ScaleMode,
    /// 光晕图叠加不透明度 (0.0 ..= 1.0)。
    pub glow_opacity: f32,
    /// 噪声图叠加不透明度 (0.0 ..= 1.0)。
    pub noise_opacity: f32,
}

/// 每帧背景状态: 由 `App::background_frame` 产出, 驱动场景选择与交叉淡化。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundFrame {
    /// 淡化起点场景索引 (fade=0 时显示)。
    pub from: usize,
    /// 淡化终点场景索引 (fade=1 时显示)。
    pub to: usize,
    /// 淡化进度 (0.0 ..= 1.0, 构造时 clamp)。
    pub fade: f32,
    /// 本帧清屏色 (随场景基调流动)。
    pub clear_color: crate::Color,
}

impl BackgroundFrame {
    /// 构造每帧背景状态;`fade` 夹到 0..1。
    pub fn new(from: usize, to: usize, fade: f32, clear_color: crate::Color) -> Self {
        Self {
            from,
            to,
            fade: fade.clamp(0.0, 1.0),
            clear_color,
        }
    }
}

/// 将每帧背景状态解析为合法的场景索引对 (纯逻辑, 便于测试)。
///
/// 索引越界时夹到最后一个场景;场景数为 0 时返回 None (无背景可画)。
fn resolve_frame(frame: BackgroundFrame, scene_count: usize) -> Option<(usize, usize, f32)> {
    if scene_count == 0 {
        return None;
    }
    let last = scene_count - 1;
    Some((frame.from.min(last), frame.to.min(last), frame.fade))
}

impl BackgroundConfig {
    /// 使用指定主背景图创建配置, 其余为默认值。
    pub fn with_image(path: impl Into<PathBuf>) -> Self {
        Self {
            image: Some(path.into()),
            ..Self::default()
        }
    }

    /// 使用场景图列表创建配置 (多场景模式, 覆盖 `image`)。
    pub fn with_scenes(paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        Self {
            scenes: paths.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// 叠加光晕图。
    pub fn with_glow(mut self, path: impl Into<PathBuf>, opacity: f32) -> Self {
        self.glow = Some(path.into());
        self.glow_opacity = opacity.clamp(0.0, 1.0);
        self
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

/// 叠加层数量 (主背景 / 光晕 / 噪声)。
const LAYER_COUNT: usize = 3;
/// 每层 quad 的顶点数。
const VERTS_PER_LAYER: usize = 6;

/// 背景渲染管线。
///
/// 每层使用独立的 uniform buffer 与顶点区段: `Queue::write_buffer`
/// 在单次 submit 前统一生效, 同一帧内多次写同一块 buffer 时
/// 只有最后一次写入可见, 因此跨 draw 复用会导致所有层参数相同。
///
/// 场景层 (层 0) 绑定 from/to 两张场景图按 fade 交叉淡化;
/// 单图与叠加层把同一张图绑到两个纹理槽, fade 恒 0。
pub struct BackgroundPipeline {
    pipeline: wgpu::RenderPipeline,
    /// 每层一个 uniform buffer (不透明度 + 淡化进度)。
    uniform_bufs: [wgpu::Buffer; LAYER_COUNT],
    uniform_binds: [wgpu::BindGroup; LAYER_COUNT],
    /// 三层 quad 共用的顶点缓冲 (每层 [`VERTS_PER_LAYER`] 个顶点)。
    vertex_buf: wgpu::Buffer,
    /// 场景纹理列表 (单图路径视为只有一个场景的列表)。
    scenes: Vec<BackgroundTexture>,
    glow: Option<BackgroundTexture>,
    noise: Option<BackgroundTexture>,
    scale: ScaleMode,
    glow_opacity: f32,
    noise_opacity: f32,
    /// 应用层每帧写入的背景状态 (场景选择 / 淡化 / 清屏色)。
    frame: Option<BackgroundFrame>,
}

/// 单个顶点: 归一化位置 (0..1) + UV。
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl BackgroundPipeline {
    /// 创建管线并按配置加载纹理; 加载失败时记录警告并继续 (回退到清屏色)。
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
        let uniform_bufs: [wgpu::Buffer; LAYER_COUNT] = std::array::from_fn(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("background uniform buffer"),
                size: 16,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        });
        let uniform_binds: [wgpu::BindGroup; LAYER_COUNT] = std::array::from_fn(|i| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("background uniform bind group"),
                layout: &uniform_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_bufs[i].as_entire_binding(),
                }],
            })
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
            bind_group_layouts: &[
                Some(&uniform_layout),
                Some(&texture_layout),
                Some(&texture_layout),
            ],
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
            size: (LAYER_COUNT * VERTS_PER_LAYER * size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // 场景路径列表: 多场景配置优先, 否则回退到单图路径。
        let scene_paths: Vec<&Path> = if !config.scenes.is_empty() {
            config.scenes.iter().map(PathBuf::as_path).collect()
        } else {
            config.image.as_deref().into_iter().collect()
        };
        let scenes: Vec<BackgroundTexture> = scene_paths
            .into_iter()
            .filter_map(|p| load_texture(device, queue, &texture_layout, p, "scene"))
            .collect();
        let glow = config
            .glow
            .as_deref()
            .and_then(|p| load_texture(device, queue, &texture_layout, p, "glow"));
        let noise = config
            .noise
            .as_deref()
            .and_then(|p| load_texture(device, queue, &texture_layout, p, "noise"));

        Self {
            pipeline,
            uniform_bufs,
            uniform_binds,
            vertex_buf,
            scenes,
            glow,
            noise,
            scale: config.scale,
            glow_opacity: config.glow_opacity.clamp(0.0, 1.0),
            noise_opacity: config.noise_opacity.clamp(0.0, 1.0),
            frame: None,
        }
    }

    /// 是否存在可绘制的背景。
    pub fn has_background(&self) -> bool {
        !self.scenes.is_empty()
    }

    /// 写入应用层产出的每帧背景状态 (场景选择 / 淡化 / 清屏色)。
    pub fn set_frame(&mut self, frame: BackgroundFrame) {
        self.frame = Some(frame);
    }

    /// 绘制背景层 (在 RectBatch 之前调用)。
    pub fn draw(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &DrawTarget,
    ) {
        let frame = self.frame.unwrap_or(BackgroundFrame {
            from: 0,
            to: 0,
            fade: 0.0,
            clear_color: target.clear_color,
        });
        let Some((from, to, fade)) = resolve_frame(frame, self.scenes.len()) else {
            return;
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("background pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: f64::from(frame.clear_color.r),
                        g: f64::from(frame.clear_color.g),
                        b: f64::from(frame.clear_color.b),
                        a: f64::from(frame.clear_color.a),
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
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));

        // 层 0 场景 (from/to 交叉淡化) / 层 1 光晕 / 层 2 噪声
        self.draw_layer(
            &mut pass,
            queue,
            target,
            0,
            &self.scenes[from],
            &self.scenes[to],
            self.scale,
            1.0,
            fade,
        );
        if let Some(glow) = &self.glow {
            self.draw_layer(
                &mut pass,
                queue,
                target,
                1,
                glow,
                glow,
                ScaleMode::Cover,
                self.glow_opacity,
                0.0,
            );
        }
        if let Some(noise) = &self.noise {
            self.draw_layer(
                &mut pass,
                queue,
                target,
                2,
                noise,
                noise,
                ScaleMode::Stretch,
                self.noise_opacity,
                0.0,
            );
        }
    }

    /// 绘制单个叠加层: 上传该层顶点与 uniform, 绑定资源后绘制。
    #[allow(clippy::too_many_arguments)]
    fn draw_layer(
        &self,
        pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        target: &DrawTarget,
        layer: usize,
        tex_from: &BackgroundTexture,
        tex_to: &BackgroundTexture,
        scale: ScaleMode,
        opacity: f32,
        fade: f32,
    ) {
        // 淡化要求 from/to 同尺寸 (场景生成管线保证统一画布);
        // UV 按 from 纹理计算, 尺寸不一致时退回只画 from。
        let (tex_from, tex_to, fade) =
            if (tex_from.width, tex_from.height) != (tex_to.width, tex_to.height) {
                log::warn!("场景图尺寸不一致, 跳过淡化");
                (tex_from, tex_from, 0.0)
            } else {
                (tex_from, tex_to, fade)
            };
        self.upload_quad(
            queue,
            target,
            layer,
            tex_from.width,
            tex_from.height,
            scale,
            opacity,
            fade,
        );
        pass.set_bind_group(0, &self.uniform_binds[layer], &[]);
        pass.set_bind_group(1, &tex_from.bind_group, &[]);
        pass.set_bind_group(2, &tex_to.bind_group, &[]);
        let first = (layer * VERTS_PER_LAYER) as u32;
        pass.draw(first..first + VERTS_PER_LAYER as u32, 0..1);
    }

    /// 按缩放模式计算顶点与 UV, 写入指定层的顶点区段与 uniform buffer。
    #[allow(clippy::too_many_arguments)]
    fn upload_quad(
        &self,
        queue: &wgpu::Queue,
        target: &DrawTarget,
        layer: usize,
        img_w: u32,
        img_h: u32,
        scale: ScaleMode,
        opacity: f32,
        fade: f32,
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
        queue.write_buffer(
            &self.vertex_buf,
            (layer * VERTS_PER_LAYER * size_of::<Vertex>()) as u64,
            bytemuck::cast_slice(&verts),
        );
        queue.write_buffer(
            &self.uniform_bufs[layer],
            0,
            bytemuck::cast_slice(&[opacity, fade, 0.0, 0.0f32]),
        );
    }
}

/// 加载 PNG 并创建纹理与 bind group; 失败时返回 None 并记录警告。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_config_default_is_empty() {
        let cfg = BackgroundConfig::default();
        assert!(cfg.image.is_none());
        assert!(cfg.glow.is_none());
        assert!(cfg.noise.is_none());
        assert_eq!(cfg.scale, ScaleMode::Stretch);
        assert!((cfg.glow_opacity - 0.0).abs() < f32::EPSILON);
        assert!((cfg.noise_opacity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn background_config_chaining() {
        let cfg = BackgroundConfig::with_image("gradient.png")
            .with_glow("glow.png", 0.15)
            .with_noise("noise.png", 0.08)
            .scale(ScaleMode::Cover);

        assert_eq!(cfg.image.as_ref().unwrap().as_os_str(), "gradient.png");
        assert_eq!(cfg.glow.as_ref().unwrap().as_os_str(), "glow.png");
        assert_eq!(cfg.noise.as_ref().unwrap().as_os_str(), "noise.png");
        assert_eq!(cfg.scale, ScaleMode::Cover);
        assert!((cfg.glow_opacity - 0.15).abs() < f32::EPSILON);
        assert!((cfg.noise_opacity - 0.08).abs() < f32::EPSILON);
    }

    #[test]
    fn background_config_opacity_is_clamped() {
        let high = BackgroundConfig::with_image("bg.png").with_glow("glow.png", 1.5);
        assert!((high.glow_opacity - 1.0).abs() < f32::EPSILON);

        let low = BackgroundConfig::with_image("bg.png").with_glow("glow.png", -0.5);
        assert!((low.glow_opacity - 0.0).abs() < f32::EPSILON);

        let high_noise = BackgroundConfig::with_image("bg.png").with_noise("noise.png", 1.5);
        assert!((high_noise.noise_opacity - 1.0).abs() < f32::EPSILON);

        let low_noise = BackgroundConfig::with_image("bg.png").with_noise("noise.png", -0.5);
        assert!((low_noise.noise_opacity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scale_mode_default_is_stretch() {
        assert_eq!(ScaleMode::default(), ScaleMode::Stretch);
    }

    #[test]
    fn background_config_with_scenes() {
        let cfg =
            BackgroundConfig::with_scenes(["a.png", "b.png", "c.png"]).scale(ScaleMode::Cover);
        assert_eq!(cfg.scenes.len(), 3);
        assert!(cfg.image.is_none());
        assert_eq!(cfg.scale, ScaleMode::Cover);
    }

    #[test]
    fn background_frame_clamps_fade() {
        let c = crate::Color::BLACK;
        assert!((BackgroundFrame::new(0, 1, -0.5, c).fade - 0.0).abs() < f32::EPSILON);
        assert!((BackgroundFrame::new(0, 1, 1.5, c).fade - 1.0).abs() < f32::EPSILON);
        assert!((BackgroundFrame::new(0, 1, 0.4, c).fade - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_frame_clamps_indices_to_scene_count() {
        let c = crate::Color::BLACK;
        assert_eq!(
            resolve_frame(BackgroundFrame::new(0, 7, 0.5, c), 4),
            Some((0, 3, 0.5))
        );
        assert_eq!(
            resolve_frame(BackgroundFrame::new(9, 9, 1.0, c), 2),
            Some((1, 1, 1.0))
        );
    }

    #[test]
    fn resolve_frame_empty_scenes_is_none() {
        let c = crate::Color::BLACK;
        assert_eq!(resolve_frame(BackgroundFrame::new(0, 0, 0.0, c), 0), None);
    }
}
