use super::*;
use crate::app::toggle_hidden;
use crate::infrastructure::runtime::Wake;

fn walls() -> [Value; 3] {
    [
        wall("a.png", "static", 1, 0),
        wall(".old/b.png", "static", 2, 0),
        wall("games/c.png", "static", 3, 0),
    ]
}

#[test]
fn legacy_dot_folders_migrate() {
    let mut app = test_app();
    app.config.set_key("filterBar.sticky", json!(true));
    app.config.set_key("filterBar.last", json!({"showHiddenFolders": false, "folder": "*"}));
    seed(&mut app, &walls());
    assert_eq!(filtered_names(&app), ["a.png", "games/c.png"]);
    assert_eq!(app.library_session.library.catalog().hidden.len(), 1);
    assert!(app.config.root()["filterBar"]["last"].get("showHiddenFolders").is_none());
    let _ = update(&mut app, Message::ToggleShowHidden);
    assert_eq!(filtered_names(&app), ["a.png", ".old/b.png", "games/c.png"]);
}

#[test]
fn card_hide_round_trip() {
    let mut app = test_app();
    seed(&mut app, &walls());
    toggle_hidden(&mut app, 0);
    assert_eq!(filtered_names(&app), [".old/b.png", "games/c.png"]);
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "wall.set_hidden");
    assert_eq!(calls[0].1["hidden"], json!(true));
    let _ = update(&mut app, Message::ToggleShowHidden);
    assert_eq!(filtered_names(&app).len(), 3);
    toggle_hidden(&mut app, 0);
    assert_eq!(drain_calls(&app)[0].1["hidden"], json!(false));
    assert!(app.library_session.library.catalog().hidden.is_empty());
}

#[test]
fn profile_command_scopes_launch() {
    let mut app = test_app();
    app.config.set_key(
        "library.profiles",
        json!([{"id": "games", "exclude": ["/wp"], "show": ["/wp/games"]}]),
    );
    seed(&mut app, &walls());
    assert_eq!(filtered_names(&app).len(), 3);
    let _ = update(&mut app, Message::Daemon(Wake::Command("profile games".into())));
    assert_eq!(app.library_session.profile_override.as_deref(), Some("games"));
    assert_eq!(filtered_names(&app), ["games/c.png"]);
    app.conceal();
    assert_eq!(app.library_session.profile_override, None);
    assert_eq!(filtered_names(&app).len(), 3);
}

#[test]
fn unsticky_skips_migration() {
    let mut app = test_app();
    app.config.set_key("filterBar.last", json!({"showHiddenFolders": false, "folder": "*"}));
    seed(&mut app, &walls());
    assert_eq!(filtered_names(&app).len(), 3);
    assert!(app.library_session.library.catalog().hidden.is_empty());
}

#[test]
fn classifier_kind_fallback() {
    use crate::app::model::library_item;
    use crate::domain::library::catalog::{Wallpaper, WallpaperKind};
    use skwd_config::LibraryItem;
    let item = |key: &str, kind: WallpaperKind, path: &str| Wallpaper {
        key: key.into(),
        kind,
        path: path.into(),
        ..Default::default()
    };
    let steam = "/home/me/.local/share/Steam/movie.mp4";
    assert_eq!(library_item(&item("we:1", WallpaperKind::Video, steam)), LibraryItem::Workshop);
    assert_eq!(library_item(&item("scene", WallpaperKind::We, "")), LibraryItem::Workshop);
    assert_eq!(
        library_item(&item("a.png", WallpaperKind::Static, "/wp/a.png")),
        LibraryItem::Folder(std::path::Path::new("/wp/a.png"))
    );
    assert_eq!(library_item(&item("a.png", WallpaperKind::Static, "")), LibraryItem::Other);
}
