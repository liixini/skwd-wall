use super::*;

#[test]
fn ui_commands() {
    let mut app = test_app();
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("cat");
    let b = wall("b.png", "static", 2, 0);
    seed(&mut app, &[a, b]);
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };
    cmd(&mut app, "mode hex");
    assert_eq!(app.config.display_mode(), "hex");
    assert_eq!(app.scene.mode, Mode::Hex);
    cmd(&mut app, "mode nonsense");
    assert_eq!(app.config.display_mode(), "hex");
    cmd(&mut app, "filter cat");
    assert_eq!(filtered_names(&app), ["a.png"]);
    cmd(&mut app, "clear");
    assert_eq!(app.library_session.filtered.len(), 2);
    cmd(&mut app, "search show me cats");
    assert!(app.tags.cloud_open);
    assert!(app.library_session.filters.tags.is_empty());
    assert_eq!(app.tags.semantic.search, "show me cats");
    cmd(&mut app, "clear");
    cmd(&mut app, "set components.wallpaperSelector.sandySwapStyle vortex");
    assert_eq!(app.config.sandy_swap_style(), "vortex");
    cmd(&mut app, "set components.wallpaperSelector.flipDurationMs 720");
    cmd(&mut app, "set components.wallpaperSelector.flipShader false");
    cmd(&mut app, "set components.wallpaperSelector.flipBackReveal false");
    assert_eq!(app.scene.card_flip_options(), (720.0, false, false));
    cmd(&mut app, "set motion.slowMs 900");
    assert!((app.scene.filter_swap_ms() - 900.0).abs() < 0.01);
    cmd(&mut app, "mode slices");
    cmd(&mut app, "set components.wallpaperSelector.sliceEdgeTilt 72");
    cmd(&mut app, "preset save Slanted cards");
    assert_eq!(app.config.selected_preset("slices").as_deref(), Some("Slanted cards"));
    assert_eq!(app.config.selector_presets("slices")[0].1["sliceEdgeTilt"], json!(72.0));
    cmd(&mut app, "mode wall");
    let camera_before = app.scene.camera_target();
    cmd(&mut app, "wheel -2");
    assert!(app.scene.camera_target() > camera_before);
    cmd(&mut app, "nav right");
    assert_eq!(app.scene.current, 1);
    cmd(&mut app, "nav left");
    assert_eq!(app.scene.current, 0);
    cmd(&mut app, "open mixer");
    assert!(app.panels.audio.is_some());
    cmd(&mut app, "nav right");
    assert_eq!(app.scene.current, 0);
    cmd(&mut app, "dismiss");
    assert!(app.panels.audio.is_none());
    cmd(&mut app, "dismiss");
    assert!(app.topmost_overlay().is_none());
    cmd(&mut app, "open playlists");
    app.panels.playlists.as_mut().unwrap().lists.push(crate::contracts::playlists::Playlist {
        id: 31,
        name: "field test".into(),
        ..Default::default()
    });
    cmd(&mut app, "playlist 31");
    assert_eq!(app.panels.playlists.as_ref().unwrap().selected, Some(31));
    cmd(&mut app, "close");
    cmd(&mut app, "open theme");
    assert!(app.panels.theme_designer.is_some());
    cmd(&mut app, "open effects brightness");
    assert_eq!(app.panels.effects.as_ref().unwrap().effect_id(), "brightness");
    cmd(&mut app, "open effects invert");
    assert_eq!(app.panels.effects.as_ref().unwrap().effect_id(), "invert");
    cmd(&mut app, "open effects");
    assert!(app.panels.effects.is_none());
    cmd(&mut app, "open displays");
    assert_eq!(
        app.panels.effects.as_ref().map(crate::frontend::effects::Effects::mode),
        Some(crate::frontend::effects::EffectsMode::Displays)
    );
    cmd(&mut app, "close");
    cmd(&mut app, "totally-bogus");
}

#[test]
fn demo_audio_volume_drives_the_open_mixer_without_unmuting() {
    let mut app = test_app();
    let before_volume = app.config.wallpaper_volume();
    let before_mute = app.config.wallpaper_mute();
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };
    let monitor =
        |name: &str, kind: &str, mute: bool, volume: u32| crate::frontend::audio_panel::AudioMon {
            name: name.to_string(),
            label: name.to_string(),
            source: format!("/{name}"),
            wtype: crate::contracts::media::MediaKind::from_key(kind),
            mute,
            volume,
            shared: false,
            paused: false,
            manual_paused: false,
        };

    command(&mut app, "demo begin");
    command(&mut app, "open mixer");
    app.panels.audio.as_mut().unwrap().mons = vec![
        monitor("DP-1", "video", true, 0),
        monitor("DP-2", "we", true, 0),
        monitor("DP-3", "static", true, 0),
    ];
    drain_calls(&app);

    command(&mut app, "audio-demo volume 72");
    let monitors = &app.panels.audio.as_ref().unwrap().mons;
    assert_eq!((monitors[0].volume, monitors[0].mute), (72, true));
    assert_eq!((monitors[1].volume, monitors[1].mute), (72, true));
    assert_eq!((monitors[2].volume, monitors[2].mute), (0, true));
    assert_eq!(app.config.wallpaper_volume(), before_volume);
    assert!(app.config.wallpaper_mute());
    assert!(drain_calls(&app).iter().any(|(method, params)| {
        method == "wall.set_audio" && params == &json!({ "volume": 0, "mute": true })
    }));
    app.on_result(
        Pending::AudioOutputs,
        &json!({"outputs": [{
            "name": "DP-1", "target": "DP-1", "connected": true,
            "width": 1920, "height": 1080, "logical_width": 1920, "logical_height": 1080,
            "current": "/late-video.mp4", "type": "video", "path": "/late-video.mp4",
            "we_id": "", "mute": false, "volume": 5, "fill": "fill", "audioShared": false
        }]}),
    );
    let late = &app.panels.audio.as_ref().unwrap().mons[0];
    assert_eq!((late.volume, late.mute), (72, true));

    command(&mut app, "demo end");
    assert_eq!(app.config.wallpaper_volume(), before_volume);
    assert_eq!(app.config.wallpaper_mute(), before_mute);
    assert!(drain_calls(&app).iter().any(|(method, params)| {
        method == "wall.set_audio"
            && params == &json!({ "volume": before_volume, "mute": before_mute })
    }));
}

#[test]
fn demo_effect_choice_drives_the_open_effects_dropdown() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.daemon.effect_definitions = crate::infrastructure::effects::decode_definitions(&[json!({
        "id": "theme",
        "label": "Theme",
        "category": "Colour",
        "params": [{
            "id": "theme",
            "label": "Palette",
            "type": "dropdown",
            "default": "Catppuccin",
            "options": [
                { "mode": "Catppuccin", "label": "Catppuccin" },
                { "mode": "Everforest", "label": "Everforest" },
                { "mode": "Rose Pine", "label": "Rose Pine" }
            ]
        }]
    })]);
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "demo begin");
    command(&mut app, "open effects theme");
    command(&mut app, "effect-demo choice theme Everforest");
    let effects = app.panels.effects.as_ref().unwrap();
    assert_eq!(
        effects.parameter_values().get("theme"),
        Some(&crate::domain::effects::EffectValue::Text("Everforest".into()))
    );
    assert!(effects.preview_queued());

    command(&mut app, "effect-demo choice theme Missing");
    assert_eq!(
        app.panels.effects.as_ref().unwrap().parameter_values().get("theme"),
        Some(&crate::domain::effects::EffectValue::Text("Everforest".into()))
    );
}

#[test]
fn demo_effect_show_reuses_a_rendered_palette_preview() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.daemon.effect_definitions = crate::infrastructure::effects::decode_definitions(&[json!({
        "id": "theme",
        "label": "Theme",
        "category": "Colour",
        "params": [{
            "id": "theme",
            "label": "Palette",
            "type": "dropdown",
            "default": "Catppuccin",
            "options": [
                { "mode": "Catppuccin", "label": "Catppuccin" },
                { "mode": "Everforest", "label": "Everforest" }
            ]
        }]
    })]);
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "demo begin");
    command(&mut app, "open effects-cached theme");
    let (source, cache_key) = {
        let effects = app.panels.effects.as_mut().unwrap();
        effects.set_str("theme", "Everforest".into());
        effects.begin_request();
        (
            effects.source_path().to_string(),
            serde_json::to_string(&crate::infrastructure::effects::encode_steps(
                &effects.preview_effects(),
            ))
            .unwrap(),
        )
    };
    app.on_result(
        Pending::EffectsPreview { source, cache_key: cache_key.clone() },
        &json!({"output": "/cache/everforest.png"}),
    );
    assert_eq!(
        app.runtime_state.demo.as_ref().unwrap().effect_previews.get(&cache_key),
        Some(&String::from("/cache/everforest.png"))
    );

    let _ = drain_calls(&app);
    command(&mut app, "dismiss");
    assert!(drain_calls(&app).iter().all(|(method, params)| method != "effects.discard"
        || params["preview"] != "/cache/everforest.png"));
    command(&mut app, "open effects-cached theme");
    let _ = drain_calls(&app);
    command(&mut app, "effect-demo show theme Everforest");
    assert_eq!(app.panels.effects.as_ref().unwrap().preview_path(), Some("/cache/everforest.png"));
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "effects.preview"));
}

#[test]
fn demo_apply_override_restores_locked_output_wallpapers() {
    let mut app = test_app();
    seed(
        &mut app,
        &[json!({
            "key": "video:forest.mp4",
            "name": "forest.mp4",
            "type": "video",
            "thumb": "/thumbs/forest.webp"
        })],
    );
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "demo begin");
    app.on_result(
        Pending::DemoOutputs,
        &json!({"outputs": [{
            "name": "DP-2", "target": "DP-2", "connected": true,
            "width": 3840, "height": 2160, "logical_width": 2160, "logical_height": 3840,
            "current": "/old.png", "type": "static", "path": "/old.png",
            "we_id": "", "mute": true, "volume": 0, "fill": "fill", "audioShared": false
        }]}),
    );
    command(&mut app, "apply-source video:forest.mp4");
    command(&mut app, "override-next");
    command(&mut app, "apply * crossfade 360");
    let calls = drain_calls(&app);
    let (_, apply) = calls.iter().find(|(method, _)| method == "wall.apply").unwrap();
    assert_eq!(apply["override_locks"], true);
    assert_eq!(app.runtime_state.demo.as_ref().unwrap().overridden_outputs.len(), 1);

    command(&mut app, "restore-overrides");
    let calls = drain_calls(&app);
    let (_, restore) = calls.iter().find(|(method, _)| method == "wall.apply").unwrap();
    assert_eq!(restore["type"], "static");
    assert_eq!(restore["path"], "/old.png");
    assert_eq!(restore["output"], "DP-2");
    assert_eq!(restore["override_locks"], true);
    assert!(app.runtime_state.demo.as_ref().unwrap().overridden_outputs.is_empty());
}

#[test]
fn demo_batch_commits_atomically() {
    let mut app = test_app();
    app.scene.render = std::sync::Arc::new(crate::frontend::scene::RenderSnapshot {
        renderer: std::sync::Arc::new(crate::contracts::rendering::RendererSnapshot {
            instances: vec![crate::contracts::rendering::InstanceRaw::default()],
            ..crate::contracts::rendering::RendererSnapshot::default()
        }),
        ..crate::frontend::scene::RenderSnapshot::default()
    });
    let command = |app: &mut App, value: String| {
        let _ = update(app, Message::Daemon(crate::infrastructure::runtime::Wake::Command(value)));
    };
    command(&mut app, "demo begin".to_string());
    let before = app.scene.mode;
    for value in [
        "mode wall",
        "tune components.wallpaperSelector.gridColumns 4",
        "tune components.wallpaperSelector.gridRows 4",
        "tune components.wallpaperSelector.gridThumbWidth 258",
    ] {
        command(&mut app, format!("batch-stage a {}", json!(value)));
        assert_eq!(app.scene.mode, before);
    }
    command(&mut app, "batch-commit a".to_string());

    let layout = layout_params(&app.config);
    assert_eq!(layout.mode, Mode::Grid);
    assert_eq!(layout.grid.cols, 4);
    assert_eq!(layout.grid.rows, 4);
    assert!((layout.grid.thumb_w - 258.0).abs() < f32::EPSILON);
    assert_eq!(app.scene.gp.cols, 4);
    assert_eq!(app.scene.gp.rows, 4);
    assert!((app.scene.gp.thumb_w - 258.0).abs() < f32::EPSILON);
    assert!(app.scene.layout_transition_active());
}

#[test]
fn ui_commands_drive_settings_search() {
    let mut app = test_app();
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };
    command(&mut app, "settings-search battery");
    assert!(app.panels.settings.open);
    assert!(app.panels.settings.search_open);
    assert_eq!(app.panels.settings.search_query, "battery");
    assert!(!app.panels.settings.search_results.is_empty());
    command(&mut app, "settings-result 0");
    assert_eq!(app.panels.settings.focus, crate::frontend::settings::SettingsFocus::Controls);
    assert!(!app.panels.settings.search_open);
}

#[test]
fn ui_state_query() {
    let mut app = test_app();
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("cat");
    let b = wall("b.png", "static", 2, 0);
    seed(&mut app, &[a, b]);
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };
    let state = |app: &App| -> serde_json::Value {
        serde_json::from_str(&crate::app::update::ui_state_json(app)).unwrap()
    };

    cmd(&mut app, "mode wall");
    let snap = state(&app);
    assert_eq!(snap["mode"], "wall");
    assert_eq!(snap["count"], 2);
    assert_eq!(snap["current"], 0);
    assert_eq!(snap["selection"], "a.png");
    assert_eq!(snap["demo_protocol"], 15);
    assert_eq!(snap["demo_active"], false);
    assert_eq!(snap["semantic"]["pending"], false);
    assert_eq!(snap["semantic"]["ranked"], 0);
    assert_eq!(snap["capturing"], false);
    assert!(snap["overlay"].is_null());
    assert_eq!(snap["cell"]["h"], app.scene.gp.cell_h());

    app.scene.grid_scroll(2.0);
    let snap = state(&app);
    assert!(
        (snap["camera"]["target"].as_f64().unwrap() as f32 - 2.0 * app.scene.gp.cell_h()).abs()
            < 0.01
    );

    cmd(&mut app, "filter cat");
    let snap = state(&app);
    assert_eq!(snap["count"], 1);
    assert_eq!(snap["filter"]["tags"], json!(["cat"]));

    cmd(&mut app, "open mixer");
    let snap = state(&app);
    assert_eq!(snap["capturing"], true);
    assert_eq!(snap["overlay"], "AudioPanel");

    let replies = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = replies.clone();
    let reply = crate::infrastructure::runtime::Reply::new(Box::new(move |text: &str| {
        sink.lock().unwrap().push(text.to_string());
    }));
    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Query("state".into(), reply.clone())),
    );
    reply.send("second call must be a no-op");
    let got = replies.lock().unwrap();
    assert_eq!(got.len(), 1);
    let parsed: serde_json::Value = serde_json::from_str(&got[0]).unwrap();
    assert_eq!(parsed["overlay"], "AudioPanel");
}

#[test]
fn demo_tag_commands_stage_bulk_work_and_filter_without_persisting() {
    let mut app = test_app();
    let mut anime_city = wall("anime-city.png", "static", 1, 0);
    anime_city["tags"] = json!("anime,city");
    let mut anime = wall("anime.png", "static", 2, 0);
    anime["tags"] = json!("anime");
    let mut city = wall("city.png", "static", 3, 0);
    city["tags"] = json!("city");
    seed(&mut app, &[anime_city, anime, city]);
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };

    cmd(&mut app, "demo begin");
    cmd(&mut app, "tag-mode begin");
    cmd(&mut app, "tag-select 0");
    cmd(&mut app, "tag-select 1");
    cmd(&mut app, "tag-bulk add curated");
    assert!(app.tags.mode);
    assert_eq!(app.tags.select.len(), 2);
    assert_eq!(app.tags.mass_tags, ["curated"]);
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.update_tags"));

    cmd(&mut app, "tag-mode end");
    assert!(!app.tags.mode);
    assert!(app.tags.select.is_empty());
    assert!(app.tags.mass_tags.is_empty());

    cmd(&mut app, "tag-query anime city");
    assert!(app.tags.cloud_open);
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    assert_eq!(app.library_session.filters.tags, ["anime", "city"]);
    assert_eq!(filtered_names(&app), ["anime-city.png"]);
    cmd(&mut app, "tag-match any");
    assert_eq!(app.library_session.filtered.len(), 3);
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.update_tags"));
}

#[test]
fn demo_selection_resolves_pending_filters_and_reveals_an_authored_target() {
    let mut app = test_app();
    let mut still = wall("still.png", "static", 1, 0);
    still["tags"] = json!("cat");
    let video = wall("motion.mp4", "video", 2, 0);
    seed(&mut app, &[still, video]);
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };

    cmd(&mut app, "filter cat");
    cmd(&mut app, "select motion.mp4");
    assert_eq!(
        crate::app::update::ui_state_json(&app).parse::<serde_json::Value>().unwrap()["selection"],
        "still.png"
    );

    cmd(&mut app, "demo begin");
    cmd(&mut app, "filter cat");
    cmd(&mut app, "kind static");
    cmd(&mut app, "select motion.mp4");

    assert_eq!(
        crate::app::update::ui_state_json(&app).parse::<serde_json::Value>().unwrap()["selection"],
        "motion.mp4"
    );
    assert_eq!(app.library_session.filters.kind, "video");
    assert!(app.library_session.filters.tags.is_empty());
}

#[test]
fn demo_selection_keeps_the_authored_wallpaper_away_from_the_list_edges() {
    let mut app = test_app();
    let wallpapers = (0..15)
        .map(|index| wall(&format!("wall-{index}.png"), "static", index, index))
        .collect::<Vec<_>>();
    seed(&mut app, &wallpapers);
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };

    cmd(&mut app, "demo begin");
    assert_eq!(app.scene.current, 6);
    cmd(&mut app, "select wall-0.png");

    assert_eq!(app.scene.current, 6);
    assert_eq!(
        crate::app::update::ui_state_json(&app).parse::<serde_json::Value>().unwrap()["selection"],
        "wall-0.png"
    );
    assert!(app.scene.current > 0);
    assert!(app.scene.current + 1 < app.library_session.filtered.len());
}

#[test]
fn demo_geometric_selection_uses_the_middle_grid_column() {
    let mut app = test_app();
    let wallpapers = (0..80)
        .map(|index| wall(&format!("wall-{index}.png"), "static", index, index))
        .collect::<Vec<_>>();
    seed(&mut app, &wallpapers);
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };

    cmd(&mut app, "demo begin");
    cmd(&mut app, "mode hex");
    cmd(&mut app, "tune components.wallpaperSelector.hexRows 5");
    cmd(&mut app, "tune components.wallpaperSelector.hexCols 12");
    cmd(&mut app, "select wall-0.png");

    assert_eq!(app.scene.current, 32);
    assert_eq!(
        crate::app::update::ui_state_json(&app).parse::<serde_json::Value>().unwrap()["selection"],
        "wall-0.png"
    );
}

#[test]
fn demo_scroll_advances_one_wall_at_a_time() {
    let mut app = test_app();
    let walls = (0..24)
        .map(|index| wall(&format!("wall-{index}.png"), "static", index, 0))
        .collect::<Vec<_>>();
    seed(&mut app, &walls);
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "demo begin");
    command(&mut app, "mode slices");
    let start = app.scene.current;
    command(&mut app, "scroll-demo -2.5");
    assert!(app.animating());

    app.tick_demo_scroll(0.2);
    assert_eq!(app.scene.current, start);
    app.tick_demo_scroll(0.2);
    assert_eq!(app.scene.current, start + 1);

    command(&mut app, "scroll-demo stop");
    app.tick_demo_scroll(1.0);
    assert_eq!(app.scene.current, start + 1);
}

#[test]
fn demo_session_transient_restores_state() {
    let mut app = test_app();
    let mut a = wall("a.png", "static", 1, 0);
    a["tags"] = json!("cat");
    a["width"] = json!(2560);
    a["height"] = json!(1440);
    let mut b = wall("b.png", "static", 2, 0);
    b["width"] = json!(1920);
    b["height"] = json!(1080);
    seed(&mut app, &[a, b]);
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };

    cmd(&mut app, "mode hex");
    cmd(&mut app, "filter cat");
    let before = app.config.root().clone();
    let before_palette = app.theme.palette;
    let before_base_palette = app.theme.base_palette;
    assert_eq!(filtered_names(&app), ["a.png"]);

    cmd(&mut app, "demo begin");
    assert!(app.runtime_state.demo.is_some());
    assert!(app.config.is_transient());
    assert_eq!(app.library_session.filtered.len(), 2);
    cmd(&mut app, "picker-demo hide");
    assert!(app.runtime_state.demo.as_ref().is_some_and(|session| session.picker_suppressed));
    cmd(&mut app, "picker-demo show");
    assert!(!app.runtime_state.demo.as_ref().is_some_and(|session| session.picker_suppressed));
    cmd(&mut app, "tune motion.slowMs 460");
    assert!((app.scene.filter_swap_ms() - 460.0).abs() < 0.01);
    cmd(&mut app, "motion 0.22");
    assert!((app.scene.motion_scale() - 0.22).abs() < 0.001);
    cmd(&mut app, "sort minimalist");
    assert_eq!(app.library_session.filters.sort, "minimalist");
    cmd(&mut app, "colour orange");
    assert_eq!(app.library_session.filters.color, 1);
    cmd(&mut app, "colour any");
    assert_eq!(app.library_session.filters.color, -1);
    cmd(&mut app, "tab theme");
    cmd(&mut app, "section 1");
    assert!(app.panels.settings.open);
    assert_eq!(app.panels.settings.tab, "theme");
    assert_eq!(app.panels.settings.section, 1);
    cmd(&mut app, "open theme");
    cmd(&mut app, "recolour dracula");
    assert_eq!(app.config.str_path(skwd_config::keys::theme::STATIC_THEME), "dracula");
    assert_eq!(app.config.str_path(skwd_config::keys::theme::POLICY), "fixed");
    assert_eq!(app.config.theme_backend(), "static");
    assert_eq!(
        app.panels.theme_designer.as_ref().and_then(|designer| designer.selected_preset()),
        Some("dracula")
    );
    cmd(&mut app, "close");
    assert_ne!(app.theme.fade_to.primary, before_palette.primary);
    cmd(&mut app, "dismiss");
    cmd(&mut app, "open playlists");
    drain_calls(&app);
    cmd(&mut app, "playlist-demo reset");
    cmd(&mut app, "playlist-demo add a.png");
    cmd(&mut app, "playlist-demo add b.png");
    cmd(&mut app, "playlist-demo order shuffle");
    cmd(&mut app, "playlist-demo move b.png -1");
    cmd(&mut app, "playlist-demo dwell 8");
    cmd(&mut app, "playlist-demo activate");
    let playlists = app.panels.playlists.as_ref().unwrap();
    assert!(playlists.demo);
    assert_eq!(playlists.selected_def().unwrap().name, "The baddest cut");
    assert_eq!(playlists.selected_def().unwrap().order, "shuffle");
    assert_eq!(playlists.selected_def().unwrap().dwell, 8);
    assert_eq!(playlists.selected_def().unwrap().count, 2);
    assert_eq!(playlists.members[0].key.as_deref(), Some("b.png"));
    assert_eq!(playlists.assign, [("*".into(), -1)]);
    assert!(drain_calls(&app).is_empty());
    cmd(&mut app, "dismiss");
    cmd(&mut app, "mode sandy");
    cmd(&mut app, "select b.png");
    assert_eq!(
        crate::app::update::ui_state_json(&app).parse::<serde_json::Value>().unwrap()["selection"],
        "b.png"
    );
    drain_calls(&app);
    cmd(&mut app, "apply DP-1 inkwell-drop 220");
    let calls = drain_calls(&app);
    let (_, apply) = calls.iter().find(|(method, _)| method == "wall.apply").expect("demo apply");
    assert_eq!(apply["output"], "DP-1");
    assert_eq!(apply["transition"], true);
    assert_eq!(apply["transition_shader"], "inkwell-drop");
    assert_eq!(apply["transition_duration_ms"], 220);
    assert_eq!(apply["notify"], false);
    cmd(&mut app, "resolution 1920x1080");
    assert_eq!(filtered_names(&app), ["b.png"]);

    cmd(&mut app, "demo end");
    assert!(app.runtime_state.demo.is_none());
    assert!(!app.config.is_transient());
    assert_eq!(app.config.root(), &before);
    assert_eq!(app.theme.palette.primary, before_palette.primary);
    assert_eq!(app.theme.palette.background, before_palette.background);
    assert_eq!(app.theme.base_palette.primary, before_base_palette.primary);
    assert_eq!(app.theme.base_palette.background, before_base_palette.background);
    assert!((app.scene.motion_scale() - 1.0).abs() < 0.001);
    assert_eq!(app.scene.mode, Mode::Hex);
    assert_eq!(filtered_names(&app), ["a.png"]);
    assert_eq!(
        crate::app::update::ui_state_json(&app).parse::<serde_json::Value>().unwrap()["selection"],
        "a.png"
    );
}

#[test]
fn demo_apply_video_transition_arm() {
    let mut app = test_app();
    seed(
        &mut app,
        &[json!({
            "key": "video:forest.mp4",
            "name": "forest.mp4",
            "type": "video",
            "thumb": "/thumbs/forest.webp"
        })],
    );
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };

    cmd(&mut app, "demo begin");
    cmd(&mut app, "select video:forest.mp4");
    drain_calls(&app);
    cmd(&mut app, "apply * soft-warp-fade 260");

    let calls = drain_calls(&app);
    let (_, apply) = calls.iter().find(|(method, _)| method == "wall.apply").expect("demo apply");
    assert_eq!(apply["type"], "video");
    assert_eq!(apply["path"], "/vids/forest.mp4");
    assert_eq!(apply["output"], "*");
    assert_eq!(apply["transition_shader"], "soft-warp-fade");
    assert_eq!(apply["transition_duration_ms"], 260);
}

#[test]
fn demo_apply_source_ignores_filters() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            json!({
                "key": "static:hero.webp",
                "name": "hero.webp",
                "type": "static",
                "thumb": "/thumbs/hero.webp"
            }),
            json!({
                "key": "video:forest.mp4",
                "name": "forest.mp4",
                "type": "video",
                "thumb": "/thumbs/forest.webp"
            }),
        ],
    );
    let cmd = |app: &mut App, command: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(command.to_string())),
        );
    };

    cmd(&mut app, "demo begin");
    cmd(&mut app, "kind static");
    cmd(&mut app, "select static:hero.webp");
    drain_calls(&app);
    cmd(&mut app, "apply-source video:forest.mp4");
    cmd(&mut app, "apply * smoke 260");

    let calls = drain_calls(&app);
    let (_, apply) = calls.iter().find(|(method, _)| method == "wall.apply").expect("demo apply");
    assert_eq!(apply["type"], "video");
    assert_eq!(apply["path"], "/vids/forest.mp4");
    assert_eq!(apply["transition_shader"], "smoke");
    assert_eq!(
        crate::app::update::ui_state_json(&app).parse::<serde_json::Value>().unwrap()["selection"],
        "static:hero.webp"
    );
}

#[test]
fn grid_card_becomes_effects_source() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            json!({
                "key": "static:first.webp",
                "name": "first.webp",
                "type": "static",
                "path": "/walls/first.webp",
                "thumb": "/thumbs/first.webp"
            }),
            json!({
                "key": "static:chosen.webp",
                "name": "chosen.webp",
                "type": "static",
                "path": "/walls/chosen.webp",
                "thumb": "/thumbs/chosen.webp"
            }),
        ],
    );
    app.scene.mode = Mode::Grid;
    assert_eq!(app.scene.current, 0);

    let _ = crate::app::helpers::apply_task(&mut app, 1);
    let selected = app.scene.current;
    crate::app::helpers::open_effects(
        &mut app,
        selected,
        crate::frontend::effects::EffectsMode::Studio,
    );

    assert_eq!(app.scene.current, 1);
    assert_eq!(app.panels.effects.as_ref().unwrap().source_path(), "/wp/chosen.webp");
}

#[test]
fn demo_blur_geometry_and_cleanup() {
    let source = std::env::temp_dir().join(format!(
        "skwd-demo-blur-source-{}-{}.png",
        std::process::id(),
        TEST_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    image::RgbaImage::from_pixel(32, 18, image::Rgba([40, 80, 120, 255])).save(&source).unwrap();
    let mut app = test_app();
    app.library_session.wallpaper_dir = source.parent().unwrap().to_string_lossy().into_owned();
    let wallpaper = wall(source.file_name().unwrap().to_str().unwrap(), "static", 210, 0);
    seed(&mut app, &[wallpaper]);
    let source_key = app.library_session.library.catalog().items[0].key.clone();
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };
    command(&mut app, "demo begin");
    drain_calls(&app);
    command(&mut app, &format!("apply-source {source_key}"));
    command(&mut app, "stage-blur 4");
    assert!(app.runtime_state.demo.as_ref().unwrap().apply_source.is_none());

    let calls = drain_calls(&app);
    let (_, apply) = calls.iter().find(|(method, _)| method == "wall.apply").expect("blur apply");
    let blurred = std::path::PathBuf::from(apply["path"].as_str().unwrap());
    assert!(blurred.starts_with(std::path::Path::new(&app.config.cache_dir()).join("demo")));
    assert!(blurred.is_file());
    assert_eq!(image::image_dimensions(&blurred).unwrap(), (32, 18));
    assert_eq!(apply["no_transition"], true);

    drain_calls(&app);
    command(&mut app, "demo end");
    let calls = drain_calls(&app);
    let (_, restore) =
        calls.iter().find(|(method, _)| method == "wall.apply").expect("sharp restore");
    assert_eq!(restore["path"], source.to_string_lossy().as_ref());
    assert_eq!(restore["no_transition"], true);
    assert!(!blurred.exists());
    std::fs::remove_file(source).unwrap();
}

#[test]
fn demo_badge_suppression_survives_refilters() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "demo begin");
    command(&mut app, "badges hide");
    command(&mut app, "search colourful anime girls");
    assert!(app.tags.cloud_open);
    assert!(app.runtime_state.demo.as_ref().unwrap().type_badges_suppressed);

    command(&mut app, "badges show");
    command(&mut app, "search mountains beside oceans and lakes");
    assert!(app.tags.cloud_open);
    assert!(!app.runtime_state.demo.as_ref().unwrap().type_badges_suppressed);
}

#[test]
fn schedule_demo_in_memory_nested() {
    let mut app = test_app();
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "schedule-demo open");
    assert!(app.panels.schedule.is_none());

    command(&mut app, "demo begin");
    let _ = drain_calls(&app);
    command(&mut app, "schedule-demo open");
    let editor = app.panels.schedule.as_ref().expect("demo schedule opens");
    assert_eq!(editor.rows.len(), 3);
    assert_eq!(editor.rows[0].name, "Rain after dark");
    assert!(editor.demo);

    command(&mut app, "schedule-demo select 1");
    command(&mut app, "schedule-demo edit 1 1");
    let editor = app.panels.schedule.as_ref().expect("demo schedule stays open");
    assert_eq!(editor.selected, 1);
    assert_eq!(editor.editing, Some(crate::frontend::schedule_editor::Editing::Node(1, vec![1])));
    assert!(matches!(
        editor.node(1, &[1]).map(|node| &node.kind),
        Some(crate::domain::schedule::ConditionKind::Group {
            operator: crate::domain::schedule::GroupOperator::Any,
            ..
        })
    ));

    command(&mut app, "schedule-demo close");
    assert!(app.panels.schedule.is_none());
    assert!(drain_calls(&app).is_empty());

    command(&mut app, "schedule-demo open");
    command(&mut app, "dismiss");
    assert!(app.panels.schedule.is_none());
    assert!(drain_calls(&app).is_empty());
}

#[test]
fn dismiss_with_nothing_open_stays() {
    let mut app = test_app();
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };
    assert!(app.topmost_overlay().is_none());

    command(&mut app, "dismiss");
    command(&mut app, "dismiss");

    command(&mut app, "open mixer");
    assert!(app.panels.audio.is_some());
}

#[test]
fn display_commands_drive_picker() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };
    let state = |app: &App| -> Value {
        serde_json::from_str(&crate::app::update::ui_state_json(app)).unwrap()
    };
    let screens = || {
        ["DP-1", "DP-2", "DP-3"]
            .into_iter()
            .map(|name| crate::frontend::effects::MonitorInfo {
                name: name.to_string(),
                target: name.to_string(),
                connected: true,
                width: 1920,
                height: 1080,
                current_thumb: None,
                kind: WallpaperKind::Static,
                mute: true,
                volume: 0,
                fill: String::from("fill"),
                locked: false,
                paused: false,
                manual_paused: false,
                current: String::from("/old.png"),
                we_id: String::new(),
            })
            .collect::<Vec<_>>()
    };

    command(&mut app, "display toggle 0");
    assert!(state(&app)["displays"].is_null());

    command(&mut app, "open displays");
    let _ = drain_calls(&app);
    command(&mut app, "display apply");
    assert!(drain_calls(&app).is_empty());
    assert!(app.panels.effects.is_some());
    app.panels.effects.as_mut().unwrap().set_monitors(screens());
    command(&mut app, "display hover 0");
    command(&mut app, "display toggle 0");
    command(&mut app, "display toggle 2");
    command(&mut app, "display toggle DP-9");
    let displays = state(&app)["displays"].clone();
    assert_eq!(displays["mode"], "displays");
    assert_eq!(displays["monitors"], json!(["DP-1", "DP-2", "DP-3"]));
    assert_eq!(displays["selected"], json!(["DP-1", "DP-3"]));

    let _ = drain_calls(&app);
    command(&mut app, "display apply");
    let applies: Vec<Value> = drain_calls(&app)
        .into_iter()
        .filter(|(method, _)| method == "wall.apply")
        .map(|(_, params)| params)
        .collect();
    let mut targets: Vec<String> = applies
        .iter()
        .map(|params| params["output"].as_str().unwrap_or_default().to_string())
        .collect();
    targets.sort();
    assert_eq!(targets, ["DP-1", "DP-3"]);
    assert!(applies.iter().all(|params| params["override_locks"] == true));
    assert!(app.panels.effects.is_none());

    command(&mut app, "open displays");
    app.panels.effects.as_mut().unwrap().set_monitors(screens());
    command(&mut app, "display all");
    assert_eq!(state(&app)["displays"]["targets"], json!(["*"]));
    let _ = drain_calls(&app);
    command(&mut app, "display apply");
    let applies: Vec<Value> = drain_calls(&app)
        .into_iter()
        .filter(|(method, _)| method == "wall.apply")
        .map(|(_, params)| params)
        .collect();
    assert_eq!(applies.len(), 1);
    assert_eq!(applies[0]["output"], "*");
    assert_eq!(applies[0]["override_locks"], true);
}

#[test]
fn demo_display_names_no_race() {
    let mut app = test_app();
    seed(&mut app, &[wall("portrait.webp", "static", 1, 0)]);
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };

    command(&mut app, "demo begin");
    command(&mut app, "open displays");
    let _ = drain_calls(&app);
    command(&mut app, "display hover DP-2");
    command(&mut app, "display toggle DP-2");
    command(&mut app, "display apply");

    let applies: Vec<Value> = drain_calls(&app)
        .into_iter()
        .filter(|(method, _)| method == "wall.apply")
        .map(|(_, params)| params)
        .collect();
    assert_eq!(applies.len(), 1);
    assert_eq!(applies[0]["output"], "DP-2");
    assert_eq!(applies[0]["path"], "/wp/portrait.webp");
}

#[test]
fn demo_display_staging() {
    let mut app = test_app();
    let mut wide = wall("wide.png", "static", 1, 0);
    wide["key"] = json!("static:wide.png");
    let mut tall = wall("tall.png", "static", 2, 0);
    tall["key"] = json!("static:tall.png");
    seed(&mut app, &[wide, tall]);
    let command = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_string())),
        );
    };
    let screens = || {
        [("DP-1", 1920, 1080), ("DP-2", 1080, 1920)]
            .into_iter()
            .map(|(name, width, height)| crate::frontend::effects::MonitorInfo {
                name: name.to_string(),
                target: name.to_string(),
                connected: true,
                width,
                height,
                current_thumb: None,
                kind: WallpaperKind::Static,
                mute: true,
                volume: 0,
                fill: String::from("fill"),
                locked: false,
                paused: false,
                manual_paused: false,
                current: String::from("/old.png"),
                we_id: String::new(),
            })
            .collect::<Vec<_>>()
    };

    command(&mut app, "display stage DP-1 static:wide.png");
    assert!(drain_calls(&app).is_empty());

    command(&mut app, "demo begin");
    command(&mut app, "open displays");
    app.panels.effects.as_mut().unwrap().set_monitors(screens());
    let _ = drain_calls(&app);

    command(&mut app, "display stage DP-1 static:wide.png");
    command(&mut app, "display stage DP-2 static:tall.png");

    assert!(drain_calls(&app).is_empty());

    let effects = app.panels.effects.as_ref().expect("display picker stays open");
    assert_eq!(effects.source_path(), "/wp/tall.png");
    assert!(effects.output_selected("DP-2"));
    assert!(!effects.output_selected("DP-1"));
    assert_eq!(effects.monitors()[0].current, "/wp/wide.png");
    assert_eq!(effects.monitors()[1].current, "/wp/tall.png");
}
