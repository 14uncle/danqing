# TODO: 应用内版本检查核心

> plan: `tasks/plan-update-check.md` | spec: `docs/specs/SPEC-update-check.md`
> feature 门控(`update`, 默认关)。产品启动检查逻辑不动: 后台线程 + 24h TTL + 失败静默。

- [ ] **U1: Cargo 依赖与 feature**
  - Acceptance: `[features] default=[]; update=["dep:ureq","dep:serde","dep:serde_json","dep:dirs","dep:open"]`;
    optional deps; 默认构建不编译它们
  - Verify: `cargo check --no-default-features` + `cargo build --features update`
  - Files: `Cargo.toml`

- [ ] **U2: 移植纯逻辑模块**
  - Acceptance: `src/update.rs` 含 parse_version/is_newer/UpdateStatus/CheckCache(is_fresh+checked_version)/
    usable_cache/UpdateHint/publish/current_hint/缓存读写
  - Verify: `cargo test --features update`(移植 pomodoro 单测全绿)
  - Files: `src/update.rs`

- [ ] **U3: GitHub 轨运输 + UpdateSpec**
  - Acceptance: `UpdateSpec`; `fetch_update_status` GET releases/latest 取 tag_name(带 User-Agent);
    `spawn_check`/`update_action_text`(前往下载)/`perform_action`(open); 失败→None 静默
  - Verify: `cargo test --features update`(mock 不触网)
  - Files: `src/update.rs`

- [ ] **U4: re-export 门控**
  - Acceptance: `lib.rs` `#[cfg(feature="update")] pub mod update;`; 开→可用, 关→无模块
  - Verify: `cargo build --features update` + `cargo check --no-default-features`
  - Files: `src/lib.rs`

- [ ] **U5: 无网络栈验证 + 三件套**
  - Acceptance: 默认构建无 ureq; fmt/clippy/test 绿(默认 + feature 各跑)
  - Verify: `cargo check --no-default-features` + `cargo fmt` + `cargo clippy --features update -- -D warnings`
  - Files: `Cargo.toml`, `src/update.rs`

- [ ] **Checkpoint: 模块验收**
  - [ ] feature 开闭可编; 纯逻辑单测绿; 默认无网络栈; 进 review
