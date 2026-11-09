use glam::Mat4;
use iced::Rectangle;

use crate::contracts::rendering::{CHAMFER, INSTANCE_FLAGS};
use crate::frontend::scene::hand::{self, Camera, PAD, Quad};
use crate::frontend::scene::layout::Mode;
use crate::rendering::scene::PROJECTED;

use super::layout_helpers::{CardSpec, vis_range};
use super::model::{RebuildCtx, RebuildSinks, SceneCore};

const FALLBACK_ASPECT: f32 = 16.0 / 9.0;

impl SceneCore {
    pub fn collection_open_for(&self, idx: usize) -> bool {
        self.collection_card == Some(idx) && self.collection_open.target > 0.5
    }

    pub fn select_collection(&mut self, idx: usize, count: usize) {
        if self.mode != Mode::Collection || count == 0 {
            return;
        }
        let idx = idx.min(count - 1);
        if self.collection_open_for(idx) {
            self.close_collection();
            return;
        }
        self.set_current(idx, count);
        if self.collection_card != Some(idx) {
            self.collection_open.snap(0.0);
        }
        self.collection_card = Some(idx);
        self.collection_open.retarget(1.0);
        self.motion.needs_frame = true;
    }

    pub fn close_collection(&mut self) -> bool {
        if self.collection_open.target <= 0.5 {
            return false;
        }
        self.collection_open.retarget(0.0);
        self.motion.needs_frame = true;
        true
    }

    pub(super) fn rebuild_collection(
        &mut self,
        ctx: &mut RebuildCtx<'_>,
        sinks: &mut RebuildSinks<'_>,
        entrance: f32,
    ) -> Option<Rectangle> {
        let count = ctx.filtered.len();
        if count == 0 {
            self.vis_lo = 0;
            self.vis_hi = 0;
            return None;
        }
        self.current = self.current.min(count - 1);
        self.camera.set_zeta(1.0);
        let radius = (self.xp.collection.count.round() - 1.0) * 0.5;
        self.set_period(count as f32, count as f32, 2.0 * (radius + 1.0), count);
        if self.layout_camera_anchor {
            self.camera.snap(self.current as f32);
            self.layout_camera_anchor = false;
        } else {
            self.aim_camera(self.current as f32, 2.0 * radius + 1.0);
        }
        let last = (count - 1) as f32;
        let center = if self.connected() {
            self.camera.x
        } else {
            self.camera.x.clamp(radius.min(last * 0.5), (last - radius).max(last * 0.5))
        };
        let (low, high) = self.slot_window(center, radius + 1.0, count);
        (self.vis_lo, self.vis_hi) = vis_range(Some((low, high)), self.current, count);
        let open = self.collection_open.x.clamp(0.0, 1.0);
        let lift = |slot: i64| {
            let idx = slot.rem_euclid(count as i64) as usize;
            if Some(idx) == self.collection_card { open * 12.0 } else { 0.0 }
        };
        let mut visible: Vec<i64> = (low..=high).collect();
        visible.sort_by(|&a, &b| {
            (a as f32 - center + lift(a)).total_cmp(&(b as f32 - center + lift(b)))
        });
        self.card.filter_cache.clear();
        for slot in visible {
            let idx = slot.rem_euclid(count as i64) as usize;
            let distance = slot as f32 - center;
            let opacity = (radius + 1.0 - distance.abs()).clamp(0.0, 1.0) * entrance;
            if opacity <= 0.001 {
                continue;
            }
            let store = ctx.filtered[idx] as usize;
            let item = &ctx.catalog.items[store];
            let aspect = if item.width > 0 && item.height > 0 {
                (item.width as f32 / item.height as f32).clamp(0.5, 2.4)
            } else {
                FALLBACK_ASPECT
            };
            let selected = Some(idx) == self.collection_card;
            let (quad, hw, hh) = collection_quad(
                self.viewport.1,
                self.avail_w(),
                self.center_x(),
                distance,
                open,
                selected,
                aspect,
                self.xp.collection,
            );
            let [cx, cy, hit_hw, hit_hh] = quad.bounds();
            let chrome_opacity = if selected && open > 0.99 { opacity } else { 0.0 };
            let corners = self.xp.collection.corners.clamped(hw.min(hh));
            let (mut body, mut hit) = self.place_card(
                ctx,
                sinks.wanted,
                sinks.chrome,
                &CardSpec {
                    filtered_idx: idx,
                    store_idx: store,
                    cx,
                    cy,
                    hw,
                    hh,
                    skew: 0.0,
                    edge_tilt: 0.0,
                    corners,
                    hex: false,
                    view: 0,
                    chrome_radius: corners.sizes[0],
                    opacity,
                    chrome_opacity,
                    body_inset: 0.0,
                    near_ok: true,
                },
            );
            let [tl, tr, br, bl] = quad.pts;
            body.misc[3] |= PROJECTED;
            body.quad_a = [tl[0], tl[1], tr[0], tr[1]];
            body.quad_b = [br[0], br[1], bl[0], bl[1]];
            body.quad_w = [tl[2], tr[2], br[2], bl[2]];
            body.quad_l = hand::card_locals(hh, PAD);
            let mut shadow = body;
            shadow.misc = [0, 0, 0, PROJECTED | (body.misc[INSTANCE_FLAGS] & CHAMFER)];
            shadow.fill = [0.0, 0.0, 0.0, 0.24];
            shadow.quad_a[1] += 5.0;
            shadow.quad_a[3] += 5.0;
            shadow.quad_b[1] += 5.0;
            shadow.quad_b[3] += 5.0;
            sinks.instances.push(shadow);
            let accent = ctx.palette.primary;
            body.border = if idx == self.current {
                [accent.r, accent.g, accent.b, 0.85 * (1.0 - open)]
            } else {
                [1.0, 1.0, 1.0, 0.16]
            };
            body.params[1] = 1.0;
            body.tint = [0.0, 0.0, 0.0, (-distance * 0.035).clamp(0.0, 0.16) * (1.0 - open)];
            hit.hw = hit_hw;
            hit.hh = hit_hh;
            hit.quad = Some(quad.pts.map(|p| [p[0], p[1]]));
            sinks.instances.push(body);
            sinks.hits.push(hit);
        }
        let [front, back] = [-radius, radius].map(|distance| {
            let (quad, _, _) = collection_quad(
                self.viewport.1,
                self.avail_w(),
                self.center_x(),
                distance,
                0.0,
                false,
                FALLBACK_ASPECT,
                self.xp.collection,
            );
            let [cx, cy, hw, hh] = quad.bounds();
            Rectangle { x: cx - hw, y: cy - hh, width: 2.0 * hw, height: 2.0 * hh }
        });
        Some(front.union(&back))
    }
}

fn collection_quad(
    vh: f32,
    available_w: f32,
    cx: f32,
    distance: f32,
    open: f32,
    selected: bool,
    aspect: f32,
    params: crate::frontend::scene::layout::CollectionParams,
) -> (Quad, f32, f32) {
    let size = (vh * params.size / 100.0).min(available_w * 0.64).max(1.0);
    let scale = 1.0 + distance * 0.045;
    let width = size * scale;
    let preview_w = (available_w * 0.76).min(vh * 0.56 * aspect);
    let (target_w, target_h, target_y, target_tilt) = if selected {
        (preview_w, preview_w / aspect, vh * 0.41, 0.0)
    } else {
        (size * 0.43, size * 0.43, vh * 0.86 + distance * size * 0.026, -55.0)
    };
    let mix = |a: f32, b: f32| a + (b - a) * open;
    let hw = mix(width, target_w) * 0.5;
    let hh = mix(width, target_h) * 0.5;
    let cy = mix(vh * 0.53 + distance * size * params.spacing / 100.0, target_y);
    let tilt = mix(-params.tilt, target_tilt).to_radians();
    let camera = Camera { d: size * 3.5, origin: [cx, cy], shift: [0.0, 0.0] };
    let matrix = Mat4::from_rotation_x(tilt);
    let quad = hand::project_quad(&camera, &matrix, &hand::card_corners(hw, hh, PAD, PAD));
    (quad, hw, hh)
}
