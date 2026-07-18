// ! @author 十四叔
// ! @date 2026/07/17

// ! 渲染层:wgpu 上下文 (实例 / 适配器 / 设备 / 队列 /surface)。
// !
// ! 本模块是允许接触图形 API 的适配层之一, 对上层暴露
// ! "清屏 + 绘制一帧矩形"的能力; 文本管线在后续模块中加入。

mod rect;
mod text;

pub use rect::{DrawTarget, RectBatch, RectPipeline};
pub use text::{TextBatch, TextPipeline};

use std::sync::Arc;

use winit::window::Window as WinitWindow;

use crate::Color;

/// 根据平台选择单一主 backend,避免实例创建时扫描多个后端。
#[cfg(target_os = "windows")]
const DEFAULT_BACKENDS: wgpu::Backends = wgpu::Backends::DX12;
#[cfg(target_os = "macos")]
const DEFAULT_BACKENDS: wgpu::Backends = wgpu::Backends::METAL;
#[cfg(all(unix, not(target_os = "macos")))]
const DEFAULT_BACKENDS: wgpu::Backends = wgpu::Backends::VULKAN;
#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    all(unix, not(target_os = "macos"))
)))]
const DEFAULT_BACKENDS: wgpu::Backends = wgpu::Backends::PRIMARY;

/// 渲染上下文初始化或运行期错误。
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// 创建 surface 失败。
    #[error("创建 surface 失败: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    /// 请求适配器失败。
    #[error("请求 GPU 适配器失败: {0}")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),
    /// 请求设备失败。
    #[error("请求 GPU 设备失败: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

/// wgpu 渲染上下文。持有设备、队列、surface 配置与各渲染管线。
pub struct Context {
    /// 渲染目标 surface。
    surface: wgpu::Surface<'static>,
    /// 逻辑设备。
    device: wgpu::Device,
    /// 命令队列。
    queue: wgpu::Queue,
    /// surface 当前配置。
    config: wgpu::SurfaceConfiguration,
    /// 清屏颜色。
    clear_color: Color,
    /// 矩形渲染管线。
    rect_pipeline: RectPipeline,
    /// 文本渲染管线。
    text_pipeline: TextPipeline,
}

impl Context {
    /// 在指定窗口上初始化 wgpu,surface 尺寸取窗口当前物理尺寸。
    pub fn new(window: Arc<WinitWindow>, clear_color: Color) -> Result<Self, RenderError> {
        pollster::block_on(Self::new_async(window, clear_color))
    }

    async fn new_async(window: Arc<WinitWindow>, clear_color: Color) -> Result<Self, RenderError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: DEFAULT_BACKENDS,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;
        log::info!("GPU 适配器: {}", adapter.get_info().name);
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("danqing device"),
                ..Default::default()
            })
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo, // vsync
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        log::info!(
            "surface 已配置: {}x{}, 格式 {format:?}",
            config.width,
            config.height
        );

        let rect_pipeline = RectPipeline::new(&device, format);
        let text_pipeline = TextPipeline::new(&device, format, crate::GlyphAtlas::DEFAULT_SIZE);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            clear_color,
            rect_pipeline,
            text_pipeline,
        })
    }

    /// 窗口尺寸变化时重建 surface 配置 (0 尺寸最小化期间忽略)。
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        log::debug!("surface 重建: {width}x{height}");
    }

    /// 渲染一帧: 清屏 → 矩形 pass → 文本 pass。
    /// 返回 false 表示出现致命错误, 应退出。
    pub fn render(&mut self, rects: &RectBatch, texts: &mut TextBatch) -> bool {
        use wgpu::CurrentSurfaceTexture as CST;
        let frame = match self.surface.get_current_texture() {
            CST::Success(frame) | CST::Suboptimal(frame) => frame,
            CST::Timeout => {
                log::warn!("获取帧超时,跳过本帧");
                return true;
            }
            CST::Occluded => {
                // 窗口被遮挡 / 最小化: 跳过本帧
                return true;
            }
            CST::Outdated | CST::Lost => {
                // surface 丢失 / 过期: 重建后继续
                self.surface.configure(&self.device, &self.config);
                return true;
            }
            CST::Validation => {
                log::error!("获取帧时出现校验错误,跳过本帧");
                return true;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });
        self.rect_pipeline.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &DrawTarget {
                view: &view,
                width: self.config.width as f32,
                height: self.config.height as f32,
                clear_color: self.clear_color,
            },
            rects,
        );
        self.text_pipeline.draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &DrawTarget {
                view: &view,
                width: self.config.width as f32,
                height: self.config.height as f32,
                clear_color: self.clear_color,
            },
            texts,
        );
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
        true
    }
}
