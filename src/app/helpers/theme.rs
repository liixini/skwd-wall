use serde_json::json;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(crate) fn local_static_palette(
    app: &App,
    name: &str,
) -> Option<crate::frontend::theme::Palette> {
    use crate::domain::theme::Candidate;
    use crate::infrastructure::theme::find_saved;

    let saved = app.config.array_values(skwd_config::keys::theme::SAVED_THEMES);
    let candidate = match find_saved(&saved, name) {
        Some(candidate) => candidate,
        None => Candidate::from_preset(name)?,
    };
    Some(crate::frontend::theme::Palette::from_candidate(&candidate))
}

pub(crate) fn theme_designer_open(app: &mut App) {
    use crate::domain::theme::Candidate;
    use crate::infrastructure::theme::find_saved;

    let saved = app.config.array_values(skwd_config::keys::theme::SAVED_THEMES);
    let current = app.config.str_path(skwd_config::keys::theme::STATIC_THEME);
    let designer = if let Some(candidate) = find_saved(&saved, &current) {
        crate::frontend::theme_designer::ThemeDesigner::new(candidate, current)
    } else if let Some(preset) = Candidate::from_preset(&current) {
        crate::frontend::theme_designer::ThemeDesigner::new_from_preset(preset, current)
    } else {
        crate::frontend::theme_designer::ThemeDesigner::new_from_preset(
            Candidate::from_preset("nord").unwrap_or_default(),
            String::from("nord"),
        )
    };
    app.panels.theme_designer = Some(designer);
    app.call_tracked(
        "theme.current",
        json!({}),
        Pending::CurrentTheme { load: app.config.theme_backend() != "static" },
    );
    app.retick();
}

pub(crate) fn theme_designer_save(app: &mut App, apply: bool) -> bool {
    use crate::infrastructure::theme::upsert_saved;

    let Some(designer) = &app.panels.theme_designer else {
        return false;
    };
    let name = designer.name_buf.trim().to_string();
    if name.is_empty() {
        return false;
    }
    let candidate = designer.candidate.clone();
    let themes = upsert_saved(
        &app.config.array_values(skwd_config::keys::theme::SAVED_THEMES),
        &name,
        &candidate,
    );
    app.config.set_key(skwd_config::keys::theme::SAVED_THEMES, serde_json::Value::Array(themes));
    if apply {
        app.config.set_key(skwd_config::keys::theme::POLICY, json!("fixed"));
        app.config.set_key(skwd_config::keys::theme::STATIC_THEME, json!(name));
        app.config.set_key(
            skwd_config::keys::theme::MODE,
            json!(if candidate.dark { "dark" } else { "light" }),
        );
    }
    app.config.persist();
    if apply {
        app.daemon.client.call("wall.retheme", json!({}));
        app.invalidate_swatch();
    }
    if let Some(designer) = app.panels.theme_designer.as_mut() {
        designer.mark_saved(name);
    }
    true
}
