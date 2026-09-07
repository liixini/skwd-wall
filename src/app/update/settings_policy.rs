use serde_json::{Value, json};

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(super) fn stage_value(app: &mut App, path: &str, value: &Value) {
    let Some(value) = skwd_config::schema::normalize_value(path, value) else {
        return;
    };
    sync_audio(app, path, &value);
    mark_deferred_reload(app, path);
    app.config.set_key(path, value);
    sync_picker_runtime(app, path);
}

pub(super) fn save_value(app: &mut App, path: &str, value: &Value) {
    let Some(value) = skwd_config::schema::normalize_value(path, value) else {
        return;
    };
    app.config.ensure_selector_enabled();
    app.config.save_key(path, value.clone());
    sync_audio(app, path, &value);
    sync_picker_runtime(app, path);
    reload_now(app, path);
}

pub(super) fn save_audio_volume(app: &mut App, volume: u32, unmute: bool) {
    app.config.ensure_selector_enabled();
    app.config.save_key(skwd_config::keys::wallpaper::VOLUME, json!(volume));
    let mut params = json!({ "volume": volume });
    if unmute && app.config.wallpaper_mute() {
        app.config.save_key(skwd_config::keys::wallpaper::MUTE, json!(false));
        params["mute"] = json!(false);
    }
    app.daemon.client.call("wall.set_audio", params);
}

pub(in crate::app) fn flush_staged(app: &mut App) {
    app.config.ensure_selector_enabled();
    app.config.persist();
    if app.panels.settings.we_dirty {
        app.panels.settings.we_dirty = false;
        app.daemon.client.call("wall.reload_we", json!({}));
    }
    if app.panels.settings.playlist_dirty {
        app.panels.settings.playlist_dirty = false;
        app.daemon.client.call("wall.playlist.reload", json!({}));
    }
    if app.panels.settings.schedule_dirty {
        app.panels.settings.schedule_dirty = false;
        app.daemon.client.call("schedule.reload", json!({}));
    }
}

fn requires_we_reload(path: &str) -> bool {
    path.starts_with(skwd_config::keys::we_render::PREFIX)
        || path == skwd_config::keys::paths::STEAM_WE_ASSETS
}

pub(super) fn is_schedule(path: &str) -> bool {
    path.starts_with(skwd_config::keys::schedule::PREFIX)
}

pub(super) fn is_playlist(path: &str) -> bool {
    path.starts_with(skwd_config::keys::playlist::PREFIX)
}

pub(super) fn is_keybind(path: &str) -> bool {
    path.starts_with(skwd_config::keys::keybind::PREFIX)
}

fn mark_deferred_reload(app: &mut App, path: &str) {
    if requires_we_reload(path) {
        app.panels.settings.we_dirty = true;
    }
    if is_playlist(path) {
        app.panels.settings.playlist_dirty = true;
    }
    if is_schedule(path) {
        app.panels.settings.schedule_dirty = true;
    }
}

fn reload_now(app: &mut App, path: &str) {
    if requires_we_reload(path) {
        app.daemon.client.call("wall.reload_we", json!({}));
    }
    if is_schedule(path) {
        app.daemon.client.call("schedule.reload", json!({}));
    }
    if is_playlist(path) {
        app.daemon.client.call("wall.playlist.reload", json!({}));
    }
}

fn sync_audio(app: &mut App, path: &str, value: &Value) {
    if path == skwd_config::keys::wallpaper::VOLUME {
        let volume = value.as_f64().unwrap_or_default() as i64;
        app.daemon.client.call("wall.set_audio", json!({ "volume": volume.clamp(0, 100) }));
    } else if path == skwd_config::keys::wallpaper::MUTE {
        app.daemon
            .client
            .call("wall.set_audio", json!({ "mute": value.as_bool().unwrap_or(false) }));
    }
}

fn sync_picker_runtime(app: &mut App, path: &str) {
    if path == skwd_config::keys::general::LANGUAGE {
        crate::i18n::set_language(&app.config.str_path(path));
        app.panels.settings.search_results.clear();
        app.retick();
    }
    if path.starts_with("videoPreview.")
        || path == skwd_config::keys::general::MAX_FPS
        || path == skwd_config::keys::performance::BATTERY_SAVER
        || path == skwd_config::keys::performance::BATTERY_FPS
    {
        app.scene.set_preview_config(
            app.config.video_preview_enabled(),
            app.config.video_preview_delay_ms(),
            app.config.video_preview_fps(),
        );
        app.scene.touch();
        app.retick();
    }
}
