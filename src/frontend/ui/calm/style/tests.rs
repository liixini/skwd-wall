use iced::widget::button::Status;

use super::super::render_tests::board;
use super::super::tint::{SELECTED_TINT, tint};
use super::item;

#[test]
fn lit_item_tinted() {
    let palette = board();
    let ground = palette.surface;
    let lit = item(ground, &palette, true, Status::Active);
    let expected = tint(ground, palette.primary, SELECTED_TINT);
    assert_eq!(lit.background, Some(expected.into()));
    assert!(item(ground, &palette, false, Status::Active).background.is_none());
    assert!(item(ground, &palette, false, Status::Hovered).background.is_some());
}
