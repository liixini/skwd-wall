use std::time::Duration;

use crate::domain::studio::Mode;

pub fn studio_time() -> (Mode, Duration) {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs() as i64);
    let local = skwd_log::local_time(seconds);
    let minute =
        (local.tm_hour * skwd_config::studio_time::MINUTES_PER_HOUR as i32 + local.tm_min) as u32;
    let dark = skwd_config::studio_time::dark_at(minute);
    let hour = if dark {
        skwd_config::studio_time::DAY_HOUR
    } else {
        skwd_config::studio_time::NIGHT_HOUR
    };
    let mode = if dark { Mode::Dark } else { Mode::Light };
    let wait = std::time::Duration::from_secs(skwd_log::seconds_until_local_hour(seconds, hour));
    (mode, wait)
}
