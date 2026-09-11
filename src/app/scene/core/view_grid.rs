use crate::frontend::scene::layout::{GridLayout, signed_hash};
use crate::frontend::scene::{InstanceRaw, roll_in_cut};

use super::layout_helpers::{CardSpec, color4};
use super::model::{RebuildCtx, RebuildSinks, SceneCore};

impl SceneCore {
    pub(super) fn rebuild_grid(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) -> Option<[f32; 4]> {
        let (vw, vh) = self.viewport;
        let gp = self.gp;
        let count = ctx.filtered.len();
        let total_w = gp.total_w();
        let view_h = gp.total_h();
        let (cx, cy) = self.wall_composition_center();
        if count == 0 {
            return None;
        }
        let placements = self.grid_placements(ctx, total_w);
        let content_h = placements.iter().map(|place| place.2 + place.4).fold(0.0_f32, f32::max);
        let max_scroll = (content_h - view_h).max(0.0);
        let target = self.camera.target.clamp(0.0, max_scroll);
        self.camera.retarget(target);
        let cam = self.camera.x;
        let left = cx - total_w * 0.5;
        let top = cy - view_h * 0.5;
        let frame = [left.max(0.0), top.max(0.0), (left + total_w).min(vw), (top + view_h).min(vh)];

        self.vis_lo = count - 1;
        self.vis_hi = 0;
        self.card.filter_cache.clear();
        for (idx, x, y, width, height) in placements {
            let raw_x = left + x + width * 0.5;
            let raw_y = top + y - cam + height * 0.5;
            let (surface_x, surface_y, surface_scale) =
                gp.cylinder_transform(raw_x, raw_y, (cx, cy));
            let (cell_x, cell_y, stage_scale) =
                gp.stage.transform(surface_x, surface_y, (cx, cy), self.viewport);
            let draw_w = width * stage_scale * surface_scale;
            let draw_h = height * stage_scale * surface_scale;
            let focus =
                if idx == self.current && self.user_engaged { gp.selected_scale } else { 1.0 };
            let half_w = draw_w * focus * 0.5;
            let half_h = draw_h * focus * 0.5;
            if cell_x + half_w <= frame[0]
                || cell_x - half_w >= frame[2]
                || cell_y + half_h <= frame[1]
                || cell_y - half_h >= frame[3]
            {
                continue;
            }
            self.vis_lo = self.vis_lo.min(idx);
            self.vis_hi = self.vis_hi.max(idx);
            self.grid_cell(ctx, sinks, idx, cell_x, cell_y, draw_w, draw_h, entrance);
        }
        self.push_flip_old(sinks.instances, sinks.wanted);
        Some(frame)
    }

    fn grid_placements(
        &self,
        ctx: &RebuildCtx<'_>,
        total_w: f32,
    ) -> Vec<(usize, f32, f32, f32, f32)> {
        let gp = self.gp;
        let count = ctx.filtered.len();
        let cols = gp.cols.max(1);
        let mut out = match gp.layout {
            GridLayout::Uniform | GridLayout::Brick | GridLayout::Cylinder => (0..count)
                .map(|idx| {
                    let row = idx / cols;
                    let col = idx % cols;
                    let stagger = if gp.layout == GridLayout::Brick && row % 2 == 1 {
                        gp.cell_w() * gp.stagger
                    } else {
                        0.0
                    };
                    (
                        idx,
                        col as f32 * gp.cell_w() + stagger,
                        row as f32 * gp.cell_h(),
                        gp.thumb_w,
                        gp.thumb_h,
                    )
                })
                .collect(),
            GridLayout::Masonry => {
                let mut bottoms = vec![0.0_f32; cols];
                let mut out = Vec::with_capacity(count);
                for idx in 0..count {
                    let col = bottoms
                        .iter()
                        .enumerate()
                        .min_by(|a, b| a.1.total_cmp(b.1))
                        .map_or(0, |(col, _)| col);
                    let item = &ctx.catalog.items[ctx.filtered[idx] as usize];
                    let aspect = if item.width > 0 && item.height > 0 {
                        item.width as f32 / item.height as f32
                    } else {
                        gp.thumb_w / gp.thumb_h.max(1.0)
                    };
                    let height =
                        (gp.thumb_w / aspect.max(0.15)).clamp(gp.thumb_h * 0.55, gp.thumb_h * 1.8);
                    let y = bottoms[col];
                    out.push((idx, col as f32 * gp.cell_w(), y, gp.thumb_w, height));
                    bottoms[col] = y + height + gp.gap_y;
                }
                out
            }
            GridLayout::Justified => {
                let mut out = Vec::with_capacity(count);
                let mut y = 0.0;
                for start in (0..count).step_by(cols) {
                    let end = (start + cols).min(count);
                    let aspects: Vec<f32> = (start..end)
                        .map(|idx| {
                            let item = &ctx.catalog.items[ctx.filtered[idx] as usize];
                            if item.width > 0 && item.height > 0 {
                                (item.width as f32 / item.height as f32).clamp(0.45, 2.5)
                            } else {
                                gp.thumb_w / gp.thumb_h.max(1.0)
                            }
                        })
                        .collect();
                    let gaps = gp.gap_x * (aspects.len().saturating_sub(1) as f32);
                    let height = ((total_w - gaps) / aspects.iter().sum::<f32>().max(0.1))
                        .clamp(gp.thumb_h * 0.58, gp.thumb_h * 1.5);
                    let mut x = 0.0;
                    for (offset, aspect) in aspects.into_iter().enumerate() {
                        let width = aspect * height;
                        out.push((start + offset, x, y, width, height));
                        x += width + gp.gap_x;
                    }
                    y += height + gp.gap_y;
                }
                out
            }
            GridLayout::Editorial => {
                let mut out = Vec::with_capacity(count);
                let block_h = gp.thumb_h * 2.0 + gp.gap_y;
                let hero_fraction = (0.5 + 0.16 * gp.stagger).clamp(0.35, 0.72);
                let hero_w = (total_w * hero_fraction).max(1.0);
                let side_w = (total_w - hero_w - gp.gap_x).max(1.0);
                let small_w = ((side_w - gp.gap_x) * 0.5).max(1.0);
                let small_h = ((block_h - gp.gap_y) * 0.5).max(1.0);
                for start in (0..count).step_by(5) {
                    let block = start / 5;
                    let y = block as f32 * (block_h + gp.gap_y);
                    let hero_right = block % 2 == 1;
                    let hero_x = if hero_right { side_w + gp.gap_x } else { 0.0 };
                    out.push((start, hero_x, y, hero_w, block_h));
                    for step in 1..5 {
                        let idx = start + step;
                        if idx >= count {
                            break;
                        }
                        let slot = step - 1;
                        let x0 = if hero_right { 0.0 } else { hero_w + gp.gap_x };
                        let x = x0 + (slot % 2) as f32 * (small_w + gp.gap_x);
                        let sy = y + (slot / 2) as f32 * (small_h + gp.gap_y);
                        out.push((idx, x, sy, small_w, small_h));
                    }
                }
                out
            }
        };

        let scatter = gp.scatter;
        let variance = gp.scale_variance;
        let flow_span = gp.span_h(gp.rows.max(1)).max(1.0);
        for (idx, x, y, width, height) in &mut out {
            if gp.flow_wave.abs() > 0.001 {
                let phase =
                    (*y + *height * 0.5) / flow_span * std::f32::consts::TAU * gp.flow_frequency;
                *x += phase.sin() * gp.flow_wave;
            }
            if scatter > 0.001 {
                *x += signed_hash(*idx, 0xc2b2_ae35) * scatter;
                *y += signed_hash(*idx, 0x27d4_eb2f) * scatter;
            }
            if variance > 0.001 {
                let factor = (1.0 + signed_hash(*idx, 0x1656_67b1) * variance).clamp(0.3, 2.0);
                let next_w = *width * factor;
                let next_h = *height * factor;
                *x += (*width - next_w) * 0.5;
                *y += (*height - next_h) * 0.5;
                *width = next_w;
                *height = next_h;
            }
        }
        out
    }

    fn grid_cell(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        idx: usize,
        cell_x: f32,
        cell_y: f32,
        width: f32,
        height: f32,
        entrance: f32,
    ) {
        let gp = self.gp;
        let store_idx = ctx.filtered[idx] as usize;
        let is_hover = Some(idx) == self.hover;
        let is_cur = idx == self.current && self.kb_nav;
        let base_lift = if is_hover { 1.0 } else { 0.0 };
        let hover_lift = ctx.hover_fades.map_or(base_lift, |fades| {
            fades.get(&idx).map_or(base_lift, |spring| spring.value().clamp(0.0, 1.0))
        });
        let border_alpha = if is_cur { hover_lift.max(0.8) } else { hover_lift };
        let focus = if idx == self.current && self.user_engaged { gp.selected_scale } else { 1.0 };
        let hw = width * focus * 0.5;
        let hh = height * focus * 0.5;
        let radius = gp.corner_radius.clamp(0.0, hw.min(hh));
        let border_width = gp.border_width.clamp(0.0, (hw.min(hh) - 1.0).max(0.0));
        let roll = self.flip_roll_for(store_idx, cell_x, cell_y);
        let outer = InstanceRaw {
            rect: [cell_x, cell_y, hw, hh],
            radii: [radius; 4],
            fill: color4(ctx.palette.surface, 0.6 * entrance),
            params: [0.0, 0.0, entrance, 0.0],
            misc: [0, 0, 1, 0],
            ..Default::default()
        };
        sinks.instances.push(outer);
        let (mut body, hit) = self.place_card(
            ctx,
            sinks.wanted,
            sinks.chrome,
            &CardSpec {
                filtered_idx: idx,
                store_idx,
                cx: cell_x,
                cy: cell_y,
                hw,
                hh,
                skew: 0.0,
                edge_tilt: 0.0,
                radii: [radius; 4],
                hex: false,
                view: 1,
                chrome_radius: radius,
                opacity: entrance,
                chrome_opacity: entrance * roll,
                body_inset: 0.0,
                near_ok: true,
            },
        );
        body.misc[2] = 1;
        if border_alpha > 0.003 {
            body.border = color4(ctx.palette.primary, border_alpha);
            body.params[1] = border_width;
        }
        self.card.filter_cache.push((body, store_idx as u32, roll));
        roll_in_cut(&mut body, roll);
        sinks.instances.push(body);
        sinks.hits.push(hit);
    }
}
