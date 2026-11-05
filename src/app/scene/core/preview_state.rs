use std::sync::Arc;
use std::time::{Duration, Instant};

use futures_channel::mpsc::UnboundedSender;

use crate::contracts::preview::{BufPool, PREVIEW_HEIGHT, PREVIEW_WIDTH, Upload, UploadQueue};
use crate::domain::library::catalog::WallpaperKind;
use crate::frontend::scene::layout::Mode;
use crate::infrastructure::preview::LiveStreamer;
use crate::infrastructure::runtime::Wake;

use super::model::{RebuildCtx, SceneCore};

const LIVE_IDLE_TIMEOUT: Duration = Duration::from_secs(2);

pub(super) struct PreviewState {
    pub(super) pending_preview: Option<(usize, Instant)>,
    pub(super) preview_enabled: bool,
    pub(super) preview_delay_ms: u64,
    pub(super) live_fps: u32,
    pub(super) wake: Option<UnboundedSender<Wake>>,
    pub(super) live: Option<LiveStreamer>,
    pub(super) live_target: Option<usize>,
    pub(super) live_injected: Option<usize>,
    pub(super) live_out: Option<LiveStreamer>,
    pub(super) out_store: Option<usize>,
    pub(super) sandy_out_pending: bool,
    pub(super) frame_pool: Arc<BufPool>,
}

impl PreviewState {
    pub(super) fn new() -> Self {
        Self {
            pending_preview: None,
            preview_enabled: true,
            preview_delay_ms: 250,
            live_fps: 30,
            wake: None,
            live: None,
            live_target: None,
            live_injected: None,
            live_out: None,
            out_store: None,
            sandy_out_pending: false,
            frame_pool: Arc::new(BufPool::new((PREVIEW_WIDTH * PREVIEW_HEIGHT * 4) as usize, 6)),
        }
    }
}

impl SceneCore {
    pub fn preview_config(&self) -> (bool, u64, u32) {
        (
            self.preview_state.preview_enabled,
            self.preview_state.preview_delay_ms,
            self.preview_state.live_fps,
        )
    }

    pub fn preview_deadline(&self) -> Option<Instant> {
        self.preview_state.pending_preview.map(|(_, at)| at)
    }

    pub fn preview_deadline_due(&self, now: Instant) -> bool {
        self.preview_deadline().is_some_and(|at| now >= at)
    }

    pub fn set_wake(&mut self, wake: UnboundedSender<Wake>) {
        self.preview_state.wake = Some(wake);
    }

    pub fn set_preview_config(&mut self, enabled: bool, delay_ms: u64, live_fps: u32) {
        log::info!("preview config: enabled={enabled} delay={delay_ms}ms fps={live_fps}");
        let disable = self.preview_state.preview_enabled && !enabled;
        self.preview_state.preview_enabled = enabled;
        self.preview_state.preview_delay_ms = delay_ms;
        self.set_live_preview_fps(live_fps);
        if disable {
            self.teardown_previews();
        }
    }

    pub fn set_live_preview_fps(&mut self, live_fps: u32) {
        if self.preview_state.live_fps == live_fps {
            return;
        }
        self.preview_state.live_fps = live_fps;
        self.preview_state.live = None;
        self.preview_state.live_out = None;
        self.preview_state.live_injected = None;
    }

    pub fn frame_pool(&self) -> &Arc<BufPool> {
        &self.preview_state.frame_pool
    }

    pub(super) fn teardown_previews(&mut self) {
        self.preview_state.pending_preview = None;
        self.preview_state.live = None;
        self.preview_state.live_out = None;
        self.preview_state.live_target = None;
        self.preview_state.live_injected = None;
        self.preview_state.out_store = None;
        self.preview_state.sandy_out_pending = false;
    }

    pub(crate) fn release_while_hidden(&mut self) {
        self.teardown_previews();
    }

    pub(super) fn preview_hover(&self) -> Option<usize> {
        match self.mode {
            Mode::Sandy | Mode::Collection => None,
            _ => self.hover,
        }
    }

    fn focused_video(&self, ctx: &RebuildCtx<'_>) -> Option<usize> {
        preview_candidates(self.preview_hover(), self.current, self.user_engaged).find_map(
            |filtered_index| {
                let store_index = *ctx.filtered.get(filtered_index)? as usize;
                (ctx.catalog.items.get(store_index)?.kind == WallpaperKind::Video)
                    .then_some(store_index)
            },
        )
    }

    pub(super) fn preview_delay(&self) -> u64 {
        if self.mode == Mode::Sandy { 0 } else { self.preview_state.preview_delay_ms }
    }

    pub(super) fn manage_preview(&mut self, ctx: &mut RebuildCtx<'_>) -> bool {
        let before = self.preview_slot_idx();
        let now = self.motion.last_tick.unwrap_or_else(Instant::now);
        let target = if self.preview_state.preview_enabled
            && !self.hidden()
            && !self.input_idle
            && !self.hand_tall_in_use()
        {
            self.focused_video(ctx)
        } else {
            None
        };
        self.manage_out(ctx, now);
        self.manage_live(ctx, now, target);
        self.preview_slot_idx() != before
    }

    fn manage_out(&mut self, ctx: &mut RebuildCtx<'_>, now: Instant) {
        self.out_capture(ctx);
        let flying = self.mode == Mode::Sandy
            && !self.sandy.prog.settled()
            && self.sandy.swirl.target < 0.5
            && self.sandy.swirl.x < 0.001;
        if flying {
            self.out_frames(ctx, now);
        } else {
            live_idle(&mut self.preview_state.live_out, now);
        }
    }

    fn out_capture(&mut self, ctx: &mut RebuildCtx<'_>) {
        if !std::mem::take(&mut self.preview_state.sandy_out_pending) {
            return;
        }
        let from_store =
            self.sandy.from.and_then(|index| ctx.filtered.get(index).map(|&store| store as usize));
        if let Some(store_index) =
            from_store.filter(|&store_index| self.preview_slot_idx() == Some(store_index))
        {
            push_out_freeze(ctx.uploads);
            self.preview_state.out_store = Some(store_index);
            if self.xp.sandy.video_out_live {
                self.out_adopt();
            }
        } else {
            self.preview_state.out_store = None;
        }
    }

    fn out_adopt(&mut self) {
        let idle = self.preview_state.live_out.take();
        self.preview_state.live_out = self.preview_state.live.take();
        self.preview_state.live = idle;
        self.preview_state.live_target = None;
        self.preview_state.live_injected = None;
    }

    fn out_frames(&mut self, ctx: &mut RebuildCtx<'_>, now: Instant) {
        if let Some(stream) = self.preview_state.live_out.as_mut() {
            stream.last_active = now;
            if let Some(frame) = stream.latest_frame() {
                push_out_frame(ctx.uploads, frame);
            }
        }
    }

    fn manage_live(&mut self, ctx: &mut RebuildCtx<'_>, now: Instant, target: Option<usize>) {
        self.live_arm(now, target);
        let Some(index) = self.preview_state.live_target else {
            live_idle(&mut self.preview_state.live, now);
            return;
        };
        let Some(path) = ctx
            .catalog
            .items
            .get(index)
            .filter(|item| item.kind == WallpaperKind::Video && !item.path.is_empty())
            .map(|item| item.path.clone())
        else {
            self.restore_live();
            return;
        };
        if self.preview_state.live.is_none()
            && let Some(wake) = self.preview_state.wake.clone()
        {
            self.preview_state.live = LiveStreamer::start(
                wake,
                now,
                self.preview_state.frame_pool.clone(),
                self.preview_state.live_fps,
            );
        }
        let Some(streamer) = self.preview_state.live.as_mut() else {
            return;
        };
        if !streamer.set_path(&path) {
            self.preview_state.live = None;
            return;
        }
        streamer.last_active = now;
        if let Some(frame) = streamer.latest_frame()
            && push_preview_frame(ctx.uploads, frame)
        {
            if self.preview_state.live_injected != Some(index) {
                log::info!("preview live: first frame injected for idx {index}");
            }
            self.preview_state.live_injected = Some(index);
        }
    }

    fn live_arm(&mut self, now: Instant, target: Option<usize>) {
        let pending_matches =
            matches!(self.preview_state.pending_preview, Some((index, _)) if Some(index) == target);
        if self.preview_state.live_target != target && !pending_matches {
            self.restore_live();
            self.preview_state.pending_preview =
                target.map(|index| (index, now + Duration::from_millis(self.preview_delay())));
        }
        if let Some((index, at)) = self.preview_state.pending_preview {
            if Some(index) != target {
                self.preview_state.pending_preview = None;
            } else if now >= at {
                self.preview_state.pending_preview = None;
                self.preview_state.live_target = Some(index);
                log::info!("preview live: target idx {index}");
            }
        }
    }

    fn restore_live(&mut self) {
        self.preview_state.live_target = None;
        self.preview_state.live_injected = None;
    }

    pub(super) fn preview_slot_idx(&self) -> Option<usize> {
        self.preview_state.live_injected
    }
}

pub(super) fn preview_candidates(
    hover: Option<usize>,
    current: usize,
    engaged: bool,
) -> impl Iterator<Item = usize> {
    hover.into_iter().chain(engaged.then_some(current))
}

fn push_preview_frame(uploads: &UploadQueue, frame: Vec<u8>) -> bool {
    if frame.len() != (PREVIEW_WIDTH * PREVIEW_HEIGHT * 4) as usize {
        return false;
    }
    uploads.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(Upload {
        tier: 3,
        layer: 0,
        x: 0,
        y: 0,
        w: PREVIEW_WIDTH,
        h: PREVIEW_HEIGHT,
        compressed: false,
        data: frame,
        recycle: None,
    });
    true
}

fn push_out_frame(uploads: &UploadQueue, frame: Vec<u8>) -> bool {
    if frame.len() != (PREVIEW_WIDTH * PREVIEW_HEIGHT * 4) as usize {
        return false;
    }
    uploads.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(Upload {
        tier: 4,
        layer: 0,
        x: 0,
        y: 0,
        w: PREVIEW_WIDTH,
        h: PREVIEW_HEIGHT,
        compressed: false,
        data: frame,
        recycle: None,
    });
    true
}

fn push_out_freeze(uploads: &UploadQueue) {
    uploads.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(Upload {
        tier: 5,
        layer: 0,
        x: 0,
        y: 0,
        w: 0,
        h: 0,
        compressed: false,
        data: Vec::new(),
        recycle: None,
    });
}

fn live_idle(slot: &mut Option<LiveStreamer>, now: Instant) {
    let Some(stream) = slot.as_mut() else {
        return;
    };
    if now.duration_since(stream.last_active) > LIVE_IDLE_TIMEOUT {
        *slot = None;
    } else {
        stream.pause();
    }
}

mod tests;
