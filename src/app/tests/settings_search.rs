use iced::keyboard::{self, Key, key::Named};
use serde_json::json;

use super::*;
use crate::contracts::settings::SettingsSource;
use crate::frontend::settings::{Control, SettingsFocus, SettingsKey, SettingsMsg};

const ROTATE: &str = "rotate";
const SECOND: usize = 1;
const DESTRUCTIVE_QUERY: &str = "optimise";
const EDITED_FPS: &str = "77";
const HEX_QUERY: &str = "hex rows";
const HEX: &str = "hex";
const SLICES: &str = "slices";
const GHOST: &str = "ghost row";

fn searching(query: &str) -> App {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.panels.settings.entrance.snap(1.0);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchInput(query.into())));
    app
}

fn key(app: &mut App, key: SettingsKey) {
    let _ = update(app, Message::Settings(SettingsMsg::Key(key)));
}

fn select(app: &mut App, wanted: impl Fn(&Control) -> bool) {
    let at = app
        .panels
        .settings
        .search_results
        .iter()
        .position(|result| result.live().is_some_and(|row| wanted(&row.control)))
        .expect("result");
    for _ in 0..=at {
        key(app, SettingsKey::Down);
    }
    assert_eq!(app.panels.settings.search_at, Some(at));
}

#[test]
fn arrows_walk_results() {
    let mut app = searching("fps");
    assert_eq!(app.panels.settings.search_at, None);
    key(&mut app, SettingsKey::Up);
    assert_eq!(app.panels.settings.search_at, None);
    key(&mut app, SettingsKey::Down);
    key(&mut app, SettingsKey::Down);
    assert_eq!(app.panels.settings.search_at, Some(SECOND));
    for _ in 0..20 {
        key(&mut app, SettingsKey::Down);
    }
    let last = app.panels.settings.search_results.len() - 1;
    assert_eq!(app.panels.settings.search_at, Some(last));
    for _ in 0..=last {
        key(&mut app, SettingsKey::Up);
    }
    assert_eq!(app.panels.settings.search_at, None);
}

#[test]
fn typing_clears_selection() {
    let mut app = searching("fps");
    key(&mut app, SettingsKey::Down);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchInput("fp".into())));
    assert_eq!(app.panels.settings.search_at, None);
}

#[test]
fn space_flips_in_place() {
    use skwd_config::keys::general::RANDOM_ROTATE;
    let mut app = searching(ROTATE);
    let wanted = |control: &Control| control.path() == Some(RANDOM_ROTATE);
    select(&mut app, wanted);
    let before = app.config.flag(RANDOM_ROTATE);
    let tab = app.panels.settings.tab.clone();
    let _ = update(
        &mut app,
        Message::KeyPressed(Key::Character(" ".into()), keyboard::Modifiers::default()),
    );
    assert_eq!(app.config.flag(RANDOM_ROTATE), !before);
    assert!(app.panels.settings.search_open);
    assert_eq!(app.panels.settings.tab, tab);
    let row = app.panels.settings.search_results.iter().find_map(|result| result.live());
    let shown = app
        .panels
        .settings
        .search_results
        .iter()
        .filter_map(|result| result.live())
        .find(|row| wanted(&row.control))
        .and_then(|row| row.control.switched_on());
    assert!(row.is_some());
    assert_eq!(shown, Some(!before));
}

#[test]
fn space_outside_activates() {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    app.panels.settings.focus = SettingsFocus::Index;
    let _ = update(
        &mut app,
        Message::KeyPressed(Key::Character(" ".into()), keyboard::Modifiers::default()),
    );
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
}

#[test]
fn menu_space_enter() {
    use skwd_config::keys::performance::GPU_PREFERENCE as PICKER_GPU;
    let mut app = searching("picker gpu");
    select(&mut app, |control| control.path() == Some(PICKER_GPU));
    key(&mut app, SettingsKey::Space);
    assert!(app.panels.settings.pop.is_some());
    assert!(app.panels.settings.focused_choice.is_some());
    let before = app.config.str_path(PICKER_GPU);
    let at = app.panels.settings.search_at;
    key(&mut app, SettingsKey::Down);
    assert_eq!(app.panels.settings.search_at, at, "arrows move the menu");
    key(&mut app, SettingsKey::Activate);
    assert!(app.panels.settings.search_open);
    assert!(app.panels.settings.pop.is_none());
    assert_ne!(app.config.str_path(PICKER_GPU), before);
}

#[test]
fn space_edits_number_seeded() {
    use skwd_config::keys::general::MAX_FPS;
    let mut app = searching("max fps");
    let wanted = |control: &Control| control.path() == Some(MAX_FPS);
    select(&mut app, wanted);
    assert!(app.panels.settings.inputs.get(MAX_FPS).is_some_and(|raw| !raw.is_empty()));
    key(&mut app, SettingsKey::Space);
    assert!(app.panels.settings.edit.is_some());
    assert!(app.panels.settings.input_edit.is_some());
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::Input(MAX_FPS.into(), EDITED_FPS.into())));
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.config.num_path(MAX_FPS), 77.0);
    assert!(app.panels.settings.search_open);
}

#[test]
fn sideways_steps_number() {
    use skwd_config::keys::general::MAX_FPS;
    let mut app = searching("max fps");
    select(&mut app, |control| control.path() == Some(MAX_FPS));
    let before = app.config.num_path(MAX_FPS);
    key(&mut app, SettingsKey::Next);
    assert!(app.config.num_path(MAX_FPS) > before);
    key(&mut app, SettingsKey::Previous);
    assert_eq!(app.config.num_path(MAX_FPS), before);
}

#[test]
fn enter_lands_selected() {
    let mut app = searching("fps");
    key(&mut app, SettingsKey::Down);
    key(&mut app, SettingsKey::Down);
    let result = app.panels.settings.search_results[1].clone();
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.tab, result.tab);
    assert!(!app.panels.settings.search_open);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
    assert_eq!(app.panels.settings.focused_control, result.row);
}

#[test]
fn enter_lands_first() {
    let mut app = searching("fps");
    let result = app.panels.settings.search_results[0].clone();
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.tab, result.tab);
    assert_eq!(app.panels.settings.focused_control, result.row);
}

#[test]
fn escape_clears_then_returns() {
    let mut app = searching("fps");
    let tab = app.panels.settings.tab.clone();
    key(&mut app, SettingsKey::Cancel);
    assert!(app.panels.settings.search_open);
    assert!(app.panels.settings.search_query.is_empty());
    assert!(app.panels.settings.search_results.is_empty());
    key(&mut app, SettingsKey::Cancel);
    assert!(!app.panels.settings.search_open);
    assert!(app.panels.settings.open);
    assert_eq!(app.panels.settings.tab, tab);
}

#[test]
fn escape_closes_menu_first() {
    use skwd_config::keys::performance::GPU_PREFERENCE as PICKER_GPU;
    let mut app = searching("picker gpu");
    select(&mut app, |control| control.path() == Some(PICKER_GPU));
    key(&mut app, SettingsKey::Space);
    key(&mut app, SettingsKey::Cancel);
    assert!(app.panels.settings.pop.is_none());
    assert_eq!(app.panels.settings.search_query, "picker gpu");
}

#[test]
fn escape_reaches_from_field() {
    let mut app = searching("fps");
    let _ = update(
        &mut app,
        Message::KeyPressed(Key::Named(Named::Escape), keyboard::Modifiers::default()),
    );
    assert!(app.panels.settings.search_query.is_empty());
}

#[test]
fn control_f_opens_search() {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(
        &mut app,
        Message::KeyPressed(Key::Character("f".into()), keyboard::Modifiers::CTRL),
    );
    assert!(app.panels.settings.search_open);
}

#[test]
fn outside_change_rereads() {
    use skwd_config::keys::general::RANDOM_ROTATE;
    let mut app = searching(ROTATE);
    let before = app.config.flag(RANDOM_ROTATE);
    app.config.save_key(RANDOM_ROTATE, json!(!before));
    super::super::update::reread_search(&mut app);
    let shown = app
        .panels
        .settings
        .search_results
        .iter()
        .filter_map(|result| result.live())
        .find(|row| row.control.path() == Some(RANDOM_ROTATE))
        .and_then(|row| row.control.switched_on());
    assert_eq!(shown, Some(!before));
}

#[test]
fn keybinding_space_inert() {
    let mut app = searching("toggle playlists");
    let at = app
        .panels
        .settings
        .search_results
        .iter()
        .position(|result| {
            matches!(
                result.setting.as_ref().map(|row| &row.control),
                Some(Control::KeyBinding { .. })
            )
        })
        .expect("key");
    for _ in 0..=at {
        key(&mut app, SettingsKey::Down);
    }
    key(&mut app, SettingsKey::Space);
    assert!(app.panels.settings.keybind_capture.is_none());
}

fn ghost(app: &App) -> crate::frontend::settings::SettingsSearchResult {
    let mut ghost = app.panels.settings.search_results[0].clone();
    let mut row = ghost.setting.clone().expect("row");
    row.title = GHOST.into();
    row.desc = GHOST.into();
    row.control = Control::Static;
    ghost.title = GHOST.into();
    ghost.setting = Some(row);
    ghost
}

fn titles(app: &App) -> Vec<String> {
    app.panels.settings.search_results.iter().map(|result| result.title.clone()).collect()
}

#[test]
fn cursor_follows_row() {
    let mut app = searching("fps");
    let shown = titles(&app);
    let ghost = ghost(&app);
    app.panels.settings.search_results.insert(0, ghost);
    app.panels.settings.search_at = Some(SECOND + 1);
    super::super::update::reread_search(&mut app);
    assert_eq!(titles(&app), shown);
    assert_eq!(app.panels.settings.search_at, Some(SECOND));
}

#[test]
fn cursor_clamps_when_gone() {
    let mut app = searching("fps");
    let last = app.panels.settings.search_results.len();
    let ghost = ghost(&app);
    app.panels.settings.search_results.push(ghost);
    app.panels.settings.search_at = Some(last);
    super::super::update::reread_search(&mut app);
    assert_eq!(app.panels.settings.search_results.len(), last);
    assert_eq!(app.panels.settings.search_at, Some(last - 1));
}

#[test]
fn arrows_skip_reread() {
    let mut app = searching("fps");
    let ghost = ghost(&app);
    app.panels.settings.search_results.insert(0, ghost);
    let before = app.panels.settings.search_results.len();
    key(&mut app, SettingsKey::Down);
    assert_eq!(app.panels.settings.search_results.len(), before);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(app.panels.settings.search_results.len(), before - 1);
}

fn destructive(control: &Control) -> bool {
    matches!(control, Control::ActionBtn { id, .. } if id.is_destructive())
}

#[test]
fn space_arms_visibly() {
    let mut app = searching(DESTRUCTIVE_QUERY);
    select(&mut app, destructive);
    key(&mut app, SettingsKey::Space);
    assert!(app.panels.settings.armed.is_some());
    key(&mut app, SettingsKey::Up);
    assert!(app.panels.settings.armed.is_none());
    key(&mut app, SettingsKey::Down);
    key(&mut app, SettingsKey::Space);
    assert!(app.panels.settings.armed.is_some());
}

#[test]
fn typing_disarms() {
    let mut app = searching(DESTRUCTIVE_QUERY);
    select(&mut app, destructive);
    key(&mut app, SettingsKey::Space);
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchInput(DESTRUCTIVE_QUERY.into())));
    assert!(app.panels.settings.armed.is_none());
}

#[test]
fn escape_disarms() {
    let mut app = searching(DESTRUCTIVE_QUERY);
    select(&mut app, destructive);
    key(&mut app, SettingsKey::Space);
    key(&mut app, SettingsKey::Cancel);
    assert!(app.panels.settings.armed.is_none());
}

#[test]
fn space_types_in_editor() {
    use skwd_config::keys::general::MAX_FPS;
    let mut app = searching("max fps");
    select(&mut app, |control| control.path() == Some(MAX_FPS));
    key(&mut app, SettingsKey::Space);
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::Input(MAX_FPS.into(), EDITED_FPS.into())));
    key(&mut app, SettingsKey::Space);
    assert!(app.panels.settings.input_edit.is_some());
}

#[test]
fn escape_cancels_number_edit() {
    use skwd_config::keys::general::MAX_FPS;
    let mut app = searching("max fps");
    select(&mut app, |control| control.path() == Some(MAX_FPS));
    let before = app.config.num_path(MAX_FPS);
    key(&mut app, SettingsKey::Space);
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::Input(MAX_FPS.into(), EDITED_FPS.into())));
    key(&mut app, SettingsKey::Cancel);
    assert!(app.panels.settings.input_edit.is_none());
    assert!(app.panels.settings.search_open);
    assert_eq!(app.config.num_path(MAX_FPS), before);
}

#[test]
fn wake_keeps_total() {
    use skwd_config::keys::selector::DISPLAY_MODE;
    let mut app = test_app();
    app.panels.settings.open = true;
    app.config.save_key(DISPLAY_MODE, json!(SLICES));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchInput(HEX_QUERY.into())));
    let shown = titles(&app);
    let total = app.panels.settings.search_total;
    assert!(app.panels.settings.search_results.iter().any(|result| result.wake.is_some()));
    app.config.save_key(DISPLAY_MODE, json!(HEX));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Commit));
    assert_eq!(titles(&app), shown);
    assert_eq!(app.panels.settings.search_total, total);
    assert!(app.panels.settings.search_results.iter().all(|result| result.wake.is_none()));
}
