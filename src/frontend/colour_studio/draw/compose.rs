use iced::Color;

use super::pen::{Ink, Pen};
use super::shapes::Touch;
use super::window::Chrome;
use super::{bar, nav, pick, preview, wheel, window};
use crate::frontend::colour_studio::controls::{Hit, Plate as PlateId};
use crate::frontend::colour_studio::layout::Frame;
use crate::frontend::colour_studio::motion::{FAST, Look, eased};
use crate::frontend::colour_studio::state::ColourStudio;

pub fn hover(studio: &ColourStudio, hit: &Hit) -> f32 {
    if studio.hover.as_ref() == Some(hit) {
        return eased(studio.now, studio.hover_at, FAST);
    }
    if studio.unhover.as_ref() == Some(hit) {
        return 1.0 - eased(studio.now, studio.unhover_at, FAST);
    }
    0.0
}

pub fn touch(studio: &ColourStudio, plate: PlateId) -> Touch {
    let press = if studio.pressed == Some(plate) {
        eased(studio.now, studio.pressed_at, FAST)
    } else {
        0.0
    };
    Touch { hover: hover(studio, &Hit::Plate(plate)), press, focus: false }
}

pub fn entering(age: f32, delay: f32, duration: f32, look: fn(f32) -> Look) -> Look {
    look(eased(age, delay, duration))
}

pub fn faint(color: Color, alpha: f32) -> Color {
    Color { a: color.a * alpha, ..color }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Window,
    Actions,
    Scene(preview::Layer),
    Pick,
    Over,
}

pub const PARTS: [(Part, Ink); 13] = [
    (Part::Window, Ink::Shapes),
    (Part::Actions, Ink::Shapes),
    (Part::Scene(preview::Layer::Current), Ink::Images),
    (Part::Scene(preview::Layer::Current), Ink::Shapes),
    (Part::Scene(preview::Layer::Previous), Ink::Images),
    (Part::Scene(preview::Layer::Previous), Ink::Shapes),
    (Part::Scene(preview::Layer::Live), Ink::Images),
    (Part::Scene(preview::Layer::Live), Ink::Shapes),
    (Part::Scene(preview::Layer::LivePrevious), Ink::Images),
    (Part::Scene(preview::Layer::LivePrevious), Ink::Shapes),
    (Part::Pick, Ink::Images),
    (Part::Pick, Ink::Shapes),
    (Part::Over, Ink::Shapes),
];

pub fn studio(
    pen: &mut Pen<'_>,
    studio: &ColourStudio,
    chrome: &Chrome,
    frame: &Frame,
    part: Part,
) {
    match part {
        Part::Window => {
            window::frame(pen, studio, chrome);
            preview::edge(pen, studio, chrome);
            let ready = studio.content_age();
            if ready.is_finite() {
                nav::draw(pen, studio, chrome, ready);
                wheel::draw(pen, studio, chrome, ready);
                bar::draw(pen, studio, chrome, ready);
            }
        }
        Part::Actions => window::chrome(pen, studio, chrome),
        Part::Scene(layer) => preview::draw(pen, studio, chrome, frame, layer),
        Part::Pick => pick::draw(pen, studio, chrome),
        Part::Over => {
            preview::strip(pen, studio, chrome);
            bar::popover(pen, studio, chrome);
        }
    }
}
