use super::*;

#[test]
fn outdoors_policy_persists_separately() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    app.config.save_key(skwd_config::keys::transition::SHADER, json!("glitch"));
    app.config.save_key(skwd_config::keys::theme::POLICY, json!("static"));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Pick(
            skwd_config::keys::weather::OUTDOORS_TRANSITION_SHADER.into(),
            "sand-helix".into(),
        )),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Toggle(
            skwd_config::keys::weather::OUTDOORS_RETHEME.into(),
            true,
        )),
    );
    assert_eq!(
        app.config.str_path(skwd_config::keys::weather::OUTDOORS_TRANSITION_SHADER),
        "sand-helix"
    );
    assert!(app.config.flag_default_config(skwd_config::keys::weather::OUTDOORS_RETHEME));
    assert_eq!(app.config.str_path(skwd_config::keys::transition::SHADER), "glitch");
    assert_eq!(app.config.str_path(skwd_config::keys::theme::POLICY), "static");
}

#[test]
fn lockscreen_waits_for_daemon() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    use skwd_config::keys::lock_screens as keys;

    let mut app = test_app();
    let action = Message::Settings(SettingsMsg::Run(ActionId::SetLockScreen(
        wall_proto::LockScreenTarget::Hyprlock,
        true,
    )));
    let _ = update(&mut app, action.clone());
    let _ = update(&mut app, action);
    let calls = drain_calls(&app);
    let changes: Vec<_> =
        calls.iter().filter(|(method, _)| method == wall_proto::rpc::LOCKSCREEN_SET).collect();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].1, json!({"target":"hyprlock","enabled":true}));
    assert!(!app.config.flag_default_config(keys::HYPRLOCK));
    std::fs::write(
        &app.config.config_path,
        serde_json::to_vec(&json!({"lockScreens":{"mode":"follow","hyprlock":true}})).unwrap(),
    )
    .unwrap();
    app.on_result(
        Pending::LockScreenSet {
            target: wall_proto::LockScreenTarget::Hyprlock,
            enabled: true,
            sddm_theme: None,
        },
        &json!({"ok":true,"target":"hyprlock","enabled":true}),
    );
    assert!(app.config.flag_default_config(keys::HYPRLOCK));
}

#[test]
fn lockscreen_error_keeps_state() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    use skwd_config::keys::lock_screens as keys;

    let mut app = test_app();
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::SetLockScreen(
            wall_proto::LockScreenTarget::Swaylock,
            true,
        ))),
    );
    app.rpc_error(
        Pending::LockScreenSet {
            target: wall_proto::LockScreenTarget::Swaylock,
            enabled: true,
            sddm_theme: None,
        },
        "config conflict".into(),
    );
    assert!(!app.config.flag_default_config(keys::SWAYLOCK));
}

#[test]
fn source_keeps_targets_disabled() {
    use crate::frontend::settings::SettingsMsg;
    use skwd_config::keys::lock_screens as keys;

    for mode in [keys::FOLLOW, keys::STATIC] {
        let mut app = test_app();
        app.config.set_key(keys::MODE, json!(keys::OFF));
        for key in keys::TARGET_FLAGS {
            app.config.set_key(key, json!(true));
        }
        app.config.set_key(keys::SDDM_IMAGE, json!("/var/lib/skwd/lockscreens/1000/wallpaper.png"));
        let _ =
            update(&mut app, Message::Settings(SettingsMsg::Pick(keys::MODE.into(), mode.into())));
        assert_eq!(app.config.str_path(keys::MODE), mode);
        for key in keys::TARGET_FLAGS {
            assert!(!app.config.flag_default_config(key));
        }
        assert!(app.config.str_path(keys::SDDM_IMAGE).is_empty());
        let saved: Value =
            serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
        assert_eq!(saved["lockScreens"]["mode"], mode);
        assert_eq!(saved["lockScreens"]["hyprlock"], false);
        assert!(
            !drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::LOCKSCREEN_SET)
        );
    }
}

#[test]
fn sddm_rollback_deferred() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    use crate::infrastructure::runtime::Wake;

    let mut app = test_app();
    let request = Pending::LockScreenSet {
        target: wall_proto::LockScreenTarget::Sddm,
        enabled: true,
        sddm_theme: None,
    };
    app.daemon.pending.insert(42, request);
    let task = update(
        &mut app,
        Message::Daemon(Wake::Ipc(IpcMsg::Response {
            id: 42,
            result: None,
            error: Some(wall_proto::ErrorInfo { code: 1, message: "export failed".into() }),
        })),
    );
    assert!(iced_runtime::task::into_stream(task).is_some());
    assert!(app.panels.settings.sddm_checking);
    assert!(app.panels.settings.sddm_rollback.is_none());
    assert!(
        app.runtime_state.toast.as_ref().is_some_and(|toast| toast.0.contains("export failed"))
    );
    assert!(app.config.str_path(skwd_config::keys::lock_screens::SDDM_IMAGE).is_empty());

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::SetLockScreen(
            wall_proto::LockScreenTarget::Hyprlock,
            true,
        ))),
    );
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::LOCKSCREEN_SET));

    let _ = update(
        &mut app,
        Message::SddmRollback { error: "export failed".into(), result: Ok(String::new()) },
    );
    assert!(!app.panels.settings.sddm_checking);
    assert!(
        app.runtime_state.toast.as_ref().is_some_and(|toast| toast.0.contains("export failed"))
    );
    assert!(app.config.str_path(skwd_config::keys::lock_screens::SDDM_IMAGE).is_empty());
}

#[test]
fn sddm_rollback_failure() {
    let mut app = test_app();
    app.panels.settings.sddm_checking = true;
    let _ = update(
        &mut app,
        Message::SddmRollback {
            error: "export failed".into(),
            result: Err("permission denied".into()),
        },
    );
    assert!(!app.panels.settings.sddm_checking);
    let toast = &app.runtime_state.toast.as_ref().unwrap().0;
    assert!(toast.contains("export failed"));
    assert!(toast.contains("permission denied"));
    assert!(app.config.str_path(skwd_config::keys::lock_screens::SDDM_IMAGE).is_empty());
}

#[test]
fn sddm_disable_omits_image() {
    let mut app = test_app();
    let _ = update(
        &mut app,
        Message::SddmSetup {
            enabled: false,
            result: Ok(crate::infrastructure::lockscreen::SddmSetup {
                path: "/var/lib/skwd/lockscreens/1000/wallpaper.png".into(),
                theme: Some("maldives".into()),
            }),
        },
    );
    let calls = drain_calls(&app);
    let (_, request) =
        calls.iter().find(|(method, _)| method == wall_proto::rpc::LOCKSCREEN_SET).unwrap();
    assert_eq!(request, &json!({"target":"sddm","enabled":false}));
}

#[test]
fn sddm_choice_can_cancel() {
    use crate::frontend::settings::{ActionId, SettingsMsg};

    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.sddm_checking = true;
    let _ = update(
        &mut app,
        Message::SddmChoices(Ok(crate::infrastructure::lockscreen::SddmChoices {
            current: String::new(),
            themes: vec!["breeze".into()],
        })),
    );
    assert_eq!(app.panels.settings.sddm_themes, ["breeze"]);
    assert!(!app.panels.settings.sddm_checking);
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::LOCKSCREEN_SET));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ChooseSddmTheme(None))));
    assert!(app.panels.settings.sddm_themes.is_empty());
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::LOCKSCREEN_SET));
}

#[test]
fn sddm_path_reaches_daemon() {
    let mut app = test_app();
    let path = "/var/lib/skwd/lockscreens/1000/wallpaper.png";
    let _ = update(
        &mut app,
        Message::SddmSetup {
            enabled: true,
            result: Ok(crate::infrastructure::lockscreen::SddmSetup {
                path: path.into(),
                theme: None,
            }),
        },
    );
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| method == wall_proto::rpc::LOCKSCREEN_SET
        && params == &json!({"target":"sddm","enabled":true,"sddm_image":path})));
    assert!(app.config.str_path(skwd_config::keys::lock_screens::SDDM_IMAGE).is_empty());
}

#[test]
fn folder_choices_persist() {
    use crate::app::state::SettingsInputEdit;
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{ActionId, SettingsMsg};

    for kind in [FolderKind::Source, FolderKind::Excluded] {
        let mut app = test_app();
        app.config.save_key(kind.key(), json!([]));
        for path in ["/walls/nature", "/walls/nature", "/walls/city"] {
            let _ = update(&mut app, Message::FolderChosen(kind, None, Ok(Some(path.into()))));
        }
        assert_eq!(
            app.config.array_values(kind.key()),
            [json!("/walls/nature"), json!("/walls/city")]
        );
        let _ = update(&mut app, Message::FolderChosen(kind, None, Ok(None)));
        assert_eq!(app.config.array_len(kind.key()), 2);
        let _ = update(
            &mut app,
            Message::FolderChosen(kind, Some(0), Ok(Some("/walls/forest".into()))),
        );
        let indexed = format!("{}.0", kind.key());
        app.panels.settings.input_edit = Some(SettingsInputEdit {
            key: indexed.clone(),
            original: "/walls/forest".into(),
            we_dirty: false,
            playlist_dirty: false,
            schedule_dirty: false,
            weather_refresh: crate::app::state::WeatherRefresh::Clean,
        });
        let _ =
            update(&mut app, Message::Settings(SettingsMsg::Input(indexed.clone(), String::new())));
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::Input(indexed, "/walls/edited".into())),
        );
        let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
        assert_eq!(
            app.config.array_values(kind.key()),
            [json!("/walls/edited"), json!("/walls/city")]
        );
        let saved: Value =
            serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
        let setting = kind.key().split('.').next_back().unwrap();
        assert_eq!(saved["library"][setting], json!(["/walls/edited", "/walls/city"]));
        for _ in 0..2 {
            let _ = update(
                &mut app,
                Message::Settings(SettingsMsg::Run(ActionId::RemoveFolder(kind, 0))),
            );
        }
        assert!(app.config.array_values(kind.key()).is_empty());
    }
}

fn edit_source(app: &mut App, raw: &str) -> String {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::SettingsMsg;
    app.config.save_key(FolderKind::Source.key(), json!(["/old"]));
    app.init_settings_inputs();
    let key = format!("{}.0", FolderKind::Source.key());
    let _ = update(app, Message::Settings(SettingsMsg::EditFolder(key.clone())));
    assert_eq!(
        app.panels.settings.input_edit.as_ref().map(|edit| edit.key.as_str()),
        Some(key.as_str())
    );
    let _ = update(app, Message::Settings(SettingsMsg::Input(key.clone(), raw.into())));
    key
}

fn add_draft(app: &mut App, kind: crate::contracts::settings::FolderKind, raw: &str) {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    let _ = update(app, Message::Settings(SettingsMsg::FolderDraft(kind, raw.into())));
    let _ = update(app, Message::Settings(SettingsMsg::Run(ActionId::AddFolder(kind))));
}

#[test]
fn draft_folders_persist() {
    use crate::contracts::settings::FolderKind;
    for kind in [FolderKind::Source, FolderKind::Excluded] {
        let mut app = test_app();
        app.config.set_key(kind.key(), json!(["/existing"]));
        add_draft(&mut app, kind, "  /added ");
        let saved: Value =
            serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
        assert_eq!(skwd_config::get(&saved, kind.key()).unwrap(), &json!(["/existing", "/added"]));
        assert!(!app.panels.settings.folder_drafts.contains_key(kind.key()));
    }
}

#[test]
fn draft_duplicate_flashes() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.config.save_key(FolderKind::Source.key(), json!(["/wp"]));
    add_draft(&mut app, FolderKind::Source, "/wp/");
    assert_eq!(app.config.array_values(FolderKind::Source.key()), [json!("/wp")]);
    let flash = crate::frontend::settings::flash_key("library.folders.0");
    assert!(app.panels.settings.bar_reveals.get(&flash).is_some_and(|tween| tween.x > 0.0));
    assert!(app.panels.settings.open);
}

#[test]
fn removed_folder_undoes() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{ActionId, SettingsMsg};
    let mut app = test_app();
    let excluded = FolderKind::Excluded;
    app.config.save_key(excluded.key(), json!(["/a", "/b", "/c"]));
    let remove = ActionId::RemoveFolder(excluded, 1);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(remove)));
    assert_eq!(app.config.array_values(excluded.key()), [json!("/a"), json!("/c")]);
    let undo = ActionId::UndoFolderRemove;
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(undo)));
    assert_eq!(app.config.array_values(excluded.key()), [json!("/a"), json!("/b"), json!("/c")]);
    assert!(app.panels.settings.folder_undo.is_none());
}

#[test]
fn undo_commits_open_edit() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{ActionId, SettingsKey, SettingsMsg};
    let mut app = test_app();
    app.panels.settings.open = true;
    let excluded = FolderKind::Excluded;
    app.config.save_key(excluded.key(), json!(["/a", "/b", "/c"]));
    app.init_settings_inputs();
    let remove = ActionId::RemoveFolder(excluded, 0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(remove)));
    let key = format!("{}.1", excluded.key());
    let _ = update(&mut app, Message::Settings(SettingsMsg::EditFolder(key.clone())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key, "/c2".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::UndoFolderRemove)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert_eq!(app.config.array_values(excluded.key()), [json!("/a"), json!("/b"), json!("/c2")]);
}

#[test]
fn folder_inputs_seeded() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    app.config.save_key(FolderKind::Source.key(), json!(["/wp"]));
    app.config.save_key(FolderKind::Excluded.key(), json!(["raw"]));
    app.init_settings_inputs();
    let inputs = &app.panels.settings.inputs;
    assert_eq!(inputs.get("library.folders.0").map(String::as_str), Some("/wp"));
    assert_eq!(inputs.get("library.excludedFolders.0").map(String::as_str), Some("raw"));
}

#[test]
fn resolved_folder_normalizes() {
    let home = skwd_config::home();
    assert_eq!(crate::app::settings_source::resolved_folder("~/walls/../x/"), format!("{home}/x"));
}

#[test]
fn section_change_drops_undo() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{ActionId, SettingsMsg};
    let mut app = test_app();
    app.config.save_key(FolderKind::Source.key(), json!(["/a"]));
    let remove = ActionId::RemoveFolder(FolderKind::Source, 0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(remove)));
    assert!(app.panels.settings.folder_undo.is_some());
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(2)));
    assert!(app.panels.settings.folder_undo.is_none());
}

#[test]
fn chooser_preserves_legacy_paths() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::paths::WALLPAPER, json!("/old/images"));
    app.config.set_key(skwd_config::keys::paths::VIDEO_WALLPAPER, json!("/old/videos"));
    let _ = update(&mut app, Message::FolderChosen(FolderKind::Source, None, Ok(None)));
    assert!(skwd_config::get(app.config.root(), FolderKind::Source.key()).is_none());
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Source, None, Ok(Some("/extra".into()))),
    );
    assert_eq!(
        app.config.array_values(FolderKind::Source.key()),
        [json!("/old/images"), json!("/old/videos"), json!("/extra")]
    );
    assert_eq!(app.config.str_path(skwd_config::keys::paths::WALLPAPER), "/old/images");
    assert_eq!(app.config.str_path(skwd_config::keys::paths::VIDEO_WALLPAPER), "/old/videos");
}

#[test]
fn chooser_reveals_scanned_folder() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    app.panels.settings.open = true;
    seed(&mut app, &[wall("/existing/old.png", "static", 1, 0)]);
    app.library_session.filters.sort = "date".into();
    app.library_session.filters.kind = "video".into();
    app.library_session.filters.favourites_only = true;
    app.tags.semantic.search = "mountain".into();
    app.tags.semantic.resolved = true;
    let search_generation = app.tags.semantic.generation;
    let _ =
        update(&mut app, Message::FolderChosen(FolderKind::Source, None, Ok(Some("/new".into()))));
    assert_eq!(app.library_session.filters.folder, "/new");
    assert!(!app.panels.settings.open);
    assert_eq!(app.library_session.filters.sort, "date");
    assert!(app.library_session.filters.kind.is_empty());
    assert!(!app.library_session.filters.favourites_only);
    assert!(app.tags.semantic.search.is_empty());
    assert_ne!(app.tags.semantic.generation, search_generation);
    assert!(app.library_session.folder_options.contains(&"/new".into()));

    app.on_result(
        Pending::List,
        &json!({"wallpapers": [wall("/existing/old.png", "static", 1, 0)]}),
    );
    assert_eq!(app.library_session.filters.folder, "/new");
    app.on_result(Pending::List, &json!({"wallpapers": [
        wall("/existing/old.png", "static", 1, 0),
        wall("/new/photo.png", "static", 2, 0),
        json!({"name": "clip.mp4", "type": "video", "thumb": "/thumbs/clip.png", "video_file": "/new/clip.mp4", "hue": 3})
    ]}));
    assert_eq!(filtered_names(&app), ["/new/photo.png", "clip.mp4"]);
}

#[test]
fn chooser_replaces_source() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.config.save_key(FolderKind::Source.key(), json!(["/old"]));
    seed(&mut app, &[wall("/old/a.png", "static", 1, 0)]);
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Source, Some(0), Ok(Some("/new".into()))),
    );
    assert_eq!(app.config.array_values(FolderKind::Source.key()), [json!("/new")]);
    assert_eq!(app.library_session.filters.folder, "/new");
    assert!(!app.panels.settings.open);
    app.on_result(Pending::List, &json!({"wallpapers": [wall("/old/a.png", "static", 1, 0)]}));
    assert_eq!(app.library_session.filters.folder, "/new");
    app.on_result(Pending::List, &json!({"wallpapers": [wall("/new/b.png", "static", 1, 0)]}));
    assert_eq!(filtered_names(&app), ["/new/b.png"]);
}

#[test]
fn all_tracks_sources() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{ActionId, SettingsMsg};

    let mut app = test_app();
    seed(&mut app, &[wall("root.png", "static", 1, 0)]);
    let mut source = wall("/new/a.png", "static", 2, 0);
    source["key"] = json!("static:a");
    let _ =
        update(&mut app, Message::FolderChosen(FolderKind::Source, None, Ok(Some("/new".into()))));
    app.on_result(Pending::List, &json!({"wallpapers": [
        wall("root.png", "static", 1, 0),
        source,
        json!({"key": "video:b", "name": "b.mp4", "type": "video", "thumb": "/thumbs/b.png", "video_file": "/new/b.mp4"}),
    ]}));
    let _ = update(&mut app, Message::SetFolder(String::new()));
    assert_eq!(app.library_session.filters.folder, "*");
    assert_eq!(filtered_names(&app), ["root.png", "/new/a.png", "b.mp4"]);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::RemoveFolder(FolderKind::Source, 2))),
    );
    assert_eq!(filtered_names(&app), ["root.png", "/new/a.png", "b.mp4"]);
    let _ = update(&mut app, Message::SetFolder("*".into()));
    assert_eq!(app.library_session.filtered.len(), 3);
}

#[test]
fn roots_reload_menu() {
    let mut app = test_app();
    let mut root = wall("root.png", "static", 1, 0);
    root["key"] = json!("static:root");
    let mut source = wall("/other/new.png", "static", 2, 0);
    source["key"] = json!("static:new");
    let walls = [root, source];
    seed(&mut app, &walls);
    let _ = update(&mut app, Message::SetFolder(String::new()));
    assert_eq!(filtered_names(&app), ["root.png", "/other/new.png"]);
    std::fs::write(
        &app.config.config_path,
        json!({"library": {
            "folders": ["/other/../other"]
        }})
        .to_string(),
    )
    .unwrap();
    assert!(app.adopt_external_config());
    assert_eq!(app.library_session.folder_roots, ["/other"]);
    assert_eq!(filtered_names(&app), ["root.png", "/other/new.png"]);
    assert_eq!(app.library_session.folder_options, ["*", "/other"]);
    std::fs::write(
        &app.config.config_path,
        json!({"library": {
            "folders": ["/wp"]
        }})
        .to_string(),
    )
    .unwrap();
    assert!(app.adopt_external_config());
    assert_eq!(filtered_names(&app), ["root.png", "/other/new.png"]);
}

#[test]
fn empty_roots_reload() {
    let mut app = test_app();
    app.library_session.filters.folder = "/wp/pending".into();
    app.rebuild_folder_options();
    assert_eq!(app.library_session.filters.folder, "/wp/pending");
    std::fs::write(
        &app.config.config_path,
        json!({"library": {"folders": ["/other"]}}).to_string(),
    )
    .unwrap();
    assert!(app.adopt_external_config());
    assert_eq!(app.library_session.folder_options, ["*", "/other"]);
    assert_eq!(app.library_session.filters.folder, "*");
}

#[test]
fn empty_default_shows_all() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::filter_bar::DEFAULT_FOLDER, json!(""));
    let cards = crate::frontend::settings::build_tab(
        "filter",
        &app.config,
        &[],
        &app.library_session.folder_options,
        "",
        &[],
    );
    assert!(cards.iter().flat_map(|(_, rows)| rows).any(|row| matches!(
        &row.control,
        crate::frontend::settings::Control::Dropdown { path, current, options, .. }
            if path == skwd_config::keys::filter_bar::DEFAULT_FOLDER
                && current == "*"
                && options.iter().any(|(key, _)| key == current)
    )));
}

#[test]
fn removed_source_clears_sticky() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{ActionId, SettingsMsg};

    let mut app = test_app();
    app.config.set_key("filterBar.sticky", json!(true));
    let _ =
        update(&mut app, Message::FolderChosen(FolderKind::Source, None, Ok(Some("/new".into()))));
    assert_eq!(app.config.last_filter_folder(), "/new");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::RemoveFolder(FolderKind::Source, 2))),
    );
    assert_eq!(app.library_session.filters.folder, "*");
    assert_eq!(app.config.last_filter_folder(), "*");
}

#[test]
fn chooser_ignores_noops() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.config.save_key(FolderKind::Source.key(), json!(["/existing"]));
    let _ = update(&mut app, Message::FolderChosen(FolderKind::Source, None, Ok(None)));
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Source, None, Ok(Some("/existing".into()))),
    );
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Excluded, None, Ok(Some("/excluded".into()))),
    );
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Source, None, Ok(Some("/other/../existing".into()))),
    );
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Source, Some(0), Ok(Some("/existing/".into()))),
    );
    assert_eq!(app.library_session.filters.folder, "*");
    assert!(app.panels.settings.open);
    assert_eq!(app.config.array_values(FolderKind::Source.key()), [json!("/existing")]);
}

#[test]
fn chooser_lists_dot_source() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Source, None, Ok(Some("/walls/.private".into()))),
    );
    assert!(app.library_session.folder_options.contains(&"/walls/.private".into()));
}

#[test]
fn excluded_paths_remain_literal() {
    use crate::contracts::settings::FolderKind;
    let mut app = test_app();
    app.config.save_key(FolderKind::Excluded.key(), json!(["nature"]));
    let absolute = std::env::current_dir().unwrap().join("nature");
    let _ = update(
        &mut app,
        Message::FolderChosen(FolderKind::Excluded, Some(0), Ok(Some(absolute.clone()))),
    );
    assert_eq!(app.config.array_values(FolderKind::Excluded.key()), [json!(absolute)]);
}

#[test]
fn manual_reveals_on_commit() {
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    app.panels.settings.open = true;
    edit_source(&mut app, "/manual");
    assert_eq!(app.library_session.filters.folder, "*");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(app.library_session.filters.folder, "/manual");
    assert!(!app.panels.settings.open);
    let _ = update(&mut app, Message::SetFolder("*".into()));
    app.on_result(Pending::List, &json!({"wallpapers": [wall("/manual/a.png", "static", 1, 0)]}));
    assert_eq!(app.library_session.filters.folder, "*");
}

#[test]
fn manual_reveals_on_navigation() {
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    app.panels.settings.open = true;
    edit_source(&mut app, "~/photos/../walls/");
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(0)));
    assert_eq!(app.library_session.filters.folder, format!("{}/walls", skwd_config::home()));
    assert!(app.library_session.folder_options.contains(&app.library_session.filters.folder));
}

#[test]
fn manual_cancel_keeps_folder() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{SettingsKey, SettingsMsg};
    let mut app = test_app();
    app.panels.settings.open = true;
    edit_source(&mut app, "/cancelled");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Cancel)));
    assert_eq!(app.library_session.filters.folder, "*");
    assert!(app.panels.settings.open);
    assert_eq!(app.config.array_values(FolderKind::Source.key()), [json!("/old")]);
}

#[test]
fn manual_same_path_stays() {
    use crate::app::state::SettingsInputEdit;
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.config.save_key(FolderKind::Source.key(), json!(["/existing"]));
    app.init_settings_inputs();
    let key = format!("{}.0", FolderKind::Source.key());
    app.panels.settings.input_edit = Some(SettingsInputEdit {
        key: key.clone(),
        original: "/existing".into(),
        we_dirty: false,
        playlist_dirty: false,
        schedule_dirty: false,
        weather_refresh: crate::app::state::WeatherRefresh::Clean,
    });
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key, "/existing/".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(app.library_session.filters.folder, "*");
    assert!(app.panels.settings.open);
}

#[test]
fn manual_blank_path_stays() {
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    app.panels.settings.open = true;
    edit_source(&mut app, "  ");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(app.library_session.filters.folder, "*");
    assert!(app.panels.settings.open);
}

#[test]
fn removes_high_folder_index() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{ActionId, SettingsMsg};
    let mut app = test_app();
    let folders = (0..65_538).map(|index| format!("folder-{index}")).collect::<Vec<_>>();
    app.config.set_key(FolderKind::Excluded.key(), json!(folders));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::RemoveFolder(FolderKind::Excluded, 65_536))),
    );
    let remaining = app.config.array_values(FolderKind::Excluded.key());
    assert_eq!(remaining.len(), 65_537);
    assert_eq!(remaining[0], "folder-0");
    assert_eq!(remaining[65_536], "folder-65537");
}

#[test]
fn matugen_enable_saves_the_generator_before_retheme_and_announces_the_switch() {
    use crate::frontend::settings::SettingsMsg;
    use skwd_config::keys;

    for initial in [
        json!({}),
        json!({"features": {"matugen": true}, "theme": {"engine": "wallust"}}),
        json!({"theme": {"policy": "fixed", "staticTheme": "nord"}}),
        json!({"theme": {"policy": "off"}}),
        json!({"theme": {"authority": "dms", "targets": ["noctalia"]}}),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.json");
        let mut config = Config::from_data(initial);
        config.config_path.clone_from(&path);
        config.set_key(keys::matugen::DEFAULT_CONFIG, json!("/existing/config.toml"));
        config.persist();
        let before = config.root().clone();
        let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let seen = std::sync::Arc::clone(&observed);
        let mut app = App::with_config_using(config, move |_| {
            crate::infrastructure::ipc::DaemonClient::recording_with_observer(move |method, _| {
                if method == "wall.retheme" {
                    let saved: Value =
                        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
                    seen.lock().unwrap().push(saved);
                }
            })
        });
        assert_eq!(app.config.root(), &before);
        drain_calls(&app);
        app.theme.shell_preview_sent = Some(0);
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::Toggle(keys::features::MATUGEN.into(), true)),
        );
        let calls = drain_calls(&app);
        let methods: Vec<_> = calls.iter().map(|(method, _)| method.as_str()).collect();
        assert!(
            methods.iter().position(|method| *method == "wall.shell_preview_end").unwrap()
                < methods.iter().position(|method| *method == "wall.retheme").unwrap()
        );
        let saved = observed.lock().unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0]["features"]["matugen"], true);
        assert_eq!(saved[0]["theme"]["policy"], "wallpaper");
        assert_eq!(saved[0]["theme"]["authority"], "skwd");
        assert_eq!(saved[0]["theme"]["engine"], "matugen");
        assert_eq!(saved[0]["defaultMatugenConfig"], "/existing/config.toml");
        assert_eq!(saved[0]["theme"]["targets"], before["theme"]["targets"]);
        assert!(app.runtime_state.toast.as_ref().is_some_and(|(text, _)| text == crate::i18n::tr("settings-matugen-enabled-notice")));
    }
}

#[test]
fn disabling_matugen_uses_iris_and_preserves_the_external_command() {
    use crate::frontend::settings::SettingsMsg;
    use skwd_config::keys;

    let mut app = App::with_config(Config::from_data(json!({
        "theme": {"policy": "wallpaper", "authority": "skwd", "engine": "matugen"},
        "defaultMatugenConfig": "/existing/config.toml", "externalMatugenCommand": "custom %path%"
    })));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Toggle(keys::features::MATUGEN.into(), false)),
    );
    assert_eq!(app.config.theme_backend(), "skwd-iris");
    assert!(!app.config.flag_default_true(keys::features::MATUGEN));
    assert_eq!(app.config.str_path(keys::matugen::DEFAULT_CONFIG), "/existing/config.toml");
    assert_eq!(app.config.str_path(keys::system::EXTERNAL_MATUGEN_COMMAND), "custom %path%");
    assert!(
        app.runtime_state
            .toast
            .as_ref()
            .is_some_and(|(text, _)| text == crate::i18n::tr("settings-matugen-disabled-notice"))
    );
}

#[test]
fn disabling_inactive_matugen_preserves_the_current_driver() {
    use crate::frontend::settings::SettingsMsg;
    use skwd_config::keys;

    let mut app = App::with_config(Config::from_data(json!({"theme": {"authority": "dms"}})));
    drain_calls(&app);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Toggle(keys::features::MATUGEN.into(), false)),
    );
    assert_eq!(app.config.theme_backend(), "dms");
    assert!(drain_calls(&app).is_empty());
    assert!(app.runtime_state.toast.is_none());
}

#[test]
fn choosing_matugen_as_colour_source_reenables_its_command() {
    use crate::frontend::settings::SettingsMsg;
    use skwd_config::keys;

    for path in [keys::theme::ENGINE, keys::theme::BACKEND] {
        let mut app = App::with_config(Config::from_data(json!({"features": {"matugen": false}})));
        let _ =
            update(&mut app, Message::Settings(SettingsMsg::Pick(path.into(), "matugen".into())));
        assert_eq!(app.config.theme_backend(), "matugen");
        assert!(app.config.flag_default_true(keys::features::MATUGEN));
    }
}

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
fn process_picker_edits_targets() {
    use crate::frontend::settings::{ActionId, SettingsMsg};

    let mut app = test_app();
    app.config.set_key(skwd_config::keys::playback::PROCESSES, json!("mpv, gamescope"));
    app.daemon.playback.available_processes = vec![String::from("steam"), String::from("mpv")];

    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ChooseRunningProcess)));
    assert!(app.panels.settings.process_picker_open);

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ProcessPickerSearch(String::from("steam"))),
    );
    assert_eq!(app.panels.settings.process_picker_query, "steam");

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ProcessPickerManualInput(String::from("steam*"))),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::ProcessPickerManualAdd));
    assert!(app.config.str_path(skwd_config::keys::playback::PROCESSES).contains("steam*"));
    assert!(app.panels.settings.process_picker_manual.is_empty());

    let _ =
        update(&mut app, Message::Settings(SettingsMsg::ProcessPickerAdd(String::from("steam"))));
    assert_eq!(
        app.config.str_path(skwd_config::keys::playback::PROCESSES),
        "mpv, gamescope, steam*, steam"
    );

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ProcessPickerRemove(String::from("gamescope"))),
    );
    assert_eq!(app.config.str_path(skwd_config::keys::playback::PROCESSES), "mpv, steam*, steam");

    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::ProcessPickerRemove(String::from("steam*"))),
    );
    assert_eq!(app.config.str_path(skwd_config::keys::playback::PROCESSES), "mpv, steam");

    let _ = update(&mut app, Message::Settings(SettingsMsg::ProcessPickerClose));
    assert!(!app.panels.settings.process_picker_open);
    assert!(app.panels.settings.process_picker_query.is_empty());
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
fn weather_settings_refresh_status() {
    use crate::frontend::settings::SettingsMsg;

    let mut app = test_app();
    drain_calls(&app);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Pick(skwd_config::keys::weather::RAINY.into(), "42".into())),
    );
    assert_eq!(app.config.str_path(skwd_config::keys::weather::RAINY), "42");
    assert!(drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::WALL_WEATHER));
}

#[test]
fn outdoors_actions_call_service() {
    use crate::frontend::settings::{ActionId, SettingsMsg};

    let mut app = test_app();
    drain_calls(&app);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::FollowOutdoors(true))));
    assert_eq!(
        drain_calls(&app),
        [(wall_proto::rpc::WALL_WEATHER_FOLLOW.into(), json!({"enabled": true}))]
    );
    assert_eq!(app.config.str_path(skwd_config::keys::weather::SOURCE), "playlists");
    assert!(!app.config.flag_default_config(skwd_config::keys::weather::ENABLED));
    assert!(app.daemon.pending.values().any(|pending| *pending == Pending::WeatherAction));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::FollowOutdoors(true))));
    assert!(drain_calls(&app).is_empty());

    app.daemon.pending.clear();
    app.config.set_key(skwd_config::keys::weather::SOURCE, json!("outdoors"));
    app.config.set_key(skwd_config::keys::weather::ENABLED, json!(true));
    app.daemon.weather = Some(crate::contracts::daemon::WeatherStatus {
        source: crate::contracts::daemon::WeatherSource::Outdoors,
        enabled: true,
        outdoors: Some(crate::contracts::daemon::OutdoorStatus {
            outputs: vec![crate::contracts::daemon::OutdoorOutput {
                output: "DP-1".into(),
                key: "woods.png".into(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    });
    for (action, method, params) in [
        (
            ActionId::OutdoorNext("DP-1".into()),
            wall_proto::rpc::WALL_WEATHER_NEXT,
            json!({"output": "DP-1"}),
        ),
        (
            ActionId::OutdoorExclude { output: "DP-1".into(), key: "woods.png".into() },
            wall_proto::rpc::WALL_WEATHER_EXCLUDE,
            json!({"key": "woods.png"}),
        ),
        (
            ActionId::OutdoorPin("DP-1".into()),
            wall_proto::rpc::WALL_WEATHER_PIN,
            json!({"output": "DP-1"}),
        ),
        (
            ActionId::OutdoorResume("DP-1".into()),
            wall_proto::rpc::WALL_WEATHER_RESUME,
            json!({"output": "DP-1"}),
        ),
        (
            ActionId::FollowOutdoors(false),
            wall_proto::rpc::WALL_WEATHER_FOLLOW,
            json!({"enabled": false}),
        ),
    ] {
        let _ = update(&mut app, Message::Settings(SettingsMsg::Run(action)));
        assert_eq!(drain_calls(&app), [(method.into(), params)]);
        app.daemon.pending.clear();
    }
}

#[test]
fn source_flip_loads_catalog() {
    let mut app = test_app();
    app.config.persist();
    let path = app.config.config_path.clone();
    drain_calls(&app);

    let mut saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    saved["weather"]["source"] = json!("outdoors");
    saved["weather"]["enabled"] = json!(true);
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    assert!(app.adopt_external_config());
    assert!(drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::WALL_LIST));

    saved["weather"]["source"] = json!("playlists");
    saved["weather"]["enabled"] = json!(false);
    std::fs::write(&path, serde_json::to_vec(&saved).unwrap()).unwrap();
    assert!(app.adopt_external_config());
    assert!(drain_calls(&app).iter().any(|(method, _)| method == wall_proto::rpc::WALL_LIST));
}

#[test]
fn weather_review_saves_members() {
    use crate::frontend::settings::{ActionId, SettingsMsg, WeatherReviewTab};
    use skwd_config::keys::weather;

    let mut app = test_app();
    seed(&mut app, &[static_wall("a.png", 1), static_wall("b.png", 2)]);
    app.config.save_key(weather::SOURCE, json!(weather::SOURCE_SEMANTIC));
    app.config.save_key(weather::SEMANTIC_RAINY, json!(""));
    app.config.save_key(weather::APPROVED_RAINY, json!(["static:missing.png"]));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ReviewWeather("rainy"))));
    assert!(drain_calls(&app).iter().any(|(method, params)| {
        method == wall_proto::rpc::WALL_WEATHER_CANDIDATES && params["category"] == "rainy"
    }));
    assert!(app.panels.settings.weather_review_loading);
    let request = app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| {
            matches!(pending, Pending::WeatherCandidates { .. }).then_some(*id)
        })
        .unwrap();
    respond(
        &mut app,
        request,
        json!({
            "category": "rainy", "query": null,
            "member_keys": ["static:a.png"], "error": null
        }),
    );
    assert_eq!(app.panels.settings.weather_review_candidates, ["static:a.png"]);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::WeatherReviewToggle("static:a.png".into())),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::WeatherReviewTab(WeatherReviewTab::Approved)),
    );
    assert_eq!(
        crate::app::update::weather_review::visible_keys(&app),
        ["static:missing.png", "static:a.png"]
    );
    let saved: Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    assert_eq!(
        skwd_config::get(&saved, weather::APPROVED_RAINY).unwrap(),
        &json!(["static:missing.png", "static:a.png"])
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::WeatherReviewToggle("static:missing.png".into())),
    );
    assert_eq!(app.config.array_values(weather::APPROVED_RAINY), [json!("static:a.png")]);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::WeatherReviewTab(WeatherReviewTab::Library)),
    );
    assert!(
        crate::app::update::weather_review::visible_keys(&app).contains(&"static:b.png".into())
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::WeatherReviewToggle("static:b.png".into())),
    );
    assert_eq!(
        app.config.array_values(weather::APPROVED_RAINY),
        [json!("static:a.png"), json!("static:b.png")]
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(crate::frontend::settings::SettingsKey::Cancel)),
    );
    assert!(app.panels.settings.weather_review_category.is_none());
    assert_eq!(app.config.array_len(weather::APPROVED_RAINY), 2);
}

#[test]
fn weather_review_rejects_stale() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    use skwd_config::keys::weather;

    let mut app = test_app();
    app.config.save_key(weather::SEMANTIC_RAINY, json!("rain"));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ReviewWeather("rainy"))));
    let stale = app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| {
            matches!(pending, Pending::WeatherCandidates { .. }).then_some(*id)
        })
        .unwrap();
    app.config.save_key(weather::SEMANTIC_RAINY, json!("storm"));
    respond(
        &mut app,
        stale,
        json!({"category": "rainy", "query": "rain", "member_keys": ["static:a.png"], "error": null}),
    );
    assert!(app.panels.settings.weather_review_candidates.is_empty());
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ReviewWeather("rainy"))));
    let fresh = app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| {
            matches!(pending, Pending::WeatherCandidates { query, .. } if query == "storm")
                .then_some(*id)
        })
        .unwrap();
    respond(
        &mut app,
        fresh,
        json!({"category": "rainy", "query": "storm", "member_keys": [], "error": "model unavailable"}),
    );
    assert_eq!(app.panels.settings.weather_review_error.as_deref(), Some("model unavailable"));
    assert!(!app.panels.settings.weather_review_loading);
}

#[test]
fn approved_without_search() {
    use crate::contracts::daemon::{WeatherSelection, WeatherSource, WeatherState, WeatherStatus};
    use skwd_config::keys::{semantic, weather};

    let mut app = test_app();
    seed(&mut app, &[static_wall("a.png", 1)]);
    app.config.save_key(weather::SOURCE, json!(weather::SOURCE_SEMANTIC));
    app.config.save_key(weather::ENABLED, json!(true));
    app.config.save_key(weather::SEMANTIC_SELECTION, json!(weather::SELECTION_APPROVED));
    app.config.save_key(weather::APPROVED_RAINY, json!(["static:a.png"]));
    app.config.save_key(semantic::ENABLED, json!(false));
    let approved = WeatherStatus {
        enabled: true,
        source: WeatherSource::Semantic,
        selection: WeatherSelection::Approved,
        status: WeatherState::Active,
        category: Some("rainy".into()),
        member_keys: vec!["static:a.png".into()],
        ..Default::default()
    };
    app.on_weather(approved.clone());
    assert_eq!(app.daemon.weather.as_ref().unwrap().member_keys, ["static:a.png"]);
    app.on_weather(WeatherStatus {
        selection: WeatherSelection::Automatic,
        member_keys: vec![],
        ..approved
    });
    assert_eq!(app.daemon.weather.as_ref().unwrap().member_keys, ["static:a.png"]);
}

#[test]
fn rejects_removed_approval() {
    use crate::contracts::daemon::{WeatherSelection, WeatherSource, WeatherState, WeatherStatus};
    use crate::frontend::settings::{ActionId, SettingsMsg};
    use skwd_config::keys::weather;

    let mut app = test_app();
    seed(&mut app, &[static_wall("a.png", 1), static_wall("b.png", 2)]);
    app.config.save_key(weather::ENABLED, json!(true));
    app.config.save_key(weather::SOURCE, json!(weather::SOURCE_SEMANTIC));
    app.config.save_key(weather::SEMANTIC_SELECTION, json!(weather::SELECTION_APPROVED));
    app.config.save_key(weather::APPROVED_RAINY, json!(["static:a.png", "static:b.png"]));
    let old = WeatherStatus {
        enabled: true,
        source: WeatherSource::Semantic,
        selection: WeatherSelection::Approved,
        status: WeatherState::Active,
        category: Some("rainy".into()),
        member_keys: vec!["static:a.png".into(), "static:b.png".into()],
        ..Default::default()
    };
    app.on_weather(old.clone());
    assert_eq!(app.daemon.weather.as_ref().unwrap().member_keys.len(), 2);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ReviewWeather("rainy"))));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::WeatherReviewToggle("static:b.png".into())),
    );
    assert!(app.daemon.weather.is_none());
    app.on_weather(old);
    assert!(app.daemon.weather.is_none());
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
    app.scene.viewport = crate::frontend::ui::MIN_VIEWPORT;
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
fn workbench_keyboard_skips_control_segments() {
    use crate::frontend::settings::{Control, SettingsFocus, SettingsKey, SettingsMsg, build_tab};

    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::SetSettingsTab(String::from("picker")));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectSection(4)));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
    );
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);

    let cards = build_tab("picker", &app.config, &[], &[], "", &[]);
    let rows = &cards[4].1;
    assert!(matches!(rows[0].control, Control::Segment));
    for _ in 0..rows.len() {
        assert!(rows[app.panels.settings.focused_control].control.is_focusable());
        let _ = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Down)));
    }
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

#[test]
fn shell_follow_switches_preserve_driver_and_persist_before_retheme() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    for authority in ["skwd", "noctalia", "dms"] {
        let directory = tempfile::tempdir().unwrap();
        let config_path = directory.path().join("config.json");
        let mut config = Config::from_data(json!({"theme": {
            "authority": authority, "targets": ["caelestia"], "policy": "wallpaper"
        }}));
        config.config_path.clone_from(&config_path);
        config.persist();
        let observed = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let observed_at_call = std::sync::Arc::clone(&observed);
        let mut app = App::with_config_using(config, move |_| {
            crate::infrastructure::ipc::DaemonClient::recording_with_observer(move |method, _| {
                if method == "wall.retheme" {
                    let root: Value =
                        serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap())
                            .unwrap();
                    observed_at_call.lock().unwrap().push(root["theme"].clone());
                }
            })
        });
        for (provider, enabled) in [
            ("noctalia", true),
            ("noctalia", true),
            ("dms", true),
            ("noctalia", false),
            ("noctalia", false),
            ("dms", false),
            ("unknown", true),
        ] {
            let _ = update(
                &mut app,
                Message::Settings(SettingsMsg::Run(ActionId::SetThemeTarget(provider, enabled))),
            );
        }
        let observed = observed.lock().unwrap();
        assert_eq!(observed.len(), 4);
        for (theme, targets) in observed.iter().zip([
            json!(["caelestia", "noctalia"]),
            json!(["caelestia", "noctalia", "dms"]),
            json!(["caelestia", "dms"]),
            json!(["caelestia"]),
        ]) {
            assert_eq!(theme["targets"], targets);
            assert_eq!(theme["authority"], authority);
            assert_eq!(theme["policy"], "wallpaper");
        }
    }
}

#[test]
fn theme_variant_control_displays_the_effective_mode() {
    use crate::contracts::settings::SettingsSource;
    for (root, expected) in [
        (json!({}), "dark"),
        (json!({"matugen":{"mode":"light"}}), "light"),
        (json!({"theme":{"mode":""},"matugen":{"mode":"light"}}), "light"),
        (json!({"theme":{"mode":"dark"},"matugen":{"mode":"light"}}), "dark"),
        (json!({"theme":{"mode":"auto"},"matugen":{"mode":"light"}}), "auto"),
    ] {
        let config = Config::from_data(root);
        assert_eq!(SettingsSource::text(&config, skwd_config::keys::theme::MODE), expected);
    }
}

#[test]
fn transition_rate_input_and_auto_leave_playback_and_preview_unchanged() {
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    let key = skwd_config::keys::transition::FPS;
    let playback = app.config.num_path(skwd_config::keys::we_render::FPS);
    let preview = app.config.num_path(skwd_config::keys::transition::PREVIEW_FPS);
    for (raw, expected) in [("120", 120.0), ("30", 30.0), ("", 0.0), ("0", 0.0)] {
        let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key.into(), raw.into())));
        let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
        assert_eq!(app.config.num_path(key), expected);
        assert_eq!(app.config.num_path(skwd_config::keys::we_render::FPS), playback);
        assert_eq!(app.config.num_path(skwd_config::keys::transition::PREVIEW_FPS), preview);
    }
}

#[test]
fn disabling_semantic_search_preserves_tags_and_rejects_shortcuts_and_late_results() {
    use crate::frontend::settings::SettingsMsg;
    use crate::frontend::tagcloud::TagMsg;
    let mut app = test_app();
    app.tags.search_mode = SearchMode::Describe;
    app.tags.semantic.search = String::from("green forest");
    app.tags.semantic.pending = true;
    app.tags.semantic.ranked = vec![String::from("old-result")];
    app.library_session.filters.tags = vec![String::from("nature")];
    let generation = app.tags.semantic.generation;
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Toggle(skwd_config::keys::semantic::ENABLED.into(), false)),
    );
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    assert!(!app.tags.semantic.pending);
    assert!(app.tags.semantic.ranked.is_empty());
    assert_eq!(app.library_session.filters.tags, ["nature"]);
    assert!(app.tags.tag_search.contains("nature"));
    let _ = update(&mut app, Message::Tag(TagMsg::CycleSearchMode));
    let _ = update(&mut app, Message::Tag(TagMsg::SearchMode(SearchMode::Describe)));
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    app.prewarm_semantic_search();
    assert!(app.runtime_state.semantic.is_none());
    app.apply_semantic_result(crate::infrastructure::semantic::SemanticResult {
        generation,
        keys: vec![String::from("late-result")],
        exclusions: Vec::new(),
        query_ms: 1.0,
        search_ms: 1.0,
        error: None,
    });
    assert!(app.tags.semantic.ranked.is_empty());
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Toggle(skwd_config::keys::semantic::ENABLED.into(), true)),
    );
    let _ = update(&mut app, Message::Tag(TagMsg::CycleSearchMode));
    assert_eq!(app.tags.search_mode, SearchMode::Describe);
    assert_eq!(app.library_session.filters.tags, ["nature"]);
}

#[test]
fn disabled_semantic_search_overrides_describe_default_on_startup_and_open() {
    let mut app = App::with_config(Config::from_data(json!({
        "semantic": {"enabled": false}, "tagging": {"defaultSearchMode": "describe"}
    })));
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    let _ = update(&mut app, Message::OpenTagCloud);
    assert_eq!(app.tags.search_mode, SearchMode::Tags);
    assert!(app.runtime_state.semantic.is_none());
}

#[test]
fn deleted_model_clears_only_matching_entries_and_disables_selected_search() {
    let mut app = test_app();
    app.config.set_key(
        skwd_config::keys::semantic::MODELS,
        json!([
            {"manifest": "/models/first/semantic-pack.json", "managed": true},
            {"manifest": "/models/second/semantic-pack.json", "managed": true}
        ]),
    );
    app.config
        .set_key(skwd_config::keys::semantic::MANIFEST, json!("/models/first/semantic-pack.json"));
    let _ = update(
        &mut app,
        Message::SemanticModelDeleted("/models/first/semantic-pack.json".into(), None, Ok(())),
    );
    assert_eq!(app.config.array_len(skwd_config::keys::semantic::MODELS), 1);
    assert_eq!(
        app.config.array_values(skwd_config::keys::semantic::MODELS)[0]["manifest"],
        "/models/second/semantic-pack.json"
    );
    assert_eq!(app.config.str_path(skwd_config::keys::semantic::MANIFEST), "");
    assert!(!app.config.flag_default_true(skwd_config::keys::semantic::ENABLED));
}

#[test]
fn failed_model_deletion_keeps_registration_and_reports_the_error() {
    let mut app = test_app();
    app.config.set_key(
        skwd_config::keys::semantic::MODELS,
        json!([{"manifest": "/model/semantic-pack.json", "managed": true}]),
    );
    let _ = update(
        &mut app,
        Message::SemanticModelDeleted(
            "/model/semantic-pack.json".into(),
            None,
            Err("permission denied".into()),
        ),
    );
    assert_eq!(app.config.array_len(skwd_config::keys::semantic::MODELS), 1);
    assert!(app.panels.settings.semantic_import_status.contains("permission denied"));
    assert!(!app.panels.settings.semantic_importing);
}

#[test]
fn default_model_deletion_disables_search_without_removing_tags_or_other_models() {
    let mut app = test_app();
    app.config.set_key(
        skwd_config::keys::semantic::MODELS,
        json!([
            {"manifest": "/manual/model.json", "name": "Manual model"}
        ]),
    );
    app.library_session.filters.tags = vec!["nature".into()];
    let _ = update(&mut app, Message::SemanticModelDeleted(String::new(), None, Ok(())));
    assert!(!app.config.flag_default_true(skwd_config::keys::semantic::ENABLED));
    assert_eq!(app.config.array_len(skwd_config::keys::semantic::MODELS), 1);
    assert_eq!(app.library_session.filters.tags, ["nature"]);
    assert!(app.runtime_state.semantic.is_none());
}

#[test]
fn manual_model_deletion_keeps_another_selected_model_enabled() {
    let mut app = test_app();
    app.config.set_key(
        skwd_config::keys::semantic::MODELS,
        json!([
            {"manifest": "/manual/model.json"}, {"manifest": "/active/model.json"}
        ]),
    );
    app.config.set_key(skwd_config::keys::semantic::MANIFEST, json!("/active/model.json"));
    let _ =
        update(&mut app, Message::SemanticModelDeleted("/manual/model.json".into(), None, Ok(())));
    assert!(app.config.flag_default_true(skwd_config::keys::semantic::ENABLED));
    assert_eq!(app.config.str_path(skwd_config::keys::semantic::MANIFEST), "/active/model.json");
    assert_eq!(app.config.array_len(skwd_config::keys::semantic::MODELS), 1);
}

#[cfg(unix)]
#[test]
fn deleting_a_reference_to_the_active_model_stops_search_before_removal() {
    use crate::frontend::settings::{ActionId, SettingsMsg};
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("semantic-pack.json");
    let alias = dir.path().join("alias.json");
    std::fs::write(&manifest, "{}").unwrap();
    std::os::unix::fs::symlink(&manifest, &alias).unwrap();
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::semantic::MANIFEST, json!(manifest));
    app.config.set_key(skwd_config::keys::semantic::MODELS, json!([{"manifest":alias}]));
    let action = ActionId::DeleteSemanticModel(0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(action.clone())));
    assert!(app.config.flag_default_true(skwd_config::keys::semantic::ENABLED));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(action)));
    assert!(!app.config.flag_default_true(skwd_config::keys::semantic::ENABLED));
    assert!(manifest.is_file());
    let _ = update(
        &mut app,
        Message::SemanticModelDeleted(
            alias.to_string_lossy().into(),
            Some(manifest.to_string_lossy().into()),
            Ok(()),
        ),
    );
    assert_eq!(app.config.str_path(skwd_config::keys::semantic::MANIFEST), "");
    assert_eq!(app.config.array_len(skwd_config::keys::semantic::MODELS), 0);
}

#[test]
fn wallpaper_background_retains_color_and_monitor_inheritance() {
    use crate::frontend::settings::{SettingsMsg, background::BackgroundControl};
    let mut app = test_app();
    app.config.set_key("display.fillColor", json!("#112233"));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::BackgroundMode("DP-1".into(), "blur".into())),
    );
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::BackgroundColor("DP-1".into(), "#cc8844".into())),
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::BackgroundCommit("DP-1".into())));
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::BackgroundMode("DP-1".into(), "blur".into())),
    );
    assert_eq!(BackgroundControl::new(&app.config, "DP-1").color, "#cc8844");
    assert_eq!(BackgroundControl::new(&app.config, "DP-2").color, "#112233");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::BackgroundMode("DP-1".into(), "inherit".into())),
    );
    assert!(BackgroundControl::new(&app.config, "DP-1").inherited);
    assert_eq!(BackgroundControl::new(&app.config, "DP-1").color, "#112233");
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::BackgroundMode("DP-1".into(), "color".into())),
    );
    assert_eq!(BackgroundControl::new(&app.config, "DP-1").color, "#cc8844");
    let saved: Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    assert_eq!(saved["display"]["backgroundColors"]["DP-1"], "#cc8844");
}

#[test]
fn wallpaper_background_reveal_animates_and_tracks_inherited_placement() {
    use crate::frontend::settings::SettingsMsg;
    let mut app = test_app();
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Pick("display.fillMode".into(), "center".into())),
    );
    app.panels.settings.tick_bars(0.05);
    let reveal = app.panels.settings.bar_reveals["background:display.fillMode"].x;
    assert!(reveal > 0.0 && reveal < 1.0);
    for _ in 0..20 {
        app.panels.settings.tick_bars(0.05);
    }
    assert_eq!(app.panels.settings.bar_reveals["background:display.fillMode"].x, 1.0);
    let motion = app.motion_profile();
    app.panels.settings.open_bar("background:display.fillModes.DP-1".into(), motion);
    for _ in 0..20 {
        app.panels.settings.tick_bars(0.05);
    }
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Pick("display.fillMode".into(), "fill".into())),
    );
    assert_eq!(app.panels.settings.bar_reveals["background:display.fillModes.DP-1"].target, 0.0);
    app.panels.settings.tick_bars(0.05);
    let reveal = app.panels.settings.bar_reveals["background:display.fillMode"].x;
    assert!(reveal > 0.0 && reveal < 1.0);
    for _ in 0..20 {
        app.panels.settings.tick_bars(0.05);
    }
    assert!(!app.panels.settings.bars_animating());
    assert!(!app.panels.settings.bar_reveals.contains_key("background:display.fillMode"));
}

#[test]
fn wallpaper_background_survives_output_refresh_with_inherited_placement() {
    use crate::contracts::daemon::{OutputStatus, OutputsResult};
    let mut app = test_app();
    app.config.set_key("display.fillMode", json!("center"));
    app.config.set_key("display.fillColor", json!("#234567"));
    app.panels.effects = Some(crate::frontend::effects::Effects::new(
        Vec::new(),
        String::from("/still.png"),
        None,
        0,
        WallpaperKind::Static,
        false,
        100,
        String::from("static:/still.png"),
    ));
    let outputs = OutputsResult {
        outputs: vec![OutputStatus {
            name: String::from("DP-1"),
            path: String::from("/still.png"),
            ..OutputStatus::default()
        }],
    };
    app.on_outputs(outputs.clone());
    let monitor = &app.panels.effects.as_ref().unwrap().monitors()[0];
    assert!(monitor.background.inherited);
    assert_eq!(monitor.background.color, "#234567");
    assert_eq!(monitor.background_reveal, 1.0);
    let motion = app.motion_profile();
    app.panels.settings.open_bar("background-picker:DP-1".into(), motion);
    for _ in 0..20 {
        app.panels.settings.tick_bars(0.05);
    }
    app.on_outputs(outputs);
    let monitor = &app.panels.effects.as_ref().unwrap().monitors()[0];
    assert_eq!(monitor.background_reveal, 1.0);
    assert_eq!(monitor.background_picker, 1.0);
    assert_eq!(monitor.background.color, "#234567");
}

#[test]
fn exclusion_scopes_resolve() {
    use crate::contracts::settings::{ExclusionScope, SettingsSource};
    let mut app = test_app();
    let home = skwd_config::home();
    app.config.set_key("library.folders", json!(["~/walls"]));
    app.config.set_key(
        "library.excludedFolders",
        json!([format!("{home}/walls/x"), "rel", "/elsewhere", " ", "~/walls/y/", "~/walls"]),
    );
    let inside = ExclusionScope::Inside(format!("{home}/walls"));
    assert_eq!(
        app.config.exclusion_scopes(),
        [
            inside.clone(),
            ExclusionScope::Everywhere,
            ExclusionScope::Outside,
            ExclusionScope::Empty,
            inside.clone(),
            inside
        ]
    );
}

#[test]
fn app_profile_persists() {
    use crate::frontend::settings::SettingsMsg;
    for (path, enabled, method) in [
        ("theme.appProfiles.kitty", true, Some("theme.app.set")),
        ("theme.appProfiles.kitty", false, None),
        ("integrations.0.profile", true, Some("theme.apps.refresh")),
        ("integrations.0.profile", false, None),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let mut app = test_app();
        app.config.config_path = directory.path().join("config.json");
        app.config.set_key("integrations", json!([{"name": "custom", "enabled": enabled}]));
        app.daemon.app_themes = Some(crate::contracts::daemon::AppThemesResult {
            apps: vec![crate::contracts::daemon::AppThemeStatus {
                id: "kitty".into(),
                name: "Kitty".into(),
                installed: true,
                config_found: true,
                config_path: String::new(),
                output_path: String::new(),
                enabled,
                state: "applied".into(),
                detail: String::new(),
                can_enable: !enabled,
                can_disable: enabled,
                can_adopt: false,
                template_path: String::new(),
                customized: false,
                can_disconnect: enabled,
                can_reconnect: false,
            }],
        });
        drain_calls(&app);
        let _ = update(&mut app, Message::Settings(SettingsMsg::Pick(path.into(), "nord".into())));
        let saved = Config::from_data(
            serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap(),
        );
        assert_eq!(saved.str_path(path), "nord");
        let calls = drain_calls(&app);
        assert_eq!(
            calls
                .iter()
                .filter(|(name, _)| name == "theme.app.set"
                    || name == "wall.retheme"
                    || name == "theme.apps.refresh")
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            method.into_iter().collect::<Vec<_>>()
        );
        if let Some((_, params)) = calls.iter().find(|(name, _)| name == "theme.app.set") {
            assert_eq!(params, &json!({"id": "kitty", "enabled": true, "adopt": false}));
        }
        if let Some((_, params)) = calls.iter().find(|(name, _)| name == "theme.apps.refresh") {
            assert_eq!(params, &json!({"integration": 0}));
        }
    }
}

#[test]
fn profile_choices_show_preview() {
    use crate::contracts::settings::SettingsSource;
    use crate::domain::theme::Candidate;

    let saved = Candidate::from_preset("nord").unwrap();
    for (mode, dark) in [("dark", true), ("light", false)] {
        let mut config = Config::from_data(json!({}));
        config.set_key(skwd_config::keys::theme::MODE, json!(mode));
        config.set_key(
            skwd_config::keys::theme::SAVED_THEMES,
            json!(crate::infrastructure::theme::upsert_saved(&[], "Mine", &saved)),
        );
        let palettes = config.palette_colors();
        let preset = &palettes.iter().find(|(name, _)| name == "nord").unwrap().1;
        assert_eq!(preset, &saved.preview_colors());
        let mut expected = saved.clone();
        expected.set_dark(dark);
        let saved_colors = &palettes.iter().find(|(name, _)| name == "Mine").unwrap().1;
        assert_eq!(saved_colors, &expected.preview_colors());
        assert_eq!(saved_colors.len(), 6);
    }
}

#[test]
fn legacy_profile_keeps_swatches() {
    use crate::contracts::settings::SettingsSource;

    let legacy = json!({
        "name": "Legacy",
        "background": "#161616",
        "surface": "#252525",
        "surfaceText": "#eeeeee",
        "primary": "#ff9933"
    });
    let mut config = Config::from_data(json!({}));
    config.set_key(skwd_config::keys::theme::MODE, json!("light"));
    config.set_key(skwd_config::keys::theme::SAVED_THEMES, json!([legacy.clone()]));
    let colors = config.palette_colors().into_iter().find(|(name, _)| name == "Legacy").unwrap().1;
    assert_eq!(
        colors,
        crate::infrastructure::theme::saved_palettes(&[legacy], None)[0].1.preview_colors()
    );
    assert_eq!(colors[0], "#161616");
    assert_eq!(colors[1], "#252525");
    assert!(colors.contains(&"#ff9933".to_string()));
}

#[test]
fn busy_profile_unchanged() {
    use crate::frontend::settings::SettingsMsg;
    for enabled in [true, false] {
        let mut app = test_app();
        let path = "theme.appProfiles.kitty";
        app.config.set_key(path, json!("dracula"));
        app.daemon.app_themes = Some(crate::contracts::daemon::AppThemesResult {
            apps: vec![crate::contracts::daemon::AppThemeStatus {
                id: "kitty".into(),
                name: "Kitty".into(),
                installed: true,
                config_found: true,
                config_path: String::new(),
                output_path: String::new(),
                enabled,
                state: "busy".into(),
                detail: String::new(),
                can_enable: false,
                can_disable: false,
                can_adopt: false,
                template_path: String::new(),
                customized: false,
                can_disconnect: false,
                can_reconnect: false,
            }],
        });
        drain_calls(&app);
        let _ = update(&mut app, Message::Settings(SettingsMsg::Pick(path.into(), "nord".into())));
        assert_eq!(app.config.str_path(path), "dracula");
        assert!(drain_calls(&app).is_empty());
    }
}
