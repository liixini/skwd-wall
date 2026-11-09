use std::time::Instant;

use crate::frontend::animation::MotionTier;
use crate::frontend::scene::layout::Mode;
use crate::frontend::scene::sandy::{
    LOD_VEL_HI, LOD_VEL_LO, clamp_cam, edge_pan, sandy_motion_from_vel, strip_bottom,
};

use super::model::{RebuildCtx, SceneCore};

const SELECT_DISTANCE: f32 = 8.0;

impl SceneCore {
    pub fn sandy_settle_now(&mut self) {
        self.sandy.prog.snap(1.0);
        self.sandy.swirl = self.motion.profile.spring(0.0, MotionTier::Slow);
        self.sandy.swirl_hold = 0.0;
        self.sandy.bmix = self.motion.profile.spring(1.0, MotionTier::Fast);
        self.sandy.bfrom = None;
        self.sandy.bfrom2 = None;
        self.sandy.bcut = 1.0;
        self.sandy.bto = None;
        self.sandy.storm_to = None;
        self.sandy.ring_to = None;
        self.sandy.filter_from = None;
        self.sandy.carry = 0.0;
        self.motion.needs_frame = true;
    }

    pub fn sandy_pointer(&mut self, x: f32, y: f32, hit: Option<usize>, count: usize) {
        let (vw, vh) = self.viewport;
        let (source_x, source_y) = self.unplace_pointer(x, y);
        let local_y = source_y - self.xp.sandy.offset_y * vh * 0.5;
        let pan = edge_pan(source_x, local_y, vw, vh, self.xp.sandy.strip_h());
        if pan != self.sandy.edge_pan {
            self.sandy.edge_pan = pan;
            self.motion.needs_frame = true;
        }
        if pan != 0.0 {
            self.sandy.cam_free = true;
            return;
        }
        let bottom = strip_bottom(vh);
        let over_strip = local_y >= bottom - self.xp.sandy.strip_h() && local_y <= bottom;
        if !over_strip {
            if self.sandy.cam_free {
                self.sandy.cam_free = false;
                self.motion.needs_frame = true;
            }
            return;
        }
        self.sandy.cam_free = true;
        let Some(idx) = hit else {
            return;
        };
        if idx == self.current {
            return;
        }
        let far_enough = self.sandy.sel_pos.is_none_or(|(sx, sy)| {
            (source_x - sx).abs() + (source_y - sy).abs() >= SELECT_DISTANCE
        });
        if far_enough {
            self.sandy.sel_pos = Some((source_x, source_y));
            self.set_current(idx, count);
            self.sandy.cam_free = true;
        }
    }

    pub fn sandy_displayed(&self) -> usize {
        self.sandy.ring_to.or(self.sandy.bto).or(self.sandy.storm_to).unwrap_or(self.current)
    }

    pub(super) fn sandy_ring_fade(&mut self, to: usize) {
        let done = self.sandy.bmix.settled() || self.sandy.bmix.x >= 0.90;
        if !done {
            return;
        }
        self.sandy.filter_from = None;
        let displayed = self.sandy.ring_to.unwrap_or(self.current);
        if displayed == to {
            self.sandy.ring_to = Some(to);
            return;
        }
        self.sandy.bfrom2 = self.sandy.bfrom.or(Some(displayed));
        self.sandy.bcut =
            if self.sandy.bmix.settled() { 1.0 } else { self.sandy.bmix.x.clamp(0.0, 1.0) };
        self.sandy.bfrom = Some(displayed);
        self.sandy.ring_to = Some(to);
        self.sandy.bmix = self.motion.profile.override_spring(
            0.0,
            (self.xp.sandy.blend_ms * 0.5 * self.xp.sandy.ring_blend).clamp(150.0, 3000.0),
        );
        self.sandy.bmix.retarget(1.0);
    }

    #[cfg(test)]
    pub fn sandy_filter_from_idx(&self) -> Option<usize> {
        self.sandy.filter_from
    }

    pub(super) fn sandy_prog_tick(&mut self, dt: f32) {
        if !self.sandy_ring_engaged() {
            self.sandy.prog.tick(dt);
        }
    }

    pub(super) fn sandy_ring_chain_tick(&mut self) {
        let ring_live = self.sandy.swirl.target > 0.5 || self.sandy.swirl.x > 0.001;
        if ring_live && self.sandy.ring_to.is_some_and(|to| to != self.current) {
            self.sandy_ring_fade(self.current);
        }
        if self.sandy.bmix.settled() && self.sandy.swirl.settled() && self.sandy.swirl.x < 0.5 {
            self.sandy.bto = None;
            self.sandy.ring_to = None;
            self.sandy.filter_from = None;
        }
    }

    pub(super) fn sandy_hero_animating(&self) -> bool {
        !self.sandy.prog.settled()
            || self.sandy.bto.is_some()
            || self.sandy.storm_to.is_some()
            || self.sandy.ring_to.is_some()
            || self.sandy.swirl.target > 0.5
            || self.sandy.swirl.x > 0.001
    }

    pub(super) fn sandy_swirl_tick(&mut self, dt: f32) {
        self.sandy.pop.tick(dt);
        self.sandy.swirl.tick(dt);
        if self.sandy.swirl.target > 0.5 {
            self.sandy.swirl_hold -= dt;
            if self.sandy.swirl_hold <= 0.0 {
                self.sandy.swirl.retarget(0.0);
                if !self.sandy.prog.settled() {
                    self.sandy.prog.snap(1.0);
                }
                self.sandy.storm_to = None;
                self.motion.needs_frame = true;
            }
        }
    }

    pub(super) fn sandy_select(&mut self, idx: usize, count: usize) -> bool {
        self.sandy.cam_free = false;
        self.sandy.pop.snap(0.0);
        self.sandy.pop.retarget(1.0);
        if self.hero_concealed() {
            self.sandy_settle_now();
            self.current = idx;
            return false;
        }
        let swirling = self.sandy.swirl.target > 0.5 || self.sandy.swirl.x > 0.02;
        if swirling {
            self.sandy.swirl.retarget(1.0);
            self.sandy.swirl_hold = self.xp.sandy.ring_hold;
            self.sandy.bto = None;
            self.sandy_ring_fade(idx);
            self.current = idx;
            self.motion.needs_frame = true;
            return false;
        }
        let new_dir = if self.travel(self.current, idx, count) >= 0 { 1.0 } else { -1.0 };
        let flipped = new_dir != self.sandy.dir;
        let ring_active = self.sandy.swirl.target > 0.5 || self.sandy.swirl.x > 0.001;
        let blending =
            (self.sandy.bfrom.is_some() && !self.sandy.bmix.settled() && self.sandy.bmix.x < 0.90)
                || ring_active;
        let interrupted = !self.sandy.prog.settled() || blending;
        if interrupted && self.xp.sandy.swap_loop && !flipped && !blending {
            self.sandy_swap_pick(idx);
        } else if interrupted && blending {
            self.sandy.swirl.retarget(1.0);
            self.sandy.swirl_hold = self.xp.sandy.ring_hold;
            self.sandy.bto = None;
            self.sandy.storm_to = None;
            self.sandy_ring_fade(idx);
        } else if interrupted {
            self.sandy_ring_pick(idx);
        } else {
            self.sandy_storm_pick(idx, new_dir);
        }
        true
    }

    pub(super) fn sandy_swap_pick(&mut self, idx: usize) {
        self.sandy.bfrom = self.sandy.bto.or(Some(self.current));
        self.sandy.bfrom2 = None;
        self.sandy.bcut = 1.0;
        self.sandy.bto = Some(idx);
        self.sandy.bmix =
            self.motion.profile.override_spring(0.0, self.xp.sandy.blend_ms.clamp(100.0, 4000.0));
        self.sandy.bmix.retarget(1.0);
    }

    pub(super) fn sandy_ring_pick(&mut self, idx: usize) {
        self.sandy.swirl.retarget(1.0);
        self.sandy.swirl_hold = self.xp.sandy.ring_hold;
        self.sandy.bto = None;
        self.sandy.bfrom2 = None;
        self.sandy.bcut = 1.0;
        self.sandy.bfrom = Some(self.sandy.storm_to.unwrap_or(self.current));
        self.sandy.ring_to = Some(idx);
        self.sandy.bmix = self.motion.profile.override_spring(
            0.0,
            (self.xp.sandy.blend_ms * 0.5 * self.xp.sandy.ring_blend).clamp(150.0, 3000.0),
        );
        self.sandy.bmix.retarget(1.0);
    }

    pub(super) fn sandy_storm_pick(&mut self, idx: usize, new_dir: f32) {
        self.sandy.from = self.sandy.bto.take().or(Some(self.current));
        self.preview_state.sandy_out_pending = true;
        self.sandy.dir = new_dir;
        self.sandy.carry = 0.0;
        self.sandy.bfrom = None;
        self.sandy.bfrom2 = None;
        self.sandy.bcut = 1.0;
        self.sandy.bto = None;
        self.sandy.storm_to = Some(idx);
        self.sandy.ring_to = None;
        self.sandy.bmix = self.motion.profile.spring(1.0, MotionTier::Fast);
        self.sandy.prog =
            self.motion.profile.override_tween(0.0, self.xp.sandy.duration_ms.clamp(200.0, 6000.0));
        self.sandy.prog.retarget(1.0);
    }

    pub(super) fn tick_sandy(&mut self, ctx: &RebuildCtx<'_>, now: Instant, dt: f32) {
        self.sandy_prog_tick(dt);
        self.sandy.bmix.tick(dt);
        self.sandy_swirl_tick(dt);
        self.hero_fade_tick(now, dt);
        if self.mode == Mode::Sandy && self.sandy.edge_pan != 0.0 && !ctx.filtered.is_empty() {
            let target = self.camera.target + self.sandy.edge_pan * self.xp.sandy.edge_speed * dt;
            let target =
                if self.connected() { target } else { clamp_cam(target, ctx.filtered.len()) };
            self.camera.retarget(target);
            self.motion.needs_frame = true;
        }
        if self.sandy.bfrom.is_some() && self.sandy.bmix.settled() {
            self.sandy.bfrom = None;
            self.sandy.bfrom2 = None;
            self.sandy.bcut = 1.0;
        }
        self.sandy_ring_chain_tick();
        let motion_target = if self.sandy.lod_max > 1.0 && self.mode == Mode::Sandy {
            sandy_motion_from_vel(self.camera.v, LOD_VEL_LO, LOD_VEL_HI)
        } else {
            0.0
        };
        self.sandy.motion = self.motion.profile.approach(
            self.sandy.motion,
            motion_target,
            dt,
            MotionTier::Standard,
        );
    }
}
