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
