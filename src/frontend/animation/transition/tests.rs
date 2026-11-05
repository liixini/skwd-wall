use std::time::{Duration, Instant};

use super::{CALM, Timing, Transition};

const SWITCH: Timing = Timing::new(Duration::from_millis(280), CALM);

#[test]
fn settles_on_time() {
    let start = Instant::now();
    let mut fill = Transition::flag(false);
    fill.toward_flag(true, start, SWITCH);
    assert!(fill.tick(start + Duration::from_millis(140)));
    assert!((fill.x - 0.5).abs() < 0.01, "calm midpoint");
    assert!(!fill.tick(start + Duration::from_millis(280)));
    assert_eq!(fill.x, 1.0);
    assert!(!fill.moving());
}

#[test]
fn waits_for_delay() {
    let start = Instant::now();
    let mut row = Transition::new(0.0);
    row.toward(1.0, start, SWITCH.after(Duration::from_millis(90)));
    assert!(row.tick(start + Duration::from_millis(90)));
    assert_eq!(row.x, 0.0);
    assert!(!row.tick(start + Duration::from_millis(370)));
    assert_eq!(row.x, 1.0);
}

#[test]
fn reverses_from_current() {
    let start = Instant::now();
    let mut fill = Transition::flag(false);
    fill.toward_flag(true, start, SWITCH);
    fill.tick(start + Duration::from_millis(100));
    let partway = fill.x;
    let turn = start + Duration::from_millis(100);
    fill.toward_flag(false, turn, SWITCH);
    assert!(fill.tick(turn));
    assert_eq!(fill.x, partway);
    assert!(!fill.tick(turn + Duration::from_millis(280)));
    assert_eq!(fill.x, 0.0);
}

#[test]
fn same_target_keeps_run() {
    let start = Instant::now();
    let mut fill = Transition::flag(false);
    fill.toward_flag(true, start, SWITCH);
    fill.toward_flag(true, start + Duration::from_millis(200), SWITCH);
    assert!(!fill.tick(start + Duration::from_millis(280)));
    assert_eq!(fill.x, 1.0);
}

#[test]
fn zero_duration_snaps() {
    let start = Instant::now();
    let mut fill = Transition::flag(false);
    fill.toward_flag(true, start, Timing::new(Duration::ZERO, CALM));
    assert!(!fill.tick(start));
    assert_eq!(fill.x, 1.0);
}
