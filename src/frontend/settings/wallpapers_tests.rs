use super::*;
use crate::frontend::settings::place;
use crate::frontend::settings::places::drawn;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source as source, plain};
use crate::frontend::settings::{Parked, search_settings};

fn sections(cfg: &dyn SettingsSource) -> Vec<Section> {
    crate::i18n::with_language("en-US", || source(cfg).place(place::WALLPAPERS).sections)
}

fn section(cfg: &dyn SettingsSource, key: &str) -> Section {
    sections(cfg).into_iter().find(|section| section.id.key == key).expect("section")
}

fn said(cfg: &dyn SettingsSource, key: &str) -> String {
    plain(&section(cfg, key).say.text)
}

fn found(
    cfg: &dyn SettingsSource,
    query: &str,
) -> Vec<crate::frontend::settings::SettingsSearchResult> {
    crate::i18n::with_language("en-US", || search_settings(query, source(cfg)))
}

fn keys_of(cfg: &dyn SettingsSource) -> Vec<&'static str> {
    sections(cfg).iter().map(|section| section.id.key).collect()
}

fn sleepers(cfg: &dyn SettingsSource) -> Vec<Asleep> {
    let source = source(cfg);
    asleep(&source.place(place::WALLPAPERS).sections, &source)
}

fn default() -> FakeSettingsSource {
    FakeSettingsSource::default()
}

#[test]
fn board_sections_only() {
    assert_eq!(keys_of(&default()), [DISPLAY_LIST, PLAY, MOTION, DRAWING, SOUND, PAUSE, PLASMA]);
    let both = default().with_text(paper::ENGINE, "awww").with_niri(true);
    assert_eq!(
        keys_of(&both),
        [DISPLAY_LIST, PLAY, MOTION, AWWW, DRAWING, SOUND, PAUSE, PLASMA, NIRI]
    );
}

#[test]
fn head_rows_drawn() {
    for cfg in [
        default(),
        default().with_text(paper::ENGINE, "awww").with_text(paper::AWWW_TRANSITION_TYPE, "none"),
        default().with_flag(transition::ENABLED, false),
        default().with_flag(keys::wallpaper::MUTE, true),
        default().with_flag(playback::FULLSCREEN, false),
        default().with_flag(draw::ENABLED, true),
        default().with_text(keys::plasma::LOCK_SCREEN_MODE, "static"),
        default().with_niri(true).with_flag(keys::niri::OVERVIEW_BACKDROP, false),
    ] {
        let composed = crate::i18n::with_language("en-US", || compose(&source(&cfg)));
        for section in &composed {
            assert!(drawn(section), "{}", section.title);
        }
    }
}

#[test]
fn system_owns_power() {
    let rows = sections(&default());
    let paths: Vec<_> =
        rows.iter().flat_map(Section::rows).filter_map(|row| row.control.path()).collect();
    assert!(!paths.contains(&paper::IDLE_PAUSE_SECONDS));
    assert!(!paths.contains(&paper::PERFORMANCE_MODE));
    assert!(paths.contains(&keys::performance::GPU_DEVICE));
    assert!(paths.contains(&paper::VIDEO_MULTI_PROCESS));
}

#[test]
fn transition_sentence_states() {
    let shader = |name| default().with_text(transition::SHADER, name);
    assert_eq!(
        said(&shader("crossfade"), MOTION),
        "[0] family, using [1], over [2]. Frame rate [3]."
    );
    assert_eq!(
        said(&shader("random"), MOTION),
        "[0] family, a different one every time, over [1]. Frame rate [2]."
    );
    let studio = said(&shader("photo-cutout"), MOTION);
    assert!(studio.starts_with("[0] family, using [1], in its "), "{studio}");
    let off = default().with_flag(transition::ENABLED, false);
    assert_eq!(said(&off, MOTION), "Off. Each wallpaper cuts straight to the next.");
}

#[test]
fn off_parks_transitions() {
    let off = default().with_flag(transition::ENABLED, false);
    let motion = section(&off, MOTION);
    assert!(motion.groups.is_empty());
    assert_eq!(motion.foot.as_deref(), Some(tr(MOTION_OFF_FOOT)));
    assert!(motion.has_body());
    assert_eq!(motion.say.parked.len(), motion.head.len());
    let hits = found(&off, "Transition FPS");
    assert!(hits.iter().any(|hit| hit.title == "Transition FPS"), "{hits:?}");
}

#[test]
fn parked_found_as_self() {
    let muted = default().with_flag(keys::wallpaper::MUTE, true);
    let at = keys_of(&muted).iter().position(|key| *key == SOUND).expect("sound");
    let audio = section(&muted, SOUND);
    let list: Vec<Row> = audio.rows().cloned().collect();
    let row = list.iter().position(|row| row.control.path() == Some(keys::wallpaper::VOLUME));
    let cause = list.iter().position(|row| row.control.path() == Some(keys::wallpaper::MUTE));
    assert_eq!(audio.say.parked, [Parked { row: row.unwrap(), cause: cause.unwrap() }]);
    let hits = found(&muted, "Wallpaper volume");
    assert!(
        hits.iter().any(|hit| hit.tab == place::WALLPAPERS
            && hit.section == at
            && Some(hit.row) == row
            && hit.title == "Wallpaper volume"),
        "{hits:?}"
    );
}

#[test]
fn hidden_found_as_self() {
    let cfg =
        default().with_flag(transition::PREVIEW, false).with_flag(keys::features::STEAM, false);
    for (query, wakes) in [
        ("Wave width", paper::ENGINE),
        ("Sharpen reduced sand", SHADER_FAMILY_KEY),
        ("Preview frame rate", transition::PREVIEW),
    ] {
        let hits = found(&cfg, query);
        let hit = hits.iter().find(|hit| hit.title == query).expect(query);
        let landed = crate::i18n::with_language("en-US", || source(&cfg).place(&hit.tab));
        assert_eq!(hit.tab_label, landed.title, "{query}");
        assert_eq!(hit.section_title, landed.sections[hit.section].title, "{query}");
        let at =
            landed.sections[hit.section].rows().position(|row| row.control.path() == Some(wakes));
        assert_eq!(Some(hit.row), at, "{query}");
    }
}

#[test]
fn hidden_lands_on_cause() {
    let hits = found(&default(), "Wave width");
    let hit = hits.iter().find(|hit| hit.title == "Wave width").expect("hit");
    let play = sections(&default());
    assert_eq!(play[hit.section].id.key, PLAY);
    assert_eq!(
        play[hit.section].row(hit.row).and_then(|row| row.control.path()),
        Some(paper::ENGINE)
    );
}

#[test]
fn sleepers_stay_out() {
    let asleep = crate::i18n::with_language("en-US", || sleepers(&default()));
    let paths: Vec<_> = asleep.iter().filter_map(|sleeper| sleeper.row.control.path()).collect();
    assert!(paths.contains(&paper::AWWW_TRANSITION_ANGLE));
    assert!(!paths.contains(&transition::DURATION_MS));
    let mut unique = paths.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), paths.len());
}

#[test]
fn awww_only_when_chosen() {
    assert!(!keys_of(&default()).contains(&AWWW));
    let awww = default().with_text(paper::ENGINE, "awww");
    assert!(keys_of(&awww).contains(&AWWW));
    assert_eq!(said(&awww, AWWW), "Images play [0] with the [1] filter, over [2].");
    let none = awww.with_text(paper::AWWW_TRANSITION_TYPE, "none");
    assert_eq!(said(&none, AWWW), "Images change with [0].");
}

#[test]
fn drawing_sentences() {
    assert_eq!(said(&default(), DRAWING), "Off. Cards set to Draw still draw over [0].");
    let on = default().with_flag(draw::ENABLED, true);
    assert_eq!(said(&on, DRAWING), "Drawing is unavailable. Choose processing files first.");
    let group = section(&on, DRAWING);
    assert!(group.say.parks(1).is_some());
}

#[test]
fn drawing_keeps_colours() {
    let group = section(&default(), DRAWING);
    let left: Vec<_> = group.groups[0].rows.iter().filter_map(|row| row.control.path()).collect();
    assert_eq!(left, [draw::PAPER_COLOR, draw::INK_COLOR, draw::INK]);
    assert!(group.rows().any(|row| {
        matches!(row.control, Control::ActionBtn { id: ActionId::ChooseDrawingPack, .. })
    }));
}

fn preparing_menu(cfg: &dyn SettingsSource, wanted: &str) -> (Vec<(String, String)>, String) {
    let group = section(cfg, DRAWING);
    group.groups[1]
        .rows
        .iter()
        .find_map(|row| match &row.control {
            Control::Dropdown { path, options, current, .. } if path == wanted => {
                Some((options.clone(), current.clone()))
            }
            _ => None,
        })
        .expect("preparing menu")
}

#[test]
fn device_menu_falls_back() {
    let (options, current) = preparing_menu(&default(), draw::DEVICE);
    let values: Vec<_> = options.iter().map(|(value, _)| value.as_str()).collect();
    assert_eq!(values, ["auto", "cuda", "webgpu", "cpu"]);
    assert_eq!(current, "auto");
    let unknown = default().with_text(draw::DEVICE, "rocm");
    assert_eq!(preparing_menu(&unknown, draw::DEVICE).1, "auto");
    let cuda = default().with_text(draw::DEVICE, "cuda");
    assert_eq!(preparing_menu(&cuda, draw::DEVICE).1, "cuda");
}

#[test]
fn memory_menu_keeps_saved() {
    let (options, current) = preparing_menu(&default(), draw::MEMORY_LIMIT_MIB);
    assert_eq!(current, draw::MEMORY_LIMIT_AUTO);
    assert_eq!(options[0].1, "Automatic (6 GiB)");
    assert!(options.contains(&(String::from("8192"), String::from("8 GiB"))));
    let preset = default().with_text(draw::MEMORY_LIMIT_MIB, "8192");
    let (kept, current) = preparing_menu(&preset, draw::MEMORY_LIMIT_MIB);
    assert_eq!((kept.len(), current.as_str()), (options.len(), "8192"));
    let odd = default().with_text(draw::MEMORY_LIMIT_MIB, "3000");
    let (saved, current) = preparing_menu(&odd, draw::MEMORY_LIMIT_MIB);
    assert_eq!(current, "3000");
    assert_eq!(saved.last(), Some(&(String::from("3000"), String::from("3000 MiB"))));
    let junk = default().with_text(draw::MEMORY_LIMIT_MIB, "lots");
    assert_eq!(preparing_menu(&junk, draw::MEMORY_LIMIT_MIB).1, draw::MEMORY_LIMIT_AUTO);
}

#[test]
fn sound_sentences() {
    let audible = || default().with_flag(keys::wallpaper::MUTE, false);
    let quiet = audible().with_flag(playback::MUTE_ON_OTHER_AUDIO, false);
    assert_eq!(said(&quiet, SOUND), "Wallpaper audio plays at [0].");
    let duck = audible().with_flag(playback::MUTE_ON_OTHER_AUDIO, true);
    assert_eq!(
        said(&duck, SOUND),
        "Wallpaper audio plays at [0] and pauses while other apps play sound."
    );
    let muted = default().with_flag(keys::wallpaper::MUTE, true);
    assert_eq!(said(&muted, SOUND), "Wallpaper audio is muted.");
}

#[test]
fn volume_echoed_once() {
    let audio = section(&default(), SOUND);
    let heads = audio.rows().filter(|row| row.control.path() == Some(keys::wallpaper::VOLUME));
    assert_eq!(heads.count(), 1);
    let echoes: Vec<_> = audio.groups.iter().flat_map(|group| &group.echoes).collect();
    assert_eq!(echoes.len(), 1);
    assert_eq!(echoes[0].1.control.path(), Some(keys::wallpaper::VOLUME));
}

#[test]
fn pause_sentences() {
    let none = default().with_flag(playback::FULLSCREEN, false);
    assert_eq!(said(&none, PAUSE), "No pause rule is on.");
    let apps = none.with_flag(playback::PROCESS_ENABLED, true);
    assert_eq!(said(&apps, PAUSE), "Pauses while listed applications run.");
    let full = default().with_flag(playback::FULLSCREEN, true);
    assert_eq!(said(&full, PAUSE), "Pauses on fullscreen windows on [0].");
    let two = full.with_flag(playback::MAXIMIZED, true).with_flag(playback::PROCESS_ENABLED, true);
    assert_eq!(said(&two, PAUSE), "Pauses on 2 window rules on [0], and for listed applications.");
}

#[test]
fn plasma_moved_here() {
    let colours = crate::i18n::with_language("en-US", || source(&default()).place(place::COLOURS));
    let paths: Vec<_> = colours
        .sections
        .iter()
        .flat_map(Section::rows)
        .filter_map(|row| row.control.path())
        .collect();
    assert!(!paths.contains(&keys::plasma::LOCK_SCREEN_MODE));
    let plasma = section(&default(), PLASMA);
    assert!(plasma.rows().any(|row| {
        matches!(row.control, Control::ActionBtn { id: ActionId::SetLockScreen(..), .. })
    }));
    let follow = default().with_text(keys::plasma::LOCK_SCREEN_MODE, "follow");
    assert_eq!(said(&follow, PLASMA), "The lock screen [0], scenes as a poster frame.");
    let still = default().with_text(keys::plasma::LOCK_SCREEN_MODE, "static");
    assert_eq!(said(&still, PLASMA), "The lock screen [0]: [1].");
}

#[test]
fn niri_sentences() {
    let niri = default()
        .with_niri(true)
        .with_flag(keys::niri::OVERVIEW_BACKDROP, true)
        .with_flag(keys::niri::BACKDROP_BLUR_STATIC, false);
    assert_eq!(said(&niri, NIRI), "Shows the current wallpaper, dimmed [0].");
    let blur = niri.with_flag(keys::niri::BACKDROP_BLUR_STATIC, true);
    assert_eq!(said(&blur, NIRI), "Shows the current wallpaper, blurred, dimmed [0].");
    let off = blur.with_flag(keys::niri::OVERVIEW_BACKDROP, false);
    assert_eq!(said(&off, NIRI), "Off. The overview keeps niri's own backdrop.");
    let group = section(&off, NIRI);
    assert!(group.head.iter().any(|row| matches!(row.control, Control::Code { .. })));
}

#[test]
fn displays_list_locks() {
    use crate::contracts::daemon::OutputStatus;
    let statuses = [
        OutputStatus { name: "DP-1".into(), ..Default::default() },
        OutputStatus { name: "DP-3".into(), ..Default::default() },
    ];
    let locked = format!("{}.DP-1", keys::display::OUTPUT_LOCKS);
    let cfg = default().with_flag(&locked, true);
    let runtime =
        crate::frontend::settings::RuntimeStatus { statuses: &statuses, ..Default::default() };
    let composed =
        crate::i18n::with_language("en-US", || displays(&PlaceSource { runtime, ..source(&cfg) }));
    assert_eq!(
        plain(&composed.say.text),
        "DP-1 is locked. Displays without their own placement use [0]."
    );
    let [places, locks] = &composed.groups[..] else { panic!("two columns") };
    assert_eq!(places.rows[0].title, "DP-1");
    assert_eq!(locks.rows[0].title, "Lock DP-1");
    let empty = said(&default(), DISPLAY_LIST);
    assert_eq!(empty, "Detecting displays. Displays without their own placement use [0].");
}

#[test]
fn random_parks_scopes() {
    let motion = section(&default().with_text(transition::SHADER, "random"), MOTION);
    let lead = motion.groups[1].rows.iter().filter(|row| row.look.cell == Cell::Checks).count();
    assert_eq!(lead, 0);
    let scope = motion.head.iter().position(|row| is_scope(&row.control)).expect("scope");
    assert!(motion.say.parks(1 + scope).is_some());
}

#[test]
fn primary_only_counts() {
    let scope = |name: &str| format!("{}.{name}", transition::SHADER_SCOPES);
    let caption = |cfg: &FakeSettingsSource| {
        let motion = section(cfg, MOTION);
        let lead = motion.rows().find(|row| row.look.cell == Cell::Checks).expect("lead").clone();
        lead.desc
    };
    let fade = || default().with_text(transition::SHADER, "crossfade");
    let total = crate::frontend::settings::tables::shaders_in_family("fade").len();
    assert_eq!(caption(&fade()), format!("none of the {total}"));
    let one = fade().with_text(&scope("crossfade"), "primary");
    assert_eq!(caption(&one), "Crossfade");
    let two = one.with_text(&scope("perlin"), "primary");
    assert_eq!(caption(&two), format!("2 of {total}"));
}

#[test]
fn stage_stays_last() {
    let motion = section(&default().with_text(transition::SHADER, "crossfade"), MOTION);
    let last = motion.rows().last().expect("row");
    assert!(matches!(last.control, Control::Preview));
}

#[test]
fn locale_keys_complete() {
    let on = default()
        .with_text(paper::ENGINE, "awww")
        .with_text(keys::plasma::LOCK_SCREEN_MODE, "follow")
        .with_niri(true)
        .with_flag(keys::niri::OVERVIEW_BACKDROP, true);
    for language in crate::i18n::LANGUAGES {
        crate::i18n::with_language(language.tag, || {
            for section in source(&on).place(place::WALLPAPERS).sections {
                assert!(!section.title.is_empty(), "{}", language.tag);
                assert!(drawn(&section), "{} {}", language.tag, section.id.key);
                assert!(!section.say.text.is_empty(), "{} {}", language.tag, section.id.key);
            }
        });
    }
}

#[test]
fn image_echo_opens() {
    let still = section(&default().with_text(keys::plasma::LOCK_SCREEN_MODE, "static"), PLASMA);
    assert!(still.body().next().is_none());
    assert!(still.has_body());
}

#[test]
fn every_sleeper_lands() {
    for cfg in [
        default(),
        default().with_niri(true),
        default().with_text(paper::ENGINE, "awww"),
        default().with_flag(keys::features::STEAM, false),
        default().with_text(keys::plasma::LOCK_SCREEN_MODE, "static"),
        default().with_text(keys::plasma::LOCK_SCREEN_MODE, "off"),
    ] {
        let src = source(&cfg);
        let places = crate::i18n::with_language("en-US", || src.places());
        let shown: HashSet<_> = places
            .iter()
            .flat_map(|place| &place.sections)
            .flat_map(Section::rows)
            .filter_map(|row| row.control.path())
            .collect();
        let mut owed: Vec<_> = crate::i18n::with_language("en-US", || asleep_rows(&cfg, &[]))
            .into_iter()
            .filter_map(|row| row.control.path().map(str::to_string))
            .filter(|path| !shown.contains(path.as_str()))
            .collect();
        owed.sort_unstable();
        owed.dedup();
        let mut reached: Vec<_> = sleepers(&cfg)
            .into_iter()
            .filter_map(|sleeper| sleeper.row.control.path().map(str::to_string))
            .collect();
        reached.sort_unstable();
        assert_eq!(owed, reached);
        let lost: Vec<_> = sleepers(&cfg)
            .into_iter()
            .filter(|sleeper| !shown.contains(sleeper.wakes))
            .filter_map(|sleeper| sleeper.row.control.path().map(str::to_string))
            .collect();
        assert!(
            lost.iter().all(|path| path.starts_with(NIRI_PREFIX) && !cfg.is_niri()),
            "{lost:?}"
        );
    }
}

#[test]
fn scene_rows_found_asleep() {
    let cfg = default().with_flag(keys::features::STEAM, false);
    let hits = found(&cfg, "Scenes");
    assert!(!hits.is_empty());
    for hit in hits {
        let landed = crate::i18n::with_language("en-US", || source(&cfg).place(&hit.tab));
        assert_eq!(hit.section_title, landed.sections[hit.section].title);
        assert!(landed.sections[hit.section].row(hit.row).is_some());
    }
}

#[test]
fn no_sleeper_dropped() {
    for cfg in [default(), default().with_niri(true), default().with_text(paper::ENGINE, "awww")] {
        let sections = sections(&cfg);
        let shown: HashSet<_> =
            sections.iter().flat_map(Section::rows).filter_map(|row| row.control.path()).collect();
        let mut asleep: Vec<_> = crate::i18n::with_language("en-US", || asleep_rows(&cfg, &[]))
            .into_iter()
            .filter_map(|row| row.control.path().map(str::to_string))
            .filter(|path| !shown.contains(path.as_str()))
            .collect();
        asleep.sort_unstable();
        asleep.dedup();
        assert_eq!(asleep.len(), sleepers(&cfg).len());
    }
}

#[test]
fn displays_without_outputs() {
    use crate::frontend::settings::RuntimeStatus;
    let cfg = default();
    let said = |connected| {
        let runtime = RuntimeStatus { connected, ..RuntimeStatus::default() };
        let section = crate::i18n::with_language("en-US", || {
            displays(&PlaceSource { runtime, ..source(&cfg) })
        });
        plain(&section.say.text)
    };
    assert_eq!(said(false), "Detecting displays. Displays without their own placement use [0].");
    assert_eq!(said(true), "No display is locked. Displays without their own placement use [0].");
}

fn prepared_status() -> drawing::Status {
    drawing::Status {
        pack: drawing::PackState::Available,
        pack_path: String::from("/models/drawing/manifest.json"),
        pack_bytes: 1_400_000_000,
        key: String::from("still:one"),
        ..Default::default()
    }
}

fn drawn_with(library: bool, running: bool) -> Section {
    let status = prepared_status();
    let runtime = crate::frontend::settings::RuntimeStatus {
        drawing: Some(&status),
        drawing_selection: Some("still:one"),
        drawing_view_count: 7,
        drawing_job: crate::frontend::settings::DrawingJob { library, running },
        ..Default::default()
    };
    let cfg = default();
    let place = crate::i18n::with_language("en-US", || {
        PlaceSource { runtime, ..source(&cfg) }.place(place::WALLPAPERS)
    });
    place.sections.into_iter().find(|section| section.id.key == DRAWING).expect("drawing")
}

fn live_button(section: &Section) -> Option<ActionId> {
    let hidden = section.say.parked.iter().map(|parked| parked.row).collect::<Vec<_>>();
    let lead = usize::from(section.master.is_some());
    section.groups.iter().flat_map(|group| &group.rows).find_map(|row| match &row.control {
        Control::ActionBtn {
            id: id @ (ActionId::PrepareDrawing | ActionId::PrepareAllDrawings),
            ..
        } if !hidden.contains(&lead) => Some(id.clone()),
        _ => None,
    })
}

#[test]
fn prepare_menu_scopes() {
    let section = drawn_with(false, false);
    let menu = section.rows().find_map(|row| match &row.control {
        Control::Dropdown { path, options, current, .. } if path == DRAWING_SCOPE_KEY => {
            Some((options.clone(), current.clone()))
        }
        _ => None,
    });
    let (options, current) = menu.expect("scope menu");
    let labels: Vec<_> = options.iter().map(|(_, label)| label.as_str()).collect();
    assert_eq!(labels, ["Selected wallpaper", "Current view · 7", "All wallpapers"]);
    assert_eq!(current, draw::SCOPE_SELECTED);
    assert_eq!(live_button(&section), Some(ActionId::PrepareDrawing));
}

#[test]
fn all_scope_runs_library() {
    let section = drawn_with(true, false);
    let current = section.rows().find_map(|row| match &row.control {
        Control::Dropdown { path, current, .. } if path == DRAWING_SCOPE_KEY => {
            Some(current.clone())
        }
        _ => None,
    });
    assert_eq!(current.as_deref(), Some(DRAWING_LIBRARY_SCOPE));
    assert_eq!(live_button(&section), Some(ActionId::PrepareAllDrawings));
    let busy = drawn_with(false, true);
    assert_eq!(live_button(&busy), Some(ActionId::PrepareAllDrawings));
}

#[test]
fn prepare_all_stays_findable() {
    let status = prepared_status();
    let runtime = crate::frontend::settings::RuntimeStatus {
        drawing: Some(&status),
        drawing_selection: Some("still:one"),
        ..Default::default()
    };
    let cfg = default();
    let hits = crate::i18n::with_language("en-US", || {
        search_settings("Prepare all wallpapers", PlaceSource { runtime, ..source(&cfg) })
    });
    assert!(hits.iter().any(|hit| hit.title == "Prepare all wallpapers"), "{hits:?}");
}

#[test]
fn drawing_notes_wrap() {
    let section = drawn_with(false, false);
    let hint = section
        .groups
        .iter()
        .flat_map(|group| &group.rows)
        .find_map(|row| match &row.look.note {
            Note::Own(text) if text.contains("manifest.json") => Some(text.clone()),
            _ => None,
        })
        .expect("pack hint");
    assert!(hint.ends_with("Model files: 1335 MiB"), "{hint}");
    let note = section.say.notes.join(" ");
    assert!(note.contains("still:one"), "{note}");
    assert!(note.contains("not prepared"), "{note}");
}

#[test]
fn scope_key_not_config() {
    let cfg = default().with_text(draw::SCOPE, draw::SCOPE_VIEW);
    assert_eq!(cfg.text(DRAWING_SCOPE_KEY), "");
    let status = prepared_status();
    let runtime = crate::frontend::settings::RuntimeStatus {
        drawing: Some(&status),
        drawing_selection: Some("still:one"),
        drawing_view_count: 7,
        ..Default::default()
    };
    let hits = crate::i18n::with_language("en-US", || {
        search_settings("Prepare", PlaceSource { runtime, ..source(&cfg) })
    });
    let hit = hits.iter().find(|hit| hit.title == "Prepare").expect("prepare row");
    assert_eq!(hit.value, "Current view · 7");
}

#[test]
fn missing_pack_install_hint() {
    let cfg = default();
    for (phase, hint) in [
        (Phase::Idle, crate::frontend::settings::DRAWING_PACK_INSTALL),
        (Phase::Checking, PACK_BLANK),
    ] {
        let status = drawing::Status { phase, ..Default::default() };
        let runtime = crate::frontend::settings::RuntimeStatus {
            drawing: Some(&status),
            ..Default::default()
        };
        let note = crate::i18n::with_language("en-US", || {
            let place = PlaceSource { runtime, ..source(&cfg) }.place(place::WALLPAPERS);
            let section = place.sections.into_iter().find(|section| section.id.key == DRAWING);
            section.expect("drawing").rows().find_map(|row| match row.control {
                Control::ActionBtn { id: ActionId::ChooseDrawingPack, .. } => {
                    Some(row.look.note.clone())
                }
                _ => None,
            })
        });
        let expected = crate::i18n::with_language("en-US", || tr(hint).to_string());
        assert_eq!(note, Some(Note::Own(expected)), "{phase:?}");
    }
}
