# Todo: Image 纹理渲染支持

## Tasks

- [x] T1: 创建图像着色器 `image.wgsl`
  - Acceptance: 顶点/片元着色器编译通过，支持 UV 采样和裁剪
  - Verify: `cargo build` 无错误
  - Files: `src/render/image.wgsl`, `src/render/image.rs`

- [x] T2: 实现 ImagePipeline GPU 管线
  - Acceptance: 能创建纹理、上传 RGBA 数据、执行渲染
  - Verify: 单元测试纹理创建
  - Files: `src/render/image.rs`

- [x] T3: 实现 ImageBatch CPU 收集器
  - Acceptance: 能收集图像实例，管理纹理缓存
  - Verify: 单元测试 push_image + 纹理缓存逻辑
  - Files: `src/render/image.rs`

- [x] T4: 改造 Image 组件
  - Acceptance: paint() 使用 ImageBatch 显示真实纹理
  - Verify: `cargo test --lib` 通过
  - Files: `src/widget/base/image.rs`, `src/widget/mod.rs`

- [x] T5: 集成到 Context
  - Acceptance: Context 持有 ImagePipeline，render_frame 调用绘制
  - Verify: showcase 编译运行无错误
  - Files: `src/render/mod.rs`, `src/window/handler.rs`

- [x] T6: 更新 showcase 演示
  - Acceptance: LOGO 图片显示真实像素，文件对话框图片正确显示
  - Verify: 手动运行 showcase 验证
  - Files: `examples/showcase.rs`
