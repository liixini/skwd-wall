#![cfg(test)]

use super::nav::{Ends, follow, land, lap, rebase, step};
use crate::frontend::animation::Spring;

#[test]
fn step_clamps_dedups() {
    assert_eq!(step(0, 1, 5, Ends::Stop), Some(1));
    assert_eq!(step(4, 1, 5, Ends::Stop), None);
    assert_eq!(step(0, -1, 5, Ends::Stop), None);
    assert_eq!(step(2, -1, 5, Ends::Stop), Some(1));
    assert_eq!(step(2, 10, 5, Ends::Stop), Some(4));
    assert_eq!(step(0, 1, 0, Ends::Stop), None);
    assert_eq!(step(0, 1, 1, Ends::Stop), None);
}

#[test]
fn step_wraps_from_edges() {
    assert_eq!(step(0, -1, 5, Ends::Wrap), Some(4));
    assert_eq!(step(4, 1, 5, Ends::Wrap), Some(0));
    assert_eq!(step(0, -10, 5, Ends::Wrap), Some(4));
    assert_eq!(step(4, 10, 5, Ends::Wrap), Some(0));
    assert_eq!(step(2, -10, 5, Ends::Wrap), Some(0));
    assert_eq!(step(2, 10, 5, Ends::Wrap), Some(4));
    assert_eq!(step(0, 1, 1, Ends::Wrap), None);
    assert_eq!(step(0, -1, 0, Ends::Wrap), None);
}

#[test]
fn step_loops_around() {
    assert_eq!(step(4, 1, 5, Ends::Loop), Some(0));
    assert_eq!(step(0, -1, 5, Ends::Loop), Some(4));
    assert_eq!(step(3, 3, 5, Ends::Loop), Some(1));
    assert_eq!(step(1, -7, 5, Ends::Loop), Some(4));
    assert_eq!(step(2, 5, 5, Ends::Loop), None);
}

#[test]
fn follow_retargets_offscreen() {
    let mut inside = Spring::for_duration_ms(0.0, 300.0);
    assert!(!follow(&mut inside, 0.0, 100.0, 500.0, 0.0..=1000.0));
    assert_eq!(inside.target, 0.0);

    let mut below = Spring::for_duration_ms(0.0, 300.0);
    assert!(follow(&mut below, 900.0, 100.0, 500.0, 0.0..=1000.0));
    assert_eq!(below.target, 500.0);

    let mut above = Spring::for_duration_ms(0.0, 300.0);
    above.retarget(400.0);
    assert!(follow(&mut above, 100.0, 100.0, 500.0, 0.0..=1000.0));
    assert_eq!(above.target, 100.0);

    let mut clamped = Spring::for_duration_ms(0.0, 300.0);
    assert!(follow(&mut clamped, 5000.0, 100.0, 500.0, 0.0..=300.0));
    assert_eq!(clamped.target, 300.0);
}

#[test]
fn land_skips_far_stretch() {
    let mut near = Spring::for_duration_ms(0.0, 300.0);
    land(&mut near, 400.0, 500.0);
    assert_eq!((near.x, near.target), (0.0, 400.0));

    let mut ahead = Spring::for_duration_ms(0.0, 300.0);
    land(&mut ahead, 9000.0, 500.0);
    assert_eq!((ahead.x, ahead.v, ahead.target), (8500.0, 0.0, 9000.0));

    let mut behind = Spring::for_duration_ms(9000.0, 300.0);
    land(&mut behind, 0.0, 500.0);
    assert_eq!((behind.x, behind.target), (500.0, 0.0));
}

#[test]
fn follow_crosses_zero() {
    let mut above = Spring::for_duration_ms(0.0, 300.0);
    assert!(follow(&mut above, -200.0, 100.0, 500.0, f32::MIN..=f32::MAX));
    assert_eq!(above.target, -200.0);
}

#[test]
fn lap_picks_nearest_copy() {
    assert_eq!(lap(10.0, 990.0, 1000.0), 1.0);
    assert_eq!(lap(990.0, 10.0, 1000.0), -1.0);
    assert_eq!(lap(400.0, 500.0, 1000.0), 0.0);
}

#[test]
fn rebase_keeps_offset() {
    let mut spring = Spring::for_duration_ms(2050.0, 300.0);
    spring.retarget(2100.0);
    spring.v = 3.0;
    rebase(&mut spring, 1000.0);
    assert_eq!((spring.x, spring.target, spring.v), (50.0, 100.0, 3.0));
}
