use super::*;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source, plain};

fn changed_cfg() -> FakeSettingsSource {
    FakeSettingsSource::default()
        .with_text(keys::wallpaper::VOLUME, "40")
        .with_flag(keys::general::RANDOM_ROTATE, true)
}

#[test]
fn defaults_change_nothing() {
    let cfg = FakeSettingsSource::default();
    let found = overview(&place_source(&cfg));
    let paths: Vec<_> = found.changes.iter().map(|change| change.path.as_str()).collect();
    assert!(paths.is_empty(), "{paths:?}");
}

#[test]
fn changes_name_owner() {
    crate::i18n::with_language("en-US", || {
        let cfg = changed_cfg();
        let found = overview(&place_source(&cfg));
        let volume = found
            .changes
            .iter()
            .find(|change| change.path == keys::wallpaper::VOLUME)
            .expect("volume");
        assert_eq!(volume.place, place::WALLPAPERS);
        assert_eq!(volume.spot, "Wallpapers · Sound");
        assert_eq!((volume.was.as_str(), volume.now.as_str()), ("100 %", "40 %"));
        assert_eq!(found.count(place::AUTOMATION), 1);
        assert_eq!(badge(place::HOME, &found).as_deref(), Some("2 changed"));
        assert_eq!(badge(place::WALLPAPERS, &found).as_deref(), Some("1"));
        assert_eq!(badge(place::LIBRARY, &found), None);
    });
}

#[test]
fn undo_matches_control() {
    let cfg = changed_cfg();
    let found = overview(&place_source(&cfg));
    let undo = |path: &str| {
        found.changes.iter().find(|change| change.path == path).map(|change| change.undo.clone())
    };
    assert_eq!(undo(keys::general::RANDOM_ROTATE), Some(Undo::Toggle(false)));
    assert_eq!(
        undo(keys::wallpaper::VOLUME),
        Some(Undo::Input { key: keys::wallpaper::VOLUME.to_string(), raw: "100".to_string() })
    );
}

fn paths_of(section: &Section) -> Vec<&str> {
    section.rows().filter_map(|row| row.control.path()).collect()
}

fn owner_paths(source: &PlaceSource<'_>, owner: &str, key: &str) -> Vec<String> {
    let place = source.place(owner);
    let section = place.sections.iter().find(|section| section.id.key == key).expect("owner");
    paths_of(section).into_iter().map(String::from).collect()
}

#[test]
fn home_mirrors_owners() {
    let cfg = FakeSettingsSource::default().with_flag(keys::general::RANDOM_ROTATE, true);
    let source = place_source(&cfg);
    let home = compose(&source);
    let sound = paths_of(&home[1]);
    assert_eq!(
        sound,
        [keys::wallpaper::VOLUME, keys::wallpaper::MUTE, keys::playback::MUTE_ON_OTHER_AUDIO]
    );
    assert_eq!(paths_of(&home[2]), [keys::theme::POLICY]);
    let rotation = paths_of(&home[3]);
    assert_eq!(rotation, owner_paths(&source, place::AUTOMATION, automation::ROTATION));
    assert!(rotation.contains(&keys::general::RANDOM_ROTATE));
    assert!(rotation.contains(&keys::general::RANDOM_INTERVAL));
    let mut motion = paths_of(&home[4]);
    let mut owner = owner_paths(&source, place::WALLPAPERS, wallpapers::MOTION);
    motion.sort_unstable();
    owner.sort_unstable();
    assert_eq!(motion, owner);
    assert!(motion.contains(&keys::transition::ENABLED));
    assert!(motion.contains(&keys::transition::DURATION_MS));
}

#[test]
fn places_skip_home() {
    let cfg = FakeSettingsSource::default();
    let places = place_source(&cfg).places();
    assert!(places.iter().all(|place| place.key != place::HOME));
}

#[test]
fn sections_stay_drawn() {
    for cfg in [
        FakeSettingsSource::default(),
        FakeSettingsSource::default().with_flag(keys::wallpaper::MUTE, true),
        FakeSettingsSource::default().with_flag(keys::transition::ENABLED, false),
        FakeSettingsSource::default().with_flag(keys::general::RANDOM_ROTATE, true),
    ] {
        for section in compose(&place_source(&cfg)) {
            assert!(super::super::places::drawn(&section), "{}", section.title);
        }
    }
}

#[test]
fn links_bound_inline() {
    let cfg = FakeSettingsSource::default()
        .with_flag(keys::wallpaper::MUTE, false)
        .with_flag(keys::general::RANDOM_ROTATE, true);
    let home = compose(&place_source(&cfg));
    assert_eq!(home[0].say.tokens.len(), 2);
    assert_eq!(home[5].say.tokens.len(), 0);
    for section in &home[1..5] {
        assert!(!section.say.tokens.is_empty(), "{}", section.title);
    }
    assert!(home[2].say.binds(0) && home[2].say.binds(1));
}

#[test]
fn sentences_follow_state() {
    crate::i18n::with_language("en-US", || {
        let cfg = FakeSettingsSource::default()
            .with_flag(keys::wallpaper::MUTE, true)
            .with_flag(keys::general::RANDOM_ROTATE, true)
            .with_text(keys::general::RANDOM_INTERVAL, "300")
            .with_flag(keys::transition::ENABLED, false);
        let says: Vec<_> =
            compose(&place_source(&cfg)).iter().map(|section| plain(&section.say.text)).collect();
        assert_eq!(says[1], "Wallpaper audio is muted.");
        assert_eq!(says[2], "Palette behaviour is [0]. More in [1].");
        assert_eq!(says[3], "A random wallpaper every [0].");
        assert_eq!(says[4], "Transitions are off. Wallpapers cut straight to the next one.");
    });
}

#[test]
fn no_blank_values() {
    let row = |control| Row::of(String::from("Title"), String::new(), control);
    let field = row(Control::TextField {
        key: String::from("x"),
        path: String::from("x"),
        placeholder: "",
        secret: false,
    });
    crate::i18n::with_language("en-US", || {
        assert_eq!(blank_word(&field), tr(NO_VALUE));
        assert_eq!(text_word(&field, ""), tr(NO_VALUE));
    });
}

#[test]
fn every_change_has_words() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::general::RANDOM_INTERVAL, "90")
        .with_flag(keys::wallpaper::MUTE, true);
    for change in &overview(&place_source(&cfg)).changes {
        assert!(!change.title.is_empty(), "{}", change.path);
        assert!(!change.was.is_empty(), "{}", change.path);
        assert!(!change.now.is_empty(), "{}", change.path);
    }
}

#[test]
fn model_status_words() {
    crate::i18n::with_language("en-US", || {
        let off = FakeSettingsSource::default().with_flag(keys::semantic::ENABLED, false);
        assert_eq!(model(&off), "off");
        let on = FakeSettingsSource::default().with_flag(keys::semantic::ENABLED, true);
        assert_eq!(model(&on), "ready");
        let gone = on.with_flag("test.defaultModelAvailable", false);
        assert_eq!(model(&gone), "no model");
    });
}

#[test]
fn analysis_overrides_model() {
    crate::i18n::with_language("en-US", || {
        let cfg = FakeSettingsSource::default();
        let source = PlaceSource { analysis: "importing 3 of 9", ..place_source(&cfg) };
        let say = plain(&compose(&source)[0].say.text);
        assert!(say.contains("Model status: importing 3 of 9."), "{say}");
        let quiet = plain(&compose(&place_source(&cfg))[0].say.text);
        assert!(quiet.contains("Model status: ready."), "{quiet}");
    });
}

#[test]
fn engine_reads_owner() {
    crate::i18n::with_language("en-US", || {
        let cfg = FakeSettingsSource::default().with_text(keys::theme::ENGINE, "wallust");
        let source = place_source(&cfg);
        let owner = source.place(place::COLOURS);
        let label = leaf_at(&owner, keys::theme::ENGINE).map(|row| shown_value(row, &cfg));
        let say = plain(&compose(&source)[0].say.text);
        assert!(say.ends_with(&format!("come from {}.", label.expect("engine row"))), "{say}");
    });
}

#[test]
fn colours_summary_real() {
    crate::i18n::with_language("en-US", || {
        let cfg = FakeSettingsSource::default().with_text(keys::theme::ENGINE, "wallust");
        let source = place_source(&cfg);
        let owner = source.place(place::COLOURS);
        let row = leaf_at(&owner, keys::theme::ENGINE).expect("engine row");
        let label = shown_value(row, &cfg);
        assert!(!label.is_empty());
        assert_ne!(label, tr(NO_VALUE));
        let sections = compose(&source);
        let say = plain(&sections[0].say.text);
        assert!(say.ends_with(&format!("come from {label}.")), "{say}");
        assert!(sections.iter().any(|section| section.id.key == COLOURS.title));
    });
}

#[test]
fn runtime_reads_fresh() {
    crate::i18n::with_language("en-US", || {
        let cfg = FakeSettingsSource::default();
        let quiet = plain(&compose(&place_source(&cfg))[0].say.text);
        assert!(quiet.contains("[0] are showing"), "{quiet}");
        let outputs = [crate::contracts::daemon::OutputStatus {
            name: String::from("DP-1"),
            current: String::from("a.png"),
            ..Default::default()
        }];
        let runtime = RuntimeStatus { statuses: &outputs, ..RuntimeStatus::default() };
        let shown = plain(&compose(&PlaceSource { runtime, ..place_source(&cfg) })[0].say.text);
        assert!(shown.contains("[0] is showing"), "{shown}");
    });
}

#[test]
fn memo_follows_language() {
    let cfg = FakeSettingsSource::default().with_text(keys::wallpaper::VOLUME, "40");
    let title = |tag| {
        crate::i18n::with_language(tag, || {
            let source = place_source(&cfg);
            let found = overview(&source);
            found
                .changes
                .iter()
                .find(|change| change.path == keys::wallpaper::VOLUME)
                .map(|change| change.spot.clone())
        })
    };
    assert_ne!(title("en-US"), title("fr-FR"));
}

#[test]
fn unschemaed_count() {
    let cfg = FakeSettingsSource::default()
        .with_flag("filterBar.show.sort.pop", false)
        .with_text("transition.shaderScopes.iris", "primary");
    let found = overview(&place_source(&cfg));
    for path in ["filterBar.show.sort.pop", "transition.shaderScopes.iris"] {
        assert!(found.changes.iter().any(|change| change.path == path), "{path}");
    }
}

#[test]
fn personal_classes() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::sources::PEXELS_API_KEY, "abcdefghij")
        .with_text(keys::general::LOCALE, "Stockholm")
        .with_text(keys::paths::STEAM_WORKSHOP, "/somewhere");
    let found = overview(&place_source(&cfg));
    for path in [keys::sources::PEXELS_API_KEY, keys::general::LOCALE, keys::paths::STEAM_WORKSHOP]
    {
        let change = found.changes.iter().find(|change| change.path == path).expect(path);
        assert!(change.spared, "{path}");
    }
    let key = found.changes.iter().find(|change| change.path == keys::sources::PEXELS_API_KEY);
    assert!(!key.expect("key").now.contains("abcdefghij"));
}

#[test]
fn stamp_tracks_runtime() {
    let cfg = FakeSettingsSource::default();
    let source = place_source(&cfg);
    let calm = stamp(&source);
    let first = overview(&source);
    assert!(Arc::ptr_eq(&first, &overview(&source)), "memo hit");
    cfg.set_niri(true);
    assert_ne!(stamp(&source), calm);
    let second = overview(&source);
    assert!(!Arc::ptr_eq(&first, &second), "rescanned");
    cfg.set_niri(false);
    assert_eq!(stamp(&source), calm);
}
