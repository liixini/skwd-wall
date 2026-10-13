use super::Builder;
use crate::contracts::daemon::{LibraryWatchMode, LibraryWatchStatus};
use crate::contracts::settings::{ExclusionScope, FolderKind, SettingsSource};
use crate::frontend::settings::{FolderLedger, FolderRow, TagTone};
use crate::i18n::{tr, tr_args};

const FLASH_PREFIX: &str = "flash:";

pub fn flash_key(key: &str) -> String {
    format!("{FLASH_PREFIX}{key}")
}

pub(super) fn tab_paths(builder: &mut Builder<'_>) {
    for kind in [FolderKind::Source, FolderKind::Excluded] {
        let (card, desc) = match kind {
            FolderKind::Source => {
                (tr("settings-paths-source-card"), tr("settings-paths-source-card-desc"))
            }
            FolderKind::Excluded => {
                (tr("settings-paths-excluded-card"), tr("settings-paths-excluded-card-desc"))
            }
        };
        builder.card(card, desc);
    }
}

pub fn folder_ledger(
    cfg: &dyn SettingsSource,
    kind: FolderKind,
    watch: Option<&LibraryWatchStatus>,
) -> FolderLedger {
    let roots = cfg.library_folders();
    let scopes = cfg.exclusion_scopes();
    let paths = match kind {
        FolderKind::Source => roots.clone(),
        FolderKind::Excluded => (0..cfg.array_len(kind.key()))
            .map(|index| cfg.text(&format!("{}.{index}", kind.key())))
            .collect(),
    };
    let rows = paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let (tag, note) = match kind {
                FolderKind::Source => watch_tag(cfg, &path, watch),
                FolderKind::Excluded => (scope_tag(scopes.get(index), &roots), None),
            };
            FolderRow { index, key: format!("{}.{index}", kind.key()), path, tag, note }
        })
        .collect();
    FolderLedger { kind, rows }
}

type Tag = Option<(String, TagTone)>;

fn watch_tag(
    cfg: &dyn SettingsSource,
    path: &str,
    watch: Option<&LibraryWatchStatus>,
) -> (Tag, Option<String>) {
    let resolved = cfg.resolved_folder(path);
    let Some(root) = watch.and_then(|watch| watch.roots.iter().find(|root| root.path == resolved))
    else {
        return (None, None);
    };
    if root.last_poll_error.is_some() || root.mode == LibraryWatchMode::UNAVAILABLE {
        return (
            Some((tr("settings-paths-watch-error").to_owned(), TagTone::Warn)),
            Some(tr("settings-paths-watch-error-note").to_owned()),
        );
    }
    if root.mode == LibraryWatchMode::POLLING || root.mode == LibraryWatchMode::RECOVERING {
        return (Some((tr("settings-paths-watch-polling").to_owned(), TagTone::Info)), None);
    }
    (None, None)
}

fn scope_tag(scope: Option<&ExclusionScope>, roots: &[String]) -> Tag {
    match scope? {
        ExclusionScope::Empty => None,
        ExclusionScope::Everywhere => {
            Some((tr("settings-paths-excluded-everywhere").to_owned(), TagTone::Quiet))
        }
        ExclusionScope::Outside => {
            Some((tr("settings-paths-excluded-outside").to_owned(), TagTone::Warn))
        }
        ExclusionScope::Inside(root) => {
            let short = crate::frontend::ui::folder_short_label(root, roots);
            Some((
                tr_args!("settings-paths-excluded-inside", root => short.as_str()),
                TagTone::Quiet,
            ))
        }
    }
}
