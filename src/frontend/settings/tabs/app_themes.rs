use crate::contracts::daemon::AppThemesResult;
use crate::contracts::settings::{SettingsSource, keys};
use crate::i18n::{tr, tr_args};

use super::{ActionId, Control, Row};

const PROFILE_MISSING_MESSAGE: &str = "settings-app-themes-profile-missing";

#[cfg(test)]
mod tests;

pub(super) fn profile(cfg: &dyn SettingsSource, path: String) -> Row {
    let current = cfg.text(&path);
    let mut options = vec![(String::new(), tr("settings-app-themes-profile-follow").into())];
    options.extend(cfg.palette_presets());
    options.extend(cfg.saved_theme_names().into_iter().map(|name| (name.clone(), name)));
    if !options.iter().any(|(name, _)| name == &current) {
        options.push((current.clone(), tr_args!(PROFILE_MISSING_MESSAGE, name => &current)));
    }
    Row {
        title: tr("settings-app-themes-profile-label").into(),
        desc: tr("settings-app-themes-profile-desc").into(),
        control: Control::Dropdown { path, options, current, palettes: cfg.palette_colors() },
    }
}

pub(super) fn rows(cfg: &dyn SettingsSource, result: Option<&AppThemesResult>) -> Vec<Row> {
    let mut rows: Vec<_> = result.map_or_else(Vec::new, |result| {
        result
            .apps
            .iter()
            .enumerate()
            .map(|(index, app)| {
                let mut profile =
                    profile(cfg, format!("{}.{id}", keys::theme::APP_PROFILES, id = app.id));
                let Control::Dropdown { options, current, .. } = &profile.control else {
                    unreachable!()
                };
                let summary = options
                    .iter()
                    .find(|(name, _)| name == current)
                    .map(|(_, label)| label.clone())
                    .unwrap_or_default();
                if app.state == crate::contracts::daemon::APP_THEME_BUSY {
                    profile.desc = tr("settings-app-themes-busy").into();
                    profile.control = Control::Static;
                }
                Row {
                    title: app.name.clone(),
                    desc: tr("settings-app-themes-desc").into(),
                    control: Control::Details {
                        id: format!("app-profile:{}", app.id),
                        summary,
                        rows: vec![
                            profile,
                            Row {
                                title: app.name.clone(),
                                desc: String::new(),
                                control: Control::AppTheme { app: app.clone(), index },
                            },
                        ],
                    },
                }
            })
            .collect()
    });
    rows.push(Row {
        title: tr("settings-app-themes-scan").into(),
        desc: tr(if result.is_some() {
            "settings-app-themes-scan-desc"
        } else {
            "settings-app-themes-loading"
        })
        .into(),
        control: Control::ActionBtn {
            id: ActionId::RefreshAppThemes,
            label: tr("settings-app-themes-refresh").into(),
        },
    });
    rows
}
