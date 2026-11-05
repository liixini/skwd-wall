use super::*;

#[test]
fn curve_hits_ends() {
    assert_eq!(EASE.at(0.0), 0.0);
    assert_eq!(EASE.at(1.0), 1.0);
    assert!(EASE.at(0.5) > 0.8);
    assert!((CSS_EASE.at(0.5) - 0.8024).abs() < 0.01);
}

#[test]
fn curve_is_monotonic() {
    let samples: Vec<f32> = (0..=100).map(|step| EASE.at(step as f32 / 100.0)).collect();
    assert!(samples.windows(2).all(|pair| pair[1] >= pair[0]));
}

#[test]
fn glide_settles_on_target() {
    let mut glide = Glide::new(0.0, BASE);
    let done = glide.toward(10.0, 1.0);
    assert!((done - (1.0 + BASE)).abs() < 1e-6);
    assert_eq!(glide.value(1.0), 0.0);
    assert_eq!(glide.value(done), 10.0);
    glide.jump(3.0);
    assert_eq!(glide.value(0.0), 3.0);
}

#[test]
fn swap_hands_over() {
    let mut swap = Swap::new(Axis::Y);
    swap.set("one".into(), 1.0, 0.0, true);
    assert!(swap.outgoing(0.0).is_none());
    let done = swap.set("two".into(), -1.0, 1.0, false);
    assert!(done > 1.0);
    let leaving = swap.outgoing(1.0).expect("old copy");
    assert_eq!(leaving.alpha, 1.0);
    assert_eq!(swap.incoming(1.0).alpha, 0.0);
    assert!(swap.incoming(1.0).dy < 0.0);
    assert!(swap.outgoing(done).is_none());
    assert_eq!(swap.incoming(done).alpha, 1.0);
}

#[test]
fn looks_rest_when_done() {
    assert_eq!(Look::lift(1.0), Look::REST);
    assert_eq!(Look::pop(1.0), Look::REST);
    assert_eq!(Look::lift(0.0).dy, crate::frontend::colour_studio::tokens::LIFT);
}

#[test]
fn swaps_skip_queued_middles() {
    let mut swap = Swap::new(Axis::X);
    swap.set("a".into(), 1.0, 0.0, false);
    let end = swap.set("b".into(), 1.0, 0.0, false);
    swap.set("c".into(), 1.0, 0.1, false);
    swap.set("d".into(), 1.0, 0.2, false);
    assert_eq!(swap.now, "b");
    assert!(swap.settle(end) > end);
    assert_eq!((swap.was.as_str(), swap.now.as_str()), ("b", "d"));
}

#[test]
fn crossfade_skips_queued_middles() {
    let mut fade = Crossfade::default();
    fade.show(0, 'a', 0.0);
    let end = fade.show(1, 'b', 0.0);
    fade.show(2, 'c', 0.1);
    fade.show(3, 'd', 0.2);
    assert_eq!(fade.shown(), Some(&'b'));
    assert!(fade.settle(end) > end);
    assert_eq!((fade.previous(end).map(|(old, _)| *old), fade.shown()), (Some('b'), Some(&'d')));
    fade.settle(end + SLOW);
    assert!(fade.previous(end + SLOW).is_none());
}
