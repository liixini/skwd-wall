use std::time::{Duration, Instant};

use iced::{Point, Rectangle, Size, mouse};

use super::{SWITCH, Switch, Track};
use crate::frontend::animation::{CSS_EASE, Timing};

const TRACKS: [Track; 2] = [
    Track { timing: SWITCH, hover: false },
    Track { timing: Timing::new(Duration::from_millis(180), CSS_EASE), hover: true },
];

fn at(start: Instant, ms: u64) -> Instant {
    start + Duration::from_millis(ms)
}

#[test]
fn first_frame_snaps() {
    let mut switch = Switch::<2>::default();
    assert_eq!(switch.values(7, true, TRACKS), [1.0, 1.0]);
    assert!(!switch.sync(7, true, Instant::now(), false, TRACKS));
    assert_eq!(switch.values(7, true, TRACKS), [1.0, 1.0]);
}

#[test]
fn settles_in_280() {
    let start = Instant::now();
    let mut switch = Switch::<2>::default();
    switch.sync(7, false, start, false, TRACKS);
    assert!(switch.sync(7, true, start, false, TRACKS));
    assert!(switch.sync(7, true, at(start, 200), false, TRACKS));
    let [wipe, border] = switch.values(7, true, TRACKS);
    assert!(wipe > 0.5 && wipe < 1.0);
    assert_eq!(border, 1.0);
    assert!(!switch.sync(7, true, at(start, 280), false, TRACKS));
    assert_eq!(switch.values(7, true, TRACKS), [1.0, 1.0]);
}

#[test]
fn hover_lights_border() {
    let start = Instant::now();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(92.0, 30.0));
    let mut switch = Switch::<2>::default();
    switch.sync(7, false, start, false, TRACKS);
    assert!(switch.hover(bounds, mouse::Cursor::Available(Point::new(10.0, 10.0))));
    assert!(switch.sync(7, false, start, false, TRACKS));
    assert!(!switch.sync(7, false, at(start, 180), false, TRACKS));
    assert_eq!(switch.values(7, false, TRACKS), [0.0, 1.0]);
    assert!(!switch.hover(bounds, mouse::Cursor::Available(Point::new(20.0, 10.0))));
}

#[test]
fn new_identity_snaps() {
    let start = Instant::now();
    let mut switch = Switch::<2>::default();
    switch.sync(7, false, start, false, TRACKS);
    switch.sync(7, true, start, false, TRACKS);
    assert_eq!(switch.values(8, false, TRACKS), [0.0, 0.0]);
    assert!(!switch.sync(8, false, at(start, 50), false, TRACKS));
}

#[test]
fn reduced_motion_snaps() {
    let start = Instant::now();
    let mut switch = Switch::<2>::default();
    switch.sync(7, false, start, false, TRACKS);
    assert!(!switch.sync(7, true, start, true, TRACKS));
    assert_eq!(switch.values(7, true, TRACKS), [1.0, 1.0]);
}
