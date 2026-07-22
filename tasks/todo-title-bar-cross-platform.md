# Todo: 全平台统一自绘标题栏

> 计划详见 `tasks/plan-title-bar-cross-platform.md`,spec 见 `docs/specs/title-bar-cross-platform.md`。

## Phase 1: 组件层 — TitleBar 样式化

- [x] **Task 1**: 按钮角色化重构 + `TitleBarStyle` 骨架（Standard 行为像素级不变）
  - 验收： 现有标题栏测试零修改通过；`.style()` builder 可用；`TitleBarStyle` 平铺导出
- [x] **Task 2**: Theme 红绿灯 token + TrafficLights 布局/绘制/hover 符号
  - 验收： 5 条模块内单测全绿（左侧顺序/命中/符号显隐/绿灯消息/平台默认）；无魔法颜色
- [x] **Task 3**: tests/title_bar_window.rs 红绿灯集成测试
  - 验收： 新增 4 条测试全绿（三按钮动作映射 + 拖拽/双击）

**Checkpoint A**: fmt --check + clippy --all-targets -D warnings + cargo test 全绿 + showcase Windows 冒烟零回归

## Phase 2: 平台层

- [x] **Task 4**: window.rs 移除 `with_decorations(false)` 的 Windows cfg 门控
  - 验收： Windows 冒烟零回归；「其他平台降级」注释清除
- [x] **Task 5**: 跨平台编译检查（`cargo check --target x86_64-unknown-linux-gnu` / `x86_64-apple-darwin`)
  - 结论： **macOS target check 通过**(2026-07-22，含 TrafficLights cfg 分支真实编译）;
    **Linux 降级** —— `yeslogic-fontconfig-sys`(font-kit 传递依赖）的 build.rs 需要
    pkg-config + Linux sysroot，本机（Windows，无 WSL 分发）无法满足；
    丹青自身代码无任何 Linux 专属 cfg 分支，Linux 编译风险集中在第三方依赖，
    留给 CI 或 Linux 真机验证

**Checkpoint B**: 三平台编译结论明确 + fmt + clippy + 全测试绿

## Phase 3: 收尾

- [ ] **Task 6**: 文档同步与最终验收
  - 验收： 旧 spec 标注取代关系；CLAUDE.md/README 同步；无「原生标题栏降级」残留表述；三件套全绿
