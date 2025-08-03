use chrono::Local;
use std::time::Duration;


pub fn format_timestamp() -> String {
    let dt = Local::now();
    dt.format("%y%m%dT%H%M").to_string()
}

pub fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    if minutes > 0 {
        format!("{} minute{} {} second{}",
                minutes, if minutes != 1 { "s" } else { "" },
                seconds, if seconds != 1 { "s" } else { "" }
            )
    } else {
        format!("{} second{}", seconds, if seconds != 1 { "s" } else { "" })
    }
}
