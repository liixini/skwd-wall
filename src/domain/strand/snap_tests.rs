use paper_control::{ClockStyle, Face};

use super::{Grip, anchor, cells, detent, snap, stretch};
use crate::domain::strand::Rect;
use crate::domain::strand::shortest;

const SCREEN: [f32; 2] = [1600.0, 900.0];
const APERTURE: Face = Face::Clock(ClockStyle::Aperture);

#[test]
fn snaps_stage_centre() {
    let snapped = snap([400.0, 280.0], [806.0, 280.0], &[], SCREEN);
    assert_eq!(snapped.centre[0], 800.0);
    assert_eq!(snapped.guides.len(), 1);
    assert!(snapped.guides[0].vertical);
    assert!(snapped.guides[0].with.is_none());
}

#[test]
fn snaps_widget_edge() {
    let other = Rect::new(100.0, 600.0, 200.0, 100.0);
    let snapped = snap([100.0, 50.0], [355.0, 200.0], &[other], SCREEN);
    assert_eq!(snapped.centre[0], 350.0);
    assert_eq!(snapped.guides[0].with, Some(other));
}

#[test]
fn keeps_inside_output() {
    let snapped = snap([400.0, 280.0], [-500.0, 2000.0], &[], SCREEN);
    assert_eq!(snapped.centre, [200.0, 760.0]);
}

#[test]
fn steps_detents() {
    assert_eq!(detent(0.31, true, [shortest(), 0.6]), Some(0.4));
    assert_eq!(detent(0.31, false, [shortest(), 0.6]), Some(0.3));
    assert_eq!(detent(0.3, true, [shortest(), 0.6]), Some(0.4));
    assert_eq!(detent(0.6, true, [shortest(), 0.6]), None);
    assert_eq!(detent(0.05, false, [shortest(), 0.6]), None);
}

#[test]
fn grip_catches_detent() {
    let bounds = Rect::new(100.0, 100.0, 400.0, 280.0);
    let grip = Grip::new(bounds, 2, [500.0, 380.0]);
    let free = grip.resize([520.0, 394.0], APERTURE, SCREEN);
    assert!(!free.snapped);
    let caught = grip.resize([493.0, 375.0], APERTURE, SCREEN);
    assert!(caught.snapped);
    assert_eq!(caught.share, 0.3);
    assert_eq!(grip.centre(APERTURE, 280.0), [300.0, 240.0]);
}

#[test]
fn stretch_holds_anchor() {
    let grip = [[100.0, 100.0], [1.0, 1.0]];
    let stretched = stretch(grip, [700.0, 380.0], [160.0, 96.0], &[], SCREEN);
    assert_eq!(stretched.rect, Rect::new(100.0, 100.0, 600.0, 280.0));
    assert!(stretched.guides.is_empty());
}

#[test]
fn stretch_snaps_edges() {
    let other = Rect::new(900.0, 200.0, 300.0, 100.0);
    let grip = [[100.0, 100.0], [1.0, 1.0]];
    let stretched = stretch(grip, [895.0, 446.0], [160.0, 96.0], &[other], SCREEN);
    assert_eq!(stretched.rect.right(), 900.0);
    assert_eq!(stretched.rect.bottom(), 450.0);
    assert_eq!(stretched.guides.len(), 2);
    assert_eq!(stretched.guides[0].with, Some(other));
}

#[test]
fn stretch_keeps_least_size() {
    let grip = [[800.0, 600.0], [-1.0, -1.0]];
    let stretched = stretch(grip, [780.0, 900.0], [160.0, 96.0], &[], SCREEN);
    assert_eq!(stretched.rect, Rect::new(640.0, 504.0, 160.0, 96.0));
}

#[test]
fn anchors_by_thirds() {
    let screen = Rect::new(0.0, 0.0, 1600.0, 900.0);
    assert_eq!(anchor(Rect::new(20.0, 20.0, 100.0, 50.0), screen), [0, 0]);
    assert_eq!(anchor(Rect::new(700.0, 800.0, 200.0, 60.0), screen), [1, 2]);
    assert_eq!(anchor(Rect::new(1400.0, 400.0, 100.0, 100.0), screen), [2, 1]);
}

#[test]
fn rounds_grid_cells() {
    assert_eq!(cells(620.0, 180.0, 20.0, 20), 3);
    assert_eq!(cells(10.0, 180.0, 20.0, 20), 1);
    assert_eq!(cells(9000.0, 180.0, 20.0, 20), 20);
}
