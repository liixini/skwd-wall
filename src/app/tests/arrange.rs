use super::*;
use crate::domain::strand::{Rect, zone_top};
use crate::frontend::arrange::{ArrangeMsg, Pointer, Target};

const VIEWPORT: (f32, f32) = (1280.0, 720.0);
const TICK: f32 = 1.0 / 60.0;
const SETTLE: usize = 240;

fn edit(app: &mut App, message: ArrangeMsg) {
    let _ = update(app, Message::Arrange(message));
    refresh_arrange_outputs(app);
}

fn settle(app: &mut App) {
    for _ in 0..SETTLE {
        crate::app::update::arrange::tick(app, TICK);
        app.chrome.tick_filter_bar(TICK);
        if app.panels.arrange.is_none() {
            return;
        }
    }
}

fn begin() -> App {
    let mut app = test_app();
    app.scene.viewport = VIEWPORT;
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    app
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
        let at = [from[0] + (to[0] - from[0]) * share, from[1] + (to[1] - from[1]) * share];
        point(app, Pointer::Move(at));
        crate::app::update::arrange::tick(app, TICK);
    }
    point(app, Pointer::Release(to));
    settle(app);
}

fn plate(app: &App, id: &str) -> [f32; 2] {
    let session = app.panels.arrange.as_ref().unwrap();
    let plates = session.editor.plates();
    let found = plates.iter().find(|(name, _)| name == id);
    let names: Vec<&String> = plates.iter().map(|(name, _)| name).collect();
    found.map_or_else(|| panic!("{id} not in {names:?}"), |(_, rect)| rect.centre())
}

fn rect(app: &App, target: Target) -> Rect {
    let rect = crate::app::update::arrange::placement::rect(app, target);
    Rect::new(rect.x, rect.y, rect.width, rect.height)
}

fn steps(app: &App) -> Vec<String> {
    let session = app.panels.arrange.as_ref().unwrap();
    session.history.entries().iter().map(|entry| entry.label.clone()).collect()
}

fn key(app: &mut App, key: iced::keyboard::Key, modifiers: iced::keyboard::Modifiers) {
    let _ = update(app, Message::KeyPressed(key, modifiers));
    settle(app);
}

fn select(app: &mut App, target: Target) {
    let centre = rect(app, target).centre();
    click(app, centre);
    assert_eq!(app.panels.arrange.as_ref().unwrap().editor.selected(), Some(target));
}

fn button(app: &App, index: usize) -> (String, [f32; 2]) {
    let (model, bar) = crate::app::view::filter_bar_target(app, VIEWPORT.0, VIEWPORT.1);
    let item = model.items.iter().filter(|item| item.edit_id().is_some()).nth(index).unwrap();
    let at = [bar.x + item.x + item.w / 2.0, bar.y + item.y + item.h / 2.0];
    (item.edit_id().unwrap(), at)
}

#[test]
fn tab_cycles_targets() {
    use iced::keyboard::{Key, Modifiers, key::Named};
    let mut app = begin();
    for target in Target::ALL.into_iter().chain([Target::Picker]) {
        key(&mut app, Key::Named(Named::Tab), Modifiers::empty());
        assert_eq!(app.panels.arrange.as_ref().unwrap().editor.selected(), Some(target));
    }
    key(&mut app, Key::Named(Named::Tab), Modifiers::SHIFT);
    assert_eq!(app.panels.arrange.as_ref().unwrap().editor.selected(), Some(Target::Swatch));
    assert_eq!(steps(&app).len(), 1);
}

#[test]
fn entry_waits_for_thread() {
    let mut app = test_app();
    app.scene.viewport = VIEWPORT;
    edit(&mut app, ArrangeMsg::Begin);
    let centre = rect(&app, Target::Picker).centre();
    point(&mut app, Pointer::Press(centre));
    point(&mut app, Pointer::Release(centre));
    assert!(app.panels.arrange.as_ref().unwrap().editor.selected().is_none());
    settle(&mut app);
    click(&mut app, centre);
    assert_eq!(app.panels.arrange.as_ref().unwrap().editor.selected(), Some(Target::Picker));
}

#[test]
fn session_cancels_exactly() {
    use iced::keyboard::{Key, Modifiers, key::Named};
    let mut app = test_app();
    let before = crate::app::update::arrange::snapshot(&app);
    app.scene.viewport = VIEWPORT;
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    select(&mut app, Target::Picker);
    let next = plate(&app, "tool:Next");
    click(&mut app, next);
    let picker = rect(&app, Target::Picker).centre();
    drag(&mut app, picker, [picker[0] - 90.0, picker[1] - 40.0]);
    assert_ne!(crate::app::update::arrange::snapshot(&app), before);
    key(&mut app, Key::Named(Named::Escape), Modifiers::empty());
    assert!(app.panels.arrange.as_ref().unwrap().editor.selected().is_none());
    key(&mut app, Key::Named(Named::Escape), Modifiers::empty());
    assert!(app.panels.arrange.is_none());
    assert_eq!(crate::app::update::arrange::snapshot(&app), before);
    assert!(!app.config.is_transient());
}

#[test]
fn done_persists_draft() {
    let mut app = test_app();
    app.scene.viewport = VIEWPORT;
    app.config.persist();
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    let saved = std::fs::read(&app.config.config_path).unwrap();
    select(&mut app, Target::Picker);
    let next = plate(&app, "tool:Next");
    click(&mut app, next);
    let mode = app.config.str_path(skwd_config::keys::selector::DISPLAY_MODE);
    assert_eq!(std::fs::read(&app.config.config_path).unwrap(), saved);
    let done = plate(&app, "done");
    click(&mut app, done);
    assert!(app.panels.arrange.is_none());
    let written: Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    assert_eq!(
        skwd_config::get(&written, skwd_config::keys::selector::DISPLAY_MODE),
        Some(&json!(mode))
    );
}

#[test]
fn control_enter_saves() {
    use iced::keyboard::{Key, Modifiers, key::Named};
    let mut app = begin();
    select(&mut app, Target::Search);
    key(&mut app, Key::Named(Named::ArrowRight), Modifiers::SHIFT);
    key(&mut app, Key::Named(Named::Enter), Modifiers::CTRL);
    assert!(app.panels.arrange.is_none());
    assert!(!app.config.is_transient());
}

#[test]
fn drag_records_once() {
    let mut app = begin();
    let picker = rect(&app, Target::Picker);
    let from = picker.centre();
    drag(&mut app, from, [from[0] - 120.0, from[1] + 30.0]);
    assert_eq!(steps(&app), ["arrange-step-start", "arrange-step-move"]);
    let moved = rect(&app, Target::Picker);
    assert!((moved.x - (picker.x - 120.0)).abs() < 1.0);
    assert_eq!(app.scene.picker_frame_bounds(), app.scene.configured_frame_bounds());
}

#[test]
fn frame_follows_springs() {
    let mut app = begin();
    let from = rect(&app, Target::Picker).centre();
    point(&mut app, Pointer::Press(from));
    point(&mut app, Pointer::Move([from[0] - 200.0, from[1]]));
    crate::app::update::arrange::tick(&mut app, TICK);
    let shown = app.scene.picker_frame_bounds().unwrap();
    let target = app.scene.configured_frame_bounds().unwrap();
    assert!(shown.x > target.x + 1.0);
    point(&mut app, Pointer::Release([from[0] - 200.0, from[1]]));
    settle(&mut app);
    assert_eq!(app.scene.picker_frame_bounds(), app.scene.configured_frame_bounds());
}

#[test]
fn leaving_abandons_drag() {
    let mut app = begin();
    let before = crate::app::update::arrange::snapshot(&app);
    let from = rect(&app, Target::Search).centre();
    point(&mut app, Pointer::Press(from));
    point(&mut app, Pointer::Move([from[0] - 80.0, from[1] + 60.0]));
    point(&mut app, Pointer::Leave);
    settle(&mut app);
    assert_eq!(crate::app::update::arrange::snapshot(&app), before);
    assert_eq!(steps(&app).len(), 1);
}

#[test]
fn undo_and_branch() {
    use iced::keyboard::{Key, Modifiers};
    let mut app = begin();
    let from = rect(&app, Target::Search).centre();
    drag(&mut app, from, [from[0] - 70.0, from[1] + 50.0]);
    let first = rect(&app, Target::Search);
    let from = first.centre();
    drag(&mut app, from, [from[0] - 70.0, from[1] + 50.0]);
    key(&mut app, Key::Character("z".into()), Modifiers::CTRL);
    assert_eq!(rect(&app, Target::Search), first);
    assert_eq!(app.panels.arrange.as_ref().unwrap().editor.selected(), Some(Target::Search));
    let from = first.centre();
    drag(&mut app, from, [from[0] + 40.0, from[1] + 20.0]);
    assert_eq!(steps(&app).len(), 3);
    key(&mut app, Key::Character("y".into()), Modifiers::CTRL);
    assert_eq!(steps(&app).len(), 3);
}

#[test]
fn reorder_commits_on_drop() {
    let mut app = begin();
    let original = app.config.filter_bar_order();
    select(&mut app, Target::Bar);
    let (first, from) = button(&app, 0);
    let (_, to) = button(&app, 2);
    drag(&mut app, from, [to[0] + 10.0, to[1]]);
    assert_ne!(app.config.filter_bar_order(), original);
    assert!(app.config.filter_bar_order().iter().position(|id| *id == first) > Some(0));
    assert_eq!(steps(&app).last().map(String::as_str), Some("arrange-step-order"));
}

#[test]
fn reorder_noop_keeps_history() {
    let mut app = begin();
    select(&mut app, Target::Bar);
    let (_, at) = button(&app, 1);
    drag(&mut app, at, [at[0] + 2.0, at[1]]);
    assert_eq!(steps(&app).len(), 1);
    assert!(app.chrome.bar.reorder.is_none());
}

#[test]
fn thread_hides_buttons() {
    let mut app = begin();
    select(&mut app, Target::Bar);
    let (id, at) = button(&app, 1);
    let below = [at[0], zone_top(VIEWPORT.1) + 30.0];
    point(&mut app, Pointer::Press(at));
    point(&mut app, Pointer::Move([at[0], at[1] + 40.0]));
    point(&mut app, Pointer::Move(below));
    point(&mut app, Pointer::Release(below));
    for _ in 0..45 {
        crate::app::update::arrange::tick(&mut app, TICK);
    }
    assert!(app.config.filter_bar_hidden_buttons().contains(&id));
    assert_eq!(steps(&app).last().map(String::as_str), Some("arrange-step-button"));
    let undo = plate(&app, "undo");
    click(&mut app, undo);
    assert!(!app.config.filter_bar_hidden_buttons().contains(&id));
}

#[test]
fn bead_click_shows_button() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::filter_bar::SHOW_RANDOM, json!(false));
    app.scene.viewport = VIEWPORT;
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    let tools = plate(&app, "family:3");
    point(&mut app, Pointer::Move(tools));
    settle(&mut app);
    let bead = plate(&app, "bead:random");
    click(&mut app, bead);
    assert!(app.config.filter_show("random"));
    assert_eq!(steps(&app).last().map(String::as_str), Some("arrange-step-button"));
    assert_eq!(app.panels.arrange.as_ref().unwrap().editor.selected(), Some(Target::Bar));
}

#[test]
fn studio_shelf_persists() {
    use crate::infrastructure::config::Config;
    use skwd_config::keys::filter_bar;

    for (id, path) in
        [("colour-studio", filter_bar::SHOW_COLOUR_STUDIO), ("widgets", filter_bar::SHOW_WIDGETS)]
    {
        let mut app = begin();
        let tools = plate(&app, "family:3");
        point(&mut app, Pointer::Move(tools));
        settle(&mut app);
        let bead = plate(&app, &format!("bead:{id}"));
        click(&mut app, bead);
        assert!(app.config.flag_default_config(path));
        edit(&mut app, ArrangeMsg::Done);
        let mut reopened = Config::from_data(json!({}));
        reopened.config_path.clone_from(&app.config.config_path);
        assert!(reopened.reload());
        assert!(reopened.flag_default_config(path));
    }
}

#[test]
fn legacy_colours_restore_one() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::filter_bar::SHOW_COLORS, json!(false));
    app.scene.viewport = VIEWPORT;
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    let colours = plate(&app, "family:2");
    point(&mut app, Pointer::Move(colours));
    settle(&mut app);
    let bead = plate(&app, "bead:color:0");
    click(&mut app, bead);
    assert!(app.config.filter_show("colors"));
    let hidden = app.config.filter_bar_hidden_buttons();
    assert!(!hidden.iter().any(|id| id == "color:0"));
    assert_eq!(hidden.len(), 12);
}

#[test]
fn layout_plates_cycle_mode() {
    let mut app = begin();
    let start = app.scene.mode;
    select(&mut app, Target::Picker);
    let next = plate(&app, "tool:Next");
    click(&mut app, next);
    assert_ne!(app.scene.mode, start);
    let previous = plate(&app, "tool:Prev");
    click(&mut app, previous);
    assert_eq!(app.scene.mode, start);
    assert_eq!(steps(&app).len(), 3);
}

#[test]
fn spacing_plate_cycles() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::selector::DISPLAY_MODE, json!("wall"));
    app.scene.viewport = VIEWPORT;
    app.apply_layout();
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    select(&mut app, Target::Picker);
    let key = crate::app::update::arrange::spacing_key(app.scene.mode);
    let before = app.config.num_path(key);
    let spacing = plate(&app, "tool:Spacing");
    click(&mut app, spacing);
    assert_ne!(app.config.num_path(key), before);
    assert_eq!(steps(&app).last().map(String::as_str), Some("arrange-step-style"));
}

#[test]
fn corners_plate_sets_shared() {
    use skwd_config::corners::{CornerStyle, GRID};
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::selector::DISPLAY_MODE, json!("wall"));
    app.config.set_key(GRID.style, json!("chamfered"));
    app.config.set_key(GRID.same, json!(false));
    app.config.set_key(GRID.values[2], json!(40));
    let kept = app.config.corners(&GRID).values;
    app.scene.viewport = VIEWPORT;
    app.apply_layout();
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    select(&mut app, Target::Picker);
    let corners = plate(&app, "tool:Corners");
    let mut seen = Vec::new();
    for _ in 0..3 {
        click(&mut app, corners);
        let now = app.config.corners(&GRID);
        assert!(now.same);
        assert_eq!(now.values, kept);
        seen.push((now.style, now.all));
    }
    assert_eq!(
        seen,
        [
            (CornerStyle::Chamfered, 12.0),
            (CornerStyle::Chamfered, 28.0),
            (CornerStyle::Straight, 0.0)
        ]
    );
    click(&mut app, corners);
    assert_eq!(app.config.corners(&GRID).style, CornerStyle::Round);
}

#[test]
fn corners_plate_cuts() {
    use skwd_config::corners::{CornerStyle, GRID};
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::selector::DISPLAY_MODE, json!("wall"));
    app.config.set_key(GRID.style, json!("straight"));
    app.scene.viewport = VIEWPORT;
    app.apply_layout();
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    select(&mut app, Target::Picker);
    let corners = plate(&app, "tool:Corners");
    let seen: Vec<_> = (0..5)
        .map(|_| {
            click(&mut app, corners);
            let now = app.config.corners(&GRID);
            (now.style, now.all)
        })
        .collect();
    assert_eq!(
        seen,
        [
            (CornerStyle::Round, 12.0),
            (CornerStyle::Round, 28.0),
            (CornerStyle::Chamfered, 12.0),
            (CornerStyle::Chamfered, 28.0),
            (CornerStyle::Straight, 0.0)
        ]
    );
}

#[test]
fn corners_plate_each() {
    use crate::frontend::arrange::CornerStep;
    use skwd_config::corners::GRID;
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::selector::DISPLAY_MODE, json!("wall"));
    app.config.set_key(GRID.style, json!("chamfered"));
    app.config.set_key(GRID.same, json!(false));
    app.config.set_key(GRID.values[2], json!(40));
    let kept = app.config.corners(&GRID).values;
    app.scene.viewport = VIEWPORT;
    app.apply_layout();
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    select(&mut app, Target::Picker);
    let shown = |app: &App| app.panels.arrange.as_ref().unwrap().editor.board.style.corners;
    assert_eq!(shown(&app), Some(CornerStep::Each));
    let corners = plate(&app, "tool:Corners");
    click(&mut app, corners);
    assert!(matches!(shown(&app), Some(CornerStep::Step(_))));
    let now = app.config.corners(&GRID);
    assert!(now.same);
    assert_eq!(now.values, kept);
}

#[test]
fn picker_keeps_bar_fixed() {
    let mut app = begin();
    let bar = rect(&app, Target::Bar);
    let from = rect(&app, Target::Picker).centre();
    drag(&mut app, from, [from[0] + 60.0, from[1] + 40.0]);
    let after = rect(&app, Target::Bar);
    assert!((after.x - bar.x).abs() < 0.01 && (after.y - bar.y).abs() < 0.01);
    assert!(app.config.filter_bar_position().is_some());
}

#[test]
fn reset_keeps_other_object() {
    let mut app = begin();
    let search = rect(&app, Target::Search);
    let from = search.centre();
    drag(&mut app, from, [from[0] - 90.0, from[1] + 70.0]);
    let from = rect(&app, Target::Picker).centre();
    drag(&mut app, from, [from[0] + 50.0, from[1] + 30.0]);
    let picker = rect(&app, Target::Picker);
    select(&mut app, Target::Search);
    let reset = plate(&app, "tool:Reset");
    click(&mut app, reset);
    assert_eq!(rect(&app, Target::Search), search);
    assert_eq!(rect(&app, Target::Picker), picker);
    assert_eq!(steps(&app).last().map(String::as_str), Some("arrange-step-reset"));
}

#[test]
fn browse_inputs_are_blocked() {
    let mut app = begin();
    let before = app.config.root().clone();
    let index = app.scene.current;
    let _ = update(&mut app, Message::Click(10.0, 10.0, crate::domain::input::MouseButton::Left));
    let _ = update(&mut app, Message::Wheel(5.0));
    let _ = update(&mut app, Message::KeyNext);
    assert!(app.panels.arrange.is_some());
    assert_eq!(app.scene.current, index);
    assert_eq!(app.config.root(), &before);
}

#[test]
fn exit_cancels_session() {
    let mut app = begin();
    let before = crate::app::update::arrange::snapshot(&app);
    let from = rect(&app, Target::Search).centre();
    drag(&mut app, from, [from[0] - 50.0, from[1] + 50.0]);
    let _ = update(&mut app, Message::Exit);
    settle(&mut app);
    assert!(app.panels.arrange.as_ref().unwrap().editor.selected().is_none());
    let _ = update(&mut app, Message::Exit);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
    assert_eq!(crate::app::update::arrange::snapshot(&app), before);
}

#[test]
fn state_lists_plates() {
    let app = begin();
    let state: Value = serde_json::from_str(&crate::app::update::ui_state_json(&app)).unwrap();
    let plates = state["arrange"]["plates"].as_array().unwrap();
    for id in ["done", "cancel", "preview"] {
        assert!(plates.iter().any(|plate| plate["id"] == id), "{id}");
    }
    assert_eq!(state["arrange"]["ready"], json!(true));
}
