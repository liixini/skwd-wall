use paper_control::{ClockStyle, Depth, Face};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Clock,
}

pub const KINDS: [Kind; 1] = [Kind::Clock];

const CLOCKS: [Face; 5] = [
    Face::Clock(ClockStyle::Aperture),
    Face::Clock(ClockStyle::ApertureRing),
    Face::Clock(ClockStyle::Greeter),
    Face::Clock(ClockStyle::ControlCenter),
    Face::Clock(ClockStyle::DayDial),
];

impl Kind {
    pub const fn of(face: Face) -> Self {
        match face {
            Face::Clock(_) => Self::Clock,
        }
    }

    pub const fn styles(self) -> &'static [Face] {
        match self {
            Self::Clock => &CLOCKS,
        }
    }

    pub const fn depth(self) -> Depth {
        match self {
            Self::Clock => Depth::Behind,
        }
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Clock => 0,
        }
    }
}

const _: () = assert!(CLOCKS.len() == ClockStyle::DayDial as usize + 1);

pub const fn style_index(face: Face) -> usize {
    let Face::Clock(style) = face;
    match style {
        ClockStyle::Aperture
        | ClockStyle::ApertureRing
        | ClockStyle::Greeter
        | ClockStyle::ControlCenter
        | ClockStyle::DayDial => style as usize,
    }
}

#[cfg(test)]
#[path = "catalogue_tests.rs"]
mod tests;
