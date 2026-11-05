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

fn press(app: &mut App, key: crate::frontend::settings::SettingsKey) {
    let _ = update(app, Message::Settings(SettingsMsg::Key(key)));
}

fn open_library(app: &mut App, key: &'static str) {
    use crate::frontend::settings::{SectionId, category::LIBRARY, place};
    app.panels.settings.open = true;
    app.panels.settings.tab = place::LIBRARY.to_string();
    app.init_settings_inputs();
    let id = SectionId::new(LIBRARY, key);
    app.panels.settings.toggle_section(id, true);
    app.panels.settings.section = app
        .place_source()
        .place(place::LIBRARY)
        .sections
        .iter()
        .position(|section| section.id == id)
        .expect("section");
    app.panels.settings.focus = crate::frontend::settings::SettingsFocus::Sections;
}

fn page_lines(app: &App) -> Vec<String> {
    use crate::frontend::settings::Control;
    let section = app.panels.settings.open_section.expect("open section");
    crate::frontend::settings::page_rows(section, &app.page_ctx())
        .into_iter()
        .map(|row| match row.control {
            Control::ActionBtn { id, .. } => format!("{id:?}"),
            Control::TextField { key, .. } => key,
            Control::Chips { path, .. } => path,
            _ => String::new(),
        })
        .collect()
}

#[test]
fn tab_walks_folder_lines() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{SettingsFocus, SettingsKey};
    let mut app = seeded();
    app.config.save_key(FolderKind::Excluded.key(), json!(["/a"]));
    open_library(&mut app, "settings-paths-excluded-card");
    let lines = page_lines(&app);
    assert_eq!(lines.len(), 6, "{lines:?}");
    let forward = SettingsKey::FocusNext { backwards: false };
    press(&mut app, forward);
    for at in 0..lines.len() {
        assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
        assert_eq!(app.panels.settings.focused_control, at);
        press(&mut app, forward);
    }
    assert_eq!(app.panels.settings.focus, SettingsFocus::Index);
}

#[test]
fn enter_edits_and_adds() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{SettingsKey, SettingsMsg, draft_key};
    let mut app = seeded();
    let kind = FolderKind::Excluded;
    app.config.save_key(kind.key(), json!([]));
    open_library(&mut app, "settings-paths-excluded-card");
    press(&mut app, SettingsKey::FocusNext { backwards: false });
    press(&mut app, SettingsKey::Activate);
    let key = draft_key(kind);
    assert_eq!(
        app.panels.settings.input_edit.as_ref().map(|edit| edit.key.clone()),
        Some(key.clone())
    );
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key, "/skip".into())));
    assert!(app.config.array_values(kind.key()).is_empty());
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(app.config.array_values(kind.key()), [json!("/skip")]);
    assert!(app.panels.settings.input_edit.is_none());
}

#[test]
fn escape_keeps_old_draft() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{SettingsKey, SettingsMsg, draft_key};
    let mut app = seeded();
    let kind = FolderKind::Excluded;
    open_library(&mut app, "settings-paths-excluded-card");
    let key = draft_key(kind);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key.clone(), "/old".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(app.config.array_values(kind.key()), [json!("/old")], "enter adds");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key.clone(), "/kept".into())));
    app.panels.settings.input_edit = None;
    let token =
        crate::frontend::settings::token_key(&key, 0, crate::frontend::settings::TokenSide::Body);
    let _ = update(&mut app, Message::Settings(SettingsMsg::EditToken(token, key.clone())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Input(key, "/typo".into())));
    press(&mut app, SettingsKey::Cancel);
    assert_eq!(
        app.panels.settings.folder_drafts.get(kind.key()).map(String::as_str),
        Some("/kept")
    );
    assert_eq!(app.config.array_values(kind.key()), [json!("/old")]);
}

#[test]
fn enter_picks_word() {
    use crate::frontend::settings::{FOLDER_SEGMENT, SettingsKey, SettingsMsg};
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    open_library(&mut app, "settings-profiles-card");
    let lines = page_lines(&app);
    let path = format!("library.profiles.0.{FOLDER_SEGMENT}.{}", folder_tag("/wp/anime"));
    let at = lines.iter().position(|line| *line == path).expect("word row") + 1;
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(at)));
    assert_eq!(app.panels.settings.focused_control, at);
    press(&mut app, SettingsKey::Activate);
    press(&mut app, SettingsKey::Down);
    press(&mut app, SettingsKey::Activate);
    assert_eq!(app.config.array_values("library.profiles")[0]["hide"], json!(["/wp/anime"]));
}

#[test]
fn enter_runs_panel_buttons() {
    use crate::frontend::settings::{SettingsKey, SettingsMsg};
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    open_library(&mut app, "settings-profiles-card");
    let lines = page_lines(&app);
    let activate =
        lines.iter().position(|line| line.starts_with("ActivateProfile")).expect("make active");
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(activate + 1)));
    press(&mut app, SettingsKey::Activate);
    assert_eq!(app.config.str_path("library.profile"), "new-profile");
    let select =
        lines.iter().position(|line| line.starts_with("SelectProfile(0)")).expect("everything");
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(select + 1)));
    press(&mut app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.profile, 0);
}

#[test]
fn first_click_arms() {
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    run(&mut app, ActionId::DeleteProfile(0));
    assert_eq!(app.panels.settings.armed, Some(ActionId::DeleteProfile(0)));
    assert_eq!(app.config.array_values("library.profiles").len(), 1);
    run(&mut app, ActionId::DeleteProfile(0));
    assert_eq!(app.panels.settings.armed, None);
    assert!(app.config.array_values("library.profiles").is_empty());
}

#[test]
fn row_actions_disarm() {
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    let moves = [
        ActionId::SelectProfile(0),
        ActionId::ActivateProfile("new-profile".into()),
        ActionId::FoldTree("/wp/anime".into(), true),
        ActionId::OpenSection(1),
        ActionId::NewProfile,
    ];
    for id in moves {
        assert!(!id.is_destructive());
        run(&mut app, ActionId::DeleteProfile(0));
        assert_eq!(app.panels.settings.armed, Some(ActionId::DeleteProfile(0)));
        run(&mut app, id);
        assert_eq!(app.panels.settings.armed, None);
    }
}

#[test]
fn tab_reaches_every_control() {
    use crate::contracts::settings::FolderKind;
    use crate::frontend::settings::{SettingsFocus, SettingsKey, place};
    let mut app = seeded();
    run(&mut app, ActionId::NewProfile);
    app.config.save_key(FolderKind::Source.key(), json!(["/wp", "/wp/anime"]));
    app.config.save_key(FolderKind::Excluded.key(), json!(["/wp/old"]));
    let sections = app.place_source().place(place::LIBRARY).sections;
    assert_eq!(sections.len(), 5);
    let counts = std::collections::HashMap::from([
        ("settings-section-folders", 10),
        ("settings-paths-excluded-card", 6),
        ("settings-profiles-card", 17),
        ("settings-library-storage", 7),
        ("settings-library-thumbnails", 4),
    ]);
    for section in sections {
        open_library(&mut app, section.id.key);
        let header = section.header_rows();
        let own = section.rows().count();
        let lines = crate::frontend::settings::page_rows(section.id, &app.page_ctx());
        let rows: Vec<_> = section.clone().into_rows().chain(lines).collect();
        let visual: Vec<usize> = (0..header).chain(own..rows.len()).chain(header..own).collect();
        let expected: Vec<usize> = visual
            .into_iter()
            .filter(|at| {
                rows[*at].control.is_focusable()
                    && !(*at < own && (section.say.parks(*at).is_some() || section.follows(*at)))
            })
            .collect();
        assert!(!expected.is_empty());
        let forward = SettingsKey::FocusNext { backwards: false };
        press(&mut app, forward);
        let mut walked = Vec::new();
        while app.panels.settings.focus == SettingsFocus::Controls {
            walked.push(app.panels.settings.focused_control);
            press(&mut app, forward);
        }
        assert_eq!(walked, expected, "{}", section.id.key);
        assert_eq!(walked.len(), counts[section.id.key], "{} count", section.id.key);
    }
}
