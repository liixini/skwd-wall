use super::*;
use crate::domain::strand::Rect;
use crate::frontend::arrange::{ArrangeMsg, Pointer, Target};
use skwd_config::keys::{filter_bar, selector};
use skwd_config::picker_layout::{Alignment, Component, Reference, Size};

const VIEWPORT: (f32, f32) = (1280.0, 720.0);
const TICK: f32 = 1.0 / 60.0;

fn begin() -> App {
    let mut app = test_app();
    app.scene.viewport = VIEWPORT;
    app.config
        .select_layout_output(Some(String::from("DP-1")), Size { width: 1280.0, height: 720.0 });
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    app
}

fn edit(app: &mut App, message: ArrangeMsg) {
    let _ = update(app, Message::Arrange(message));
    refresh_arrange_outputs(app);
}

fn settle(app: &mut App) {
    for _ in 0..240 {
        crate::app::update::arrange::tick(app, TICK);
        if app.panels.arrange.is_none() {
            return;
        }
    }
}

fn point(app: &mut App, pointer: Pointer) {
    edit(app, ArrangeMsg::Pointer(pointer));
}

fn click(app: &mut App, at: [f32; 2]) {
    point(app, Pointer::Press(at));
    point(app, Pointer::Release(at));
    settle(app);
}

fn drag(app: &mut App, from: [f32; 2], to: [f32; 2]) {
    point(app, Pointer::Press(from));
    for step in 1..=8 {
        let share = step as f32 / 8.0;
        point(
            app,
            Pointer::Move([
                from[0] + (to[0] - from[0]) * share,
                from[1] + (to[1] - from[1]) * share,
            ]),
        );
        crate::app::update::arrange::tick(app, TICK);
    }
    point(app, Pointer::Release(to));
    settle(app);
}

fn rect(app: &App, target: Target) -> Rect {
    let rect = crate::app::update::arrange::placement::rect(app, target);
    Rect::new(rect.x, rect.y, rect.width, rect.height)
}

fn plate(app: &App, id: &str) -> [f32; 2] {
    let plates = app.panels.arrange.as_ref().unwrap().editor.plates();
    plates.iter().find(|(name, _)| name == id).map(|(_, rect)| rect.centre()).unwrap()
}

#[test]
fn reset_keeps_other_components() {
    let mut app = begin();
    let search = rect(&app, Target::Search).centre();
    drag(&mut app, search, [search[0] - 60.0, search[1] + 80.0]);
    let moved = app.config.num_path(selector::TAG_CLOUD_POSITION_X);
    let picker = rect(&app, Target::Picker).centre();
    drag(&mut app, picker, [picker[0] + 70.0, picker[1] + 20.0]);
    let centre = rect(&app, Target::Picker).centre();
    click(&mut app, centre);
    let reset = plate(&app, "tool:Reset");
    click(&mut app, reset);
    assert!(app.config.position(Component::Picker).is_none());
    assert_eq!(app.config.num_path(selector::TAG_CLOUD_POSITION_X), moved);
}

#[test]
fn reset_keeps_other_outputs() {
    let mut app = begin();
    let viewport = Size { width: 1920.0, height: 1080.0 };
    let mut other = app.config.layout_profile("DP-2", viewport).unwrap();
    other.values.insert(selector::FRAME_X.to_owned(), Some(json!(70)));
    app.config.set_layout_profile("DP-2", &other).unwrap();
    let picker = rect(&app, Target::Picker).centre();
    drag(&mut app, picker, [picker[0] - 50.0, picker[1] + 40.0]);
    assert!(app.config.position(Component::Picker).is_some());
    let centre = rect(&app, Target::Picker).centre();
    click(&mut app, centre);
    let reset = plate(&app, "tool:Reset");
    click(&mut app, reset);
    assert!(app.config.position(Component::Picker).is_none());
    assert_eq!(app.config.layout_profile("DP-2", viewport).unwrap(), other);
}

#[test]
fn snapped_bar_attaches_to_picker() {
    let mut app = begin();
    let picker = rect(&app, Target::Picker);
    let bar = rect(&app, Target::Bar);
    let from = [bar.x + 4.0, bar.centre()[1]];
    let to = [from[0] + 3.0, picker.y - bar.h / 2.0 - 6.0];
    drag(&mut app, from, to);
    let position = app.config.position(Component::Bar).expect("bar position");
    assert_eq!(position.reference, Reference::Picker);
    assert_eq!(position.alignment.factors()[1], 0.0);
    let picker = rect(&app, Target::Picker).centre();
    let bar = rect(&app, Target::Bar);
    drag(&mut app, picker, [picker[0] - 40.0, picker[1] + 30.0]);
    let followed = rect(&app, Target::Bar);
    assert!((followed.x - (bar.x - 40.0)).abs() < 1.0);
    assert!((followed.y - (bar.y + 30.0)).abs() < 1.0);
}

#[test]
fn free_drop_pins_to_screen() {
    let mut app = begin();
    let search = rect(&app, Target::Search);
    let goal = [VIEWPORT.0 - search.w / 2.0 - 30.0, 300.0];
    drag(&mut app, search.centre(), goal);
    let position = app.config.position(Component::Search).expect("search position");
    assert_eq!(position.reference, Reference::Screen);
    assert_eq!(position.alignment, Alignment::Right);
    let placed = rect(&app, Target::Search);
    assert!((placed.centre()[0] - goal[0]).abs() < 11.0);
}

#[test]
fn grid_resize_sets_cells() {
    let mut app = test_app();
    app.config.set_key(selector::DISPLAY_MODE, json!("wall"));
    app.scene.viewport = VIEWPORT;
    app.apply_layout();
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    let centre = rect(&app, Target::Picker).centre();
    click(&mut app, centre);
    let gem = plate(&app, "gem:2");
    let picker = rect(&app, Target::Picker);
    let cell = app.config.grid_thumb_width() + app.config.grid_gap_x();
    drag(&mut app, gem, [gem[0] - 2.0 * cell, gem[1]]);
    let columns = app.config.num_path(selector::GRID_COLUMNS) as usize;
    let resized = rect(&app, Target::Picker);
    assert!(resized.w < picker.w);
    assert_eq!(app.scene.grid_capacity().0, columns);
    assert_eq!([resized.x, resized.y], [picker.x, picker.y]);
}

#[test]
fn conflict_prompt_resolves() {
    let mut app = test_app();
    app.scene.viewport = VIEWPORT;
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    let search = rect(&app, Target::Search).centre();
    drag(&mut app, search, [search[0] - 60.0, search[1] - 140.0]);
    let original = crate::app::update::arrange::snapshot(&app);
    let mut external = app.config.document().clone();
    external["components"]["wallpaperSelector"]["tagCloudPositionX"] = json!(3);
    std::fs::write(&app.config.config_path, serde_json::to_vec(&external).unwrap()).unwrap();
    let done = plate(&app, "done");
    click(&mut app, done);
    assert!(app.panels.arrange.as_ref().unwrap().conflict.is_some());
    let keep = plate(&app, "choice:Keep");
    click(&mut app, keep);
    assert!(app.panels.arrange.is_none());
    assert!(!app.config.is_transient());
    let _ = original;
}

#[test]
fn displays_plate_needs_output() {
    let app = begin();
    assert!(
        app.panels.arrange.as_ref().unwrap().editor.plates().iter().any(|(id, _)| id == "displays")
    );
    let mut plain = test_app();
    plain.scene.viewport = VIEWPORT;
    edit(&mut plain, ArrangeMsg::Begin);
    settle(&mut plain);
    assert!(
        !plain
            .panels
            .arrange
            .as_ref()
            .unwrap()
            .editor
            .plates()
            .iter()
            .any(|(id, _)| id == "displays")
    );
}

#[test]
fn conflict_choices_preserve_siblings() {
    for keep in [false, true] {
        let mut app = test_app();
        let original = app.config.document().clone();
        let _ = app.config.begin_transient();
        app.config.set_key(filter_bar::POSITION_X, json!(25));
        app.config.set_key(filter_bar::POSITION_Y, json!(30));
        let mut external = original.clone();
        external["filterBar"] = json!({"positionX": 75, "external": true});
        std::fs::write(&app.config.config_path, serde_json::to_vec(&external).unwrap()).unwrap();
        let crate::infrastructure::config::LayoutSaveError::Conflict(reviewed) =
            app.config.finish_layout(&original, true).unwrap_err()
        else {
            panic!("expected conflict")
        };
        assert_eq!(*reviewed, external);
        app.config.resolve_layout(&original, &reviewed, keep).unwrap();
        assert_eq!(app.config.num_path(filter_bar::POSITION_X), if keep { 25.0 } else { 75.0 });
        assert_eq!(app.config.num_path(filter_bar::POSITION_Y), 30.0);
        assert_eq!(app.config.document()["filterBar"]["external"], true);
    }
}

#[test]
fn conflict_review_detects_changes() {
    let mut app = test_app();
    let original = app.config.document().clone();
    let _ = app.config.begin_transient();
    app.config.set_key(filter_bar::POSITION_X, json!(25));
    let mut reviewed = original.clone();
    reviewed["filterBar"] = json!({"positionX": 75});
    let mut latest = reviewed.clone();
    latest["filterBar"]["positionX"] = json!(90);
    std::fs::write(&app.config.config_path, serde_json::to_vec(&latest).unwrap()).unwrap();
    assert!(matches!(
        app.config.resolve_layout(&original, &reviewed, true),
        Err(crate::infrastructure::config::LayoutSaveError::Conflict(_))
    ));
    assert_eq!(app.config.num_path(filter_bar::POSITION_X), 25.0);
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&app.config.config_path).unwrap()).unwrap(),
        latest
    );
}
