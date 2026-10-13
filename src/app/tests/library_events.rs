use super::*;

#[test]
fn cached_event_insert() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 10, 0),
            wall("b.png", "static", 20, 0),
            wall("c.png", "static", 30, 0),
        ],
    );
    app.on_event("skwd.wall.cached", &wall("new.png", "static", 15, 0));
    assert_eq!(app.library_session.library.catalog().items.len(), 4);
    assert_eq!(filtered_names(&app), ["a.png", "new.png", "b.png", "c.png"]);
    assert!(
        app.library_session
            .filtered
            .iter()
            .all(|&idx| (idx as usize) < app.library_session.library.catalog().items.len())
    );
    app.on_event("skwd.wall.cached", &wall("a.png", "static", 10, 0));
    assert_eq!(app.library_session.library.catalog().items.len(), 4);
    assert_eq!(app.library_session.filtered.len(), 4);
}

#[test]
fn cached_event_filtered_out() {
    let mut app = test_app();
    seed(&mut app, &[wall("nature/a.png", "static", 10, 0), wall("city/b.png", "static", 20, 0)]);
    let _ = update(&mut app, Message::SetFolder(String::from("nature")));
    assert_eq!(filtered_names(&app), ["nature/a.png"]);
    app.on_event("skwd.wall.cached", &wall("city/new.png", "static", 15, 0));
    assert_eq!(app.library_session.library.catalog().items.len(), 3);
    assert_eq!(filtered_names(&app), ["nature/a.png"]);
    app.on_event("skwd.wall.cached", &wall("nature/new.png", "static", 5, 0));
    assert_eq!(filtered_names(&app), ["nature/new.png", "nature/a.png"]);
}

#[test]
fn cached_event_playlist() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 10, 0)]);
    let mut keys = std::collections::HashSet::new();
    keys.insert(String::from("a.png"));
    app.library_session.playlist_filter = Some((1, String::from("pl"), keys));
    app.on_event("skwd.wall.cached", &wall("b.png", "static", 20, 0));
    assert_eq!(app.library_session.library.catalog().items.len(), 2);
    assert_eq!(filtered_names(&app), ["a.png"]);
}

#[test]
fn removal_consistency() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 10, 0),
            wall("b.png", "static", 20, 0),
            wall("c.png", "static", 30, 0),
        ],
    );
    app.on_event("skwd.wall.removed", &json!({"key": "b.png"}));
    assert_eq!(app.library_session.library.catalog().items.len(), 2);
    assert_eq!(filtered_names(&app), ["a.png", "c.png"]);
    assert!(
        app.library_session
            .filtered
            .iter()
            .all(|&idx| (idx as usize) < app.library_session.library.catalog().items.len())
    );
    app.on_event("skwd.wall.removed", &json!({"key": "ghost.png"}));
    assert_eq!(app.library_session.library.catalog().items.len(), 2);
    app.on_event("skwd.wall.file_removed", &json!({"name": "c.png"}));
    assert_eq!(filtered_names(&app), ["a.png"]);
    assert!(
        app.library_session
            .filtered
            .iter()
            .all(|&idx| (idx as usize) < app.library_session.library.catalog().items.len())
    );
}

#[test]
fn download_done_routing() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Steam);
    browser.session.items.push(browser_item("999"));
    browser.session.pending_apply = Some(String::from("999"));
    app.source_browser.browser = Some(browser);
    drain_calls(&app);
    app.on_event("skwd.wall.download", &json!({"id": "999", "status": "done", "path": "/dl/999"}));
    let calls = drain_calls(&app);
    let apply = calls.iter().find(|(method, _)| method == "wall.apply").expect("steam apply sent");
    assert_eq!(apply.1["type"], "we");
    assert_eq!(apply.1["we_id"], "999");
    assert!(app.source_browser.browser.as_ref().unwrap().session.pending_apply.is_none());

    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("w1"));
    browser.session.pending_apply = Some(String::from("w1"));
    app.source_browser.browser = Some(browser);
    app.on_event(
        "skwd.wall.download",
        &json!({"id": "w1", "status": "done", "path": "/dl/w1.png"}),
    );
    let calls = drain_calls(&app);
    let apply =
        calls.iter().find(|(method, _)| method == "wall.apply").expect("wallhaven apply sent");
    assert_eq!(apply.1["type"], "static");
    assert_eq!(apply.1["path"], "/dl/w1.png");
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(browser.session.items[0].downloaded);
    assert_eq!(browser.session.items[0].downloaded_path.as_deref(), Some("/dl/w1.png"));
}

#[test]
fn esc_topmost_first() {
    use crate::app::overlay::Overlay;
    let mut app = test_app();
    app.input.help_open = true;
    app.source_browser.browser = Some(Browser::new(Source::Wallhaven));
    app.panels.settings.open = true;
    app.tags.cloud_open = true;

    assert_eq!(app.topmost_overlay(), Some(Overlay::Help));
    for expect in [Overlay::Help, Overlay::Browser, Overlay::TagCloud, Overlay::Settings] {
        assert_eq!(app.topmost_overlay(), Some(expect));
        assert!(app.close_topmost_overlay());
    }
    assert_eq!(app.topmost_overlay(), None);
    assert!(!app.close_topmost_overlay());
}

#[test]
fn overlay_close_clears() {
    use crate::app::overlay::Overlay;
    let mut app = test_app();
    app.tags.cloud_open = true;
    app.tags.semantic.search = String::from("for");
    app.tags.tag_search = String::from("forest");
    app.close_overlay(Overlay::TagCloud);
    assert!(!app.tags.cloud_open);
    assert!(app.tags.semantic.search.is_empty());
    assert!(app.tags.tag_search.is_empty());

    app.tags.mode = true;
    app.tags.mass_tags.push(String::from("x"));
    app.close_overlay(Overlay::TagMode);
    assert!(!app.tags.mode);
    assert!(app.tags.mass_tags.is_empty());
}

#[test]
fn opening_downloads_closes_the_tag_cloud() {
    let mut app = test_app();
    let _ = update(&mut app, Message::OpenTagCloud);
    let _ = update(
        &mut app,
        Message::Tag(crate::frontend::tagcloud::TagMsg::CloudClick(String::from("forest"), false)),
    );
    assert!(app.tags.cloud_open);

    let _ = update(&mut app, Message::OpenBrowser(String::from("wallhaven")));

    assert!(app.source_browser.browser.is_some());
    assert!(!app.tags.cloud_open);
    assert_eq!(app.library_session.filters.tags, [String::from("forest")]);
}

#[test]
fn menu_capturing_overlays() {
    use crate::app::overlay::Overlay;
    let mut app = test_app();
    assert!(!app.menu_capturing());
    app.input.help_open = true;
    assert!(app.menu_capturing());
    app.close_overlay(Overlay::Help);
    assert!(!app.menu_capturing());
    app.tags.cloud_open = true;
    assert!(!app.menu_capturing());
}

#[test]
fn download_done_video() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Youtube);
    browser.session.items.push(browser_item("yt1"));
    browser.session.pending_apply = Some(String::from("yt1"));
    app.source_browser.browser = Some(browser);
    drain_calls(&app);
    app.on_event(
        "skwd.wall.download",
        &json!({"id": "yt1", "status": "done", "path": "/vid/youtube-yt1.mp4"}),
    );
    let calls = drain_calls(&app);
    let apply =
        calls.iter().find(|(method, _)| method == "wall.apply").expect("youtube apply sent");
    assert_eq!(apply.1["type"], "video");
    assert_eq!(apply.1["path"], "/vid/youtube-yt1.mp4");
}

#[test]
fn download_error_clears() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("w1"));
    browser.session.items[0].downloading = true;
    browser.session.pending_apply = Some(String::from("w1"));
    app.source_browser.browser = Some(browser);
    app.on_event("skwd.wall.download", &json!({"id": "w1", "status": "error", "message": "quota"}));
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(!browser.session.items[0].downloading);
    assert!(browser.session.pending_apply.is_none());
    assert_eq!(browser.session.error.as_deref(), Some("quota"));
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.apply"));
}

#[test]
fn applied_event_audio() {
    let mut app = test_app();
    seed(&mut app, &[wall("v.mp4", "video", 1, 0), wall("a.png", "static", 2, 0)]);
    assert!(!app.panels.audio_active);
    app.on_event("skwd.wall.applied", &json!({"name": "v.mp4", "type": "video"}));
    assert!(app.panels.audio_active);
    let video = app
        .library_session
        .library
        .catalog()
        .items
        .iter()
        .find(|item| item.name == "v.mp4")
        .unwrap();
    assert_eq!(video.apply_count, 1);
    app.on_event("skwd.wall.applied", &json!({"name": "a.png", "type": "static"}));
    assert!(!app.panels.audio_active);
}

#[test]
fn audio_indicator_tracks_live_mix() {
    let mut app = test_app();
    app.config.save_key(skwd_config::keys::wallpaper::MUTE, json!(false));

    app.on_result(
        Pending::Outputs,
        &json!({
            "outputs": [{
                "name": "DP-1", "type": "video", "path": "/v.mp4",
                "mute": true, "volume": 80
            }]
        }),
    );
    assert!(app.panels.audio_active);
    assert!(!app.panels.audio_playing);

    app.on_result(
        Pending::Outputs,
        &json!({
            "outputs": [{
                "name": "DP-1", "type": "video", "path": "/v.mp4",
                "mute": false, "volume": 80
            }]
        }),
    );
    assert!(app.panels.audio_playing);

    app.on_result(
        Pending::Outputs,
        &json!({
            "outputs": [{
                "name": "DP-1", "type": "video", "path": "/v.mp4",
                "mute": false, "volume": 0
            }]
        }),
    );
    assert!(!app.panels.audio_playing);
}

#[test]
fn browser_banner_tracks_configured_output() {
    let mut app = test_app();
    app.config.save_key(skwd_config::keys::system::MONITOR, json!("DP-2"));
    app.on_result(
        Pending::Outputs,
        &json!({
            "outputs": [
                {"name": "DP-1", "type": "static", "path": "/wall/one.png"},
                {"name": "DP-2", "type": "static", "path": "/wall/two.png"}
            ]
        }),
    );
    assert_eq!(app.daemon.current_wallpaper_art.as_deref(), Some("/wall/two.png"));
    assert_eq!(
        app.daemon.output_wallpaper_art.get("DP-2").map(String::as_str),
        Some("/wall/two.png")
    );
}

#[test]
fn output_art_prefers_catalog_thumb() {
    let mut app = test_app();
    seed(
        &mut app,
        &[json!({
            "name": "forest.png", "type": "static", "path": "/wall/forest.png",
            "thumb": "/thumb/forest.webp"
        })],
    );
    let path = app.library_session.library.catalog().items[0].path.clone();
    app.on_result(
        Pending::Outputs,
        &json!({
            "outputs": [{
                "name": "DP-1", "type": "static", "path": path
            }]
        }),
    );
    assert_eq!(
        app.daemon.output_wallpaper_art.get("DP-1").map(String::as_str),
        Some("/thumb/forest.webp")
    );
}

#[test]
fn browser_banner_uses_library_art() {
    let mut app = test_app();
    seed(
        &mut app,
        &[json!({
            "name": "loop.mp4", "type": "video", "path": "/video/loop.mp4",
            "video_file": "/video/loop.mp4", "thumb": "/thumb/loop.jpg"
        })],
    );
    app.on_result(
        Pending::Outputs,
        &json!({
            "outputs": [{
                "name": "DP-1", "type": "video", "path": "/video/loop.mp4",
                "current": "/video/loop.mp4"
            }]
        }),
    );
    assert_eq!(app.daemon.current_wallpaper_art.as_deref(), Some("/thumb/loop.jpg"));
    assert_eq!(
        app.daemon.output_wallpaper_art.get("DP-1").map(String::as_str),
        Some("/thumb/loop.jpg")
    );
}

#[test]
fn applied_event_fronts_recent() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 300),
            wall("b.png", "static", 2, 200),
            wall("c.png", "static", 3, 100),
        ],
    );
    app.library_session.filters.sort = "recent".into();
    app.refilter();

    app.on_event("skwd.wall.applied", &json!({"key": "c.png", "type": "static"}));

    let first = app.library_session.filtered[0] as usize;
    assert_eq!(app.library_session.library.catalog().items[first].name, "c.png");
    assert_eq!(app.library_session.library.catalog().items[first].last_applied, 1);
}

#[test]
fn scan_done_refetch() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    app.on_event("skwd.wall.scan_done", &json!({"count": 5}));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == "wall.list"));
    assert!(app.daemon.pending.values().any(|pending| *pending == Pending::List));
    app.daemon.pending.clear();
    app.on_event("skwd.wall.scan_done", &json!({"count": 0, "total": 3}));
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.list"));
}

#[test]
fn same_count_exclusion_refetch() {
    let mut app = test_app();
    seed(&mut app, &[wall("nature/a.png", "static", 1, 0)]);
    drain_calls(&app);
    app.daemon.pending.insert(999, Pending::List);
    app.config.save_key(skwd_config::keys::library::EXCLUDED_FOLDERS, json!(["nature"]));
    app.on_event("skwd.wall.config_changed", &json!({"library_changed": true}));
    assert!(!app.daemon.pending.contains_key(&999));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.list"));
    app.on_result(Pending::List, &json!({"wallpapers": [wall("city/b.png", "static", 1, 0)]}));
    assert_eq!(filtered_names(&app), ["city/b.png"]);
    app.on_event("skwd.wall.config_changed", &json!({}));
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.list"));
}

#[test]
fn semantic_index_refresh_drops_worker() {
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Tags;
    app.on_event("skwd.wall.semantic_index_ready", &json!({"items": 5}));
    assert!(app.runtime_state.semantic.is_none());
}

#[test]
fn stale_refresh_respects_editor_removals() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.scene.set_flipped_for_test(Some(0));
    app.tags.editing = true;
    super::begin_card_tag_edit(&mut app, &[]);

    let mut stale = wall("a.png", "static", 1, 0);
    stale["tags"] = json!("brown,brown background,brown ribbon");
    app.on_result(Pending::List, &json!({"wallpapers": [stale]}));

    assert_eq!(app.tags.card_key.as_deref(), Some("a.png"));
    assert!(app.tags.card_locked.is_empty());
    assert_eq!(app.library_session.library.catalog().tags["a.png"], Vec::<String>::new());
}

#[test]
fn browser_wall_scroll_clamps() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut browser = Browser::new(Source::Wallhaven);
    for i in 0..60 {
        browser.session.items.push(browser_item(&format!("w{i}")));
    }
    app.source_browser.browser = Some(browser);
    let browser = app.source_browser.browser.as_ref().unwrap();
    app.source_browser.wall.rebuild_catalogue(browser, true);
    let mut now = Instant::now();
    frame(&mut app, now, 1280.0, 720.0);
    let _ = update(
        &mut app,
        Message::Browser(crate::frontend::browser::BrowserMsg::WallInput(
            crate::frontend::ui::BrowserWallInput::Wheel(-40.0),
        )),
    );
    for _ in 0..600 {
        now += Duration::from_millis(16);
        frame(&mut app, now, 1280.0, 720.0);
        assert!(app.source_browser.wall.scene.camera_pos() >= -1.0);
    }
    assert!(app.source_browser.wall.scene.camera_pos() > 0.0);
    app.source_browser.wall.scene.reset_to_start(60);
    tick_frames(&mut app, &mut now, 600);
    assert!(app.source_browser.wall.scene.camera_pos().abs() < 1.0);
}

#[test]
fn applied_event_refreshes_outputs() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    drain_calls(&app);
    app.daemon.pending.clear();

    app.on_event("skwd.wall.applied", &json!({"name": "a.png", "type": "static"}));

    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.outputs"));
    assert!(app.daemon.pending.values().any(|pending| *pending == Pending::Outputs));
}

#[test]
fn picker_monitor_apply_scopes_completed_downloads() {
    for (source, kind) in
        [(Source::Steam, "we"), (Source::Wallhaven, "static"), (Source::Youtube, "video")]
    {
        let mut app = test_app();
        app.config.save_key(skwd_config::keys::general::APPLY_ON_PICKER_MONITOR, json!(true));
        app.runtime_state.picker_output = Some("DP-2".into());
        let mut browser = Browser::new(source);
        browser.session.items.push(browser_item("download1"));
        browser.session.pending_apply = Some("download1".into());
        app.source_browser.browser = Some(browser);
        app.on_event(
            "skwd.wall.download",
            &json!({"id":"download1", "status":"done", "path":"/dl/wallpaper.png"}),
        );
        let calls = drain_calls(&app);
        let apply = calls.iter().find(|(method, _)| method == "wall.apply").unwrap();
        assert_eq!(apply.1["output"], "DP-2");
        assert_eq!(apply.1["type"], kind);
    }
}
