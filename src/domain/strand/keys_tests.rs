use super::{Chord, Command, Key, command};

fn chord(key: Key, control: bool, shift: bool) -> Chord {
    Chord { key, control, shift }
}

#[test]
fn keyboard_map() {
    let cases = [
        (chord(Key::Char('z'), true, false), true, Some(Command::Undo)),
        (chord(Key::Char('Z'), true, true), false, Some(Command::Redo)),
        (chord(Key::Char('y'), true, false), false, Some(Command::Redo)),
        (chord(Key::Escape, false, false), true, Some(Command::Deselect)),
        (chord(Key::Escape, false, false), false, Some(Command::Cancel)),
        (chord(Key::Tab, false, true), false, Some(Command::Tab(false))),
        (chord(Key::Left, false, false), true, Some(Command::Nudge([-1.0, 0.0]))),
        (chord(Key::Down, false, true), true, Some(Command::Nudge([0.0, 10.0]))),
        (chord(Key::Left, false, false), false, None),
        (chord(Key::Char('='), false, false), true, Some(Command::Detent(true))),
        (chord(Key::Char('\u{2212}'), false, false), true, Some(Command::Detent(false))),
        (chord(Key::Char('B'), false, true), true, Some(Command::Depth)),
        (chord(Key::Char('['), false, false), true, Some(Command::Style(false))),
        (chord(Key::Char(']'), false, false), true, Some(Command::Style(true))),
        (chord(Key::PageUp, false, false), true, Some(Command::Raise)),
        (chord(Key::PageDown, false, false), true, Some(Command::Lower)),
        (chord(Key::Backspace, false, false), true, Some(Command::Remove)),
        (chord(Key::Char('q'), false, false), true, None),
    ];
    for (chord, selected, expected) in cases {
        assert_eq!(command(chord, selected), expected, "{chord:?}");
    }
}
