use super::displays;
use crate::app::state::edit_history::{EditHistory, EditSnapshot};
use crate::frontend::arrange::Arrange;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum Closing {
    Saved,
    Cancelled,
}

pub(in crate::app) struct ArrangeSession {
    pub(in crate::app) editor: Arrange,
    pub(in crate::app) output_reviews: std::collections::BTreeMap<String, displays::OutputReview>,
    pub(in crate::app) disconnected_outputs: Vec<String>,
    pub(in crate::app) outputs: displays::OutputRefresh,
    pub(in crate::app) display_copy: Option<displays::DisplayCopy>,
    pub(in crate::app) history: EditHistory,
    pub(in crate::app) conflict: Option<Box<serde_json::Value>>,
    pub(in crate::app) preview_config: Option<serde_json::Value>,
    pub(in crate::app) bar_visible: bool,
    pub(in crate::app) gesture: Option<EditSnapshot>,
    pub(in crate::app) shown: bool,
    pub(in crate::app) closing: Option<Closing>,
}

impl ArrangeSession {
    pub(in crate::app) fn gesturing(&self) -> bool {
        self.gesture.is_some()
    }
}
