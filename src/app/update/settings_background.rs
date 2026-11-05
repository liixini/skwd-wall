use iced::Task;
use serde_json::json;

use crate::app::{App, Message};
use crate::frontend::animation::MotionTier;
use crate::frontend::settings::background::BackgroundControl;
use skwd_config::keys;

pub(super) fn animate_placement(app: &mut App, path: &str, old: &str, new: &str) {
    let visible = |mode: &str| if matches!(mode, "center" | "fit") { 1.0 } else { 0.0 };
    let motion = app.motion_profile();
    app.panels
        .settings
        .bar_reveals
        .entry(BackgroundControl::reveal_id(path))
        .or_insert_with(|| motion.tween(visible(old), MotionTier::Fast))
        .retarget(visible(new));
}

fn paths(output: &str) -> (String, String) {
    if output.is_empty() {
        (keys::display::BACKGROUND_MODE.to_string(), keys::display::FILL_COLOR.to_string())
    } else {
        (
            format!("{}.{output}", keys::display::BACKGROUND_MODES),
            format!("{}.{output}", keys::display::BACKGROUND_COLORS),
        )
    }
}

pub(super) fn picker(app: &mut App, output: &str) -> Task<Message> {
    let background = BackgroundControl::new(&app.config, output);
    let _ = mode(app, output, "color");
    let motion = app.motion_profile();
    app.panels.settings.toggle_bar(background.picker_id(), motion);
    sync_monitors(app);
    app.retick();
    Task::none()
}

pub(super) fn mode(app: &mut App, output: &str, value: &str) -> Task<Message> {
    if !matches!(value, "color" | "blur" | "inherit") || output.is_empty() && value == "inherit" {
        return Task::none();
    }
    let previous = BackgroundControl::new(&app.config, output);
    let (mode_path, color_path) = paths(output);
    if !output.is_empty() && app.config.str_path(&color_path).is_empty() && value != "inherit" {
        app.config.set_key(&color_path, json!(previous.color));
    }
    app.config.set_key(&mode_path, json!(value));
    if value != "color"
        && let Some(reveal) = app.panels.settings.bar_reveals.get_mut(&previous.picker_id())
    {
        reveal.retarget(0.0);
    }
    commit(app, output)
}

pub(super) fn color(app: &mut App, output: &str, color: &str) -> Task<Message> {
    if crate::domain::theme::hex_to_hsv(color).is_none() {
        return Task::none();
    }
    let (mode_path, color_path) = paths(output);
    app.config.set_key(&mode_path, json!("color"));
    app.config.set_key(&color_path, json!(color));
    app.invalidate_settings();
    sync_monitors(app);
    app.retick();
    Task::none()
}

pub(super) fn commit(app: &mut App, output: &str) -> Task<Message> {
    app.config.persist();
    for status in &app.daemon.output_statuses {
        if !output.is_empty() && status.name != output {
            continue;
        }
        let mut params = json!({"type": status.kind.as_key(), "output": status.target(),
            "mute": status.mute, "volume": status.volume, "notify": false, "no_transition": true});
        if status.kind == crate::contracts::media::MediaKind::WallpaperEngine {
            params["we_id"] = json!(status.we_id);
        } else {
            if status.path.is_empty() {
                continue;
            }
            params["path"] = json!(status.path);
        }
        app.daemon.client.call("wall.apply", params);
    }
    app.invalidate_settings();
    sync_monitors(app);
    app.retick();
    Task::none()
}

pub(in crate::app) fn sync_monitors(app: &mut App) {
    for (id, reveal) in &mut app.panels.settings.bar_reveals {
        let Some(path) = id.strip_prefix(BackgroundControl::REVEAL_PREFIX) else {
            continue;
        };
        let selected = app.config.str_path(path);
        let selected = if selected.is_empty() {
            app.config.str_path(keys::display::FILL_MODE)
        } else {
            selected
        };
        reveal.retarget(if matches!(selected.as_str(), "center" | "fit") { 1.0 } else { 0.0 });
    }
    if let Some(effects) = app.panels.effects.as_mut() {
        for monitor in effects.monitors_mut() {
            monitor.background = BackgroundControl::new(&app.config, &monitor.name);
            let path = format!("{}.{}", keys::display::FILL_MODES, monitor.name);
            let selected = app.config.str_path(&path);
            let selected = if selected.is_empty() {
                app.config.str_path(keys::display::FILL_MODE)
            } else {
                selected
            };
            monitor.background_reveal =
                app.panels.settings.bar_reveals.get(&BackgroundControl::reveal_id(&path)).map_or(
                    if matches!(selected.as_str(), "center" | "fit") { 1.0 } else { 0.0 },
                    |value| value.x,
                );
            monitor.background_picker = app
                .panels
                .settings
                .bar_reveals
                .get(&monitor.background.picker_id())
                .map_or(0.0, |value| value.x);
        }
    }
}
