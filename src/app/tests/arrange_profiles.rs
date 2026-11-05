use super::*;
use skwd_config::keys::{filter_bar, motion, selector};
use skwd_config::picker_layout::{Alignment, Component, Offset, Position, Reference, Size};

const VIEWPORT: Size = Size { width: 1280.0, height: 720.0 };
const PORTRAIT: Size = Size { width: 720.0, height: 1280.0 };

#[test]
fn output_edits_stay_isolated() {
    let mut app = test_app();
    app.config.set_key(selector::FRAME_X, json!(10));
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    app.config.set_key(selector::FRAME_X, json!(25));
    assert_eq!(app.config.num_path(selector::FRAME_X), 25.0);
    assert_eq!(skwd_config::get(app.config.document(), selector::FRAME_X), Some(&json!(10)));
    app.config.select_layout_output(Some(String::from("DP-2")), PORTRAIT);
    assert_eq!(app.config.num_path(selector::FRAME_X), 10.0);
    app.config.set_key(selector::FRAME_X, json!(75));
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    assert_eq!(app.config.num_path(selector::FRAME_X), 25.0);
    assert_eq!(app.config.layout_outputs(), ["DP-1", "DP-2"]);
}

#[test]
fn persistence_keeps_global_layout() {
    let mut app = test_app();
    app.config.set_key(selector::FRAME_X, json!(10));
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    app.config.set_key(selector::FRAME_X, json!(25));
    app.config.persist();
    let saved: Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    assert_eq!(skwd_config::get(&saved, selector::FRAME_X), Some(&json!(10)));
    let mut reopened = Config::from_data(saved);
    reopened.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    assert_eq!(reopened.num_path(selector::FRAME_X), 25.0);
    reopened.select_layout_output(None, VIEWPORT);
    assert_eq!(reopened.num_path(selector::FRAME_X), 10.0);
}

#[test]
fn copying_preserves_disconnected() {
    let mut app = test_app();
    app.config.select_layout_output(Some(String::from("disconnected")), VIEWPORT);
    app.config.set_key(selector::FRAME_X, json!(80));
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    app.config.set_key(selector::GRID_COLUMNS, json!(7));
    app.config.set_key(selector::GRID_CORNER_RADIUS, json!(18));
    app.config.set_key(filter_bar::BUTTON_ORDER, json!("folder,type:we,settings"));
    let profile = app.config.layout_profile("DP-1", VIEWPORT).unwrap();
    app.config.set_layout_profile("DP-2", &profile).unwrap();
    app.config.select_layout_output(Some(String::from("DP-2")), VIEWPORT);
    assert_eq!(app.config.num_path(selector::GRID_COLUMNS), 7.0);
    assert_eq!(app.config.num_path(selector::GRID_CORNER_RADIUS), 18.0);
    assert_eq!(app.config.str_path(filter_bar::BUTTON_ORDER), "folder,type:we,settings");
    app.config.select_layout_output(Some(String::from("disconnected")), VIEWPORT);
    assert_eq!(app.config.num_path(selector::FRAME_X), 80.0);
}

#[test]
fn relationships_survive_switching() {
    let mut app = test_app();
    let position = Position {
        reference: Reference::Picker,
        alignment: Alignment::Top,
        offset: Offset { x: 0.0, y: -24.0 },
    };
    assert!(app.config.set_position(Component::Search, Some(position)).is_err());
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    app.config.set_position(Component::Search, Some(position)).unwrap();
    app.config.select_layout_output(Some(String::from("DP-2")), PORTRAIT);
    assert_eq!(app.config.position(Component::Search), None);
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    assert_eq!(app.config.position(Component::Search), Some(position));
    app.config.set_position(Component::Search, None).unwrap();
    assert_eq!(app.config.position(Component::Search), None);
}

#[test]
fn motion_stays_global() {
    let mut app = test_app();
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    app.config.set_key(motion::REDUCED, json!(true));
    app.config.set_key(filter_bar::LAST_KIND, json!("video"));
    app.config.set_key(selector::FRAME_X, json!(25));
    app.config.select_layout_output(Some(String::from("DP-2")), VIEWPORT);
    assert!(app.config.reduced_motion());
    assert_eq!(app.config.str_path(filter_bar::LAST_KIND), "video");
    let profile = app.config.layout_profile("DP-1", VIEWPORT).unwrap();
    assert!(!profile.values.contains_key(motion::REDUCED));
    assert!(!profile.values.contains_key(filter_bar::LAST_KIND));
}

#[test]
fn transient_restores_canonical() {
    let mut app = test_app();
    app.config.set_key(selector::FRAME_X, json!(10));
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    let original = app.config.begin_transient();
    app.config.set_key(selector::FRAME_X, json!(25));
    app.config.restore_transient(original);
    assert_eq!(app.config.num_path(selector::FRAME_X), 10.0);
    assert!(app.config.layout_outputs().is_empty());
}

#[test]
fn external_profile_merges_safely() {
    let mut app = test_app();
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    app.config.set_key(selector::FRAME_X, json!(10));
    app.config.persist();
    let original = app.config.begin_transient();
    app.config.set_key(selector::FRAME_X, json!(25));
    let mut external = Config::from_data(original.clone());
    external.select_layout_output(Some(String::from("DP-2")), VIEWPORT);
    external.set_key(selector::FRAME_X, json!(75));
    std::fs::write(&app.config.config_path, serde_json::to_vec(external.document()).unwrap())
        .unwrap();
    app.config.finish_layout(&original, true).unwrap();
    assert_eq!(app.config.num_path(selector::FRAME_X), 25.0);
    app.config.select_layout_output(Some(String::from("DP-2")), VIEWPORT);
    assert_eq!(app.config.num_path(selector::FRAME_X), 75.0);
}

#[test]
fn batch_reports_invalid_profile() {
    let mut app = test_app();
    app.config.set_key(skwd_config::picker_layout::ROOT, json!({"version": 2, "outputs": {}}));
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    let original = app.config.document().clone();
    assert!(app.config.set_keys(&[(selector::FRAME_X, json!(25))]).is_err());
    assert_eq!(app.config.document(), &original);
}

#[test]
fn profile_batch_cost() {
    let mut app = test_app();
    for output in ["DP-1", "DP-2", "disconnected"] {
        app.config.select_layout_output(Some(output.to_owned()), VIEWPORT);
        app.config.set_key(selector::FRAME_X, json!(10));
    }
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    let mut samples = Vec::new();
    for step in 0..2000 {
        let start = Instant::now();
        app.config
            .set_keys(&[
                (selector::FRAME_X, json!(step % 100)),
                (selector::FRAME_Y, json!(step % 80)),
                (selector::FRAME_WIDTH, json!(80)),
                (selector::FRAME_HEIGHT, json!(60)),
                (selector::FRAME_ENABLED, json!(true)),
            ])
            .unwrap();
        std::hint::black_box(app.config.num_path(selector::FRAME_X));
        std::hint::black_box(app.config.position(Component::Search));
        samples.push(start.elapsed().as_micros());
    }
    samples.sort_unstable();
    eprintln!(
        "layout batch with effective read: median={}us p95={}us max={}us",
        samples[1000], samples[1900], samples[1999]
    );
    assert_eq!(app.config.num_path(selector::FRAME_X), 99.0);
}

#[test]
fn monitor_adopts_connector_profile() {
    let mut app = test_app();
    app.config.select_layout_output(Some(String::from("DP-1")), VIEWPORT);
    app.config.set_key(selector::FRAME_X, json!(25));
    let before = app.config.document().clone();
    let monitor = Some(String::from("Dell P2723DE 317ZNR3"));
    app.config.select_layout_monitor(monitor, Some(String::from("DP-1")), VIEWPORT);
    assert_eq!(app.config.layout_output(), Some("Dell P2723DE 317ZNR3"));
    assert_eq!(app.config.num_path(selector::FRAME_X), 25.0);
    assert_eq!(app.config.document(), &before);
    app.config.set_key(selector::FRAME_Y, json!(30));
    assert_eq!(app.config.layout_outputs(), ["Dell P2723DE 317ZNR3"]);
    assert_eq!(app.config.num_path(selector::FRAME_X), 25.0);
    assert_eq!(app.config.num_path(selector::FRAME_Y), 30.0);
}

#[test]
fn new_monitor_borrows_closest_layout() {
    let mut app = test_app();
    for (output, viewport, x) in [
        ("Square", Size { width: 1280.0, height: 1024.0 }, 25),
        ("Exact", Size { width: 1920.0, height: 1080.0 }, 40),
        ("Tall", Size { width: 1080.0, height: 1920.0 }, 60),
    ] {
        app.config.select_layout_output(Some(String::from(output)), viewport);
        app.config.set_key(selector::FRAME_X, json!(x));
    }
    let before = app.config.document().clone();
    let pick = |app: &mut App, viewport: Size| {
        app.config.select_layout_output(Some(String::from("New")), viewport);
        app.config.num_path(selector::FRAME_X)
    };
    assert_eq!(pick(&mut app, Size { width: 1920.0, height: 1080.0 }), 40.0);
    assert_eq!(pick(&mut app, Size { width: 2560.0, height: 1440.0 }), 40.0);
    assert_eq!(pick(&mut app, Size { width: 1280.0, height: 1000.0 }), 25.0);
    assert_eq!(pick(&mut app, Size { width: 1200.0, height: 1920.0 }), 60.0);
    assert_eq!(app.config.document(), &before);
    assert_eq!(app.config.layout_outputs(), ["Exact", "Square", "Tall"]);
}
