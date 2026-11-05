use crate::frontend::browser::model::{
    Browser, BrowserMsg, RATIOS, RESOLUTIONS, SORT_KEYS, TOP_RANGES, TOPLIST_SORT, is_selected,
};
use crate::frontend::ui::browser_bar::{wallhaven_range_label_key, wallhaven_sort_label_key};
use crate::frontend::ui::swatch_color;
use crate::i18n::tr;

use super::rail::{Entry, HeadingAction, Section, choices};

const CONTENT_KEY: &str = "browser-group-content";
const ORDER_KEY: &str = "browser-group-order";
const TOP_RANGE_KEY: &str = "browser-group-top-range";
const SAFETY_KEY: &str = "browser-group-safety";
const COLOUR_KEY: &str = "browser-group-colour";
const RESOLUTION_KEY: &str = "browser-group-resolution";
const RATIO_KEY: &str = "browser-group-ratio";
const MAX_SIZE_KEY: &str = "browser-max-size";
const COLLECTION_KEY: &str = "browser-group-collection";
const MIN_MODE_KEY: &str = "browser-min-size";
const EXACT_MODE_KEY: &str = "browser-exact-size";
const ANY_KEY: &str = "browser-any";
const ALL_KEY: &str = "browser-all";
const CONTENT: [(&str, &str); 3] =
    [("general", "browser-general"), ("anime", "browser-anime"), ("people", "browser-people")];
const PURITY: [(&str, &str); 3] =
    [("sfw", "browser-sfw"), ("sketchy", "browser-sketchy"), ("nsfw", "browser-nsfw")];
const COLOURS: usize = 13;
const COLOUR_COLUMNS: usize = 7;
const NO_COLOUR: i64 = -1;

fn any_or(key: &str, label: &'static str) -> &'static str {
    if key.is_empty() { tr(ANY_KEY) } else { label }
}

fn labelled(
    title_key: &'static str,
    pairs: &[(&'static str, &'static str)],
    chosen: impl Fn(&str) -> bool,
    press: impl Fn(&'static str) -> BrowserMsg,
) -> Section {
    let entries = pairs
        .iter()
        .map(|&(key, label)| Entry::choice(key, any_or(key, label), chosen(key), press(key)))
        .collect();
    Section::new(title_key, entries)
}

fn toggles(
    title_key: &'static str,
    items: &[(&'static str, &'static str)],
    on: &[bool],
    press: impl Fn(u8) -> BrowserMsg,
) -> Section {
    let entries = items
        .iter()
        .zip(on)
        .zip(0u8..)
        .map(|((&(id, label_key), &active), bit)| {
            Entry::choice(id, tr(label_key), active, press(bit))
        })
        .collect();
    Section::new(title_key, entries)
}

pub(super) fn sections(browser: &Browser) -> Vec<Section> {
    let request = &browser.request;
    let wallhaven = &request.wallhaven;
    let browsing = wallhaven.collection.is_empty();
    let mut sections = Vec::new();
    if browsing {
        sections.push(toggles(
            CONTENT_KEY,
            &CONTENT,
            &[wallhaven.general, wallhaven.anime, wallhaven.people],
            BrowserMsg::ToggleCategory,
        ));
        sections.push(choices(
            ORDER_KEY,
            SORT_KEYS,
            |key| tr(wallhaven_sort_label_key(key)),
            |key| request.sorting == key,
            |key| BrowserMsg::SetSort(key.to_owned()),
        ));
        if request.sorting == TOPLIST_SORT {
            sections.push(choices(
                TOP_RANGE_KEY,
                TOP_RANGES.map(|(key, _)| key),
                |key| tr(wallhaven_range_label_key(key)),
                |key| wallhaven.top_range == key,
                |key| BrowserMsg::SetTopRange(key.to_owned()),
            ));
        }
    }
    sections.push(toggles(
        SAFETY_KEY,
        &PURITY,
        &[wallhaven.sfw, wallhaven.sketchy, request.nsfw],
        BrowserMsg::TogglePurity,
    ));
    if browsing {
        let swatches = (0..COLOURS)
            .map(|index| {
                let chosen = wallhaven.color == index as i64;
                let mut entry = Entry::choice(
                    index.to_string(),
                    "",
                    chosen,
                    BrowserMsg::SetColor(if chosen { NO_COLOUR } else { index as i64 }),
                );
                entry.swatch = Some(swatch_color(index, true));
                entry
            })
            .collect();
        let mut colour = Section::new(COLOUR_KEY, swatches);
        colour.columns = COLOUR_COLUMNS;
        sections.push(colour);
        let exact = wallhaven.res_exact;
        let sizes = RESOLUTIONS
            .into_iter()
            .map(|(key, label)| {
                let chosen = if key.is_empty() {
                    wallhaven.atleast.is_empty() && wallhaven.resolutions.is_empty()
                } else if exact {
                    is_selected(&wallhaven.resolutions, key)
                } else {
                    wallhaven.atleast == key
                };
                Entry::choice(
                    key,
                    any_or(key, label),
                    chosen,
                    BrowserMsg::SetAtleast(key.to_owned()),
                )
            })
            .collect();
        let mut resolution = Section::new(RESOLUTION_KEY, sizes);
        resolution.actions = [(MIN_MODE_KEY, !exact), (EXACT_MODE_KEY, exact)]
            .into_iter()
            .map(|(key, current)| HeadingAction {
                key,
                current,
                message: (!current).then_some(BrowserMsg::ToggleResExact),
            })
            .collect();
        sections.push(resolution);
        sections.push(labelled(
            RATIO_KEY,
            &RATIOS,
            |key| {
                if key.is_empty() {
                    wallhaven.ratios.is_empty()
                } else {
                    is_selected(&wallhaven.ratios, key)
                }
            },
            |key| BrowserMsg::SetRatios(key.to_owned()),
        ));
    }
    if !browsing || !wallhaven.res_exact {
        sections.push(labelled(
            MAX_SIZE_KEY,
            &RESOLUTIONS,
            |key| wallhaven.atmost == key,
            |key| BrowserMsg::SetAtmost(key.to_owned()),
        ));
    }
    if !wallhaven.collections.is_empty() {
        let all =
            Entry::choice("", tr(ALL_KEY), browsing, BrowserMsg::SetCollection(String::new()));
        let entries = std::iter::once(all)
            .chain(wallhaven.collections.iter().map(|(id, label)| {
                let shown = if label.is_empty() { id.as_str() } else { label.as_str() };
                Entry::choice(
                    id.clone(),
                    shown,
                    &wallhaven.collection == id,
                    BrowserMsg::SetCollection(id.clone()),
                )
            }))
            .collect();
        sections.push(Section::new(COLLECTION_KEY, entries));
    }
    sections
}
