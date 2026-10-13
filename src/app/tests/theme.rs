use super::*;

fn steam_online(app: &mut App) {
    app.config.set_key(skwd_config::keys::steam::BACKEND, json!("steam"));
    app.daemon.steam_client_available = Some(true);
}

#[test]
fn theme_preview_debounce() {
    let mut app = test_app();
    let dir = std::env::temp_dir().join(format!(
        "skwd-app-thumbs-{}-{}",
        std::process::id(),
        TEST_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let red = dir.join("red.png");
    image::RgbaImage::from_pixel(8, 8, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let walls = [
        json!({
            "name": "red.png",
            "type": "static",
            "thumb": red.to_string_lossy(),
            "hue": 1,
            "mtime": 0
        }),
        wall("b.png", "static", 2, 0),
    ];
    seed(&mut app, &walls);
    let t0 = Instant::now();
    assert!(app.update_theme_preview(t0, 0.016));
    assert!(app.theme.preview_target.is_none());
    let _ = app.update_theme_preview(t0 + Duration::from_millis(20), 0.016);
    assert!(app.theme.preview_target.is_none());
    let _ = app.update_theme_preview(t0 + Duration::from_millis(60), 0.016);
    assert_eq!(app.theme.preview_target, Some(0));
    assert_eq!(app.theme.job_pending, Some(0));
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "theme.preview"));

    let now = t0 + Duration::from_millis(60);
    deliver_preview(&mut app, 0);
    assert!(app.theme.job_pending.is_none());
    assert!(app.theme.cache.contains_key(&0));
    assert_eq!(app.theme.swatch_target, Some(0));
    app.source_browser.browser = Some(Browser::new(Source::Wallhaven));
    let _ = app.update_theme_preview(now + Duration::from_millis(16), 0.016);
    assert!(app.theme.preview_target.is_none());
}

#[test]
fn stale_palette_ignored() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.theme.preview_target = Some(1);
    app.theme.job_pending = Some(1);
    app.theme.swatch_target = None;
    app.theme.fade_t.snap(1.0);

    deliver_preview(&mut app, 0);

    assert!(app.theme.cache.contains_key(&0));
    assert_eq!(app.theme.job_pending, Some(1));
    assert!(app.theme.fade_t.settled());
    assert!(app.theme.swatch_target.is_none());
}

#[test]
fn hovered_reply_drives_fade() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    app.theme.preview_target = Some(0);
    app.theme.job_pending = Some(0);
    app.theme.fade_t.snap(1.0);

    deliver_preview(&mut app, 0);

    assert_eq!(app.theme.job_pending, None);
    assert!(!app.theme.fade_t.settled());
    assert_eq!(app.theme.fade_to.primary, crate::frontend::theme::parse_hex("#f06e44").unwrap());
}

#[test]
fn theme_sub_bar_flow() {
    let mut app = test_app();
    app.chrome.filter_bar_visible = false;
    let _ = update(&mut app, Message::ToggleThemePanel);
    assert!(app.theme.bar_open);
    assert!(app.chrome.filter_bar_visible);
    assert!(!app.menu_capturing());
    let _ =
        update(&mut app, Message::Theme(crate::frontend::theme_designer::ThemeMsg::BackendMenu));
    assert_eq!(app.chrome.bar.menu, Some(crate::frontend::ui::MenuKind::Backends));
    assert!(app.menu_capturing());
    let _ = update(&mut app, Message::FolderMenuToggle);
    assert_eq!(app.chrome.bar.menu, Some(crate::frontend::ui::MenuKind::Folders));
    app.chrome.bar.menu = None;
    let _ = update(
        &mut app,
        Message::Theme(crate::frontend::theme_designer::ThemeMsg::Option(
            crate::contracts::picker::theme_setting::BACKEND,
            "matugen",
        )),
    );
    assert_eq!(app.config.str_path("theme.backend"), "skwd-iris");
    assert_eq!(app.config.str_path("theme.policy"), "wallpaper");
    assert_eq!(app.config.str_path("theme.authority"), "skwd");
    assert_eq!(app.config.str_path("theme.engine"), "matugen");
    assert_eq!(app.config.theme_backend(), "matugen");
    let _ = update(
        &mut app,
        Message::Theme(crate::frontend::theme_designer::ThemeMsg::Option(
            "matugen.schemeType",
            "scheme-rainbow",
        )),
    );
    assert_eq!(app.config.str_path("matugen.schemeType"), "scheme-rainbow");
    let _ = update(
        &mut app,
        Message::Theme(crate::frontend::theme_designer::ThemeMsg::Option(
            "matugen.colorIndex",
            "2",
        )),
    );
    assert_eq!(app.config.num_path("matugen.colorIndex"), 2.0);
    let calls = drain_calls(&app);
    assert_eq!(calls.iter().filter(|(method, _)| method == "wall.retheme").count(), 3);
    let _ =
        update(&mut app, Message::Theme(crate::frontend::theme_designer::ThemeMsg::BackendMenu));
    let _ = update(&mut app, Message::Exit);
    assert_eq!(app.chrome.bar.menu, None);
    assert!(app.theme.bar_open);
    let _ = update(&mut app, Message::Exit);
    assert!(!app.theme.bar_open);
}

#[test]
fn theme_option_ends_shell_preview_before_retheme() {
    let mut app = test_app();
    app.theme.shell_preview_sent = Some(0);
    let _ = update(
        &mut app,
        Message::Theme(crate::frontend::theme_designer::ThemeMsg::Option(
            crate::contracts::picker::theme_setting::MODE,
            "light",
        )),
    );

    let calls = drain_calls(&app);
    let preview_end = calls
        .iter()
        .position(|(method, _)| method == "wall.shell_preview_end")
        .expect("shell preview ends when an option is committed");
    let retheme = calls
        .iter()
        .position(|(method, _)| method == "wall.retheme")
        .expect("option commit rethemes");
    assert!(preview_end < retheme);
    assert_eq!(app.config.str_path(skwd_config::keys::theme::MODE), "light");
}

#[test]
fn backend_menu_writes_key() {
    assert_eq!(crate::contracts::picker::theme_setting::BACKEND, skwd_config::keys::theme::BACKEND);
    let mut app = test_app();
    let _ =
        update(&mut app, Message::Theme(crate::frontend::theme_designer::ThemeMsg::BackendMenu));
    assert_eq!(app.chrome.bar.menu, Some(crate::frontend::ui::MenuKind::Backends));
    let _ = update(
        &mut app,
        Message::Theme(crate::frontend::theme_designer::ThemeMsg::Option(
            crate::contracts::picker::theme_setting::BACKEND,
            "wallust",
        )),
    );
    assert_eq!(app.chrome.bar.menu, None);
    assert_eq!(app.config.theme_backend(), "wallust");
    assert!(app.config.str_path("ui.themeSelection").is_empty());
}

#[test]
fn theme_audition_changes_selected_profile() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Command(String::from(
            "open theme-audition",
        ))),
    );
    assert!(app.theme.audition_open);
    assert!(!app.theme.bar_open);
    assert!(app.picker_obscured());
    let request = app
        .daemon
        .pending
        .iter()
        .find(|(_, pending)| matches!(pending, Pending::ThemePreviews { .. }))
        .map(|(id, _)| *id)
        .expect("theme preview request");
    respond(
        &mut app,
        request,
        json!({
            "backend": "skwd-iris",
            "backends": ["skwd-iris", "wallust"],
            "previews": [{
                "backend": "skwd-iris",
                "key": "theme.scheme",
                "value": "content",
                "label": "Content",
                "palette": {
                    "primary": "#7f67be",
                    "primaryText": "#ffffff",
                    "tertiary": "#9b4d75",
                    "surface": "#17151d",
                    "surfaceText": "#e9e0eb",
                    "surfaceVariant": "#49454f",
                    "surfaceContainer": "#211f26",
                    "background": "#121016",
                    "outline": "#938f99"
                }
            }]
        }),
    );
    assert_eq!(app.theme.audition_previews.len(), 1);
    assert_eq!(app.theme.audition_backends, ["skwd-iris", "wallust"]);
    drain_calls(&app);
    let _ = update(
        &mut app,
        Message::ThemeAuditionSelect {
            backend: String::from("skwd-iris"),
            key: String::from("theme.scheme"),
            value: String::from("content"),
        },
    );
    assert_eq!(app.config.str_path(skwd_config::keys::theme::SCHEME), "content");
    let calls = drain_calls(&app);
    assert_eq!(calls.iter().filter(|(method, _)| method == "wall.retheme").count(), 1);
    assert!(calls.iter().all(|(method, _)| method != "wall.apply"));
}

#[test]
fn audition_backend_nav_applies_nothing() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.theme.audition_open = true;
    app.theme.audition_backend = String::from("skwd-iris");
    app.theme.audition_backends = vec![String::from("skwd-iris"), String::from("wallust")];
    let before = app.config.root().clone();
    drain_calls(&app);

    let _ = update(&mut app, Message::ThemeAuditionBackend(String::from("wallust")));

    assert_eq!(app.theme.audition_backend, "wallust");
    assert_eq!(app.config.root(), &before);
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| {
        method == "theme.previews" && params["backend"] == json!("wallust")
    }));
    assert!(calls.iter().all(|(method, _)| method != "wall.retheme" && method != "wall.apply"));
}

#[test]
fn blank_designer_saves_no_effects() {
    use crate::domain::theme::Candidate;
    use crate::frontend::theme_designer::{ThemeDesigner, ThemeMsg};

    let mut app = test_app();
    let config_before = app.config.root().clone();
    let config_path = app.config.config_path.clone();
    assert!(!config_path.exists());

    app.panels.theme_designer =
        Some(ThemeDesigner::new(Candidate::from_preset("dracula").unwrap(), String::from("   ")));
    let palette_before = app.theme.palette.primary;
    let base_before = app.theme.base_palette.primary;
    let fade_from_before = app.theme.fade_from.primary;
    let fade_to_before = app.theme.fade_to.primary;
    let fade_before = (app.theme.fade_t.x, app.theme.fade_t.target);
    drain_calls(&app);

    for message in [ThemeMsg::SaveTheme, ThemeMsg::SaveApply] {
        let _ = update(&mut app, Message::Theme(message));

        assert_eq!(app.config.root(), &config_before);
        assert!(!config_path.exists());
        assert!(drain_calls(&app).is_empty());
        assert_eq!(app.theme.palette.primary, palette_before);
        assert_eq!(app.theme.base_palette.primary, base_before);
        assert_eq!(app.theme.fade_from.primary, fade_from_before);
        assert_eq!(app.theme.fade_to.primary, fade_to_before);
        assert_eq!((app.theme.fade_t.x, app.theme.fade_t.target), fade_before);
    }
}

#[test]
fn preset_after_saved_never_overwrites() {
    use crate::domain::theme::Candidate;
    use crate::frontend::theme_designer::{ThemeDesigner, ThemeMsg};

    let mut app = test_app();
    let saved_candidate = Candidate::from_preset("nord").unwrap();
    let saved = crate::infrastructure::theme::upsert_saved(&[], "Mine", &saved_candidate);
    app.config.set_key(skwd_config::keys::theme::SAVED_THEMES, json!(saved));
    app.panels.theme_designer =
        Some(ThemeDesigner::new(Candidate::from_preset("dracula").unwrap(), String::new()));

    let _ = update(&mut app, Message::Theme(ThemeMsg::LoadSaved(String::from("Mine"))));
    let _ = update(&mut app, Message::Theme(ThemeMsg::Preset(String::from("dracula"))));

    let designer = app.panels.theme_designer.as_ref().unwrap();
    assert_eq!(designer.name_buf, "");
    assert_eq!(designer.selected_preset(), Some("dracula"));

    let _ = update(&mut app, Message::Theme(ThemeMsg::SaveTheme));
    let saved = app.config.array_values(skwd_config::keys::theme::SAVED_THEMES);
    assert_eq!(crate::infrastructure::theme::find_saved(&saved, "Mine"), Some(saved_candidate));
}

#[test]
fn delete_saved_confirms_rethemes_nord() {
    use crate::domain::theme::Candidate;
    use crate::frontend::theme::Palette;
    use crate::frontend::theme_designer::{ThemeDesigner, ThemeMsg};

    let mut app = test_app();
    let saved_candidate = Candidate::from_preset("dracula").unwrap();
    let saved = crate::infrastructure::theme::upsert_saved(&[], "Mine", &saved_candidate);
    app.config.set_key(skwd_config::keys::theme::SAVED_THEMES, json!(saved));
    app.config.set_key(skwd_config::keys::theme::BACKEND, json!("static"));
    app.config.set_key(skwd_config::keys::theme::STATIC_THEME, json!("Mine"));
    app.panels.theme_designer = Some(ThemeDesigner::new(saved_candidate, String::from("Mine")));
    let config_path = app.config.config_path.clone();
    assert!(!config_path.exists());
    drain_calls(&app);

    let delete = || Message::Theme(ThemeMsg::DeleteSaved(String::from("Mine")));
    let _ = update(&mut app, delete());

    assert_eq!(app.panels.theme_designer.as_ref().unwrap().armed_delete(), Some("Mine"));
    assert!(!config_path.exists());
    assert!(drain_calls(&app).is_empty());
    assert!(
        crate::infrastructure::theme::find_saved(
            &app.config.array_values(skwd_config::keys::theme::SAVED_THEMES),
            "Mine"
        )
        .is_some()
    );

    let _ = update(&mut app, delete());

    assert!(config_path.exists());
    assert_eq!(app.config.str_path(skwd_config::keys::theme::STATIC_THEME), "nord");
    assert!(
        crate::infrastructure::theme::find_saved(
            &app.config.array_values(skwd_config::keys::theme::SAVED_THEMES),
            "Mine"
        )
        .is_none()
    );
    let calls = drain_calls(&app);
    assert_eq!(calls.iter().filter(|(method, _)| method == "wall.retheme").count(), 1);

    let expected = Palette::from_candidate(&Candidate::from_preset("nord").unwrap());
    let colors = |palette: &Palette| {
        [
            palette.primary,
            palette.primary_text,
            palette.tertiary,
            palette.surface,
            palette.surface_text,
            palette.surface_variant,
            palette.surface_container,
            palette.background,
            palette.outline,
        ]
    };
    assert_eq!(colors(&app.theme.base_palette), colors(&expected));
    assert_eq!(colors(&app.theme.fade_to), colors(&expected));
    let designer = app.panels.theme_designer.as_ref().unwrap();
    assert_eq!(designer.name_buf, "");
    assert_eq!(designer.selected_preset(), Some("nord"));
    assert_eq!(designer.armed_delete(), None);
}

#[test]
fn theme_bar_keeps_live_preview() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);
    let _ = update(&mut app, Message::ToggleThemePanel);
    assert!(app.theme.bar_open);
    let t0 = Instant::now();
    for i in 0..=40u64 {
        frame(&mut app, t0 + Duration::from_millis(i * 16), 800.0, 600.0);
    }
    assert!(app.theme.preview_target.is_some());
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview"));
    app.theme.swatch.clear();
    app.on_event("skwd.wall.theme_done", &json!({"source": "x"}));
    assert_eq!(app.theme.swatch.len(), 6);
}

#[test]
fn theme_backends_reply() {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleThemePanel);
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == "theme.backends"));
    assert!(app.theme.backends.is_none());
    let id = *app
        .daemon
        .pending
        .iter()
        .find(|(_, pending)| matches!(pending, Pending::ThemeBackends))
        .expect("pending theme.backends request")
        .0;
    respond(&mut app, id, json!({ "backends": ["off", "native", "matugen"] }));
    assert_eq!(
        app.theme.backends.as_deref(),
        Some(&[String::from("off"), String::from("native"), String::from("matugen")][..])
    );
}

#[test]
fn delete_routing() {
    let mut app = test_app();
    let mut we = wall("Nice Scene", "we", 3, 0);
    we["we_id"] = json!("4242");
    we["steam_subscribed"] = json!(true);
    seed(&mut app, &[wall("a.png", "static", 1, 0), we, wall("c.png", "static", 2, 0)]);
    steam_online(&mut app);
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);

    let wi = app
        .library_session
        .filtered
        .iter()
        .position(|&idx| app.library_session.library.catalog().items[idx as usize].we_id == "4242")
        .expect("we item filtered in");
    app.scene.set_current(wi, 3);
    app.scene.toggle_flip(wi);
    let t0 = Instant::now();
    for i in 0..=30u64 {
        frame(&mut app, t0 + Duration::from_millis(i * 16), 800.0, 600.0);
    }
    let panel = app.scene.render.back.clone().expect("back panel present");
    assert_eq!(panel.removal.label(), "Unsubscribe");
    let layout = crate::frontend::ui::back_layout(&panel);
    let (dx, dy) = (
        layout.actions.delete.0 + layout.actions.delete.2 / 2.0,
        layout.actions.delete.1 + layout.actions.delete.3 / 2.0,
    );
    let _ = update(&mut app, Message::Click(dx, dy, crate::domain::input::MouseButton::Left));
    let calls = drain_calls(&app);
    let rm = calls.iter().find(|(method, _)| method == "wall.remove").expect("remove sent");
    assert_eq!(rm.1["we_id"], "4242");
    assert!(rm.1.get("path").is_none());
    assert_eq!(rm.1["unsubscribe"], true);
    frame(&mut app, t0 + Duration::from_secs(1), 800.0, 600.0);
    assert_eq!(app.scene.render.back.as_ref().unwrap().removal.label(), "Unsubscribing…");

    assert!(!app.scene.filter_flip_running());
    app.on_event(
        "skwd.wall.removed",
        &json!({ "key": app.library_session.library.catalog().items[1].key.clone() }),
    );
    assert_eq!(app.library_session.library.catalog().items.len(), 2);
    assert!(app.scene.filter_flip_running());

    let si = app
        .library_session
        .filtered
        .iter()
        .position(|&idx| app.library_session.library.catalog().items[idx as usize].we_id.is_empty())
        .expect("a static item survives");
    app.scene.set_current(si, 2);
    app.scene.toggle_flip(si);
    for i in 31..=60u64 {
        frame(&mut app, t0 + Duration::from_millis(i * 16), 800.0, 600.0);
    }
    let panel = app.scene.render.back.clone().expect("back panel re-emitted");
    let layout = crate::frontend::ui::back_layout(&panel);
    let (dx, dy) = (
        layout.actions.delete.0 + layout.actions.delete.2 / 2.0,
        layout.actions.delete.1 + layout.actions.delete.3 / 2.0,
    );
    let _ = update(&mut app, Message::Click(dx, dy, crate::domain::input::MouseButton::Left));
    let calls = drain_calls(&app);
    let rm = calls.iter().find(|(method, _)| method == "wall.remove").expect("remove sent");
    let deleted =
        &app.library_session.library.catalog().items[app.library_session.filtered[si] as usize];
    assert_eq!(rm.1["path"], deleted.path);
}

#[test]
fn workshop_video_delete() {
    let mut app = test_app();
    let mut wv = wall("burbank", "video", 4, 0);
    wv["key"] = json!("we:1127552307");
    wv["steam_subscribed"] = json!(true);
    wv["video_file"] =
        json!("/steam/steamapps/workshop/content/431960/1127552307/burbank - sorry.mp4");
    seed(&mut app, &[wv, wall("plain.mp4", "video", 5, 0)]);
    steam_online(&mut app);

    let wi = app
        .library_session
        .filtered
        .iter()
        .position(|&idx| {
            app.library_session.library.catalog().items[idx as usize].key == "we:1127552307"
        })
        .expect("workshop video filtered in");
    assert!(
        app.library_session.library.catalog().items[app.library_session.filtered[wi] as usize]
            .we_id
            .is_empty()
    );
    let wsi = app.library_session.filtered[wi];
    super::update::delete_wallpaper(&mut app, wsi);
    let calls = drain_calls(&app);
    let rm = calls.iter().find(|(method, _)| method == "wall.remove").expect("remove sent");
    assert_eq!(rm.1["we_id"], "1127552307");
    assert!(rm.1.get("path").is_none());
    assert_eq!(rm.1["unsubscribe"], true);
    assert!(!calls.iter().any(|(method, _)| method == "wall.list"));

    let pi = app
        .library_session
        .filtered
        .iter()
        .position(|&idx| {
            app.library_session.library.catalog().items[idx as usize].key != "we:1127552307"
        })
        .expect("plain video present");
    let psi = app.library_session.filtered[pi];
    super::update::delete_wallpaper(&mut app, psi);
    let calls = drain_calls(&app);
    let rm = calls.iter().find(|(method, _)| method == "wall.remove").expect("remove sent");
    assert!(rm.1.get("we_id").is_none());
}

#[test]
fn removal_confirms_late() {
    let mut app = test_app();
    seed(&mut app, &[wall("first.png", "static", 1, 0)]);
    drain_calls(&app);
    super::update::delete_wallpaper(&mut app, 0);
    assert_eq!(drain_calls(&app).len(), 1);
    super::update::delete_wallpaper(&mut app, 0);
    assert!(drain_calls(&app).is_empty());
    let id = *app
        .daemon
        .pending
        .iter()
        .find(|(_, request)| matches!(request, Pending::RemoveWallpaper { .. }))
        .unwrap()
        .0;
    app.handle_ipc(crate::infrastructure::ipc::IpcMsg::Response {
        id,
        result: Some(json!({"removed": "/wallpapers/first.png", "unsubscribed": false})),
        error: None,
    });
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.list"));
    assert_eq!(app.library_session.library.catalog().items.len(), 1);
}

#[test]
fn removal_failure_preserves() {
    let mut app = test_app();
    let mut scene = wall("scene", "we", 1, 0);
    scene["we_id"] = json!("42");
    scene["steam_subscribed"] = json!(true);
    seed(&mut app, &[scene]);
    steam_online(&mut app);
    app.scene.set_flipped_for_test(Some(0));
    drain_calls(&app);
    super::update::delete_wallpaper(&mut app, 0);
    drain_calls(&app);
    let id = *app
        .daemon
        .pending
        .iter()
        .find(|(_, request)| matches!(request, Pending::RemoveWallpaper { .. }))
        .unwrap()
        .0;
    app.handle_ipc(crate::infrastructure::ipc::IpcMsg::Response {
        id,
        result: None,
        error: Some(wall_proto::ErrorInfo { code: 1, message: "Steam is offline".into() }),
    });
    assert_eq!(app.library_session.library.catalog().items.len(), 1);
    assert_eq!(app.scene.flipped(), Some(0));
    assert!(drain_calls(&app).is_empty());
    assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("Steam is offline"));
    super::update::delete_wallpaper(&mut app, 0);
    assert_eq!(drain_calls(&app).len(), 1);
}

#[test]
fn steamcmd_removal_local() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::steam::BACKEND, json!("steamcmd"));
    let mut scene = wall("scene", "we", 1, 0);
    scene["we_id"] = json!("42");
    seed(&mut app, &[scene]);
    drain_calls(&app);
    super::update::delete_wallpaper(&mut app, 0);
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].1, json!({"we_id": "42", "unsubscribe": false}));
}

#[test]
fn unknown_steam_removal_disabled() {
    use crate::frontend::scene::RemovalAction;

    let mut app = test_app();
    let mut scene = wall("scene", "we", 1, 0);
    scene["we_id"] = json!("42");
    scene["steam_confirmation_required"] = json!(true);
    seed(&mut app, &[scene]);
    app.config.set_key(skwd_config::keys::steam::BACKEND, json!("steamcmd"));
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);
    app.scene.toggle_flip(0);
    let now = Instant::now();
    for frame_no in 0..=30 {
        frame(&mut app, now + Duration::from_millis(frame_no * 16), 800.0, 600.0);
    }
    let panel = app.scene.render.back.as_ref().unwrap();
    assert_eq!(panel.removal, RemovalAction::SteamStatusUnknown);
    assert_eq!(panel.removal.label(), "Steam status unknown");
    assert!(!panel.removal.enabled());
    let layout = crate::frontend::ui::back_layout(panel);
    let x = layout.actions.delete.0 + layout.actions.delete.2 / 2.0;
    let y = layout.actions.delete.1 + layout.actions.delete.3 / 2.0;
    drain_calls(&app);
    let _ = update(&mut app, Message::Click(x, y, crate::domain::input::MouseButton::Left));
    super::update::delete_wallpaper(&mut app, 0);
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.remove"));
}

#[test]
fn unsubscribe_connection_gate() {
    use crate::frontend::scene::RemovalAction;

    for (backend, connection, daemon, enabled) in [
        ("steam", Some(true), true, true),
        ("steam", Some(false), true, false),
        ("steam", None, true, false),
        ("steam", Some(true), false, false),
        ("steamcmd", Some(true), true, false),
        ("", Some(true), true, true),
        ("other", Some(true), true, false),
    ] {
        let mut app = test_app();
        let mut scene = wall("scene", "we", 1, 0);
        scene["we_id"] = json!("42");
        scene["steam_subscribed"] = json!(true);
        seed(&mut app, &[scene]);
        app.config.set_key(skwd_config::keys::steam::BACKEND, json!(backend));
        app.daemon.steam_client_available = connection;
        app.daemon.connected = daemon;
        app.scene.viewport = (800.0, 600.0);
        app.scene.set_visible(true);
        app.scene.toggle_flip(0);
        let now = Instant::now();
        for frame_no in 0..=30 {
            frame(&mut app, now + Duration::from_millis(frame_no * 16), 800.0, 600.0);
        }
        let panel = app.scene.render.back.as_ref().unwrap();
        assert_eq!(panel.removal.label(), "Unsubscribe", "{backend} {connection:?}");
        assert_eq!(panel.removal.enabled(), enabled, "{backend} {connection:?}");
        assert_eq!(panel.removal == RemovalAction::Unsubscribe, enabled);
        let state: Value = serde_json::from_str(&super::update::ui_state_json(&app)).unwrap();
        assert_eq!(state["card_back"]["removal"]["enabled"], enabled);
        drain_calls(&app);
        super::update::delete_wallpaper(&mut app, 0);
        let calls = drain_calls(&app);
        assert_eq!(calls.iter().any(|(method, _)| method == "wall.remove"), enabled);
    }
}

#[test]
fn unsubscribe_status_refresh() {
    let mut app = test_app();
    seed(&mut app, &[wall("scene", "we", 1, 0)]);
    steam_online(&mut app);
    app.refresh_unsubscribe_status();
    assert_eq!(app.daemon.steam_client_available, None);
    assert_eq!(drain_calls(&app).iter().filter(|(method, _)| method == "status").count(), 1);
    app.refresh_unsubscribe_status();
    assert!(drain_calls(&app).is_empty());
    let id = status_request_id(&app);
    reject(&mut app, id, "offline");
    assert_eq!(app.daemon.steam_client_available, None);
    assert_eq!(drain_calls(&app).iter().filter(|(method, _)| method == "status").count(), 1);
    let id = status_request_id(&app);
    respond(&mut app, id, json!({wall_proto::STEAM_CLIENT_AVAILABLE_FIELD: true}));
    assert!(app.unsubscribe_available());
    app.handle_ipc(IpcMsg::Disconnected);
    assert!(!app.unsubscribe_available());
    assert_eq!(app.daemon.steam_client_available, None);
}

#[test]
fn unsubscribe_ignores_old_status() {
    let mut app = test_app();
    seed(&mut app, &[wall("scene", "we", 1, 0)]);
    steam_online(&mut app);
    let old = app.call_tracked(wall_proto::rpc::STATUS, json!({}), Pending::Status);
    drain_calls(&app);
    app.refresh_unsubscribe_status();
    assert!(app.daemon.steam_refresh_needed);
    assert!(!app.unsubscribe_available());
    assert!(drain_calls(&app).is_empty());
    respond(&mut app, old, json!({wall_proto::STEAM_CLIENT_AVAILABLE_FIELD: true}));
    assert!(!app.unsubscribe_available());
    assert!(!app.daemon.steam_refresh_needed);
    assert_eq!(drain_calls(&app).iter().filter(|(method, _)| method == "status").count(), 1);
    let fresh = status_request_id(&app);
    respond(&mut app, fresh, json!({wall_proto::STEAM_CLIENT_AVAILABLE_FIELD: false}));
    assert!(!app.unsubscribe_available());
    assert_eq!(app.daemon.steam_client_available, Some(false));
}

#[test]
fn unsubscribe_backend_switch() {
    let mut app = test_app();
    seed(&mut app, &[wall("scene", "we", 1, 0)]);
    steam_online(&mut app);
    let pick = |app: &mut App, backend: &str| {
        let _ = update(
            app,
            Message::Settings(crate::frontend::settings::SettingsMsg::Pick(
                skwd_config::keys::steam::BACKEND.into(),
                backend.into(),
            )),
        );
    };
    pick(&mut app, "steamcmd");
    assert!(!app.unsubscribe_available());
    assert_eq!(app.daemon.steam_client_available, None);
    drain_calls(&app);
    pick(&mut app, "steam");
    assert!(!app.unsubscribe_available());
    assert_eq!(drain_calls(&app).iter().filter(|(method, _)| method == "status").count(), 1);
}

#[test]
fn unavailable_unsubscribe_click() {
    let mut app = test_app();
    let mut scene = wall("scene", "we", 1, 0);
    scene["we_id"] = json!("42");
    scene["steam_subscribed"] = json!(true);
    seed(&mut app, &[scene]);
    app.config.set_key(skwd_config::keys::steam::BACKEND, json!("steamcmd"));
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);
    app.scene.toggle_flip(0);
    let now = Instant::now();
    for frame_no in 0..=30 {
        frame(&mut app, now + Duration::from_millis(frame_no * 16), 800.0, 600.0);
    }
    let panel = app.scene.render.back.as_ref().unwrap();
    assert!(!panel.removal.enabled());
    let layout = crate::frontend::ui::back_layout(panel);
    let x = layout.actions.delete.0 + layout.actions.delete.2 / 2.0;
    let y = layout.actions.delete.1 + layout.actions.delete.3 / 2.0;
    drain_calls(&app);
    let _ = update(&mut app, Message::Click(x, y, crate::domain::input::MouseButton::Left));
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.remove"));
}

#[test]
fn subscribed_flip_probes_steam() {
    for subscribed in [false, true] {
        let mut app = test_app();
        let mut scene = wall("scene", "we", 1, 0);
        scene["steam_subscribed"] = json!(subscribed);
        seed(&mut app, &[scene]);
        app.scene.viewport = (800.0, 600.0);
        app.scene.set_visible(true);
        steam_online(&mut app);
        drain_calls(&app);
        let _ = update(&mut app, Message::KeyFlip);
        assert_eq!(
            drain_calls(&app).iter().filter(|(method, _)| method == "status").count(),
            usize::from(subscribed)
        );
        assert_eq!(app.daemon.steam_client_available, if subscribed { None } else { Some(true) });
    }
}

#[test]
fn removal_refreshes_subscription() {
    let mut app = test_app();
    app.scene.set_visible(true);
    let mut scene = wall("42", "we", 1, 0);
    scene["we_id"] = json!("42");
    seed(&mut app, &[scene.clone()]);
    app.scene.set_flipped_for_test(Some(0));
    drain_calls(&app);
    super::update::delete_wallpaper(&mut app, 0);
    drain_calls(&app);
    let id = *app
        .daemon
        .pending
        .iter()
        .find(|(_, request)| matches!(request, Pending::RemoveWallpaper { .. }))
        .unwrap()
        .0;
    app.handle_ipc(crate::infrastructure::ipc::IpcMsg::Response {
        id,
        result: None,
        error: Some(wall_proto::ErrorInfo { code: 1, message: "Use Unsubscribe".into() }),
    });
    assert_eq!(drain_calls(&app)[0].0, wall_proto::rpc::WALL_LIST);
    let id =
        *app.daemon.pending.iter().find(|(_, request)| matches!(request, Pending::List)).unwrap().0;
    scene["steam_subscribed"] = json!(true);
    app.handle_ipc(crate::infrastructure::ipc::IpcMsg::Response {
        id,
        result: Some(json!({"wallpapers": [scene], "count": 1})),
        error: None,
    });
    frame(&mut app, Instant::now() + Duration::from_secs(1), 800.0, 600.0);
    assert!(app.library_session.library.catalog().items[0].steam_subscribed);
    app.scene.toggle_flip(0);
    frame(&mut app, Instant::now() + Duration::from_secs(2), 800.0, 600.0);
    assert_eq!(app.scene.render.back.as_ref().unwrap().removal.label(), "Unsubscribe");
    drain_calls(&app);
    steam_online(&mut app);
    super::update::delete_wallpaper(&mut app, 0);
    assert_eq!(drain_calls(&app)[0].1["unsubscribe"], true);
}

#[test]
fn unsubscribe_warns() {
    let mut app = test_app();
    app.on_event("skwd.wall.unsubscribed", &json!({ "id": "4242", "ok": true, "warn": false }));
    assert!(app.runtime_state.toast.is_none());
    app.on_event("skwd.wall.unsubscribed", &json!({ "id": "4242", "ok": false, "warn": false }));
    assert!(app.runtime_state.toast.is_none());
    app.on_event("skwd.wall.unsubscribed", &json!({ "id": "4242", "ok": false, "warn": true }));
    assert!(
        app.runtime_state
            .toast
            .as_ref()
            .is_some_and(|(text, _)| text.contains("could not unsubscribe"))
    );
}

#[test]
fn config_mode_0600() {
    use std::os::unix::fs::PermissionsExt;
    let mut app = test_app();
    app.config.save_key("wallhaven.apiKey", json!("secret"));
    let mode = std::fs::metadata(&app.config.config_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

#[test]
fn removal_closes_back_panel() {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("b.png", "static", 2, 0),
            wall("c.png", "static", 3, 0),
        ],
    );
    app.scene.viewport = (800.0, 600.0);
    app.scene.set_visible(true);
    let fi = 0usize;
    app.scene.set_current(fi, 3);
    app.scene.toggle_flip(fi);
    let t0 = Instant::now();
    for i in 0..=20u64 {
        frame(&mut app, t0 + Duration::from_millis(i * 16), 800.0, 600.0);
    }
    assert!(app.scene.flip_open());
    let victim = app.library_session.library.catalog().items
        [app.library_session.filtered[1] as usize]
        .key
        .clone();
    app.on_event("skwd.wall.removed", &json!({ "key": victim }));
    assert!(!app.scene.flip_open());
}

#[test]
fn tint_follows_backends() {
    use super::theme_bar::tint_follows_wallpaper;
    for backend in ["static", "off"] {
        assert!(!tint_follows_wallpaper(backend), "{backend}");
    }
    for backend in ["", "native", "matugen", "wallust", "pywal", "noctalia", "dms"] {
        assert!(tint_follows_wallpaper(backend), "{backend}");
    }
}

#[test]
fn palette_fade_frames() {
    let mut app = test_app();
    app.theme.fade_t.snap(1.0);
    let idle = app.animating();
    app.theme.fade_t.run(0.3, 1.0);
    assert!(app.animating());
    app.theme.fade_t.snap(1.0);
    assert_eq!(app.animating(), idle);
}

#[test]
fn theme_picks_write_canonical_model() {
    use skwd_config::keys::theme;

    let mut app = test_app();
    let pick = |app: &mut App, value: &str| {
        let _ = update(
            app,
            Message::Settings(crate::frontend::settings::SettingsMsg::Pick(
                theme::BACKEND.into(),
                value.into(),
            )),
        );
    };

    pick(&mut app, "matugen");
    assert_eq!(app.config.theme_backend(), "matugen");
    assert_eq!(app.config.str_path(theme::POLICY), "wallpaper");
    assert_eq!(app.config.str_path(theme::AUTHORITY), "skwd");
    assert_eq!(app.config.str_path(theme::ENGINE), "matugen");

    pick(&mut app, "static");
    assert_eq!(app.config.theme_backend(), "static");
    assert_eq!(app.config.str_path(theme::POLICY), "fixed");

    pick(&mut app, "noctalia");
    assert_eq!(app.config.theme_backend(), "noctalia");
    assert_eq!(app.config.str_path(theme::AUTHORITY), "noctalia");

    let text = std::fs::read_to_string(&app.config.config_path).expect("persisted config");
    let value: Value = serde_json::from_str(&text).expect("valid persisted config");
    assert_eq!(value["theme"]["policy"], "wallpaper");
    assert_eq!(value["theme"]["authority"], "noctalia");
    assert!(value["theme"].get("backend").is_none(), "{}", value["theme"]);
}

#[test]
fn close_audition_closes_overlay_only() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    let _ = update(&mut app, Message::OpenThemeAudition);
    assert!(app.theme.audition_open);
    assert!(!app.theme.audition_focused);
    let _ = update(&mut app, Message::CloseThemeAudition);
    assert!(!app.theme.audition_open);
    assert!(!app.picker_obscured());
}

#[test]
fn focused_audition_needs_launch_flag() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    assert!(!app.theme.audition_focused);
    let _ = update(&mut app, Message::OpenThemeAudition);
    assert!(!app.theme.audition_focused);
    let startup = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/startup.rs"),
    )
    .expect("read startup.rs");
    assert!(startup.contains("audition_focused = true"));
}

#[test]
fn preview_failure_releases_demand() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0), wall("b.png", "static", 2, 0)]);
    let backend = app.config.theme_backend();
    app.theme.job_pending = Some(0);
    app.daemon.pending.insert(11, Pending::ThemePreview { card: 0, backend });

    reject(&mut app, 11, "backend unavailable");

    assert!(app.theme.job_pending.is_none());
}

#[test]
fn disconnect_releases_pending_preview_work() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.request_preview_palette(0);
    app.theme.audition_pending_backend = Some(String::from("matugen"));
    assert_eq!(app.theme.job_pending, Some(0));

    app.handle_ipc(IpcMsg::Disconnected);

    assert!(app.theme.job_pending.is_none());
    assert!(app.theme.audition_pending_backend.is_none());
}

#[test]
fn dropped_call_starts_no_job() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.daemon.client = crate::infrastructure::ipc::DaemonClient::disconnected();

    app.request_preview_palette(0);

    assert!(app.theme.job_pending.is_none());
}

#[test]
fn wallpaper_profile_keeps_captured_source_and_both_variants() {
    use crate::contracts::daemon::CurrentTheme;
    use crate::domain::theme::Candidate;
    use crate::frontend::theme_designer::{ThemeDesigner, ThemeMsg};

    let mut app = test_app();
    seed(&mut app, &[wall("other.png", "static", 1, 0)]);
    let mut candidate = Candidate::from_preset("nord").unwrap();
    let secondary =
        skwd_palette::material::ROLE_KEYS.iter().position(|key| *key == "secondary").unwrap();
    candidate.colors[secondary] = "#abcdef".into();
    let mut designer = ThemeDesigner::new(candidate.clone(), String::new());
    designer.wallpaper = Some(CurrentTheme {
        key: "static:source.png".into(),
        name: "source.png".into(),
        thumb: "/tmp/source.webp".into(),
        palette: candidate.clone(),
        dark: true,
    });
    designer.candidate.colors[0] = "#123456".into();
    app.panels.theme_designer = Some(designer);
    drain_calls(&app);
    let _ = update(&mut app, Message::Theme(ThemeMsg::SaveWallpaper));
    let _ = update(&mut app, Message::Theme(ThemeMsg::Variant(false)));
    app.panels.theme_designer.as_mut().unwrap().candidate.colors[0] = "#654321".into();
    let _ = update(&mut app, Message::Theme(ThemeMsg::SaveWallpaper));
    let _ = update(&mut app, Message::Theme(ThemeMsg::ToggleWallpaper(false)));
    let stored: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    let profiles = stored["theme"]["wallpaperProfiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0]["key"], "static:source.png");
    assert_eq!(profiles[0]["dark"]["primary"], "#123456");
    assert_eq!(profiles[0]["light"]["primary"], "#654321");
    assert_eq!(profiles[0]["dark"]["_scheme"]["colors"]["secondary"]["dark"]["color"], "#abcdef");
    assert_eq!(profiles[0]["enabled"], false);
    let _ = update(&mut app, Message::Theme(ThemeMsg::Variant(true)));
    assert_eq!(app.panels.theme_designer.as_ref().unwrap().candidate.colors[0], "#123456");
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.apply"));
}

#[test]
fn pinned_wallpaper_settings_restore_before_apply() {
    use crate::frontend::theme_designer::ThemeMsg;

    let mut app = test_app();
    seed(&mut app, &[wall("pinned.png", "static", 1, 0)]);
    app.scene.set_current(0, app.library_session.filtered.len());
    app.config.save_key(skwd_config::keys::theme::MODE, json!("light"));
    app.config.save_key(skwd_config::keys::theme::STYLE, json!("pastel"));

    let _ = update(&mut app, Message::Theme(ThemeMsg::ToggleWallpaperSettings));
    assert!(crate::app::update::theme::wallpaper_settings_pinned(&app));

    let _ =
        update(&mut app, Message::Theme(ThemeMsg::Option(skwd_config::keys::theme::MODE, "dark")));
    assert_eq!(
        crate::app::update::theme::selected_wallpaper_settings(&app)
            .and_then(|settings| settings.get(skwd_config::keys::theme::MODE).cloned()),
        Some(json!("dark"))
    );

    app.config.save_key(skwd_config::keys::theme::MODE, json!("light"));
    app.config.save_key(skwd_config::keys::theme::STYLE, json!("natural"));
    let key = app.library_session.library.catalog().items[0].key.clone();
    crate::app::update::theme::restore_wallpaper_settings(&mut app, &key);

    assert_eq!(app.config.str_path(skwd_config::keys::theme::MODE), "dark");
    assert_eq!(app.config.str_path(skwd_config::keys::theme::STYLE), "pastel");
}

#[test]
fn saving_a_theme_only_changes_the_pin_when_applied() {
    use crate::frontend::theme_designer::{ThemeDesigner, ThemeMsg};
    use skwd_config::keys::theme;

    let mut app = test_app();
    seed(&mut app, &[wall("pinned.png", "static", 1, 0)]);
    let _ = update(&mut app, Message::Theme(ThemeMsg::ToggleWallpaperSettings));
    let before = app.config.array_values(theme::WALLPAPER_PROFILES);
    let candidate = crate::domain::theme::Candidate::from_preset("dracula").unwrap();
    app.panels.theme_designer = Some(ThemeDesigner::new(candidate, String::from("Pinned edit")));

    let _ = update(&mut app, Message::Theme(ThemeMsg::SaveTheme));
    assert_eq!(app.config.array_values(theme::WALLPAPER_PROFILES), before);
    let _ = update(&mut app, Message::Theme(ThemeMsg::SaveApply));
    let pinned = app.config.array_values(theme::WALLPAPER_PROFILES);
    assert_eq!(pinned[0]["settings"][theme::POLICY], "fixed");
    assert_eq!(pinned[0]["settings"][theme::STATIC_THEME], "Pinned edit");
    assert_eq!(
        pinned[0]["settings"].as_object().unwrap().len(),
        skwd_config::theme_profile::KEYS.len()
    );
    app.config.save_key(theme::STATIC_THEME, json!("nord"));
    let key = app.library_session.library.catalog().items[0].key.clone();
    crate::app::update::theme::restore_wallpaper_settings(&mut app, &key);
    assert_eq!(app.config.str_path(theme::STATIC_THEME), "Pinned edit");
}

#[test]
fn pinned_hover_uses_its_backend_when_global_theme_is_off() {
    use skwd_config::keys::theme;

    let mut app = test_app();
    seed(&mut app, &[json!({"name": "pinned.png", "type": "static", "thumb": "/pinned.png"})]);
    let key = app.library_session.library.catalog().items[0].key.clone();
    app.config.set_key(theme::POLICY, json!("off"));
    app.config.set_key(
        theme::WALLPAPER_PROFILES,
        json!([{
            "key": key, "settingsPinned": true,
            "settings": {"theme.policy": "fixed", "theme.staticTheme": "dracula"}
        }]),
    );
    let before = app.config.root().clone();
    let now = Instant::now();
    app.update_theme_preview(now, 0.016);
    app.update_theme_preview(now + Duration::from_millis(60), 0.016);
    assert_eq!(app.theme.preview_target, Some(0));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| method == "theme.preview"
        && params["settings"][theme::STATIC_THEME] == "dracula"));
    assert!(calls.iter().any(|(method, _)| method == "wall.shell_preview"));
    app.daemon
        .pending
        .insert(9, Pending::ThemePreview { card: 0, backend: String::from("static") });
    respond(&mut app, 9, preview_reply());
    assert!(app.theme.cache.contains_key(&0));
    assert_eq!(app.config.root(), &before);
}

#[test]
fn unpin_preserves_saved_colours_and_ends_hover() {
    use crate::frontend::theme_designer::ThemeMsg;
    use skwd_config::keys::theme;

    let mut app = test_app();
    seed(&mut app, &[wall("pinned.png", "static", 1, 0)]);
    let key = app.library_session.library.catalog().items[0].key.clone();
    app.config.set_key(
        theme::WALLPAPER_PROFILES,
        json!([{
            "key": key, "enabled": true, "dark": {"primary": "#123456"},
            "settingsPinned": true, "settings": {"theme.mode": "light"}
        }]),
    );
    app.theme.shell_preview_sent = Some(0);
    let _ = update(&mut app, Message::Theme(ThemeMsg::ToggleWallpaperSettings));
    let profiles = app.config.array_values(theme::WALLPAPER_PROFILES);
    assert!(profiles[0].get("settingsPinned").is_none());
    assert!(profiles[0].get("settings").is_none());
    assert_eq!(profiles[0]["dark"]["primary"], "#123456");
    assert_eq!(profiles[0]["enabled"], true);
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "wall.shell_preview_end"));
}

#[test]
fn pinned_backend_matches_full_config_overlay_without_mutation() {
    use crate::app::update::theme::wallpaper_theme_backend_for;
    use skwd_config::keys::theme;

    let mut app = test_app();
    for legacy in
        ["", "off", "static", "matugen", "wallust", "noctalia", "dms", "caelestia", "end4"]
    {
        for settings in [
            json!({}),
            json!({"theme.policy": "fixed"}),
            json!({"theme.policy": "off"}),
            json!({"theme.policy": "wallpaper", "theme.authority": "noctalia"}),
            json!({"theme.authority": "skwd", "theme.engine": "matugen"}),
            json!({"theme.policy": "invalid", "theme.engine": 42}),
        ] {
            app.config = Config::from_data(json!({
                "theme": {"backend": legacy, "wallpaperProfiles": [{
                    "key": "pinned", "settingsPinned": true, "settings": settings,
                    "dark": {"primary": "#123456"}, "light": {"primary": "#abcdef"}
                }]},
                "matugen": {"mode": "light"},
                "unrelated": {"values": [1, 2, 3]}
            }));
            let before = app.config.root().clone();
            let mut expected = before.clone();
            let overrides = skwd_config::theme_profile::settings(
                app.config.array_slice(theme::WALLPAPER_PROFILES),
                "pinned",
            )
            .unwrap();
            for (path, value) in overrides {
                let (group, name) = path.split_once('.').unwrap();
                expected[group][name] = value;
            }
            assert_eq!(
                wallpaper_theme_backend_for(&app, "pinned"),
                skwd_config::theme_backend(&expected)
            );
            assert_eq!(wallpaper_theme_backend_for(&app, "missing"), app.config.theme_backend());
            assert_eq!(app.config.root(), &before);
        }
    }
}

#[cfg(feature = "obs-heap")]
#[test]
fn theme_preview_allocations_do_not_grow_with_saved_profiles() {
    use crate::infrastructure::observability::allocation::thread_alloc_count;
    use skwd_config::keys::theme;

    let mut app = test_app();
    seed(&mut app, &[wall("profile.png", "static", 1, 0)]);
    let key = app.library_session.library.catalog().items[0].key.clone();
    app.theme.preview_target = Some(0);
    app.theme.fade_t.snap(1.0);
    let now = Instant::now();
    for pinned in [false, true] {
        let mut small_allocations = 0;
        for count in [1, 78] {
            let colors: serde_json::Map<String, Value> = (0..64)
                .map(|index| (format!("color-{index}"), json!({"color": "#123456", "tone": 40})))
                .collect();
            let profiles: Vec<_> = (0..count)
                .map(|index| {
                    json!({
                        "key": if index == 0 {key.clone()} else {format!("other-{index}")},
                        "settingsPinned": pinned && index == 0,
                        "settings": {"theme.policy": "fixed", "theme.staticTheme": "nord"},
                        "dark": {"_scheme": {"colors": colors}},
                        "light": {"_scheme": {"colors": colors}}
                    })
                })
                .collect();
            app.config.set_key(theme::WALLPAPER_PROFILES, json!(profiles));
            app.config.set_key("unrelated", json!({"nested": vec!["payload"; count * 128]}));
            app.update_theme_preview(now, 0.016);
            let before = thread_alloc_count();
            for _ in 0..32 {
                app.update_theme_preview(now, 0.016);
                assert_eq!(crate::app::update::theme::wallpaper_settings_pinned(&app), pinned);
            }
            let allocations = thread_alloc_count() - before;
            assert!(
                allocations <= 128 * 32,
                "pinned={pinned}, count={count}, allocations={allocations}"
            );
            if count == 1 {
                small_allocations = allocations;
            } else {
                assert_eq!(allocations, small_allocations, "pinned={pinned}");
            }
        }
    }
}

#[cfg(feature = "obs-heap")]
#[test]
fn saved_theme_names_do_not_clone_palette_payloads() {
    use crate::contracts::settings::SettingsSource;
    use crate::infrastructure::observability::allocation::thread_alloc_count;

    let themes: Vec<_> = (0..78)
        .map(|index| json!({"name": format!("theme-{index}"), "_scheme": vec!["#123456"; 1500]}))
        .collect();
    let app = test_app();
    let mut config = app.config;
    config.set_key(skwd_config::keys::theme::SAVED_THEMES, json!(themes));
    let before = thread_alloc_count();
    let names = config.saved_theme_names();
    let allocations = thread_alloc_count() - before;
    assert_eq!(names.len(), 78);
    assert_eq!(names[0], "theme-0");
    assert!(allocations < 100, "{allocations} allocations to read 78 names");
}

#[test]
fn assigned_palette_save() {
    use crate::domain::theme::Candidate;
    use crate::frontend::theme_designer::{ThemeDesigner, ThemeMsg};
    for (path, apply) in [
        ("theme.appProfiles.kitty", false),
        ("integrations.0.profile", false),
        ("theme.appProfiles.kitty", true),
        ("integrations.0.profile", true),
    ] {
        let mut app = test_app();
        app.config.set_key("integrations", json!([{"name": "custom"}]));
        app.config.set_key(path, json!("Mine"));
        app.config.set_key(skwd_config::keys::theme::STATIC_THEME, json!("nord"));
        let candidate = Candidate::from_preset("dracula").unwrap();
        app.panels.theme_designer = Some(ThemeDesigner::new(candidate.clone(), "Mine".into()));
        drain_calls(&app);
        let _ = update(
            &mut app,
            Message::Theme(if apply { ThemeMsg::SaveApply } else { ThemeMsg::SaveTheme }),
        );
        let stored = crate::infrastructure::theme::find_saved(
            app.config.array_slice(skwd_config::keys::theme::SAVED_THEMES),
            "Mine",
        );
        assert_eq!(stored, Some(candidate));
        assert_eq!(app.config.str_path(path), "Mine");
        assert_eq!(
            app.config.str_path(skwd_config::keys::theme::STATIC_THEME),
            if apply { "Mine" } else { "nord" }
        );
        assert!(drain_calls(&app).iter().any(|(method, params)| method == "theme.apps.refresh"
            && params == &json!({"profile": "Mine"})));
    }
}

#[test]
fn assigned_palette_delete() {
    use crate::domain::theme::Candidate;
    use crate::frontend::theme_designer::{ThemeDesigner, ThemeMsg};
    for path in ["theme.appProfiles.kitty", "integrations.0.profile"] {
        let mut app = test_app();
        let candidate = Candidate::from_preset("nord").unwrap();
        let saved = crate::infrastructure::theme::upsert_saved(&[], "Mine", &candidate);
        app.config.set_key(skwd_config::keys::theme::SAVED_THEMES, json!(saved));
        app.config.set_key("integrations", json!([{"name": "custom"}]));
        app.config.set_key(path, json!("Mine"));
        app.panels.theme_designer = Some(ThemeDesigner::new(candidate, "Mine".into()));
        drain_calls(&app);
        for _ in 0..2 {
            let _ = update(&mut app, Message::Theme(ThemeMsg::DeleteSaved("Mine".into())));
        }
        assert!(
            crate::infrastructure::theme::find_saved(
                app.config.array_slice(skwd_config::keys::theme::SAVED_THEMES),
                "Mine"
            )
            .is_some()
        );
        assert_eq!(app.config.str_path(path), "Mine");
        assert!(app.runtime_state.toast.is_some());
        assert!(drain_calls(&app).is_empty());
    }
}
