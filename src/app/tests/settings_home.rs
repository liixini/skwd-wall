use skwd_config::{corners, keys};

use super::*;
use crate::frontend::settings::{
    ActionId, SettingsFocus, SettingsKey, SettingsMsg, place, setting_changes,
};

fn opened() -> App {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    app
}

fn on_home() -> App {
    let mut app = opened();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab(place::HOME.to_string())));
    app.panels.settings.previous = None;
    app.panels.settings.section_anim.snap(1.0);
    app
}

fn changes(app: &App) -> Vec<crate::contracts::settings::SettingChange> {
    setting_changes(&app.place_source())
}

fn run(app: &mut App, id: ActionId) {
    let _ = update(app, Message::Settings(SettingsMsg::Run(id)));
}

#[test]
fn fresh_config_unchanged() {
    let app = opened();
    let found: Vec<_> =
        changes(&app).into_iter().map(|change| (change.path, change.was, change.now)).collect();
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn reset_restores_default() {
    let mut app = opened();
    app.config.save_key(keys::general::RANDOM_ROTATE, json!(true));
    app.config.save_key(keys::wallpaper::VOLUME, json!(40));
    let found = changes(&app);
    assert_eq!(found.len(), 2);
    for change in found {
        run(&mut app, ActionId::ResetSetting(Box::new(change)));
    }
    assert!(changes(&app).is_empty());
    assert!(!app.config.flag_default_config(keys::general::RANDOM_ROTATE));
    assert_eq!(app.config.num_path(keys::wallpaper::VOLUME), 100.0);
}

#[test]
fn reset_all_arms_first() {
    let mut app = opened();
    app.config.save_key(keys::general::RANDOM_ROTATE, json!(true));
    run(&mut app, ActionId::ResetEverything);
    assert_eq!(changes(&app).len(), 1);
    assert_eq!(app.panels.settings.armed, Some(ActionId::ResetEverything));
    run(&mut app, ActionId::ResetEverything);
    assert!(changes(&app).is_empty());
    assert_eq!(app.panels.settings.armed, None);
}

#[test]
fn reset_binding_clears() {
    let mut app = opened();
    app.config.save_key(keys::keybind::FAVOURITE, json!("ctrl+alt+k"));
    let found = changes(&app);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].place, place::KEYBINDS);
    run(&mut app, ActionId::ResetSetting(Box::new(found[0].clone())));
    assert!(changes(&app).is_empty());
    assert_eq!(app.config.str_path(keys::keybind::FAVOURITE), "");
}

#[test]
fn links_open_owner() {
    let mut app = opened();
    let (owner, setting) = (place::AUTOMATION, Some(keys::general::RANDOM_ROTATE));
    run(&mut app, ActionId::OpenPlace { place: owner, setting });
    assert_eq!(app.panels.settings.tab, owner);
    assert!(app.panels.settings.open_section.is_some());
    run(&mut app, ActionId::OpenPlace { place: place::COLOURS, setting: None });
    assert_eq!(app.panels.settings.tab, place::AUTOMATION, "waits for the roll");
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab(place::HOME.to_string())));
}

#[test]
fn memo_follows_writes() {
    let mut app = opened();
    assert!(changes(&app).is_empty());
    app.config.set_key(keys::wallpaper::MUTE, json!(false));
    assert_eq!(changes(&app).len(), 1);
    app.config.remove_key(keys::wallpaper::MUTE);
    assert!(changes(&app).is_empty());
}

#[test]
fn reset_each_kind() {
    let mut app = opened();
    for (path, value) in [
        (keys::paper::WALLPAPER_LAYER, json!("top")),
        (keys::general::LOCALE, json!("Stockholm")),
        (keys::selector::SLICE_WIDTH, json!(222)),
        (keys::general::RANDOM_INTERVAL, json!(90)),
    ] {
        app.config.save_key(path, value);
        let found = changes(&app);
        assert_eq!(found.len(), 1, "{path}");
        run(&mut app, ActionId::ResetSetting(Box::new(found[0].clone())));
        assert!(changes(&app).is_empty(), "{path}");
    }
}

#[test]
fn empty_list_closes() {
    let mut app = on_home();
    app.config.save_key(keys::general::RANDOM_ROTATE, json!(true));
    let found = changes(&app);
    let last = app.place_source().place(place::HOME).sections.len() - 1;
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(last)));
    assert!(app.panels.settings.open_section.is_some());
    run(&mut app, ActionId::ResetSetting(Box::new(found[0].clone())));
    assert_eq!(app.panels.settings.open_section, None);
}

fn press(app: &mut App, key: SettingsKey) {
    let _ = update(app, Message::Settings(SettingsMsg::Key(key)));
}

#[test]
fn keyboard_follows_links() {
    let mut app = on_home();
    let next = SettingsKey::FocusNext { backwards: false };
    press(&mut app, next);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    press(&mut app, next);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    press(&mut app, SettingsKey::Activate);
    assert!(app.panels.settings.search_open, "search row");
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchClose));
    app.panels.settings.focus = SettingsFocus::Sections;
    app.panels.settings.section = 2;
    press(&mut app, next);
    press(&mut app, next);
    press(&mut app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.tab, place::COLOURS);
}

#[test]
fn keyboard_resets_rows() {
    let mut app = on_home();
    app.config.save_key(keys::general::RANDOM_ROTATE, json!(true));
    let last = app.place_source().place(place::HOME).sections.len() - 1;
    let _ = update(&mut app, Message::Settings(SettingsMsg::ToggleSection(last)));
    press(&mut app, SettingsKey::FocusNext { backwards: false });
    assert_eq!(app.panels.settings.focused_control, 0);
    press(&mut app, SettingsKey::Activate);
    assert!(changes(&app).is_empty());
}

#[test]
fn reset_all_spares_personal() {
    let mut app = opened();
    app.config.save_key(keys::sources::PEXELS_API_KEY, json!("abcdefghij"));
    app.config.save_key(keys::general::LOCALE, json!("Stockholm"));
    app.config.save_key(keys::paths::STEAM_WORKSHOP, json!("/somewhere"));
    app.config.save_key(keys::library::FOLDERS, json!(["/one"]));
    app.config.save_key(keys::general::RANDOM_ROTATE, json!(true));
    run(&mut app, ActionId::ResetEverything);
    run(&mut app, ActionId::ResetEverything);
    assert_eq!(app.config.str_path(keys::sources::PEXELS_API_KEY), "abcdefghij");
    assert_eq!(app.config.str_path(keys::general::LOCALE), "Stockholm");
    assert_eq!(app.config.str_path(keys::paths::STEAM_WORKSHOP), "/somewhere");
    assert_eq!(app.config.array_len(keys::library::FOLDERS), 1);
    assert!(!app.config.flag_default_config(keys::general::RANDOM_ROTATE));
    let left: Vec<_> = changes(&app).into_iter().map(|change| change.spared).collect();
    assert_eq!(left.len(), 4);
    assert!(left.iter().all(|spared| *spared));
}

#[test]
fn reset_all_writes_once() {
    let mut app = opened();
    app.config.save_key(keys::general::RANDOM_ROTATE, json!(true));
    app.config.save_key(keys::wallpaper::VOLUME, json!(40));
    app.config.save_key(keys::selector::SLICE_WIDTH, json!(222));
    app.config.save_key(keys::keybind::FAVOURITE, json!("ctrl+alt+k"));
    run(&mut app, ActionId::ResetEverything);
    let before = crate::infrastructure::config::persisted();
    run(&mut app, ActionId::ResetEverything);
    assert_eq!(crate::infrastructure::config::persisted() - before, 1);
    assert!(changes(&app).is_empty());
    assert_eq!(app.config.str_path(keys::keybind::FAVOURITE), "");
}

#[test]
fn reset_all_skips_aliases() {
    let mut app = opened();
    app.config.save_key(keys::wallpaper::VOLUME, json!(40));
    let mut paths: Vec<_> = changes(&app).into_iter().map(|change| change.path).collect();
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), changes(&app).len());
}

#[test]
fn lists_count() {
    let mut app = opened();
    app.config.save_key(keys::library::FOLDERS, json!(["/one", "/two"]));
    app.config.save_key(keys::library::EXCLUDED_FOLDERS, json!(["/three"]));
    let found = changes(&app);
    let list = |path: &str| found.iter().find(|change| change.path == path);
    assert_eq!(list(keys::library::FOLDERS).map(|change| change.place), Some(place::LIBRARY));
    assert!(list(keys::library::EXCLUDED_FOLDERS).is_some());
    let change = list(keys::library::FOLDERS).expect("folders").clone();
    run(&mut app, ActionId::ResetSetting(Box::new(change)));
    assert_eq!(app.config.array_len(keys::library::FOLDERS), 0);
}

fn corner_changes(app: &App) -> usize {
    changes(app).iter().filter(|change| corners::owner(&change.path).is_some()).count()
}

fn leave_straight(app: &mut App, keys: &corners::CornerKeys) {
    let pick = SettingsMsg::Pick(keys.style.into(), "round".into());
    let _ = update(app, Message::Settings(pick));
    assert!(app.config.corners_all_stored(keys));
}

fn untouched(app: &App, keys: &corners::CornerKeys) {
    assert_eq!(app.config.stored(keys.all), None);
    assert_eq!(app.config.stored(keys.same), None);
}

fn undo_style(app: &mut App, keys: &corners::CornerKeys) {
    let style = changes(app).into_iter().find(|change| change.path == keys.style);
    run(app, ActionId::ResetSetting(Box::new(style.expect("style change"))));
}

#[test]
fn undo_leaving_straight() {
    let mut app = opened();
    leave_straight(&mut app, &corners::HAND);
    undo_style(&mut app, &corners::HAND);
    untouched(&app, &corners::HAND);
    assert_eq!(corner_changes(&app), 0);
}

#[test]
fn undo_keeps_other_mode() {
    let mut app = opened();
    leave_straight(&mut app, &corners::HAND);
    leave_straight(&mut app, &corners::DEPTH);
    undo_style(&mut app, &corners::HAND);
    untouched(&app, &corners::HAND);
    assert!(app.config.corners_all_stored(&corners::DEPTH));
    undo_style(&mut app, &corners::DEPTH);
    untouched(&app, &corners::DEPTH);
}

#[test]
fn reset_everything_leaving_straight() {
    let mut app = opened();
    leave_straight(&mut app, &corners::HAND);
    run(&mut app, ActionId::ResetEverything);
    run(&mut app, ActionId::ResetEverything);
    untouched(&app, &corners::HAND);
    assert_eq!(corner_changes(&app), 0);
}

#[test]
fn reset_each_leaving_straight() {
    let mut app = opened();
    leave_straight(&mut app, &corners::HAND);
    for change in changes(&app).into_iter().rev() {
        run(&mut app, ActionId::ResetSetting(Box::new(change)));
    }
    untouched(&app, &corners::HAND);
    assert_eq!(corner_changes(&app), 0);
}
