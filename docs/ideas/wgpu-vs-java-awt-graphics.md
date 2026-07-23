# wgpu 与 Java AWT Graphics 对比

> 日期：2026/07/22
> 背景: 丹青选择 wgpu 作为自绘渲染底座,本文记录与 Java 2D 时代代表技术 AWT Graphics/Graphics2D 的系统性对比,作为选型依据存档。

## 一句话结论

Graphics2D 是"告诉画笔画什么",API 友好但天花板低;wgpu 是"告诉 GPU 怎么渲染",前期成本高昂，但性能、视觉效果上限和跨平台一致性完全不在一个时代。丹青选 wgpu 是正确决定——毛玻璃模糊、SDF 圆角、流畅动画这些阶段 1 的视觉效果，在 Graphics2D 的技术世代里要么做不到，要么帧率撑不住。

## 定位与年代

| | Rust wgpu | Java AWT Graphics/Graphics2D |
|---|---|---|
| 诞生 | 2020+(WebGPU 标准的 Rust 实现) | 1995(AWT 1.0),Graphics2D 随 Java 2D(1998) |
| 抽象对象 | **GPU**:直接管理缓冲区、管线、纹理、着色器 | **画布**:CPU 软件光栅化为主,绘制调用式 API |
| 渲染模型 | 保留资源 + 每帧提交 command encoder | 立即模式 (immediate mode),画完即弃 |
| 底层后端 | Vulkan / Metal / D3D12 / GLES | 平台 2D 栈 (GDI、X11、Quartz),部分路径可走 OpenGL/D3D 加速 |

## 性能

差距最悬殊的维度。

- **wgpu**:图元走 GPU 管线,一次 draw call 可以提交成千上万个矩形/字形实例(丹青的 RectBatch/TextBatch 就是这个模式)。60fps 下渲染数千个 SDF 圆角矩形 + 文本图集,GPU 占用很低;毛玻璃模糊、阴影、渐变这类效果靠 shader 几乎免费。
- **AWT Graphics2D**:默认 CPU 光栅化,每帧重绘靠双缓冲避免闪烁。抗锯齿、半透明合成(AlphaComposite)、大半径模糊都是 CPU 密集型操作,复杂界面容易掉到不可用的帧率。Java2D 有 OpenGL/D3D 加速管道但默认不总开启,且行为因平台而异。

量级差异:同样画 5000 个圆角矩形,wgpu 是 GPU 一次实例化提交;Graphics2D 是 5000 次 CPU 路径光栅化。

## 文本渲染

| | wgpu | Graphics2D |
|---|---|---|
| 字形处理 | 无内建文本,需自己组合(fontdue/cosmic-text 栅格化 + 字形图集 + shader) | 内建:`drawString`,字体度量、抗锯齿、复杂文本布局都有 |
| 质量上限 | 取决于自己的实现(丹青:图集 + 每字形 quad) | 成熟,LCD 子像素渲染、hinting 开箱即用 |
| 工作量 | 大——这是丹青 `text/` 模块存在的原因 | 几乎为零 |

**Graphics2D 最大的现实优势就在文本。** wgpu 是"给你 GPU，文本自己想办法"。

## API 风格

**Graphics2D** —— 命令式画笔，非常直觉：

```java
g2.setColor(Color.BLUE);
g2.fillRoundRect(x, y, w, h, 8, 8);
g2.drawString("你好", x, y);
```

**wgpu** —— 显式资源管理，样板代码量大：

```rust
// 创建管线、顶点缓冲、uniform、绑定组...
let render_pass = encoder.begin_render_pass(&desc);
render_pass.set_pipeline(&pipeline);
render_pass.set_bind_group(0, &bind_group, &[]);
render_pass.draw(0..6, 0..instance_count);
```

wgpu 的学习曲线前置 (设备/队列/管线/绑定组布局一套概念),但一旦封装好 (丹青的 `render/`),上层也能做到接近画笔式的体验。

## 跨平台与生态

- **wgpu**:Windows/macOS/Linux/Web(编译到 WASM 走 WebGPU)/移动端。一份代码到处跑,行为高度一致——因为着色器和管线是自己写的,不依赖平台 2D 栈的差异。
- **AWT**:桌面三平台,但渲染细节(字体 hinting、抗锯齿策略)因平台而异,"write once, debug everywhere" 的老梗部分源于此。无 Web、无现代移动端。

## 适用场景

| 场景 | 更合适的选择 |
|---|---|
| 自绘 UI 框架、毛玻璃/动画/高帧率 (丹青的目标) | **wgpu** |
| 快速画个内部工具界面、打印/报表 2D 图形 | Graphics2D |
| 图表、科学可视化 (大量图元) | wgpu |
| 学习计算机图形学/需要精确控制每帧 | wgpu |
| 维护 Swing 遗留系统 | Graphics2D(没得选) |

## 对丹青的启示

1. **文本是唯一需要持续投入的短板**。wgpu 不提供任何文本设施，丹青的 `text/`(字体加载、字形图集、`line_layout` 排版) 就是为此存在，未来复杂文本 (双向、shaping) 都要在这里补齐。
2. **批量提交是性能的关键模式**。RectBatch/TextBatch 的实例化设计要保留，任何"每图元一次 draw call"的写法都是倒退。
3. **视觉效果是 wgpu 的免费午餐**,应充分利用：模糊、渐变、阴影、动画在 shader 里成本极低，这正是阶段 1 毛玻璃设计系统的技术底气。
