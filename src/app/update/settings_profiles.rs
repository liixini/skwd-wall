use std::path::Path;

use iced::Task;
use serde_json::{Value, json};

use crate::app::{App, Message};
use crate::contracts::settings::keys::library::profile::{ID, NAME, WORKSHOP};
use crate::contracts::settings::{
    EVERYTHING_PROFILE, ProfileState, SettingsSource, folder_tag, keys,
};
use crate::frontend::settings::{STATES, WORKSHOP_TAG, segment_key, state_position};
use crate::i18n::tr;

const FALLBACK_ID: &str = "profile";
const ID_SEPARATOR: &str = "-";
const FIRST_SUFFIX: u32 = 2;

pub(super) fn add_profile(app: &mut App) -> Task<Message> {
    let mut profiles = app.config.array_values(keys::library::PROFILES);
    let name = tr("settings-profiles-new-name");
    let id = unique_id(&slug(name), &profiles);
    profiles.push(json!({ ID: id, NAME: name }));
    let index = profiles.len() - 1;
    save_profiles(app, profiles);
    app.init_settings_inputs();
    app.panels.settings.select_profile(index + 1);
    app.retick();
    iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&profile_path(
        index, NAME,
    )))
}

pub(super) fn remove_profile(app: &mut App, index: usize) {
    let mut profiles = app.config.array_values(keys::library::PROFILES);
    if index >= profiles.len() {
        return;
    }
    let removed = profiles.remove(index);
    let removed_id = removed[ID].as_str();
    if removed_id == Some(app.config.str_path(keys::library::PROFILE).as_str()) {
        app.config.save_key(keys::library::PROFILE, json!(EVERYTHING_PROFILE));
    }
    if removed_id == app.library_session.profile_override.as_deref() {
        app.library_session.profile_override = None;
    }
    let remaining = profiles.len();
    save_profiles(app, profiles);
    app.init_settings_inputs();
    app.panels.settings.select_profile(app.panels.settings.profile.min(remaining));
}

fn set_folder_state(app: &mut App, index: usize, tag: u64, state: ProfileState) {
    let folders = app.library_session.all_folders.clone();
    let Some(position) = folders.iter().position(|folder| folder_tag(folder) == tag) else {
        return;
    };
    let folder = folders[position].clone();
    let (states, _) = app.config.profile_states(index, &folders);
    slide(app, index, &tag.to_string(), states[position], state);
    edit_profile(app, index, |entry| {
        for list in STATES.map(|(state, _)| state.key()) {
            let kept: Vec<Value> = entry[list]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|path| {
                    path.as_str().is_some_and(|path| !Path::new(path).starts_with(&folder))
                })
                .cloned()
                .collect();
            entry[list] = Value::Array(kept);
        }
        if let Some(paths) = entry[state.key()].as_array_mut() {
            paths.push(json!(folder));
        }
    });
}

pub(super) fn pick(app: &mut App, path: &str, value: &str) -> bool {
    let Some(rest) =
        path.strip_prefix(keys::library::PROFILES).and_then(|rest| rest.strip_prefix('.'))
    else {
        return false;
    };
    if let Some((index, field, state)) = parse_pick(rest, value) {
        if field == WORKSHOP {
            set_workshop_state(app, index, state);
        } else if let Some(tag) = folder_tag_of(field) {
            set_folder_state(app, index, tag, state);
        }
    }
    true
}

fn parse_pick<'a>(rest: &'a str, value: &str) -> Option<(usize, &'a str, ProfileState)> {
    let (index, field) = rest.split_once('.')?;
    let index = index.parse::<usize>().ok()?;
    let (state, _) =
        crate::frontend::settings::STATES.iter().find(|(state, _)| state.key() == value)?;
    Some((index, field, *state))
}

fn folder_tag_of(field: &str) -> Option<u64> {
    field
        .strip_prefix(crate::frontend::settings::FOLDER_SEGMENT)
        .and_then(|rest| rest.strip_prefix('.'))
        .and_then(|tag| tag.parse::<u64>().ok())
}

pub(super) fn copy_command(app: &App, index: usize, helm: bool) -> Task<Message> {
    let profiles = app.config.array_values(keys::library::PROFILES);
    let Some(id) = profiles.get(index).and_then(|entry| entry[ID].as_str()) else {
        return Task::none();
    };
    let [helm_command, flag_command] = crate::frontend::settings::profile_commands(id);
    iced::clipboard::write(if helm { helm_command } else { flag_command })
}

fn set_workshop_state(app: &mut App, index: usize, state: ProfileState) {
    let (_, workshop) = app.config.profile_states(index, &app.library_session.all_folders);
    slide(app, index, WORKSHOP_TAG, workshop, state);
    edit_profile(app, index, |entry| entry[WORKSHOP] = json!(state.key()));
}

fn profile_path(index: usize, field: &str) -> String {
    format!("{}.{index}.{field}", keys::library::PROFILES)
}

fn slide(app: &mut App, index: usize, tag: &str, from: ProfileState, to: ProfileState) {
    let profiles = app.config.array_values(keys::library::PROFILES);
    let Some(id) = profiles.get(index).and_then(|entry| entry[ID].as_str()) else {
        return;
    };
    let key = segment_key(id, tag);
    let motion = app.motion_profile();
    app.panels.settings.slide_segment(key, state_position(from), state_position(to), motion);
    app.retick();
}

fn edit_profile(app: &mut App, index: usize, edit: impl FnOnce(&mut Value)) {
    let mut profiles = app.config.array_values(keys::library::PROFILES);
    let Some(entry) = profiles.get_mut(index).filter(|entry| entry.is_object()) else {
        return;
    };
    edit(entry);
    save_profiles(app, profiles);
}

fn save_profiles(app: &mut App, profiles: Vec<Value>) {
    app.config.save_key(keys::library::PROFILES, Value::Array(profiles));
    app.reload_library_profile();
}

fn slug(name: &str) -> String {
    let words: Vec<String> = name
        .split(|letter: char| !letter.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    if words.is_empty() { String::from(FALLBACK_ID) } else { words.join(ID_SEPARATOR) }
}

fn unique_id(base: &str, profiles: &[Value]) -> String {
    let taken = |id: &str| id == EVERYTHING_PROFILE || profiles.iter().any(|entry| entry[ID] == id);
    let mut id = base.to_owned();
    for count in FIRST_SUFFIX.. {
        if !taken(&id) {
            break;
        }
        id = format!("{base}{ID_SEPARATOR}{count}");
    }
    id
}
