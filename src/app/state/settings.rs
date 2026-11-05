#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use crate::frontend::animation::{MotionProfile, MotionTier, Tween};
use crate::frontend::settings::{
    ActionId, SectionId, SettingsFocus, SettingsSearchResult, WeatherReviewTab,
};
use crate::frontend::ui::calm::GROW;

pub(crate) struct KeybindCapture {
    pub(crate) path: String,
    pub(crate) action: crate::domain::input::InputAction,
    pub(crate) slot: usize,
}

pub(crate) struct KeyMove {
    pub(crate) key: String,
    pub(crate) from: Vec<&'static str>,
    pub(crate) before: Vec<(&'static str, String)>,
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

pub(crate) struct SideWrite {
    pub(crate) cause: &'static str,
    pub(crate) key: &'static str,
    pub(crate) prior: Option<serde_json::Value>,
    pub(crate) wrote: Option<serde_json::Value>,
}

pub(crate) struct SddmRollback {
    pub(crate) enabled: bool,
    pub(crate) theme: Option<String>,
    pub(crate) error: String,
}

pub(crate) enum SettingsNavigation {
    Page { tab: String },
    Search { tab: String, section: usize, row: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeatherQueue {
    pub(crate) mode: String,
    pub(crate) stopped: bool,
}

pub(crate) struct SettingsState {
    pub(crate) open: bool,
    pub(crate) layout_studio: bool,
    pub(crate) layouts: crate::frontend::settings::LayoutsUi,
    pub(crate) entrance: Tween,
    pub(crate) tab_anim: Tween,
    pub(crate) emblem_anim: Tween,
    pub(crate) inputs: HashMap<String, String>,
    pub(crate) tab: String,
    pub(crate) section: usize,
    pub(crate) open_section: Option<SectionId>,
    pub(crate) queued_section: Option<SectionId>,
    pub(crate) closing_section: Option<SectionId>,
    pub(crate) section_fold: Tween,
    pub(crate) busy: Tween,
    pub(crate) pop: Option<u64>,
    pub(crate) edit: Option<u64>,
    pub(crate) previous: Option<crate::frontend::settings::PageSnapshot>,
    pub(crate) pending: Option<SettingsNavigation>,
    pub(crate) control_page: usize,
    pub(crate) focus: SettingsFocus,
    pub(crate) focused_control: usize,
    pub(crate) focused_choice: Option<usize>,
    pub(crate) expanded_details: HashSet<String>,
    pub(crate) bar_reveals: HashMap<String, Tween>,
    pub(crate) profile: usize,
    pub(crate) profile_filter: String,
    pub(crate) folder_drafts: HashMap<&'static str, String>,
    pub(crate) folder_undo: Option<FolderUndo>,
    pub(crate) side_writes: Vec<SideWrite>,
    pub(crate) search_open: bool,
    pub(crate) search_query: String,
    pub(crate) search_results: Vec<SettingsSearchResult>,
    pub(crate) search_total: usize,
    pub(crate) search_at: Option<usize>,
    pub(crate) input_edit: Option<SettingsInputEdit>,
    pub(crate) resolution_height: bool,
    pub(crate) keybind_capture: Option<KeybindCapture>,
    pub(crate) keys_query: String,
    pub(crate) keys_moved: Option<KeyMove>,
    pub(crate) keys_same: Option<String>,
    pub(crate) keys_lit: Option<&'static str>,
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
    pub(crate) weather_after_follow: Option<WeatherQueue>,
    pub(crate) weather_review_generation: u64,
    pub(crate) weather_review_focus: usize,
    pub(crate) preview_desired_path: String,
    pub(crate) preview_in_flight_path: String,
    pub(crate) preview_allocated_path: String,
    pub(crate) preview_allocation: Option<iced_runtime::image::Allocation>,
    pub(crate) semantic_importing: bool,
    pub(crate) folder_picking: bool,
    pub(crate) semantic_import_status: String,
    pub(crate) drawing_library: bool,
    pub(crate) library_job: bool,
    pub(crate) sddm_themes: Vec<String>,
    pub(crate) sddm_checking: bool,
    pub(crate) sddm_rollback: Option<SddmRollback>,
}

impl Default for SettingsState {
    fn default() -> Self {
        let motion = MotionProfile::default();
        Self {
            open: false,
            layout_studio: false,
            layouts: crate::frontend::settings::LayoutsUi::default(),
            entrance: motion.tween(0.0, MotionTier::Standard),
            tab_anim: motion.tween(1.0, MotionTier::Fast),
            emblem_anim: motion.tween(1.0, MotionTier::Fast),
            inputs: HashMap::new(),
            tab: String::from(crate::frontend::settings::place::LAYOUT),
            section: 0,
            open_section: None,
            queued_section: None,
            closing_section: None,
            section_fold: motion.override_tween(1.0, fold_ms()),
            busy: motion.override_tween(0.0, fold_ms()),
            pop: None,
            edit: None,
            previous: None,
            pending: None,
            control_page: 0,
            focus: SettingsFocus::Sections,
            focused_control: 0,
            focused_choice: None,
            expanded_details: HashSet::new(),
            bar_reveals: HashMap::new(),
            profile: 0,
            profile_filter: String::new(),
            folder_drafts: HashMap::new(),
            folder_undo: None,
            side_writes: Vec::new(),
            search_open: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_total: 0,
            search_at: None,
            input_edit: None,
            resolution_height: false,
            keybind_capture: None,
            keys_query: String::new(),
            keys_moved: None,
            keys_same: None,
            keys_lit: None,
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
            weather_after_follow: None,
            weather_review_generation: 0,
            weather_review_focus: 0,
            preview_desired_path: String::new(),
            preview_in_flight_path: String::new(),
            preview_allocated_path: String::new(),
            preview_allocation: None,
            semantic_importing: false,
            folder_picking: false,
            semantic_import_status: String::new(),
            drawing_library: false,
            library_job: false,
            sddm_themes: Vec::new(),
            sddm_checking: false,
            sddm_rollback: None,
        }
    }
}

const MS_PER_SECOND: f32 = 1_000.0;

fn fold_ms() -> f32 {
    GROW.duration.as_secs_f32() * MS_PER_SECOND
}

impl SettingsState {
    pub(crate) fn toggle_section(&mut self, id: SectionId, instant: bool) {
        self.pop = None;
        self.edit = None;
        let folding = self.closing_section.filter(|_| !instant && !self.section_fold.settled());
        let (open, queued) = match (self.open_section, folding) {
            (Some(open), _) if open == id => (None, None),
            (Some(_), _) if !instant => (None, Some(id)),
            (None, Some(_)) if self.queued_section == Some(id) => (None, None),
            (None, Some(_)) => (None, Some(id)),
            _ => (Some(id), None),
        };
        self.closing_section = self.open_section.filter(|_| !instant && open.is_none()).or(folding);
        let restart = folding.is_none() || self.open_section.is_some();
        self.open_section = open;
        self.queued_section = queued;
        if restart {
            self.fold(instant);
        }
    }

    pub(crate) fn close_sections(&mut self) {
        self.open_section = None;
        self.queued_section = None;
        self.closing_section = None;
        self.pop = None;
        self.edit = None;
        self.section_fold.snap(1.0);
        self.busy.snap(0.0);
    }

    fn fold(&mut self, instant: bool) {
        let busy = if self.open_section.is_some() { 1.0 } else { 0.0 };
        if instant {
            self.section_fold.snap(1.0);
            self.busy.snap(busy);
        } else {
            self.section_fold.run(0.0, 1.0);
            self.busy.retarget(busy);
        }
    }

    pub(crate) fn sections_animating(&self) -> bool {
        !self.section_fold.settled() || !self.busy.settled() || self.queued_section.is_some()
    }

    pub(crate) fn tick_sections(&mut self, dt: f32) {
        self.section_fold.tick(dt);
        self.busy.tick(dt);
        if !self.section_fold.settled() {
            return;
        }
        self.closing_section = None;
        if let Some(next) = self.queued_section.take() {
            self.open_section = Some(next);
            self.fold(false);
        }
    }

    pub(crate) fn navigation_tab(&self) -> &str {
        match self.pending.as_ref() {
            Some(SettingsNavigation::Page { tab, .. } | SettingsNavigation::Search { tab, .. }) => {
                crate::frontend::settings::canonical_place(tab).unwrap_or(&self.tab)
            }
            None => &self.tab,
        }
    }

    pub(crate) fn set_motion_profile(&mut self, motion: MotionProfile) {
        let choreography = &crate::frontend::settings::SETTINGS_MOTION;
        motion.retime_tween_override(&mut self.entrance, choreography.open_ms);
        motion.retime_tween_override(&mut self.tab_anim, choreography.content_ms);
        motion.retime_tween_override(&mut self.emblem_anim, choreography.emblem_ms);
        motion.retime_tween_override(&mut self.section_anim, choreography.content_ms);
        motion.retime_tween_override(&mut self.section_fold, fold_ms());
        motion.retime_tween_override(&mut self.busy, fold_ms());
        if choreography.reduced(motion) {
            self.entrance.snap(1.0);
            self.tab_anim.snap(1.0);
            self.emblem_anim.snap(1.0);
            self.section_anim.snap(1.0);
            self.previous = None;
            if let Some(next) = self.queued_section.take() {
                self.open_section = Some(next);
            }
            self.fold(true);
        }
        motion.retime_tween(&mut self.control_anim, MotionTier::Fast);
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
        self.profile = position;
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
