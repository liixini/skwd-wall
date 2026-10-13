use super::*;
use crate::contracts::settings::{ProfileState, folder_tag};
use crate::frontend::settings::{ActionId, FOLDER_SEGMENT, SettingsMsg};

fn run(app: &mut App, id: ActionId) {
    let _ = update(app, Message::Settings(SettingsMsg::Run(id)));
}

fn pick_folder(app: &mut App, folder: &str, state: ProfileState) {
    let path = format!("library.profiles.0.{FOLDER_SEGMENT}.{}", folder_tag(folder));
    let _ = update(app, Message::Settings(SettingsMsg::Pick(path, state.key().into())));
}

fn seeded() -> App {
    let mut app = test_app();
    seed(
        &mut app,
        &[
            wall("a.png", "static", 1, 0),
            wall("anime/b.png", "static", 2, 0),
            wall("anime/old/c.png", "static", 3, 0),
        ],
    );
    app
}

#[test]
fn new_profiles_get_ids() {
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    run(&mut app, ActionId::NewProfile);
    let ids: Vec<Value> = app
        .config
        .array_values("library.profiles")
        .iter()
        .map(|entry| entry["id"].clone())
        .collect();
    assert_eq!(ids, [json!("new-profile"), json!("new-profile-2")]);
    assert_eq!(app.panels.settings.profile, 2);
    assert_eq!(
        app.panels.settings.inputs.get("library.profiles.1.name").map(String::as_str),
        Some("New profile")
    );
}

#[test]
fn folder_state_applies() {
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    pick_folder(&mut app, "/wp/anime/old", ProfileState::Exclude);
    pick_folder(&mut app, "/wp/anime", ProfileState::Hide);
    let entry = &app.config.array_values("library.profiles")[0];
    assert_eq!(entry["hide"], json!(["/wp/anime"]));
    assert_eq!(entry["exclude"], json!([]));
    app.config.save_key("library.profile", json!("new-profile"));
    app.reload_library_profile();
    assert_eq!(filtered_names(&app), ["a.png"]);
    let workshop = String::from("library.profiles.0.workshop");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Pick(workshop, "exclude".into())));
    assert_eq!(app.config.array_values("library.profiles")[0]["workshop"], json!("exclude"));
}

#[test]
fn delete_resets_active() {
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    app.config.save_key("library.profile", json!("new-profile"));
    run(&mut app, ActionId::DeleteProfile(0));
    assert_eq!(app.config.array_values("library.profiles").len(), 1);
    assert_eq!(app.panels.settings.profile, 1);
    run(&mut app, ActionId::DeleteProfile(0));
    assert!(app.config.array_values("library.profiles").is_empty());
    assert_eq!(app.panels.settings.profile, 0);
    assert_eq!(app.config.str_path("library.profile"), "everything");
}

#[test]
fn delete_before_keeps_selection() {
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    run(&mut app, ActionId::NewProfile);
    assert_eq!(app.panels.settings.profile, 2);
    run(&mut app, ActionId::DeleteProfile(0));
    run(&mut app, ActionId::DeleteProfile(0));
    let ids: Vec<Value> = app
        .config
        .array_values("library.profiles")
        .iter()
        .map(|entry| entry["id"].clone())
        .collect();
    assert_eq!(ids, [json!("new-profile-2")]);
    assert_eq!(app.panels.settings.profile, 1);
}

#[test]
fn pick_rejects_bad_paths() {
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    let before = app.config.array_values("library.profiles");
    for (path, value) in [
        ("library.profiles.x.workshop", "hide"),
        ("library.profiles.0.workshop", "sideways"),
        (&format!("library.profiles.0.{FOLDER_SEGMENT}.nope"), "hide"),
        ("library.profile", "hide"),
    ] {
        let _ = update(&mut app, Message::Settings(SettingsMsg::Pick(path.into(), value.into())));
    }
    assert_eq!(app.config.array_values("library.profiles"), before);
    assert_eq!(app.config.str_path("library.profile"), "hide");
}
