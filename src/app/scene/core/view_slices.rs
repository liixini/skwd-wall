use std::collections::HashSet;

use crate::frontend::scene::layout;
use crate::frontend::scene::{
    InstanceRaw, card_flip_phases, card_flip_shader_payload, roll_in_cut,
};
use crate::infrastructure::preview::is_thumb;

use super::atlas_helpers::{enqueue_far, enqueue_near};
use super::layout_helpers::{
    CardSpec, STORM_CUTOFF, TOP_BAR, apply_position_parallax, prefetch_reach, vis_range, widen,
};
use super::model::{RebuildCtx, RebuildSinks, SceneCore};

impl SceneCore {
    pub(super) fn rebuild_slices(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let (vw, vh) = self.viewport;
        let sp = self.sp;
        let count = ctx.filtered.len();
        let strip_w = (sp.layout_width(sp.expanded_w)
            + (sp.visible_count.saturating_sub(1)) as f32 * sp.slice_stride())
        .min(self.avail_w() - 40.0);
        let half_view = strip_w / 2.0;
        let cx = self.center_x() + sp.offset_x * vw * 0.5;
        let cy = vh * 0.5 + sp.offset_y * vh * 0.5;

        let card_h = sp.slice_h + TOP_BAR + 60.0;

        if count == 0 {
            return;
        }
        self.current = self.current.min(count - 1);

        let mut centers = Vec::with_capacity(count);
        let mut acc = 0.0f32;
        let mut tacc = 0.0f32;
        let mut target_center = 0.0f32;
        for idx in 0..count {
            let layout_w = sp.layout_width(self.width_of(idx));
            centers.push(acc + layout_w * 0.5);
            acc += layout_w + sp.spacing;

            let target_layout_w = sp.layout_width(self.target_width_of(idx));
            if idx == self.current {
                target_center = tacc + target_layout_w * 0.5;
            }
            tacc += target_layout_w + sp.spacing;
        }

        let margin = sp.expanded_w;
        self.set_period(acc, tacc, strip_w + 2.0 * margin, count);
        self.camera.set_zeta(if sp.wobble { 0.45 } else { 1.0 });
        self.position_slice_camera(target_center, strip_w);
        let cam = self.camera.x;
        let bend = layout::wobble_bend(self.camera.v, sp.expanded_w);

        let top = cy - card_h * 0.5 + TOP_BAR + 15.0;
        let item_cy = top + sp.slice_h * 0.5;

        let mut seen = None;
        let mut visible: Vec<usize> = (0..count)
            .filter(|&idx| {
                let half_w = self.width_of(idx) * 0.5;
                let (center, turns) = self.near(centers[idx], cam);
                let x0 = center - half_w - (cam - half_view);
                let x1 = center + half_w - (cam - half_view);
                let shown = x1 >= -margin && x0 <= strip_w + margin;
                if shown {
                    seen = Some(widen(seen, idx as i64 + turns * count as i64));
                }
                shown
            })
            .collect();
        (self.vis_lo, self.vis_hi) = vis_range(seen, self.current, count);
        self.slice_order(&mut visible, count);

        self.card.filter_cache.clear();
        for &idx in &visible {
            let w = self.width_of(idx);
            let item_cx = cx - cam + self.near(centers[idx], cam).0;
            let opacity = layout::slice_opacity(
                item_cx,
                cx,
                half_view,
                sp.layout_width(sp.expanded_w),
                sp.slice_stride(),
            ) * entrance;
            if opacity <= 0.01 {
                continue;
            }
            self.slice_card(
                ctx,
                sinks,
                idx,
                w,
                item_cx,
                item_cy,
                opacity,
                half_view,
                bend,
                sp,
                sp.parallax.then_some((item_cx - cx) / (half_view * 1.2).max(1.0)),
            );
        }
        self.push_flip_old(sinks.instances, sinks.wanted);
    }

    pub(super) fn position_slice_camera(&mut self, cur_center: f32, reach: f32) {
        if self.layout_camera_anchor || !self.sp.settled_to(&self.sp_target) {
            self.camera.snap(cur_center);
            self.layout_camera_anchor = false;
        } else {
            self.aim_camera(cur_center, reach);
        }
    }

    fn slice_order(&self, visible: &mut [usize], count: usize) {
        visible.sort_by_key(|&idx| {
            if idx == self.current {
                i64::MAX
            } else if Some(idx) == self.hover {
                i64::MAX - 1
            } else {
                -self.travel(self.current, idx, count).abs()
            }
        });
    }

    pub(super) fn slice_card(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        idx: usize,
        w: f32,
        item_cx: f32,
        item_cy: f32,
        opacity: f32,
        half_view: f32,
        bend: f32,
        sp: layout::SliceParams,
        parallax: Option<f32>,
    ) {
        let cx = self.center_x() + sp.offset_x * self.viewport.0 * 0.5;
        let store_idx = ctx.filtered[idx] as usize;
        let is_current = idx == self.current;
        let is_hover = Some(idx) == self.hover;
        let roll = self.flip_roll_for(store_idx, item_cx, item_cy);
        let corners = sp.corners.sheared(w, sp.slice_h, sp.skew, sp.edge_tilt);
        if sp.shadows {
            let (sh_x, sh_y, sh_a) = if is_current { (4.0, 10.0, 0.5) } else { (2.0, 5.0, 0.3) };
            let mut shadow = InstanceRaw {
                rect: [item_cx + sh_x, item_cy + sh_y, w * 0.5, sp.slice_h * 0.5],
                fill: [0.0, 0.0, 0.0, sh_a],
                params: [sp.skew, 0.0, opacity * roll, 0.0],
                shape: [sp.edge_tilt, 0.0, 0.0, 0.0],
                ..Default::default()
            };
            corners.stamp(&mut shadow);
            sinks.instances.push(shadow);
        }
        let chrome_op = if self.card.flipped == Some(idx) {
            opacity * (1.0 - self.card.flip.x).clamp(0.0, 1.0)
        } else {
            opacity
        };
        let (mut body, hit) = self.place_card(
            ctx,
            sinks.wanted,
            sinks.chrome,
            &CardSpec {
                filtered_idx: idx,
                store_idx,
                cx: item_cx,
                cy: item_cy,
                hw: w * 0.5,
                hh: sp.slice_h * 0.5,
                skew: sp.skew,
                edge_tilt: sp.edge_tilt,
                corners,
                hex: false,
                view: 0,
                chrome_radius: corners.sizes[0],
                opacity,
                chrome_opacity: if self.mode == layout::Mode::Depth && !is_current {
                    0.0
                } else {
                    chrome_op * roll
                },
                body_inset: 0.0,
                near_ok: true,
            },
        );
        if self.card.flipped == Some(idx) {
            let phases = card_flip_phases(
                self.card.flip.x,
                self.card.flip_shader_enabled,
                self.card.flip_back_enabled,
            );
            body.flip = card_flip_shader_payload(phases.shader, idx, self.card.flip_effect);
            let progress = self.card.flip.x;
            self.card.pending_back = Some(self.make_back_panel(
                ctx,
                store_idx,
                item_cx,
                item_cy,
                w * 0.5,
                sp.slice_h * 0.5,
                sp.skew,
                sp.edge_tilt,
                corners,
                progress,
            ));
        }
        let sel = self
            .card
            .selection
            .get(&idx)
            .map_or(if is_current { 1.0 } else { 0.0 }, |spr| spr.x.clamp(0.0, 1.0));
        let prim = ctx.palette.primary;
        let (base_border, base_dim) = if is_hover {
            ([prim.r, prim.g, prim.b, 0.4], 0.15)
        } else {
            ([0.0, 0.0, 0.0, 0.6], 0.4)
        };
        let border_sel = if self.mode == layout::Mode::Depth && !self.xp.depth.selection_frame {
            0.0
        } else {
            sel
        };
        let cur_border = [prim.r, prim.g, prim.b, 1.0];
        body.border = [
            base_border[0] + (cur_border[0] - base_border[0]) * border_sel,
            base_border[1] + (cur_border[1] - base_border[1]) * border_sel,
            base_border[2] + (cur_border[2] - base_border[2]) * border_sel,
            base_border[3] + (cur_border[3] - base_border[3]) * border_sel,
        ];
        body.params[1] = 1.0 + 2.0 * border_sel;
        if self.mode == layout::Mode::Depth && !self.xp.depth.selection_frame {
            body.border = [0.0; 4];
            body.params[1] = 0.0;
            if sp.skew == 0.0 && sp.edge_tilt == 0.0 && corners.sizes == [0.0; 4] {
                body.misc[3] |= crate::rendering::scene::UNFRAMED_RECT;
            }
        }
        body.tint = [0.0, 0.0, 0.0, base_dim * (1.0 - sel)];
        if sp.wobble && self.card.flipped != Some(idx) {
            let pad = layout::wobble_pad(sp.wobble_strength);
            body.rect[2] *= pad;
            body.rect[3] *= pad;
            body.flip[2] = pad;
            body.misc[3] |= 32;
            body.flip[3] = bend * sp.wobble_strength;
            body.flip[1] = ((item_cx - cx) / half_view.max(1.0)).clamp(-1.0, 1.0) * 0.5;
        }
        if let Some(position) = parallax {
            apply_position_parallax(&mut body, position);
        }
        self.card.filter_cache.push((body, store_idx as u32, roll));
        roll_in_cut(&mut body, roll);
        sinks.instances.push(body);
        sinks.hits.push(hit);
    }

    pub(super) fn prefetch_ring(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        wanted: &mut HashSet<usize>,
        count: usize,
    ) {
        if count == 0 || self.scroll_vel > STORM_CUTOFF {
            return;
        }
        let Some(atlas) = ctx.atlas.as_mut() else { return };
        let (lo, hi) = (self.vis_lo, self.vis_hi);
        let looped = self.connected();
        let span = if lo <= hi { hi - lo + 1 } else { hi + count - lo + 1 };
        let (far_look, near_look) = if crate::contracts::preview::compressed_thumbnails() {
            prefetch_reach(span, atlas.near_capacity() as usize)
        } else {
            (prefetch_reach(span, 0).0, 0)
        };
        let far_look = if looped { far_look.min(count.saturating_sub(span) / 2) } else { far_look };
        for step in 1..=far_look {
            let ring = if looped {
                [Some((lo + count - step) % count), Some((hi + step) % count)]
            } else {
                [lo.checked_sub(step), Some(hi + step).filter(|&cand| cand < count)]
            };
            for idx in ring.into_iter().flatten() {
                let store_idx = ctx.filtered[idx] as usize;
                let item = &ctx.catalog.items[store_idx];
                wanted.insert(store_idx);
                if step <= near_look
                    && !atlas.near.is_known(store_idx)
                    && !atlas.near_failed.contains(&store_idx)
                    && is_thumb(&item.thumb)
                {
                    enqueue_near(ctx.pool, atlas, item, store_idx, false, self.direct_thumbnails);
                }
                if !atlas.far.is_known(store_idx)
                    && !atlas.failed.contains(&store_idx)
                    && is_thumb(&item.thumb)
                {
                    enqueue_far(ctx.pool, atlas, item, store_idx, false, self.direct_thumbnails);
                }
            }
        }
    }
}
