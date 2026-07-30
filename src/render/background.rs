//! @author 十四叔
//! @date 2026/07/19

//! 窗口背景图渲染: 将 `assets/background/` 下的渐变 / 光晕 / 噪声图
//! 绘制在组件树之下。
//!
//! 当前支持一张主背景图与可选的光晕、噪声叠加图, 并提供 Stretch/Fit/Cover
//! 三种缩放模式。多场景模式下, 场景图按 2 槽 LRU 懒加载:
//! `new` 阶段只预读 PNG 字节 (~1MB), 真正上传为 wgpu 纹理推迟到 `set_frame`
//! 调用时, 同时常驻最多 2 张 (`from` + `to` 跨淡化的两端)。

use std::collections::{HashMap, VecDeque};
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
    /// 动效时间 (秒, 注入时间轴; 默认 0, 经 [`BackgroundFrame::with_motion`] 设置)。
    pub time: f32,
    /// 雨丝动效强度 (0.0 ..= 1.0; 默认 0 = 无动效, shader 输出与静态一致)。
    pub rain_intensity: f32,
    /// 篝火动效强度 (0.0 ..= 1.0; 默认 0 = 无动效; 与雨并存, 交叉淡化期间可同时非零)。
    pub fire_intensity: f32,
    /// 海动效强度 (0.0 ..= 1.0; 默认 0 = 无动效; 与雨/火并存, 交叉淡化期间可同时非零)。
    pub sea_intensity: f32,
    /// 雨钟 (秒): 雨丝下落时间轴, 暂停时冻结 — 雨丝定格可见 (2026-07-29 用户裁定)。
    /// 默认 0, 经 [`BackgroundFrame::with_rain_time`] 设置, 上传 uniform 前取模。
    pub rain_time: f32,
    /// 山动效强度 (0.0 ..= 1.0; 默认 0 = 无动效; 与雨/火/海并存, 交叉淡化期间可同时非零)。
    pub mountain_intensity: f32,
    /// 森林动效强度 (0.0 ..= 1.0; 默认 0 = 无动效; 与雨/火/海/山并存, 交叉淡化期间可同时非零)。
    pub forest_intensity: f32,
}

impl BackgroundFrame {
    /// 构造每帧背景状态;`fade` 夹到 0..1, 动效参数默认为 0 (无动效)。
    pub fn new(from: usize, to: usize, fade: f32, clear_color: crate::Color) -> Self {
        Self {
            from,
            to,
            fade: fade.clamp(0.0, 1.0),
            clear_color,
            time: 0.0,
            rain_intensity: 0.0,
            fire_intensity: 0.0,
            sea_intensity: 0.0,
            rain_time: 0.0,
            mountain_intensity: 0.0,
            forest_intensity: 0.0,
        }
    }

    /// 设置场景动效参数 (时间秒 + 雨丝强度); 强度夹到 0..1。
    pub fn with_motion(mut self, time: f32, rain_intensity: f32) -> Self {
        self.time = time;
        self.rain_intensity = rain_intensity.clamp(0.0, 1.0);
        self
    }

    /// 设置篝火动效强度; 强度夹到 0..1。
    pub fn with_fire(mut self, fire_intensity: f32) -> Self {
        self.fire_intensity = fire_intensity.clamp(0.0, 1.0);
        self
    }

    /// 设置海动效强度; 强度夹到 0..1。
    pub fn with_sea(mut self, sea_intensity: f32) -> Self {
        self.sea_intensity = sea_intensity.clamp(0.0, 1.0);
        self
    }

    /// 设置山动效强度; 强度夹到 0..1。
    pub fn with_mountain(mut self, mountain_intensity: f32) -> Self {
        self.mountain_intensity = mountain_intensity.clamp(0.0, 1.0);
        self
    }

    /// 设置森林动效强度; 强度夹到 0..1。
    pub fn with_forest(mut self, forest_intensity: f32) -> Self {
        self.forest_intensity = forest_intensity.clamp(0.0, 1.0);
        self
    }

    /// 设置雨钟 (雨丝下落时间轴, 秒); 推进/冻结节奏由调用方控制。
    pub fn with_rain_time(mut self, rain_time: f32) -> Self {
        self.rain_time = rain_time;
        self
    }
}

/// 场景动效时间取模周期 (秒): 与 background.wgsl 雨/火/海效果频率的公共周期一致
/// (雨丝速度 0.125/0.25/0.375、火/海效频率取 1/8 Hz 整数倍 → 公共周期 8s)。
/// 上传 uniform 前取模, 避免常驻数小时后 f32 时间精度退化导致相位抖动。
const MOTION_WRAP_SECS: f32 = 8.0;

/// 动效时间取模 (纯逻辑): 折回 `[0, MOTION_WRAP_SECS)`; 负值按欧几里得余数处理。
fn wrap_motion_time(time: f32) -> f32 {
    time.rem_euclid(MOTION_WRAP_SECS)
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
/// 场景纹理 LRU 容量: `from` + `to` 跨淡化的两端, 2 槽即够。
const SCENE_CACHE_CAPACITY: usize = 2;

/// 背景渲染管线。
///
/// 每层使用独立的 uniform buffer 与顶点区段: `Queue::write_buffer`
/// 在单次 submit 前统一生效, 同一帧内多次写同一块 buffer 时
/// 只有最后一次写入可见, 因此跨 draw 复用会导致所有层参数相同。
///
/// 场景层 (层 0) 绑定 from/to 两张场景图按 fade 交叉淡化;
/// 单图与叠加层把同一张图绑到两个纹理槽, fade 恒 0。
///
/// 场景纹理走 2 槽 LRU: `scene_bytes` 在 `new` 阶段全量预读 (~1MB),
/// `device` / `queue` / `texture_layout` clone 持有用于按需创建纹理,
/// `scene_cache` + `lru_order` 实现按访问顺序的纹理驻留。
pub struct BackgroundPipeline {
    pipeline: wgpu::RenderPipeline,
    /// 每层一个 uniform buffer (不透明度 + 淡化进度)。
    uniform_bufs: [wgpu::Buffer; LAYER_COUNT],
    uniform_binds: [wgpu::BindGroup; LAYER_COUNT],
    /// 三层 quad 共用的顶点缓冲 (每层 [`VERTS_PER_LAYER`] 个顶点)。
    vertex_buf: wgpu::Buffer,
    /// 场景图原始 PNG 字节 (按场景索引;`new` 阶段预读, 总量约 1MB)。
    scene_bytes: Vec<Vec<u8>>,
    /// 场景图原始尺寸 (供纹理创建时 `Extent3d` 使用, 与 `scene_bytes` 平行)。
    scene_dims: Vec<(u32, u32)>,
    /// 场景纹理 LRU: 命中 [`SCENE_CACHE_CAPACITY`] 槽。
    scene_cache: HashMap<usize, BackgroundTexture>,
    /// 场景访问顺序 (front = 最近, back = 最久未用, 淘汰时弹出 back)。
    lru_order: VecDeque<usize>,
    /// 设备句柄 (clone 持有, 用于按需创建纹理; wgpu 30 内部 Arc, clone 廉价)。
    device: wgpu::Device,
    /// 队列句柄 (clone 持有, 用于按需上传纹理)。
    queue: wgpu::Queue,
    /// 纹理 bind group layout (clone 持有, 用于按需创建 bind group)。
    texture_layout: wgpu::BindGroupLayout,
    /// 光晕叠加纹理 (单一资源, 启动时即用, 不进 LRU)。
    glow: Option<BackgroundTexture>,
    /// 噪声叠加纹理 (单一资源, 启动时即用, 不进 LRU)。
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
                size: 32,
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
        // 预读所有场景的 PNG 字节与尺寸; 真正的 GPU 纹理推迟到
        // `set_frame` 调用时按需创建 (见 `ensure_loaded`)。
        let mut scene_bytes = Vec::with_capacity(scene_paths.len());
        let mut scene_dims = Vec::with_capacity(scene_paths.len());
        for path in &scene_paths {
            match read_scene_bytes(path) {
                Some((bytes, dims)) => {
                    scene_bytes.push(bytes);
                    scene_dims.push(dims);
                }
                None => {
                    // 预读失败: 占位空字节保持索引对齐, ensure_loaded 静默
                    // 跳过, draw 端走缺失分支降级 (clear_color 透出)。
                    log::warn!("场景图预读失败, 该场景将不显示: {}", path.display());
                    scene_bytes.push(Vec::new());
                    scene_dims.push((0, 0));
                }
            }
        }
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
            scene_bytes,
            scene_dims,
            scene_cache: HashMap::new(),
            lru_order: VecDeque::new(),
            device: device.clone(),
            queue: queue.clone(),
            texture_layout: texture_layout.clone(),
            glow,
            noise,
            scale: config.scale,
            glow_opacity: config.glow_opacity.clamp(0.0, 1.0),
            noise_opacity: config.noise_opacity.clamp(0.0, 1.0),
            frame: None,
        }
    }

    /// 是否存在可绘制的背景。
    ///
    /// 基于"已配置的 scene_bytes 项", 而非"已加载的 GPU 纹理" ——
    /// 框架应用层关心"是否有背景要画", 不关心此刻是否 decode 完毕。
    pub fn has_background(&self) -> bool {
        !self.scene_bytes.is_empty()
    }

    /// 写入应用层产出的每帧背景状态 (场景选择 / 淡化 / 清屏色)。
    ///
    /// 同时确保 `from` 和 `to` 两个场景的 GPU 纹理在 LRU 中 (按需创建,
    /// 必要时淘汰最久未用项); 这样 `draw` 可以假设两端纹理就绪。
    pub fn set_frame(&mut self, frame: BackgroundFrame) {
        self.frame = Some(frame);
        self.ensure_loaded(frame.from);
        self.ensure_loaded(frame.to);
    }

    /// 确保指定场景索引的 GPU 纹理在 LRU 中。
    ///
    /// 命中: 刷新 LRU 顺序 (移到 front) 后返回。
    /// 未命中: 从 `scene_bytes` decode 并创建 wgpu 纹理, 插入缓存;
    /// 若缓存已满, 弹出 `lru_order` 尾部索引并丢弃其 `BackgroundTexture`
    /// (wgpu 通过 `Drop` 自动释放对应 GPU 资源)。
    fn ensure_loaded(&mut self, idx: usize) {
        // 越界或预读失败的空字节: 静默跳过, draw 端处理缺失分支。
        if idx >= self.scene_bytes.len() || self.scene_bytes[idx].is_empty() {
            return;
        }
        // LRU 命中: 移到 front, 立即返回。
        if self.scene_cache.contains_key(&idx) {
            if let Some(pos) = self.lru_order.iter().position(|&i| i == idx) {
                self.lru_order.remove(pos);
            }
            self.lru_order.push_front(idx);
            return;
        }
        // 未命中: decode + 创建纹理 (PNG 字节在此才解码, 见 read_scene_bytes)。
        let bytes = self.scene_bytes[idx].clone();
        let dims = self.scene_dims[idx];
        let Some(tex) = self.create_scene_texture(&bytes, dims, idx) else {
            // 解码失败: 置空槽位, 后续 ensure_loaded 走空字节静默守卫,
            // 避免每帧重试解码 + 刷 warn 日志 (set_frame 是每帧调用的)。
            // 字节不可变, 失败槽位永远不可能之后解码成功, 置空无损。
            self.scene_bytes[idx] = Vec::new();
            return;
        };
        // 缓存已满时淘汰 LRU 尾。
        while self.scene_cache.len() >= SCENE_CACHE_CAPACITY {
            if let Some(evicted) = self.lru_order.pop_back() {
                self.scene_cache.remove(&evicted);
            } else {
                break;
            }
        }
        self.scene_cache.insert(idx, tex);
        self.lru_order.push_front(idx);
    }

    /// 从预读的 PNG 字节解码并创建 wgpu 纹理 (懒加载的实际解码上传)。
    /// 解码失败返回 None (调用方跳过插入, draw 端走缺失分支降级)。
    fn create_scene_texture(
        &self,
        bytes: &[u8],
        dims: (u32, u32),
        idx: usize,
    ) -> Option<BackgroundTexture> {
        let img = match image::load_from_memory(bytes) {
            Ok(img) => img.into_rgba8(),
            Err(err) => {
                log::warn!("场景图解码失败 scene[{idx}]: {err}");
                return None;
            }
        };
        let (width, height) = dims;
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("scene[{idx}] texture")),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("scene[{idx}] sampler")),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("scene[{idx}] bind group")),
            layout: &self.texture_layout,
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
        self.queue.write_texture(
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
            time: 0.0,
            rain_intensity: 0.0,
            fire_intensity: 0.0,
            sea_intensity: 0.0,
            rain_time: 0.0,
            mountain_intensity: 0.0,
            forest_intensity: 0.0,
        });
        let Some((from, to, fade)) = resolve_frame(frame, self.scene_bytes.len()) else {
            return;
        };
        // 场景层动效参数 (雨/火/海/山/森林强度 + 取模后的时间与雨钟); 叠加层无动效恒 0。
        let motion = [
            frame.rain_intensity,
            wrap_motion_time(frame.time),
            frame.fire_intensity,
            frame.sea_intensity,
            wrap_motion_time(frame.rain_time),
            frame.mountain_intensity,
            frame.forest_intensity,
        ];

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
        // LRU 缺失分支: set_frame 已尝试 ensure_loaded, 但越界或预读
        // 失败仍可能留下空槽, 这里做优雅降级 — 单图无淡化, 缺则跳过。
        let tex_from = self.scene_cache.get(&from);
        let tex_to = self.scene_cache.get(&to);
        if let (Some(tex_from), Some(tex_to)) = (tex_from, tex_to) {
            self.draw_layer(
                &mut pass, queue, target, 0, tex_from, tex_to, self.scale, 1.0, fade, motion,
            );
        } else if let Some(only) = tex_from.or(tex_to) {
            // 仅一端就绪: 单图绘制, fade=0 (无淡化)
            self.draw_layer(
                &mut pass, queue, target, 0, only, only, self.scale, 1.0, 0.0, motion,
            );
        } // 两端都缺失: 不画场景层, 让 clear_color 透出
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
                [0.0; 7],
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
                [0.0; 7],
            );
        }
    }

    /// 绘制单个叠加层: 上传该层顶点与 uniform, 绑定资源后绘制。
    /// `motion` = [雨丝强度, 取模后的动效时间, 篝火强度, 海强度, 取模后的雨钟, 山强度, 森林强度], 仅场景层 (层 0) 非零。
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
        motion: [f32; 7],
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
            motion,
        );
        pass.set_bind_group(0, &self.uniform_binds[layer], &[]);
        pass.set_bind_group(1, &tex_from.bind_group, &[]);
        pass.set_bind_group(2, &tex_to.bind_group, &[]);
        let first = (layer * VERTS_PER_LAYER) as u32;
        pass.draw(first..first + VERTS_PER_LAYER as u32, 0..1);
    }

    /// 按缩放模式计算顶点与 UV, 写入指定层的顶点区段与 uniform buffer。
    /// uniform 布局 (36B): [opacity, fade, 雨丝强度, 动效时间, 篝火强度, 海强度, 雨钟, 山强度, 森林强度]。
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
        motion: [f32; 7],
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
            bytemuck::cast_slice(&[
                opacity, fade, motion[0], motion[1], motion[2], motion[3], motion[4], motion[5],
                motion[6],
            ]),
        );
    }
}

/// 读取场景 PNG 文件字节与尺寸 (尺寸从内存字节解析头, 不解码); 失败时返回 None。
///
/// 与 `load_texture` 不同: 本函数只读原始 PNG 字节 (5 场景合计 ~0.8MB),
/// 不接触 wgpu 设备, 也不做 RGBA 解码 — 解码推迟到 `ensure_loaded`
/// 懒加载路径, 避免启动期为 5 张图常驻 ~31MB 解码缓冲。
/// 尺寸与字节同源 (单次文件读取), 不存在两次读文件之间被替换的不一致窗口。
fn read_scene_bytes(path: &Path) -> Option<(Vec<u8>, (u32, u32))> {
    let data = std::fs::read(path).ok()?;
    let dims = image::ImageReader::new(std::io::Cursor::new(&data))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    Some((data, dims))
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
    fn background_frame_motion_defaults_zero() {
        let f = BackgroundFrame::new(0, 1, 0.5, crate::Color::BLACK);
        assert_eq!(f.time, 0.0);
        assert_eq!(f.rain_intensity, 0.0);
        assert_eq!(f.fire_intensity, 0.0);
        assert_eq!(f.sea_intensity, 0.0);
        assert_eq!(f.mountain_intensity, 0.0);
        assert_eq!(f.forest_intensity, 0.0);
    }

    #[test]
    fn with_motion_sets_time_and_clamps_intensity() {
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c).with_motion(12.5, 1.7);
        assert!((f.time - 12.5).abs() < f32::EPSILON);
        assert!((f.rain_intensity - 1.0).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_motion(3.0, -0.2);
        assert_eq!(f.rain_intensity, 0.0);
    }

    #[test]
    fn with_fire_sets_and_clamps_intensity() {
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c).with_fire(0.8);
        assert!((f.fire_intensity - 0.8).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_fire(1.7);
        assert!((f.fire_intensity - 1.0).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_fire(-0.2);
        assert_eq!(f.fire_intensity, 0.0);
    }

    #[test]
    fn with_sea_sets_and_clamps_intensity() {
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c).with_sea(0.8);
        assert!((f.sea_intensity - 0.8).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_sea(1.7);
        assert!((f.sea_intensity - 1.0).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_sea(-0.2);
        assert_eq!(f.sea_intensity, 0.0);
    }

    #[test]
    fn with_motion_and_with_fire_are_independent() {
        // 雨/火是两个并存标量 (交叉淡化期间可同时非零), 链式设置互不覆盖。
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c)
            .with_motion(2.5, 0.4)
            .with_fire(0.6);
        assert!((f.time - 2.5).abs() < f32::EPSILON);
        assert!((f.rain_intensity - 0.4).abs() < f32::EPSILON);
        assert!((f.fire_intensity - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn with_sea_is_independent_of_rain_and_fire() {
        // 雨/火/海是三个并存标量 (交叉淡化期间两两可同时非零), 链式设置互不覆盖。
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c)
            .with_motion(2.5, 0.4)
            .with_fire(0.6)
            .with_sea(0.7);
        assert!((f.time - 2.5).abs() < f32::EPSILON);
        assert!((f.rain_intensity - 0.4).abs() < f32::EPSILON);
        assert!((f.fire_intensity - 0.6).abs() < f32::EPSILON);
        assert!((f.sea_intensity - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn with_mountain_sets_and_clamps_intensity() {
        // 山效果是并存标量; 强度夹到 [0, 1]。
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c).with_mountain(0.8);
        assert!((f.mountain_intensity - 0.8).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_mountain(1.7);
        assert!((f.mountain_intensity - 1.0).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_mountain(-0.2);
        assert_eq!(f.mountain_intensity, 0.0);
    }

    #[test]
    fn with_forest_sets_and_clamps_intensity() {
        // 森林效果是并存标量; 强度夹到 [0, 1]。
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c).with_forest(0.65);
        assert!((f.forest_intensity - 0.65).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_forest(1.5);
        assert!((f.forest_intensity - 1.0).abs() < f32::EPSILON);

        let f = BackgroundFrame::new(0, 0, 0.0, c).with_forest(-0.3);
        assert_eq!(f.forest_intensity, 0.0);
    }

    #[test]
    fn with_mountain_is_independent_of_rain_fire_sea_forest() {
        // 山是第五个并存标量 (与雨/火/海/森林并存, 交叉淡化期间可同时非零)。
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c)
            .with_motion(2.5, 0.4)
            .with_fire(0.6)
            .with_sea(0.7)
            .with_mountain(0.5)
            .with_forest(0.55);
        assert!((f.time - 2.5).abs() < f32::EPSILON);
        assert!((f.rain_intensity - 0.4).abs() < f32::EPSILON);
        assert!((f.fire_intensity - 0.6).abs() < f32::EPSILON);
        assert!((f.sea_intensity - 0.7).abs() < f32::EPSILON);
        assert!((f.mountain_intensity - 0.5).abs() < f32::EPSILON);
        assert!((f.forest_intensity - 0.55).abs() < f32::EPSILON);
    }

    #[test]
    fn with_rain_time_sets_clock_and_defaults_zero() {
        // 雨钟独立于动效时间 (雨丝暂停定格可见, 走自己的冻结时间轴)。
        let c = crate::Color::BLACK;
        let f = BackgroundFrame::new(0, 0, 0.0, c);
        assert_eq!(f.rain_time, 0.0, "雨钟默认 0 (静态一致)");
        let f = f.with_motion(2.5, 0.4).with_rain_time(1.75);
        assert!((f.time - 2.5).abs() < f32::EPSILON);
        assert!((f.rain_time - 1.75).abs() < f32::EPSILON);
    }

    #[test]
    fn wrap_motion_time_wraps_and_stays_positive() {
        assert!(wrap_motion_time(0.0).abs() < f32::EPSILON);
        assert!(wrap_motion_time(MOTION_WRAP_SECS).abs() < f32::EPSILON);
        assert!((wrap_motion_time(9.5) - 1.5).abs() < 1e-6);
        assert!((wrap_motion_time(-0.5) - (MOTION_WRAP_SECS - 0.5)).abs() < 1e-6);
        // 常驻数小时的大时间值仍折回周期内 (f32 精度护栏)。
        assert!(wrap_motion_time(36000.0) < MOTION_WRAP_SECS);
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
