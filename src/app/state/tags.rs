use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::domain::library::search::TagEntry;
use crate::frontend::animation::{MotionProfile, MotionTier, Spring, Tween};
pub(crate) struct TagState {
    pub(crate) editing: bool,
    pub(crate) focus_pending: bool,
    pub(crate) input: String,
    pub(crate) card_locked: Vec<String>,
    pub(crate) card_key: Option<String>,
    pub(crate) card_drawer_open: bool,
    pub(crate) mode: bool,
    pub(crate) select: HashSet<u32>,
    pub(crate) mass_input: String,
    pub(crate) mass_tags: Vec<String>,
    pub(crate) cloud_open: bool,
    pub(crate) matching_tags_open: bool,
    pub(crate) sort_az: bool,
    pub(crate) search_mode: SearchMode,
    pub(crate) tag_search: String,
    pub(crate) cloud_entrance: Tween,
    pub(crate) cloud_scroll: Spring,
    pub(crate) semantic: SemanticSearchState,
    pub(crate) cloud_cache: RefCell<Option<(u64, Rc<Vec<TagEntry>>)>>,
}

#[derive(Default)]
pub(crate) struct SemanticSearchState {
    pub(crate) search: String,
    pub(crate) generation: u64,
    pub(crate) pending: bool,
    pub(crate) resolved: bool,
    pub(crate) error: Option<String>,
    pub(crate) ranked: Vec<String>,
    pub(crate) exclusions: Vec<String>,
    pub(crate) sort_override: bool,
    pub(crate) query_ms: f64,
    pub(crate) search_ms: f64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    #[default]
    Tags,
    Describe,
}

impl SearchMode {
    pub(crate) fn from_config(value: &str) -> Self {
        if value == "describe" { Self::Describe } else { Self::Tags }
    }
}

impl Default for TagState {
    fn default() -> Self {
        let motion = MotionProfile::default();
        Self {
            editing: false,
            focus_pending: false,
            input: String::new(),
            card_locked: Vec::new(),
            card_key: None,
            card_drawer_open: false,
            mode: false,
            select: HashSet::new(),
            mass_input: String::new(),
            mass_tags: Vec::new(),
            cloud_open: false,
            matching_tags_open: false,
            sort_az: false,
            search_mode: SearchMode::Tags,
            tag_search: String::new(),
            cloud_entrance: motion.tween(0.0, MotionTier::Standard),
            cloud_scroll: motion.spring(0.0, MotionTier::Standard),
            semantic: SemanticSearchState::default(),
            cloud_cache: RefCell::new(None),
        }
    }
}

impl TagState {
    pub(crate) fn with_search_mode(search_mode: SearchMode) -> Self {
        Self { search_mode, ..Self::default() }
    }

    pub(crate) fn set_motion_profile(&mut self, motion: MotionProfile) {
        motion.retime_tween(&mut self.cloud_entrance, MotionTier::Standard);
        motion.retime_spring(&mut self.cloud_scroll, MotionTier::Standard);
    }
}
