# Plan: Image 纹理渲染支持

## 目标

给 `Image` 组件添加真实像素渲染能力，支持显示 PNG/JPEG 图片内容。

## 架构

```
┌─────────────────────────────────────────────────────────┐
│                    Image 组件 (widget)                    │
│  - 存储 RGBA 数据 + 尺寸                                 │
│  - layout() 计算约束                                     │
│  - paint() 向 ImageBatch 添加实例                        │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                    ImageBatch (CPU)                       │
│  - 收集 ImageInstance 列表                                │
│  - 纹理缓存: HashMap<(w,h), TextureId>                   │
│  - 需要上传的纹理队列                                     │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│                    ImagePipeline (GPU)                    │
│  - 纹理创建/上传                                         │
│  - 采样器 + 绑定组                                       │
│  - 渲染管线 + 着色器                                     │
└─────────────────────────────────────────────────────────┘
```

## 实现步骤

### T1: 创建图像着色器

**文件:** `src/render/image.wgsl`

- 顶点着色器: 接收 quad 位置 + UV
- 片元着色器: 采样纹理 RGBA
- 支持裁剪矩形

### T2: 实现 ImagePipeline

**文件:** `src/render/image.rs`

- `ImagePipeline::new()` 创建管线
- `upload_texture()` 上传 RGBA 数据到 GPU
- `draw()` 批量绘制图像实例

### T3: 实现 ImageBatch

**文件:** `src/render/image.rs` (同一文件)

- `ImageBatch::new()` 初始化
- `push_image()` 添加图像实例
- `push_texture()` 标记需要上传的纹理
- `draw()` 调用 ImagePipeline 绘制

### T4: 改造 Image 组件

**文件:** `src/widget/base/image.rs`

- 新增 `texture_id: Option<usize>` 字段
- `paint()` 中调用 `batch.push_texture()` + `batch.push_image()`
- 保持宽高比计算不变

### T5: 集成到 Context

**文件:** `src/render/mod.rs`

- Context 新增 `image_pipeline: ImagePipeline`
- `render_frame()` 中调用 `image_batch.draw()`

### T6: 更新 showcase 演示

**文件:** `examples/showcase.rs`

- 验证 LOGO 图片显示
- 验证文件对话框打开图片显示

## 验证点

1. T1-T2 完成后: 单元测试着色器编译
2. T3-T4 完成后: `cargo test --lib` 通过
3. T5-T6 完成后: showcase 显示真实图片
4. 最终: `cargo clippy -- -D warnings` 零警告
