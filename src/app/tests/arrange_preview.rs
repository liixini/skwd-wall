use super::*;
use crate::frontend::arrange::{ArrangeMsg, Target};
use iced::keyboard::{Key, Modifiers, key::Named};

fn begin() -> App {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Begin));
    for _ in 0..240 {
        crate::app::update::arrange::tick(&mut app, 1.0 / 60.0);
    }
    app
}

#[test]
fn preview_preserves_editor() {
    let mut app = begin();
    for _ in 0..=Target::Search.index() {
        let _ = update(&mut app, Message::KeyPressed(Key::Named(Named::Tab), Modifiers::empty()));
    }
    let history = app.panels.arrange.as_ref().unwrap().history.entries().len();
    let config = app.config.root().clone();
    crate::app::update::arrange::preview::enter(&mut app);
    let _ = update(&mut app, Message::SetSort(String::from("name")));
    crate::app::update::arrange::preview::leave(&mut app);
    let session = app.panels.arrange.as_ref().unwrap();
    assert_eq!(session.editor.selected(), Some(Target::Search));
    assert_eq!(session.history.entries().len(), history);
    assert_eq!(app.config.root(), &config);
}

#[test]
fn preview_blocks_side_effects() {
    let mut app = begin();
    crate::app::update::arrange::preview::enter(&mut app);
    let _ = drain_calls(&app);
    for message in [
        Message::ApplyCurrent,
        Message::ApplyRandom,
        Message::ToggleRandomRotate,
        Message::KeyFavourite,
        Message::KeyFlip,
        Message::ToggleAudioPanel,
        Message::OpenPlaylists,
    ] {
        let _ = update(&mut app, message);
    }
    assert!(drain_calls(&app).is_empty());
    assert!(!app.scene.flip_open());
}

#[test]
fn cancel_preserves_external() {
    let mut app = begin();
    let original = app.config.root().clone();
    app.config.set_key("filterBar.position.x", json!(25));
    let mut external = original.clone();
    external["unrelated"] = json!("external");
    std::fs::write(&app.config.config_path, serde_json::to_vec(&external).unwrap()).unwrap();
    app.config.finish_layout(&original, false).unwrap();
    assert_eq!(app.config.root()["unrelated"], "external");
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&app.config.config_path).unwrap()).unwrap(),
        external
    );
}

#[test]
fn done_merges_external() {
    let mut app = begin();
    let original = app.config.root().clone();
    app.config.set_key("filterBar.position.x", json!(25));
    let mut external = original.clone();
    external["unrelated"] = json!("external");
    std::fs::write(&app.config.config_path, serde_json::to_vec(&external).unwrap()).unwrap();
    app.config.finish_layout(&original, true).unwrap();
    assert_eq!(app.config.root()["unrelated"], "external");
    assert_eq!(app.config.root()["filterBar"]["position"]["x"], 25);
}

#[test]
fn done_rejects_conflict() {
    let mut app = begin();
    let original = app.config.root().clone();
    app.config.set_key("filterBar.position.x", json!(25));
    let mut external = app.config.root().clone();
    external["filterBar"]["position"]["x"] = json!(75);
    std::fs::write(&app.config.config_path, serde_json::to_vec(&external).unwrap()).unwrap();
    assert!(app.config.finish_layout(&original, true).is_err());
    assert!(app.config.is_transient());
    assert_eq!(app.config.root()["filterBar"]["position"]["x"], 25);
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&app.config.config_path).unwrap()).unwrap(),
        external
    );
}

#[test]
fn preview_click_selects() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    seed(&mut app, &[wall("first", "static", 0, 0), wall("second", "static", 1, 1)]);
    let _ = update(&mut app, Message::SetViewMode(String::from("wall")));
    let _ = update(&mut app, Message::Arrange(ArrangeMsg::Begin));
    crate::app::update::arrange::preview::enter(&mut app);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    let hit = app.scene.render.hits.iter().find(|hit| hit.index == 1).unwrap();
    let (x, y) = (hit.cx, hit.cy);
    let _ = drain_calls(&app);
    let _ = update(&mut app, Message::Click(x, y, crate::domain::input::MouseButton::Left));
    assert_eq!(app.scene.current, 1);
    assert!(drain_calls(&app).is_empty());
    let _ = update(&mut app, Message::Click(0.0, 0.0, crate::domain::input::MouseButton::Left));
    assert!(crate::app::update::arrange::preview::active(&app));
}

#[test]
fn preview_escape_returns() {
    let mut app = begin();
    crate::app::update::arrange::preview::enter(&mut app);
    let _ = update(
        &mut app,
        Message::KeyPressed(
            iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            iced::keyboard::Modifiers::empty(),
        ),
    );
    assert!(app.panels.arrange.is_some());
    assert!(!crate::app::update::arrange::preview::active(&app));
}

#[test]
fn malformed_config_stays_transient() {
    let mut app = begin();
    let original = app.config.root().clone();
    std::fs::write(&app.config.config_path, "{").unwrap();
    assert!(app.config.finish_layout(&original, true).is_err());
    assert!(app.config.is_transient());
    assert_eq!(std::fs::read_to_string(&app.config.config_path).unwrap(), "{");
}

#[test]
fn done_merges_sibling_keys() {
    let mut app = begin();
    let original = app.config.root().clone();
    app.config.set_key("filterBar.position.x", json!(25));
    let mut external = original.clone();
    external["filterBar"] = json!({"position": {"y": 75}});
    std::fs::write(&app.config.config_path, serde_json::to_vec(&external).unwrap()).unwrap();
    app.config.finish_layout(&original, true).unwrap();
    assert_eq!(app.config.root()["filterBar"]["position"], json!({"x": 25, "y": 75}));
}
