#[cfg(test)]
#[path = "sources_tests.rs"]
mod tests;

use crate::contracts::settings::{SettingsSource, keys};
use crate::frontend::browser::{TOPLIST_SORT, TREND_SORT};
use crate::i18n::{text, tr, tr_args};

use super::category::SOURCES;
use super::places::{
    Bind, Checks, Choice, LIST_KEY, PlaceSource, Spec, bind, column, heads, listed, looked, park,
    rows, spoken, switched, unfold,
};
use super::tables::STEAM_METHODS;
use super::tabs::{take_card, take_control_paths};
use super::{Cell, Control, Group, Row, Say, Section, SectionId};

use keys::selector as grid;
use keys::sources as providers;
use keys::{steam, wallhaven};

const PROVIDERS: &str = "settings-section-source-providers";
pub(super) const WALLHAVEN: &str = "settings-sources-wallhaven-card";
pub(super) const WORKSHOP: &str = "settings-sources-workshop-card";
pub(super) const OTHERS: &str = "settings-sources-other";

const WALLHAVEN_SAY: &str = "settings-sources-wallhaven-say";
const WALLHAVEN_OFF: &str = "settings-sources-wallhaven-off";
const WORKSHOP_SAY: &str = "settings-sources-workshop-say";
const WORKSHOP_OFF: &str = "settings-sources-workshop-off";
const SEARCHING: &str = "settings-sources-searching";
const SEARCHING_NONE: &str = "settings-sources-searching-none";
const NEEDS_KEY: &str = "settings-sources-needs-key";
const SWITCHED_OFF: &str = "settings-sources-off";
const ACCOUNT: &str = "settings-sources-account";
const BROWSER: &str = "settings-sources-browser";
const FOLDERS: &str = "settings-sources-browser-folders";
const PHOTOS: &str = "settings-sources-photos";
const DAILY: &str = "settings-sources-daily";
const NOT_SET: &str = "settings-sources-not-set";
const DETECTED: &str = "settings-sources-detected";
const METHOD_HINTS: [&str; 2] =
    ["settings-sources-method-steam-hint", "settings-sources-method-steamcmd-hint"];
const METHODS: [(&str, &str); 2] = [
    (STEAM_METHODS[0], "settings-sources-method-steam"),
    (STEAM_METHODS[1], "settings-sources-method-steamcmd"),
];

const WALLHAVEN_SLOTS: [Spec<'_>; 5] = [
    Spec::Value(Bind::new("sort", wallhaven::DEFAULT_SORT)),
    Spec::Value(Bind::new("range", wallhaven::DEFAULT_TOP_RANGE)),
    Spec::Value(Bind::new("ratio", wallhaven::DEFAULT_RATIOS)),
    Spec::Checks(Checks {
        arg: "categories",
        paths: &[wallhaven::DEFAULT_GENERAL, wallhaven::DEFAULT_ANIME, wallhaven::DEFAULT_PEOPLE],
        heading: "settings-source-default-category",
        blank: "settings-sources-no-category",
    }),
    Spec::Checks(Checks {
        arg: "purity",
        paths: &[wallhaven::DEFAULT_SFW, wallhaven::DEFAULT_SKETCHY, wallhaven::DEFAULT_NSFW],
        heading: "settings-sources-content",
        blank: "settings-sources-no-content",
    }),
];
const WORKSHOP_SLOTS: [Bind<'_>; 5] = [
    Bind::new("sort", steam::DEFAULT_SORT),
    Bind::new("days", steam::DEFAULT_TREND_DAYS),
    Bind::new("type", steam::DEFAULT_TYPE),
    Bind::new("resolution", steam::DEFAULT_RESOLUTION),
    Bind::new("category", steam::DEFAULT_CATEGORY),
];
const TOP: Choice = Choice { arg: "top", values: ["no", "yes"] };
const TREND: Choice = Choice { arg: "trend", values: ["no", "yes"] };
const MANY: Choice = Choice { arg: "count", values: ["one", "some"] };

const WALLHAVEN_ACCOUNT: [(&str, Option<&str>); 3] = [
    (wallhaven::API_KEY, Some("settings-sources-nsfw-hint")),
    (wallhaven::USERNAME, None),
    (providers::WALLHAVEN_SHOW_APPLY_BUTTON, None),
];
const WALLHAVEN_BROWSER: [(&str, Option<&str>); 2] =
    [(wallhaven::DEFAULT_ATLEAST, None), (wallhaven::DEFAULT_ATMOST, None)];
const WORKSHOP_ACCOUNT: [(&str, Option<&str>); 3] = [
    (steam::USERNAME, Some("settings-sources-steamcmd-hint")),
    (steam::API_KEY, None),
    (steam::DEFAULT_NSFW, Some("settings-sources-workshop-only")),
];
const WORKSHOP_APPLY: [(&str, Option<&str>); 1] = [(providers::STEAM_SHOW_APPLY_BUTTON, None)];
const WORKSHOP_PATHS: [(&str, Option<&str>); 3] = [
    (keys::paths::STEAM_WORKSHOP, None),
    (keys::paths::STEAM_WE_ASSETS, None),
    (keys::paths::STEAM, None),
];
const PHOTO_ROWS: [(&str, Option<&str>); 6] = [
    (providers::UNSPLASH_ENABLED, None),
    (providers::UNSPLASH_ACCESS_KEY, Some("settings-sources-unsplash-hint")),
    (providers::UNSPLASH_SHOW_APPLY_BUTTON, None),
    (providers::PEXELS_ENABLED, None),
    (providers::PEXELS_API_KEY, Some("settings-sources-pexels-hint")),
    (providers::PEXELS_SHOW_APPLY_BUTTON, None),
];
const DAILY_ROWS: [(&str, Option<&str>); 6] = [
    (providers::BING_ENABLED, Some("settings-sources-bing-hint")),
    (providers::BING_MARKET, None),
    (providers::YOUTUBE_ENABLED, Some("settings-sources-youtube-hint")),
    (providers::YOUTUBE_MAX_HEIGHT, None),
    (providers::YOUTUBE_MAX_MINUTES, Some("settings-sources-clip-hint")),
    (providers::YOUTUBE_SHOW_APPLY_BUTTON, None),
];

struct Pair {
    first: &'static str,
    second: &'static str,
    title: &'static str,
    hint: Option<&'static str>,
}

const WALLHAVEN_PAIRS: [Pair; 2] = [
    Pair {
        first: grid::WALLHAVEN_COLUMNS,
        second: grid::WALLHAVEN_ROWS,
        title: "settings-wallhaven-grid-card",
        hint: Some("settings-sources-grid-hint"),
    },
    Pair {
        first: grid::WALLHAVEN_THUMB_WIDTH,
        second: grid::WALLHAVEN_THUMB_HEIGHT,
        title: "settings-wallhaven-thumb-card",
        hint: Some("settings-sources-thumb-hint"),
    },
];
const WORKSHOP_PAIRS: [Pair; 2] = [
    Pair {
        first: grid::STEAM_COLUMNS,
        second: grid::STEAM_ROWS,
        title: "settings-steam-grid-card",
        hint: None,
    },
    Pair {
        first: grid::STEAM_THUMB_WIDTH,
        second: grid::STEAM_THUMB_HEIGHT,
        title: "settings-steam-thumb-card",
        hint: None,
    },
];

struct Provider {
    name: &'static str,
    enabled: &'static str,
    key: Option<(&'static str, &'static str)>,
}

const OTHER_PROVIDERS: [Provider; 4] = [
    Provider {
        name: "settings-sources-unsplash-card",
        enabled: providers::UNSPLASH_ENABLED,
        key: Some((providers::UNSPLASH_ACCESS_KEY, "access")),
    },
    Provider {
        name: "settings-sources-pexels-card",
        enabled: providers::PEXELS_ENABLED,
        key: Some((providers::PEXELS_API_KEY, "api")),
    },
    Provider { name: "settings-sources-bing-card", enabled: providers::BING_ENABLED, key: None },
    Provider {
        name: "settings-sources-youtube-card",
        enabled: providers::YOUTUBE_ENABLED,
        key: None,
    },
];

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let mut cards = source.cards(SOURCES);
    let mut pool = unfold(take_card(&mut cards, text(PROVIDERS)), |_| true);
    vec![wallhaven(cfg, &mut pool), workshop(cfg, &mut pool), others(cfg, &mut pool)]
}

fn pairs(group: &mut Group, pool: &mut Vec<Row>, table: &[Pair]) {
    for pair in table {
        let mut first = rows(pool, &[(pair.first, pair.hint)]);
        let mut second = rows(pool, &[(pair.second, None)]);
        for row in &mut first {
            row.title = tr(pair.title).to_string();
        }
        for row in &mut second {
            row.look.cell = Cell::Joined;
        }
        group.rows.extend(first.into_iter().chain(second));
    }
}

fn blank(heading: &'static str, rows: Vec<Row>, empty: &'static str) -> Group {
    Group { blank: Some(tr(empty).to_string()), ..column(Some(heading), rows) }
}

fn parked(mut section: Section, master: &str, optional: (&str, &str), shown: bool) -> Section {
    let (cause, row) = optional;
    if section.switched_off() {
        park(&mut section, master, |_| true);
    } else if !shown {
        park(&mut section, cause, |candidate| candidate.control.path() == Some(row));
    }
    section
}

fn wallhaven(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let master = take_control_paths(pool, &[keys::features::WALLHAVEN]).pop();
    let head = heads(pool, &WALLHAVEN_SLOTS);
    let account = blank(ACCOUNT, rows(pool, &WALLHAVEN_ACCOUNT), NOT_SET);
    let mut browser = column(Some(BROWSER), rows(pool, &WALLHAVEN_BROWSER));
    pairs(&mut browser, pool, &WALLHAVEN_PAIRS);
    let section = Section {
        master,
        head,
        groups: vec![account, browser],
        ..Section::new(
            SectionId::new(SOURCES, WALLHAVEN),
            tr(WALLHAVEN),
            tr("settings-general-feature-wallhaven-desc"),
        )
    };
    let top = cfg.text(wallhaven::DEFAULT_SORT) == TOPLIST_SORT;
    let range = wallhaven::DEFAULT_TOP_RANGE;
    let slots: Vec<_> =
        WALLHAVEN_SLOTS.into_iter().filter(|slot| top || slot.paths() != [range]).collect();
    let section = spoken(section, |section| {
        switched(section, WALLHAVEN_OFF, || bind(section, WALLHAVEN_SAY, &slots, &[TOP.pick(top)]))
    });
    parked(section, keys::features::WALLHAVEN, (wallhaven::DEFAULT_SORT, range), top)
}

fn method(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Vec<Row> {
    let stored = cfg.text(steam::BACKEND);
    let steamcmd = !stored.is_empty() && stored != STEAM_METHODS[0];
    let mut method = rows(pool, &[(steam::BACKEND, Some(METHOD_HINTS[usize::from(steamcmd)]))]);
    for row in &mut method {
        row.look.cell = Cell::Buttons(&METHODS);
        if let Control::Dropdown { current, .. } = &mut row.control {
            *current = STEAM_METHODS[usize::from(steamcmd)].to_string();
        }
    }
    method
}

fn workshop(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let master = take_control_paths(pool, &[keys::features::STEAM]).pop();
    let head = heads(pool, &WORKSHOP_SLOTS);
    let mut account = method(cfg, pool);
    account.extend(rows(pool, &WORKSHOP_ACCOUNT));
    let account = blank(ACCOUNT, account, NOT_SET);
    let mut folders = blank(FOLDERS, Vec::new(), DETECTED);
    pairs(&mut folders, pool, &WORKSHOP_PAIRS);
    folders.rows.extend(rows(pool, &WORKSHOP_APPLY));
    folders.rows.extend(looked(rows(pool, &WORKSHOP_PATHS), Cell::Field));
    let section = Section {
        master,
        head,
        groups: vec![account, folders],
        ..Section::new(
            SectionId::new(SOURCES, WORKSHOP),
            tr(WORKSHOP),
            tr("settings-general-feature-steam-desc"),
        )
    };
    let trend = cfg.text(steam::DEFAULT_SORT) == TREND_SORT;
    let days = steam::DEFAULT_TREND_DAYS;
    let slots: Vec<_> =
        WORKSHOP_SLOTS.into_iter().filter(|slot| trend || slot.path != days).collect();
    let section = spoken(section, |section| {
        switched(section, WORKSHOP_OFF, || {
            bind(section, WORKSHOP_SAY, &slots, &[TREND.pick(trend)])
        })
    });
    parked(section, keys::features::STEAM, (steam::DEFAULT_SORT, days), trend)
}

fn others(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let photos = blank(PHOTOS, rows(pool, &PHOTO_ROWS), NOT_SET);
    let daily = column(Some(DAILY), rows(pool, &DAILY_ROWS));
    let section = Section {
        groups: vec![photos, daily],
        ..Section::new(
            SectionId::new(SOURCES, OTHERS),
            tr(OTHERS),
            tr("settings-sources-other-desc"),
        )
    };
    let text = searching(cfg);
    spoken(section, |_| Some(Say { text, ..Say::default() }))
}

fn searching(cfg: &dyn SettingsSource) -> String {
    let waiting = |provider: &Provider| {
        provider.key.filter(|(path, _)| cfg.text(path).trim().is_empty()).map(|(_, kind)| kind)
    };
    let (on, off): (Vec<_>, Vec<_>) =
        OTHER_PROVIDERS.iter().partition(|provider| cfg.flag(provider.enabled));
    let ready: Vec<_> = on
        .iter()
        .filter(|provider| waiting(provider).is_none())
        .map(|provider| tr(provider.name))
        .collect();
    let mut parts = vec![if ready.is_empty() {
        tr(SEARCHING_NONE).to_string()
    } else {
        tr_args!(SEARCHING, sources => listed(LIST_KEY, &ready))
    }];
    for provider in &on {
        if let Some(kind) = waiting(provider) {
            parts.push(tr_args!(NEEDS_KEY, source => tr(provider.name), key => kind));
        }
    }
    if !off.is_empty() {
        let names: Vec<_> = off.iter().map(|provider| tr(provider.name)).collect();
        let (_, count) = MANY.pick(names.len() > 1);
        parts.push(tr_args!(SWITCHED_OFF, count => count, sources => listed(LIST_KEY, &names)));
    }
    parts.concat()
}
