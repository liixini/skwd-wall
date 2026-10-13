use std::path::PathBuf;

use iced::Task;
use serde_json::json;

use crate::app::{App, Message};
use crate::contracts::settings::FolderKind;
use crate::domain::library::filter::Filters;
use crate::infrastructure::file_picker::{self, FilePicker};

const FOLDER_PICKER_ERROR_KEY: &str = "settings-paths-folder-picker-error";

fn folders(app: &App, kind: FolderKind) -> Vec<String> {
    match kind {
        FolderKind::Source => {
            crate::contracts::settings::SettingsSource::library_folders(&app.config)
        }
        FolderKind::Excluded => app
            .config
            .array_values(kind.key())
            .iter()
            .filter_map(|folder| folder.as_str().map(str::to_owned))
            .collect(),
    }
}

pub(super) fn add_folder(app: &mut App, kind: FolderKind) -> Task<Message> {
    if app.panels.settings.folder_picking {
        return Task::none();
    }
    let draft = app.panels.settings.folder_drafts.remove(kind.key()).unwrap_or_default();
    let path = draft.trim();
    if !path.is_empty() {
        append_folder(app, kind, path);
    }
    Task::none()
}

fn append_folder(app: &mut App, kind: FolderKind, path: &str) {
    let mut folders = folders(app, kind);
    if let Some(existing) = folders.iter().position(|folder| same_folder(kind, folder, path)) {
        flash(app, kind, existing);
        return;
    }
    folders.push(path.to_owned());
    app.panels.settings.folder_undo = None;
    save_folders(app, kind, &folders);
    flash(app, kind, folders.len() - 1);
    if kind == FolderKind::Source {
        reveal_source_folder(app, path);
    }
}

fn replace_folder(app: &mut App, kind: FolderKind, index: usize, path: &str) {
    let mut folders = folders(app, kind);
    let taken = folders
        .iter()
        .enumerate()
        .any(|(other, folder)| other != index && same_folder(kind, folder, path));
    let Some(folder) = folders.get_mut(index).filter(|folder| !same_folder(kind, folder, path))
    else {
        return;
    };
    if taken && kind == FolderKind::Source {
        return;
    }
    path.clone_into(folder);
    app.panels.settings.folder_undo = None;
    save_folders(app, kind, &folders);
    if kind == FolderKind::Source {
        reveal_source_folder(app, path);
    }
}

fn same_folder(kind: FolderKind, folder: &str, other: &str) -> bool {
    match kind {
        FolderKind::Source => normalized_folder(folder) == normalized_folder(other),
        FolderKind::Excluded => folder == other,
    }
}

fn flash(app: &mut App, kind: FolderKind, index: usize) {
    let key = crate::frontend::settings::flash_key(&format!("{}.{index}", kind.key()));
    let motion = app.motion_profile();
    app.panels.settings.flash(key, motion);
    app.retick();
}

pub(super) fn restore_folder(app: &mut App) -> Task<Message> {
    let Some(undo) = app.panels.settings.folder_undo.take() else {
        return Task::none();
    };
    let mut folders = folders(app, undo.kind);
    if folders.iter().any(|folder| same_folder(undo.kind, folder, &undo.path)) {
        return Task::none();
    }
    let index = undo.index.min(folders.len());
    folders.insert(index, undo.path);
    save_folders(app, undo.kind, &folders);
    flash(app, undo.kind, index);
    Task::none()
}

pub(super) fn choose_folder(
    app: &mut App,
    kind: FolderKind,
    index: Option<usize>,
) -> Task<Message> {
    if app.panels.settings.folder_picking {
        return Task::none();
    }
    let title = crate::i18n::tr("settings-paths-folder-choose-action").to_owned();
    let start = index.and_then(|index| folders(app, kind).get(index).map(PathBuf::from));
    app.panels.settings.folder_picking = true;
    let choose = iced_runtime::task::blocking(move |mut sender| {
        let result = file_picker::choose(&FilePicker {
            title: &title,
            directory: true,
            filters: &[],
            start: start.as_deref(),
        });
        let _ = sender.try_send(Message::FolderChosen(kind, index, result));
    });
    crate::app::warm::stash(app).chain(choose)
}

pub(super) fn folder_chosen(
    app: &mut App,
    kind: FolderKind,
    index: Option<usize>,
    result: Result<Option<PathBuf>, String>,
) -> Task<Message> {
    app.panels.settings.folder_picking = false;
    match result {
        Ok(Some(path)) => {
            let path = path.to_string_lossy().into_owned();
            match index {
                Some(index) => replace_folder(app, kind, index, &path),
                None => append_folder(app, kind, &path),
            }
        }
        Ok(None) => {}
        Err(error) => {
            app.show_toast(crate::i18n::tr_args!(FOLDER_PICKER_ERROR_KEY, error => error));
        }
    }
    crate::app::warm::unstash(app)
}

pub(super) fn remove_folder(app: &mut App, kind: FolderKind, index: usize) -> Task<Message> {
    if app.panels.settings.folder_picking {
        return Task::none();
    }
    let mut folders = folders(app, kind);
    if index < folders.len() {
        let removed = folders.remove(index);
        let selected = kind == FolderKind::Source
            && normalized_folder(&removed).as_deref()
                == Some(app.library_session.filters.folder.as_str());
        if selected {
            app.change_filters(|filters| filters.folder = Filters::default().folder);
        }
        save_folders(app, kind, &folders);
        app.panels.settings.folder_undo =
            Some(crate::app::state::FolderUndo { kind, index, path: removed });
    }
    Task::none()
}

pub(super) fn reveal_committed_folder(app: &mut App, key: &str, original: &str) {
    let Some(index) = key
        .strip_prefix(FolderKind::Source.key())
        .and_then(|suffix| suffix.strip_prefix('.'))
        .and_then(|index| index.parse::<usize>().ok())
    else {
        return;
    };
    let sources = folders(app, FolderKind::Source);
    let Some(path) = sources.get(index) else { return };
    if path.is_empty()
        || normalized_folder(path) == normalized_folder(original)
        || sources
            .iter()
            .filter(|folder| normalized_folder(folder) == normalized_folder(path))
            .count()
            != 1
    {
        return;
    }
    let path = path.clone();
    reveal_source_folder(app, &path);
}

fn reveal_source_folder(app: &mut App, path: &str) {
    let Some(folder) = normalized_folder(path) else { return };
    let sort = app.library_session.filters.sort.clone();
    app.library_session.playlist_filter = None;
    app.tags.tag_search.clear();
    app.tags.semantic.search.clear();
    app.clear_semantic_search();
    app.change_filters(|filters| *filters = Filters { folder, sort, ..Filters::default() });
    app.rebuild_folder_options();
    app.close_settings();
}

fn normalized_folder(path: &str) -> Option<String> {
    (!path.trim().is_empty()).then(|| crate::app::settings_source::resolved_folder(path))
}

pub(super) fn edit_folder(app: &mut App, key: &str, raw: &str) -> bool {
    for kind in [FolderKind::Source, FolderKind::Excluded] {
        let Some(index) = key
            .strip_prefix(kind.key())
            .and_then(|suffix| suffix.strip_prefix('.'))
            .and_then(|index| index.parse::<usize>().ok())
        else {
            continue;
        };
        let mut folders = folders(app, kind);
        if let Some(folder) = folders.get_mut(index) {
            raw.clone_into(folder);
            app.config.set_key(kind.key(), json!(folders));
        }
        return true;
    }
    false
}

fn save_folders(app: &mut App, kind: FolderKind, folders: &[String]) {
    app.config.save_key(kind.key(), json!(folders));
    if kind == FolderKind::Source {
        app.rebuild_folder_options();
        app.refilter_from_start();
    }
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
}
