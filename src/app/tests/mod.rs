#![cfg(test)]

mod audio;
mod effects;
mod filters;
mod frame_clock;
mod input;
mod library_events;
mod overview;
mod playlists;
mod runtime;
mod scene;
mod scene_properties;
mod search;
mod settings;
mod tag_editor;
mod theme;
mod ui_commands;

use super::helpers::{apply_params, collect_neighbors};
use super::input::{key_message, on_event, settings_key_message};
use super::runtime::{apply_error_message, download_update};
use super::{
    App, EFFECT_NAMES, Message, Pending, begin_card_tag_edit, empty_library_hint, layout_params,
    mass_tag_suggestions, needs_list_refresh, scene_visibility, startup_filters,
    sync_library_search, theme_bar, update, version_mismatch, view,
};
use crate::app::SearchMode;
use crate::contracts::browser::DownloadUpdate;
use crate::domain::library::catalog::{Catalog, WallpaperKind};
use crate::frontend::browser::{Browser, BrowserItem, Source};
use crate::frontend::scene::layout::{MIN_HEX_R, Mode};
use crate::frontend::tagcloud::TagMsg;
use crate::infrastructure::config::Config;
use crate::infrastructure::ipc::IpcMsg;
use iced::keyboard;
use serde_json::{Value, json};
use std::time::{Duration, Instant};

static TEST_DIR_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) fn test_app() -> App {
    let dir = std::env::temp_dir().join(format!(
        "skwd-app-test-{}-{}",
        std::process::id(),
        TEST_DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let mut config = Config::from_data(json!({
        "paths": {
            "cache": dir.join("cache").to_string_lossy(),
            "wallpaper": "/wp",
            "videoWallpaper": "/vids"
        }
    }));
    config.config_path = dir.join("config.json");
    App::with_config_using(config, |_| crate::infrastructure::ipc::DaemonClient::recording())
}

fn tick_frames(app: &mut App, now: &mut Instant, frames: usize) {
    for _ in 0..frames {
        *now += Duration::from_millis(16);
        frame(app, *now, 1280.0, 720.0);
    }
}

fn frame(app: &mut App, now: Instant, width: f32, height: f32) {
    app.scene.viewport = (width, height);
    let _ = update(app, Message::Daemon(crate::infrastructure::runtime::Wake::Frame(now)));
}

fn wall(name: &str, kind: &str, hue: i64, mtime: i64) -> Value {
    json!({
        "name": name,
        "type": kind,
        "thumb": format!("/thumbs/{name}.png"),
        "hue": hue,
        "mtime": mtime
    })
}

fn respond(app: &mut App, id: u64, result: Value) {
    app.handle_ipc(IpcMsg::Response { id, result: Some(result), error: None });
}

fn reject(app: &mut App, id: u64, error: &str) {
    reject_with_code(app, id, -32000, error);
}

fn reject_with_code(app: &mut App, id: u64, code: i32, message: &str) {
    app.handle_ipc(IpcMsg::Response {
        id,
        result: None,
        error: Some(wall_proto::ErrorInfo { code, message: message.into() }),
    });
}

fn seed(app: &mut App, walls: &[Value]) {
    let _ = app.daemon.client.call("subscribe", json!({"events": ["skwd."]}));
    let list_id = app.daemon.client.call("wall.list", json!({"favourites": false}));
    app.handle_ipc(IpcMsg::Connected { list_id });
    respond(app, list_id, json!({"wallpapers": walls}));
    app.daemon.pending.clear();
    drain_calls(app);
}

fn edit_catalog(app: &mut App, edit: impl FnOnce(&mut Catalog)) {
    let mut catalog = app.library_session.library.catalog().clone();
    edit(&mut catalog);
    app.library_session.library.replace(catalog);
}

fn drain_calls(app: &App) -> Vec<(String, Value)> {
    app.daemon.client.drain_calls()
}

fn filtered_names(app: &App) -> Vec<String> {
    app.library_session
        .filtered
        .iter()
        .map(|&idx| app.library_session.library.catalog().items[idx as usize].name.clone())
        .collect()
}

pub(crate) fn browser_item(id: &str) -> BrowserItem {
    BrowserItem {
        id: id.to_string(),
        full_url: format!("https://x/{id}"),
        thumb_path: String::new(),
        title: id.to_string(),
        resolution: String::new(),
        purity: String::new(),
        file_size: 0,
        category: String::new(),
        thumb_ready: false,
        downloaded: false,
        downloading: false,
        queued: false,
        progress: 0.0,
        phase: String::new(),
        duration_secs: 0,
        downloaded_path: None,
        preview_path: None,
        thumb_failed: false,
        attribution: String::new(),
        attribution_url: String::new(),
        track_url: String::new(),
    }
}

fn dl_ev(status: &str, mut extra: serde_json::Value) -> DownloadUpdate {
    extra["id"] = json!("x");
    extra["status"] = json!(status);
    crate::infrastructure::browser::decode_download_event(&extra).unwrap()
}

fn status_request_id(app: &App) -> u64 {
    app.daemon
        .pending
        .iter()
        .find(|(_, pending)| matches!(pending, Pending::Status))
        .map(|(id, _)| *id)
        .expect("status pending")
}

fn preview_reply() -> serde_json::Value {
    serde_json::json!({
        "backend": "native",
        "colors": ["#f06e44", "#7a5b3a", "#c6b8ad", "#bdaea2", "#cfc4bb", "#8f8279"],
        "palette": {
            "primary": "#f06e44",
            "tertiary": "#7a5b3a",
            "surfaceVariant": "#c6b8ad",
            "surfaceContainer": "#bdaea2",
            "surface": "#cfc4bb",
            "background": "#b5a99f",
            "outline": "#8f8279",
            "surfaceText": "#1a1a1a",
            "primaryText": "#1a1a1a",
        },
    })
}

fn deliver_preview(app: &mut App, card: usize) {
    let backend = app.config.theme_backend();
    app.daemon.pending.insert(9, Pending::ThemePreview { card, backend });
    respond(app, 9, preview_reply());
}

fn key_press(key: keyboard::Key, shift: bool) -> iced::Event {
    iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: if shift { keyboard::Modifiers::SHIFT } else { keyboard::Modifiers::empty() },
        text: None,
        repeat: false,
    })
}

fn settle_preview(app: &mut App) {
    std::sync::Arc::make_mut(&mut app.scene.render).placeholders = false;
    app.input.last_activity = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    app.schedule_frame();
}

fn settled_preview_app() -> App {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(120.0));
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    app.panels.settings.open = true;
    app.panels.settings.tab = String::from("motion");
    app.panels.settings.section = 3;
    tick_frames(&mut app, &mut now, 600);
    app
}
