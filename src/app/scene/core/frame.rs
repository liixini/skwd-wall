use iced::{Point, Rectangle};

use crate::contracts::rendering::INSTANCE_FLAGS;
use crate::frontend::scene::layout::{GridParams, Hit, Mode};
use crate::frontend::scene::{BackPanel, Chrome, InstanceRaw, TransSnap};
use crate::rendering::scene::PROJECTED;

use super::model::SceneCore;

const MIN_FRAME_FRACTION: f32 = 0.05;

#[derive(Clone, Copy)]
pub(super) struct Placement {
    dx: f32,
    dy: f32,
}

impl Placement {
    fn between(source: Rectangle, target: Rectangle) -> Self {
        Self {
            dx: target.center_x() - source.center_x(),
            dy: target.center_y() - source.center_y(),
        }
    }

    pub(super) fn point(self, x: f32, y: f32) -> (f32, f32) {
        (x + self.dx, y + self.dy)
    }

    fn unpoint(self, x: f32, y: f32) -> (f32, f32) {
        (x - self.dx, y - self.dy)
    }

    pub(super) fn instances(self, instances: &mut [InstanceRaw]) {
        for inst in instances {
            (inst.rect[0], inst.rect[1]) = self.point(inst.rect[0], inst.rect[1]);
            if inst.misc[INSTANCE_FLAGS] & PROJECTED != 0 {
                for quad in [&mut inst.quad_a, &mut inst.quad_b] {
                    for pair in quad.chunks_exact_mut(2) {
                        (pair[0], pair[1]) = self.point(pair[0], pair[1]);
                    }
                }
            }
        }
    }

    pub(super) fn hits(self, hits: &mut [Hit]) {
        for hit in hits {
            (hit.cx, hit.cy) = self.point(hit.cx, hit.cy);
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
        }
    }

    pub(super) fn back(self, back: &mut BackPanel) {
        (back.cx, back.cy) = self.point(back.cx, back.cy);
    }

    pub(super) fn transition(self, trans: &mut TransSnap) {
        (trans.origin[0], trans.origin[1]) = self.point(trans.origin[0], trans.origin[1]);
    }
}

pub(super) fn from_edges([left, top, right, bottom]: [f32; 4]) -> Rectangle {
    Rectangle { x: left, y: top, width: right - left, height: bottom - top }
}

pub(super) fn edges(rect: Rectangle) -> [f32; 4] {
    [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height]
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
        let target = self.placed_frame(source, self.picker_frame_bounds()?);
        Some(Placement::between(source, target))
    }

    pub(crate) fn frame_sizes_picker(&self) -> bool {
        self.mode == Mode::Grid
    }

    fn placed_frame(&self, natural: Rectangle, area: Rectangle) -> Rectangle {
        if self.frame_sizes_picker() {
            return area;
        }
        let Point { x, y } = area.center();
        let (vw, vh) = self.viewport;
        Rectangle {
            x: (x - natural.width * 0.5).clamp(0.0, (vw - natural.width).max(0.0)),
            y: (y - natural.height * 0.5).clamp(0.0, (vh - natural.height).max(0.0)),
            ..natural
        }
    }

    pub(super) fn unplace_pointer(&self, x: f32, y: f32) -> (f32, f32) {
        self.picker_placement().map_or((x, y), |placement| placement.unpoint(x, y))
    }

    pub(super) fn unplace_card(&self, rect: [f32; 4]) -> [f32; 4] {
        let Some(placement) = self.picker_placement() else { return rect };
        let [cx, cy, width, height] = rect;
        let (x, y) = placement.unpoint(cx, cy);
        [x, y, width, height]
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
        let Some([x, y, width, height]) = self.picker_frame else {
            return self.natural_frame;
        };
        let width = width.clamp(MIN_FRAME_FRACTION, 1.0);
        let height = height.clamp(MIN_FRAME_FRACTION, 1.0);
        let area = Rectangle {
            x: x.clamp(0.0, 1.0 - width) * self.viewport.0,
            y: y.clamp(0.0, 1.0 - height) * self.viewport.1,
            width: width * self.viewport.0,
            height: height * self.viewport.1,
        };
        Some(self.natural_frame.map_or(area, |natural| self.placed_frame(natural, area)))
    }

    pub(super) fn frame_placement(
        &mut self,
        natural: Option<Rectangle>,
    ) -> Option<(Placement, Rectangle)> {
        self.natural_frame = natural;
        let target = self.picker_frame_bounds()?;
        Some((self.picker_placement()?, target))
    }

    pub(super) fn screen_frame(&self) -> Rectangle {
        Rectangle { x: 0.0, y: 0.0, width: self.viewport.0, height: self.viewport.1 }
    }
}
