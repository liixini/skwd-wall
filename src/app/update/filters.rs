use iced::Task;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(super) fn set_folder(app: &mut App, folder: String) -> Task<Message> {
    app.chrome.bar.menu = None;
    app.chrome.bar.menu_scroll.snap(0.0);
    app.library_session.playlist_filter = None;
    app.change_filters(|flt| flt.folder = folder);
    Task::none()
}

pub(super) fn cycle_folder(app: &mut App, backwards: bool) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    let options = &app.library_session.folder_options;
    if options.is_empty() {
        return Task::none();
    }
    let current = options.iter().position(|folder| *folder == app.library_session.filters.folder);
    let index = current.map_or(0, |index| {
        if backwards {
            (index + options.len() - 1) % options.len()
        } else {
            (index + 1) % options.len()
        }
    });
    set_folder(app, options[index].clone())
}

pub(super) fn toggle_folder(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    let folder = if app.library_session.filters.folder == "*" { "" } else { "*" };
    set_folder(app, folder.into())
}

pub(super) fn toggle_hidden_folders(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    app.close_flip_after_removal();
    app.change_filters(|filters| {
        filters.show_hidden_folders = !filters.show_hidden_folders;
        if !filters.folder_visible(&filters.folder) {
            filters.folder = "*".into();
        }
    });
    app.rebuild_folder_options();
    app.show_toast(crate::i18n::tr(if app.library_session.filters.show_hidden_folders {
        "status-hidden-folders-shown"
    } else {
        "status-hidden-folders-hidden"
    }));
    Task::none()
}

pub(super) fn set_color_filter(app: &mut App, val: i64) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    let color = match val {
        i64::MIN => crate::frontend::ui::cycle_color_left(app.library_session.filters.color),
        i64::MAX => crate::frontend::ui::cycle_color_right(app.library_session.filters.color),
        other => other,
    };
    app.change_filters(|flt| flt.color = color);
    Task::none()
}

pub(super) fn toggle_random_rotate(app: &mut App) -> Task<Message> {
    let on = !app.config.flag_default_config(skwd_config::keys::general::RANDOM_ROTATE);
    app.config.set_key(skwd_config::keys::general::RANDOM_ROTATE, serde_json::json!(on));
    app.config.persist();
    app.daemon.client.call("wall.rotation_wake", serde_json::json!({}));
    app.show_toast(crate::i18n::tr(if on {
        "status-random-rotation-on"
    } else {
        "status-random-rotation-off"
    }));
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}

pub(super) fn toggle_filter_bar(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    app.chrome.filter_bar_visible = !app.chrome.filter_bar_visible;
    app.chrome.bar.menu = None;
    app.chrome.bar.cache.clear();
    Task::none()
}
