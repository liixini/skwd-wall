use std::collections::HashSet;

use crate::domain::library::catalog::WallpaperKind;
use glam::{Mat4, Vec3};

use crate::frontend::scene::hand::{
    self, Axis, CROP_ZOOM, Camera, GHOST_LEVELS, PAD, Pose, Quad, Rot, Y,
};
use crate::frontend::scene::layout::{HexShape, Hit};
use crate::frontend::scene::{Chrome, InstanceRaw};
use crate::rendering::scene::atlas::AtlasMap;
use crate::rendering::scene::{BACKDROP, BACKFACE, GHOST, MUTED, PROJECTED, RIBBON_COLUMNS};

use super::layout_helpers::{chrome_kind, color4};
use super::model::{Face, Layout, RebuildCtx, RebuildSinks, RevealPhase, SceneCore};

pub(super) struct Stage {
    pub(super) k: f32,
    pub(super) hw: f32,
    pub(super) hh: f32,
    pub(super) radius: f32,
    pub(super) skew: f32,
    pub(super) cam: Camera,
    pub(super) rig: (f32, f32),
    pub(super) parallax: (f32, f32),
    pub(super) viewport: (f32, f32),
}

struct Draw {
    depth: f32,
    inst: InstanceRaw,
    hit: Option<Hit>,
}

struct Slat {
    store: usize,
    idx: usize,
    slot: usize,
    len: usize,
    n: f32,
    layouts: (Layout, Layout),
    turn: f32,
    turns: u32,
    faces: [Face; 2],
}

impl SceneCore {
    pub(super) fn hand_stage(&self) -> Stage {
        let (vw, vh) = self.viewport;
        let hp = self.xp.hand;
        let k = self.hand_stage_scale();
        let rig = (self.hand.rig_x.x, self.hand.rig_y.x);
        let hw = hp.card_w * k * 0.5;
        let hh = hp.card_h * k * 0.5;
        Stage {
            k,
            hw,
            hh,
            radius: (hp.radius * k).clamp(0.0, (hw.min(hh) - 1.0).max(0.0)),
            skew: hp.skew * k,
            cam: Camera {
                d: hp.perspective * k,
                origin: [
                    self.center_x() + hp.offset_x * vw * 0.5,
                    vh * hand::ANCHOR_Y + hp.offset_y * vh * 0.5,
                ],
                shift: [0.0, hand::PERSPECTIVE_DROP * vh],
            },
            rig,
            parallax: hand::parallax(rig.0, rig.1, k),
            viewport: (vw, vh),
        }
    }

    pub(super) fn rebuild_hand(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let count = ctx.filtered.len();
        if count == 0 {
            self.hand.shown.clear();
            return;
        }
        self.current = self.current.min(count - 1);
        let hp = self.xp.hand;
        let stage = self.hand_stage();
        let cur_store = ctx.filtered[self.current] as usize;
        if hp.backdrop {
            let ready =
                ctx.atlas.as_ref().is_some_and(|atlas| atlas.near.ready(cur_store).is_some());
            let (store, prev, fade) = self.hand_backdrop_target(cur_store, ready);
            if let Some(prev) = prev.filter(|_| fade < 0.999) {
                self.hand_backdrop(ctx, sinks, prev, entrance);
            }
            if let Some(store) = store {
                let alpha = if prev.is_some() { fade } else { 1.0 };
                self.hand_backdrop(ctx, sinks, store, entrance * alpha);
            }
        }
        let mut draws: Vec<Draw> = Vec::with_capacity(48);
        let mut late_hits: Vec<Hit> = Vec::new();
        let size = hp.count.max(1);
        if let Some(deal) = self.hand.deal.take() {
            self.hand.shown = deal.cards.iter().map(|card| card.store).collect();
            for card in &deal.cards {
                let pose = card.pose.at(deal.t);
                let m = self.hand_card_matrix(&stage, &pose, card.slot, 1.0);
                if hp.ghosts {
                    for (level, ghost) in card.ghosts.iter().enumerate() {
                        let gm = self.hand_card_matrix(&stage, &ghost.at(deal.t), card.slot, 1.0);
                        self.hand_ghost(
                            ctx,
                            sinks.wanted,
                            &mut draws,
                            &stage,
                            card.store,
                            &gm,
                            level,
                            entrance,
                        );
                    }
                }
                if card.ribbons.is_empty() {
                    self.hand_plain(
                        ctx, sinks, &mut draws, &stage, card.store, card.n, &m, entrance, None,
                        0.0, 0.0,
                    );
                } else {
                    let poses: Vec<Pose> =
                        card.ribbons.iter().map(|ribbon| ribbon.at(deal.t)).collect();
                    self.hand_ribbons(
                        ctx, sinks, &mut draws, &stage, card.store, card.slot, card.n, &m, &poses,
                        entrance, false, 0.0,
                    );
                }
            }
            self.hand.deal = Some(deal);
        } else {
            let len = hand::hand_len(self.hand.offset, count, size);
            if self.current < self.hand.offset || self.current >= self.hand.offset + len {
                self.hand.offset = hand::hand_window(self.current, count, size);
            }
            let offset = self.hand.offset;
            let len = hand::hand_len(offset, count, size);
            self.hand.shown = (0..len).map(|slot| ctx.filtered[offset + slot] as usize).collect();
            let push = self.hand_push();
            let reveal = self.hand.reveal.as_ref().map(|reveal| {
                ((reveal.from, reveal.to), reveal.turn, reveal.turns_done, reveal.faces)
            });
            for slot in 0..len {
                let idx = offset + slot;
                let store = ctx.filtered[idx] as usize;
                let n = Self::hand_slot_n(slot, len);
                let sel_t = self.hand_sel_t(idx);
                let lift_t = self.hand_lift_t(idx);
                let pose = self.hand_rest_pose(slot, len, push);
                let opacity = entrance;
                if let Some((layouts, turn, turns, faces)) = reveal {
                    let slat = Slat { store, idx, slot, len, n, layouts, turn, turns, faces };
                    self.hand_slat(ctx, sinks, &mut draws, &stage, &pose, &slat, opacity);
                } else if self.card.flipped == Some(idx) {
                    if let Some(hit) = self.hand_flipped(
                        ctx, sinks, &mut draws, &stage, store, idx, slot, n, &pose, opacity,
                    ) {
                        late_hits.push(hit);
                    }
                } else {
                    let m = self.hand_card_matrix(&stage, &pose, slot, 1.0);
                    self.hand_plain(
                        ctx,
                        sinks,
                        &mut draws,
                        &stage,
                        store,
                        n,
                        &m,
                        opacity,
                        Some(idx),
                        sel_t,
                        lift_t,
                    );
                }
            }
        }
        self.hand_reveal_panel(ctx, &draws, cur_store);
        draws.sort_by(|a, b| a.depth.total_cmp(&b.depth));
        for draw in draws {
            if let Some(hit) = draw.hit {
                sinks.hits.push(hit);
            }
            sinks.instances.push(draw.inst);
        }
        sinks.hits.extend(late_hits);
        let len = hand::hand_len(self.hand.offset, count, size);
        self.vis_lo = self.hand.offset;
        self.vis_hi = (self.hand.offset + len).saturating_sub(1).max(self.vis_lo);
    }

    pub(super) fn hand_card_matrix(
        &self,
        stage: &Stage,
        pose: &Pose,
        slot: usize,
        rig_mix: f32,
    ) -> Mat4 {
        let rig =
            hand::rot(hand::X, stage.rig.0 * rig_mix) * hand::rot(hand::Y, stage.rig.1 * rig_mix);
        let mut m = rig * pose.matrix();
        if self.xp.hand.bob {
            let bob = hand::bob(self.hand.bob_t, slot, stage.k);
            let still = Pose::new([0.0; 3], &[Rot::new(hand::Z, 0.0)], 1.0);
            m *= Pose::mix(&bob, &still, 1.0 - rig_mix).matrix();
        }
        m
    }

    fn hand_quad(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        wanted: &mut HashSet<usize>,
        store: usize,
        quad: &Quad,
        locals: [f32; 4],
        stage: &Stage,
        columns: bool,
        shift: (f32, f32),
        extent: (f32, f32),
    ) -> InstanceRaw {
        let (hw, hh) = extent;
        let [cx, cy, _, _] = quad.bounds();
        let mut inst = self.body_instance(
            ctx,
            wanted,
            store,
            [cx, cy, hw, hh],
            [stage.radius.min((hw.min(hh) - 1.0).max(0.0)); 4],
            stage.skew,
            0.0,
            true,
        );
        inst.shape[1] = stage.skew.abs() * 0.5;
        if inst.misc[0] > 0 {
            let [x, y, w, h] = inst.crop;
            let nw = w / CROP_ZOOM;
            let nh = h / CROP_ZOOM;
            let sx = shift.0 / (hw * 2.0).max(1.0) * nw;
            let sy = shift.1 / (hh * 2.0).max(1.0) * nh;
            inst.crop = [
                (x + (w - nw) * 0.5 + sx).clamp(x, x + w - nw),
                (y + (h - nh) * 0.5 + sy).clamp(y, y + h - nh),
                nw,
                nh,
            ];
        }
        inst.misc[3] |= PROJECTED | if columns { RIBBON_COLUMNS } else { 0 };
        inst.border = [0.0, 0.0, 0.0, 0.5];
        inst.params[1] = 1.0;
        set_quad(&mut inst, quad, locals);
        inst
    }

    fn hand_plate(
        quad: &Quad,
        locals: [f32; 4],
        stage: &Stage,
        columns: bool,
        fill: [f32; 4],
        opacity: f32,
        seam: f32,
    ) -> InstanceRaw {
        let [cx, cy, _, _] = quad.bounds();
        let mut inst = InstanceRaw {
            rect: [cx, cy, stage.hw, stage.hh],
            radii: [stage.radius; 4],
            fill,
            border: [1.0, 1.0, 1.0, 0.17 * seam],
            params: [stage.skew, 1.0, opacity, 0.0],
            shape: [0.0, stage.skew.abs() * 0.5, 0.0, 0.0],
            misc: [0, 0, 0, PROJECTED | if columns { RIBBON_COLUMNS } else { 0 }],
            ..Default::default()
        };
        set_quad(&mut inst, quad, locals);
        inst
    }

    fn hand_plain(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        draws: &mut Vec<Draw>,
        stage: &Stage,
        store: usize,
        n: f32,
        m: &Mat4,
        opacity: f32,
        idx: Option<usize>,
        sel_t: f32,
        lift_t: f32,
    ) {
        let quad = hand::project_quad(
            &stage.cam,
            m,
            &hand::card_corners(stage.hw, stage.hh, PAD + stage.skew.abs() * 0.5, PAD),
        );
        if !quad.facing() || !quad.visible(stage.viewport.0, stage.viewport.1) {
            return;
        }
        let depth_factor = 1.0 - 0.16 * n.abs();
        let shift = (stage.parallax.0 * depth_factor, stage.parallax.1 * depth_factor);
        let mut inst = self.hand_quad(
            ctx,
            sinks.wanted,
            store,
            &quad,
            hand::card_locals(stage.hh, PAD),
            stage,
            false,
            shift,
            (stage.hw, stage.hh),
        );
        inst.params[2] = opacity;
        let prim = ctx.palette.primary;
        if idx.is_some() {
            let base = [prim.r * lift_t, prim.g * lift_t, prim.b * lift_t, 0.5 + 0.1 * lift_t];
            let cur = color4(prim, 1.0);
            inst.border = [
                base[0] + (cur[0] - base[0]) * sel_t,
                base[1] + (cur[1] - base[1]) * sel_t,
                base[2] + (cur[2] - base[2]) * sel_t,
                base[3] + (cur[3] - base[3]) * sel_t,
            ];
            inst.params[1] = 1.0 + 2.0 * sel_t;
        }
        let [cx, cy, hw, hh] = quad.bounds();
        let hit = idx.map(|index| Hit {
            index,
            cx,
            cy,
            hw,
            hh,
            skew: 0.0,
            edge_tilt: 0.0,
            hex: false,
            hex_shape: HexShape::Hexagon,
            triangle_direction: 0,
            quad: None,
        });
        if idx == Some(self.current) {
            let item = &ctx.catalog.items[store];
            sinks.chrome.push(Chrome {
                view: 0,
                cx,
                cy,
                hw,
                hh,
                skew: 0.0,
                edge_tilt: 0.0,
                kind: chrome_kind(item.kind.as_str()),
                has_video: item.effective_kind() == WallpaperKind::Video,
                favourite: ctx.catalog.is_favourite(item),
                hidden: ctx.catalog.is_hidden(item),
                radius: 0.0,
                opacity,
            });
        }
        draws.push(Draw { depth: quad.depth, inst, hit });
    }

    fn hand_ribbons(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        draws: &mut Vec<Draw>,
        stage: &Stage,
        store: usize,
        slot: usize,
        n: f32,
        m_card: &Mat4,
        poses: &[Pose],
        opacity: f32,
        show_back: bool,
        seam: f32,
    ) -> Option<[f32; 4]> {
        let hp = self.xp.hand;
        let axis = hp.axis;
        let columns = axis == Axis::Columns;
        let total = if columns { stage.hw * 2.0 } else { stage.hh * 2.0 };
        let pad_x = PAD + stage.skew.abs() * 0.5;
        let (pad_along, pad_cross) = if columns { (pad_x, PAD) } else { (PAD, pad_x) };
        let cuts = hand::ribbon_cuts(
            poses.len(),
            total,
            hp.cut,
            hp.variance,
            self.hand.lseed,
            slot,
            pad_along,
        );
        let depth_factor = 1.0 - 0.16 * n.abs();
        let shift = (stage.parallax.0 * depth_factor, stage.parallax.1 * depth_factor);
        let pv = ctx.palette.surface_variant;
        let plate_fill = [pv.r, pv.g, pv.b, 1.0];
        let mut quads = Vec::with_capacity(poses.len());
        for (cut, pose) in cuts.iter().zip(poses) {
            let origin = hand::ribbon_origin(cut, axis);
            let back = match axis {
                Axis::Rows => Mat4::from_translation(Vec3::new(0.0, -cut.mid, 0.0)),
                Axis::Columns => Mat4::from_translation(Vec3::new(-cut.mid, 0.0, 0.0)),
            };
            let m = *m_card * origin * pose.matrix() * back;
            let quad = hand::project_quad(
                &stage.cam,
                &m,
                &hand::ribbon_corners(cut, axis, stage.hw, stage.hh, pad_cross),
            );
            quads.push(quad);
            if !quad.visible(stage.viewport.0, stage.viewport.1) {
                continue;
            }
            let locals = hand::ribbon_locals(cut, axis);
            if quad.facing() {
                let mut inst = self.hand_quad(
                    ctx,
                    sinks.wanted,
                    store,
                    &quad,
                    locals,
                    stage,
                    columns,
                    shift,
                    (stage.hw, stage.hh),
                );
                inst.params[2] = opacity;
                if seam > 0.001 {
                    inst.border = [1.0, 1.0, 1.0, 0.11 * seam];
                }
                draws.push(Draw { depth: quad.depth, inst, hit: None });
            } else if show_back {
                let mirrored = hand::mirrored_cut(cut);
                let quad = hand::project_quad(
                    &stage.cam,
                    &m,
                    &hand::ribbon_corners(&mirrored, axis, stage.hw, stage.hh, pad_cross),
                );
                let locals = hand::ribbon_locals(&mirrored, axis);
                let mut inst = self.hand_quad(
                    ctx,
                    sinks.wanted,
                    store,
                    &quad,
                    locals,
                    stage,
                    columns,
                    shift,
                    (stage.hw, stage.hh),
                );
                if inst.misc[0] == 0 {
                    inst =
                        Self::hand_plate(&quad, locals, stage, columns, plate_fill, opacity, seam);
                } else {
                    inst.misc[3] |= BACKFACE;
                    inst.fill = plate_fill;
                    inst.border = [1.0, 1.0, 1.0, 0.17 * seam];
                    inst.params[1] = 1.0;
                    inst.params[2] = opacity;
                    inst.shape[2] = cut.mid;
                }
                draws.push(Draw { depth: quad.depth, inst, hit: None });
            }
        }
        (!quads.is_empty()).then(|| hand::union_bounds(&quads))
    }

    fn hand_reveal_panel(&mut self, ctx: &RebuildCtx<'_>, draws: &[Draw], cur_store: usize) {
        let Some(reveal) = self.hand.reveal.as_ref() else { return };
        let visible = reveal.faces[hand::reveal_face(reveal.turns_done, 0.0)];
        let incoming = reveal.faces[((reveal.turns_done + 1) % 2) as usize];
        let ss = self.xp.hand.speed_scale();
        let f = (reveal.turn / hand::reveal_turn_end(reveal.len, ss).max(0.001)).clamp(0.0, 1.0);
        let turning = reveal.phase == RevealPhase::Turning;
        let is_back = |face: Face| matches!(face, Face::Back { .. });
        let progress = if !turning && is_back(visible) {
            1.0
        } else if turning && is_back(incoming) {
            0.74 + 0.26 * ((f - 0.75) / 0.25).clamp(0.0, 1.0)
        } else if turning && is_back(visible) {
            1.0 - f * 1.2
        } else {
            0.0
        };
        if progress < hand::FLIP_LAND_AT - 0.04 {
            return;
        }
        let hits = draws.iter().filter_map(|draw| draw.hit.as_ref());
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for hit in hits {
            x0 = x0.min(hit.cx - hit.hw);
            y0 = y0.min(hit.cy - hit.hh);
            x1 = x1.max(hit.cx + hit.hw);
            y1 = y1.max(hit.cy + hit.hh);
        }
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let (cx, cy, hw, hh) = ((x0 + x1) * 0.5, (y0 + y1) * 0.5, (x1 - x0) * 0.5, (y1 - y0) * 0.5);
        self.card.pending_back = Some(
            self.make_back_panel(ctx, cur_store, cx, cy, hw, hh, 0.0, 0.0, [0.0; 4], progress),
        );
    }

    fn hand_slat(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        draws: &mut Vec<Draw>,
        stage: &Stage,
        rest: &Pose,
        slat: &Slat,
        opacity: f32,
    ) {
        let hp = self.xp.hand;
        let ss = hp.speed_scale();
        let order = hand::reveal_order(slat.slot, slat.len, slat.turns);
        let t = hand::reveal_turn(slat.turn, order, ss);
        if t > 0.0 && t < 1.0 {
            for (lag, alpha, blur) in hand::REVEAL_GHOST_LAG {
                let lagged = hand::reveal_turn(slat.turn - lag * ss, order, ss);
                if lagged <= 0.0 || (lagged - t).abs() < 1e-4 {
                    continue;
                }
                if let Some((mut inst, quad, _)) =
                    self.slat_instance(ctx, sinks.wanted, stage, rest, slat, lagged)
                    && inst.misc[0] > 0
                {
                    inst.misc[3] |= GHOST;
                    inst.flip[3] = blur;
                    inst.border = [0.0; 4];
                    inst.params[1] = 0.0;
                    inst.params[2] = alpha * opacity;
                    draws.push(Draw { depth: quad.depth, inst, hit: None });
                }
            }
        }
        let Some((mut inst, quad, index)) =
            self.slat_instance(ctx, sinks.wanted, stage, rest, slat, t)
        else {
            return;
        };
        inst.params[2] = opacity;
        let [cx, cy, hw, hh] = quad.bounds();
        let hit = Hit {
            index,
            cx,
            cy,
            hw,
            hh,
            skew: 0.0,
            edge_tilt: 0.0,
            hex: false,
            hex_shape: HexShape::Hexagon,
            triangle_direction: 0,
            quad: None,
        };
        draws.push(Draw { depth: quad.depth, inst, hit: Some(hit) });
    }

    fn slat_instance(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        wanted: &mut HashSet<usize>,
        stage: &Stage,
        rest: &Pose,
        slat: &Slat,
        t: f32,
    ) -> Option<(InstanceRaw, Quad, usize)> {
        let (vw, vh) = stage.viewport;
        let gap = hand::REVEAL_GAP * stage.k;
        let row_frame = hand::reveal_frame(vw, vh, false);
        let column_frame = hand::reveal_frame(vw, vh, true);
        let fill = self.xp.hand.reveal_fill;
        let card = (stage.hw, stage.hh);
        let extent = |layout: Layout| match (layout, fill) {
            (Layout::Fan, _) => card,
            (Layout::Row, true) => hand::slat_half_extent(slat.len, row_frame, gap, false),
            (Layout::Column, true) => hand::slat_half_extent(slat.len, column_frame, gap, true),
            (Layout::Row, false) => hand::keep_extent(slat.len, card, row_frame, gap, false),
            (Layout::Column, false) => hand::keep_extent(slat.len, card, column_frame, gap, true),
        };
        let layout_pose = |layout: Layout| match layout {
            Layout::Fan => *rest,
            Layout::Row => hand::row_pose(slat.n, extent(Layout::Row).0 * 2.0, gap),
            Layout::Column => hand::column_pose(slat.n, extent(Layout::Column).0 * 2.0, gap),
        };
        let (from, to) = slat.layouts;
        let travel = hand::EASE_REVEAL_TRAVEL.at(t);
        let (pose, (hw, hh)) = if from == to {
            (layout_pose(to), extent(to))
        } else {
            let (fw, fh) = extent(from);
            let (tw, th) = extent(to);
            (
                Pose::mix(&layout_pose(from), &layout_pose(to), travel),
                (fw + (tw - fw) * travel, fh + (th - fh) * travel),
            )
        };
        let gather = match (from, to) {
            (Layout::Fan, Layout::Fan) => 0.0,
            (Layout::Fan, _) => t,
            (_, Layout::Fan) => 1.0 - t,
            _ => 1.0,
        };
        let dir = hand::reveal_dir(slat.slot, slat.turns);
        let (turn, e) = hand::slat_turn_pose(t, dir, stage.k);
        let m = self.hand_card_matrix(stage, &pose, slat.slot, 1.0)
            * hand::rot(Y, 180.0 * slat.turns as f32 * dir)
            * turn.matrix();
        let quad = hand::project_quad(
            &stage.cam,
            &m,
            &hand::card_corners(hw, hh, PAD + stage.skew.abs() * 0.5, PAD),
        );
        if !quad.visible(vw, vh) {
            return None;
        }
        let quad = if quad.facing() { quad } else { quad.mirrored() };
        let locals = hand::card_locals(hh, PAD);
        let face = slat.faces[hand::reveal_face(slat.turns, e)];
        let prim = ctx.palette.primary;
        let depth_factor = 1.0 - 0.16 * slat.n.abs() * (1.0 - gather);
        let shift = (stage.parallax.0 * depth_factor, stage.parallax.1 * depth_factor);
        let n = slat.len as f32;
        let (row_w, row_h) = {
            let (rw, rh) = extent(Layout::Row);
            (n * rw * 2.0 + (n - 1.0) * gap, rh * 2.0)
        };
        let (stack_w, stack_h) = {
            let (cw, ch) = extent(Layout::Column);
            (ch * 2.0, n * cw * 2.0 + (n - 1.0) * gap)
        };
        Some(match face {
            Face::Card => {
                let mut inst = self.hand_quad(
                    ctx,
                    wanted,
                    slat.store,
                    &quad,
                    locals,
                    stage,
                    false,
                    shift,
                    (hw, hh),
                );
                inst.border = [prim.r, prim.g, prim.b, 0.5];
                (inst, quad, slat.idx)
            }
            Face::Slice { store, column } | Face::Back { store, column } => {
                let [cx, cy, _, _] = quad.bounds();
                let mut inst = self.body_instance(
                    ctx,
                    wanted,
                    store,
                    [cx, cy, hw, hh],
                    [stage.radius.min((hw.min(hh) - 1.0).max(0.0)); 4],
                    stage.skew,
                    0.0,
                    true,
                );
                if column {
                    inst.misc[0] = 3;
                    inst.misc[1] = 0;
                    inst.uv = [0.0, 0.0, 1.0, 1.0];
                    inst.params[3] = 1.0;
                    inst.crop = hand::column_crop(slat.slot, slat.len, stack_w, stack_h, gap);
                } else if inst.misc[0] > 0 {
                    inst.crop = hand::slice_crop(slat.slot, slat.len, row_w, row_h, gap);
                }
                inst.shape[1] = stage.skew.abs() * 0.5;
                inst.misc[3] |= PROJECTED;
                if matches!(face, Face::Back { .. }) {
                    inst.misc[3] |= MUTED;
                    let pv = ctx.palette.surface_variant;
                    inst.fill = [pv.r, pv.g, pv.b, 1.0];
                }
                inst.border = [prim.r, prim.g, prim.b, 0.35];
                inst.params[1] = 1.0;
                set_quad(&mut inst, &quad, locals);
                (inst, quad, self.current)
            }
        })
    }

    fn hand_flipped(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        draws: &mut Vec<Draw>,
        stage: &Stage,
        store: usize,
        idx: usize,
        slot: usize,
        n: f32,
        rest: &Pose,
        opacity: f32,
    ) -> Option<Hit> {
        let hp = self.xp.hand;
        let nr = hp.ribbons.max(1);
        let p = self.card.flip.x.clamp(0.0, 1.0);
        let closing = self.hand.flip_closing;
        let total_ms = if closing { hand::flip_close_ms(nr) } else { hand::flip_open_ms(nr) };
        let tau = if closing {
            ((hand::FLIP_LAND_AT - p) / hand::FLIP_LAND_AT).clamp(0.0, 1.0) * total_ms
        } else {
            (p / hand::FLIP_LAND_AT).min(1.0) * total_ms
        };
        let relax = hand::relax(p);
        let twist = hand::flip_twist_pose(tau, closing, stage.k);
        let flat = Pose::new([0.0; 3], &[Rot::new(Y, 0.0)], 1.0);
        let twist = Pose::mix(&twist, &flat, relax);
        let m_card = self.hand_card_matrix(stage, rest, slot, 1.0 - relax) * twist.matrix();
        let seed = self.hand.flip_seed;
        let poses: Vec<Pose> = (0..nr)
            .map(|r| {
                hand::flip_ribbon_pose(
                    tau,
                    closing,
                    r,
                    nr,
                    hand::ribbon_dir(seed, slot, r, false),
                    stage.k,
                    hp.axis,
                )
            })
            .collect();
        let seam = if p > 0.001 { 1.0 - relax } else { 0.0 };
        self.hand_ribbons(
            ctx,
            sinks,
            draws,
            stage,
            store,
            slot,
            n,
            &m_card,
            &poses,
            opacity,
            p > 0.001,
            seam,
        )?;
        let outline = hand::project_quad(
            &stage.cam,
            &m_card,
            &hand::card_corners(stage.hw, stage.hh, PAD + stage.skew.abs() * 0.5, PAD),
        );
        let [cx, cy, hw, hh] = outline.bounds();
        let panel_from = hand::FLIP_LAND_AT - 0.04;
        if p >= panel_from {
            let panel_p = 0.74 + 0.26 * ((p - panel_from) / (1.0 - panel_from)).clamp(0.0, 1.0);
            self.card.pending_back =
                Some(self.make_back_panel(ctx, store, cx, cy, hw, hh, 0.0, 0.0, [0.0; 4], panel_p));
        }
        Some(Hit {
            index: idx,
            cx,
            cy,
            hw,
            hh,
            skew: 0.0,
            edge_tilt: 0.0,
            hex: false,
            hex_shape: HexShape::Hexagon,
            triangle_direction: 0,
            quad: None,
        })
    }

    fn hand_ghost(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        wanted: &mut HashSet<usize>,
        draws: &mut Vec<Draw>,
        stage: &Stage,
        store: usize,
        m: &Mat4,
        level: usize,
        entrance: f32,
    ) {
        let quad = hand::project_quad(
            &stage.cam,
            m,
            &hand::card_corners(stage.hw, stage.hh, PAD + stage.skew.abs() * 0.5, PAD),
        );
        if !quad.visible(stage.viewport.0, stage.viewport.1) {
            return;
        }
        let (alpha, blur, _) = GHOST_LEVELS[level];
        let mut inst = self.hand_quad(
            ctx,
            wanted,
            store,
            &quad,
            hand::card_locals(stage.hh, PAD),
            stage,
            false,
            (0.0, 0.0),
            (stage.hw, stage.hh),
        );
        if inst.misc[0] == 0 {
            return;
        }
        inst.misc[3] |= GHOST;
        inst.flip[3] = blur;
        inst.border = [0.0; 4];
        inst.params[1] = 0.0;
        inst.params[2] = alpha * entrance;
        draws.push(Draw { depth: quad.depth, inst, hit: None });
    }

    fn hand_backdrop(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        store: usize,
        entrance: f32,
    ) {
        let (vw, vh) = self.viewport;
        let mut inst = self.body_instance(
            ctx,
            sinks.wanted,
            store,
            [vw * 0.5, vh * 0.5, vw * 0.5, vh * 0.5],
            [0.0; 4],
            0.0,
            0.0,
            true,
        );
        if inst.misc[0] == 3
            && let Some(id) = ctx.atlas.as_ref().and_then(|atlas| atlas.near.ready(store))
        {
            let (offset, scale, layer) = AtlasMap::near_uv(id);
            inst.misc[0] = 1;
            inst.misc[1] = layer;
            inst.uv = [offset[0], offset[1], scale[0], scale[1]];
            inst.params[3] = 1.0;
        }
        if inst.misc[0] != 1 {
            return;
        }
        inst.misc[3] |= BACKDROP;
        inst.params[2] = entrance;
        inst.flip[3] = self.xp.hand.blur;
        inst.border = [0.0; 4];
        sinks.instances.push(inst);
    }
}

fn set_quad(inst: &mut InstanceRaw, quad: &Quad, locals: [f32; 4]) {
    let [tl, tr, br, bl] = quad.pts;
    inst.quad_a = [tl[0], tl[1], tr[0], tr[1]];
    inst.quad_b = [br[0], br[1], bl[0], bl[1]];
    inst.quad_w = [tl[2], tr[2], br[2], bl[2]];
    inst.quad_l = locals;
}
