const WEEK: u64 = 604800000;
const DAY: u64 = 86400000;
const HOUR: u64 = 3600000;
const MINUTE: u64 = 60000;
const SECOND: u64 = 1000;

/// Format ETA from milliseconds to a human-readable string with up to two time units
pub(crate) fn format_eta(eta_millis: u64) -> String {
    if eta_millis == 0 || eta_millis < SECOND {
        return "0 secs".to_string();
    }

    let weeks = eta_millis / WEEK;
    let days = (eta_millis % WEEK) / DAY;
    let hours = (eta_millis % DAY) / HOUR;
    let minutes = (eta_millis % HOUR) / MINUTE;
    let seconds = (eta_millis % MINUTE) / SECOND;

    let mut parts = Vec::new();

    if weeks > 0 {
        parts.push(format!("{} weeks", weeks));
    }
    if days > 0 {
        parts.push(format!("{} days", days));
    }
    if hours > 0 {
        parts.push(format!("{} hours", hours));
    }
    if minutes > 0 {
        parts.push(format!("{} mins", minutes));
    }
    if seconds > 0 {
        parts.push(format!("{} secs", seconds));
    }

    parts.into_iter().take(2).collect::<Vec<_>>().join(" ")
}

#[test]
fn test_format_eta() {
    assert_eq!(format_eta(0), "0 secs");
    assert_eq!(format_eta(999), "0 secs");
    assert_eq!(format_eta(1000), "1 secs");
    assert_eq!(format_eta(60_000), "1 mins");
    assert_eq!(format_eta(60_999), "1 mins");
    assert_eq!(format_eta(120_000), "2 mins");
    assert_eq!(format_eta(3_600_000), "1 hours");
    assert_eq!(format_eta(3_600_999), "1 hours");
    assert_eq!(format_eta(3_661_000), "1 hours 1 mins");
    assert_eq!(format_eta(86_400_000), "1 days");
    assert_eq!(format_eta(86_460_000), "1 days 1 mins");
    assert_eq!(format_eta(604_800_000), "1 weeks");
    assert_eq!(format_eta(604_860_000), "1 weeks 1 mins");
    assert_eq!(format_eta(1_000_000), "16 mins 40 secs");
    assert_eq!(format_eta(10_000_000), "2 hours 46 mins");
    assert_eq!(format_eta(864_000_000), "1 weeks 3 days");
}
