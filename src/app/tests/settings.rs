use super::*;

#[test]
fn noctalia_mode_persists_before_retheme() {
    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("config.json");
    let mut config = Config::from_data(json!({"noctalia": {"themeMode": "follow"}}));
    config.config_path.clone_from(&config_path);
    config.persist();
    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed_at_call = std::sync::Arc::clone(&observed);
    let mut app = App::with_config_using(config, move |_| {
        crate::infrastructure::ipc::DaemonClient::recording_with_observer(move |method, _| {
            if method == "wall.retheme" {
                let root: Value =
                    serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
                observed_at_call.lock().unwrap().push(root["noctalia"]["themeMode"].clone());
            }
        })
    });
    for mode in ["keep", "dark", "light", "auto", "follow"] {
        let _ = update(
            &mut app,
            Message::Settings(crate::frontend::settings::SettingsMsg::Pick(
                skwd_config::keys::noctalia::THEME_MODE.to_string(),
                mode.to_string(),
            )),
        );
    }
    assert_eq!(*observed.lock().unwrap(), ["keep", "dark", "light", "auto", "follow"]);
}

#[test]
fn settings_reload_sees_persisted() {
    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("config.json");
    let mut config = Config::from_data(json!({
        "schedule": { "enabled": false },
        "paths": {
            "cache": directory.path().join("cache").to_string_lossy(),
            "wallpaper": "/wp",
            "videoWallpaper": "/vids"
        }
    }));
    config.config_path.clone_from(&config_path);
    config.persist();

    let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed_at_call = std::sync::Arc::clone(&observed);
    let path_at_call = config_path.clone();
    let mut app = App::with_config_using(config, move |_| {
        crate::infrastructure::ipc::DaemonClient::recording_with_observer(move |method, _| {
            if method == "schedule.reload" {
                let text = std::fs::read_to_string(&path_at_call).unwrap();
                let root: Value = serde_json::from_str(&text).unwrap();
                observed_at_call.lock().unwrap().push(root["schedule"]["enabled"].clone());
            }
        })
    });

    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::Toggle(
            skwd_config::keys::schedule::ENABLED.to_string(),
            true,
        )),
    );

    assert_eq!(*observed.lock().unwrap(), [json!(true)]);
}

#[test]
fn imported_pack_registered() {
    let mut app = test_app();
    let pack = crate::infrastructure::semantic_pack::ImportedSemanticPack {
        format: 1,
        id: String::from("example/preferred-model"),
        version: String::from("v2"),
        dimensions: 768,
        manifest: std::path::PathBuf::from("/models/preferred/semantic-pack.json"),
        bytes: 600 * 1_024 * 1_024,
    };

    let _ = update(&mut app, Message::SemanticModelImported(Ok(Some(pack))));

    let models = app.config.array_values(skwd_config::keys::semantic::MODELS);
    assert_eq!(models.len(), 1);
    assert_eq!(models[0]["id"], "example/preferred-model");
    assert_eq!(models[0]["version"], "v2");
    assert_eq!(models[0]["dimensions"], 768);
    assert_eq!(models[0]["managed"], true);
    assert_eq!(
        app.config.str_path(skwd_config::keys::semantic::MANIFEST),
        "/models/preferred/semantic-pack.json"
    );
    assert!(app.panels.settings.semantic_import_status.contains("600 MiB"));
}

#[test]
fn reimported_pack_not_duplicated() {
    let mut app = test_app();
    let pack = crate::infrastructure::semantic_pack::ImportedSemanticPack {
        format: 1,
        id: String::from("example/model"),
        version: String::from("v1"),
        dimensions: 512,
        manifest: std::path::PathBuf::from("/models/example/semantic-pack.json"),
        bytes: 42,
    };

    let _ = update(&mut app, Message::SemanticModelImported(Ok(Some(pack.clone()))));
    let _ = update(&mut app, Message::SemanticModelImported(Ok(Some(pack))));

    assert_eq!(app.config.array_len(skwd_config::keys::semantic::MODELS), 1);
}

#[test]
fn schedule_toggle_persists() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::schedule::ENABLED, json!(false));
    crate::app::helpers::sched_open(&mut app);
    assert!(!app.panels.schedule.as_ref().unwrap().enabled);
    drain_calls(&app);

    let _ = update(
        &mut app,
        Message::Sched(crate::frontend::schedule_editor::SchedMsg::SetEnabled(true)),
    );

    assert!(app.panels.schedule.as_ref().unwrap().enabled);
    assert!(app.config.flag_default_config(skwd_config::keys::schedule::ENABLED));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "schedule.reload"));
}

#[test]
fn schedule_rule_toggle_keeps_rule() {
    let mut app = test_app();
    crate::app::helpers::sched_open(&mut app);
    app.panels.schedule.as_mut().unwrap().add_rule();
    drain_calls(&app);

    let _ = update(
        &mut app,
        Message::Sched(crate::frontend::schedule_editor::SchedMsg::SetRuleEnabled(0, false)),
    );

    let editor = app.panels.schedule.as_ref().unwrap();
    assert_eq!(editor.rows.len(), 1);
    assert!(!editor.rows[0].enabled);
    assert_eq!(app.config.array_values(skwd_config::keys::schedule::RULES)[0]["enabled"], false);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "schedule.reload"));
}

#[test]
fn settings_toggle_persists() {
    let mut app = test_app();
    let cfg_path = app.config.config_path.clone();
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.panels.settings.open);
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == "theme.backends"));
    assert_eq!(app.panels.settings.entrance.x, 0.0);
    assert!(!app.panels.settings.entrance.settled());
    assert!(!app.panels.settings.inputs.is_empty());
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(!app.panels.settings.open);
    let text = std::fs::read_to_string(&cfg_path).expect("closing settings persists config");
    let v: Value = serde_json::from_str(&text).unwrap();
    assert!(v.is_object());
    assert!(!cfg_path.with_extension("json.tmp").exists());
}

#[test]
fn workshop_settings_reload_boundary() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    drain_calls(&app);

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Toggle(
            skwd_config::keys::we_render::DISABLE_PARTICLES.into(),
            true,
        )),
    );
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.reload_we"));

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Pick(
            skwd_config::keys::we_render::SCALING.into(),
            String::from("fit"),
        )),
    );
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.reload_we"));

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(
            skwd_config::keys::we_render::FPS.into(),
            String::from("48"),
        )),
    );
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.reload_we"));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.reload_we"));
}

#[test]
fn settings_invalidation_no_reentry() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    app.theme.fade_t.run(0.0, 1.0);

    app.invalidate_settings();

    assert!(app.runtime_state.last_tick.is_none());
}

#[test]
fn overlay_flushes_pending_edits() {
    let mut app = test_app();
    let cfg_path = app.config.config_path.clone();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::Input(
            skwd_config::keys::general::MAX_FPS.into(),
            String::from("144"),
        )),
    );

    let _ = update(&mut app, Message::OpenBrowser(String::from("wallhaven")));

    assert!(!app.panels.settings.open);
    let text = std::fs::read_to_string(&cfg_path).expect("opening browser persists settings edits");
    let value: Value = serde_json::from_str(&text).expect("valid persisted config");
    assert_eq!(value["general"]["maxFps"], 144.0);
}

#[test]
fn settings_preview_tracks_latest_selection() {
    let mut app = test_app();
    seed(&mut app, &[wall("first.png", "static", 1, 0), wall("second.png", "static", 2, 0)]);

    let first = app.selected_settings_preview_path().to_string();
    assert_eq!(first, "/thumbs/first.png.png");
    let _ = app.sync_settings_preview();
    assert!(app.panels.settings.preview_in_flight_path.is_empty());

    let _ = update(&mut app, Message::ToggleSettings);
    assert_eq!(app.panels.settings.preview_in_flight_path, first);

    app.scene.set_current(1, app.library_session.filtered.len());
    let second = app.selected_settings_preview_path().to_string();
    let _ = app.sync_settings_preview();
    assert_eq!(app.panels.settings.preview_desired_path, second);
    assert_eq!(app.panels.settings.preview_in_flight_path, first);

    app.finish_settings_preview(&first, None);
    assert!(app.panels.settings.preview_in_flight_path.is_empty());
    assert!(app.panels.settings.preview_allocated_path.is_empty());
    let _ = app.sync_settings_preview();
    assert_eq!(app.panels.settings.preview_in_flight_path, second);

    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.panels.settings.preview_desired_path.is_empty());
    assert!(app.panels.settings.preview_in_flight_path.is_empty());
    assert!(app.panels.settings.preview_allocated_path.is_empty());
    assert!(app.panels.settings.preview_allocation.is_none());
}

#[test]
fn settings_preview_uses_bounded_thumbnail() {
    let mut app = test_app();
    seed(&mut app, &[wall("large.png", "static", 1, 0)]);
    let original = app.config.config_path.parent().unwrap().join("large-original.png");
    std::fs::write(&original, b"full-resolution-placeholder").unwrap();
    edit_catalog(&mut app, |catalog| {
        catalog.items[0].path = original.to_string_lossy().into_owned();
    });

    assert_eq!(app.selected_settings_preview_path(), "/thumbs/large.png.png");
}

#[test]
fn settings_preview_missing_video_fallback() {
    let mut app = test_app();
    let mut video = wall("clip.mp4", "video", 1, 0);
    video["preview"] = json!("/definitely/missing/skwd-preview.webp");
    seed(&mut app, &[video]);

    assert_eq!(app.selected_settings_preview_path(), "/thumbs/clip.mp4.png");
}

#[test]
fn settings_pick_live() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    assert_eq!(app.scene.xp.sandy.swap_style, 1.0);
    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::Pick(
            "components.wallpaperSelector.sandySwapStyle".into(),
            "castle".into(),
        )),
    );
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 8);
    assert_eq!(app.scene.xp.sandy.swap_style, 3.0);
}

#[test]
fn placement_reapplies_one_output() {
    let mut app = test_app();
    app.daemon.output_statuses = vec![crate::contracts::daemon::OutputStatus {
        name: String::from("DP-2"),
        kind: crate::contracts::media::MediaKind::Static,
        path: String::from("/walls/current.png"),
        mute: true,
        volume: 42,
        ..Default::default()
    }];
    drain_calls(&app);

    let _ = update(
        &mut app,
        Message::Settings(crate::frontend::settings::SettingsMsg::Pick(
            String::from("display.fillModes.DP-2"),
            String::from("fit"),
        )),
    );

    assert_eq!(app.config.str_path("display.fillModes.DP-2"), "fit");
    let calls = drain_calls(&app);
    let (_, params) =
        calls.iter().find(|(method, _)| method == "wall.apply").expect("reapply sent");
    assert_eq!(params["output"], "DP-2");
    assert_eq!(params["path"], "/walls/current.png");
    assert_eq!(params["no_transition"], true);
    assert_eq!(params["notify"], false);
}

#[test]
fn layout_studio_inset_while_live() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.panels.settings.open);
    assert_eq!(app.panels.settings.tab, "picker");
    assert_eq!(app.panels.settings.inset_target, 0.0);

    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(1)));
    assert!(app.panels.settings.inset_target > 0.0);
    assert!(!app.scene.hidden());

    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(4)));
    assert_eq!(app.panels.settings.inset_target, 0.0);
    assert_eq!(app.panels.settings.section, 4);
    assert_eq!(app.panels.settings.control_page, 0);
    assert_eq!(app.panels.settings.section_anim.x, 0.0);

    let _ = update(&mut app, Message::SetSettingsTab(String::from("picker")));
    assert_eq!(app.panels.settings.tab, "picker");
    assert_eq!(app.panels.settings.section, 0);
    assert_eq!(app.panels.settings.control_page, 0);
    assert_eq!(app.panels.settings.inset_target, 0.0);

    let _ = update(&mut app, Message::SetSettingsTab(String::from("search")));
    assert_eq!(app.panels.settings.tab, "filter");
    assert_eq!(app.panels.settings.inset_target, 0.0);
    assert_eq!(app.panels.settings.section_anim.x, 0.0);

    let _ = update(&mut app, Message::SetSettingsTab(String::from("motion")));
    assert_eq!(app.panels.settings.tab, "motion");
    assert_eq!(app.panels.settings.inset_target, 0.0);
    assert_eq!(app.panels.settings.section_anim.x, 0.0);

    let _ = update(&mut app, Message::SetSettingsTab(String::from("picker")));
    assert_eq!(app.panels.settings.tab, "picker");
    assert_eq!(app.panels.settings.inset_target, 0.0);
}

#[test]
fn stack_bars_toggle_only_reveal() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    let key = skwd_config::keys::keybind::PLAYLISTS;
    let before = app.config.str_path(key);
    let id = String::from("setting-group:motion.fast");

    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleBar(id.clone(), 0)));
    let reveal = app.panels.settings.bar_reveals.get(&id).expect("open bar reveal");
    assert_eq!(reveal.target, 1.0);
    assert_eq!(reveal.x, 0.0);
    assert_eq!(app.config.str_path(key), before);

    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleBar(id.clone(), 0)));
    let reveal = app.panels.settings.bar_reveals.get(&id).expect("closing bar reveal");
    assert_eq!(reveal.target, 0.0);
    assert_eq!(app.config.str_path(key), before);
}

#[test]
fn layout_studio_returns_to_settings() {
    use crate::frontend::settings::{SettingsFocus, SettingsMsg};

    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(1)));
    assert!(app.panels.settings.inset_target > 0.0);
    assert!(!app.scene.hidden());

    let _ = update(&mut app, Message::Settings(SettingsMsg::LeaveLayoutStudio));

    assert!(app.panels.settings.open);
    assert_eq!(app.panels.settings.tab, "picker");
    assert_eq!(app.panels.settings.section, 0);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    assert_eq!(app.panels.settings.inset_target, 0.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    assert!(app.scene.hidden());
}

#[test]
fn studio_saves_applies_on_canvas() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    app.scene.viewport = (1366.0, 768.0);
    let mode = app.config.selector_mode();
    let before = app.config.selector_presets(&mode).len();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(1)));
    let _ = update(&mut app, Message::SavePreset);
    let selected = app.config.selected_preset(&mode).expect("new preset becomes active");
    assert_eq!(selected, "Style 1");
    assert_eq!(app.config.selector_presets(&mode).len(), before + 1);
    let persisted: Value =
        serde_json::from_str(&std::fs::read_to_string(&app.config.config_path).unwrap()).unwrap();
    assert_eq!(
        persisted["components"]["wallpaperSelector"]["presets"][mode.as_str()][0]["name"],
        selected
    );

    let _ = update(&mut app, Message::ApplyPreset(mode, selected));
    assert!(app.panels.settings.open);
    assert_eq!(app.panels.settings.section, 1);
    assert!(!app.scene.hidden());
}

#[test]
fn workbench_keyboard_navigation() {
    use crate::frontend::settings::{SettingsFocus, SettingsKey, SettingsMsg};

    let mut app = test_app();
    app.scene.viewport = (1536.0, 960.0);
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::SetSettingsTab(String::from("picker")));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    assert_eq!(app.panels.settings.section, 0);

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    assert_eq!(app.panels.settings.section, 1);

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Up)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    assert_eq!(app.panels.settings.section, 0);

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Previous)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Next)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);

    let _ = update(&mut app, Message::SetSettingsTab(String::from("sources")));

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Next)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    assert_eq!(app.panels.settings.focused_control, 0);

    let before = app.config.flag_default_true(skwd_config::keys::features::WALLHAVEN);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert!(app.panels.settings.expanded_details.contains("source.wallhaven"));
    assert_eq!(app.config.flag_default_true(skwd_config::keys::features::WALLHAVEN), before);

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Next)));
    assert_eq!(app.panels.settings.focused_control, 0);

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
    assert_eq!(app.panels.settings.focused_control, 1);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Up)));
    assert_eq!(app.panels.settings.focused_control, 0);

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Previous)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Next)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    assert_eq!(app.panels.settings.focused_control, 1);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: true })),
    );
    assert_eq!(app.panels.settings.focused_control, 0);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: true })),
    );
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: true })),
    );
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);
    let tab_before = app.panels.settings.tab.clone();
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);
    assert_ne!(app.panels.settings.tab, tab_before);

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
}

#[test]
fn workbench_mouse_tab_owns_focus() {
    use crate::frontend::settings::{SettingsFocus, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    app.panels.settings.focus = SettingsFocus::Controls;

    let _ =
        update(&mut app, Message::Settings(SettingsMsg::SelectTab(String::from("performance"))));

    assert_eq!(app.panels.settings.tab, "performance");
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);
}

#[test]
fn workbench_tab_leaves_search() {
    use crate::frontend::settings::{SettingsFocus, SettingsKey, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    assert!(app.panels.settings.search_open);

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );

    assert!(!app.panels.settings.search_open);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
}

#[test]
fn workbench_tab_cycles_focus() {
    use crate::frontend::settings::{SettingsFocus, SettingsKey, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);

    let tab = |app: &mut App, backwards| {
        let _ =
            update(app, Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards })));
    };
    tab(&mut app, false);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    assert_eq!(app.panels.settings.focused_control, 0);
    tab(&mut app, false);
    assert_ne!((app.panels.settings.control_page, app.panels.settings.focused_control), (0, 0));
    tab(&mut app, true);
    assert_eq!(app.panels.settings.control_page, 0);
    assert_eq!(app.panels.settings.focused_control, 0);
    tab(&mut app, true);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    tab(&mut app, true);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);
    tab(&mut app, false);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
}

#[test]
fn workbench_ctrl_tab_cycles_pages() {
    use crate::frontend::settings::{SettingsFocus, SettingsKey, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::SetSettingsTab(String::from("picker")));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::CategoryNext { backwards: false })),
    );
    assert_eq!(app.panels.settings.tab, "filter");
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::CategoryNext { backwards: true })),
    );
    assert_eq!(app.panels.settings.tab, "picker");
}

#[test]
fn workbench_tab_walks_grouped_fields() {
    use crate::frontend::settings::{SettingsFocus, SettingsKey, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::SetSettingsTab(String::from("motion")));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(
        app.panels.settings.input_edit.as_ref().map(|edit| edit.key.as_str()),
        Some(skwd_config::keys::motion::FAST_MS)
    );

    for key in [skwd_config::keys::motion::STANDARD_MS, skwd_config::keys::motion::SLOW_MS] {
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
        );
        assert_eq!(
            app.panels.settings.input_edit.as_ref().map(|edit| edit.key.as_str()),
            Some(key)
        );
        assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    }

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    assert!(app.panels.settings.input_edit.is_none());
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);
}

#[test]
fn settings_search_opens_canonical_control() {
    use crate::frontend::settings::{SettingsFocus, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    assert!(app.panels.settings.search_open);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::SearchInput(String::from("battery saver"))),
    );
    let result = app
        .panels
        .settings
        .search_results
        .iter()
        .find(|result| result.tab == "performance")
        .cloned()
        .expect("performance search result");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::OpenSearchResult(result.tab, result.section, result.row)),
    );
    assert_eq!(app.panels.settings.tab, "performance");
    assert_eq!(app.panels.settings.section, result.section);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    assert!(!app.panels.settings.search_open);
    assert!(app.panels.settings.search_query.is_empty());
}

#[test]
fn settings_search_inline_keybind() {
    use crate::frontend::settings::{SettingsFocus, SettingsMsg};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::SearchInput(String::from("toggle playlists"))),
    );
    let result = app
        .panels
        .settings
        .search_results
        .iter()
        .find(|result| result.tab == "picker")
        .cloned()
        .expect("shortcut search result");

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::OpenSearchResult(result.tab, result.section, result.row)),
    );

    assert_eq!(app.panels.settings.section, 4);
    assert!(app.panels.settings.bar_reveals.is_empty());
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
}

#[test]
fn workbench_enter_commits_escape_restores() {
    use crate::frontend::settings::{SettingsKey, SettingsMsg};

    let mut app = test_app();
    app.scene.viewport = (1536.0, 960.0);
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::SetSettingsTab(String::from("picker")));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );

    let key = skwd_config::keys::general::UI_SCALE;
    let original = app.config.num_path(key);
    let original_input = app.panels.settings.inputs.get(key).cloned().unwrap();

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert!(app.panels.settings.input_edit.is_some());
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(key.to_string(), String::from("1.75"))),
    );
    assert_eq!(app.config.num_path(key), 1.75);

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert!(app.panels.settings.input_edit.is_none());
    assert_eq!(app.config.num_path(key), original);
    assert_eq!(app.panels.settings.inputs.get(key), Some(&original_input));

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Input(key.to_string(), String::from("1.65"))),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert!(app.panels.settings.input_edit.is_none());
    assert_eq!(app.config.num_path(key), 1.65);
}

#[test]
fn workbench_escape_leaves_choice() {
    use crate::frontend::settings::{SettingsKey, SettingsMsg};

    let mut app = test_app();
    app.scene.viewport = (1536.0, 960.0);
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert!(app.panels.settings.focused_choice.is_some());

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert!(app.panels.settings.open);
    assert!(app.panels.settings.focused_choice.is_none());

    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert!(!app.panels.settings.open);
}

#[test]
fn sync_library_search_join() {
    let numeric = crate::domain::library::search::NumericQuery::default();
    assert_eq!(super::sync_library_search(&[], &numeric), "");
    assert_eq!(super::sync_library_search(&[String::from("sunset")], &numeric), "sunset ");
    assert_eq!(
        super::sync_library_search(&[String::from("sunset"), String::from("-city")], &numeric,),
        "sunset -city "
    );
}

#[test]
fn sync_library_search_keeps_numeric_predicates() {
    let numeric = crate::domain::library::search::parse_numeric_query("width:>=1920 duration:<30s");
    assert_eq!(
        super::sync_library_search(&[String::from("sunset")], &numeric),
        "sunset width:>=1920 duration:<30s "
    );
}

#[test]
fn layout_studio_releases_preview() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    seed(&mut app, &[wall("first.png", "static", 1, 0)]);
    let _ = update(&mut app, Message::ToggleSettings);
    let first = app.panels.settings.preview_in_flight_path.clone();
    assert!(!first.is_empty());
    app.finish_settings_preview(&first, None);

    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(1)));

    assert!(app.panels.settings.preview_desired_path.is_empty());
    assert!(app.panels.settings.preview_in_flight_path.is_empty());
    assert!(app.panels.settings.preview_allocated_path.is_empty());
    assert!(app.panels.settings.preview_allocation.is_none());

    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(0)));
    assert_eq!(app.panels.settings.preview_in_flight_path, first);
}

#[test]
fn hard_overlay_releases_inset() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(1)));
    assert!(app.panels.settings.inset_target > 0.0);

    app.input.help_open = true;
    let _ = update(&mut app, Message::Noop);
    assert!(app.picker_obscured());
    assert_eq!(app.panels.settings.inset_target, 0.0);

    app.input.help_open = false;
    let _ = update(&mut app, Message::Noop);
    assert!(app.panels.settings.inset_target > 0.0);
}
