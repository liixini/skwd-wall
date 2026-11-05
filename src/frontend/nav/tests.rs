#![cfg(test)]

use super::nav::{follow, step};
use crate::frontend::animation::Spring;

#[test]
fn step_clamps_dedups() {
    assert_eq!(step(0, 1, 5, false), Some(1));
    assert_eq!(step(4, 1, 5, false), None);
    assert_eq!(step(0, -1, 5, false), None);
    assert_eq!(step(2, -1, 5, false), Some(1));
    assert_eq!(step(2, 10, 5, false), Some(4));
    assert_eq!(step(0, 1, 0, false), None);
    assert_eq!(step(0, 1, 1, false), None);
}

#[test]
fn step_wraps_from_edges() {
    assert_eq!(step(0, -1, 5, true), Some(4));
    assert_eq!(step(4, 1, 5, true), Some(0));
    assert_eq!(step(0, -10, 5, true), Some(4));
    assert_eq!(step(4, 10, 5, true), Some(0));
    assert_eq!(step(2, -10, 5, true), Some(0));
    assert_eq!(step(2, 10, 5, true), Some(4));
    assert_eq!(step(0, 1, 1, true), None);
    assert_eq!(step(0, -1, 0, true), None);
}

#[test]
fn follow_retargets_offscreen() {
    let mut inside = Spring::for_duration_ms(0.0, 300.0);
    assert!(!follow(&mut inside, 0.0, 100.0, 500.0, 1000.0));
    assert_eq!(inside.target, 0.0);

    let mut below = Spring::for_duration_ms(0.0, 300.0);
    assert!(follow(&mut below, 900.0, 100.0, 500.0, 1000.0));
    assert_eq!(below.target, 500.0);

    let mut above = Spring::for_duration_ms(0.0, 300.0);
    above.retarget(400.0);
    assert!(follow(&mut above, 100.0, 100.0, 500.0, 1000.0));
    assert_eq!(above.target, 100.0);

    let mut clamped = Spring::for_duration_ms(0.0, 300.0);
    assert!(follow(&mut clamped, 5000.0, 100.0, 500.0, 300.0));
    assert_eq!(clamped.target, 300.0);
}
