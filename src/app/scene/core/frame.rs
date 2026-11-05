use iced::{Point, Rectangle, Size};

use crate::contracts::rendering::INSTANCE_FLAGS;
use crate::frontend::scene::layout::{GridParams, Hit};
use crate::frontend::scene::{BackPanel, Chrome, InstanceRaw, SandySnap, TransSnap};
use crate::rendering::scene::PROJECTED;

use super::model::SceneCore;

const MIN_FRAME_FRACTION: f32 = 0.05;

#[derive(Clone, Copy)]
pub(super) struct Placement {
    scale: f32,
    dx: f32,
    dy: f32,
}

impl Placement {
    fn new(source: Rectangle, target: Rectangle) -> Self {
        let scale = (target.width / source.width).min(target.height / source.height);
        let dx = target.center_x() - source.center_x() * scale;
        let dy = target.center_y() - source.center_y() * scale;
        Self { scale, dx, dy }
    }

    fn translate(source: Rectangle, target: Rectangle) -> Self {
        Self {
            scale: 1.0,
            dx: target.center_x() - source.center_x(),
            dy: target.center_y() - source.center_y(),
        }
    }

    pub(super) fn point(self, x: f32, y: f32) -> (f32, f32) {
        (x * self.scale + self.dx, y * self.scale + self.dy)
    }

    fn unpoint(self, x: f32, y: f32) -> (f32, f32) {
        ((x - self.dx) / self.scale, (y - self.dy) / self.scale)
    }

    pub(super) fn instances(self, instances: &mut [InstanceRaw]) {
        for inst in instances {
            (inst.rect[0], inst.rect[1]) = self.point(inst.rect[0], inst.rect[1]);
            for value in &mut inst.rect[2..] {
                *value *= self.scale;
            }
            for value in &mut inst.radii {
                *value *= self.scale;
            }
            inst.params[0] *= self.scale;
            inst.params[1] *= self.scale;
            inst.shape[0] *= self.scale;
            if inst.misc[INSTANCE_FLAGS] & PROJECTED != 0 {
                for quad in [&mut inst.quad_a, &mut inst.quad_b] {
                    for pair in quad.chunks_exact_mut(2) {
                        (pair[0], pair[1]) = self.point(pair[0], pair[1]);
                    }
                }
                for value in &mut inst.quad_l {
                    *value *= self.scale;
                }
            }
        }
    }

    pub(super) fn hits(self, hits: &mut [Hit]) {
        for hit in hits {
            (hit.cx, hit.cy) = self.point(hit.cx, hit.cy);
            hit.hw *= self.scale;
            hit.hh *= self.scale;
            hit.skew *= self.scale;
            hit.edge_tilt *= self.scale;
            if let Some(quad) = &mut hit.quad {
                for point in quad {
                    (point[0], point[1]) = self.point(point[0], point[1]);
                }
            }
        }
    }

    pub(super) fn chrome(self, chrome: &mut [Chrome]) {
        for item in chrome {
            (item.cx, item.cy) = self.point(item.cx, item.cy);
            item.hw *= self.scale;
            item.hh *= self.scale;
            item.skew *= self.scale;
            item.edge_tilt *= self.scale;
            item.radius *= self.scale;
        }
    }

    pub(super) fn back(self, back: &mut BackPanel) {
        (back.cx, back.cy) = self.point(back.cx, back.cy);
        back.hw *= self.scale;
        back.hh *= self.scale;
        back.skew *= self.scale;
        back.edge_tilt *= self.scale;
        back.corners = back.corners.scaled(self.scale);
    }

    pub(super) fn sandy(self, snap: &mut SandySnap) {
        (snap.center[0], snap.center[1]) = self.point(snap.center[0], snap.center[1]);
        snap.hero[0] *= self.scale;
        snap.hero[1] *= self.scale;
    }

    pub(super) fn transition(self, trans: &mut TransSnap) {
        (trans.origin[0], trans.origin[1]) = self.point(trans.origin[0], trans.origin[1]);
    }
}

impl SceneCore {
    pub(crate) fn grid_capacity(&self) -> (usize, usize) {
        let params = self.effective_grid_params();
        (params.cols, params.rows)
    }

    pub(super) fn effective_grid_params(&self) -> GridParams {
        let mut gp = self.gp;
        if self.picker_frame.is_some()
            && let Some(target) = self.picker_frame_bounds()
        {
            gp.cols = gp.columns_in(target.width);
            gp.rows = gp.rows_in(target.height);
        }
        gp
    }

    pub(super) fn picker_placement(&self) -> Option<Placement> {
        self.picker_frame?;
        let source = self.natural_frame?;
        let target = self.picker_frame_bounds()?;
        (source.width > 0.0 && source.height > 0.0).then(|| {
            if self.mode == crate::frontend::scene::layout::Mode::Grid {
                Placement::translate(source, target)
            } else {
                Placement::new(source, target)
            }
        })
    }

    pub(super) fn unplace_pointer(&self, x: f32, y: f32) -> (f32, f32) {
        self.picker_placement().map_or((x, y), |placement| placement.unpoint(x, y))
    }

    pub(super) fn frame_pointer(&self, x: f32, y: f32) -> (f32, f32) {
        let Some(frame) = self.picker_frame.and_then(|_| self.picker_frame_bounds()) else {
            return (x, y);
        };
        (
            (x - frame.x) * self.viewport.0 / frame.width.max(1.0),
            (y - frame.y) * self.viewport.1 / frame.height.max(1.0),
        )
    }

    pub(super) fn unplace_card(&self, rect: [f32; 4]) -> [f32; 4] {
        let Some(placement) = self.picker_placement() else { return rect };
        let [cx, cy, width, height] = rect;
        let (x, y) = placement.unpoint(cx, cy);
        [x, y, width / placement.scale, height / placement.scale]
    }

    pub fn set_picker_frame(&mut self, frame: Option<[f32; 4]>) {
        if self.picker_frame != frame {
            self.picker_frame = frame;
            self.motion.needs_frame = true;
        }
    }

    pub fn show_frame(&mut self, shown: Option<Rectangle>) {
        if self.frame_shown != shown {
            self.frame_shown = shown;
            self.motion.needs_frame = true;
        }
    }

    pub fn picker_frame_bounds(&self) -> Option<Rectangle> {
        self.frame_shown.or_else(|| self.configured_frame_bounds())
    }

    pub(crate) fn configured_frame_bounds(&self) -> Option<Rectangle> {
        self.picker_frame
            .map(|frame| {
                let [x, y, width, height] = frame;
                let width = width.clamp(MIN_FRAME_FRACTION, 1.0);
                let height = height.clamp(MIN_FRAME_FRACTION, 1.0);
                let x = x.clamp(0.0, 1.0 - width);
                let y = y.clamp(0.0, 1.0 - height);
                Rectangle {
                    x: x * self.viewport.0,
                    y: y * self.viewport.1,
                    width: width * self.viewport.0,
                    height: height * self.viewport.1,
                }
            })
            .or(self.natural_frame)
    }

    pub(super) fn frame_placement(
        &mut self,
        hits: &[Hit],
        clip: Option<[f32; 4]>,
    ) -> Option<(Placement, Rectangle)> {
        let natural = clip.map_or_else(
            || hit_bounds(hits),
            |clip| {
                let [left, top, right, bottom] = clip;
                Some(Rectangle { x: left, y: top, width: right - left, height: bottom - top })
            },
        );
        self.natural_frame = natural;
        let target = self.picker_frame_bounds()?;
        Some((self.picker_placement()?, target))
    }
}

fn hit_bounds(hits: &[Hit]) -> Option<Rectangle> {
    let mut left = f32::INFINITY;
    let mut top = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    let mut bottom = f32::NEG_INFINITY;
    for hit in hits {
        if let Some(quad) = hit.quad {
            for [x, y] in quad {
                left = left.min(x);
                top = top.min(y);
                right = right.max(x);
                bottom = bottom.max(y);
            }
        } else {
            let width = hit.hw + hit.skew.abs() + hit.edge_tilt.abs();
            left = left.min(hit.cx - width);
            top = top.min(hit.cy - hit.hh);
            right = right.max(hit.cx + width);
            bottom = bottom.max(hit.cy + hit.hh);
        }
    }
    left.is_finite()
        .then(|| Rectangle::new(Point::new(left, top), Size::new(right - left, bottom - top)))
}
