use std::collections::BTreeMap;

use skwd_config::picker_layout::{LayoutProfile, Size};

use crate::frontend::arrange::{DisplayRecipient, DisplayReview};

pub(in crate::app) struct DisplayCopy {
    pub source: String,
    pub profile: LayoutProfile,
    pub recipients: Vec<DisplayRecipient>,
    pub sizes: BTreeMap<String, Size>,
    pub review: Option<DisplayDraft>,
}

pub(in crate::app) struct DisplayDraft {
    pub profile: LayoutProfile,
    pub view: DisplayReview,
}

pub(in crate::app) struct OutputReview {
    pub viewport: Size,
    pub save_disconnected: bool,
}

#[derive(Default)]
pub(in crate::app) struct OutputRefresh {
    pub request: u64,
    pub action: Option<crate::frontend::arrange::ArrangeMsg>,
    pub document: Option<serde_json::Value>,
    pub ready: bool,
}
