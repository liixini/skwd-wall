use iced::Color;

use super::{ROW_TINT, SELECTED_TINT, tint};

fn rgb8(color: Color) -> [u8; 3] {
    let [r, g, b, _] = color.into_rgba8();
    [r, g, b]
}

#[test]
fn matches_board_mix() {
    let surface = Color::from_rgb8(21, 24, 30);
    let primary = Color::from_rgb8(240, 163, 94);
    assert_eq!(rgb8(tint(surface, primary, ROW_TINT)), [34, 32, 34]);
    assert_eq!(rgb8(tint(surface, primary, SELECTED_TINT)), [60, 49, 42]);
    assert_eq!(tint(surface, primary, ROW_TINT).a, 1.0);
}

#[test]
fn ink_alpha_counts() {
    let surface = Color::from_rgb8(0, 0, 0);
    let half = Color { a: 0.5, ..Color::WHITE };
    assert_eq!(rgb8(tint(surface, half, 1.0)), [128, 128, 128]);
}
