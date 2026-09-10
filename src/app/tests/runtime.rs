use super::*;

#[test]
fn message_size_bounded() {
    let size = std::mem::size_of::<Message>();
    assert!(size <= 160, "Message is {size} bytes");
}

#[test]
fn power_change_updates_policy() {
    let mut app = test_app();
    assert!(!app.config.on_battery_power());

    app.on_event(wall_proto::ev::POWER_CHANGED, &json!({"on_battery": true}));
    assert!(app.config.on_battery_power());
    assert_eq!(app.config.max_fps(), 60.0);

    app.on_event(wall_proto::ev::POWER_CHANGED, &json!({"on_battery": false}));
    assert!(!app.config.on_battery_power());
}

#[test]
fn version_mismatch_warns() {
    assert!(version_mismatch("0.1.0", "0.1.0").is_none());
    let warn = version_mismatch("0.2.0", "0.1.0").expect("differing versions warn");
    assert!(warn.contains("0.1.0") && warn.contains("0.2.0"));
    assert!(warn.contains("mismatch"));
}

#[test]
fn empty_library_hints() {
    assert!(empty_library_hint(3, "/wp", true, true).is_none());
    let hint = empty_library_hint(0, "/home/me/walls", true, true).expect("empty library hints");
    assert!(hint.contains("/home/me/walls"));
    assert!(!hint.contains('\u{2014}') && !hint.contains('\u{2013}'));
    let never = empty_library_hint(0, "/wp", false, false).expect("never-connected hints");
    assert!(never.contains("Connecting to skwd-walld"));
    let lost = empty_library_hint(5, "/wp", false, true).expect("disconnect hints");
    assert!(lost.contains("Lost connection"));
}

#[test]
fn connect_probes_version() {
    let mut app = test_app();
    drain_calls(&app);
    app.handle_ipc(IpcMsg::Connected { list_id: 5 });
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == "status"));
}

#[test]
fn stale_daemon_toast() {
    let mut app = test_app();
    app.handle_ipc(IpcMsg::Connected { list_id: 1 });
    let sid = status_request_id(&app);
    respond(&mut app, sid, json!({"ok": true, "version": "0.0.0-ancient"}));
    let toast = app.runtime_state.toast.as_ref().expect("mismatch surfaces a toast");
    assert!(toast.0.contains("mismatch"));
}

#[test]
fn matching_version_silent() {
    let mut app = test_app();
    app.handle_ipc(IpcMsg::Connected { list_id: 1 });
    let sid = status_request_id(&app);
    respond(&mut app, sid, json!({"version": env!("CARGO_PKG_VERSION")}));
    assert!(app.runtime_state.toast.is_none());
}

#[test]
fn status_response_sets_library_watcher_state() {
    let mut app = test_app();
    let status = wall_proto::LibraryWatchStatus {
        ok: true,
        degraded: true,
        mode: wall_proto::library_watch_mode::POLLING.to_string(),
        interval_seconds: Some(45),
        entry_budget_per_root: Some(2048),
        ..Default::default()
    };

    app.on_result(Pending::Status, &json!({"library_watch": status}));

    let stored = app.daemon.library_watch.as_ref().expect("watcher status stored");
    assert_eq!(stored.mode, wall_proto::library_watch_mode::POLLING);
    assert_eq!(stored.interval_seconds, Some(45));
    assert_eq!(stored.entry_budget_per_root, Some(2048));
}

#[test]
fn malformed_and_unknown_output_results_preserve_live_state() {
    let mut app = test_app();
    app.on_result(
        Pending::Outputs,
        &json!({
            "outputs": [{
                "name": "DP-1",
                "type": "video",
                "path": "/video/live.mp4",
                "mute": false,
                "volume": 70
            }]
        }),
    );
    let before = app.daemon.output_statuses.clone();
    assert_eq!(before.len(), 1);

    app.on_result(Pending::Outputs, &json!({"outputs": "invalid"}));
    assert_eq!(app.daemon.output_statuses, before);

    app.on_result(Pending::Outputs, &json!({}));
    assert_eq!(app.daemon.output_statuses, before);

    app.on_result(Pending::Outputs, &json!({"schema_version": 2, "outputs": []}));
    assert_eq!(app.daemon.output_statuses, before);
}

#[test]
fn malformed_and_unknown_library_results_preserve_catalogue() {
    let mut app = test_app();
    seed(&mut app, &[wall("kept.png", "static", 1, 0)]);

    app.on_result(Pending::List, &json!({"wallpapers": false}));
    assert_eq!(app.library_session.library.catalog().items[0].name, "kept.png");

    app.on_result(Pending::List, &json!({}));
    assert_eq!(app.library_session.library.catalog().items[0].name, "kept.png");

    app.on_result(Pending::List, &json!({"schema_version": 2, "wallpapers": []}));
    assert_eq!(app.library_session.library.catalog().items[0].name, "kept.png");
}

#[test]
fn incompatible_protocol_does_not_start_picker_session() {
    let mut app = test_app();
    drain_calls(&app);
    app.on_result(
        Pending::Status,
        &json!({
            "version": env!("CARGO_PKG_VERSION"),
            "protocol": {"name": "skwd-wall", "version": 2},
            "capabilities": ["picker-session"]
        }),
    );
    assert!(app.runtime_state.toast.is_none());
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "picker.session.begin"));
}

#[test]
fn malformed_theme_previews_follow_request_error_path() {
    let mut app = test_app();
    app.theme.audition_backend = "wallust".into();
    app.theme.audition_pending_backend = Some("wallust".into());
    app.theme.audition_loading = true;

    app.on_result(
        Pending::ThemePreviews { backend: "wallust".into() },
        &json!({"backend": "wallust"}),
    );

    assert!(!app.theme.audition_loading);
    assert!(app.theme.audition_pending_backend.is_none());
    assert!(app.theme.audition_error.as_deref().is_some_and(|error| error.contains("invalid")));
}

#[test]
fn watcher_status_event_replaces_runtime_state() {
    let mut app = test_app();
    app.daemon.library_watch = Some(crate::contracts::daemon::LibraryWatchStatus {
        mode: crate::contracts::daemon::LibraryWatchMode::POLLING.to_string(),
        ..Default::default()
    });

    app.on_event(
        wall_proto::ev::WATCH_STATUS,
        &json!({
            "ok": true,
            "degraded": false,
            "mode": wall_proto::library_watch_mode::NATIVE,
            "detail": "native file watching active",
            "roots": []
        }),
    );

    let stored = app.daemon.library_watch.as_ref().expect("watcher status stored");
    assert!(stored.ok);
    assert!(!stored.degraded);
    assert_eq!(stored.mode, wall_proto::library_watch_mode::NATIVE);
}

#[test]
fn hud_wakes_skip_scene_tick() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    assert!(app.runtime_state.last_tick.is_none());

    let _ = update(&mut app, Message::Daemon(crate::infrastructure::runtime::Wake::HudTick));
    assert!(app.runtime_state.last_tick.is_none());

    app.daemon.pending.insert(901, Pending::Diag);
    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Ipc(IpcMsg::Response {
            id: 901,
            result: Some(json!({"banner": "walld: ok"})),
            error: None,
        })),
    );
    assert_eq!(app.daemon.diagnostic, "walld: ok");
    assert!(app.runtime_state.last_tick.is_none());

    let _ = update(&mut app, Message::Daemon(crate::infrastructure::runtime::Wake::Decoded));
    assert!(app.runtime_state.last_tick.is_some());
}

#[test]
fn doctor_summary_lines() {
    let check = |status: &str, name: &str| crate::contracts::daemon::DoctorCheck {
        status: status.into(),
        check: name.into(),
        detail: String::new(),
    };
    let all_pass = vec![check("pass", "a"), check("pass", "b")];
    assert_eq!(doctor_summary(&all_pass), "Diagnostics: all 2 checks passed");
    let mixed = vec![check("pass", "a"), check("fail", "skwd-wall-vk"), check("warn", "matugen")];
    assert_eq!(doctor_summary(&mixed), "Diagnostics: 2 issues - skwd-wall-vk; matugen");
    let single = vec![check("fail", "skwd-wall-vk")];
    assert_eq!(doctor_summary(&single), "Diagnostics: 1 issue - skwd-wall-vk");
}

#[test]
fn apply_error_messages() {
    assert_eq!(
        apply_error_message("file_missing", "no such file"),
        "Apply failed: wallpaper file is missing (no such file)"
    );
    assert_eq!(
        apply_error_message("renderer_unavailable", "video media capability requires skwd-wall-vk"),
        "Apply failed: renderer could not start (video media capability requires skwd-wall-vk)"
    );
    assert_eq!(apply_error_message("bad_request", "missing path"), "Apply failed: invalid request");
    assert_eq!(apply_error_message("apply_failed", ""), "Apply failed");
}

#[test]
fn scene_visibility_tabs() {
    let (dm, hidden) = scene_visibility(false, "picker", 1, false);
    assert!(!dm && !hidden);

    let (dm, hidden) = scene_visibility(true, "picker", 1, false);
    assert!(dm && !hidden);

    let (dm, hidden) = scene_visibility(true, "picker", 0, false);
    assert!(!dm && hidden);

    let (dm, hidden) = scene_visibility(true, "playback", 1, false);
    assert!(!dm && hidden);

    let (dm, hidden) = scene_visibility(true, "picker", 1, true);
    assert!(!dm && hidden);
}

#[test]
fn list_refresh_gate() {
    assert!(needs_list_refresh(0, 3103, None));
    assert!(!needs_list_refresh(3103, 3103, None));
    assert!(needs_list_refresh(0, 0, Some(3103)));
    assert!(needs_list_refresh(3103, 2, Some(3103)));
    assert!(needs_list_refresh(3100, 0, Some(3103)));
    assert!(!needs_list_refresh(3103, 0, Some(3103)));
}

#[test]
fn hex_radius_floor() {
    let cfg = Config::from_data(json!({
        "components": {"wallpaperSelector": {"displayMode": "hex", "hexRadius": 1.0}}
    }));
    assert!(layout_params(&cfg).hex.r >= MIN_HEX_R);
    assert_eq!(cfg.hex_radius(), 1.0);
}

#[test]
fn effect_names() {
    assert_eq!(
        EFFECT_NAMES,
        [
            "Ignite",
            "Edge Fracture",
            "Tonal Wipe",
            "Ash",
            "Depth Parallax",
            "Bokeh Bloom",
            "Light Streaks",
            "Voxel Extrude",
            "Pixel Sort",
            "Rack Focus",
            "Topographic",
            "Tonal Layers",
        ]
    );
}

#[test]
fn connect_pending_ids() {
    let mut app = test_app();
    app.handle_ipc(IpcMsg::Connected { list_id: 42 });
    assert_eq!(app.daemon.pending.get(&42), Some(&Pending::List));
    let calls = drain_calls(&app);
    let methods: Vec<&str> = calls.iter().map(|(method, _)| method.as_str()).collect();
    assert_eq!(methods, ["effects.list", "wall.outputs", "status", "task.list"]);
    assert_eq!(app.daemon.pending.get(&1), Some(&Pending::EffectThemes));
    assert_eq!(app.daemon.pending.get(&2), Some(&Pending::Outputs));
    assert_eq!(app.daemon.pending.get(&3), Some(&Pending::Status));
    assert_eq!(app.daemon.pending.get(&4), Some(&Pending::TaskList));
}

#[test]
fn download_error_keys() {
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("w1"));
    browser.session.pending_apply = Some(String::from("w1"));
    let mut ev = dl_ev("error", json!({"error": "429 too many requests"}));
    ev.id = String::from("w1");
    assert!(download_update(&mut browser, &ev).is_none());
    assert_eq!(browser.session.error.as_deref(), Some("429 too many requests"));
    assert_eq!(browser.session.pending_apply, None);
    let mut ev = dl_ev("auth_error", json!({"message": "steam guard code required"}));
    ev.id = String::from("w1");
    download_update(&mut browser, &ev);
    assert_eq!(browser.session.error.as_deref(), Some("steam guard code required"));
}

#[test]
fn rpc_error_download() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("w1"));
    browser.session.items[0].downloading = true;
    browser.session.pending_apply = Some(String::from("w1"));
    app.source_browser.browser = Some(browser);
    app.daemon
        .pending
        .insert(77, Pending::BrowserDownload { source: Source::Wallhaven, id: String::from("w1") });
    reject(&mut app, 77, "boom");
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(!browser.session.items[0].downloading);
    assert!(browser.session.pending_apply.is_none());
    assert_eq!(browser.session.error.as_deref(), Some("boom"));
    assert!(app.daemon.pending.is_empty());
}

#[test]
fn rpc_error_browser_message_only() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("w1"));
    browser.session.search_generation = 3;
    app.source_browser.browser = Some(browser);
    app.daemon.pending.insert(
        91,
        Pending::BrowserSearch { source: Source::Wallhaven, append: false, generation: 3 },
    );
    reject_with_code(&mut app, 91, -32001, "rate limited");
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert_eq!(browser.session.error.as_deref(), Some("rate limited"));
}

#[test]
fn stale_browser_search_error_is_discarded() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.loading = true;
    browser.session.search_generation = 5;
    app.source_browser.browser = Some(browser);
    app.daemon.pending.insert(
        92,
        Pending::BrowserSearch { source: Source::Wallhaven, append: false, generation: 4 },
    );

    reject_with_code(&mut app, 92, -32001, "source list request superseded");

    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(browser.session.loading);
    assert!(browser.session.error.is_none());
}

#[test]
fn stale_browser_search_result_is_discarded() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.loading = true;
    browser.session.items.push(browser_item("current"));
    browser.session.search_generation = 5;
    app.source_browser.browser = Some(browser);
    app.daemon.pending.insert(
        93,
        Pending::BrowserSearch { source: Source::Wallhaven, append: false, generation: 4 },
    );

    respond(
        &mut app,
        93,
        json!({
            "generation": 4,
            "current_page": 1,
            "last_page": 1,
            "results": [{"id": "stale"}]
        }),
    );

    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(browser.session.loading);
    assert_eq!(browser.session.items[0].id, "current");
}

#[test]
fn mismatched_browser_search_envelope_is_discarded() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.loading = true;
    browser.session.items.push(browser_item("current"));
    browser.session.search_generation = 5;
    app.source_browser.browser = Some(browser);
    app.daemon.pending.insert(
        94,
        Pending::BrowserSearch { source: Source::Wallhaven, append: false, generation: 5 },
    );

    respond(
        &mut app,
        94,
        json!({
            "generation": 4,
            "current_page": 1,
            "last_page": 1,
            "results": [{"id": "mismatched"}]
        }),
    );

    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(browser.session.loading);
    assert_eq!(browser.session.items[0].id, "current");
}

#[test]
fn replacement_searches_advance_generation() {
    let mut app = test_app();
    app.source_browser.browser = Some(Browser::new(Source::Wallhaven));
    drain_calls(&app);

    app.run_browser_search(false);
    let first = drain_calls(&app);
    assert_eq!(first[0].1["generation"], 1);

    app.run_browser_search(false);
    let second = drain_calls(&app);
    assert_eq!(second[0].1["generation"], 2);

    app.run_browser_search(true);
    let append = drain_calls(&app);
    assert_eq!(append[0].1["generation"], 2);
}

#[test]
fn browser_search_generation_survives_closing_and_reopening() {
    let mut app = test_app();
    app.source_browser.activate(Source::Wallhaven);
    drain_calls(&app);
    app.run_browser_search(false);
    app.run_browser_search(false);
    drain_calls(&app);

    app.source_browser.close();
    app.source_browser.activate(Source::Wallhaven);
    app.run_browser_search(false);
    assert_eq!(drain_calls(&app)[0].1["generation"], 3);

    app.source_browser.activate(Source::Steam);
    app.run_browser_search(false);
    assert_eq!(drain_calls(&app)[0].1["generation"], 4);

    app.source_browser.activate(Source::Wallhaven);
    app.run_browser_search(true);
    assert_eq!(drain_calls(&app)[0].1["generation"], 3);
    app.run_browser_search(false);
    assert_eq!(drain_calls(&app)[0].1["generation"], 5);
}

#[test]
fn browser_preview_waits_for_prepared() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("w1"));
    browser.session.preview = Some(0);
    app.source_browser.browser = Some(browser);

    app.on_result(
        Pending::BrowserPreview { source: Source::Wallhaven, id: String::from("w1") },
        &json!({"status": "ready", "path": "/cache/unprepared.png"}),
    );
    assert!(app.source_browser.browser.as_ref().unwrap().session.items[0].preview_path.is_none());

    app.on_event(
        wall_proto::ev::PREVIEW_READY,
        &json!({"id": "w1", "path": "/cache/prepared.png"}),
    );
    assert_eq!(
        app.source_browser.browser.as_ref().unwrap().session.items[0].preview_path.as_deref(),
        Some("/cache/prepared.png")
    );
}

#[test]
fn stale_browser_preview_discarded() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("current"));
    browser.session.items.push(browser_item("stale"));
    browser.session.preview = Some(0);
    app.source_browser.browser = Some(browser);
    let directory = tempfile::tempdir().unwrap();
    let preview = directory.path().join("stale.jpg");
    std::fs::write(&preview, b"stale").unwrap();

    app.on_event(
        wall_proto::ev::PREVIEW_READY,
        &json!({"id": "stale", "path": preview.to_string_lossy()}),
    );

    assert!(!preview.exists());
    assert!(app.source_browser.browser.as_ref().unwrap().session.items[1].preview_path.is_none());
}

#[test]
fn rpc_error_effects() {
    let mut app = test_app();
    let mut effects = crate::frontend::effects::Effects::new(
        Vec::new(),
        String::from("/src.png"),
        None,
        0,
        WallpaperKind::Static,
        true,
        80,
        String::from("static:src.png"),
    );
    effects.begin_request();
    effects.mark_preview_queued();
    app.panels.effects = Some(effects);
    app.daemon.pending.insert(
        5,
        Pending::EffectsPreview { source: String::from("/src.png"), cache_key: String::new() },
    );
    reject(&mut app, 5, "bad");
    let effects = app.panels.effects.as_ref().unwrap();
    assert!(!effects.is_busy());
    assert!(!effects.preview_queued());
    assert_eq!(effects.status(), "The preview failed: bad");
}

#[test]
fn stale_effect_preview_discarded() {
    let mut app = test_app();
    app.panels.effects = Some(crate::frontend::effects::Effects::new(
        Vec::new(),
        String::from("/second.png"),
        None,
        1,
        WallpaperKind::Static,
        true,
        80,
        String::from("static:second.png"),
    ));
    let _ = drain_calls(&app);

    app.on_result(
        Pending::EffectsPreview { source: String::from("/first.png"), cache_key: String::new() },
        &json!({"output": "/cache/first-effect.png"}),
    );

    let effects = app.panels.effects.as_ref().unwrap();
    assert_eq!(effects.source_path(), "/second.png");
    assert!(effects.preview_path().is_none());
    assert!(drain_calls(&app).iter().any(|(method, params)| {
        method == "effects.discard" && params["preview"] == "/cache/first-effect.png"
    }));
}

#[test]
fn disconnect_drops_pending() {
    let mut app = test_app();
    let mut effects = crate::frontend::effects::Effects::new(
        Vec::new(),
        String::new(),
        None,
        0,
        WallpaperKind::Static,
        true,
        80,
        String::new(),
    );
    effects.begin_request();
    app.panels.effects = Some(effects);
    app.daemon
        .pending
        .insert(5, Pending::EffectsPreview { source: String::new(), cache_key: String::new() });
    app.handle_ipc(IpcMsg::Disconnected);
    assert!(app.daemon.pending.is_empty());
    assert!(app.library_session.list_dirty);
    assert!(!app.panels.effects.as_ref().unwrap().is_busy());
    respond(&mut app, 5, json!({"output": "/late.png"}));
    assert!(!app.panels.effects.as_ref().unwrap().is_busy());
}

#[test]
fn unmatched_response_noop() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    respond(&mut app, 424_242, json!({"wallpapers": []}));
    assert_eq!(app.library_session.library.catalog().items.len(), 1);
}

#[test]
fn per_app_wake_channel() {
    let one = test_app();
    let two = test_app();
    assert!(!one.runtime_state.wake_tx.same_receiver(&two.runtime_state.wake_tx));
    assert!(one.runtime_state.wake_tx.same_receiver(&one.runtime_state.wake_tx.clone()));
}

#[test]
fn picker_session_needs_capability() {
    let mut app = test_app();
    app.handle_ipc(IpcMsg::Connected { list_id: 42 });
    let status_id = 3;
    let _ = drain_calls(&app);
    respond(
        &mut app,
        status_id,
        json!({"version": env!("CARGO_PKG_VERSION"), "capabilities": ["wallpapers"]}),
    );
    let without: Vec<String> = drain_calls(&app).into_iter().map(|(method, _)| method).collect();
    assert!(!without.iter().any(|method| method == "picker.session.begin"));

    let mut app = test_app();
    app.handle_ipc(IpcMsg::Connected { list_id: 42 });
    let _ = drain_calls(&app);
    respond(
        &mut app,
        status_id,
        json!({"version": env!("CARGO_PKG_VERSION"), "capabilities": ["wallpapers", "picker-session"]}),
    );
    let with: Vec<String> = drain_calls(&app).into_iter().map(|(method, _)| method).collect();
    assert_eq!(with, ["picker.session.begin"]);
}

#[test]
fn failed_page_stops_pagination() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("w1"));
    browser.session.page = 1;
    browser.session.last_page = 9;
    browser.session.search_generation = 4;
    app.source_browser.browser = Some(browser);
    app.daemon.pending.insert(
        71,
        Pending::BrowserSearch { source: Source::Wallhaven, append: true, generation: 4 },
    );

    reject(&mut app, 71, "rate limited");
    assert!(app.source_browser.browser.as_ref().unwrap().session.page_failed);
    drain_calls(&app);

    app.tick_browser_anim(Instant::now());

    assert!(!drain_calls(&app).iter().any(|(method, _)| method.contains("search")));
}

#[test]
fn fresh_search_rearms_pagination() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.page_failed = true;
    app.source_browser.browser = Some(browser);

    app.run_browser_search(false);

    assert!(!app.source_browser.browser.as_ref().unwrap().session.page_failed);
}

#[test]
fn scene_thumbnail_update_reloads_only_the_matching_tile() {
    use crate::domain::library::catalog::Wallpaper;
    use crate::rendering::scene::atlas::AtlasMap;
    let mut app = test_app();
    app.library_session.library.insert(Wallpaper {
        key: "we:42".into(),
        thumb: "/old/we-thumbs/42.webp".into(),
        ..Wallpaper::default()
    });
    app.library_session.library.insert(Wallpaper {
        key: "other".into(),
        name: "other".into(),
        ..Wallpaper::default()
    });
    let mut atlas = AtlasMap::new(2);
    for index in 0..2 {
        atlas.near.acquire(index).unwrap();
        atlas.far.acquire(index).unwrap();
        atlas.near.mark_ready(index);
        atlas.far.mark_ready(index);
    }
    atlas.failed.insert(0);
    atlas.near_failed.insert(0);
    app.preview_resources.atlas = Some(atlas);
    app.on_event(
        wall_proto::ev::THUMBNAIL_UPDATED,
        &json!({"key": "we:42", "thumb": "/new/we-thumbs/42.webp"}),
    );
    let atlas = app.preview_resources.atlas.as_ref().unwrap();
    assert!(!atlas.near.is_known(0));
    assert!(!atlas.far.is_known(0));
    assert!(!atlas.failed.contains(&0));
    assert!(!atlas.near_failed.contains(&0));
    assert!(atlas.near.ready(1).is_some());
    assert!(atlas.far.ready(1).is_some());
    assert_eq!(app.library_session.library.catalog().items[0].thumb, "/new/we-thumbs/42.webp");
}

#[test]
fn reset_thumbnail_accepts_older_we_cards_without_capture_metadata() {
    let mut app = test_app();
    let mut scene = wall("scene", "we", 1, 0);
    scene["we_id"] = json!("42");
    seed(&mut app, &[scene, wall("still.png", "static", 1, 0)]);
    drain_calls(&app);
    let _ = update(&mut app, Message::ResetThumbnail(1));
    assert!(drain_calls(&app).is_empty());
    let _ = update(&mut app, Message::ResetThumbnail(0));
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], (wall_proto::rpc::WALL_RESET_THUMBNAIL.into(), json!({"we_id": "42"})));
    let _ = update(&mut app, Message::ResetThumbnail(0));
    assert!(drain_calls(&app).is_empty());
    let key = app.library_session.library.catalog().items[0].key.clone();
    app.on_event(
        wall_proto::ev::THUMBNAIL_UPDATED,
        &json!({"key": key, "thumb": "/fresh.webp", "generated": true}),
    );
    assert!(app.library_session.library.catalog().items[0].thumbnail_generated);
    assert_eq!(app.library_session.library.catalog().items[0].thumb, "/fresh.webp");
    let _ = update(&mut app, Message::ResetThumbnail(0));
    assert!(drain_calls(&app).is_empty());
}
