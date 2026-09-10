use iced::Element;

use crate::frontend::settings;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(super) fn settings_layer(app: &App) -> Element<'_, Message> {
    crate::zone!("settings_layer");
    let source = settings::SourceCtx {
        config: &app.config,
        values: &app.panels.settings.inputs,
        themes: &app.daemon.effect_themes,
        folders: &app.library_session.folder_options,
        backends: app.theme.backends.as_deref().unwrap_or(&[]),
        outputs: &app.daemon.output_names,
        output_statuses: &app.daemon.output_statuses,
        output_previews: &app.daemon.output_wallpaper_art,
        library_watch: app.daemon.library_watch.as_ref(),
        playback: Some(&app.daemon.playback),
        thumbnail_task: app.daemon.tasks.values().find(|task| task.id == "we-thumbnails"),
        analysis: app.panels.settings.semantic_import_status.clone(),
    };
    let focus = settings::FocusCtx {
        keyboard_focus: app.panels.settings.focus,
        focused_control: app.panels.settings.focused_control,
        focused_choice: app.panels.settings.focused_choice,
        expanded_details: &app.panels.settings.expanded_details,
        bar_reveals: &app.panels.settings.bar_reveals,
        active_input: app.panels.settings.input_edit.as_ref().map(|edit| edit.key.as_str()),
        armed: app.panels.settings.armed,
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
        preview_path: app.selected_settings_preview_path(),
        transition_preview: app.panels.transition_preview.handle.clone(),
        tab_transition: app.panels.settings.tab_anim.x,
        tab: &app.panels.settings.tab,
        selected_section: app.panels.settings.section,
        search_open: app.panels.settings.search_open,
        search_query: &app.panels.settings.search_query,
        search_results: &app.panels.settings.search_results,
        keybind_capture: keybind_capture_view(app),
    })
}

fn keybind_capture_view(app: &App) -> Option<settings::KeybindCaptureView> {
    let capture = app.panels.settings.keybind_capture.as_ref()?;
    let conflict = capture
        .triggers
        .iter()
        .find_map(|trigger| {
            app.input
                .bindings
                .shared_trigger(capture.action, trigger)
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
    Some(settings::KeybindCaptureView {
        title: crate::i18n::tr(capture.title_key).to_string(),
        binding,
        edited: capture.edited,
        conflict,
    })
}
