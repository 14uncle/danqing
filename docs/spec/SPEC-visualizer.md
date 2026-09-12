# Spec: visualizer — Treemap + 饼图 + 扩展名分布可视化

> 模块 id: `visualizer`
> 依赖: `scanner`
> @author 十四叔
> @date 2026/09/11
> 状态: **已批准**

## Objective

将 scanner 产出的 FileTree 渲染为交互式可视化，帮助用户直观理解磁盘空间分布。

**用户故事**：
- 作为用户，我希望看到整个磁盘的 treemap 一览，这样我能一眼发现大文件夹
- 作为用户，我希望点击 treemap 区块下钻到子目录，这样我能层层定位问题
- 作为用户，我希望看到文件扩展名分布（饼图），这样我知道哪类文件占空间最多
- 作为用户，我希望深色/浅色主题切换，这样夜间使用不刺眼

## Tech Stack

- **渲染**: wgpu（danqing 框架复用，实例化渲染 treemap 矩形）
- **Treemap 算法**: Squarified Treemap（Bruls et al. 2000，最大化长宽比）
- **文本渲染**: danqing 文本管线（wgpu glyph）
- **交互**: 鼠标悬停高亮 + 点击下钻 + 右键返回上级

## Commands

```bash
# 构建
cargo build --release

# 测试
cargo test -p danqing-disk-visualizer
```

## Project Structure

```
danqing-disk-visualizer/
├── src/
│   ├── lib.rs              # 公开 API：Visualizer, TreemapData, PieData
│   ├── treemap/
│   │   ├── mod.rs          # treemap 布局算法
│   │   ├── layout.rs       # Squarified 算法实现
│   │   └── render.rs       # wgpu 实例化渲染
│   ├── pie/
│   │   ├── mod.rs          # 饼图布局
│   │   └── render.rs       # wgpu 扇形渲染
│   ├── ext_dist.rs         # 扩展名分布统计
│   ├── interaction.rs      # 鼠标命中检测 + 下钻状态机
│   └── theme.rs            # 深色/浅色配色方案
├── tests/
│   ├── layout_unit.rs      # treemap 布局单元测试
│   └── ext_dist_unit.rs
```

## Code Style

```rust
/// Treemap 布局结果：每个节点的屏幕坐标和尺寸
#[derive(Debug, Clone)]
pub struct TreemapRect {
    /// 对应 FileTree 中的路径
    pub path: PathBuf,
    /// 屏幕坐标 x, y, width, height（像素）
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// 该节点占父节点的比例（用于颜色饱和度）
    pub ratio: f32,
    /// 显示标签（文件夹名或文件名）
    pub label: String,
}
```

## Testing Strategy

| 测试层级 | 框架 | 位置 | 覆盖要求 |
|----------|------|------|----------|
| 单元测试 | `#[test]` | `src/` | Squarified 布局算法 100%；扩展名统计 100% |
| 视觉验收 | 手动 | 无 | 100万节点 treemap 流畅交互（60fps）|

## Boundaries

- **Always do**: treemap 矩形无重叠无空隙；深色主题是默认主题
- **Ask first**: treemap 配色方案（按扩展名类别 vs 按目录层级 vs 按大小梯度）
- **Never do**: treemap 矩形面积与文件大小不成比例

## Success Criteria

1. Squarified Treemap 布局正确：所有矩形无重叠、无空隙、面积与大小成正比
2. 100万文件节点渲染帧率 ≥ 60fps（本机 Iris Xe 核显）
3. 点击下钻延迟 < 100ms
4. 扩展名饼图正确聚合（Top 20 + "其他"）
5. 深色/浅色主题切换即时生效

## Open Questions

1. treemap 区块颜色映射策略？建议：按扩展名类别（图片=蓝、视频=紫、代码=绿、压缩=橙）
2. 是否需要文件名标签自动缩略（区块太小时隐藏）？建议：是
3. 是否需要 hover tooltip 显示完整路径+大小+修改时间？建议：是
