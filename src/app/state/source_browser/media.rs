use std::{
    collections::{HashSet, VecDeque},
    time::{Duration, Instant},
};

use crate::contracts::browser::{PreviewIdentity, PreviewSize};

pub(crate) struct PreviewDecode {
    pub(crate) key: PreviewIdentity,
    pub(crate) generation: u64,
    pub(crate) path: String,
    pub(crate) size: PreviewSize,
}

#[derive(Default)]
pub(crate) struct BrowserMediaState {
    pub(crate) key: Option<PreviewIdentity>,
    pub(crate) generation: u64,
    pub(crate) thumbnail_generation: u64,
    pub(crate) since: Option<Instant>,
    pub(crate) requested: bool,
    pub(crate) refresh: bool,
    pub(crate) decode_busy: bool,
    pub(crate) decode_pending: Option<PreviewDecode>,
    pub(crate) thumbnails: VecDeque<PreviewDecode>,
    pub(crate) thumbnail_queued: HashSet<PreviewIdentity>,
    pub(crate) thumbnail_done: HashSet<PreviewIdentity>,
    pub(crate) thumbnail_order: VecDeque<PreviewIdentity>,
    pub(crate) requests: HashSet<PreviewIdentity>,
    pub(crate) failed: HashSet<PreviewIdentity>,
    pub(crate) decode_path: Option<String>,
    pub(crate) handles: Vec<iced::widget::image::Handle>,
    pub(crate) delays: Vec<Duration>,
    pub(crate) duration: Duration,
    pub(crate) started: Option<Instant>,
    pub(crate) cards: CardAnimations,
}

#[derive(Default)]
pub(crate) struct CardAnimations {
    pub(crate) queue: VecDeque<PreviewDecode>,
    pub(crate) requests: std::collections::HashMap<String, Instant>,
    pub(crate) seen: HashSet<String>,
    pub(crate) order: VecDeque<String>,
    pub(crate) epoch: Option<Instant>,
    pub(crate) generation: u64,
    pub(crate) search: u64,
    pub(crate) visible: bool,
}

impl CardAnimations {
    pub(crate) fn reset(&mut self) {
        *self = Self { generation: self.generation.wrapping_add(1), ..Self::default() };
    }

    pub(crate) fn idle(&self) -> bool {
        self.queue.is_empty() && self.requests.is_empty() && self.seen.is_empty()
    }
}

impl BrowserMediaState {
    pub(crate) fn frame_index(&self, now: Instant) -> usize {
        let Some(started) = self.started else { return 0 };
        crate::contracts::browser::preview_frame_index(
            &self.delays,
            self.duration,
            now.saturating_duration_since(started),
        )
    }
}
