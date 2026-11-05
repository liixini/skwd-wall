use super::*;
use crate::infrastructure::runtime::Wake;

fn shown_app() -> App {
    let mut app = test_app();
    let id = iced::window::Id::unique();
    let _ = update(&mut app, Message::WindowOpened { id, width: 1280.0, height: 720.0 });
    app
}

fn cmd(app: &mut App, command: &str) {
    let _ = update(app, Message::Daemon(Wake::Command(command.to_string())));
}

#[test]
fn stash_toggles_and_keeps_settings() {
    let mut app = shown_app();
    let _ = update(&mut app, Message::ToggleSettings);
    assert!(app.panels.settings.open);

    cmd(&mut app, "stash");
    assert!(app.runtime_state.stashed);
    assert!(app.panels.settings.open);

    cmd(&mut app, "stash");
    assert!(!app.runtime_state.stashed);
    assert!(app.panels.settings.open);
}

#[test]
fn stashed_overlay_schedules_no_frames() {
    let mut app = shown_app();
    let _ = update(&mut app, Message::Stash);
    let before = app.runtime_state.last_tick;

    app.runtime_state.next_animation_tick = Some(Instant::now());
    app.runtime_state.animation_interval = Some(Duration::from_millis(16));
    app.preview_resources.render_loop_active = true;
    let _ = update(&mut app, Message::Daemon(Wake::Frame(Instant::now() + Duration::from_secs(1))));
    app.retick();

    assert_eq!(app.runtime_state.last_tick, before);
    assert!(app.runtime_state.next_animation_tick.is_none());
    assert!(!app.preview_resources.render_loop_active);
}

#[test]
fn relaunch_restores_stashed_overlay() {
    let mut app = shown_app();
    let _ = update(&mut app, Message::Stash);
    let _ = update(&mut app, Message::Daemon(Wake::Toggle));
    assert!(!app.runtime_state.stashed);
    assert!(app.runtime_state.overlay.is_some());
}

#[test]
fn show_and_open_restore_stashed_overlay() {
    let mut app = shown_app();
    let _ = update(&mut app, Message::Stash);
    cmd(&mut app, "show");
    assert!(!app.runtime_state.stashed);

    let _ = update(&mut app, Message::Stash);
    cmd(&mut app, "open mixer");
    assert!(!app.runtime_state.stashed);
    assert!(app.panels.audio.is_some());
}

#[test]
fn stash_needs_a_surface() {
    let mut app = test_app();
    cmd(&mut app, "stash");
    assert!(!app.runtime_state.stashed);
}

#[test]
fn stash_state_is_reported() {
    let mut app = shown_app();
    let _ = update(&mut app, Message::Stash);
    let state: Value = serde_json::from_str(&crate::app::update::ui_state_json(&app)).unwrap();
    assert_eq!(state["stashed"], json!(true));
}

#[test]
fn stash_closes_strand() {
    let mut app = shown_app();
    app.panels.strand = Some(crate::frontend::strand::Strand::new([1280.0, 720.0], 1.0, None));
    cmd(&mut app, "stash");
    assert!(app.panels.strand.is_none());
}
