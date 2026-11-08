use super::*;
use iced::event::Status;
use iced::keyboard::key::Named;
use skwd_config::keys::selector::LOOP_AROUND;

fn looping_app(count: i64, mode: Mode, on: bool) -> App {
    let mut app = test_app();
    let walls: Vec<Value> =
        (0..count).map(|idx| wall(&format!("w{idx}"), "static", idx % 360, idx)).collect();
    seed(&mut app, &walls);
    app.scene.mode = mode;
    app.scene.gp.cols = 5;
    app.scene.hp.rows = 3;
    app.config.set_key(LOOP_AROUND, json!(on));
    app
}

fn press(app: &mut App, keys: &[Message]) -> usize {
    for key in keys {
        let _ = update(app, key.clone());
    }
    app.scene.current
}

#[test]
fn edges_by_mode() {
    let last = Message::KeyEdge { last: true };
    let cases = [
        (Mode::Slices, vec![Message::KeyPrev], 0, 11),
        (Mode::Slices, vec![last.clone(), Message::KeyNext], 11, 0),
        (Mode::Grid, vec![Message::KeyUp], 0, 10),
        (Mode::Grid, vec![last.clone(), Message::KeyDown], 11, 1),
        (Mode::Hex, vec![Message::KeyUp], 0, 2),
        (Mode::Hex, vec![last.clone(), Message::KeyDown], 11, 9),
        (Mode::Hex, vec![Message::Wheel(1.0)], 0, 9),
        (Mode::Slices, vec![Message::Wheel(1.0)], 0, 11),
        (Mode::Slices, vec![last.clone(), last, Message::KeyPage { backwards: false }], 11, 0),
        (Mode::Slices, vec![Message::KeyEdge { last: false }], 0, 0),
    ];
    for (mode, keys, off, on) in cases {
        assert_eq!(press(&mut looping_app(12, mode, false), &keys), off, "{mode:?} off");
        assert_eq!(press(&mut looping_app(12, mode, true), &keys), on, "{mode:?} on");
    }
}

fn key(app: &mut App, event: iced::Event) -> usize {
    let message = on_event(event, Status::Ignored, iced::window::Id::unique()).unwrap();
    press(app, &[message])
}

#[test]
fn held_keys_stop() {
    let mut app = looping_app(10, Mode::Slices, true);
    let left = keyboard::Key::Named(Named::ArrowLeft);
    let release = iced::Event::Keyboard(keyboard::Event::KeyReleased {
        key: left.clone(),
        modified_key: left.clone(),
        physical_key: keyboard::key::Physical::Unidentified(
            keyboard::key::NativeCode::Unidentified,
        ),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
    });
    assert_eq!(key(&mut app, key_press(left.clone(), false)), 9);
    assert_eq!(press(&mut app, &[Message::KeyNext]), 0);
    assert_eq!(key(&mut app, key_press(left.clone(), false)), 0);
    assert_eq!(press(&mut app, &[Message::KeyPrev]), 9);
    assert_eq!(press(&mut app, &[Message::KeyNext]), 0);
    assert_eq!(key(&mut app, release), 0);
    assert_eq!(key(&mut app, key_press(left, false)), 9);
}

#[test]
fn wheel_bursts_stop() {
    let mut app = looping_app(10, Mode::Slices, true);
    assert_eq!(press(&mut app, &[Message::Wheel(1.0)]), 9);
    assert_eq!(press(&mut app, &[Message::Wheel(-1.0)]), 9);
    app.input.last_wheel = None;
    assert_eq!(press(&mut app, &[Message::Wheel(-1.0)]), 0);
    app.input.last_wheel = None;
    assert_eq!(press(&mut app, &[Message::Wheel(0.5), Message::Wheel(0.5)]), 9);
}

fn session(app: &mut App) -> &mut crate::frontend::browser::BrowserSessionState {
    &mut app.source_browser.browser.as_mut().unwrap().session
}

#[test]
fn browser_loops_when_loaded() {
    let mut app = looping_app(0, Mode::Slices, true);
    app.scene.viewport = (1280.0, 720.0);
    let mut browser = Browser::new(Source::Wallhaven);
    for idx in 0..12 {
        browser.session.items.push(browser_item(&format!("w{idx}")));
    }
    app.source_browser.browser = Some(browser);
    app.config.set_key(LOOP_AROUND, json!(false));
    let _ = press(&mut app, &[Message::KeyNext, Message::KeyPrev]);
    assert_eq!(session(&mut app).hover, Some(0));
    app.config.set_key(LOOP_AROUND, json!(true));
    let _ = press(&mut app, &[Message::KeyPrev]);
    assert_eq!(session(&mut app).hover, Some(11));
    session(&mut app).loading = true;
    let _ = press(&mut app, &[Message::KeyNext]);
    assert_eq!(session(&mut app).hover, Some(11));
    session(&mut app).loading = false;
    session(&mut app).last_page = 2;
    let _ = press(&mut app, &[Message::KeyNext]);
    assert_eq!(session(&mut app).hover, Some(11));
}

const GRID_WALLS: i64 = 40;
const GRID_LAST: usize = 39;
const SMALL_WALLS: i64 = 5;
const SMALL_LAST: usize = 4;
const RING_WALLS: i64 = 400;
const RING_LAST: usize = 399;

fn settled(count: i64, mode: Mode, on: bool) -> App {
    let mut app = looping_app(count, mode, on);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    app
}

fn rest(app: &mut App) {
    app.input.last_wheel = None;
}

#[test]
fn connected_wraps_held() {
    for mode in [Mode::Slices, Mode::Grid] {
        let mut app = settled(RING_WALLS, mode, true);
        assert!(app.scene.connected(), "{mode:?}");
        let left = keyboard::Key::Named(Named::ArrowLeft);
        app.scene.set_current(1, RING_WALLS as usize);
        assert_eq!(key(&mut app, key_press(left.clone(), false)), 0, "{mode:?}");
        assert_eq!(key(&mut app, key_press(left, false)), RING_LAST, "{mode:?}");
    }
}

#[test]
fn hex_column_held_stops() {
    let mut app = settled(RING_WALLS, Mode::Hex, true);
    assert!(app.scene.connected());
    app.scene.set_current(1, RING_WALLS as usize);
    let up = keyboard::Key::Named(Named::ArrowUp);
    assert_eq!(key(&mut app, key_press(up.clone(), false)), 0);
    assert_eq!(key(&mut app, key_press(up, false)), 0);
}

#[test]
fn connected_wheel_bursts() {
    let mut app = settled(RING_WALLS, Mode::Slices, true);
    assert_eq!(press(&mut app, &[Message::Wheel(1.0)]), RING_LAST);
    assert_eq!(press(&mut app, &[Message::Wheel(-1.0)]), 0);
    assert_eq!(press(&mut app, &[Message::Wheel(1.0)]), RING_LAST);
}

#[test]
fn grid_wheel_scrolls_round() {
    for on in [false, true] {
        let mut app = looping_app(RING_WALLS, Mode::Grid, on);
        let mut now = Instant::now();
        tick_frames(&mut app, &mut now, 60);
        assert_eq!(press(&mut app, &[Message::Wheel(1.0)]), 0, "loop {on}");
        tick_frames(&mut app, &mut now, 60);
        assert_eq!(app.scene.camera_pos() > 0.0, on, "loop {on}");
    }
}

#[test]
fn small_grid_keeps_ends() {
    let mut app = settled(SMALL_WALLS, Mode::Grid, true);
    assert!(!app.scene.connected());
    assert_eq!(press(&mut app, &[Message::Wheel(1.0)]), 0);
    assert_eq!(press(&mut app, &[Message::KeyPrev]), SMALL_LAST);
}

fn browser_wheel(amount: f32) -> Message {
    Message::Browser(crate::frontend::browser::BrowserMsg::WallInput(
        crate::frontend::ui::BrowserWallInput::Wheel(amount),
    ))
}

#[test]
fn browser_wheel_loops() {
    let mut app = looping_app(0, Mode::Slices, true);
    app.scene.viewport = (1280.0, 720.0);
    let mut browser = Browser::new(Source::Wallhaven);
    for idx in 0..GRID_WALLS {
        browser.session.items.push(browser_item(&format!("w{idx}")));
    }
    app.source_browser.wall.rebuild_catalogue(&browser, true);
    app.source_browser.browser = Some(browser);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 60);
    let _ = update(&mut app, browser_wheel(-1.0));
    assert_eq!(session(&mut app).hover, None);
    rest(&mut app);
    let _ = update(&mut app, browser_wheel(1.0));
    assert_eq!(session(&mut app).hover, None);
    rest(&mut app);
    let _ = update(&mut app, browser_wheel(1.0));
    assert_eq!(session(&mut app).hover, Some(GRID_LAST));
    tick_frames(&mut app, &mut now, 60);
    rest(&mut app);
    let _ = update(&mut app, browser_wheel(-1.0));
    assert_eq!(session(&mut app).hover, Some(0));
}
