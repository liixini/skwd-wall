use super::*;
use crate::domain::studio::{Mode, ModeChoice, Source, Studio};
use skwd_palette::studio::Tuning;

#[test]
fn stale_time_wakes() {
    let (effective, _) = crate::infrastructure::theme::studio_time();
    let stale = if effective == Mode::Dark { Mode::Light } else { Mode::Dark };
    let mut studio = Studio::new(Source::Theme("nord".into()), stale, Tuning::default());
    studio.mode_choice = ModeChoice::ByTime;
    let now = Instant::now();
    assert_eq!(studio_deadline(now, &studio), Some(now));
    studio.mode = effective;
    assert!(studio_deadline(now, &studio) > Some(now));
}
