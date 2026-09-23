use super::{
    App, Browser, Message, Pending, Source, browser_item, drain_calls, reject, respond, seed,
    test_app, tick_frames, update, wall,
};
use serde_json::{Value, json};
use std::time::Instant;

fn command(app: &mut App, value: &str) {
    let _ = update(
        app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Command(value.to_owned())),
    );
}

fn state(app: &App) -> Value {
    serde_json::from_str::<Value>(&crate::app::update::ui_state_json(app)).unwrap()["downloads"]
        .clone()
}

fn demo_browser() -> App {
    let mut app = test_app();
    seed(&mut app, &[wall("local.png", "static", 1, 0)]);
    command(&mut app, "demo begin");
    command(&mut app, "browser-demo source wallhaven");
    app.source_browser.browser.as_mut().unwrap().session.items =
        vec![browser_item("first"), browser_item("chosen")];
    drain_calls(&app);
    app
}

#[test]
fn browser_demo_commands_require_an_active_demo() {
    let mut app = test_app();
    seed(&mut app, &[]);
    command(&mut app, "browser-demo source wallhaven");
    assert!(app.source_browser.browser.is_none());
    assert!(drain_calls(&app).is_empty());

    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.items.push(browser_item("chosen"));
    browser.session.preview = Some(0);
    app.source_browser.browser = Some(browser);
    let before = state(&app);
    for value in [
        "browser-demo source bing",
        "browser-demo search forest",
        "browser-demo preview chosen",
        "browser-demo download chosen",
        "browser-demo preview-close",
        "browser-demo close",
    ] {
        command(&mut app, value);
        assert_eq!(state(&app), before, "{value}");
        assert!(!app.source_browser.preview_closing, "{value}");
        assert!(drain_calls(&app).is_empty(), "{value}");
    }
}

#[test]
fn browser_demo_source_uses_the_normal_provider_requests() {
    let mut app = test_app();
    seed(&mut app, &[]);
    app.config.set_key("sources.bing.enabled", json!(true));
    command(&mut app, "demo begin");
    drain_calls(&app);

    command(&mut app, "browser-demo source wallhaven");
    assert_eq!(app.source_browser.browser.as_ref().unwrap().source, Source::Wallhaven);
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, _)| method == "wallhaven.collections"));
    let search = calls.iter().find(|(method, _)| method == "wallhaven.search").unwrap();
    assert_eq!(search.1["page"], 1);
    assert_eq!(search.1["generation"], app.source_browser.search_generation);
    assert_eq!(state(&app)["loading"], true);

    command(&mut app, "browser-demo source bing");
    assert_eq!(app.source_browser.browser.as_ref().unwrap().source, Source::Bing);
    assert!(drain_calls(&app).iter().any(|(method, params)| {
        method == "source.list" && params["source"] == "bing" && params["page"] == 1
    }));
    assert!(app.source_browser.tabs.contains_key(&Source::Wallhaven));
}

#[test]
fn browser_demo_search_submits_the_query_and_rejects_stale_results() {
    let mut app = demo_browser();
    let previous_generation = app.source_browser.search_generation;
    let old_request = app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| {
            matches!(pending, Pending::BrowserSearch { generation, .. } if *generation == previous_generation)
                .then_some(*id)
        })
        .unwrap();
    command(&mut app, "browser-demo search misty coastal forest");
    let generation = app.source_browser.search_generation;
    assert_eq!(generation, previous_generation + 1);
    let calls = drain_calls(&app);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "wallhaven.search");
    assert_eq!(calls[0].1["query"], "misty coastal forest");
    assert_eq!(calls[0].1["page"], 1);
    assert_eq!(calls[0].1["generation"], generation);
    assert_eq!(state(&app)["query"], "misty coastal forest");
    assert_eq!(state(&app)["loading"], true);
    respond(
        &mut app,
        old_request,
        json!({"results": [{"id": "stale"}], "generation": previous_generation}),
    );
    assert_eq!(state(&app)["loading"], true);
    assert_eq!(state(&app)["items"][0]["id"], "first");
    let request = app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| {
            matches!(pending, Pending::BrowserSearch { generation: current, .. } if *current == generation)
                .then_some(*id)
        })
        .unwrap();
    respond(
        &mut app,
        request,
        json!({"results": [{"id": "coast", "full_url": "https://x/coast"}], "generation": generation}),
    );
    let snapshot = state(&app);
    assert_eq!(snapshot["loading"], false);
    assert!(snapshot["error"].is_null());
    assert_eq!(snapshot["items"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["items"][0]["id"], "coast");
}

#[test]
fn browser_demo_preview_resolves_a_stable_id_and_reports_readiness() {
    let mut app = demo_browser();
    command(&mut app, "browser-demo preview chosen");
    assert_eq!(app.source_browser.browser.as_ref().unwrap().session.preview, Some(1));
    assert_eq!(
        drain_calls(&app),
        vec![("wallhaven.preview".into(), json!({"id": "chosen", "full_url": "https://x/chosen"}))]
    );
    let snapshot = state(&app);
    assert_eq!(snapshot["preview"]["id"], "chosen");
    assert_eq!(snapshot["preview"]["ready"], false);
    app.on_event(
        wall_proto::ev::PREVIEW_READY,
        &json!({"id": "chosen", "path": "/missing-demo-preview.png"}),
    );
    assert_eq!(state(&app)["preview"]["ready"], true);
    assert_eq!(state(&app)["preview"]["path"], "/missing-demo-preview.png");
    assert!(app.source_browser.browser.as_ref().unwrap().session.items[0].preview_path.is_none());
}

#[test]
fn browser_demo_download_tracks_real_rpc_and_download_events() {
    let mut app = demo_browser();
    command(&mut app, "browser-demo download chosen");
    assert_eq!(
        drain_calls(&app),
        vec![(
            "wallhaven.download".into(),
            json!({"id": "chosen", "full_url": "https://x/chosen"})
        )]
    );
    assert_eq!(state(&app)["items"][1]["downloading"], true);
    for (status, extra) in [
        ("queued", json!({})),
        ("downloading", json!({"progress": 0.5})),
        ("done", json!({"path": "/downloads/chosen.jpg"})),
    ] {
        let mut event = extra;
        event["id"] = json!("chosen");
        event["status"] = json!(status);
        app.on_event(wall_proto::ev::DOWNLOAD, &event);
        let snapshot = state(&app);
        let item = &snapshot["items"][1];
        assert_eq!(item["queued"], status == "queued");
        assert_eq!(item["downloading"], status != "done");
        assert_eq!(item["downloaded"], status == "done");
        if status == "downloading" {
            assert_eq!(item["progress"], 0.5);
        }
        if status == "done" {
            assert_eq!(item["path"], "/downloads/chosen.jpg");
        }
        assert_eq!(snapshot["items"][0]["downloaded"], false);
    }
    assert!(drain_calls(&app).iter().all(|(method, _)| method != "wall.apply"));
}

#[test]
fn browser_demo_invalid_provider_and_item_ids_do_nothing() {
    let mut app = demo_browser();
    let before = state(&app);
    for value in [
        "browser-demo source missing-provider",
        "browser-demo preview absent",
        "browser-demo preview 0",
        "browser-demo download absent",
        "browser-demo download",
    ] {
        command(&mut app, value);
        assert_eq!(state(&app), before, "{value}");
        assert!(drain_calls(&app).is_empty(), "{value}");
    }
}

#[test]
fn browser_demo_closes_the_preview_before_closing_the_browser() {
    let mut app = demo_browser();
    command(&mut app, "browser-demo preview chosen");
    app.source_browser.preview_animation = 1.0;
    command(&mut app, "browser-demo preview-close");
    assert!(app.source_browser.preview_closing);
    assert!(app.source_browser.browser.is_some());
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    assert!(state(&app)["preview"].is_null());
    assert_eq!(state(&app)["open"], true);
    command(&mut app, "browser-demo close");
    let snapshot = state(&app);
    assert_eq!(snapshot["open"], false);
    assert!(snapshot["query"].is_null());
    assert_eq!(snapshot["items"], json!([]));
}

#[test]
fn browser_demo_exposes_search_failure_and_clears_it_on_retry() {
    let mut app = demo_browser();
    let request = app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| matches!(pending, Pending::BrowserSearch { .. }).then_some(*id))
        .unwrap();
    reject(&mut app, request, "fixture unavailable");
    assert_eq!(state(&app)["loading"], false);
    assert_eq!(state(&app)["error"], "fixture unavailable");
    command(&mut app, "browser-demo search mountains");
    assert_eq!(state(&app)["loading"], true);
    assert!(state(&app)["error"].is_null());
    assert!(drain_calls(&app).iter().any(|(method, params)| {
        method == "wallhaven.search" && params["query"] == "mountains"
    }));
}

#[test]
fn browser_demo_batch_opens_the_provider_before_submitting_its_query() {
    let mut app = test_app();
    seed(&mut app, &[]);
    command(&mut app, "demo begin");
    drain_calls(&app);
    for value in ["browser-demo source wallhaven", "browser-demo search forest"] {
        command(&mut app, &format!("batch-stage downloads {}", json!(value)));
    }
    assert!(app.source_browser.browser.is_none());
    assert!(drain_calls(&app).is_empty());
    command(&mut app, "batch-commit downloads");
    let calls = drain_calls(&app);
    let searches: Vec<_> =
        calls.iter().filter(|(method, _)| method == "wallhaven.search").collect();
    assert_eq!(searches.len(), 2);
    assert_eq!(searches[0].1["query"], "");
    assert_eq!(searches[1].1["query"], "forest");
    assert_eq!(searches[1].1["generation"], app.source_browser.search_generation);
    assert_eq!(state(&app)["source"], "wallhaven");
    assert_eq!(state(&app)["query"], "forest");
    assert_eq!(state(&app)["loading"], true);
}

#[test]
fn workshop_copy_writes_only_the_id_to_the_standard_clipboard() {
    use iced::futures::{StreamExt, executor::block_on};
    let mut app = demo_browser();
    let task = crate::app::update::update_inner(
        &mut app,
        Message::Browser(crate::frontend::browser::BrowserMsg::CopyWorkshopId("3804689861".into())),
    );
    let mut actions = iced_runtime::task::into_stream(task).unwrap();
    let action = block_on(actions.next()).unwrap();
    let iced_runtime::Action::Clipboard(iced_runtime::clipboard::Action::Write {
        target,
        contents,
    }) = action
    else {
        panic!("expected a clipboard write");
    };
    assert_eq!(target, iced::advanced::clipboard::Kind::Standard);
    assert_eq!(contents, "3804689861");
    assert!(block_on(actions.next()).is_none());
    assert!(drain_calls(&app).is_empty());
}
