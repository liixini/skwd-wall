use std::time::Duration;

use super::{TRACKS, label};
use crate::frontend::animation::{CALM, CSS_EASE};

#[test]
fn timing_matches_design() {
    let [wipe, border] = TRACKS;
    assert_eq!(
        (wipe.timing.duration, wipe.timing.curve, wipe.hover),
        (Duration::from_millis(280), CALM, false)
    );
    assert_eq!(
        (border.timing.duration, border.timing.curve, border.hover),
        (Duration::from_millis(180), CSS_EASE, true)
    );
}

#[test]
fn rtl_labels_anchor_start() {
    use iced::advanced::text::Alignment;
    use iced::{Color, Point};
    let centre = Point::new(100.0, 20.0);
    let latin = label("On", centre, Color::WHITE, 1.0);
    assert_eq!((latin.position, latin.align_x), (centre, Alignment::Center));
    let arabic = label("مفعّل", centre, Color::WHITE, 1.0);
    assert_eq!(arabic.align_x, Alignment::Default);
    assert!(arabic.position.x < centre.x, "{:?}", arabic.position);
}
