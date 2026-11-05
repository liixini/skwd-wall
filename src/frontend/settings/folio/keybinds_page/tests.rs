use iced::{Event, mouse};

use crate::app::Message;
use crate::contracts::picker::KEY_BINDINGS;
use crate::domain::input::InputMap;
use crate::domain::input::{MouseButton, parse_binding};
use crate::frontend::settings::SettingsMsg;
use crate::frontend::settings::folio::sections::Paint;
use crate::frontend::settings::place;
use crate::frontend::settings::test_source::{FakeSettingsSource, place_source};

use super::{Chip, ChipCell, KeyLook, KeysCtx, Side, chip_cell, chip_split, fitted_chips, groups};

const ROOM: f32 = 142.0;

#[test]
fn whole_heads_first() {
    let tiers = [[60.0, 50.0], [60.0, 40.0], [50.0, 40.0]];
    assert_eq!(fitted_chips(tiers, ROOM), [60.0, 50.0]);
}

#[test]
fn heads_wrap_next() {
    let tiers = [[60.0, 90.0], [60.0, 62.0], [50.0, 62.0]];
    assert_eq!(fitted_chips(tiers, ROOM), [60.0, 62.0]);
}

#[test]
fn chips_split_last_resort() {
    let tiers = [[125.0, 90.0], [125.0, 62.0], [72.0, 62.0]];
    assert_eq!(fitted_chips(tiers, ROOM), [72.0, 62.0]);
}

#[test]
fn chips_split_last() {
    let split = |label| chip_split(label, KeyLook::Bound);
    assert_eq!(split("Ctrl+Shift+Right-click"), Some(("Ctrl+Shift+", "Right-click")));
    assert_eq!(split("Page Down"), None);
    assert_eq!(split("+"), None);
    assert_eq!(split("Ctrl++"), None);
    assert_eq!(chip_split("Press a key", KeyLook::Listening), Some(("Press a ", "key")));
}

fn unbound() -> InputMap {
    InputMap::from_bindings(
        KEY_BINDINGS.into_iter().map(|entry| (entry.action, String::from("none"))),
    )
}

#[test]
fn no_chip_blank() {
    for map in [InputMap::default(), unbound()] {
        let cfg = FakeSettingsSource::default().with_bindings(map.clone());
        let section = place_source(&cfg).place(place::KEYBINDS).sections.remove(0);
        for group in groups(&section, &map, KeysCtx::default()) {
            for line in group.lines {
                assert!(!line.default_text.is_empty(), "{}", line.title);
                assert!(line.chips.iter().all(|chip| !chip.label.is_empty()), "{}", line.title);
            }
        }
    }
}

const CHIP_PATH: &str = "keys.favourite";
const CHIP_SPOT: iced::Point = iced::Point::new(4.0, 6.0);

fn press_chip(look: KeyLook, buttons: &[mouse::Button]) -> Vec<Message> {
    use std::collections::{HashMap, HashSet};

    use crate::frontend::animation::MotionProfile;
    use crate::frontend::settings::{CalmCtx, FocusCtx, SettingsFocus};
    use crate::frontend::ui::calm::render_tests::board;
    use iced_runtime::user_interface::{Cache, UserInterface};

    let values = HashMap::new();
    let folds = HashSet::new();
    let bars = HashMap::new();
    let palette = board();
    let paint = Paint {
        values: &values,
        focus: FocusCtx {
            keyboard_focus: SettingsFocus::Index,
            focused_control: 0,
            focused_choice: None,
            expanded_details: &folds,
            bar_reveals: &bars,
            active_input: None,
            armed: None,
        },
        calm: CalmCtx {
            open: None,
            section: 0,
            busy: 0.0,
            pop: None,
            edit: None,
            instant: true,
            closing: None,
        },
        scale: 1.0,
        palette: &palette,
        motion: MotionProfile::default(),
        body: 600.0,
        dim: false,
        reading: 600.0,
        owner: 0,
        undivided: false,
        trim: crate::frontend::settings::Trim::None,
    };
    let chip = Chip { label: String::from("Press a key"), look };
    let cell = ChipCell {
        chip,
        slot: 0,
        width: 120.0,
        path: CHIP_PATH,
        title: "Favourite card",
        default_text: "F",
        changed: false,
        side: Side::BelowEnd,
        focused: false,
        palette,
        plane: palette.surface,
        paint,
    };
    let mut renderer = crate::frontend::headless::renderer(crate::frontend::ui::UI_FONT);
    let screen = iced::Size::new(300.0, 200.0);
    let mut ui = UserInterface::build(chip_cell(cell), screen, Cache::default(), &mut renderer);
    let cursor = mouse::Cursor::Available(CHIP_SPOT);
    let mut events = vec![Event::Mouse(mouse::Event::CursorMoved { position: CHIP_SPOT })];
    for button in buttons {
        events.push(Event::Mouse(mouse::Event::ButtonPressed(*button)));
        events.push(Event::Mouse(mouse::Event::ButtonReleased(*button)));
    }
    let mut messages = Vec::new();
    let _ = ui.update(
        &events,
        cursor,
        &mut renderer,
        &mut iced::advanced::clipboard::Null,
        &mut messages,
    );
    messages
}

#[test]
#[ignore = "needs a Vulkan adapter"]
fn listening_chip_binds_clicks() {
    let clicks = press_chip(
        KeyLook::Listening,
        &[mouse::Button::Left, mouse::Button::Right, mouse::Button::Middle],
    );
    let bound: Vec<_> = clicks
        .iter()
        .map(|message| match message {
            Message::Settings(SettingsMsg::KeybindCaptureClick(button)) => Some(*button),
            _ => None,
        })
        .collect();
    assert_eq!(
        bound,
        [Some(MouseButton::Left), Some(MouseButton::Right), Some(MouseButton::Middle)]
    );
}

#[test]
#[ignore = "needs a Vulkan adapter"]
fn opening_click_only_opens() {
    let messages = press_chip(KeyLook::Bound, &[mouse::Button::Left]);
    assert!(matches!(
        messages.as_slice(),
        [Message::Settings(SettingsMsg::KeybindCapture(path, 0))] if path == CHIP_PATH
    ));
}

#[test]
fn unbound_shows_default() {
    let map = unbound();
    let cfg = FakeSettingsSource::default().with_bindings(map.clone());
    let section = place_source(&cfg).place(place::KEYBINDS).sections.remove(0);
    let lines: Vec<_> = groups(&section, &map, KeysCtx::default())
        .into_iter()
        .flat_map(|group| group.lines)
        .collect();
    assert_eq!(lines.len(), KEY_BINDINGS.len());
    for line in lines {
        let entry = KEY_BINDINGS.into_iter().find(|entry| entry.path == line.path).expect("entry");
        let bound =
            parse_binding(entry.action.default_binding()).is_some_and(|keys| !keys.is_empty());
        assert_eq!(line.changed, bound, "{}", line.title);
    }
}
