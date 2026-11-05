use std::time::Duration;

use super::{Paint, TRACKS};
use crate::frontend::animation::{CALM, CSS_EASE};
use crate::frontend::theme::Palette;

fn paint(label: &str) -> Paint<'_> {
    Paint {
        identity: 1,
        label,
        open: false,
        focused: false,
        instant: false,
        scale: 1.0,
        palette: Palette::default(),
        plane: Palette::default().surface,
    }
}

#[test]
fn timing_matches_design() {
    let [turn, fill, border] = TRACKS;
    assert_eq!((turn.timing.duration, turn.timing.curve), (Duration::from_millis(280), CALM));
    assert_eq!(
        (fill.timing.duration, fill.timing.curve, fill.hover),
        (Duration::from_millis(280), CSS_EASE, false)
    );
    assert!(border.hover);
}

#[test]
fn width_follows_label() {
    let short = paint("Options").width();
    let long = paint("Weitere Einstellungen anzeigen").width();
    let (label, chevron, gap) = paint("Options").sizes();
    assert_eq!(short, label + gap + chevron + 24.0);
    assert!(long > short);
}
