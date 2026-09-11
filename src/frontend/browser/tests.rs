#![cfg(test)]

use super::model::*;
use super::view::{BrowserCardAction, browser_card_action_at, flight_label};
use crate::contracts::browser::{DownloadStatus, DownloadUpdate, SearchRequest};
use crate::frontend::scene::layout::{HexShape, Hit};

#[test]
fn card_actions_use_the_card_bottom_edge() {
    let hit = Hit {
        index: 4,
        cx: 100.0,
        cy: 100.0,
        hw: 60.0,
        hh: 45.0,
        skew: 0.0,
        edge_tilt: 0.0,
        hex: false,
        hex_shape: HexShape::Hexagon,
        triangle_direction: 0,
    };
    assert_eq!(browser_card_action_at(&hit, 80.0, 140.0, 1.0), Some(BrowserCardAction::Save));
    assert_eq!(browser_card_action_at(&hit, 130.0, 140.0, 1.0), Some(BrowserCardAction::Apply));
    assert_eq!(browser_card_action_at(&hit, 100.0, 100.0, 1.0), None);
}

#[test]
fn parse_clock_junk() {
    assert_eq!(parse_clock("45"), Some(45));
    assert_eq!(parse_clock("3:00"), Some(180));
    assert_eq!(parse_clock("0:07"), Some(7));
    assert_eq!(parse_clock("10:20:51"), Some(37251));
    assert_eq!(parse_clock(" 1:30 "), Some(90));
    assert_eq!(parse_clock(""), None);
    assert_eq!(parse_clock("abc"), None);
    assert_eq!(parse_clock("1:"), None);
    assert_eq!(parse_clock("-5"), None);
}

#[test]
fn fmt_clock_round_trips() {
    for secs in [0u64, 7, 90, 180, 3599, 3600, 37251] {
        assert_eq!(parse_clock(&fmt_clock(secs)), Some(secs));
    }
    assert_eq!(fmt_clock(180), "3:00");
    assert_eq!(fmt_clock(37251), "10:20:51");
}

#[test]
fn progress_label_percent() {
    let mut item = mk("x", "", false);
    item.downloading = true;
    assert_eq!(progress_label("Saving", &item), "Saving\u{2026}");
    item.progress = 0.452;
    assert_eq!(progress_label("Saving", &item), "Saving 45%");
    item.phase = "audio".into();
    assert_eq!(progress_label("Downloading", &item), "Downloading 45% \u{00b7} Audio");
}

#[test]
fn download_update_phases() {
    let update = |status, path: Option<&str>, progress, message: Option<&str>| DownloadUpdate {
        id: String::from("video"),
        status,
        path: path.map(str::to_string),
        progress,
        message: message.map(str::to_string),
    };
    let mut item = mk("video", "", false);
    item.apply_download_update(&update(
        DownloadStatus::Queued,
        None,
        None,
        Some("queued - 2 ahead"),
    ));
    assert!(item.queued && item.downloading);
    assert_eq!(item.phase, "queued - 2 ahead");

    item.apply_download_update(&update(
        DownloadStatus::Downloading,
        None,
        Some(0.5),
        Some("video"),
    ));
    assert!(!item.queued);
    assert!(item.downloading);
    assert_eq!(item.progress, 0.5);

    item.apply_download_update(&update(DownloadStatus::Failed, None, None, None));
    assert!(!item.queued && !item.downloading);

    item.apply_download_update(&update(DownloadStatus::Done, Some("/tmp/video.mp4"), None, None));
    assert!(item.downloaded);
    assert_eq!(item.downloaded_path.as_deref(), Some("/tmp/video.mp4"));
}

#[test]
fn toggle_selected_keys() {
    let mut sel = String::new();
    toggle_selected(&mut sel, "16x9");
    assert_eq!(sel, "16x9");
    toggle_selected(&mut sel, "16x10");
    assert_eq!(sel, "16x9,16x10");
    assert!(is_selected(&sel, "16x9") && is_selected(&sel, "16x10"));
    toggle_selected(&mut sel, "16x9");
    assert_eq!(sel, "16x10");
    assert!(!is_selected(&sel, "16x9"));
    toggle_selected(&mut sel, "");
    assert_eq!(sel, "");
    assert!(!is_selected("16x9,16x10", ""));
}

#[test]
fn flight_label_priority() {
    assert_eq!(flight_label(false, 0, 0, false, "Wallhaven", None), None);
    assert_eq!(
        flight_label(false, 0, 0, true, "Wallhaven", None).as_deref(),
        Some("Searching Wallhaven\u{2026}")
    );
    assert_eq!(
        flight_label(false, 0, 0, true, "Steam Workshop", None).as_deref(),
        Some("Searching Steam Workshop\u{2026}")
    );
    assert_eq!(
        flight_label(false, 1, 0, true, "Wallhaven", None).as_deref(),
        Some("Downloading\u{2026}")
    );
    assert_eq!(
        flight_label(false, 3, 0, true, "Wallhaven", None).as_deref(),
        Some("Downloading 3\u{2026}")
    );
    assert_eq!(
        flight_label(true, 2, 0, true, "Wallhaven", None).as_deref(),
        Some("Applying\u{2026}")
    );
}

#[test]
fn flight_label_queue() {
    assert_eq!(
        flight_label(false, 2, 3, false, "YouTube", None).as_deref(),
        Some("Downloading 2 \u{00b7} 3 queued\u{2026}")
    );
    assert_eq!(
        flight_label(false, 0, 2, false, "YouTube", None).as_deref(),
        Some("2 queued\u{2026}")
    );
    let mut item = mk("v", "", false);
    item.downloading = true;
    item.queued = true;
    item.phase = "queued - 2 ahead".into();
    assert_eq!(
        flight_label(false, 0, 1, false, "YouTube", Some(&item)).as_deref(),
        Some("Queued \u{00b7} 2 ahead\u{2026}")
    );
    item.phase.clear();
    assert_eq!(
        flight_label(false, 0, 1, false, "YouTube", Some(&item)).as_deref(),
        Some("Queued\u{2026}")
    );
}

#[test]
fn flight_label_percent() {
    let mut item = mk("yt1", "/t/yt1.webp", true);
    item.progress = 0.0;
    assert_eq!(
        flight_label(true, 0, 0, false, "YouTube", Some(&item)).as_deref(),
        Some("Applying\u{2026}")
    );
    item.progress = 0.55;
    item.phase = String::from("clipping");
    assert_eq!(
        flight_label(true, 0, 0, false, "YouTube", Some(&item)).as_deref(),
        Some("Applying 55% \u{00b7} Clipping")
    );
    assert_eq!(
        flight_label(false, 1, 0, false, "YouTube", Some(&item)).as_deref(),
        Some("Downloading 55% \u{00b7} Clipping")
    );
}

#[test]
fn wallhaven_search_request() {
    let mut browser = Browser::new(Source::Wallhaven);
    browser.request.query = "forest cabin".into();
    browser.request.sorting = "toplist".into();
    browser.request.wallhaven.top_range = "1w".into();
    browser.request.wallhaven.atleast = "3840x2160".into();
    browser.request.wallhaven.ratios = "16x9".into();
    browser.request.wallhaven.color = 0;
    let SearchRequest::Wallhaven(request) = browser.search_request(3, "") else {
        panic!("expected Wallhaven search");
    };
    assert_eq!(request.query, "forest cabin");
    assert_eq!(request.categories, [true, true, true]);
    assert_eq!(request.purity, [true, false, false]);
    assert_eq!(request.sorting, "toplist");
    assert_eq!(request.top_range, "1w");
    assert_eq!(request.atleast, "3840x2160");
    assert_eq!(request.ratios, "16x9");
    assert_eq!(request.color_hue, Some(0));
    assert_eq!(request.page, 3);
}

#[test]
fn photo_provider_filters() {
    let mut unsplash = Browser::new(Source::Unsplash);
    unsplash.request.catalog.unsplash.order_by = "latest".into();
    unsplash.request.catalog.unsplash.orientation = "squarish".into();
    unsplash.request.catalog.unsplash.color = "teal".into();
    unsplash.request.catalog.unsplash.content_filter = "low".into();
    let SearchRequest::Catalog(request) = unsplash.search_request(2, "ignored") else {
        panic!("expected catalog search");
    };
    assert_eq!(request.order_by, "latest");
    assert_eq!(request.orientation, "squarish");
    assert_eq!(request.color, "teal");
    assert_eq!(request.content_filter, "low");
    assert!(request.size.is_empty());

    let mut pexels = Browser::new(Source::Pexels);
    pexels.request.catalog.pexels.orientation = "portrait".into();
    pexels.request.catalog.pexels.size = "large".into();
    pexels.request.catalog.pexels.color = "blue".into();
    let SearchRequest::Catalog(request) = pexels.search_request(3, "ignored") else {
        panic!("expected catalog search");
    };
    assert_eq!(request.orientation, "portrait");
    assert_eq!(request.size, "large");
    assert_eq!(request.color, "blue");
}

fn mk(id: &str, path: &str, ready: bool) -> BrowserItem {
    BrowserItem {
        id: id.into(),
        full_url: String::new(),
        thumb_path: path.into(),
        title: String::new(),
        resolution: String::new(),
        purity: String::new(),
        file_size: 0,
        category: String::new(),
        thumb_ready: ready,
        downloaded: false,
        downloading: false,
        queued: false,
        progress: 0.0,
        phase: String::new(),
        duration_secs: 0,
        downloaded_path: None,
        preview_path: None,
        thumb_failed: false,
        attribution: String::new(),
        attribution_url: String::new(),
        track_url: String::new(),
    }
}

#[test]
fn preview_loading() {
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items = vec![mk("a", "/c/a.jpg", true), mk("b", "/c/b.jpg", true)];

    assert!(!browser.preview_loading());

    browser.session.preview = Some(0);
    assert!(browser.preview_loading());

    browser.session.items[0].preview_path = Some("/c/a-full.jpg".into());
    assert!(!browser.preview_loading());

    browser.session.items[0].preview_path = Some(String::new());
    assert!(browser.preview_loading());

    browser.session.preview = Some(9);
    assert!(!browser.preview_loading());
}

#[test]
fn preview_thumb_fallback() {
    let mut item = mk("progressive", "/cache/thumb.webp", true);
    assert_eq!(super::view::preview_image_path(&item), Some(("/cache/thumb.webp", false)));

    item.preview_path = Some(String::from("/cache/full.webp"));
    assert_eq!(super::view::preview_image_path(&item), Some(("/cache/full.webp", true)));
}

#[test]
fn steam_search_request() {
    let mut browser = Browser::new(Source::Steam);
    browser.request.query = "city".into();
    browser.request.steam.kind = "Video".into();
    browser.request.steam.category = "Anime".into();
    browser.request.steam.resolution = "1920 x 1080".into();
    let SearchRequest::Steam(request) = browser.search_request(2, "") else {
        panic!("expected Steam search");
    };
    assert_eq!(request.query, "city");
    assert_eq!(request.query_type, 3);
    assert_eq!(request.trend_days, 7);
    assert_eq!(request.page, 2);
    assert_eq!(request.request_type, "Video");
    assert_eq!(request.category, "Anime");
    assert_eq!(request.resolution, "1920 x 1080");
    assert!(!request.allow_nsfw);

    browser.request.nsfw = true;
    let SearchRequest::Steam(request) = browser.search_request(1, "") else {
        panic!("expected Steam search");
    };
    assert!(request.allow_nsfw);

    browser.request.steam.kind = String::new();
    browser.request.steam.category = String::new();
    browser.request.steam.resolution = String::new();
    let SearchRequest::Steam(request) = browser.search_request(1, "") else {
        panic!("expected Steam search");
    };
    assert!(request.request_type.is_empty());
    assert!(request.category.is_empty());
    assert!(request.resolution.is_empty());
}

#[test]
fn steam_sorting_fallback() {
    let mut browser = Browser::new(Source::Steam);
    browser.request.sorting = "9".into();
    let SearchRequest::Steam(request) = browser.search_request(1, "") else {
        panic!("expected Steam search");
    };
    assert_eq!(request.query_type, 9);
    browser.request.sorting = "21".into();
    let SearchRequest::Steam(request) = browser.search_request(1, "") else {
        panic!("expected Steam search");
    };
    assert_eq!(request.query_type, 21);
    browser.request.steam.trend_days = "3".into();
    browser.request.sorting = "3".into();
    let SearchRequest::Steam(request) = browser.search_request(1, "") else {
        panic!("expected Steam search");
    };
    assert_eq!(request.trend_days, 3);
    browser.request.sorting = "junk".into();
    let SearchRequest::Steam(request) = browser.search_request(1, "") else {
        panic!("expected Steam search");
    };
    assert_eq!(request.query_type, 3);
}

#[test]
fn fmt_size_boundaries() {
    assert_eq!(fmt_size(0), "");
    assert_eq!(fmt_size(1023), "");
    assert_eq!(fmt_size(1024), "1 KB");
    assert_eq!(fmt_size((1 << 20) - 1), "1023 KB");
    assert_eq!(fmt_size(1 << 20), "1.0 MB");
    assert_eq!(fmt_size(3 * (1 << 20) + (1 << 19)), "3.5 MB");
}

#[test]
fn busy_session_signals() {
    let mut browser = Browser::new(Source::Wallhaven);
    assert!(!browser.busy());

    browser.session.loading = true;
    assert!(browser.busy());
    browser.session.loading = false;

    browser.session.items = vec![mk("a", "", false)];
    assert!(browser.thumbs_pending());
    assert!(browser.busy());
    browser.session.items[0].thumb_failed = true;
    assert!(!browser.busy());
    browser.session.items[0].thumb_ready = true;
    browser.session.items[0].thumb_failed = false;
    assert!(!browser.busy());

    browser.session.preview = Some(0);
    assert!(browser.preview_loading());
    assert!(browser.busy());
    browser.session.items[0].preview_path = Some("/c/a-full.jpg".into());
    assert!(!browser.busy());

    browser.session.pending_apply = Some("a".into());
    assert!(browser.transfer_busy());
    assert!(browser.busy());
    browser.session.pending_apply = None;

    browser.session.items[0].downloading = true;
    assert!(browser.transfer_busy());
    assert!(browser.busy());
    browser.session.items[0].downloading = false;
    assert!(!browser.busy());

    let BrowserSessionState {
        submitted_search: _,
        search_generation: _,
        page: _,
        last_page: _,
        next_cursor: _,
        loading: _,
        items,
        hover: _,
        preview: _,
        error: _,
        page_failed: _,
        pending_apply: _,
    } = &browser.session;
    let BrowserItem {
        id: _,
        full_url: _,
        thumb_path: _,
        title: _,
        resolution: _,
        purity: _,
        file_size: _,
        category: _,
        thumb_ready: _,
        downloaded: _,
        downloading: _,
        queued: _,
        progress: _,
        phase: _,
        duration_secs: _,
        downloaded_path: _,
        preview_path: _,
        thumb_failed: _,
        attribution: _,
        attribution_url: _,
        track_url: _,
    } = &items[0];
}

#[test]
fn search_request_defaults() {
    let browser = Browser::new(Source::Wallhaven);
    let SearchRequest::Wallhaven(request) = browser.search_request(1, "") else {
        panic!("expected Wallhaven search");
    };
    assert_eq!(request.top_range, "1M");
    assert!(request.atleast.is_empty());
    assert!(request.ratios.is_empty());
    assert_eq!(request.color_hue, None);
}
