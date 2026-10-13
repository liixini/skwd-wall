use std::collections::HashSet;

use super::ActionId;
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
    Number {
        key: String,
        path: String,
        unit: &'static str,
    },
    TextField {
        key: String,
        path: String,
        placeholder: &'static str,
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
    MotionWeights {
        weights: Vec<(String, String, ActionId)>,
    },
    ActionBtn {
        id: ActionId,
        label: String,
    },
    ActionChips {
        items: Vec<(ActionId, String)>,
    },
    Presets {
        mode: String,
        items: Vec<(String, bool)>,
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
            Self::MotionWeights { weights } => {
                weights.first().map(|(_, key, _)| format!("setting-group:{key}"))
            }
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

    pub fn is_compact_field(&self) -> bool {
        self.is_inline_editor() || self.is_group_editor() || self.is_compact_action()
    }

    pub fn is_wide_field(&self) -> bool {
        matches!(
            self,
            Self::StackBar { .. }
                | Self::Segment
                | Self::Code { .. }
                | Self::AppTheme { .. }
                | Self::Details { .. }
        )
    }

    pub fn is_focusable(&self) -> bool {
        !matches!(self, Self::Segment)
    }
}

#[derive(Debug, Clone)]
pub struct Row {
    pub title: String,
    pub desc: String,
    pub control: Control,
}

#[derive(Clone)]
pub struct Card {
    pub title: &'static str,
    pub subtitle: &'static str,
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
    pub kind: crate::contracts::settings::FolderKind,
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
