# Plan: 应用内版本检查核心 (update-check)

> spec: `docs/specs/SPEC-update-check.md`; 框架级, feature 门控, 默认关。
> 移植 danqing-pomodoro `src/update.rs` 纯逻辑 + GitHub 轨运输, 去 store/授权。

## Overview

收 `danqing::update` 进框架: 版本对比/24h TTL 缓存+换版作废/后台线程检查/`current_hint` 角标 +
GitHub Releases 输入。产品只供 `UpdateSpec`(repo/user_agent/releases_page/当前版本) 与设置 UI 行。
默认不启用(不拉 `ureq/serde/dirs` 等网络栈)。

## Architecture Decisions

1. **feature 门控**: `Cargo.toml` 加 `[features] default = []`, `update = ["dep:ureq","dep:serde",
   "dep:serde_json","dep:dirs","dep:open"]`; 核心模块 `#[cfg(feature="update")]`。默认构建零新依赖。
2. **产品注入口**: `UpdateSpec` 结构(repo/user_agent/releases_page/current_version), 框架据此
   GitHub GET + 提示文案; 版本来源由产品给(`CARGO_PKG_VERSION` 或 MSIX)。
3. **纯逻辑零 UI/网络依赖**: parse/is_newer/cache/hint 纯函数可单测; 只有 GitHub 运输接触网络。
4. **store/MSIX 不进来**: 授权/IAP/store 触发是产品政策, 明确不收(留产品)。

## Task List

### U1: Cargo 依赖与 feature
- **Acceptance**: `Cargo.toml` 加 `[features]`(default=[]; update=可选依赖); 新增 optional
  `ureq/serde/serde_json/dirs/open`。默认构建不编译它们。
- **Verify**: `cargo check --no-default-features`(无新依赖报错) + `cargo build --features update`。
- **Files**: `Cargo.toml` | S

### U2: 移植纯逻辑模块
- **Acceptance**: `src/update.rs` 含 `VersionTriple/parse_version/is_newer/UpdateStatus`
  (UpToDate/KnownVersion/UnknownVersion) `CheckCache`(is_fresh TTL + checked_version)
  `usable_cache`(换版作废) `UpdateHint` `publish/current_hint` 缓存读写(`dirs` 配置路径)。
  沿用 pomodoro 已测行为。
- **Verify**: `cargo test --features update` — 移植 pomodoro update 单测全绿。
- **Files**: `src/update.rs` | M

### U3: GitHub 轨运输 + UpdateSpec
- **Acceptance**: `UpdateSpec` 结构; `fetch_update_status` GET `releases/latest` 取 `tag_name`(带
  User-Agent); `spawn_check`(读缓存→fresh 则止→后台线程 refetch→写缓存+发布); `update_action_text`
  (GitHub=前往下载) `perform_action`(open releases_page)。网络/解析失败→None 静默。
- **Verify**: `cargo test --features update` — mock URL/字符串断言不真实触网; 失败路径 None。
- **Files**: `src/update.rs` | M

### U4: re-export 门控
- **Acceptance**: `src/lib.rs` `#[cfg(feature="update")] pub mod update;`(或 re-export 关键类型)。
  feature 开 → `danqing::update::current_hint` 可用; 关 → 无此模块。
- **Verify**: `cargo build --features update` + `cargo check --no-default-features`。
- **Files**: `src/lib.rs` | S

### U5: 无网络栈验证 + 三件套
- **Acceptance**: `cargo tree`/`cargo check --no-default-features` 确认默认无 `ureq`; fmt/clippy/test 绿。
- **Verify**: `cargo fmt` + `cargo clippy --features update -- -D warnings` + 默认/feature 各跑 test。
- **Files**: `Cargo.toml`, `src/update.rs` | S

### Checkpoint: 模块验收 (U5 后)
- [ ] feature 开闭均可编; 纯逻辑单测全绿; 默认无网络栈; 进 review。

## Risks and Mitigations
| 风险 | 影响 | 缓解 |
|---|---|---|
| 默认构建误引网络依赖 | 中 | `cargo check --no-default-features` 锁; optional 依赖 |
| 缓存路径与 pomodoro 冲突 | 低 | 沿用 `%APPDATA%/danqing/update-check.json`(产品少无冲突); 冲突再按产品分词 |
| ureq 网络行为不稳定 | 低 | 失败→None 静默; 后台线程不阻塞 UI |

## Open Questions
- `open::that` 由框架 `update` 引还是产品引 → 暂框架引 `open`, plan 定(U3 用框架 `open`)。
- 缓存路径统一 `update-check.json` 还是按产品 → 暂统一, 冲突再分词(U2 用统一路径)。
