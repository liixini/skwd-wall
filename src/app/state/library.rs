use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::backend::picker::library::LibraryState;
use crate::domain::library::filter::Filters;

pub(crate) type PlaylistFilter = (i64, String, HashSet<String>);

#[derive(Default)]
pub(crate) struct PreheatState {
    sent: Option<String>,
    pending: Option<(String, Instant)>,
}

impl PreheatState {
    pub(crate) fn poll(
        &mut self,
        target: Option<&str>,
        now: Instant,
        delay: Duration,
    ) -> Option<String> {
        let Some(target) = target else {
            self.pending = None;
            return None;
        };
        if self.sent.as_deref() == Some(target) {
            self.pending = None;
            return None;
        }
        if let Some((pending, due)) = self.pending.as_ref()
            && pending == target
        {
            if now < *due {
                return None;
            }
            let path = pending.clone();
            self.pending = None;
            self.sent = Some(path.clone());
            return Some(path);
        }
        self.pending = Some((target.to_string(), now + delay));
        None
    }
}

pub(crate) struct LibrarySession {
    pub(crate) library: LibraryState,
    pub(crate) filters: Filters,
    pub(crate) filtered: Vec<u32>,
    pub(crate) preheat: PreheatState,
    pub(crate) wallpaper_dir: String,
    pub(crate) video_dir: String,
    pub(crate) folder_options: Vec<String>,
    pub(crate) folder_hidden: Vec<bool>,
    pub(crate) all_folders: Vec<String>,
    pub(crate) folder_roots: Vec<String>,
    pub(crate) profile_override: Option<String>,
    pub(crate) playlist_filter: Option<PlaylistFilter>,
    pub(crate) filter_transition_pending: bool,
    pub(crate) visible_count: usize,
    pub(crate) list_dirty: bool,
    pub(crate) selection_restored: bool,
}

impl LibrarySession {
    pub(crate) fn new(filters: Filters, wallpaper_dir: String, video_dir: String) -> Self {
        Self {
            library: LibraryState::default(),
            filters,
            filtered: Vec::new(),
            preheat: PreheatState::default(),
            wallpaper_dir,
            video_dir,
            folder_options: Vec::new(),
            folder_hidden: Vec::new(),
            all_folders: Vec::new(),
            folder_roots: Vec::new(),
            profile_override: crate::infrastructure::runtime::startup_profile(),
            playlist_filter: None,
            filter_transition_pending: false,
            visible_count: 0,
            list_dirty: false,
            selection_restored: false,
        }
    }
}

#[cfg(test)]
mod tests;
