use chrono::{DateTime, Utc};

pub fn time_diff_in_words(time: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let diff = now.signed_duration_since(time);

    let minutes = diff.num_minutes();
    let hours = diff.num_hours();
    let days = diff.num_days();

    if minutes < 1 {
        "now".to_string()
    } else if minutes == 1 {
        "1m".to_string()
    } else if minutes < 60 {
        format!("{}m", minutes)
    } else if hours == 1 {
        "1h".to_string()
    } else if hours < 24 {
        format!("{}h", hours)
    } else if days == 1 {
        "1d".to_string()
    } else {
        format!("{}d", days)
    }
}
