use std::time::Instant;

use iced::Task;

#[allow(clippy::wildcard_imports)]
use super::*;

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    let detail_was_open = app.detail_open();
    let frame = matches!(&message, Message::Daemon(crate::infrastructure::runtime::Wake::Frame(_)));
    let sync_settings_preview =
        !frame && !matches!(&message, Message::Viewport(..) | Message::MouseMoved(..));
    let sync_filter_bar = !frame && !matches!(&message, Message::MouseMoved(..));
    let task = update_inner(app, message);
    post_update::finish(app, task, detail_was_open, sync_settings_preview, sync_filter_bar)
}

pub(crate) fn update_inner(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::WindowOpened { id, width, height } => {
            super::warm::adopt_first_window(app, id, width, height)
        }
        Message::Daemon(crate::infrastructure::runtime::Wake::Toggle) => super::warm::toggle(app),
        Message::Daemon(crate::infrastructure::runtime::Wake::Hide) => {
            super::warm::exit_picker(app)
        }
        Message::Daemon(crate::infrastructure::runtime::Wake::Command(command)) => {
            run_ui_command(app, &command)
        }
        Message::Daemon(crate::infrastructure::runtime::Wake::Query(_, reply)) => {
            app.retick();
            reply.send(&ui_state_json(app));
            Task::none()
        }
        Message::Daemon(crate::infrastructure::runtime::Wake::Frame(sent_at)) => {
            let (width, height) = app.scene.viewport;
            if width > 0.0 && height > 0.0 {
                let now = Instant::now().max(sent_at);
                lifecycle::tick(app, now, width, height)
            } else {
                Task::none()
            }
        }
        Message::Daemon(wake) => {
            if app.handle_wake(wake) {
                app.retick();
            }
            Task::none()
        }
        Message::Viewport(width, height) => {
            app.scene.viewport = (width, height);
            app.scene.touch();
            app.retick();
            Task::none()
        }
        Message::MouseMoved(x, y) => picker::mouse_moved(app, x, y),
        Message::Click(x, y, button) => picker::click(app, x, y, button),
        Message::Wheel(amount) => navigation::wheel(app, amount),
        Message::KeyPrev => navigation::key_prev(app),
        Message::KeyNext => navigation::key_next(app),
        Message::ApplyCurrent => navigation::apply_current(app),
        Message::KeyFavourite => navigation::key_favourite(app),
        Message::KeyFlip => navigation::key_flip(app),
        Message::KeyEffects => navigation::key_effects(app),
        Message::KeyStudio => navigation::key_studio(app),
        Message::SetColorFilter(value) => filters::set_color_filter(app, value),
        Message::SetTypeFilter(kind) => {
            app.change_filters(|filters| filters.kind = kind);
            Task::none()
        }
        Message::SetFolder(folder) => filters::set_folder(app, folder),
        Message::CycleFolder { backwards } => filters::cycle_folder(app, backwards),
        Message::ToggleFolder => filters::toggle_folder(app),
        Message::ToggleHiddenFolders => filters::toggle_hidden_folders(app),
        Message::SetSort(sort) => {
            if app.tags.search_mode == SearchMode::Describe
                && !app.tags.semantic.search.trim().is_empty()
            {
                app.tags.semantic.sort_override = true;
            }
            app.change_filters(|filters| filters.sort = sort);
            Task::none()
        }
        Message::SetOrient(orientation) => {
            let active = app.library_session.filters.resolution.clone();
            let incompatible = !active.is_empty()
                && app
                    .config
                    .resolution_presets()
                    .iter()
                    .find(|preset| preset.key() == active)
                    .is_some_and(|preset| !preset.matches_shape(&orientation));
            app.change_filters(|filters| {
                filters.orient = orientation;
                if incompatible {
                    filters.resolution.clear();
                }
            });
            Task::none()
        }
        Message::SetResolution(resolution) => {
            app.change_filters(|filters| filters.resolution = resolution);
            Task::none()
        }
        Message::ToggleFavourites => {
            app.change_filters(|filters| filters.favourites_only = !filters.favourites_only);
            Task::none()
        }
        Message::ToggleRandomRotate => filters::toggle_random_rotate(app),
        Message::ToggleFilterBar => filters::toggle_filter_bar(app),
        Message::BarHover(hover, menu_hover) => {
            app.chrome.bar.hover = hover;
            app.chrome.bar.menu_hover = menu_hover;
            app.chrome.bar.cache.clear();
            Task::none()
        }
        Message::FolderMenuToggle => {
            panels::toggle_bar_menu(app, crate::frontend::ui::MenuKind::Folders)
        }
        Message::FolderMenuScroll(delta) => panels::folder_menu_scroll(app, delta),
        Message::ToggleSettings => panels::toggle_settings(app),
        Message::ToggleAudio => panels::toggle_audio(app),
        Message::TaskControl { id, action } => {
            app.daemon
                .client
                .call("task.control", serde_json::json!({"id": id, "action": action.as_str()}));
            Task::none()
        }
        Message::Audio(message) => effects::audio_update(app, message),
        Message::ToggleThemePanel => panels::toggle_theme_panel(app),
        Message::OpenThemeAudition => theme::open_theme_audition(app),
        Message::CloseThemeAudition => {
            if app.theme.audition_focused {
                super::warm::exit_picker(app);
            }
            app.close_overlay(crate::app::overlay::Overlay::ThemeAudition);
            Task::none()
        }
        Message::ThemeAuditionBackend(backend) => theme::inspect_theme_audition(app, &backend),
        Message::ThemeAuditionSelect { backend, key, value } => {
            theme::select_theme_audition(app, &backend, &key, &value)
        }
        Message::Theme(message) => theme::update(app, message),
        Message::SceneProps(message) => {
            scene_properties::update(app, message);
            Task::none()
        }
        Message::ResetThumbnail(index) => picker::reset_thumbnail(app, index),
        Message::OpenSceneProps => {
            app.open_scene_properties();
            Task::none()
        }
        Message::ToggleAudioPanel => effects::toggle_audio_panel(app),
        Message::MonFill(output, mode) => effects::mon_fill(app, &output, &mode),
        Message::SetMods(mods) => {
            app.input.mods = mods;
            Task::none()
        }
        Message::PointerUp => {
            if app.panels.schedule.as_ref().is_some_and(|ed| ed.drag.is_some()) {
                return update_inner(
                    app,
                    Message::Sched(crate::frontend::schedule_editor::SchedMsg::DragEnd),
                );
            }
            Task::none()
        }
        Message::KeyPressed(key, modifiers) => {
            if app.panels.theme_designer.is_some() {
                if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)) {
                    return update_inner(app, Message::Exit);
                }
                return Task::none();
            }
            if app.panels.settings.open && app.panels.settings.keybind_capture.is_some() {
                return settings::keybind_capture_key(app, &key, modifiers);
            }
            if app.panels.settings.open
                && let Some(message) = super::input::settings_key_message(&key, modifiers)
            {
                return update_inner(app, message);
            }
            if app.tags.cloud_open && matches!(&key, iced::keyboard::Key::Character(_)) {
                return iced::widget::operation::focus(tag_query_id());
            }
            let Some(message) = super::input::key_message(&app.input.bindings, &key, modifiers)
            else {
                return Task::none();
            };
            update_inner(app, message)
        }
        Message::ToggleEffects => effects::toggle_effects(app),
        Message::Effects(message) => effects::update(app, message),
        Message::Sched(message) => schedule::update(app, message),
        Message::SetViewMode(mode) => settings::set_view_mode(app, &mode),
        Message::Settings(message) => settings::update(app, message),
        Message::SemanticModelImported(result) => settings::semantic_model_imported(app, result),
        Message::SettingsPreviewAllocated(path, allocation) => {
            app.finish_settings_preview(&path, allocation);
            Task::none()
        }
        Message::SetSettingsTab(tab) => settings::set_settings_tab(app, tab),
        Message::ApplyPreset(mode, name) => settings::apply_preset(app, &mode, &name),
        Message::SavePreset => settings::save_preset(app),
        Message::PresetNameInput(text) => settings::preset_name_input(app, &text),
        Message::DeletePreset(mode, name) => settings::delete_preset(app, &mode, &name),
        Message::KeyUp => navigation::key_up(app),
        Message::KeyDown => navigation::key_down(app),
        Message::Tag(message) => tags::update(app, message),
        Message::ToggleTagMode => tags::toggle_tag_mode(app),
        Message::MassTagInput(text) => tags::mass_tag_input(app, text),
        Message::MassTagAdd(tag) => tags::mass_tag_add(app, &tag),
        Message::MassTagRemove(index) => tags::mass_tag_remove(app, index),
        Message::MassTagApply => {
            commit_mass_tags(app);
            Task::none()
        }
        Message::OpenTagCloud => tags::open_tag_cloud(app),
        Message::CloseTagCloud => {
            app.close_overlay(crate::app::overlay::Overlay::TagCloud);
            Task::none()
        }
        Message::PaneWheel(key, delta) => panels::pane_wheel(app, key, delta),
        Message::PaneScrolled(key, position, max) => panels::pane_scrolled(app, key, position, max),
        Message::ToggleHelp => panels::toggle_help(app),
        Message::ClearTags => tags::clear_tags(app),
        Message::OpenSourceBrowser => {
            let source = app.source_browser.last_source.key();
            browser::open_browser(app, source)
        }
        Message::OpenBrowser(source) => browser::open_browser(app, &source),
        Message::CloseBrowser => browser::close_browser(app),
        Message::OpenPlaylists => playlists::open_playlists(app),
        Message::ClosePlaylists => playlists::close_playlists(app),
        Message::Pl(message) => playlists::update(app, message),
        Message::CardPicker(message) => playlists::card_picker_update(app, message),
        Message::Browser(message) => browser::update(app, message),
        Message::Noop => Task::none(),
        Message::Exit => lifecycle::exit(app),
        #[cfg(target_os = "linux")]
        other => {
            log::warn!("unhandled message: {other:?}");
            Task::none()
        }
    }
}
