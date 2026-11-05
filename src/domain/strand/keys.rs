const NUDGE: f32 = 1.0;
const NUDGE_FAR: f32 = 10.0;
const UNDO: char = 'z';
const REDO: char = 'y';
const DEPTH: char = 'b';
const PREVIOUS: char = '[';
const NEXT: char = ']';
const GROW: [char; 2] = ['+', '='];
const SHRINK: [char; 3] = ['-', '_', '\u{2212}'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Escape,
    Tab,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Delete,
    Backspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    pub key: Key,
    pub control: bool,
    pub shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Undo,
    Redo,
    Deselect,
    Cancel,
    Tab(bool),
    Nudge([f32; 2]),
    Detent(bool),
    Depth,
    Style(bool),
    Raise,
    Lower,
    Remove,
}

pub fn command(chord: Chord, selected: bool) -> Option<Command> {
    let Chord { key, control, shift } = chord;
    if control {
        return match key {
            Key::Char(letter) if letter.eq_ignore_ascii_case(&UNDO) => {
                Some(if shift { Command::Redo } else { Command::Undo })
            }
            Key::Char(letter) if letter.eq_ignore_ascii_case(&REDO) => Some(Command::Redo),
            _ => None,
        };
    }
    match key {
        Key::Escape => Some(if selected { Command::Deselect } else { Command::Cancel }),
        Key::Tab => Some(Command::Tab(!shift)),
        _ if !selected => None,
        Key::Left | Key::Right | Key::Up | Key::Down => {
            let step = if shift { NUDGE_FAR } else { NUDGE };
            let offset = match key {
                Key::Left => [-step, 0.0],
                Key::Right => [step, 0.0],
                Key::Up => [0.0, -step],
                _ => [0.0, step],
            };
            Some(Command::Nudge(offset))
        }
        Key::PageUp => Some(Command::Raise),
        Key::PageDown => Some(Command::Lower),
        Key::Delete | Key::Backspace => Some(Command::Remove),
        Key::Char(letter) if GROW.contains(&letter) => Some(Command::Detent(true)),
        Key::Char(letter) if SHRINK.contains(&letter) => Some(Command::Detent(false)),
        Key::Char(letter) if letter.eq_ignore_ascii_case(&DEPTH) => Some(Command::Depth),
        Key::Char(PREVIOUS) => Some(Command::Style(false)),
        Key::Char(NEXT) => Some(Command::Style(true)),
        Key::Char(_) => None,
    }
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod tests;
