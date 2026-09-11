use iced::Task;

use crate::frontend::scene::layout::GridParams;
use crate::infrastructure::config::Config;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(crate) fn browser_wall_params(config: &Config) -> GridParams {
    let layout = config.browser_grid(false);
    let total_w =
        layout.thumb_w * layout.cols as f32 + layout.gap_x * layout.cols.saturating_sub(1) as f32;
    let total_h =
        layout.thumb_h * layout.rows as f32 + layout.gap_y * layout.rows.saturating_sub(1) as f32;
    GridParams {
        cols: layout.cols,
        rows: layout.rows,
        thumb_w: total_w / layout.cols.max(1) as f32,
        thumb_h: total_h / layout.rows.max(1) as f32,
        gap_x: 0.0,
        gap_y: 0.0,
        corner_radius: 0.0,
        border_width: 0.0,
        ..GridParams::default()
    }
}

pub(crate) fn browser_key_nav(app: &mut App, delta: i64) -> Task<Message> {
    let Some(browser) = app.source_browser.browser.as_ref() else {
        return Task::none();
    };
    let count = browser.session.items.len() as i64;
    if count == 0 {
        return Task::none();
    }
    if let Some(current) = browser.session.preview {
        if delta.abs() != 1 {
            return Task::none();
        }
        let next = current as i64 + delta;
        if next < 0 || next >= count {
            return Task::none();
        }
        return update(
            app,
            Message::Browser(crate::frontend::browser::BrowserMsg::OpenPreview(next as usize)),
        );
    }
    let next = match app.source_browser.wall.scene.hover {
        Some(current) => current as i64 + delta,
        None => 0,
    };
    if next < 0 || next >= count {
        return Task::none();
    }
    let next = next as usize;
    app.source_browser.wall.scene.kb_nav = true;
    app.source_browser.wall.scene.set_current(next, count as usize);
    app.source_browser.wall.scene.hover = Some(next);
    if let Some(browser) = app.source_browser.browser.as_mut() {
        browser.session.hover = Some(next);
    }
    app.source_browser.wall.chrome_cache.clear();
    app.retick();
    Task::none()
}

pub(crate) fn apply_browser_defaults(
    browser: &mut crate::frontend::browser::Browser,
    config: &Config,
) {
    use crate::frontend::browser::{
        RATIOS, RESOLUTIONS, SORT_KEYS, STEAM_CATEGORIES, STEAM_RESOLUTIONS, STEAM_SORTS,
        STEAM_TREND_DAYS, STEAM_TYPES, Source, TOP_RANGES,
    };
    use skwd_config::keys::{steam, wallhaven};

    let choice = |path: &str, choices: &[&str], fallback: &str| {
        let value = config.str_path(path);
        if choices.contains(&value.as_str()) { value } else { fallback.to_string() }
    };
    match browser.source {
        Source::Wallhaven => {
            browser.request.sorting = choice(wallhaven::DEFAULT_SORT, &SORT_KEYS, "toplist");
            let request = &mut browser.request.wallhaven;
            request.top_range =
                choice(wallhaven::DEFAULT_TOP_RANGE, &TOP_RANGES.map(|(key, _)| key), "1M");
            request.atleast =
                choice(wallhaven::DEFAULT_ATLEAST, &RESOLUTIONS.map(|(key, _)| key), "");
            request.atmost =
                choice(wallhaven::DEFAULT_ATMOST, &RESOLUTIONS.map(|(key, _)| key), "");
            request.ratios = choice(wallhaven::DEFAULT_RATIOS, &RATIOS.map(|(key, _)| key), "");
            request.general = config.flag_default_config(wallhaven::DEFAULT_GENERAL);
            request.anime = config.flag_default_config(wallhaven::DEFAULT_ANIME);
            request.people = config.flag_default_config(wallhaven::DEFAULT_PEOPLE);
            if !request.general && !request.anime && !request.people {
                request.general = true;
            }
            request.sfw = config.flag_default_config(wallhaven::DEFAULT_SFW);
            request.sketchy = config.flag_default_config(wallhaven::DEFAULT_SKETCHY);
            browser.request.nsfw = config.flag_default_config(wallhaven::DEFAULT_NSFW);
            if !request.sfw && !request.sketchy && !browser.request.nsfw {
                request.sfw = true;
            }
        }
        Source::Steam => {
            browser.request.sorting = choice(steam::DEFAULT_SORT, &STEAM_SORTS, "3");
            browser.request.steam.kind = choice(steam::DEFAULT_TYPE, &STEAM_TYPES, "");
            browser.request.steam.category = choice(steam::DEFAULT_CATEGORY, &STEAM_CATEGORIES, "");
            browser.request.steam.resolution =
                choice(steam::DEFAULT_RESOLUTION, &STEAM_RESOLUTIONS, "");
            browser.request.steam.trend_days =
                choice(steam::DEFAULT_TREND_DAYS, &STEAM_TREND_DAYS, "7");
            browser.request.nsfw = config.flag_default_config(steam::DEFAULT_NSFW);
        }
        _ => {}
    }
}
