use super::ActionId;

#[derive(Debug, Clone)]
pub enum Control {
    Toggle {
        path: String,
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
    },
    MotionWeights {
        weights: Vec<(String, String, ActionId)>,
    },
    ActionBtn {
        id: ActionId,
        label: String,
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
        matches!(self, Self::Toggle { .. })
    }

    pub fn is_compact_field(&self) -> bool {
        self.is_inline_editor() || self.is_group_editor() || self.is_compact_action()
    }

    pub fn is_wide_field(&self) -> bool {
        matches!(self, Self::StackBar { .. } | Self::Code { .. })
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
