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
}
