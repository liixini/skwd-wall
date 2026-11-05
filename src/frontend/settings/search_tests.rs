use super::*;
use crate::contracts::settings::SettingsSource;
use crate::frontend::settings::RuntimeStatus;
use crate::frontend::settings::place;
use crate::frontend::settings::test_source::FakeSettingsSource;

fn source<'a>(cfg: &'a dyn SettingsSource, runtime: RuntimeStatus<'a>) -> PlaceSource<'a> {
    PlaceSource { cfg, themes: &[], folders: &[], analysis: "", backends: &[], runtime }
}

const BROAD: &str = "a";
const ROTATE: &str = "rotate";
const ASLEEP: &str = "awww";
const HEX_QUERY: &str = "hex rows";
const HEX_MODE: &str = "hex";
const SLICES_MODE: &str = "slices";

fn find(query: &str, cfg: &dyn SettingsSource) -> Vec<SettingsSearchResult> {
    search_settings(query, source(cfg, RuntimeStatus::default()))
}

#[test]
fn search_aliases_across_tabs() {
    let cfg = FakeSettingsSource::default();
    let battery = find("laptop power", &cfg);
    assert!(!battery.is_empty());
    assert!(battery.iter().any(|result| result.tab == "system"));
    let frame_rate = find("frame rate", &cfg);
    assert!(frame_rate.iter().any(|result| result.title.to_lowercase().contains("fps")));
    let engine = find("we workshop", &cfg);
    assert!(engine.iter().any(|result| result.tab == "wallpapers"));
}

#[test]
fn page_sections_searchable() {
    let cfg = FakeSettingsSource::default();
    for (query, title) in [
        ("excluded", "Excluded folders"),
        ("folders", "Wallpaper folders"),
        ("profiles", "Profiles"),
    ] {
        let results = find(query, &cfg);
        assert!(
            results.iter().any(|result| result.tab == "library" && result.section_title == title),
            "{query}"
        );
    }
}

#[test]
fn blank_search_empty() {
    let cfg = FakeSettingsSource::default();
    assert!(find("   ", &cfg).is_empty());
}

#[test]
fn search_registered_pack() {
    use crate::contracts::settings::drawing::{PackState, Status};

    let cfg = FakeSettingsSource::default();
    let status = Status {
        pack: PackState::Available,
        pack_path: String::from("/models/manifest.json"),
        ..Status::default()
    };
    let runtime = RuntimeStatus { drawing: Some(&status), ..RuntimeStatus::default() };
    let results = search_settings("Choose processing files", source(&cfg, runtime));
    let result = results
        .iter()
        .find(|result| result.title == crate::i18n::tr("settings-drawing-pack-action-label"))
        .unwrap();
    let place = source(&cfg, runtime).place(&result.tab);
    assert!(matches!(
        place.sections[result.section].row(result.row).unwrap().control,
        Control::ActionBtn { id: super::super::ActionId::ChooseDrawingPack, .. }
    ));
}

#[test]
fn nested_source_controls_match() {
    let cfg = FakeSettingsSource::default();
    for query in ["show apply button", "SHOW APPLY BUTTON", "Show Apply button"] {
        let results = find(query, &cfg);
        assert_eq!(results.len(), 5, "{query}");
        assert!(results.iter().all(|result| result.tab == "sources"));
    }
}

#[test]
fn turkish_search_matches_dotted_and_dotless_i() {
    for (title, query) in [
        ("İndirmeler", "indirmeler"),
        ("Işık", "ışık"),
        ("IŞIK", "ışık"),
        ("İNDİRMELER", "indirmeler"),
    ] {
        assert_eq!(normalized(title), normalized(query), "{title}: {query}");
    }
}

fn hit(title: &str, score: u32, hidden: bool) -> SettingsSearchResult {
    SettingsSearchResult {
        tab: String::new(),
        tab_label: String::new(),
        section: 0,
        section_title: String::new(),
        row: 0,
        title: title.to_string(),
        desc: String::new(),
        value: String::new(),
        setting: None,
        home: None,
        wake: None,
        elsewhere: false,
        score,
        hidden,
    }
}

#[test]
fn hidden_ranks_after_visible() {
    let mut results = vec![hit("Aaa", 2, true), hit("Zzz", 2, false), hit("Mmm", 1, true)];
    rank(&mut results);
    let order: Vec<_> = results.iter().map(|result| result.title.as_str()).collect();
    assert_eq!(order, ["Mmm", "Zzz", "Aaa"]);
}

#[test]
fn hidden_labels_match() {
    use place::WALLPAPERS;

    let cfg = FakeSettingsSource::default();
    crate::i18n::with_language("en-US", || {
        let src = source(&cfg, RuntimeStatus::default());
        let places = src.places();
        let home = src.place(WALLPAPERS);
        let sleepers = src.asleep(WALLPAPERS, &home.sections);
        let wakes: Vec<_> = sleepers.iter().map(|sleeper| sleeper.wakes).collect();
        let lands = |wake: &str| {
            places
                .iter()
                .flat_map(|place| &place.sections)
                .flat_map(Section::rows)
                .any(|row| row.control.path() == Some(wake))
        };
        let landed: Vec<_> = sleepers.iter().filter(|sleeper| lands(sleeper.wakes)).collect();
        assert!(landed.len() > 1);
        for sleeper in landed {
            let hits = scan(&sleeper.row.title, &places, src);
            let own: Vec<_> = hits
                .iter()
                .filter(|hit| {
                    hit.wake.is_some() && hit.tab != place::LAYOUT && hit.title == sleeper.row.title
                })
                .collect();
            assert!(!own.is_empty(), "{}", sleeper.row.title);
            for hit in own {
                let place = src.place(&hit.tab);
                assert_eq!(hit.tab_label, place.title);
                assert_eq!(hit.section_title, place.sections[hit.section].title);
                let row =
                    place.sections[hit.section].row(hit.row).and_then(|row| row.control.path());
                assert!(row.is_some_and(|path| wakes.contains(&path)), "{}", hit.title);
            }
        }
    });
}

#[test]
fn results_capped_counted() {
    let cfg = FakeSettingsSource::default();
    let hits = search_hits(BROAD, source(&cfg, RuntimeStatus::default()));
    assert_eq!(hits.shown.len(), RESULT_LIMIT);
    assert!(hits.total > RESULT_LIMIT);
}

#[test]
fn parked_ranks_hidden() {
    use crate::contracts::settings::keys;

    let off = FakeSettingsSource::default().with_flag(keys::semantic::ENABLED, false);
    let found = crate::i18n::with_language("en-US", || find("Active model", &off));
    let model = found.iter().find(|hit| hit.title == "Active model").expect("model");
    assert!(model.hidden);
    assert!(model.live().is_some(), "parked stays editable");
    let on = FakeSettingsSource::default().with_flag(keys::semantic::ENABLED, true);
    let found = crate::i18n::with_language("en-US", || find("Active model", &on));
    assert!(!found.iter().find(|hit| hit.title == "Active model").expect("model").hidden);
}

#[test]
fn asleep_names_waker() {
    let cfg = FakeSettingsSource::default().with_niri(true);
    let found = crate::i18n::with_language("en-US", || find(ASLEEP, &cfg));
    let hit = found.iter().find(|hit| hit.wake.is_some()).expect("asleep");
    assert!(hit.live().is_none());
    assert!(hit.setting.is_some());
    assert!(hit.home.is_none());
}

#[test]
fn niri_only_on_niri() {
    use crate::contracts::settings::keys;

    let title = |cfg: &FakeSettingsSource| {
        let src = source(cfg, RuntimeStatus::default());
        let places = src.places();
        let found = scan("overview", &places, src);
        found.iter().any(|hit| {
            hit.setting.as_ref().and_then(|row| row.control.path())
                == Some(keys::niri::OVERVIEW_MODE)
        }) || found
            .iter()
            .any(|hit| hit.tab == place::WALLPAPERS && hit.section_title.contains("iri"))
    };
    crate::i18n::with_language("en-US", || {
        assert!(title(&FakeSettingsSource::default().with_niri(true)));
        assert!(!title(&FakeSettingsSource::default().with_niri(false)));
    });
}

#[test]
fn flags_elsewhere_matches() {
    let cfg = FakeSettingsSource::default();
    crate::i18n::with_language("en-US", || {
        let named = find("Max FPS", &cfg);
        assert!(!named[0].elsewhere);
        let alias = find("frame rate", &cfg);
        assert!(alias.iter().any(|hit| hit.title.contains("FPS") && hit.elsewhere));
    });
}

#[test]
fn reread_keeps_order() {
    use crate::contracts::settings::keys;

    let on = FakeSettingsSource::default().with_flag(keys::general::RANDOM_ROTATE, true);
    let off = FakeSettingsSource::default().with_flag(keys::general::RANDOM_ROTATE, false);
    let mut results = search_hits(ROTATE, source(&on, RuntimeStatus::default())).shown;
    let order: Vec<_> = results.iter().map(|hit| hit.title.clone()).collect();
    assert!(!order.is_empty());
    assert!(reread_settings(&mut results, source(&off, RuntimeStatus::default())));
    let after: Vec<_> = results.iter().map(|hit| hit.title.clone()).collect();
    assert_eq!(order, after);
    let flipped = results.iter().any(|hit| {
        matches!(
            hit.setting.as_ref().map(|row| &row.control),
            Some(Control::Toggle { path, value: false }) if path == keys::general::RANDOM_ROTATE
        )
    });
    assert!(flipped);
}

#[test]
fn places_feed_results() {
    let cfg = FakeSettingsSource::default().with_niri(true);
    crate::i18n::with_language("en-US", || {
        let src = source(&cfg, RuntimeStatus::default());
        for place in src.places() {
            for section in &place.sections {
                if section.rows().next().is_none() {
                    let hits = find(&section.title, &cfg);
                    assert!(hits.iter().any(|hit| hit.title == section.title), "{}", section.title);
                }
            }
        }
    });
}

fn mode_cfg(mode: &str) -> FakeSettingsSource {
    FakeSettingsSource::default()
        .with_text(crate::contracts::settings::keys::selector::DISPLAY_MODE, mode)
}

fn hex_rows(cfg: &FakeSettingsSource) -> Vec<SettingsSearchResult> {
    search_hits(HEX_QUERY, source(cfg, RuntimeStatus::default())).shown
}

#[test]
fn wake_flips_both_ways() {
    crate::i18n::with_language("en-US", || {
        let asleep = mode_cfg(SLICES_MODE);
        let live = mode_cfg(HEX_MODE);
        let mut results = hex_rows(&asleep);
        let slept = results.iter().filter(|hit| hit.wake.is_some()).count();
        assert!(slept > 0);
        assert!(reread_settings(&mut results, source(&live, RuntimeStatus::default())));
        assert!(results.iter().all(|hit| hit.wake.is_none()));
        assert!(results.iter().filter_map(SettingsSearchResult::live).count() >= slept);
        assert!(reread_settings(&mut results, source(&asleep, RuntimeStatus::default())));
        assert_eq!(results.iter().filter(|hit| hit.wake.is_some()).count(), slept);
    });
}

#[test]
fn wake_updates_crumb() {
    crate::i18n::with_language("en-US", || {
        let mut results = hex_rows(&mode_cfg(SLICES_MODE));
        let slept = results.iter().position(|hit| hit.wake.is_some()).expect("asleep");
        let before = results[slept].section_title.clone();
        reread_settings(&mut results, source(&mode_cfg(HEX_MODE), RuntimeStatus::default()));
        let home = source(&mode_cfg(HEX_MODE), RuntimeStatus::default()).place(&results[slept].tab);
        assert_eq!(results[slept].section_title, home.sections[results[slept].section].title);
        assert_ne!(results[slept].section_title, before);
    });
}

#[test]
fn asleep_value_refreshes() {
    use crate::contracts::settings::keys;

    crate::i18n::with_language("en-US", || {
        let first = mode_cfg(SLICES_MODE).with_text(keys::selector::HEX_ROWS, "3");
        let second = mode_cfg(SLICES_MODE).with_text(keys::selector::HEX_ROWS, "9");
        let mut results = hex_rows(&first);
        let slept = results.iter().position(|hit| hit.wake.is_some()).expect("asleep");
        let before = results[slept].value.clone();
        reread_settings(&mut results, source(&second, RuntimeStatus::default()));
        assert_ne!(results[slept].value, before);
    });
}
