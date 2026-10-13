use super::*;
use crate::frontend::effects::{EffectsMode, EffectsMsg};

fn open(app: &mut App) -> u64 {
    crate::app::helpers::open_effects_without_preview(app, 0, EffectsMode::Displays);
    app.panels.effects.as_ref().unwrap().activity.list_request
}

fn output_request(app: &App) -> u64 {
    app.panels.effects.as_ref().unwrap().activity.output_request
}

fn activities() -> Value {
    json!({
        "supported": true,
        "current": "work",
        "activities": [
            {"id": "work", "name": "Work", "current": true},
            {"id": "rest", "name": "Rest", "current": false}
        ]
    })
}

fn output(path: &str) -> Value {
    json!({"outputs": [{"name": "DP-1", "current": path, "path": path, "type": "static"}]})
}

fn ready_video_apply(app: &mut App) -> String {
    seed(app, &[wall("video.mp4", "video", 1, 2), wall("still.png", "static", 2, 1)]);
    app.library_session.filters.sort = "date".into();
    app.refilter();
    let list = open(app);
    let key = app.panels.effects.as_ref().unwrap().source_key().to_string();
    respond(app, list, activities());
    let request = output_request(app);
    respond(app, request, output("/wp/live.png"));
    let _ = update(app, Message::Effects(EffectsMsg::MonitorToggle("DP-1".into())));
    let _ = drain_calls(app);
    key
}

#[test]
fn activity_apply_tracks_source() {
    let mut app = test_app();
    let key = ready_video_apply(&mut app);
    let mut catalog = app.library_session.library.catalog().clone();
    let mut newer = catalog.items[1].clone();
    newer.key = "new.png".into();
    newer.name = "new.png".into();
    newer.path = "/wp/new.png".into();
    newer.mtime = 3;
    catalog.items.push(newer);
    app.on_list(catalog);
    let current = app.library_session.filtered[0] as usize;
    assert_ne!(app.library_session.library.catalog().items[current].key, key);
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
    assert!(drain_calls(&app).iter().any(|(method, params)| {
        method == "wall.apply" && params["type"] == "video" && params["path"] == "/vids/video.mp4"
    }));
}

#[test]
fn activity_apply_missing_source() {
    let mut app = test_app();
    let key = ready_video_apply(&mut app);
    let mut catalog = app.library_session.library.catalog().clone();
    catalog.items.retain(|item| item.key != key);
    app.on_list(catalog);
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
    assert!(app.daemon.activity_applies.is_empty());
    assert_eq!(app.runtime_state.toast.as_ref().unwrap().0, "Apply failed");
}

#[test]
fn activity_selection_scopes_apply() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    app.on_outputs(crate::contracts::daemon::OutputsResult {
        outputs: vec![crate::contracts::daemon::OutputStatus {
            name: "DP-1".into(),
            current: "/wp/live.png".into(),
            ..Default::default()
        }],
    });
    let live_art = app.daemon.current_wallpaper_art.clone();
    let list = open(&mut app);
    let _ = drain_calls(&app);
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
    respond(&mut app, list, activities());
    let current = output_request(&app);
    assert_eq!(app.panels.effects.as_ref().unwrap().activity.selected, "work");
    respond(&mut app, current, output("/wp/work.png"));
    let _ = update(&mut app, Message::Effects(EffectsMsg::ActivitySelect("rest".into())));
    let inactive = output_request(&app);
    let calls = drain_calls(&app);
    assert!(
        calls
            .iter()
            .any(|(method, params)| { method == "wall.outputs" && params["activity"] == "rest" })
    );
    assert!(!app.panels.effects.as_ref().unwrap().activity.ready);
    assert!(app.panels.effects.as_ref().unwrap().monitors().is_empty());
    respond(&mut app, inactive, output("/wp/rest.png"));
    let effects = app.panels.effects.as_ref().unwrap();
    assert!(effects.activity.ready);
    assert_eq!(effects.monitors()[0].current, "/wp/rest.png");
    assert_eq!(app.daemon.current_wallpaper_art, live_art);
    assert_eq!(app.daemon.output_statuses[0].current, "/wp/live.png");
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorToggle("DP-1".into())));
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
    let calls = drain_calls(&app);
    assert!(calls.iter().any(|(method, params)| {
        method == "wall.apply" && params["activity"] == "rest" && params["output"] == "DP-1"
    }));
    let apply = *app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| matches!(pending, Pending::ActivityApply { .. }).then_some(id))
        .unwrap();
    respond(
        &mut app,
        apply,
        json!({"applied": "/wp/a.png", "activity": "rest", "visible": false, "queued": false}),
    );
    assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("Rest"));
}

#[test]
fn activity_replies_stay_selected() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    let list = open(&mut app);
    respond(&mut app, list, activities());
    let old_current = output_request(&app);
    let _ = update(&mut app, Message::Effects(EffectsMsg::ActivitySelect("rest".into())));
    let old_inactive = output_request(&app);
    let _ = update(&mut app, Message::Effects(EffectsMsg::ActivitySelect("work".into())));
    let current = output_request(&app);
    respond(&mut app, current, output("/wp/work.png"));
    respond(&mut app, old_inactive, output("/wp/rest.png"));
    reject(&mut app, old_current, "old request failed");
    let effects = app.panels.effects.as_ref().unwrap();
    assert_eq!(effects.activity.selected, "work");
    assert_eq!(effects.monitors()[0].current, "/wp/work.png");
    assert!(effects.activity.error.is_none());
    let stale_list = open(&mut app);
    let fresh_list = open(&mut app);
    respond(&mut app, stale_list, activities());
    assert!(!app.panels.effects.as_ref().unwrap().activity.ready);
    respond(&mut app, fresh_list, activities());
    assert_eq!(app.panels.effects.as_ref().unwrap().activity.selected, "work");
}

#[test]
fn activity_errors_block_apply() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    let list = open(&mut app);
    respond(&mut app, list, activities());
    let _ = update(&mut app, Message::Effects(EffectsMsg::ActivitySelect("rest".into())));
    let request = output_request(&app);
    reject(&mut app, request, "Activity was removed");
    assert!(!app.panels.effects.as_ref().unwrap().activity.can_apply());
    let _ = drain_calls(&app);
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
    assert!(!drain_calls(&app).iter().any(|(method, _)| method == "wall.apply"));
}

#[test]
fn activity_apply_rejection_surfaces() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    let list = open(&mut app);
    respond(&mut app, list, activities());
    let _ = update(&mut app, Message::Effects(EffectsMsg::ActivitySelect("rest".into())));
    let request = output_request(&app);
    respond(&mut app, request, output("/wp/rest.png"));
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorToggle("DP-1".into())));
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
    let apply = *app
        .daemon
        .pending
        .iter()
        .find_map(|(id, pending)| matches!(pending, Pending::ActivityApply { .. }).then_some(id))
        .unwrap();
    reject(&mut app, apply, "Activity was removed");
    let toast = &app.runtime_state.toast.as_ref().unwrap().0;
    assert!(toast.contains("Activity was removed"));
    assert!(!toast.contains("Saved for"));
}

#[test]
fn non_plasma_keeps_outputs() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    let list = open(&mut app);
    respond(&mut app, list, json!({"supported": false, "current": "", "activities": []}));
    app.on_outputs(
        crate::infrastructure::rpc_results::decode_outputs(&output("/wp/live.png")).unwrap(),
    );
    let effects = app.panels.effects.as_ref().unwrap();
    assert!(!effects.activity.supported);
    assert!(effects.activity.can_apply());
    assert_eq!(effects.monitors()[0].current, "/wp/live.png");
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorToggle("DP-1".into())));
    let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
    assert!(
        drain_calls(&app)
            .iter()
            .any(|(method, params)| { method == "wall.apply" && params.get("activity").is_none() })
    );
}

#[test]
fn activity_refresh_keeps_selection() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.png", "static", 1, 0)]);
    let list = open(&mut app);
    respond(&mut app, list, activities());
    let _ = update(&mut app, Message::Effects(EffectsMsg::ActivitySelect("rest".into())));
    let stale = output_request(&app);
    app.on_event(wall_proto::ev::OUTPUTS_CHANGED, &json!({}));
    let list = app.panels.effects.as_ref().unwrap().activity.list_request;
    respond(&mut app, stale, output("/wp/stale.png"));
    assert!(!app.panels.effects.as_ref().unwrap().activity.can_apply());
    let mut changed = activities();
    changed["current"] = json!("rest");
    changed["activities"][0]["current"] = json!(false);
    changed["activities"][1]["current"] = json!(true);
    respond(&mut app, list, changed);
    assert_eq!(app.panels.effects.as_ref().unwrap().activity.selected, "rest");
    assert_eq!(app.panels.effects.as_ref().unwrap().activity.current, "rest");
    let request = output_request(&app);
    respond(&mut app, request, output("/wp/rest.png"));
    assert!(app.panels.effects.as_ref().unwrap().activity.can_apply());
    assert!(!app.panels.effects.as_ref().unwrap().activity.inactive());
    app.on_event(wall_proto::ev::OUTPUTS_CHANGED, &json!({}));
    let list = app.panels.effects.as_ref().unwrap().activity.list_request;
    let mut removed = activities();
    removed["activities"].as_array_mut().unwrap().pop();
    respond(&mut app, list, removed);
    let effects = app.panels.effects.as_ref().unwrap();
    assert_eq!(effects.activity.selected, "rest");
    assert!(!effects.activity.can_apply());
    assert!(effects.monitors().is_empty());
}

#[test]
fn activity_batch_waits() {
    for fail in [false, true] {
        let mut app = test_app();
        seed(&mut app, &[wall("a.png", "static", 1, 0)]);
        let list = open(&mut app);
        respond(&mut app, list, activities());
        let _ = update(&mut app, Message::Effects(EffectsMsg::ActivitySelect("rest".into())));
        let request = output_request(&app);
        respond(&mut app, request, json!({"outputs": [{"name": "DP-1"}, {"name": "DP-2"}]}));
        let _ = drain_calls(&app);
        let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorToggleAll));
        let _ = update(&mut app, Message::Effects(EffectsMsg::MonitorApplySelected));
        let targets: Vec<_> = drain_calls(&app)
            .into_iter()
            .filter(|(method, _)| method == "wall.apply")
            .map(|(_, params)| params["output"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(targets, ["DP-1", "DP-2"]);
        let mut replies: Vec<_> = app
            .daemon
            .pending
            .iter()
            .filter_map(|(id, kind)| matches!(kind, Pending::ActivityApply { .. }).then_some(*id))
            .collect();
        replies.sort_unstable();
        respond(
            &mut app,
            replies[0],
            json!({"activity": "rest", "visible": false, "queued": false}),
        );
        assert!(app.runtime_state.toast.is_none());
        if fail {
            respond(&mut app, replies[1], json!({"activity": "rest"}));
            assert!(!app.runtime_state.toast.as_ref().unwrap().0.contains("Saved for"));
        } else {
            respond(
                &mut app,
                replies[1],
                json!({"activity": "rest", "visible": false, "queued": false}),
            );
            assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("Saved for Rest"));
        }
        assert!(app.daemon.activity_applies.is_empty());
    }
}

fn apply_pair(app: &mut App, activity: &str) -> Vec<(u64, String, String)> {
    let list = open(app);
    respond(app, list, activities());
    let _ = update(app, Message::Effects(EffectsMsg::ActivitySelect(activity.into())));
    let request = output_request(app);
    respond(app, request, json!({"outputs": [{"name": "DP-1"}, {"name": "DP-2"}]}));
    let _ = update(app, Message::Effects(EffectsMsg::MonitorToggleAll));
    let _ = update(app, Message::Effects(EffectsMsg::MonitorApplySelected));
    let mut requests: Vec<_> = app
        .daemon
        .activity_requests
        .iter()
        .filter(|(_, request)| request.batch == app.daemon.next_activity_apply)
        .map(|(token, request)| (request.id, token.clone(), request.output.clone()))
        .collect();
    requests.sort_by_key(|request| request.0);
    requests
}

fn apply_event(request: &(u64, String, String), activity: &str) -> Value {
    json!({"request_id": request.0, "apply_token": request.1, "output": request.2,
        "activity": activity, "visible": true, "ok": true})
}

fn queue_replies(app: &mut App, requests: &[(u64, String, String)], activity: &str) {
    for request in requests {
        respond(app, request.0, json!({"activity": activity, "visible": false, "queued": true}));
    }
}

#[test]
fn queued_current_pair() {
    for early in [false, true] {
        let mut app = test_app();
        seed(&mut app, &[wall("a.mp4", "video", 1, 0)]);
        let requests = apply_pair(&mut app, "work");
        assert_eq!(requests.len(), 2);
        let mut foreign = apply_event(&requests[0], "work");
        foreign["apply_token"] = json!("other-client");
        app.on_event(wall_proto::ev::APPLY_RESULT, &foreign);
        assert_eq!(app.daemon.activity_requests.len(), 2);
        if early {
            app.on_event(wall_proto::ev::APPLY_RESULT, &apply_event(&requests[0], "work"));
        }
        queue_replies(&mut app, &requests, "work");
        assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("Preparing"));
        app.on_event(
            wall_proto::ev::APPLY_RESULT,
            &json!({"request_id": requests[usize::from(early)].0, "ok": false}),
        );
        assert_eq!(app.runtime_state.toast.as_ref().unwrap().0, "Apply failed");
        assert_eq!(app.daemon.activity_requests.len(), if early { 1 } else { 2 });
        for request in &requests {
            app.on_event(wall_proto::ev::APPLY_RESULT, &apply_event(request, "work"));
        }
        assert!(app.daemon.activity_requests.is_empty());
        assert!(app.daemon.activity_applies.is_empty());
        assert_eq!(app.runtime_state.toast.as_ref().unwrap().0, "Apply failed");
        queue_replies(&mut app, &requests, "work");
        assert_eq!(app.runtime_state.toast.as_ref().unwrap().0, "Apply failed");
    }
}

#[test]
fn queued_ordinary_failure_surfaces() {
    for matching in [false, true] {
        let mut app = test_app();
        seed(&mut app, &[wall("a.mp4", "video", 1, 0)]);
        let requests = apply_pair(&mut app, "work");
        queue_replies(&mut app, &requests, "work");
        let id = if matching { requests[0].0 } else { requests[1].0 + 1 };
        app.on_event(
            wall_proto::ev::APPLY_RESULT,
            &json!({"request_id": id, "ok": false, "detail": "ordinary apply failed"}),
        );
        assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("ordinary apply failed"));
        assert_eq!(app.daemon.activity_requests.len(), 2);
        assert_eq!(app.daemon.activity_applies.len(), 1);
    }
}

#[test]
fn queued_failure_deletion() {
    for early in [false, true] {
        let mut app = test_app();
        seed(&mut app, &[wall("a.mp4", "video", 1, 0)]);
        let requests = apply_pair(&mut app, "rest");
        let mut failed = apply_event(&requests[1], "rest");
        failed["ok"] = json!(false);
        failed["visible"] = json!(false);
        failed["detail"] = json!("Activity was removed");
        if early {
            app.on_event(wall_proto::ev::APPLY_RESULT, &failed);
        }
        queue_replies(&mut app, &requests, "rest");
        app.on_event(wall_proto::ev::APPLY_RESULT, &apply_event(&requests[0], "rest"));
        if !early {
            assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("Preparing"));
            app.on_event(wall_proto::ev::APPLY_RESULT, &failed);
        }
        assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("Activity was removed"));
        assert!(!app.runtime_state.toast.as_ref().unwrap().0.contains("Saved"));
        assert!(app.daemon.activity_applies.is_empty());
        assert!(app.daemon.activity_requests.is_empty());
    }
}

#[test]
fn queued_scope_cancel() {
    for outcome in ["cancelled", "superseded"] {
        let mut app = test_app();
        seed(&mut app, &[wall("a.mp4", "video", 1, 0)]);
        let requests = apply_pair(&mut app, "rest");
        queue_replies(&mut app, &requests, "rest");
        for (field, wrong) in
            [("request_id", json!(0)), ("activity", json!("work")), ("output", json!("DP-3"))]
        {
            let mut event = apply_event(&requests[0], "rest");
            event[field] = wrong;
            app.on_event(wall_proto::ev::APPLY_RESULT, &event);
            assert_eq!(app.daemon.activity_requests.len(), 2);
        }
        let mut cancelled = apply_event(&requests[0], "rest");
        cancelled["ok"] = json!(false);
        cancelled[outcome] = json!(true);
        app.on_event(wall_proto::ev::APPLY_RESULT, &cancelled);
        app.on_event(wall_proto::ev::APPLY_RESULT, &apply_event(&requests[1], "rest"));
        assert!(app.daemon.activity_applies.is_empty());
        assert!(app.daemon.activity_requests.is_empty());
        assert!(app.runtime_state.toast.is_none());
    }
}

#[test]
fn queued_batches_stay_latest() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.mp4", "video", 1, 0)]);
    let old = apply_pair(&mut app, "rest");
    queue_replies(&mut app, &old, "rest");
    let latest = apply_pair(&mut app, "work");
    queue_replies(&mut app, &latest, "work");
    let preparing = app.runtime_state.toast.as_ref().unwrap().0.clone();
    for request in old {
        let mut failed = apply_event(&request, "rest");
        failed["ok"] = json!(false);
        failed["detail"] = json!("old failure");
        app.on_event(wall_proto::ev::APPLY_RESULT, &failed);
    }
    assert_eq!(app.runtime_state.toast.as_ref().unwrap().0, preparing);
    app.handle_ipc(IpcMsg::Disconnected);
    assert!(app.daemon.activity_requests.is_empty());
    assert!(app.daemon.activity_applies.is_empty());
    assert!(!app.runtime_state.toast.as_ref().unwrap().0.contains("Saved"));
}

#[test]
fn queued_inactive_saved() {
    let mut app = test_app();
    seed(&mut app, &[wall("a.mp4", "video", 1, 0)]);
    let requests = apply_pair(&mut app, "rest");
    queue_replies(&mut app, &requests, "rest");
    for (idx, request) in requests.iter().enumerate() {
        let mut saved = apply_event(request, "rest");
        saved["visible"] = json!(false);
        app.on_event(wall_proto::ev::APPLY_RESULT, &saved);
        if idx == 0 {
            assert!(app.runtime_state.toast.as_ref().unwrap().0.contains("Preparing"));
        }
    }
    assert_eq!(app.runtime_state.toast.as_ref().unwrap().0, "Saved for Rest.");
    assert!(app.daemon.activity_requests.is_empty());
    assert!(app.daemon.activity_applies.is_empty());
}
