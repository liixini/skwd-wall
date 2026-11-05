use std::collections::HashMap;
use std::time::Instant;

use crate::frontend::animation::{MotionProfile, MotionTier, Tween};
use crate::frontend::theme::Palette;

#[derive(Debug, Clone)]
pub(crate) struct ThemeAuditionPreview {
    pub(crate) backend: String,
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) label: String,
    pub(crate) palette: Palette,
}

pub(crate) struct ThemeState {
    pub(crate) palette: Palette,
    pub(crate) base_palette: Palette,
    pub(crate) preview_target: Option<usize>,
    pub(crate) hover_since: Option<(usize, Instant)>,
    pub(crate) cache: HashMap<usize, Palette>,
    pub(crate) job_pending: Option<usize>,
    pub(crate) suspended: bool,
    pub(crate) shell_preview_sent: Option<usize>,
    pub(crate) swatch: Vec<iced::Color>,
    pub(crate) swatch_target: Option<usize>,
    pub(crate) swatch_cache: HashMap<usize, Vec<iced::Color>>,
    pub(crate) fade_from: Palette,
    pub(crate) fade_to: Palette,
    pub(crate) fade_t: Tween,
    pub(crate) bar_open: bool,
    pub(crate) backends: Option<Vec<String>>,
    pub(crate) audition_open: bool,
    pub(crate) audition_focused: bool,
    pub(crate) audition_loading: bool,
    pub(crate) audition_backend: String,
    pub(crate) audition_backends: Vec<String>,
    pub(crate) audition_pending_backend: Option<String>,
    pub(crate) audition_previews: Vec<ThemeAuditionPreview>,
    pub(crate) audition_error: Option<String>,
}

impl ThemeState {
    pub(crate) fn new(palette: &Palette, motion: MotionProfile) -> Self {
        Self {
            palette: *palette,
            base_palette: *palette,
            preview_target: None,
            hover_since: None,
            cache: HashMap::new(),
            job_pending: None,
            suspended: false,
            shell_preview_sent: None,
            swatch: Vec::new(),
            swatch_target: None,
            swatch_cache: HashMap::new(),
            fade_from: *palette,
            fade_to: *palette,
            fade_t: motion.tween(1.0, MotionTier::Standard),
            bar_open: false,
            backends: None,
            audition_open: false,
            audition_focused: false,
            audition_loading: false,
            audition_backend: String::new(),
            audition_backends: Vec::new(),
            audition_pending_backend: None,
            audition_previews: Vec::new(),
            audition_error: None,
        }
    }

    pub(crate) fn set_motion_profile(&mut self, motion: MotionProfile) {
        motion.retime_tween(&mut self.fade_t, MotionTier::Standard);
    }
}
