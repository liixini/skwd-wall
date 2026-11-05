use std::time::{Duration, Instant};

use iced::{Color, Vector};

use super::{Kind, Phase, PlaneSpec, RevealSpec, Rise, StaggerSpec, Tick, grow, row_delay};
use crate::frontend::animation::Transition;

const CLOSED: Color = Color::BLACK;
const OPEN: Color = Color::WHITE;

fn spec(index: usize, open: bool) -> StaggerSpec {
    StaggerSpec { identity: 1, index, open, instant: false, planes: [CLOSED, OPEN] }
}

fn body(open: bool) -> Kind {
    Kind::Grow(RevealSpec { identity: 1, open, instant: false, floor: 0.0 })
}

fn at(start: Instant, ms: u64) -> Instant {
    start + Duration::from_millis(ms)
}

#[test]
fn rows_stagger_by_fifty() {
    let delays: Vec<u128> = (0..8).map(|index| row_delay(index).as_millis()).collect();
    assert_eq!(delays, [90, 140, 190, 240, 290, 340, 340, 340]);
}

#[test]
fn height_grows_in_390() {
    let start = Instant::now();
    let mut height = Transition::flag(false);
    assert!(grow(&mut height, true, false, start));
    assert!(grow(&mut height, true, false, at(start, 195)));
    assert!((height.x - 0.5).abs() < 0.01, "calm midpoint");
    assert!(!grow(&mut height, true, false, at(start, 390)));
    assert_eq!(height.x, 1.0);
}

#[test]
fn layout_height_midway() {
    let start = Instant::now();
    let mut phase = Phase::new(body(false));
    assert!(phase.folded());
    assert_eq!(phase.tick(body(true), start), Tick { redraw: true, relayout: true });
    assert!(!phase.folded());
    phase.tick(body(true), at(start, 195));
    assert!((phase.height(200.0) - 100.0).abs() < 2.0);
    phase.tick(body(true), at(start, 390));
    assert_eq!(phase.height(200.0), 200.0);
}

#[test]
fn idle_requests_nothing() {
    let start = Instant::now();
    for kind in [body(true), body(false), Kind::Rise(spec(2, true))] {
        let mut phase = Phase::new(kind);
        assert_eq!(phase.tick(kind, start), Tick { redraw: false, relayout: false });
        assert_eq!(phase.tick(kind, at(start, 1000)), Tick { redraw: false, relayout: false });
    }
}

#[test]
fn moving_blocks_input() {
    let start = Instant::now();
    let mut phase = Phase::new(body(true));
    assert!(phase.live());
    phase.tick(body(false), start);
    phase.tick(body(false), at(start, 100));
    assert!(!phase.live(), "closing");
    phase.tick(body(false), at(start, 390));
    assert!(phase.folded() && !phase.live());
    phase.tick(body(true), at(start, 400));
    phase.tick(body(true), at(start, 700));
    assert!(!phase.live(), "opening");
    phase.tick(body(true), at(start, 790));
    assert!(phase.live());
}

#[test]
fn reduced_motion_settles() {
    let start = Instant::now();
    let instant = Kind::Grow(RevealSpec { identity: 1, open: true, instant: true, floor: 0.0 });
    let mut phase = Phase::new(body(false));
    assert!(!phase.tick(instant, start).redraw);
    assert!(phase.live());
    let mut row = Rise::new(false);
    assert!(!row.sync(StaggerSpec { instant: true, ..spec(3, true) }, start));
    assert_eq!((row.fade.x, row.offset()), (1.0, Vector::ZERO));
}

#[test]
fn row_fades_after_delay() {
    let start = Instant::now();
    let mut row = Rise::new(false);
    assert_eq!(row.offset(), Vector::new(0.0, -6.0));
    assert!(row.sync(spec(1, true), start));
    assert!(row.sync(spec(1, true), at(start, 140)));
    assert_eq!(row.fade.x, 0.0);
    assert!(row.sync(spec(1, true), at(start, 300)));
    assert_eq!(row.fade.x, 1.0);
    assert!(row.lift.x < 1.0);
    assert!(!row.sync(spec(1, true), at(start, 390)));
    assert_eq!(row.offset(), Vector::ZERO);
}

#[test]
fn veil_follows_plane() {
    let start = Instant::now();
    let mut row = Rise::new(false);
    assert_eq!(row.veil(spec(0, true)), CLOSED);
    row.sync(spec(0, true), start);
    row.sync(spec(0, true), at(start, 195));
    let veil = row.veil(spec(0, true));
    assert!((veil.r - 0.5).abs() < 0.01, "plane midway");
    assert!(veil.a > 0.0 && veil.a < 1.0);
}

#[test]
fn close_skips_delay() {
    let start = Instant::now();
    let mut row = Rise::new(true);
    assert!(row.sync(spec(5, false), start));
    assert!(row.sync(spec(5, false), at(start, 80)));
    assert!(row.fade.x > 0.0 && row.fade.x < 1.0);
    assert!(!row.sync(spec(5, false), at(start, 390)));
    assert_eq!((row.fade.x, row.offset()), (0.0, Vector::new(0.0, -6.0)));
}

#[test]
fn plane_tracks_veil() {
    let start = Instant::now();
    let tint =
        |open| Kind::Plane(PlaneSpec { identity: 1, open, instant: false, planes: [CLOSED, OPEN] });
    let mut section = Phase::new(tint(false));
    let mut row = Phase::new(Kind::Rise(spec(3, false)));
    for ms in [0, 60, 130, 195, 300, 390] {
        let tick = section.tick(tint(true), at(start, ms));
        row.tick(Kind::Rise(spec(3, true)), at(start, ms));
        assert_eq!(section.rise.plane.x, row.rise.plane.x, "at {ms}");
        assert!(!tick.relayout);
    }
    assert_eq!(section.rise.plane.x, 1.0);
}
