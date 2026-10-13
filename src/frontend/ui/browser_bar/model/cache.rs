use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::frontend::browser::{Browser, Source};
use crate::i18n::tr;

use super::super::super::bar::BarItem;
use super::super::super::misc::text_width;
use super::super::super::{mirror_x_for, rtl};
use super::builder::browser_bar_items;
use super::types::BrowserAct;

const SAFETY_GROUP: &str = "browser-group-safety";
const SAFETY_COLOUR_GROUP: &str = "browser-group-safety-colour";

fn browser_bar_key(browser: &Browser, scale: f32, max_width: f32, rtl: bool) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    scale.to_bits().hash(&mut hasher);
    max_width.to_bits().hash(&mut hasher);
    rtl.hash(&mut hasher);
    browser.source.key().hash(&mut hasher);
    browser.request.catalog.max_duration.hash(&mut hasher);
    browser.request.catalog.unsplash.order_by.hash(&mut hasher);
    browser.request.catalog.unsplash.orientation.hash(&mut hasher);
    browser.request.catalog.unsplash.color.hash(&mut hasher);
    browser.request.catalog.unsplash.content_filter.hash(&mut hasher);
    browser.request.catalog.pexels.orientation.hash(&mut hasher);
    browser.request.catalog.pexels.size.hash(&mut hasher);
    browser.request.catalog.pexels.color.hash(&mut hasher);
    browser.request.sorting.hash(&mut hasher);
    browser.request.steam.kind.hash(&mut hasher);
    browser.request.steam.resolution.hash(&mut hasher);
    browser.request.steam.category.hash(&mut hasher);
    browser.request.steam.trend_days.hash(&mut hasher);
    (
        browser.request.wallhaven.general,
        browser.request.wallhaven.anime,
        browser.request.wallhaven.people,
        browser.request.wallhaven.sfw,
        browser.request.wallhaven.sketchy,
        browser.request.nsfw,
    )
        .hash(&mut hasher);
    browser.request.wallhaven.color.hash(&mut hasher);
    browser.request.wallhaven.top_range.hash(&mut hasher);
    browser.request.wallhaven.atleast.hash(&mut hasher);
    browser.request.wallhaven.atmost.hash(&mut hasher);
    browser.request.wallhaven.res_exact.hash(&mut hasher);
    browser.request.wallhaven.resolutions.hash(&mut hasher);
    browser.request.wallhaven.ratios.hash(&mut hasher);
    browser.request.wallhaven.collection.hash(&mut hasher);
    browser.request.wallhaven.collections.len().hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn browser_bar_items_compact(
    browser: &Browser,
    scale: f32,
    max_width: f32,
) -> Rc<Vec<(BarItem, BrowserAct)>> {
    let rtl = rtl();
    let key = browser_bar_key(browser, scale, max_width, rtl);
    if let Some((previous_key, items)) = browser.view.bar_items.borrow().as_ref()
        && *previous_key == key
    {
        return items.clone();
    }
    let items = Rc::new(compact_reflow(browser, scale, max_width, rtl));
    *browser.view.bar_items.borrow_mut() = Some((key, items.clone()));
    items
}

pub(super) fn compact_reflow(
    browser: &Browser,
    scale: f32,
    max_width: f32,
    rtl: bool,
) -> Vec<(BarItem, BrowserAct)> {
    let source = browser_bar_items(browser, scale);
    let mut output = Vec::with_capacity(source.len());
    let mut current_group = None;
    let mut section = 0usize;
    let mut x = 0.0_f32;
    let mut y = 0.0_f32;
    let mut row_height = 0.0_f32;
    let wrap_gap = 4.0 * scale;
    let group_gap = 13.0 * scale;
    let item_gap = 3.0 * scale;

    for (mut item, action) in source {
        if matches!(action, BrowserAct::Close) {
            continue;
        }
        let group = compact_section_label(browser, &action);
        let starts_group = current_group.is_some_and(|previous| previous != group);
        if current_group.is_none() || starts_group {
            if starts_group {
                y += row_height + group_gap;
            }
            section += 1;
            let heading = format!("{section:02}  {group}");
            output.push((
                BarItem {
                    x: 0.0,
                    y,
                    w: max_width,
                    h: 14.0 * scale,
                    skew: 0.0,
                    label: heading,
                    nerd: false,
                    text_size: 8.5 * scale,
                    swatch: None,
                    notice: None,
                    active: false,
                    action: None,
                    z: 0,
                },
                BrowserAct::Label,
            ));
            y += 20.0 * scale;
            x = 0.0;
            row_height = 0.0;
        }
        current_group = Some(group);

        item.skew = 0.0;
        if item.swatch.is_some() {
            item.w = 18.0 * scale;
            item.h = 18.0 * scale;
        } else if is_order_action(&action) {
            item.w = text_width(&item.label, item.text_size, item.nerd) + 20.0 * scale;
            item.h = 20.0 * scale;
        } else {
            item.w = text_width(&item.label, item.text_size, item.nerd) + 16.0 * scale;
            item.h = 22.0 * scale;
        }
        if x > 0.0 && x + item.w > max_width {
            y += row_height + wrap_gap;
            x = 0.0;
            row_height = 0.0;
        }
        item.x = x;
        item.y = y;
        x += item.w + item_gap;
        row_height = row_height.max(item.h);
        output.push((item, action));
    }
    for (item, _) in &mut output {
        item.x = mirror_x_for(rtl, item.x, item.w, max_width);
    }
    output
}

fn compact_section_label(browser: &Browser, action: &BrowserAct) -> &'static str {
    tr(match (browser.source, action) {
        (Source::Wallhaven, BrowserAct::Category(_)) => "browser-group-content",
        (Source::Wallhaven, BrowserAct::Sort(_) | BrowserAct::TopRange(_))
        | (Source::Steam, BrowserAct::Sort(_) | BrowserAct::SteamFilter("days", _))
        | (Source::Unsplash, BrowserAct::CatalogFilter("unsplash-order", _)) => {
            "browser-group-order"
        }
        (Source::Wallhaven, BrowserAct::Purity(_) | BrowserAct::Color(_)) => {
            if browser.request.wallhaven.collection.is_empty() {
                SAFETY_COLOUR_GROUP
            } else {
                SAFETY_GROUP
            }
        }
        (
            Source::Wallhaven,
            BrowserAct::ResMode | BrowserAct::Atleast(_) | BrowserAct::Ratios(_),
        ) => "browser-group-format",
        (Source::Wallhaven, BrowserAct::Atmost(_) | BrowserAct::Label) => "browser-group-ceiling",
        (Source::Wallhaven, BrowserAct::Collection(_)) => "browser-group-collection",
        (Source::Steam, BrowserAct::SteamFilter("type" | "nsfw", _)) => {
            "browser-group-media-safety"
        }
        (Source::Steam, BrowserAct::SteamFilter("res", _)) => "browser-group-resolution",
        (Source::Steam, BrowserAct::SteamFilter("cat", _)) => "browser-group-subject",
        (Source::Unsplash, BrowserAct::CatalogFilter("unsplash-safety", _)) => SAFETY_GROUP,
        (Source::Unsplash, BrowserAct::CatalogFilter("unsplash-orientation", _))
        | (Source::Pexels, BrowserAct::CatalogFilter("pexels-orientation", _)) => {
            "browser-group-orientation"
        }
        (Source::Unsplash, BrowserAct::CatalogFilter("unsplash-color", _))
        | (Source::Pexels, BrowserAct::CatalogFilter("pexels-color", _)) => "browser-group-colour",
        (Source::Pexels, BrowserAct::CatalogFilter("pexels-size", _)) => "browser-group-size",
        (Source::Youtube, BrowserAct::MaxDuration(_)) => "browser-group-duration",
        _ => "browser-group-options",
    })
}

pub(crate) fn is_order_action(action: &BrowserAct) -> bool {
    matches!(action, BrowserAct::Sort(_) | BrowserAct::TopRange(_))
        || matches!(action, BrowserAct::SteamFilter("days", _))
        || matches!(action, BrowserAct::CatalogFilter("unsplash-order", _))
}

pub fn browser_bar_compact_size(browser: &Browser, scale: f32, max_width: f32) -> (f32, f32) {
    let items = browser_bar_items_compact(browser, scale, max_width);
    let width = items.iter().map(|(item, _)| item.x + item.w).fold(0.0f32, f32::max);
    let height = items.iter().map(|(item, _)| item.y + item.h).fold(0.0f32, f32::max);
    (width.max(1.0), height.max(1.0))
}
