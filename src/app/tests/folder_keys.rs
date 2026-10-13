use super::*;
use crate::domain::input::{InputAction, InputMap};
use iced::keyboard::key::Named;

fn press(app: &mut App, key: keyboard::Key) {
    let _ = update(app, Message::KeyPressed(key, keyboard::Modifiers::CTRL));
}

fn fixtures(app: &mut App) {
    seed(
        app,
        &[
            wall("root.png", "static", 1, 0),
            wall(".private/a.png", "static", 2, 0),
            wall(".private/deep/b.png", "static", 3, 0),
            wall("nature/a.png", "static", 4, 0),
            wall("nature/.private/b.png", "static", 5, 0),
            wall("zebra/a.png", "static", 6, 0),
        ],
    );
}

#[test]
fn refreshed_folder_selection() {
    let mut app = test_app();
    app.config.set_key("library.folders", json!(["/home/me/posters"]));
    let walls = [wall("/home/me/posters/a.jpg", "static", 1, 0)];
    for (saved, expected) in
        [("home/me/posters", "*"), ("/home/me/posters", "/home/me/posters"), ("", "*"), ("*", "*")]
    {
        app.library_session.filters.folder = saved.into();
        seed(&mut app, &[]);
        assert_eq!(app.library_session.filters.folder, saved);
        seed(&mut app, &walls);
        assert_eq!(app.library_session.filters.folder, expected);
    }
}

#[test]
fn folder_keys_cycle_menu_order_and_wrap() {
    let mut app = test_app();
    fixtures(&mut app);
    let options = app.library_session.folder_options.clone();
    let _ = update(&mut app, Message::SetFolder(String::new()));
    assert_eq!(app.library_session.filters.folder, "*");
    for expected in options.iter().skip(1).chain(options.iter().take(1)) {
        press(&mut app, keyboard::Key::Named(Named::ArrowRight));
        assert_eq!(&app.library_session.filters.folder, expected);
    }
    press(&mut app, keyboard::Key::Named(Named::ArrowLeft));
    assert_eq!(app.library_session.filters.folder, *options.last().unwrap());
    app.library_session.filters.folder = "removed".into();
    press(&mut app, keyboard::Key::Named(Named::ArrowRight));
    assert_eq!(app.library_session.filters.folder, "*");
    assert!(drain_calls(&app).is_empty());
}

#[test]
fn folder_toggle_reuses_filter_and_clears_playlist() {
    let mut app = test_app();
    fixtures(&mut app);
    app.library_session.playlist_filter =
        Some((1, "test".into(), std::collections::HashSet::new()));
    press(&mut app, keyboard::Key::Character("m".into()));
    assert_eq!(app.library_session.filters.folder, "*");
    assert!(app.library_session.playlist_filter.is_none());
    assert_eq!(app.library_session.filtered.len(), 6);
    press(&mut app, keyboard::Key::Character("m".into()));
    assert_eq!(app.library_session.filters.folder, "*");
    assert_eq!(app.library_session.filtered.len(), 6);
    let _ = update(&mut app, Message::SetFolder("/wp/nature".into()));
    assert_eq!(filtered_names(&app), ["nature/a.png", "nature/.private/b.png"]);
    press(&mut app, keyboard::Key::Character("m".into()));
    assert_eq!(app.library_session.filters.folder, "*");
}

fn public_profile(app: &mut App) {
    app.config.set_key(
        "library.profiles",
        json!([{"id": "public", "hide": ["/wp/nature"], "exclude": ["/wp/zebra"]}]),
    );
    app.config.set_key("library.profile", json!("public"));
}

#[test]
fn show_hidden_reveals_profile() {
    let mut app = test_app();
    public_profile(&mut app);
    fixtures(&mut app);
    let catalog = app.library_session.library.catalog().items.clone();
    assert_eq!(filtered_names(&app), ["root.png", ".private/a.png", ".private/deep/b.png"]);
    assert!(!app.library_session.folder_options.iter().any(|folder| folder == "/wp/nature"));
    press(&mut app, keyboard::Key::Character("h".into()));
    assert!(app.library_session.filters.reveal);
    assert_eq!(
        filtered_names(&app),
        [
            "root.png",
            ".private/a.png",
            ".private/deep/b.png",
            "nature/a.png",
            "nature/.private/b.png"
        ]
    );
    assert!(app.library_session.folder_options.iter().any(|folder| folder == "/wp/nature"));
    assert!(!app.library_session.folder_options.iter().any(|folder| folder == "/wp/zebra"));
    let _ = update(&mut app, Message::SetFolder("/wp/nature".into()));
    press(&mut app, keyboard::Key::Character("h".into()));
    assert!(!app.library_session.filters.reveal);
    assert_eq!(app.library_session.filters.folder, "*");
    assert_eq!(app.library_session.library.catalog().items, catalog);
    assert!(drain_calls(&app).is_empty());
}

#[test]
fn nothing_hidden_ignores_key() {
    let mut app = test_app();
    fixtures(&mut app);
    press(&mut app, keyboard::Key::Character("h".into()));
    assert!(!app.library_session.filters.reveal);
    assert_eq!(app.library_session.filtered.len(), 6);
}

#[test]
fn reveal_never_persists() {
    let mut app = test_app();
    app.config.set_key("filterBar.sticky", json!(true));
    public_profile(&mut app);
    fixtures(&mut app);
    press(&mut app, keyboard::Key::Character("h".into()));
    assert!(app.library_session.filters.reveal);
    let data: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    assert!(data["filterBar"]["last"].get("showHiddenFolders").is_none());
    assert!(!startup_filters(&Config::from_data(data)).reveal);
    app.conceal();
    assert!(!app.library_session.filters.reveal);
    assert_eq!(app.library_session.filtered.len(), 3);
}

#[test]
fn folder_shortcuts_respect_capture_and_can_be_rebound() {
    let mut app = test_app();
    fixtures(&mut app);
    let before = app.library_session.filters.clone();
    app.panels.settings.open = true;
    for key in [
        keyboard::Key::Named(Named::ArrowRight),
        keyboard::Key::Character("m".into()),
        keyboard::Key::Character("h".into()),
    ] {
        press(&mut app, key);
    }
    assert_eq!(app.library_session.filters, before);
    app.panels.settings.open = false;
    app.input.bindings = InputMap::from_bindings([
        (InputAction::FolderToggle, "ctrl+j".into()),
        (InputAction::ShowHidden, "none".into()),
    ]);
    press(&mut app, keyboard::Key::Character("m".into()));
    press(&mut app, keyboard::Key::Character("h".into()));
    assert_eq!(app.library_session.filters, before);
    press(&mut app, keyboard::Key::Character("j".into()));
    assert_eq!(app.library_session.filters.folder, "*");
}

#[test]
fn folder_shortcuts_settle_after_rapid_changes_in_every_mode() {
    for mode in ["slices", "wall", "hex", "sandy"] {
        let mut app = test_app();
        public_profile(&mut app);
        fixtures(&mut app);
        let _ = update(&mut app, Message::SetViewMode(mode.into()));
        let mut now = Instant::now();
        tick_frames(&mut app, &mut now, 180);
        for _ in 0..7 {
            press(&mut app, keyboard::Key::Character("h".into()));
            press(&mut app, keyboard::Key::Character("m".into()));
        }
        tick_frames(&mut app, &mut now, 240);
        assert_eq!(app.library_session.filters.folder, "*", "{mode}");
        assert!(app.library_session.filters.reveal, "{mode}");
        assert_eq!(app.library_session.filtered.len(), 5, "{mode}");
        assert!(!app.library_session.filter_transition_pending, "{mode}");
        assert!(!app.scene.filter_swap_active(), "{mode}");
    }
}

#[test]
fn empty_library_folder_shortcuts_are_valid() {
    let mut app = test_app();
    seed(&mut app, &[]);
    press(&mut app, keyboard::Key::Named(Named::ArrowRight));
    press(&mut app, keyboard::Key::Named(Named::ArrowLeft));
    press(&mut app, keyboard::Key::Character("h".into()));
    assert!(app.library_session.filtered.is_empty());
    assert_eq!(app.library_session.folder_options, ["*", "/wp", "/vids"]);
}
