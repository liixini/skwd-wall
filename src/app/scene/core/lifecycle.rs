use std::sync::Arc;

use crate::frontend::animation::{MotionProfile, MotionTier};
use crate::frontend::scene::layout::{ExtraParams, GridParams, HexParams, Mode, SliceParams};
use crate::frontend::scene::{LaunchAnim, Transition};

use super::super::FrameDemand;
use super::layout_helpers::{center_layout, clamp_inset};
use super::model::SceneCore;

const FRAME_SHOWN_REASON: &str = "frame_shown";

impl SceneCore {
    pub fn set_direct_thumbnails(&mut self, direct: bool) {
        self.direct_thumbnails = direct;
    }

    pub(super) fn camera_precision(&self) -> f32 {
        if self.mode == Mode::Depth {
            (0.05 / self.xp.depth.spacing.max(self.xp.depth.width).max(1.0))
                .max(self.camera.target.abs() * f32::EPSILON)
        } else {
            0.05
        }
    }

    pub(super) fn camera_settled(&self) -> bool {
        self.camera.settled_with_precision(self.camera_precision())
    }

    pub fn motion_scale(&self) -> f32 {
        self.motion.scale
    }

    #[cfg(test)]
    pub(crate) fn motion_duration_ms(&self, tier: MotionTier) -> f32 {
        self.motion_ms(tier)
    }

    pub(super) fn motion_ms(&self, tier: MotionTier) -> f32 {
        (self.motion.profile.duration_ms(tier) * self.motion.scale).max(35.0)
    }

    pub fn set_motion_profile(&mut self, profile: MotionProfile) {
        if self.motion.profile == profile {
            return;
        }
        self.motion.profile = profile;
        self.retime_shared_motion();
        self.motion.needs_frame = true;
    }

    pub(super) fn retime_shared_motion(&mut self) {
        let profile = self.motion.profile;
        let scale = self.motion.scale;
        profile.retime_spring_override(
            &mut self.collection_open,
            profile.duration_ms(MotionTier::Standard) * scale * 100.0
                / self.xp_target.collection.speed,
        );
        let navigation_ms = match self.mode {
            Mode::Depth => self.xp_target.depth.navigation_ms,
            Mode::Collection => {
                profile.duration_ms(MotionTier::Standard) * 100.0 / self.xp_target.collection.speed
            }
            Mode::Grid => profile.duration_ms(MotionTier::Slow),
            _ => profile.duration_ms(MotionTier::Standard),
        };
        profile.retime_spring_override(&mut self.camera, navigation_ms * scale);
        profile.retime_spring_override(
            &mut self.card.tag_open,
            profile.duration_ms(MotionTier::Fast) * scale,
        );
        profile.retime_spring_override(
            &mut self.card.chip_pop,
            profile.duration_ms(MotionTier::Slow) * scale,
        );
        profile.retime_spring_override(
            &mut self.sandy.swirl,
            profile.duration_ms(MotionTier::Slow) * scale,
        );
        profile.retime_spring_override(
            &mut self.sandy.pop,
            profile.duration_ms(MotionTier::Slow) * scale,
        );
        profile.retime_spring_override(
            &mut self.sandy.hero_fade,
            profile.duration_ms(MotionTier::Standard) * scale,
        );
        profile.retime_spring_override(
            &mut self.motion.visibility,
            profile.duration_ms(MotionTier::Standard) * scale,
        );
        profile.retime_spring_override(
            &mut self.motion.center_inset,
            profile.duration_ms(MotionTier::Standard) * scale,
        );
        for spring in self.card.widths.values_mut() {
            profile
                .retime_spring_override(spring, profile.duration_ms(MotionTier::Standard) * scale);
        }
        for spring in self.card.hex_scales.values_mut() {
            profile
                .retime_spring_override(spring, profile.duration_ms(MotionTier::Standard) * scale);
        }
        for spring in self.card.selection.values_mut() {
            profile
                .retime_spring_override(spring, profile.duration_ms(MotionTier::Standard) * scale);
        }
        for spring in self.card.fades.values_mut() {
            profile.retime_spring_override(spring, profile.duration_ms(MotionTier::Fast) * scale);
        }
        if let Some(transition) = self.motion.transition.as_mut() {
            profile.retime_spring_override(
                &mut transition.progress,
                profile.duration_ms(MotionTier::Standard) * scale,
            );
        }
    }

    pub fn set_motion_scale(&mut self, scale: f32) {
        let scale = scale.clamp(0.1, 2.0);
        if (self.motion.scale - scale).abs() < f32::EPSILON {
            return;
        }
        self.motion.scale = scale;
        self.retime_shared_motion();
        self.motion.needs_frame = true;
    }

    pub fn set_card_flip_options(
        &mut self,
        duration_ms: f32,
        animate_shader: bool,
        animate_back: bool,
    ) {
        let duration_ms = duration_ms.clamp(100.0, 5_000.0);
        if (self.card.flip_duration_ms - duration_ms).abs() > f32::EPSILON {
            self.card.flip_duration_ms = duration_ms;
            self.card.flip.retime(duration_ms);
        }
        self.card.flip_shader_enabled = animate_shader;
        self.card.flip_back_enabled = animate_back;
        self.motion.needs_frame = true;
    }

    pub fn set_filter_swap_ms(&mut self, milliseconds: f32) {
        self.card.filter_ms = milliseconds.max(50.0);
    }

    #[cfg(test)]
    pub(crate) fn filter_swap_ms(&self) -> f32 {
        self.card.filter_ms
    }

    #[cfg(test)]
    pub(crate) fn card_flip_options(&self) -> (f32, bool, bool) {
        (self.card.flip_duration_ms, self.card.flip_shader_enabled, self.card.flip_back_enabled)
    }

    pub fn set_sandy_res_scale(&mut self, scale: f32) {
        let clamped = scale.clamp(0.25, 1.0);
        if (self.sandy.res_scale - clamped).abs() > 1e-4 {
            self.sandy.res_scale = clamped;
            self.motion.needs_frame = true;
        }
    }

    pub fn set_sandy_lod(&mut self, lod_max: f32) {
        let clamped = lod_max.clamp(1.0, 4.0);
        if (self.sandy.lod_max - clamped).abs() > 1e-4 {
            self.sandy.lod_max = clamped;
            self.motion.needs_frame = true;
        }
    }

    pub(super) fn anim_reason(&self) -> &'static str {
        if self.motion.needs_frame {
            "needs_frame"
        } else if self.render.placeholders && !self.hidden() {
            "placeholders"
        } else if self.sandy.swirl.target > 0.5 {
            "sandy_ring"
        } else if self.motion.transition.is_some() {
            "transition"
        } else if self.frame_shown.is_some() {
            FRAME_SHOWN_REASON
        } else if !self.motion.entrance.settled() {
            "entrance"
        } else if !self.collection_open.settled() {
            "collection_open"
        } else if !self.camera_settled() {
            "camera"
        } else if self.card.widths.values().any(|spr| !spr.settled()) {
            "widths"
        } else if self.card.hex_scales.values().any(|spr| !spr.settled()) {
            "hex_scales"
        } else if !self.card.fades.is_empty() {
            "fades"
        } else if !self.card.selection.is_empty() {
            "sel"
        } else if self.scroll_vel > 0.0 {
            "scroll_vel"
        } else if !self.card.flip.settled() {
            "flip"
        } else if (self.card.det_p - self.card.det_target).abs() > 0.001 {
            "detail"
        } else if (self.card.fav_fill - self.card.fav_target).abs() > 0.001 {
            "fav"
        } else if !self.motion.visibility.settled() {
            "visibility"
        } else if !self.motion.center_inset.settled() {
            "center_inset"
        } else if !self.sp.settled_to(&self.sp_target) {
            "slice_params"
        } else if !self.gp.settled_to(&self.gp_target) {
            "grid_params"
        } else if !self.hp.settled_to(&self.hp_target) {
            "hex_params"
        } else if !self.xp.settled_to(&self.xp_target) {
            "extra_params"
        } else if !self.sandy.prog.settled() {
            "sandy_transition"
        } else if self.mode == Mode::Sandy && self.sandy.motion > 0.001 {
            "sandy_lod"
        } else if let Some(reason) = self.hand_reason() {
            reason
        } else {
            "none"
        }
    }

    pub fn frame_demand(&self) -> FrameDemand {
        let mut demand = FrameDemand::Idle;
        if self.motion.needs_frame
            || (self.render.placeholders && !self.hidden())
            || self.sandy.hero_restore_at.is_some()
        {
            demand = FrameDemand::Passive;
        }
        if !self.card.fades.is_empty()
            || !self.motion.visibility.settled()
            || (!self.motion.entrance.settled() && self.motion.launch_anim == LaunchAnim::Fade)
            || !self.sandy.hero_fade.settled()
        {
            demand = FrameDemand::Preview;
        }
        if self.motion.transition.is_some()
            || self.frame_shown.is_some()
            || (!self.motion.entrance.settled() && self.motion.launch_anim != LaunchAnim::Fade)
            || !self.camera_settled()
            || !self.collection_open.settled()
            || self.card.widths.values().any(|spr| !spr.settled())
            || self.card.hex_scales.values().any(|spr| !spr.settled())
            || !self.card.selection.is_empty()
            || !self.card.flip.settled()
            || !self.card.tag_open.settled()
            || !self.card.chip_pop.settled()
            || (self.card.det_p - self.card.det_target).abs() > 0.001
            || (self.card.fav_fill - self.card.fav_target).abs() > 0.001
            || !self.motion.center_inset.settled()
            || !self.sp.settled_to(&self.sp_target)
            || !self.gp.settled_to(&self.gp_target)
            || !self.hp.settled_to(&self.hp_target)
            || !self.xp.settled_to(&self.xp_target)
            || !self.sandy.prog.settled()
            || !self.sandy.bmix.settled()
            || self.sandy.swirl.target > 0.5
            || !self.sandy.swirl.settled()
            || !self.sandy.pop.settled()
            || !self.card.filter_old.is_empty()
            || !self.card.filter_in.is_empty()
            || self.hand_animating()
        {
            demand = FrameDemand::Motion;
        }
        if self.scroll_vel > 0.0 || (self.mode == Mode::Sandy && self.sandy.motion > 0.001) {
            demand = FrameDemand::Direct;
        }
        demand
    }

    pub fn is_animating(&self) -> bool {
        self.frame_demand() != FrameDemand::Idle
    }

    pub fn theme_target(&self) -> Option<usize> {
        self.hover.or(Some(self.current))
    }

    pub fn set_visible(&mut self, visible: bool) {
        let target = if visible { 1.0 } else { 0.0 };
        if (self.motion.visibility.target - target).abs() > f32::EPSILON {
            if self.motion.open_fade_ms <= 0.0 {
                self.motion.visibility.snap(target);
            } else {
                let mut spr = self
                    .motion
                    .profile
                    .override_spring(self.motion.visibility.x, self.motion.open_fade_ms.max(35.0));
                spr.retarget(target);
                self.motion.visibility = spr;
            }
            self.motion.needs_frame = true;
        }
    }

    pub fn await_window(&mut self) {
        self.motion.entrance.snap(self.motion.open_fade_from);
        self.motion.entrance_pending = false;
    }

    pub fn begin_open_fade(&mut self) {
        if self.motion.launch_anim == LaunchAnim::None || self.motion.open_fade_ms <= 0.0 {
            self.motion.entrance.snap(1.0);
            self.motion.entrance_pending = false;
            self.motion.needs_frame = true;
            return;
        }
        let mut spr = self
            .motion
            .profile
            .override_spring(self.motion.open_fade_from, self.motion.open_fade_ms.max(35.0));
        spr.snap(self.motion.open_fade_from);
        self.motion.entrance = spr;
        self.motion.entrance_pending = true;
        self.motion.needs_frame = true;
    }

    pub fn set_launch_animation(&mut self, kind: &str) {
        self.motion.launch_anim = LaunchAnim::from_key(kind);
    }

    pub(super) fn start_pending_entrance(&mut self) {
        if self.motion.entrance_pending {
            self.motion.entrance_pending = false;
            self.motion.entrance.retarget(1.0);
            self.motion.needs_frame = true;
        }
    }

    pub fn set_open_fade_from(&mut self, from: f32) {
        let from = from.clamp(0.0, 1.0);
        if (self.motion.open_fade_from - from).abs() < f32::EPSILON {
            return;
        }
        self.motion.open_fade_from = from;
        if !self.motion.entrance.settled() && self.motion.entrance.x < from {
            self.motion.entrance.x = from;
        }
    }

    pub fn set_open_fade_ms(&mut self, ms: f32) {
        let ms = ms.max(0.0);
        if (self.motion.open_fade_ms - ms).abs() < f32::EPSILON {
            return;
        }
        self.motion.open_fade_ms = ms;
        if !self.motion.entrance.settled() {
            if ms <= 0.0 {
                self.motion.entrance.snap(1.0);
                self.motion.entrance_pending = false;
            } else {
                let mut spr =
                    self.motion.profile.override_spring(self.motion.entrance.x, ms.max(35.0));
                spr.retarget(1.0);
                self.motion.entrance = spr;
            }
        }
    }

    pub fn open_fade(&self) -> f32 {
        (self.motion.entrance.x * self.motion.visibility.value()).clamp(0.0, 1.0)
    }

    pub fn open_fade_settled(&self) -> bool {
        self.motion.entrance.settled() && self.motion.visibility.settled()
    }

    #[cfg(test)]
    pub fn open_fade_ms(&self) -> f32 {
        self.motion.open_fade_ms
    }

    pub fn hidden(&self) -> bool {
        self.motion.visibility.target == 0.0 && self.motion.visibility.settled()
    }

    pub fn set_center_inset(&mut self, inset: f32) {
        let target = inset.max(0.0);
        if (self.motion.center_inset.target - target).abs() > f32::EPSILON {
            self.motion.center_inset.retarget(target);
            self.motion.needs_frame = true;
        }
    }

    pub fn set_reading_direction(&mut self, rtl: bool) {
        if self.reading_rtl != rtl {
            self.reading_rtl = rtl;
            self.motion.needs_frame = true;
        }
    }

    pub fn set_filter_bar_footprint(&mut self, footprint: Option<(bool, f32, f32)>) {
        if self.filter_bar_footprint == footprint {
            return;
        }
        self.filter_bar_footprint = footprint;
        self.motion.needs_frame = true;
    }

    pub fn wall_composition_center(&self) -> (f32, f32) {
        let (vw, vh) = self.viewport;
        let gap = 8.0 * self.motion.scale;
        match self.filter_bar_footprint {
            Some((true, bar_w, _)) => {
                let shift = if self.reading_rtl { -(bar_w + gap) } else { bar_w + gap };
                (vw * 0.5 + shift * 0.5, vh * 0.5)
            }
            Some((false, _, bar_h)) => (vw * 0.5, vh * 0.5 + (bar_h + gap) * 0.5),
            None => (vw * 0.5, vh * 0.5),
        }
    }

    pub(super) fn center_inset_px(&self) -> f32 {
        clamp_inset(self.viewport.0, self.motion.center_inset.x)
    }

    pub(super) fn center_x(&self) -> f32 {
        center_layout(self.viewport.0, self.motion.center_inset.x, self.reading_rtl).0
    }

    pub(super) fn avail_w(&self) -> f32 {
        center_layout(self.viewport.0, self.motion.center_inset.x, self.reading_rtl).1
    }

    pub fn set_params(
        &mut self,
        sp: SliceParams,
        gp: GridParams,
        hp: HexParams,
        xp: &ExtraParams,
        animate: bool,
    ) {
        let topology_changed = match self.mode {
            Mode::Slices => self.sp_target.topology_differs(&sp),
            Mode::Grid => self.gp_target.topology_differs(&gp),
            Mode::Hex => self.hp_target.topology_differs(&hp),
            Mode::Sandy | Mode::Hand | Mode::Depth | Mode::Collection => false,
        };
        if animate && topology_changed {
            self.begin_transition(0, [0.5, 0.5]);
        }
        if (self.mode == Mode::Slices && !self.sp_target.settled_to(&sp))
            || (self.mode == Mode::Hex && !self.hp_target.settled_to(&hp))
        {
            self.layout_camera_anchor = true;
        }
        self.sp_target = sp;
        self.gp_target = gp;
        self.hp_target = hp;
        let speed_changed = self.xp_target.depth.navigation_ms != xp.depth.navigation_ms
            || self.xp_target.collection.speed != xp.collection.speed;
        self.xp_target = *xp;
        if speed_changed {
            self.retime_shared_motion();
        }
        if !animate {
            self.sp = sp;
            self.gp = gp;
            self.hp = hp;
            self.xp = *xp;
        }
        self.motion.needs_frame = true;
    }

    pub fn set_grid_params(&mut self, gp: GridParams, animate: bool) {
        let xp = self.xp;
        self.set_params(self.sp, gp, self.hp, &xp, animate);
    }

    pub(super) fn morph_params(&mut self, dt: f32) {
        let amt = self.motion.profile.approach_k(dt, MotionTier::Standard);
        if !self.sp.settled_to(&self.sp_target) {
            self.sp.morph_toward(&self.sp_target, amt);
            if self.sp.settled_to(&self.sp_target) {
                self.sp = self.sp_target;
            }
        }
        if !self.gp.settled_to(&self.gp_target) {
            self.gp.morph_toward(&self.gp_target, amt);
            if self.gp.settled_to(&self.gp_target) {
                self.gp = self.gp_target;
            }
        }
        if !self.hp.settled_to(&self.hp_target) {
            self.hp.morph_toward(&self.hp_target, amt);
            if self.hp.settled_to(&self.hp_target) {
                self.hp = self.hp_target;
            }
        }
        if !self.xp.settled_to(&self.xp_target) {
            self.xp.morph_toward(&self.xp_target, amt);
            if self.xp.settled_to(&self.xp_target) {
                self.xp = self.xp_target;
            }
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        if mode == self.mode {
            return;
        }
        self.mode = mode;
        self.collection_open.snap(0.0);
        self.collection_card = None;
        self.retime_shared_motion();
        self.camera.set_zeta(1.0);
        self.card.widths.clear();
        self.card.hex_scales.clear();
        self.hover = None;
        self.camera.snap(0.0);
        self.camera_jump = false;
        self.period = None;
        self.ring = None;
        self.layout_camera_anchor =
            matches!(mode, Mode::Slices | Mode::Depth | Mode::Hex | Mode::Collection);
        self.card.filter_cache.clear();
        self.card.filter_old.clear();
        self.card.filter_cell.clear();
        self.card.filter_in.clear();
        self.card.filter_wave = 10.0;
        self.hand.deal = None;
        self.hand.lift.clear();
        self.hand.drag = None;
        self.motion.needs_frame = true;
    }

    pub fn begin_transition(&mut self, kind: u32, origin: [f32; 2]) {
        if self.motion.transition.is_some() || self.render.instances.is_empty() {
            return;
        }
        let mut progress = self.motion.profile.spring(0.0, MotionTier::Standard);
        progress.retarget(1.0);
        self.motion.transition = Some(Transition {
            old: Arc::new(self.render.instances.clone()),
            progress,
            kind,
            origin,
        });
        self.motion.needs_frame = true;
    }

    pub(crate) fn layout_transition_active(&self) -> bool {
        self.motion.transition.is_some()
    }

    pub fn touch(&mut self) {
        self.motion.needs_frame = true;
    }

    pub(crate) fn sandy_ring_engaged(&self) -> bool {
        self.sandy.swirl.target > 0.5 || self.sandy.swirl.x > 0.02
    }
}
