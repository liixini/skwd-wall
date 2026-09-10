use std::collections::HashSet;

use crate::domain::library::catalog::{Wallpaper, WallpaperKind};
use crate::frontend::scene::layout::Mode;
use crate::frontend::scene::{BackPanel, InstanceRaw, card_flip_phases, card_flip_shader_payload};

use super::model::{RebuildCtx, SceneCore};

fn human_size(bytes: i64) -> String {
    let val = bytes as f64;
    if val >= 1.0e9 {
        format!("{:.1} GB", val / 1.0e9)
    } else if val >= 1.0e6 {
        format!("{:.1} MB", val / 1.0e6)
    } else if val >= 1.0e3 {
        format!("{:.0} KB", val / 1.0e3)
    } else {
        format!("{bytes} B")
    }
}

fn civil_date(unix: i64) -> String {
    let days = unix / 86400;
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let yy = if m <= 2 { y + 1 } else { y };
    format!("{yy:04}-{m:02}-{d:02}")
}

fn back_title(item: &Wallpaper) -> String {
    let base = item.name.rsplit('/').next().unwrap_or(&item.name);
    base.rsplit_once('.').map_or(base, |(stem, _)| stem).to_string()
}

fn type_label(item: &Wallpaper) -> &'static str {
    match item.effective_kind() {
        WallpaperKind::Video => crate::i18n::tr("card-back-type-video"),
        WallpaperKind::We => crate::i18n::tr("card-back-type-we"),
        WallpaperKind::Static => crate::i18n::tr("card-back-type-image"),
    }
}

fn build_item_fields(item: &Wallpaper) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    if item.width > 0 && item.height > 0 {
        fields.push((
            crate::i18n::tr("card-back-field-resolution").to_string(),
            format!("{} x {}", item.width, item.height),
        ));
    }
    if item.filesize > 0 {
        fields
            .push((crate::i18n::tr("card-back-field-size").to_string(), human_size(item.filesize)));
    }
    if item.mtime > 0 {
        fields.push((
            crate::i18n::tr("card-back-field-modified").to_string(),
            civil_date(item.mtime),
        ));
    }
    if item.apply_count > 0 {
        fields.push((
            crate::i18n::tr("card-back-field-applied").to_string(),
            crate::i18n::card_back_applied_times(item.apply_count),
        ));
    }
    fields
}

impl SceneCore {
    pub fn flipped(&self) -> Option<usize> {
        self.card.flipped
    }

    #[cfg(test)]
    pub(crate) fn set_flipped_for_test(&mut self, flipped: Option<usize>) {
        self.card.flipped = flipped;
    }

    pub fn flip_open(&self) -> bool {
        self.card.flipped.is_some() && (self.card.flip.target > 0.5 || self.card.det_target > 0.5)
    }

    pub fn close_flip(&mut self) {
        if self.card.flipped.is_some() {
            if matches!(self.mode, Mode::Slices | Mode::Sandy)
                && !self.card.flip_shader_enabled
                && !self.card.flip_back_enabled
            {
                self.card.flip.snap(0.0);
                self.card.flipped = None;
            } else {
                self.card.flip.retarget(0.0);
            }
            self.card.det_target = 0.0;
            self.card.tag_open.snap(0.0);
            self.motion.needs_frame = true;
        }
    }

    pub fn set_tag_editing(&mut self, open: bool) {
        self.card.tag_open.retarget(if open { 1.0 } else { 0.0 });
        self.motion.needs_frame = true;
    }

    pub fn pop_tag_chip(&mut self, idx: usize) {
        self.card.pop_idx = idx as i32;
        self.card.chip_pop.snap(0.0);
        self.card.chip_pop.retarget(1.0);
        self.motion.needs_frame = true;
    }

    pub fn open_detail(&mut self, idx: usize, src: [f32; 4]) {
        if self.card.flipped != Some(idx) {
            self.card.det_p = 0.0;
        }
        self.card.flipped = Some(idx);
        self.card.det_src = src;
        self.card.det_target = 1.0;
        self.card.fav_snap = true;
        self.motion.needs_frame = true;
    }

    fn drive_fav(&mut self, is_fav: bool) -> f32 {
        self.card.fav_target = if is_fav { 1.0 } else { 0.0 };
        if self.card.fav_snap {
            self.card.fav_fill = self.card.fav_target;
            self.card.fav_snap = false;
        }
        self.card.fav_fill
    }

    pub(super) fn make_back_panel(
        &mut self,
        ctx: &RebuildCtx<'_>,
        store_idx: usize,
        cx: f32,
        cy: f32,
        hw: f32,
        hh: f32,
        skew: f32,
        edge_tilt: f32,
        radii: [f32; 4],
        progress: f32,
    ) -> BackPanel {
        let item = &ctx.catalog.items[store_idx];
        let fields = build_item_fields(item);
        let is_fav = ctx.catalog.is_favourite(item);
        let fav_fill = self.drive_fav(is_fav);
        BackPanel {
            cx,
            cy,
            hw,
            hh,
            skew,
            edge_tilt,
            radii,
            progress,
            coordinated_flip: matches!(self.mode, Mode::Slices | Mode::Sandy),
            embedded: self.mode == Mode::Slices,
            animate_flip_shader: self.card.flip_shader_enabled,
            animate_flip_back: self.card.flip_back_enabled,
            title: back_title(item),
            kind_label: type_label(item).to_string(),
            fields,
            tags: ctx.catalog.tags_for(item).to_vec(),
            favourite: is_fav,
            fav_fill,
            add_open: self.card.tag_open.x.clamp(0.0, 1.0),
            chip_pop: self.card.chip_pop.x.clamp(0.0, 1.0),
            pop_idx: self.card.pop_idx,
            static_img: item.kind == WallpaperKind::Static,
            overview_available: crate::infrastructure::runtime::is_niri(),
            scene_properties: item.effective_kind() == WallpaperKind::We && !item.we_id.is_empty(),
            reset_thumbnail: item.kind == WallpaperKind::We && !item.we_id.is_empty(),
        }
    }

    pub fn set_favourite(&mut self, fav: bool) {
        self.card.fav_target = if fav { 1.0 } else { 0.0 };
        self.motion.needs_frame = true;
    }

    pub fn toggle_flip(&mut self, idx: usize) {
        if self.card.flipped == Some(idx) && self.card.flip.target > 0.5 {
            self.close_flip();
        } else {
            if self.card.flipped != Some(idx) {
                self.card.flipped = Some(idx);
                self.card.flip.snap(0.0);
            }
            if !self.card.flip_shader_enabled && !self.card.flip_back_enabled {
                self.card.flip.snap(1.0);
            } else {
                self.card.flip.retarget(1.0);
            }
            self.card.fav_snap = true;
        }
        self.motion.needs_frame = true;
    }

    pub(super) fn rebuild_flip_overlay(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        instances: &mut Vec<InstanceRaw>,
        wanted: &mut HashSet<usize>,
    ) {
        if self.mode == Mode::Slices {
            return;
        }
        let Some(fi) = self.card.flipped else { return };
        let prog = self.card.det_p.clamp(0.0, 1.0);
        if prog <= 0.001 {
            return;
        }
        let Some(&si) = ctx.filtered.get(fi) else { return };
        let store_idx = si as usize;
        let (vw, vh) = self.viewport;
        let ease = 1.0 - (1.0 - prog).powi(2);
        let reveal = ((prog - 0.42) / 0.58).clamp(0.0, 1.0);
        let thw = (vw * 0.455).min((vw - 72.0).max(320.0) * 0.5);
        let thh = (vh * 0.43).min((vh - 64.0).max(240.0) * 0.5);
        let src = self.card.det_src;
        let cx = src[0] + (vw * 0.5 - src[0]) * ease;
        let cy = src[1] + (vh * 0.5 - src[1]) * ease;
        let hw = src[2] + (thw - src[2]) * ease;
        let hh = src[3] + (thh - src[3]) * ease;
        let radii = [0.0; 4];
        instances.push(InstanceRaw {
            rect: [vw * 0.5, vh * 0.5, vw * 0.5, vh * 0.5],
            fill: [0.0, 0.0, 0.0, 0.72 * ease],
            params: [0.0, 0.0, 1.0, 0.0],
            misc: [0, 0, 0, 4],
            ..Default::default()
        });
        instances.push(InstanceRaw {
            rect: [cx, cy + 14.0 * ease, hw + 8.0 * ease, hh + 8.0 * ease],
            radii: [0.0; 4],
            fill: [0.0, 0.0, 0.0, 0.45 * ease],
            params: [0.0, 0.0, ease, 0.0],
            misc: [0, 0, 0, 0],
            ..Default::default()
        });
        let mut body =
            self.body_instance(ctx, wanted, store_idx, [cx, cy, hw, hh], radii, 0.0, 0.0, true);
        body.params[2] = (prog * 8.0).min(1.0);
        let phases =
            card_flip_phases(prog, self.card.flip_shader_enabled, self.card.flip_back_enabled);
        body.flip = card_flip_shader_payload(phases.shader, fi, self.card.flip_effect);
        body.tint = [0.0, 0.0, 0.0, 0.20 * reveal];
        instances.push(body);
        let progress = 0.55 + reveal * 0.45;
        self.card.pending_back = Some(self.make_back_panel(
            ctx,
            store_idx,
            vw * 0.5,
            vh * 0.5,
            thw,
            thh,
            0.0,
            0.0,
            radii,
            progress,
        ));
    }
}
