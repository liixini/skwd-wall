use super::*;
use crate::contracts::daemon::{WeatherSource, WeatherStatus};
use crate::frontend::settings::places::marked;
use crate::frontend::settings::test_source::{
    FakeSettingsSource, place_source, plain, render, weather_status,
};
use crate::frontend::settings::{RuntimeStatus, Slot, place};
use crate::frontend::ui::calm::{VALUE_CLOSE, VALUE_OPEN, sentinel};

const ROTATE: &str = keys::general::RANDOM_ROTATE;
const STATIC: &str = keys::general::RANDOM_INCLUDE_STATIC;
const VIDEO: &str = keys::general::RANDOM_INCLUDE_VIDEO;
const ENGINE: &str = keys::general::RANDOM_INCLUDE_WE;
const FAVOURED: &str = keys::general::RANDOM_INCLUDE_FAVOURITES;
const PICK_ONLY: &str = keys::system::PICK_ONLY_MODE;
const ON_RESTORE: &str = keys::system::POST_PROCESS_ON_RESTORE;

fn sections_with(cfg: &dyn SettingsSource, weather: Option<&WeatherStatus>) -> Vec<Section> {
    let runtime = RuntimeStatus { weather, ..RuntimeStatus::default() };
    let source = PlaceSource { runtime, ..place_source(cfg) };
    crate::i18n::with_language("en-US", || source.place(place::AUTOMATION).sections)
}

fn section(cfg: &dyn SettingsSource, key: &str) -> Section {
    sections_with(cfg, None).into_iter().find(|section| section.id.key == key).expect("section")
}

fn location(source: &str, weather: Option<&WeatherStatus>) -> Section {
    let cfg = FakeSettingsSource::default()
        .with_flag(keys::weather::ENABLED, true)
        .with_text(keys::weather::SOURCE, source);
    sections_with(&cfg, weather).swap_remove(2)
}

fn path(row: &Row) -> Option<&str> {
    row.control.path()
}

fn bound(section: &Section) -> Vec<&str> {
    section.say.rows().filter_map(|at| section.row(at)).filter_map(path).collect()
}

fn group_paths(section: &Section) -> Vec<Vec<&str>> {
    section.groups.iter().map(|group| group.rows.iter().filter_map(path).collect()).collect()
}

fn kinds(section: &Section) -> Vec<GroupKind> {
    section.groups.iter().map(|group| group.kind).collect()
}

fn actions(rows: &[Row]) -> Vec<ActionId> {
    rows.iter()
        .flat_map(|row| match &row.control {
            Control::ActionBtn { id, .. } | Control::ToggleAction { id, .. } => vec![id.clone()],
            Control::ActionChips { items } => items.iter().map(|(id, _)| id.clone()).collect(),
            _ => Vec::new(),
        })
        .collect()
}

fn current(row: &Row) -> Option<&str> {
    match &row.control {
        Control::Dropdown { current, .. } => Some(current),
        _ => None,
    }
}

#[test]
fn board_sections_lead() {
    let keys: Vec<_> = sections_with(&FakeSettingsSource::default(), None)
        .iter()
        .map(|section| section.id.key)
        .collect();
    assert_eq!(keys, [ROTATION, SCHEDULE, LOCATION, STARTUP, COMMANDS]);
}

#[test]
fn rotation_sentence_states() {
    for (flags, expected) in [
        ([true, true, true, false, false], "Every [0], a random pick from images and videos."),
        (
            [true, true, true, true, true],
            "Every [0], a random pick from favourite images, videos and Wallpaper Engine items.",
        ),
        (
            [true, false, false, true, true],
            "Every [0], a random pick from favourite Wallpaper Engine items.",
        ),
        ([true, false, false, false, true], "Every [0], a random pick from an empty pool."),
        ([false, true, true, false, false], "Off. The wallpaper only changes when you pick one."),
    ] {
        let cfg = [ROTATE, STATIC, VIDEO, ENGINE, FAVOURED]
            .into_iter()
            .zip(flags)
            .fold(FakeSettingsSource::default(), |cfg, (key, on)| cfg.with_flag(key, on));
        let rotation = section(&cfg, ROTATION);
        assert_eq!(plain(&rotation.say.text), expected);
        assert_eq!(rotation.switched_off(), !flags[0]);
    }
}

#[test]
fn rotation_rows_bound() {
    let rotation = section(&FakeSettingsSource::default().with_flag(ROTATE, true), ROTATION);
    assert_eq!(rotation.master.as_ref().and_then(path), Some(ROTATE));
    assert_eq!(bound(&rotation), [keys::general::RANDOM_INTERVAL]);
    assert!(matches!(rotation.say.tokens[0], Slot::Value { form: Form::Duration, .. }));
    assert_eq!(
        group_paths(&rotation),
        [vec![STATIC, VIDEO, ENGINE, FAVOURED], vec![keys::general::RANDOM_INTERVAL]]
    );
    let hints: Vec<_> = rotation.body().map(Row::hint).collect();
    assert_eq!(hints, ["", "", "", "", "minimum 10 s"]);
    assert!(rotation.say.text.contains(VALUE_OPEN) && rotation.say.text.contains(VALUE_CLOSE));
    let off = section(&FakeSettingsSource::default().with_flag(ROTATE, false), ROTATION);
    assert!(off.say.tokens.is_empty());
}

#[test]
fn schedule_sentence_states() {
    for (enabled, start, expected) in [
        (
            true,
            true,
            "Rules are on; the first matching rule applies on startup. A wallpaper you pick yourself still wins.",
        ),
        (
            true,
            false,
            "Rules are on; the first matching rule waits for the next change. A wallpaper you pick yourself still wins.",
        ),
        (false, true, "Off. Your rules are kept until you turn this on."),
    ] {
        let cfg = FakeSettingsSource::default()
            .with_flag(keys::schedule::ENABLED, enabled)
            .with_flag(keys::schedule::APPLY_ON_START, start);
        let schedule = section(&cfg, SCHEDULE);
        assert_eq!(plain(&schedule.say.text), expected);
        assert_eq!(schedule.switched_off(), !enabled);
        assert_eq!(schedule.master.as_ref().and_then(path), Some(keys::schedule::ENABLED));
        assert_eq!(group_paths(&schedule), [vec![keys::schedule::APPLY_ON_START], vec![]]);
        assert_eq!(actions(&schedule.groups[1].rows), [ActionId::OpenScheduleEditor]);
    }
}

#[test]
fn location_mode_menu() {
    let outdoors = weather_status(WeatherSource::Outdoors);
    for (enabled, source, weather, mode) in [
        (false, keys::weather::SOURCE_PLAYLISTS, None, WEATHER_MODES[0]),
        (true, keys::weather::SOURCE_PLAYLISTS, None, keys::weather::SOURCE_PLAYLISTS),
        (true, keys::weather::SOURCE_SEMANTIC, None, keys::weather::SOURCE_SEMANTIC),
        (true, keys::weather::SOURCE_OUTDOORS, Some(&outdoors), keys::weather::SOURCE_OUTDOORS),
    ] {
        let cfg = FakeSettingsSource::default()
            .with_flag(keys::weather::ENABLED, enabled)
            .with_text(keys::weather::SOURCE, source);
        let location = sections_with(&cfg, weather).swap_remove(2);
        assert_eq!(
            plain(&location.say.text),
            "Weather for [0], sun times at [1], [2]. Weather changes: [3]."
        );
        assert!(location.master.is_none());
        assert_eq!(bound(&location), PLACE_SLOTS.map(|slot| slot.path));
        assert_eq!(location.head.iter().find_map(current), Some(mode), "{source}");
        assert_eq!(actions(&location.head), [ActionId::RefreshWeather]);
        let blanks = location
            .say
            .tokens
            .iter()
            .filter(|slot| matches!(slot, Slot::Value { blank: Some(_), .. }))
            .count();
        assert_eq!(blanks, 3);
        assert_eq!(location.head[0].hint(), tr(CITY_HINT));
        let ids = location.rows().filter_map(path).collect::<Vec<_>>();
        assert!(!ids.contains(&keys::weather::ENABLED) && !ids.contains(&keys::weather::SOURCE));
        assert!(
            !actions(&location.rows().cloned().collect::<Vec<_>>())
                .iter()
                .any(|id| matches!(id, ActionId::FollowOutdoors(_)))
        );
    }
}

#[test]
fn playlist_mode_body() {
    let location = location(keys::weather::SOURCE_PLAYLISTS, None);
    assert_eq!(kinds(&location), [GroupKind::Spread, GroupKind::Lines]);
    let grid = &location.groups[0];
    let titles: Vec<_> = grid.rows.iter().map(|row| row.title.as_str()).collect();
    assert_eq!(titles, ["Clear", "Cloudy", "Rainy", "Snowy", "Stormy", "Foggy", "Windy"]);
    assert!(grid.rows.iter().all(|row| row.hint().is_empty()));
    assert!(grid.note.as_ref().is_some_and(|note| !note.is_empty()));
    let line = &location.groups[1].rows;
    let paths: Vec<_> = line.iter().filter_map(path).collect();
    assert_eq!(paths, [keys::weather::OUTDOORS_TRANSITION_SHADER, keys::weather::OUTDOORS_RETHEME]);
    assert!(actions(line).is_empty());
}

#[test]
fn search_mode_body() {
    let status = weather_status(WeatherSource::Semantic);
    let location = location(keys::weather::SOURCE_SEMANTIC, Some(&status));
    let paths = group_paths(&location);
    assert_eq!(
        paths[0][..5],
        [
            keys::weather::SEMANTIC_SELECTION,
            keys::weather::SEMANTIC_MAX_RESULTS,
            keys::weather::SEMANTIC_DWELL_SECONDS,
            keys::weather::OUTDOORS_TRANSITION_SHADER,
            keys::weather::OUTDOORS_RETHEME,
        ]
    );
    assert_eq!(paths[1], WEATHER_QUERIES.map(|(query, _)| query));
    let footer = &location.groups[0].rows[5..];
    assert_eq!(footer[0].control.path(), Some(WEATHER_REVIEW_KEY));
    assert_eq!(current(&footer[0]), Some("cloudy"));
    assert!(!actions(footer).iter().any(|id| matches!(id, ActionId::ReviewWeather(_))));
    assert_eq!(location.say.notes.len(), 1);
    assert!(location.say.notes[0].contains("grey overcast sky"), "{}", location.say.notes[0]);
}

#[test]
fn outdoor_mode_body() {
    let status = weather_status(WeatherSource::Outdoors);
    let location = location(keys::weather::SOURCE_OUTDOORS, Some(&status));
    assert_eq!(
        group_paths(&location)[..2],
        [vec![keys::weather::OUTDOORS_TRANSITION_SHADER], vec![keys::weather::OUTDOORS_RETHEME]]
    );
    let displays = &location.groups[2].rows;
    let names: Vec<_> = displays.iter().map(|row| row.title.as_str()).collect();
    assert_eq!(names, ["DP-1", "HDMI-A-1"]);
    let Control::ActionChips { items } = &displays[0].control else { panic!("chips") };
    let labels: Vec<_> = items.iter().map(|(_, label)| label.as_str()).collect();
    assert_eq!(labels, ["Another", "Exclude", "Keep"]);
    assert!(location.rows().all(|row| !matches!(row.control, Control::StackBar { .. })));
    let footer = actions(&location.groups[3].rows);
    assert!(matches!(footer[..], [ActionId::OutdoorCredits(_)]), "{footer:?}");
    assert!(location.say.notes[0].contains("Outside now: Spring"), "{}", location.say.notes[0]);
}

#[test]
fn location_notes_states() {
    let cfg = FakeSettingsSource::default();
    let pending = RuntimeStatus { weather_pending: true, ..RuntimeStatus::default() };
    let source = PlaceSource { runtime: pending, ..place_source(&cfg) };
    let location = crate::i18n::with_language("en-US", || {
        source.place(place::AUTOMATION).sections.swap_remove(2)
    });
    assert_eq!(location.say.notes.len(), 1);
    assert!(location.say.notes[0].starts_with(tr(PENDING)));
    assert!(
        location.say.notes[0].contains("Add a city or coordinates"),
        "{}",
        location.say.notes[0]
    );
}

#[test]
fn conditions_follow_categories() {
    for (at, (category, _, label)) in WEATHER_CATEGORIES.iter().enumerate() {
        assert_eq!(WEATHER_CONDITIONS[at].0, *category);
        assert_eq!(WEATHER_QUERIES[at].1, *label);
    }
}

#[test]
fn startup_sentence_states() {
    for (restore, notify, expected) in [
        (
            true,
            true,
            "On startup Skwd re-applies the last wallpaper. Wallpaper changes send a notification.",
        ),
        (
            false,
            false,
            "On startup Skwd leaves the desktop as it is. Wallpaper changes stay silent.",
        ),
    ] {
        let cfg = FakeSettingsSource::default()
            .with_flag(keys::system::RESTORE_ON_STARTUP, restore)
            .with_flag(keys::general::NOTIFY_ON_WALLPAPER_CHANGE, notify);
        let startup = section(&cfg, STARTUP);
        assert_eq!(plain(&startup.say.text), expected);
        assert_eq!(
            group_paths(&startup),
            [
                vec![keys::system::RESTORE_ON_STARTUP],
                vec![keys::general::NOTIFY_ON_WALLPAPER_CHANGE]
            ]
        );
        assert!(startup.master.is_none() && startup.head.is_empty());
    }
}

#[test]
fn commands_sentence_states() {
    for (count, pick, rerun, expected) in [
        (
            0,
            false,
            false,
            "No commands run after each change. Skwd applies the wallpaper itself, and the commands skip the startup restore.",
        ),
        (
            1,
            true,
            false,
            "1 command runs after each change. Skwd leaves applying to your commands, and the commands skip the startup restore.",
        ),
        (
            2,
            false,
            true,
            "2 commands run after each change. Skwd applies the wallpaper itself, and the commands also run on the startup restore.",
        ),
    ] {
        let cfg = FakeSettingsSource::default()
            .with_array_len(keys::post_processing::LIST, count)
            .with_flag(PICK_ONLY, pick)
            .with_flag(ON_RESTORE, rerun);
        assert_eq!(plain(&section(&cfg, COMMANDS).say.text), expected);
    }
}

#[test]
fn command_lines_grouped() {
    let cfg = FakeSettingsSource::default().with_array_len(keys::post_processing::LIST, 2);
    let commands = section(&cfg, COMMANDS);
    assert_eq!(
        kinds(&commands),
        [GroupKind::Column, GroupKind::Column, GroupKind::Lines, GroupKind::Lines]
    );
    assert_eq!(
        group_paths(&commands)[..3],
        [
            vec![PICK_ONLY],
            vec![ON_RESTORE],
            vec![
                "postProcessing.0.type",
                "postProcessing.0.command",
                "postProcessing.1.type",
                "postProcessing.1.command",
            ]
        ]
    );
    let titles: Vec<_> = commands.groups[2].rows.iter().map(|row| row.title.as_str()).collect();
    assert!(titles.contains(&"Command 2"), "{titles:?}");
    assert_eq!(
        actions(&commands.groups[2].rows),
        [ActionId::RemovePostCommand(0), ActionId::RemovePostCommand(1)]
    );
    let add = &commands.groups[3].rows;
    assert_eq!(actions(add), [ActionId::AddPostCommand]);
    assert_eq!(add[0].hint(), tr("settings-automation-placeholders"));
}

#[test]
fn head_rows_drawn() {
    for source in WEATHER_MODES {
        let status = weather_status(WeatherSource::Outdoors);
        let cfg = FakeSettingsSource::default()
            .with_flag(keys::weather::ENABLED, source != WEATHER_MODES[0])
            .with_text(keys::weather::SOURCE, source);
        let composed = crate::i18n::with_language("en-US", || {
            compose(&PlaceSource {
                runtime: RuntimeStatus { weather: Some(&status), ..RuntimeStatus::default() },
                ..place_source(&cfg)
            })
        });
        assert!(composed.iter().all(crate::frontend::settings::places::drawn), "{source}");
        assert_eq!(composed.len(), sections_with(&cfg, None).len());
    }
}

fn slots(count: u32) -> Vec<String> {
    (0..count).map(|slot| sentinel(slot).expect("slot").to_string()).collect()
}

fn spanned(text: &str) -> bool {
    text.matches(VALUE_OPEN).count() == text.matches(VALUE_CLOSE).count()
        && text.matches(VALUE_OPEN).count() > 0
}

fn render_marked(tag: &str, key: &str, pairs: &[(&str, &str)]) -> String {
    let mut args = marked();
    for (name, value) in pairs {
        args.set(*name, *value);
    }
    crate::i18n::Catalog::for_locale(tag).format(key, Some(&args))
}

#[test]
fn sentences_keep_slots() {
    let marks = slots(4);
    for language in crate::i18n::LANGUAGES {
        let tag = language.tag;
        let rotation = render_marked(tag, ROTATION_SAY, &[("interval", &marks[0]), ("pool", "x")]);
        assert_eq!(rotation.matches(&marks[0]).count(), 1, "{tag}");
        assert!(spanned(&rotation), "{tag} {rotation}");
        let pairs: Vec<_> =
            PLACE_SLOTS.iter().zip(&marks).map(|(slot, mark)| (slot.arg, mark.as_str())).collect();
        let place = render(tag, LOCATION_SAY, &pairs);
        assert!(marks.iter().all(|mark| place.matches(mark.as_str()).count() == 1), "{tag}");
        for value in START.values {
            assert!(spanned(&render_marked(tag, SCHEDULE_SAY, &[(START.arg, value)])), "{tag}");
        }
        for restore in RESTORE.values {
            for notify in NOTIFY.values {
                let pairs = [(RESTORE.arg, restore), (NOTIFY.arg, notify)];
                assert!(spanned(&render_marked(tag, STARTUP_SAY, &pairs)), "{tag}");
            }
        }
        for listed in LISTED.values {
            for apply in APPLY.values {
                for rerun in RERUN.values {
                    let pairs = [
                        (LISTED.arg, listed),
                        (COMMANDS_ARG, "x"),
                        (APPLY.arg, apply),
                        (RERUN.arg, rerun),
                    ];
                    assert!(spanned(&render_marked(tag, COMMANDS_SAY, &pairs)), "{tag}");
                }
            }
        }
        let catalog = crate::i18n::Catalog::for_locale(tag);
        for count in [1, 2, 5, 21] {
            let mut args = marked();
            args.set(COUNT_ARG, count);
            assert!(spanned(&catalog.format(COMMAND_COUNT, Some(&args))), "{tag} {count}");
        }
    }
}

#[test]
fn selectors_change_text() {
    for language in crate::i18n::LANGUAGES {
        let tag = language.tag;
        let start = |on| render_marked(tag, SCHEDULE_SAY, &[START.pick(on)]);
        assert_ne!(start(true), start(false), "{tag}");
        for (at, choice) in [RESTORE, NOTIFY].iter().enumerate() {
            let pairs = |on| {
                let mut pairs = [RESTORE.pick(true), NOTIFY.pick(true)];
                pairs[at] = choice.pick(on);
                pairs
            };
            let say = |on| render_marked(tag, STARTUP_SAY, &pairs(on));
            assert_ne!(say(true), say(false), "{tag} {}", choice.arg);
        }
        for (at, choice) in [LISTED, APPLY, RERUN].iter().enumerate() {
            let pairs = |on| {
                let mut pairs = vec![
                    LISTED.pick(true),
                    APPLY.pick(true),
                    RERUN.pick(true),
                    (COMMANDS_ARG, "x"),
                ];
                pairs[at] = choice.pick(on);
                pairs
            };
            let say = |on| render_marked(tag, COMMANDS_SAY, &pairs(on));
            assert_ne!(say(true), say(false), "{tag} {}", choice.arg);
        }
        let kinds = ["a", "b", "c"];
        let pools: std::collections::BTreeSet<_> = (0..=kinds.len())
            .map(|count| crate::i18n::with_language(tag, || listed(POOL, &kinds[..count])))
            .collect();
        assert_eq!(pools.len(), kinds.len() + 1, "{tag}");
        let favoured = |on| render(tag, POOL_FAVOURITES, &[FAVOURITES.pick(on), ("pool", "a")]);
        assert_ne!(favoured(true), favoured(false), "{tag}");
    }
}

#[test]
fn slot_rows_match() {
    let location = location(keys::weather::SOURCE_PLAYLISTS, None);
    let mode = location.say.tokens.last().expect("mode");
    assert_eq!(mode, &Slot::Value { row: 3, blank: None, form: Form::Plain });
}
