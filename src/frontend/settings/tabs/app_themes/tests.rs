use super::{profile, rows};
use crate::contracts::daemon::{AppThemeStatus, AppThemesResult};
use crate::frontend::settings::Control;
use crate::frontend::settings::test_source::FakeSettingsSource;
use crate::i18n::tr;

fn app(id: &str) -> AppThemeStatus {
    AppThemeStatus {
        id: id.into(),
        name: id.into(),
        installed: true,
        config_found: true,
        config_path: String::new(),
        output_path: String::new(),
        enabled: true,
        state: "applied".into(),
        detail: String::new(),
        can_enable: false,
        can_disable: true,
        can_adopt: false,
        template_path: String::new(),
        customized: false,
        can_disconnect: true,
        can_reconnect: false,
    }
}

#[test]
fn profile_choices() {
    let cfg = FakeSettingsSource::default().with_saved_theme("My colours");
    let row = profile(&cfg, "theme.appProfiles.waybar".into());
    let Control::Dropdown { options, current, .. } = row.control else { panic!() };
    assert_eq!(current, "");
    assert_eq!(
        options,
        [
            (String::new(), tr("settings-app-themes-profile-follow").into()),
            ("nord".into(), "Nord".into()),
            ("dracula".into(), "Dracula".into()),
            ("My colours".into(), "My colours".into()),
        ]
    );
}

#[test]
fn separate_app_profiles() {
    let cfg = FakeSettingsSource::default().with_text("theme.appProfiles.waybar", "nord");
    let apps = AppThemesResult { apps: vec![app("waybar"), app("kitty")] };
    let result = rows(&cfg, Some(&apps));
    for (index, (id, choice, label)) in
        [("waybar", "nord", "Nord"), ("kitty", "", tr("settings-app-themes-profile-follow"))]
            .into_iter()
            .enumerate()
    {
        let Control::Details { summary, rows, .. } = &result[index].control else { panic!() };
        assert_eq!(summary, label);
        assert!(matches!(&rows[0].control, Control::Dropdown { path, current, .. }
            if path == &format!("theme.appProfiles.{id}") && current == choice));
        assert!(matches!(&rows[1].control, Control::AppTheme { app, index: app_index }
            if app.id == id && *app_index == index));
    }
}

#[test]
fn missing_profile_visible() {
    let cfg = FakeSettingsSource::default().with_text("integrations.0.profile", "Removed profile");
    let row = profile(&cfg, "integrations.0.profile".into());
    let Control::Dropdown { options, current, .. } = row.control else { panic!() };
    assert_eq!(current, "Removed profile");
    assert!(
        options
            .iter()
            .any(|(key, label)| key == &current && label == "Unavailable: Removed profile")
    );
}

#[test]
fn busy_profile_waits() {
    let cfg = FakeSettingsSource::default();
    let mut status = app("waybar");
    status.state = "busy".into();
    let result = rows(&cfg, Some(&AppThemesResult { apps: vec![status] }));
    let Control::Details { rows, .. } = &result[0].control else { panic!() };
    assert!(matches!(rows[0].control, Control::Static));
}
