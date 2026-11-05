use std::collections::HashSet;

use super::ActionId;

const MASK: &str = "••••";
const MASK_SHOWN: usize = 4;
use crate::contracts::settings::ProfileState;

#[derive(Debug, Clone)]
pub enum Control {
    Display(crate::frontend::display_control::DisplayControl),
    AppTheme {
        app: crate::contracts::daemon::AppThemeStatus,
        index: usize,
    },
    Toggle {
        path: String,
        value: bool,
    },
    ToggleAction {
        id: ActionId,
        value: bool,
    },
    Colour {
        path: String,
        hex: String,
    },
    Number {
        key: String,
        path: String,
        unit: &'static str,
    },
    TextField {
        key: String,
        path: String,
        placeholder: &'static str,
        secret: bool,
    },
    Resolution {
        key: String,
        path: String,
        width_placeholder: &'static str,
        height_placeholder: &'static str,
    },
    KeyBinding {
        key: String,
        path: String,
        default: &'static str,
    },
    Dropdown {
        path: String,
        options: Vec<(String, String)>,
        current: String,
        palettes: Vec<(String, Vec<String>)>,
    },
    Chips {
        path: String,
        options: Vec<(String, String)>,
        current: String,
        disabled: Vec<String>,
        background: Option<super::background::BackgroundControl>,
    },
    ActionBtn {
        id: ActionId,
        label: String,
    },
    ActionChips {
        items: Vec<(ActionId, String)>,
    },
    Details {
        id: String,
        summary: String,
        rows: Vec<Row>,
    },
    StackBar {
        id: String,
        summary: String,
        preview: Option<String>,
        rows: Vec<Row>,
    },
    Segment,
    Static,
    Code {
        snippet: &'static str,
    },
    Preview,
}

impl Control {
    pub fn compact_bar_id(&self) -> Option<String> {
        match self {
            Self::StackBar { id, .. } => Some(id.clone()),
            _ => None,
        }
    }

    pub fn is_group_editor(&self) -> bool {
        self.compact_bar_id().is_some()
    }

    pub fn is_inline_editor(&self) -> bool {
        matches!(self, Self::Number { .. } | Self::TextField { .. } | Self::KeyBinding { .. })
    }

    pub fn is_compact_action(&self) -> bool {
        matches!(self, Self::Toggle { .. } | Self::ToggleAction { .. })
    }

    pub fn is_focusable(&self) -> bool {
        !matches!(self, Self::Segment | Self::Preview)
    }

    pub fn path(&self) -> Option<&str> {
        match self {
            Self::Toggle { path, .. }
            | Self::Colour { path, .. }
            | Self::Number { path, .. }
            | Self::TextField { path, .. }
            | Self::Resolution { path, .. }
            | Self::KeyBinding { path, .. }
            | Self::Dropdown { path, .. }
            | Self::Chips { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn ticked(&self) -> Option<bool> {
        match self {
            Self::Dropdown { options, current, .. } if options.len() == 2 => {
                Some(*current == options[1].0)
            }
            _ => self.switched_on(),
        }
    }

    pub fn current_label(&self) -> Option<&str> {
        match self {
            Self::Dropdown { options, current, .. } | Self::Chips { options, current, .. } => {
                options.iter().find(|(value, _)| value == current).map(|(_, label)| label.as_str())
            }
            _ => None,
        }
    }

    pub fn unlisted(&self) -> bool {
        matches!(self, Self::Dropdown { current, .. } | Self::Chips { current, .. } if !current.is_empty())
            && self.current_label().is_none()
    }

    pub fn token(&self) -> Option<String> {
        match self {
            Self::Number { key, .. }
            | Self::TextField { key, .. }
            | Self::Resolution { key, .. } => Some(key.clone()),
            Self::Dropdown { path, .. }
            | Self::Chips { path, .. }
            | Self::Colour { path, .. }
            | Self::KeyBinding { path, .. } => Some(path.clone()),
            Self::ActionChips { items } => items.last().map(|(_, label)| label.clone()),
            _ => None,
        }
    }

    pub fn switched_on(&self) -> Option<bool> {
        match self {
            Self::Toggle { value, .. } | Self::ToggleAction { value, .. } => Some(*value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub title: String,
    pub desc: String,
    pub control: Control,
    pub look: Look,
}

impl Row {
    pub fn of(title: String, desc: String, control: Control) -> Self {
        Self { title, desc, control, look: Look::default() }
    }

    pub fn hint(&self) -> &str {
        match &self.look.note {
            Note::Desc => &self.desc,
            Note::Own(note) => note,
            Note::Quiet => "",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Note {
    #[default]
    Desc,
    Own(String),
    Quiet,
}

pub const DIMENSIONS: [bool; 2] = [true, false];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Cell {
    #[default]
    Value,
    Joined,
    Buttons(&'static [(&'static str, &'static str)]),
    Field,
    Checks,
    Menu,
    Beside,
    Nudge,
    Corners {
        cut: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct More {
    pub short: Vec<String>,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Look {
    pub cell: Cell,
    pub note: Note,
    pub blank: Option<String>,
    pub heading: Option<String>,
    pub more: Option<More>,
    pub own_title: Option<String>,
}

pub fn masked(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    let shown: String = chars[chars.len().saturating_sub(MASK_SHOWN)..].iter().collect();
    format!("{MASK}{shown}")
}

#[derive(Clone)]
pub struct Card {
    pub key: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
}

impl Card {
    pub fn new(title: crate::i18n::Text, subtitle: &'static str) -> Self {
        Self { key: title.key, title: title.value, subtitle }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionId {
    pub category: &'static str,
    pub key: &'static str,
    pub nth: usize,
}

impl SectionId {
    pub const fn new(category: &'static str, key: &'static str) -> Self {
        Self { category, key, nth: 0 }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Form {
    #[default]
    Plain,
    Duration,
    Link,
    Mention,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot {
    Value { row: usize, blank: Option<String>, form: Form },
    Checks { rows: Vec<usize>, heading: String, blank: String },
}

impl Slot {
    pub fn lead(&self) -> usize {
        match self {
            Self::Value { row, .. } => *row,
            Self::Checks { rows, .. } => rows.first().copied().unwrap_or_default(),
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = usize> + '_ {
        let rows = match self {
            Self::Value { row, .. } => std::slice::from_ref(row),
            Self::Checks { rows, .. } => rows.as_slice(),
        };
        rows.iter().copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parked {
    pub row: usize,
    pub cause: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Say {
    pub text: String,
    pub tokens: Vec<Slot>,
    pub aside: Option<String>,
    pub notes: Vec<String>,
    pub parked: Vec<Parked>,
}

impl Say {
    pub fn rows(&self) -> impl Iterator<Item = usize> + '_ {
        self.tokens.iter().flat_map(Slot::rows)
    }

    pub fn binds(&self, row: usize) -> bool {
        self.rows().any(|at| at == row)
    }

    pub fn parks(&self, row: usize) -> Option<usize> {
        self.parked.iter().find(|parked| parked.row == row).map(|parked| parked.cause)
    }

    pub fn follows(&self, row: usize) -> bool {
        self.tokens.iter().any(|slot| {
            matches!(slot, Slot::Checks { .. })
                && slot.lead() != row
                && slot.rows().any(|at| at == row)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupKind {
    Column,
    Spread,
    Lines,
    Wide,
    Foot,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub heading: Option<String>,
    pub note: Option<String>,
    pub rows: Vec<Row>,
    pub follows_master: bool,
    pub kind: GroupKind,
    pub blank: Option<String>,
    pub echoes: Vec<(usize, Row)>,
}

impl Group {
    pub fn new(kind: GroupKind, heading: Option<String>, rows: Vec<Row>) -> Self {
        Self {
            heading,
            note: None,
            rows,
            follows_master: false,
            kind,
            blank: None,
            echoes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.echoes.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trim {
    None,
    Bottom,
    Rows,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub id: SectionId,
    pub title: String,
    pub desc: String,
    pub say: Say,
    pub master: Option<Row>,
    pub head: Vec<Row>,
    pub groups: Vec<Group>,
    pub foot: Option<String>,
    pub wide: bool,
    pub undivided: bool,
    pub trim: Trim,
    pub min_columns: usize,
    pub opens: Option<&'static str>,
}

impl Section {
    pub fn new(id: SectionId, title: &str, desc: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            desc: desc.to_string(),
            say: Say::default(),
            master: None,
            head: Vec::new(),
            groups: Vec::new(),
            foot: None,
            wide: false,
            undivided: false,
            trim: Trim::None,
            min_columns: 0,
            opens: None,
        }
    }

    pub fn rows(&self) -> impl Iterator<Item = &Row> {
        self.master.iter().chain(&self.head).chain(self.body())
    }

    pub fn header_rows(&self) -> usize {
        usize::from(self.master.is_some()) + self.head.len()
    }

    pub fn master_on(&self) -> Option<bool> {
        self.master.as_ref().and_then(|row| row.control.switched_on())
    }

    pub fn switched_off(&self) -> bool {
        self.master_on() == Some(false)
    }

    pub fn body(&self) -> impl Iterator<Item = &Row> {
        self.groups.iter().flat_map(|group| &group.rows)
    }

    pub fn into_rows(self) -> impl Iterator<Item = Row> {
        self.master
            .into_iter()
            .chain(self.head)
            .chain(self.groups.into_iter().flat_map(|group| group.rows))
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows().nth(index)
    }

    pub fn checks(&self, lead: usize) -> Option<Vec<usize>> {
        let cells: Vec<Cell> = self.rows().skip(lead).map(|row| row.look.cell).collect();
        let members = cells.iter().skip(1).take_while(|cell| **cell == Cell::Joined).count();
        (cells.first() == Some(&Cell::Checks)).then(|| (lead + 1..=lead + members).collect())
    }

    pub fn checked_by(&self, row: usize) -> Option<usize> {
        let cells: Vec<Cell> = self.rows().take(row + 1).map(|row| row.look.cell).collect();
        if cells.get(row) != Some(&Cell::Joined) {
            return None;
        }
        let lead = cells.iter().rposition(|cell| *cell != Cell::Joined)?;
        (cells[lead] == Cell::Checks).then_some(lead)
    }

    pub fn follows(&self, row: usize) -> bool {
        self.say.follows(row) || self.checked_by(row).is_some()
    }

    pub fn has_body(&self) -> bool {
        self.body().next().is_some()
            || self.foot.is_some()
            || self.groups.iter().any(|group| !group.echoes.is_empty())
            || super::page_section(self.id).is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Place {
    pub key: &'static str,
    pub title: &'static str,
    pub note: &'static str,
    pub index_note: &'static str,
    pub strip: bool,
    pub sections: Vec<Section>,
}

pub struct PageSnapshot {
    pub place: Place,
    pub inputs: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagTone {
    Quiet,
    Info,
    Warn,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FolderRow {
    pub index: usize,
    pub key: String,
    pub path: String,
    pub tag: Option<(String, TagTone)>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FolderLedger {
    pub rows: Vec<FolderRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileEntry {
    pub index: Option<usize>,
    pub id: String,
    pub name: String,
    pub active: bool,
}

impl ProfileEntry {
    pub fn builtin(&self) -> bool {
        self.index.is_none()
    }

    pub fn name_key(&self) -> Option<String> {
        use crate::contracts::settings::keys::library;
        self.index.map(|index| format!("{}.{index}.{}", library::PROFILES, library::profile::NAME))
    }
}

#[derive(Debug, Clone)]
pub struct ProfilesPage {
    pub profiles: Vec<ProfileEntry>,
    pub selected: usize,
    pub summary: String,
    pub tree: Vec<TreeNode>,
    pub no_match: bool,
}

impl ProfilesPage {
    pub fn current(&self) -> &ProfileEntry {
        &self.profiles[self.selected.min(self.profiles.len() - 1)]
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub path: String,
    pub label: String,
    pub depth: u8,
    pub state: Option<ProfileState>,
    pub mixed: bool,
    pub hidden_inside: usize,
    pub excluded_inside: usize,
    pub children: bool,
    pub collapsed: bool,
}

impl TreeNode {
    pub fn open_by_default(&self) -> bool {
        self.depth == 0
    }
}

pub fn mark_folds(nodes: &mut [TreeNode], toggled: &HashSet<String>) {
    for node in nodes.iter_mut() {
        node.collapsed = node.children && (node.open_by_default() == toggled.contains(&node.id));
    }
}
