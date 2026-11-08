use super::layout::{CardCorners, feq, lerp, slice_midline_width};

pub const STRIP_MARGIN: f32 = 26.0;
pub const BAR_ZONE: f32 = 48.0;
pub const BAR_GAP: f32 = 12.0;

pub const LOD_VEL_LO: f32 = 1.5;
pub const LOD_VEL_HI: f32 = 6.0;

pub const HERO_HALF_W: f32 = 391.0;
pub const HERO_HALF_H: f32 = 220.0;

pub fn sandy_motion_from_vel(vel: f32, lo: f32, hi: f32) -> f32 {
    crate::frontend::animation::smoothstep((vel.abs() - lo) / (hi - lo).max(1e-3))
}

pub fn sandy_lod_grain(base: f32, lod_max: f32, motion: f32) -> f32 {
    (base * (1.0 + (lod_max - 1.0) * motion.clamp(0.0, 1.0))).clamp(1.0, 32.0)
}

pub fn grain_grid(grain: f32, half_width: f32, half_height: f32) -> (f32, f32) {
    let grain = grain.max(1.0);
    (
        (half_width * 2.0 / grain).round().clamp(32.0, 640.0),
        (half_height * 2.0 / grain).round().clamp(18.0, 360.0),
    )
}

pub fn grain_verts(grain: f32, half_width: f32, half_height: f32) -> u32 {
    let (columns, rows) = grain_grid(grain, half_width, half_height);
    columns as u32 * rows as u32 * 12
}

#[derive(Debug, Clone, Copy)]
pub struct SandyParams {
    pub offset_x: f32,
    pub offset_y: f32,
    pub center_h: f32,
    pub slice_w: f32,
    pub slice_h: f32,
    pub spacing: f32,
    pub duration_ms: f32,
    pub blend_ms: f32,
    pub strands: f32,
    pub twist: f32,
    pub orbit: f32,
    pub turbulence: f32,
    pub waist: f32,
    pub front: f32,
    pub fan: f32,
    pub arc: f32,
    pub swap_loop: bool,
    pub swap_style: f32,
    pub skew: f32,
    pub corners: CardCorners,
    pub edge_speed: f32,
    pub ring_size: f32,
    pub ring_spin: f32,
    pub ring_wave: f32,
    pub ring_soft: f32,
    pub ring_blend: f32,
    pub ring_hold: f32,
    pub grain: f32,
    pub video_out_live: bool,
}

impl SandyParams {
    pub fn morph_toward(&mut self, target: &Self, amount: f32) {
        self.offset_x = lerp(self.offset_x, target.offset_x, amount);
        self.offset_y = lerp(self.offset_y, target.offset_y, amount);
        self.center_h = lerp(self.center_h, target.center_h, amount);
        self.slice_w = lerp(self.slice_w, target.slice_w, amount);
        self.slice_h = lerp(self.slice_h, target.slice_h, amount);
        self.spacing = lerp(self.spacing, target.spacing, amount);
        self.duration_ms = target.duration_ms;
        self.blend_ms = target.blend_ms;
        self.strands = target.strands;
        self.twist = lerp(self.twist, target.twist, amount);
        self.orbit = lerp(self.orbit, target.orbit, amount);
        self.turbulence = lerp(self.turbulence, target.turbulence, amount);
        self.waist = lerp(self.waist, target.waist, amount);
        self.front = lerp(self.front, target.front, amount);
        self.fan = lerp(self.fan, target.fan, amount);
        self.arc = lerp(self.arc, target.arc, amount);
        self.swap_loop = target.swap_loop;
        self.swap_style = target.swap_style;
        self.skew = lerp(self.skew, target.skew, amount);
        self.corners.morph_toward(&target.corners, amount);
        self.edge_speed = target.edge_speed;
        self.ring_size = lerp(self.ring_size, target.ring_size, amount);
        self.ring_spin = lerp(self.ring_spin, target.ring_spin, amount);
        self.ring_wave = lerp(self.ring_wave, target.ring_wave, amount);
        self.ring_soft = lerp(self.ring_soft, target.ring_soft, amount);
        self.ring_blend = target.ring_blend;
        self.ring_hold = target.ring_hold;
        self.grain = lerp(self.grain, target.grain, amount);
        self.video_out_live = target.video_out_live;
    }

    pub fn settled_to(&self, target: &Self) -> bool {
        feq(self.offset_x, target.offset_x)
            && feq(self.offset_y, target.offset_y)
            && feq(self.center_h, target.center_h)
            && feq(self.slice_w, target.slice_w)
            && feq(self.slice_h, target.slice_h)
            && feq(self.spacing, target.spacing)
            && self.duration_ms == target.duration_ms
            && self.blend_ms == target.blend_ms
            && self.strands == target.strands
            && feq(self.twist, target.twist)
            && feq(self.orbit, target.orbit)
            && feq(self.turbulence, target.turbulence)
            && feq(self.waist, target.waist)
            && feq(self.front, target.front)
            && feq(self.fan, target.fan)
            && feq(self.arc, target.arc)
            && self.swap_loop == target.swap_loop
            && self.swap_style == target.swap_style
            && feq(self.skew, target.skew)
            && self.corners.settled_to(&target.corners)
            && self.edge_speed == target.edge_speed
            && feq(self.ring_size, target.ring_size)
            && feq(self.ring_spin, target.ring_spin)
            && feq(self.ring_wave, target.ring_wave)
            && feq(self.ring_soft, target.ring_soft)
            && self.ring_blend == target.ring_blend
            && self.ring_hold == target.ring_hold
            && feq(self.grain, target.grain)
            && self.video_out_live == target.video_out_live
    }

    pub fn center_hw(&self) -> f32 {
        self.center_h * 8.0 / 9.0
    }

    pub fn center_hh(&self) -> f32 {
        self.center_h * 0.5
    }

    pub fn stride(&self) -> f32 {
        slice_midline_width(self.slice_w, self.skew) + self.spacing
    }

    pub fn strip_h(&self) -> f32 {
        self.slice_h + STRIP_MARGIN * 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Place {
    pub i: usize,
    pub x: f32,
    pub s: f32,
}

pub fn strip_bottom(viewport_height: f32) -> f32 {
    viewport_height - BAR_ZONE
}

pub fn edge_pan(
    x: f32,
    y: f32,
    viewport_width: f32,
    viewport_height: f32,
    strip_height: f32,
) -> f32 {
    let bottom = strip_bottom(viewport_height);
    if y < bottom - strip_height || y > bottom {
        return 0.0;
    }
    let zone = (viewport_width * 0.14).max(1.0);
    if x < zone {
        -(1.0 - x / zone).clamp(0.0, 1.0)
    } else if x > viewport_width - zone {
        (1.0 - (viewport_width - x) / zone).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub fn clamp_cam(camera: f32, count: usize) -> f32 {
    camera.clamp(0.0, count.saturating_sub(1) as f32)
}

pub fn place(
    count: usize,
    camera: f32,
    params: &SandyParams,
    center_x: f32,
    available_width: f32,
    looped: bool,
) -> Vec<Place> {
    strip_place(count, camera, params.stride(), center_x, available_width, looped)
}

pub fn strip_scroll(
    camera: f32,
    count: usize,
    stride: f32,
    margin: f32,
    available_width: f32,
) -> f32 {
    let usable = (available_width - 2.0 * margin).max(stride);
    let content_span = (count.max(1) as f32 - 1.0) * stride;
    let max_scroll = (content_span - usable).max(0.0);
    (camera * stride - usable * 0.5).clamp(0.0, max_scroll)
}

pub fn strip_place(
    count: usize,
    camera: f32,
    stride: f32,
    center_x: f32,
    available_width: f32,
    looped: bool,
) -> Vec<Place> {
    let mut places = Vec::new();
    if count == 0 {
        return places;
    }
    let stride = stride.max(8.0);
    let margin = stride;
    let left = center_x - available_width * 0.5;
    let right = center_x + available_width * 0.5;
    let usable = (available_width - 2.0 * margin).max(stride);
    let content_span = (count.saturating_sub(1) as f32) * stride;
    let last = count as i64 - 1;
    let (scroll, first, last) = if looped {
        (camera * stride - usable * 0.5, i64::MIN, i64::MAX)
    } else {
        (strip_scroll(camera, count, stride, margin, available_width), 0, last)
    };
    let origin =
        if content_span < usable { center_x - content_span * 0.5 } else { left + margin - scroll };
    let span = (((available_width + margin) / stride).ceil() as i64) + 1;
    let anchor = ((scroll / stride).round() as i64) + 1;
    for index in (anchor - span).max(first)..=(anchor + span).min(last) {
        let x = origin + index as f32 * stride;
        let distance_to_edge = (x - left).min(right - x);
        let fade = (distance_to_edge / margin).clamp(0.0, 1.0);
        if fade < 0.05 {
            continue;
        }
        places.push(Place { i: index.rem_euclid(count as i64) as usize, x, s: fade });
    }
    places
}

mod tests;
