use super::*;
use crate::app::update::arrange::placement;
use crate::app::{
    ENTRY_HEIGHT, ENTRY_PAD, SWATCH_CELL, SWATCH_STACK_GAP, rest_bottom, swatch_height,
};
use crate::domain::strand::{Rect, zone_top};
use crate::frontend::arrange::{ArrangeMsg, Pointer, Target};
use crate::frontend::settings::{ActionId, SettingsMsg};
use crate::frontend::strand::tokens::PLATE;
use skwd_config::keys::filter_bar;
use skwd_config::picker_layout::{
    Alignment, Component, LayoutProfile, Offset, Position, Reference, Size,
};

const LOWEST: (f32, f32) = (1366.0, 768.0);
const TICK: f32 = 1.0 / 60.0;
const SETTLE_TICKS: usize = 240;
const FLOATS: [Target; 2] = [Target::Arrange, Target::Swatch];
const HIT: f32 = PLATE.height / 2.0;
const SCALES: [f32; 2] = [1.0, 1.5];
const VIEWPORTS: [(f32, f32); 2] = [LOWEST, (1920.0, 1080.0)];
const CORNERS: [(f32, f32); 4] = [(0.0, 0.0), (100.0, 0.0), (0.0, 100.0), (100.0, 100.0)];
const NEW_KEYS: [&str; 8] = [
    filter_bar::SHOW_ARRANGE,
    filter_bar::SHOW_SWATCH,
    filter_bar::ARRANGE_POSITION_ENABLED,
    filter_bar::ARRANGE_POSITION_X,
    filter_bar::ARRANGE_POSITION_Y,
    filter_bar::SWATCH_POSITION_ENABLED,
    filter_bar::SWATCH_POSITION_X,
    filter_bar::SWATCH_POSITION_Y,
];

fn app_at(viewport: (f32, f32)) -> App {
    let mut app = test_app();
    app.scene.viewport = viewport;
    app
}

fn edit(app: &mut App, message: ArrangeMsg) {
    let _ = update(app, Message::Arrange(message));
    refresh_arrange_outputs(app);
}

fn settle(app: &mut App) {
    for _ in 0..SETTLE_TICKS {
        crate::app::update::arrange::tick(app, TICK);
        if app.panels.arrange.is_none() {
            return;
        }
    }
}

fn begin(viewport: (f32, f32)) -> App {
    open(app_at(viewport))
}

fn open(mut app: App) -> App {
    let viewport = app.scene.viewport;
    let size = Size { width: viewport.0, height: viewport.1 };
    app.config.select_layout_output(Some(String::from("DP-1")), size);
    app.runtime_state.picker_output = Some(String::from("DP-1"));
    app.daemon.output_statuses = vec![crate::contracts::daemon::OutputStatus {
        name: String::from("DP-1"),
        target: String::from("DP-1"),
        connected: true,
        width: viewport.0 as i32,
        height: viewport.1 as i32,
        logical_width: viewport.0 as i32,
        logical_height: viewport.1 as i32,
        ..crate::contracts::daemon::OutputStatus::default()
    }];
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

fn rect(app: &App, target: Target) -> Rect {
    let rect = placement::rect(app, target);
    Rect::new(rect.x, rect.y, rect.width, rect.height)
}

fn grab(app: &App, target: Target) -> [f32; 2] {
    let rect = rect(app, target);
    let side = if target == Target::Arrange { rect.x + 2.0 } else { rect.right() - 2.0 };
    [side, rect.centre()[1]]
}

fn plate(app: &App, id: &str) -> [f32; 2] {
    let plates = app.panels.arrange.as_ref().unwrap().editor.plates();
    plates.iter().find(|(name, _)| name == id).map(|(_, rect)| rect.centre()).unwrap()
}

#[test]
fn old_config_unchanged() {
    for viewport in [LOWEST, (3840.0, 2160.0)] {
        let app = app_at(viewport);
        assert!(app.config.arrange_position().is_none());
        assert!(app.config.swatch_position().is_none());
        let entry = rect(&app, Target::Arrange);
        assert_eq!(entry.x, ENTRY_PAD);
        assert!((entry.bottom() - rest_bottom(viewport.1)).abs() < 0.01);
        assert!((entry.h - ENTRY_HEIGHT).abs() < 0.01);
        assert!((rect(&app, Target::Swatch).h - swatch_height(1.0)).abs() < 0.01);
        for target in FLOATS {
            assert!(placement::present(&app.config, target));
        }
    }
}

#[test]
fn strip_stacks_above() {
    for viewport in VIEWPORTS {
        for scale in SCALES {
            let mut app = app_at(viewport);
            app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(scale));
            let (entry, strip) = (rect(&app, Target::Arrange), rect(&app, Target::Swatch));
            assert_eq!(strip.x, entry.x);
            assert!((strip.bottom() + SWATCH_STACK_GAP - entry.y).abs() < 0.01, "{strip:?}");
            assert!(!strip.overlaps(entry));
            assert!((strip.h - swatch_height(scale)).abs() < 0.01);
        }
    }
}

#[test]
fn defaults_clear_thread() {
    for viewport in VIEWPORTS {
        for scale in SCALES {
            for off in [false, true] {
                let mut app = app_at(viewport);
                app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(scale));
                app.config.set_key(filter_bar::SHOW_ARRANGE, json!(!off));
                let app = open(app);
                let session = app.panels.arrange.as_ref().unwrap();
                let laid = session.editor.spool.laid.as_ref().unwrap();
                let mut threads: Vec<Rect> =
                    session.editor.plates().into_iter().map(|(_, rect)| rect).collect();
                threads.extend(laid.plan.items.iter().filter(|placed| !placed.off).map(|placed| {
                    let x = session.editor.spool.item_x(session.editor.now, placed.item)
                        + placed.reach[0];
                    let y = session.editor.spool.thread_at(x);
                    Rect::new(x, y - HIT, placed.reach[1] - placed.reach[0], PLATE.height)
                }));
                let floats: Vec<Rect> = FLOATS
                    .into_iter()
                    .filter(|target| placement::present(&app.config, *target))
                    .map(|target| rect(&app, target))
                    .collect();
                for (at, float) in floats.iter().enumerate() {
                    assert!(float.bottom() <= zone_top(viewport.1), "{float:?}");
                    assert!(threads.iter().all(|thread| !float.overlaps(*thread)), "{float:?}");
                    assert!(floats[at + 1..].iter().all(|other| !float.overlaps(*other)));
                }
            }
        }
    }
}

#[test]
fn strip_takes_bottom() {
    for viewport in VIEWPORTS {
        for scale in SCALES {
            let mut app = app_at(viewport);
            app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(scale));
            app.config.set_key(filter_bar::SHOW_ARRANGE, json!(false));
            let strip = rect(&app, Target::Swatch);
            assert_eq!(strip.x, ENTRY_PAD);
            let lifted = strip.bottom() < rest_bottom(viewport.1) - 0.01;
            let panel = rect(&app, Target::Search);
            assert!(!lifted || strip.bottom() <= panel.y - SWATCH_STACK_GAP + 0.01, "{strip:?}");
            assert!(lifted || (strip.bottom() - rest_bottom(viewport.1)).abs() < 0.01, "{strip:?}");
        }
    }
}

#[test]
fn moved_entry_frees_stack() {
    let mut app = app_at(LOWEST);
    let (enabled, x, y) = placement::position_keys(Target::Arrange);
    app.config.set_key(enabled, json!(true));
    app.config.set_key(x, json!(50));
    app.config.set_key(y, json!(10));
    let strip = rect(&app, Target::Swatch);
    assert!((strip.bottom() - rest_bottom(LOWEST.1)).abs() < 0.01, "{strip:?}");
}

#[test]
fn strip_width_true() {
    let mut app = app_at(LOWEST);
    let cells = 13;
    app.theme.swatch = vec![iced::Color::WHITE; cells];
    assert!((rect(&app, Target::Swatch).w - cells as f32 * SWATCH_CELL).abs() < 0.01);
}

fn clicks(app: &App, at: [f32; 2]) -> bool {
    use iced::advanced::clipboard::Null;
    use iced::mouse::{Button, Cursor, Event as Mouse};
    use iced_runtime::user_interface::{Cache, UserInterface};
    let view = crate::app::swatch_overlay(app).expect("strip");
    let mut renderer = crate::frontend::headless::renderer(crate::frontend::ui::UI_FONT);
    let screen = iced::Size::new(app.scene.viewport.0, app.scene.viewport.1);
    let mut ui = UserInterface::build(view, screen, Cache::default(), &mut renderer);
    let press = [
        iced::Event::Mouse(Mouse::ButtonPressed(Button::Left)),
        iced::Event::Mouse(Mouse::ButtonReleased(Button::Left)),
    ];
    let mut messages = Vec::new();
    let cursor = Cursor::Available(iced::Point::new(at[0], at[1]));
    let _ = ui.update(&press, cursor, &mut renderer, &mut Null, &mut messages);
    messages.iter().any(|message| matches!(message, Message::ToggleThemePanel))
}

#[test]
#[ignore = "needs a Vulkan adapter"]
fn strip_click_edges() {
    for cells in [0, 13] {
        let mut app = app_at(LOWEST);
        app.theme.swatch = vec![iced::Color::WHITE; cells];
        let strip = rect(&app, Target::Swatch);
        let inside = [[strip.x + 1.0, strip.y + 1.0], [strip.right() - 1.0, strip.bottom() - 1.0]];
        let outside = [[strip.x - 2.0, strip.y + 1.0], [strip.right() + 2.0, strip.bottom() - 1.0]];
        assert!(inside.into_iter().all(|at| clicks(&app, at)), "{strip:?}");
        assert!(outside.into_iter().all(|at| !clicks(&app, at)), "{strip:?}");
    }
}

#[test]
fn entry_stays_under_editor() {
    let mut app = app_at(LOWEST);
    assert!(crate::app::view::entry_shown(&app, true, true));
    assert!(!crate::app::view::entry_shown(&app, false, true));
    assert!(!crate::app::view::entry_shown(&app, true, false));
    app = begin(LOWEST);
    assert!(app.panels.arrange.is_some());
    assert!(crate::app::view::entry_shown(&app, true, true));
    app.config.set_key(filter_bar::SHOW_ARRANGE, json!(false));
    assert!(!crate::app::view::entry_shown(&app, true, true));
}

fn place_at(app: &mut App, target: Target, corner: (f32, f32)) {
    let (enabled, x, y) = placement::position_keys(target);
    app.config.set_key(enabled, json!(true));
    app.config.set_key(x, json!(corner.0));
    app.config.set_key(y, json!(corner.1));
}

#[test]
fn toolbar_clears_plate() {
    for target in FLOATS {
        for corner in CORNERS {
            let mut app = app_at(LOWEST);
            place_at(&mut app, target, corner);
            let mut app = open(app);
            app.panels.arrange.as_mut().unwrap().editor.select(target);
            settle(&mut app);
            let plate = rect(&app, target);
            let plates = app.panels.arrange.as_ref().unwrap().editor.plates();
            let tools: Vec<_> = plates.iter().filter(|(id, _)| id.starts_with("tool:")).collect();
            assert_eq!(tools.len(), 2, "{target:?} {corner:?}");
            for (id, tool) in tools {
                assert!(!plate.overlaps(*tool), "{id} {target:?} {corner:?} {plate:?} {tool:?}");
                assert!(tool.x >= 0.0 && tool.right() <= LOWEST.0, "{id} {tool:?}");
                assert!(tool.y >= 0.0 && tool.bottom() <= LOWEST.1, "{id} {tool:?}");
            }
        }
    }
}

#[test]
fn thread_clears_floats() {
    let viewport = VIEWPORTS[1];
    let mut app = app_at(viewport);
    for target in FLOATS {
        place_at(&mut app, target, (50.0, 96.0));
    }
    let app = open(app);
    let editor = &app.panels.arrange.as_ref().unwrap().editor;
    let spans = &editor.spool.laid.as_ref().unwrap().spans;
    assert!(!spans.is_empty());
    for (id, plate) in editor.plates() {
        for target in FLOATS {
            assert!(!plate.overlaps(rect(&app, target)), "{id} {target:?}");
        }
    }
}

fn stored(app: &App) -> serde_json::Map<String, Value> {
    let profile = &app.config.document()[skwd_config::picker_layout::ROOT]["outputs"]["DP-1"];
    profile["values"].as_object().cloned().unwrap_or_default()
}

#[test]
fn saved_layout_omits_floats() {
    let mut app = app_at(LOWEST);
    let size = Size { width: LOWEST.0, height: LOWEST.1 };
    app.config.select_layout_output(Some(String::from("DP-1")), size);
    app.config.set_key(filter_bar::POSITION_X, json!(10));
    let values = stored(&app);
    assert!(values.contains_key(filter_bar::POSITION_X));
    for key in NEW_KEYS {
        assert!(!values.contains_key(key), "{key}");
    }
    let profile = &app.config.document()[skwd_config::picker_layout::ROOT]["outputs"]["DP-1"];
    assert!(profile["positions"].as_object().unwrap().is_empty());
}

#[test]
fn moved_float_saved_alone() {
    let mut app = begin(LOWEST);
    let from = grab(&app, Target::Arrange);
    drag(&mut app, from, [900.0, 200.0]);
    let values = stored(&app);
    assert!(values.contains_key(filter_bar::ARRANGE_POSITION_X));
    assert!(!values.contains_key(filter_bar::SWATCH_POSITION_X));
    assert!(!values.contains_key(filter_bar::SHOW_SWATCH));
}

#[test]
fn reset_unsets_floats() {
    let mut app = begin(LOWEST);
    let from = grab(&app, Target::Arrange);
    drag(&mut app, from, [900.0, 200.0]);
    let moved = rect(&app, Target::Arrange);
    click(&mut app, moved.centre());
    let reset = plate(&app, "tool:Reset");
    click(&mut app, reset);
    let values = stored(&app);
    for key in NEW_KEYS {
        assert!(!values.contains_key(key), "{key}");
    }
}

#[test]
fn profile_without_floats_resets() {
    let mut app = app_at(LOWEST);
    let size = Size { width: LOWEST.0, height: LOWEST.1 };
    let portrait = Size { width: size.height, height: size.width };
    app.config.set_layout_profile("DP-2", &LayoutProfile::capture(&json!({}), portrait)).unwrap();
    app.config.select_layout_output(Some(String::from("DP-1")), size);
    place_at(&mut app, Target::Arrange, (30.0, 40.0));
    place_at(&mut app, Target::Swatch, (60.0, 20.0));
    let position = Position {
        reference: Reference::Screen,
        alignment: Alignment::TopLeft,
        offset: Offset { x: 5.0, y: 5.0 },
    };
    app.config.set_position(Component::Arrange, Some(position)).unwrap();
    assert!(app.config.arrange_position().is_some() && app.config.swatch_position().is_some());
    assert!(app.config.position(Component::Arrange).is_some());
    app.config.select_layout_output(Some(String::from("DP-2")), portrait);
    assert!(app.config.arrange_position().is_none());
    assert!(app.config.swatch_position().is_none());
    assert!(app.config.position(Component::Arrange).is_none());
    app.config.select_layout_output(Some(String::from("DP-1")), size);
    assert_eq!(app.config.arrange_position(), Some((0.3, 0.4)));
}

#[test]
fn off_leaves_target() {
    let mut app = app_at(LOWEST);
    app.config.set_key(filter_bar::SHOW_ARRANGE, json!(false));
    app.config.set_key(filter_bar::SHOW_SWATCH, json!(false));
    for target in FLOATS {
        assert!(!placement::present(&app.config, target));
    }
    assert!(crate::app::swatch_overlay(&app).is_none());
    edit(&mut app, ArrangeMsg::Begin);
    let session = app.panels.arrange.as_ref().unwrap();
    assert_eq!(session.editor.targets(), [Target::Picker, Target::Bar, Target::Search]);
}

#[test]
fn theme_off_drops_strip() {
    let mut app = app_at(LOWEST);
    app.config.set_key(skwd_config::keys::theme::BACKEND, json!("off"));
    assert!(!placement::present(&app.config, Target::Swatch));
    assert!(placement::present(&app.config, Target::Arrange));
    edit(&mut app, ArrangeMsg::Begin);
    let session = app.panels.arrange.as_ref().unwrap();
    let beads: Vec<&str> = session
        .editor
        .board
        .shelves
        .iter()
        .flat_map(|shelf| &shelf.beads)
        .map(|bead| bead.id.as_str())
        .collect();
    assert!(!beads.contains(&"swatch"));
    assert!(beads.contains(&"arrange"));
}

#[test]
fn settings_open_when_off() {
    let mut app = app_at(LOWEST);
    app.panels.settings.open = true;
    let off = SettingsMsg::Toggle(filter_bar::SHOW_ARRANGE.to_owned(), false);
    let _ = update(&mut app, Message::Settings(off));
    assert!(!placement::present(&app.config, Target::Arrange));
    assert!(!crate::app::view::entry_shown(&app, true, true));
    let run = SettingsMsg::Run(ActionId::ArrangePicker);
    let _ = update(&mut app, Message::Settings(run));
    assert!(app.panels.arrange.is_some());
    assert!(!app.panels.settings.open);
    assert!(!app.panels.arrange.as_ref().unwrap().editor.targets().contains(&Target::Arrange));
}

#[test]
fn drag_stores_position() {
    for target in FLOATS {
        let mut app = begin(LOWEST);
        let (enabled, x, y) = placement::position_keys(target);
        assert!(!app.config.flag_default_config(enabled));
        let goal = [700.0, 300.0];
        let from = grab(&app, target);
        drag(&mut app, from, goal);
        assert!(app.config.flag_default_config(enabled));
        assert!(app.config.num_path(x) > 0.0 && app.config.num_path(y) > 0.0);
        let placed = rect(&app, target);
        assert!((placed.y - (goal[1] - rect(&app, target).h / 2.0)).abs() < 11.0, "{placed:?}");
        let position = app.config.position(placement::component(target)).expect("position");
        assert_eq!(position.reference, Reference::Screen);
    }
}

#[test]
fn placement_survives_reload() {
    let mut app = begin(LOWEST);
    let from = grab(&app, Target::Arrange);
    drag(&mut app, from, [900.0, 200.0]);
    let from = grab(&app, Target::Swatch);
    drag(&mut app, from, [500.0, 420.0]);
    let placed = FLOATS.map(|target| rect(&app, target));
    edit(&mut app, ArrangeMsg::Done);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
    let saved: Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    let mut reloaded = app_at(LOWEST);
    reloaded.config = Config::from_data(saved);
    let size = Size { width: LOWEST.0, height: LOWEST.1 };
    reloaded.config.select_layout_output(Some(String::from("DP-1")), size);
    assert_eq!(FLOATS.map(|target| rect(&reloaded, target)), placed);
}

#[test]
fn percent_placement_reloads() {
    let mut app = app_at(LOWEST);
    let (enabled, x, y) = placement::position_keys(Target::Swatch);
    app.config.set_key(enabled, json!(true));
    app.config.set_key(x, json!(50));
    app.config.set_key(y, json!(25));
    app.config.persist();
    let saved: Value =
        serde_json::from_slice(&std::fs::read(&app.config.config_path).unwrap()).unwrap();
    let mut reloaded = app_at(LOWEST);
    reloaded.config = Config::from_data(saved);
    let placed = rect(&reloaded, Target::Swatch);
    assert_eq!([placed.x, placed.y], [LOWEST.0 / 2.0, LOWEST.1 / 4.0]);
}

#[test]
fn placement_clamps_screen() {
    for viewport in [LOWEST, (800.0, 600.0), (3840.0, 2160.0)] {
        for target in FLOATS {
            let mut app = app_at(viewport);
            let (enabled, x, y) = placement::position_keys(target);
            app.config.set_key(enabled, json!(true));
            app.config.set_key(x, json!(100));
            app.config.set_key(y, json!(100));
            let placed = rect(&app, target);
            assert!(placed.x >= 0.0 && placed.right() <= viewport.0, "{placed:?}");
            assert!(placed.y >= 0.0 && placed.bottom() <= viewport.1, "{placed:?}");
        }
    }
}

#[test]
fn offset_clamps_smaller_screen() {
    let mut app = begin(LOWEST);
    let from = grab(&app, Target::Arrange);
    drag(&mut app, from, [LOWEST.0 - 10.0, LOWEST.1 - 10.0]);
    let position = app.config.position(Component::Arrange).expect("position");
    assert_eq!(position.alignment, Alignment::BottomRight);
    app.scene.viewport = (800.0, 600.0);
    let size = Size { width: 800.0, height: 600.0 };
    app.config.select_layout_output(Some(String::from("DP-1")), size);
    let placed = rect(&app, Target::Arrange);
    assert!(placed.right() <= 800.0 && placed.bottom() <= 600.0, "{placed:?}");
}

#[test]
fn hide_then_undo() {
    let mut app = begin(LOWEST);
    let from = grab(&app, Target::Swatch);
    click(&mut app, from);
    let hide = plate(&app, "tool:Hide");
    click(&mut app, hide);
    assert!(!placement::present(&app.config, Target::Swatch));
    let session = app.panels.arrange.as_ref().unwrap();
    assert!(!session.editor.targets().contains(&Target::Swatch));
    assert_eq!(session.history.entries().len(), 2);
    edit(&mut app, ArrangeMsg::Undo);
    settle(&mut app);
    assert!(placement::present(&app.config, Target::Swatch));
    assert!(app.panels.arrange.as_ref().unwrap().editor.targets().contains(&Target::Swatch));
}

#[test]
fn reset_restores_placement() {
    let mut app = begin(LOWEST);
    let start = rect(&app, Target::Arrange);
    let other = rect(&app, Target::Swatch);
    let from = grab(&app, Target::Arrange);
    drag(&mut app, from, [800.0, 300.0]);
    let moved = rect(&app, Target::Arrange);
    assert_ne!(moved, start);
    click(&mut app, moved.centre());
    let reset = plate(&app, "tool:Reset");
    click(&mut app, reset);
    assert_eq!(rect(&app, Target::Arrange), start);
    assert_eq!(rect(&app, Target::Swatch), other);
}

#[test]
fn cancel_restores_keys() {
    let mut app = begin(LOWEST);
    let start = rect(&app, Target::Swatch);
    let from = grab(&app, Target::Swatch);
    drag(&mut app, from, [800.0, 300.0]);
    edit(&mut app, ArrangeMsg::Cancel);
    settle(&mut app);
    assert!(app.panels.arrange.is_none());
    assert!(app.config.swatch_position().is_none());
    assert_eq!(rect(&app, Target::Swatch), start);
}

#[test]
fn display_review_fits() {
    let mut app = begin(LOWEST);
    let from = grab(&app, Target::Arrange);
    drag(&mut app, from, [LOWEST.0 - 10.0, LOWEST.1 - 10.0]);
    let smaller = crate::contracts::daemon::OutputStatus {
        name: String::from("DP-2"),
        target: String::from("DP-2"),
        connected: true,
        width: 800,
        height: 600,
        logical_width: 800,
        logical_height: 600,
        ..crate::contracts::daemon::OutputStatus::default()
    };
    app.daemon.output_statuses.push(smaller);
    edit(&mut app, ArrangeMsg::OpenDisplays);
    edit(&mut app, ArrangeMsg::ReviewDisplay(String::from("DP-2")));
    edit(&mut app, ArrangeMsg::FitDisplay);
    let session = app.panels.arrange.as_ref().unwrap();
    let review = session.display_copy.as_ref().unwrap().review.as_ref().unwrap();
    let placed = review.view.after.arrange.expect("arrange plate");
    assert!(placed.x + placed.width <= 800.0 && placed.y + placed.height <= 600.0, "{placed:?}");
    assert!(review.view.after.swatch.is_some());
}

const OUT: &str = "SKWD_FLOATS_OUT";

fn shoot(app: &App, out: &str, name: &str) {
    let size = iced::Size::new(app.scene.viewport.0, app.scene.viewport.1);
    let path = format!("{out}/{name}.png");
    crate::frontend::headless::capture(size, crate::frontend::ui::UI_FONT, &path, || {
        crate::app::view::view_single(app)
    });
}

#[test]
#[ignore = "manual offscreen render of the floating controls"]
fn render_floats() {
    let out = std::env::var(OUT).expect(OUT);
    let mut app = app_at(LOWEST);
    std::sync::Arc::make_mut(&mut app.scene.render).vis = 1.0;
    shoot(&app, &out, "picker-default");
    app.config.set_key(filter_bar::ORIENTATION, json!("vertical"));
    app.sync_filter_bar_chrome();
    app.chrome.filter_bar_visible = true;
    app.chrome.filter_bar_reveal.retarget(1.0);
    for _ in 0..SETTLE_TICKS {
        app.chrome.tick_filter_bar(TICK);
    }
    shoot(&app, &out, "picker-vertical");
    app.config.set_key(filter_bar::ORIENTATION, json!("horizontal"));
    app.config.set_key(filter_bar::SHOW_ARRANGE, json!(false));
    shoot(&app, &out, "picker-arrange-off");
    app.config.set_key(filter_bar::SHOW_SWATCH, json!(false));
    shoot(&app, &out, "picker-both-off");
    let mut app = begin(LOWEST);
    std::sync::Arc::make_mut(&mut app.scene.render).vis = 1.0;
    shoot(&app, &out, "editor-default");
    let from = grab(&app, Target::Swatch);
    click(&mut app, from);
    shoot(&app, &out, "editor-swatch-selected");
    drag(&mut app, from, [700.0, 300.0]);
    let from = grab(&app, Target::Arrange);
    click(&mut app, from);
    shoot(&app, &out, "editor-arrange-selected");
    drag(&mut app, from, [1200.0, 140.0]);
    shoot(&app, &out, "editor-moved");
    let at = rect(&app, Target::Arrange).centre();
    click(&mut app, at);
    let hide = plate(&app, "tool:Hide");
    click(&mut app, hide);
    shoot(&app, &out, "editor-arrange-hidden");
}

#[test]
fn old_profile_keeps_hidden() {
    let mut app = app_at(LOWEST);
    let size = Size { width: LOWEST.0, height: LOWEST.1 };
    app.config.set_layout_profile("DP-1", &LayoutProfile::capture(&json!({}), size)).unwrap();
    app.config.set_key(filter_bar::SHOW_ARRANGE, json!(false));
    place_at(&mut app, Target::Arrange, (30.0, 40.0));
    app.config.select_layout_output(Some(String::from("DP-1")), size);
    assert!(!placement::present(&app.config, Target::Arrange));
    assert!(app.config.arrange_position().is_none());
}

#[test]
fn vertical_bar_clears_floats() {
    for viewport in VIEWPORTS {
        let mut app = app_at(viewport);
        app.config.set_key(filter_bar::ORIENTATION, json!("vertical"));
        app.sync_filter_bar_chrome();
        for _ in 0..SETTLE_TICKS {
            app.chrome.tick_filter_bar(TICK);
        }
        assert!(app.chrome.filter_bar_vertical);
        let bar = rect(&app, Target::Bar);
        for target in FLOATS {
            assert!(!bar.overlaps(rect(&app, target)), "{viewport:?} {bar:?} {target:?}");
        }
    }
}
