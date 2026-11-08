use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::frontend::animation::{MotionTier, Spring};
use crate::frontend::nav::{Ends, follow, land, lap, rebase, step};
use crate::frontend::scene::layout::{GridLayout, Mode};

use super::layout_helpers::slice_scroll_steps;

const RING_DRIFT: f32 = 0.5;
use super::model::SceneCore;

impl SceneCore {
    pub fn shift_after_insert(&mut self, pos: usize) {
        let bump = |slot: &mut usize| {
            if *slot >= pos {
                *slot += 1;
            }
        };
        let bump_opt = |slot: &mut Option<usize>| {
            if let Some(idx) = slot.as_mut()
                && *idx >= pos
            {
                *idx += 1;
            }
        };
        bump(&mut self.current);
        bump_opt(&mut self.hover);
        bump_opt(&mut self.collection_card);
        bump_opt(&mut self.card.flipped);
        bump_opt(&mut self.sandy.from);
        bump_opt(&mut self.sandy.bfrom);
        bump_opt(&mut self.sandy.bfrom2);
        bump_opt(&mut self.sandy.bto);
        bump_opt(&mut self.sandy.storm_to);
        bump_opt(&mut self.sandy.ring_to);
        bump_opt(&mut self.sandy.filter_from);
        self.hand_shift_after_insert(pos);
        let shift_map = |map: &mut HashMap<usize, Spring>| {
            *map = map.drain().map(|(k, v)| (if k >= pos { k + 1 } else { k }, v)).collect();
        };
        shift_map(&mut self.card.widths);
        shift_map(&mut self.card.fades);
        shift_map(&mut self.card.hex_scales);
        shift_map(&mut self.card.selection);
        shift_map(&mut self.hand.lift);
        self.motion.needs_frame = true;
    }

    pub fn set_current(&mut self, idx: usize, count: usize) {
        if count == 0 {
            return;
        }
        self.user_engaged = true;
        let idx = idx.min(count - 1);
        if idx == self.current {
            return;
        }
        self.close_collection();
        self.hover = None;
        if self.card.flipped.is_some() {
            if self.mode == Mode::Hand {
                self.close_flip();
            } else {
                self.card.flipped = None;
                self.card.flip.snap(0.0);
            }
        }
        if self.mode == Mode::Slices {
            self.slice_select(idx);
        }
        if matches!(self.mode, Mode::Hex | Mode::Depth) {
            if self.mode == Mode::Hex {
                self.hex_row = idx % self.hp.rows.max(1);
            }
            let old = self.current;
            let motion_ms = if self.mode == Mode::Depth {
                self.xp.depth.navigation_ms * self.motion.scale
            } else {
                self.motion_ms(MotionTier::Standard)
            };
            let mut off = self
                .motion
                .profile
                .override_spring(self.card.selection.get(&old).map_or(1.0, |spr| spr.x), motion_ms);
            off.retarget(0.0);
            self.card.selection.insert(old, off);
            let mut on = self
                .motion
                .profile
                .override_spring(self.card.selection.get(&idx).map_or(0.0, |spr| spr.x), motion_ms);
            on.retarget(1.0);
            self.card.selection.insert(idx, on);
        }
        if self.mode == Mode::Grid && self.kb_nav {
            self.grid_follow(idx);
        }
        if self.mode == Mode::Sandy && !self.sandy_select(idx, count) {
            return;
        }
        if self.mode == Mode::Hand {
            self.hand_select(idx, count);
        }
        self.camera_jump |= self.travel(self.current, idx, count).unsigned_abs()
            > self.step_span() as u64
            && (self.mode != Mode::Grid || self.kb_nav);
        self.current = idx;
        self.motion.needs_frame = true;
    }

    fn step_span(&self) -> usize {
        match self.mode {
            Mode::Grid => self.effective_grid_params().cols.max(1),
            Mode::Hex => self.hp.rows.max(1),
            _ => 1,
        }
    }

    pub(super) fn aim_camera(&mut self, target: f32, reach: f32) {
        let (target, _) = self.near(target, self.camera.target);
        if self.camera_jump {
            land(&mut self.camera, target, reach);
        } else {
            self.camera.retarget(target);
        }
        if let Some(period) = self.period {
            rebase(&mut self.camera, period);
        }
    }

    pub fn connected(&self) -> bool {
        self.period.is_some()
    }

    pub(super) fn set_period(&mut self, period: f32, rest: f32, window: f32, count: usize) {
        self.period = (self.loop_around && period > window).then_some(period);
        let moved = self.ring.is_some_and(|(prev, was)| {
            self.period.is_none() || count != was || (prev - rest).abs() > RING_DRIFT
        });
        self.layout_camera_anchor |= moved;
        self.ring = self.period.map(|_| (rest, count));
    }

    pub(super) fn near(&self, pos: f32, around: f32) -> (f32, i64) {
        self.period.map_or((pos, 0), |period| {
            let turns = lap(pos, around, period);
            (pos + turns * period, turns as i64)
        })
    }

    pub(super) fn travel(&self, from: usize, to: usize, count: usize) -> i64 {
        let delta = to as i64 - from as i64;
        if !self.connected() {
            return delta;
        }
        let half = count as i64 / 2;
        (delta + half).rem_euclid(count as i64) - half
    }

    pub(super) fn slot_window(&self, center: f32, reach: f32, count: usize) -> (i64, i64) {
        let lo = (center - reach).floor() as i64;
        let hi = (center + reach).ceil() as i64;
        if self.connected() { (lo, hi) } else { (lo.max(0), hi.min(count as i64 - 1)) }
    }

    pub(super) fn scroll_bounds(&self, max: f32) -> RangeInclusive<f32> {
        if self.connected() { f32::MIN..=f32::MAX } else { 0.0..=max }
    }

    fn slice_select(&mut self, idx: usize) {
        let old = self.current;
        let sp = self.sp;
        let motion_ms = self.motion_ms(MotionTier::Standard);
        let zeta = if sp.wobble { 0.5 } else { 1.0 };
        let w_old = self
            .card
            .widths
            .entry(old)
            .or_insert_with(|| self.motion.profile.override_spring(sp.expanded_w, motion_ms));
        w_old.set_zeta(zeta);
        w_old.retarget(sp.slice_w);
        let w_new = self
            .card
            .widths
            .entry(idx)
            .or_insert_with(|| self.motion.profile.override_spring(sp.slice_w, motion_ms));
        w_new.set_zeta(zeta);
        w_new.retarget(sp.expanded_w);
        self.card
            .selection
            .entry(old)
            .or_insert_with(|| self.motion.profile.override_spring(1.0, motion_ms))
            .retarget(0.0);
        self.card
            .selection
            .entry(idx)
            .or_insert_with(|| self.motion.profile.override_spring(0.0, motion_ms))
            .retarget(1.0);
    }

    fn grid_follow(&mut self, idx: usize) {
        let gp = self.effective_grid_params();
        let cell_h = gp.cell_h();
        let view_h = gp.total_h();
        let (row_top, item_h) = match gp.layout {
            GridLayout::Editorial => {
                let block_items = self.editorial_block_items(gp.cols.max(1));
                if block_items == 1 {
                    (idx as f32 * cell_h, gp.thumb_h)
                } else {
                    let block_h = gp.thumb_h * 2.0 + gp.gap_y;
                    ((idx / block_items) as f32 * (block_h + gp.gap_y), block_h)
                }
            }
            _ => ((idx / gp.cols.max(1)) as f32 * cell_h, cell_h),
        };
        let (mid, _) = self.near(row_top + item_h * 0.5, self.camera.target + view_h * 0.5);
        let bounds = self.scroll_bounds(f32::MAX);
        follow(&mut self.camera, mid - item_h * 0.5, item_h, view_h, bounds);
    }

    pub(super) fn start_camera(&self) -> f32 {
        match self.mode {
            Mode::Slices => self.sp.expanded_w * 0.5,
            Mode::Hex => self.hp.r,
            _ => 0.0,
        }
    }

    pub fn relayout(&mut self, count: usize) {
        self.collection_open.snap(0.0);
        self.collection_card = None;
        self.card.widths.clear();
        self.current = self.current.min(count.saturating_sub(1));
        if self.mode == Mode::Hand {
            self.hand_start_middle(count);
        }
        self.hover = None;
        self.motion.needs_frame = true;
    }

    pub fn reset_to_start(&mut self, count: usize) {
        self.reset_to_index(0, count);
    }

    pub fn restore_selection(&mut self, idx: usize, count: usize, camera: Option<f32>) {
        if self.user_engaged || count == 0 {
            return;
        }
        self.user_engaged = true;
        self.reset_to_index(idx, count);
        self.layout_camera_anchor = matches!(
            self.mode,
            Mode::Slices | Mode::Depth | Mode::Hex | Mode::Grid | Mode::Collection
        );
        if let Some(camera) = camera.filter(|value| value.is_finite()) {
            self.camera.snap(camera);
            self.layout_camera_anchor = false;
            self.sandy.cam_free = self.mode == Mode::Sandy;
        }
    }

    pub fn reset_to_index(&mut self, idx: usize, count: usize) {
        self.collection_open.snap(0.0);
        self.collection_card = None;
        self.card.widths.clear();
        self.card.hex_scales.clear();
        self.card.selection.clear();
        self.current = idx.min(count.saturating_sub(1));
        self.hex_row = self.current % self.hp.rows.max(1);
        self.hover = None;
        if self.mode == Mode::Sandy {
            self.camera.snap(crate::frontend::scene::sandy::clamp_cam(self.current as f32, count));
        } else if self.mode == Mode::Hand {
            self.hand_reset(count);
        } else {
            self.camera.snap(self.start_camera());
            self.layout_camera_anchor =
                matches!(self.mode, Mode::Slices | Mode::Depth | Mode::Hex | Mode::Collection);
        }
        self.motion.needs_frame = true;
    }

    pub fn on_decoded(&mut self, store_idx: usize) {
        let mut spr = self.motion.profile.spring(0.0, MotionTier::Fast);
        spr.retarget(1.0);
        self.card.fades.insert(store_idx, spr);
        self.motion.needs_frame = true;
    }

    pub fn width_of(&self, idx: usize) -> f32 {
        if let Some(spr) = self.card.widths.get(&idx) {
            spr.x
        } else if idx == self.current {
            self.sp.expanded_w
        } else {
            self.sp.slice_w
        }
    }

    pub(super) fn target_width_of(&self, idx: usize) -> f32 {
        if idx == self.current { self.sp.expanded_w } else { self.sp.slice_w }
    }

    pub fn camera_target(&self) -> f32 {
        self.camera.target
    }

    pub fn camera_pos(&self) -> f32 {
        self.camera.x
    }

    pub fn visible_range(&self) -> (usize, usize) {
        (self.vis_lo, self.vis_hi)
    }

    pub fn grid_scroll(&mut self, rows: f32) {
        let step = self.effective_grid_params().cell_h();
        self.camera.retarget(self.camera.target + rows * step);
        self.scroll_vel += rows.abs();
        self.motion.needs_frame = true;
    }

    pub fn grid_scroll_end(&self, rows: f32) -> bool {
        if rows > 0.0 {
            self.camera.target >= self.grid_max_scroll
        } else {
            self.camera.target <= 0.0
        }
    }

    pub fn card_flip_effect(&self) -> u32 {
        self.card.flip_effect
    }

    pub fn set_effect(&mut self, id: u32) {
        self.card.flip_effect = id;
        if self.card.flipped.is_some() {
            if !self.card.flip_shader_enabled && !self.card.flip_back_enabled {
                self.card.flip.snap(1.0);
            } else {
                self.card.flip.snap(0.0);
                self.card.flip.retarget(1.0);
            }
        }
        self.motion.needs_frame = true;
    }

    pub fn slice_scroll(&mut self, amount: f32, count: usize) {
        if count == 0 {
            return;
        }
        let steps = self.slice_steps(amount);
        self.slice_move(steps, count, Ends::Stop);
    }

    pub fn slice_steps(&mut self, amount: f32) -> i64 {
        slice_scroll_steps(&mut self.scroll_accum, amount)
    }

    pub fn slice_move(&mut self, steps: i64, count: usize, ends: Ends) {
        if count == 0 {
            return;
        }
        if steps == 0 {
            self.motion.needs_frame = true;
            return;
        }
        self.scroll_vel += steps.unsigned_abs() as f32;
        let target = step(self.current, -(steps as i32), count, ends).unwrap_or(self.current);
        self.set_current(target, count);
    }
}
