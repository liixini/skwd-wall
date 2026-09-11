use crate::frontend::browser::{Browser, RATIOS, RESOLUTIONS, SORT_KEYS, TOP_RANGES, is_selected};
use crate::i18n::tr;

use super::super::super::bar::BarItem;
use super::item::push_item;
use super::types::BrowserAct;

pub(crate) fn sort_label_key(key: &str) -> &'static str {
    match key {
        "toplist" => "browser-top",
        "hot" => "browser-hot",
        "date_added" => "browser-new",
        "relevance" => "browser-relevant",
        "views" => "browser-views",
        "favorites" => "browser-favourites",
        _ => "browser-random",
    }
}

pub(crate) fn range_label_key(key: &str) -> &'static str {
    match key {
        "1d" => "browser-day",
        "3d" => "browser-3-days",
        "1w" => "browser-week",
        "1M" => "browser-month",
        "3M" => "browser-3-months",
        "6M" => "browser-6-months",
        _ => "browser-year",
    }
}

fn any_or(key: &str, label: &'static str) -> &'static str {
    if key.is_empty() { tr("browser-any") } else { label }
}

pub(super) fn build_rows(
    browser: &Browser,
    scale: f32,
    start_x: f32,
    output: &mut Vec<(BarItem, BrowserAct)>,
) {
    let second_row_x = build_top_row(browser, scale, start_x, output);
    build_purity_row(browser, scale, second_row_x, output);
    build_resolution_rows(browser, scale, output);
    build_collections_row(browser, scale, output);
}

fn build_top_row(
    browser: &Browser,
    scale: f32,
    mut x: f32,
    output: &mut Vec<(BarItem, BrowserAct)>,
) -> f32 {
    for (bit, label) in
        [(0u8, tr("browser-general")), (1, tr("browser-anime")), (2, tr("browser-people"))]
    {
        let active = matches!(bit, 0 if browser.request.wallhaven.general)
            || matches!(bit, 1 if browser.request.wallhaven.anime)
            || matches!(bit, 2 if browser.request.wallhaven.people);
        push_item(
            output,
            scale,
            &mut x,
            0.0,
            label,
            false,
            10.0 * scale,
            active,
            BrowserAct::Category(bit),
        );
    }
    x += 8.0 * scale;
    for key in SORT_KEYS {
        push_item(
            output,
            scale,
            &mut x,
            0.0,
            tr(sort_label_key(key)),
            false,
            10.0 * scale,
            browser.request.sorting == key,
            BrowserAct::Sort(key),
        );
    }
    if browser.request.sorting != "toplist" {
        return 0.0;
    }
    let mut range_x = 0.0;
    for (key, _) in TOP_RANGES {
        push_item(
            output,
            scale,
            &mut range_x,
            30.0 * scale,
            tr(range_label_key(key)),
            false,
            9.0 * scale,
            browser.request.wallhaven.top_range == key,
            BrowserAct::TopRange(key),
        );
    }
    range_x + 10.0 * scale
}

fn build_purity_row(
    browser: &Browser,
    scale: f32,
    start_x: f32,
    output: &mut Vec<(BarItem, BrowserAct)>,
) {
    let height = 24.0 * scale;
    let y = height + 6.0 * scale;
    let mut x = start_x;
    for (bit, label) in
        [(0u8, tr("browser-sfw")), (1, tr("browser-sketchy")), (2, tr("browser-nsfw"))]
    {
        let active = matches!(bit, 0 if browser.request.wallhaven.sfw)
            || matches!(bit, 1 if browser.request.wallhaven.sketchy)
            || matches!(bit, 2 if browser.request.nsfw);
        push_item(
            output,
            scale,
            &mut x,
            y,
            label,
            false,
            10.0 * scale,
            active,
            BrowserAct::Purity(bit),
        );
    }
    x += 10.0 * scale;
    for index in 0..13usize {
        let width = 28.0 * scale;
        let active = browser.request.wallhaven.color == index as i64;
        output.push((
            BarItem {
                x,
                y,
                w: width,
                h: height,
                skew: 10.0 * scale,
                label: String::new(),
                nerd: false,
                text_size: 0.0,
                swatch: Some(index),
                notice: None,
                active,
                action: None,
                z: if active { 10 } else { 1 },
            },
            BrowserAct::Color(if active { -1 } else { index as i64 }),
        ));
        x += width - 10.0 * scale;
    }
}

fn build_resolution_rows(browser: &Browser, scale: f32, output: &mut Vec<(BarItem, BrowserAct)>) {
    let height = 24.0 * scale;
    let y = 2.0 * height + 12.0 * scale;
    let mut x = 0.0;
    let mode_label = if browser.request.wallhaven.res_exact {
        tr("browser-exact-size")
    } else {
        tr("browser-min-size")
    };
    push_item(
        output,
        scale,
        &mut x,
        y,
        mode_label,
        false,
        9.0 * scale,
        browser.request.wallhaven.res_exact,
        BrowserAct::ResMode,
    );
    x += 6.0 * scale;
    for (key, label) in RESOLUTIONS {
        let active = if key.is_empty() {
            browser.request.wallhaven.atleast.is_empty()
                && browser.request.wallhaven.resolutions.is_empty()
        } else if browser.request.wallhaven.res_exact {
            is_selected(&browser.request.wallhaven.resolutions, key)
        } else {
            browser.request.wallhaven.atleast == key
        };
        push_item(
            output,
            scale,
            &mut x,
            y,
            any_or(key, label),
            false,
            9.0 * scale,
            active,
            BrowserAct::Atleast(key),
        );
    }
    x += 10.0 * scale;
    for (key, label) in RATIOS {
        let active = if key.is_empty() {
            browser.request.wallhaven.ratios.is_empty()
        } else {
            is_selected(&browser.request.wallhaven.ratios, key)
        };
        push_item(
            output,
            scale,
            &mut x,
            y,
            any_or(key, label),
            false,
            9.0 * scale,
            active,
            BrowserAct::Ratios(key),
        );
    }
    if browser.request.wallhaven.res_exact {
        return;
    }

    let max_y = 3.0 * height + 18.0 * scale;
    let mut max_x = 0.0;
    push_item(
        output,
        scale,
        &mut max_x,
        max_y,
        tr("browser-max-size"),
        false,
        9.0 * scale,
        false,
        BrowserAct::Label,
    );
    max_x += 6.0 * scale;
    for (key, label) in RESOLUTIONS {
        let active = if key.is_empty() {
            browser.request.wallhaven.atmost.is_empty()
        } else {
            browser.request.wallhaven.atmost == key
        };
        push_item(
            output,
            scale,
            &mut max_x,
            max_y,
            any_or(key, label),
            false,
            9.0 * scale,
            active,
            BrowserAct::Atmost(key),
        );
    }
}

fn build_collections_row(browser: &Browser, scale: f32, output: &mut Vec<(BarItem, BrowserAct)>) {
    if browser.request.wallhaven.collections.is_empty() {
        return;
    }
    let height = 24.0 * scale;
    let y = if browser.request.wallhaven.res_exact {
        3.0 * height + 18.0 * scale
    } else {
        4.0 * height + 24.0 * scale
    };
    let mut x = 0.0;
    push_item(
        output,
        scale,
        &mut x,
        y,
        tr("browser-all"),
        false,
        9.0 * scale,
        browser.request.wallhaven.collection.is_empty(),
        BrowserAct::Collection(String::new()),
    );
    x += 6.0 * scale;
    for (id, label) in &browser.request.wallhaven.collections {
        let shown = if label.is_empty() { id.as_str() } else { label.as_str() };
        push_item(
            output,
            scale,
            &mut x,
            y,
            shown,
            false,
            9.0 * scale,
            &browser.request.wallhaven.collection == id,
            BrowserAct::Collection(id.clone()),
        );
    }
}
