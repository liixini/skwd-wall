use iced::Task;
use log::info;
use serde_json::{Value, json};

use crate::app::state::SettingsInputEdit;
use crate::app::{App, EFFECT_NAMES, Message, Pending, WE_THUMBNAILS_TASK};
use crate::frontend::settings::{
    ActionId, Control, PRESET_NAME_KEY, SettingsFocus, SettingsKey, SettingsMsg,
};

pub(super) fn update(app: &mut App, msg: SettingsMsg) -> Task<Message> {
    match msg {
        SettingsMsg::Input(key, raw) => settings_input(app, &key, &raw),
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
            let (current_width, current_height) =
                current.split_once(['x', 'X', '×']).unwrap_or(("", ""));
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
        SettingsMsg::Toggle(path, value) => settings_toggle(app, &path, value),
        SettingsMsg::Commit => settings_commit(app),
        SettingsMsg::Pick(path, value) => settings_pick(app, &path, &value),
        SettingsMsg::BackgroundPicker(output) => super::settings_background::picker(app, &output),
        SettingsMsg::BackgroundMode(output, mode) => {
            super::settings_background::mode(app, &output, &mode)
        }
        SettingsMsg::BackgroundColor(output, color) => {
            super::settings_background::color(app, &output, &color)
        }
        SettingsMsg::BackgroundCommit(output) => super::settings_background::commit(app, &output),
        SettingsMsg::SelectSection(section) => select_settings_section(app, section),
        SettingsMsg::LeaveLayoutStudio => select_settings_section(app, 0),
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
        SettingsMsg::EditFolder(key) => {
            commit_settings_input_edit(app);
            if !app.panels.settings.open {
                return Task::none();
            }
            begin_settings_input_edit(app, &key);
            app.invalidate_settings();
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&key))
        }
        SettingsMsg::FolderDraft(kind, raw) => {
            app.panels.settings.folder_drafts.insert(kind.key(), raw);
            app.invalidate_settings();
            Task::none()
        }
        SettingsMsg::ProfileFilter(query) => {
            app.panels.settings.profile_filter = query;
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
        SettingsMsg::SearchClose => {
            close_settings_search(app);
            app.retick();
            unfocus_settings_input()
        }
        SettingsMsg::SearchInput(query) => {
            app.panels.settings.search_query = query;
            refresh_settings_search(app);
            app.retick();
            Task::none()
        }
        SettingsMsg::OpenSearchResult(tab, section, row) => {
            open_settings_search_result(app, tab, section, row)
        }
        SettingsMsg::SelectTab(tab) => {
            app.panels.settings.focus = SettingsFocus::Index;
            set_settings_tab(app, tab)
        }
        SettingsMsg::Key(key) => settings_key(app, key),
        SettingsMsg::Run(id) => settings_run(app, id),
        SettingsMsg::KeybindCapture(path, slot) => open_keybind_capture(app, &path, slot),
        SettingsMsg::KeybindCaptureCancel => {
            app.panels.settings.keybind_capture = None;
            app.retick();
            Task::none()
        }
        SettingsMsg::KeybindCaptureApply => apply_keybind_capture(app),
        SettingsMsg::KeybindCaptureDefault => {
            let Some(capture) = app.panels.settings.keybind_capture.take() else {
                return Task::none();
            };
            store_keybind(app, &capture.path, "");
            Task::none()
        }
        SettingsMsg::KeybindCaptureUnbind => {
            let Some(capture) = app.panels.settings.keybind_capture.as_mut() else {
                return Task::none();
            };
            capture.triggers.clear();
            capture.edited = true;
            app.retick();
            Task::none()
        }
        SettingsMsg::KeybindCaptureClick(button) => {
            let mods = app.input.mods;
            capture_trigger(
                app,
                crate::domain::input::Trigger::Mouse(crate::domain::input::MouseSpec {
                    mods,
                    button,
                }),
            );
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
    let slot = slot.min(crate::domain::input::SLOTS - 1);
    let current = app.input.bindings.triggers(descriptor.action);
    app.panels.settings.keybind_capture = Some(crate::app::state::KeybindCapture {
        path: path.to_string(),
        action: descriptor.action,
        title_key: descriptor.title_key,
        slot,
        triggers: crate::domain::input::slot_triggers(current, slot).to_vec(),
        edited: false,
    });
    app.retick();
    unfocus_settings_input()
}

fn capture_trigger(app: &mut App, trigger: crate::domain::input::Trigger) {
    let Some(capture) = app.panels.settings.keybind_capture.as_mut() else {
        return;
    };
    capture.triggers.clear();
    capture.triggers.push(trigger);
    capture.edited = true;
    app.retick();
}

fn apply_keybind_capture(app: &mut App) -> Task<Message> {
    let Some(capture) = app.panels.settings.keybind_capture.take() else {
        return Task::none();
    };
    if !capture.edited {
        app.retick();
        return Task::none();
    }
    commit_binding_slot(
        app,
        capture.action,
        &capture.path,
        capture.slot,
        capture.triggers.into_iter().next(),
    );
    Task::none()
}

fn commit_binding_slot(
    app: &mut App,
    action: crate::domain::input::InputAction,
    path: &str,
    slot: usize,
    replacement: Option<crate::domain::input::Trigger>,
) {
    use crate::domain::input::{binding_config, parse_binding, with_slot};
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
        let remaining: Vec<_> = app
            .input
            .bindings
            .triggers(other)
            .iter()
            .filter(|existing| **existing != trigger)
            .cloned()
            .collect();
        let value = binding_config(&remaining);
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

fn store_keybind(app: &mut App, path: &str, value: &str) {
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
    use iced::keyboard::key::Named;
    if matches!(key, iced::keyboard::Key::Named(Named::Escape)) {
        app.panels.settings.keybind_capture = None;
        app.retick();
        return Task::none();
    }
    if matches!(key, iced::keyboard::Key::Named(Named::Enter)) {
        return apply_keybind_capture(app);
    }
    let Some(id) = crate::app::input::key_id(key) else {
        return Task::none();
    };
    capture_trigger(
        app,
        crate::domain::input::Trigger::Key(crate::domain::input::KeySpec {
            mods: crate::app::input::mods(modifiers),
            id,
        }),
    );
    Task::none()
}

fn settings_key(app: &mut App, key: SettingsKey) -> Task<Message> {
    if app.panels.settings.weather_review_category.is_some() {
        return super::weather_review::key(app, key);
    }
    match key {
        SettingsKey::Search => open_settings_search(app),
        SettingsKey::Cancel => {
            if app.panels.settings.search_open {
                close_settings_search(app);
                app.retick();
                unfocus_settings_input()
            } else if app.panels.settings.input_edit.is_some() {
                cancel_settings_input_edit(app)
            } else if app.panels.settings.focused_choice.take().is_some() {
                app.retick();
                Task::none()
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
                app.panels.settings.resolution_height = !backwards;
                return iced::widget::operation::focus(
                    crate::frontend::settings::workbench_input_id(&format!(
                        "{}.{}",
                        edit.key,
                        if backwards { "width" } else { "height" }
                    )),
                );
            }
            if let Some(task) = tab_within_motion_weights(app, backwards) {
                return task;
            }
            commit_settings_input_edit(app);
            close_settings_search(app);
            app.panels.settings.focused_choice = None;
            move_settings_tab_focus(app, backwards);
            app.retick();
            unfocus_settings_input()
        }
        SettingsKey::CategoryNext { backwards } => {
            commit_settings_input_edit(app);
            close_settings_search(app);
            app.panels.settings.focused_choice = None;
            move_settings_tab(app, if backwards { -1 } else { 1 })
        }
        SettingsKey::Previous | SettingsKey::Next | SettingsKey::Up | SettingsKey::Down => {
            if app.panels.settings.search_open {
                return Task::none();
            }
            match app.panels.settings.focus {
                SettingsFocus::Index => match key {
                    SettingsKey::Up | SettingsKey::Down => move_settings_index(app, key),
                    SettingsKey::Next => focus_settings_sections(app),
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
                    _ => Task::none(),
                },
            }
        }
        SettingsKey::Activate => {
            if app.panels.settings.search_open {
                let Some(result) = app.panels.settings.search_results.first().cloned() else {
                    return Task::none();
                };
                open_settings_search_result(app, result.tab, result.section, result.row)
            } else {
                activate_settings_focus(app)
            }
        }
    }
}

fn open_settings_search(app: &mut App) -> Task<Message> {
    commit_settings_input_edit(app);
    app.panels.settings.search_open = true;
    app.panels.settings.focus = SettingsFocus::Index;
    app.panels.settings.focused_choice = None;
    refresh_settings_search(app);
    app.retick();
    iced::widget::operation::focus(crate::frontend::settings::settings_search_input_id())
}

fn close_settings_search(app: &mut App) {
    app.panels.settings.search_open = false;
    app.panels.settings.search_query.clear();
    app.panels.settings.search_results.clear();
}

fn refresh_settings_search(app: &mut App) {
    if !app.panels.settings.search_open {
        app.panels.settings.search_results.clear();
        return;
    }
    app.panels.settings.search_results = crate::frontend::settings::search_settings(
        &app.panels.settings.search_query,
        &app.config,
        &app.daemon.effect_themes,
        &app.library_session.all_folders,
        &app.panels.settings.semantic_import_status,
        app.theme.backends.as_deref().unwrap_or(&[]),
        app.daemon.app_themes.as_ref(),
    );
}

fn open_settings_search_result(
    app: &mut App,
    tab: String,
    section: usize,
    row: usize,
) -> Task<Message> {
    let tab_task = set_settings_tab(app, tab);
    let section_count = settings_cards(app).len();
    if section_count == 0 {
        return tab_task;
    }
    app.panels.settings.section = section.min(section_count - 1);
    app.panels.settings.control_page = 0;
    app.panels.settings.focused_control = 0;
    let pages = settings_control_pages(app);
    let mut remaining = row;
    for (page_index, page) in pages.iter().enumerate() {
        if remaining < page.len() {
            app.panels.settings.control_page = page_index;
            app.panels.settings.focused_control = remaining;
            match &page[remaining].control {
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
            break;
        }
        remaining = remaining.saturating_sub(page.len());
    }
    app.panels.settings.focus = SettingsFocus::Controls;
    app.panels.settings.focused_choice = None;
    close_settings_search(app);
    app.panels.settings.section_anim.run(0.0, 1.0);
    app.panels.settings.control_anim.run(0.0, 1.0);
    app.retick();
    Task::batch([tab_task, unfocus_settings_input()])
}

fn unfocus_settings_input() -> Task<Message> {
    iced::widget::operation::focus(iced::widget::Id::new("settings-workbench-keyboard-focus"))
}

fn settings_cards(
    app: &App,
) -> Vec<(crate::frontend::settings::Card, Vec<crate::frontend::settings::Row>)> {
    crate::frontend::settings::build_tab_with_runtime_status(
        &app.panels.settings.tab,
        &app.config,
        &app.daemon.effect_themes,
        &app.library_session.all_folders,
        "",
        app.theme.backends.as_deref().unwrap_or(&[]),
        app.settings_runtime_status(),
    )
}

fn settings_control_pages(app: &App) -> Vec<Vec<crate::frontend::settings::Row>> {
    if crate::frontend::settings::page_section(
        &app.panels.settings.tab,
        app.panels.settings.section,
    )
    .is_some()
    {
        return Vec::new();
    }
    let rows = settings_cards(app)
        .into_iter()
        .nth(app.panels.settings.section)
        .map_or_else(Vec::new, |(_, rows)| rows);
    (!rows.is_empty()).then_some(rows).into_iter().collect()
}

fn current_keyboard_control(app: &App) -> Option<Control> {
    let pages = settings_control_pages(app);
    let page = app.panels.settings.control_page.min(pages.len().saturating_sub(1));
    pages.get(page)?.get(app.panels.settings.focused_control).map(|row| row.control.clone())
}

fn settings_control_positions(
    pages: &[Vec<crate::frontend::settings::Row>],
) -> Vec<(usize, usize)> {
    pages
        .iter()
        .enumerate()
        .flat_map(|(page, rows)| {
            rows.iter()
                .enumerate()
                .filter(|(_, row)| row.control.is_focusable())
                .map(move |(control, _)| (page, control))
        })
        .collect()
}

fn choice_count(control: &Control) -> usize {
    match control {
        Control::Dropdown { options, .. } | Control::Chips { options, .. } => options.len(),
        Control::ActionChips { items } => items.len(),
        Control::KeyBinding { .. } => crate::domain::input::SLOTS,
        Control::Presets { items, .. } => 1 + items.len() * 2,
        _ => 0,
    }
}

fn choice_enabled(control: &Control, index: usize) -> bool {
    match control {
        Control::Dropdown { options, .. } => index < options.len(),
        Control::Chips { options, disabled, .. } => {
            options.get(index).is_some_and(|(key, _)| !disabled.contains(key))
        }
        Control::Presets { items, .. } => index < 1 + items.len() * 2,
        Control::ActionChips { items } => index < items.len(),
        Control::KeyBinding { .. } => index < crate::domain::input::SLOTS,
        _ => false,
    }
}

fn initial_choice(control: &Control) -> usize {
    let current = match control {
        Control::Dropdown { options, current, .. } => {
            options.iter().position(|(key, _)| key == current).unwrap_or(0)
        }
        Control::Chips { options, current, .. } => {
            options.iter().position(|(key, _)| key == current).unwrap_or(0)
        }
        Control::Presets { items, .. } => {
            items.iter().position(|(_, active)| *active).map_or(0, |index| 1 + index * 2)
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
    let tabs = crate::frontend::settings::visible_tabs(&app.config);
    if tabs.is_empty() {
        return Task::none();
    }
    let current = tabs.iter().position(|(key, _)| *key == app.panels.settings.tab).unwrap_or(0);
    let next = (current as isize + delta).rem_euclid(tabs.len() as isize) as usize;
    app.panels.settings.focus = SettingsFocus::Index;
    set_settings_tab(app, tabs[next].0.to_string())
}

fn move_settings_index(app: &mut App, key: SettingsKey) -> Task<Message> {
    let tabs = crate::frontend::settings::visible_tabs(&app.config);
    if tabs.is_empty() {
        return Task::none();
    }
    let current = tabs.iter().position(|(tab, _)| *tab == app.panels.settings.tab).unwrap_or(0);
    let next = match key {
        SettingsKey::Up => current.saturating_sub(1),
        SettingsKey::Down => (current + 1).min(tabs.len() - 1),
        _ => current,
    };
    if next == current {
        return Task::none();
    }
    app.panels.settings.focus = SettingsFocus::Index;
    set_settings_tab(app, tabs[next].0.to_string())
}

fn move_settings_section(app: &mut App, key: SettingsKey) -> Task<Message> {
    let count = settings_cards(app).len();
    if count == 0 {
        return Task::none();
    }
    let current = app.panels.settings.section.min(count - 1);
    let section = match key {
        SettingsKey::Up => current.saturating_sub(1),
        SettingsKey::Down => (current + 1).min(count - 1),
        _ => current,
    };
    if section == current {
        return Task::none();
    }
    select_settings_section(app, section)
}

fn focus_settings_sections(app: &mut App) -> Task<Message> {
    app.panels.settings.focus = SettingsFocus::Sections;
    app.panels.settings.focused_choice = None;
    app.retick();
    Task::none()
}

fn drill_into_settings_controls(app: &mut App) -> Task<Message> {
    let pages = settings_control_pages(app);
    if pages.first().is_none_or(Vec::is_empty) {
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
    Task::none()
}

fn select_settings_section(app: &mut App, section: usize) -> Task<Message> {
    commit_settings_input_edit(app);
    close_settings_search(app);
    app.panels.settings.focus = SettingsFocus::Sections;
    app.panels.settings.focused_choice = None;
    app.panels.settings.focused_control = 0;
    app.panels.settings.control_page = 0;
    if app.panels.settings.section != section {
        app.panels.settings.section = section;
        app.panels.settings.armed = None;
        app.panels.settings.folder_undo = None;
        app.panels.settings.section_anim.run(0.0, 1.0);
        app.panels.settings.control_anim.snap(1.0);
    }
    app.retick();
    Task::none()
}

fn clamp_keyboard_control(app: &mut App) {
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
        .find(|position| *position >= current)
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
        SettingsFocus::Index => app.panels.settings.focus = SettingsFocus::Sections,
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

fn tab_within_motion_weights(app: &mut App, backwards: bool) -> Option<Task<Message>> {
    let active_key = app.panels.settings.input_edit.as_ref()?.key.clone();
    let Control::MotionWeights { weights } = current_keyboard_control(app)? else {
        return None;
    };
    let current = weights.iter().position(|(_, key, _)| key == &active_key)?;
    let next = if backwards {
        current.checked_sub(1)?
    } else {
        let next = current + 1;
        (next < weights.len()).then_some(next)?
    };
    let next_key = weights[next].1.clone();
    commit_settings_input_edit(app);
    begin_settings_input_edit(app, &next_key);
    Some(iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&next_key)))
}

fn move_settings_choice(app: &mut App, forwards: bool) -> Task<Message> {
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
        SettingsFocus::Sections => {
            move_settings_tab_focus(app, false);
            app.retick();
            Task::none()
        }
        SettingsFocus::Controls => activate_settings_control(app),
    }
}

fn activate_settings_control(app: &mut App) -> Task<Message> {
    if app.panels.settings.input_edit.is_some() {
        return settings_commit(app);
    }
    let Some(control) = current_keyboard_control(app) else {
        return Task::none();
    };
    match control {
        Control::Display(control) => super::update_inner(app, control.activate()),
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
            begin_settings_input_edit(app, &key);
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&key))
        }
        Control::Resolution { key, .. } => {
            if key.ends_with(".to")
                && app.panels.settings.inputs.get(&key).is_none_or(String::is_empty)
            {
                return update(app, SettingsMsg::ResolutionLimit(key, true));
            }
            app.panels.settings.resolution_height = false;
            begin_settings_input_edit(app, &key);
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&format!(
                "{key}.width"
            )))
        }
        Control::MotionWeights { weights } => {
            let Some((_, key, _)) = weights.first() else {
                return Task::none();
            };
            let motion = app.motion_profile();
            app.panels.settings.open_bar(format!("setting-group:{key}"), motion);
            begin_settings_input_edit(app, key);
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(key))
        }
        Control::Dropdown { path, options, current, .. } => {
            let Some(choice) = app.panels.settings.focused_choice else {
                app.panels.settings.focused_choice =
                    Some(options.iter().position(|(key, _)| key == &current).unwrap_or(0));
                app.retick();
                return Task::none();
            };
            app.panels.settings.focused_choice = None;
            options
                .get(choice)
                .map_or_else(Task::none, |(value, _)| settings_pick(app, &path, value))
        }
        Control::Chips { path, options, current, disabled, background } => {
            let Some(choice) = app.panels.settings.focused_choice else {
                app.panels.settings.focused_choice = Some(initial_choice(&Control::Chips {
                    path,
                    options,
                    current,
                    disabled,
                    background,
                }));
                app.retick();
                return Task::none();
            };
            app.panels.settings.focused_choice = None;
            let Some((value, _)) = options.get(choice) else {
                return Task::none();
            };
            if disabled.contains(value) { Task::none() } else { settings_pick(app, &path, value) }
        }
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
        Control::Presets { mode, items } => {
            let Some(choice) = app.panels.settings.focused_choice else {
                app.panels.settings.focused_choice = Some(
                    items.iter().position(|(_, active)| *active).map_or(0, |index| 1 + index * 2),
                );
                app.retick();
                return Task::none();
            };
            app.panels.settings.focused_choice = None;
            if choice == 0 {
                return save_preset(app);
            }
            let item = (choice - 1) / 2;
            let Some((name, _)) = items.get(item) else {
                return Task::none();
            };
            if (choice - 1).is_multiple_of(2) {
                apply_preset(app, &mode, name)
            } else {
                delete_preset(app, &mode, name)
            }
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

fn commit_settings_input_edit(app: &mut App) {
    if let Some(edit) = app.panels.settings.input_edit.take() {
        super::settings_policy::flush_staged(app);
        super::settings_folders::reveal_committed_folder(app, &edit.key, &edit.original);
    }
}

fn cancel_settings_input_edit(app: &mut App) -> Task<Message> {
    let Some(edit) = app.panels.settings.input_edit.take() else {
        return Task::none();
    };
    let restore = if edit.key == PRESET_NAME_KEY {
        preset_name_input(app, &edit.original)
    } else {
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
    if key.starts_with(skwd_config::keys::library::PROFILES)
        && key.ends_with(crate::contracts::settings::keys::library::profile::NAME)
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
    }
    if editing.is_some() {
        app.retick();
        unfocus_settings_input()
    } else {
        Task::none()
    }
}

pub(super) fn set_settings_tab(app: &mut App, tab: String) -> Task<Message> {
    let tab = crate::frontend::settings::canonical_category(tab);
    commit_settings_input_edit(app);
    close_settings_search(app);
    let changed = app.panels.settings.tab != tab;
    app.panels.settings.tab = tab;
    app.panels.settings.section = 0;
    app.panels.settings.control_page = 0;
    app.panels.settings.focused_control = 0;
    app.panels.settings.focused_choice = None;
    if changed {
        app.panels.settings.section_anim.run(0.0, 1.0);
    } else {
        app.panels.settings.section_anim.snap(1.0);
    }
    app.panels.settings.control_anim.snap(1.0);
    app.panels.settings.armed = None;
    app.panels.settings.folder_undo = None;
    app.init_settings_inputs();
    if changed {
        app.panels.settings.tab_anim.run(0.0, 1.0);
        app.chrome.pane_scrolls.remove("settings");
    }
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn settings_pick(app: &mut App, path: &str, value: &str) -> Task<Message> {
    if super::settings_profiles::pick(app, path, value) {
        return Task::none();
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
        super::settings_background::animate_placement(app, path, &old, value);
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
            app.start_fade(pal);
        }
        app.daemon.client.call("wall.retheme", json!({}));
        app.invalidate_swatch();
    }
}

pub(super) fn settings_pick_side_effects(app: &mut App, path: &str, value: &str) {
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

fn weather_action(app: &mut App, method: &str, params: Value) {
    if app.daemon.pending.values().any(|pending| *pending == Pending::WeatherAction) {
        return;
    }
    app.call_tracked(method, params, Pending::WeatherAction);
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
                app.daemon.client.call(
                    wall_proto::rpc::TASK_CONTROL,
                    json!({"id": WE_THUMBNAILS_TASK, "action": "stop"}),
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
        ActionId::ResetMotionFast => reset_motion_weight(app, skwd_config::keys::motion::FAST_MS),
        ActionId::ResetMotionStandard => {
            reset_motion_weight(app, skwd_config::keys::motion::STANDARD_MS);
        }
        ActionId::ResetMotionSlow => reset_motion_weight(app, skwd_config::keys::motion::SLOW_MS),
        ActionId::ResetKeybinds => reset_keybinds(app),
        ActionId::SetNavKeys(keys) => set_nav_keys(app, keys),
    }
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

fn reset_motion_weight(app: &mut App, path: &str) {
    app.config.remove_key(path);
    app.config.persist();
    app.apply_layout();
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
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

pub(super) fn apply_preset(app: &mut App, mode: &str, name: &str) -> Task<Message> {
    app.config.apply_selector_preset(mode, name);
    app.apply_layout();
    app.init_settings_inputs();
    app.panels.settings.inputs.insert(PRESET_NAME_KEY.to_string(), name.to_string());
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn save_preset(app: &mut App) -> Task<Message> {
    let mode = app.config.selector_mode();
    let name = app.config.next_preset_name(&mode, |number| {
        crate::i18n::tr_args!(
            "settings-selector-preset-generated-name",
            number => number.to_string()
        )
    });
    app.config.save_selector_preset(&mode, &name);
    app.panels.settings.inputs.insert(PRESET_NAME_KEY.to_string(), name);
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn preset_name_input(app: &mut App, text: &str) -> Task<Message> {
    app.panels.settings.inputs.insert(PRESET_NAME_KEY.to_string(), text.to_string());
    let mode = app.config.selector_mode();
    app.config.rename_selected_preset(&mode, text);
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn delete_preset(app: &mut App, mode: &str, name: &str) -> Task<Message> {
    app.config.delete_selector_preset(mode, name);
    let selected = app.config.selected_preset(mode).unwrap_or_default();
    app.panels.settings.inputs.insert(PRESET_NAME_KEY.to_string(), selected);
    app.invalidate_settings();
    app.retick();
    Task::none()
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
