use serde_json::json;

use super::*;
use crate::app::tests::test_app;
use crate::infrastructure::config::persisted;

fn held_early(app: &mut App) -> Option<()> {
    let mut held = Held::new(app);
    held.config.save_key(keys::general::RANDOM_ROTATE, json!(true));
    held.config.save_key(keys::wallpaper::VOLUME, json!(40));
    None?;
    Some(())
}

#[test]
fn early_return_writes() {
    let mut app = test_app();
    let before = persisted();
    assert!(held_early(&mut app).is_none());
    assert_eq!(persisted() - before, 1);
    app.config.save_key(keys::wallpaper::VOLUME, json!(50));
    assert_eq!(persisted() - before, 2, "hold released");
}
