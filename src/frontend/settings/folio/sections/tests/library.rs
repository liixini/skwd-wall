use std::collections::{HashMap, HashSet};

use crate::contracts::daemon::{LibraryWatchMode, LibraryWatchRootStatus, LibraryWatchStatus};
use crate::contracts::settings::{ExclusionScope, FolderKind, ProfileState, keys};
use crate::frontend::settings::test_source::FakeSettingsSource;
use crate::frontend::settings::{ActionId, Control, FoldersCtx, PageCtx, ProfilesCtx, SectionId};

use super::super::{TokenSide, token_key};
use super::{FitState, LIBRARY, Shot, capture_live, localized};

const FOLDERS: &str = "settings-section-folders";
const EXCLUDED: &str = "settings-paths-excluded-card";
const PROFILES: &str = "settings-profiles-card";
const STORAGE: &str = "settings-library-storage";
const THUMBS: &str = "settings-library-thumbnails";
const ROOT: &str = "/home/someone/Pictures/walls";
const NAS: &str = "/mnt/nas/art";
const USB: &str = "/run/media/someone/Backup/walls";

#[derive(Clone, Copy, Default)]
pub(super) struct Shelf<'a> {
    pub folders: &'a [String],
    pub watch: Option<&'a LibraryWatchStatus>,
    pub selected: usize,
    pub filter: &'a str,
    pub undo: Option<(FolderKind, &'a str)>,
    pub draft: Option<(FolderKind, &'a str)>,
    pub armed: Option<&'a ActionId>,
}

pub(super) fn seed(
    cfg: &dyn crate::contracts::settings::SettingsSource,
    shelf: Shelf<'_>,
    values: &mut HashMap<String, String>,
) {
    let drafts = HashMap::new();
    let folds = HashSet::new();
    let ctx = PageCtx {
        cfg,
        folders: shelf.folders,
        watch: shelf.watch,
        profiles: ProfilesCtx { selected: shelf.selected, filter: shelf.filter },
        lists: FoldersCtx { drafts: &drafts, undo: shelf.undo },
        folds: &folds,
    };
    for key in [FOLDERS, EXCLUDED, PROFILES] {
        let id = SectionId::new(crate::frontend::settings::category::LIBRARY, key);
        for row in crate::frontend::settings::page_rows(id, &ctx) {
            if let Control::TextField { key, path, .. } = row.control {
                values.insert(key, cfg.text(&path));
            }
        }
    }
    if let Some((kind, draft)) = shelf.draft {
        values.insert(crate::frontend::settings::draft_key(kind), draft.to_string());
    }
}

fn leaked<T>(value: T) -> &'static T {
    Box::leak(Box::new(value))
}

fn tree() -> Vec<String> {
    let mut tree = vec![ROOT.to_string()];
    for leaf in ["anime", ".private", "city", "nature", "space", "minimal", "games"] {
        tree.push(format!("{ROOT}/{leaf}"));
    }
    tree.push(NAS.to_string());
    for leaf in ["figure", "nsfw"] {
        tree.push(format!("{NAS}/{leaf}"));
    }
    tree.push(USB.to_string());
    tree
}

fn listed(cfg: FakeSettingsSource, path: &str, items: &[&str]) -> FakeSettingsSource {
    let mut cfg = cfg.with_array_len(path, items.len());
    for (at, item) in items.iter().enumerate() {
        cfg = cfg.with_text(&format!("{path}.{at}"), item);
    }
    cfg
}

fn profile(cfg: FakeSettingsSource, at: usize, id: &str, name: &str) -> FakeSettingsSource {
    let base = format!("{}.{at}", keys::library::PROFILES);
    cfg.with_text(&format!("{base}.{}", keys::library::profile::ID), id)
        .with_text(&format!("{base}.{}", keys::library::profile::NAME), name)
}

fn shelved() -> FakeSettingsSource {
    let roots = [
        ROOT,
        "/home/someone/Videos/walls",
        NAS,
        USB,
        "/home/someone/Downloads/art",
        "/srv/share/photography/wallpapers",
    ];
    let excluded = [".cache", "thumbs", "/tmp/old-walls", "/home/someone/Pictures/walls/drafts"];
    let cfg = listed(FakeSettingsSource::default(), keys::library::FOLDERS, &roots);
    let cfg = listed(cfg, keys::library::EXCLUDED_FOLDERS, &excluded).with_exclusion_scopes(vec![
        ExclusionScope::Everywhere,
        ExclusionScope::Everywhere,
        ExclusionScope::Outside,
        ExclusionScope::Inside(ROOT.to_string()),
    ]);
    let cfg = cfg.with_array_len(keys::library::PROFILES, 3);
    let cfg = profile(cfg, 0, "public", "Public");
    let cfg = profile(cfg, 1, "streaming", "Streaming");
    let cfg = profile(cfg, 2, "family-room-tv", "Family room TV");
    cfg.with_text(keys::library::PROFILE, "public")
        .with_flag(keys::library::POLLING_FALLBACK, true)
        .with_flag(keys::performance::AUTO_OPTIMIZE_IMAGES, true)
        .with_profile_state(0, &format!("{ROOT}/anime"), ProfileState::Hide)
        .with_profile_state(0, &format!("{ROOT}/.private"), ProfileState::Exclude)
        .with_profile_state(0, &format!("{NAS}/nsfw"), ProfileState::Exclude)
        .with_profile_state(0, &format!("{NAS}/figure"), ProfileState::Hide)
}

const CONVERGED_AGO_MS: u64 = 125_000;

fn polling() -> LibraryWatchStatus {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |value| u64::try_from(value.as_millis()).unwrap_or_default());
    LibraryWatchStatus {
        ok: true,
        mode: LibraryWatchMode::POLLING.to_string(),
        interval_seconds: Some(60),
        entry_budget_per_root: Some(4096),
        last_successful_convergence_unix_ms: Some(now_ms.saturating_sub(CONVERGED_AGO_MS)),
        roots: vec![
            LibraryWatchRootStatus {
                path: NAS.to_string(),
                mode: LibraryWatchMode::POLLING.to_string(),
                ..LibraryWatchRootStatus::default()
            },
            LibraryWatchRootStatus {
                path: USB.to_string(),
                mode: LibraryWatchMode::POLLING.to_string(),
                last_poll_error: Some("gone".to_string()),
                ..LibraryWatchRootStatus::default()
            },
        ],
        ..LibraryWatchStatus::default()
    }
}

fn watching(mode: &str) -> LibraryWatchStatus {
    LibraryWatchStatus { ok: true, mode: mode.to_string(), ..LibraryWatchStatus::default() }
}

struct State {
    name: &'static str,
    cfg: fn() -> FakeSettingsSource,
    shelf: fn() -> Shelf<'static>,
}

fn stocked() -> Shelf<'static> {
    Shelf { folders: leaked(tree()), watch: Some(leaked(polling())), ..Shelf::default() }
}

const TABLE: [State; 13] = [
    State { name: "first-run", cfg: FakeSettingsSource::default, shelf: Shelf::default },
    State {
        name: "empty",
        cfg: || FakeSettingsSource::default().with_array_len(keys::library::FOLDERS, 0),
        shelf: Shelf::default,
    },
    State {
        name: "long-names",
        cfg: || {
            profile(
                shelved(),
                1,
                "guests",
                "The family room television profile for guests and visitors",
            )
            .with_text(keys::library::PROFILE, "guests")
        },
        shelf: || Shelf { selected: 2, ..stocked() },
    },
    State {
        name: "four-unread",
        cfg: || {
            let roots = [ROOT, "/home/someone/Videos/walls", NAS, USB];
            listed(shelved(), keys::library::FOLDERS, &roots)
        },
        shelf: stocked,
    },
    State { name: "stocked", cfg: shelved, shelf: || Shelf { selected: 1, ..stocked() } },
    State {
        name: "undo-draft",
        cfg: shelved,
        shelf: || Shelf {
            undo: Some((FolderKind::Source, "/home/someone/Pictures/old-walls")),
            draft: Some((FolderKind::Source, "/home/someone/Downloads/walls")),
            selected: 1,
            ..stocked()
        },
    },
    State {
        name: "undo-excluded",
        cfg: shelved,
        shelf: || Shelf {
            undo: Some((FolderKind::Excluded, "/home/someone/Pictures/old-walls")),
            draft: Some((FolderKind::Excluded, "/tmp/old")),
            selected: 2,
            ..stocked()
        },
    },
    State {
        name: "armed",
        cfg: shelved,
        shelf: || Shelf {
            selected: 3,
            armed: Some(leaked(ActionId::DeleteProfile(2))),
            ..stocked()
        },
    },
    State { name: "everything", cfg: shelved, shelf: stocked },
    State {
        name: "filtered",
        cfg: shelved,
        shelf: || Shelf { selected: 1, filter: "an", ..stocked() },
    },
    State {
        name: "no-match",
        cfg: shelved,
        shelf: || Shelf { selected: 1, filter: "zzz", ..stocked() },
    },
    State {
        name: "trash-on",
        cfg: || {
            shelved()
                .with_flag(keys::performance::AUTO_DELETE_IMAGE_TRASH, true)
                .with_flag(keys::effects::AUTO_RECOLOR, true)
                .with_flag(keys::performance::AUTO_OPTIMIZE_IMAGES, false)
                .with_flag(keys::library::POLLING_FALLBACK, false)
        },
        shelf: || Shelf { watch: Some(leaked(watching(LibraryWatchMode::NATIVE))), ..stocked() },
    },
    State {
        name: "watch-down",
        cfg: || shelved().with_flag(keys::library::POLLING_FALLBACK, false),
        shelf: || Shelf {
            watch: Some(leaked(watching(LibraryWatchMode::UNAVAILABLE))),
            ..stocked()
        },
    },
];

pub(super) fn states() -> Vec<FitState> {
    TABLE
        .iter()
        .map(|state| FitState {
            name: state.name.to_string(),
            cfg: (state.cfg)(),
            weather: None,
            statuses: Vec::new(),
            drawing: None,
            playback: None,
            connected: false,
            apps: None,
            backends: Vec::new(),
            sddm: Vec::new(),
        })
        .collect()
}

pub(super) fn shelf(name: &str) -> Shelf<'static> {
    TABLE
        .iter()
        .find(|state| state.name == name)
        .map_or_else(Shelf::default, |state| (state.shelf)())
}

#[test]
#[ignore = "manual offscreen render of the Library place"]
fn render_library() {
    use crate::frontend::settings::category::LIBRARY as CATEGORY;
    let out = localized();
    let mut heights = Vec::new();
    let section = |key| Some(SectionId::new(CATEGORY, key));
    let sentence = |path: &str| Some(token_key(path, 0, TokenSide::Sentence));
    let body = |path: &str| Some(token_key(path, 0, TokenSide::Body));
    let shots = [
        ("library", "stocked", None, None),
        ("library-first-run", "first-run", None, None),
        ("library-empty", "empty", None, None),
        ("library-empty-open", "empty", section(FOLDERS), None),
        ("library-folders", "stocked", section(FOLDERS), None),
        ("library-undo", "undo-draft", section(FOLDERS), None),
        ("library-native", "trash-on", section(FOLDERS), None),
        ("library-down", "watch-down", section(FOLDERS), None),
        ("library-excluded", "stocked", section(EXCLUDED), None),
        ("library-excluded-undo", "undo-excluded", section(EXCLUDED), None),
        ("library-excluded-none", "empty", section(EXCLUDED), None),
        ("library-profiles", "stocked", section(PROFILES), None),
        ("library-long-names", "long-names", section(PROFILES), None),
        ("library-everything", "everything", section(PROFILES), None),
        ("library-armed", "armed", section(PROFILES), None),
        ("library-filtered", "filtered", section(PROFILES), None),
        ("library-no-match", "no-match", section(PROFILES), None),
        ("library-storage", "stocked", section(STORAGE), None),
        ("library-storage-on", "trash-on", section(STORAGE), None),
        ("library-thumbnails", "stocked", section(THUMBS), None),
        ("library-active-menu", "stocked", None, sentence(keys::library::PROFILE)),
        (
            "library-quality-menu",
            "stocked",
            None,
            sentence(keys::performance::IMAGE_OPTIMIZE_PRESET),
        ),
        (
            "library-size-menu",
            "stocked",
            None,
            sentence(keys::performance::IMAGE_OPTIMIZE_RESOLUTION),
        ),
        ("library-theme-menu", "trash-on", section(STORAGE), body(keys::effects::AUTO_THEME)),
    ];
    let table = states();
    for (name, state, open, pop) in shots {
        let fit = table.iter().find(|fit| fit.name == state).expect("state");
        let shot = Shot { place: LIBRARY, cfg: &fit.cfg, open, pop, edit: None, weather: None };
        let (view, used) = capture_live(&shot, &fit.live(), Some(&format!("{out}/{name}.png")));
        heights.push(format!("{name}: used {used:.0} of {view:.0}"));
    }
    std::fs::write(format!("{out}/heights.txt"), heights.join("\n")).expect("heights");
}
