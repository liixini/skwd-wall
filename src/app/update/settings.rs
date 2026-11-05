use iced::Task;
use log::info;
use serde_json::{Value, json};

use crate::app::state::{SettingsInputEdit, WeatherQueue};
use crate::app::{App, EFFECT_NAMES, Message, Pending, WE_THUMBNAILS_TASK};
use crate::contracts::daemon::AppThemeStatus;
use crate::domain::library::filter::resolution_parts;
use crate::frontend::settings::{ActionId, Control, SettingsFocus, SettingsKey, SettingsMsg};

pub(super) fn update(app: &mut App, msg: SettingsMsg) -> Task<Message> {
    match msg {
        SettingsMsg::Input(key, raw) => {
            if app.panels.settings.input_edit.as_ref().is_some_and(|edit| edit.key != key) {
                commit_settings_input_edit(app);
            }
            begin_settings_input_edit(app, &key);
            settings_input(app, &key, &raw)
        }
        SettingsMsg::ResolutionInput(key, width, raw) => {
            if !raw.bytes().all(|byte| byte.is_ascii_digit()) || raw.len() > 9 {
                return Task::none();
            }
            if app.panels.settings.input_edit.as_ref().is_some_and(|edit| edit.key != key) {
                commit_settings_input_edit(app);
            }
            begin_settings_input_edit(app, &key);
            app.panels.settings.resolution_height = !width;
            let current = app
                .panels
                .settings
                .inputs
                .get(&key)
                .cloned()
                .unwrap_or_else(|| app.config.str_path(&key));
            let (current_width, current_height) = resolution_parts(&current);
            let value = if width {
                format!("{raw}x{current_height}")
            } else {
                format!("{current_width}x{raw}")
            };
            settings_input(app, &key, if value == "x" { "" } else { &value })
        }
        SettingsMsg::ResolutionLimit(key, enabled) => {
            let Some(base) = key.strip_suffix(".to") else { return Task::none() };
            commit_settings_input_edit(app);
            let value =
                if enabled { app.config.str_path(&format!("{base}.from")) } else { String::new() };
            let task = settings_input(app, &key, &value);
            super::settings_policy::flush_staged(app);
            task
        }
        SettingsMsg::Toggle(path, value) => {
            close_token_edit(app);
            settings_toggle(app, &path, value)
        }
        SettingsMsg::NumberStep(key, step) => {
            if !step.is_finite() {
                return Task::none();
            }
            let current = app
                .panels
                .settings
                .inputs
                .get(&key)
                .and_then(|raw| raw.parse::<f64>().ok())
                .filter(|number| number.is_finite())
                .unwrap_or_else(|| app.config.num_path(&key));
            let previous = app
                .panels
                .settings
                .inputs
                .get(&key)
                .map(String::as_str)
                .filter(|raw| raw.parse::<f64>().ok().is_some_and(f64::is_finite));
            let Some(raw) = crate::frontend::settings::step_number(&key, current, step)
                .and_then(|next| crate::frontend::settings::number_step_text(next, step, previous))
            else {
                return Task::none();
            };
            let input = settings_input(app, &key, &raw);
            let commit = settings_commit(app);
            Task::batch([input, commit])
        }
        SettingsMsg::Commit => {
            app.panels.settings.edit = None;
            settings_commit(app)
        }
        SettingsMsg::Pick(path, value) => {
            close_token_edit(app);
            app.panels.settings.pop = None;
            settings_pick(app, &path, &value)
        }
        SettingsMsg::BackgroundPicker(output) => super::settings_background::picker(app, &output),
        SettingsMsg::BackgroundMode(output, mode) => {
            super::settings_background::mode(app, &output, &mode)
        }
        SettingsMsg::BackgroundColor(output, color) => {
            super::settings_background::color(app, &output, &color)
        }
        SettingsMsg::BackgroundCommit(output) => super::settings_background::commit(app, &output),
        SettingsMsg::ColourPicker(path) => super::settings_colour::picker(app, &path),
        SettingsMsg::Colour(path, hex) => super::settings_colour::change(app, &path, &hex),
        SettingsMsg::ColourCommit(path) => super::settings_colour::commit(app, &path),
        SettingsMsg::ToggleSection(section) => toggle_settings_section(app, section),
        SettingsMsg::OpenPop(key) => {
            close_token_edit(app);
            commit_settings_input_edit(app);
            app.panels.settings.keybind_capture = None;
            let opening = app.panels.settings.pop != Some(key);
            app.panels.settings.pop = opening.then_some(key);
            app.panels.settings.focused_choice = opening
                .then(|| current_keyboard_control(app))
                .flatten()
                .filter(|control| {
                    app.panels.settings.focus == SettingsFocus::Controls
                        && control_token(app, control) == Some(key)
                })
                .map(|control| initial_choice(&control));
            app.retick();
            Task::none()
        }
        SettingsMsg::ClosePop => {
            commit_settings_input_edit(app);
            app.panels.settings.pop = None;
            app.panels.settings.edit = None;
            app.panels.settings.focused_choice = None;
            app.retick();
            unfocus_settings_input()
        }
        SettingsMsg::EditToken(token, key) => {
            commit_settings_input_edit(app);
            app.panels.settings.pop = None;
            app.panels.settings.edit = Some(token);
            begin_settings_input_edit(app, &key);
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&key))
        }
        SettingsMsg::EnterLayoutStudio => {
            if !crate::frontend::settings::is_picker_layout_section(
                app.panels.settings.open_section,
            ) {
                return Task::none();
            }
            commit_settings_input_edit(app);
            app.panels.settings.layout_studio = true;
            app.panels.settings.previous = None;
            app.panels.settings.pending = None;
            app.panels.settings.section_anim.snap(1.0);
            app.retick();
            Task::none()
        }
        SettingsMsg::Layout(message) => super::settings_layouts::update(app, message),
        SettingsMsg::LeaveLayoutStudio => {
            app.panels.settings.layout_studio = false;
            app.retick();
            Task::none()
        }
        SettingsMsg::FocusControl(control) => {
            commit_settings_input_edit(app);
            close_settings_search(app);
            app.panels.settings.focus = SettingsFocus::Controls;
            app.panels.settings.focused_control = control;
            app.panels.settings.focused_choice = None;
            clamp_keyboard_control(app);
            app.retick();
            unfocus_settings_input()
        }
        SettingsMsg::ToggleDetails(id, control) => {
            commit_settings_input_edit(app);
            close_settings_search(app);
            app.panels.settings.focus = SettingsFocus::Controls;
            app.panels.settings.focused_control = control;
            app.panels.settings.focused_choice = None;
            toggle_settings_details(app, &id);
            unfocus_settings_input()
        }
        SettingsMsg::ToggleBar(id, control) => {
            commit_settings_input_edit(app);
            close_settings_search(app);
            app.panels.settings.focus = SettingsFocus::Controls;
            app.panels.settings.focused_control = control;
            app.panels.settings.focused_choice = None;
            let motion = app.motion_profile();
            app.panels.settings.toggle_bar(id, motion);
            app.retick();
            unfocus_settings_input()
        }
        SettingsMsg::SelectProfile(position) => {
            commit_settings_input_edit(app);
            app.panels.settings.select_profile(position);
            app.retick();
            unfocus_settings_input()
        }
        SettingsMsg::ProfileFilter(query) => {
            use crate::frontend::settings::QUERY_FOLD;
            let settings = &mut app.panels.settings;
            settings.expanded_details.retain(|key| !key.starts_with(QUERY_FOLD));
            settings.bar_reveals.retain(|key, _| !key.starts_with(QUERY_FOLD));
            settings
                .inputs
                .insert(crate::frontend::settings::FILTER_KEY.to_string(), query.clone());
            settings.profile_filter = query;
            app.invalidate_settings();
            Task::none()
        }
        SettingsMsg::ToggleFold(id, open_by_default) => {
            let motion = app.motion_profile();
            app.panels.settings.toggle_fold(id, open_by_default, motion);
            app.retick();
            Task::none()
        }
        SettingsMsg::SearchOpen => open_settings_search(app),
        SettingsMsg::SearchClose => search_cancel(app),
        SettingsMsg::SearchInput(query) => {
            app.panels.settings.search_query = query;
            app.panels.settings.search_at = None;
            disarm(app);
            refresh_settings_search(app);
            app.retick();
            Task::none()
        }
        SettingsMsg::OpenSearchResult(tab, section, row) => {
            open_settings_search_result(app, tab, section, row)
        }
        SettingsMsg::SelectTab(tab) => {
            app.panels.settings.focus = SettingsFocus::Index;
            set_settings_tab(app, &tab)
        }
        SettingsMsg::Key(key) => settings_key(app, key),
        SettingsMsg::Run(id) => {
            close_token_edit(app);
            app.panels.settings.pop = None;
            app.panels.settings.keybind_capture = None;
            settings_run(app, id)
        }
        SettingsMsg::KeybindCapture(path, slot) => open_keybind_capture(app, &path, slot),
        SettingsMsg::KeybindCaptureCancel => {
            app.panels.settings.keybind_capture = None;
            app.retick();
            Task::none()
        }
        SettingsMsg::KeybindCaptureDefault => {
            let Some(capture) = app.panels.settings.keybind_capture.take() else {
                return Task::none();
            };
            app.panels.settings.keys_moved = None;
            app.panels.settings.keys_same = None;
            store_keybind(app, &capture.path, "");
            Task::none()
        }
        SettingsMsg::KeybindCaptureUnbind => {
            commit_capture(app, None);
            Task::none()
        }
        SettingsMsg::KeybindCaptureClick(button) => {
            let mods = app.input.mods;
            commit_capture(
                app,
                Some(crate::domain::input::Trigger::Mouse(crate::domain::input::MouseSpec {
                    mods,
                    button,
                })),
            );
            Task::none()
        }
        SettingsMsg::KeysFind(query) => {
            app.panels.settings.keybind_capture = None;
            app.panels.settings.keys_query = query;
            app.retick();
            Task::none()
        }
        SettingsMsg::ProcessPickerClose => {
            app.panels.settings.process_picker_open = false;
            app.panels.settings.process_picker_query.clear();
            app.panels.settings.process_picker_manual.clear();
            app.panels.settings.process_picker_scroll = 0.0;
            app.retick();
            Task::none()
        }
        SettingsMsg::ProcessPickerSearch(query) => {
            app.panels.settings.process_picker_query = query;
            app.retick();
            Task::none()
        }
        SettingsMsg::ProcessPickerManualInput(process) => {
            app.panels.settings.process_picker_manual = process;
            app.retick();
            Task::none()
        }
        SettingsMsg::ProcessPickerManualAdd => {
            let process = app.panels.settings.process_picker_manual.trim().to_string();
            if !process.is_empty() {
                let task = process_picker_add(app, &process);
                app.panels.settings.process_picker_manual.clear();
                return task;
            }
            Task::none()
        }
        SettingsMsg::ProcessPickerScroll(offset) => {
            app.panels.settings.process_picker_scroll = offset;
            app.retick();
            Task::none()
        }
        SettingsMsg::ProcessPickerAdd(process) => process_picker_add(app, &process),
        SettingsMsg::ProcessPickerRemove(process) => process_picker_remove(app, &process),
        SettingsMsg::WeatherReviewClose => {
            super::weather_review::close(app);
            Task::none()
        }
        SettingsMsg::WeatherReviewTab(tab) => {
            app.panels.settings.weather_review_tab = tab;
            app.panels.settings.weather_review_focus = match tab {
                crate::frontend::settings::WeatherReviewTab::Suggested => 0,
                crate::frontend::settings::WeatherReviewTab::Approved => 1,
                crate::frontend::settings::WeatherReviewTab::Library => 2,
            };
            app.retick();
            Task::none()
        }
        SettingsMsg::WeatherReviewSearch(query) => {
            app.panels.settings.weather_review_query = query;
            app.panels.settings.weather_review_focus = 0;
            app.retick();
            Task::none()
        }
        SettingsMsg::WeatherReviewToggle(key) => {
            super::weather_review::toggle(app, &key);
            Task::none()
        }
    }
}

fn process_picker_add(app: &mut App, process: &str) -> Task<Message> {
    let mut processes = configured_processes(app);
    if !processes.iter().any(|name| name == process) {
        processes.push(process.to_string());
        save_processes(app, &processes);
    }
    Task::none()
}

fn process_picker_remove(app: &mut App, process: &str) -> Task<Message> {
    let mut processes = configured_processes(app);
    processes.retain(|name| name != process);
    save_processes(app, &processes);
    Task::none()
}

fn configured_processes(app: &App) -> Vec<String> {
    app.config
        .str_path(skwd_config::keys::playback::PROCESSES)
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

fn save_processes(app: &mut App, processes: &[String]) {
    app.config.save_key(skwd_config::keys::playback::PROCESSES, json!(processes.join(", ")));
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
}

fn open_keybind_capture(app: &mut App, path: &str, slot: usize) -> Task<Message> {
    let Some(descriptor) =
        crate::contracts::picker::KEY_BINDINGS.into_iter().find(|entry| entry.path == path)
    else {
        return Task::none();
    };
    commit_settings_input_edit(app);
    close_settings_search(app);
    app.panels.settings.pop = None;
    app.panels.settings.keys_lit = None;
    app.panels.settings.keys_same = None;
    app.panels.settings.keybind_capture = Some(crate::app::state::KeybindCapture {
        path: path.to_string(),
        action: descriptor.action,
        slot: slot.min(crate::domain::input::SLOTS - 1),
    });
    app.retick();
    unfocus_settings_input()
}

fn commit_capture(app: &mut App, replacement: Option<crate::domain::input::Trigger>) {
    use crate::contracts::picker::KEY_BINDINGS;
    use crate::domain::input::{SLOTS, slot_triggers};
    let Some(capture) = app.panels.settings.keybind_capture.take() else {
        return;
    };
    app.panels.settings.keys_same = None;
    let other = SLOTS - 1 - capture.slot;
    if let Some(trigger) = replacement.as_ref().filter(|trigger| {
        slot_triggers(app.input.bindings.triggers(capture.action), other).contains(trigger)
    }) {
        app.panels.settings.keys_same = Some(trigger.label());
        app.panels.settings.keys_moved = None;
        return;
    }
    let holders: Vec<_> = replacement
        .iter()
        .flat_map(|trigger| app.input.bindings.trigger_holders(capture.action, trigger))
        .filter_map(|holder| KEY_BINDINGS.into_iter().find(|entry| entry.action == holder))
        .collect();
    let before = KEY_BINDINGS
        .into_iter()
        .filter(|entry| entry.action == capture.action || holders.contains(entry))
        .map(|entry| (entry.path, app.config.str_path(entry.path)))
        .collect();
    app.panels.settings.keys_moved =
        replacement.as_ref().filter(|_| !holders.is_empty()).map(|key| {
            crate::app::state::KeyMove {
                key: key.label(),
                from: holders.iter().map(|entry| entry.title_key).collect(),
                before,
            }
        });
    commit_binding_slot(app, capture.action, &capture.path, capture.slot, replacement);
}

fn undo_key_move(app: &mut App) {
    let Some(moved) = app.panels.settings.keys_moved.take() else {
        return;
    };
    for (path, value) in moved.before {
        app.panels.settings.inputs.insert(path.to_string(), value.clone());
        super::settings_policy::stage_value(app, path, &json!(value));
    }
    super::settings_policy::flush_staged(app);
    app.reload_bindings();
    app.invalidate_settings();
    app.retick();
}

fn commit_binding_slot(
    app: &mut App,
    action: crate::domain::input::InputAction,
    path: &str,
    slot: usize,
    replacement: Option<crate::domain::input::Trigger>,
) {
    use crate::domain::input::{binding_config, parse_binding, with_slot, without};
    let stolen: Vec<_> = replacement
        .iter()
        .flat_map(|trigger| {
            app.input
                .bindings
                .trigger_holders(action, trigger)
                .map(|other| (other, trigger.clone()))
        })
        .collect();
    for (other, trigger) in stolen {
        let Some(descriptor) =
            crate::contracts::picker::KEY_BINDINGS.into_iter().find(|entry| entry.action == other)
        else {
            continue;
        };
        let value = binding_config(&without(app.input.bindings.triggers(other), &trigger));
        app.panels.settings.inputs.insert(descriptor.path.to_string(), value.clone());
        super::settings_policy::stage_value(app, descriptor.path, &json!(value));
    }
    let triggers = with_slot(app.input.bindings.triggers(action), slot, replacement);
    let value = if parse_binding(action.default_binding()).as_deref() == Some(triggers.as_slice()) {
        String::new()
    } else {
        binding_config(&triggers)
    };
    store_keybind(app, path, &value);
}

fn set_nav_keys(app: &mut App, keys: crate::domain::input::NavKeys) {
    app.panels.settings.keys_moved = None;
    app.panels.settings.keys_same = None;
    for (action, key) in keys.second_keys() {
        let Some(descriptor) =
            crate::contracts::picker::KEY_BINDINGS.into_iter().find(|entry| entry.action == action)
        else {
            continue;
        };
        commit_binding_slot(
            app,
            action,
            descriptor.path,
            1,
            crate::domain::input::Trigger::parse(key),
        );
    }
}

pub(super) fn store_keybind(app: &mut App, path: &str, value: &str) {
    app.panels.settings.inputs.insert(path.to_string(), value.to_string());
    super::settings_policy::stage_value(app, path, &json!(value));
    super::settings_policy::flush_staged(app);
    app.reload_bindings();
    app.invalidate_settings();
    app.retick();
}

pub(super) fn keybind_capture_key(
    app: &mut App,
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
) -> Task<Message> {
    use crate::domain::input::{KeyId, KeySpec, Trigger};
    use iced::keyboard::key::Named;
    if matches!(key, iced::keyboard::Key::Named(Named::Escape)) {
        app.panels.settings.keybind_capture = None;
        app.input.quiet = Some(key.clone());
        app.retick();
        return Task::none();
    }
    let Some(id) = crate::app::input::key_id(key) else {
        return Task::none();
    };
    app.input.quiet = Some(key.clone());
    let symbol = matches!(&id, KeyId::Char(text)
        if text.chars().all(|ch| !ch.is_alphanumeric() && !ch.is_whitespace()));
    let mut mods = crate::app::input::mods(modifiers);
    mods.shift &= !symbol;
    commit_capture(app, Some(Trigger::Key(KeySpec { mods, id })));
    Task::none()
}

fn settings_key(app: &mut App, key: SettingsKey) -> Task<Message> {
    let key = if key == SettingsKey::Space && !app.panels.settings.search_open {
        SettingsKey::Activate
    } else {
        key
    };
    if app.panels.settings.previous.is_some()
        && !app.panels.settings.section_anim.settled()
        && app.panels.settings.focus == SettingsFocus::Controls
        && matches!(key, SettingsKey::Activate | SettingsKey::Next | SettingsKey::Previous)
    {
        return Task::none();
    }

    if app.panels.settings.weather_review_category.is_some() {
        return super::weather_review::key(app, key);
    }
    match key {
        SettingsKey::Search => open_settings_search(app),
        SettingsKey::Space => search_space(app),
        SettingsKey::Cancel => {
            if super::settings_layouts::cancel_rename(app) {
                unfocus_settings_input()
            } else if app.panels.settings.search_open {
                search_cancel(app)
            } else if app.panels.settings.input_edit.is_some() {
                app.panels.settings.edit = None;
                cancel_settings_input_edit(app)
            } else if app.panels.settings.pop.take().is_some()
                || app.panels.settings.edit.take().is_some()
            {
                app.panels.settings.focused_choice = None;
                app.retick();
                Task::none()
            } else if app.panels.settings.focused_choice.take().is_some() {
                app.retick();
                Task::none()
            } else if !app.panels.settings.keys_query.is_empty() {
                app.panels.settings.keys_query.clear();
                app.retick();
                unfocus_settings_input()
            } else {
                super::panels::toggle_settings(app)
            }
        }
        SettingsKey::FocusNext { backwards } => {
            if let Some(edit) = &app.panels.settings.input_edit
                && edit.key.starts_with(skwd_config::keys::filter_bar::RESOLUTION_PRESETS)
                && (edit.key.ends_with(".from") || edit.key.ends_with(".to"))
                && app.panels.settings.resolution_height == backwards
            {
                let key = edit.key.clone();
                app.panels.settings.resolution_height = !backwards;
                app.panels.settings.edit = Some(focus_token(app, &key, usize::from(!backwards)));
                app.retick();
                return iced::widget::operation::focus(
                    crate::frontend::settings::workbench_input_id(&key),
                );
            }
            if tab_within_key_row(app, backwards) {
                return Task::none();
            }
            commit_settings_input_edit(app);
            close_settings_search(app);
            app.panels.settings.focused_choice = None;
            move_settings_tab_focus(app, backwards);
            if backwards && current_keyboard_control(app).is_some_and(|control| two_stops(&control))
            {
                app.panels.settings.focused_choice = Some(1);
            }
            app.retick();
            Task::batch([unfocus_settings_input(), reveal_focused(app)])
        }
        SettingsKey::CategoryNext { backwards } => {
            commit_settings_input_edit(app);
            close_settings_search(app);
            app.panels.settings.focused_choice = None;
            move_settings_tab(app, if backwards { -1 } else { 1 })
        }
        SettingsKey::Previous | SettingsKey::Next | SettingsKey::Up | SettingsKey::Down => {
            if app.panels.settings.search_open {
                return search_arrow(app, key);
            }
            match app.panels.settings.focus {
                SettingsFocus::Index => match key {
                    SettingsKey::Up | SettingsKey::Down => move_settings_index(app, key),
                    SettingsKey::Next => focus_settings_sections(app),
                    _ => Task::none(),
                },
                SettingsFocus::Strip => match key {
                    SettingsKey::Up => super::settings_layouts::strip_choice(app, false),
                    SettingsKey::Down => super::settings_layouts::strip_choice(app, true),
                    SettingsKey::Next => super::settings_layouts::strip_arrow(app, true),
                    SettingsKey::Previous => super::settings_layouts::strip_arrow(app, false),
                    _ => Task::none(),
                },
                SettingsFocus::Sections => match key {
                    SettingsKey::Up | SettingsKey::Down => move_settings_section(app, key),
                    SettingsKey::Next => drill_into_settings_controls(app),
                    SettingsKey::Previous => drill_out_to_settings_index(app),
                    _ => Task::none(),
                },
                SettingsFocus::Controls if app.panels.settings.focused_choice.is_some() => {
                    match key {
                        SettingsKey::Up | SettingsKey::Down => {
                            move_settings_choice(app, matches!(key, SettingsKey::Down))
                        }
                        SettingsKey::Previous => {
                            app.panels.settings.focused_choice = None;
                            app.retick();
                            Task::none()
                        }
                        SettingsKey::Next => activate_settings_control(app),
                        _ => Task::none(),
                    }
                }
                SettingsFocus::Controls => match key {
                    SettingsKey::Up => move_settings_control(app, -1),
                    SettingsKey::Down => move_settings_control(app, 1),
                    SettingsKey::Previous => focus_settings_sections(app),
                    SettingsKey::Next => {
                        tab_within_key_row(app, false);
                        Task::none()
                    }
                    _ => Task::none(),
                },
            }
        }
        SettingsKey::Activate => {
            if app.panels.settings.search_open {
                search_enter(app)
            } else {
                activate_settings_focus(app)
            }
        }
    }
}

fn open_settings_search(app: &mut App) -> Task<Message> {
    commit_settings_input_edit(app);
    app.panels.settings.keybind_capture = None;
    app.panels.settings.search_open = true;
    app.panels.settings.focus = SettingsFocus::Index;
    app.panels.settings.focused_choice = None;
    app.panels.settings.search_at = None;
    refresh_settings_search(app);
    app.retick();
    iced::widget::operation::focus(crate::frontend::settings::settings_search_input_id())
}

fn disarm(app: &mut App) {
    if app.panels.settings.armed.take().is_some() {
        app.invalidate_settings();
    }
}

fn close_settings_search(app: &mut App) {
    app.panels.settings.keybind_capture = None;
    app.panels.settings.search_open = false;
    app.panels.settings.search_query.clear();
    app.panels.settings.search_results.clear();
    app.panels.settings.search_at = None;
}

fn refresh_settings_search(app: &mut App) {
    disarm(app);
    if !app.panels.settings.search_open {
        app.panels.settings.search_results.clear();
        return;
    }
    let hits = crate::frontend::settings::search_hits(
        &app.panels.settings.search_query,
        app.place_source(),
    );
    app.panels.settings.search_results = hits.shown;
    app.panels.settings.search_total = hits.total;
    app.seed_search_inputs();
}

pub(in crate::app) fn reread_search(app: &mut App) {
    if app.panels.settings.search_results.is_empty() {
        return;
    }
    let selected = app
        .panels
        .settings
        .search_at
        .and_then(|at| app.panels.settings.search_results.get(at).cloned());
    let mut results = std::mem::take(&mut app.panels.settings.search_results);
    let kept = crate::frontend::settings::reread_settings(&mut results, app.place_source());
    app.panels.settings.search_results = results;
    if !kept {
        let at = app.panels.settings.search_at;
        refresh_settings_search(app);
        app.panels.settings.search_at = selected
            .and_then(|old| {
                app.panels.settings.search_results.iter().position(|result| result.is(&old))
            })
            .or_else(|| {
                at.map(|at| at.min(app.panels.settings.search_results.len().saturating_sub(1)))
            })
            .filter(|at| *at < app.panels.settings.search_results.len());
        return;
    }
    app.seed_search_inputs();
}

fn search_row(app: &App) -> Option<&crate::frontend::settings::Row> {
    let settings = &app.panels.settings;
    settings.search_results.get(settings.search_at?)?.live()
}

fn search_arrow(app: &mut App, key: SettingsKey) -> Task<Message> {
    disarm(app);
    let down = matches!(key, SettingsKey::Down);
    if !matches!(key, SettingsKey::Up | SettingsKey::Down) {
        let Some(Control::Number { key: input, path, .. }) =
            search_row(app).map(|row| row.control.clone())
        else {
            return Task::none();
        };
        let (_, _, step) = crate::frontend::settings::scrub_bounds(&path);
        let step = if key == SettingsKey::Next { step } else { -step };
        return update(app, SettingsMsg::NumberStep(input, step));
    }
    if app.panels.settings.pop.is_some() && app.panels.settings.focused_choice.is_some() {
        return move_settings_choice(app, down);
    }
    let settings = &mut app.panels.settings;
    settings.pop = None;
    let last = settings.search_results.len().checked_sub(1);
    settings.search_at = match (settings.search_at, down, last) {
        (_, _, None) | (None | Some(0), false, _) => None,
        (None, true, _) => Some(0),
        (Some(at), false, _) => Some(at - 1),
        (Some(at), true, Some(last)) => Some((at + 1).min(last)),
    };
    let at = settings.search_at;
    app.retick();
    at.map_or_else(
        || iced::widget::operation::focus(crate::frontend::settings::settings_search_input_id()),
        |at| Task::batch([unfocus_settings_input(), crate::frontend::settings::reveal_result(at)]),
    )
}

fn search_space(app: &mut App) -> Task<Message> {
    use crate::frontend::settings::{TokenSide, token_key};
    if app.panels.settings.input_edit.is_some() {
        return Task::none();
    }
    let Some(control) = search_row(app).map(|row| row.control.clone()) else {
        return Task::none();
    };
    match control {
        Control::ActionChips { .. } => Task::none(),
        Control::Colour { path, .. } => {
            update(app, SettingsMsg::OpenPop(token_key(&path, 0, TokenSide::Body)))
        }
        Control::Resolution { .. } => {
            app.panels.settings.focused_choice.get_or_insert(0);
            activate_settings_control(app)
        }
        _ => activate_settings_control(app),
    }
}

fn search_enter(app: &mut App) -> Task<Message> {
    let settings = &app.panels.settings;
    if settings.input_edit.is_some() {
        settings_commit(app)
    } else if settings.pop.is_some() && settings.focused_choice.is_some() {
        activate_settings_control(app)
    } else {
        let at = settings.search_at.unwrap_or(0);
        let Some(result) = settings.search_results.get(at).cloned() else {
            return Task::none();
        };
        open_settings_search_result(app, result.tab, result.section, result.row)
    }
}

fn search_cancel(app: &mut App) -> Task<Message> {
    disarm(app);
    if app.panels.settings.input_edit.is_some() {
        app.panels.settings.edit = None;
        return cancel_settings_input_edit(app);
    }
    let settings = &mut app.panels.settings;
    if settings.pop.take().is_some() || settings.edit.take().is_some() {
        settings.focused_choice = None;
        app.retick();
        return Task::none();
    }
    if settings.search_query.is_empty() {
        close_settings_search(app);
        app.retick();
        return unfocus_settings_input();
    }
    settings.search_query.clear();
    settings.search_at = None;
    refresh_settings_search(app);
    app.retick();
    iced::widget::operation::focus(crate::frontend::settings::settings_search_input_id())
}

pub(super) fn open_settings_search_result(
    app: &mut App,
    tab: String,
    section: usize,
    row: usize,
) -> Task<Message> {
    if app.panels.settings.open
        && app.panels.settings.previous.is_some()
        && !app.panels.settings.section_anim.settled()
    {
        app.panels.settings.pending =
            Some(crate::app::state::SettingsNavigation::Search { tab, section, row });
        close_settings_search(app);
        return Task::none();
    }
    let tab_task = if app.panels.settings.tab == tab {
        commit_settings_input_edit(app);
        close_settings_search(app);
        app.panels.settings.layout_studio = false;
        Task::none()
    } else {
        set_settings_tab(app, &tab)
    };
    let place = settings_place(app, &app.panels.settings.tab);
    let Some(target) = place.sections.get(section.min(place.sections.len().saturating_sub(1)))
    else {
        return tab_task;
    };
    let id = target.id;
    let body_row = row >= target.header_rows();
    app.panels.settings.pop = None;
    app.panels.settings.edit = None;
    app.panels.settings.section = place.sections.iter().position(|item| item.id == id).unwrap_or(0);
    if target.has_body() && app.panels.settings.open_section != Some(id) {
        let instant = section_instant(app);
        app.panels.settings.toggle_section(id, instant);
    }
    app.panels.settings.control_page = 0;
    app.panels.settings.focused_control = row;
    if let Some(found) = target.row(row) {
        match &found.control {
            Control::Details { id, .. } => {
                app.panels.settings.expanded_details.insert(id.clone());
            }
            control => {
                if let Some(id) = control.compact_bar_id() {
                    let motion = app.motion_profile();
                    app.panels.settings.open_bar(id, motion);
                }
            }
        }
    }
    app.panels.settings.focus = SettingsFocus::Controls;
    app.panels.settings.focused_choice = None;
    close_settings_search(app);
    app.panels.settings.control_anim.run(0.0, 1.0);
    app.retick();
    let reveal_row =
        if body_row { crate::frontend::settings::reveal_row(id, row) } else { Task::none() };
    Task::batch([
        tab_task,
        unfocus_settings_input(),
        crate::frontend::settings::reveal_section(id),
        reveal_row,
    ])
}

fn section_instant(app: &App) -> bool {
    !app.panels.settings.open
        || crate::frontend::settings::SETTINGS_MOTION.reduced(app.motion_profile())
}

fn reveal_focused(app: &App) -> Task<Message> {
    let settings = &app.panels.settings;
    if settings.focus != SettingsFocus::Controls {
        return Task::none();
    }
    let place = settings_place(app, &settings.tab);
    match place.sections.get(settings.section) {
        Some(section) if section.id == crate::frontend::settings::KEYS_SHEET => {
            crate::frontend::settings::reveal_row(section.id, settings.focused_control)
        }
        Some(section)
            if settings.open_section == Some(section.id)
                && settings.focused_control >= section.header_rows() =>
        {
            Task::batch([
                crate::frontend::settings::reveal_section(section.id),
                crate::frontend::settings::reveal_row(section.id, settings.focused_control),
                if settings.focused_control >= section.rows().count() {
                    crate::frontend::settings::reveal_line(section.id, settings.focused_control)
                } else {
                    Task::none()
                },
            ])
        }
        _ => Task::none(),
    }
}

fn unfocus_settings_input() -> Task<Message> {
    iced::widget::operation::focus(iced::widget::Id::new("settings-workbench-keyboard-focus"))
}

pub(super) fn settings_place(app: &App, tab: &str) -> crate::frontend::settings::Place {
    app.place_source().place(tab)
}

fn keyboard_section(app: &App) -> Option<crate::frontend::settings::Section> {
    let mut sections = settings_place(app, &app.panels.settings.tab).sections;
    let focused = app.panels.settings.section;
    if sections.get(focused).is_some_and(|section| !section.has_body()) {
        return Some(sections.swap_remove(focused));
    }
    let open = app.panels.settings.open_section?;
    sections.into_iter().find(|section| section.id == open)
}

struct ControlPage {
    rows: Vec<crate::frontend::settings::Row>,
    skipped: Vec<usize>,
    header: usize,
    own: usize,
}

fn settings_control_pages(app: &App) -> Vec<ControlPage> {
    let moved = app.panels.settings.keys_moved.is_some();
    keyboard_section(app)
        .map(|section| {
            let own = section.rows().count();
            let skipped = (0..own)
                .filter(|at| {
                    section.say.parks(*at).is_some()
                        || section.follows(*at)
                        || (moved && crate::frontend::settings::conflict_links(&section, *at))
                })
                .collect();
            let header = section.header_rows();
            let lines = crate::frontend::settings::page_rows(section.id, &app.page_ctx());
            let undo = moved && section.id == crate::frontend::settings::KEYS_SHEET;
            let extra = undo.then(crate::frontend::settings::undo_row);
            let rows = section.into_rows().chain(lines).chain(extra).collect();
            ControlPage { rows, skipped, header, own }
        })
        .filter(|page| !page.rows.is_empty())
        .into_iter()
        .collect()
}

fn current_keyboard_control(app: &App) -> Option<Control> {
    if app.panels.settings.search_open {
        return search_row(app).map(|row| row.control.clone());
    }
    keyboard_control_before(app, 0)
}

fn keyboard_control_before(app: &App, back: usize) -> Option<Control> {
    let pages = settings_control_pages(app);
    let page = app.panels.settings.control_page.min(pages.len().saturating_sub(1));
    let at = app.panels.settings.focused_control.checked_sub(back)?;
    pages.get(page)?.rows.get(at).map(|row| row.control.clone())
}

struct AppDetails {
    app: AppThemeStatus,
    profile: Control,
    slot: u8,
}

impl AppDetails {
    fn actions(&self) -> Vec<ActionId> {
        crate::frontend::settings::panel_actions(&self.app, self.slot)
    }

    fn key(&self) -> u64 {
        crate::frontend::settings::app_key(&self.app.id)
    }
}

fn app_details(app: &App) -> Option<AppDetails> {
    if app.panels.settings.search_open {
        return None;
    }
    let profile = current_keyboard_control(app)?;
    let Control::AppTheme { app: status, index } = keyboard_control_before(app, 1)? else {
        return None;
    };
    let slot = u8::try_from(index).unwrap_or(u8::MAX);
    Some(AppDetails { app: status, profile, slot })
}

fn settings_control_positions(pages: &[ControlPage]) -> Vec<(usize, usize)> {
    pages
        .iter()
        .enumerate()
        .flat_map(|(page, controls)| {
            (0..controls.header)
                .chain(controls.own..controls.rows.len())
                .chain(controls.header..controls.own)
                .filter(|at| {
                    controls.rows[*at].control.is_focusable() && !controls.skipped.contains(at)
                })
                .map(move |control| (page, control))
        })
        .collect()
}

fn choice_count(control: &Control) -> usize {
    match control {
        Control::Dropdown { options, .. } | Control::Chips { options, .. } => {
            options.len() + usize::from(control.unlisted())
        }
        Control::ActionChips { items } => items.len(),
        Control::KeyBinding { .. } => crate::domain::input::SLOTS,
        Control::Resolution { .. } => crate::frontend::settings::DIMENSIONS.len(),
        _ => 0,
    }
}

fn choice_enabled(control: &Control, index: usize) -> bool {
    let custom = usize::from(control.unlisted());
    if index < custom {
        return true;
    }
    match control {
        Control::Dropdown { options, .. } => index - custom < options.len(),
        Control::Chips { options, disabled, .. } => {
            options.get(index - custom).is_some_and(|(key, _)| !disabled.contains(key))
        }
        Control::ActionChips { items } => index < items.len(),
        Control::KeyBinding { .. } => index < crate::domain::input::SLOTS,
        Control::Resolution { .. } => index < crate::frontend::settings::DIMENSIONS.len(),
        _ => false,
    }
}

fn initial_choice(control: &Control) -> usize {
    if control.unlisted() {
        return 0;
    }
    let current = match control {
        Control::Dropdown { options, current, .. } => {
            options.iter().position(|(key, _)| key == current).unwrap_or(0)
        }
        Control::Chips { options, current, .. } => {
            options.iter().position(|(key, _)| key == current).unwrap_or(0)
        }
        _ => 0,
    };
    if choice_enabled(control, current) {
        current
    } else {
        (0..choice_count(control)).find(|index| choice_enabled(control, *index)).unwrap_or(0)
    }
}

fn next_choice(control: &Control, current: usize, forwards: bool) -> Option<usize> {
    let count = choice_count(control);
    (1..=count)
        .map(|step| {
            if forwards {
                (current + step) % count
            } else {
                (current + count - step % count) % count
            }
        })
        .find(|index| choice_enabled(control, *index))
}

fn move_settings_tab(app: &mut App, delta: isize) -> Task<Message> {
    let places = crate::frontend::settings::place_keys();
    let current =
        places.iter().position(|key| *key == app.panels.settings.navigation_tab()).unwrap_or(0);
    let next = (current as isize + delta).rem_euclid(places.len() as isize) as usize;
    app.panels.settings.focus = SettingsFocus::Index;
    set_settings_tab(app, places[next])
}

fn move_settings_index(app: &mut App, key: SettingsKey) -> Task<Message> {
    let places = crate::frontend::settings::place_keys();
    let current =
        places.iter().position(|place| *place == app.panels.settings.navigation_tab()).unwrap_or(0);
    let next = match key {
        SettingsKey::Up => current.saturating_sub(1),
        SettingsKey::Down => (current + 1).min(places.len() - 1),
        _ => current,
    };
    if next == current {
        return Task::none();
    }
    app.panels.settings.focus = SettingsFocus::Index;
    set_settings_tab(app, places[next])
}

fn move_settings_section(app: &mut App, key: SettingsKey) -> Task<Message> {
    let place = settings_place(app, &app.panels.settings.tab);
    let count = place.sections.len();
    if count == 0 {
        return Task::none();
    }
    let current = app.panels.settings.section.min(count - 1);
    let next = match key {
        SettingsKey::Up => current.saturating_sub(1),
        SettingsKey::Down => (current + 1).min(count - 1),
        _ => current,
    };
    app.panels.settings.section = next;
    app.panels.settings.focus = SettingsFocus::Sections;
    app.panels.settings.focused_choice = None;
    app.retick();
    crate::frontend::settings::reveal_section(place.sections[next].id)
}

fn focus_settings_sections(app: &mut App) -> Task<Message> {
    app.panels.settings.focus = SettingsFocus::Sections;
    app.panels.settings.focused_choice = None;
    app.retick();
    Task::none()
}

fn drill_into_settings_controls(app: &mut App) -> Task<Message> {
    let place = settings_place(app, &app.panels.settings.tab);
    let Some(section) = place.sections.get(app.panels.settings.section) else {
        return Task::none();
    };
    if section.has_body() && app.panels.settings.open_section != Some(section.id) {
        let instant = section_instant(app);
        app.panels.settings.toggle_section(section.id, instant);
    }
    if section.rows().next().is_none()
        && crate::frontend::settings::page_section(section.id).is_none()
    {
        app.retick();
        return Task::none();
    }
    focus_keyboard_control(app, 0, 0);
    app.retick();
    Task::none()
}

fn drill_out_to_settings_index(app: &mut App) -> Task<Message> {
    app.panels.settings.focus = SettingsFocus::Index;
    app.panels.settings.focused_choice = None;
    app.retick();
    Task::none()
}

fn move_settings_control(app: &mut App, delta: isize) -> Task<Message> {
    let pages = settings_control_pages(app);
    if pages.is_empty() {
        return Task::none();
    }
    let positions = settings_control_positions(&pages);
    if positions.is_empty() {
        return Task::none();
    }
    let current = positions
        .iter()
        .position(|&(page, control)| {
            page == app.panels.settings.control_page
                && control == app.panels.settings.focused_control
        })
        .unwrap_or(0);
    let next =
        (current as isize + delta).clamp(0, positions.len().saturating_sub(1) as isize) as usize;
    let (page, control) = positions[next];
    focus_keyboard_control(app, page, control);
    app.retick();
    reveal_focused(app)
}

fn toggle_settings_section(app: &mut App, section: usize) -> Task<Message> {
    commit_settings_input_edit(app);
    close_settings_search(app);
    let place = settings_place(app, &app.panels.settings.tab);
    let Some(target) = place.sections.get(section) else {
        return Task::none();
    };
    if target.has_body() {
        let instant = section_instant(app);
        app.panels.settings.toggle_section(target.id, instant);
    }
    app.panels.settings.section = section;
    app.panels.settings.focus = SettingsFocus::Sections;
    app.panels.settings.focused_choice = None;
    app.panels.settings.focused_control = 0;
    app.panels.settings.control_page = 0;
    app.panels.settings.armed = None;
    app.panels.settings.folder_undo = None;
    app.panels.settings.layout_studio = false;
    app.retick();
    Task::batch([unfocus_settings_input(), crate::frontend::settings::reveal_section(target.id)])
}

pub(super) fn open_settings_section(app: &mut App, section: usize) -> Task<Message> {
    let place = settings_place(app, &app.panels.settings.tab);
    match place.sections.get(section) {
        Some(target) if app.panels.settings.open_section != Some(target.id) => {
            toggle_settings_section(app, section)
        }
        _ => Task::none(),
    }
}

pub(super) fn clamp_keyboard_control(app: &mut App) {
    let pages = settings_control_pages(app);
    let positions = settings_control_positions(&pages);
    if positions.is_empty() {
        app.panels.settings.control_page = 0;
        app.panels.settings.focused_control = 0;
        return;
    }
    let current = (app.panels.settings.control_page, app.panels.settings.focused_control);
    let (page, control) = positions
        .iter()
        .copied()
        .find(|position| *position == current)
        .or_else(|| positions.iter().copied().find(|position| *position >= current))
        .unwrap_or_else(|| *positions.last().unwrap());
    app.panels.settings.control_page = page;
    app.panels.settings.focused_control = control;
}

fn move_settings_tab_focus(app: &mut App, backwards: bool) {
    let pages = settings_control_pages(app);
    let positions = settings_control_positions(&pages);
    match app.panels.settings.focus {
        SettingsFocus::Index if backwards => {
            if let Some(&(page, control)) = positions.last() {
                focus_keyboard_control(app, page, control);
            } else {
                app.panels.settings.focus = SettingsFocus::Sections;
            }
        }
        SettingsFocus::Index if super::settings_layouts::strip_focusable(app) => {
            super::settings_layouts::focus_strip(app);
        }
        SettingsFocus::Index => app.panels.settings.focus = SettingsFocus::Sections,
        SettingsFocus::Strip if backwards => {
            app.panels.settings.pop = None;
            app.panels.settings.focus = SettingsFocus::Index;
        }
        SettingsFocus::Strip => {
            app.panels.settings.pop = None;
            app.panels.settings.focused_control = 0;
            app.panels.settings.focus = SettingsFocus::Sections;
        }
        SettingsFocus::Sections if backwards && super::settings_layouts::strip_focusable(app) => {
            super::settings_layouts::focus_strip(app);
        }
        SettingsFocus::Sections if backwards => app.panels.settings.focus = SettingsFocus::Index,
        SettingsFocus::Sections => {
            if let Some(&(page, control)) = positions.first() {
                focus_keyboard_control(app, page, control);
            } else {
                app.panels.settings.focus = SettingsFocus::Index;
            }
        }
        SettingsFocus::Controls => {
            if positions.is_empty() {
                app.panels.settings.focus =
                    if backwards { SettingsFocus::Sections } else { SettingsFocus::Index };
                return;
            }
            let current = positions
                .iter()
                .position(|&(page, control)| {
                    page == app.panels.settings.control_page
                        && control == app.panels.settings.focused_control
                })
                .unwrap_or(0);
            if backwards {
                if let Some(&(page, control)) =
                    current.checked_sub(1).and_then(|i| positions.get(i))
                {
                    focus_keyboard_control(app, page, control);
                } else {
                    app.panels.settings.focus = SettingsFocus::Sections;
                }
            } else if let Some(&(page, control)) = positions.get(current + 1) {
                focus_keyboard_control(app, page, control);
            } else {
                app.panels.settings.focus = SettingsFocus::Index;
            }
        }
    }
}

fn focus_keyboard_control(app: &mut App, page: usize, control: usize) {
    let page_changed = app.panels.settings.control_page != page;
    app.panels.settings.focus = SettingsFocus::Controls;
    app.panels.settings.control_page = page;
    app.panels.settings.focused_control = control;
    app.panels.settings.focused_choice = None;
    if page_changed {
        app.panels.settings.control_anim.run(0.0, 1.0);
    }
}

fn two_stops(control: &Control) -> bool {
    matches!(control, Control::KeyBinding { .. })
        || crate::frontend::settings::links_control(control)
}

fn tab_within_key_row(app: &mut App, backwards: bool) -> bool {
    let stops = app.panels.settings.focus == SettingsFocus::Controls
        && app.panels.settings.input_edit.is_none()
        && current_keyboard_control(app).is_some_and(|control| two_stops(&control));
    let choice = app.panels.settings.focused_choice.unwrap_or(0);
    let last = crate::domain::input::SLOTS - 1;
    if !stops || (backwards && choice == 0) || (!backwards && choice >= last) {
        return false;
    }
    app.panels.settings.focused_choice = Some(if backwards { choice - 1 } else { choice + 1 });
    app.retick();
    true
}

fn focused_checks(app: &App) -> Option<(u64, Vec<Control>)> {
    use crate::frontend::settings::{Slot, token_key, token_side};
    if app.panels.settings.search_open {
        return None;
    }
    let section = keyboard_section(app)?;
    let focused = app.panels.settings.focused_control;
    let rows = section
        .say
        .tokens
        .iter()
        .find(|slot| matches!(slot, Slot::Checks { .. }) && slot.lead() == focused)
        .map(|slot| slot.rows().collect())
        .or_else(|| section.checks(focused))?;
    let items: Vec<_> = rows
        .into_iter()
        .filter_map(|at| section.row(at).map(|row| row.control.clone()))
        .filter(|control| control.ticked().is_some() && control.path().is_some())
        .collect();
    let lead = items.first().and_then(Control::path)?.to_string();
    Some((token_key(&lead, 0, token_side(&section, focused)), items))
}

fn activate_checks(app: &mut App) -> Option<Task<Message>> {
    let (key, items) = focused_checks(app)?;
    let Some(choice) = app.panels.settings.focused_choice else {
        app.panels.settings.pop = Some(key);
        app.panels.settings.focused_choice = Some(0);
        app.retick();
        return Some(Task::none());
    };
    match items.get(choice)? {
        Control::Dropdown { path, options, current, .. } => {
            let other = options.iter().find(|(value, _)| value != current)?;
            Some(settings_pick(app, path, &other.0))
        }
        control => {
            let on = control.ticked()?;
            Some(settings_toggle(app, control.path()?, !on))
        }
    }
}

fn move_app_stop(app: &mut App, forwards: bool, stops: usize) {
    let current = app.panels.settings.focused_choice.unwrap_or_default();
    let next = if forwards { current + 1 } else { current + stops - 1 };
    app.panels.settings.focused_choice = Some(next % stops);
    app.retick();
}

fn open_app_stop(app: &mut App, details: &AppDetails) -> Task<Message> {
    let choice = app.panels.settings.focused_choice.unwrap_or_default();
    let Some(id) = choice.checked_sub(1).and_then(|at| details.actions().into_iter().nth(at))
    else {
        let Control::Dropdown { path, .. } = &details.profile else {
            return Task::none();
        };
        app.panels.settings.pop = Some(focus_token(app, path, 0));
        app.panels.settings.focused_choice = Some(initial_choice(&details.profile));
        app.retick();
        return Task::none();
    };
    settings_run(app, id)
}

fn activate_details(app: &mut App) -> Option<Task<Message>> {
    let details = app_details(app)?;
    match app.panels.settings.pop {
        None => {
            app.panels.settings.pop = Some(details.key());
            app.panels.settings.focused_choice = Some(0);
            app.retick();
            Some(Task::none())
        }
        Some(open) if open == details.key() => Some(open_app_stop(app, &details)),
        Some(_) => None,
    }
}

fn move_settings_choice(app: &mut App, forwards: bool) -> Task<Message> {
    if let Some(details) =
        app_details(app).filter(|details| app.panels.settings.pop == Some(details.key()))
    {
        move_app_stop(app, forwards, 1 + details.actions().len());
        return Task::none();
    }
    if let Some((_, items)) = focused_checks(app) {
        let current = app.panels.settings.focused_choice.unwrap_or_default();
        let next = if forwards { current + 1 } else { current.saturating_sub(1) };
        app.panels.settings.focused_choice = Some(next.min(items.len().saturating_sub(1)));
        app.retick();
        return Task::none();
    }
    let Some(control) = current_keyboard_control(app) else {
        return Task::none();
    };
    let count = choice_count(&control);
    if count == 0 {
        return Task::none();
    }
    let current = app.panels.settings.focused_choice.unwrap_or_else(|| initial_choice(&control));
    app.panels.settings.focused_choice = next_choice(&control, current, forwards);
    app.retick();
    Task::none()
}

fn activate_settings_focus(app: &mut App) -> Task<Message> {
    match app.panels.settings.focus {
        SettingsFocus::Index => {
            app.panels.settings.focus = SettingsFocus::Sections;
            app.retick();
            Task::none()
        }
        SettingsFocus::Sections
            if app.panels.settings.tab == crate::frontend::settings::place::KEYBINDS =>
        {
            iced::widget::operation::focus(crate::frontend::settings::keys_find_id())
        }
        SettingsFocus::Sections => {
            move_settings_tab_focus(app, false);
            app.retick();
            reveal_focused(app)
        }
        SettingsFocus::Strip => super::settings_layouts::strip_activate(app),
        SettingsFocus::Controls => activate_settings_control(app),
    }
}

fn activate_settings_control(app: &mut App) -> Task<Message> {
    if app.panels.settings.input_edit.is_some() {
        return settings_commit(app);
    }
    if let Some(task) = activate_checks(app) {
        return task;
    }
    if let Some(task) = activate_details(app) {
        return task;
    }
    let Some(control) = current_keyboard_control(app) else {
        return Task::none();
    };
    match control {
        Control::Display(_) => live_output(app).map_or_else(Task::none, |output| {
            update(app, SettingsMsg::OpenPop(crate::frontend::settings::live_key(&output)))
        }),
        Control::AppTheme { app: entry, .. } if crate::frontend::settings::busy(&entry) => {
            Task::none()
        }
        Control::AppTheme { app: entry, .. } => {
            let index = app
                .daemon
                .app_themes
                .as_ref()
                .and_then(|result| result.apps.iter().position(|row| row.id == entry.id));
            index.map_or_else(Task::none, |index| {
                settings_run(
                    app,
                    ActionId::SetAppTheme(index as u8, !entry.enabled && !entry.can_disable),
                )
            })
        }
        Control::Toggle { path, value } => settings_toggle(app, &path, !value),
        Control::Colour { path, .. } => {
            let key = focus_token(app, &path, 0);
            update(app, SettingsMsg::OpenPop(key))
        }
        Control::KeyBinding { path, .. } => {
            let Some(slot) = app.panels.settings.focused_choice else {
                app.panels.settings.focused_choice = Some(0);
                app.retick();
                return Task::none();
            };
            app.panels.settings.focused_choice = None;
            open_keybind_capture(app, &path, slot)
        }
        Control::Number { key, .. } | Control::TextField { key, .. } => {
            app.panels.settings.edit = Some(focus_token(app, &key, 0));
            begin_settings_input_edit(app, &key);
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&key))
        }
        Control::Resolution { key, .. } => {
            if key.ends_with(".to")
                && app.panels.settings.inputs.get(&key).is_none_or(String::is_empty)
            {
                return update(app, SettingsMsg::ResolutionLimit(key, true));
            }
            let Some(slot) = app.panels.settings.focused_choice else {
                app.panels.settings.focused_choice = Some(0);
                app.retick();
                return Task::none();
            };
            app.panels.settings.focused_choice = None;
            app.panels.settings.resolution_height = slot > 0;
            app.panels.settings.edit = Some(focus_token(app, &key, slot));
            begin_settings_input_edit(app, &key);
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&key))
        }
        menu @ (Control::Dropdown { .. } | Control::Chips { .. }) => pick_menu(app, &menu),
        Control::ActionBtn { id, .. } | Control::ToggleAction { id, .. } => settings_run(app, id),
        Control::ActionChips { items } => {
            let Some(choice) = app.panels.settings.focused_choice else {
                app.panels.settings.focused_choice = Some(0);
                app.retick();
                return Task::none();
            };
            app.panels.settings.focused_choice = None;
            items.get(choice).map_or_else(Task::none, |(id, _)| settings_run(app, id.clone()))
        }
        Control::Details { id, .. } => {
            toggle_settings_details(app, &id);
            Task::none()
        }
        Control::StackBar { id, .. } => {
            let motion = app.motion_profile();
            app.panels.settings.toggle_bar(id, motion);
            app.retick();
            Task::none()
        }
        Control::Segment | Control::Static | Control::Code { .. } | Control::Preview => {
            Task::none()
        }
    }
}

fn live_output(app: &App) -> Option<String> {
    let pages = settings_control_pages(app);
    let page = app.panels.settings.control_page.min(pages.len().saturating_sub(1));
    let rows = &pages.get(page)?.rows;
    let upto = (app.panels.settings.focused_control + 1).min(rows.len());
    rows[..upto].iter().rev().find_map(|row| match &row.control {
        Control::Chips { background: Some(background), .. } => Some(background.output.clone()),
        _ => None,
    })
}

fn pick_menu(app: &mut App, menu: &Control) -> Task<Message> {
    let (Control::Dropdown { path, options, .. } | Control::Chips { path, options, .. }) = menu
    else {
        return Task::none();
    };
    let Some(choice) = app.panels.settings.focused_choice else {
        app.panels.settings.focused_choice = Some(initial_choice(menu));
        app.panels.settings.pop = Some(focus_token(app, path, 0));
        app.retick();
        return Task::none();
    };
    app.panels.settings.focused_choice = None;
    app.panels.settings.pop = None;
    let picked = choice.checked_sub(usize::from(menu.unlisted())).and_then(|at| options.get(at));
    let Some((value, _)) = picked.filter(|_| choice_enabled(menu, choice)) else {
        app.retick();
        return Task::none();
    };
    settings_pick(app, path, value)
}

fn close_token_edit(app: &mut App) {
    if app.panels.settings.edit.take().is_some() {
        commit_settings_input_edit(app);
    }
}

fn control_token(app: &App, control: &Control) -> Option<u64> {
    match control {
        Control::Dropdown { path, .. } | Control::Chips { path, .. } => {
            Some(focus_token(app, path, 0))
        }
        chips if crate::frontend::settings::second_keys_row(chips) => {
            Some(crate::frontend::settings::second_keys_token())
        }
        _ => None,
    }
}

fn focus_token(app: &App, identity: &str, slot: usize) -> u64 {
    use crate::frontend::settings::{TokenSide, token_side};
    if app.panels.settings.search_open {
        return crate::frontend::settings::token_key(identity, slot, TokenSide::Sentence);
    }
    let control = app.panels.settings.focused_control;
    let side =
        keyboard_section(app).map_or(TokenSide::Body, |section| token_side(&section, control));
    crate::frontend::settings::token_key(identity, slot, side)
}

fn toggle_settings_details(app: &mut App, id: &str) {
    if !app.panels.settings.expanded_details.remove(id) {
        app.panels.settings.expanded_details.insert(id.to_string());
    }
    app.retick();
}

pub(super) fn begin_settings_input_edit(app: &mut App, key: &str) {
    if app.panels.settings.input_edit.is_some() {
        return;
    }
    let original = app.panels.settings.inputs.get(key).cloned().unwrap_or_default();
    app.panels.settings.input_edit = Some(SettingsInputEdit {
        key: key.to_string(),
        original,
        we_dirty: app.panels.settings.we_dirty,
        playlist_dirty: app.panels.settings.playlist_dirty,
        schedule_dirty: app.panels.settings.schedule_dirty,
        weather_refresh: app.panels.settings.weather_refresh,
    });
    app.retick();
}

fn commit_closes_settings(app: &mut App) -> bool {
    let open = app.panels.settings.open;
    commit_settings_input_edit(app);
    open && !app.panels.settings.open
}

pub(super) fn commit_settings_input_edit(app: &mut App) {
    if let Some(edit) = app.panels.settings.input_edit.take() {
        super::settings_policy::flush_staged(app);
        super::settings_folders::reveal_committed_folder(app, &edit.key, &edit.original);
    }
}

fn cancel_settings_input_edit(app: &mut App) -> Task<Message> {
    let Some(edit) = app.panels.settings.input_edit.take() else {
        return Task::none();
    };
    let restore = {
        app.panels.settings.inputs.insert(edit.key.clone(), edit.original.clone());
        settings_input_store(app, &edit.key, &edit.original);
        if edit.key.starts_with("keys.") {
            app.reload_bindings();
        }
        Task::none()
    };
    app.panels.settings.we_dirty = edit.we_dirty;
    app.panels.settings.playlist_dirty = edit.playlist_dirty;
    app.panels.settings.schedule_dirty = edit.schedule_dirty;
    app.panels.settings.weather_refresh = edit.weather_refresh;
    if super::settings_policy::changes_weather_members(&edit.key) {
        app.call_tracked(wall_proto::rpc::WALL_WEATHER, json!({}), Pending::Weather);
    }
    app.invalidate_settings();
    app.retick();
    Task::batch([restore, unfocus_settings_input()])
}

pub(super) fn settings_input(app: &mut App, key: &str, raw: &str) -> Task<Message> {
    app.panels.settings.inputs.insert(key.to_string(), raw.to_string());
    settings_input_store(app, key, raw);
    if key.starts_with(skwd_config::keys::filter_bar::RESOLUTION_PRESETS) {
        refresh_resolution_presets(app);
        app.invalidate_settings();
    }
    if super::settings_policy::is_keybind(key) {
        app.reload_bindings();
        app.invalidate_settings();
    }
    Task::none()
}

pub(super) fn settings_input_store(app: &mut App, key: &str, raw: &str) {
    if super::settings_folders::edit_folder(app, key, raw) {
        return;
    }
    if key.starts_with(skwd_config::keys::filter_bar::RESOLUTION_PRESETS) {
        if let Some(base) = key.strip_suffix(".from").or_else(|| key.strip_suffix(".to")) {
            let from_key = format!("{base}.from");
            let to_key = format!("{base}.to");
            let from = app
                .panels
                .settings
                .inputs
                .get(&from_key)
                .cloned()
                .unwrap_or_else(|| app.config.str_path(&from_key));
            let to = app
                .panels
                .settings
                .inputs
                .get(&to_key)
                .cloned()
                .unwrap_or_else(|| app.config.str_path(&to_key));
            if crate::domain::library::filter::resolution_bounds_valid(&from, &to) {
                super::settings_policy::stage_value(app, &from_key, &json!(from));
                super::settings_policy::stage_value(app, &to_key, &json!(to));
            }
            return;
        }
        if key.ends_with(".label") {
            super::settings_policy::stage_value(app, key, &json!(raw));
            return;
        }
    }
    if key == skwd_config::keys::transition::FPS && raw.trim().is_empty() {
        super::settings_policy::stage_value(app, key, &json!(0));
        return;
    }
    if key == skwd_config::keys::playback::PROCESSES
        || (key.starts_with(skwd_config::keys::library::PROFILES)
            && key.ends_with(crate::contracts::settings::keys::library::profile::NAME))
    {
        super::settings_policy::stage_value(app, key, &json!(raw));
        return;
    }
    let is_text = !raw.trim().is_empty() && raw.trim().parse::<f64>().is_err();
    let resolution_dimension = key.starts_with(skwd_config::keys::filter_bar::RESOLUTION_PRESETS)
        && (key.ends_with(".width") || key.ends_with(".height"));
    if resolution_dimension {
        if let Ok(value) = raw.trim().parse::<i64>() {
            super::settings_policy::stage_value(app, key, &json!(value));
        } else {
            super::settings_policy::stage_value(app, key, &json!(raw));
        }
    } else if key == skwd_config::keys::selector::HEX_ARC_INTENSITY_X10 {
        if let Ok(val) = raw.trim().parse::<f64>() {
            super::settings_policy::stage_value(
                app,
                skwd_config::keys::selector::HEX_ARC_INTENSITY,
                &json!(val / 10.0),
            );
            app.apply_layout();
        }
    } else if let Ok(val) = raw.trim().parse::<f64>() {
        super::settings_policy::stage_value(app, key, &json!(val));
        app.apply_layout();
    } else if is_text || raw.trim().is_empty() {
        super::settings_policy::stage_value(app, key, &json!(raw));
    }
}

pub(super) fn settings_toggle(app: &mut App, path: &str, value: bool) -> Task<Message> {
    if path == skwd_config::keys::drawing::ENABLED {
        if value
            && !app.daemon.drawing.as_ref().is_some_and(|status| {
                status.prepared && app.drawing_selection() == Some(status.key.as_str())
            })
        {
            app.show_toast(crate::i18n::tr("settings-drawing-enable-unprepared"));
            return Task::none();
        }
        if !value {
            app.daemon.client.call(wall_proto::rpc::DRAWING_CANCEL, json!({}));
        }
    }
    if path == skwd_config::keys::features::MATUGEN {
        return super::theme::set_matugen_enabled(app, value);
    }
    super::settings_policy::save_value(app, path, &json!(value));
    app.apply_layout();
    app.init_settings_inputs();
    app.invalidate_settings();
    Task::none()
}

pub(super) fn settings_commit(app: &mut App) -> Task<Message> {
    let editing = app.panels.settings.input_edit.take();
    super::settings_policy::flush_staged(app);
    if let Some(edit) = &editing {
        super::settings_folders::reveal_committed_folder(app, &edit.key, &edit.original);
        if let Some(kind) = crate::frontend::settings::folder_draft(&edit.key) {
            let _ = super::settings_folders::add_folder(app, kind);
        }
    }
    if editing.is_some() {
        app.retick();
        unfocus_settings_input()
    } else {
        Task::none()
    }
}

pub(super) fn set_settings_tab(app: &mut App, requested: &str) -> Task<Message> {
    let route = crate::frontend::settings::legacy_section(requested);
    let Some(tab) = crate::frontend::settings::canonical_place(requested) else {
        return Task::none();
    };
    commit_settings_input_edit(app);
    close_settings_search(app);
    app.panels.settings.pop = None;
    app.panels.settings.edit = None;
    app.panels.settings.keybind_capture = None;
    if app.panels.settings.open
        && app.panels.settings.previous.is_some()
        && !app.panels.settings.section_anim.settled()
    {
        app.panels.settings.pending =
            Some(crate::app::state::SettingsNavigation::Page { tab: requested.to_string() });
        return Task::none();
    }
    let changed = app.panels.settings.tab != tab;
    if changed {
        snapshot_settings_page(app);
        app.panels.settings.close_sections();
        app.panels.settings.keys_query.clear();
        app.panels.settings.keys_moved = None;
        app.panels.settings.keys_same = None;
        app.panels.settings.keys_lit = None;
        app.panels.settings.section = 0;
    } else {
        app.panels.settings.section_anim.snap(1.0);
    }
    app.panels.settings.layout_studio = false;
    app.panels.settings.tab = tab.to_string();
    app.panels.settings.control_page = 0;
    app.panels.settings.focused_control = 0;
    app.panels.settings.focused_choice = None;
    app.panels.settings.control_anim.snap(1.0);
    app.panels.settings.armed = None;
    app.panels.settings.folder_undo = None;
    app.init_settings_inputs();
    if changed {
        app.panels.settings.tab_anim.run(0.0, 1.0);
        if !crate::frontend::settings::SETTINGS_MOTION.reduced(app.motion_profile()) {
            app.panels.settings.emblem_anim.run(0.0, 1.0);
        }
        app.chrome.pane_scrolls.remove("settings");
    }
    let top = if changed {
        iced::widget::operation::snap_to(
            crate::frontend::settings::settings_reading_id(),
            iced::widget::scrollable::RelativeOffset::START,
        )
    } else {
        Task::none()
    };
    let focus = app.panels.settings.focus;
    let opened = route
        .and_then(|key| {
            settings_place(app, tab).sections.iter().position(|section| section.id.key == key)
        })
        .map_or_else(Task::none, |section| open_settings_section(app, section));
    app.panels.settings.focus = focus;
    app.invalidate_settings();
    app.retick();
    Task::batch([top, opened])
}

pub(super) fn open_drawing_setup(app: &mut App) -> Task<Message> {
    let opening =
        if app.panels.settings.open { Task::none() } else { super::panels::toggle_settings(app) };
    let tab = crate::frontend::settings::place::WALLPAPERS.to_string();
    let Some(section) = settings_place(app, &tab)
        .sections
        .iter()
        .position(|section| section.title == crate::i18n::tr("settings-drawing-card"))
    else {
        return opening;
    };
    let navigating = open_settings_search_result(app, tab, section, 0);
    Task::batch([opening, navigating])
}

fn snapshot_settings_page(app: &mut App) {
    let motion = app.motion_profile();
    if !app.panels.settings.open || crate::frontend::settings::SETTINGS_MOTION.reduced(motion) {
        app.panels.settings.previous = None;
        app.panels.settings.section_anim.snap(1.0);
        return;
    }
    app.panels.settings.previous = Some(crate::frontend::settings::PageSnapshot {
        place: settings_place(app, &app.panels.settings.tab),
        inputs: app.panels.settings.inputs.clone(),
    });
    motion.retime_tween_override(
        &mut app.panels.settings.section_anim,
        crate::frontend::settings::SETTINGS_MOTION.content_ms,
    );
    app.panels.settings.section_anim.run(0.0, 1.0);
}

pub(super) fn finish_settings_transition(app: &mut App) -> Task<Message> {
    if !app.panels.settings.section_anim.settled() {
        return Task::none();
    }
    app.panels.settings.previous = None;
    let Some(pending) = app.panels.settings.pending.take() else {
        return Task::none();
    };
    match pending {
        crate::app::state::SettingsNavigation::Page { tab } => set_settings_tab(app, &tab),
        crate::app::state::SettingsNavigation::Search { tab, section, row } => {
            open_settings_search_result(app, tab, section, row)
        }
    }
}

pub(super) fn settings_pick(app: &mut App, path: &str, value: &str) -> Task<Message> {
    if let Some(task) = drawing_pick(app, path, value) {
        return task;
    }
    if super::settings_profiles::pick(app, path, value) {
        return Task::none();
    }
    if let Some(task) = super::settings_layouts::pick(app, path, value) {
        return task;
    }
    if let Some(task) = weather_pick(app, path, value) {
        return task;
    }
    let profile_id = path.strip_prefix(&format!("{}.", skwd_config::keys::theme::APP_PROFILES));
    if profile_id.is_some_and(|id| {
        app.daemon.app_themes.as_ref().is_some_and(|result| {
            result.apps.iter().any(|entry| {
                entry.id == id && entry.state == crate::contracts::daemon::APP_THEME_BUSY
            })
        })
    }) {
        return Task::none();
    }
    if path == skwd_config::keys::display::FILL_MODE || path.starts_with("display.fillModes.") {
        let old = app.config.str_path(path);
        let old = if old.is_empty() {
            app.config.str_path(skwd_config::keys::display::FILL_MODE)
        } else {
            old
        };
        let resolved = if value.is_empty() {
            app.config.str_path(skwd_config::keys::display::FILL_MODE)
        } else {
            value.to_string()
        };
        super::settings_background::animate_placement(app, path, &old, &resolved);
    }
    if path == "playback.addProcess" {
        let _ = process_picker_add(app, value);
        app.daemon.playback.available_processes.clear();
        return Task::none();
    }

    if path == crate::frontend::settings::SHADER_FAMILY_KEY {
        let current = app.config.str_path(skwd_config::keys::transition::SHADER);
        let shader = crate::frontend::settings::family_default(&current, value);
        return settings_pick(app, skwd_config::keys::transition::SHADER, &shader);
    }
    if path == skwd_config::keys::theme::ENGINE && value == "matugen" {
        app.config.set_key(skwd_config::keys::features::MATUGEN, json!(true));
    }
    let reveal_hand = path == skwd_config::keys::selector::DISPLAY_MODE
        && crate::contracts::picker::Mode::try_from_key(value)
            == Some(crate::contracts::picker::Mode::Hand)
        && app.panels.settings.open
        && app.panels.settings.tab == crate::frontend::settings::place::LAYOUT
        && !app.panels.settings.layout_studio;
    if reveal_hand {
        snapshot_settings_page(app);
    }
    if path == skwd_config::keys::theme::BACKEND {
        super::theme::save_theme_selection(app, value);
    } else {
        super::settings_policy::save_value(app, path, &json!(value));
    }
    if let Some(output) = path.strip_prefix("display.fillModes.")
        && let Some(status) = app.daemon.output_statuses.iter().find(|status| status.name == output)
    {
        let mut params = json!({
            "type": status.kind.as_key(),
            "output": output,
            "mute": status.mute,
            "volume": status.volume,
            "notify": false,
            "no_transition": true
        });
        if status.kind == crate::contracts::media::MediaKind::WallpaperEngine {
            params["we_id"] = json!(status.we_id);
        } else {
            params["path"] = json!(status.path);
        }
        app.daemon.client.call("wall.apply", params);
    }
    refresh_theme_setting(app, path, value, profile_id);
    app.apply_layout();
    if path == skwd_config::keys::display::FILL_MODE {
        let _ = super::settings_background::commit(app, "");
    }
    settings_pick_side_effects(app, path, value);
    super::settings_background::sync_monitors(app);
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
    if reveal_hand
        && let Some(section) = settings_place(app, &app.panels.settings.tab)
            .sections
            .iter()
            .position(|section| section.id.key == crate::frontend::settings::layout_place::SHAPE)
    {
        return open_settings_section(app, section);
    }
    Task::none()
}

fn refresh_theme_setting(app: &mut App, path: &str, value: &str, profile_id: Option<&str>) {
    if let Some(id) = profile_id {
        if app
            .daemon
            .app_themes
            .as_ref()
            .is_some_and(|result| result.apps.iter().any(|entry| entry.id == id && entry.enabled))
        {
            app.call_tracked(
                wall_proto::rpc::THEME_APP_SET,
                json!(wall_proto::AppThemeSet { id: id.into(), enabled: true, adopt: false }),
                Pending::AppThemeSet,
            );
        }
    } else if let Some(index) = path
        .strip_prefix(&format!("{}.", skwd_config::keys::integrations::LIST))
        .and_then(|suffix| {
            suffix.strip_suffix(&format!(".{}", skwd_config::keys::integrations::PROFILE))
        })
        .and_then(|index| index.parse::<usize>().ok())
    {
        if app.config.flag_default_true(&format!(
            "{}.{index}.{}",
            skwd_config::keys::integrations::LIST,
            skwd_config::keys::integrations::ENABLED
        )) {
            app.call_tracked(
                wall_proto::rpc::THEME_APPS_REFRESH,
                json!(wall_proto::AppThemeRefresh { integration: Some(index), profile: None }),
                Pending::AppThemeSet,
            );
        }
    } else if path.starts_with(skwd_config::keys::matugen::PREFIX)
        || path.starts_with(skwd_config::keys::theme::PREFIX)
        || path == skwd_config::keys::noctalia::THEME_MODE
    {
        info!("theme pick: {path}={value}");
        if app.config.theme_backend() == "static"
            && let Some(pal) = crate::app::helpers::local_static_palette(app, value)
            && path == skwd_config::keys::theme::STATIC_THEME
        {
            app.theme.base_palette = pal;
            app.start_fade(&pal);
        }
        app.daemon.client.call("wall.retheme", json!({}));
        app.invalidate_swatch();
    }
}

pub(super) fn settings_pick_side_effects(app: &mut App, path: &str, value: &str) {
    if path == skwd_config::keys::selector::DISPLAY_MODE {
        super::settings_layouts::close_vanished_section(app);
    }
    if path.starts_with(skwd_config::keys::filter_bar::RESOLUTION_PRESETS)
        && let Some(base) = path.strip_suffix(".orientation")
    {
        for bound in ["from", "to"] {
            let key = format!("{base}.{bound}");
            if let Some((width, height)) =
                crate::domain::library::filter::parse_resolution(&app.config.str_path(&key))
            {
                let (width, height) = if value == "tall" {
                    (width.min(height), width.max(height))
                } else {
                    (width.max(height), width.min(height))
                };
                app.config.save_key(&key, json!(format!("{width}x{height}")));
            }
        }
        refresh_resolution_presets(app);
    }
    if path == skwd_config::keys::filter_bar::DEFAULT_FOLDER {
        let folder = app.config.default_folder();
        app.change_filters(|flt| flt.folder = folder);
    }
    if path == skwd_config::keys::selector::FLIP_EFFECT
        && let Some(id) = EFFECT_NAMES.iter().position(|&n| n == value)
    {
        app.scene.set_effect(id as u32);
    }
    if matches!(
        path,
        skwd_config::keys::semantic::MANIFEST | skwd_config::keys::semantic::INDEX_PROFILE
    ) {
        app.clear_semantic_search();
        app.request_semantic_search();
    }
}

struct ArrayAction {
    add: ActionId,
    remove: fn(u16) -> ActionId,
    key: &'static str,
    template: fn() -> serde_json::Value,
    after: Option<fn(&mut App)>,
}

const ARRAY_ACTIONS: &[ArrayAction] = &[
    ArrayAction {
        add: ActionId::AddPostCommand,
        remove: ActionId::RemovePostCommand,
        key: skwd_config::keys::post_processing::LIST,
        template: || json!({"command": "", "type": "all"}),
        after: None,
    },
    ArrayAction {
        add: ActionId::AddIntegration,
        remove: ActionId::RemoveIntegration,
        key: skwd_config::keys::integrations::LIST,
        template: || json!({"name": "", "template": "", "output": ""}),
        after: None,
    },
    ArrayAction {
        add: ActionId::AddResolutionPreset,
        remove: ActionId::RemoveResolutionPreset,
        key: skwd_config::keys::filter_bar::RESOLUTION_PRESETS,
        template: || json!({"label": crate::i18n::tr("settings-filter-resolution-band-custom"), "orientation": "wide", "from": "1920x1080", "to": ""}),
        after: Some(refresh_resolution_presets),
    },
    ArrayAction {
        add: ActionId::AddSemanticModel,
        remove: ActionId::RemoveSemanticModel,
        key: skwd_config::keys::semantic::MODELS,
        template: || json!({"name": "", "manifest": ""}),
        after: None,
    },
];

const fn remove_index(id: &ActionId) -> Option<u16> {
    match id {
        ActionId::RemovePostCommand(index)
        | ActionId::RemoveIntegration(index)
        | ActionId::RemoveResolutionPreset(index)
        | ActionId::RemoveSemanticModel(index) => Some(*index),
        _ => None,
    }
}

fn run_array_action(app: &mut App, id: &ActionId) {
    let index = remove_index(id);
    let matched = ARRAY_ACTIONS
        .iter()
        .find(|row| &row.add == id || index.is_some_and(|at| &(row.remove)(at) == id));
    let Some(row) = matched else {
        return;
    };
    match index {
        Some(at) => app.config.array_remove(row.key, at as usize),
        None => app.config.array_push(row.key, (row.template)()),
    }
    app.init_settings_inputs();
    if let Some(after) = row.after {
        after(app);
    }
}

fn drawing_pick(app: &mut App, path: &str, value: &str) -> Option<Task<Message>> {
    if path != crate::frontend::settings::DRAWING_SCOPE_KEY {
        return None;
    }
    let library = value == crate::frontend::settings::DRAWING_LIBRARY_SCOPE;
    app.panels.settings.drawing_library = library;
    app.panels.settings.library_job = false;
    Some(if library {
        Task::none()
    } else {
        settings_pick(app, skwd_config::keys::drawing::SCOPE, value)
    })
}

fn weather_pick(app: &mut App, path: &str, value: &str) -> Option<Task<Message>> {
    if path == crate::frontend::settings::WEATHER_MODE_KEY {
        return Some(weather_mode(app, value));
    }
    if path != crate::frontend::settings::WEATHER_REVIEW_KEY {
        return None;
    }
    let conditions = crate::frontend::settings::WEATHER_CONDITIONS;
    let picked = conditions.iter().find(|(category, _)| *category == value);
    Some(picked.map_or_else(Task::none, |(category, _)| super::weather_review::open(app, category)))
}

fn weather_busy(app: &App) -> bool {
    app.daemon.pending.values().any(|pending| *pending == Pending::WeatherAction)
}

fn following(app: &App) -> bool {
    crate::frontend::settings::following_outdoors(
        app.place_source().cfg,
        app.daemon.weather.as_ref(),
    )
}

fn weather_mode(app: &mut App, mode: &str) -> Task<Message> {
    if weather_busy(app) {
        let stopped =
            app.panels.settings.weather_after_follow.as_ref().is_some_and(|queue| queue.stopped);
        app.panels.settings.weather_after_follow =
            Some(WeatherQueue { mode: mode.to_string(), stopped });
        return Task::none();
    }
    if following(app) && mode != skwd_config::keys::weather::SOURCE_OUTDOORS {
        app.panels.settings.weather_after_follow =
            Some(WeatherQueue { mode: mode.to_string(), stopped: true });
        return settings_run(app, ActionId::FollowOutdoors(false));
    }
    switch_weather(app, mode)
}

fn switch_weather(app: &mut App, mode: &str) -> Task<Message> {
    if mode == skwd_config::keys::weather::SOURCE_OUTDOORS {
        return if following(app) {
            Task::none()
        } else {
            settings_run(app, ActionId::FollowOutdoors(true))
        };
    }
    let enabled = mode != crate::frontend::settings::WEATHER_OFF;
    let source = if enabled {
        settings_pick(app, skwd_config::keys::weather::SOURCE, mode)
    } else {
        Task::none()
    };
    Task::batch([source, settings_toggle(app, skwd_config::keys::weather::ENABLED, enabled)])
}

pub(super) fn resume_weather_mode(app: &mut App) -> Task<Message> {
    if weather_busy(app) {
        return Task::none();
    }
    let Some(queue) = app.panels.settings.weather_after_follow.take() else {
        return Task::none();
    };
    let outdoors = queue.mode == skwd_config::keys::weather::SOURCE_OUTDOORS;
    if queue.stopped && following(app) && !outdoors {
        return Task::none();
    }
    weather_mode(app, &queue.mode)
}

fn weather_action(app: &mut App, method: &str, params: Value) {
    if app.daemon.pending.values().any(|pending| *pending == Pending::WeatherAction) {
        return;
    }
    app.call_tracked(method, params, Pending::WeatherAction);
}

pub(super) fn drawing_pack_chosen(
    app: &mut App,
    result: Result<Option<std::path::PathBuf>, String>,
) -> Task<Message> {
    app.panels.settings.folder_picking = false;
    match result {
        Ok(Some(path)) => {
            app.call_tracked(
                wall_proto::rpc::DRAWING_REGISTER,
                json!(wall_proto::drawing::RegisterRequest { path: path.display().to_string() }),
                Pending::DrawingAction,
            );
        }
        Ok(None) => {}
        Err(error) => {
            app.show_toast(
                crate::i18n::tr_args!(crate::app::constants::DRAWING_ERROR, error => error),
            );
        }
    }
    crate::app::warm::unstash(app)
}

pub(super) fn settings_run(app: &mut App, id: ActionId) -> Task<Message> {
    if id.is_destructive() && app.panels.settings.armed.as_ref() != Some(&id) {
        app.panels.settings.armed = Some(id);
        app.invalidate_settings();
        app.retick();
        return Task::none();
    }
    app.panels.settings.armed = None;
    app.invalidate_settings();
    match id {
        id @ (ActionId::UsePremade(_) | ActionId::Nudge(..) | ActionId::CentreNudge(_)) => {
            return super::settings_layouts::run(app, &id);
        }
        ActionId::ArrangePicker => {
            return super::arrange::update(app, crate::frontend::arrange::ArrangeMsg::Begin);
        }
        ActionId::SetThemeTarget(provider, enabled) => {
            if !matches!(provider, "noctalia" | "dms") {
                return Task::none();
            }
            let mut targets = app.config.array_values(skwd_config::keys::theme::TARGETS);
            if targets.iter().any(|value| value.as_str() == Some(provider)) == enabled {
                return Task::none();
            }
            targets.retain(|value| value.as_str() != Some(provider));
            if enabled {
                targets.push(json!(provider));
            }
            super::settings_policy::save_value(
                app,
                skwd_config::keys::theme::TARGETS,
                &json!(targets),
            );
            app.init_settings_inputs();
            app.daemon.client.call("wall.retheme", json!({}));
        }
        ActionId::SetLockScreen(target, enabled) => {
            if app
                .daemon
                .pending
                .values()
                .any(|pending| matches!(pending, Pending::LockScreenSet { .. }))
                || app.panels.settings.sddm_checking
            {
                return Task::none();
            }
            if target == wall_proto::LockScreenTarget::Sddm {
                app.panels.settings.sddm_themes.clear();
                app.panels.settings.sddm_checking = true;
                app.show_toast(crate::i18n::tr("settings-lockscreens-pending"));
                app.retick();
                if enabled {
                    return iced_runtime::task::blocking(|mut sender| {
                        let result = crate::infrastructure::lockscreen::sddm_choices();
                        let _ = sender.try_send(Message::SddmChoices(result));
                    });
                }
                return sddm_helper(false, None);
            }
            lockscreen_set(app, target, enabled, None, None);
        }
        ActionId::ChooseSddmTheme(theme) => {
            if app.panels.settings.sddm_checking {
                return Task::none();
            }
            let Some(theme) = theme else {
                app.panels.settings.sddm_themes.clear();
                app.retick();
                return Task::none();
            };
            if !app.panels.settings.sddm_themes.contains(&theme) {
                return Task::none();
            }
            app.panels.settings.sddm_themes.clear();
            app.panels.settings.sddm_checking = true;
            app.show_toast(crate::i18n::tr("settings-lockscreens-pending"));
            app.retick();
            return sddm_helper(true, Some(theme));
        }
        ActionId::CopyAppThemeTemplate(index) => {
            if let Some(entry) =
                app.daemon.app_themes.as_ref().and_then(|result| result.apps.get(index as usize))
            {
                return iced::clipboard::write(entry.template_path.clone());
            }
        }
        ActionId::CustomizeAppTheme(index, action) => {
            if app.daemon.pending.values().any(|pending| matches!(pending, Pending::AppThemeSet)) {
                return Task::none();
            }
            let Some(entry) = app
                .daemon
                .app_themes
                .as_mut()
                .and_then(|result| result.apps.get_mut(index as usize))
            else {
                return Task::none();
            };
            let allowed = match action {
                "disconnect" => entry.can_disconnect,
                "reconnect" => entry.can_reconnect,
                "create-template" | "reset-template" => !entry.template_path.is_empty(),
                _ => false,
            };
            if !allowed {
                return Task::none();
            }
            let id = entry.id.clone();
            entry.state = "busy".into();
            entry.can_enable = false;
            entry.can_disable = false;
            app.call_tracked(
                "theme.app.customize",
                json!({"id": id, "action": action}),
                Pending::AppThemeSet,
            );
        }
        ActionId::RefreshAppThemes => {
            if app.daemon.pending.values().any(|pending| matches!(pending, Pending::AppThemeSet)) {
                return Task::none();
            }
            app.call_tracked("theme.apps", json!({}), Pending::AppThemes);
        }
        ActionId::SetAppTheme(index, _)
        | ActionId::AdoptAppTheme(index)
        | ActionId::RefreshAppTheme(index) => {
            if app.daemon.pending.values().any(|pending| matches!(pending, Pending::AppThemeSet)) {
                return Task::none();
            }
            let adopt = matches!(id, ActionId::AdoptAppTheme(_));
            let refresh = matches!(id, ActionId::RefreshAppTheme(_));
            let enabled = match id {
                ActionId::SetAppTheme(_, value) => value,
                _ => true,
            };
            let Some(entry) = app
                .daemon
                .app_themes
                .as_mut()
                .and_then(|result| result.apps.get_mut(index as usize))
            else {
                return Task::none();
            };
            if !(if adopt {
                entry.can_adopt
            } else if refresh {
                entry.enabled
                    && entry.can_disable
                    && !matches!(entry.state.as_str(), "changed" | "interrupted")
            } else if enabled {
                entry.can_enable
            } else {
                entry.can_disable
            }) {
                return Task::none();
            }
            let id = entry.id.clone();
            entry.state = "busy".into();
            entry.can_enable = false;
            entry.can_disable = false;
            app.call_tracked(
                wall_proto::rpc::THEME_APP_SET,
                json!(wall_proto::AppThemeSet { id, enabled, adopt }),
                Pending::AppThemeSet,
            );
        }
        ActionId::CaptureWeThumbnails => {
            if app
                .daemon
                .tasks
                .values()
                .any(|task| task.id == WE_THUMBNAILS_TASK && task.state.is_active())
            {
                super::dispatcher::control_task(
                    app,
                    WE_THUMBNAILS_TASK,
                    crate::contracts::daemon::TaskControl::Stop,
                );
            } else {
                app.daemon.client.call(wall_proto::rpc::WALL_CAPTURE_THUMBNAILS, json!({}));
            }
        }
        ActionId::RefreshWeather => {
            app.call_tracked(wall_proto::rpc::WALL_WEATHER_REFRESH, json!({}), Pending::Weather);
        }
        ActionId::ResumeWeather => {
            app.call_tracked(wall_proto::rpc::WALL_WEATHER_RESUME, json!({}), Pending::Weather);
        }
        ActionId::ReviewWeather(category) => return super::weather_review::open(app, category),
        ActionId::FollowOutdoors(enabled) => {
            weather_action(
                app,
                wall_proto::rpc::WALL_WEATHER_FOLLOW,
                json!(wall_proto::WeatherFollowRequest { enabled }),
            );
        }
        ActionId::OutdoorNext(output) => {
            weather_action(
                app,
                wall_proto::rpc::WALL_WEATHER_NEXT,
                json!(wall_proto::WeatherOutputRequest { output }),
            );
        }
        ActionId::OutdoorExclude { output, key } => {
            if !app
                .daemon
                .weather
                .as_ref()
                .and_then(|weather| weather.outdoors.as_ref())
                .is_some_and(|outdoors| {
                    outdoors.outputs.iter().any(|entry| entry.output == output && entry.key == key)
                })
            {
                return Task::none();
            }
            weather_action(
                app,
                wall_proto::rpc::WALL_WEATHER_EXCLUDE,
                json!(wall_proto::WeatherExcludeRequest { key }),
            );
        }
        ActionId::OutdoorPin(output) => {
            weather_action(
                app,
                wall_proto::rpc::WALL_WEATHER_PIN,
                json!(wall_proto::WeatherOutputRequest { output }),
            );
        }
        ActionId::OutdoorResume(output) => {
            weather_action(
                app,
                wall_proto::rpc::WALL_WEATHER_RESUME,
                json!(wall_proto::WeatherOutputRequest { output }),
            );
        }
        ActionId::OutdoorCredits(path) => {
            return super::picker::reveal_path(std::path::PathBuf::from(path));
        }
        ActionId::ClearCache => {
            app.daemon.client.call("wall.clear_data", json!({}));
        }
        ActionId::ChooseDrawingPack => {
            if app.panels.settings.folder_picking {
                return Task::none();
            }
            let title = crate::i18n::tr("settings-drawing-pack-picker-title").to_string();
            let filter = crate::i18n::tr("settings-drawing-pack-picker-filter").to_string();
            app.panels.settings.folder_picking = true;
            let choose = iced_runtime::task::blocking(move |mut sender| {
                let result = crate::infrastructure::file_picker::choose(
                    &crate::infrastructure::file_picker::FilePicker {
                        title: &title,
                        directory: false,
                        filters: &[filter],
                        start: None,
                    },
                );
                let _ = sender.try_send(Message::DrawingPackChosen(result));
            });
            return crate::app::warm::stash(app).chain(choose);
        }
        ActionId::DetachDrawingPack => {
            app.call_tracked(wall_proto::rpc::DRAWING_DETACH, json!({}), Pending::DrawingAction);
        }
        ActionId::PrepareDrawing => {
            let ready = app.daemon.drawing.as_ref().is_some_and(|status| {
                status.pack == wall_proto::drawing::PackState::Available
                    && !matches!(
                        status.phase,
                        wall_proto::drawing::Phase::Checking
                            | wall_proto::drawing::Phase::Preparing
                            | wall_proto::drawing::Phase::Paused
                    )
            });
            if !ready {
                return Task::none();
            }
            if app.config.str_path(skwd_config::keys::drawing::SCOPE)
                == skwd_config::keys::drawing::SCOPE_VIEW
            {
                let keys: Vec<String> = app
                    .drawing_view_keys()
                    .take(wall_proto::drawing::MAX_PREPARE_KEYS + 1)
                    .map(str::to_string)
                    .collect();
                if keys.is_empty() {
                    app.show_toast(crate::i18n::tr(crate::frontend::settings::DRAWING_VIEW_EMPTY));
                    return Task::none();
                }
                if keys.len() > wall_proto::drawing::MAX_PREPARE_KEYS {
                    app.show_toast(crate::i18n::tr_args!(
                        crate::frontend::settings::DRAWING_VIEW_LIMIT,
                        count => app.drawing_view_keys().count(),
                        limit => wall_proto::drawing::MAX_PREPARE_KEYS
                    ));
                    return Task::none();
                }
                app.call_tracked(
                    wall_proto::rpc::DRAWING_PREPARE,
                    json!(wall_proto::drawing::PrepareRequest { keys, ..Default::default() }),
                    Pending::DrawingAction,
                );
                return Task::none();
            }
            let Some(key) = app.drawing_selection() else { return Task::none() };
            app.call_tracked(
                wall_proto::rpc::DRAWING_PREPARE,
                json!(wall_proto::drawing::KeyRequest { key: key.to_string(), output: None }),
                Pending::DrawingAction,
            );
        }
        ActionId::PrepareAllDrawings => {
            let running = app
                .daemon
                .tasks
                .values()
                .any(|task| task.id == wall_proto::drawing::TASK && task.state.is_active());
            if running {
                super::dispatcher::control_task(
                    app,
                    wall_proto::drawing::TASK,
                    crate::contracts::daemon::TaskControl::Stop,
                );
            } else {
                app.panels.settings.drawing_library = false;
                app.panels.settings.library_job = true;
                app.call_tracked(
                    wall_proto::rpc::DRAWING_PREPARE,
                    json!(wall_proto::drawing::PrepareRequest {
                        library: true,
                        ..Default::default()
                    }),
                    Pending::DrawingAction,
                );
            }
        }
        ActionId::CancelDrawing => {
            app.call_tracked(wall_proto::rpc::DRAWING_CANCEL, json!({}), Pending::DrawingAction);
        }
        ActionId::PauseDrawing => {
            app.call_tracked(wall_proto::rpc::DRAWING_PAUSE, json!({}), Pending::DrawingAction);
        }
        ActionId::ResumeDrawing => {
            let Some(key) = app
                .daemon
                .drawing
                .as_ref()
                .filter(|status| status.phase == wall_proto::drawing::Phase::Paused)
                .map(|status| status.active_key.clone())
                .filter(|key| !key.is_empty())
            else {
                return Task::none();
            };
            app.call_tracked(
                wall_proto::rpc::DRAWING_RESUME,
                json!(wall_proto::drawing::KeyRequest { key, output: None }),
                Pending::DrawingAction,
            );
        }
        ActionId::ClearDrawingCache => {
            app.call_tracked(
                wall_proto::rpc::DRAWING_CLEAR_CACHE,
                json!({}),
                Pending::DrawingAction,
            );
        }
        ActionId::PreviewDrawing => {
            let Some(key) = app.drawing_selection() else { return Task::none() };
            if !app
                .daemon
                .drawing
                .as_ref()
                .is_some_and(|status| status.prepared && status.key == key)
            {
                return Task::none();
            }
            app.call_tracked(
                wall_proto::rpc::DRAWING_PREVIEW,
                json!(wall_proto::drawing::KeyRequest { key: key.to_string(), output: None }),
                Pending::DrawingAction,
            );
            return crate::app::warm::stash(app);
        }
        ActionId::RecomputeColors => {
            app.daemon.client.call("wall.recompute_colors", json!({}));
        }
        ActionId::OptimizeImages => {
            app.daemon.client.call("optimize.start", json!({}));
        }
        ActionId::RefreshBackdrop => {
            app.daemon.client.call("wall.refresh_overview_backdrop", json!({}));
        }
        ActionId::CopyLayerRule => {
            return iced::clipboard::write(String::from(crate::frontend::settings::NIRI_SNIPPET));
        }
        ActionId::CopyProfileCommand(index, helm) => {
            return super::settings_profiles::copy_command(app, usize::from(index), helm);
        }
        ActionId::ExportConfiguration => return transfer_configuration(true),
        ActionId::ImportConfiguration => return transfer_configuration(false),
        ActionId::ImportSemanticModel => return import_semantic_model(app),
        ActionId::DeleteSemanticModel(index) => return delete_semantic_model(app, Some(index)),
        ActionId::DeleteActiveSemanticModel => return delete_semantic_model(app, None),
        ActionId::AddPostCommand
        | ActionId::RemovePostCommand(_)
        | ActionId::AddIntegration
        | ActionId::RemoveIntegration(_)
        | ActionId::AddResolutionPreset
        | ActionId::RemoveResolutionPreset(_)
        | ActionId::AddSemanticModel => run_array_action(app, &id),
        ActionId::AddFolder(kind) => {
            commit_settings_input_edit(app);
            return super::settings_folders::add_folder(app, kind);
        }
        ActionId::ChooseFolder(kind, index) => {
            commit_settings_input_edit(app);
            return super::settings_folders::choose_folder(app, kind, index);
        }
        ActionId::RemoveFolder(kind, index) => {
            if commit_closes_settings(app) {
                return Task::none();
            }
            return super::settings_folders::remove_folder(app, kind, index);
        }
        ActionId::UndoFolderRemove => {
            if commit_closes_settings(app) {
                return Task::none();
            }
            return super::settings_folders::restore_folder(app);
        }
        ActionId::NewProfile => return super::settings_profiles::add_profile(app),
        ActionId::SelectProfile(position) => {
            return update(app, SettingsMsg::SelectProfile(position));
        }
        ActionId::ActivateProfile(profile) => {
            return settings_pick(app, skwd_config::keys::library::PROFILE, &profile);
        }
        ActionId::FoldTree(id, open) => return update(app, SettingsMsg::ToggleFold(id, open)),
        ActionId::OpenSection(section) => return open_settings_section(app, section),
        ActionId::DeleteProfile(index) => {
            super::settings_profiles::remove_profile(app, usize::from(index));
        }
        ActionId::CreateResolutionPresetBand(band) => add_resolution_preset_band(app, band),
        ActionId::RemoveSemanticModel(_) => {
            run_array_action(app, &id);
            reconcile_semantic_model(app);
        }
        ActionId::OpenScheduleEditor => {
            crate::app::helpers::sched_open(app);
        }
        ActionId::ChooseRunningProcess => {
            app.panels.settings.process_picker_open = true;
            app.panels.settings.process_picker_query.clear();
            app.call_tracked("playback.processes", json!({}), Pending::PlaybackProcesses);
            app.retick();
        }
        ActionId::OpenThemeDesigner => {
            crate::app::helpers::theme_designer_open(app);
        }
        ActionId::GenerateBugReport => {
            app.call_tracked("status.bug_report", json!({}), Pending::BugReport);
        }
        ActionId::ResetKeybinds => reset_keybinds(app),
        ActionId::SetNavKeys(keys) => set_nav_keys(app, keys),
        ActionId::LightKey(path) => {
            app.panels.settings.keys_lit = Some(path);
            app.panels.settings.keys_query.clear();
            app.retick();
        }
        ActionId::UndoKeyMove => {
            undo_key_move(app);
            clamp_keyboard_control(app);
        }
        ActionId::OpenSearch => return open_settings_search(app),
        ActionId::OpenPlace { place, setting } => {
            return super::settings_home::open_place(app, place, setting);
        }
        ActionId::ResetSetting(change) => return super::settings_home::reset(app, &change),
        ActionId::ResetEverything => return super::settings_home::reset_everything(app),
    }
    Task::none()
}

fn transfer_configuration(export: bool) -> Task<Message> {
    let labels = crate::infrastructure::config_pack::Labels {
        export_title: crate::i18n::tr("settings-configuration-export-dialog"),
        import_title: crate::i18n::tr("settings-configuration-import-dialog"),
    };
    iced_runtime::task::blocking(move |mut sender| {
        let message = if export {
            Message::ConfigurationExported(crate::infrastructure::config_pack::export(&labels))
        } else {
            Message::ConfigurationImported(crate::infrastructure::config_pack::import(&labels))
        };
        let _ = sender.try_send(message);
    })
}

pub(super) fn configuration_exported(
    app: &mut App,
    result: Result<Option<std::path::PathBuf>, String>,
) -> Task<Message> {
    match result {
        Ok(Some(path)) => {
            let path = path.display().to_string();
            app.show_toast(crate::i18n::tr_args!(
                "settings-configuration-export-complete",
                path => path
            ));
        }
        Ok(None) => {}
        Err(error) => app.show_toast(crate::i18n::tr_args!(
            "settings-configuration-export-error",
            error => error
        )),
    }
    app.retick();
    Task::none()
}

pub(super) fn configuration_imported(
    app: &mut App,
    result: Result<Option<std::path::PathBuf>, String>,
) -> Task<Message> {
    match result {
        Ok(Some(path)) => {
            app.adopt_external_config();
            let path = path.display().to_string();
            app.show_toast(crate::i18n::tr_args!(
                "settings-configuration-import-complete",
                path => path
            ));
        }
        Ok(None) => {}
        Err(error) => app.show_toast(crate::i18n::tr_args!(
            "settings-configuration-import-error",
            error => error
        )),
    }
    app.retick();
    Task::none()
}

fn lockscreen_set(
    app: &mut App,
    target: wall_proto::LockScreenTarget,
    enabled: bool,
    sddm_image: Option<String>,
    sddm_theme: Option<String>,
) {
    app.call_tracked(
        wall_proto::rpc::LOCKSCREEN_SET,
        json!(wall_proto::LockScreenSetRequest { target, enabled, sddm_image }),
        Pending::LockScreenSet { target, enabled, sddm_theme },
    );
    app.show_toast(crate::i18n::tr("settings-lockscreens-pending"));
    app.retick();
}

fn sddm_helper(enabled: bool, theme: Option<String>) -> Task<Message> {
    iced_runtime::task::blocking(move |mut sender| {
        let result = crate::infrastructure::lockscreen::prepare_sddm(enabled, theme.as_deref());
        let _ = sender.try_send(Message::SddmSetup { enabled, result });
    })
}

pub(super) fn sddm_rollback(app: &mut App) -> Task<Message> {
    let Some(rollback) = app.panels.settings.sddm_rollback.take() else {
        return Task::none();
    };
    iced_runtime::task::blocking(move |mut sender| {
        let result = crate::infrastructure::lockscreen::set_sddm(
            rollback.enabled,
            rollback.theme.as_deref(),
        );
        let _ = sender.try_send(Message::SddmRollback { error: rollback.error, result });
    })
}

pub(super) fn sddm_rollback_done(app: &mut App, error: &str, result: Result<String, String>) {
    app.panels.settings.sddm_checking = false;
    if let Err(rollback) = result {
        app.show_toast(crate::i18n::tr_args!(
            crate::app::constants::LOCKSCREEN_ROLLBACK_ERROR,
            error => error,
            rollback => rollback,
        ));
    } else {
        app.show_toast(crate::i18n::tr_args!(
            crate::app::constants::LOCKSCREEN_ERROR,
            error => error,
        ));
    }
    app.retick();
}

pub(super) fn sddm_choices(
    app: &mut App,
    result: Result<crate::infrastructure::lockscreen::SddmChoices, String>,
) -> Task<Message> {
    app.panels.settings.sddm_checking = false;
    match result {
        Ok(choices) if !choices.current.is_empty() => {
            app.panels.settings.sddm_checking = true;
            sddm_helper(true, None)
        }
        Ok(choices) if !choices.themes.is_empty() && app.panels.settings.open => {
            app.panels.settings.sddm_themes = choices.themes;
            app.retick();
            Task::none()
        }
        Ok(_) => {
            app.show_toast(crate::i18n::tr("settings-lockscreens-no-sddm-theme"));
            app.retick();
            Task::none()
        }
        Err(error) => {
            app.show_toast(
                crate::i18n::tr_args!(crate::app::constants::LOCKSCREEN_ERROR, error => &error),
            );
            app.retick();
            Task::none()
        }
    }
}

pub(super) fn sddm_setup(
    app: &mut App,
    enabled: bool,
    result: Result<crate::infrastructure::lockscreen::SddmSetup, String>,
) -> Task<Message> {
    app.panels.settings.sddm_checking = false;
    match result {
        Ok(setup) => lockscreen_set(
            app,
            wall_proto::LockScreenTarget::Sddm,
            enabled,
            enabled.then_some(setup.path),
            setup.theme,
        ),
        Err(error) => {
            app.show_toast(
                crate::i18n::tr_args!(crate::app::constants::LOCKSCREEN_ERROR, error => &error),
            );
            app.retick();
        }
    }
    Task::none()
}

fn reset_keybinds(app: &mut App) {
    app.panels.settings.keys_moved = None;
    app.panels.settings.keys_same = None;
    app.panels.settings.keys_lit = None;
    for descriptor in crate::contracts::picker::KEY_BINDINGS {
        app.panels.settings.inputs.insert(descriptor.path.to_string(), String::new());
        super::settings_policy::stage_value(app, descriptor.path, &json!(""));
    }
    super::settings_policy::flush_staged(app);
    app.reload_bindings();
    app.show_toast(crate::i18n::tr("status-keybinds-reset"));
    app.invalidate_settings();
    app.retick();
}

fn import_semantic_model(app: &mut App) -> Task<Message> {
    if app.panels.settings.semantic_importing {
        return Task::none();
    }
    let selected = app.config.str_path(skwd_config::keys::semantic::MANIFEST);
    let tooling = match crate::infrastructure::semantic::SemanticTooling::discover(
        &app.config.cache_dir(),
        &selected,
    ) {
        Ok(tooling) => tooling,
        Err(error) => {
            app.panels.settings.semantic_import_status = crate::i18n::tr_args!(
                "settings-tagging-model-import-error",
                error => error
            );
            app.invalidate_settings();
            app.retick();
            return Task::none();
        }
    };
    let labels = crate::infrastructure::semantic_pack::PickerLabels {
        title: crate::i18n::tr("settings-tagging-model-import-dialog-title").to_string(),
        packs: crate::i18n::tr("settings-tagging-model-import-filter").to_string(),
        all_files: crate::i18n::tr("settings-tagging-model-import-all-files").to_string(),
    };
    app.panels.settings.semantic_importing = true;
    app.panels.settings.semantic_import_status =
        crate::i18n::tr("settings-tagging-model-import-progress").to_string();
    app.invalidate_settings();
    app.retick();
    iced_runtime::task::blocking(move |mut sender| {
        let result = crate::infrastructure::semantic_pack::choose_and_import(
            &tooling.bin,
            &tooling.runtime,
            &labels,
        );
        let _ = sender.try_send(Message::SemanticModelImported(result));
    })
}

pub(super) fn semantic_model_imported(
    app: &mut App,
    result: Result<Option<crate::infrastructure::semantic_pack::ImportedSemanticPack>, String>,
) -> Task<Message> {
    app.panels.settings.semantic_importing = false;
    match result {
        Ok(Some(pack)) => {
            let manifest = pack.manifest.display().to_string();
            let exists =
                app.config.array_values(skwd_config::keys::semantic::MODELS).iter().any(|model| {
                    model.get("manifest").and_then(serde_json::Value::as_str) == Some(&manifest)
                });
            if !exists {
                app.config.array_push(
                    skwd_config::keys::semantic::MODELS,
                    json!({
                        "name": pack.id,
                        "manifest": manifest,
                        "managed": true,
                        "id": pack.id,
                        "version": pack.version,
                        "dimensions": pack.dimensions,
                        "bytes": pack.bytes
                    }),
                );
            }
            app.panels.settings.semantic_import_status = crate::i18n::tr_args!(
                "settings-tagging-model-import-complete",
                id => &pack.id,
                version => &pack.version,
                size => format_pack_size(pack.bytes)
            );
            settings_pick(app, skwd_config::keys::semantic::MANIFEST, &manifest)
        }
        Ok(None) => {
            app.panels.settings.semantic_import_status.clear();
            app.invalidate_settings();
            app.retick();
            Task::none()
        }
        Err(error) => {
            app.panels.settings.semantic_import_status = crate::i18n::tr_args!(
                "settings-tagging-model-import-error",
                error => error
            );
            app.invalidate_settings();
            app.retick();
            Task::none()
        }
    }
}

fn format_pack_size(bytes: u64) -> String {
    const KIB: u64 = 1_024;
    const MIB: u64 = KIB * 1_024;
    const GIB: u64 = MIB * 1_024;
    if bytes >= GIB {
        format!("{:.1} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.0} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.0} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn add_resolution_preset_band(app: &mut App, band: &str) {
    let (label, from, to) = match band {
        "fhd" => ("settings-filter-resolution-band-fhd", "1920x1080", "2559x1439"),
        "qhd" => ("settings-filter-resolution-band-qhd", "2560x1440", "3839x2159"),
        "4k" => ("settings-filter-resolution-band-4k", "3840x2160", "5119x2879"),
        "5k" => ("settings-filter-resolution-band-5k", "5120x2880", "7679x4319"),
        "8k" => ("settings-filter-resolution-band-8k", "7680x4320", ""),
        "custom" => ("settings-filter-resolution-band-custom", "1920x1080", ""),
        _ => return,
    };
    let root = skwd_config::keys::filter_bar::RESOLUTION_PRESETS;
    let base = format!("{root}.{}", app.config.array_len(root));
    let preset =
        json!({"label": crate::i18n::tr(label), "orientation": "wide", "from": from, "to": to});
    app.config.array_push(skwd_config::keys::filter_bar::RESOLUTION_PRESETS, preset);
    if band == "custom" {
        app.panels.settings.expanded_details.insert(format!("{base}.custom"));
    }
    app.panels.settings.expanded_details.insert(base);
    app.init_settings_inputs();
    app.invalidate_settings();
    refresh_resolution_presets(app);
}

fn refresh_resolution_presets(app: &mut App) {
    let active = app.library_session.filters.resolution.clone();
    let active_exists = active.is_empty()
        || app.config.resolution_presets().iter().any(|preset| preset.key() == active);
    if active_exists {
        app.chrome.bar.cache.clear();
        app.retick();
    } else {
        app.change_filters(|filters| filters.resolution.clear());
    }
}

fn reconcile_semantic_model(app: &mut App) {
    let selected = app.config.str_path(skwd_config::keys::semantic::MANIFEST);
    let retained = (0..app.config.array_len(skwd_config::keys::semantic::MODELS)).any(|index| {
        app.config.str_path(&format!("{}.{index}.manifest", skwd_config::keys::semantic::MODELS))
            == selected
    });
    if !selected.is_empty() && !retained {
        super::settings_policy::save_value(app, skwd_config::keys::semantic::MANIFEST, &json!(""));
    }
    app.clear_semantic_search();
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
}

pub(super) fn set_view_mode(app: &mut App, mode: &str) -> Task<Message> {
    let mode = crate::frontend::scene::layout::Mode::from_key(mode);
    app.config.ensure_selector_enabled();
    app.config.save_key(skwd_config::keys::selector::DISPLAY_MODE, json!(mode.as_key()));
    app.apply_layout();
    app.init_settings_inputs();
    app.invalidate_settings();
    Task::none()
}

pub(super) fn cycle_view_mode(app: &mut App, backwards: bool) -> Task<Message> {
    use crate::contracts::picker::Mode;
    if app.menu_capturing() {
        return Task::none();
    }
    let keys = Mode::ALL.map(Mode::as_key);
    let current = Mode::from_key(&app.config.display_mode()).as_key();
    match crate::frontend::ui::cycle_key(&keys, current, backwards) {
        Some(mode) => set_view_mode(app, mode),
        None => Task::none(),
    }
}

#[cfg(test)]
mod tests;

fn delete_semantic_model(app: &mut App, index: Option<u16>) -> Task<Message> {
    if app.panels.settings.semantic_importing {
        return Task::none();
    }
    let models = app.config.array_values(skwd_config::keys::semantic::MODELS);
    let selected = app.config.str_path(skwd_config::keys::semantic::MANIFEST);
    let manifest = match index {
        Some(index) => {
            let Some(path) = models
                .get(usize::from(index))
                .and_then(|model| model.get("manifest"))
                .and_then(serde_json::Value::as_str)
                .filter(|path| !path.trim().is_empty())
            else {
                return Task::none();
            };
            path.to_string()
        }
        None => selected.clone(),
    };
    let resolved = if index.is_some() {
        Ok(std::path::PathBuf::from(&manifest))
    } else {
        crate::infrastructure::semantic::SemanticPaths::manifest(&app.config.cache_dir(), &manifest)
    };
    let id = models
        .iter()
        .find(|model| {
            model.get("manifest").and_then(serde_json::Value::as_str) == Some(manifest.as_str())
                && model.get("managed").and_then(serde_json::Value::as_bool) == Some(true)
        })
        .and_then(|model| model.get("id"))
        .and_then(serde_json::Value::as_str)
        .map(String::from);
    let current_path = crate::infrastructure::semantic::SemanticPaths::manifest(
        &app.config.cache_dir(),
        &selected,
    )
    .ok()
    .and_then(|path| path.canonicalize().ok());
    let target_path = resolved.as_ref().ok().and_then(|path| path.canonicalize().ok());
    let selected = (index.is_none()
        || selected == manifest
        || current_path.is_some() && current_path == target_path)
        .then_some(selected);
    if selected.is_some() {
        super::settings_policy::save_value(
            app,
            skwd_config::keys::semantic::ENABLED,
            &json!(false),
        );
    }
    app.panels.settings.semantic_importing = true;
    app.panels.settings.semantic_import_status =
        crate::i18n::tr("settings-semantic-delete-progress").to_string();
    app.invalidate_settings();
    app.retick();
    iced_runtime::task::blocking(move |mut sender| {
        let result = resolved.and_then(|path| {
            crate::infrastructure::semantic_pack::delete_model(&path, id.as_deref())
        });
        let _ = sender.try_send(Message::SemanticModelDeleted(manifest, selected, result));
    })
}

pub(super) fn semantic_model_deleted(
    app: &mut App,
    manifest: &str,
    selected: Option<&str>,
    result: Result<(), String>,
) -> Task<Message> {
    app.panels.settings.semantic_importing = false;
    match result {
        Ok(()) => {
            let mut models = app.config.array_values(skwd_config::keys::semantic::MODELS);
            models.retain(|model| {
                model.get("manifest").and_then(serde_json::Value::as_str) != Some(manifest)
            });
            app.config.save_key(skwd_config::keys::semantic::MODELS, json!(models));
            let current = app.config.str_path(skwd_config::keys::semantic::MANIFEST);
            if current == manifest || selected == Some(current.as_str()) {
                super::settings_policy::save_value(
                    app,
                    skwd_config::keys::semantic::ENABLED,
                    &json!(false),
                );
                super::settings_policy::save_value(
                    app,
                    skwd_config::keys::semantic::MANIFEST,
                    &json!(""),
                );
            }
            app.panels.settings.semantic_import_status =
                crate::i18n::tr("settings-semantic-delete-complete").to_string();
        }
        Err(error) => {
            app.panels.settings.semantic_import_status =
                crate::i18n::tr_args!("settings-semantic-delete-error", error => error);
        }
    }
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
    Task::none()
}
