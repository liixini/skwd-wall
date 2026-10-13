#![cfg(test)]

use super::tile_is_wide;

#[test]
fn display_tiles_go_wide_only_with_room() {
    assert!(!tile_is_wide((1280.0, 800.0), 1.0));
    assert!(tile_is_wide(crate::frontend::ui::MIN_VIEWPORT, 1.0));
    assert!(tile_is_wide((1920.0, 1080.0), 1.0));
    assert!(tile_is_wide((2560.0, 1440.0), 1.0));
    assert!(!tile_is_wide((1920.0, 1080.0), 1.5));
}
