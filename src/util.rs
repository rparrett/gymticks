use chrono::{DateTime, Utc};

pub fn time_diff_in_words(time: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let diff = now.signed_duration_since(time);

    let minutes = diff.num_minutes();
    let hours = diff.num_hours();
    let days = diff.num_days();

    if minutes < 1 {
        "now".to_string()
    } else if minutes == 1 {
        "1 min".to_string()
    } else if minutes < 60 {
        format!("{} min", minutes)
    } else if hours == 1 {
        "1 hr".to_string()
    } else if hours < 24 {
        format!("{} hr", hours)
    } else if days == 1 {
        "1 day".to_string()
    } else {
        format!("{} days", days)
    }
}
