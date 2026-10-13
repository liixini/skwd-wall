#[allow(clippy::wildcard_imports)]
use super::*;

mod browser;
mod dispatcher;
mod effects;
mod filters;
mod lifecycle;
mod navigation;
mod panels;
mod picker;
mod playlists;
mod post_update;
mod scene_properties;
mod schedule;
mod settings;
pub(in crate::app) mod settings_background;
mod settings_folders;
pub(in crate::app) mod settings_policy;
mod settings_profiles;
mod tags;
pub(crate) mod theme;
mod ui_command;
pub(in crate::app) mod weather_review;

pub use dispatcher::update;
pub(crate) use dispatcher::update_inner;
pub(crate) use effects::effects_do_preview;
pub(crate) use lifecycle::drive_transition_preview;
#[cfg(test)]
pub(super) use picker::delete_wallpaper;
use ui_command::run_ui_command;
pub(crate) use ui_command::ui_state_json;
