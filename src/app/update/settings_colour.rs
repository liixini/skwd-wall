use iced::Task;
use serde_json::json;

use crate::app::{App, Message};
use crate::frontend::settings::colour;

pub(super) fn picker(app: &mut App, path: &str) -> Task<Message> {
    let motion = app.motion_profile();
    app.panels.settings.toggle_bar(colour::picker_id(path), motion);
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn change(app: &mut App, path: &str, hex: &str) -> Task<Message> {
    if colour::normalized(hex).is_none() {
        return Task::none();
    }
    app.config.set_key(path, json!(hex));
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn commit(app: &mut App, path: &str) -> Task<Message> {
    let hex = app.config.str_path(path);
    super::settings_policy::save_value(app, path, &json!(hex));
    app.invalidate_settings();
    app.retick();
    Task::none()
}
