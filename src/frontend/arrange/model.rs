use iced::Rectangle;

use crate::domain::strand::Rect;
use crate::frontend::ui::edit_id;

const ARRANGE_TITLE: &str = "settings-filter-show-arrange-label";
const SWATCH_TITLE: &str = "settings-filter-show-swatch-label";
pub(crate) const PERCENT: f32 = 100.0;
pub(crate) const MIN_FRAME: f32 = 96.0;
pub(crate) const MIN_WIDTH: f32 = 160.0;
pub(crate) const MAX_CELLS: usize = 20;
pub(crate) const LAYOUTS: [&str; 7] =
    ["slices", "depth", "collection", "wall", "hex", "sandy", "hand"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Target {
    Picker,
    Bar,
    Search,
    Arrange,
    Swatch,
}

impl Target {
    pub(crate) const ALL: [Self; 5] =
        [Self::Picker, Self::Bar, Self::Search, Self::Arrange, Self::Swatch];

    pub(crate) fn index(self) -> usize {
        self as usize
    }

    pub(crate) fn edit_id(self) -> Option<&'static str> {
        match self {
            Self::Picker | Self::Bar | Self::Search => None,
            Self::Arrange => Some(edit_id::ARRANGE),
            Self::Swatch => Some(edit_id::SWATCH),
        }
    }

    pub(crate) fn title_key(self) -> Option<&'static str> {
        match self {
            Self::Picker | Self::Bar | Self::Search => None,
            Self::Arrange => Some(ARRANGE_TITLE),
            Self::Swatch => Some(SWATCH_TITLE),
        }
    }

    pub(crate) fn floating(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.edit_id() == Some(id))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pointer {
    Press([f32; 2]),
    Move([f32; 2]),
    Release([f32; 2]),
    Leave,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrangeMsg {
    Begin,
    Pointer(Pointer),
    Preview,
    ConfirmDisconnected,
    ResolveConflict(bool),
    OpenDisplays,
    CloseDisplays,
    ReviewDisplay(String),
    FitDisplay,
    AllowDisconnected(bool),
    ApplyDisplay,
    SkipDisplay,
    ForgetDisplay(String),
    Undo,
    Redo,
    Cancel,
    Done,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tool {
    Prev,
    Layout,
    Next,
    Spacing,
    Corners,
    Hide,
    Reset,
}

impl Tool {
    pub(crate) fn leads(self) -> bool {
        matches!(self, Self::Spacing | Self::Reset)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Family {
    Types,
    Sort,
    Colours,
    Tools,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Bead {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) shown: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Shelf {
    pub(crate) family: Family,
    pub(crate) beads: Vec<Bead>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Button {
    pub(crate) id: String,
    pub(crate) rect: Rect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Grid {
    pub(crate) cell: [f32; 2],
    pub(crate) gap: [f32; 2],
    pub(crate) cells: [usize; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum CornerStep {
    Step(usize),
    Each,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Style {
    pub(crate) layout: &'static str,
    pub(crate) spacing: Option<usize>,
    pub(crate) corners: Option<CornerStep>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Prompt {
    Conflict,
    Disconnected(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Copy {
    pub(crate) recipients: Vec<DisplayRecipient>,
    pub(crate) review: Option<DisplayReview>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Board {
    pub(crate) rects: [Rect; Target::ALL.len()],
    pub(crate) attached: [bool; Target::ALL.len()],
    pub(crate) present: [bool; Target::ALL.len()],
    pub(crate) buttons: Vec<Button>,
    pub(crate) shelves: Vec<Shelf>,
    pub(crate) style: Style,
    pub(crate) grid: Option<Grid>,
    pub(crate) resizable: bool,
    pub(crate) steps: usize,
    pub(crate) displays: bool,
    pub(crate) copy: Option<Copy>,
    pub(crate) prompt: Option<Prompt>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Anchor {
    Screen([usize; 2]),
    Picker([usize; 2]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Move,
    Size,
    Grid,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Effect {
    Place(Target, Rect),
    Grid(Rect, [usize; 2]),
    Commit(Target, Anchor, Step),
    Nudge(Target, Rect, Anchor, bool),
    Abandon,
    Reorder(String, [f32; 2]),
    Drop,
    Hide(String),
    Show(String, Option<[f32; 2]>),
    Layout(bool),
    Spacing,
    Corners,
    Reset(Target),
    Message(ArrangeMsg),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DisplayRecipient {
    pub(crate) name: String,
    pub(crate) connected: bool,
    pub(crate) reviewable: bool,
    pub(crate) staged: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DisplayGeometry {
    pub(crate) viewport: (f32, f32),
    pub(crate) picker: Option<Rectangle>,
    pub(crate) bar: Rectangle,
    pub(crate) search: Rectangle,
    pub(crate) arrange: Option<Rectangle>,
    pub(crate) swatch: Option<Rectangle>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DisplayReview {
    pub(crate) output: String,
    pub(crate) connected: bool,
    pub(crate) before: DisplayGeometry,
    pub(crate) after: DisplayGeometry,
    pub(crate) compatible: bool,
    pub(crate) save_disconnected: bool,
}
