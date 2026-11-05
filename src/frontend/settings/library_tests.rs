use super::*;
use crate::frontend::settings::RuntimeStatus;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source, plain};

const ROOT: &str = "/walls";

fn watched(mode: &str) -> LibraryWatchStatus {
    LibraryWatchStatus { ok: true, mode: mode.to_string(), ..LibraryWatchStatus::default() }
}

fn rooted(cfg: FakeSettingsSource) -> FakeSettingsSource {
    cfg.with_array_len(library::FOLDERS, 1).with_text(&format!("{}.0", library::FOLDERS), ROOT)
}

fn say_of(section: &Section) -> String {
    plain(&section.say.text)
}

fn folders_say(cfg: &FakeSettingsSource, watch: Option<&LibraryWatchStatus>) -> Section {
    let runtime = RuntimeStatus { library_watch: watch, ..RuntimeStatus::default() };
    let source = PlaceSource { runtime, ..place_source(cfg) };
    crate::i18n::with_language("en-US", || {
        compose(&source).into_iter().next().expect("folders section")
    })
}

#[test]
fn says_folder_states() {
    let empty = FakeSettingsSource::default().with_array_len(library::FOLDERS, 0);
    let none = folders_say(&empty, None);
    assert_eq!(say_of(&none), "No wallpaper folders yet. Add one to start building the library.");
    assert!(none.head.is_empty());

    let on = rooted(FakeSettingsSource::default()).with_flag(library::POLLING_FALLBACK, true);
    let polling = folders_say(&on, Some(&watched(LibraryWatchMode::NATIVE)));
    assert_eq!(
        say_of(&polling),
        "Scanning 1 folder. Polling covers what native watching misses, every [0]."
    );
    assert_eq!(polling.head.len(), 1);
    assert!(polling.say.notes.is_empty());

    let off = rooted(FakeSettingsSource::default()).with_flag(library::POLLING_FALLBACK, false);
    let native = folders_say(&off, Some(&watched(LibraryWatchMode::NATIVE)));
    assert_eq!(
        say_of(&native),
        "Scanning 1 folder. Native file watching is active; polling is idle."
    );
    assert!(native.head.is_empty());
    let down = folders_say(&off, Some(&watched(LibraryWatchMode::UNAVAILABLE)));
    assert_eq!(say_of(&down), "Scanning 1 folder. Native watching failed and polling is off.");
    assert_eq!(down.say.notes.len(), 1, "status keeps a home");
}

#[test]
fn counts_unread_folders() {
    let root = |error: Option<&str>| crate::contracts::daemon::LibraryWatchRootStatus {
        path: ROOT.to_string(),
        mode: LibraryWatchMode::POLLING.to_string(),
        last_poll_error: error.map(str::to_string),
        ..Default::default()
    };
    let watch = LibraryWatchStatus {
        mode: LibraryWatchMode::POLLING.to_string(),
        roots: vec![root(Some("gone")), root(None)],
        ..LibraryWatchStatus::default()
    };
    let cfg = rooted(FakeSettingsSource::default()).with_flag(library::POLLING_FALLBACK, true);
    assert!(say_of(&folders_say(&cfg, Some(&watch))).ends_with(" One folder cannot be read."));
}

fn storage_of(cfg: &FakeSettingsSource) -> Section {
    crate::i18n::with_language("en-US", || {
        compose(&place_source(cfg)).into_iter().nth(3).expect("storage")
    })
}

#[test]
fn storage_reads_flags() {
    let cfg = FakeSettingsSource::default()
        .with_flag(perf::AUTO_OPTIMIZE_IMAGES, false)
        .with_flag(perf::AUTO_DELETE_IMAGE_TRASH, false)
        .with_flag(effects::AUTO_RECOLOR, false);
    let off = storage_of(&cfg);
    assert_eq!(
        say_of(&off),
        "New images stay as they are. Optimise all uses [0] quality, capped at [1]. \
         Trashed originals stay until you delete them. New wallpapers are not recoloured."
    );
    let on = storage_of(
        &FakeSettingsSource::default()
            .with_flag(perf::AUTO_OPTIMIZE_IMAGES, true)
            .with_flag(perf::AUTO_DELETE_IMAGE_TRASH, true)
            .with_flag(effects::AUTO_RECOLOR, true),
    );
    assert_eq!(
        say_of(&on),
        "New images become WebP at [0] quality, capped at [1]. \
         Trashed originals stay for [2], then are deleted. New wallpapers also get a recoloured copy."
    );
}

#[test]
fn retention_stays_reachable() {
    let path = |row: &Row| row.control.path().map(str::to_string);
    let shut =
        storage_of(&FakeSettingsSource::default().with_flag(perf::AUTO_DELETE_IMAGE_TRASH, false));
    assert!(!shut.head.iter().any(|row| path(row).as_deref() == Some(perf::IMAGE_TRASH_DAYS)));
    assert!(shut.body().any(|row| path(row).as_deref() == Some(perf::IMAGE_TRASH_DAYS)));
    let open =
        storage_of(&FakeSettingsSource::default().with_flag(perf::AUTO_DELETE_IMAGE_TRASH, true));
    assert!(open.head.iter().any(|row| path(row).as_deref() == Some(perf::IMAGE_TRASH_DAYS)));
    assert!(!open.groups[0].echoes.is_empty(), "echoed in the body");
}

#[test]
fn theme_parks_without_recolour() {
    let off = storage_of(&FakeSettingsSource::default().with_flag(effects::AUTO_RECOLOR, false));
    let at = off.rows().position(|row| row.control.path() == Some(effects::AUTO_THEME));
    assert!(off.say.parks(at.expect("theme stays findable")).is_some());
    let on = storage_of(&FakeSettingsSource::default().with_flag(effects::AUTO_RECOLOR, true));
    assert!(on.body().any(|row| row.control.path() == Some(effects::AUTO_THEME)));
}

fn outside_section(scopes: Vec<crate::contracts::settings::ExclusionScope>) -> Section {
    let count = scopes.len();
    let mut cfg = FakeSettingsSource::default()
        .with_array_len(library::EXCLUDED_FOLDERS, count)
        .with_exclusion_scopes(scopes);
    for at in 0..count {
        cfg = cfg.with_text(&format!("{}.{at}", library::EXCLUDED_FOLDERS), "/x");
    }
    crate::i18n::with_language("en-US", || excluded(&cfg))
}

#[test]
fn excluded_warns_outside() {
    use crate::contracts::settings::ExclusionScope::{Everywhere, Outside};
    assert_eq!(say_of(&outside_section(Vec::new())), "Nothing is excluded.");
    assert_eq!(say_of(&outside_section(vec![Everywhere])), "Never scanning 1 folder.");
    assert_eq!(
        say_of(&outside_section(vec![Everywhere, Outside])),
        "Never scanning 2 folders. One is outside every wallpaper folder and skips nothing."
    );
}

fn with_profile(active: &str) -> FakeSettingsSource {
    FakeSettingsSource::default()
        .with_array_len(library::PROFILES, 1)
        .with_text("library.profiles.0.id", "public")
        .with_text("library.profiles.0.name", "Public")
        .with_text(library::PROFILE, active)
        .with_profile_state(0, ROOT, ProfileState::Hide)
}

#[test]
fn active_menu_lists() {
    let folders = [ROOT.to_string()];
    let say = |active: &str| {
        let cfg = with_profile(active);
        let source = PlaceSource { folders: &folders, ..place_source(&cfg) };
        crate::i18n::with_language("en-US", || profiles(&source))
    };
    let everything = say("");
    assert!(
        say_of(&everything).starts_with("Active profile [0], which shows every folder. Ctrl+H")
    );
    let Control::Dropdown { options, current, .. } = &everything.head[0].control else {
        panic!("menu");
    };
    assert_eq!(current, EVERYTHING_PROFILE);
    assert_eq!(
        options.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
        ["everything", "public"]
    );
    assert!(say_of(&say("public")).starts_with("Active profile [0]: 1 hidden, 0 excluded. Ctrl+H"));
    assert!(say("gone").head[0].control.unlisted(), "stale id reads custom");
}

fn ctx<'a>(
    cfg: &'a FakeSettingsSource,
    folders: &'a [String],
    selected: usize,
    drafts: &'a std::collections::HashMap<&'static str, String>,
    folds: &'a HashSet<String>,
) -> PageCtx<'a> {
    PageCtx {
        cfg,
        folders,
        watch: None,
        profiles: ProfilesCtx { selected, filter: "" },
        lists: FoldersCtx { drafts, undo: None },
        folds,
    }
}

#[test]
fn folder_lines_order() {
    let cfg = rooted(FakeSettingsSource::default());
    let (mut drafts, folds) = (std::collections::HashMap::new(), HashSet::new());
    let view = ctx(&cfg, &[], 0, &drafts, &folds);
    let id = SectionId::new(LIBRARY, FOLDERS);
    let kinds = |rows: Vec<Row>| {
        rows.iter()
            .map(|row| match &row.control {
                Control::TextField { .. } => "text",
                Control::ActionBtn { id: ActionId::AddFolder(_), .. } => "add",
                Control::ActionBtn { .. } => "action",
                _ => "other",
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(kinds(page_rows(id, &view)), ["text", "action", "action", "text", "action"]);
    drafts.insert(FolderKind::Source.key(), "/more".to_string());
    let view = ctx(&cfg, &[], 0, &drafts, &folds);
    assert_eq!(kinds(page_rows(id, &view)), ["text", "action", "action", "text", "action", "add"]);
}

#[test]
fn collapsed_hides_children() {
    let folders = [ROOT.to_string(), format!("{ROOT}/anime")];
    let cfg = with_profile("public");
    let drafts = std::collections::HashMap::new();
    let mut folds = HashSet::new();
    let id = SectionId::new(LIBRARY, PROFILES);
    let states = |folds: &HashSet<String>| {
        let view = ctx(&cfg, &folders, 1, &drafts, folds);
        page_rows(id, &view)
            .iter()
            .filter(|row| matches!(row.control, Control::Chips { .. }))
            .count()
    };
    assert_eq!(states(&folds), 3, "root, child, workshop");
    let view = ctx(&cfg, &folders, 1, &drafts, &folds);
    let root = profile_page(&view).tree[0].id.clone();
    folds.insert(root);
    assert_eq!(states(&folds), 2, "child folded away");
}

#[test]
fn no_value_blank() {
    let plain = FakeSettingsSource::default().with_text(effects::AUTO_THEME, "");
    let nameless = FakeSettingsSource::default()
        .with_array_len(library::PROFILES, 1)
        .with_flag(effects::AUTO_RECOLOR, true);
    for cfg in [plain, nameless] {
        let sections = crate::i18n::with_language("en-US", || compose(&place_source(&cfg)));
        for row in sections.iter().flat_map(Section::rows) {
            if let Control::Dropdown { .. } | Control::Chips { .. } = row.control {
                let shown = row.control.current_label();
                assert!(shown.is_none_or(|text| !text.is_empty()), "{}", row.title);
            }
        }
    }
    let cfg = rooted(FakeSettingsSource::default());
    let (drafts, folds) = (std::collections::HashMap::new(), HashSet::new());
    let view = ctx(&cfg, &[], 1, &drafts, &folds);
    for key in [FOLDERS, EXCLUDED, PROFILES] {
        for row in
            crate::i18n::with_language("en-US", || page_rows(SectionId::new(LIBRARY, key), &view))
        {
            if let Control::TextField { placeholder, .. } = row.control {
                assert!(!placeholder.is_empty(), "{key}");
            }
        }
    }
}

#[test]
fn links_reach_sections() {
    let cfg = FakeSettingsSource::default();
    let sections = crate::i18n::with_language("en-US", || compose(&place_source(&cfg)));
    assert_eq!(sections[EXCLUDED_SECTION].id.key, EXCLUDED);
    assert_eq!(sections[PROFILES_SECTION].id.key, PROFILES);
}

#[test]
fn everything_stays_locked() {
    let folders = [ROOT.to_string()];
    let cfg = with_profile("public");
    let (drafts, folds) = (std::collections::HashMap::new(), HashSet::new());
    let view = ctx(&cfg, &folders, 0, &drafts, &folds);
    let rows = page_rows(SectionId::new(LIBRARY, PROFILES), &view);
    assert!(!rows.iter().any(|row| matches!(row.control, Control::Chips { .. })));
    assert!(!rows.iter().any(|row| matches!(
        row.control,
        Control::ActionBtn { id: ActionId::DeleteProfile(_), .. }
    )));
}
