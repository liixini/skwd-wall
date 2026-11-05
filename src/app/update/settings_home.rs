use std::collections::HashSet;

use iced::Task;
use serde_json::json;
use skwd_config::keys;

use crate::app::{App, Message};
use crate::contracts::settings::{SettingChange, Undo};
use crate::frontend::settings::SettingsFocus;

use super::settings::{
    clamp_keyboard_control, open_settings_search_result, set_settings_tab, settings_input,
    settings_pick, settings_place, settings_toggle, store_keybind,
};
use super::settings_layouts::put_back;

pub(super) fn open_place(
    app: &mut App,
    place: &'static str,
    setting: Option<&'static str>,
) -> Task<Message> {
    let target = setting.and_then(|path| {
        settings_place(app, place).sections.iter().enumerate().find_map(|(section, candidate)| {
            let row = candidate.rows().position(|row| row.control.path() == Some(path))?;
            Some((section, row))
        })
    });
    let Some((section, row)) = target else {
        app.panels.settings.focus = SettingsFocus::Index;
        return set_settings_tab(app, place);
    };
    open_settings_search_result(app, place.to_string(), section, row)
}

fn undo(app: &mut App, change: &SettingChange) -> Task<Message> {
    let path = change.path.as_str();
    let task = if put_back(app, |write| write.key == path) {
        Task::none()
    } else {
        reverse(app, path, &change.undo)
    };
    put_back(app, |write| write.cause == path);
    task
}

fn reverse(app: &mut App, path: &str, undo: &Undo) -> Task<Message> {
    match undo {
        Undo::Toggle(value) => settings_toggle(app, path, *value),
        Undo::Pick(value) => settings_pick(app, path, value),
        Undo::Input { key, raw } => {
            let task = settings_input(app, key, raw);
            super::settings_policy::flush_staged(app);
            task
        }
        Undo::Binding => {
            app.panels.settings.keys_moved = None;
            app.panels.settings.keys_same = None;
            store_keybind(app, path, "");
            Task::none()
        }
        Undo::Forget => {
            app.config.remove_key(path);
            app.config.persist();
            app.apply_layout();
            app.init_settings_inputs();
            Task::none()
        }
        Undo::Clear => {
            app.config.save_key(path, json!([]));
            if path == keys::library::PROFILES {
                app.reload_library_profile();
            }
            app.rebuild_folder_options();
            app.refilter_from_start();
            Task::none()
        }
    }
}

fn settle(app: &mut App) {
    app.init_settings_inputs();
    let place = settings_place(app, &app.panels.settings.tab);
    let open = app.panels.settings.open_section;
    if place.sections.iter().any(|section| Some(section.id) == open && !section.has_body()) {
        app.panels.settings.close_sections();
    }
    clamp_keyboard_control(app);
    app.invalidate_settings();
    app.retick();
}

pub(super) fn reset(app: &mut App, change: &SettingChange) -> Task<Message> {
    let task = undo(app, change);
    settle(app);
    task
}

struct Held<'a>(&'a mut App);

impl<'a> Held<'a> {
    fn new(app: &'a mut App) -> Self {
        app.config.hold_writes();
        Self(app)
    }
}

impl std::ops::Deref for Held<'_> {
    type Target = App;

    fn deref(&self) -> &App {
        self.0
    }
}

impl std::ops::DerefMut for Held<'_> {
    fn deref_mut(&mut self) -> &mut App {
        self.0
    }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        self.0.config.release_writes();
    }
}

pub(super) fn reset_everything(app: &mut App) -> Task<Message> {
    let mut paths = HashSet::new();
    let mut held = Held::new(app);
    let mut tasks = Vec::new();
    while let Some(change) = crate::frontend::settings::fresh_changes(&held.place_source())
        .into_iter()
        .find(|change| !change.spared && paths.insert(change.path.clone()))
    {
        tasks.push(undo(&mut held, &change));
    }
    drop(held);
    settle(app);
    Task::batch(tasks)
}

#[cfg(test)]
#[path = "settings_home_tests.rs"]
mod tests;
