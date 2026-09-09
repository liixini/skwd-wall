use iced::Task;
use log::info;
use serde_json::json;

use crate::app::state::SettingsInputEdit;
use crate::app::{App, EFFECT_NAMES, Message, Pending};
use crate::frontend::settings::{
    ActionId, Control, PRESET_NAME_KEY, SettingsFocus, SettingsKey, SettingsMsg,
};

pub(super) fn update(app: &mut App, msg: SettingsMsg) -> Task<Message> {
    match msg {
        SettingsMsg::Input(key, raw) => settings_input(app, &key, &raw),
        SettingsMsg::Toggle(path, value) => settings_toggle(app, &path, value),
        SettingsMsg::Commit => settings_commit(app),
        SettingsMsg::Pick(path, value) => settings_pick(app, &path, &value),
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
        SettingsMsg::KeybindCapture(path) => open_keybind_capture(app, &path),
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
    }
}

fn open_keybind_capture(app: &mut App, path: &str) -> Task<Message> {
    let Some(descriptor) =
        crate::contracts::picker::KEY_BINDINGS.into_iter().find(|entry| entry.path == path)
    else {
        return Task::none();
    };
    commit_settings_input_edit(app);
    close_settings_search(app);
    app.panels.settings.keybind_capture = Some(crate::app::state::KeybindCapture {
        path: path.to_string(),
        action: descriptor.action,
        title_key: descriptor.title_key,
        triggers: app.input.bindings.triggers(descriptor.action).to_vec(),
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
    let value = crate::domain::input::binding_config(&capture.triggers);
    let stolen: Vec<_> = capture
        .triggers
        .iter()
        .filter_map(|trigger| {
            app.input
                .bindings
                .shared_trigger(capture.action, trigger)
                .map(|other| (other, trigger.clone()))
        })
        .collect();
    for (action, trigger) in stolen {
        let Some(descriptor) =
            crate::contracts::picker::KEY_BINDINGS.into_iter().find(|entry| entry.action == action)
        else {
            continue;
        };
        let remaining: Vec<_> = app
            .input
            .bindings
            .triggers(action)
            .iter()
            .filter(|existing| **existing != trigger)
            .cloned()
            .collect();
        let value = crate::domain::input::binding_config(&remaining);
        app.panels.settings.inputs.insert(descriptor.path.to_string(), value.clone());
        super::settings_policy::stage_value(app, descriptor.path, &json!(value));
    }
    store_keybind(app, &capture.path, &value);
    Task::none()
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
        &app.library_session.folder_options,
        &app.panels.settings.semantic_import_status,
        app.theme.backends.as_deref().unwrap_or(&[]),
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
        &app.library_session.folder_options,
        "",
        app.theme.backends.as_deref().unwrap_or(&[]),
        &app.daemon.output_names,
        &app.daemon.output_statuses,
        &app.daemon.output_wallpaper_art,
        app.daemon.library_watch.as_ref(),
        Some(&app.daemon.playback),
    )
}

fn settings_control_pages(app: &App) -> Vec<Vec<crate::frontend::settings::Row>> {
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

fn choice_count(control: &Control) -> usize {
    match control {
        Control::Dropdown { options, .. } | Control::Chips { options, .. } => options.len(),
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
    let mut controls = pages
        .iter()
        .enumerate()
        .flat_map(|(page, rows)| (0..rows.len()).map(move |control| (page, control)));
    let positions: Vec<(usize, usize)> = controls.by_ref().collect();
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
        app.panels.settings.section_anim.run(0.0, 1.0);
        app.panels.settings.control_anim.snap(1.0);
    }
    app.retick();
    Task::none()
}

fn clamp_keyboard_control(app: &mut App) {
    let pages = settings_control_pages(app);
    if pages.is_empty() {
        app.panels.settings.control_page = 0;
        app.panels.settings.focused_control = 0;
        return;
    }
    app.panels.settings.control_page =
        app.panels.settings.control_page.min(pages.len().saturating_sub(1));
    let count = pages[app.panels.settings.control_page].len();
    app.panels.settings.focused_control =
        app.panels.settings.focused_control.min(count.saturating_sub(1));
}

fn move_settings_tab_focus(app: &mut App, backwards: bool) {
    let pages = settings_control_pages(app);
    match app.panels.settings.focus {
        SettingsFocus::Index if backwards => {
            if let Some((page, controls)) = pages.iter().enumerate().next_back() {
                focus_keyboard_control(app, page, controls.len().saturating_sub(1));
            } else {
                app.panels.settings.focus = SettingsFocus::Sections;
            }
        }
        SettingsFocus::Index => app.panels.settings.focus = SettingsFocus::Sections,
        SettingsFocus::Sections if backwards => app.panels.settings.focus = SettingsFocus::Index,
        SettingsFocus::Sections => {
            if pages.is_empty() {
                app.panels.settings.focus = SettingsFocus::Index;
            } else {
                focus_keyboard_control(app, 0, 0);
            }
        }
        SettingsFocus::Controls => {
            if pages.is_empty() {
                app.panels.settings.focus =
                    if backwards { SettingsFocus::Sections } else { SettingsFocus::Index };
                return;
            }
            let page = app.panels.settings.control_page.min(pages.len() - 1);
            let control =
                app.panels.settings.focused_control.min(pages[page].len().saturating_sub(1));
            if backwards {
                if control > 0 {
                    focus_keyboard_control(app, page, control - 1);
                } else if page > 0 {
                    focus_keyboard_control(app, page - 1, pages[page - 1].len().saturating_sub(1));
                } else {
                    app.panels.settings.focus = SettingsFocus::Sections;
                }
            } else if control + 1 < pages[page].len() {
                focus_keyboard_control(app, page, control + 1);
            } else if page + 1 < pages.len() {
                focus_keyboard_control(app, page + 1, 0);
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
        Control::Toggle { path, value } => settings_toggle(app, &path, !value),
        Control::KeyBinding { path, .. } => open_keybind_capture(app, &path),
        Control::Number { key, .. } | Control::TextField { key, .. } => {
            begin_settings_input_edit(app, &key);
            iced::widget::operation::focus(crate::frontend::settings::workbench_input_id(&key))
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
        Control::Chips { path, options, current, disabled } => {
            let Some(choice) = app.panels.settings.focused_choice else {
                app.panels.settings.focused_choice =
                    Some(initial_choice(&Control::Chips { path, options, current, disabled }));
                app.retick();
                return Task::none();
            };
            app.panels.settings.focused_choice = None;
            let Some((value, _)) = options.get(choice) else {
                return Task::none();
            };
            if disabled.contains(value) { Task::none() } else { settings_pick(app, &path, value) }
        }
        Control::ActionBtn { id, .. } => settings_run(app, id),
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
        Control::Static | Control::Code { .. } | Control::Preview => Task::none(),
    }
}

fn toggle_settings_details(app: &mut App, id: &str) {
    if !app.panels.settings.expanded_details.remove(id) {
        app.panels.settings.expanded_details.insert(id.to_string());
    }
    app.retick();
}

fn begin_settings_input_edit(app: &mut App, key: &str) {
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
    });
    app.retick();
}

fn commit_settings_input_edit(app: &mut App) {
    if app.panels.settings.input_edit.take().is_some() {
        super::settings_policy::flush_staged(app);
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
    app.invalidate_settings();
    app.retick();
    Task::batch([restore, unfocus_settings_input()])
}

pub(super) fn settings_input(app: &mut App, key: &str, raw: &str) -> Task<Message> {
    app.panels.settings.inputs.insert(key.to_string(), raw.to_string());
    settings_input_store(app, key, raw);
    if key.starts_with(skwd_config::keys::filter_bar::RESOLUTION_PRESETS) {
        refresh_resolution_presets(app);
    }
    if super::settings_policy::is_keybind(key) {
        app.reload_bindings();
        app.invalidate_settings();
    }
    Task::none()
}

pub(super) fn settings_input_store(app: &mut App, key: &str, raw: &str) {
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
    super::settings_policy::save_value(app, path, &json!(value));
    app.apply_layout();
    app.init_settings_inputs();
    app.invalidate_settings();
    Task::none()
}

pub(super) fn settings_commit(app: &mut App) -> Task<Message> {
    let editing = app.panels.settings.input_edit.take().is_some();
    super::settings_policy::flush_staged(app);
    if editing {
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
    if path == "playback.addProcess" {
        let mut processes = app
            .config
            .str_path(skwd_config::keys::playback::PROCESSES)
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if !processes.iter().any(|name| name == value) {
            processes.push(value.to_string());
        }
        app.config.save_key(skwd_config::keys::playback::PROCESSES, json!(processes.join(", ")));
        app.daemon.playback.available_processes.clear();
        app.init_settings_inputs();
        app.retick();
        return Task::none();
    }

    if path == crate::frontend::settings::SHADER_FAMILY_KEY {
        let current = app.config.str_path(skwd_config::keys::transition::SHADER);
        let shader = crate::frontend::settings::family_default(&current, value);
        return settings_pick(app, skwd_config::keys::transition::SHADER, &shader);
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
    if path.starts_with(skwd_config::keys::matugen::PREFIX)
        || path.starts_with(skwd_config::keys::theme::PREFIX)
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
    app.apply_layout();
    settings_pick_side_effects(app, path, value);
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
    Task::none()
}

pub(super) fn settings_pick_side_effects(app: &mut App, path: &str, value: &str) {
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
        template: || json!({"label": "NEW", "orientation": "wide", "from": "", "to": ""}),
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

const fn remove_index(id: ActionId) -> Option<u16> {
    match id {
        ActionId::RemovePostCommand(index)
        | ActionId::RemoveIntegration(index)
        | ActionId::RemoveResolutionPreset(index)
        | ActionId::RemoveSemanticModel(index) => Some(index),
        _ => None,
    }
}

fn run_array_action(app: &mut App, id: ActionId) {
    let index = remove_index(id);
    let matched = ARRAY_ACTIONS
        .iter()
        .find(|row| row.add == id || index.is_some_and(|at| (row.remove)(at) == id));
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

pub(super) fn settings_run(app: &mut App, id: ActionId) -> Task<Message> {
    if id.is_destructive() && app.panels.settings.armed != Some(id) {
        app.panels.settings.armed = Some(id);
        app.invalidate_settings();
        app.retick();
        return Task::none();
    }
    app.panels.settings.armed = None;
    app.invalidate_settings();
    match id {
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
            return iced::clipboard::write(String::from(
                "layer-rule {\n    match namespace=\"^skwd-paper-backdrop$\"\n    place-within-backdrop true\n}",
            ));
        }
        ActionId::ImportSemanticModel => return import_semantic_model(app),
        ActionId::AddPostCommand
        | ActionId::RemovePostCommand(_)
        | ActionId::AddIntegration
        | ActionId::RemoveIntegration(_)
        | ActionId::AddResolutionPreset
        | ActionId::RemoveResolutionPreset(_)
        | ActionId::AddSemanticModel => run_array_action(app, id),
        ActionId::RemoveSemanticModel(_) => {
            run_array_action(app, id);
            reconcile_semantic_model(app);
        }
        ActionId::OpenScheduleEditor => {
            crate::app::helpers::sched_open(app);
        }
        ActionId::ChooseRunningProcess => {
            app.call_tracked("playback.processes", json!({}), Pending::PlaybackProcesses);
        }
        ActionId::OpenThemeDesigner => {
            crate::app::helpers::theme_designer_open(app);
        }
        ActionId::RunDoctor => {
            app.call_tracked("status.doctor", json!({}), Pending::Doctor);
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
