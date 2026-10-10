//! @author 十四叔
//! @date 2026/09/10
//!
//! 时间工具: 相对时间、历法换算、时区偏移。
//!
//! 下沉自 danqing-clipboard `history_list.rs:109-172` (2026-09-10, 小件包 S2)。
//! `local_tz_offset_seconds` 改走 chrono (已依赖), 跨平台零 FFI; 原版走
//! `windows` 0.62 `GetTimeZoneInformation`, chrono 在 Windows 内部调同系 API, 实测一致。

/// 相对时间戳五档: 刚刚 / N 分钟前 / HH:MM (今天) / 昨天 / MM-DD (更早)。
///
/// 纯函数: 时钟与时区由调用方注入 (`tz_offset` 为本地相对 UTC 的秒数, 含 DST)。
pub fn relative_time(last_seen: i64, now: i64, tz_offset: i64) -> String {
    let diff = now - last_seen;
    if diff < 60 {
        return "刚刚".to_string(); // 含时钟回拨 (负值) 兜底
    }
    if diff < 3600 {
        return format!("{} 分钟前", diff / 60);
    }
    let local_seen = last_seen + tz_offset;
    let day_seen = local_seen.div_euclid(86_400);
    let day_now = (now + tz_offset).div_euclid(86_400);
    match day_now - day_seen {
        0 => {
            let t = local_seen.rem_euclid(86_400);
            format!("{:02}:{:02}", t / 3600, (t % 3600) / 60)
        }
        1 => "昨天".to_string(),
        _ => {
            let (_, m, d) = civil_from_days(day_seen);
            format!("{m:02}-{d:02}")
        }
    }
}

/// days since 1970-01-01 → (year, month, day)。
///
/// Howard Hinnant 历法换算, 纯整数, 无浮点。
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// 本地时区相对 UTC 的当前偏移秒 (含 DST)。
///
/// 走 `chrono::Local::now().offset()` —— chrono 在 Windows 内部调
/// `GetTimeZoneInformation`, 与原版 clipboard FFI 实测一致; Linux/macOS 天然可用。
pub fn local_tz_offset_seconds() -> i64 {
    chrono::Local::now().offset().local_minus_utc() as i64
}

/// 当前 Unix epoch 毫秒 (取不到系统时钟 = 0, 溢出 = 0, 不 panic)。
///
/// 下沉自 danqing-luoban `app.rs::epoch_millis` (2026-10-10, 复盘小件包)。
pub fn epoch_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().try_into().unwrap_or(0))
        .unwrap_or(0)
}

/// [`SystemTime`] → Unix epoch 毫秒 (早于 1970 = 0; 超出 `i64` = `i64::MAX`; 均饱和不 panic)。
///
/// 下沉自 danqing-dropshelf `store.rs::time_to_ms` (2026-10-10, 复盘小件包)。
pub fn system_time_to_millis(t: std::time::SystemTime) -> i64 {
    t.duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

/// Unix epoch 毫秒 → [`SystemTime`] (负值按 0; 溢出饱和到 epoch —— 不 panic)。
///
/// `UNIX_EPOCH + Duration` 的溢出在 `panic = "abort"` 的 release 下是**启动即崩**,
/// 故全程走 `checked_add` 兜底 (下沉自 danqing-dropshelf `store.rs::ms_to_time`)。
pub fn millis_to_system_time(ms: i64) -> std::time::SystemTime {
    std::time::SystemTime::UNIX_EPOCH
        .checked_add(std::time::Duration::from_millis(ms.max(0) as u64))
        .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CST: i64 = 8 * 3600;

    #[test]
    fn just_now_under_60s() {
        assert_eq!(relative_time(950, 1000, 0), "刚刚");
        assert_eq!(relative_time(1000, 1000, 0), "刚刚");
    }

    #[test]
    fn clock_skew_falls_back_to_just_now() {
        assert_eq!(
            relative_time(2000, 1000, 0),
            "刚刚",
            "未来时间不Panic不显负"
        );
    }

    #[test]
    fn minutes_ago_under_1h() {
        assert_eq!(relative_time(940, 1000, 0), "1 分钟前");
        assert_eq!(relative_time(1000 - 59 * 60, 1000, 0), "59 分钟前");
    }

    #[test]
    fn same_day_shows_hh_mm() {
        assert_eq!(relative_time(9 * 3600 + 30 * 60, 18 * 3600, 0), "09:30");
    }

    #[test]
    fn yesterday_when_prev_day() {
        assert_eq!(relative_time(23 * 3600, 86_400 + 3600, 0), "昨天");
    }

    #[test]
    fn older_shows_mm_dd() {
        let seen = 1_000_000_000;
        assert_eq!(relative_time(seen, seen + 5 * 86_400, CST), "09-09");
    }

    #[test]
    fn civil_anchor_leap() {
        assert_eq!(civil_from_days(11_574), (2001, 9, 9));
        assert_eq!(civil_from_days(11_016), (2000, 2, 29));
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn tz_offset_shifts_day_boundary() {
        let seen = 23 * 3600 + 30 * 60;
        let now = seen + 3600;
        assert_eq!(relative_time(seen, now, 0), "昨天");
        assert_eq!(relative_time(seen, now, CST), "07:30");
    }

    // ── epoch 毫秒互转 (小件包) ──

    #[test]
    fn epoch_millis_is_a_plausible_wall_clock() {
        // 真实值随机器时钟走, 只断言量级 (晚于 2020-01-01)。
        assert!(epoch_millis() > 1_577_836_800_000, "应晚于 2020-01-01");
    }

    #[test]
    fn system_time_to_millis_saturates_low() {
        use std::time::{Duration, UNIX_EPOCH};
        assert_eq!(system_time_to_millis(UNIX_EPOCH), 0);
        assert_eq!(
            system_time_to_millis(UNIX_EPOCH + Duration::from_millis(1500)),
            1500
        );
        // 早于 1970: duration_since 失败 → 0, 不 panic
        assert_eq!(
            system_time_to_millis(UNIX_EPOCH - Duration::from_secs(1)),
            0
        );
    }

    #[test]
    fn millis_to_system_time_saturates_never_panics() {
        use std::time::UNIX_EPOCH;
        assert_eq!(millis_to_system_time(0), UNIX_EPOCH);
        assert_eq!(millis_to_system_time(-5), UNIX_EPOCH, "负值按 0");
        assert_eq!(millis_to_system_time(i64::MIN), UNIX_EPOCH);
        // 溢出走 checked_add → 饱和到 epoch (release panic=abort 下不炸)
        assert_eq!(millis_to_system_time(i64::MAX), UNIX_EPOCH);
    }

    #[test]
    fn millis_round_trips_through_system_time() {
        let ms: i64 = 1_700_000_000_000;
        assert_eq!(system_time_to_millis(millis_to_system_time(ms)), ms);
    }
}
