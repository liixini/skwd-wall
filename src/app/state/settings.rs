#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use crate::domain::input::Trigger;
use crate::frontend::animation::{MotionProfile, MotionTier, Tween};
use crate::frontend::settings::{ActionId, SettingsFocus, SettingsSearchResult, WeatherReviewTab};

pub(crate) struct KeybindCapture {
    pub(crate) path: String,
    pub(crate) action: crate::domain::input::InputAction,
    pub(crate) title_key: &'static str,
    pub(crate) slot: usize,
    pub(crate) triggers: Vec<Trigger>,
    pub(crate) edited: bool,
}

pub(crate) struct SettingsInputEdit {
    pub(crate) key: String,
    pub(crate) original: String,
    pub(crate) we_dirty: bool,
    pub(crate) playlist_dirty: bool,
    pub(crate) schedule_dirty: bool,
    pub(crate) weather_refresh: WeatherRefresh,
}

#[derive(Clone, Copy, Default)]
pub(crate) enum WeatherRefresh {
    #[default]
    Clean,
    Pending,
}

pub(crate) struct FolderUndo {
    pub(crate) kind: crate::contracts::settings::FolderKind,
    pub(crate) index: usize,
    pub(crate) path: String,
}

pub(crate) struct SddmRollback {
    pub(crate) enabled: bool,
    pub(crate) theme: Option<String>,
    pub(crate) error: String,
}

pub(crate) struct SettingsState {
    pub(crate) open: bool,
    pub(crate) entrance: Tween,
    pub(crate) tab_anim: Tween,
    pub(crate) inputs: HashMap<String, String>,
    pub(crate) tab: String,
    pub(crate) section: usize,
    pub(crate) control_page: usize,
    pub(crate) focus: SettingsFocus,
    pub(crate) focused_control: usize,
    pub(crate) focused_choice: Option<usize>,
    pub(crate) expanded_details: HashSet<String>,
    pub(crate) bar_reveals: HashMap<String, Tween>,
    pub(crate) profile: usize,
    pub(crate) profile_filter: String,
    pub(crate) profile_enter: Tween,
    pub(crate) profile_marker: Tween,
    pub(crate) folder_drafts: HashMap<&'static str, String>,
    pub(crate) folder_undo: Option<FolderUndo>,
    pub(crate) search_open: bool,
    pub(crate) search_query: String,
    pub(crate) search_results: Vec<SettingsSearchResult>,
    pub(crate) input_edit: Option<SettingsInputEdit>,
    pub(crate) resolution_height: bool,
    pub(crate) keybind_capture: Option<KeybindCapture>,
    pub(crate) process_picker_open: bool,
    pub(crate) process_picker_query: String,
    pub(crate) process_picker_manual: String,
    pub(crate) process_picker_scroll: f32,
    pub(crate) section_anim: Tween,
    pub(crate) control_anim: Tween,
    pub(crate) inset_target: f32,
    pub(crate) armed: Option<ActionId>,
    pub(crate) we_dirty: bool,
    pub(crate) playlist_dirty: bool,
    pub(crate) schedule_dirty: bool,
    pub(crate) weather_refresh: WeatherRefresh,
    pub(crate) weather_review_category: Option<&'static str>,
    pub(crate) weather_review_tab: WeatherReviewTab,
    pub(crate) weather_review_query: String,
    pub(crate) weather_review_candidates: Vec<String>,
    pub(crate) weather_review_error: Option<String>,
    pub(crate) weather_review_loading: bool,
    pub(crate) weather_review_generation: u64,
    pub(crate) weather_review_focus: usize,
    pub(crate) preview_desired_path: String,
    pub(crate) preview_in_flight_path: String,
    pub(crate) preview_allocated_path: String,
    pub(crate) preview_allocation: Option<iced_runtime::image::Allocation>,
    pub(crate) semantic_importing: bool,
    pub(crate) folder_picking: bool,
    pub(crate) semantic_import_status: String,
    pub(crate) sddm_themes: Vec<String>,
    pub(crate) sddm_checking: bool,
    pub(crate) sddm_rollback: Option<SddmRollback>,
}

impl Default for SettingsState {
    fn default() -> Self {
        let motion = MotionProfile::default();
        Self {
            open: false,
            entrance: motion.tween(0.0, MotionTier::Standard),
            tab_anim: motion.tween(1.0, MotionTier::Fast),
            inputs: HashMap::new(),
            tab: String::from("picker"),
            section: 0,
            control_page: 0,
            focus: SettingsFocus::Sections,
            focused_control: 0,
            focused_choice: None,
            expanded_details: HashSet::new(),
            bar_reveals: HashMap::new(),
            profile: 0,
            profile_filter: String::new(),
            profile_enter: motion.tween(1.0, MotionTier::Fast),
            profile_marker: motion.tween(0.0, MotionTier::Fast),
            folder_drafts: HashMap::new(),
            folder_undo: None,
            search_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
            input_edit: None,
            resolution_height: false,
            keybind_capture: None,
            process_picker_open: false,
            process_picker_query: String::new(),
            process_picker_manual: String::new(),
            process_picker_scroll: 0.0,
            section_anim: motion.tween(1.0, MotionTier::Standard),
            control_anim: motion.tween(1.0, MotionTier::Fast),
            inset_target: 0.0,
            armed: None,
            we_dirty: false,
            playlist_dirty: false,
            schedule_dirty: false,
            weather_refresh: WeatherRefresh::Clean,
            weather_review_category: None,
            weather_review_tab: WeatherReviewTab::Suggested,
            weather_review_query: String::new(),
            weather_review_candidates: Vec::new(),
            weather_review_error: None,
            weather_review_loading: false,
            weather_review_generation: 0,
            weather_review_focus: 0,
            preview_desired_path: String::new(),
            preview_in_flight_path: String::new(),
            preview_allocated_path: String::new(),
            preview_allocation: None,
            semantic_importing: false,
            folder_picking: false,
            semantic_import_status: String::new(),
            sddm_themes: Vec::new(),
            sddm_checking: false,
            sddm_rollback: None,
        }
    }
}

impl SettingsState {
    pub(crate) fn set_motion_profile(&mut self, motion: MotionProfile) {
        motion.retime_tween(&mut self.entrance, MotionTier::Standard);
        motion.retime_tween(&mut self.tab_anim, MotionTier::Fast);
        motion.retime_tween(&mut self.section_anim, MotionTier::Standard);
        motion.retime_tween(&mut self.control_anim, MotionTier::Fast);
        motion.retime_tween(&mut self.profile_enter, MotionTier::Fast);
        motion.retime_tween(&mut self.profile_marker, MotionTier::Fast);
        for reveal in self.bar_reveals.values_mut() {
            motion.retime_tween(reveal, MotionTier::Fast);
        }
    }

    pub(crate) fn open_bar(&mut self, id: String, motion: MotionProfile) {
        self.bar_reveals
            .entry(id)
            .or_insert_with(|| motion.tween(0.0, MotionTier::Fast))
            .retarget(1.0);
    }

    pub(crate) fn toggle_fold(&mut self, id: String, open_by_default: bool, motion: MotionProfile) {
        let open = open_by_default != self.expanded_details.contains(&id);
        let reveal = self
            .bar_reveals
            .entry(id.clone())
            .or_insert_with(|| motion.tween(if open { 1.0 } else { 0.0 }, MotionTier::Fast));
        reveal.retarget(if open { 0.0 } else { 1.0 });
        if !self.expanded_details.remove(&id) {
            self.expanded_details.insert(id);
        }
    }

    pub(crate) fn select_profile(&mut self, position: usize) {
        if self.profile != position {
            self.profile_enter.run(0.0, 1.0);
        }
        self.profile = position;
        self.profile_marker.retarget(position as f32);
        self.armed = None;
    }

    pub(crate) fn slide_segment(&mut self, key: String, from: f32, to: f32, motion: MotionProfile) {
        self.bar_reveals
            .entry(key)
            .or_insert_with(|| motion.tween(from, MotionTier::Fast))
            .retarget(to);
    }

    pub(crate) fn flash(&mut self, key: String, motion: MotionProfile) {
        self.bar_reveals
            .entry(key)
            .or_insert_with(|| motion.tween(0.0, MotionTier::Slow))
            .run(1.0, 0.0);
    }

    pub(crate) fn profile_animating(&self) -> bool {
        !self.profile_enter.settled() || !self.profile_marker.settled()
    }

    pub(crate) fn tick_profile(&mut self, dt: f32) {
        self.profile_enter.tick(dt);
        self.profile_marker.tick(dt);
    }

    pub(crate) fn toggle_bar(&mut self, id: String, motion: MotionProfile) {
        let reveal =
            self.bar_reveals.entry(id).or_insert_with(|| motion.tween(0.0, MotionTier::Fast));
        reveal.retarget(if reveal.target > 0.5 { 0.0 } else { 1.0 });
    }

    pub(crate) fn bars_animating(&self) -> bool {
        self.bar_reveals.values().any(|reveal| !reveal.settled())
    }

    pub(crate) fn tick_bars(&mut self, dt: f32) {
        for reveal in self.bar_reveals.values_mut() {
            reveal.tick(dt);
        }
        self.bar_reveals.retain(|_, reveal| reveal.target > 0.0 || reveal.x > 0.0);
    }
}
