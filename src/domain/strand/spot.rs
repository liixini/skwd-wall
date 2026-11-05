use paper_control::{Depth, Face, WIDGET_HEIGHTS, Widget};

use super::catalogue::Kind;
use super::geometry::{Rect, fit};

const REFERENCE: [f32; 2] = [1600.0, 900.0];
const MARGIN: f32 = 48.0;
const TALLEST: f32 = 0.6;
const CLOCK_CORNER: [f32; 2] = [376.0, 76.0];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    pub id: u32,
    pub face: Face,
    pub depth: Depth,
    pub centre: [f32; 2],
    pub height: f32,
}

pub fn frame(face: Face) -> [f32; 2] {
    paper_widgets::frame(face)
}

pub fn width_at(face: Face, height: f32) -> f32 {
    let [width, tall] = frame(face);
    width * height / tall
}

pub fn shortest() -> f32 {
    *WIDGET_HEIGHTS.start()
}

pub fn tallest(face: Face, output: [f32; 2]) -> f32 {
    let [width, tall] = frame(face);
    let [screen, high] = output;
    (screen * tall / width / high).min(*WIDGET_HEIGHTS.end()).max(shortest())
}

pub fn natural_height(face: Face, output: [f32; 2]) -> f32 {
    let [width, tall] = frame(face);
    let [screen, high] = output;
    let room = screen - 2.0 * MARGIN * screen / REFERENCE[0];
    (tall * high / REFERENCE[1]).min(high * TALLEST).min(room * tall / width)
}

impl Spot {
    pub fn bounds(&self) -> Rect {
        Rect::centred(self.centre, [width_at(self.face, self.height), self.height])
    }

    pub fn kind(&self) -> Kind {
        Kind::of(self.face)
    }

    #[must_use]
    pub fn kept(self, output: [f32; 2]) -> Self {
        let size = [width_at(self.face, self.height), self.height];
        let centre =
            std::array::from_fn(|axis| fit(self.centre[axis], size[axis] / 2.0, output[axis]));
        Self { centre, ..self }
    }

    pub fn usual(id: u32, face: Face, output: [f32; 2]) -> Self {
        let height = natural_height(face, output);
        let corner = match Kind::of(face) {
            Kind::Clock => CLOCK_CORNER,
        };
        let size = [width_at(face, height), height];
        let centre = std::array::from_fn(|axis| {
            corner[axis] * output[axis] / REFERENCE[axis] + size[axis] / 2.0
        });
        Self { id, face, depth: Kind::of(face).depth(), centre, height }.kept(output)
    }

    pub fn placed(id: u32, widget: &Widget, output: [f32; 2], scale: f32) -> Option<Self> {
        let [logical, buffer] = surfaces(output, scale);
        let place =
            paper_widgets::place(logical, buffer, frame(widget.face), widget.pin, widget.height)?;
        let [x, y, w, h] = place.bounds;
        let centre = [x + w / 2.0, y + h / 2.0];
        Some(Self { id, face: widget.face, depth: widget.depth, centre, height: h })
    }

    pub fn widget(&self, output: [f32; 2], scale: f32, name: Option<String>) -> Widget {
        let [logical, buffer] = surfaces(output, scale);
        let share = (self.height / output[1]).clamp(shortest(), *WIDGET_HEIGHTS.end());
        let size = paper_widgets::sized(logical, buffer, frame(self.face), Some(share))
            .expect("sized output");
        let Rect { x, y, w, h } = Rect::centred(self.centre, size);
        let (pin, height) = paper_widgets::pin([x, y, w, h], output);
        Widget { face: self.face, depth: self.depth, pin, height: Some(height), output: name }
    }
}

fn surfaces(output: [f32; 2], scale: f32) -> [[u32; 2]; 2] {
    [output.map(|side| side.round() as u32), output.map(|side| (side * scale).round() as u32)]
}

pub fn peers(list: &[Spot], index: usize) -> impl Iterator<Item = usize> + '_ {
    let spot = list[index];
    let bounds = spot.bounds();
    list.iter().enumerate().filter_map(move |(other, peer)| {
        (other != index && peer.depth == spot.depth && peer.bounds().overlaps(bounds))
            .then_some(other)
    })
}

pub fn restack(list: &[Spot], index: usize, up: bool) -> Option<usize> {
    if up {
        peers(list, index).find(|other| *other > index)
    } else {
        peers(list, index).filter(|other| *other < index).last()
    }
}

pub fn summit(list: &[Spot], index: usize) -> Option<usize> {
    peers(list, index).filter(|other| *other > index).last()
}

pub fn shift(list: &mut Vec<Spot>, from: usize, to: usize) {
    let spot = list.remove(from);
    list.insert(to.min(list.len()), spot);
}

pub fn cycle(list: &[Spot], index: usize, forward: bool) -> Option<Face> {
    let spot = list[index];
    let styles = spot.kind().styles();
    let at = super::catalogue::style_index(spot.face);
    let count = styles.len();
    (1..count)
        .map(|step| {
            let offset = if forward { step } else { count - step };
            styles[(at + offset) % count]
        })
        .find(|face| {
            !list.iter().enumerate().any(|(other, peer)| other != index && peer.face == *face)
        })
}

#[cfg(test)]
#[path = "spot_tests.rs"]
mod tests;
