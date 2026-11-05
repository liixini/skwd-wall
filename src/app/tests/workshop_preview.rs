use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::tests::{browser_item, drain_calls, test_app};
use crate::app::{Message, Pending, update};
use crate::contracts::browser::{
    PreviewFrame, PreviewFrames, PreviewIdentity, PreviewSize, Source, WorkshopDetails,
    WorkshopPreview,
};
use crate::frontend::browser::{Browser, BrowserMsg};

fn preview_app() -> crate::app::App {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Steam);
    browser.session.items.push(browser_item("3819668918"));
    browser.session.items.push(browser_item("3798696286"));
    app.source_browser.browser = Some(browser);
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(0)));
    drain_calls(&app);
    app
}

fn thumbnail_frame() -> PreviewFrames {
    PreviewFrames {
        frames: vec![PreviewFrame {
            width: 1,
            height: 1,
            rgba: Arc::from(vec![255, 0, 0, 255]),
            delay: Duration::ZERO,
        }],
        duration: Duration::ZERO,
    }
}

#[test]
fn closes_animated_preview() {
    let mut app = preview_app();
    let key = app.source_browser.media.key.clone().unwrap();
    let generation = app.source_browser.media.generation;
    let delay = Duration::from_millis(100);
    let frames = [vec![255, 0, 0, 255], vec![0, 255, 0, 255]]
        .into_iter()
        .map(|rgba| PreviewFrame { width: 1, height: 1, rgba: Arc::from(rgba), delay })
        .collect();
    app.on_browser_preview_decoded(
        &key,
        generation,
        PreviewSize::Hero,
        Ok(PreviewFrames { frames, duration: delay * 2 }),
    );
    let started = app.source_browser.media.started.unwrap();
    app.sync_browser_media(started + delay);
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert_eq!(browser.view.media.frames, 2);
    assert_eq!(browser.view.media.frame, 1);
    let _ = update(&mut app, Message::Browser(BrowserMsg::ClosePreview));
    assert!(app.source_browser.media.handles.is_empty());
    assert!(app.source_browser.media.key.is_none());
}

#[test]
fn retries_terminal_failure() {
    let mut app = preview_app();
    let key = app.source_browser.media.key.clone().unwrap();
    app.rpc_error(
        Pending::BrowserPreview { source: key.source, id: key.id.clone(), url: key.url.clone() },
        "network failed".to_owned(),
    );
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(!browser.preview_loading());
    assert_eq!(browser.view.media.error.as_deref(), Some("network failed"));
    let _ = update(&mut app, Message::Browser(BrowserMsg::RetryPreview));
    let calls = drain_calls(&app);
    assert_eq!(calls.iter().filter(|(method, _)| method == "steam.preview").count(), 1);
    assert_eq!(
        calls.iter().find(|(method, _)| method == "steam.preview").unwrap().1["refresh"],
        true
    );
    assert!(app.source_browser.browser.as_ref().unwrap().view.media.error.is_none());
}

#[test]
fn rejects_stale_decode() {
    let mut app = preview_app();
    let old = app.source_browser.media.key.clone().unwrap();
    let generation = app.source_browser.media.generation;
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(1)));
    app.on_browser_preview_decoded(&old, generation, PreviewSize::Hero, Err("old failure".into()));
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert_eq!(browser.view.media.item_id, "3798696286");
    assert!(browser.view.media.error.is_none());
    assert!(browser.view.media.loading);
}

#[test]
fn hover_waits_before_fetch() {
    let mut app = preview_app();
    let _ = update(&mut app, Message::Browser(BrowserMsg::ClosePreview));
    app.source_browser.preview_closing = false;
    let browser = app.source_browser.browser.as_mut().unwrap();
    browser.session.preview = None;
    browser.session.hover = Some(1);
    let now = Instant::now();
    app.sync_browser_media(now);
    assert!(drain_calls(&app).is_empty());
    app.sync_browser_media(now + Duration::from_millis(180));
    assert_eq!(drain_calls(&app).iter().filter(|(method, _)| method == "steam.preview").count(), 1);
    app.source_browser.browser.as_mut().unwrap().session.hover = None;
    app.sync_browser_media(now + Duration::from_millis(180));
    assert!(app.source_browser.media.key.is_none());
}

#[test]
fn refetches_evicted_preview() {
    let mut app = preview_app();
    let key = app.source_browser.media.key.clone().unwrap();
    app.source_browser.media.requests.remove(&key);
    app.source_browser.media.key = None;
    let browser = app.source_browser.browser.as_mut().unwrap();
    let item = &mut browser.session.items[0];
    item.preview_paths.insert(item.full_url.clone(), "/missing/evicted-preview.jpg".into());
    item.preview_path = Some("/missing/evicted-preview.jpg".into());
    app.sync_browser_media(Instant::now());
    assert_eq!(drain_calls(&app).iter().filter(|(method, _)| method == "steam.preview").count(), 1);
    assert!(!app.source_browser.media.decode_busy);
}

#[test]
fn reports_gallery_cache() {
    let mut app = preview_app();
    let browser = app.source_browser.browser.as_mut().unwrap();
    browser.view.media.url = "https://example.com/gallery.png".into();
    browser.session.items[0]
        .preview_paths
        .insert(browser.view.media.url.clone(), "/cache/gallery.png".into());
    let state: serde_json::Value =
        serde_json::from_str(&crate::app::update::ui_state_json(&app)).unwrap();
    assert_eq!(state["downloads"]["preview"]["path"], "/cache/gallery.png");
}

#[test]
fn caches_thumbnail_bytes() {
    let mut app = preview_app();
    let key = PreviewIdentity {
        source: Source::Steam,
        id: "3819668918".into(),
        url: "https://images.steamusercontent.com/gallery".into(),
    };
    app.source_browser.browser.as_mut().unwrap().session.items[0].workshop =
        Some(WorkshopDetails {
            previews: vec![WorkshopPreview::Image { url: key.url.clone() }],
            ..WorkshopDetails::default()
        });
    app.on_browser_preview_decoded(
        &key,
        app.source_browser.media.thumbnail_generation,
        PreviewSize::Thumbnail,
        Ok(thumbnail_frame()),
    );
    let item = &app.source_browser.browser.as_ref().unwrap().session.items[0];
    assert!(item.preview_handles.contains_key(&key.url));
    app.clear_browser_previews();
    assert!(
        app.source_browser.browser.as_ref().unwrap().session.items[0].preview_handles.is_empty()
    );
}

#[test]
fn prefetches_associated_media() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Steam);
    let mut item = browser_item("3819668918");
    item.workshop = Some(WorkshopDetails {
        previews: vec![
            WorkshopPreview::Image { url: String::from("https://images.steamusercontent.com/a") },
            WorkshopPreview::Youtube { id: String::from("PEZIDtt1ceA") },
        ],
        ..WorkshopDetails::default()
    });
    browser.session.items.push(item);
    app.source_browser.browser = Some(browser);
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(0)));
    let calls = drain_calls(&app);
    let urls: std::collections::HashSet<_> = calls
        .iter()
        .filter(|(method, _)| method == "steam.preview")
        .filter_map(|(_, params)| params["full_url"].as_str())
        .collect();
    assert_eq!(urls.len(), 2);
    assert!(urls.contains("https://x/3819668918"));
    assert!(urls.contains("https://images.steamusercontent.com/a"));
    assert!(!urls.contains("https://i.ytimg.com/vi/PEZIDtt1ceA/hqdefault.jpg"));
    assert!(calls.iter().all(|(method, _)| method != "source.details"));
}

#[test]
fn paces_large_gallery() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Steam);
    let mut item = browser_item("3819668918");
    let mut previews = (0..40)
        .map(|index| WorkshopPreview::Image {
            url: format!("https://images.steamusercontent.com/{index}"),
        })
        .collect::<Vec<_>>();
    let duplicate = previews[0].clone();
    previews.push(duplicate);
    item.workshop = Some(WorkshopDetails { previews, ..WorkshopDetails::default() });
    browser.session.items.push(item);
    app.source_browser.browser = Some(browser);
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(0)));

    let mut calls = drain_calls(&app);
    let mut seen = std::collections::HashSet::new();
    while !calls.is_empty() {
        assert!(calls.len() <= 2);
        for (_, params) in calls {
            let url = params["full_url"].as_str().unwrap().to_owned();
            assert!(seen.insert(url.clone()));
            app.fail_browser_preview(
                &PreviewIdentity { source: Source::Steam, id: "3819668918".into(), url },
                "network failed".into(),
            );
        }
        calls = drain_calls(&app);
    }
    assert_eq!(seen.len(), 41);
    app.request_workshop_media();
    assert!(drain_calls(&app).is_empty());
}

#[test]
fn bounds_thumbnail_queue() {
    let mut app = preview_app();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("poster.png");
    image::RgbaImage::new(1, 1).save(&path).unwrap();
    let path = path.to_string_lossy().into_owned();
    let item = &mut app.source_browser.browser.as_mut().unwrap().session.items[0];
    let previews = (0..200)
        .map(|index| {
            let url = format!("https://images.steamusercontent.com/{index}");
            item.preview_paths.insert(url.clone(), path.clone());
            WorkshopPreview::Image { url }
        })
        .collect();
    item.workshop = Some(WorkshopDetails { previews, ..WorkshopDetails::default() });
    app.request_workshop_media();
    assert_eq!(app.source_browser.media.thumbnail_queued.len(), 128);
    assert!(drain_calls(&app).is_empty());
}

#[test]
fn rejects_stale_thumbnail() {
    let mut app = preview_app();
    let key = PreviewIdentity {
        source: Source::Steam,
        id: "3819668918".into(),
        url: "https://images.steamusercontent.com/gallery".into(),
    };
    app.source_browser.browser.as_mut().unwrap().session.items[0].workshop =
        Some(WorkshopDetails {
            previews: vec![WorkshopPreview::Image { url: key.url.clone() }],
            ..WorkshopDetails::default()
        });
    let stale = app.source_browser.media.thumbnail_generation;
    app.clear_browser_previews();
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(0)));
    app.source_browser.media.thumbnail_queued.insert(key.clone());
    app.on_browser_preview_decoded(&key, stale, PreviewSize::Thumbnail, Ok(thumbnail_frame()));
    app.on_browser_preview_decoded(&key, stale, PreviewSize::Thumbnail, Err("old failure".into()));
    assert!(app.source_browser.media.thumbnail_queued.contains(&key));
    assert!(!app.source_browser.media.failed.contains(&key));
    assert!(
        app.source_browser.browser.as_ref().unwrap().session.items[0].preview_handles.is_empty()
    );
}

#[test]
fn keeps_thumbnail_on_selection() {
    let mut app = preview_app();
    let key = PreviewIdentity {
        source: Source::Steam,
        id: "3819668918".into(),
        url: "https://images.steamusercontent.com/gallery".into(),
    };
    app.source_browser.browser.as_mut().unwrap().session.items[0].workshop =
        Some(WorkshopDetails {
            previews: vec![WorkshopPreview::Image { url: key.url.clone() }],
            ..WorkshopDetails::default()
        });
    let epoch = app.source_browser.media.thumbnail_generation;
    app.source_browser.media.thumbnail_queued.insert(key.clone());
    let _ = update(&mut app, Message::Browser(BrowserMsg::SelectPreview(1)));
    assert_eq!(app.source_browser.media.thumbnail_generation, epoch);
    app.on_browser_preview_decoded(&key, epoch, PreviewSize::Thumbnail, Ok(thumbnail_frame()));
    assert!(
        app.source_browser.browser.as_ref().unwrap().session.items[0]
            .preview_handles
            .contains_key(&key.url)
    );
    assert!(!app.source_browser.media.thumbnail_queued.contains(&key));
}

#[test]
fn prioritizes_new_hero() {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Steam);
    let mut first = browser_item("first");
    first.workshop = Some(WorkshopDetails {
        previews: vec![WorkshopPreview::Image {
            url: "https://images.steamusercontent.com/poster".into(),
        }],
        ..WorkshopDetails::default()
    });
    browser.session.items.push(first);
    browser.session.items.push(browser_item("second"));
    app.source_browser.browser = Some(browser);
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(0)));
    assert_eq!(drain_calls(&app).iter().filter(|(method, _)| method == "steam.preview").count(), 2);
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(1)));
    let calls = drain_calls(&app);
    assert!(
        calls.iter().any(|(method, params)| method == "steam.preview"
            && params["full_url"] == "https://x/second")
    );
    assert_eq!(app.source_browser.media.requests.len(), 3);
}

#[test]
fn evicts_for_selected_thumbnail() {
    let mut app = preview_app();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("poster.png");
    image::RgbaImage::new(1, 1).save(&path).unwrap();
    let path = path.to_string_lossy().into_owned();
    let mut previews = Vec::new();
    for index in 0..129 {
        let url = format!("https://images.steamusercontent.com/{index}");
        previews.push(WorkshopPreview::Image { url: url.clone() });
        let key =
            PreviewIdentity { source: Source::Steam, id: "3819668918".into(), url: url.clone() };
        if index < 128 {
            app.source_browser.browser.as_mut().unwrap().session.items[0]
                .preview_handles
                .insert(url, iced::widget::image::Handle::from_path(path.clone()));
            app.source_browser.media.thumbnail_done.insert(key.clone());
            app.source_browser.media.thumbnail_order.push_back(key);
        } else {
            app.source_browser.browser.as_mut().unwrap().session.items[0]
                .preview_paths
                .insert(url, path.clone());
        }
    }
    app.source_browser.browser.as_mut().unwrap().session.items[0].workshop =
        Some(WorkshopDetails { previews, ..WorkshopDetails::default() });
    let _ = update(&mut app, Message::Browser(BrowserMsg::SelectPreview(129)));
    let key = app.source_browser.media.key.clone().unwrap();
    assert!(app.source_browser.media.thumbnail_queued.contains(&key));
    app.on_browser_preview_decoded(
        &key,
        app.source_browser.media.thumbnail_generation,
        PreviewSize::Thumbnail,
        Ok(thumbnail_frame()),
    );
    let item = &app.source_browser.browser.as_ref().unwrap().session.items[0];
    assert_eq!(item.preview_handles.len(), 128);
    assert!(item.preview_handles.contains_key(&key.url));
    assert!(!item.preview_handles.contains_key("https://images.steamusercontent.com/0"));
}

#[test]
fn stepping_keeps_entrance() {
    let mut app = preview_app();
    app.source_browser.preview_animation = 1.0;
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(1)));
    assert_eq!(app.source_browser.preview_animation, 1.0);
    app.source_browser.browser.as_mut().expect("browser").session.preview = None;
    let _ = update(&mut app, Message::Browser(BrowserMsg::OpenPreview(0)));
    assert!(app.source_browser.preview_animation < 1.0);
}

fn card_app(preview: Option<&std::path::Path>) -> crate::app::App {
    let mut app = test_app();
    let mut browser = Browser::new(Source::Steam);
    let mut item = browser_item("3754377004");
    item.preview_path = preview.map(|path| path.display().to_string());
    browser.session.items.push(item);
    app.source_browser.browser = Some(browser);
    Arc::make_mut(&mut app.source_browser.wall.scene.render).hits.push(
        crate::frontend::scene::layout::Hit {
            index: 0,
            cx: 100.0,
            cy: 100.0,
            hw: 50.0,
            hh: 50.0,
            skew: 0.0,
            edge_tilt: 0.0,
            hex: false,
            hex_shape: crate::frontend::scene::layout::HexShape::Hexagon,
            triangle_direction: 0,
            quad: None,
        },
    );
    app
}

fn card_key() -> PreviewIdentity {
    PreviewIdentity {
        source: Source::Steam,
        id: String::from("3754377004"),
        url: String::from("https://x/3754377004"),
    }
}

fn animated_frames() -> PreviewFrames {
    let frame = thumbnail_frame().frames.remove(0);
    let delay = Duration::from_millis(100);
    PreviewFrames {
        frames: vec![PreviewFrame { delay, ..frame.clone() }, PreviewFrame { delay, ..frame }],
        duration: delay * 2,
    }
}

#[test]
fn animates_visible_cards() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("preview.jpg");
    std::fs::write(&path, b"gif").expect("write");
    let mut app = card_app(Some(&path));
    app.sync_browser_media(Instant::now());
    let key = card_key();
    assert!(app.source_browser.media.cards.seen.contains(&key.url));

    let generation = app.source_browser.media.cards.generation;
    app.on_browser_preview_decoded(&key, generation, PreviewSize::Card, Ok(animated_frames()));
    app.sync_browser_media(Instant::now());
    let browser = app.source_browser.browser.as_ref().expect("browser");
    assert!(browser.view.animations.contains_key(&key.url));
    assert!(app.animating());
}

#[test]
fn hidden_cards_idle() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("preview.jpg");
    std::fs::write(&path, b"gif").expect("write");
    let mut app = card_app(Some(&path));
    app.sync_browser_media(Instant::now());
    let generation = app.source_browser.media.cards.generation;
    app.on_browser_preview_decoded(
        &card_key(),
        generation,
        PreviewSize::Card,
        Ok(animated_frames()),
    );
    Arc::make_mut(&mut app.source_browser.wall.scene.render).hits.clear();
    app.sync_browser_media(Instant::now());
    assert!(!app.source_browser.media.cards.visible);
}

#[test]
fn still_cards_stay_static() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("preview.jpg");
    std::fs::write(&path, b"jpeg").expect("write");
    let mut app = card_app(Some(&path));
    app.sync_browser_media(Instant::now());
    let still = PreviewFrames { frames: Vec::new(), duration: Duration::ZERO };
    let generation = app.source_browser.media.cards.generation;
    app.on_browser_preview_decoded(&card_key(), generation, PreviewSize::Card, Ok(still));
    app.sync_browser_media(Instant::now());
    let browser = app.source_browser.browser.as_ref().expect("browser");
    assert!(browser.view.animations.is_empty());
    assert!(app.source_browser.media.cards.queue.is_empty());
}

#[test]
fn fetches_card_previews() {
    let mut app = card_app(None);
    app.sync_browser_media(Instant::now());
    let url = card_key().url;
    assert!(app.source_browser.media.cards.requests.contains_key(&url));
    app.adopt_browser_preview(&card_key(), "/nonexistent/preview.jpg", None);
    assert!(!app.source_browser.media.cards.requests.contains_key(&url));
    assert!(app.source_browser.media.cards.seen.contains(&url));
}

#[test]
fn failed_cards_not_retried() {
    let mut app = card_app(None);
    app.sync_browser_media(Instant::now());
    drain_calls(&app);
    app.fail_browser_preview(&card_key(), String::from("offline"));
    app.sync_browser_media(Instant::now());
    assert!(drain_calls(&app).is_empty());
    assert!(app.source_browser.media.cards.requests.is_empty());
}

#[test]
fn new_search_reanimates() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("preview.jpg");
    std::fs::write(&path, b"gif").expect("write");
    let mut app = card_app(Some(&path));
    app.sync_browser_media(Instant::now());
    let old = app.source_browser.media.cards.generation;
    app.source_browser.browser.as_mut().expect("browser").session.search_generation += 1;
    app.sync_browser_media(Instant::now());
    assert_ne!(app.source_browser.media.cards.generation, old);
    assert!(app.source_browser.media.cards.seen.contains(&card_key().url));
    app.on_browser_preview_decoded(&card_key(), old, PreviewSize::Card, Ok(animated_frames()));
    let browser = app.source_browser.browser.as_ref().expect("browser");
    assert!(browser.view.animations.is_empty());
}

#[test]
fn leaving_steam_frees_cards() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("preview.jpg");
    std::fs::write(&path, b"gif").expect("write");
    let mut app = card_app(Some(&path));
    app.sync_browser_media(Instant::now());
    let generation = app.source_browser.media.cards.generation;
    app.on_browser_preview_decoded(
        &card_key(),
        generation,
        PreviewSize::Card,
        Ok(animated_frames()),
    );
    app.source_browser.activate(Source::Wallhaven);
    app.sync_browser_media(Instant::now());
    assert!(app.source_browser.media.cards.idle());
    assert!(!app.source_browser.media.cards.visible);
    let steam = app.source_browser.tabs.get(&Source::Steam).expect("steam tab");
    assert!(steam.view.animations.is_empty());
}
