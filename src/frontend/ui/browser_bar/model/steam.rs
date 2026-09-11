use crate::frontend::browser::{
    Browser, STEAM_CATEGORIES, STEAM_RESOLUTIONS, STEAM_SORTS, STEAM_TREND_DAYS, STEAM_TYPES,
};
use crate::i18n::tr;

use super::super::super::bar::BarItem;
use super::super::super::misc::text_width;
use super::item::push_item;
use super::types::BrowserAct;

pub(crate) fn sort_label_key(key: &str) -> &'static str {
    match key {
        "0" => "browser-top",
        "1" => "browser-new",
        "21" => "browser-updated",
        "9" => "browser-subscribers",
        _ => "browser-trend",
    }
}

pub(crate) fn trend_days_label_key(key: &str) -> &'static str {
    match key {
        "1" => "browser-1-day",
        "3" => "browser-3-days",
        _ => "browser-7-days",
    }
}

pub(crate) fn type_label_key(key: &str) -> &'static str {
    match key {
        "Scene" => "browser-scene",
        "Video" => "browser-video",
        _ => "browser-scene-video",
    }
}

pub(crate) fn resolution_label(key: &str) -> &'static str {
    match key {
        "1920 x 1080" => "1080p",
        "2560 x 1440" => "2K",
        "3840 x 2160" => "4K",
        "2560 x 1080" => "UW",
        "3440 x 1440" => "UWQHD",
        "3840 x 1080" => tr("browser-dual"),
        _ => tr("browser-any"),
    }
}

pub(crate) fn category_label_key(key: &str) -> &'static str {
    match key {
        "Abstract" => "browser-abstract",
        "Animal" => "browser-animal",
        "Anime" => "browser-anime",
        "CGI" => "browser-cgi",
        "Cyberpunk" => "browser-cyberpunk",
        "Fantasy" => "browser-fantasy",
        "Game" => "browser-game",
        "Girls" => "browser-girls",
        "Guys" => "browser-guys",
        "Landscape" => "browser-landscape",
        "Medieval" => "browser-medieval",
        "Music" => "browser-music",
        "Nature" => "browser-nature",
        "Pixel art" => "browser-pixel-art",
        "Relaxing" => "browser-relaxing",
        "Retro" => "browser-retro",
        "Sci-Fi" => "browser-sci-fi",
        "Technology" => "browser-technology",
        "Vehicle" => "browser-vehicle",
        _ => "browser-all",
    }
}

pub(super) fn build_rows(
    browser: &Browser,
    scale: f32,
    mut x: f32,
    output: &mut Vec<(BarItem, BrowserAct)>,
) {
    let height = 24.0 * scale;
    let skew = 8.0 * scale;
    let row_height = height + 6.0 * scale;
    for key in STEAM_SORTS {
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

    if browser.request.sorting == "3" {
        x += 12.0 * scale;
        for key in STEAM_TREND_DAYS {
            push_item(
                output,
                scale,
                &mut x,
                0.0,
                tr(trend_days_label_key(key)),
                false,
                10.0 * scale,
                browser.request.steam.trend_days == key,
                BrowserAct::SteamFilter("days", key.to_string()),
            );
        }
    }

    let mut type_x = 0.0;
    for key in STEAM_TYPES {
        push_item(
            output,
            scale,
            &mut type_x,
            row_height,
            tr(type_label_key(key)),
            false,
            10.0 * scale,
            browser.request.steam.kind == key,
            BrowserAct::SteamFilter("type", key.to_string()),
        );
    }
    type_x += 12.0 * scale;
    push_item(
        output,
        scale,
        &mut type_x,
        row_height,
        tr("browser-sfw"),
        false,
        10.0 * scale,
        !browser.request.nsfw,
        BrowserAct::SteamFilter("nsfw", "0".into()),
    );
    push_item(
        output,
        scale,
        &mut type_x,
        row_height,
        tr("browser-nsfw"),
        false,
        10.0 * scale,
        browser.request.nsfw,
        BrowserAct::SteamFilter("nsfw", "1".into()),
    );

    let mut resolution_x = 0.0;
    let resolution_y = 2.0 * row_height;
    for key in STEAM_RESOLUTIONS {
        push_item(
            output,
            scale,
            &mut resolution_x,
            resolution_y,
            resolution_label(key),
            false,
            9.0 * scale,
            browser.request.steam.resolution == key,
            BrowserAct::SteamFilter("res", key.to_string()),
        );
    }

    let max_width = 720.0 * scale;
    let mut category_x = 0.0;
    let mut category_y = 3.0 * row_height;
    for key in STEAM_CATEGORIES {
        let shown = tr(category_label_key(key));
        let width = text_width(shown, 9.0 * scale, false) + 24.0 * scale + skew;
        if category_x > 0.0 && category_x + width > max_width {
            category_x = 0.0;
            category_y += row_height;
        }
        push_item(
            output,
            scale,
            &mut category_x,
            category_y,
            shown,
            false,
            9.0 * scale,
            browser.request.steam.category == key,
            BrowserAct::SteamFilter("cat", key.to_string()),
        );
    }
}
