#[cfg(test)]
#[path = "automation_tests.rs"]
mod tests;

use crate::contracts::settings::{SettingsSource, keys};
use crate::i18n::{text, tr};

use super::category::AUTOMATION;
use super::places::{
    Bind, Choice, NO_SLOTS, PlaceSource, SUMMARY_JOINER, bind, column, legacy, listed, marked,
    note, noted, rows, spoken, switched, take, unfold,
};
use super::tables::{WEATHER_CONDITIONS, WEATHER_MODE_KEY, WEATHER_MODES, WEATHER_REVIEW_KEY};
use super::tabs::{WEATHER_CATEGORIES, WEATHER_QUERIES, take_card, take_control_paths};
use super::{ActionId, Control, Form, Group, GroupKind, Look, Note, Row, Section, SectionId};

pub(super) const ROTATION: &str = "settings-section-random-rotation";
const SCHEDULE: &str = "settings-section-schedule";
const LOCATION: &str = "settings-automation-location";
const STARTUP: &str = "settings-automation-startup";
const COMMANDS: &str = "settings-automation-commands";
const LOCATION_CARD: &str = "settings-section-location-weather";
const STARTUP_CARD: &str = "settings-section-startup-notifications";
const BEHAVIOUR_CARD: &str = "settings-section-post-apply-behaviour";
const COMMANDS_CARD: &str = "settings-section-post-apply-commands";
const ADVANCED: &str = "weather.advanced";

const ROTATION_SAY: &str = "settings-automation-rotation-say";
const ROTATION_OFF: &str = "settings-automation-rotation-off";
const POOL: &str = "settings-automation-pool";
const POOL_FAVOURITES: &str = "settings-automation-pool-favourites";
const SCHEDULE_SAY: &str = "settings-automation-schedule-say";
const SCHEDULE_OFF: &str = "settings-automation-schedule-off";
const LOCATION_SAY: &str = "settings-automation-location-say";
const STARTUP_SAY: &str = "settings-automation-startup-say";
const COMMAND_COUNT: &str = "settings-automation-command-count";
const COMMANDS_SAY: &str = "settings-automation-commands-say";
const PENDING: &str = "settings-outdoors-pending-desc";
const HELD: &str = "settings-outdoors-held-desc";
const INTERVAL_HINT: &str = "settings-automation-interval-hint";
const POOL_HEADING: &str = "settings-automation-pool-heading";
const TIMING: &str = "settings-automation-timing";
const START_HINT: &str = "settings-automation-start-hint";
const EDITOR_HINT: &str = "settings-automation-editor-hint";
const PICK_ONLY_HINT: &str = "settings-automation-pick-only-hint";
const COMMANDS_HEADING: &str = "settings-automation-commands-heading";
const CITY_HINT: &str = "settings-automation-location-hint";
const PLACEHOLDERS: &str = "settings-automation-placeholders";
const MODE_TITLE: &str = "settings-weather-card";
const REVIEW_TITLE: &str = "settings-weather-review-label";
const PLAYLISTS_HEADING: &str = "settings-automation-playlists-heading";
const MODE_LABELS: [&str; WEATHER_MODES.len()] = [
    "settings-automation-mode-off",
    "settings-automation-mode-outdoors",
    "settings-automation-mode-playlists",
    "settings-automation-mode-semantic",
];
const DISPLAY_LABELS: [&str; 3] = [
    "settings-automation-display-another",
    "settings-automation-display-exclude",
    "settings-automation-display-keep",
];

pub(super) const INTERVAL_SLOT: Bind<'_> =
    Bind { form: Form::Duration, ..Bind::new("interval", keys::general::RANDOM_INTERVAL) };
const PLACE_SLOTS: [Bind<'_>; 4] = [
    Bind { blank: Some("settings-automation-no-city"), ..Bind::new("city", keys::general::LOCALE) },
    Bind {
        blank: Some("settings-automation-no-latitude"),
        ..Bind::new("lat", keys::schedule::LATITUDE)
    },
    Bind {
        blank: Some("settings-automation-no-longitude"),
        ..Bind::new("lon", keys::schedule::LONGITUDE)
    },
    Bind::new("mode", WEATHER_MODE_KEY),
];
const TUNING: [&str; 3] = [
    keys::weather::SEMANTIC_SELECTION,
    keys::weather::SEMANTIC_MAX_RESULTS,
    keys::weather::SEMANTIC_DWELL_SECONDS,
];
const POOL_KINDS: [(&str, &str); 3] = [
    (keys::general::RANDOM_INCLUDE_STATIC, "settings-automation-pool-images"),
    (keys::general::RANDOM_INCLUDE_VIDEO, "settings-automation-pool-videos"),
    (keys::general::RANDOM_INCLUDE_WE, "settings-automation-pool-engine"),
];
const FAVOURITES: Choice = Choice { arg: "favourites", values: ["all", "only"] };
const START: Choice = Choice { arg: "start", values: ["waits", "applies"] };
const RESTORE: Choice = Choice { arg: "restore", values: ["leaves", "restores"] };
const NOTIFY: Choice = Choice { arg: "notify", values: ["silent", "notifies"] };
const APPLY: Choice = Choice { arg: "apply", values: ["itself", "commands"] };
const RERUN: Choice = Choice { arg: "rerun", values: ["skip", "also"] };
const LISTED: Choice = Choice { arg: "listed", values: ["none", "some"] };
const SOURCES: [&str; 2] = [keys::weather::SOURCE_PLAYLISTS, keys::weather::SOURCE_SEMANTIC];
const COMMANDS_ARG: &str = "commands";
const COUNT_ARG: &str = "count";
const POOL_ARG: &str = "pool";

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let mut cards = source.cards(AUTOMATION);
    let mut sections = vec![
        rotation(cfg, take_card(&mut cards, text(ROTATION))),
        schedule(cfg, take_card(&mut cards, text(SCHEDULE))),
        location(source, take_card(&mut cards, text(LOCATION_CARD))),
        startup(cfg, take_card(&mut cards, text(STARTUP_CARD))),
        commands(
            cfg,
            take_card(&mut cards, text(BEHAVIOUR_CARD)),
            take_card(&mut cards, text(COMMANDS_CARD)),
        ),
    ];
    sections.extend(
        cards
            .into_iter()
            .map(|(card, rows)| legacy(SectionId::new(AUTOMATION, card.key), &card, rows)),
    );
    sections
}

fn rotation(cfg: &dyn SettingsSource, mut card: Vec<Row>) -> Section {
    let master = take_control_paths(&mut card, &[keys::general::RANDOM_ROTATE]).pop();
    let interval = rows(&mut card, &[(keys::general::RANDOM_INTERVAL, Some(INTERVAL_HINT))]);
    let pool = pool(cfg);
    let section = Section {
        master,
        groups: vec![column(Some(POOL_HEADING), noted(card, None)), column(Some(TIMING), interval)],
        ..Section::new(
            SectionId::new(AUTOMATION, ROTATION),
            tr(ROTATION),
            tr("settings-automation-rotation-desc"),
        )
    };
    spoken(section, |section| {
        switched(section, ROTATION_OFF, || {
            bind(section, ROTATION_SAY, &[INTERVAL_SLOT], &[(POOL_ARG, &pool)])
        })
    })
}

fn pool(cfg: &dyn SettingsSource) -> String {
    let kinds: Vec<_> =
        POOL_KINDS.iter().filter(|(path, _)| cfg.flag(path)).map(|(_, kind)| tr(kind)).collect();
    let pool = listed(POOL, &kinds);
    if kinds.is_empty() {
        return pool;
    }
    let (arg, favourites) = FAVOURITES.pick(cfg.flag(keys::general::RANDOM_INCLUDE_FAVOURITES));
    let mut args = fluent::FluentArgs::new();
    args.set(POOL_ARG, pool);
    args.set(arg, favourites);
    crate::i18n::format(POOL_FAVOURITES, &args)
}

fn schedule(cfg: &dyn SettingsSource, mut card: Vec<Row>) -> Section {
    let master = take_control_paths(&mut card, &[keys::schedule::ENABLED]).pop();
    let on_start = rows(&mut card, &[(keys::schedule::APPLY_ON_START, Some(START_HINT))]);
    let section = Section {
        master,
        groups: vec![column(None, on_start), column(None, noted(card, Some(EDITOR_HINT)))],
        ..Section::new(
            SectionId::new(AUTOMATION, SCHEDULE),
            tr(SCHEDULE),
            tr("settings-automation-schedule-desc"),
        )
    };
    let start = START.pick(cfg.flag(keys::schedule::APPLY_ON_START));
    spoken(section, |section| {
        switched(section, SCHEDULE_OFF, || bind(section, SCHEDULE_SAY, &NO_SLOTS, &[start]))
    })
}

fn mode(cfg: &dyn SettingsSource, following: bool) -> &'static str {
    if following {
        return keys::weather::SOURCE_OUTDOORS;
    }
    if !cfg.flag(keys::weather::ENABLED) {
        return WEATHER_MODES[0];
    }
    let source = cfg.text(keys::weather::SOURCE);
    SOURCES.into_iter().find(|state| *state == source).unwrap_or(SOURCES[0])
}

fn menu(title: &'static str, path: &str, options: Vec<(String, String)>, current: &str) -> Row {
    Row::of(
        tr(title).to_string(),
        String::new(),
        Control::Dropdown {
            path: path.to_string(),
            options,
            current: current.to_string(),
            palettes: Vec::new(),
        },
    )
}

fn condition_label(category: &str) -> Option<&'static str> {
    WEATHER_CONDITIONS.iter().find(|(key, _)| *key == category).map(|(_, label)| tr(label))
}

fn conditions(rows: Vec<Row>) -> Vec<Row> {
    rows.into_iter()
        .map(|row| {
            let path = row.control.path().unwrap_or_default();
            let at = WEATHER_CATEGORIES
                .iter()
                .position(|(_, map, _)| *map == path)
                .or_else(|| WEATHER_QUERIES.iter().position(|(query, _)| *query == path));
            let title = at.map_or(row.title, |at| tr(WEATHER_CONDITIONS[at].1).to_string());
            let look = Look { note: Note::Quiet, ..row.look };
            Row { title, look, ..row }
        })
        .collect()
}

fn review(source: &PlaceSource<'_>) -> Row {
    let options = WEATHER_CONDITIONS
        .iter()
        .map(|(category, label)| ((*category).to_string(), tr(label).to_string()))
        .collect();
    let now = source.runtime.weather.and_then(|status| status.weather.first());
    let current = now
        .filter(|category| condition_label(category).is_some())
        .map_or(WEATHER_CONDITIONS[0].0, String::as_str);
    menu(REVIEW_TITLE, WEATHER_REVIEW_KEY, options, current)
}

fn display(row: Row, notes: &mut Vec<String>) -> Row {
    let Control::StackBar { summary, rows, .. } = row.control else {
        return row;
    };
    let inner = rows.into_iter().next();
    let control = match inner.map(|inner| inner.control) {
        Some(Control::ActionChips { items }) => Control::ActionChips {
            items: items
                .into_iter()
                .map(|(id, label)| {
                    let short = match id {
                        ActionId::OutdoorNext(_) => Some(DISPLAY_LABELS[0]),
                        ActionId::OutdoorExclude { .. } => Some(DISPLAY_LABELS[1]),
                        ActionId::OutdoorPin(_) => Some(DISPLAY_LABELS[2]),
                        _ => None,
                    };
                    (id, short.map_or(label, |key| tr(key).to_string()))
                })
                .collect(),
        },
        _ => Control::Static,
    };
    let held = matches!(&control, Control::ActionChips { items }
        if items.iter().any(|(id, _)| matches!(id, ActionId::OutdoorResume(_))));
    if held && !notes.contains(&tr(HELD).to_string()) {
        notes.push(tr(HELD).to_string());
    }
    Row { desc: [summary, row.desc].join(SUMMARY_JOINER), control, ..row }
}

fn location(source: &PlaceSource<'_>, rows: Vec<Row>) -> Section {
    let cfg = source.cfg;
    let mut rows = unfold(rows, |id| id == ADVANCED);
    let mut head = take_control_paths(&mut rows, &PLACE_SLOTS.map(|slot| slot.path));
    if let Some(city) = head.first_mut() {
        city.look.note = Note::Own(tr(CITY_HINT).to_string());
    }
    let follow = take(&mut rows, |control| {
        matches!(control, Control::ActionBtn { id: ActionId::FollowOutdoors(_), .. })
    });
    let following = follow.iter().any(|row| {
        matches!(row.control, Control::ActionBtn { id: ActionId::FollowOutdoors(false), .. })
    });
    take_control_paths(&mut rows, &[keys::weather::ENABLED, keys::weather::SOURCE]);
    let options = WEATHER_MODES
        .iter()
        .zip(MODE_LABELS)
        .map(|(mode, label)| ((*mode).to_string(), tr(label).to_string()))
        .collect();
    head.push(menu(MODE_TITLE, WEATHER_MODE_KEY, options, mode(cfg, following)));
    let mut notes = Vec::new();
    for row in take(&mut rows, |control| matches!(control, Control::Static)) {
        notes.push(note(&row));
    }
    let reviews = take(&mut rows, |control| {
        matches!(control, Control::ActionBtn { id: ActionId::ReviewWeather(_), .. })
    });
    let transition = take_control_paths(&mut rows, &[keys::weather::OUTDOORS_TRANSITION_SHADER]);
    let retheme = take_control_paths(&mut rows, &[keys::weather::OUTDOORS_RETHEME]);
    let maps = take_control_paths(&mut rows, &WEATHER_CATEGORIES.map(|(_, path, _)| path));
    let queries = take_control_paths(&mut rows, &WEATHER_QUERIES.map(|(path, _)| path));
    let tuning = take_control_paths(&mut rows, &TUNING);
    let displays: Vec<_> = take(&mut rows, |control| matches!(control, Control::StackBar { .. }))
        .into_iter()
        .map(|row| display(row, &mut notes))
        .collect();
    let hint = maps.first().map(|row| row.desc.clone());
    head.extend(take(&mut rows, |control| {
        matches!(control, Control::ActionBtn { id: ActionId::RefreshWeather, .. })
    }));
    let mut footer = Vec::new();
    if !reviews.is_empty() {
        footer.push(review(source));
    }
    footer.extend(rows);
    let mut groups = Vec::new();
    if !queries.is_empty() {
        let left = tuning.into_iter().chain(transition).chain(retheme).chain(footer).collect();
        groups.push(column(None, noted(left, None)));
        groups.push(column(None, conditions(queries)));
        groups.push(Group::new(GroupKind::Lines, None, displays));
    } else if following || !displays.is_empty() {
        groups.push(column(None, transition));
        groups.push(column(None, retheme));
        groups.push(Group::new(GroupKind::Lines, None, displays));
        groups.push(Group::new(GroupKind::Lines, None, footer));
    } else {
        let heading = Some(tr(PLAYLISTS_HEADING).to_string());
        let grid = Group::new(GroupKind::Spread, heading, conditions(maps));
        groups.push(Group { note: hint, ..grid });
        let line = transition.into_iter().chain(retheme).chain(footer).collect();
        groups.push(Group::new(GroupKind::Lines, None, noted(line, None)));
    }
    if source.runtime.weather_pending {
        notes.insert(0, tr(PENDING).to_string());
    }
    let id = SectionId::new(AUTOMATION, LOCATION);
    let desc = tr("settings-automation-location-desc");
    let mut composed =
        spoken(Section { head, groups, ..Section::new(id, tr(LOCATION), desc) }, |section| {
            bind(section, LOCATION_SAY, &PLACE_SLOTS, &[])
        });
    if !notes.is_empty() {
        composed.say.notes.push(notes.join(SUMMARY_JOINER));
    }
    composed
}

fn startup(cfg: &dyn SettingsSource, mut rows: Vec<Row>) -> Section {
    let restore = take_control_paths(&mut rows, &[keys::system::RESTORE_ON_STARTUP]);
    let section = Section {
        groups: vec![column(None, noted(restore, None)), column(None, noted(rows, None))],
        ..Section::new(
            SectionId::new(AUTOMATION, STARTUP),
            tr(STARTUP),
            tr("settings-automation-startup-desc"),
        )
    };
    let args = [
        RESTORE.pick(cfg.flag(keys::system::RESTORE_ON_STARTUP)),
        NOTIFY.pick(cfg.flag(keys::general::NOTIFY_ON_WALLPAPER_CHANGE)),
    ];
    spoken(section, |section| bind(section, STARTUP_SAY, &NO_SLOTS, &args))
}

fn commands(cfg: &dyn SettingsSource, mut behaviour: Vec<Row>, mut entries: Vec<Row>) -> Section {
    let pick = rows(&mut behaviour, &[(keys::system::PICK_ONLY_MODE, Some(PICK_ONLY_HINT))]);
    let add = take(&mut entries, |control| {
        matches!(control, Control::ActionBtn { id: ActionId::AddPostCommand, .. })
    });
    let section = Section {
        groups: vec![
            column(None, pick),
            column(None, noted(behaviour, None)),
            Group::new(GroupKind::Lines, Some(tr(COMMANDS_HEADING).into()), entries),
            Group::new(GroupKind::Lines, None, noted(add, Some(PLACEHOLDERS))),
        ],
        ..Section::new(
            SectionId::new(AUTOMATION, COMMANDS),
            tr(COMMANDS),
            tr("settings-automation-commands-desc"),
        )
    };
    let count = cfg.array_len(keys::post_processing::LIST);
    let mut counted = marked();
    counted.set(COUNT_ARG, count);
    let listed = crate::i18n::format(COMMAND_COUNT, &counted);
    let args = [
        LISTED.pick(count > 0),
        (COMMANDS_ARG, listed.as_str()),
        APPLY.pick(cfg.flag(keys::system::PICK_ONLY_MODE)),
        RERUN.pick(cfg.flag(keys::system::POST_PROCESS_ON_RESTORE)),
    ];
    spoken(section, |section| bind(section, COMMANDS_SAY, &NO_SLOTS, &args))
}
