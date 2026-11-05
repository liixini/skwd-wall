#[cfg(test)]
#[path = "library_tests.rs"]
mod tests;

use std::collections::HashSet;

use crate::contracts::daemon::{LibraryWatchMode, LibraryWatchStatus};
use crate::contracts::settings::{
    EVERYTHING_PROFILE, FolderKind, ProfileState, SettingsSource, keys,
};
use crate::i18n::{text, tr, tr_args};

use super::category::LIBRARY;
use super::places::{Bind, NO_SLOTS, PlaceSource, bind, column, noted, park, rows, spoken, take};
use super::tabs::{take_card, take_control_paths};
use super::{
    ActionId, Control, FoldersCtx, Group, GroupKind, Note, PageSection, ProfilesCtx, ProfilesPage,
    Row, STATES, Section, SectionId, TreeNode, Trim, mark_folds, page_section, profile_commands,
    profiles_page,
};

use keys::effects;
use keys::library;
use keys::performance as perf;

pub(super) const FOLDERS: &str = "settings-section-folders";
pub(super) const EXCLUDED: &str = "settings-paths-excluded-card";
pub(super) const PROFILES: &str = "settings-profiles-card";
pub(super) const STORAGE: &str = "settings-library-storage";
pub(super) const THUMBNAILS: &str = "settings-library-thumbnails";

pub const FOLDER_LIST: &str = "settings-library-folder-list";
pub const TREE_LIST: &str = "settings-library-tree-list";
pub const QUERY_FOLD: &str = "query:";
pub const FILTER_KEY: &str = "library.profileFilter";

const EXCLUDED_SECTION: usize = 1;
const PROFILES_SECTION: usize = 2;
const DRAFT_SUFFIX: &str = ".draft";
const SPACER: &str = "\u{a0}";

const WATCHING_CARD: &str = "settings-section-library-watching";
const IMAGES_CARD: &str = "settings-section-images-recovery";
const VARIANTS_CARD: &str = "settings-section-generated-variants";
const THUMBS_CARD: &str = "settings-section-thumbnails-cache";

const FOLDERS_TITLE: &str = "settings-paths-source-card";
const FOLDERS_DESC: &str = "settings-library-folders-desc";
const FOLDERS_SAY: &str = "settings-library-folders-say";
const FOLDERS_EMPTY: &str = "settings-paths-source-empty";
const UNREAD: &str = "settings-library-unread";
const WATCHING: &str = "settings-library-watching";
const POLL_HINT: &str = "settings-library-poll-hint";
const INTERVAL_HINT: &str = "settings-library-interval-hint";
const COUNT: &str = "settings-paths-count";
const EXCLUDED_DESC: &str = "settings-library-excluded-desc";
const EXCLUDED_SAY: &str = "settings-library-excluded-say";
const EXCLUDED_EMPTY: &str = "settings-paths-excluded-empty";
const OUTSIDE: &str = "settings-library-outside";
const PROFILES_DESC: &str = "settings-library-profiles-desc";
const PROFILES_SAY: &str = "settings-library-profiles-say";
const ACTIVE: &str = "settings-library-active";
const NO_RULES: &str = "settings-profiles-summary-none";
const NO_VALUE: &str = "settings-control-none";
const STORAGE_DESC: &str = "settings-library-storage-desc";
const STORAGE_SAY: &str = "settings-library-storage-say";
const RETENTION_HINT: &str = "settings-library-retention-hint";
const OPTIMISE_HINT: &str = "settings-library-optimise-hint";
const TRASH: &str = "settings-performance-trash-card";
const RECOLOUR: &str = "settings-library-recolour";
const THUMBS_DESC: &str = "settings-library-thumbnails-desc";
const THUMBS_SAY: &str = "settings-library-thumbnails-say";
const REBUILD: &str = "settings-library-rebuild";
const START_OVER: &str = "settings-library-start-over";
const RECOMPUTE_HINT: &str = "settings-library-recompute-hint";
const CLEAR_HINT: &str = "settings-library-clear-hint";
const CHOOSE: &str = "settings-paths-folder-choose-action";
const REMOVE: &str = "settings-paths-remove-action";
const ADD: &str = "settings-paths-add-action";
const UNDO: &str = "settings-paths-undo-action";
const FOLDER_BLANK: &str = "settings-paths-empty-path";
const SOURCE_DRAFT: &str = "settings-paths-source-draft";
const EXCLUDED_DRAFT: &str = "settings-paths-excluded-draft";
const PROFILE_NAME: &str = "settings-profiles-name-placeholder";
const FILTER_BLANK: &str = "settings-profiles-filter-placeholder";
const NEW_PROFILE: &str = "settings-profiles-add-label";
const COPY: &str = "settings-profiles-copy-action";
const DELETE: &str = "settings-profiles-delete-action";
const MAKE_ACTIVE: &str = "settings-profiles-make-active";

const STATE_ARG: &str = "state";
const FOLDERS_ARG: &str = "folders";
const UNREAD_ARG: &str = "unread";
const OUTSIDE_ARG: &str = "outside";
const SUMMARY_ARG: &str = "summary";
const OPTIMISE_ARG: &str = "optimise";
const TRASH_ARG: &str = "trash";
const RECOLOUR_ARG: &str = "recolour";
const POLLING: &str = "polling";
const NATIVE: &str = "native";
const DOWN: &str = "down";
const UNKNOWN: &str = "unknown";
const NONE: &str = "none";
const SOME: &str = "some";
const ON: &str = "on";
const OFF: &str = "off";

const EVERY_SLOT: Bind<'_> = Bind::new("every", library::POLLING_INTERVAL_SECONDS);
const MENU_SLOT: Bind<'_> = Bind::new("menu", library::PROFILE);
const QUALITY_SLOT: Bind<'_> = Bind::new("quality", perf::IMAGE_OPTIMIZE_PRESET);
const SIZE_SLOT: Bind<'_> = Bind::new("size", perf::IMAGE_OPTIMIZE_RESOLUTION);
const DAYS_SLOT: Bind<'_> = Bind::new("days", perf::IMAGE_TRASH_DAYS);
const JOBS_SLOT: Bind<'_> = Bind::new("jobs", perf::MAX_THUMB_JOBS);

#[derive(Clone, Copy)]
pub struct PageCtx<'a> {
    pub cfg: &'a dyn SettingsSource,
    pub folders: &'a [String],
    pub watch: Option<&'a LibraryWatchStatus>,
    pub profiles: ProfilesCtx<'a>,
    pub lists: FoldersCtx<'a>,
    pub folds: &'a HashSet<String>,
}

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let mut cards = source.cards(LIBRARY);
    let watching = take_card(&mut cards, text(WATCHING_CARD));
    let images = take_card(&mut cards, text(IMAGES_CARD));
    let variants = take_card(&mut cards, text(VARIANTS_CARD));
    let thumbs = take_card(&mut cards, text(THUMBS_CARD));
    vec![
        folders(source, watching),
        excluded(source.cfg),
        profiles(source),
        storage(source.cfg, images, variants),
        thumbnails(thumbs),
    ]
}

fn counted(count: usize) -> String {
    tr_args!(COUNT, count => count)
}

fn unread(watch: Option<&LibraryWatchStatus>) -> usize {
    watch.map_or(0, |watch| {
        let failed = |root: &&crate::contracts::daemon::LibraryWatchRootStatus| {
            root.last_poll_error.is_some() || root.mode == LibraryWatchMode::UNAVAILABLE
        };
        watch.roots.iter().filter(failed).count()
    })
}

fn plainly_native(watch: Option<&LibraryWatchStatus>) -> bool {
    watch.is_some_and(|watch| {
        watch.mode == LibraryWatchMode::NATIVE
            && watch.last_successful_convergence_unix_ms.is_none()
    })
}

fn folders(source: &PlaceSource<'_>, mut pool: Vec<Row>) -> Section {
    let cfg = source.cfg;
    let watch = source.runtime.library_watch;
    let status = take(&mut pool, |control| matches!(control, Control::Static));
    let fallback = rows(&mut pool, &[(library::POLLING_FALLBACK, Some(POLL_HINT))]);
    let interval = rows(&mut pool, &[(library::POLLING_INTERVAL_SECONDS, Some(INTERVAL_HINT))]);
    let count = cfg.library_folders().len();
    let polling = cfg.flag(library::POLLING_FALLBACK);
    let bound = polling && count > 0;
    let mut paired = Group::new(GroupKind::Column, Some(SPACER.to_string()), Vec::new());
    let head = if bound {
        paired.echoes = interval.first().cloned().map(|row| (0, row)).into_iter().collect();
        noted(interval, None)
    } else {
        paired.rows = interval;
        Vec::new()
    };
    let section = Section {
        head,
        groups: vec![column(Some(WATCHING), fallback), paired],
        ..Section::new(SectionId::new(LIBRARY, FOLDERS), tr(FOLDERS_TITLE), tr(FOLDERS_DESC))
    };
    let state = if polling {
        POLLING
    } else if plainly_native(watch) {
        NATIVE
    } else if watch.is_some_and(|watch| watch.mode == LibraryWatchMode::UNAVAILABLE) {
        DOWN
    } else {
        UNKNOWN
    };
    let failed = unread(watch);
    let reads =
        if failed == 0 || !polling { String::new() } else { tr_args!(UNREAD, count => failed) };
    let shown = counted(count);
    let mut section = spoken(section, |section| {
        if count == 0 {
            return bind(section, FOLDERS_EMPTY, &NO_SLOTS, &[]);
        }
        let slots = if bound { &[EVERY_SLOT][..] } else { &[] };
        let args =
            [(STATE_ARG, state), (FOLDERS_ARG, shown.as_str()), (UNREAD_ARG, reads.as_str())];
        bind(section, FOLDERS_SAY, slots, &args)
    });
    if count > 0 && !plainly_native(watch) {
        section.say.notes.extend(status.first().map(super::places::note));
    }
    section
}

fn excluded_paths(cfg: &dyn SettingsSource) -> Vec<String> {
    (0..cfg.array_len(library::EXCLUDED_FOLDERS))
        .map(|at| cfg.text(&format!("{}.{at}", library::EXCLUDED_FOLDERS)))
        .collect()
}

fn excluded(cfg: &dyn SettingsSource) -> Section {
    let count = excluded_paths(cfg).len();
    let outside = cfg
        .exclusion_scopes()
        .iter()
        .filter(|scope| matches!(scope, crate::contracts::settings::ExclusionScope::Outside))
        .count();
    let warned = if outside == 0 { String::new() } else { tr_args!(OUTSIDE, count => outside) };
    let shown = counted(count);
    let section = Section {
        trim: Trim::Bottom,
        ..Section::new(SectionId::new(LIBRARY, EXCLUDED), tr(EXCLUDED), tr(EXCLUDED_DESC))
    };
    spoken(section, |section| {
        if count == 0 {
            return bind(section, EXCLUDED_EMPTY, &NO_SLOTS, &[]);
        }
        let args = [(FOLDERS_ARG, shown.as_str()), (OUTSIDE_ARG, warned.as_str())];
        bind(section, EXCLUDED_SAY, &NO_SLOTS, &args)
    })
}

fn active_id(cfg: &dyn SettingsSource) -> String {
    let active = cfg.text(library::PROFILE);
    if active.is_empty() { EVERYTHING_PROFILE.to_string() } else { active }
}

fn profiles(source: &PlaceSource<'_>) -> Section {
    let cfg = source.cfg;
    let current = active_id(cfg);
    let listed = profiles_page(cfg, source.folders, 0, "");
    let at = listed.profiles.iter().position(|entry| entry.id == current).unwrap_or(0);
    let page = profiles_page(cfg, source.folders, at, "");
    let options = page
        .profiles
        .iter()
        .map(|entry| {
            let name = if entry.name.is_empty() { &entry.id } else { &entry.name };
            (entry.id.clone(), worded(name))
        })
        .collect();
    let menu = Row::of(
        tr(ACTIVE).to_string(),
        String::new(),
        Control::Dropdown {
            path: library::PROFILE.to_string(),
            options,
            current,
            palettes: Vec::new(),
        },
    );
    let section = Section {
        head: vec![menu],
        ..Section::new(SectionId::new(LIBRARY, PROFILES), tr(PROFILES), tr(PROFILES_DESC))
    };
    let state = if page.summary == tr(NO_RULES) { NONE } else { SOME };
    let args = [(STATE_ARG, state), (SUMMARY_ARG, page.summary.as_str())];
    spoken(section, |section| bind(section, PROFILES_SAY, &[MENU_SLOT], &args))
}

fn worded(label: &str) -> String {
    if label.is_empty() { tr(NO_VALUE).to_string() } else { label.to_string() }
}

fn acted(pool: &mut Vec<Row>, id: &ActionId) -> Vec<Row> {
    take(pool, |control| matches!(control, Control::ActionBtn { id: found, .. } if found == id))
}

fn storage(cfg: &dyn SettingsSource, mut images: Vec<Row>, mut variants: Vec<Row>) -> Section {
    let master = take_control_paths(&mut images, &[perf::AUTO_OPTIMIZE_IMAGES]).pop();
    let mut head = take_control_paths(
        &mut images,
        &[perf::IMAGE_OPTIMIZE_PRESET, perf::IMAGE_OPTIMIZE_RESOLUTION],
    );
    let deletes = cfg.flag(perf::AUTO_DELETE_IMAGE_TRASH);
    let recolours = cfg.flag(effects::AUTO_RECOLOR);
    let days = rows(&mut images, &[(perf::IMAGE_TRASH_DAYS, Some(RETENTION_HINT))]);
    let mut trash = rows(&mut images, &[(perf::AUTO_DELETE_IMAGE_TRASH, None)]);
    let mut echoes = Vec::new();
    if deletes {
        echoes = days.first().cloned().map(|row| (1, row)).into_iter().collect();
        head.extend(noted(days, None));
    } else {
        trash.extend(days);
    }
    trash.extend(noted(acted(&mut images, &ActionId::OptimizeImages), Some(OPTIMISE_HINT)));
    let mut recolour = rows(&mut variants, &[(effects::AUTO_RECOLOR, None)]);
    let mut theme = rows(&mut variants, &[(effects::AUTO_THEME, None)]);
    for row in &mut theme {
        if let Control::Dropdown { options, .. } = &mut row.control {
            for (_, label) in options {
                *label = worded(label);
            }
        }
    }
    if recolours {
        recolour.extend(theme);
    } else {
        head.extend(theme);
    }
    let mut bin = column(Some(TRASH), trash);
    bin.echoes = echoes;
    let section = Section {
        master,
        head,
        groups: vec![bin, column(Some(RECOLOUR), recolour)],
        ..Section::new(SectionId::new(LIBRARY, STORAGE), tr(STORAGE), tr(STORAGE_DESC))
    };
    let switch = |on: bool| if on { ON } else { OFF };
    let args = [
        (OPTIMISE_ARG, switch(cfg.flag(perf::AUTO_OPTIMIZE_IMAGES))),
        (TRASH_ARG, switch(deletes)),
        (RECOLOUR_ARG, switch(recolours)),
    ];
    let mut slots = vec![QUALITY_SLOT, SIZE_SLOT];
    if deletes {
        slots.push(DAYS_SLOT);
    }
    let mut section = spoken(section, |section| bind(section, STORAGE_SAY, &slots, &args));
    if !recolours {
        park(&mut section, effects::AUTO_RECOLOR, |row| {
            row.control.path() == Some(effects::AUTO_THEME)
        });
    }
    section
}

fn thumbnails(mut pool: Vec<Row>) -> Section {
    let head = noted(take_control_paths(&mut pool, &[perf::MAX_THUMB_JOBS]), None);
    let mut rebuild = noted(acted(&mut pool, &ActionId::RecomputeColors), Some(RECOMPUTE_HINT));
    let capture = acted(&mut pool, &ActionId::CaptureWeThumbnails);
    rebuild.extend(capture.into_iter().map(|row| Row {
        desc: String::new(),
        look: super::Look { note: Note::Desc, ..row.look },
        ..row
    }));
    let start_over = noted(acted(&mut pool, &ActionId::ClearCache), Some(CLEAR_HINT));
    let section = Section {
        head,
        groups: vec![column(Some(REBUILD), rebuild), column(Some(START_OVER), start_over)],
        ..Section::new(SectionId::new(LIBRARY, THUMBNAILS), tr(THUMBNAILS), tr(THUMBS_DESC))
    };
    spoken(section, |section| bind(section, THUMBS_SAY, &[JOBS_SLOT], &[]))
}

pub fn draft_key(kind: FolderKind) -> String {
    format!("{}{DRAFT_SUFFIX}", kind.key())
}

pub fn folder_draft(key: &str) -> Option<FolderKind> {
    [FolderKind::Source, FolderKind::Excluded].into_iter().find(|kind| key == draft_key(*kind))
}

pub fn state_position(state: ProfileState) -> f32 {
    STATES.iter().position(|(candidate, _)| *candidate == state).map_or(0.0, |at| at as f32)
}

fn field(key: String, placeholder: &'static str) -> Control {
    Control::TextField { path: key.clone(), key, placeholder, secret: false }
}

fn line(control: Control) -> Row {
    Row::of(String::new(), String::new(), control)
}

fn action(id: ActionId, label: &str) -> Row {
    line(Control::ActionBtn { id, label: label.to_string() })
}

pub fn page_rows(section: SectionId, ctx: &PageCtx<'_>) -> Vec<Row> {
    match page_section(section) {
        Some(PageSection::Folders(kind)) => folder_rows(kind, ctx),
        Some(PageSection::Profiles) => profile_rows(ctx),
        None => Vec::new(),
    }
}

fn folder_rows(kind: FolderKind, ctx: &PageCtx<'_>) -> Vec<Row> {
    let ledger = super::folder_ledger(ctx.cfg, kind, ctx.watch);
    let mut rows = Vec::new();
    for folder in ledger.rows {
        rows.push(line(field(folder.key, tr(FOLDER_BLANK))));
        rows.push(action(ActionId::ChooseFolder(kind, Some(folder.index)), tr(CHOOSE)));
        rows.push(action(ActionId::RemoveFolder(kind, folder.index), tr(REMOVE)));
    }
    if ctx.lists.undo.is_some_and(|(undone, _)| undone == kind) {
        rows.push(action(ActionId::UndoFolderRemove, tr(UNDO)));
    }
    let blank = match kind {
        FolderKind::Source => SOURCE_DRAFT,
        FolderKind::Excluded => EXCLUDED_DRAFT,
    };
    rows.push(line(field(draft_key(kind), tr(blank))));
    rows.push(action(ActionId::ChooseFolder(kind, None), tr(CHOOSE)));
    if ctx.lists.drafts.get(kind.key()).is_some_and(|draft| !draft.trim().is_empty()) {
        rows.push(action(ActionId::AddFolder(kind), tr(ADD)));
    }
    if kind == FolderKind::Excluded {
        rows.push(action(ActionId::OpenSection(PROFILES_SECTION), tr(PROFILES)));
    }
    rows
}

fn fold_key(node: &TreeNode, filtering: bool) -> (String, bool) {
    if filtering {
        (format!("{QUERY_FOLD}{}", node.id), true)
    } else {
        (node.id.clone(), node.open_by_default())
    }
}

pub(crate) fn profile_page(ctx: &PageCtx<'_>) -> ProfilesPage {
    let filter = ctx.profiles.filter;
    let mut page = profiles_page(ctx.cfg, ctx.folders, ctx.profiles.selected, filter);
    if filter.trim().is_empty() {
        mark_folds(&mut page.tree, ctx.folds);
    } else {
        for node in &mut page.tree {
            node.collapsed = node.children && ctx.folds.contains(&fold_key(node, true).0);
        }
    }
    let mut hidden: Option<u8> = None;
    page.tree.retain(|node| {
        if hidden.is_some_and(|depth| node.depth > depth) {
            return false;
        }
        hidden = node.collapsed.then_some(node.depth);
        true
    });
    page
}

fn states(path: &str, state: ProfileState) -> Control {
    let options = STATES.map(|(state, key)| (state.key().to_string(), tr(key).to_string()));
    Control::Chips {
        path: path.to_string(),
        options: options.into(),
        current: state.key().to_string(),
        disabled: Vec::new(),
        background: None,
    }
}

fn profile_rows(ctx: &PageCtx<'_>) -> Vec<Row> {
    let page = profile_page(ctx);
    let entry = page.current();
    let filtering = !ctx.profiles.filter.trim().is_empty();
    let mut rows: Vec<Row> = page
        .profiles
        .iter()
        .enumerate()
        .map(|(at, profile)| action(ActionId::SelectProfile(at), &profile.name))
        .collect();
    rows.push(action(ActionId::NewProfile, tr(NEW_PROFILE)));
    if let Some(index) = entry.index {
        for (helm, command) in [true, false].into_iter().zip(profile_commands(&entry.id)) {
            let copied = ActionId::CopyProfileCommand(index as u16, helm);
            rows.push(Row { title: command, ..action(copied, tr(COPY)) });
        }
        rows.push(action(ActionId::DeleteProfile(index as u16), tr(DELETE)));
    }
    if let Some(key) = entry.name_key() {
        rows.push(line(field(key, tr(PROFILE_NAME))));
    }
    if !entry.active {
        rows.push(action(ActionId::ActivateProfile(entry.id.clone()), tr(MAKE_ACTIVE)));
    }
    rows.push(line(field(FILTER_KEY.to_string(), tr(FILTER_BLANK))));
    for node in &page.tree {
        if node.children {
            let (key, open) = fold_key(node, filtering);
            rows.push(action(ActionId::FoldTree(key, open), &node.label));
        }
        match node.state {
            None => rows.push(action(ActionId::OpenSection(EXCLUDED_SECTION), tr(EXCLUDED))),
            Some(state) if !node.path.is_empty() => {
                rows.push(Row { title: node.label.clone(), ..line(states(&node.path, state)) });
            }
            Some(_) => {}
        }
    }
    rows
}
