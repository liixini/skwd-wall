#![cfg(test)]

#[test]
fn initial_surface_overlay() {
    let shell = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell/shell.rs"),
    )
    .expect("read shell.rs");
    assert!(!shell.contains("StartMode::Background"));
    assert!(
        shell.contains("exclusive_zone: -1")
            && shell.contains("KeyboardInteractivity::Exclusive")
            && shell.contains("size: Some((0, 0))")
    );
    let warm = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/warm.rs"),
    )
    .expect("read warm.rs");
    assert!(!warm.contains("layershell_open"));
}

#[test]
fn hide_exits_cold() {
    let warm = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/warm.rs"),
    )
    .expect("read warm.rs");
    assert!(warm.contains("crate::hard_exit(0)"));
    assert!(warm.contains("fn exit_picker(app: &mut App) -> !"));
    assert!(!warm.contains("fn hide(") && !warm.contains("fn show("));
    assert!(
        !warm.contains("keep_warm") && !warm.contains("warm_forever") && !warm.contains("IdleExit")
    );
    let update = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/app/update/mod.rs"),
    )
    .expect("read update.rs");
    assert!(!update.contains("crate::hard_exit"));
}

#[test]
fn wake_channel_via_app_state() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in [
        "src/app/warm.rs",
        "src/app/runtime/semantic.rs",
        "src/app/helpers/effects.rs",
        "src/app/update/browser.rs",
        "src/app/runtime/events.rs",
        "src/app/helpers/selection.rs",
    ] {
        let source = std::fs::read_to_string(root.join(file)).expect("read wake consumer");
        assert!(!source.contains("wake_sender"), "{file}");
    }
    let shell = std::fs::read_to_string(root.join("src/shell/shell.rs")).expect("read shell.rs");
    assert!(shell.contains("wake_sender"));
}

#[test]
fn late_output_activates_profile() {
    use serde_json::json;
    use skwd_config::{keys::selector, picker_layout::Size};

    let mut app = crate::app::tests::test_app();
    app.scene.viewport = (600.0, 800.0);
    let viewport = Size { width: 600.0, height: 800.0 };
    app.config.select_layout_output(Some(String::from("HEADLESS-2")), viewport);
    app.config
        .set_keys(&[
            (selector::FRAME_ENABLED, json!(true)),
            (selector::FRAME_X, json!(8)),
            (selector::FRAME_Y, json!(71)),
            (selector::FRAME_WIDTH, json!(84)),
            (selector::FRAME_HEIGHT, json!(23)),
        ])
        .unwrap();
    app.config.select_layout_output(None, viewport);
    app.snap_layout();
    let saved = app.config.document().clone();
    super::sync_output(&mut app, None);
    assert!(app.config.picker_frame().is_none());
    super::sync_output(&mut app, Some(String::from("HEADLESS-2")));
    assert_eq!(app.config.layout_output(), Some("HEADLESS-2"));
    assert_eq!(app.runtime_state.picker_output.as_deref(), Some("HEADLESS-2"));
    assert_eq!(app.config.picker_frame(), Some([0.08, 0.71, 0.84, 0.23]));
    let frame = crate::app::update::arrange::picker_rect(&app);
    for (actual, expected) in
        [frame.x, frame.y, frame.width, frame.height].into_iter().zip([48.0, 568.0, 504.0, 184.0])
    {
        assert!((actual - expected).abs() < 0.001);
    }
    let render = app.scene.render.clone();
    super::sync_output(&mut app, Some(String::from("HEADLESS-2")));
    assert!(std::sync::Arc::ptr_eq(&render, &app.scene.render));
    assert_eq!(app.config.document(), &saved);
    super::sync_output(&mut app, Some(String::from("HEADLESS-1")));
    assert_eq!(app.config.layout_output(), Some("HEADLESS-1"));
    assert_eq!(app.config.picker_frame(), Some([0.08, 0.71, 0.84, 0.23]));
    assert_eq!(app.config.document(), &saved);
}

#[test]
fn draft_output_stays_bound() {
    let mut app = crate::app::tests::test_app();
    app.scene.viewport = (600.0, 800.0);
    super::sync_output(&mut app, Some(String::from("HEADLESS-1")));
    let _ = crate::app::update(
        &mut app,
        crate::app::Message::Arrange(crate::frontend::arrange::ArrangeMsg::Begin),
    );
    let before = app.config.document().clone();
    super::sync_output(&mut app, Some(String::from("HEADLESS-2")));
    assert_eq!(app.config.layout_output(), Some("HEADLESS-1"));
    assert_eq!(app.runtime_state.picker_output.as_deref(), Some("HEADLESS-1"));
    assert_eq!(app.config.document(), &before);
}
