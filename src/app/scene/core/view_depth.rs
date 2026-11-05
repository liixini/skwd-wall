use super::model::{RebuildCtx, RebuildSinks, SceneCore};

impl SceneCore {
    pub(super) fn rebuild_depth(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) {
        let count = ctx.filtered.len();
        if count == 0 {
            return;
        }
        self.current = self.current.min(count - 1);
        self.camera.set_zeta(1.0);
        if self.layout_camera_anchor {
            self.camera.snap(self.current as f32);
            self.layout_camera_anchor = false;
        } else {
            self.camera.retarget(self.current as f32);
        }
        let center = self.camera.x.clamp(0.0, count.saturating_sub(1) as f32);
        let falloff = self.xp.depth.falloff;
        let spacing = self.xp.depth.spacing;
        let radius = (self.sp.visible_count.clamp(3, 21) as f32 - 1.0) * 0.5;
        let parallax_extent = depth_offset(radius + 1.0, falloff);
        let height = self.sp.slice_h.max(1.0);
        let width = self.xp.depth.width;
        let natural_span = depth_offset(radius, falloff) * spacing + width * 0.5;
        let fit = ((self.avail_w() * 0.5 - 36.0).max(1.0) / natural_span)
            .min(self.viewport.1 * 0.7 / height)
            .min(1.0);
        let width = width * fit;
        let height = height * fit;
        let cx = self.center_x();
        let cy = self.viewport.1 * 0.5;
        let low = ((center - radius - 1.0).floor().max(0.0) as usize).min(count - 1);
        let high = ((center + radius + 1.0).ceil() as usize).min(count - 1);
        self.vis_lo = low;
        self.vis_hi = high;
        let mut visible: Vec<usize> = (low..=high).collect();
        visible.sort_by(|&a, &b| (b as f32 - center).abs().total_cmp(&(a as f32 - center).abs()));
        self.card.filter_cache.clear();
        for idx in visible {
            let n = idx as f32 - center;
            let scale = 1.0 / (1.0 + falloff * n.abs());
            let opacity =
                crate::frontend::animation::smoothstep((radius + 1.0 - n.abs()).clamp(0.0, 1.0))
                    * entrance;
            if opacity <= 0.0001 {
                continue;
            }
            let offset = depth_offset(n, falloff);
            let item_cx = cx + offset * spacing * fit;
            let item_cy = cy;
            let sp = crate::frontend::scene::layout::SliceParams {
                slice_h: height * scale,
                skew: (self.sp.skew * fit).clamp(-width * 0.3, width * 0.3) * scale,
                edge_tilt: 0.0,
                corners: self.sp.corners.scaled(fit * scale),
                wobble: false,
                ..self.sp
            };
            self.slice_card(
                ctx,
                sinks,
                idx,
                width * scale,
                item_cx,
                item_cy,
                opacity,
                self.avail_w() * 0.5,
                0.0,
                sp,
                Some(offset / parallax_extent),
            );
        }
        self.push_flip_old(sinks.instances, sinks.wanted);
    }
}

fn depth_offset(distance: f32, falloff: f32) -> f32 {
    if falloff < 0.0001 {
        distance
    } else {
        distance.signum() * (1.0 + falloff * distance.abs()).ln() / falloff
    }
}
