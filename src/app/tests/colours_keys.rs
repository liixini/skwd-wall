use super::*;
use crate::contracts::daemon::{AppThemeStatus, AppThemesResult};
use crate::frontend::settings::{
    Control, SettingsFocus, SettingsKey, SettingsMsg, TokenSide, app_key, token_key,
};

const PROFILE: &str = "theme.appProfiles.kitty";
const APPS: &str = "Where colours go";

fn kitty() -> AppThemeStatus {
    AppThemeStatus {
        id: "kitty".into(),
        name: "Kitty".into(),
        installed: true,
        config_found: true,
        config_path: "/c/kitty.conf".into(),
        output_path: "/c/colors.conf".into(),
        enabled: true,
        state: "applied".into(),
        detail: String::new(),
        can_enable: false,
        can_disable: true,
        can_adopt: false,
        template_path: "/t/kitty.tmpl".into(),
        customized: false,
        can_disconnect: true,
        can_reconnect: false,
    }
}

fn focused(status: AppThemeStatus) -> App {
    let mut app = test_app();
    app.panels.settings.open = true;
    app.daemon.app_themes = Some(AppThemesResult { apps: vec![status] });
    let mut now = Instant::now();
    let _ = update(&mut app, Message::Settings(SettingsMsg::SelectTab("colours".into())));
    tick_frames(&mut app, &mut now, 60);
    let section = open_section(&mut app, APPS);
    tick_frames(&mut app, &mut now, 60);
    let at = section
        .rows()
        .position(|row| matches!(row.control, Control::AppTheme { .. }))
        .expect("app row");
    app.panels.settings.focus = SettingsFocus::Controls;
    app.panels.settings.focused_control = at + 1;
    let _ = drain_calls(&app);
    app
}

fn press(app: &mut App, key: SettingsKey) -> usize {
    update(app, Message::Settings(SettingsMsg::Key(key))).units()
}

fn stop(app: &mut App, stop: usize) {
    press(app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.pop, Some(app_key("kitty")));
    for _ in 0..stop {
        press(app, SettingsKey::Down);
    }
    assert_eq!(app.panels.settings.focused_choice, Some(stop));
}

fn customize(action: &str) -> (&'static str, Value) {
    ("theme.app.customize", json!({"id": "kitty", "action": action}))
}

#[test]
fn enter_opens_panel() {
    let mut app = focused(kitty());
    press(&mut app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.pop, Some(app_key("kitty")));
    assert_eq!(app.panels.settings.focused_choice, Some(0));
    press(&mut app, SettingsKey::Cancel);
    assert_eq!((app.panels.settings.pop, app.panels.settings.focused_choice), (None, None));
    assert!(app.panels.settings.open);
    assert_eq!(app.panels.settings.focus, SettingsFocus::Controls);
}

#[test]
fn arrows_wrap_stops() {
    let mut app = focused(kitty());
    stop(&mut app, 0);
    press(&mut app, SettingsKey::Up);
    assert_eq!(app.panels.settings.focused_choice, Some(3));
    press(&mut app, SettingsKey::Down);
    assert_eq!(app.panels.settings.focused_choice, Some(0));
}

#[test]
fn profile_stop_opens_menu() {
    let mut app = focused(kitty());
    stop(&mut app, 0);
    press(&mut app, SettingsKey::Activate);
    assert_eq!(app.panels.settings.pop, Some(token_key(PROFILE, 0, TokenSide::Body)));
    assert!(app.panels.settings.focused_choice.is_some());
    press(&mut app, SettingsKey::Cancel);
    assert_eq!(app.panels.settings.pop, None);
}

#[test]
fn actions_run_by_keyboard() {
    let customized = AppThemeStatus { customized: true, ..kitty() };
    let changed = AppThemeStatus {
        state: "changed".into(),
        can_disable: false,
        can_reconnect: true,
        ..kitty()
    };
    let conflict = AppThemeStatus {
        state: "conflict".into(),
        enabled: false,
        can_disable: false,
        can_adopt: true,
        can_disconnect: false,
        template_path: String::new(),
        ..kitty()
    };
    let refresh = ("theme.app.set", json!({"id": "kitty", "enabled": true, "adopt": false}));
    let adopt = ("theme.app.set", json!({"id": "kitty", "enabled": true, "adopt": true}));
    for (status, at, call) in [
        (kitty(), 1, refresh),
        (kitty(), 2, customize("create-template")),
        (kitty(), 3, customize("disconnect")),
        (customized.clone(), 3, customize("reset-template")),
        (customized.clone(), 4, customize("disconnect")),
        (changed.clone(), 1, customize("create-template")),
        (changed.clone(), 2, customize("disconnect")),
        (changed, 3, customize("reconnect")),
        (conflict, 1, adopt),
    ] {
        let mut app = focused(status);
        stop(&mut app, at);
        press(&mut app, SettingsKey::Activate);
        let calls = drain_calls(&app);
        let (method, params) = call;
        assert!(
            calls.iter().any(|(sent, args)| sent == method && *args == params),
            "{at} {calls:?}"
        );
    }
}

#[test]
fn copy_goes_to_clipboard() {
    let mut app = focused(AppThemeStatus { customized: true, ..kitty() });
    stop(&mut app, 2);
    assert_eq!(press(&mut app, SettingsKey::Activate), 1);
    assert!(drain_calls(&app).is_empty());
}

#[test]
fn busy_ignores_enter() {
    let mut app = focused(AppThemeStatus { state: "busy".into(), ..kitty() });
    app.panels.settings.focused_control -= 1;
    press(&mut app, SettingsKey::Activate);
    assert!(drain_calls(&app).is_empty());
    assert_eq!(app.panels.settings.pop, None);
}

#[test]
fn tab_leaves_panel() {
    let mut app = focused(kitty());
    stop(&mut app, 0);
    let before = (app.panels.settings.focus, app.panels.settings.focused_control);
    press(&mut app, SettingsKey::FocusNext { backwards: false });
    let after = (app.panels.settings.focus, app.panels.settings.focused_control);
    assert_eq!(app.panels.settings.focused_choice, None);
    assert_ne!(before, after);
}

#[test]
fn search_skips_details() {
    use skwd_config::keys::performance::GPU_PREFERENCE;
    let mut app = focused(kitty());
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchOpen));
    let _ = update(&mut app, Message::Settings(SettingsMsg::SearchInput("picker gpu".into())));
    let at = app
        .panels
        .settings
        .search_results
        .iter()
        .position(|result| {
            result.live().is_some_and(|row| row.control.path() == Some(GPU_PREFERENCE))
        })
        .expect("gpu result");
    for _ in 0..=at {
        press(&mut app, SettingsKey::Down);
    }
    press(&mut app, SettingsKey::Space);
    assert_ne!(app.panels.settings.pop, Some(app_key("kitty")));
    assert!(app.panels.settings.pop.is_some());
}
