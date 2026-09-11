use super::*;
use crate::app::helpers::apply_browser_defaults;
use crate::frontend::browser::{Browser, Source};

#[test]
fn source_defaults_reach_requests_without_changing_factory_defaults() {
    let mut app = test_app();
    for source in [Source::Wallhaven, Source::Steam] {
        let mut browser = Browser::new(source);
        let expected = browser.search_request(1, "");
        apply_browser_defaults(&mut browser, &app.config);
        assert_eq!(browser.search_request(1, ""), expected);
    }
    for (path, value) in [
        ("wallhaven.defaults.sort", json!("hot")),
        ("wallhaven.defaults.atleast", json!("3840x2160")),
        ("wallhaven.defaults.ratios", json!("21x9")),
        ("wallhaven.defaults.anime", json!(false)),
        ("steam.defaults.type", json!("Scene")),
        ("steam.defaults.category", json!("Nature")),
        ("steam.defaults.resolution", json!("3840 x 2160")),
        ("steam.defaults.trendDays", json!("3")),
    ] {
        app.config.set_key(path, value);
    }
    let mut browser = Browser::new(Source::Wallhaven);
    apply_browser_defaults(&mut browser, &app.config);
    let crate::contracts::browser::SearchRequest::Wallhaven(request) =
        browser.search_request(1, "")
    else {
        panic!("Wallhaven request expected")
    };
    assert_eq!(request.sorting, "hot");
    assert_eq!(request.atleast, "3840x2160");
    assert_eq!(request.ratios, "21x9");
    assert_eq!(request.categories, [true, false, true]);
    let mut browser = Browser::new(Source::Steam);
    apply_browser_defaults(&mut browser, &app.config);
    let crate::contracts::browser::SearchRequest::Steam(request) = browser.search_request(1, "")
    else {
        panic!("Steam request expected")
    };
    assert_eq!(request.request_type, "Scene");
    assert_eq!(request.category, "Nature");
    assert_eq!(request.resolution, "3840 x 2160");
    assert_eq!(request.trend_days, 3);
}

#[test]
fn browser_defaults_apply_once_per_session_and_again_after_closing() {
    let mut app = test_app();
    app.config.set_key("steam.defaults.type", json!("Scene"));
    app.source_browser.activate(Source::Steam);
    app.run_browser_search(false);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().request.steam.kind, "Scene");
    app.source_browser.browser.as_mut().unwrap().request.steam.kind = "Video".into();
    app.source_browser.activate(Source::Wallhaven);
    app.run_browser_search(false);
    app.source_browser.activate(Source::Steam);
    app.run_browser_search(false);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().request.steam.kind, "Video");
    app.source_browser.close();
    app.source_browser.activate(Source::Steam);
    app.run_browser_search(false);
    assert_eq!(app.source_browser.browser.as_ref().unwrap().request.steam.kind, "Scene");
}

#[test]
fn invalid_source_defaults_fall_back_to_searchable_values() {
    let mut app = test_app();
    for path in [
        "wallhaven.defaults.sort",
        "wallhaven.defaults.atleast",
        "steam.defaults.type",
        "steam.defaults.trendDays",
    ] {
        app.config.set_key(path, json!("invalid"));
    }
    for path in [
        "wallhaven.defaults.general",
        "wallhaven.defaults.anime",
        "wallhaven.defaults.people",
        "wallhaven.defaults.sfw",
    ] {
        app.config.set_key(path, json!(false));
    }
    let mut browser = Browser::new(Source::Wallhaven);
    apply_browser_defaults(&mut browser, &app.config);
    assert_eq!(browser.request.sorting, "toplist");
    assert!(browser.request.wallhaven.atleast.is_empty());
    assert!(browser.request.wallhaven.general);
    assert!(browser.request.wallhaven.sfw);
    let mut browser = Browser::new(Source::Steam);
    apply_browser_defaults(&mut browser, &app.config);
    assert!(browser.request.steam.kind.is_empty());
    assert_eq!(browser.request.steam.trend_days, "7");
}

#[test]
fn filter_edits_wait_for_one_submitted_request() {
    use crate::frontend::browser::BrowserMsg;
    let mut app = test_app();
    app.config.set_key("sources.wallhaven.showApplyButton", json!(true));
    app.source_browser.activate(Source::Wallhaven);
    app.run_browser_search(false);
    drain_calls(&app);
    let generation = app.source_browser.search_generation;
    for message in [
        BrowserMsg::ToggleCategory(1),
        BrowserMsg::TogglePurity(1),
        BrowserMsg::SetSort("hot".into()),
        BrowserMsg::SetTopRange("1w".into()),
        BrowserMsg::SetAtleast("1920x1080".into()),
        BrowserMsg::SetAtmost("3840x2160".into()),
        BrowserMsg::SetRatios("16x9".into()),
        BrowserMsg::SetColor(3),
        BrowserMsg::SetCollection("collection".into()),
        BrowserMsg::SearchInput("mountains".into()),
    ] {
        let _ = update(&mut app, Message::Browser(message));
        assert!(drain_calls(&app).is_empty());
    }
    assert_eq!(app.source_browser.search_generation, generation);
    let _ = update(&mut app, Message::Browser(BrowserMsg::SearchSubmit));
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "wallhaven.search");
    let params = &calls[0].1;
    assert_eq!(params["categories"], "101");
    assert_eq!(params["purity"], "110");
    assert_eq!(params["sorting"], "hot");
    assert_eq!(params["topRange"], "1w");
    assert_eq!(params["atleast"], "1920x1080");
    assert_eq!(params["atmost"], "3840x2160");
    assert_eq!(params["ratios"], "16x9");
    assert_eq!(params["collection"], "collection");
    assert_eq!(params["query"], "mountains");
    assert_eq!(params["page"], 1);
    assert_eq!(app.source_browser.search_generation, generation + 1);
}

#[test]
fn hiding_apply_restores_immediate_searching() {
    use crate::frontend::browser::BrowserMsg;
    let mut app = test_app();
    assert!(!app.config.browser_apply_button(Source::Wallhaven));
    app.source_browser.activate(Source::Wallhaven);
    app.run_browser_search(false);
    drain_calls(&app);
    let _ = update(&mut app, Message::Browser(BrowserMsg::ToggleCategory(1)));
    assert_eq!(drain_calls(&app).len(), 1);
    app.config.set_key("sources.wallhaven.showApplyButton", json!(true));
    let _ = update(&mut app, Message::Browser(BrowserMsg::ToggleCategory(2)));
    assert!(drain_calls(&app).is_empty());
    app.config.set_key("sources.wallhaven.showApplyButton", json!(false));
    let _ = update(&mut app, Message::Browser(BrowserMsg::SetSort("hot".into())));
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].1["categories"], "100");
}

#[test]
fn pagination_keeps_submitted_filters_until_apply() {
    use crate::frontend::browser::BrowserMsg;
    let mut app = test_app();
    app.config.set_key("sources.wallhaven.showApplyButton", json!(true));
    for (source, message) in [
        (Source::Wallhaven, BrowserMsg::ToggleCategory(1)),
        (Source::Steam, BrowserMsg::SteamFilter("type".into(), "Scene".into())),
        (Source::Unsplash, BrowserMsg::CatalogFilter("unsplash-color".into(), "blue".into())),
        (Source::Pexels, BrowserMsg::CatalogFilter("pexels-size".into(), "large".into())),
        (Source::Youtube, BrowserMsg::SetMaxDuration("120".into())),
    ] {
        app.config.set_key(&format!("sources.{}.showApplyButton", source.key()), json!(true));
        app.source_browser.close();
        app.source_browser.activate(source);
        drain_calls(&app);
        app.run_browser_search(false);
        let mut original = drain_calls(&app);
        let _ = update(&mut app, Message::Browser(message));
        assert!(drain_calls(&app).is_empty(), "{source:?}");
        app.source_browser.browser.as_mut().unwrap().session.page = 1;
        app.run_browser_search(true);
        original[0].1["page"] = json!(2);
        assert_eq!(drain_calls(&app), original, "{source:?}");
        let _ = update(&mut app, Message::Browser(BrowserMsg::SearchSubmit));
        let applied = drain_calls(&app);
        assert_eq!(applied.len(), 1);
        original[0].1["page"] = json!(1);
        original[0].1["generation"] = applied[0].1["generation"].clone();
        assert_ne!(applied, original, "{source:?}");
    }
}

#[test]
fn source_switch_preserves_unapplied_edits_and_submitted_pages() {
    use crate::frontend::browser::BrowserMsg;
    let mut app = test_app();
    app.config.set_key("sources.wallhaven.showApplyButton", json!(true));
    app.source_browser.activate(Source::Wallhaven);
    app.run_browser_search(false);
    let mut initial = drain_calls(&app);
    initial.retain(|(method, _)| method == "wallhaven.search");
    let _ = update(&mut app, Message::Browser(BrowserMsg::ToggleCategory(1)));
    app.source_browser.activate(Source::Steam);
    app.run_browser_search(false);
    drain_calls(&app);
    app.source_browser.activate(Source::Wallhaven);
    assert!(!app.source_browser.browser.as_ref().unwrap().request.wallhaven.anime);
    app.source_browser.browser.as_mut().unwrap().session.page = 1;
    app.run_browser_search(true);
    initial[0].1["page"] = json!(2);
    assert_eq!(drain_calls(&app), initial);
}

#[test]
fn automatic_retry_does_not_submit_pending_filters() {
    use crate::frontend::browser::BrowserMsg;
    let mut app = test_app();
    app.config.set_key("sources.wallhaven.showApplyButton", json!(true));
    app.source_browser.activate(Source::Wallhaven);
    drain_calls(&app);
    app.run_browser_search(false);
    let mut original = drain_calls(&app);
    let _ = update(&mut app, Message::Browser(BrowserMsg::ToggleCategory(1)));
    app.run_browser_search(false);
    let retried = drain_calls(&app);
    original[0].1["generation"] = retried[0].1["generation"].clone();
    assert_eq!(original, retried);
    assert!(!app.source_browser.browser.as_ref().unwrap().request.wallhaven.anime);
}

#[test]
fn each_source_chooses_when_to_submit_filters() {
    use crate::frontend::browser::BrowserMsg;
    let mut app = test_app();
    app.config.set_key("sources.wallhaven.showApplyButton", json!(true));
    app.config.set_key("sources.steam.showApplyButton", json!(false));
    app.source_browser.activate(Source::Wallhaven);
    app.run_browser_search(false);
    drain_calls(&app);
    let _ = update(&mut app, Message::Browser(BrowserMsg::ToggleCategory(1)));
    assert!(drain_calls(&app).is_empty());
    app.source_browser.activate(Source::Steam);
    app.run_browser_search(false);
    drain_calls(&app);
    let _ =
        update(&mut app, Message::Browser(BrowserMsg::SteamFilter("type".into(), "Scene".into())));
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "steam.search");
    assert_eq!(calls[0].1["tags"], json!(["Scene"]));
    app.source_browser.activate(Source::Wallhaven);
    let _ = update(&mut app, Message::Browser(BrowserMsg::TogglePurity(1)));
    assert!(drain_calls(&app).is_empty());
    assert!(app.config.browser_apply_button(Source::Wallhaven));
    assert!(!app.config.browser_apply_button(Source::Steam));
    assert!(!app.config.browser_apply_button(Source::Bing));
    let _ = update(&mut app, Message::Browser(BrowserMsg::SearchSubmit));
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].1["categories"], "101");
    assert_eq!(calls[0].1["purity"], "110");
}
