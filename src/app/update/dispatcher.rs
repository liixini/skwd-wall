use std::time::Instant;

use iced::Task;

#[allow(clippy::wildcard_imports)]
use super::*;

const COPY_IMAGE_ERROR: &str = "card-back-copy-image-failed";
const COPY_PATH_ERROR: &str = "card-back-copy-path-failed";
const WORKSHOP_LINK_ERROR: &str = "browser-link-failed";

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    let output = app.runtime_state.overlay.and_then(crate::shell::picker_output);
    super::warm::sync_output(app, output);
    if (app.panels.strand.is_some()
        || app.panels.arrange.is_some() && !arrange::preview::active(app))
        && (steps_cards(&message)
            || matches!(
                &message,
                Message::ApplyCurrent
                    | Message::KeyFlip
                    | Message::KeyFavourite
                    | Message::CycleMode { .. }
            ))
    {
        return Task::none();
    }
    let detail_was_open = app.detail_open();
    let flip_was_open = app.scene.flip_open();
    let drawing_card_before = app.flipped_drawing_key().map(str::to_owned);
    let stashed = app.runtime_state.stashed;
    let steam_client_selected = app.config.steam_uses_client();
    let audio_view = app.panels.audio_view();
    let frame = matches!(&message, Message::Daemon(crate::infrastructure::runtime::Wake::Frame(_)));
    let sync_settings_preview = !frame
        && !matches!(
            &message,
            Message::Viewport(..)
                | Message::MouseMoved(..)
                | Message::MouseLeft
                | Message::KeyReleased(_)
        );
    let sync_filter_bar = !frame
        && !matches!(
            &message,
            Message::MouseMoved(..) | Message::MouseLeft | Message::KeyReleased(_)
        );
    app.input.held = false;
    let navigates = navigates(&message);
    let target_before = app.scene.theme_target();
    let task = update_inner(app, message);
    if navigates && app.scene.theme_target() != target_before {
        app.theme.armed = true;
    }
    let task = Task::batch([task, arrange::inventory::resume(app)]);
    let drawing_card_after = app.flipped_drawing_key().map(str::to_owned);
    if drawing_card_after.is_some() && (!flip_was_open || drawing_card_after != drawing_card_before)
    {
        app.scene.set_card_drawing(
            drawing_card_after.as_deref().expect("opened drawing card"),
            crate::frontend::scene::DrawingCardState::Checking,
        );
        app.request_drawing_status();
    }
    let subscribed_card_opened = app.scene.flip_open()
        && !flip_was_open
        && app
            .scene
            .flipped()
            .and_then(|index| app.library_session.filtered.get(index))
            .and_then(|&index| app.library_session.library.catalog().items.get(index as usize))
            .is_some_and(|item| item.steam_subscribed);
    if subscribed_card_opened
        || (stashed && !app.runtime_state.stashed)
        || steam_client_selected != app.config.steam_uses_client()
    {
        app.refresh_unsubscribe_status();
    }
    if audio_view != app.panels.audio_view() && !app.panels.audio_volumes.is_empty() {
        app.panels.audio_volumes.clear();
        app.call_tracked("wall.outputs", serde_json::json!({}), Pending::Outputs);
    }
    post_update::finish(app, task, detail_was_open, sync_settings_preview, sync_filter_bar)
}

fn steps_cards(message: &Message) -> bool {
    matches!(
        message,
        Message::Click(..)
            | Message::Wheel(_)
            | Message::KeyPrev
            | Message::KeyNext
            | Message::KeyUp
            | Message::KeyDown
            | Message::KeyPage { .. }
            | Message::KeyEdge { .. }
    )
}

fn navigates(message: &Message) -> bool {
    steps_cards(message) || matches!(message, Message::MouseMoved(..) | Message::KeyPressed(..))
}

pub(crate) fn update_inner(app: &mut App, message: Message) -> Task<Message> {
    if arrange::preview::active(app) && !arrange::preview::allows(&message) {
        return Task::none();
    }
    match message {
        Message::BrowserLinkOpened(result) => {
            if let Err(error) = result {
                app.show_toast(crate::i18n::tr_args!(WORKSHOP_LINK_ERROR, error => error));
            }
            Task::none()
        }
        Message::WindowOpened { id, width, height } => {
            super::warm::adopt_first_window(app, id, width, height)
        }
        Message::SddmChoices(result) => super::settings::sddm_choices(app, result),
        Message::SddmSetup { enabled, result } => super::settings::sddm_setup(app, enabled, result),
        Message::SddmRollback { error, result } => {
            super::settings::sddm_rollback_done(app, &error, result);
            Task::none()
        }
        Message::Daemon(crate::infrastructure::runtime::Wake::Toggle) => super::warm::toggle(app),
        Message::Stash => super::warm::stash(app),
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
            if !app.runtime_state.stashed && width > 0.0 && height > 0.0 {
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
            Task::batch([
                super::settings::sddm_rollback(app),
                super::settings::resume_weather_mode(app),
            ])
        }
        Message::Viewport(width, height) => {
            app.scene.viewport = (width, height);
            let output = app.runtime_state.picker_output.clone();
            super::arrange::placement::select(app, output);
            app.apply_layout();
            app.scene.touch();
            app.retick();
            Task::none()
        }
        Message::Arrange(message) => arrange::update(app, message),
        Message::MouseMoved(x, y) => picker::mouse_moved(app, x, y),
        Message::MouseLeft => {
            app.leave_filter_bar();
            Task::none()
        }
        Message::Click(x, y, button) => picker::click(app, x, y, button),
        Message::Wheel(amount) => navigation::wheel(app, amount),
        Message::KeyPrev => navigation::key_step(app, -1),
        Message::KeyNext => navigation::key_step(app, 1),
        Message::ApplyCurrent => navigation::apply_current(app),
        Message::KeyFavourite => navigation::key_favourite(app),
        Message::KeyFlip => navigation::key_flip(app),
        Message::KeyReveal => navigation::key_reveal(app),
        Message::KeyEffects => navigation::key_effects(app),
        Message::KeyStudio => navigation::key_studio(app),
        Message::KeyColourStudio => colour_studio::open(app),
        Message::Studio(msg) => colour_studio::update(app, &msg),
        Message::Strand(msg) => strand::update(app, msg),
        Message::OpenWidgets => strand::open(app),
        Message::StudioPixels(key, decoded) => {
            colour_studio::pixels(app, &key, decoded);
            Task::none()
        }
        Message::SetColorFilter(value) => filters::set_color_filter(app, value),
        Message::SetTypeFilter(kind) => {
            app.change_filters(|filters| filters.kind = kind);
            Task::none()
        }
        Message::SetFolder(folder) => filters::set_folder(app, folder),
        Message::CycleFolder { backwards } => filters::cycle_folder(app, backwards),
        Message::ToggleFolder => filters::toggle_folder(app),
        Message::ToggleShowHidden => filters::toggle_show_hidden(app),
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
        Message::CycleType { backwards } => filters::cycle_type(app, backwards),
        Message::CycleSort { backwards } => filters::cycle_sort(app, backwards),
        Message::ToggleRandomRotate => filters::toggle_random_rotate(app),
        Message::ApplyRandom => navigation::apply_random(app),
        Message::ToggleFilterBar => filters::toggle_filter_bar(app),
        Message::BarHover(hover, menu_hover) => {
            app.chrome.bar.hover = hover;
            app.chrome.bar.menu_hover = menu_hover;
            app.chrome.bar.cache.repaint();
            Task::none()
        }
        Message::FolderMenuToggle => {
            panels::toggle_bar_menu(app, crate::frontend::ui::MenuKind::Folders)
        }
        Message::FolderMenuScroll(delta) => panels::folder_menu_scroll(app, delta),
        Message::ToggleSettings => panels::toggle_settings(app),
        Message::ToggleAudio => panels::toggle_audio(app),
        Message::TaskControl { id, action } => {
            control_task(app, &id, action);
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
        Message::RevealInFiles(index) => picker::reveal_in_files(app, index),
        Message::Revealed(Ok(())) => crate::app::warm::stash(app),
        Message::CopyWallpaper { index, kind } => picker::copy_wallpaper(app, index, kind),
        Message::WallpaperCopied { kind, result } => {
            use crate::app::message::CopyKind;
            match (kind, result) {
                (CopyKind::Image, Ok(())) => {
                    app.show_toast(crate::i18n::tr("card-back-copy-image-done").to_owned());
                }
                (CopyKind::Path, Ok(())) => {
                    app.show_toast(crate::i18n::tr("card-back-copy-path-done").to_owned());
                }
                (CopyKind::Image, Err(error)) => {
                    app.show_toast(crate::i18n::tr_args!(COPY_IMAGE_ERROR, error => error));
                }
                (CopyKind::Path, Err(error)) => {
                    app.show_toast(crate::i18n::tr_args!(COPY_PATH_ERROR, error => error));
                }
            }
            Task::none()
        }
        Message::Revealed(Err(error)) => {
            app.show_toast(crate::i18n::tr_args!("card-back-reveal-failed", error => error));
            Task::none()
        }
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
            app.scene.hand_drag_end();
            if app.panels.schedule.as_ref().is_some_and(|ed| ed.drag.is_some()) {
                return update_inner(
                    app,
                    Message::Sched(crate::frontend::schedule_editor::SchedMsg::DragEnd),
                );
            }
            Task::none()
        }
        Message::KeyReleased(key) => {
            if app.input.down.as_ref() == Some(&key) {
                app.input.down = None;
            }
            if app.input.quiet.as_ref() == Some(&key) {
                app.input.quiet = None;
            }
            Task::none()
        }
        Message::WindowUnfocused => {
            app.input.down = None;
            app.input.quiet = None;
            if app.panels.settings.keybind_capture.take().is_some() {
                app.retick();
            }
            Task::none()
        }
        Message::KeyPressed(key, modifiers) => {
            app.input.held = app.input.down.replace(key.clone()).as_ref() == Some(&key);
            if app.input.held && app.input.quiet.as_ref() == Some(&key) {
                return Task::none();
            }
            if app.panels.strand.is_some() {
                return strand::key(app, &key, modifiers);
            }
            if app.panels.arrange.is_some()
                && (!arrange::preview::active(app)
                    || matches!(
                        key,
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)
                    ))
            {
                return arrange::key(app, &key, modifiers);
            }
            if app.panels.colour_studio.is_some() {
                return colour_studio::key(app, &key, modifiers);
            }
            if app.panels.theme_designer.is_some() {
                if matches!(key, iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)) {
                    return update_inner(app, Message::Exit);
                }
                return Task::none();
            }
            if app.panels.settings.open && app.panels.settings.keybind_capture.is_some() {
                if app.input.held {
                    return Task::none();
                }
                return settings::keybind_capture_key(app, &key, modifiers);
            }
            if app.panels.settings.open
                && let Some(message) = super::input::settings_key_message(&key, modifiers)
            {
                if app.input.held
                    && matches!(
                        message,
                        Message::Settings(crate::frontend::settings::SettingsMsg::Key(
                            crate::frontend::settings::SettingsKey::Activate
                                | crate::frontend::settings::SettingsKey::Space
                        ))
                    )
                {
                    return Task::none();
                }
                return update_inner(app, message);
            }
            if app.tags.cloud_open
                && matches!(&key, iced::keyboard::Key::Character(_))
                && !modifiers.control()
                && !modifiers.alt()
            {
                return iced::widget::operation::focus(tag_query_id());
            }
            let Some(message) =
                super::input::key_message(&app.input.bindings, &key, modifiers, app.input_scopes())
            else {
                return Task::none();
            };
            update_inner(app, message)
        }
        Message::ToggleEffects => effects::toggle_effects(app),
        Message::Effects(message) => effects::update(app, message),
        Message::Sched(message) => schedule::update(app, message),
        Message::SetViewMode(mode) => settings::set_view_mode(app, &mode),
        Message::CycleMode { backwards } => settings::cycle_view_mode(app, backwards),
        Message::Settings(message) => {
            let changes = message.changes_config();
            let task = settings::update(app, message);
            if changes {
                settings::reread_search(app);
            }
            task
        }
        Message::FolderChosen(kind, index, result) => {
            super::settings_folders::folder_chosen(app, kind, index, result)
        }
        Message::ConfigurationExported(result) => settings::configuration_exported(app, result),
        Message::ConfigurationImported(result) => settings::configuration_imported(app, result),
        Message::SemanticModelImported(result) => settings::semantic_model_imported(app, result),
        Message::DrawingPackChosen(result) => settings::drawing_pack_chosen(app, result),
        Message::SemanticModelDeleted(manifest, selected, result) => {
            settings::semantic_model_deleted(app, &manifest, selected.as_deref(), result)
        }
        Message::SettingsPreviewAllocated(path, allocation) => {
            app.finish_settings_preview(&path, allocation);
            Task::none()
        }
        Message::SetSettingsTab(tab) => settings::set_settings_tab(app, &tab),
        Message::KeyUp => navigation::key_row(app, -1),
        Message::KeyDown => navigation::key_row(app, 1),
        Message::KeyPage { backwards } => navigation::key_page(app, backwards),
        Message::KeyEdge { last } => navigation::key_edge(app, last),
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
        Message::ToggleDownloads => browser::toggle_downloads(app),
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

pub(super) fn control_task(app: &App, id: &str, action: crate::contracts::daemon::TaskControl) {
    app.daemon.client.call(
        wall_proto::rpc::TASK_CONTROL,
        serde_json::json!({"id": id, "action": action.as_str()}),
    );
}
