use paper_control::Face;

use super::geometry::{Rect, fit};
use super::spot::{shortest, tallest, width_at};

pub const SNAP: f32 = 10.0;
pub const DETENTS: f32 = 10.0;
pub const CATCH: f32 = 0.012;
const SPLITS: [f32; 3] = [0.5, 1.0 / 3.0, 2.0 / 3.0];
const THIRDS: [f32; 2] = [1.0 / 3.0, 2.0 / 3.0];
const SIDES: [f32; 3] = [-1.0, 0.0, 1.0];
pub const CORNERS: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Guide {
    pub vertical: bool,
    pub at: f32,
    pub with: Option<Rect>,
}

impl Guide {
    pub fn key(&self) -> (bool, i32) {
        (self.vertical, self.at.round() as i32)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapped {
    pub centre: [f32; 2],
    pub guides: Vec<Guide>,
}

pub fn snap(size: [f32; 2], wanted: [f32; 2], others: &[Rect], output: [f32; 2]) -> Snapped {
    let mut centre = wanted;
    let mut guides = Vec::new();
    for axis in 0..2 {
        let stage = SPLITS.iter().map(|split| (split * output[axis], None));
        let edges = others.iter().flat_map(|other| {
            let (start, length) = if axis == 0 { (other.x, other.w) } else { (other.y, other.h) };
            SIDES.iter().map(move |side| (start + length * (side + 1.0) / 2.0, Some(*other)))
        });
        let half = size[axis] / 2.0;
        let best = stage
            .chain(edges)
            .flat_map(|(line, with)| {
                SIDES.iter().map(move |side| (line - (wanted[axis] + side * half), line, with))
            })
            .filter(|(gap, _, _)| gap.abs() <= SNAP)
            .min_by(|one, other| one.0.abs().total_cmp(&other.0.abs()));
        if let Some((gap, at, with)) = best {
            centre[axis] += gap;
            guides.push(Guide { vertical: axis == 0, at, with });
        }
        centre[axis] = fit(centre[axis], half, output[axis]);
    }
    Snapped { centre, guides }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stretched {
    pub rect: Rect,
    pub guides: Vec<Guide>,
}

pub fn stretch(
    grip: [[f32; 2]; 2],
    at: [f32; 2],
    least: [f32; 2],
    others: &[Rect],
    output: [f32; 2],
) -> Stretched {
    let [anchor, sign] = grip;
    let mut edge = at;
    let mut guides = Vec::new();
    for axis in 0..2 {
        let stage = SPLITS.iter().map(|split| (split * output[axis], None));
        let edges = others.iter().flat_map(|other| {
            let (start, length) = if axis == 0 { (other.x, other.w) } else { (other.y, other.h) };
            SIDES.iter().map(move |side| (start + length * (side + 1.0) / 2.0, Some(*other)))
        });
        let best = stage
            .chain(edges)
            .map(|(line, with)| (line - at[axis], line, with))
            .filter(|(gap, _, _)| gap.abs() <= SNAP)
            .min_by(|one, other| one.0.abs().total_cmp(&other.0.abs()));
        let (low, high) = if sign[axis] > 0.0 {
            (anchor[axis] + least[axis], output[axis])
        } else {
            (0.0, anchor[axis] - least[axis])
        };
        let wanted = best.map_or(at[axis], |(_, line, _)| line);
        edge[axis] = wanted.clamp(low, high.max(low));
        if let Some((_, line, with)) = best.filter(|(_, line, _)| *line == edge[axis]) {
            guides.push(Guide { vertical: axis == 0, at: line, with });
        }
    }
    let [left, right] = [anchor[0].min(edge[0]), anchor[0].max(edge[0])];
    let [top, bottom] = [anchor[1].min(edge[1]), anchor[1].max(edge[1])];
    Stretched { rect: Rect::new(left, top, right - left, bottom - top), guides }
}

pub fn anchor(rect: Rect, frame: Rect) -> [usize; 2] {
    let third = |centre: f32, start: f32, length: f32| {
        let share = (centre - start) / length.max(1.0);
        THIRDS.iter().take_while(|edge| share >= **edge).count()
    };
    let [x, y] = rect.centre();
    [third(x, frame.x, frame.w), third(y, frame.y, frame.h)]
}

pub fn cells(span: f32, cell: f32, gap: f32, most: usize) -> usize {
    (((span + gap) / (cell + gap).max(1.0)).round() as usize).clamp(1, most)
}

pub fn detent(share: f32, up: bool, range: [f32; 2]) -> Option<f32> {
    let scaled = share * DETENTS;
    let step = if up { (scaled + CATCH).floor() + 1.0 } else { (scaled - CATCH).ceil() - 1.0 };
    let next = (step / DETENTS).clamp(range[0], range[1]);
    ((next - share).abs() >= CATCH / DETENTS).then_some(next)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grip {
    pub anchor: [f32; 2],
    pub sign: [f32; 2],
    direction: [f32; 2],
    diagonal: f32,
    along: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sized {
    pub share: f32,
    pub snapped: bool,
}

impl Grip {
    pub fn new(bounds: Rect, corner: usize, press: [f32; 2]) -> Self {
        let sign = CORNERS[corner];
        let [cx, cy] = bounds.centre();
        let anchor = [cx - sign[0] * bounds.w / 2.0, cy - sign[1] * bounds.h / 2.0];
        let diagonal = bounds.w.hypot(bounds.h);
        let direction = [sign[0] * bounds.w / diagonal, sign[1] * bounds.h / diagonal];
        let mut grip = Self { anchor, sign, direction, diagonal, along: 0.0, height: bounds.h };
        grip.along = grip.project(press);
        grip
    }

    fn project(&self, at: [f32; 2]) -> f32 {
        (at[0] - self.anchor[0]) * self.direction[0] + (at[1] - self.anchor[1]) * self.direction[1]
    }

    pub fn most(&self, face: Face, output: [f32; 2]) -> f32 {
        let room_y = if self.sign[1] > 0.0 { output[1] - self.anchor[1] } else { self.anchor[1] };
        let room_x = if self.sign[0] > 0.0 { output[0] - self.anchor[0] } else { self.anchor[0] };
        let wide = room_x / width_at(face, output[1]) * output[1];
        tallest(face, output).min(room_y / output[1]).min(wide / output[1]).max(shortest())
    }

    pub fn resize(&self, at: [f32; 2], face: Face, output: [f32; 2]) -> Sized {
        let ratio = (self.project(at) - (self.along - self.diagonal)) / self.diagonal;
        let most = self.most(face, output);
        let share = (self.height * ratio / output[1]).clamp(shortest(), most);
        let step = (share * DETENTS).round() / DETENTS;
        let snapped = step > 0.0 && step <= most && (share - step).abs() < CATCH;
        Sized { share: if snapped { step } else { share }, snapped }
    }

    pub fn centre(&self, face: Face, height: f32) -> [f32; 2] {
        let width = width_at(face, height);
        [self.anchor[0] + self.sign[0] * width / 2.0, self.anchor[1] + self.sign[1] * height / 2.0]
    }
}

#[cfg(test)]
#[path = "snap_tests.rs"]
mod tests;
