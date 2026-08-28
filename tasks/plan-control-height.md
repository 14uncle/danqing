# Plan: 统一控件高度 Token

> Spec: `docs/specs/spec-control-height-token.md`  
> 日期: 2026-08-27

## 依赖图

```
Theme (token) ──┬──→ TextInput ──→ IconInput
                └──→ Button
```

## 垂直切片

每个 task 是一条完整路径：从 token 定义到组件使用到测试验证。

### Phase 1: Token 定义

1. **T1** — `Theme` trait 新增 `control_height` + `LightTheme` 实现 + 测试

### Phase 2: 组件改造 (可并行)

2. **T2** — TextInput 使用 `control_height` + 测试更新
3. **T3** — Button 使用 `control_height` + 测试更新

### Phase 3: 复合适配

4. **T4** — IconInput 适配 + 测试更新

### Phase 4: 集成验证

5. **T5** — showcase 视觉验证 + 全量测试 + clippy

## 检查点

- **CP1** (T1 后): `cargo test theme` 通过，`control_height()` 返回 32.0
- **CP2** (T2+T3 后): 组件单元测试通过，layout 输出高度 == 32
- **CP3** (T4 后): IconInput 测试通过
- **CP4** (T5 后): `cargo test --lib --tests` + `cargo clippy -- -D warnings` + `cargo fmt --check` 全绿
