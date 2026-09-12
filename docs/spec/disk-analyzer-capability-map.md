# Capability Map: 磁盘空间分析器

> @author 十四叔
> @date 2026/09/11
> 状态: **已批准**

| Module id | Responsibility | Depends on | Spec |
|---|---|---|---|
| `scanner` | MFT 直读 + 标准遍历双引擎，文件树构建 | — | [SPEC-scanner.md](SPEC-scanner.md) |
| `visualizer` | Treemap + 饼图 + 扩展名分布渲染 (wgpu) | scanner | [SPEC-visualizer.md](SPEC-visualizer.md) |
| `analyzer` | SSD 悬崖警告 + 开发者缓存识别 + 云文件占位符检测 | scanner | [SPEC-analyzer.md](SPEC-analyzer.md) |
| `ui` | 主题/窗口/交互/导出 CSV-JSON | visualizer, analyzer | [SPEC-ui.md](SPEC-ui.md) |
| `pro` | 历史趋势 + 重复文件检测 + WSL2 VHDX + CLI | scanner, analyzer | [SPEC-pro.md](SPEC-pro.md) |

## Build Order

```
scanner → visualizer, analyzer（并行）→ ui → pro
```

## 定价与分层

| 版本 | 包含模块 | 价格 |
|------|----------|------|
| 免费版 | scanner + visualizer + analyzer + ui | $0 |
| Pro 版 | 全部 | ¥19 / $2.99 买断 |
