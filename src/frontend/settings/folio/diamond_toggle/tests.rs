use std::time::{Duration, Instant};

use super::{GEOMETRY, State, ToggleIdentity};

fn identity() -> ToggleIdentity {
    ToggleIdentity::Path("settings.switch".into())
}

#[test]
fn completes_switch_timing() {
    let start = Instant::now();
    let mut state = State::default();
    assert!(!state.sync(&identity(), false, start, false));
    assert!(state.sync(&identity(), true, start, false));
    let middle = start + Duration::from_millis(210);
    assert!(state.sync(&identity(), true, middle, false));
    let visual = state.visual(true);
    assert!(visual.travel > 0.5 && visual.travel < 1.0);
    assert!(visual.fill > 0.5 && visual.fill < 1.0);
    assert_eq!(visual.off, 0.0);
    assert_eq!(visual.on, 1.0);
    assert!(!state.sync(&identity(), true, start + Duration::from_millis(420), false));
    assert_eq!(state.visual(true).travel, 1.0);
    assert_eq!(state.visual(true).fill, 1.0);
}

#[test]
fn retarget_keeps_position() {
    let start = Instant::now();
    let mut state = State::default();
    state.sync(&identity(), false, start, false);
    state.sync(&identity(), true, start, false);
    state.sync(&identity(), true, start + Duration::from_millis(100), false);
    let before = state.visual(true).travel;
    state.sync(&identity(), false, start + Duration::from_millis(100), false);
    assert_eq!(state.visual(false).travel, before);
    state.sync(&identity(), false, start + Duration::from_millis(520), false);
    assert_eq!(state.visual(false).travel, 0.0);
}

#[test]
fn fastest_motion_snaps() {
    let start = Instant::now();
    let mut state = State::default();
    state.sync(&identity(), false, start, true);
    assert!(!state.sync(&identity(), true, start, true));
    assert_eq!(state.visual(true).travel, 1.0);
}

#[test]
fn matches_thumb_travel() {
    assert_eq!(GEOMETRY.track_width - 2.0 * GEOMETRY.thumb_inset - GEOMETRY.thumb, 48.0);
}

#[test]
fn new_row_snaps() {
    let start = Instant::now();
    let mut state = State::default();
    state.sync(&identity(), false, start, false);
    state.sync(&identity(), true, start, false);
    state.sync(&identity(), true, start + Duration::from_millis(100), false);
    let other = ToggleIdentity::Path("settings.other".into());
    assert_eq!(state.visible(&other, false).travel, 0.0);
    assert!(!state.sync(&other, false, start + Duration::from_millis(100), false));
    assert_eq!(state.visual(false).travel, 0.0);
    assert_eq!(state.visual(false).fill, 0.0);
}
