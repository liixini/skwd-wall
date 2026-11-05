use skwd_palette::studio::{Area, Role};

use crate::domain::studio::{Group, ModeChoice};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    View,
    Warmth,
    Colour,
    Contrast,
    FineTune,
    Shade,
    Compare,
}

impl Row {
    pub const ALL: [Self; 7] = [
        Self::View,
        Self::Warmth,
        Self::Colour,
        Self::Contrast,
        Self::FineTune,
        Self::Shade,
        Self::Compare,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn cycled(self, dir: i8) -> Self {
        Self::ALL[cycle(self.index(), dir, Self::ALL.len())]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Desktop,
    Browser,
    Chat,
    Music,
    Code,
}

impl View {
    pub const ALL: [Self; 5] = [Self::Desktop, Self::Browser, Self::Chat, Self::Music, Self::Code];
}

pub const TUNABLE: [Role; 7] =
    [Role::Base, Role::Panel, Role::Text, Role::Quiet, Role::Accent, Role::Second, Role::Third];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Knob {
    Warmth,
    Colour,
    Contrast,
}

const KNOBS: [(Knob, Row); 3] =
    [(Knob::Warmth, Row::Warmth), (Knob::Colour, Row::Colour), (Knob::Contrast, Row::Contrast)];

impl Knob {
    pub const ALL: [Self; 3] = [Self::Warmth, Self::Colour, Self::Contrast];

    pub fn row(self) -> Row {
        KNOBS.iter().find(|(knob, _)| *knob == self).map_or(Row::Warmth, |(_, row)| *row)
    }

    pub fn of(row: Row) -> Option<Self> {
        KNOBS.iter().find(|(_, item)| *item == row).map(|(knob, _)| *knob)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plate {
    Close,
    Mode(ModeChoice),
    Use,
    SaveAs,
    Save,
    Cancel,
    ClearArea,
    CancelPick,
    Undo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hit {
    Plate(Plate),
    Tab(Row),
    Pop,
    Option(Row, usize),
    Section(Group),
    Kid(String),
    Saved(usize),
    Value(Row),
    Node(i8),
    Track(Knob),
}

impl Hit {
    pub fn in_bar(&self) -> bool {
        matches!(
            self,
            Self::Tab(_)
                | Self::Value(_)
                | Self::Pop
                | Self::Option(..)
                | Self::Track(_)
                | Self::Node(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Down,
    Drag,
    Up,
    Lost,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Insert(String),
    Erase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StudioMsg {
    Close,
    Apply,
    Undo,
    Pick(String),
    PickSaved(usize),
    OpenGroup(Group),
    Mode(ModeChoice),
    Row(Row),
    Step(Row, i8, bool),
    Slide(Knob, f32, Phase),
    Notch(Knob, i32),
    Reset(Knob),
    Choose(Row, usize),
    Shade(i8),
    Compare,
    Split(f32, Phase),
    Scroll(Group, f32),
    SchemeStep(i8),
    Hover(Option<Hit>),
    Press(Option<Plate>),
    SaveAs,
    Save,
    CancelSave,
    Name(Edit),
    StartPick,
    CancelPick,
    ClearArea,
    Drag(Area, Phase),
    SkipIntro,
}

pub fn cycle(index: usize, dir: i8, len: usize) -> usize {
    (index as isize + isize::from(dir)).rem_euclid(len as isize) as usize
}
