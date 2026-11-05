use super::*;
use crate::domain::input::{InputAction, KeyId, Mods, NavKeys};
use crate::frontend::settings::{ActionId, SettingsMsg};
use iced::keyboard::Modifiers;
use iced::keyboard::key::Named;

const PICKER: crate::domain::input::ActiveScopes =
    crate::domain::input::ActiveScopes { fields: false, search: false, downloads: false };

fn capture(app: &mut App, path: &str, slot: usize, key: keyboard::Key) {
    let _ = update(app, Message::Settings(SettingsMsg::KeybindCapture(path.into(), slot)));
    let _ = update(app, Message::KeyPressed(key, Modifiers::default()));
}

fn char_action(app: &App, text: &str) -> Option<InputAction> {
    app.input.bindings.lookup_key(&KeyId::Char(text.into()), Mods::NONE, PICKER)
}

fn settings_app() -> App {
    let mut app = test_app();
    let _ = update(&mut app, Message::ToggleSettings);
    app
}

#[test]
fn second_slot_capture_keeps_the_arrow_key() {
    let mut app = settings_app();
    capture(&mut app, "keys.navLeft", 1, keyboard::Key::Character("h".into()));
    assert_eq!(app.config.str_path("keys.navLeft"), "left, h");
    assert_eq!(
        app.input.bindings.lookup_key(&KeyId::Left, Mods::NONE, PICKER),
        Some(InputAction::NavLeft)
    );
    assert_eq!(char_action(&app, "h"), Some(InputAction::NavLeft));
    assert_eq!(char_action(&app, "a"), None);
}

#[test]
fn primary_capture_keeps_the_second_key() {
    let mut app = settings_app();
    capture(&mut app, "keys.navUp", 0, keyboard::Key::Character("i".into()));
    assert_eq!(app.config.str_path("keys.navUp"), "i, w");
    assert_eq!(app.input.bindings.lookup_key(&KeyId::Up, Mods::NONE, PICKER), None);
}

#[test]
fn unbinding_one_slot_leaves_the_other() {
    let mut app = settings_app();
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.navDown".into(), 1)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureUnbind));
    assert_eq!(app.config.str_path("keys.navDown"), "down");
    assert_eq!(char_action(&app, "s"), None);
}

#[test]
fn capture_back_to_the_default_stores_no_override() {
    let mut app = settings_app();
    capture(&mut app, "keys.navRight", 1, keyboard::Key::Character("l".into()));
    assert_eq!(app.config.str_path("keys.navRight"), "right, l");
    capture(&mut app, "keys.navRight", 1, keyboard::Key::Character("d".into()));
    assert_eq!(app.config.str_path("keys.navRight"), "");
}

#[test]
fn vim_and_wasd_rewrite_the_second_keys() {
    let mut app = settings_app();
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::SetNavKeys(NavKeys::Vim))));
    for (path, value) in [
        ("keys.navLeft", "left, h"),
        ("keys.navDown", "down, j"),
        ("keys.navUp", "up, k"),
        ("keys.navRight", "right, l"),
    ] {
        assert_eq!(app.config.str_path(path), value, "{path}");
    }
    for (text, action) in [
        ("h", InputAction::NavLeft),
        ("j", InputAction::NavDown),
        ("k", InputAction::NavUp),
        ("l", InputAction::NavRight),
    ] {
        assert_eq!(char_action(&app, text), Some(action), "{text}");
    }
    assert_eq!(char_action(&app, "a"), None);
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::SetNavKeys(NavKeys::Wasd))));
    for path in ["keys.navLeft", "keys.navDown", "keys.navUp", "keys.navRight"] {
        assert_eq!(app.config.str_path(path), "", "{path}");
    }
    assert_eq!(char_action(&app, "a"), Some(InputAction::NavLeft));
    assert_eq!(char_action(&app, "h"), None);
}

#[test]
fn vim_keys_take_a_letter_from_another_action() {
    let mut app = settings_app();
    capture(&mut app, "keys.playlists", 0, keyboard::Key::Character("j".into()));
    assert_eq!(char_action(&app, "j"), Some(InputAction::Playlists));
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::SetNavKeys(NavKeys::Vim))));
    assert_eq!(char_action(&app, "j"), Some(InputAction::NavDown));
    assert_eq!(app.config.str_path("keys.playlists"), "none");
    assert_eq!(app.input.bindings.conflicts(), Vec::new());
}

#[test]
fn keyboard_activation_picks_a_slot() {
    use crate::frontend::settings::{Control, SettingsKey};
    let mut app = settings_app();
    let mut now = Instant::now();
    let _ = update(&mut app, Message::SetSettingsTab(String::from("keybinds")));
    tick_frames(&mut app, &mut now, 60);
    let section = open_section(&mut app, "Controls");
    let control = section
        .rows()
        .position(|row| {
            matches!(&row.control, Control::KeyBinding { path, .. } if path == "keys.navLeft")
        })
        .expect("nav left row");
    tick_frames(&mut app, &mut now, 60);
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(control)));
    let key = |app: &mut App, key| {
        let _ = update(app, Message::Settings(SettingsMsg::Key(key)));
    };
    key(&mut app, SettingsKey::Activate);
    assert!(app.panels.settings.keybind_capture.is_none());
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    key(&mut app, SettingsKey::Down);
    assert_eq!(app.panels.settings.focused_choice, Some(1));
    key(&mut app, SettingsKey::Down);
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    key(&mut app, SettingsKey::Up);
    key(&mut app, SettingsKey::Next);
    let capture = app.panels.settings.keybind_capture.as_ref().expect("capture open");
    assert_eq!(capture.action, InputAction::NavLeft);
    assert_eq!(capture.slot, 1);
}

#[test]
fn enter_and_tab_bind() {
    let mut app = settings_app();
    capture(&mut app, "keys.navLeft", 1, keyboard::Key::Named(Named::Tab));
    assert_eq!(app.config.str_path("keys.navLeft"), "left, tab");
    assert!(app.panels.settings.keybind_capture.is_none());
    capture(&mut app, "keys.navLeft", 1, keyboard::Key::Named(Named::Enter));
    assert_eq!(app.config.str_path("keys.navLeft"), "left, enter");
}

#[test]
fn symbols_drop_shift() {
    let mut app = settings_app();
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.help".into(), 1)));
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Character("!".into()), Modifiers::SHIFT),
    );
    assert_eq!(app.config.str_path("keys.help"), "?, !");
}

#[test]
fn modifiers_keep_listening() {
    let mut app = settings_app();
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.help".into(), 1)));
    let _ =
        update(&mut app, Message::KeyPressed(keyboard::Key::Named(Named::Shift), Modifiers::SHIFT));
    assert!(app.panels.settings.keybind_capture.is_some());
}

#[test]
fn stealing_notes_undo() {
    let mut app = settings_app();
    capture(&mut app, "keys.settings", 0, keyboard::Key::Character("p".into()));
    assert_eq!(app.config.str_path("keys.playlists"), "none");
    let moved = app.panels.settings.keys_moved.as_ref().expect("note");
    assert_eq!(moved.key, "P");
    assert_eq!(moved.from, ["keybind-playlists"]);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::UndoKeyMove)));
    assert_eq!(app.config.str_path("keys.playlists"), "");
    assert_eq!(app.config.str_path("keys.settings"), "");
    assert!(app.panels.settings.keys_moved.is_none());
    assert_eq!(char_action(&app, "p"), Some(InputAction::Playlists));
}

#[test]
fn plain_bind_clears_note() {
    let mut app = settings_app();
    capture(&mut app, "keys.settings", 0, keyboard::Key::Character("p".into()));
    assert!(app.panels.settings.keys_moved.is_some());
    capture(&mut app, "keys.help", 1, keyboard::Key::Character("=".into()));
    assert!(app.panels.settings.keys_moved.is_none());
}

#[test]
fn leaving_ends_capture() {
    let mut app = settings_app();
    let _ = update(&mut app, Message::SetSettingsTab(String::from("keybinds")));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeysFind("tag".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.help".into(), 1)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("system".into())));
    assert!(app.panels.settings.keybind_capture.is_none());
    assert!(app.panels.settings.keys_query.is_empty());
}

#[test]
fn find_clears_on_escape() {
    let mut app = settings_app();
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeysFind("tag".into())));
    let _ = update(
        &mut app,
        Message::KeyPressed(keyboard::Key::Named(Named::Escape), Modifiers::default()),
    );
    assert!(app.panels.settings.keys_query.is_empty());
    assert!(app.panels.settings.open);
}

#[test]
fn conflict_link_lights() {
    let mut app = settings_app();
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeysFind("tag".into())));
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::LightKey("keys.help"))));
    assert_eq!(app.panels.settings.keys_lit, Some("keys.help"));
    assert!(app.panels.settings.keys_query.is_empty());
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.help".into(), 0)));
    assert_eq!(app.panels.settings.keys_lit, None);
}

#[test]
fn tab_reaches_every_chip() {
    use crate::frontend::settings::{Control, SettingsKey};
    let mut app = settings_app();
    let _ = update(&mut app, Message::SetSettingsTab(String::from("keybinds")));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    let place = app.place_source().place("keybinds");
    let rows: Vec<_> = place.sections[0].rows().cloned().collect();
    let mut seen = Vec::new();
    for _ in 0..=rows.len() * crate::domain::input::SLOTS {
        let _ = update(
            &mut app,
            Message::Settings(SettingsMsg::Key(SettingsKey::FocusNext { backwards: false })),
        );
        if app.panels.settings.focus == crate::frontend::settings::SettingsFocus::Controls {
            let slot = app.panels.settings.focused_choice.unwrap_or(0);
            seen.push((app.panels.settings.focused_control, slot));
        }
    }
    let wanted = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| !matches!(row.control, Control::Segment | Control::Static));
    for (at, row) in wanted {
        assert!(seen.contains(&(at, 0)), "{}", row.title);
        if matches!(row.control, Control::KeyBinding { .. }) {
            assert!(seen.contains(&(at, 1)), "{}", row.title);
        }
    }
}

#[test]
fn enter_opens_second_menu() {
    use crate::frontend::settings::{Control, SettingsKey, second_keys_row, second_keys_token};
    let mut app = settings_app();
    let _ = update(&mut app, Message::SetSettingsTab(String::from("keybinds")));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    let place = app.place_source().place("keybinds");
    let row = place.sections[0].rows().position(|row| second_keys_row(&row.control)).expect("row");
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(row)));
    let _ = update(&mut app, Message::Settings(SettingsMsg::OpenPop(second_keys_token())));
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    let key = |app: &mut App, key| {
        let _ = update(app, Message::Settings(SettingsMsg::Key(key)));
    };
    key(&mut app, SettingsKey::Down);
    key(&mut app, SettingsKey::Activate);
    assert_eq!(app.config.str_path("keys.navLeft"), "left, h");
    assert!(matches!(
        place.sections[0].row(row).map(|row| &row.control),
        Some(Control::ActionChips { .. })
    ));
}

#[test]
fn enter_focuses_find() {
    use crate::frontend::settings::{SettingsFocus, SettingsKey};
    let mut app = settings_app();
    let _ = update(&mut app, Message::SetSettingsTab(String::from("keybinds")));
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    app.panels.settings.focus = SettingsFocus::Sections;
    let task = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(task.units(), 1, "focus operation");
    assert_eq!(app.panels.settings.focus, SettingsFocus::Sections);
    let _ = update(&mut app, Message::SetSettingsTab(String::from("system")));
    app.panels.settings.focus = SettingsFocus::Sections;
    let task = update(&mut app, Message::Settings(SettingsMsg::Key(SettingsKey::Activate)));
    assert_eq!(task.units(), 0, "other places move focus");
}

fn slots_of(app: &App, action: InputAction) -> [Vec<crate::domain::input::Trigger>; 2] {
    use crate::domain::input::slot_triggers;
    [0, 1].map(|slot| slot_triggers(app.input.bindings.triggers(action), slot).to_vec())
}

fn arrow(named: Named) -> keyboard::Key {
    keyboard::Key::Named(named)
}

fn unbind(app: &mut App, path: &str, slot: usize) {
    let _ = update(app, Message::Settings(SettingsMsg::KeybindCapture(path.into(), slot)));
    let _ = update(app, Message::Settings(SettingsMsg::KeybindCaptureUnbind));
}

#[test]
fn unbinding_first_keeps_second() {
    let mut app = settings_app();
    unbind(&mut app, "keys.navLeft", 0);
    let [first, second] = slots_of(&app, InputAction::NavLeft);
    assert!(first.is_empty());
    assert_eq!(second.len(), 1);
    assert_eq!(app.config.str_path("keys.navLeft"), "a, a");
    assert_eq!(char_action(&app, "a"), Some(InputAction::NavLeft));
}

#[test]
fn second_into_empty_first() {
    let mut app = settings_app();
    unbind(&mut app, "keys.playlists", 0);
    capture(&mut app, "keys.playlists", 1, keyboard::Key::Character("x".into()));
    let [first, second] = slots_of(&app, InputAction::Playlists);
    assert!(first.is_empty());
    assert_eq!(second.len(), 1);
    assert_eq!(char_action(&app, "x"), Some(InputAction::Playlists));
    capture(&mut app, "keys.playlists", 0, keyboard::Key::Character("p".into()));
    let [first, second] = slots_of(&app, InputAction::Playlists);
    assert_eq!((first.len(), second.len()), (1, 1));
}

#[test]
fn stealing_keeps_slot_places() {
    let mut app = settings_app();
    capture(&mut app, "keys.settings", 0, arrow(Named::ArrowLeft));
    let [first, second] = slots_of(&app, InputAction::NavLeft);
    assert!(first.is_empty());
    assert_eq!(second.len(), 1);
    capture(&mut app, "keys.help", 0, keyboard::Key::Character("a".into()));
    let [first, second] = slots_of(&app, InputAction::NavLeft);
    assert!(first.is_empty() && second.is_empty());
}

#[test]
fn same_key_notice() {
    let mut app = settings_app();
    capture(&mut app, "keys.navLeft", 1, arrow(Named::ArrowLeft));
    assert_eq!(app.config.str_path("keys.navLeft"), "");
    assert_eq!(app.panels.settings.keys_same.as_deref(), Some("Left"));
    assert!(app.panels.settings.keybind_capture.is_none());
    capture(&mut app, "keys.navLeft", 1, keyboard::Key::Character("h".into()));
    assert!(app.panels.settings.keys_same.is_none());
}

#[test]
fn every_holder_is_named() {
    let mut app = settings_app();
    for path in ["keys.playlists", "keys.help"] {
        app.config.set_key(path, serde_json::json!("x"));
    }
    app.reload_bindings();
    capture(&mut app, "keys.settings", 0, keyboard::Key::Character("x".into()));
    let moved = app.panels.settings.keys_moved.as_ref().expect("note");
    assert_eq!(moved.from, ["keybind-playlists", "keybind-help"]);
    assert_eq!(app.config.str_path("keys.playlists"), "none");
    assert_eq!(app.config.str_path("keys.help"), "none");
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::UndoKeyMove)));
    assert_eq!(app.config.str_path("keys.playlists"), "x");
    assert_eq!(app.config.str_path("keys.help"), "x");
    assert_eq!(app.config.str_path("keys.settings"), "");
}

#[test]
fn reset_restores_every_action() {
    let mut app = settings_app();
    for entry in crate::contracts::picker::KEY_BINDINGS {
        unbind(&mut app, entry.path, 0);
        unbind(&mut app, entry.path, 1);
        assert!(app.input.bindings.triggers(entry.action).is_empty(), "{}", entry.path);
    }
    for _ in 0..2 {
        let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::ResetKeybinds)));
    }
    for entry in crate::contracts::picker::KEY_BINDINGS {
        assert_eq!(app.config.str_path(entry.path), "", "{}", entry.path);
    }
    let fresh = crate::domain::input::InputMap::default();
    for action in InputAction::ALL {
        assert_eq!(app.input.bindings.triggers(action), fresh.triggers(action), "{action:?}");
    }
}

#[test]
fn click_binds_while_listening() {
    use crate::domain::input::MouseButton;
    let mut app = settings_app();
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::KeybindCapture("keys.favourite".into(), 0)),
    );
    assert_eq!(app.config.str_path("keys.favourite"), "");
    assert!(app.panels.settings.keybind_capture.is_some());
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureClick(MouseButton::Left)));
    assert_eq!(app.config.str_path("keys.favourite"), "click");
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCaptureClick(MouseButton::Right)));
    assert_eq!(app.config.str_path("keys.favourite"), "click");
}

fn keys_row_app() -> (App, Instant) {
    use crate::frontend::settings::Control;
    let mut app = settings_app();
    let mut now = Instant::now();
    let _ = update(&mut app, Message::SetSettingsTab(String::from("keybinds")));
    tick_frames(&mut app, &mut now, 60);
    let section = open_section(&mut app, "Controls");
    let control = section
        .rows()
        .position(|row| {
            matches!(&row.control, Control::KeyBinding { path, .. } if path == "keys.navLeft")
        })
        .expect("nav left row");
    tick_frames(&mut app, &mut now, 60);
    let _ = update(&mut app, Message::Settings(SettingsMsg::FocusControl(control)));
    (app, now)
}

fn press(app: &mut App, key: keyboard::Key) {
    let _ = update(app, Message::KeyPressed(key, Modifiers::default()));
}

fn release(app: &mut App, key: keyboard::Key) {
    let _ = update(app, Message::KeyReleased(key));
}

#[test]
fn held_enter_never_binds() {
    let (mut app, _) = keys_row_app();
    press(&mut app, arrow(Named::Enter));
    press(&mut app, arrow(Named::Enter));
    assert!(app.panels.settings.keybind_capture.is_none());
    release(&mut app, arrow(Named::Enter));
    press(&mut app, arrow(Named::Enter));
    assert!(app.panels.settings.keybind_capture.is_some());
    press(&mut app, arrow(Named::Enter));
    press(&mut app, arrow(Named::Enter));
    assert!(app.panels.settings.keybind_capture.is_some());
    assert_eq!(app.config.str_path("keys.navLeft"), "");
    release(&mut app, arrow(Named::Enter));
    press(&mut app, arrow(Named::Enter));
    assert_eq!(app.config.str_path("keys.navLeft"), "enter, a");
}

#[test]
fn held_key_quiet() {
    let (mut app, _) = keys_row_app();
    app.panels.settings.focused_choice = Some(0);
    let _ =
        update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.navLeft".into(), 1)));
    press(&mut app, arrow(Named::ArrowRight));
    assert!(app.panels.settings.keybind_capture.is_none());
    app.panels.settings.focused_choice = Some(0);
    press(&mut app, arrow(Named::ArrowRight));
    assert!(app.panels.settings.keybind_capture.is_none());
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    release(&mut app, arrow(Named::ArrowRight));
    press(&mut app, arrow(Named::ArrowRight));
    assert!(app.panels.settings.keybind_capture.is_some());
}

#[test]
fn escape_repeat_keeps_settings() {
    let mut app = settings_app();
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.help".into(), 1)));
    press(&mut app, arrow(Named::Escape));
    press(&mut app, arrow(Named::Escape));
    assert!(app.panels.settings.open);
}

fn listening() -> App {
    let mut app = settings_app();
    let _ = update(&mut app, Message::Settings(SettingsMsg::KeybindCapture("keys.help".into(), 1)));
    assert!(app.panels.settings.keybind_capture.is_some());
    app
}

#[test]
fn search_ends_capture() {
    let mut app = listening();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    assert!(app.panels.settings.keybind_capture.is_none());
    press(&mut app, keyboard::Key::Character("x".into()));
    assert_eq!(app.config.str_path("keys.help"), "");
}

#[test]
fn search_close_ends_capture() {
    let mut app = listening();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchClose));
    assert!(app.panels.settings.keybind_capture.is_none());
}

#[test]
fn focus_loss_ends_capture() {
    let mut app = listening();
    app.input.down = Some(arrow(Named::Enter));
    let _ = update(&mut app, Message::WindowUnfocused);
    assert!(app.panels.settings.keybind_capture.is_none());
    assert!(app.input.down.is_none());
}

#[test]
fn closing_ends_capture() {
    let mut app = listening();
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.panels.settings.keybind_capture.is_none());
}

#[test]
fn place_switch_ends_capture() {
    let mut app = listening();
    let _ = update(&mut app, Message::SetSettingsTab(String::from("system")));
    assert!(app.panels.settings.keybind_capture.is_none());
}
