use paper_control::{ClockStyle, Depth, Face, Pin, Widget};

use super::{Spot, cycle, restack, summit};

const SCREEN: [f32; 2] = [1600.0, 900.0];
const APERTURE: Face = Face::Clock(ClockStyle::Aperture);
const GREETER: Face = Face::Clock(ClockStyle::Greeter);

fn spot(id: u32, face: Face, centre: [f32; 2], height: f32) -> Spot {
    Spot { id, face, depth: Depth::Behind, centre, height }
}

#[test]
fn pins_offsets() {
    let widget = spot(1, APERTURE, [400.0, 300.0], 280.0).widget(SCREEN, 1.0, None);
    assert_eq!(widget.pin.anchor, [0.0, 0.5]);
    assert_eq!(widget.pin.offset, [200.0, 160.0 - 0.5 * (900.0 - 280.0)]);
    let right = spot(2, APERTURE, [1400.0, 740.0], 280.0).widget(SCREEN, 1.0, None);
    assert_eq!(right.pin.anchor, [1.0, 1.0]);
    assert_eq!(right.pin.offset, [0.0, -20.0]);
}

#[test]
fn widget_round_trip() {
    for centre in [[300.0, 200.0], [800.0, 450.0], [1300.0, 700.0]] {
        let original = spot(3, APERTURE, centre, 280.0);
        let back = Spot::placed(3, &original.widget(SCREEN, 1.0, None), SCREEN, 1.0).unwrap();
        for axis in 0..2 {
            assert!((back.centre[axis] - centre[axis]).abs() < 1e-3, "centre {centre:?}");
        }
        assert!((back.height - 280.0).abs() < 1e-3);
    }
}

#[test]
fn clamps_height() {
    let tiny = spot(4, GREETER, [800.0, 450.0], 10.0).widget(SCREEN, 1.0, None);
    assert_eq!(tiny.height, Some(0.05));
    let huge = spot(5, GREETER, [800.0, 450.0], 2000.0).widget(SCREEN, 1.0, None);
    assert_eq!(huge.height, Some(1.0));
}

#[test]
fn natural_frame_height() {
    let widget = Widget {
        face: APERTURE,
        depth: Depth::Front,
        pin: Pin { anchor: [0.0, 0.0], offset: [10.0, 20.0] },
        height: None,
        output: None,
    };
    let placed = Spot::placed(6, &widget, SCREEN, 1.0).unwrap();
    assert_eq!(placed.height, 280.0);
    assert_eq!(placed.centre, [210.0, 160.0]);
    assert_eq!(placed.depth, Depth::Front);
}

#[test]
fn usual_spot_fits() {
    let usual = Spot::usual(7, APERTURE, [1366.0, 768.0]);
    let bounds = usual.bounds();
    assert!(bounds.x >= 0.0 && bounds.right() <= 1366.0);
    assert!(bounds.y >= 0.0 && bounds.bottom() <= 768.0);
    assert_eq!(usual.depth, Depth::Behind);
}

#[test]
fn restacks_overlaps() {
    let list = [
        spot(1, APERTURE, [400.0, 300.0], 280.0),
        spot(2, GREETER, [1400.0, 100.0], 70.0),
        spot(3, Face::Clock(ClockStyle::DayDial), [450.0, 320.0], 300.0),
        spot(4, Face::Clock(ClockStyle::ControlCenter), [420.0, 300.0], 160.0),
    ];
    assert_eq!(restack(&list, 0, true), Some(2));
    assert_eq!(restack(&list, 3, false), Some(2));
    assert_eq!(restack(&list, 1, true), None);
    assert_eq!(summit(&list, 0), Some(3));
}

#[test]
fn cycles_free_styles() {
    let list = [
        spot(1, APERTURE, [400.0, 300.0], 280.0),
        spot(2, Face::Clock(ClockStyle::ApertureRing), [1000.0, 300.0], 280.0),
    ];
    assert_eq!(cycle(&list, 0, true), Some(GREETER));
    assert_eq!(cycle(&list, 0, false), Some(Face::Clock(ClockStyle::DayDial)));
}
