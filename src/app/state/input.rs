use std::time::Instant;

use crate::domain::input::{InputMap, Mods};

pub(crate) struct InputState {
    pub(crate) mods: Mods,
    pub(crate) last_activity: Instant,
    pub(crate) help_open: bool,
    pub(crate) bindings: InputMap,
    pub(crate) held: bool,
    pub(crate) down: Option<iced::keyboard::Key>,
    pub(crate) quiet: Option<iced::keyboard::Key>,
    pub(crate) last_wheel: Option<Instant>,
    pub(crate) wheel_armed: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            mods: Mods::NONE,
            last_activity: Instant::now(),
            help_open: false,
            bindings: InputMap::default(),
            held: false,
            down: None,
            quiet: None,
            last_wheel: None,
            wheel_armed: false,
        }
    }
}
