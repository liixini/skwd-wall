use super::{
    App, Config, Instant, Value, drain_calls, edit_catalog, filtered_names, json, seed, test_app,
    tick_frames, wall,
};
use skwd_config::keys::selector::{LAST_APPLIED_KEY, START_POSITION};

fn keyed_wall(name: &str, kind: &str, hue: i64, mtime: i64) -> Value {
    let mut item = wall(name, kind, hue, mtime);
    item["key"] = json!(format!("{kind}:{name}"));
    item["width"] = json!(if hue % 3 == 0 { 720 } else { 1280 });
    item["height"] = json!(720);
    item
}

fn saved_selection(enabled: bool, key: &str) -> App {
    let mut app = test_app();
    app.config.set_key(START_POSITION, json!(if enabled { "applied" } else { "beginning" }));
    app.config.set_key(LAST_APPLIED_KEY, json!(key));
    app
}

fn reopen(app: &App) -> App {
    let mut config = Config::from_data(
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap(),
    );
    config.config_path = app.config.config_path.clone();
    let mut reopened =
        App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording());
    reopened.on_list(app.library_session.library.catalog().clone());
    assert!(!drain_calls(&reopened).iter().any(|(method, _)| method == "wall.apply"));
    reopened
}

fn browsing_app(mode: &str) -> App {
    let mut app = test_app();
    app.config.set_key(START_POSITION, json!("browsing"));
    app.config.set_key(skwd_config::keys::selector::DISPLAY_MODE, json!(mode));
    let mut app = App::with_config_using(app.config, |_| {
        crate::infrastructure::ipc::DaemonClient::recording()
    });
    let walls: Vec<_> =
        (0..100).map(|index| keyed_wall(&format!("wall-{index}"), "static", index, 0)).collect();
    seed(&mut app, &walls);
    app
}

#[test]
fn browsing_without_applying_survives_close_in_every_mode() {
    for mode in ["slices", "depth", "hex", "wall", "sandy", "hand", "collection"] {
        let mut app = browsing_app(mode);
        app.scene.set_current(75, 100);
        let mut now = Instant::now();
        tick_frames(&mut app, &mut now, 120);
        app.on_hidden();
        let mut reopened = reopen(&app);
        assert_eq!(reopened.scene.current, 75, "{mode}");
        tick_frames(&mut reopened, &mut now, 120);
        assert_eq!(reopened.scene.current, 75, "{mode}");
        assert!(reopened.config.str_path(LAST_APPLIED_KEY).is_empty());
    }
}

#[test]
fn browsing_restores_scroll_even_when_selected_card_is_offscreen() {
    let mut app = browsing_app("wall");
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    app.scene.grid_scroll(15.0);
    tick_frames(&mut app, &mut now, 120);
    assert_eq!(app.scene.current, 0);
    assert!(app.scene.visible_range().0 > 0);
    let camera = app.scene.camera_pos();
    app.on_hidden();
    let mut reopened = reopen(&app);
    tick_frames(&mut reopened, &mut now, 120);
    assert_eq!(reopened.scene.current, 0);
    assert!((reopened.scene.camera_pos() - camera).abs() < 0.1);
    assert!(reopened.scene.visible_range().0 > 0);
}

#[test]
fn changed_library_restores_visible_anchor_instead_of_stale_scroll() {
    let mut app = browsing_app("wall");
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    app.scene.grid_scroll(15.0);
    tick_frames(&mut app, &mut now, 120);
    app.save_browse_position();
    let anchor =
        skwd_config::get(app.config.root(), skwd_config::keys::selector::LAST_BROWSE_POSITION)
            .unwrap()["anchor"]
            .as_str()
            .unwrap()
            .to_string();
    edit_catalog(&mut app, |catalog| {
        catalog.items.remove(0);
    });
    let mut reopened = reopen(&app);
    tick_frames(&mut reopened, &mut now, 120);
    let index = reopened.library_session.filtered[reopened.scene.current] as usize;
    assert_eq!(reopened.library_session.library.catalog().items[index].key, anchor);
    assert!(reopened.scene.render.hits.iter().any(|hit| hit.index == reopened.scene.current));
}

#[test]
fn applied_and_browsed_positions_remain_distinct() {
    let mut app = browsing_app("slices");
    app.config.set_key(skwd_config::keys::general::CLOSE_ON_SELECTION, json!(false));
    let _ = crate::app::helpers::apply_task(&mut app, 2);
    app.scene.set_current(75, 100);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    app.save_browse_position();
    assert_eq!(reopen(&app).scene.current, 75);
    app.config.save_key(START_POSITION, json!("applied"));
    assert_eq!(reopen(&app).scene.current, 2);
    app.config.save_key(START_POSITION, json!("beginning"));
    assert_eq!(reopen(&app).scene.current, 0);
}

#[test]
fn apply_remembers_identity_across_restart_and_reordering() {
    let mut app = saved_selection(true, "");
    seed(&mut app, &[keyed_wall("a", "static", 10, 0), keyed_wall("b", "static", 20, 0)]);
    let _ = crate::app::helpers::apply_task(&mut app, 1);
    let data = serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    let mut config = Config::from_data(data);
    config.config_path = app.config.config_path.clone();
    let mut reopened =
        App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording());
    seed(
        &mut reopened,
        &[
            keyed_wall("new", "static", 5, 0),
            keyed_wall("a", "static", 10, 0),
            keyed_wall("b", "static", 20, 0),
        ],
    );
    assert_eq!(reopened.scene.current, 2);
    assert_eq!(filtered_names(&reopened)[reopened.scene.current], "b");
}

#[test]
fn disabled_selection_starts_at_beginning_and_does_not_save() {
    let mut app = test_app();
    assert_eq!(app.config.str_path(START_POSITION), "beginning");
    app.config.set_key(LAST_APPLIED_KEY, json!("static:b"));
    seed(&mut app, &[keyed_wall("a", "static", 10, 0), keyed_wall("b", "static", 20, 0)]);
    assert_eq!(app.scene.current, 0);
    let _ = crate::app::helpers::apply_task(&mut app, 0);
    assert_eq!(app.config.str_path(LAST_APPLIED_KEY), "static:b");
}

#[test]
fn removed_or_filtered_selection_starts_at_beginning() {
    for key in ["static:removed", "video:b"] {
        let mut app = saved_selection(true, key);
        app.library_session.filters.kind = "static".into();
        seed(&mut app, &[keyed_wall("a", "static", 10, 0), keyed_wall("b", "video", 20, 0)]);
        assert_eq!(app.scene.current, 0);
        assert_eq!(filtered_names(&app), ["a"]);
        assert!(app.library_session.selection_restored);
    }
}

#[test]
fn selection_waits_for_nonempty_catalogue() {
    let mut app = saved_selection(true, "static:b");
    seed(&mut app, &[]);
    assert!(!app.library_session.selection_restored);
    seed(&mut app, &[keyed_wall("a", "static", 10, 0), keyed_wall("b", "static", 20, 0)]);
    assert_eq!(app.scene.current, 1);
}

#[test]
fn refresh_does_not_restore_again() {
    let mut app = saved_selection(true, "static:b");
    let walls = [keyed_wall("a", "static", 10, 0), keyed_wall("b", "static", 20, 0)];
    seed(&mut app, &walls);
    assert_eq!(app.scene.current, 1);
    app.scene.set_current(0, 2);
    seed(&mut app, &walls);
    assert_eq!(app.scene.current, 0);
}

#[test]
fn restore_does_not_override_user_navigation() {
    let mut app = saved_selection(true, "static:b");
    app.scene.set_current(0, 1);
    seed(&mut app, &[keyed_wall("a", "static", 10, 0), keyed_wall("b", "static", 20, 0)]);
    assert_eq!(app.scene.current, 0);
}

#[test]
fn restored_selection_is_visible_in_each_picker_mode() {
    for (mode, grid_layout) in [
        ("slices", "uniform"),
        ("hex", "uniform"),
        ("sandy", "uniform"),
        ("hand", "uniform"),
        ("wall", "uniform"),
        ("wall", "brick"),
        ("wall", "masonry"),
        ("wall", "justified"),
        ("wall", "editorial"),
        ("wall", "cylinder"),
    ] {
        let mut app = saved_selection(true, "static:wall-75");
        app.config.set_key(skwd_config::keys::selector::DISPLAY_MODE, json!(mode));
        app.config.set_key(skwd_config::keys::selector::GRID_LAYOUT, json!(grid_layout));
        let mut app = App::with_config_using(app.config, |_| {
            crate::infrastructure::ipc::DaemonClient::recording()
        });
        app.scene.begin_open_fade();
        let walls: Vec<_> = (0..100)
            .map(|index| keyed_wall(&format!("wall-{index}"), "static", index, 0))
            .collect();
        seed(&mut app, &walls);
        assert_eq!(filtered_names(&app)[app.scene.current], "wall-75", "{mode}");
        let mut now = Instant::now();
        tick_frames(&mut app, &mut now, 120);
        assert!(
            app.scene.render.hits.iter().any(|hit| hit.index == app.scene.current),
            "{mode}: selected {} outside {:?}",
            app.scene.current,
            app.scene.visible_range()
        );
    }
}

#[test]
fn first_hand_card_survives_restoration_and_catalogue_refresh() {
    let mut app = browsing_app("hand");
    app.scene.set_current(0, 100);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    app.on_hidden();
    let mut reopened = reopen(&app);
    assert_eq!(reopened.scene.current, 0);
    reopened.on_list(app.library_session.library.catalog().clone());
    tick_frames(&mut reopened, &mut now, 120);
    assert_eq!(reopened.scene.current, 0);
    assert!(reopened.scene.render.hits.iter().any(|hit| hit.index == 0));
}

#[test]
fn malformed_saved_browsing_position_starts_normally() {
    for value in [json!(null), json!("invalid"), json!({"selection": "static:wall-75"})] {
        let mut app = browsing_app("slices");
        app.config.save_key(skwd_config::keys::selector::LAST_BROWSE_POSITION, value);
        assert_eq!(reopen(&app).scene.current, 0);
    }
}

#[test]
fn changed_grid_layout_restores_visible_anchor() {
    let mut app = browsing_app("wall");
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    app.scene.grid_scroll(15.0);
    tick_frames(&mut app, &mut now, 120);
    app.save_browse_position();
    let anchor =
        skwd_config::get(app.config.root(), skwd_config::keys::selector::LAST_BROWSE_POSITION)
            .unwrap()["anchor"]
            .as_str()
            .unwrap()
            .to_owned();
    app.config.save_key(skwd_config::keys::selector::GRID_LAYOUT, json!("masonry"));
    let mut reopened = reopen(&app);
    tick_frames(&mut reopened, &mut now, 120);
    let index = reopened.library_session.filtered[reopened.scene.current] as usize;
    assert_eq!(reopened.library_session.library.catalog().items[index].key, anchor);
    assert!(reopened.scene.render.hits.iter().any(|hit| hit.index == reopened.scene.current));
}
