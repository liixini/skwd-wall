use iced::Task;
use serde_json::json;

#[allow(clippy::wildcard_imports)]
use super::super::*;

use crate::frontend::theme_designer::ThemeMsg;

pub(super) fn update(app: &mut App, msg: ThemeMsg) -> Task<Message> {
    match msg {
        ThemeMsg::LoadCurrent => {
            app.call_tracked("theme.current", json!({}), Pending::CurrentTheme { load: true });
            Task::none()
        }
        ThemeMsg::SaveWallpaper => {
            save_wallpaper_profile(app, None);
            Task::none()
        }
        ThemeMsg::ToggleWallpaper(enabled) => {
            save_wallpaper_profile(app, Some(enabled));
            Task::none()
        }
        ThemeMsg::Variant(dark) => tdes_mutate(app, |designer| designer.set_variant(dark)),
        ThemeMsg::RoleFilter(filter) => tdes_mutate(app, |designer| designer.role_filter = filter),
        ThemeMsg::BackendMenu => {
            super::panels::toggle_bar_menu(app, crate::frontend::ui::MenuKind::Backends)
        }
        ThemeMsg::Option(key, val) => theme_option(app, key, val),
        ThemeMsg::DesignClose => {
            app.panels.theme_designer = None;
            app.retick();
            Task::none()
        }
        ThemeMsg::RoleSelect(idx) => tdes_mutate(app, |des| des.select_role(idx as usize)),
        ThemeMsg::Hue(hue) => tdes_mutate(app, |des| des.set_hue(hue)),
        ThemeMsg::SV(sat, val) => tdes_mutate(app, |des| des.set_sv(sat, val)),
        ThemeMsg::DragEnd => {
            tdes_mutate(app, crate::frontend::theme_designer::ThemeDesigner::push_recent)
        }
        ThemeMsg::HexInput(text) => tdes_mutate(app, |des| des.hex_buf = text),
        ThemeMsg::HexSubmit => tdes_mutate(app, |des| {
            if des.apply_hex() {
                des.push_recent();
            }
        }),
        ThemeMsg::Recent(hex) => tdes_mutate(app, |des| des.pick_recent(&hex)),
        ThemeMsg::Preset(name) => tdes_mutate(app, |des| {
            if let Some(cand) = crate::domain::theme::Candidate::from_preset(&name) {
                des.start_from_preset(name, cand);
            }
        }),
        ThemeMsg::SeedGen => tdes_mutate(app, |des| {
            let seed = des.candidate.colors[des.selected].clone();
            if let Some(cand) =
                crate::domain::theme::Candidate::from_seed(&seed, des.candidate.dark)
            {
                des.start_from(cand);
            }
        }),
        ThemeMsg::NameInput(text) => tdes_mutate(app, |des| des.set_name(text)),
        ThemeMsg::ResetColour => {
            tdes_mutate(app, crate::frontend::theme_designer::ThemeDesigner::reset_colour)
        }
        ThemeMsg::Reset => tdes_mutate(app, crate::frontend::theme_designer::ThemeDesigner::reset),
        ThemeMsg::SaveTheme => {
            if crate::app::helpers::theme_designer_save(app, false) {
                app.retick();
            }
            Task::none()
        }
        ThemeMsg::SaveApply => theme_save_apply(app),
        ThemeMsg::LoadSaved(name) => theme_load_saved(app, name),
        ThemeMsg::DeleteSaved(name) => theme_request_delete_saved(app, &name),
    }
}

pub(super) fn open_theme_audition(app: &mut App) -> Task<Message> {
    app.theme.audition_open = true;
    app.theme.bar_open = false;
    app.chrome.bar.menu = None;
    app.theme.audition_error = None;
    app.theme.audition_backend = app.config.theme_backend();
    app.request_theme_previews();
    app.retick();
    Task::none()
}

pub(super) fn inspect_theme_audition(app: &mut App, backend: &str) -> Task<Message> {
    if !app.theme.audition_backends.iter().any(|candidate| candidate == backend)
        || app.theme.audition_backend == backend
    {
        return Task::none();
    }
    app.theme.audition_backend = backend.to_string();
    app.theme.audition_previews.clear();
    app.theme.audition_error = None;
    app.request_theme_previews();
    app.retick();
    Task::none()
}

pub(super) fn select_theme_audition(
    app: &mut App,
    backend: &str,
    key: &str,
    value: &str,
) -> Task<Message> {
    let backend_known =
        crate::frontend::ui::THEME_BACKENDS.iter().any(|(candidate, _)| *candidate == backend);
    let key_known = matches!(
        key,
        skwd_config::keys::theme::SCHEME
            | skwd_config::keys::theme::STYLE
            | skwd_config::keys::theme::STATIC_THEME
            | skwd_config::keys::theme::WALLUST_PALETTE
            | skwd_config::keys::theme::PYWAL_SATURATE
            | skwd_config::keys::theme::NOCTALIA_SCHEME
            | skwd_config::keys::matugen::SCHEME_TYPE
    );
    let offered =
        app.theme.audition_previews.iter().any(|preview| {
            preview.backend == backend && preview.key == key && preview.value == value
        });
    if !backend_known || !key_known || !offered {
        return Task::none();
    }
    set_theme_selection(app, backend);
    app.config.save_key(key, json!(value));
    app.daemon.client.call("wall.retheme", json!({}));
    app.theme.cache.clear();
    app.invalidate_swatch();
    app.theme.shell_preview_sent = None;
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}

fn theme_save_apply(app: &mut App) -> Task<Message> {
    if !crate::app::helpers::theme_designer_save(app, true) {
        return Task::none();
    }
    if let Some(designer) = &app.panels.theme_designer {
        let palette = crate::frontend::theme::Palette::from_candidate(&designer.candidate);
        app.theme.base_palette = palette;
        app.start_fade(palette);
    }
    app.retick();
    Task::none()
}

fn theme_load_saved(app: &mut App, name: String) -> Task<Message> {
    let saved = app.config.array_values(skwd_config::keys::theme::SAVED_THEMES);
    if let Some(candidate) = crate::infrastructure::theme::find_saved(&saved, &name) {
        tdes_mutate(app, |designer| designer.load_saved(name, candidate))
    } else {
        Task::none()
    }
}

fn theme_request_delete_saved(app: &mut App, name: &str) -> Task<Message> {
    let Some(designer) = app.panels.theme_designer.as_mut() else {
        return Task::none();
    };
    if !designer.request_delete(name.to_owned()) {
        app.retick();
        return Task::none();
    }
    theme_delete_saved(app, name)
}

fn theme_delete_saved(app: &mut App, name: &str) -> Task<Message> {
    let themes = crate::infrastructure::theme::remove_saved(
        &app.config.array_values(skwd_config::keys::theme::SAVED_THEMES),
        name,
    );
    app.config.set_key(skwd_config::keys::theme::SAVED_THEMES, serde_json::Value::Array(themes));
    let selected = app.config.str_path(skwd_config::keys::theme::STATIC_THEME) == name;
    let active = selected && app.config.theme_backend() == "static";
    if selected {
        app.config.set_key(skwd_config::keys::theme::STATIC_THEME, json!("nord"));
    }
    app.config.persist();
    if active && let Some(candidate) = crate::domain::theme::Candidate::from_preset("nord") {
        let palette = crate::frontend::theme::Palette::from_candidate(&candidate);
        app.theme.base_palette = palette;
        app.start_fade(palette);
        app.daemon.client.call("wall.retheme", json!({}));
        app.invalidate_swatch();
        if let Some(designer) = app.panels.theme_designer.as_mut() {
            designer.start_from_preset(String::from("nord"), candidate);
        }
    }
    app.retick();
    Task::none()
}

pub(super) fn theme_option(app: &mut App, key: &'static str, value: &'static str) -> Task<Message> {
    if value == "toggle" {
        let current = app.config.flag_default_config(key);
        app.config.save_key(key, json!(!current));
    } else if key == skwd_config::keys::matugen::COLOR_INDEX {
        app.config.save_key(key, json!(value.parse::<u64>().unwrap_or(0)));
    } else if key == skwd_config::keys::theme::BACKEND {
        save_theme_selection(app, value);
    } else {
        app.config.save_key(key, json!(value));
    }
    if key == skwd_config::keys::theme::BACKEND {
        app.chrome.bar.menu = None;
    }
    app.daemon.client.call("wall.retheme", json!({}));
    app.theme.cache.clear();
    app.invalidate_swatch();
    app.theme.shell_preview_sent = None;
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}

pub(super) fn save_theme_selection(app: &mut App, value: &str) {
    set_theme_selection(app, value);
    app.config.persist();
}

fn set_theme_selection(app: &mut App, value: &str) {
    use skwd_config::keys::theme;

    match value {
        "static" => app.config.set_key(theme::POLICY, json!("fixed")),
        "off" => app.config.set_key(theme::POLICY, json!("off")),
        "noctalia" | "dms" => {
            app.config.set_key(theme::POLICY, json!("wallpaper"));
            app.config.set_key(theme::AUTHORITY, json!(value));
        }
        engine => {
            app.config.set_key(theme::POLICY, json!("wallpaper"));
            app.config.set_key(theme::AUTHORITY, json!("skwd"));
            app.config.set_key(theme::ENGINE, json!(engine));
        }
    }
}

fn tdes_mutate(
    app: &mut App,
    mutate: impl FnOnce(&mut crate::frontend::theme_designer::ThemeDesigner),
) -> Task<Message> {
    if let Some(designer) = app.panels.theme_designer.as_mut() {
        mutate(designer);
        designer.clear_delete_confirmation();
        app.retick();
    }
    Task::none()
}

fn save_wallpaper_profile(app: &mut App, enabled: Option<bool>) {
    let Some(designer) = app.panels.theme_designer.as_mut() else {
        return;
    };
    let Some(wallpaper) = &designer.wallpaper else {
        return;
    };
    let mut profiles = app.config.array_values(skwd_config::keys::theme::WALLPAPER_PROFILES);
    let index = profiles
        .iter()
        .position(|profile| profile["key"].as_str() == Some(&wallpaper.key))
        .unwrap_or_else(|| {
            profiles.push(json!({"key": wallpaper.key, "name": wallpaper.name}));
            profiles.len() - 1
        });
    let profile = &mut profiles[index];
    if enabled.is_none() {
        for dark in [true, false] {
            let mut candidate = designer.candidate.clone();
            candidate.set_dark(dark);
            profile[if dark { "dark" } else { "light" }] =
                crate::infrastructure::theme::encode_candidate(&candidate, true);
        }
    }
    profile["enabled"] = json!(enabled.unwrap_or(true));
    designer.profile_enabled = enabled.unwrap_or(true);
    designer.error = None;
    app.config.save_key(skwd_config::keys::theme::WALLPAPER_PROFILES, json!(profiles));
    app.theme.cache.clear();
    app.invalidate_swatch();
    app.daemon.client.call("wall.retheme", json!({}));
    app.retick();
}
