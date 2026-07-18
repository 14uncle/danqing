# 丹青 (danqing)

一个 Rust 编写的跨平台自绘 UI 框架。M1 里程碑已打通 **winit 事件 → 保留模式组件树 → wgpu 像素** 的完整链路。

## 技术栈

| 职责 | 选型 | 说明 |
|---|---|---|
| 窗口/事件循环 | `winit` 0.30 | 跨平台窗口与事件抽象 |
| GPU 渲染 | `wgpu` 30 | D3D12/Vulkan/Metal 自动选择 |
| 字形栅格化 | `fontdue` 0.9 | 纯 Rust,按字排版 |
| 字形图集 | `etagere` 0.3 | shelf-packer 缓存字形位图 |
| 系统字体 | `font-kit` 0.14 | 加载系统中文字体,失败回退 OFL 内嵌字体 |

## 运行

```bash
cargo run --example showcase   # 打开 M1 演示页
cargo test                     # 全部测试(无需 GPU)
cargo clippy -- -D warnings    # 静态检查
cargo fmt                      # 格式化
cargo build --release          # 发布构建
```

> 首次构建会从 jsdelivr 下载 OFL 回退字体(ZCOOL XiaoWei),仓库内不提交字体二进制。

## 架构

```
examples/showcase.rs   M1 演示页(唯一持续生长的示例)
src/
  lib.rs              公开 API 统一 re-export
  app.rs              App trait + run_app() 入口
  window.rs           winit 平台适配层(唯一接触 OS 窗口 API 的地方)
  event.rs            平台无关事件类型
  layout.rs           值类型 + 约束传递 + 布局算法
  render/             wgpu 渲染管线(矩形 SDF、文本图集)
  text/               字体加载 + 字形图集(纯 CPU)
  widget/             保留模式组件:Box/Text/Button/Column/Row/Padding/Center
```

依赖方向只允许向下:`widget/`、`layout.rs`、`event.rs` 为纯逻辑,不依赖 `winit`/`wgpu`。

## M1 已交付

- 跨平台开窗与持续渲染(vsync `Fifo`)
- SDF 圆角矩形 + 抗锯齿
- 中英文文本渲染(系统字体优先,内嵌 OFL 兜底)
- 保留模式组件树:Column/Row/Padding/Center/Box/Text/Button
- 鼠标命中分发(hover/pressed 状态)与按钮计数
- 键盘事件直达应用层:方向键/WASD 移动方块,字符键回显
- 布局/事件/图集纯逻辑单元测试

## M2 已交付

- 全局焦点管理:Tab/Shift+Tab 遍历、鼠标点击聚焦、焦点环视觉反馈
- 单行 `TextInput`:光标、选区、键盘编辑、IME 合成、剪贴板复制/剪切/粘贴
- `Button` 支持焦点与空格/回车触发

## 许可证

MIT OR Apache-2.0
