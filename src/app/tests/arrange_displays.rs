use super::*;
use crate::frontend::arrange::{ArrangeMsg, Pointer};
use skwd_config::keys::selector;
use skwd_config::picker_layout::Size;

fn begin() -> App {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    app.config.set_screen_width(1280.0);
    app.runtime_state.picker_output = Some(String::from("DP-1"));
    app.config
        .select_layout_output(Some(String::from("DP-1")), Size { width: 1280.0, height: 720.0 });
    app.config
        .set_keys(&[
            (selector::FRAME_ENABLED, json!(true)),
            (selector::FRAME_X, json!(10)),
            (selector::FRAME_Y, json!(20)),
            (selector::FRAME_WIDTH, json!(80)),
            (selector::FRAME_HEIGHT, json!(60)),
        ])
        .unwrap();
    app.daemon.output_statuses = ["DP-1", "DP-2"]
        .into_iter()
        .map(|name| crate::contracts::daemon::OutputStatus {
            name: name.to_owned(),
            target: name.to_owned(),
            connected: true,
            width: 1280,
            height: 720,
            logical_width: 1280,
            logical_height: 720,
            ..crate::contracts::daemon::OutputStatus::default()
        })
        .collect();
    app.apply_layout();
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Begin));
    settle(&mut app);
    app
}

fn settle(app: &mut App) {
    for _ in 0..240 {
        crate::app::update::arrange::tick(app, 1.0 / 60.0);
        if app.panels.arrange.is_none() {
            return;
        }
    }
}

fn edit(app: &mut App, message: ArrangeMsg) {
    let _ = update(app, Message::Arrange(message));
    refresh_arrange_outputs(app);
    settle(app);
}

#[test]
fn review_keeps_live_state() {
    use crate::app::update::arrange::displays;

    let mut app = begin();
    let before = app.config.document().clone();
    let render = app.scene.render.clone();
    let _ = drain_calls(&app);
    displays::update(&mut app, ArrangeMsg::OpenDisplays);
    displays::update(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    displays::update(&mut app, ArrangeMsg::FitDisplay);
    assert_eq!(app.config.document(), &before);
    assert!(std::sync::Arc::ptr_eq(&render, &app.scene.render));
    assert!(drain_calls(&app).is_empty());
}

#[test]
fn apply_stages_recipient() {
    let mut app = begin();
    let original = app.config.document().clone();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    assert!(app.config.is_transient());
    assert_eq!(app.config.layout_outputs(), ["DP-1", "DP-2"]);
    assert_eq!(
        app.config.document()["pickerLayouts"]["outputs"]["DP-1"],
        original["pickerLayouts"]["outputs"]["DP-1"]
    );
    edit(&mut app, ArrangeMsg::Cancel);
    assert_eq!(app.config.document(), &original);
}

#[test]
fn done_retains_pending_review() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    edit(&mut app, ArrangeMsg::Done);
    assert!(app.config.is_transient());
    assert!(app.panels.arrange.as_ref().unwrap().display_copy.as_ref().unwrap().review.is_some());
    edit(&mut app, ArrangeMsg::CloseDisplays);
    assert!(app.panels.arrange.as_ref().unwrap().display_copy.is_some());
    edit(&mut app, ArrangeMsg::SkipDisplay);
    edit(&mut app, ArrangeMsg::CloseDisplays);
    assert!(app.panels.arrange.as_ref().unwrap().display_copy.is_none());
}

#[test]
fn disconnect_requires_consent() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    app.daemon.output_statuses[1].connected = false;
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    assert_eq!(app.config.layout_outputs(), ["DP-1"]);
    let review = &app
        .panels
        .arrange
        .as_ref()
        .unwrap()
        .display_copy
        .as_ref()
        .unwrap()
        .review
        .as_ref()
        .unwrap()
        .view;
    assert!(!review.connected);
    edit(&mut app, ArrangeMsg::AllowDisconnected(true));
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    assert_eq!(app.config.layout_outputs(), ["DP-1", "DP-2"]);
}

#[test]
fn unknown_size_blocks_review() {
    let mut app = begin();
    app.daemon.output_statuses[1].width = 0;
    app.daemon.output_statuses[1].height = 0;
    app.daemon.output_statuses[1].logical_width = 0;
    app.daemon.output_statuses[1].logical_height = 0;
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    let copy = app.panels.arrange.as_ref().unwrap().display_copy.as_ref().unwrap();
    assert!(!copy.recipients[0].reviewable);
    assert!(copy.review.is_none());
}

#[test]
fn projection_matches_live_controls() {
    let mut app = begin();
    app.snap_layout();
    let profile = app.config.layout_profile("DP-1", Size { width: 1280.0, height: 720.0 }).unwrap();
    let config = app.config.project_layout("DP-1", &profile).unwrap();
    let projected = crate::app::view::layout::LayoutContext::projected(&config, app.scene.viewport);
    let current = crate::app::view::layout::LayoutContext::current(&app, app.scene.viewport);
    assert_eq!(
        crate::app::view::bar_bounds(&app, &projected),
        crate::app::view::bar_bounds(&app, &current)
    );
    assert_eq!(
        crate::app::view::tag_cloud_bounds(&app, &projected),
        crate::app::view::tag_cloud_bounds(&app, &current)
    );
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    assert!(
        app.panels
            .arrange
            .as_ref()
            .unwrap()
            .display_copy
            .as_ref()
            .unwrap()
            .review
            .as_ref()
            .unwrap()
            .view
            .before
            .picker
            .is_none()
    );
}

#[test]
fn oversized_candidate_needs_fit() {
    use skwd_config::picker_layout::{Alignment, Component, Offset, Position, Reference};
    let mut app = begin();
    let far = Position {
        reference: Reference::Screen,
        alignment: Alignment::TopLeft,
        offset: Offset { x: 1100.0, y: 600.0 },
    };
    app.config.set_position(Component::Search, Some(far)).unwrap();
    let small = &mut app.daemon.output_statuses[1];
    (small.width, small.height, small.logical_width, small.logical_height) = (800, 450, 800, 450);
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    assert!(
        !app.panels
            .arrange
            .as_ref()
            .unwrap()
            .display_copy
            .as_ref()
            .unwrap()
            .review
            .as_ref()
            .unwrap()
            .view
            .compatible
    );
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    assert_eq!(app.config.layout_outputs(), ["DP-1"]);
    edit(&mut app, ArrangeMsg::FitDisplay);
    assert!(
        app.panels
            .arrange
            .as_ref()
            .unwrap()
            .display_copy
            .as_ref()
            .unwrap()
            .review
            .as_ref()
            .unwrap()
            .view
            .compatible
    );
}

#[test]
fn staged_disconnect_blocks_done() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    edit(&mut app, ArrangeMsg::CloseDisplays);
    app.daemon.output_statuses[1].connected = false;
    edit(&mut app, ArrangeMsg::Done);
    assert_eq!(app.panels.arrange.as_ref().unwrap().disconnected_outputs, ["DP-2"]);
    edit(&mut app, ArrangeMsg::ConfirmDisconnected);
    edit(&mut app, ArrangeMsg::Done);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn resized_recipient_needs_review() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    app.daemon.output_statuses[1].logical_width = 1000;
    edit(&mut app, ArrangeMsg::Done);
    assert!(app.panels.arrange.is_some());
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    assert_eq!(
        app.panels
            .arrange
            .as_ref()
            .unwrap()
            .display_copy
            .as_ref()
            .unwrap()
            .review
            .as_ref()
            .unwrap()
            .view
            .after
            .viewport
            .0,
        1000.0
    );
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    edit(&mut app, ArrangeMsg::Done);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn unknown_inventory_blocks_save() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    edit(&mut app, ArrangeMsg::ApplyDisplay);
    app.daemon.output_statuses.clear();
    edit(&mut app, ArrangeMsg::Done);
    assert!(app.panels.arrange.is_some());
    assert!(app.panels.arrange.as_ref().unwrap().disconnected_outputs.is_empty());
}

#[test]
fn resized_source_needs_preview() {
    let mut app = begin();
    app.config.set_key(selector::FRAME_X, json!(15));
    app.daemon.output_statuses[0].logical_width = 1000;
    app.scene.viewport.0 = 1000.0;
    edit(&mut app, ArrangeMsg::Done);
    assert!(app.panels.arrange.is_some());
    assert!(
        app.runtime_state
            .toast
            .as_ref()
            .unwrap()
            .0
            .contains(crate::i18n::tr("arrange-source-resized"))
    );
    edit(&mut app, ArrangeMsg::Preview);
    assert!(app.panels.arrange.as_ref().unwrap().preview_config.is_some());
    edit(&mut app, ArrangeMsg::Preview);
    edit(&mut app, ArrangeMsg::Done);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn done_refreshes_inventory() {
    let mut app = begin();
    app.config.set_key(selector::FRAME_X, json!(15));
    app.daemon.output_statuses[0].logical_width = 640;
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    assert_ne!(request, 0);
    assert!(app.config.is_transient());
    let mut fresh = app.daemon.output_statuses.clone();
    fresh[0].logical_width = 1280;
    respond(&mut app, request, output_inventory(&fresh));
    let _ = crate::app::update::arrange::inventory::resume(&mut app);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn review_refreshes_recipient() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::ReviewDisplay(String::from("DP-2"))));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    let mut fresh = app.daemon.output_statuses.clone();
    fresh[1].logical_width = 1000;
    respond(&mut app, request, output_inventory(&fresh));
    let _ = crate::app::update::arrange::inventory::resume(&mut app);
    let review = app
        .panels
        .arrange
        .as_ref()
        .unwrap()
        .display_copy
        .as_ref()
        .unwrap()
        .review
        .as_ref()
        .unwrap();
    assert_eq!(review.view.after.viewport, (1000.0, 720.0));
}

#[test]
fn editing_cancels_pending_save() {
    use iced::keyboard::{Key, Modifiers, key::Named};
    let mut app = begin();
    let _ = update(&mut app, Message::KeyPressed(Key::Named(Named::Tab), Modifiers::empty()));
    app.config.set_key(selector::FRAME_X, json!(15));
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    let _ = update(&mut app, Message::KeyPressed(Key::Named(Named::ArrowRight), Modifiers::SHIFT));
    let fresh = app.daemon.output_statuses.clone();
    respond(&mut app, request, output_inventory(&fresh));
    let _ = crate::app::update::arrange::inventory::resume(&mut app);
    assert!(app.panels.arrange.is_some());
    assert!(app.config.is_transient());
    edit(&mut app, ArrangeMsg::Done);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn release_keeps_pending_save() {
    let mut app = begin();
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    let _ = crate::app::update::update_inner(&mut app, Message::PointerUp);
    let release = ArrangeMsg::Pointer(Pointer::Release([1.0, 1.0]));
    let _ = update(&mut app, Message::Arrange(release));
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Pointer(Pointer::Move([0.0, 0.0]))));
    assert_eq!(app.panels.arrange.as_ref().unwrap().outputs.request, request);
    let fresh = app.daemon.output_statuses.clone();
    respond(&mut app, request, output_inventory(&fresh));
    let _ = crate::app::update::arrange::inventory::resume(&mut app);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn navigation_keeps_pending_save() {
    let mut app = begin();
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    for pointer in [Pointer::Move([20.0, 20.0]), Pointer::Move([400.0, 300.0]), Pointer::Leave] {
        let _ = update(&mut app, Message::Arrange(ArrangeMsg::Pointer(pointer)));
        assert_eq!(app.panels.arrange.as_ref().unwrap().outputs.request, request);
    }
    let fresh = app.daemon.output_statuses.clone();
    respond(&mut app, request, output_inventory(&fresh));
    let _ = crate::app::update::arrange::inventory::resume(&mut app);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn canceled_reply_stays_canceled() {
    let mut app = begin();
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    edit(&mut app, ArrangeMsg::Cancel);
    edit(&mut app, ArrangeMsg::Begin);
    let fresh = app.daemon.output_statuses.clone();
    respond(&mut app, request, output_inventory(&fresh));
    let _ = crate::app::update::arrange::inventory::resume(&mut app);
    assert!(app.panels.arrange.is_some());
    assert!(app.config.is_transient());
}

#[test]
fn failed_inventory_is_retryable() {
    let mut app = begin();
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    reject(&mut app, request, "offline");
    assert_eq!(app.panels.arrange.as_ref().unwrap().outputs.request, 0);
    assert!(app.config.is_transient());
    edit(&mut app, ArrangeMsg::Done);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
}

#[test]
fn changed_draft_cancels_save() {
    let mut app = begin();
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Done));
    assert_eq!(app.panels.arrange.as_ref().unwrap().outputs.request, request);
    app.config.set_key(selector::FRAME_X, json!(15));
    let fresh = app.daemon.output_statuses.clone();
    respond(&mut app, request, output_inventory(&fresh));
    let _ = crate::app::update::arrange::inventory::resume(&mut app);
    assert!(app.config.is_transient());
    assert!(app.panels.arrange.is_some());
}

#[test]
fn escape_closes_display_list() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    assert!(app.panels.arrange.as_ref().unwrap().display_copy.is_some());
    edit(&mut app, ArrangeMsg::Escape);
    assert!(app.panels.arrange.as_ref().unwrap().display_copy.is_none());
}

#[test]
fn escape_preserves_pending_review() {
    let mut app = begin();
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    edit(&mut app, ArrangeMsg::Escape);
    let copy = app.panels.arrange.as_ref().unwrap().display_copy.as_ref().unwrap();
    assert_eq!(copy.review.as_ref().unwrap().view.output, "DP-2");
}

#[test]
fn preset_opens_display_review() {
    let mut app = begin();
    let _ = crate::app::update(&mut app, Message::Arrange(ArrangeMsg::OpenDisplays));
    let session = app.panels.arrange.as_ref().unwrap();
    assert_ne!(session.outputs.request, 0);
    assert_eq!(session.outputs.document.as_ref(), Some(app.config.document()));
    let reply = IpcMsg::Response {
        id: session.outputs.request,
        result: Some(output_inventory(&app.daemon.output_statuses)),
        error: None,
    };
    let _ = crate::app::update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Ipc(reply)),
    );
    assert!(app.panels.arrange.as_ref().unwrap().display_copy.is_some());
}

#[test]
fn idle_copy_invalidates_scene() {
    use std::time::{Duration, Instant};

    let mut app = begin();
    let start = Instant::now().checked_sub(Duration::from_secs(4)).expect("valid test deadline");
    app.runtime_state.last_tick = Some(start);
    for frame in 1..=240 {
        app.run_tick(start + Duration::from_secs_f32(frame as f32 / 60.0));
    }
    assert_eq!(app.scene.frame_demand(), crate::app::scene::FrameDemand::Idle);
    let render = app.scene.render.clone();
    let _ = crate::app::update(&mut app, Message::Arrange(ArrangeMsg::OpenDisplays));
    let request = app.panels.arrange.as_ref().unwrap().outputs.request;
    let reply = IpcMsg::Response {
        id: request,
        result: Some(output_inventory(&app.daemon.output_statuses)),
        error: None,
    };
    let _ = crate::app::update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Ipc(reply)),
    );
    assert!(app.panels.arrange.as_ref().unwrap().display_copy.is_some());
    assert!(
        !std::sync::Arc::ptr_eq(&render, &app.scene.render)
            || app.scene.frame_demand() != crate::app::scene::FrameDemand::Idle
    );
    app.run_tick(Instant::now());
    assert_eq!(app.scene.frame_demand(), crate::app::scene::FrameDemand::Idle);
}

#[test]
fn live_output_beats_remembered_name() {
    let mut app = begin();
    app.daemon.output_statuses.push(crate::contracts::daemon::OutputStatus {
        name: String::from("DP-2"),
        target: String::from("@monitor:remembered"),
        connected: false,
        ..crate::contracts::daemon::OutputStatus::default()
    });
    edit(&mut app, ArrangeMsg::OpenDisplays);
    let copy = app.panels.arrange.as_ref().unwrap().display_copy.as_ref().unwrap();
    let recipient = copy.recipients.iter().find(|recipient| recipient.name == "DP-2").unwrap();
    assert!(recipient.connected);
    assert!(recipient.reviewable);
}

fn monitors() -> App {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    app.config.set_screen_width(1280.0);
    let viewport = Size { width: 1280.0, height: 720.0 };
    for (output, x) in [("Old 4", 5), ("DP-2", 15)] {
        app.config.select_layout_output(Some(String::from(output)), viewport);
        app.config
            .set_keys(&[(selector::FRAME_ENABLED, json!(true)), (selector::FRAME_X, json!(x))])
            .unwrap();
    }
    app.daemon.output_statuses =
        [("DP-1", "Alpha 1", true), ("DP-2", "Beta 2", true), ("DP-9", "Gamma 3", false)]
            .into_iter()
            .map(|(name, monitor, connected)| crate::contracts::daemon::OutputStatus {
                name: name.to_owned(),
                monitor: monitor.to_owned(),
                target: name.to_owned(),
                connected,
                width: 1280,
                height: 720,
                logical_width: 1280,
                logical_height: 720,
                ..crate::contracts::daemon::OutputStatus::default()
            })
            .collect();
    app.runtime_state.picker_output = Some(String::from("DP-1"));
    crate::app::update::arrange::placement::select(&mut app, Some(String::from("DP-1")));
    app.apply_layout();
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Begin));
    settle(&mut app);
    edit(&mut app, ArrangeMsg::OpenDisplays);
    app
}

fn listed(app: &App) -> Vec<(String, bool)> {
    let copy = app.panels.arrange.as_ref().unwrap().display_copy.as_ref().unwrap();
    copy.recipients.iter().map(|recipient| (recipient.name.clone(), recipient.connected)).collect()
}

#[test]
fn recipients_follow_monitors() {
    let app = monitors();
    assert_eq!(app.config.layout_output(), Some("Alpha 1"));
    assert_eq!(listed(&app), [(String::from("Beta 2"), true), (String::from("Old 4"), false)]);
}

#[test]
fn forget_drops_disconnected_profile() {
    let mut app = monitors();
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("Old 4")));
    edit(&mut app, ArrangeMsg::ForgetDisplay(String::from("Old 4")));
    assert_eq!(app.config.layout_outputs(), ["DP-2"]);
    assert_eq!(listed(&app), [(String::from("Beta 2"), true)]);
    let copy = app.panels.arrange.as_ref().unwrap().display_copy.as_ref().unwrap();
    assert!(copy.review.is_none());
    for adopted in ["DP-2", "Beta 2"] {
        edit(&mut app, ArrangeMsg::ForgetDisplay(String::from(adopted)));
        assert_eq!(app.config.layout_outputs(), ["DP-2"]);
    }
    edit(&mut app, ArrangeMsg::Undo);
    assert_eq!(app.config.layout_outputs(), ["DP-2", "Old 4"]);
    assert_eq!(listed(&app), [(String::from("Beta 2"), true), (String::from("Old 4"), false)]);
    edit(&mut app, ArrangeMsg::Cancel);
    assert_eq!(app.config.layout_outputs(), ["DP-2", "Old 4"]);
}
