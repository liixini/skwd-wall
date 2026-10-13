use std::collections::HashSet;
use std::sync::Arc;

use crate::contracts::rendering::RendererSnapshot;
use crate::domain::library::catalog::WallpaperKind;
use crate::frontend::scene::layout::{self, Hit, Mode};
use crate::frontend::scene::{
    Chrome, InstanceRaw, RenderSnapshot, TransSnap, apply_entrance_motion,
};
use crate::infrastructure::preview::is_webp;
use crate::rendering::scene::atlas::AtlasMap;

use super::super::widget::shimmer_eligible;
use super::atlas_helpers::{atlas_debug_enabled, audit_atlas, enqueue_far, enqueue_near};
use super::layout_helpers::{
    CardSpec, STORM_CUTOFF, THUMB_H, THUMB_W, chrome_kind, near_qualifies,
};
use super::model::{RebuildCtx, RebuildSinks, SceneCore};

impl SceneCore {
    pub(super) fn rebuild(&mut self, mut ctx: RebuildCtx<'_>) {
        let (vw, vh) = self.viewport;
        if vw <= 0.0 || vh <= 0.0 {
            return;
        }
        let far_layers = ctx.atlas.as_ref().map_or(1, |atlas| atlas.far_layers);
        if let Some(atlas) = ctx.atlas.as_mut() {
            atlas.near.begin_frame();
            atlas.far.begin_frame();
            for (_, store_idx, _) in &self.card.filter_old {
                atlas.near.pin(*store_idx as usize);
                atlas.far.pin(*store_idx as usize);
            }
        }
        self.card.pending_back = None;
        let mut instances = Vec::with_capacity(96);
        let mut hits = Vec::with_capacity(48);
        let mut chrome = Vec::with_capacity(48);
        let mut wanted = HashSet::new();
        let entrance = self.motion.entrance.x.clamp(0.0, 1.0);
        let mut sinks = RebuildSinks {
            instances: &mut instances,
            hits: &mut hits,
            chrome: &mut chrome,
            wanted: &mut wanted,
        };

        let clip = match self.mode {
            Mode::Slices => {
                self.rebuild_slices(&mut ctx, &mut sinks, entrance);
                None
            }
            Mode::Depth => {
                self.rebuild_depth(&mut ctx, &mut sinks, entrance);
                None
            }
            Mode::Collection => {
                self.rebuild_collection(&mut ctx, &mut sinks, entrance);
                None
            }
            Mode::Grid => self.rebuild_grid(&mut ctx, &mut sinks, entrance),
            Mode::Hex => {
                self.rebuild_hex(&mut ctx, &mut sinks, entrance);
                None
            }
            Mode::Sandy => {
                self.rebuild_sandy(&mut ctx, &mut sinks, entrance);
                None
            }
            Mode::Hand => {
                self.rebuild_hand(&mut ctx, &mut sinks, entrance);
                None
            }
        };
        self.rebuild_flip_overlay(&mut ctx, sinks.instances, sinks.wanted);
        let prefetch_n = ctx.filtered.len();
        self.prefetch_ring(&mut ctx, sinks.wanted, prefetch_n);
        let near_layers = ctx.atlas.as_ref().map_or(0, |atlas| atlas.near.layer_extent());
        ctx.pool.set_wanted(wanted);

        hits.reverse();
        let accent = {
            let col = ctx.palette.primary;
            [col.r, col.g, col.b, 1.0]
        };
        let transition = self.motion.transition.as_ref().map(|trans| TransSnap {
            old: trans.old.clone(),
            progress: trans.progress.x,
            kind: trans.kind,
            origin: trans.origin,
            accent,
        });
        apply_entrance_motion(&mut instances, self.motion.launch_anim, entrance, vw, vh);
        if atlas_debug_enabled() {
            audit_atlas(
                &instances,
                ctx.atlas.as_ref(),
                self.card.filter_old.len(),
                self.scroll_vel,
            );
        }
        let placeholders = instances.iter().any(|inst| inst.misc[3] & 2 != 0);
        let now_t = (crate::infrastructure::observability::elapsed_ms() as f32) / 1000.0;
        let renderer = Arc::new(RendererSnapshot {
            instances,
            far_layers,
            near_layers,
            transition,
            clip,
            sandy: self.sandy.snap.take(),
            time: now_t,
            vis: self.motion.visibility.value().clamp(0.0, 1.0),
        });
        self.render = Arc::new(RenderSnapshot {
            renderer,
            hits,
            chrome,
            placeholders,
            back: self.card.pending_back.take(),
            overlay: !matches!(self.mode, Mode::Slices | Mode::Depth) && self.card.det_p > 0.05,
        });
    }

    pub(super) fn body_instance(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        wanted: &mut HashSet<usize>,
        store_idx: usize,
        rect: [f32; 4],
        radii: [f32; 4],
        skew: f32,
        edge_tilt: f32,
        near_ok: bool,
    ) -> InstanceRaw {
        wanted.insert(store_idx);
        let item = &ctx.catalog.items[store_idx];
        let storm = self.scroll_vel > STORM_CUTOFF;
        let is_cur =
            ctx.filtered.get(self.current).is_some_and(|&store| store as usize == store_idx);
        let is_hover = self
            .hover
            .and_then(|index| ctx.filtered.get(index))
            .is_some_and(|&store| store as usize == store_idx);
        let near_candidate =
            crate::contracts::preview::compressed_thumbnails() || is_cur || is_hover;
        let pv = ctx.palette.surface_variant;
        let mut inst = InstanceRaw {
            rect,
            radii,
            fill: [pv.r, pv.g, pv.b, 0.8],
            params: [skew, 0.0, 1.0, 0.0],
            shape: [edge_tilt, 0.0, 0.0, 0.0],
            ..Default::default()
        };
        let preview_idx = self.preview_slot_idx();
        let Some(atlas) = ctx.atlas.as_mut() else {
            return inst;
        };
        if near_candidate {
            atlas.near.pin(store_idx);
            if near_ok
                && near_qualifies(is_cur, storm, rect[2] * 2.0, rect[3] * 2.0)
                && !atlas.near.is_known(store_idx)
                && !atlas.near_failed.contains(&store_idx)
                && is_webp(&item.thumb)
            {
                enqueue_near(ctx.pool, atlas, item, store_idx, true, self.direct_thumbnails);
            }
        }
        atlas.far.pin(store_idx);
        if !atlas.far.is_known(store_idx)
            && !atlas.failed.contains(&store_idx)
            && is_webp(&item.thumb)
        {
            enqueue_far(ctx.pool, atlas, item, store_idx, true, self.direct_thumbnails);
        }
        let (mode, uv_o, uv_s, layer) = if preview_idx == Some(store_idx) {
            (3u32, [0.0, 0.0], [1.0, 1.0], 0)
        } else if let Some(id) = near_candidate.then(|| atlas.near.ready(store_idx)).flatten() {
            let (off, scale, layer) = AtlasMap::near_uv(id);
            (1u32, off, scale, layer)
        } else if let Some(id) = atlas.far.ready(store_idx) {
            let (off, scale, layer) = atlas.far_uv(id);
            (2u32, off, scale, layer)
        } else {
            (0u32, [0.0; 2], [1.0; 2], 0)
        };
        if shimmer_eligible(
            mode,
            self.mode,
            is_webp(&item.thumb),
            atlas.failed.contains(&store_idx),
            atlas.near_failed.contains(&store_idx),
        ) {
            inst.misc[3] |= 2;
        }
        if mode > 0 {
            let fade = self.card.fades.get(&store_idx).map_or(1.0, |spr| spr.x);
            let (co, cs) = layout::cover_crop(rect[2] * 2.0, rect[3] * 2.0, THUMB_W, THUMB_H);
            inst.misc = [mode, layer, 0, 0];
            inst.uv = [uv_o[0], uv_o[1], uv_s[0], uv_s[1]];
            inst.crop = [co[0], co[1], cs[0], cs[1]];
            inst.params[3] = fade.clamp(0.0, 1.0);
        }
        inst
    }

    pub(super) fn place_card(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        wanted: &mut HashSet<usize>,
        chrome: &mut Vec<Chrome>,
        spec: &CardSpec,
    ) -> (InstanceRaw, Hit) {
        let body_rect = [spec.cx, spec.cy, spec.hw - spec.body_inset, spec.hh - spec.body_inset];
        let mut body = self.body_instance(
            ctx,
            wanted,
            spec.store_idx,
            body_rect,
            spec.radii,
            spec.skew,
            spec.edge_tilt,
            spec.near_ok,
        );
        body.params[2] = spec.opacity;
        if spec.hex {
            body.misc[3] |= 1;
        }
        let item = &ctx.catalog.items[spec.store_idx];
        chrome.push(Chrome {
            view: spec.view,
            cx: spec.cx,
            cy: spec.cy,
            hw: spec.hw,
            hh: spec.hh,
            skew: spec.skew,
            edge_tilt: spec.edge_tilt,
            kind: chrome_kind(item.kind.as_str()),
            has_video: item.effective_kind() == WallpaperKind::Video,
            favourite: ctx.catalog.is_favourite(item),
            hidden: ctx.catalog.is_hidden(item),
            radius: spec.chrome_radius,
            opacity: spec.chrome_opacity,
        });
        let hit = Hit {
            index: spec.filtered_idx,
            cx: spec.cx,
            cy: spec.cy,
            hw: spec.hw,
            hh: spec.hh,
            skew: spec.skew,
            edge_tilt: spec.edge_tilt,
            hex: spec.hex,
            hex_shape: layout::HexShape::Hexagon,
            triangle_direction: 0,
            quad: None,
        };
        (body, hit)
    }
}
