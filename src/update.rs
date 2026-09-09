//! @author 十四叔
//! @date 2026/09/06
//!
//! 应用内更新检查核心: 纯逻辑 (版本对比/24h TTL 缓存+换版作废/后台检查/每帧角标)
//! 与 GitHub 轨运输 (`releases/latest` 查 tag)。整模块挂 `update` feature (默认关)。
//! 产品经 [`UpdateSpec`] 注入 repo/user_agent/发布页/当前版本 —— 框架不替产品定身份。
//! 只有 GitHub 轨; store/MSIX 轨与授权/IAP 是产品政策, 明确留在产品。
//! 约定: 任何一步解析/读写/网络失败都按「无新版」静默处理, 不打扰用户。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 检查结果缓存新鲜度: 24 小时内不重复发起网络查询。
pub const CACHE_TTL_SECS: u64 = 24 * 60 * 60;
/// 检查请求全局超时 (启动后一次性后台调用, 不阻塞 UI)。
const FETCH_TIMEOUT_SECS: u64 = 10;

/// 产品注入的身份/来源: 框架据此查 GitHub Releases 与生成「前往下载」。
///
/// 字段均 `&'static str`: 仓库/发布页/user_agent 是编译期常量;
/// `current_version` 对 GitHub 轨取 `env!("CARGO_PKG_VERSION")` (编译期静态),
/// 商店轨 (产品自管) 需自行产出 'static 版本串。
#[derive(Debug, Clone, Copy)]
pub struct UpdateSpec {
    /// 仓库 `owner/name`, 如 "14uncle/danqing-log"。
    pub repo: &'static str,
    /// 请求 User-Agent (GitHub API 无 UA 直接 403)。
    pub user_agent: &'static str,
    /// 「前往下载」跳转的发布页。
    pub releases_page: &'static str,
    /// 当前版本的来源 (产品自报)。
    pub current_version: &'static str,
}

/// 版本号三元组 (major, minor, patch)。
pub type VersionTriple = (u64, u64, u64);

/// 解析版本串: 容忍 `v0.2.1` / `0.2.1` 两种写法, 拒绝一切非法格式
/// (段数不对、非数字、预发布后缀如 `0.2.1-beta` —— `/releases/latest` 本就不含预发布)。
pub fn parse_version(s: &str) -> Option<VersionTriple> {
    let s = s.trim();
    let s = s.strip_prefix('v').unwrap_or(s);
    let mut parts = s.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None; // 四段式 (商店包版本) 不在比较域内
    }
    Some((major, minor, patch))
}

/// remote 是否严格比 current 新; 任一端解析失败按「无新版」处理。
pub fn is_newer(current: &str, remote: &str) -> bool {
    match (parse_version(current), parse_version(remote)) {
        (Some(cur), Some(rem)) => rem > cur,
        _ => false,
    }
}

/// 检查结论 (随缓存落盘)。GitHub 轨只用到 `KnownVersion` / `UpToDate`;
/// `UnknownVersion` 保留以兼容商店轨语义 (无从验证版本号时只报「有新版本」)。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum UpdateStatus {
    /// 已是最新。
    UpToDate,
    /// 有新版且已知版本号 (GitHub 轨: releases/latest 的 tag)。
    KnownVersion(String),
    /// 有新版但版本号未知 (商店轨语义; GitHub 轨不会走到)。
    UnknownVersion,
}

/// 检查结果缓存: 落 `%APPDATA%/danqing/update-check-<repo>.json` (按 repo 分词,
/// 避免多产品共用同一缓存文件)。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CheckCache {
    /// 上次检查成功的 wall-clock 秒。
    pub checked_at_secs: u64,
    /// 写入本缓存的二进制版本: 换版 (更新/降级) 后旧缓存作废, 见 [`usable_cache`]。
    pub checked_version: String,
    /// 检查结论。
    pub status: UpdateStatus,
}

impl CheckCache {
    /// 缓存是否在 TTL 内; 时钟回拨 (checked_at 在未来) 按新鲜处理。
    pub fn is_fresh(&self, now_secs: u64) -> bool {
        now_secs.saturating_sub(self.checked_at_secs) < CACHE_TTL_SECS
    }
}

/// 指定仓库的缓存文件路径 (按 repo 分词, 避免多产品相互覆盖)。
pub fn cache_path(spec: &UpdateSpec) -> Option<PathBuf> {
    let slug = spec.repo.replace('/', "-");
    dirs::config_dir().map(|p| p.join("danqing").join(format!("update-check-{slug}.json")))
}

/// 从指定路径读缓存: 文件缺失/损坏/解析失败一律 None (交给下次重新检查)。
pub fn load_cache_from(path: &Path) -> Option<CheckCache> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// 写缓存到指定路径 (先建父目录); 失败由调用方降级为 warn 日志。
pub fn save_cache_to(path: &Path, cache: &CheckCache) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(cache).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// 缓存仅对写入它的二进制版本有效: 版本不一致 (更新/降级) 即作废重查。
fn usable_cache(cache: Option<CheckCache>, current: &str) -> Option<CheckCache> {
    cache.filter(|c| c.checked_version == current)
}

/// 「版本」行的更新提示模型: 有新版时给出文案与按钮。
pub struct UpdateHint {
    /// 状态文案 (版本号已归一化为 vX.Y.Z 展示)。
    pub status: String,
    /// 操作按钮文案 (GitHub 轨恒为「前往下载」)。
    pub action: &'static str,
}

/// 更新按钮文案: 框架只做 GitHub 轨, 跳发布页下载。
pub fn update_action_text() -> &'static str {
    "前往下载"
}

/// 全局检查结果 (含发布代次): 后台线程 [`publish`] 写, UI 线程经 [`current_hint`] 每帧读。
static CHECK_CACHE: std::sync::Mutex<(u64, Option<CheckCache>)> = std::sync::Mutex::new((0, None));

/// 代次发行器 (单调递增): spawn_check 入口与无代次 publish 各取一号。
static GEN: AtomicU64 = AtomicU64::new(0);

/// 发布检查结果 (覆盖式; None = 清空)。锁中毒时放弃本次发布 (不 panic)。
/// 无代次发布 (产品直调): 恒取最新号, 必然应用。
pub fn publish(cache: Option<CheckCache>) {
    publish_with_gen(GEN.fetch_add(1, Ordering::Relaxed) + 1, cache);
}

/// 带代次发布: 代次闸门拒绝乱序晚到的旧检查, 不得覆盖新代次结果
/// (盘点簇B 标记的潜在缺口: 裸线程 + 全局缓存无代次防护)。
fn publish_with_gen(generation: u64, cache: Option<CheckCache>) {
    if let Ok(mut guard) = CHECK_CACHE.lock() {
        if accept_gen(guard.0, generation) {
            *guard = (generation, cache);
        }
    }
}

/// 代次闸门: 新代次或同代次 (同轮 spawn_check 的二次发布) 放行; 旧代次 = 乱序晚到, 拒绝。
fn accept_gen(published: u64, incoming: u64) -> bool {
    incoming >= published
}

/// 当前更新提示: 全局缓存 + spec 的当前版本合成, UI 每帧调用。
/// 返回 Some = 设置按钮亮角标; 无缓存/已最新/版本号解析失败 → None (界面零变化)。
pub fn current_hint(spec: &UpdateSpec) -> Option<UpdateHint> {
    update_hint(spec.current_version, CHECK_CACHE.lock().ok()?.1.as_ref())
}

/// 由缓存计算更新提示: 无缓存/已最新/版本追平/解析失败 → None。
pub fn update_hint(current: &str, cache: Option<&CheckCache>) -> Option<UpdateHint> {
    match &cache?.status {
        UpdateStatus::UpToDate => None,
        UpdateStatus::UnknownVersion => Some(UpdateHint {
            status: "有新版本".to_string(),
            action: update_action_text(),
        }),
        UpdateStatus::KnownVersion(latest) => {
            let (major, minor, patch) = parse_version(latest)?;
            if !is_newer(current, latest) {
                return None;
            }
            Some(UpdateHint {
                status: format!("有新版本 v{major}.{minor}.{patch}"),
                action: update_action_text(),
            })
        }
    }
}

/// 当前 wall-clock 秒 (缓存新鲜度基准; 失败回退 0 视为过期)。
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 启动更新检查: 读缓存立即发布 (过期缓存经 [`usable_cache`] 版本闸门), 过期/缺失
/// 才后台线程重查; 成功写缓存并发布, 失败静默 (一行 warn, 本次会话不重试)。
pub fn spawn_check(spec: UpdateSpec) {
    let generation = GEN.fetch_add(1, Ordering::Relaxed) + 1;
    let cached = usable_cache(
        cache_path(&spec).and_then(|p| load_cache_from(&p)),
        spec.current_version,
    );
    let fresh = cached.as_ref().is_some_and(|c| c.is_fresh(now_secs()));
    publish_with_gen(generation, cached);
    if fresh {
        return;
    }
    // 后台线程不 join: 进程退出即终止, 无泄漏。
    std::thread::spawn(move || match fetch_update_status(&spec) {
        Some(status) => {
            let cache = CheckCache {
                checked_at_secs: now_secs(),
                checked_version: spec.current_version.to_string(),
                status,
            };
            match cache_path(&spec) {
                Some(path) => {
                    if let Err(err) = save_cache_to(&path, &cache) {
                        log::warn!("更新检查缓存写入失败: {err}");
                    }
                }
                None => log::warn!("配置目录不可得, 更新检查结果不落盘"),
            }
            publish_with_gen(generation, Some(cache));
        }
        None => log::warn!("更新检查失败, 本次会话不再重试"),
    });
}

/// 执行更新动作 (GitHub 轨: 跳发布页手动下载)。
pub fn perform_action(spec: &UpdateSpec) {
    if let Err(err) = open::that(spec.releases_page) {
        log::warn!("打开发布页失败: {err}");
    }
}

/// GitHub 轨: 查 releases/latest 的 tag_name; 网络/解析任何失败返回 None (静默)。
fn fetch_update_status(spec: &UpdateSpec) -> Option<UpdateStatus> {
    #[derive(serde::Deserialize)]
    struct Release {
        tag_name: String,
    }
    let api = format!("https://api.github.com/repos/{}/releases/latest", spec.repo);
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(FETCH_TIMEOUT_SECS)))
        .build();
    // GitHub API 无 User-Agent 直接 403。
    let release: Release = ureq::Agent::new_with_config(config)
        .get(&api)
        .header("User-Agent", spec.user_agent)
        .call()
        .ok()?
        .body_mut()
        .read_json()
        .ok()?;
    Some(UpdateStatus::KnownVersion(release.tag_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_version ---

    #[test]
    fn parse_version_accepts_plain_and_v_prefixed() {
        assert_eq!(parse_version("0.2.1"), Some((0, 2, 1)));
        assert_eq!(parse_version("v0.2.1"), Some((0, 2, 1)));
        assert_eq!(parse_version("v10.20.30"), Some((10, 20, 30)));
    }

    #[test]
    fn parse_version_rejects_malformed() {
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("v"), None);
        assert_eq!(parse_version("0.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("0.2.1-beta"), None);
        assert_eq!(parse_version("garbage"), None);
    }

    // --- is_newer ---

    #[test]
    fn is_newer_compares_each_component() {
        assert!(is_newer("0.2.0", "0.2.1")); // patch 新
        assert!(is_newer("0.2.0", "0.3.0")); // minor 新
        assert!(is_newer("0.2.0", "1.0.0")); // major 新
        assert!(is_newer("0.2.0", "v0.2.1")); // 带 v 前缀的 tag
    }

    #[test]
    fn is_newer_rejects_equal_older_and_garbage() {
        assert!(!is_newer("0.2.0", "0.2.0")); // 相等
        assert!(!is_newer("1.0.0", "0.9.9")); // 远端更旧
        assert!(!is_newer("0.2.0", "garbage")); // 远端非法
        assert!(!is_newer("garbage", "0.2.1")); // 本地非法
    }

    // --- 缓存新鲜度 ---

    #[test]
    fn cache_freshness_respects_ttl_boundary() {
        let cache = CheckCache {
            checked_at_secs: 1000,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::UpToDate,
        };
        assert!(cache.is_fresh(1000 + CACHE_TTL_SECS - 1)); // TTL 内
        assert!(!cache.is_fresh(1000 + CACHE_TTL_SECS)); // 恰好到期算过期
        assert!(cache.is_fresh(500)); // 时钟回拨按新鲜处理
    }

    // --- 缓存读写 ---

    fn temp_cache_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "danqing-update-check-test-{}-{tag}.json",
            std::process::id()
        ))
    }

    #[test]
    fn cache_roundtrip_preserves_content() {
        let path = temp_cache_path("roundtrip");
        let cache = CheckCache {
            checked_at_secs: 1_757_000_000,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::KnownVersion("v0.2.1".to_string()),
        };
        save_cache_to(&path, &cache).expect("写缓存");
        assert_eq!(load_cache_from(&path), Some(cache));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_cache_tolerates_missing_and_corrupt() {
        let missing = temp_cache_path("missing");
        assert_eq!(load_cache_from(&missing), None);

        let corrupt = temp_cache_path("corrupt");
        std::fs::write(&corrupt, "{not json").expect("写坏文件");
        assert_eq!(load_cache_from(&corrupt), None);
        let _ = std::fs::remove_file(&corrupt);
    }

    // --- 缓存版本闸门 (换版作废) ---

    #[test]
    fn usable_cache_drops_cache_from_other_binary_version() {
        let cache = CheckCache {
            checked_at_secs: 1,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::UnknownVersion,
        };
        assert_eq!(
            usable_cache(Some(cache.clone()), "0.2.0"),
            Some(cache.clone())
        );
        assert_eq!(usable_cache(Some(cache), "0.2.7"), None);
        assert_eq!(usable_cache(None, "0.2.7"), None);
    }

    #[test]
    fn load_cache_tolerates_legacy_format_without_version() {
        // checked_version 引入前的旧格式: 反序列化失败按无缓存处理, 触发重查, 不留尸。
        let legacy = temp_cache_path("legacy");
        std::fs::write(&legacy, r#"{"checked_at_secs":1,"status":"UpToDate"}"#).expect("写旧格式");
        assert_eq!(load_cache_from(&legacy), None);
        let _ = std::fs::remove_file(&legacy);
    }

    // --- 更新提示模型 ---

    #[test]
    fn update_hint_none_when_uptodate_or_not_newer() {
        assert!(update_hint("0.2.0", None).is_none()); // 无缓存
        let up_to_date = CheckCache {
            checked_at_secs: 1,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::UpToDate,
        };
        assert!(update_hint("0.2.0", Some(&up_to_date)).is_none());
        let older = CheckCache {
            checked_at_secs: 1,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::KnownVersion("v0.2.0".to_string()),
        };
        assert!(update_hint("0.2.0", Some(&older)).is_none()); // 版本追平
    }

    #[test]
    fn update_hint_normalizes_known_version_display() {
        let newer = CheckCache {
            checked_at_secs: 1,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::KnownVersion("9.9.9".to_string()),
        };
        let hint = update_hint("0.2.0", Some(&newer)).expect("应有提示");
        assert_eq!(hint.status, "有新版本 v9.9.9");
        assert_eq!(hint.action, update_action_text());
    }

    #[test]
    fn update_hint_unknown_version_omits_version_number() {
        let cache = CheckCache {
            checked_at_secs: 1,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::UnknownVersion,
        };
        let hint = update_hint("0.2.0", Some(&cache)).expect("应有提示");
        assert_eq!(hint.status, "有新版本");
        assert_eq!(hint.action, update_action_text());
    }

    // --- 全局状态 → 提示合成 ---

    #[test]
    fn current_hint_reflects_published_cache() {
        let spec = UpdateSpec {
            repo: "14uncle/danqing-log",
            user_agent: "danqing-log",
            releases_page: "https://github.com/14uncle/danqing-log/releases/latest",
            current_version: "0.2.0",
        };
        publish(Some(CheckCache {
            checked_at_secs: 1,
            checked_version: "0.2.0".to_string(),
            status: UpdateStatus::KnownVersion("v99.0.0".to_string()),
        }));
        assert!(current_hint(&spec).is_some());
        publish(None);
        assert!(current_hint(&spec).is_none());
    }

    #[test]
    fn generation_gate_rejects_stale_publish() {
        // 代次闸门 (簇B 潜在缺口加固): 新代次应用; 同代次二次发布放行
        // (spawn_check 的缓存直发 + 网络回写同号); 旧代次乱序晚到拒绝。
        // 只测纯函数: 全局静态 (CHECK_CACHE/GEN) 的集成测试会与
        // current_hint_reflects_published_cache 并行互踩; 接线仅 3 行, 由评审覆盖。
        assert!(accept_gen(0, 1), "新代次应用");
        assert!(accept_gen(5, 5), "同代次二次发布放行");
        assert!(!accept_gen(5, 4), "旧代次乱序晚到拒绝");
    }
}
