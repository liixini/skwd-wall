use crate::frontend::animation::{self as anim, MotionTier};
use crate::frontend::scene::layout::{self, CardCorners, HexCurve, HexShape, Hit};
use crate::frontend::scene::{InstanceRaw, roll_in_cut};

use super::layout_helpers::{
    CardSpec, THUMB_H, THUMB_W, TOP_BAR, apply_position_parallax, color4, vis_range,
};
use super::model::{RebuildCtx, RebuildSinks, SceneCore};

impl SceneCore {
    pub(super) fn rebuild_hex(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let (vw, vh) = self.viewport;
        let hp = self.hp;
        let count = ctx.filtered.len();
        if count == 0 {
            return;
        }
        let rows = hp.rows.max(1);
        let total_cols = count.div_ceil(rows);
        self.current = self.current.min(count - 1);
        let sel_col = self.current / rows;

        let card_h = hp.content_h() + TOP_BAR + 90.0;
        let cy = vh * 0.5;
        let view_top = cy - card_h * 0.5 + TOP_BAR + 15.0;
        let view_h = card_h - TOP_BAR - 15.0 - 20.0;
        let y_offset = ((view_h - hp.content_h()) / 2.0).max(0.0);

        let sel_center = hp.column_x(sel_col);
        let cx = self.center_x();
        let inset = self.center_inset_px();
        let fade_zone = ((self.avail_w() - hp.visible_band()) / 2.0).max(0.0);
        let min_cam = hp.r;
        let max_cam = hp.column_x(total_cols.saturating_sub(1));
        let new_target = self.hex_camera(sel_center, cx, inset, fade_zone, vw);
        self.position_hex_camera(new_target.clamp(min_cam.min(max_cam), min_cam.max(max_cam)));
        let cam = self.camera.x;

        let coarse_margin = vw + hp.item_half_w().hypot(hp.item_half_h()) * 2.0;
        let (lens_x, lens_y, _) = hp.stage.transform(cx, cy, (cx, cy), (vw, vh));

        self.card.filter_cache.clear();
        let mut deferred: Vec<(f32, Option<InstanceRaw>, InstanceRaw, Hit)> = Vec::new();
        let mut lo = usize::MAX;
        let mut hi = 0usize;
        for col in 0..total_cols {
            let raw_x = cx - cam + hp.column_x(col);
            if raw_x < -coarse_margin || raw_x > vw + coarse_margin {
                continue;
            }
            let orbit_window = (hp.orbit_radius * std::f32::consts::PI * 1.25).max(vw * 1.75);
            if hp.orbit.abs() > 0.001 && (raw_x - cx).abs() > orbit_window {
                continue;
            }
            let (deformed_col_x, deformed_col_y) = hp.deform(col * rows, raw_x, cy, (cx, cy));
            let (screen_col_x, _, _) =
                hp.stage.transform(deformed_col_x, deformed_col_y, (cx, cy), (vw, vh));
            let scale = self.hex_scale(col, screen_col_x, inset, fade_zone, vw);
            if scale <= 0.01 {
                continue;
            }
            let (curve, odd_offset) = self.hex_offsets(col, raw_x, cx);
            for row in 0..rows {
                let idx = col * rows + row;
                if idx >= count {
                    break;
                }
                let raw_y = view_top + y_offset + hp.row_y(row) + odd_offset + curve;
                let (deformed_x, deformed_y) = hp.deform(idx, raw_x, raw_y, (cx, cy));
                let (item_x, item_cy, stage_scale) =
                    hp.stage.transform(deformed_x, deformed_y, (cx, cy), (vw, vh));
                let distance = ((item_x - lens_x).powi(2) + (item_cy - lens_y).powi(2)).sqrt();
                let lens_t = (1.0 - distance / hp.lens_radius.max(1.0)).clamp(0.0, 1.0);
                let lens_scale = (1.0 + hp.lens * anim::smoothstep(lens_t)).max(0.15);
                let render_scale = scale * stage_scale * lens_scale;
                let card_hw = hp.item_half_w() * render_scale;
                let card_hh = hp.item_half_h() * render_scale;
                let guard = 10.0;
                if item_x + card_hw < -guard
                    || item_x - card_hw > vw + guard
                    || item_cy + card_hh < -guard
                    || item_cy - card_hh > vh + guard
                {
                    continue;
                }
                lo = lo.min(idx);
                hi = hi.max(idx);
                self.hex_card(
                    ctx,
                    sinks,
                    &mut deferred,
                    idx,
                    item_x,
                    item_cy,
                    render_scale,
                    entrance,
                );
            }
        }
        deferred.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (_, shadow, body, hit) in deferred {
            if let Some(sh) = shadow {
                sinks.instances.push(sh);
            }
            sinks.instances.push(body);
            sinks.hits.push(hit);
        }
        self.push_flip_old(sinks.instances, sinks.wanted);
        (self.vis_lo, self.vis_hi) = vis_range(lo, hi, self.current);
    }

    pub(super) fn position_hex_camera(&mut self, selected_center: f32) {
        if self.layout_camera_anchor || !self.hp.settled_to(&self.hp_target) {
            self.camera.snap(selected_center);
            self.layout_camera_anchor = false;
        } else {
            self.camera.retarget(selected_center);
        }
    }

    fn inset_sides(&self, inset: f32) -> (f32, f32) {
        if self.reading_rtl { (0.0, inset) } else { (inset, 0.0) }
    }

    fn hex_camera(&self, sel_center: f32, cx: f32, inset: f32, fade_zone: f32, vw: f32) -> f32 {
        let (lead, trail) = self.inset_sides(inset);
        let left_bound = lead + fade_zone + self.hp.step_x();
        let right_bound = vw - trail - fade_zone - self.hp.step_x();
        if self.kb_nav || right_bound <= left_bound {
            return sel_center;
        }
        let cam_t = self.camera.target;
        let screen_x = cx - cam_t + sel_center;
        if screen_x < left_bound {
            cam_t + screen_x - left_bound
        } else if screen_x > right_bound {
            cam_t + screen_x - right_bound
        } else {
            cam_t
        }
    }

    pub(super) fn hex_scale(
        &mut self,
        col: usize,
        col_center: f32,
        inset: f32,
        fade_zone: f32,
        vw: f32,
    ) -> f32 {
        let (lead, trail) = self.inset_sides(inset);
        let left = lead + fade_zone;
        let right = vw - trail - fade_zone;
        let half_band = ((right - left) * 0.5).max(1.0);
        let dn = (col_center - f32::midpoint(left, right)).abs() / half_band;
        let base = 1.0 - 0.2 * anim::smoothstep((dn - 0.7) / 0.3);
        let over = (left - col_center).max(col_center - right).max(0.0);
        let cut = (1.0 - over / (self.hp.step_x() * 1.2)).clamp(0.0, 1.0);
        let target = if self.hp.curve == HexCurve::Flat { 1.0 } else { base * cut * cut };
        let motion_ms = self.motion_ms(MotionTier::Standard);
        let spring = self
            .card
            .hex_scales
            .entry(col)
            .or_insert_with(|| self.motion.profile.override_spring(target, motion_ms));
        spring.retarget(target);
        spring.x.clamp(0.0, 1.0)
    }

    fn hex_offsets(&self, col: usize, col_center: f32, cx: f32) -> (f32, f32) {
        let hp = self.hp;
        let normalized = (col_center - cx) / (cx).max(1.0);
        let phase = normalized * std::f32::consts::PI * hp.curve_frequency;
        let curve = match hp.curve {
            HexCurve::Flat | HexCurve::Cylinder => 0.0,
            HexCurve::Arc => -normalized * normalized * hp.r,
            HexCurve::Wave => phase.sin() * hp.r,
            HexCurve::S => normalized.powi(3) * hp.r,
        } * hp.curve_strength;
        let odd_offset = hp.stagger_offset(col);
        (curve, odd_offset)
    }

    fn hex_card(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        deferred: &mut Vec<(f32, Option<InstanceRaw>, InstanceRaw, Hit)>,
        idx: usize,
        col_center: f32,
        item_cy: f32,
        scale: f32,
        entrance: f32,
    ) {
        let hp = self.hp;
        let row = idx % hp.rows.max(1);
        let col = idx / hp.rows.max(1);
        let triangle_direction = match hp.shape {
            HexShape::Hexagon | HexShape::Diamond | HexShape::Rhombus => 0,
            HexShape::Triangle => u8::from(!(row + col).is_multiple_of(2)),
        };
        let shape_code = match hp.shape {
            HexShape::Hexagon => 0,
            HexShape::Triangle => 1 + u32::from(triangle_direction),
            HexShape::Diamond => 5,
            HexShape::Rhombus => 6,
        };
        let store_idx = ctx.filtered[idx] as usize;
        let is_selected = idx == self.current;
        let is_hover = Some(idx) == self.hover && !is_selected;
        let sel_t = self
            .card
            .selection
            .get(&idx)
            .map_or(if is_selected { 1.0 } else { 0.0 }, |sp| sp.x.clamp(0.0, 1.0));
        let ps = scale;
        let roll = self.flip_roll_for(store_idx, col_center, item_cy);
        let (mut body, mut hit) = self.place_card(
            ctx,
            sinks.wanted,
            sinks.chrome,
            &CardSpec {
                filtered_idx: idx,
                store_idx,
                cx: col_center,
                cy: item_cy,
                hw: hp.item_half_w() * ps,
                hh: hp.item_half_h() * ps,
                skew: 0.0,
                edge_tilt: 0.0,
                corners: CardCorners::default(),
                hex: true,
                view: 2,
                chrome_radius: 0.0,
                opacity: scale.clamp(0.0, 1.0) * entrance,
                chrome_opacity: scale.clamp(0.0, 1.0) * entrance * roll,
                body_inset: 0.0,
                near_ok: true,
            },
        );
        hit.hex_shape = hp.shape;
        hit.triangle_direction = triangle_direction;
        if hp.parallax {
            let center = self.center_x();
            let span = if col_center < center { center } else { self.viewport.0 - center };
            let position = (col_center - center) / (span + body.rect[2]).max(1.0);
            apply_position_parallax(&mut body, position);
        } else if body.misc[0] > 0 {
            let (co, cs) = layout::cover_crop(
                hp.item_half_w() * 2.6,
                hp.item_half_h() * 2.6,
                THUMB_W,
                THUMB_H,
            );
            let inner = 1.0 / 1.3;
            body.crop = [
                co[0] + cs[0] * (1.0 - inner) * 0.5,
                co[1] + cs[1] * (1.0 - inner) * 0.5,
                cs[0] * inner,
                cs[1] * inner,
            ];
        }
        body.misc[3] |= shape_code << 8;
        if sel_t > 0.01 {
            body.border = color4(ctx.palette.primary, 0.55 + 0.45 * sel_t);
            body.params[1] = 1.5 + 1.5 * sel_t;
        } else if is_hover {
            body.border = color4(ctx.palette.primary, 0.55);
            body.params[1] = 2.0;
        } else {
            body.border = [0.0, 0.0, 0.0, 0.5];
            body.params[1] = 1.5;
        }
        self.card.filter_cache.push((body, store_idx as u32, roll));
        roll_in_cut(&mut body, roll);
        let defer_key = sel_t + if is_hover { 0.3 } else { 0.0 };
        if defer_key > 0.01 {
            let shadow = (sel_t > 0.01).then(|| {
                let mut sh = InstanceRaw {
                    rect: [
                        col_center + 3.0,
                        item_cy + 7.0,
                        hp.item_half_w() * ps,
                        hp.item_half_h() * ps,
                    ],
                    fill: [0.0, 0.0, 0.0, 0.35 * sel_t * scale * entrance],
                    params: [0.0, 0.0, 1.0, 0.0],
                    ..Default::default()
                };
                sh.misc[3] |= 1 | (shape_code << 8);
                sh
            });
            deferred.push((defer_key, shadow, body, hit));
        } else {
            sinks.instances.push(body);
            sinks.hits.push(hit);
        }
    }
}
