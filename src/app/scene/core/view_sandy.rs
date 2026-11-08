use std::collections::HashSet;

use crate::frontend::animation as anim;
use crate::frontend::scene::layout::{CardCorners, HexShape, Hit};
use crate::frontend::scene::sandy;
use crate::frontend::scene::{
    InstanceRaw, SandySnap, card_flip_phases, card_flip_shader_payload, roll_in_cut,
};
use crate::rendering::scene::{sandy_video_in, sandy_video_out};

use super::layout_helpers::{CardSpec, TOP_BAR, sandy_card_border};
use super::model::{RebuildCtx, RebuildSinks, SceneCore};

impl SceneCore {
    pub(super) fn rebuild_sandy(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let (vw, vh) = self.viewport;
        let count = ctx.filtered.len();
        self.sandy.snap = None;
        if count == 0 {
            return;
        }
        self.current = self.current.min(count - 1);
        let params = self.xp.sandy;
        let reach = self.avail_w() / params.stride();
        self.set_period(count as f32, count as f32, reach + 1.0, count);
        let target = if self.sandy.cam_free {
            self.camera.target
        } else {
            sandy::clamp_cam(self.current as f32, count)
        };
        if self.layout_camera_anchor {
            self.camera.snap(sandy::clamp_cam(self.current as f32, count));
            self.layout_camera_anchor = false;
        }
        self.aim_camera(target, reach);
        let cam = self.camera.x;
        let cx = self.center_x() + params.offset_x * vw * 0.5;
        let offset_y = params.offset_y * vh * 0.5;
        let strip_h = params.strip_h();
        let strip_bottom = sandy::strip_bottom(vh) + offset_y;
        let strip_cy = strip_bottom - strip_h * 0.5;
        let cy = (TOP_BAR + 20.0 + (sandy::strip_bottom(vh) - strip_h)) * 0.5 + offset_y;
        let blending =
            !self.sandy.bmix.settled() || !self.sandy.swirl.settled() || self.sandy.swirl.x > 0.001;
        let mut prog = self.sandy.prog.x.clamp(0.0, 1.0);
        if blending {
            prog = prog.min(0.995);
        }

        let chw = params.center_hw();
        let chh = params.center_hh();
        let shadow_t = anim::window(prog, 0.86, 0.96);
        let ring_t = anim::window(self.sandy.swirl.x, 0.08, 0.30);
        let hero_vis = self.sandy.hero_fade.x.clamp(0.0, 1.0);
        let shadow_op = shadow_t * entrance * hero_vis * (1.0 - ring_t);
        if shadow_op > 0.01 {
            sinks.instances.push(InstanceRaw {
                rect: [cx, cy + 12.0, chw + 44.0, chh + 44.0],
                fill: [0.0, 0.0, 0.0, 0.55 * shadow_op],
                radii: [18.0; 4],
                params: [0.0, 48.0, 1.0, 0.0],
                misc: [0, 0, 0, 64],
                ..Default::default()
            });
        }

        let places = sandy::place(count, cam, &params, cx, self.avail_w(), self.connected());
        self.card.filter_cache.clear();
        for pl in &places {
            self.sandy_card(ctx, sinks, pl, strip_cy, entrance);
        }
        self.push_flip_old(sinks.instances, sinks.wanted);
        (self.vis_lo, self.vis_hi) = places
            .first()
            .zip(places.last())
            .map_or((self.current, self.current), |(first, last)| (first.i, last.i));

        let cur_store = ctx.filtered[self.current] as usize;
        let ring_quiet = self.sandy.swirl.target < 0.5 && self.sandy.swirl.x < 0.001;
        let video_live = self.preview_slot_idx() == Some(cur_store) && ring_quiet;
        let video_in = sandy_video_in(prog, video_live);
        let out_from = self
            .sandy
            .from
            .and_then(|fi| ctx.filtered.get(fi).map(|&store| store as usize))
            .unwrap_or(cur_store);
        let video_out =
            sandy_video_out(self.preview_state.out_store == Some(out_from), !ring_quiet);
        let solid_op = if prog < 0.999 { 0.0 } else { 1.0 };
        if solid_op > 0.01 {
            self.sandy_hero(
                ctx,
                sinks.instances,
                sinks.hits,
                sinks.wanted,
                cur_store,
                [cx, cy],
                [chw, chh],
                solid_op * entrance * hero_vis,
            );
        } else {
            sinks.wanted.insert(cur_store);
        }
        let bto_store = self.sandy_wanted(ctx, sinks.wanted, cur_store);
        self.sandy_snapshot(
            ctx,
            cur_store,
            bto_store,
            prog,
            [cx, cy],
            [chw, chh],
            video_in,
            video_out,
        );
    }

    fn sandy_card(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        pl: &sandy::Place,
        strip_cy: f32,
        entrance: f32,
    ) {
        let params = self.xp.sandy;
        let store_idx = ctx.filtered[pl.i] as usize;
        let is_hover = Some(pl.i) == self.hover;
        let is_current = pl.i == self.current;
        let pop = if is_current {
            (std::f32::consts::PI * self.sandy.pop.x.clamp(0.0, 1.0)).sin()
        } else {
            0.0
        };
        let scale = pl.s
            * if is_hover {
                1.12
            } else if is_current {
                1.06 + 0.11 * pop
            } else {
                1.0
            };
        let roll = self.flip_roll_for(store_idx, pl.x, strip_cy);

        let shh = params.slice_h * 0.5 * scale;
        let shw = params.slice_w * 0.5 * scale;
        let (mut body, hit) = self.place_card(
            ctx,
            sinks.wanted,
            sinks.chrome,
            &CardSpec {
                filtered_idx: pl.i,
                store_idx,
                cx: pl.x,
                cy: strip_cy,
                hw: shw,
                hh: shh,
                skew: params.skew,
                edge_tilt: 0.0,
                corners: params.corners.sheared(shw * 2.0, shh * 2.0, params.skew, 0.0),
                hex: false,
                view: 1,
                chrome_radius: 0.0,
                opacity: pl.s * entrance,
                chrome_opacity: 0.0,
                body_inset: 0.0,
                near_ok: true,
            },
        );
        sandy_card_border(&mut body, ctx.palette.primary, is_hover, is_current, pop, roll);
        self.card.filter_cache.push((body, store_idx as u32, roll));
        roll_in_cut(&mut body, roll);
        sinks.instances.push(body);
        sinks.hits.push(hit);
    }

    fn sandy_hero(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        instances: &mut Vec<InstanceRaw>,
        hits: &mut Vec<Hit>,
        wanted: &mut HashSet<usize>,
        cur_store: usize,
        center: [f32; 2],
        hero: [f32; 2],
        opacity: f32,
    ) {
        let [cx, cy] = center;
        let [chw, chh] = hero;
        let mut big = self.body_instance(
            ctx,
            wanted,
            cur_store,
            [cx, cy, chw, chh],
            CardCorners::default(),
            0.0,
            0.0,
            true,
        );
        big.params[2] = opacity;
        if self.card.flipped == Some(self.current) {
            let phases = card_flip_phases(
                self.card.flip.x,
                self.card.flip_shader_enabled,
                self.card.flip_back_enabled,
            );
            big.flip = card_flip_shader_payload(phases.shader, self.current, self.card.flip_effect);
            self.card.pending_back = Some(self.make_back_panel(
                ctx,
                cur_store,
                cx,
                cy,
                chw,
                chh,
                0.0,
                0.0,
                CardCorners::default(),
                self.card.flip.x,
            ));
        }
        instances.push(big);
        hits.push(Hit {
            index: self.current,
            cx,
            cy,
            hw: chw,
            hh: chh,
            skew: 0.0,
            edge_tilt: 0.0,
            hex: false,
            hex_shape: HexShape::default(),
            triangle_direction: 0,
            quad: None,
        });
    }

    fn sandy_wanted(
        &self,
        ctx: &RebuildCtx<'_>,
        wanted: &mut HashSet<usize>,
        cur_store: usize,
    ) -> Option<usize> {
        let bto_store = self
            .sandy
            .bto
            .or(self.sandy.ring_to)
            .or(self.sandy.storm_to)
            .and_then(|fi| ctx.filtered.get(fi).map(|&store| store as usize));
        if let Some(bs) = bto_store
            && bs != cur_store
        {
            wanted.insert(bs);
        }
        for held in [self.sandy.bfrom, self.sandy.bfrom2, self.sandy.storm_to] {
            if let Some(bs) = held.and_then(|fi| ctx.filtered.get(fi).map(|&store| store as usize))
                && bs != cur_store
            {
                wanted.insert(bs);
            }
        }
        if let Some(bs) = self.sandy.filter_from
            && bs != cur_store
        {
            wanted.insert(bs);
        }
        bto_store
    }

    fn sandy_snapshot(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        cur_store: usize,
        bto_store: Option<usize>,
        prog: f32,
        center: [f32; 2],
        hero: [f32; 2],
        video_in: f32,
        video_out: f32,
    ) {
        let Some(atlas) = ctx.atlas.as_mut() else {
            return;
        };
        let params = self.xp.sandy;
        let from_store = self
            .sandy
            .from
            .and_then(|fi| ctx.filtered.get(fi).map(|&store| store as usize))
            .unwrap_or(cur_store);
        atlas.near.touch(from_store);
        atlas.near.touch(cur_store);
        let bfrom_store = self.sandy.filter_from.or_else(|| {
            self.sandy.bfrom.and_then(|fi| ctx.filtered.get(fi).map(|&store| store as usize))
        });
        if let Some(bs) = bfrom_store {
            atlas.near.touch(bs);
        }
        if let Some(bs) = bto_store {
            atlas.near.touch(bs);
        }
        let mut layer_b2 = bfrom_store.and_then(|bs| atlas.near.ready(bs));
        if params.swap_style >= 16.5 && layer_b2.is_none() {
            layer_b2 = atlas.near.ready(from_store);
        }
        let bfrom2_store =
            self.sandy.bfrom2.and_then(|fi| ctx.filtered.get(fi).map(|&store| store as usize));
        if let Some(bs) = bfrom2_store {
            atlas.near.touch(bs);
        }
        let layer_b3 = bfrom2_store.and_then(|bs| atlas.near.ready(bs));
        let layer_b = bto_store
            .and_then(|bs| atlas.near.ready(bs))
            .or_else(|| atlas.near.ready(cur_store))
            .or(layer_b2)
            .or(layer_b3)
            .or_else(|| atlas.near.ready(from_store));
        let layer_a = atlas.near.ready(from_store).or(layer_b);
        let (Some(layer_a), Some(layer_b)) = (layer_a, layer_b) else {
            return;
        };
        if prog >= 0.999 {
            return;
        }
        let [cx, cy] = center;
        let [chw, chh] = hero;
        self.sandy.snap = Some(SandySnap {
            layer_a,
            layer_b,
            progress: prog,
            center: [cx, cy],
            hero: [chw, chh],
            dir: self.sandy.dir,
            seed: (self.sandy.storm_to.or(self.sandy.ring_to).unwrap_or(self.current) % 977) as f32
                * 0.013,
            carry: self.sandy.carry,
            layer_b2: layer_b2.unwrap_or(layer_b),
            layer_b3: layer_b3.or(layer_b2).unwrap_or(layer_b),
            bcut: self.sandy.bcut.clamp(0.0, 1.0),
            bmix: if layer_b2.is_some() { self.sandy.bmix.x.clamp(0.0, 1.0) } else { 1.0 },
            swirl: self.sandy.swirl.x.clamp(0.0, 1.0),
            wave: if self.sandy.bto.is_some() { 1.0 } else { 0.0 },
            video_in,
            video_out,
            ring: [
                params.ring_spin.clamp(0.0, 3.0),
                params.ring_wave.clamp(0.0, 3.0),
                params.ring_soft.clamp(0.25, 3.0),
                params.ring_size.clamp(0.25, 3.0),
            ],
            grid: {
                let base = params.grain.max(1.0);
                let grain = sandy::sandy_lod_grain(base, self.sandy.lod_max, self.sandy.motion);
                let (gx, gy) = sandy::grain_grid(grain, chw, chh);
                [gx, gy]
            },
            res_scale: self.sandy.res_scale,
            knobs: [
                params.strands.clamp(2.0, 64.0),
                params.twist,
                params.orbit,
                params.turbulence,
                params.waist,
                params.front.clamp(0.1, 1.6),
                params.fan.clamp(0.1, 1.2),
                params.arc.clamp(0.0, 3.0),
                if params.swap_loop { 1.0 } else { 0.0 },
                params.swap_style.clamp(0.0, 17.0).round(),
            ],
        });
    }
}
