# Spec: 丹青 (danqing) — 跨平台自绘 UI 框架

> 状态: **已批准** (2026-07-16,全部 Open Questions 已决)
> 创建: 2026-07-16
> 命名: 2026-07-16 定名 **danqing(丹青)**,crate 名 `danqing`(已确认 crates.io 可用)

## Objective

构建一个 Rust 编写的跨平台自绘 UI 框架 **丹青(danqing)**,作为后续**桌面应用、小工具、小游戏**的统一基础设施。

核心决策(已与作者确认):

- **语言**: Rust (edition 2024, MSRV 1.85)
- **UI 范式**: 保留模式 —— 组件树持久存在,事件回调修改状态,框架负责重绘
- **自绘边界**: 所有控件经自有渲染管线绘制;窗口/事件循环等 OS 适配层使用 `winit`,不手写 Win32/Cocoa/X11
- **渲染深度**: 基于 `wgpu` GPU 抽象(自动选择 D3D12/Vulkan/Metal 后端),不做软光栅、不直接写裸图形 API
- **文本**: 字体解析/栅格化依赖成熟库,不从零解析 TTF

**首个里程碑(M1 · 最小闭环)**: 跨平台开窗 + 基础图元/文本绘制 + 键鼠事件响应 + 一个 showcase 演示页。M1 的目标是打通"从 winit 事件到 wgpu 像素"的完整链路,组件丰富度后置。

## Tech Stack

| 职责 | 选型 | 说明 |
|---|---|---|
| 窗口/事件循环 | `winit` 0.30.13 | Rust 生态事实标准 |
| GPU 抽象 | `wgpu` 30 | D3D12/Vulkan/Metal 自动选择;已验证与 winit 0.30 兼容(raw-window-handle 0.6) |
| 字形栅格化 | `fontdue` 0.9 | 纯 Rust、API 简单;CJK 无需复杂整形,按字排版即可 |
| 字形图集 | `etagere` 0.3 | 缓存栅格化字形到 GPU 纹理 |
| 系统字体查找 | `font-kit` 0.14 | 运行时加载系统中文字体(如微软雅黑);配内嵌 OFL 回退字体 |
| 辅助 | `bytemuck` `pollster` `log` `env_logger` `anyhow` `thiserror` | 常规基础设施 |

**演进点(M1 不做,架构上预留):**

- `lyon` —— 任意矢量路径(曲线/复杂图形),M1 只支持矩形族
- `cosmic-text` —— 复杂整形、双向文本、富文本排版
- `taffy` —— 完整 flexbox 布局;M1 用自写的 Column/Row/Padding/Center
- 多窗口、DPI 缩放

## Commands

```bash
cargo build                    # 构建
cargo run --example showcase   # 运行演示页(M1 的主要验证方式)
cargo test                     # 全部测试
cargo clippy -- -D warnings    # 静态检查,要求零警告
cargo fmt                      # 格式化
cargo build --release          # 发布构建
```

## Project Structure

```
Cargo.toml           → 包定义(单 crate,后续按需拆 workspace)
src/lib.rs           → 库入口,显式 re-export 公开 API
src/app.rs           → App trait、框架入口 run()
src/window.rs        → winit 窗口与事件循环封装(平台适配层)
src/event.rs         → 事件类型(鼠标/键盘)与分发
src/render/mod.rs    → wgpu 上下文(设备/队列/surface)
src/render/rect.rs   → 矩形族渲染管线(SDF 圆角,实例化 quad)
src/render/text.rs   → 文本渲染管线(图集采样)
src/text/font.rs     → 字体加载(font-kit 找系统字体 + 内嵌回退)
src/text/atlas.rs    → 字形图集(shelf-packer)
src/layout.rs        → 布局:约束传递与尺寸计算
src/widget/mod.rs    → Widget trait、组件树(Node)
src/widget/*.rs      → 内建组件:Box/Text/Column/Row/Padding/Center/Button
examples/showcase.rs → M1 演示页
tests/               → 集成测试(布局/事件等纯逻辑,不需要 GPU)
docs/specs/spec.md         → 本规格
```

约束:`window.rs` 与 `render/` 以下可以碰平台/图形 API;`widget/` `layout.rs` 必须是纯逻辑。

## Code Style

- `rustfmt` 默认配置;`cargo clippy -- -D warnings` 零警告
- 公共 API 写中文文档注释,命名遵循 Rust 英文惯例
- 错误处理:库代码用 `thiserror` 定义错误类型,example 用 `anyhow`
- 模块小而专;公开 API 一律经 `lib.rs` re-export,不允许用户路径深穿

示例(目标风格,非最终 API):

```rust
/// 文本组件。
///
/// 显示一段单行文本,字号与颜色可在构建时指定。
pub struct Text {
    content: String,
    font_size: f32,
    color: Color,
}

impl Text {
    /// 创建文本组件,默认字号 16.0、颜色为不透明黑色。
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font_size: 16.0,
            color: Color::BLACK,
        }
    }

    /// 设置字号(逻辑像素)。
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}
```

## Testing Strategy

- **单元测试**(模块内 `#[cfg(test)]`):布局计算、事件命中分发、字形图集分配 —— 全部为纯逻辑,CI/本地无需 GPU
- **集成测试**(`tests/`):组件树构建 + 布局 + 模拟事件分发的端到端逻辑
- **渲染验证**:M1 阶段靠 `cargo run --example showcase` 人工确认。wgpu 校验层默认关闭以避免启动/关闭延迟;如需启用,设置 `DANQING_WGPU_VALIDATION=1` 或 `WGPU_VALIDATION=1`,并确保无校验错误
- **覆盖率**:M1 不设硬指标,但布局/事件/图集三个纯逻辑模块必须有测试
- GPU 相关代码通过 trait 隔离,避免纯逻辑测试依赖设备

## Boundaries

**Always(每次都要做):**
- 提交前 `cargo fmt` + `cargo clippy -- -D warnings` + `cargo test` 全绿
- 新公共类型/函数带中文文档注释
- 平台相关代码只允许出现在适配层(`window.rs` / `render/`)
- 新增组件必须出现在 showcase 中(以用代测)

**Ask first(先问再做):**
- 新增外部依赖
- 修改已稳定的公开 API
- 引入 `unsafe`
- 改动渲染管线的架构(如更换图集策略、引入新管线)

**Never(绝不做):**
- 提交字体等二进制大文件进仓库(字体走系统加载或用户自带路径)
- 为通过测试而删除/跳过失败测试
- 在 `widget/` `layout.rs` 中写平台特定代码
- 提交密钥、凭据

## Success Criteria(M1 验收标准)

| # | 条件 | 验证方式 |
|---|---|---|
| 1 | `cargo run --example showcase` 在 Windows 打开窗口,简单场景稳定 ~60 FPS(vsync) | 人工运行 |
| 2 | showcase 展示:彩色矩形 + 圆角矩形(边缘抗锯齿)、**中文与英文文本**、可交互按钮(hover 变色、点击计数)、键盘响应区(按键移动方块) | 人工运行 |
| 3 | 鼠标移动/按下/抬起/滚轮事件经命中测试正确分发;键盘字符与功能键正确接收 | 人工运行 + 事件分发单元测试 |
| 4 | 关闭窗口干净退出:无 panic;默认 wgpu 校验层关闭,如启用则校验层零错误 | 人工运行 |
| 5 | `cargo test` 全绿;`cargo clippy -- -D warnings` 通过 | 命令验证 |
| 6 | 适配层之外无 Windows 专有 API(winit/wgpu 天然跨平台),结构上为 macOS/Linux 就绪 | 代码评审 |

## Open Questions(全部已决,2026-07-16)

1. ~~crate 名称~~ ✅ **danqing(丹青)**,crates.io 裸名可用
2. ~~中文字体策略~~ ✅ **方案 C**:`font-kit` 系统字体为主 + 内嵌开源字体兜底
3. ~~键盘焦点~~ ✅ M1 无焦点系统,键盘事件直送应用层;焦点系统留待 M2
4. ~~Rust 工具链~~ ✅ 用户自行通过 rustup 安装

## 前置条件(阻塞 M1)

- [ ] 安装 Rust 工具链(rustup + stable):`winget install Rustlang.Rustup` 或 https://rustup.rs
