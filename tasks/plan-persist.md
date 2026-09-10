# Implementation Plan: `persist` 持久化模块

> Spec: `docs/specs/SPEC-persist.md`。来源: `docs/intent/framework-sinking.md` 簇D。
> 政策门① 裁决: 2026-09-09 用户「开口子」，feature-gated。
> 每任务完成 = 验收条件全勾 + 三件套绿; 按序推进, Checkpoint 处人工过目。

## Overview

五 API 下沉: `config_dir` / `atomic_save` / `load_or_default` / `DirtyFlag` / `VersionedDoc`。
feature gate `persist`，产品 opt-in。依赖全已有 (dirs/serde/serde_json)。
先框架侧纯逻辑，再 pomodoro 迁移验证。

## Architecture Decisions

1. **feature-gated 模块**: `danqing/src/persist.rs` 在 `Cargo.toml` features 下声明，
   默认不开。产品 `features = ["persist"]` opt-in。与 update.rs 同级形态。
2. **时间源注入**: `DirtyFlag` 的节流时钟用闭包注入 (与 anim 模块同策略)，
   产品侧传 `Instant::now`，测试侧传可控时间，无 flaky。
3. **VersionedDoc 版本号独立**: 每个文档类型自带 `CURRENT_VERSION` 常量，
   stats.rs 的 format_version 与 state.rs 的可能不同步。
4. **load_or_default 泛型约束**: `Default + DeserializeOwned`，产品侧按需包装
   更具体错误语义 (如 pomodoro 的损坏文件 rename 备份)。
5. **atomic_save 扩展名策略**: `.tmp` 后缀加在原扩展名后 (如 `pomodoro.json.tmp`)，
   与目标同目录保证 rename 原子。

## Task List

### Phase 1: 框架组件

- [x] **T1: 模块骨架 + feature gate + `config_dir` + `atomic_save` + `load_or_default`** ✅ 2026-09-09
  - 文件: `danqing/src/persist.rs` (新建), `danqing/Cargo.toml`, `danqing/src/lib.rs`
  - 验收: `config_dir("test")` 返回含产品名路径 + 自动创建;
    `atomic_save` 写入+读回一致; `load_or_default` 缺失→默认 / 损坏→默认+不 panic
  - 测试: 5+ 条单测 (config_dir 路径 / atomic_save 读回 / load_or_default 两分支 / atomic_save 目录不存在时自动创建)
  - 估时: S
  - 依赖: 无

- [x] **T2: `DirtyFlag<T>` 脏标记+节流** ✅ 2026-09-09
  - 文件: `danqing/src/persist.rs` (追加)
  - 验收: mark_dirty → tick 未到阈值不 flush → tick 到阈值 flush;
    写失败保留 dirty + 更新 last_flush; flush 强制写; is_dirty 正确
  - 测试: 5+ 条单测 (节流阈值 / 强制 flush / 写失败保留 dirty / 不脏不写 / 时间注入)
  - 估时: S
  - 依赖: T1

- [x] **T3: `VersionedDoc<T>` 版本化文档** ✅ 2026-09-09
  - 文件: `danqing/src/persist.rs` (追加)
  - 验收: 正常版本读写; 未来版本 → 清空 + refuse_overwrite;
    refuse_overwrite 时 save 跳过 + warn; 损坏 → 默认值
  - 测试: 5+ 条单测 (正常读写 / 未来版本拒写 / 损坏回退 / 版本匹配 / is_future_version)
  - 估时: S
  - 依赖: T1

### Checkpoint 1: 框架侧完成

- [x] 三件套绿; 14 条单测全锁定 (14 tests, 0 failed) ✅ 2026-09-09
- [x] review (全模块) — 通过 ✅ 2026-09-09
- [x] 用户过目 — "批准" ✅ 2026-09-09

### Phase 2: 产品迁移

- [x] **T4: pomodoro state.rs 迁移** — `save_to_path` / `load_from_path` 改用 `atomic_save` / `load_or_default` ✅ 2026-09-09 (pomodoro `ef7a199`)
  - 文件: `danqing-pomodoro/src/state.rs` | M
  - 验收: 既有测试零改动全绿 (行为保持判据)
  - 估时: S
  - 依赖: T1, danqing push

- [x] **T5: pomodoro stats.rs 迁移** — `FocusHistory` 的版本保护改用 `VersionedDoc` ✅ 2026-09-09 (pomodoro `6ccebe4`/`60057f8`)
  - 文件: `danqing-pomodoro/src/stats.rs` | M
  - 验收: 既有 22+ 测试零改动全绿
  - 估时: M
  - 依赖: T3, danqing push

- [x] **T6: pomodoro main.rs 脏标记迁移** — 跳过 (DirtyFlag 时间源与测试耦合不兼容)
  - 文件: `danqing-pomodoro/src/main.rs` | M
  - 原因: DirtyFlag 用 Instant(墙钟), pomodoro 用 Duration(动画时钟); 20+ 测试直接赋值 last_save_at; 双脏标记需两个 DirtyFlag 实例
  - 结论: 保留手写逻辑, DirtyFlag 留给新产品使用
  - 估时: S-M → N/A
  - 依赖: T2, danqing push

### Checkpoint 2: 全量验收

- [x] 各仓三件套绿; 分仓分别提交, message 注明关联 ✅ 2026-09-09
- [ ] 人工过目 (pomodoro 启动/退出/持久化行为不变)
- [ ] 进 review 阶段

## Risks and Mitigations

| 风险 | 影响 | 缓解 |
|------|------|------|
| atomic_save rename 跨文件系统失败 | 低 | .tmp 与目标同目录，rename 原子 |
| DirtyFlag 节流间隔与产品 tick 频率不匹配 | 低 | 产品注入时间源 + 可配 throttle |
| VersionedDoc 与现有 format_version 不兼容 | 中 | pomodoro stats.rs 先行验证，版本号对齐 |
| feature gate 泄漏 (未开 persist 时编译报错) | 低 | CI 两条: 无 feature + 有 feature |

## 开放问题 (plan 已答)

- DirtyFlag 时间源 → 闭包注入 (Instant::now 产品侧 / 可控时间测试侧)
- VersionedDoc 版本号 → 每文档类型独立 (不全局)
