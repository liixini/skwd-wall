#![cfg(test)]

use super::*;
use crate::contracts::settings::FolderKind;
use crate::frontend::settings::test_source::FakeSettingsSource;

fn cfg() -> FakeSettingsSource {
    FakeSettingsSource::default()
}

fn weather_rows(cards: &[(Card, Vec<Row>)]) -> Vec<Row> {
    let section = &cards.iter().find(|(card, _)| card.title == "Location & weather").unwrap().1;
    section
        .iter()
        .flat_map(|row| match &row.control {
            Control::Details { rows, .. } => rows.clone(),
            _ => vec![row.clone()],
        })
        .collect()
}

#[test]
fn outdoors_display_controls() {
    use crate::contracts::daemon::{
        LightPhase, OutdoorMatch, OutdoorOutput, OutdoorSeason, OutdoorStatus, WeatherSource,
        WeatherState, WeatherStatus,
    };

    let cfg = cfg()
        .with_flag(keys::weather::ENABLED, true)
        .with_text(keys::weather::SOURCE, keys::weather::SOURCE_OUTDOORS);
    let status = WeatherStatus {
        enabled: true,
        source: WeatherSource::Outdoors,
        status: WeatherState::Active,
        weather: vec!["cloudy".into()],
        outdoors: Some(OutdoorStatus {
            season: OutdoorSeason::Autumn,
            light: LightPhase::Dusk,
            match_kind: OutdoorMatch::Library,
            location: "Stockholm".into(),
            credits: Some("/assets/outdoors/sources.json".into()),
            outputs: vec![OutdoorOutput {
                output: "DP-1".into(),
                key: "woods.png".into(),
                name: "Woods".into(),
                preview: Some("/walls/woods-thumb.png".into()),
                held: false,
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&status), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(rows.iter().any(|row| matches!(
        &row.control,
        Control::ActionBtn { id: ActionId::FollowOutdoors(false), label }
            if label == "Stop following"
    )));
    assert!(
        rows.iter()
            .any(|row| row.title == "Outside now" && row.desc == "Autumn · cloudy · Evening")
    );
    assert!(rows.iter().any(|row| row.title == "Weather location" && row.desc == "Stockholm"));
    assert!(rows.iter().any(|row| matches!(
        &row.control,
        Control::ActionBtn { id: ActionId::OutdoorCredits(path), .. }
            if path == "/assets/outdoors/sources.json"
    )));
    let display = rows.iter().find(|row| row.title == "DP-1").unwrap();
    let Control::StackBar { summary, preview, rows, .. } = &display.control else { panic!() };
    assert_eq!(summary, "Woods");
    assert_eq!(preview.as_deref(), Some("/walls/woods-thumb.png"));
    assert!(matches!(&rows[0].control, Control::ActionChips { items }
        if items.iter().any(|(id, _)| id == &ActionId::OutdoorNext("DP-1".into()))
            && items.iter().any(|(id, _)| id == &ActionId::OutdoorExclude { output: "DP-1".into(), key: "woods.png".into() })
            && items.iter().any(|(id, _)| id == &ActionId::OutdoorPin("DP-1".into()))));
    let section = &cards.iter().find(|(card, _)| card.title == "Location & weather").unwrap().1;
    assert!(section.iter().any(
        |row| matches!(&row.control, Control::Details { id, .. } if id == "weather.advanced")
    ));
}

#[test]
fn outdoors_missing_location() {
    let cards = build_tab("automation", &cfg(), &[], &[], "", &[]);
    let rows = weather_rows(&cards);
    assert!(rows.iter().any(|row| matches!(
        &row.control,
        Control::ActionBtn { id: ActionId::FollowOutdoors(true), .. }
    )));
    assert!(
        rows.iter().any(|row| row.title == "Weather location" && row.desc.contains("Add one city"))
    );
}

#[test]
fn outdoors_policy_controls() {
    let config = cfg()
        .with_text(keys::transition::SHADER, "sand-helix")
        .with_text(keys::theme::POLICY, "wallpaper");
    let cards = build_tab("automation", &config, &[], &[], "", &[]);
    let rows = weather_rows(&cards);
    let transition = rows.iter().find(|row| row.title == "Transition type").unwrap();
    let Control::Dropdown { path, options, current, .. } = &transition.control else { panic!() };
    assert_eq!(path, keys::weather::OUTDOORS_TRANSITION_SHADER);
    assert_eq!(current, "crossfade");
    assert!(options.iter().any(|option| option == &("crossfade".into(), "Crossfade".into())));
    assert!(options.iter().any(|option| option == &("random".into(), "Random".into())));
    assert_eq!(options.len(), super::super::tables::SHADERS.len());
    let retheme = rows.iter().find(|row| row.title == "Colour retheming").unwrap();
    assert!(matches!(&retheme.control, Control::Toggle { path, value: false }
        if path == keys::weather::OUTDOORS_RETHEME));

    let config = cfg()
        .with_text(keys::weather::OUTDOORS_TRANSITION_SHADER, "sand-helix")
        .with_flag(keys::weather::OUTDOORS_RETHEME, true);
    let rows = weather_rows(&build_tab("automation", &config, &[], &[], "", &[]));
    assert!(rows.iter().any(|row| matches!(&row.control,
        Control::Dropdown { path, current, .. }
            if path == keys::weather::OUTDOORS_TRANSITION_SHADER && current == "sand-helix")));
    assert!(rows.iter().any(|row| matches!(&row.control,
        Control::Toggle { path, value: true } if path == keys::weather::OUTDOORS_RETHEME)));
}

#[test]
fn outdoors_pending_controls() {
    use crate::contracts::daemon::{OutdoorOutput, OutdoorStatus, WeatherSource, WeatherStatus};

    let status = WeatherStatus {
        source: WeatherSource::Outdoors,
        outdoors: Some(OutdoorStatus {
            outputs: vec![OutdoorOutput {
                output: "DP-1".into(),
                key: "a.png".into(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg(),
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&status), weather_pending: true, ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(rows.iter().any(|row| matches!(&row.control,
        Control::ActionBtn { label, .. } if label == "Updating…")));
    let display = rows.iter().find(|row| row.title == "DP-1").unwrap();
    assert!(
        matches!(&display.control, Control::StackBar { rows, .. } if matches!(&rows[0].control, Control::Static))
    );
}

#[test]
fn outdoors_fallback_feedback() {
    use crate::contracts::daemon::{
        OutdoorMatch, OutdoorOutput, OutdoorStatus, WeatherSource, WeatherState, WeatherStatus,
    };

    let mut status = WeatherStatus {
        source: WeatherSource::Outdoors,
        enabled: true,
        status: WeatherState::Active,
        outdoors: Some(OutdoorStatus {
            match_kind: OutdoorMatch::Starter,
            outputs: vec![OutdoorOutput {
                output: "DP-1".into(),
                key: "starter.png".into(),
                held: true,
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg(),
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&status), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(
        rows.iter()
            .any(|row| row.title == "Status" && row.desc == "Using the built-in starter set")
    );
    let display = rows.iter().find(|row| row.title == "DP-1").unwrap();
    assert!(matches!(&display.control, Control::StackBar { rows, .. }
        if matches!(&rows[0].control, Control::ActionChips { items }
            if items == &[(ActionId::OutdoorResume("DP-1".into()), "Resume this display".into())])));

    status.status = WeatherState::Unavailable;
    let outdoors = status.outdoors.as_mut().unwrap();
    outdoors.stale = true;
    outdoors.match_kind = OutdoorMatch::Ambient;
    outdoors.outputs[0].held = false;
    outdoors.outputs.push(OutdoorOutput {
        output: "DP-2".into(),
        key: "held.png".into(),
        held: true,
        ..Default::default()
    });
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg(),
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&status), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(
        rows.iter()
            .any(|row| row.title == "Status" && row.desc.contains("last matching wallpapers"))
    );
    let display = rows.iter().find(|row| row.title == "DP-1").unwrap();
    let Control::StackBar { rows: controls, .. } = &display.control else { panic!() };
    let Control::ActionChips { items } = &controls[0].control else { panic!() };
    assert!(items.iter().all(|(id, _)| !matches!(id, ActionId::OutdoorNext(_))));
    assert!(items.iter().any(|(id, _)| matches!(id, ActionId::OutdoorExclude { .. })));
    assert!(items.iter().any(|(id, _)| matches!(id, ActionId::OutdoorPin(_))));
    let held = rows.iter().find(|row| row.title == "DP-2").unwrap();
    let Control::StackBar { rows: controls, .. } = &held.control else { panic!() };
    assert!(matches!(&controls[0].control, Control::ActionChips { items }
        if items == &[(ActionId::OutdoorResume("DP-2".into()), "Resume this display".into())]));
}

#[test]
fn weather_mapping_controls() {
    use crate::contracts::daemon::{WeatherState, WeatherStatus};

    let cfg = cfg().with_flag(keys::weather::ENABLED, true).with_text(keys::weather::RAINY, "42");
    let playlists = vec![crate::contracts::playlists::Playlist {
        id: 42,
        name: "Rain walls".into(),
        ..Default::default()
    }];
    let weather = WeatherStatus {
        weather: vec!["rainy".into(), "windy".into()],
        enabled: true,
        status: WeatherState::Active,
        category: Some("rainy".into()),
        playlist_id: Some(42),
        playlist_name: Some("Rain walls".into()),
        resolved_count: 3,
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&weather), playlists: &playlists, ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(rows.iter().any(|row| matches!(
        &row.control,
        Control::Dropdown { path, current, options, .. }
            if path == keys::weather::RAINY && current == "42"
                && options.iter().any(|(id, label)| id == "42" && label.contains("Rain walls"))
    )));
    assert!(
        rows.iter().any(|row| row.title == "Status" && row.desc == "Weather playlist is active.")
    );
    assert!(rows.iter().any(|row| row.title == "Current weather" && row.desc == "rainy, windy"));
    assert!(
        rows.iter().any(|row| row.title == "Available wallpapers" && row.desc == "3 wallpapers")
    );
}

#[test]
fn semantic_weather_controls() {
    let cfg = cfg().with_text(keys::weather::SOURCE, "semantic");
    let weather = crate::contracts::daemon::WeatherStatus {
        source: crate::contracts::daemon::WeatherSource::Semantic,
        status: crate::contracts::daemon::WeatherState::Active,
        query: Some("rainy streets".into()),
        resolved_count: 4,
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&weather), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    for path in [
        keys::weather::SEMANTIC_CLEAR,
        keys::weather::SEMANTIC_CLOUDY,
        keys::weather::SEMANTIC_RAINY,
        keys::weather::SEMANTIC_SNOWY,
        keys::weather::SEMANTIC_STORMY,
        keys::weather::SEMANTIC_FOGGY,
        keys::weather::SEMANTIC_WINDY,
    ] {
        assert!(rows.iter().any(
            |row| matches!(&row.control, Control::TextField { path: actual, .. } if actual == path)
        ));
    }
    for path in [keys::weather::SEMANTIC_MAX_RESULTS, keys::weather::SEMANTIC_DWELL_SECONDS] {
        assert!(rows.iter().any(
            |row| matches!(&row.control, Control::Number { path: actual, .. } if actual == path)
        ));
    }
    assert!(!rows.iter().any(
        |row| matches!(&row.control, Control::Dropdown { path, .. } if path == keys::weather::RAINY)
    ));
    assert!(rows.iter().any(|row| row.title == "Current search" && row.desc == "rainy streets"));
    assert!(
        rows.iter().any(|row| row.title == "Available wallpapers" && row.desc == "4 wallpapers")
    );
    assert!(rows.iter().any(|row| matches!(
        &row.control,
        Control::Dropdown { path, current, .. }
            if path == keys::weather::SOURCE && current == "semantic"
    )));
}

#[test]
fn semantic_search_unavailable() {
    let weather = crate::contracts::daemon::WeatherStatus {
        source: crate::contracts::daemon::WeatherSource::Semantic,
        status: crate::contracts::daemon::WeatherState::SearchUnavailable,
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg(),
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&weather), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(rows.iter().any(|row| row.title == "Status" && row.desc.contains("Search & tags")));
}

#[test]
fn semantic_unmapped_copy() {
    let weather = crate::contracts::daemon::WeatherStatus {
        source: crate::contracts::daemon::WeatherSource::Semantic,
        status: crate::contracts::daemon::WeatherState::Unmapped,
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg().with_text(keys::weather::SOURCE, "semantic"),
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&weather), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(rows.iter().any(|row| row.title == "Status" && row.desc.contains("search text")));
    assert!(rows.iter().any(|row| row.title == "Change wallpaper with the weather"
        && row.desc.contains("selected source")));
}

#[test]
fn unmapped_weather_visible() {
    let cfg = cfg();
    let weather = crate::contracts::daemon::WeatherStatus {
        weather: vec!["clear".into(), "sunny".into()],
        status: crate::contracts::daemon::WeatherState::Unmapped,
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&weather), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(rows.iter().any(|row| row.title == "Status" && row.desc.contains("No playlist")));
    assert!(rows.iter().any(|row| row.title == "Current weather" && row.desc == "sunny"));
}

#[test]
fn loading_weather_visible() {
    let cfg = cfg();
    let weather = crate::contracts::daemon::WeatherStatus {
        status: crate::contracts::daemon::WeatherState::Loading,
        ..Default::default()
    };
    let cards = build_tab_with_runtime_status(
        "automation",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { weather: Some(&weather), ..Default::default() },
    );
    let rows = weather_rows(&cards);
    assert!(
        rows.iter().any(|row| row.title == "Status" && row.desc == "Updating weather matches.")
    );
}

#[test]
fn resume_requires_direct_pause() {
    let cfg = cfg();
    for resume_available in [false, true] {
        let weather = crate::contracts::daemon::WeatherStatus {
            status: crate::contracts::daemon::WeatherState::Paused,
            paused_outputs: vec!["DP-1".into()],
            resume_available,
            ..Default::default()
        };
        let cards = build_tab_with_runtime_status(
            "automation",
            &cfg,
            &[],
            &[],
            "",
            &[],
            RuntimeStatus { weather: Some(&weather), ..Default::default() },
        );
        let rows = weather_rows(&cards);
        assert!(rows.iter().any(|row| row.title == "Paused displays" && row.desc == "DP-1"));
        assert_eq!(
            rows.iter().any(|row| matches!(
                row.control,
                Control::ActionBtn { id: ActionId::ResumeWeather, .. }
            )),
            resume_available,
        );
    }
}

#[test]
fn excluded_rows_show_scope() {
    use crate::contracts::settings::ExclusionScope;
    use crate::frontend::settings::{TagTone, folder_ledger};
    let excluded = keys::library::EXCLUDED_FOLDERS;
    let cfg = cfg()
        .with_array_len(keys::library::FOLDERS, 1)
        .with_text(&format!("{}.0", keys::library::FOLDERS), "/walls")
        .with_array_len(excluded, 3)
        .with_text(&format!("{excluded}.0"), "/walls/anime")
        .with_text(&format!("{excluded}.1"), "/other")
        .with_exclusion_scopes(vec![
            ExclusionScope::Inside("/walls".into()),
            ExclusionScope::Outside,
            ExclusionScope::Empty,
        ]);
    let ledger = folder_ledger(&cfg, FolderKind::Excluded, None);
    let rows: Vec<_> = ledger
        .rows
        .iter()
        .map(|row| (row.path.as_str(), row.tag.as_ref().map(|(text, tone)| (text.as_str(), *tone))))
        .collect();
    assert_eq!(
        rows,
        [
            ("/walls/anime", Some(("Inside walls", TagTone::Quiet))),
            (
                "/other",
                Some(("Not inside any wallpaper folder, so it skips nothing", TagTone::Warn))
            ),
            ("", None),
        ]
    );
}

#[test]
fn watch_tags_follow_status() {
    use crate::contracts::daemon::{LibraryWatchRootStatus, LibraryWatchStatus};
    use crate::frontend::settings::{TagTone, folder_ledger};
    let cfg = cfg()
        .with_array_len(keys::library::FOLDERS, 3)
        .with_text("library.folders.0", "/native")
        .with_text("library.folders.1", "/nas")
        .with_text("library.folders.2", "/gone");
    let root = |path: &str, mode: &str, error: Option<&str>| LibraryWatchRootStatus {
        path: path.into(),
        mode: mode.into(),
        last_poll_error: error.map(str::to_owned),
        ..LibraryWatchRootStatus::default()
    };
    let watch = LibraryWatchStatus {
        roots: vec![
            root("/native", "native", None),
            root("/nas", "polling", None),
            root("/gone", "polling", Some("denied")),
        ],
        ..LibraryWatchStatus::default()
    };
    let ledger = folder_ledger(&cfg, FolderKind::Source, Some(&watch));
    let tags: Vec<_> = ledger
        .rows
        .iter()
        .map(|row| (row.tag.as_ref().map(|(_, tone)| *tone), row.note.is_some()))
        .collect();
    assert_eq!(tags, [(None, false), (Some(TagTone::Info), false), (Some(TagTone::Warn), true)]);
    assert!(folder_ledger(&cfg, FolderKind::Source, None).rows.iter().all(|row| row.tag.is_none()));
}

#[test]
fn profile_page_lists_states() {
    use crate::contracts::settings::ProfileState;
    use crate::frontend::settings::profiles_page;
    let folders = [String::from("/walls"), String::from("/walls/anime")];
    let cfg = cfg()
        .with_array_len(keys::library::PROFILES, 1)
        .with_text("library.profiles.0.id", "public")
        .with_text("library.profiles.0.name", "Public")
        .with_text(keys::library::PROFILE, "public")
        .with_profile_state(0, "/walls/anime", ProfileState::Hide);
    let page = profiles_page(&cfg, &folders, 1, "");
    let names: Vec<_> =
        page.profiles.iter().map(|entry| (entry.name.as_str(), entry.active)).collect();
    assert_eq!(names, [("Everything", false), ("Public", true)]);
    assert_eq!(page.current().name_key().as_deref(), Some("library.profiles.0.name"));
    let states: Vec<_> = page.tree.iter().map(|node| (node.depth, node.state)).collect();
    assert_eq!(
        states,
        [
            (0, Some(ProfileState::Show)),
            (1, Some(ProfileState::Hide)),
            (0, Some(ProfileState::Show))
        ]
    );
    assert!(page.tree[0].children && page.tree[0].mixed && page.tree[0].hidden_inside == 1);
    assert_eq!(page.tree[2].path, "library.profiles.0.workshop");
    assert!(page.summary.contains('1'));
    let everything = profiles_page(&cfg, &folders, 0, "");
    assert!(everything.current().builtin());
    assert!(everything.tree.iter().all(|node| node.path.is_empty()));
    assert_eq!(profiles_page(&cfg, &folders, 9, "").selected, 1);
}

#[test]
fn profile_filter_keeps_ancestors() {
    use crate::frontend::settings::profiles_page;
    let folders = [
        String::from("/w"),
        String::from("/w/anime"),
        String::from("/w/games"),
        String::from("/w/games/x"),
    ];
    let cfg =
        cfg().with_array_len(keys::library::PROFILES, 1).with_text("library.profiles.0.id", "p");
    let page = profiles_page(&cfg, &folders, 1, "GAM");
    let shape: Vec<_> = page.tree.iter().map(|node| (node.label.as_str(), node.children)).collect();
    assert_eq!(shape, [("/w", true), ("games", false)]);
    assert!(!page.no_match);
    let deep = profiles_page(&cfg, &folders, 1, "x");
    let labels: Vec<_> = deep.tree.iter().map(|node| node.label.as_str()).collect();
    assert_eq!(labels, ["/w", "games", "x"]);
    let none = profiles_page(&cfg, &folders, 1, "zzz");
    assert!(none.no_match && none.tree.is_empty());
}

#[test]
fn matugen_toggle_reflects_the_active_generator_and_keeps_configuration_editable() {
    for (cfg, enabled) in [
        (cfg(), false),
        (cfg().with_flag(keys::features::MATUGEN, true), false),
        (cfg().with_text(keys::theme::BACKEND, "matugen"), true),
        (
            cfg()
                .with_text(keys::theme::BACKEND, "matugen")
                .with_flag(keys::features::MATUGEN, false),
            false,
        ),
        (
            cfg()
                .with_text(keys::theme::BACKEND, "matugen")
                .with_text(keys::theme::POLICY, "fixed"),
            false,
        ),
        (
            cfg()
                .with_text(keys::theme::BACKEND, "matugen")
                .with_text(keys::theme::AUTHORITY, "dms"),
            false,
        ),
    ] {
        let mut builder = Builder { cfg: &cfg, cards: Vec::new() };
        theme_tabs::tab_matugen(&mut builder);
        let rows: Vec<_> = builder.cards.iter().flat_map(|(_, rows)| rows).collect();
        assert!(rows.iter().any(|row| matches!(&row.control,
            Control::Toggle { path, value } if path == keys::features::MATUGEN && *value == enabled
        )));
        let paths: Vec<_> = rows
            .iter()
            .filter_map(|row| match &row.control {
                Control::TextField { path, .. } => Some(path.as_str()),
                _ => None,
            })
            .collect();
        assert!(paths.contains(&keys::matugen::DEFAULT_CONFIG));
        assert!(paths.contains(&keys::system::EXTERNAL_MATUGEN_COMMAND));
    }
}

#[test]
fn playback_exposes_wallpaper_load_timeout_in_seconds() {
    let cards = build_tab("playback", &cfg(), &[], &[], "", &[]);
    let controls: Vec<_> = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| match &row.control {
            Control::Number { path, unit, .. } if path == keys::paper::LOAD_TIMEOUT_SECONDS => {
                Some((row.title.as_str(), *unit))
            }
            _ => None,
        })
        .collect();
    assert_eq!(controls, [("Wallpaper load timeout", "s")]);
}

#[test]
fn noctalia_mode_overrides_in_integrations() {
    for mode in ["follow", "keep", "dark", "light", "auto"] {
        let cfg = cfg().with_text(keys::noctalia::THEME_MODE, mode);
        let cards = build_tab("integrations", &cfg, &[], &[], "", &[]);
        let controls: Vec<_> = cards
            .iter()
            .flat_map(|(_, rows)| rows)
            .filter_map(|row| match &row.control {
                Control::Dropdown { path, current, options, .. }
                    if path == keys::noctalia::THEME_MODE =>
                {
                    Some((current, options))
                }
                _ => None,
            })
            .collect();
        assert_eq!(controls.len(), 1);
        assert_eq!(controls[0].0, mode);
        assert_eq!(
            controls[0].1.iter().map(|(value, _)| value.as_str()).collect::<Vec<_>>(),
            ["follow", "keep", "dark", "light", "auto"]
        );
    }
}

#[test]
fn picker_controls_bindings() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let titles: Vec<_> = cards.iter().map(|(card, _)| card.title).collect();
    assert_eq!(&titles[4..], ["Controls"]);
    let controls = &cards
        .iter()
        .find(|(card, _)| card.title == "Controls")
        .expect("picker controls section")
        .1;
    let segments: Vec<_> = controls
        .iter()
        .filter(|row| matches!(row.control, Control::Segment))
        .map(|row| row.title.as_str())
        .collect();
    assert_eq!(
        segments,
        [
            "Wallpaper actions",
            "Panels",
            "Filters & tags",
            "Folders",
            "Navigation",
            "Downloads",
            "Fixed controls",
            "Reset controls",
        ]
    );
    let bindings: Vec<_> = controls
        .iter()
        .filter_map(|row| match &row.control {
            Control::KeyBinding { path, default, .. } => Some((path.as_str(), *default)),
            _ => None,
        })
        .collect();
    assert_eq!(bindings.len(), crate::contracts::picker::KEY_BINDINGS.len());
    assert_eq!(bindings[0], (skwd_config::keys::keybind::SELECT, "click"));
    assert!(bindings.contains(&(skwd_config::keys::keybind::THEME_PANEL, "c")));
}

#[test]
fn shortcut_conflicts_get_their_own_segment() {
    let cfg = cfg().with_bindings(crate::domain::input::InputMap::from_bindings([
        (crate::domain::input::InputAction::Playlists, String::from("c")),
        (crate::domain::input::InputAction::ThemePanel, String::from("c")),
    ]));
    let cards = build_tab("picker", &cfg, &[], &[], "", &[]);
    let rows = &cards.iter().find(|(card, _)| card.title == "Controls").unwrap().1;
    let conflicts = rows
        .iter()
        .position(|row| row.title == "Conflicts" && matches!(row.control, Control::Segment))
        .expect("conflict segment");
    let fixed = rows
        .iter()
        .position(|row| row.title == "Fixed controls" && matches!(row.control, Control::Segment))
        .expect("fixed controls segment");
    assert_eq!(conflicts + 2, fixed);
    assert!(matches!(rows[conflicts + 1].control, Control::Static));
}

#[test]
fn theme_dropdown_presets() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::theme::BACKEND, "static")
        .with_saved_theme("Mine");
    let cards = build_tab("theme", &cfg, &[], &[], "", &[]);
    let options = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .find_map(|row| match &row.control {
            Control::Dropdown { path, options, .. } if path == "theme.staticTheme" => {
                Some(options.clone())
            }
            _ => None,
        })
        .expect("static theme dropdown exists when the backend is static");
    let values: Vec<&str> = options.iter().map(|(v, _)| v.as_str()).collect();
    assert!(values.contains(&"nord"));
    assert!(values.contains(&"dracula"));
    assert!(values.contains(&"Mine"));
    assert!(values.contains(&"custom"));
}

#[test]
fn theme_page_separation() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::theme::POLICY, "wallpaper")
        .with_text(keys::theme::AUTHORITY, "skwd")
        .with_text(keys::theme::ENGINE, "wallust");
    let available = [String::from("wallust"), String::from("noctalia")];
    let cards = build_tab("theme", &cfg, &[], &[], "", &available);
    let titles: Vec<_> = cards.iter().map(|(card, _)| card.title).collect();
    assert!(titles.contains(&"Behaviour"));
    assert!(titles.contains(&"Who drives colour"));
    assert!(titles.contains(&"Colour source"));
    assert!(titles.contains(&"Engine options"));

    let paths: Vec<_> = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| match &row.control {
            Control::Dropdown { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect();
    assert!(paths.contains(&keys::theme::POLICY));
    assert!(paths.contains(&keys::theme::AUTHORITY));
    assert!(paths.contains(&keys::theme::ENGINE));
    assert!(!paths.contains(&keys::theme::BACKEND));
}

#[test]
fn external_authority_replaces_engine() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::theme::POLICY, "wallpaper")
        .with_text(keys::theme::AUTHORITY, "noctalia");
    let cards = build_tab("theme", &cfg, &[], &[], "", &[String::from("noctalia")]);
    let titles: Vec<_> = cards.iter().map(|(card, _)| card.title).collect();
    assert!(titles.contains(&"Who drives colour"));
    assert!(titles.contains(&"Imported result"));
    assert!(!titles.contains(&"Colour source"));
}

#[test]
fn matugen_under_theme() {
    fn row_paths(rows: &[Row]) -> Vec<&str> {
        rows.iter()
            .flat_map(|row| match &row.control {
                Control::Toggle { path, .. } | Control::TextField { path, .. } => {
                    vec![path.as_str()]
                }
                Control::Details { rows, .. } | Control::StackBar { rows, .. } => row_paths(rows),
                _ => Vec::new(),
            })
            .collect()
    }
    fn paths(cards: &[(Card, Vec<Row>)]) -> Vec<&str> {
        cards.iter().flat_map(|(_, rows)| row_paths(rows)).collect()
    }

    let cfg = FakeSettingsSource::default().with_array_len(keys::integrations::LIST, 1);
    let theme = build_tab("theme", &cfg, &[], &[], "", &[]);
    let integrations = build_tab("integrations", &cfg, &[], &[], "", &[]);
    let theme_paths = paths(&theme);
    let integration_paths = paths(&integrations);

    assert!(theme_paths.contains(&keys::features::MATUGEN));
    assert!(theme_paths.contains(&keys::matugen::DEFAULT_CONFIG));
    assert!(theme_paths.contains(&keys::system::EXTERNAL_MATUGEN_COMMAND));
    assert!(theme_paths.contains(&"integrations.0.template"));
    assert!(!integration_paths.contains(&keys::features::MATUGEN));
    assert!(!integration_paths.iter().any(|path| path.starts_with("workspace.")));
    assert!(integrations.iter().any(|(card, _)| card.title == "Lockscreens"));
    assert!(!integrations.iter().any(|(card, _)| card.title == "KDE Plasma"));
}

#[test]
fn lockscreens_under_integrations() {
    let cards = build_tab("integrations", &cfg(), &[], &[], "", &[]);
    let lockscreens = &cards.iter().find(|(card, _)| card.title == "Lockscreens").unwrap().1;
    let names: Vec<_> = lockscreens.iter().map(|row| row.title.as_str()).collect();
    assert_eq!(
        names,
        [
            "KDE Plasma",
            "Hyprlock",
            "Swaylock",
            "Dank Material Shell",
            "Noctalia",
            "SDDM",
            "Advanced"
        ]
    );
    assert!(cards.iter().any(|(card, _)| card.title == "Noctalia"));
    assert!(cards.iter().any(|(card, _)| card.title == "Dank Material Shell"));
    assert!(lockscreens[..6].iter().all(|row| matches!(row.control, Control::ActionBtn { .. })));
    assert!(
        matches!(&lockscreens[6].control, Control::Details { id, .. } if id == keys::lock_screens::MODE)
    );
}

#[test]
fn lockscreen_source_controls() {
    let off = build_tab("integrations", &cfg(), &[], &[], "", &[]);
    let static_cfg = cfg().with_text(keys::lock_screens::MODE, keys::lock_screens::STATIC);
    let static_cards = build_tab("integrations", &static_cfg, &[], &[], "", &[]);
    let source = |cards: Vec<(Card, Vec<Row>)>| {
        cards.into_iter().find(|(card, _)| card.title == "Lockscreens").unwrap().1
    };
    let off_rows = source(off);
    let Control::Details { rows: off_advanced, .. } = &off_rows[6].control else { panic!() };
    assert!(matches!(&off_advanced[0].control, Control::Dropdown { options, current, .. }
        if current == keys::lock_screens::FOLLOW
            && options.iter().map(|(value, _)| value.as_str()).collect::<Vec<_>>()
                == [keys::lock_screens::FOLLOW, keys::lock_screens::STATIC]));
    assert!(!off_advanced.iter().any(|row| matches!(&row.control, Control::TextField { path, .. } if path == keys::lock_screens::IMAGE)));
    let static_rows = source(static_cards);
    let Control::Details { rows: static_advanced, .. } = &static_rows[6].control else { panic!() };
    assert!(static_advanced.iter().any(|row| matches!(&row.control, Control::TextField { path, .. } if path == keys::lock_screens::IMAGE)));
}

#[test]
fn lockscreen_actions_match_state() {
    use crate::contracts::settings::LockScreenTarget;

    let legacy = cfg().with_flag(keys::lock_screens::HYPRLOCK, true);
    let enabled = cfg()
        .with_text(keys::lock_screens::MODE, keys::lock_screens::FOLLOW)
        .with_flag(keys::lock_screens::HYPRLOCK, true);
    for (source, expected) in [(legacy, false), (enabled, true)] {
        let cards = build_tab("integrations", &source, &[], &[], "", &[]);
        let rows = &cards.iter().find(|(card, _)| card.title == "Lockscreens").unwrap().1;
        assert!(matches!(&rows[1].control,
            Control::ActionBtn { id: ActionId::SetLockScreen(LockScreenTarget::Hyprlock, next), label }
                if *next != expected && label == if expected { "Disable" } else { "Enable" }));
    }
}

#[test]
fn sddm_theme_choices() {
    let themes = vec!["breeze".to_string(), "elarun".to_string()];
    let cards = build_tab_with_runtime_status(
        "integrations",
        &cfg(),
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { sddm_themes: &themes, ..Default::default() },
    );
    let rows = &cards.iter().find(|(card, _)| card.title == "Lockscreens").unwrap().1;
    assert!(matches!(&rows[6].control, Control::ActionChips { items }
        if matches!(&items[0].0, ActionId::ChooseSddmTheme(Some(theme)) if theme == "breeze")
            && matches!(&items[2].0, ActionId::ChooseSddmTheme(None))));
    assert!(matches!(&rows[7].control, Control::Details { .. }));
}

#[test]
fn plasma_lock_modes_preserved() {
    let modes = [
        ("off", None),
        ("static", Some(keys::plasma::LOCK_SCREEN_IMAGE)),
        ("follow", Some(keys::plasma::LOCK_SCREEN_DYNAMIC)),
    ];
    for (mode, conditional) in modes {
        let source = cfg().with_text(keys::plasma::LOCK_SCREEN_MODE, mode);
        let cards = build_tab("integrations", &source, &[], &[], "", &[]);
        let lockscreens = &cards.iter().find(|(card, _)| card.title == "Lockscreens").unwrap().1;
        let Control::Details { rows, .. } = &lockscreens[6].control else { panic!() };
        assert!(
            matches!(&rows[1].control, Control::Dropdown { path, current, .. } if path == keys::plasma::LOCK_SCREEN_MODE && current == mode)
        );
        let paths: Vec<_> = rows
            .iter()
            .skip(2)
            .filter_map(|row| match &row.control {
                Control::TextField { path, .. } | Control::Dropdown { path, .. } => {
                    Some(path.as_str())
                }
                _ => None,
            })
            .collect();
        assert_eq!(paths, conditional.into_iter().collect::<Vec<_>>());
    }
}

#[test]
fn lockscreen_paths_in_schema() {
    for path in [
        keys::lock_screens::MODE,
        keys::lock_screens::IMAGE,
        keys::lock_screens::HYPRLOCK,
        keys::lock_screens::SWAYLOCK,
        keys::lock_screens::DMS,
        keys::lock_screens::NOCTALIA,
        keys::lock_screens::SDDM_IMAGE,
    ] {
        assert!(crate::contracts::settings::schema::find(path).is_some(), "{path}");
    }
}

#[test]
fn niri_unified_section() {
    let cfg = FakeSettingsSource::default();
    let mut builder = Builder { cfg: &cfg, cards: Vec::new() };
    theme_tabs::tab_niri(&mut builder, &[String::from("Catppuccin")]);

    let titles: Vec<_> = builder.cards.iter().map(|(card, _)| card.title).collect();
    assert_eq!(titles, ["Niri"]);
    assert_eq!(builder.cards[0].1.len(), 16);
    assert!(matches!(
        &builder.cards[0].1[5].control,
        Control::Code { snippet } if *snippet == NIRI_SNIPPET
    ));
    let blur: Vec<_> = builder.cards[0].1[7..13]
        .iter()
        .map(|row| match &row.control {
            Control::Toggle { path, value } => (path.as_str(), Some(*value)),
            Control::Number { path, .. } => (path.as_str(), None),
            other => panic!("unexpected blur control {other:?}"),
        })
        .collect();
    assert_eq!(
        blur,
        [
            (keys::niri::BACKDROP_BLUR_STATIC, Some(true)),
            (keys::niri::BACKDROP_BLUR_STATIC_RADIUS, None),
            (keys::niri::BACKDROP_BLUR_VIDEO, Some(false)),
            (keys::niri::BACKDROP_BLUR_VIDEO_RADIUS, None),
            (keys::niri::BACKDROP_BLUR_WE, Some(false)),
            (keys::niri::BACKDROP_BLUR_WE_RADIUS, None),
        ]
    );
    assert!(matches!(
        &builder.cards[0].1.last().expect("appearance palette").control,
        Control::Dropdown { path, .. } if path == keys::niri::BACKDROP_THEME
    ));
}

#[test]
fn niri_stationary_controls_use_current_wallpaper() {
    let cfg = FakeSettingsSource::default().with_text(keys::niri::OVERVIEW_MODE, "stationary");
    let mut builder = Builder { cfg: &cfg, cards: Vec::new() };
    theme_tabs::tab_niri(&mut builder, &[String::from("Catppuccin")]);
    let rows = &builder.cards[0].1;
    let paths: Vec<&str> = rows
        .iter()
        .filter_map(|row| match &row.control {
            Control::Dropdown { path, .. }
            | Control::Toggle { path, .. }
            | Control::TextField { path, .. }
            | Control::Number { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect();
    for path in [
        keys::niri::OVERVIEW_MODE,
        keys::niri::OVERVIEW_BACKDROP,
        keys::niri::BACKDROP_BLUR_STATIC,
        keys::niri::BACKDROP_BLUR_STATIC_RADIUS,
        keys::niri::BACKDROP_BLUR_VIDEO,
        keys::niri::BACKDROP_BLUR_VIDEO_RADIUS,
        keys::niri::BACKDROP_BLUR_WE,
        keys::niri::BACKDROP_BLUR_WE_RADIUS,
        keys::niri::BACKDROP_DIM,
    ] {
        assert!(paths.contains(&path), "missing {path}");
    }
    for path in [
        keys::niri::BACKDROP,
        keys::niri::BACKDROP_FOLLOW_WALLPAPER,
        keys::niri::BACKDROP_AUTO_THEME,
        keys::niri::BACKDROP_THEME,
    ] {
        assert!(!paths.contains(&path), "irrelevant stationary control {path}");
    }
    assert!(rows.iter().any(|row| matches!(&row.control,
        Control::Dropdown { path, options, current, .. }
        if path == keys::niri::OVERVIEW_MODE && current == "stationary"
        && options.iter().map(|(value, _)| value.as_str()).collect::<Vec<_>>() == ["separate", "stationary"])));
}

#[test]
fn monitor_choices_detected() {
    let cfg = FakeSettingsSource::default().with_text("monitor", "DP-9");
    let cards = build_tab_with_runtime_status(
        "picker",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus {
            outputs: &[String::from("HDMI-A-1"), String::from("DP-1"), String::from("DP-1")],
            ..Default::default()
        },
    );
    let options = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .find_map(|row| match &row.control {
            Control::Chips { path, options, current, .. } if path == "monitor" => {
                assert_eq!(current, "DP-9");
                Some(options.clone())
            }
            _ => None,
        })
        .expect("picker settings expose monitor choices");
    assert_eq!(
        options,
        [
            (String::new(), String::from("Focused monitor")),
            (String::from("DP-1"), String::from("DP-1")),
            (String::from("HDMI-A-1"), String::from("HDMI-A-1")),
            (String::from("DP-9"), String::from("DP-9 (unavailable)")),
        ]
    );
}

#[test]
fn displays_placement_lock() {
    let cfg = FakeSettingsSource::default()
        .with_text("display.fillMode", "fill")
        .with_flag("display.outputLocks.DP-1", true);
    let statuses = [crate::contracts::daemon::OutputStatus {
        name: String::from("DP-1"),
        logical_width: 2560,
        logical_height: 1440,
        current: String::from("/walls/forest.png"),
        kind: crate::contracts::media::MediaKind::Static,
        path: String::from("/walls/forest.png"),
        fill: String::from("fit"),
        ..Default::default()
    }];
    let previews = std::collections::HashMap::from([(
        String::from("DP-1"),
        String::from("/thumbs/forest.webp"),
    )]);
    let cards = build_tab_with_runtime_status(
        "displays",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus {
            outputs: &[String::from("DP-1")],
            statuses: &statuses,
            previews: Some(&previews),
            ..Default::default()
        },
    );
    let rows = &cards[0].1;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "DP-1");
    assert!(rows[0].desc.contains("2560 × 1440"));
    let Control::StackBar { id, summary, preview, rows, .. } = &rows[0].control else {
        panic!("display uses the compact stack bar");
    };
    assert_eq!(id, "display:DP-1");
    assert!(summary.contains("forest.png"));
    assert_eq!(preview.as_deref(), Some("/thumbs/forest.webp"));
    assert!(rows[0].title.contains("Placement"));
    assert!(matches!(
        &rows[0].control,
        Control::Chips { path, current, .. }
            if path == "display.fillModes.DP-1" && current == "fit"
    ));
    assert!(matches!(
        &rows[1].control,
        Control::Toggle { path, value: true } if path == "display.outputLocks.DP-1"
    ));
}

#[test]
fn remembered_monitor_offline() {
    let cfg = FakeSettingsSource::default().with_text("display.fillMode", "fill");
    let statuses = [crate::contracts::daemon::OutputStatus {
        name: String::from("DP-3"),
        target: String::from("@monitor:Dell U2723QE @ DP-3"),
        connected: false,
        width: 2560,
        height: 1440,
        current: String::from("/walls/return.png"),
        kind: crate::contracts::media::MediaKind::Static,
        path: String::from("/walls/return.png"),
        ..Default::default()
    }];
    let cards = build_tab_with_runtime_status(
        "displays",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus {
            outputs: &[],
            statuses: &statuses,
            previews: Some(&std::collections::HashMap::new()),
            ..Default::default()
        },
    );
    let row = &cards[0].1[0];
    assert_eq!(row.title, "DP-3");
    assert_eq!(row.desc, "Offline · 2560 × 1440");
    let Control::StackBar { id, summary, .. } = &row.control else {
        panic!("remembered display uses the compact stack bar");
    };
    assert_eq!(id, "display:@monitor:Dell U2723QE @ DP-3");
    assert_eq!(summary, "return.png");
}

#[test]
fn library_watching_surfaces_polling_state_and_controls() {
    let cfg = FakeSettingsSource::default()
        .with_flag(keys::library::POLLING_FALLBACK, true)
        .with_text(keys::library::POLLING_INTERVAL_SECONDS, "45");
    let now_ms =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()
            as u64;
    let status = crate::contracts::daemon::LibraryWatchStatus {
        ok: true,
        degraded: true,
        mode: crate::contracts::daemon::LibraryWatchMode::POLLING.to_string(),
        detail: String::from("native watching unavailable for one root"),
        interval_seconds: Some(45),
        entry_budget_per_root: Some(2048),
        last_successful_convergence_unix_ms: Some(now_ms.saturating_sub(65_000)),
        roots: vec![crate::contracts::daemon::LibraryWatchRootStatus {
            path: String::from("/mnt/library"),
            mode: crate::contracts::daemon::LibraryWatchMode::POLLING.to_string(),
            last_successful_convergence_unix_ms: Some(now_ms.saturating_sub(65_000)),
            ..Default::default()
        }],
    };
    let cards = build_tab_with_runtime_status(
        "library",
        &cfg,
        &[],
        &[],
        "",
        &[],
        RuntimeStatus { library_watch: Some(&status), ..RuntimeStatus::default() },
    );
    let rows = &cards
        .iter()
        .find(|(card, _)| card.title == "Library watching")
        .expect("library watching section")
        .1;

    assert_eq!(rows[0].title, "Polling fallback active");
    assert!(rows[0].desc.contains("one library folder"));
    assert!(rows[0].desc.contains("2048 entries"));
    assert!(rows[0].desc.contains("every 45 seconds"));
    assert!(rows[0].desc.contains("1 minute ago"));
    assert!(matches!(
        &rows[1].control,
        Control::Toggle { path, value: true } if path == keys::library::POLLING_FALLBACK
    ));
    assert!(matches!(
        &rows[2].control,
        Control::Number { path, unit, .. }
            if path == keys::library::POLLING_INTERVAL_SECONDS && *unit == "s"
    ));
}

#[test]
fn library_watching_explains_recovery_and_unavailable_states() {
    let cfg = FakeSettingsSource::default();
    for (mode, expected_title, expected_help) in [
        (
            crate::contracts::daemon::LibraryWatchMode::RECOVERING,
            "Native watching recovered",
            "hand-off scan is still running",
        ),
        (
            crate::contracts::daemon::LibraryWatchMode::UNAVAILABLE,
            "Library watching unavailable",
            "Enable Polling fallback",
        ),
    ] {
        let status = crate::contracts::daemon::LibraryWatchStatus {
            mode: mode.to_string(),
            ..Default::default()
        };
        let cards = build_tab_with_runtime_status(
            "library",
            &cfg,
            &[],
            &[],
            "",
            &[],
            RuntimeStatus { library_watch: Some(&status), ..RuntimeStatus::default() },
        );
        let status_row = &cards
            .iter()
            .find(|(card, _)| card.title == "Library watching")
            .expect("library watching section")
            .1[0];
        assert_eq!(status_row.title, expected_title);
        assert!(status_row.desc.contains(expected_help));
    }
}

#[test]
fn motion_weights_resets() {
    let cards = build_tab("motion", &cfg(), &[], &[], "", &[]);
    let rows = &cards
        .iter()
        .find(|(card, _)| card.title == "Shared speeds")
        .expect("shared motion section")
        .1;
    assert!(matches!(
        rows.as_slice(),
        [Row { control: Control::MotionWeights { weights }, .. }]
            if weights.iter().map(|(_, path, _)| path.as_str()).collect::<Vec<_>>()
                == [keys::motion::FAST_MS, keys::motion::STANDARD_MS, keys::motion::SLOW_MS]
    ));
}

#[test]
fn selector_no_shader_modes() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    for (_, rows) in &cards {
        for row in rows {
            if let Control::Dropdown { path, options, .. } = &row.control {
                assert!(
                    !path.ends_with("displayMode")
                        || options.iter().all(|(v, _)| !v.starts_with("shader:"))
                );
            }
        }
    }
}

#[test]
fn picker_layout_presets() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    assert!(cards.iter().all(|(card, _)| card.title != "Presets"));
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    assert!(rows.iter().any(|row| row.title == "Display mode"));
    assert!(rows.iter().any(|row| matches!(row.control, Control::Presets { .. })));
    assert!(rows.iter().any(|row| {
        matches!(&row.control, Control::TextField { key, .. } if key == PRESET_NAME_KEY)
    }));
}

#[test]
fn picker_layout_predicate() {
    use crate::frontend::settings::is_picker_layout_section;
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let live: Vec<usize> =
        (0..cards.len()).filter(|&section| is_picker_layout_section("picker", section)).collect();
    let studio = cards
        .iter()
        .position(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    assert_eq!(live, [studio], "the live-scene predicate must follow the composed picker tab");
}

#[test]
fn library_page_predicate() {
    use crate::frontend::settings::{PageSection, page_section};
    let cards = build_tab("library", &cfg(), &[], &[], "", &[]);
    let pages: Vec<(&str, PageSection)> = (0..cards.len())
        .filter_map(|section| {
            page_section("library", section).map(|page| (cards[section].0.title, page))
        })
        .collect();
    assert_eq!(
        pages,
        [
            ("Folders", PageSection::Folders(FolderKind::Source)),
            ("Excluded folders", PageSection::Folders(FolderKind::Excluded)),
            ("Profiles", PageSection::Profiles),
        ]
    );
}

#[test]
fn transition_preview_predicate() {
    use crate::frontend::settings::is_transition_preview_section;
    let sand = FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix");
    let cards = build_tab("motion", &sand, &[], &[], "", &[]);
    let titles: Vec<&str> = cards.iter().map(|(card, _)| card.title).collect();
    let live: Vec<usize> = (0..cards.len())
        .filter(|&section| is_transition_preview_section("motion", section))
        .collect();
    let transitions = titles.iter().position(|title| *title == "Wallpaper transitions").unwrap();
    assert_eq!(live, [transitions], "the preview runs only where its stage is visible");
    assert!(!titles.contains(&"Preview"));
    assert!(cards[transitions].1.iter().any(|row| matches!(row.control, Control::Preview)));
}

#[test]
fn wall_studio_mode_led() {
    let config = FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "wall");
    let cards = build_tab("picker", &config, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    let paths = rows
        .iter()
        .filter_map(|row| match &row.control {
            Control::Toggle { path, .. }
            | Control::Number { path, .. }
            | Control::Dropdown { path, .. }
            | Control::Chips { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for path in
        [keys::selector::GRID_LAYOUT, keys::selector::GRID_COLUMNS, keys::selector::GRID_ROWS]
    {
        assert!(paths.contains(&path), "missing light-edit control for {path}");
    }
    for hidden in [
        keys::selector::GRID_STAGE_SHEAR_X,
        keys::selector::GRID_STAGE_DEPTH_ANGLE,
        keys::selector::GRID_FLOW_WAVE,
        keys::selector::GRID_SCATTER,
        keys::selector::GRID_SCALE_VARIANCE,
    ] {
        assert!(!paths.contains(&hidden), "engine internal leaked into Folio: {hidden}");
    }
    let mode = rows
        .iter()
        .find(|row| row.title == "Composition · Wall mode")
        .expect("Wall exposes authored modes");
    assert!(matches!(
        &mode.control,
        Control::Chips { options, .. }
            if options.iter().any(|(value, _)| value == "cylinder")
    ));

    let cylinder = FakeSettingsSource::default()
        .with_text(keys::selector::DISPLAY_MODE, "wall")
        .with_text(keys::selector::GRID_LAYOUT, "cylinder");
    let cylinder_cards = build_tab("picker", &cylinder, &[], &[], "", &[]);
    let cylinder_rows = &cylinder_cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio")
        .1;
    assert!(cylinder_rows.iter().any(|row| row.title == "Composition · Curvature"));
    assert!(cylinder_rows.iter().any(|row| row.title == "Composition · Cylinder radius"));
}

#[test]
fn hex_studio_shape_families() {
    let config = FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "hex");
    let cards = build_tab("picker", &config, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout studio");
    let layout = rows
        .iter()
        .find(|row| row.title == "Composition · Geometric mode")
        .expect("Geometric exposes authored layout choices");
    assert!(matches!(
        &layout.control,
        Control::Chips { options, .. }
            if options.iter().map(|(value, _)| value.as_str())
                .eq(["flat", "arc", "wave", "s", "cylinder"])
    ));
    let shape = rows
        .iter()
        .find(|row| row.title == "Composition · Tile family")
        .expect("Geometric exposes geometric card shapes");
    assert!(matches!(
        &shape.control,
        Control::Chips { options, .. }
            if options.iter().map(|(value, _)| value.as_str())
                .eq(["triangle", "hexagon", "diamond", "rhombus"])
    ));
    let paths = rows
        .iter()
        .filter_map(|row| match &row.control {
            Control::Number { path, .. } | Control::Dropdown { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    for hidden in [
        keys::selector::HEX_TWIST,
        keys::selector::HEX_SCATTER,
        keys::selector::HEX_STAGE_SHEAR_X,
        keys::selector::HEX_STAGE_DEPTH_ANGLE,
        keys::selector::HEX_LENS,
    ] {
        assert!(!paths.contains(&hidden), "engine internal leaked into Folio: {hidden}");
    }
}

#[test]
fn picker_type_badges() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Cards & previews")
        .expect("picker exposes card presentation settings");
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Toggle { path, value }
                if path == keys::selector::SHOW_TYPE_BADGES && *value
        )
    }));
}

#[test]
fn slices_expose_live_edge_tilt() {
    let config = FakeSettingsSource::default().with_text(keys::selector::DISPLAY_MODE, "slices");
    let cards = build_tab("picker", &config, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Layout & presets")
        .expect("picker exposes its layout settings");
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Number { path, unit, .. }
                if path == keys::selector::SLICE_EDGE_TILT && *unit == "px"
        ) && row.title == "Slice size · Edge tilt"
    }));
}

#[test]
fn filter_search_top_level() {
    let picker = build_tab("picker", &cfg(), &[], &[], "", &[]);
    assert!(picker.iter().all(|(card, _)| card.title != "Filter bar"));

    let filter = build_tab("filter", &cfg(), &[], &[], "", &[]);
    assert!(filter.iter().all(|(card, rows)| {
        card.title != "Weather filter"
            && rows.iter().all(|row| {
                !matches!(&row.control, Control::Toggle { path, .. } if path == keys::general::WEATHER_MATCH)
            })
    }));
    assert_eq!(
        filter.iter().map(|(card, _)| card.title).collect::<Vec<_>>(),
        [
            "Appearance",
            "Buttons",
            "Resolution presets",
            "Defaults",
            "Visibility",
            "Search",
            "Semantic models"
        ]
    );
    assert!(filter.iter().flat_map(|(_, rows)| rows).any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == keys::filter_bar::VISUAL_STYLE
                    && current == "match"
                    && options.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>()
                        == ["match", "slices", "hex", "wall"]
        )
    }));
}

#[test]
fn position_tab_groups_independent_surfaces() {
    let cards = build_tab("position", &cfg(), &[], &[], "", &[]);
    assert_eq!(
        cards.iter().map(|(card, _)| card.title).collect::<Vec<_>>(),
        [
            "Slices picker",
            "Geometric picker",
            "Wall picker",
            "Sandy picker",
            "Card hand picker",
            "Filter bar position",
            "Search panel position",
        ]
    );
    assert!(cards.iter().all(|(card, _)| !card.subtitle.is_empty()));
    let paths = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| match &row.control {
            Control::Number { path, .. } => Some(path.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            keys::selector::SLICE_STAGE_X,
            keys::selector::SLICE_STAGE_Y,
            keys::selector::HEX_STAGE_X,
            keys::selector::HEX_STAGE_Y,
            keys::selector::GRID_STAGE_X,
            keys::selector::GRID_STAGE_Y,
            keys::selector::SANDY_STAGE_X,
            keys::selector::SANDY_STAGE_Y,
            keys::selector::HAND_STAGE_X,
            keys::selector::HAND_STAGE_Y,
            keys::filter_bar::OFFSET_X,
            keys::filter_bar::OFFSET_Y,
            keys::selector::TAG_CLOUD_OFFSET_X,
            keys::selector::TAG_CLOUD_OFFSET_Y,
        ]
    );
}

#[test]
fn resolution_preset_items() {
    let root = keys::filter_bar::RESOLUTION_PRESETS;
    let config = FakeSettingsSource::default()
        .with_array_len(root, 2)
        .with_text(&format!("{root}.0.label"), "FHD")
        .with_text(&format!("{root}.0.orientation"), "wide")
        .with_text(&format!("{root}.0.from"), "1920x1080")
        .with_text(&format!("{root}.0.to"), "2559x1439")
        .with_text(&format!("{root}.1.label"), "2K")
        .with_text(&format!("{root}.1.orientation"), "tall")
        .with_text(&format!("{root}.1.from"), "2560x1440")
        .with_text(&format!("{root}.1.to"), "3839x2159");
    let cards = build_tab("filter", &config, &[], &[], "", &[]);
    let rows = &cards
        .iter()
        .find(|(card, _)| card.title == "Resolution presets")
        .expect("resolution preset section")
        .1;
    assert_eq!(rows.len(), 3);
    let Control::ActionChips { items } = &rows[0].control else {
        panic!("resolution presets must offer size bands");
    };
    assert_eq!(items.len(), 6);
    assert!(matches!(items[0].0, ActionId::CreateResolutionPresetBand("fhd")));
    let Control::Details { id, summary, rows: fields } = &rows[1].control else {
        panic!("resolution preset must use a details item");
    };
    assert_eq!(rows[1].title, "FHD");
    assert_eq!(id, "filterBar.resolutionPresets.0");
    assert_eq!(summary, "Wide · 1920 × 1080 - 2559 × 1439");
    assert_eq!(fields.len(), 4);
    assert!(matches!(
        &fields[0].control,
        Control::TextField { path, .. } if path == "filterBar.resolutionPresets.0.label"
    ));
    assert!(matches!(
        &fields[1].control,
        Control::Chips { path, .. } if path == "filterBar.resolutionPresets.0.orientation"
    ));
    let Control::Details { rows: bounds, .. } = &fields[2].control else {
        panic!("exact dimensions belong in the custom range foldout");
    };
    assert!(matches!(
        &bounds[0].control,
        Control::Resolution { path, .. } if path == "filterBar.resolutionPresets.0.from"
    ));
    assert!(matches!(
        &bounds[1].control,
        Control::Resolution { path, .. } if path == "filterBar.resolutionPresets.0.to"
    ));
    assert!(matches!(
        &fields[3].control,
        Control::ActionBtn { id: ActionId::RemoveResolutionPreset(0), .. }
    ));
}

#[test]
fn selector_flip_controls() {
    let cards = build_tab("motion", &cfg(), &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Picker motion")
        .expect("picker motion section");
    for path in [
        keys::selector::FLIP_EFFECT,
        keys::selector::FLIP_SHADER,
        keys::selector::FLIP_BACK_REVEAL,
        keys::selector::FLIP_DURATION_MS,
    ] {
        assert!(rows.iter().any(|row| match &row.control {
            Control::Toggle { path: actual, .. }
            | Control::Dropdown { path: actual, .. }
            | Control::Number { path: actual, .. } => actual == path,
            _ => false,
        }));
    }
    assert!(rows.iter().any(|row| {
        matches!(
            row.control,
            Control::Toggle { ref path, value: true }
                if path == keys::selector::FLIP_SHADER
        )
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            row.control,
            Control::Toggle { ref path, value: true }
                if path == keys::selector::FLIP_BACK_REVEAL
        )
    }));
}

#[test]
fn transition_style_section() {
    let cfg = FakeSettingsSource::default().with_text(keys::transition::SHADER, "sand-helix");
    let cards = build_tab("motion", &cfg, &[], &[], "", &[]);
    assert_eq!(
        cards.iter().map(|(card, _)| card.title).collect::<Vec<_>>(),
        [
            "Shared speeds",
            "Launch",
            "Picker motion",
            "Wallpaper transitions",
            "Transition performance"
        ]
    );
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Wallpaper transitions")
        .expect("wallpaper transitions section");
    assert!(rows.iter().any(|row| row.title == "Family"));
    assert!(rows.iter().any(|row| row.title == "Transition"));
    assert!(rows.iter().any(|row| row.title == "Show preview"));
    assert!(rows.iter().any(|row| matches!(row.control, Control::Preview)));
}

#[test]
fn transition_perf_families() {
    let cfg = FakeSettingsSource::default().with_text(keys::transition::SHADER, "fadecolor");
    let cards = build_tab("motion", &cfg, &[], &[], "", &[]);
    let (_, rows) = cards
        .iter()
        .find(|(card, _)| card.title == "Transition performance")
        .expect("transition performance section");
    assert!(rows.iter().any(|row| row.title == "Fadecolor"));
    assert!(!rows.iter().any(|row| row.title == "Quality"));
}

#[test]
fn performance_power_controls() {
    let cards = build_tab("performance", &cfg(), &[], &[], "", &[]);
    assert_eq!(cards.first().map(|(card, _)| card.title), Some("Power"));
    let rows: Vec<&Row> = cards.iter().flat_map(|(_, rows)| rows).collect();
    for path in [
        keys::performance::BATTERY_SAVER,
        keys::performance::GPU_PREFERENCE,
        keys::performance::BATTERY_FPS,
        keys::performance::BATTERY_VIDEO_IDLE_SECONDS,
        keys::performance::BATTERY_WALLPAPER_PERFORMANCE,
        keys::general::MAX_FPS,
    ] {
        assert!(
            rows.iter().any(|row| match &row.control {
                Control::Toggle { path: actual, .. }
                | Control::Dropdown { path: actual, .. }
                | Control::Number { path: actual, .. } => actual == path,
                _ => false,
            }),
            "missing {path}"
        );
    }
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == keys::performance::GPU_PREFERENCE
                    && current == "auto"
                    && options.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>()
                        == ["auto", "low", "high", "none"]
        )
    }));
    assert!(rows.iter().any(|row| row.title == "Wallpaper Engine scene renderer"));
}

#[test]
fn power_status_battery_saver() {
    let status = |cfg: &FakeSettingsSource| {
        build_tab("performance", cfg, &[], &[], "", &[])
            .into_iter()
            .flat_map(|(_, rows)| rows)
            .find(|row| row.title == "Current power source")
            .map(|row| row.desc)
            .expect("performance tab has a power status row")
    };
    let disabled = FakeSettingsSource::default()
        .with_on_battery(true)
        .with_flag(keys::performance::BATTERY_SAVER, false);
    let enabled = FakeSettingsSource::default()
        .with_on_battery(true)
        .with_flag(keys::performance::BATTERY_SAVER, true);

    assert_eq!(status(&disabled), "Battery (automatic limits are disabled)");
    assert_eq!(status(&enabled), "Battery (automatic limits are active)");
    assert_eq!(
        status(&FakeSettingsSource::default()),
        "External power, desktop, or unavailable (automatic limits are inactive)"
    );
}

#[test]
fn sand_cost_format() {
    assert_eq!(super::selector_tab::compact_count(2_764_800), "2.8M");
    assert_eq!(super::selector_tab::compact_count(460_404), "460K");
    assert_eq!(super::selector_tab::compact_count(28_860), "29K");
    assert_eq!(super::selector_tab::heaviness(2_764_800), "Very heavy");
    assert_eq!(super::selector_tab::heaviness(460_404), "Heavy");
    assert_eq!(super::selector_tab::heaviness(113_880), "Moderate");
    assert_eq!(super::selector_tab::heaviness(28_860), "Light");
}

#[test]
fn folder_dropdown_values() {
    let opts = super::builder::folder_dropdown_options(&[
        String::new(),
        String::from("*"),
        String::from("anime"),
        String::from("nature"),
    ]);
    assert_eq!(opts[0].0, "*");
    assert_eq!(opts[1], (String::from("anime"), String::from("anime")));
    assert_eq!(opts[2], (String::from("nature"), String::from("nature")));
    assert_eq!(opts.len(), 3);
}

#[test]
fn folder_sentinel_labels() {
    let opts = super::builder::folder_dropdown_options(&[String::new(), String::from("*")]);
    let all = crate::i18n::tr("settings-filter-all-folders");
    assert_eq!(opts[0], (String::from("*"), all.to_string()));
    assert_ne!(opts[0].1, opts[0].0);
    assert_eq!(opts.len(), 1);
}

#[test]
fn static_renderer_choices() {
    assert_eq!(super::super::tables::ENGINES, [("skwd-paper", "Skwd-paper"), ("awww", "awww")]);
}

#[test]
fn tinier_video_engine_is_visible_but_disabled() {
    let cards = build_tab("playback", &cfg(), &[], &[], "", &[]);
    let row = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .find(|row| {
            matches!(
                &row.control,
                Control::Chips { path, .. } if path == keys::paper::VIDEO_ENGINE
            )
        })
        .expect("playback exposes the video engine choices");
    let Control::Chips { options, disabled, .. } = &row.control else {
        unreachable!();
    };

    assert_eq!(
        options,
        &[
            (String::from("vulkan"), String::from("Vulkan")),
            (String::from("tinier"), String::from("Tinier (work in progress)")),
        ]
    );
    assert_eq!(disabled, &[String::from("tinier")]);
    assert!(row.desc.contains("cannot be selected"));
}

#[test]
fn sources_compact_index() {
    let cfg = cfg()
        .with_flag(keys::features::WALLHAVEN, false)
        .with_flag(keys::sources::UNSPLASH_ENABLED, true);
    let cards = build_tab("sources", &cfg, &[], &[], "", &[]);
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].0.title, "Source providers");

    let expected = [
        ("Wallhaven", "source.wallhaven", "Disabled", 19),
        ("Unsplash", "source.unsplash", "Enabled", 3),
        ("Pexels", "source.pexels", "Disabled", 3),
        ("Bing daily", "source.bing", "Disabled", 2),
        ("Steam Workshop", "source.workshop", "Enabled", 18),
        ("YouTube", "source.youtube", "Disabled", 4),
    ];
    assert_eq!(cards[0].1.len(), expected.len());
    for (row, (title, expected_id, expected_summary, field_count)) in
        cards[0].1.iter().zip(expected)
    {
        assert_eq!(row.title, title);
        let Control::Details { id, summary, rows } = &row.control else {
            panic!("source providers must use reusable details items");
        };
        assert_eq!(id, expected_id);
        assert_eq!(summary, expected_summary);
        assert_eq!(rows.len(), field_count);
    }
}

#[test]
fn theme_options_available() {
    let loading = super::theme_tabs::engine_options(&[], "skwd-iris");
    let loading_ids: Vec<_> = loading.iter().map(|(id, _)| *id).collect();
    assert_eq!(loading_ids, ["skwd-iris"]);

    let detected: Vec<String> =
        ["skwd-iris", "matugen", "wallust", "caelestia", "noctalia", "dms", "end4"]
            .iter()
            .map(|id| (*id).to_string())
            .collect();
    let options = super::theme_tabs::engine_options(&detected, "skwd-iris");
    let ids: Vec<_> = options.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, ["skwd-iris", "matugen", "wallust"]);

    let legacy = super::theme_tabs::engine_options(&detected, "skwd-wallust");
    assert!(legacy.iter().any(|(id, label)| { *id == "skwd-wallust" && label.contains("legacy") }));

    let drivers = super::theme_tabs::authority_options(&detected, "skwd");
    assert_eq!(
        drivers.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        ["skwd", "caelestia", "noctalia", "dms", "end4"]
    );
}

#[test]
fn search_single_index() {
    let cfg =
        FakeSettingsSource::default().with_text(keys::tagging::DEFAULT_SEARCH_MODE, "describe");
    assert!(
        crate::frontend::settings::visible_tabs(&cfg)
            .iter()
            .any(|(key, label)| *key == "filter" && *label == "Filter & Search")
    );
    let cards = build_tab("filter", &cfg, &[], &[], "Tagging 3/10", &[]);
    assert_eq!(cards.len(), 7);
    let cards = &cards[5..];
    assert_eq!(cards[0].0.title, "Search");
    assert_eq!(cards[1].0.title, "Semantic models");

    let expected = [("Search", "search.discovery", "Describe", 3)];
    assert_eq!(cards[0].1.len(), expected.len());
    for (row, (title, expected_id, expected_summary, field_count)) in
        cards[0].1.iter().zip(expected)
    {
        assert_eq!(row.title, title);
        let Control::Details { id, summary, rows } = &row.control else {
            panic!("search and tagging tasks must use reusable details items");
        };
        assert_eq!(id, expected_id);
        assert_eq!(summary, expected_summary);
        assert_eq!(rows.len(), field_count);
    }

    let nested: Vec<_> = cards[0]
        .1
        .iter()
        .flat_map(|row| match &row.control {
            Control::Details { rows, .. } => rows.as_slice(),
            _ => &[],
        })
        .collect();
    assert!(nested.iter().any(|row| row.title == "Describe search"));
    assert!(nested.iter().any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == skwd_config::keys::tagging::DEFAULT_SEARCH_MODE
                    && options.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>()
                        == ["tags", "describe"]
                    && current == "describe"
        )
    }));
}

#[test]
fn model_shelf_packs() {
    let manifest = "/models/pe-core/semantic-pack.json";
    let cfg = FakeSettingsSource::default()
        .with_array_len(keys::semantic::MODELS, 1)
        .with_text("semantic.models.0.name", "PE-Core L14 336")
        .with_text("semantic.models.0.manifest", manifest)
        .with_text(keys::semantic::MANIFEST, manifest)
        .with_text(keys::semantic::INDEX_PROFILE, "multiview");
    let cards = build_tab("filter", &cfg, &[], &[], "", &[]);
    let rows = &cards.iter().find(|(card, _)| card.title == "Semantic models").unwrap().1;
    assert!(rows.iter().any(|row| {
        matches!(row.control, Control::ActionBtn { id: ActionId::ImportSemanticModel, .. })
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Dropdown { path, options, current, .. }
                if path == keys::semantic::MANIFEST
                    && current == manifest
                    && options.iter().any(|(value, label)| value == manifest && label == "PE-Core L14 336")
        )
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Chips { path, current, .. }
                if path == keys::semantic::INDEX_PROFILE && current == "multiview"
        )
    }));
    assert!(rows.iter().any(|row| {
        matches!(
            &row.control,
            Control::Details { id, summary, .. }
                if id == "semantic.models.0" && summary == manifest
        )
    }));
}

#[test]
fn model_import_progress() {
    let cards =
        build_tab("filter", &FakeSettingsSource::default(), &[], &[], "Validating pack", &[]);
    assert!(cards.iter().find(|(card, _)| card.title == "Semantic models").unwrap().1.iter().any(
        |row| {
            row.title == "Model status"
                && row.desc == "Validating pack"
                && matches!(row.control, Control::Static)
        }
    ));
}

#[test]
fn controls_no_ghosts() {
    let cards = build_tab("picker", &cfg(), &[], &[], "", &[]);
    let (_, rows) =
        cards.iter().find(|(card, _)| card.title == "Controls").expect("picker controls section");
    let mut dump = Vec::new();
    for row in rows {
        let kind = match &row.control {
            Control::KeyBinding { path, default, .. } => format!("binding {path} = {default}"),
            Control::Segment => String::from("segment"),
            Control::Static => String::from("static"),
            Control::ActionBtn { .. } => String::from("action"),
            other => format!("{other:?}"),
        };
        dump.push(format!("{} | {} | {kind}", row.title, row.desc));
    }
    println!("{}", dump.join("\n"));
    assert_eq!(
        rows.iter().filter(|row| matches!(row.control, Control::KeyBinding { .. })).count(),
        crate::contracts::picker::KEY_BINDINGS.len(),
        "every action is rebindable from this page"
    );
    assert!(
        rows.iter()
            .filter(|row| matches!(row.control, Control::Static))
            .all(|row| !row.desc.trim().is_empty()),
        "a static row must carry its own explanation, not an empty value slot"
    );
    assert!(
        rows.iter()
            .filter(|row| matches!(row.control, Control::Segment))
            .all(|row| { !row.title.trim().is_empty() && !row.desc.trim().is_empty() })
    );
    assert!(rows.iter().any(|row| matches!(row.control, Control::ActionBtn { .. })));
}

#[test]
fn performance_lists_detected_devices_and_retains_unavailable_selection() {
    let devices = vec![
        crate::contracts::capabilities::GraphicsDevice {
            id: "uuid:11111111111111111111111111111111".into(),
            name: "Integrated GPU".into(),
        },
        crate::contracts::capabilities::GraphicsDevice {
            id: "uuid:22222222222222222222222222222222".into(),
            name: "Discrete GPU".into(),
        },
    ];
    let cfg = FakeSettingsSource::default()
        .with_devices(devices.clone())
        .with_text(keys::performance::GPU_DEVICE, &devices[1].id);
    let cards = build_tab("performance", &cfg, &[], &[], "", &[]);
    let row = cards.iter().flat_map(|(_, rows)| rows).find(|row| matches!(&row.control, Control::Dropdown { path, .. } if path == keys::performance::GPU_DEVICE)).unwrap();
    let Control::Dropdown { options, current, .. } = &row.control else { unreachable!() };
    assert_eq!(current, &devices[1].id);
    assert_eq!(
        options,
        &vec![
            ("auto".into(), "Automatic".into()),
            (devices[0].id.clone(), devices[0].name.clone()),
            (devices[1].id.clone(), devices[1].name.clone())
        ]
    );
    let cfg =
        FakeSettingsSource::default().with_text(keys::performance::GPU_DEVICE, &devices[1].id);
    let cards = build_tab("performance", &cfg, &[], &[], "", &[]);
    assert!(cards.iter().flat_map(|(_, rows)| rows).any(|row| matches!(&row.control, Control::Dropdown { path, options, current, .. } if path == keys::performance::GPU_DEVICE && current == &devices[1].id && options.iter().any(|(id, label)| id == current && label.contains("unavailable")))));
}

#[test]
fn language_tab_lists_supported_languages_and_system_default() {
    for (saved, expected) in [
        ("auto", "auto"),
        ("sv", "sv-SE"),
        ("es-ES", "es-ES"),
        ("ja_JP.UTF-8", "ja-JP"),
        ("fa_IR.UTF-8", "fa-IR"),
        ("tr", "tr-TR"),
    ] {
        let cfg = cfg().with_text(keys::general::LANGUAGE, saved);
        assert!(
            crate::frontend::settings::visible_tabs(&cfg).iter().any(|(id, _)| *id == "language")
        );
        let cards = build_tab("language", &cfg, &[], &[], "", &[]);
        assert_eq!(cards.len(), 1);
        let Control::Dropdown { path, options, current, .. } = &cards[0].1[0].control else {
            panic!("language choice missing");
        };
        assert_eq!(path, keys::general::LANGUAGE);
        assert_eq!(current, expected);
        assert_eq!(
            options.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            [
                "auto", "en-US", "sv-SE", "es-ES", "pt-BR", "ru-RU", "zh-CN", "ja-JP", "ar-SA",
                "fr-FR", "bn-BD", "ur-PK", "hi-IN", "fa-IR", "tr-TR"
            ]
        );
        assert_eq!(
            options[1..].iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>(),
            [
                "English",
                "Svenska",
                "Español",
                "Português (Brasil)",
                "Русский",
                "简体中文",
                "日本語",
                "العربية",
                "Français",
                "বাংলা",
                "اردو",
                "हिन्दी",
                "فارسی",
                "Türkçe"
            ]
        );
    }
}

#[test]
fn automatic_pause_controls_have_one_playback_owner() {
    let config = cfg();
    let paths = [
        keys::playback::PROCESS_ENABLED,
        keys::playback::PROCESSES,
        keys::playback::FULLSCREEN,
        keys::playback::MAXIMIZED,
        keys::playback::FULLSCREEN_SCOPE,
        keys::playback::RESUME_DELAY,
        keys::playback::MUTE_ON_OTHER_AUDIO,
        keys::paper::IDLE_PAUSE_SECONDS,
    ];
    let cards = build_tab("playback", &config, &[], &[], "", &[]);
    for path in paths {
        assert!(
            cards.iter().flat_map(|(_, rows)| rows).any(|row| match &row.control {
                Control::Toggle { path: actual, .. }
                | Control::Number { path: actual, .. }
                | Control::TextField { path: actual, .. }
                | Control::Dropdown { path: actual, .. } => actual == path,
                Control::ActionBtn { id: ActionId::ChooseRunningProcess, .. } => {
                    path == keys::playback::PROCESSES
                }
                _ => false,
            }),
            "missing {path}"
        );
    }
}

#[test]
fn full_width_pause_is_only_exposed_on_niri() {
    for niri in [false, true] {
        let config = cfg().with_niri(niri).with_flag(keys::niri::FULL_WIDTH_PAUSE, true);
        let cards = build_tab("playback", &config, &[], &[], "", &[]);
        let count = cards.iter().flat_map(|(_, rows)| rows).filter(|row| {
            matches!(&row.control, Control::Toggle { path, .. } if path == keys::niri::FULL_WIDTH_PAUSE)
        }).count();
        assert_eq!(count, usize::from(niri));
    }
}

#[test]
fn hand_mode_exposes_every_hand_knob_in_the_picker_page() {
    let cfg = cfg().with_text(keys::selector::DISPLAY_MODE, "hand");
    let cards = build_tab("picker", &cfg, &[], &[], "", &[]);
    let paths: Vec<String> = cards
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| match &row.control {
            Control::Number { path, .. }
            | Control::Toggle { path, .. }
            | Control::Chips { path, .. } => Some(path.clone()),
            _ => None,
        })
        .collect();
    for key in [
        keys::selector::HAND_COUNT,
        keys::selector::HAND_CARD_WIDTH,
        keys::selector::HAND_CARD_HEIGHT,
        keys::selector::HAND_SPREAD,
        keys::selector::HAND_FAN_ANGLE,
        keys::selector::HAND_FAN_ROLL,
        keys::selector::HAND_ARCH,
        keys::selector::HAND_CORNER_RADIUS,
        keys::selector::HAND_SKEW,
        keys::selector::HAND_PERSPECTIVE,
        keys::selector::HAND_TILT,
        keys::selector::HAND_SPEED,
        keys::selector::HAND_MOVE,
        keys::selector::HAND_MOVE_CORKSCREW,
        keys::selector::HAND_MOVE_CASCADE,
        keys::selector::HAND_MOVE_SHUFFLE,
        keys::selector::HAND_MOVE_RIBBON,
        keys::selector::HAND_MOVE_SPIRAL,
        keys::selector::HAND_RIBBON_AXIS,
        keys::selector::HAND_RIBBONS,
        keys::selector::HAND_CUT,
        keys::selector::HAND_CUT_VARIANCE,
        keys::selector::HAND_GHOSTS,
        keys::selector::HAND_BACKDROP,
        keys::selector::HAND_BACKDROP_BLUR,
        keys::selector::HAND_BOB,
        keys::selector::HAND_REVEAL_FILL,
    ] {
        assert!(paths.iter().any(|path| path == key), "missing {key}");
    }
}

#[test]
fn shell_follow_switches_reflect_independent_targets() {
    for targets in [vec![], vec!["noctalia"], vec!["dms"], vec!["noctalia", "dms"]] {
        let mut source = cfg().with_array_len(keys::theme::TARGETS, targets.len());
        for (index, provider) in targets.iter().enumerate() {
            source = source.with_text(&format!("{}.{index}", keys::theme::TARGETS), provider);
        }
        let cards = build_tab("integrations", &source, &[], &[], "", &[]);
        let switches: Vec<_> = cards
            .iter()
            .flat_map(|(_, rows)| rows)
            .filter_map(|row| match row.control {
                Control::ToggleAction { id: ActionId::SetThemeTarget(provider, next), value } => {
                    assert_eq!(row.title, crate::i18n::tr("settings-integrations-follow-label"));
                    assert_eq!(value, targets.contains(&provider));
                    assert_eq!(next, !value);
                    assert!(row.control.is_compact_action());
                    Some(provider)
                }
                _ => None,
            })
            .collect();
        assert_eq!(switches, ["noctalia", "dms"]);
    }
}

#[test]
fn other_audio_status_row_follows_daemon_playback_state() {
    let audio_rows = |cfg: &FakeSettingsSource,
                      status: &crate::contracts::daemon::PlaybackStatus| {
        build_tab_with_runtime_status(
            "playback",
            cfg,
            &[],
            &[],
            "",
            &[],
            RuntimeStatus { playback: Some(status), ..RuntimeStatus::default() },
        )
        .into_iter()
        .find(|(card, _)| card.title == "Audio")
        .expect("audio section")
        .1
    };
    let enabled = cfg().with_flag(keys::playback::MUTE_ON_OTHER_AUDIO, true);
    for (supported, ducked, expected) in [
        (false, false, "Sound server detection unavailable"),
        (true, false, "Listening for sound from other applications"),
        (true, true, "Wallpaper audio paused: another application is playing sound"),
    ] {
        let status = crate::contracts::daemon::PlaybackStatus {
            other_audio_supported: supported,
            audio_ducked: ducked,
            ..Default::default()
        };
        let rows = audio_rows(&enabled, &status);
        let row = rows.last().expect("status row");
        assert!(row.title.starts_with(expected), "{}", row.title);
        assert!(matches!(row.control, Control::Static));
        assert!(rows.iter().any(|row| matches!(
            &row.control,
            Control::Toggle { path, value: true } if path == keys::playback::MUTE_ON_OTHER_AUDIO
        )));
    }
    let rows = audio_rows(&cfg(), &crate::contracts::daemon::PlaybackStatus::default());
    assert!(rows.iter().all(|row| !matches!(row.control, Control::Static)));
}

#[test]
fn default_and_manual_models_have_destructive_delete_actions() {
    let cfg = FakeSettingsSource::default()
        .with_array_len(keys::semantic::MODELS, 1)
        .with_text("semantic.models.0.manifest", "/manual/model.json");
    let cards = build_tab("filter", &cfg, &[], &[], "", &[]);
    let rows = &cards.iter().find(|(card, _)| card.title == "Semantic models").unwrap().1;
    assert!(rows.iter().any(|row| matches!(
        row.control,
        Control::ActionBtn { id: ActionId::DeleteActiveSemanticModel, .. }
    )));
    let detail = rows
        .iter()
        .find_map(|row| match &row.control {
            Control::Details { rows, .. } => Some(rows),
            _ => None,
        })
        .unwrap();
    assert!(detail.iter().any(|row| matches!(
        row.control,
        Control::ActionBtn { id: ActionId::DeleteSemanticModel(0), .. }
    )));
    assert!(ActionId::DeleteActiveSemanticModel.is_destructive());
    assert!(ActionId::DeleteSemanticModel(0).is_destructive());
}

#[test]
fn removed_default_model_is_not_presented_as_installed() {
    let cfg = FakeSettingsSource::default().with_flag("test.defaultModelAvailable", false);
    let cards = build_tab("filter", &cfg, &[], &[], "", &[]);
    let rows = &cards.iter().find(|(card, _)| card.title == "Semantic models").unwrap().1;
    assert!(!rows.iter().any(|row| matches!(
        row.control,
        Control::ActionBtn { id: ActionId::DeleteActiveSemanticModel, .. }
    )));
    assert!(rows.iter().any(|row| matches!(&row.control,
        Control::Dropdown { path, options, .. } if path == keys::semantic::MANIFEST
            && options == &vec![(String::new(), String::from("Not installed"))])));
}

#[test]
fn matugen_settings_offer_smart_mode_and_scheme() {
    let cfg = cfg()
        .with_text(keys::theme::ENGINE, "matugen")
        .with_text(keys::theme::MODE, "smart")
        .with_text(keys::matugen::SCHEME_TYPE, "scheme-smart");
    let mut builder = Builder { cfg: &cfg, cards: Vec::new() };
    theme_tabs::tab_theme(&mut builder, &[]);
    for (key, expected) in
        [(keys::theme::MODE, "smart"), (keys::matugen::SCHEME_TYPE, "scheme-smart")]
    {
        assert!(builder.cards.iter().flat_map(|(_, rows)| rows).any(|row|
            matches!(&row.control, Control::Dropdown { path, current, options, .. }
                if path == key && current == expected && options.iter().any(|(value, _)| value == expected))));
    }
}

#[test]
fn selector_parallax_toggle_belongs_to_the_active_layout() {
    for (mode, own, other) in [
        ("slices", keys::selector::SLICE_PARALLAX, keys::selector::HEX_PARALLAX),
        ("hex", keys::selector::HEX_PARALLAX, keys::selector::SLICE_PARALLAX),
    ] {
        for enabled in [false, true] {
            let cfg = cfg().with_text(keys::selector::DISPLAY_MODE, mode).with_flag(own, enabled);
            let mut builder = Builder { cfg: &cfg, cards: Vec::new() };
            selector_tab::tab_selector(&mut builder);
            let rows: Vec<_> = builder.cards.iter().flat_map(|(_, rows)| rows).collect();
            assert!(rows.iter().any(|row| matches!(&row.control,
                Control::Toggle { path, value } if path == own && *value == enabled)));
            assert!(!rows.iter().any(|row| matches!(&row.control,
                Control::Toggle { path, .. } if path == other)));
        }
    }
}

#[test]
fn collection_settings_include_own_controls_and_presets() {
    let cfg = cfg().with_text(keys::selector::DISPLAY_MODE, "collection");
    let cards = build_tab("picker", &cfg, &[], &[], "", &[]);
    let rows: Vec<_> = cards.iter().flat_map(|(_, rows)| rows).collect();
    assert!(rows.iter().any(|row| matches!(&row.control, Control::Chips { options, .. }
        if options.iter().any(|(key, _)| key == "collection"))));
    assert!(
        rows.iter().any(|row| matches!(&row.control, Control::Number { path, .. } if path == keys::selector::COLLECTION_SIZE))
    );
    assert!(
        rows.iter().any(
            |row| matches!(&row.control, Control::Presets { mode, .. } if mode == "collection")
        )
    );
}

#[test]
fn current_wallpapers_exposes_live_controls_only_for_connected_media() {
    use crate::contracts::daemon::OutputStatus;
    use crate::contracts::media::MediaKind;
    use crate::frontend::display_control::DisplayControl;
    let cfg = cfg().with_text(keys::display::THEME_OUTPUT, "DP-1");
    for (kind, connected, expected) in [
        (MediaKind::Static, true, 1),
        (MediaKind::Video, true, 3),
        (MediaKind::WallpaperEngine, true, 3),
        (MediaKind::Video, false, 0),
    ] {
        let output = OutputStatus {
            name: "DP-1".into(),
            target: "@monitor:Panel @ DP-1".into(),
            connected,
            kind,
            mute: true,
            volume: 37,
            paused: true,
            manual_paused: false,
            ..Default::default()
        };
        let cards = build_tab_with_runtime_status(
            "displays",
            &cfg,
            &[],
            &[],
            "",
            &[],
            RuntimeStatus {
                outputs: &[],
                statuses: &[output],
                previews: Some(&std::collections::HashMap::default()),
                ..Default::default()
            },
        );
        let Control::StackBar { rows, .. } = &cards[0].1[0].control else {
            panic!("display controls")
        };
        assert_eq!(rows.len(), 2 + expected);
        let live: Vec<_> = rows
            .iter()
            .filter_map(|row| {
                if let Control::Display(control) = &row.control { Some(control) } else { None }
            })
            .collect();
        assert_eq!(live.len(), expected);
        if connected {
            assert!(
                matches!(live[0], DisplayControl::Colours { active: true, target } if target == "@monitor:Panel @ DP-1")
            );
        }
        if expected == 3 {
            assert!(matches!(
                live[1],
                DisplayControl::Audio { muted: true, volume: 37, source_preview: false, .. }
            ));
            assert!(matches!(
                live[2],
                DisplayControl::Playback { paused: true, manual: false, .. }
            ));
        }
    }
}

#[test]
fn stationary_wallpaper_hides_overview_only_pause_control() {
    for (mode, enabled, visible) in
        [("separate", true, true), ("stationary", true, false), ("stationary", false, true)]
    {
        let cfg = FakeSettingsSource::default()
            .with_text(keys::niri::OVERVIEW_MODE, mode)
            .with_flag(keys::niri::OVERVIEW_BACKDROP, enabled)
            .with_flag(keys::niri::OVERVIEW_ONLY_PLAYBACK, true);
        let mut builder = Builder { cfg: &cfg, cards: Vec::new() };
        media_tabs::tab_paper(&mut builder);
        let rows: Vec<_> = builder.cards.iter().flat_map(|(_, rows)| rows).collect();
        assert_eq!(
            rows.iter().any(|row| matches!(&row.control,
            Control::Toggle { path, .. } if path == keys::niri::OVERVIEW_ONLY_PLAYBACK)),
            visible
        );
        assert!(rows.iter().any(|row| matches!(&row.control,
            Control::Toggle { path, .. } if path == keys::playback::FULLSCREEN)));
    }
}

#[test]
fn tree_follows_ancestry() {
    use crate::contracts::settings::ProfileState;
    let folders = [
        String::from("/w"),
        String::from("/w/anime"),
        String::from("/w/anime-2"),
        String::from("/w/anime/x"),
        String::from("/w/anime/x/deep"),
    ];
    let cfg = cfg()
        .with_array_len(keys::library::PROFILES, 1)
        .with_text("library.profiles.0.id", "p")
        .with_array_len(keys::library::EXCLUDED_FOLDERS, 2)
        .with_text("library.excludedFolders.0", "/w/anime/x/old")
        .with_text("library.excludedFolders.1", "raw")
        .with_profile_state(0, "/w/anime/x", ProfileState::Hide)
        .with_profile_state(0, "/w/anime/x/deep", ProfileState::Hide);
    let mut nodes = crate::frontend::settings::profiles_page(&cfg, &folders, 1, "").tree;
    nodes.pop();
    let shape: Vec<(&str, u8, bool, bool)> = nodes
        .iter()
        .map(|node| (node.label.as_str(), node.depth, node.children, node.state.is_none()))
        .collect();
    assert_eq!(
        shape,
        [
            ("/w", 0, true, false),
            ("anime", 1, true, false),
            ("x", 2, true, false),
            ("deep", 3, false, false),
            ("old", 3, false, true),
            ("anime-2", 1, false, false),
            ("raw", 1, false, true),
        ]
    );
    assert_eq!(nodes[0].hidden_inside, 1);
    assert!(nodes[0].mixed && nodes[1].mixed && !nodes[2].mixed);
    let ids: std::collections::HashSet<_> = nodes.iter().map(|node| node.id.as_str()).collect();
    assert_eq!(ids.len(), nodes.len());
}

#[test]
fn fold_marks_nested() {
    use crate::frontend::settings::{TreeNode, mark_folds};
    let node = |id: &str, depth: u8, children: bool| TreeNode {
        id: id.into(),
        path: String::new(),
        label: String::new(),
        depth,
        state: None,
        mixed: false,
        hidden_inside: 0,
        children,
        collapsed: false,
    };
    let mut nodes =
        vec![node("r", 0, true), node("a", 1, true), node("b", 2, false), node("c", 1, true)];
    let toggled: std::collections::HashSet<String> = ["r".to_string(), "c".to_string()].into();
    mark_folds(&mut nodes, &toggled);
    let collapsed: Vec<bool> = nodes.iter().map(|node| node.collapsed).collect();
    assert_eq!(collapsed, [true, true, false, false]);
}
