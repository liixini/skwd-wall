use glam::{Mat4, Quat, Vec3};

use super::layout::{CardCorners, feq, lerp};

pub const DESIGN_W: f32 = 1440.0;
pub const DESIGN_H: f32 = 900.0;
pub const ANCHOR_Y: f32 = 0.48;
pub const PERSPECTIVE_DROP: f32 = 0.018;
pub const PAD: f32 = 2.0;
pub const RIBBON_OVERLAP: f32 = 0.6;
pub const RIBBON_MIN: f32 = 2.0;
pub const CROP_ZOOM: f32 = 1.10;
pub const OUT_MS: f32 = 620.0;
pub const IN_MS: f32 = 900.0;
pub const CARD_STAGGER_MS: f32 = 40.0;
pub const RIBBON_CARD_STAGGER_MS: f32 = 26.0;
pub const RIBBON_STAGGER_MS: f32 = 34.0;
pub const SWAP_GAP_MS: f32 = 80.0;
pub const GHOST_LEVELS: [(f32, f32, f32); 2] = [(0.46, 0.65, 1.22), (0.26, 1.5, 1.5)];
pub const FLIP_LIFT_MS: f32 = 500.0;
pub const FLIP_CLOSE_LIFT_MS: f32 = 520.0;
pub const FLIP_OPEN: [(f32, f32); 2] = [(470.0, 44.0), (640.0, 46.0)];
pub const FLIP_CLOSE: [(f32, f32); 2] = [(490.0, 46.0), (680.0, 48.0)];
pub const TWIST_MS: f32 = 900.0;
pub const BOB_PERIOD_S: f32 = 5.4;
pub const BOB_DELAY_S: f32 = 0.42;
pub const TILT_X_DEG: f32 = 4.5;
pub const TILT_Y_DEG: f32 = 6.0;
pub const RELAX_WINDOW: (f32, f32) = (0.3, 0.68);
pub const FLIP_LAND_AT: f32 = 0.72;
pub const REVEAL_TURN_MS: f32 = 440.0;
pub const REVEAL_STAGGER_MS: f32 = 55.0;
pub const REVEAL_GHOST_LAG: [(f32, f32, f32); 2] = [(0.05, 0.42, 0.7), (0.1, 0.24, 1.3)];
pub const REVEAL_GAP: f32 = 8.0;
pub const THUMB_ASPECT: f32 = 16.0 / 9.0;
pub const ROW_FIT: f32 = 0.92;
pub const COLUMN_FIT: f32 = 0.78;
pub const TALL_ASPECT: f32 = 9.0 / 16.0;

pub const X: [f32; 3] = [1.0, 0.0, 0.0];
pub const Y: [f32; 3] = [0.0, 1.0, 0.0];
pub const Z: [f32; 3] = [0.0, 0.0, 1.0];

pub use crate::contracts::picker::{
    HandAxis as Axis, HandCut as Cut, HandMove as Move, HandVariance as Variance,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DealMode {
    Cycle,
    Random,
}

impl DealMode {
    pub fn from_key(value: &str) -> Self {
        if value == "random" { Self::Random } else { Self::Cycle }
    }
}

pub fn pick_move(mode: DealMode, enabled: [bool; 5], last: Move, seed: f32) -> Move {
    let pool: Vec<Move> = Move::ALL.iter().copied().filter(|mv| enabled[mv.index()]).collect();
    let pool = if pool.is_empty() { Move::ALL.to_vec() } else { pool };
    match mode {
        DealMode::Cycle => {
            let at = pool.iter().position(|mv| *mv == last).map_or(0, |at| (at + 1) % pool.len());
            pool[at]
        }
        DealMode::Random => {
            let choices: Vec<Move> = if pool.len() > 1 {
                pool.iter().copied().filter(|mv| *mv != last).collect()
            } else {
                pool
            };
            let at = (seed.clamp(0.0, 0.999_99) * choices.len() as f32) as usize;
            choices[at.min(choices.len() - 1)]
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HandParams {
    pub offset_x: f32,
    pub offset_y: f32,
    pub count: usize,
    pub card_w: f32,
    pub card_h: f32,
    pub spread: f32,
    pub ribbons: usize,
    pub fan_angle: f32,
    pub fan_roll: f32,
    pub arch: f32,
    pub corners: CardCorners,
    pub skew: f32,
    pub blur: f32,
    pub axis: Axis,
    pub cut: Cut,
    pub variance: Variance,
    pub deal_mode: DealMode,
    pub moves: [bool; 5],
    pub speed: f32,
    pub tilt: f32,
    pub perspective: f32,
    pub ghosts: bool,
    pub bob: bool,
    pub backdrop: bool,
    pub reveal_fill: bool,
}

impl Default for HandParams {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            count: 5,
            card_w: 168.0,
            card_h: 432.0,
            spread: 126.0,
            ribbons: 6,
            fan_angle: 12.0,
            fan_roll: 8.5,
            arch: 20.0,
            corners: CardCorners::default(),
            skew: 0.0,
            blur: 1.0,
            axis: Axis::Rows,
            cut: Cut::Straight,
            variance: Variance::None,
            deal_mode: DealMode::Cycle,
            moves: [true; 5],
            speed: 1.0,
            tilt: 1.0,
            perspective: 1700.0,
            ghosts: true,
            bob: false,
            backdrop: true,
            reveal_fill: true,
        }
    }
}

impl HandParams {
    pub fn morph_toward(&mut self, target: &Self, amount: f32) {
        self.offset_x = lerp(self.offset_x, target.offset_x, amount);
        self.offset_y = lerp(self.offset_y, target.offset_y, amount);
        self.card_w = lerp(self.card_w, target.card_w, amount);
        self.card_h = lerp(self.card_h, target.card_h, amount);
        self.spread = lerp(self.spread, target.spread, amount);
        self.perspective = lerp(self.perspective, target.perspective, amount);
        self.fan_angle = lerp(self.fan_angle, target.fan_angle, amount);
        self.fan_roll = lerp(self.fan_roll, target.fan_roll, amount);
        self.arch = lerp(self.arch, target.arch, amount);
        self.corners.morph_toward(&target.corners, amount);
        self.skew = lerp(self.skew, target.skew, amount);
        self.blur = lerp(self.blur, target.blur, amount);
        self.count = target.count;
        self.ribbons = target.ribbons;
        self.axis = target.axis;
        self.cut = target.cut;
        self.variance = target.variance;
        self.deal_mode = target.deal_mode;
        self.moves = target.moves;
        self.speed = target.speed;
        self.tilt = target.tilt;
        self.ghosts = target.ghosts;
        self.bob = target.bob;
        self.backdrop = target.backdrop;
        self.reveal_fill = target.reveal_fill;
    }

    pub fn settled_to(&self, target: &Self) -> bool {
        feq(self.offset_x, target.offset_x)
            && feq(self.offset_y, target.offset_y)
            && feq(self.card_w, target.card_w)
            && feq(self.card_h, target.card_h)
            && feq(self.spread, target.spread)
            && feq(self.perspective, target.perspective)
            && feq(self.fan_angle, target.fan_angle)
            && feq(self.fan_roll, target.fan_roll)
            && feq(self.arch, target.arch)
            && self.corners.settled_to(&target.corners)
            && feq(self.skew, target.skew)
            && feq(self.blur, target.blur)
            && self.count == target.count
            && self.ribbons == target.ribbons
            && self.axis == target.axis
            && self.cut == target.cut
            && self.variance == target.variance
            && self.deal_mode == target.deal_mode
            && self.moves == target.moves
            && self.speed == target.speed
            && self.tilt == target.tilt
            && self.ghosts == target.ghosts
            && self.bob == target.bob
            && self.backdrop == target.backdrop
            && self.reveal_fill == target.reveal_fill
    }

    pub fn speed_scale(&self) -> f32 {
        1.0 / self.speed.max(0.35)
    }

    pub fn fan(&self, k: f32) -> Fan {
        Fan {
            spread: self.spread * k,
            angle: self.fan_angle,
            roll: self.fan_roll,
            arch: self.arch * k,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fan {
    pub spread: f32,
    pub angle: f32,
    pub roll: f32,
    pub arch: f32,
}

#[cfg(test)]
pub const DESIGN_FAN: Fan = Fan { spread: 126.0, angle: 12.0, roll: 8.5, arch: 20.0 };

pub fn stage_scale(viewport_width: f32, viewport_height: f32) -> f32 {
    (viewport_height / DESIGN_H).min(viewport_width / DESIGN_W).max(0.2)
}

pub fn rnd(a: f32, b: f32, c: f32, d: f32) -> f32 {
    let x = (a * 127.1 + b * 311.7 + c * 74.7 + d * 269.5 + 17.3).sin() * 43_758.547;
    x - x.floor()
}

pub use crate::frontend::animation::Bezier;

pub const EASE_DEAL_OUT: Bezier = Bezier(0.62, 0.0, 0.86, 0.24);
pub const EASE_DEAL_IN: Bezier = Bezier(0.16, 1.06, 0.28, 1.0);
pub const EASE_RIBBON_OUT: Bezier = Bezier(0.42, 0.0, 0.7, 0.2);
pub const EASE_RIBBON_IN: Bezier = Bezier(0.2, 0.92, 0.26, 1.0);
pub const EASE_FLIP_LIFT: Bezier = Bezier(0.38, 0.02, 0.28, 1.0);
pub const EASE_FLIP_LAND: Bezier = Bezier(0.2, 1.12, 0.3, 1.0);
pub const EASE_FLIP_CLOSE: Bezier = Bezier(0.2, 1.14, 0.3, 1.0);
pub const EASE_TWIST: Bezier = Bezier(0.34, 1.36, 0.32, 1.0);
pub const EASE_REVEAL_TRAVEL: Bezier = Bezier(0.3, 0.0, 0.25, 1.08);
pub const EASE_REVEAL_TURN: Bezier = Bezier(0.36, 0.0, 0.2, 1.24);
#[cfg(test)]
pub const EASE_LINEAR: Bezier = Bezier(0.25, 0.25, 0.75, 0.75);

pub fn rot(axis: [f32; 3], deg: f32) -> Mat4 {
    let axis = Vec3::from_array(axis).normalize_or_zero();
    if axis == Vec3::ZERO || deg.abs() < 1e-6 {
        return Mat4::IDENTITY;
    }
    Mat4::from_axis_angle(axis, deg.to_radians())
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rot {
    pub axis: [f32; 3],
    pub deg: f32,
}

impl Rot {
    pub const fn new(axis: [f32; 3], deg: f32) -> Self {
        Self { axis, deg }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub t: [f32; 3],
    pub rots: [Rot; 3],
    pub n: usize,
    pub s: f32,
}

impl Pose {
    pub const REST: Pose = Pose {
        t: [0.0; 3],
        rots: [Rot::new(Z, 0.0), Rot::new(Z, 0.0), Rot::new(Z, 0.0)],
        n: 0,
        s: 1.0,
    };

    pub fn new(t: [f32; 3], rots: &[Rot], s: f32) -> Self {
        let mut pose = Pose { t, s, ..Pose::REST };
        for (slot, rot) in pose.rots.iter_mut().zip(rots) {
            *slot = *rot;
        }
        pose.n = rots.len().min(3);
        pose
    }

    pub fn matrix(&self) -> Mat4 {
        let mut m = Mat4::from_translation(Vec3::from_array(self.t));
        for r in &self.rots[..self.n] {
            m *= rot(r.axis, r.deg);
        }
        if (self.s - 1.0).abs() > 1e-6 {
            m *= Mat4::from_scale(Vec3::splat(self.s));
        }
        m
    }

    fn orders_match(a: &Pose, b: &Pose) -> bool {
        a.rots[..a.n.min(b.n)]
            .iter()
            .zip(&b.rots[..a.n.min(b.n)])
            .all(|(ra, rb)| axis_eq(ra.axis, rb.axis))
    }

    fn quat(&self) -> Quat {
        let mut q = Quat::IDENTITY;
        for r in &self.rots[..self.n] {
            let axis = Vec3::from_array(r.axis).normalize_or_zero();
            if axis != Vec3::ZERO {
                q *= Quat::from_axis_angle(axis, r.deg.to_radians());
            }
        }
        q
    }

    pub fn mix(a: &Pose, b: &Pose, u: f32) -> Pose {
        let t = [lerp(a.t[0], b.t[0], u), lerp(a.t[1], b.t[1], u), lerp(a.t[2], b.t[2], u)];
        let s = lerp(a.s, b.s, u);
        if Pose::orders_match(a, b) {
            let n = a.n.max(b.n);
            let mut rots = Pose::REST.rots;
            for (i, slot) in rots.iter_mut().enumerate().take(n) {
                let axis = if i < b.n { b.rots[i].axis } else { a.rots[i].axis };
                let da = if i < a.n { a.rots[i].deg } else { 0.0 };
                let db = if i < b.n { b.rots[i].deg } else { 0.0 };
                *slot = Rot::new(axis, lerp(da, db, u));
            }
            return Pose { t, rots, n, s };
        }
        let (axis, angle) = a.quat().slerp(b.quat(), u).to_axis_angle();
        if angle.abs() < 1e-5 {
            return Pose::new(t, &[Rot::new(Z, 0.0)], s);
        }
        Pose::new(t, &[Rot::new(axis.to_array(), angle.to_degrees())], s)
    }
}

fn axis_eq(a: [f32; 3], b: [f32; 3]) -> bool {
    (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4 && (a[2] - b[2]).abs() < 1e-4
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoseTween {
    pub from: Pose,
    pub to: Pose,
    pub delay: f32,
    pub dur: f32,
    pub ease: Bezier,
}

impl PoseTween {
    pub fn new(from: Pose, to: Pose, delay_ms: f32, dur_ms: f32, ease: Bezier) -> Self {
        Self { from, to, delay: delay_ms / 1000.0, dur: (dur_ms / 1000.0).max(0.001), ease }
    }

    pub fn at(&self, t: f32) -> Pose {
        let u = self.ease.at(((t - self.delay) / self.dur).clamp(0.0, 1.0));
        Pose::mix(&self.from, &self.to, u)
    }

    pub fn end(&self) -> f32 {
        self.delay + self.dur
    }

    pub fn retarget(&mut self, now: f32, to: Pose, delay_ms: f32, dur_ms: f32, ease: Bezier) {
        *self = PoseTween::new(self.at(now), to, delay_ms, dur_ms, ease);
    }
}

pub fn fan_pose(n: f32, fan: &Fan, k: f32, push: f32, lift: f32, zback: f32, s: f32) -> Pose {
    Pose::new(
        [n * fan.spread * push, n.abs().powf(1.7) * fan.arch + lift, -n.abs() * 55.0 * k + zback],
        &[Rot::new(Y, n * -fan.angle), Rot::new(Z, n * fan.roll)],
        s,
    )
}

pub fn reveal_frame(viewport_width: f32, viewport_height: f32, tall: bool) -> (f32, f32) {
    let aspect = if tall { TALL_ASPECT } else { THUMB_ASPECT };
    let w = (viewport_width * ROW_FIT).min(viewport_height * COLUMN_FIT * aspect);
    (w, w / aspect)
}

pub fn slat_half_extent(len: usize, frame: (f32, f32), gap: f32, tall: bool) -> (f32, f32) {
    let len = len.max(1) as f32;
    if tall {
        let slat_h = (frame.1 - (len - 1.0) * gap) / len;
        (slat_h * 0.5, frame.0 * 0.5)
    } else {
        let slat_w = (frame.0 - (len - 1.0) * gap) / len;
        (slat_w * 0.5, frame.1 * 0.5)
    }
}

pub fn row_pose(n: f32, slat_w: f32, gap: f32) -> Pose {
    Pose::new([n * (slat_w + gap), 0.0, 0.0], &[Rot::new(Y, 0.0), Rot::new(Z, 0.0)], 1.0)
}

pub fn column_pose(n: f32, slat_h: f32, gap: f32) -> Pose {
    Pose::new([0.0, n * (slat_h + gap), 0.0], &[Rot::new(Y, 0.0), Rot::new(Z, -90.0)], 1.0)
}

fn cover(tile_aspect: f32, box_aspect: f32) -> (f32, f32, f32, f32) {
    if box_aspect >= tile_aspect {
        let h = tile_aspect / box_aspect;
        (0.0, (1.0 - h) * 0.5, 1.0, h)
    } else {
        let w = box_aspect / tile_aspect;
        ((1.0 - w) * 0.5, 0.0, w, 1.0)
    }
}

pub fn slice_crop(slot: usize, len: usize, row_w: f32, row_h: f32, gap: f32) -> [f32; 4] {
    let len = len.max(1);
    let row_w = row_w.max(1.0);
    let (x0, y0, w, h) = cover(THUMB_ASPECT, row_w / row_h.max(1.0));
    let band = (row_w - (len as f32 - 1.0) * gap) / len as f32;
    let start = slot.min(len - 1) as f32 * (band + gap) / row_w;
    [x0 + w * start, y0, w * band / row_w, h]
}

pub fn column_crop(slot: usize, len: usize, stack_w: f32, stack_h: f32, gap: f32) -> [f32; 4] {
    let len = len.max(1);
    let stack_h = stack_h.max(1.0);
    let (u0, v0, uw, vh) = cover(TALL_ASPECT, stack_w.max(1.0) / stack_h);
    let band = (stack_h - (len as f32 - 1.0) * gap) / len as f32;
    let v = v0 + vh * slot.min(len - 1) as f32 * (band + gap) / stack_h;
    let bv = vh * band / stack_h;
    [1.0 - v - bv, u0, bv, uw]
}

pub fn keep_extent(
    len: usize,
    card: (f32, f32),
    frame: (f32, f32),
    gap: f32,
    tall: bool,
) -> (f32, f32) {
    let len = len.max(1) as f32;
    let (hw, hh) = card;
    let along = (len * hw * 2.0).max(1.0);
    let (limit_along, limit_cross) = if tall { (frame.1, frame.0) } else { (frame.0, frame.1) };
    let fit = ((limit_along - (len - 1.0) * gap) / along).min(limit_cross / (hh * 2.0).max(1.0));
    let fit = fit.min(1.0);
    (hw * fit, hh * fit)
}

pub fn reveal_turn(clock_s: f32, order: usize, speed_scale: f32) -> f32 {
    let delay = order as f32 * REVEAL_STAGGER_MS * speed_scale / 1000.0;
    let dur = REVEAL_TURN_MS * speed_scale / 1000.0;
    ((clock_s - delay) / dur).clamp(0.0, 1.0)
}

pub fn reveal_order(slot: usize, len: usize, turns_done: u32) -> usize {
    if turns_done.is_multiple_of(2) { slot } else { len.saturating_sub(1).saturating_sub(slot) }
}

pub fn reveal_dir(slot: usize, turns_done: u32) -> f32 {
    if (slot as u32 + turns_done).is_multiple_of(2) { 1.0 } else { -1.0 }
}

pub fn slat_turn_pose(t: f32, dir: f32, k: f32) -> (Pose, f32) {
    let t = t.clamp(0.0, 1.0);
    let e = EASE_REVEAL_TURN.at(t);
    let s = (t * std::f32::consts::PI).sin();
    let pose = Pose::new(
        [0.0, -46.0 * k * s, 150.0 * k * s],
        &[Rot::new(Y, dir * 180.0 * e), Rot::new(X, dir * -14.0 * s), Rot::new(Z, dir * 9.0 * s)],
        1.0 + 0.07 * s,
    );
    (pose, e)
}

pub fn reveal_turn_end(len: usize, speed_scale: f32) -> f32 {
    (len.saturating_sub(1) as f32 * REVEAL_STAGGER_MS + REVEAL_TURN_MS) * speed_scale / 1000.0
}

pub fn reveal_face(turns_done: u32, t: f32) -> usize {
    let angle = 180.0 * (turns_done as f32 + t.clamp(0.0, 1.0));
    (((angle + 90.0) / 180.0).floor() as u32 % 2) as usize
}

pub fn sel_pose(k: f32) -> Pose {
    Pose::new([0.0, -40.0 * k, 170.0 * k], &[Rot::new(Y, 0.0), Rot::new(Z, 0.0)], 1.02)
}

pub fn move_pose(mv: Move, n: f32, s: f32, k: f32) -> Pose {
    match mv {
        Move::Corkscrew => Pose::new(
            [n * 95.0 * k, s * -780.0 * k, s * 560.0 * k],
            &[Rot::new(Y, s * (940.0 + n * 90.0)), Rot::new(Z, s * (210.0 + n * 55.0))],
            0.08,
        ),
        Move::Shuffle => Pose::new(
            [(s * 1010.0 + n * 130.0) * k, s * n * 70.0 * k, n * 110.0 * k],
            &[Rot::new(Z, s * (72.0 + n * 26.0)), Rot::new(Y, s * 200.0)],
            0.7,
        ),
        Move::Cascade => Pose::new(
            [n * 150.0 * k, s * 720.0 * k, s * 260.0 * k],
            &[Rot::new(X, s * -64.0), Rot::new(Z, n * 8.5)],
            0.44,
        ),
        Move::Ribbon => {
            Pose::new([n * 138.0 * k, s * -34.0 * k, s * 40.0 * k], &[Rot::new(Z, n * 8.5)], 1.04)
        }
        Move::Spiral => {
            let a2 = 1.15 * n + 0.5;
            Pose::new(
                [a2.cos() * 520.0 * s * k, a2.sin() * 430.0 * s * k, -560.0 * k],
                &[Rot::new([0.4, 1.0, 0.3], s * (520.0 + n * 120.0))],
                0.24,
            )
        }
    }
}

fn along_cross(axis: Axis, along: f32, cross: f32, z: f32) -> [f32; 3] {
    match axis {
        Axis::Rows => [cross, along, z],
        Axis::Columns => [along, cross, z],
    }
}

fn hinge(axis: Axis, deg: f32) -> Rot {
    match axis {
        Axis::Rows => Rot::new(X, deg),
        Axis::Columns => Rot::new(Y, deg),
    }
}

fn swing(axis: Axis, deg: f32) -> Rot {
    match axis {
        Axis::Rows => Rot::new(Y, deg),
        Axis::Columns => Rot::new(X, deg),
    }
}

pub fn ribbon_deal_pose(r: usize, s: f32, k: f32, axis: Axis) -> Pose {
    let d = if r % 2 == 1 { s } else { -s };
    let rf = r as f32;
    Pose::new(
        along_cross(axis, 0.0, d * (420.0 + rf * 64.0) * k, (90.0 + rf * 26.0) * k),
        &[hinge(axis, d * (72.0 + rf * 8.0)), Rot::new(Z, d * 9.0)],
        0.92,
    )
}

pub fn ribbon_flip_pose(stage: u8, r: usize, nr: usize, dir: f32, k: f32, axis: Axis) -> Pose {
    let mid = (nr as f32 - 1.0) / 2.0;
    let rf = r as f32;
    let tail = (nr - 1 - r) as f32;
    match stage {
        1 => Pose::new(
            along_cross(
                axis,
                dir * (84.0 + rf * 7.0) * k,
                (rf - mid) * 34.0 * k,
                (120.0 + rf * 22.0) * k,
            ),
            &[hinge(axis, dir * 104.0), swing(axis, dir * 44.0), Rot::new(Z, dir * 17.0)],
            1.0,
        ),
        2 => Pose::new([0.0; 3], &[hinge(axis, dir * 180.0)], 1.0),
        3 => Pose::new(
            along_cross(
                axis,
                -dir * (92.0 + tail * 7.0) * k,
                (mid - rf) * 34.0 * k,
                (150.0 + tail * 20.0) * k,
            ),
            &[hinge(axis, dir * 258.0), swing(axis, -dir * 54.0), Rot::new(Z, -dir * 21.0)],
            1.0,
        ),
        4 => Pose::new([0.0; 3], &[hinge(axis, dir * 360.0)], 1.0),
        _ => Pose::REST,
    }
}

pub fn twist_pose(stage: u8, k: f32) -> Pose {
    let (ry, tz, s) = match stage {
        1 => (-26.0, 70.0, 1.05),
        2 => (-13.0, 120.0, 1.06),
        3 => (27.0, 84.0, 1.05),
        _ => (0.0, 0.0, 1.0),
    };
    Pose::new([0.0, 0.0, tz * k], &[Rot::new(Y, ry)], s)
}

pub fn ribbon_dir(seed: f32, card: usize, r: usize, deal: bool) -> f32 {
    if rnd(seed * 613.0, card as f32, r as f32, if deal { 1.0 } else { 7.0 }) > 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub fn flip_open_ms(nr: usize) -> f32 {
    FLIP_LIFT_MS + FLIP_OPEN[1].0 + (nr.saturating_sub(1)) as f32 * FLIP_OPEN[1].1
}

pub fn flip_close_ms(nr: usize) -> f32 {
    FLIP_CLOSE_LIFT_MS + FLIP_CLOSE[1].0 + (nr.saturating_sub(1)) as f32 * FLIP_CLOSE[1].1
}

pub fn flip_ribbon_pose(
    tau_ms: f32,
    closing: bool,
    r: usize,
    nr: usize,
    dir: f32,
    k: f32,
    axis: Axis,
) -> Pose {
    let tail = (nr - 1 - r) as f32;
    let rf = r as f32;
    if closing {
        let (s3, s4) =
            (ribbon_flip_pose(3, r, nr, dir, k, axis), ribbon_flip_pose(4, r, nr, dir, k, axis));
        let landed = ribbon_flip_pose(2, r, nr, dir, k, axis);
        let lift =
            PoseTween::new(landed, s3, tail * FLIP_CLOSE[0].1, FLIP_CLOSE[0].0, EASE_FLIP_LIFT);
        if tau_ms < FLIP_CLOSE_LIFT_MS {
            return lift.at(tau_ms / 1000.0);
        }
        let land = PoseTween::new(
            lift.at(FLIP_CLOSE_LIFT_MS / 1000.0),
            s4,
            rf * FLIP_CLOSE[1].1,
            FLIP_CLOSE[1].0,
            EASE_FLIP_CLOSE,
        );
        return land.at((tau_ms - FLIP_CLOSE_LIFT_MS) / 1000.0);
    }
    let s1 = ribbon_flip_pose(1, r, nr, dir, k, axis);
    let s2 = ribbon_flip_pose(2, r, nr, dir, k, axis);
    let lift = PoseTween::new(Pose::REST, s1, rf * FLIP_OPEN[0].1, FLIP_OPEN[0].0, EASE_FLIP_LIFT);
    if tau_ms < FLIP_LIFT_MS {
        return lift.at(tau_ms / 1000.0);
    }
    let land = PoseTween::new(
        lift.at(FLIP_LIFT_MS / 1000.0),
        s2,
        tail * FLIP_OPEN[1].1,
        FLIP_OPEN[1].0,
        EASE_FLIP_LAND,
    );
    land.at((tau_ms - FLIP_LIFT_MS) / 1000.0)
}

pub fn flip_twist_pose(tau_ms: f32, closing: bool, k: f32) -> Pose {
    let (lift_ms, first, second) = if closing {
        (FLIP_CLOSE_LIFT_MS, twist_pose(3, k), twist_pose(4, k))
    } else {
        (FLIP_LIFT_MS, twist_pose(1, k), twist_pose(2, k))
    };
    let start = if closing { twist_pose(2, k) } else { Pose::REST };
    let lift = PoseTween::new(start, first, 0.0, TWIST_MS, EASE_TWIST);
    if tau_ms < lift_ms {
        return lift.at(tau_ms / 1000.0);
    }
    let land = PoseTween::new(lift.at(lift_ms / 1000.0), second, 0.0, TWIST_MS, EASE_TWIST);
    land.at((tau_ms - lift_ms) / 1000.0)
}

pub fn bob(time_s: f32, slot: usize, k: f32) -> Pose {
    let phase = ((time_s - slot as f32 * BOB_DELAY_S) / BOB_PERIOD_S).rem_euclid(1.0);
    let s = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    Pose::new([0.0, lerp(-6.0, 7.0, s) * k, 0.0], &[Rot::new(Z, lerp(-0.7, 0.9, s))], 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RibbonCut {
    pub a0: f32,
    pub a1: f32,
    pub b0: f32,
    pub b1: f32,
    pub mid: f32,
}

pub fn ribbon_cuts(
    nr: usize,
    total: f32,
    cut: Cut,
    variance: Variance,
    lseed: f32,
    card: usize,
    pad: f32,
) -> Vec<RibbonCut> {
    let nr = nr.clamp(1, 14);
    let band = total / nr as f32;
    let skew0 = cut.factor() * band;
    let vary = variance.factor();
    let mut bounds = Vec::with_capacity(nr + 1);
    bounds.push(0.0);
    if vary > 0.0 {
        let weights: Vec<f32> = (0..nr)
            .map(|kk| 1.0 + vary * (rnd(lseed * 997.0, card as f32, kk as f32, 11.0) - 0.5) * 1.5)
            .collect();
        let sum: f32 = weights.iter().sum();
        let mut acc = 0.0;
        for (kk, weight) in weights.iter().enumerate() {
            acc += weight / sum * total;
            bounds.push(if kk == nr - 1 { total } else { acc });
        }
    } else {
        for kk in 1..=nr {
            bounds.push(kk as f32 * band);
        }
    }
    let mut skews: Vec<f32> = (0..=nr)
        .map(|kk| {
            let jitter = if vary > 0.0 {
                1.0 + vary * (rnd(lseed * 331.0, card as f32, kk as f32, 13.0) - 0.5) * 1.6
            } else {
                1.0
            };
            skew0 * jitter
        })
        .collect();
    for kk in 1..nr.saturating_sub(1) {
        let room = 2.0 * (bounds[kk + 1] - bounds[kk] - RIBBON_MIN).max(0.0);
        skews[kk + 1] = skews[kk + 1].clamp(skews[kk] - room, skews[kk] + room);
    }
    let half = total * 0.5;
    let far = half + pad + total;
    (0..nr)
        .map(|kk| {
            let (a, b) = (bounds[kk] - half, bounds[kk + 1] - half);
            let (ska, skb) = (skews[kk], skews[kk + 1]);
            let first = kk == 0;
            let last = kk == nr - 1;
            RibbonCut {
                a0: if first { -far } else { a - ska * 0.5 - RIBBON_OVERLAP },
                a1: if first { -far } else { a + ska * 0.5 - RIBBON_OVERLAP },
                b0: if last { far } else { b - skb * 0.5 + RIBBON_OVERLAP },
                b1: if last { far } else { b + skb * 0.5 + RIBBON_OVERLAP },
                mid: (a + b) * 0.5,
            }
        })
        .collect()
}

pub fn mirrored_cut(cut: &RibbonCut) -> RibbonCut {
    let m = 2.0 * cut.mid;
    RibbonCut { a0: m - cut.b0, a1: m - cut.b1, b0: m - cut.a0, b1: m - cut.a1, mid: cut.mid }
}

pub fn ribbon_corners(cut: &RibbonCut, axis: Axis, hw: f32, hh: f32, pad: f32) -> [[f32; 3]; 4] {
    match axis {
        Axis::Rows => [
            [-hw - pad, cut.a0, 0.0],
            [hw + pad, cut.a1, 0.0],
            [hw + pad, cut.b1, 0.0],
            [-hw - pad, cut.b0, 0.0],
        ],
        Axis::Columns => [
            [cut.a0, -hh - pad, 0.0],
            [cut.b0, -hh - pad, 0.0],
            [cut.b1, hh + pad, 0.0],
            [cut.a1, hh + pad, 0.0],
        ],
    }
}

pub fn ribbon_locals(cut: &RibbonCut, axis: Axis) -> [f32; 4] {
    match axis {
        Axis::Rows => [cut.a0, cut.a1, cut.b1, cut.b0],
        Axis::Columns => [cut.a0, cut.b0, cut.b1, cut.a1],
    }
}

pub fn card_corners(hw: f32, hh: f32, pad_x: f32, pad_y: f32) -> [[f32; 3]; 4] {
    [
        [-hw - pad_x, -hh - pad_y, 0.0],
        [hw + pad_x, -hh - pad_y, 0.0],
        [hw + pad_x, hh + pad_y, 0.0],
        [-hw - pad_x, hh + pad_y, 0.0],
    ]
}

pub fn card_locals(hh: f32, pad: f32) -> [f32; 4] {
    [-hh - pad, -hh - pad, hh + pad, hh + pad]
}

pub fn ribbon_origin(cut: &RibbonCut, axis: Axis) -> Mat4 {
    match axis {
        Axis::Rows => Mat4::from_translation(Vec3::new(0.0, cut.mid, 0.0)),
        Axis::Columns => Mat4::from_translation(Vec3::new(cut.mid, 0.0, 0.0)),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub d: f32,
    pub origin: [f32; 2],
    pub shift: [f32; 2],
}

impl Camera {
    pub fn project(&self, p: [f32; 3]) -> [f32; 3] {
        let w = ((self.d - p[2]) / self.d.max(1.0)).max(0.05);
        [
            self.origin[0] + self.shift[0] + (p[0] - self.shift[0]) / w,
            self.origin[1] + self.shift[1] + (p[1] - self.shift[1]) / w,
            w,
        ]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub pts: [[f32; 3]; 4],
    pub depth: f32,
}

pub fn project_quad(cam: &Camera, m: &Mat4, corners: &[[f32; 3]; 4]) -> Quad {
    let mut pts = [[0.0; 3]; 4];
    let mut depth = 0.0;
    for (slot, corner) in pts.iter_mut().zip(corners) {
        let world = m.transform_point3(Vec3::from_array(*corner)).to_array();
        depth += world[2] * 0.25;
        *slot = cam.project(world);
    }
    Quad { pts, depth }
}

impl Quad {
    pub fn mirrored(&self) -> Quad {
        let [tl, tr, br, bl] = self.pts;
        Quad { pts: [tr, tl, bl, br], depth: self.depth }
    }

    pub fn facing(&self) -> bool {
        let [tl, tr, _, bl] = self.pts;
        let ax = tr[0] - tl[0];
        let ay = tr[1] - tl[1];
        let bx = bl[0] - tl[0];
        let by = bl[1] - tl[1];
        ax * by - ay * bx > 0.0
    }

    pub fn bounds(&self) -> [f32; 4] {
        let mut min = [f32::MAX, f32::MAX];
        let mut max = [f32::MIN, f32::MIN];
        for p in &self.pts {
            min[0] = min[0].min(p[0]);
            min[1] = min[1].min(p[1]);
            max[0] = max[0].max(p[0]);
            max[1] = max[1].max(p[1]);
        }
        [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (max[0] - min[0]) * 0.5,
            (max[1] - min[1]) * 0.5,
        ]
    }

    pub fn visible(&self, viewport_width: f32, viewport_height: f32) -> bool {
        let [cx, cy, hw, hh] = self.bounds();
        cx + hw >= -8.0
            && cx - hw <= viewport_width + 8.0
            && cy + hh >= -8.0
            && cy - hh <= viewport_height + 8.0
    }
}

pub fn union_bounds(quads: &[Quad]) -> [f32; 4] {
    let mut min = [f32::MAX, f32::MAX];
    let mut max = [f32::MIN, f32::MIN];
    for quad in quads {
        for p in &quad.pts {
            min[0] = min[0].min(p[0]);
            min[1] = min[1].min(p[1]);
            max[0] = max[0].max(p[0]);
            max[1] = max[1].max(p[1]);
        }
    }
    if quads.is_empty() {
        return [0.0; 4];
    }
    [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (max[0] - min[0]) * 0.5,
        (max[1] - min[1]) * 0.5,
    ]
}

pub fn parallax(rig_x: f32, rig_y: f32, k: f32) -> (f32, f32) {
    ((rig_y / 4.0).clamp(-1.0, 1.0) * 40.0 * k, (-rig_x / 3.0).clamp(-1.0, 1.0) * 30.0 * k)
}

pub fn tilt_target(
    x: f32,
    y: f32,
    viewport_width: f32,
    viewport_height: f32,
    tilt: f32,
) -> (f32, f32) {
    (
        -((y / viewport_height.max(1.0)) - 0.5) * TILT_X_DEG * tilt,
        ((x / viewport_width.max(1.0)) - 0.5) * TILT_Y_DEG * tilt,
    )
}

pub fn fan_slot(slot: usize, len: usize, excluded: Option<usize>) -> f32 {
    match excluded {
        Some(ex) if ex != slot && len > 1 => {
            let j = if slot > ex { slot - 1 } else { slot };
            j as f32 - (len as f32 - 2.0) * 0.5
        }
        _ => slot as f32 - (len.max(1) as f32 - 1.0) * 0.5,
    }
}

pub fn hand_window(current: usize, count: usize, size: usize) -> usize {
    let size = size.max(1);
    if count <= size {
        return 0;
    }
    ((current / size) * size).min(count - size)
}

pub fn hand_len(offset: usize, count: usize, size: usize) -> usize {
    count.saturating_sub(offset).min(size.max(1))
}

pub fn relax(progress: f32) -> f32 {
    let (lo, hi) = RELAX_WINDOW;
    let t = ((progress - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

mod tests;
