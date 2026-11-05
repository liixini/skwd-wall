use crate::frontend::browser::model::{
    Browser, BrowserMsg, STEAM_CATEGORIES, STEAM_DAYS_FILTER, STEAM_NSFW_FILTER, STEAM_NSFW_VALUE,
    STEAM_RESOLUTION_FILTER, STEAM_RESOLUTIONS, STEAM_SFW_VALUE, STEAM_SORTS, STEAM_TREND_DAYS,
    STEAM_TYPE_FILTER, STEAM_TYPES, TREND_SORT, TagMark,
};
use crate::frontend::ui::EXCLUDED_MARK;
use crate::frontend::ui::browser_bar::{
    steam_category_label_key, steam_resolution_label, steam_sort_label_key,
    steam_trend_days_label_key, steam_type_label_key,
};
use crate::i18n::tr;

use super::rail::{Entry, HeadingAction, Section, choices};

const ORDER_KEY: &str = "browser-group-order";
const PERIOD_KEY: &str = "browser-group-period";
const MEDIA_KEY: &str = "browser-group-media";
const SAFETY_KEY: &str = "browser-group-safety";
const RESOLUTION_KEY: &str = "browser-group-resolution";
const TAGS_KEY: &str = "browser-group-tags";
const CLEAR_KEY: &str = "browser-tags-clear";
const SFW_KEY: &str = "browser-sfw";
const NSFW_KEY: &str = "browser-nsfw";

pub(super) fn sections(browser: &Browser) -> Vec<Section> {
    let request = &browser.request;
    let filter = |kind: &str| {
        let kind = kind.to_owned();
        move |value: &str| BrowserMsg::SteamFilter(kind.clone(), value.to_owned())
    };
    let mut sections = vec![choices(
        ORDER_KEY,
        STEAM_SORTS,
        |key| tr(steam_sort_label_key(key)),
        |key| request.sorting == key,
        |key| BrowserMsg::SetSort(key.to_owned()),
    )];
    if request.sorting == TREND_SORT {
        sections.push(choices(
            PERIOD_KEY,
            STEAM_TREND_DAYS,
            |key| tr(steam_trend_days_label_key(key)),
            |key| request.steam.trend_days == key,
            filter(STEAM_DAYS_FILTER),
        ));
    }
    sections.push(choices(
        MEDIA_KEY,
        STEAM_TYPES,
        |key| tr(steam_type_label_key(key)),
        |key| request.steam.kind == key,
        filter(STEAM_TYPE_FILTER),
    ));
    sections.push(choices(
        SAFETY_KEY,
        [STEAM_SFW_VALUE, STEAM_NSFW_VALUE],
        |key| tr(if key == STEAM_NSFW_VALUE { NSFW_KEY } else { SFW_KEY }),
        |key| request.nsfw == (key == STEAM_NSFW_VALUE),
        filter(STEAM_NSFW_FILTER),
    ));
    sections.push(choices(
        RESOLUTION_KEY,
        STEAM_RESOLUTIONS,
        steam_resolution_label,
        |key| request.steam.resolution == key,
        filter(STEAM_RESOLUTION_FILTER),
    ));
    let tags: Vec<Entry> = STEAM_CATEGORIES
        .into_iter()
        .filter(|key| !key.is_empty())
        .map(|key| {
            let mark = request.steam.tag_mark(key);
            let name = tr(steam_category_label_key(key));
            Entry {
                id: key.to_owned(),
                title: if mark == TagMark::Exclude {
                    format!("{EXCLUDED_MARK}{name}")
                } else {
                    name.to_owned()
                },
                mark,
                press: BrowserMsg::SteamTag(key.to_owned(), false),
                exclude: Some(BrowserMsg::SteamTag(key.to_owned(), true)),
                swatch: None,
            }
        })
        .collect();
    let marked = tags.iter().any(|entry| entry.mark != TagMark::Off);
    let mut section = Section::new(TAGS_KEY, tags);
    if marked {
        section.actions.push(HeadingAction {
            key: CLEAR_KEY,
            current: true,
            message: Some(BrowserMsg::SteamTag(String::new(), false)),
        });
    }
    sections.push(section);
    sections
}
