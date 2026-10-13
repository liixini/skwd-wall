use super::*;

#[test]
fn filter_swap_startup() {
    let dir = std::env::temp_dir().join(format!(
        "skwd-flipms-{}-{}",
        std::process::id(),
        TEST_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut config = Config::from_data(json!({
        "paths": { "cache": dir.join("cache").to_string_lossy(), "wallpaper": "/wp",
            "videoWallpaper": "/vids" },
        "motion": { "slowMs": 1600.0 }
    }));
    config.config_path = dir.join("config.json");
    let app =
        App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording());
    assert!((app.scene.filter_swap_ms() - 1600.0).abs() < 0.01);
}

#[test]
fn open_fade_startup() {
    let dir = std::env::temp_dir().join(format!(
        "skwd-openfade-{}-{}",
        std::process::id(),
        TEST_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut config = Config::from_data(json!({
        "paths": { "cache": dir.join("cache").to_string_lossy(), "wallpaper": "/wp",
            "videoWallpaper": "/vids" },
        "motion": { "standardMs": 777.0 }
    }));
    config.config_path = dir.join("config.json");
    let app =
        App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording());
    assert!((app.scene.open_fade_ms() - 777.0).abs() < 0.01);
    assert!(app.scene.open_fade() < 1.0);
}

#[test]
fn motion_weights_reach_all_state() {
    use crate::frontend::animation::MotionTier;

    let dir = std::env::temp_dir().join(format!(
        "skwd-motion-policy-{}-{}",
        std::process::id(),
        TEST_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut config = Config::from_data(json!({
        "paths": {
            "cache": dir.join("cache").to_string_lossy(),
            "wallpaper": "/wp",
            "videoWallpaper": "/vids"
        },
        "motion": { "fastMs": 90.0, "standardMs": 330.0, "slowMs": 810.0 }
    }));
    config.config_path = dir.join("config.json");
    let mut app =
        App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording());

    assert_eq!(app.scene.motion_duration_ms(MotionTier::Fast), 90.0);
    assert_eq!(app.scene.motion_duration_ms(MotionTier::Standard), 330.0);
    assert_eq!(app.scene.motion_duration_ms(MotionTier::Slow), 810.0);
    assert_eq!(app.panels.settings.control_anim.duration_ms(), 90.0);
    assert_eq!(app.panels.settings.entrance.duration_ms(), 330.0);
    assert!((app.chrome.bar.menu_scroll.duration_ms() - 330.0).abs() < 0.01);
    assert_eq!(app.source_browser.entrance.duration_ms(), 810.0);
    assert_eq!(app.source_browser.wall.scene.motion_duration_ms(MotionTier::Standard), 330.0);
    assert_eq!(app.tags.cloud_entrance.duration_ms(), 330.0);
    assert!((app.tags.cloud_scroll.duration_ms() - 330.0).abs() < 0.01);
    assert_eq!(app.theme.fade_t.duration_ms(), 330.0);

    app.panels.audio =
        Some(crate::frontend::audio_panel::AudioPanel::new_with_motion(app.motion_profile()));
    assert_eq!(app.panels.audio.as_ref().unwrap().open.duration_ms(), 90.0);
}

#[test]
fn set_view_mode() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    assert_eq!(app.scene.mode, Mode::Slices);
    let _ = update(&mut app, Message::SetViewMode(String::from("grid")));
    assert_eq!(app.scene.mode, Mode::Grid);
    assert_eq!(app.config.display_mode(), "wall");
    let text = std::fs::read_to_string(&app.config.config_path).expect("mode saved to disk");
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["components"]["wallpaperSelector"]["displayMode"], "wall");
    let _ = update(&mut app, Message::SetViewMode(String::from("hex")));
    assert_eq!(app.scene.mode, Mode::Hex);
    let _ = update(&mut app, Message::SetViewMode(String::from("collection")));
    assert_eq!(app.scene.mode, Mode::Collection);
    assert_eq!(app.config.display_mode(), "collection");
}

#[test]
fn shader_mode_degrades() {
    let cfg = Config::from_data(json!({
        "components": {"wallpaperSelector": {"displayMode": "shader:plasma"}}
    }));
    assert_eq!(layout_params(&cfg).mode, Mode::Slices);
    for (name, expected) in [
        ("grid", Mode::Grid),
        ("wall", Mode::Grid),
        ("hex", Mode::Hex),
        ("sandy", Mode::Sandy),
        ("nova", Mode::Sandy),
        ("hand", Mode::Hand),
        ("depth", Mode::Depth),
    ] {
        let cfg = Config::from_data(json!({
            "components": {"wallpaperSelector": {"displayMode": name}}
        }));
        assert_eq!(layout_params(&cfg).mode, expected, "{name}");
    }
    for name in ["spiral", "mosaic", "cascade", "warp", "fluid", "lens", "shelves"] {
        let cfg = Config::from_data(json!({
            "components": {"wallpaperSelector": {"displayMode": name}}
        }));
        assert_eq!(layout_params(&cfg).mode, Mode::Slices, "{name}");
    }
}

#[test]
fn apply_static_neighbors() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    let _ = update(&mut app, Message::ApplyCurrent);
    let calls = drain_calls(&app);
    let apply = calls.iter().find(|(method, _)| method == "wall.apply").expect("wall.apply sent");
    assert_eq!(apply.1["type"], "static");
    assert_eq!(apply.1["path"], "/wp/a.png");
    assert_eq!(apply.1["neighbors"], json!(["/wp/b.png", "/wp/c.png"]));
}

#[test]
fn apply_we_item() {
    let mut app = test_app();
    let mut we = wall("workshop item", "we", 1, 0);
    we["we_id"] = json!("w42");
    seed(&mut app, &[we, wall("b.png", "static", 2, 0)]);
    let _ = update(&mut app, Message::ApplyCurrent);
    let calls = drain_calls(&app);
    let apply = calls.iter().find(|(method, _)| method == "wall.apply").expect("wall.apply sent");
    assert_eq!(apply.1["type"], "we");
    assert_eq!(apply.1["we_id"], "w42");
    assert_eq!(apply.1["screens"], json!([]));
    assert!(apply.1.get("neighbors").is_none());
    assert!(apply.1.get("path").is_none());
}

#[test]
fn scene_props_load_and_write() {
    let mut app = test_app();
    let mut we = wall("workshop item", "we", 1, 0);
    we["we_id"] = json!("scene-42");
    seed(&mut app, &[we]);

    let _ = update(&mut app, Message::OpenSceneProps);
    let panel = app.panels.scene_properties.as_ref().unwrap();
    assert_eq!((panel.we_id.as_str(), panel.title.as_str()), ("scene-42", "workshop item"));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| {
        method == wall_proto::rpc::WALL_WE_PROPERTIES && params["we_id"] == "scene-42"
    }));

    app.on_result(
        Pending::SceneProperties { we_id: "scene-42".into(), revision: 0, writes: Vec::new() },
        &json!({
            "we_id": "scene-42",
            "properties": [{
                "name": "glow",
                "label": "Glow",
                "kind": "bool",
                "value": true,
                "default": true
            }]
        }),
    );
    let _ = update(
        &mut app,
        Message::SceneProps(crate::frontend::scene_properties::ScenePropMsg::Toggle("glow".into())),
    );
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| {
        method == wall_proto::rpc::WALL_SET_WE_PROPERTY
            && params["we_id"] == "scene-42"
            && params["name"] == "glow"
            && params["value"] == false
    }));
}

#[test]
fn collect_neighbors_caps() {
    let mut app = test_app();
    let walls: Vec<Value> =
        (0..30).map(|i| wall(&format!("i{i:02}.png"), "static", 5, 0)).collect();
    seed(&mut app, &walls);
    let picks = collect_neighbors(&app, "/wp/i05.png");
    let expected: Vec<String> =
        [6, 4, 7, 3, 8, 2, 9, 1, 10, 0, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
            .iter()
            .map(|i| format!("/wp/i{i:02}.png"))
            .collect();
    assert_eq!(picks, expected);
    assert_eq!(picks.len(), 20);
    assert!(!picks.contains(&String::from("/wp/i05.png")));
    assert!(collect_neighbors(&app, "/wp/unknown.png").is_empty());
}

#[test]
fn apply_params_kinds() {
    use crate::domain::library::catalog::Wallpaper;
    let img =
        Wallpaper { kind: WallpaperKind::Static, path: "/wp/a.png".into(), ..Default::default() };
    let params = apply_params(&img, vec!["/wp/b.png".into()]);
    assert_eq!(params["type"], "static");
    assert_eq!(params["neighbors"][0], "/wp/b.png");

    let we = Wallpaper { kind: WallpaperKind::We, we_id: "111".into(), ..Default::default() };
    assert_eq!(apply_params(&we, vec![])["type"], "we");
}

#[test]
fn keyboard_nav_clamps() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    let _ = update(&mut app, Message::KeyPrev);
    assert_eq!(app.scene.current, 0);
    let _ = update(&mut app, Message::KeyNext);
    let _ = update(&mut app, Message::KeyNext);
    let _ = update(&mut app, Message::KeyNext);
    assert_eq!(app.scene.current, 2);
    app.panels.settings.open = true;
    let _ = update(&mut app, Message::KeyPrev);
    assert_eq!(app.scene.current, 2);
    drain_calls(&app);
    let _ = update(&mut app, Message::ApplyCurrent);
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.apply"));
}

#[test]
fn hand_layout_params_follow_config() {
    use crate::frontend::scene::hand::{Axis, Cut, DealMode, Variance};
    let cfg = Config::from_data(json!({}));
    let hand = layout_params(&cfg).extra.hand;
    assert_eq!(hand.count, 5);
    assert_eq!(hand.deal_mode, DealMode::Cycle);
    assert_eq!(hand.moves, [true; 5]);
    assert_eq!(hand.axis, Axis::Rows);
    assert!(hand.ghosts && hand.backdrop && !hand.bob);
    assert!(hand.reveal_fill);
    let cfg = Config::from_data(json!({
        "components": {"wallpaperSelector": {
            "displayMode": "hand", "handMove": "cascade", "handCut": "slant",
            "handCutVariance": "soft", "handRibbonAxis": "columns", "handRibbons": 8,
            "handSpeed": 150, "handStageY": 20, "handRevealFill": false
        }}
    }));
    let params = layout_params(&cfg);
    assert_eq!(params.mode, Mode::Hand);
    assert_eq!(params.extra.hand.deal_mode, DealMode::Cycle);
    assert_eq!(params.extra.hand.moves, [false, true, false, false, false]);
    assert_eq!(params.extra.hand.cut, Cut::Slant);
    assert_eq!(params.extra.hand.variance, Variance::Soft);
    assert_eq!(params.extra.hand.axis, Axis::Columns);
    assert_eq!(params.extra.hand.ribbons, 8);
    assert!((params.extra.hand.speed - 1.5).abs() < 1e-6);
    assert!((params.extra.hand.offset_y - 0.2).abs() < 1e-6);
    assert!(!params.extra.hand.reveal_fill);
}

fn compact_app(selector: &serde_json::Value) -> App {
    let dir = std::env::temp_dir().join(format!(
        "skwd-compact-{}-{}",
        std::process::id(),
        TEST_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut config = Config::from_data(json!({
        "paths": { "cache": dir.join("cache").to_string_lossy(), "wallpaper": "/wp",
            "videoWallpaper": "/vids" },
        "components": { "wallpaperSelector": selector }
    }));
    config.config_path = dir.join("config.json");
    App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording())
}

fn open_window(app: &mut App, width: f32, height: f32) {
    let _ = update(app, Message::WindowOpened { id: iced::window::Id::unique(), width, height });
}

#[test]
fn compact_defaults_follow_first_window() {
    let mut app = compact_app(&json!({}));
    assert_eq!(app.scene.sp.slice_h, 520.0);
    let (min_w, min_h) = crate::frontend::ui::MIN_VIEWPORT;
    open_window(&mut app, min_w, min_h);
    assert_eq!(app.scene.sp.slice_h, 360.0);
    assert_eq!(app.scene.sp.visible_count, 8);
    assert_eq!(app.scene.sp.slice_w, 90.0);
    assert_eq!(app.scene.gp.thumb_w, 220.0);
    assert_eq!(app.scene.hp.r, 100.0);
    assert_eq!(app.scene.xp.sandy.slice_w, 68.0);
    assert_eq!(app.config.slice_height(), 360.0);
}

#[test]
fn regular_defaults_stay_on_wide_window() {
    let mut app = compact_app(&json!({}));
    open_window(&mut app, 1920.0, 1080.0);
    assert_eq!(app.scene.sp.slice_h, 520.0);
    assert_eq!(app.scene.sp.visible_count, 12);
    assert_eq!(app.scene.gp.thumb_w, 300.0);
}

#[test]
fn saved_layout_keys_survive_compact_window() {
    let mut app = compact_app(&json!({ "sliceHeight": 400.0, "gridThumbWidth": 260.0 }));
    let (min_w, min_h) = crate::frontend::ui::MIN_VIEWPORT;
    open_window(&mut app, min_w, min_h);
    assert_eq!(app.scene.sp.slice_h, 400.0);
    assert_eq!(app.scene.sp.visible_count, 8);
    assert_eq!(app.scene.gp.thumb_w, 260.0);
    assert_eq!(app.scene.gp.thumb_h, 124.0);
}

#[test]
fn picker_monitor_apply_scopes_every_wallpaper_kind() {
    for kind in ["static", "video", "we"] {
        let mut app = test_app();
        app.config.save_key(skwd_config::keys::general::APPLY_ON_PICKER_MONITOR, json!(true));
        app.runtime_state.picker_output = Some("DP-2".into());
        let mut item = wall("wallpaper", kind, 1, 0);
        item["we_id"] = json!("42");
        seed(&mut app, &[item]);
        let _ = update(&mut app, Message::ApplyCurrent);
        let calls = drain_calls(&app);
        let apply = calls.iter().find(|(method, _)| method == "wall.apply").unwrap();
        assert_eq!(apply.1["output"], "DP-2");
        assert!(apply.1.get("screens").is_none());
        assert!(apply.1.get("override_locks").is_none());
        assert_eq!(app.daemon.last_wallpaper.as_ref().unwrap()["output"], "DP-2");
    }
}

#[test]
fn picker_monitor_apply_default_keeps_existing_targets() {
    let mut app = test_app();
    app.runtime_state.picker_output = Some("DP-2".into());
    seed(&mut app, &[wall("wallpaper", "static", 1, 0)]);
    let _ = update(&mut app, Message::ApplyCurrent);
    let calls = drain_calls(&app);
    let apply = calls.iter().find(|(method, _)| method == "wall.apply").unwrap();
    assert!(apply.1.get("output").is_none());
}

#[test]
fn picker_monitor_apply_requires_known_monitor() {
    let mut app = test_app();
    app.config.save_key(skwd_config::keys::general::APPLY_ON_PICKER_MONITOR, json!(true));
    app.config.save_key(skwd_config::keys::selector::START_POSITION, json!("applied"));
    app.config.save_key(skwd_config::keys::selector::LAST_APPLIED_KEY, json!("previous"));
    seed(&mut app, &[wall("wallpaper", "static", 1, 0)]);
    let _ = update(&mut app, Message::ApplyCurrent);
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.apply"));
    assert_eq!(app.config.str_path(skwd_config::keys::selector::LAST_APPLIED_KEY), "previous");
    assert!(app.runtime_state.toast.is_some());
}

#[test]
fn optional_parallax_reaches_each_layout_independently() {
    for (slices, hex) in [(false, false), (true, false), (false, true), (true, true)] {
        let config = Config::from_data(json!({"components":{"wallpaperSelector":{
            "sliceParallax":slices,"hexParallax":hex
        }}}));
        let params = layout_params(&config);
        assert_eq!(params.slices.parallax, slices);
        assert_eq!(params.hex.parallax, hex);
    }
    let defaults = layout_params(&Config::from_data(json!({})));
    assert!(!defaults.slices.parallax && !defaults.hex.parallax);
}

#[test]
fn depth_settings_do_not_change_slices() {
    let mut config = Config::from_data(json!({"components":{"wallpaperSelector":{
        "displayMode":"depth", "depthHeight":650, "depthCount":7, "depthCorners":24,
        "depthSkew":0, "sliceHeight":360, "visibleCount":10
    }}}));
    let depth = super::super::startup::layout_params(&config);
    assert_eq!(depth.slices.slice_h, 650.0);
    assert_eq!(depth.slices.visible_count, 7);
    assert_eq!(depth.slices.corners, [24.0; 4]);
    config.set_key(skwd_config::keys::selector::DISPLAY_MODE, json!("slices"));
    let slices = super::super::startup::layout_params(&config);
    assert_eq!(slices.slices.slice_h, 360.0);
    assert_eq!(slices.slices.visible_count, 10);
}

#[test]
fn depth_default_profile_reaches_the_scene_and_matches_render_defaults() {
    let config =
        Config::from_data(json!({"components":{"wallpaperSelector":{"displayMode":"depth"}}}));
    let params = layout_params(&config);
    let fallback = crate::frontend::scene::layout::DepthParams::default();
    assert_eq!(params.slices.slice_h, 520.0);
    assert_eq!(params.slices.visible_count, 5);
    assert_eq!(params.slices.corners, [0.0; 4]);
    assert_eq!(params.slices.skew, 0.0);
    assert!(!config.depth_shadows());
    assert!(!params.extra.depth.selection_frame);
    assert_eq!(params.extra.depth.width, 280.0);
    assert_eq!(params.extra.depth.spacing, 280.0);
    assert_eq!(params.extra.depth.falloff, 0.05);
    assert_eq!(params.extra.depth.navigation_ms, 1000.0);
    assert_eq!(params.extra.depth.selection_frame, fallback.selection_frame);
    assert_eq!(params.extra.depth.width, fallback.width);
    assert_eq!(params.extra.depth.spacing, fallback.spacing);
    assert_eq!(params.extra.depth.falloff, fallback.falloff);
    assert_eq!(params.extra.depth.navigation_ms, fallback.navigation_ms);
}
