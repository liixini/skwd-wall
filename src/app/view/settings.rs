use iced::Element;

use crate::frontend::settings;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(super) fn settings_layer(app: &App) -> Element<'_, Message> {
    crate::zone!("settings_layer");
    let source = app.settings_source_ctx();
    let focus = settings::FocusCtx {
        keyboard_focus: app.panels.settings.focus,
        focused_control: app.panels.settings.focused_control,
        focused_choice: app.panels.settings.focused_choice,
        expanded_details: &app.panels.settings.expanded_details,
        bar_reveals: &app.panels.settings.bar_reveals,
        active_input: app.panels.settings.input_edit.as_ref().map(|edit| edit.key.as_str()),
        armed: app.panels.settings.armed.as_ref(),
    };
    let chrome = settings::ChromeCtx {
        viewport: app.scene.viewport,
        scale: app.config.ui_scale(),
        entrance: app.panels.settings.entrance.x,
        palette: &app.theme.palette,
    };
    if settings::is_picker_layout_section(&app.panels.settings.tab, app.panels.settings.section) {
        return settings::picker_layout_workbench(&source, focus, chrome);
    }
    settings::settings_workbench(settings::WorkbenchInput {
        source,
        focus,
        chrome,
        profiles: settings::ProfilesCtx {
            selected: app.panels.settings.profile,
            filter: &app.panels.settings.profile_filter,
            enter: app.panels.settings.profile_enter.x,
            marker: app.panels.settings.profile_marker.x,
        },
        folders: settings::FoldersCtx {
            drafts: &app.panels.settings.folder_drafts,
            undo: app
                .panels
                .settings
                .folder_undo
                .as_ref()
                .map(|undo| (undo.kind, undo.path.as_str())),
        },
        preview_path: app.selected_settings_preview_path(),
        transition_preview: app.panels.transition_preview.handle.clone(),
        tab_transition: app.panels.settings.tab_anim.x,
        tab: &app.panels.settings.tab,
        selected_section: app.panels.settings.section,
        search_open: app.panels.settings.search_open,
        search_query: &app.panels.settings.search_query,
        search_results: &app.panels.settings.search_results,
        keybind_capture: keybind_capture_view(app),
        process_picker: process_picker_view(app),
        weather_review: weather_review_view(app),
    })
}

fn weather_review_view(app: &App) -> Option<settings::WeatherReviewView> {
    let review = &app.panels.settings;
    let category = review.weather_review_category?;
    let approved = crate::app::update::weather_review::approved(app, category);
    let catalog = app.library_session.library.catalog();
    let catalog_by_key = catalog
        .items
        .iter()
        .map(|wallpaper| (wallpaper.key.as_str(), wallpaper))
        .collect::<std::collections::HashMap<_, _>>();
    let items = crate::app::update::weather_review::visible_keys(app)
        .into_iter()
        .map(|key| {
            let wallpaper = catalog_by_key.get(key.as_str()).copied();
            settings::WeatherReviewItem {
                name: wallpaper.map_or_else(|| key.clone(), |wallpaper| wallpaper.name.clone()),
                thumbnail: wallpaper.map_or_else(String::new, |wallpaper| wallpaper.thumb.clone()),
                approved: approved.contains(&key),
                missing: wallpaper.is_none(),
                key,
            }
        })
        .collect();
    Some(settings::WeatherReviewView {
        category: category.to_owned(),
        tab: review.weather_review_tab,
        query: review.weather_review_query.clone(),
        items,
        loading: review.weather_review_loading,
        error: review.weather_review_error.clone(),
        focus: review.weather_review_focus,
        library_limited: review.weather_review_tab == settings::WeatherReviewTab::Library
            && catalog.items.len() > crate::app::update::weather_review::VISIBLE_LIMIT,
    })
}

fn process_picker_view(app: &App) -> Option<settings::ProcessPickerView> {
    app.panels.settings.process_picker_open.then(|| {
        let targets = app
            .config
            .str_path(skwd_config::keys::playback::PROCESSES)
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        let query = app.panels.settings.process_picker_query.to_ascii_lowercase();
        let mut running = app
            .daemon
            .playback
            .available_processes
            .iter()
            .map(|process| process_name(process))
            .filter(|process| !targets.iter().any(|target| target.eq_ignore_ascii_case(process)))
            .filter(|process| query.is_empty() || process.to_ascii_lowercase().contains(&query))
            .collect::<Vec<_>>();
        running.sort_unstable();
        running.dedup();
        settings::ProcessPickerView {
            targets,
            running,
            query: app.panels.settings.process_picker_query.clone(),
            manual: app.panels.settings.process_picker_manual.clone(),
            scroll: app.panels.settings.process_picker_scroll,
        }
    })
}

fn process_name(process: &str) -> String {
    process
        .split_whitespace()
        .next()
        .unwrap_or(process)
        .rsplit('/')
        .next()
        .unwrap_or(process)
        .to_string()
}

fn keybind_capture_view(app: &App) -> Option<settings::KeybindCaptureView> {
    let capture = app.panels.settings.keybind_capture.as_ref()?;
    let conflict = capture
        .triggers
        .iter()
        .find_map(|trigger| {
            app.input
                .bindings
                .trigger_holders(capture.action, trigger)
                .next()
                .map(|other| (other, trigger.label()))
        })
        .and_then(|(other, trigger)| {
            crate::contracts::picker::KEY_BINDINGS
                .into_iter()
                .find(|entry| entry.action == other)
                .map(|entry| {
                    crate::i18n::tr_args!(
                        "settings-keybind-capture-conflict",
                        binding => trigger,
                        action => crate::i18n::tr(entry.title_key)
                    )
                })
        });
    let binding = if capture.triggers.is_empty() {
        crate::i18n::tr("settings-keybind-unbound").to_string()
    } else {
        crate::domain::input::binding_label(&capture.triggers)
    };
    let action = crate::i18n::tr(capture.title_key);
    Some(settings::KeybindCaptureView {
        title: if capture.slot == 0 {
            action.to_string()
        } else {
            crate::i18n::tr_args!("settings-keybind-capture-second-title", action => action)
        },
        binding,
        edited: capture.edited,
        conflict,
    })
}
