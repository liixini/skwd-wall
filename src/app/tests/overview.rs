use super::*;
use skwd_config::keys::niri::{BACKDROP, BACKDROP_FOLLOW_WALLPAPER, OVERVIEW_BACKDROP};

#[test]
fn overview_confirmation_matches_the_saved_source_for_every_wallpaper_kind() {
    for (kind, we_id, source) in
        [("static", "", "/wp/test"), ("video", "", "/vids/test"), ("we", "123", "we:123")]
    {
        let mut app = test_app();
        let mut item = wall("test", kind, 1, 0);
        item["we_id"] = json!(we_id);
        seed(&mut app, &[item]);
        app.scene.set_flipped_for_test(Some(0));
        app.config.save_key(OVERVIEW_BACKDROP, json!(true));
        app.config.save_key(BACKDROP_FOLLOW_WALLPAPER, json!(false));
        app.config.save_key(BACKDROP, json!(source));
        assert!(view::overview_set(&app), "{kind}");
        app.config.save_key(BACKDROP, json!("/thumbs/test.png"));
        assert!(!view::overview_set(&app), "thumbnail is not the selected {kind} source");
    }
}

#[test]
fn overview_confirmation_requires_an_enabled_fixed_wallpaper() {
    let mut app = test_app();
    seed(&mut app, &[wall("test", "video", 1, 0)]);
    app.scene.set_flipped_for_test(Some(0));
    app.config.save_key(BACKDROP, json!("/vids/test"));
    for (enabled, follow, expected) in
        [(false, false, false), (true, true, false), (true, false, true)]
    {
        app.config.save_key(OVERVIEW_BACKDROP, json!(enabled));
        app.config.save_key(BACKDROP_FOLLOW_WALLPAPER, json!(follow));
        assert_eq!(view::overview_set(&app), expected);
    }
    app.scene.set_flipped_for_test(None);
    assert!(!view::overview_set(&app));
}

#[test]
fn overview_confirmation_rejects_empty_sources() {
    let mut app = test_app();
    seed(&mut app, &[wall("test", "video", 1, 0)]);
    edit_catalog(&mut app, |catalog| catalog.items[0].path.clear());
    app.scene.set_flipped_for_test(Some(0));
    app.config.save_key(OVERVIEW_BACKDROP, json!(true));
    app.config.save_key(BACKDROP_FOLLOW_WALLPAPER, json!(false));
    app.config.save_key(BACKDROP, json!(""));
    assert!(!view::overview_set(&app));
}
