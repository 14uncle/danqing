# Spec: 应用内版本检查核心 (update-check)

> @author 十四叔 · @date 2026/09/06
> 前置: interview-me + 用户裁决「下沉框架」(产品越来越多, 每家一份后端不值得)。
> 只下沉**纯逻辑核心 + GitHub 轨运输**(最通用), 产品保留身份/版本来源/运输/授权/UI。
> 移植自 danqing-pomodoro `src/update.rs`(已有充分单测), 去 store/授权, 收进框架模块 `danqing::update`。

## Objective

多个产品(log/pomodoro/clipboard/未来)都要「有没有更新的版本检查」。每家抄一份会让
`parse_version`/缓存 TTL/换版作废 的正确性各修一遍。本 spec 把这一核心收进框架:

- **纯逻辑**: `VersionTriple` / `parse_version` / `is_newer` / `UpdateStatus` /
  `CheckCache`(24h TTL + `checked_version` 换版作废) / `UpdateHint` /
  全局发布/读取(`publish`/`current_hint`,线程安全, UI 每帧读) / 缓存读写路径(`dirs`)。
- **运输**: GitHub 轨(`releases/latest` 查 `tag_name`, `ureq` GET, 带 User-Agent)。
- **门控**: 整模块挂 `update` feature(**默认关闭**), 不让网络/序列化栈进框架默认依赖。

**产品侧保留(不下沉)**: repo(owner/name)、user_agent、跳转的 releases_page、版本来源
(`CARGO_PKG_VERSION` vs MSIX)、store/MSIX 轨 + IAP/授权、设置 UI 那行。产品经
`danqing = { features = ["update"] }` 开启。

## Tech Stack

- 新增**optional 依赖**(均挂 `update` feature): `ureq`, `serde`, `serde_json`, `dirs`, `open`。
- 纯逻辑不依赖 wgpu/winit —— 不违反「widget/layout/event/text 纯逻辑」分层,
  只是新增一个**纯逻辑应用模块**, 与渲染无关。
- 参考: `danqing-pomodoro/src/update.rs`(现成, 已带 parse/cache/hint 单测)。

## Commands

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test --lib --tests        # update 模块单测 (URL 用 mock/不触网)
cargo check --no-default-features   # 确认未启用 update 时不拉网络栈
```

## Project Structure

```
src/update.rs        ← danqing::update: 纯逻辑 + GitHub 轨运输 (feature-gated, 见下)
src/lib.rs           ← re-export (挂 #[cfg(feature = "update")])
Cargo.toml           ← 新增 [features] update = ["dep:ureq", ...] + optional 依赖
```

### 门控约定

`#[cfg(feature = "update")]` 包住整个 `update` 模块 + lib.rs 的 re-export。
`Cargo.toml` 新增 `[features]` 段:

```toml
[features]
default = []
update = ["dep:ureq", "dep:serde", "dep:serde_json", "dep:dirs", "dep:open"]
```

产品开启:`danqing = { version = "...", features = ["update"] }`(本机 path 联动同理)。

## Code Style

沿用 pomodoro `update.rs` 的中文 doc + 纯逻辑无副作用 + 失败一律「无新版」静默。
产品注入点用一个轻量 `UpdateSpec` 结构(或带默认的配置):

```rust
/// 产品注入的身份/来源: 框架据此查 GitHub Releases 与生成「前往下载」。
pub struct UpdateSpec {
    pub repo: &'static str,        // "14uncle/danqing-log"
    pub user_agent: &'static str,  // 如 "danqing-log"
    pub releases_page: &'static str,
    pub current_version: &'static str, // 或闭包(MSIX 包身份版本, store 轨用)
}
```

## Testing Strategy

- 单测(纯逻辑, 不触网): `parse_version` 合法/非法; `is_newer` 逐分量 + 相等/更旧/垃圾;
  `CheckCache::is_fresh` TTL 边界 + 时钟回拨; 缓存读写往返 + 缺失/损坏容错;
  `usable_cache` 换版作废; `update_hint` UpToDate/not-newer→None、KnownVersion 归一化展示、
  UnknownVersion 不显示版本; `current_hint` 反映 publish。
- 运输(mock): GitHub 轨 `fetch_update_status` 用离线的假 URL/字符串断言(不真实触网);
  网络失败/GitHub 403→None 静默。
- 三件套 + `cargo check --no-default-features`(默认不拉网络栈)。

## Boundaries

- **Always**: 纯逻辑 + GitHub 轨下沉; 失败即「无新版」静默; TTL+换版作废; 默认 feature 关闭。
- **Ask first**: 新增运输后端(如自建 manifest/S3 端点 —— 现只 GitHub); 拉 store/MSIX
  轨进框架(授权/IAP 是产品政策, 现明确**不收**); 自动替换 exe / 自动安装。
- **Never**: 把 store/MSIX 轨、IAP/授权、设置 UI 那行收进框架; 把网络栈设为默认依赖;
  改 `%APPDATA%/danqing/update-check.json` 的既有格式语义(沿用 pomodoro 已定格式)。

## Success Criteria

- [ ] `danqing::update` 纯逻辑 + GitHub 轨在 `update` feature 下编过, 单测全绿(移植 pomodoro 覆盖)。
- [ ] 默认(无 feature)构建**不**拉 `ureq/serde/dirs`(`cargo check --no-default-features` 验证)。
- [ ] 产品侧可经 `UpdateSpec` 注入 repo/来源, `current_hint()` 每帧给 UI 角标, 有新版给「前往下载」。
- [ ] 后台线程检查不阻塞 UI; 网络失败静默 None。

## Open Questions

- `open::that`(打开发布页)是否由框架引, 还是产品自行 `open`? → 暂框架引 `open`(常用), plan 定。
- 缓存路径沿用 pomodoro 的 `%APPDATA%/danqing/update-check.json`, 还是按产品分
  (`{app}/update-check.json`)? → 沿用统一路径(产品少时无冲突), plan 定; 若冲突再分词。
