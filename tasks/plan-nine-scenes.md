# 实施计划：沉浸世界九场景

## 依赖图

```
[1] 场景定义 (scenes.rs)
    ↓
[2] Uniform 字段 (background.rs + background.wgsl) — 可能不需要新增
    ↓
[3] Shader 动效 (background.wgsl) — 4个新场景
    ↓
[4] 动效策略 (motion.rs) — 4组新函数
    ↓
[5] 主程序接入 (main.rs) — 强度连线
    ↓
[6] 环境音生成 (export-ambient.py) — 4个新音色
    ↓
[7] 音频映射 (ambient.rs) — SCENE_AUDIO 扩展
    ↓
[8] 场景图导出 (export-scenes.py) — 4个新配置
    ↓
[9] 测试验证
```

## 并行机会

- [1] 场景定义 和 [6] 环境音生成 可并行（互不依赖）
- [3] Shader 动效 和 [4] 动效策略 可并行（独立文件）
- [8] 场景图导出 等用户生成 AI 图后执行

## 风险

1. **Uniform buffer 容量**：当前16×f32=64B 已满。如果新场景需要额外 uniform 字段，需要扩展 buffer 或复用现有字段。→ 策略：通过 from/to 淡化机制复用，不新增字段。
2. **音频辨识度**：程序化生成的环境音可能区分度不够。→ 策略：每个场景用独特的 tonal elements（金属共鸣/混响/人声/节奏），不只靠 noise shaping。
3. **Shader 复杂度**：4个新场景的动效可能互相干扰。→ 策略：每个场景的 shader 函数独立，通过 intensity 字段控制开关。

## 验证检查点

- 完成 [1]+[2] 后：`cargo test --lib` 通过
- 完成 [3]+[4]+[5] 后：`cargo clippy` 零警告，showcase 可运行
- 完成 [6]+[7]+[8] 后：`cargo run --example pomodoro` 9个场景可切换
- 最终：窗口隐藏听音辨识测试
