use crate::frontend::scene::InstanceRaw;

pub(super) const EDITORIAL_CARDS: usize = 5;
pub(super) const EDITORIAL_HERO_COLUMNS: usize = 2;
pub(super) const EDITORIAL_STAGGER_GAIN: f32 = 0.16;
pub(super) const EDITORIAL_HERO_MIN: f32 = 0.35;
pub(super) const EDITORIAL_HERO_MAX: f32 = 0.72;

pub(super) const THUMB_W: f32 = 640.0;
pub(super) const THUMB_H: f32 = 360.0;
pub(super) const TOP_BAR: f32 = 50.0;
pub(super) const STORM_CUTOFF: f32 = 15.0;
pub(super) const NEAR_MIN_DISPLAY_W: f32 = crate::rendering::scene::atlas::FAR_W as f32;
pub(super) const SCROLL_DECAY: f32 = 8.0;

pub(super) fn near_qualifies(
    is_current: bool,
    storm: bool,
    display_width: f32,
    display_height: f32,
) -> bool {
    let source_width_demand = display_width.max(display_height * THUMB_W / THUMB_H);
    is_current || (!storm && source_width_demand >= NEAR_MIN_DISPLAY_W)
}

pub(super) fn prefetch_reach(span: usize, near_capacity: usize) -> (usize, usize) {
    (span.max(1), near_capacity.saturating_sub(span) / 4)
}

pub(super) fn clamp_inset(viewport_width: f32, raw_inset: f32) -> f32 {
    raw_inset.max(0.0).min(viewport_width * 0.6)
}

pub(super) fn ramp_toward(current: f32, target: f32, step: f32) -> f32 {
    if target > current { (current + step).min(target) } else { (current - step).max(target) }
}

pub(super) fn widen(seen: Option<(i64, i64)>, slot: i64) -> (i64, i64) {
    seen.map_or((slot, slot), |(lo, hi)| (lo.min(slot), hi.max(slot)))
}

pub(super) fn vis_range(seen: Option<(i64, i64)>, current: usize, count: usize) -> (usize, usize) {
    let Some((lo, hi)) = seen else {
        return (current, current);
    };
    let len = count as i64;
    if hi - lo + 1 >= len {
        return (0, count.saturating_sub(1));
    }
    (lo.rem_euclid(len) as usize, hi.rem_euclid(len) as usize)
}

pub(super) fn center_layout(viewport_width: f32, raw_inset: f32, rtl: bool) -> (f32, f32) {
    let inset = clamp_inset(viewport_width, raw_inset);
    let lead = if rtl { 0.0 } else { inset };
    (lead + (viewport_width - inset) * 0.5, (viewport_width - inset).max(1.0))
}

pub(super) struct CardSpec {
    pub(super) filtered_idx: usize,
    pub(super) store_idx: usize,
    pub(super) cx: f32,
    pub(super) cy: f32,
    pub(super) hw: f32,
    pub(super) hh: f32,
    pub(super) skew: f32,
    pub(super) edge_tilt: f32,
    pub(super) corners: crate::frontend::scene::layout::CardCorners,
    pub(super) hex: bool,
    pub(super) view: u8,
    pub(super) chrome_radius: f32,
    pub(super) opacity: f32,
    pub(super) chrome_opacity: f32,
    pub(super) body_inset: f32,
    pub(super) near_ok: bool,
}

pub(super) fn chrome_kind(kind: &str) -> u8 {
    match kind {
        wall_proto::kind::WE => 2,
        wall_proto::kind::VIDEO => 1,
        _ => 0,
    }
}

pub(super) fn slice_scroll_steps(accumulator: &mut f32, amount: f32) -> i64 {
    if amount.abs() >= 1.0 {
        *accumulator = 0.0;
        amount.round() as i64
    } else {
        *accumulator += amount;
        let steps = accumulator.trunc() as i64;
        *accumulator -= steps as f32;
        steps
    }
}

pub(super) fn color4(color: iced::Color, alpha: f32) -> [f32; 4] {
    [color.r, color.g, color.b, alpha]
}

pub(super) fn sandy_card_border(
    body: &mut InstanceRaw,
    primary: iced::Color,
    is_hover: bool,
    is_current: bool,
    pop: f32,
    unfold: f32,
) {
    if is_hover || is_current {
        body.border = color4(primary, if is_hover { 1.0 } else { (0.7 + 0.3 * pop) * unfold });
        body.params[1] = if is_hover { 2.0 } else { 1.6 + 0.9 * pop };
    } else {
        body.border = [0.0, 0.0, 0.0, 0.5 * unfold];
        body.params[1] = 1.1;
    }
}

pub(super) fn apply_position_parallax(body: &mut InstanceRaw, position: f32) {
    let [_, _, w, h] = body.crop;
    let nw = w / 1.4;
    let nh = h / 1.4;
    body.crop = [(1.0 - nw) * (0.5 + position.clamp(-1.0, 1.0) * 0.5), (1.0 - nh) * 0.5, nw, nh];
}
