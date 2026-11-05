use crate::frontend::components::{Ink, Pen};

use super::super::state::Strand;
use super::{selection, stage, thread};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Backdrop,
    Scrim,
    Widgets,
    Chrome,
}

pub const PARTS: [(Part, Ink); 4] = [
    (Part::Backdrop, Ink::Images),
    (Part::Scrim, Ink::Shapes),
    (Part::Widgets, Ink::Images),
    (Part::Chrome, Ink::Shapes),
];

pub fn draw(pen: &mut Pen<'_>, strand: &Strand, part: Part) {
    match part {
        Part::Backdrop => stage::backdrop(pen, strand),
        Part::Scrim => stage::dim(pen, strand),
        Part::Widgets => stage::widgets(pen, strand),
        Part::Chrome => {
            stage::wipes(pen, strand);
            selection::ticks(pen, strand);
            selection::guides(pen, strand);
            thread::strand(pen, strand);
            selection::frame(pen, strand);
        }
    }
}
