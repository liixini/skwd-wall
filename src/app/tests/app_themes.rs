use super::*;
use crate::frontend::settings::{ActionId, SettingsMsg};

fn status() -> Value {
    json!({"apps": [{"id": "kitty", "name": "Kitty", "installed": true,
        "config_found": true, "config_path": "/config/kitty/kitty.conf",
        "output_path": "/config/kitty/skwd-colors.conf", "enabled": false,
        "state": "ready", "detail": "", "can_enable": true, "can_disable": false,
        "can_adopt": false}]})
}

#[test]
fn switch_waits_for_service_confirmation_and_prevents_duplicate_requests() {
    let mut app = test_app();
    app.on_result(Pending::AppThemes, &status());
    let _ = drain_calls(&app);
    let action = Message::Settings(SettingsMsg::Run(ActionId::SetAppTheme(0, true)));
    let _ = update(&mut app, action.clone());
    let _ = update(&mut app, action);
    let calls = drain_calls(&app);
    let changes: Vec<_> = calls.iter().filter(|(method, _)| method == "theme.app.set").collect();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].1, json!({"id": "kitty", "enabled": true, "adopt": false}));
    let row = &app.daemon.app_themes.as_ref().unwrap().apps[0];
    assert!(!row.enabled);
    assert_eq!(row.state, "busy");
    let mut confirmed = status();
    confirmed["apps"][0]["enabled"] = json!(true);
    confirmed["apps"][0]["state"] = json!("reload-sent");
    app.on_result(Pending::AppThemeSet, &confirmed);
    assert!(app.daemon.app_themes.as_ref().unwrap().apps[0].enabled);
}

#[test]
fn failed_setup_refreshes_status_and_unavailable_apps_cannot_be_enabled() {
    let mut app = test_app();
    let mut unavailable = status();
    unavailable["apps"][0]["can_enable"] = json!(false);
    app.on_result(Pending::AppThemes, &unavailable);
    let _ = drain_calls(&app);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::SetAppTheme(0, true))));
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "theme.app.set"));
    app.rpc_error(Pending::AppThemeSet, "config changed".into());
    assert!(drain_calls(&app).iter().any(|(method, _)| method == "theme.apps"));
}

#[test]
fn app_search_targets_the_runtime_row_and_migration_is_explicit() {
    let mut app = test_app();
    let mut result = status();
    result["apps"][0]["can_enable"] = json!(false);
    result["apps"][0]["can_adopt"] = json!(true);
    result["apps"][0]["state"] = json!("conflict");
    app.on_result(Pending::AppThemes, &result);
    let found = crate::frontend::settings::search_settings("Kitty", app.place_source());
    assert_eq!(found[0].tab, "colours");
    assert_eq!(found[0].row, 0);
    let _ = drain_calls(&app);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::AdoptAppTheme(0))));
    assert!(
        drain_calls(&app)
            .iter()
            .any(|(method, params)| method == "theme.app.set" && params["adopt"] == true)
    );
}

#[test]
fn refresh_uses_the_managed_reload_without_toggling_the_app_off() {
    let mut app = test_app();
    let mut result = status();
    result["apps"][0]["enabled"] = json!(true);
    result["apps"][0]["can_enable"] = json!(false);
    result["apps"][0]["can_disable"] = json!(true);
    result["apps"][0]["state"] = json!("reload-needed");
    app.on_result(Pending::AppThemes, &result);
    let _ = drain_calls(&app);
    let _ = update(&mut app, Message::Settings(SettingsMsg::Run(ActionId::RefreshAppTheme(0))));
    assert!(drain_calls(&app).iter().any(|(method, params)| method == "theme.app.set"
        && params["enabled"] == true
        && params["adopt"] == false));
    assert!(app.daemon.app_themes.as_ref().unwrap().apps[0].enabled);
    assert_eq!(app.daemon.app_themes.as_ref().unwrap().apps[0].state, "busy");
}

#[test]
fn changed_themes_offer_explicit_recovery_without_using_the_restore_switch() {
    let mut app = test_app();
    let mut result = status();
    result["apps"][0]["enabled"] = json!(true);
    result["apps"][0]["state"] = json!("changed");
    result["apps"][0]["can_enable"] = json!(false);
    result["apps"][0]["can_disable"] = json!(false);
    result["apps"][0]["can_disconnect"] = json!(true);
    result["apps"][0]["can_reconnect"] = json!(true);
    result["apps"][0]["template_path"] = json!("/config/skwd-wall-v2/app-themes/kitty.template");
    app.on_result(Pending::AppThemes, &result);
    let _ = drain_calls(&app);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::CustomizeAppTheme(0, "disconnect"))),
    );
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| method == "theme.app.customize"
        && params == &json!({"id":"kitty", "action":"disconnect"})));
    assert!(!calls.iter().any(|(method, _)| method == "theme.app.set"));
    result["apps"][0]["state"] = json!("disconnected");
    result["apps"][0]["enabled"] = json!(false);
    result["apps"][0]["can_disconnect"] = json!(false);
    app.daemon.pending.clear();
    app.on_result(Pending::AppThemeSet, &result);
    let _ = drain_calls(&app);
    let _ = update(
        &mut app,
        Message::Settings(SettingsMsg::Run(ActionId::CustomizeAppTheme(0, "reconnect"))),
    );
    assert!(
        drain_calls(&app)
            .iter()
            .any(|(method, params)| method == "theme.app.customize"
                && params["action"] == "reconnect")
    );
}
