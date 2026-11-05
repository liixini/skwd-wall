use paper_control::{ClockStyle, Depth, Face, Pin, Widget};

use super::{Saved, merge};

fn clock(x: f32) -> Widget {
    Widget {
        face: Face::Clock(ClockStyle::Aperture),
        depth: Depth::Behind,
        pin: Pin { anchor: [0.0, 0.0], offset: [x, 0.0] },
        height: None,
        output: None,
    }
}

fn encode(widget: &Widget) -> String {
    format!("placed {}", widget.pin.offset[0])
}

#[test]
fn keeps_passthrough_order() {
    let raw = ["a", "bad", "b", "other"].map(String::from).to_vec();
    let saved = vec![Saved::Kept(1), Saved::Placed(clock(4.0)), Saved::Placed(clock(9.0))];
    let merged = merge(raw, &[0, 2], saved, encode);
    assert_eq!(merged, ["b", "bad", "placed 4", "other", "placed 9"]);
}

#[test]
fn drops_removed() {
    let raw = ["a", "keep", "b"].map(String::from).to_vec();
    let merged = merge(raw, &[0, 2], vec![Saved::Kept(1)], encode);
    assert_eq!(merged, ["b", "keep"]);
}
