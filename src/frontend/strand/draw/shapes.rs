use iced::{Color, Point};
use paper_widgets::Tones;

use super::super::labels::shown;
use super::super::tokens::{FILL_SHOWN, PLATE, RIM, THREAD};
use crate::domain::strand::Rect;
use crate::frontend::components::{Pen, Words};

const SLIT_CORNERS: usize = 6;

pub fn color(ink: paper_widgets::Ink) -> Color {
    let [red, green, blue] = ink.rgb;
    Color::from_rgba8(red, green, blue, ink.alpha)
}

pub fn point(at: [f32; 2]) -> Point {
    Point::new(at[0], at[1])
}

pub fn diamond(centre: Point, reach: f32) -> [Point; 4] {
    [
        Point::new(centre.x, centre.y - reach),
        Point::new(centre.x + reach, centre.y),
        Point::new(centre.x, centre.y + reach),
        Point::new(centre.x - reach, centre.y),
    ]
}

pub fn slit(rect: Rect) -> [Point; SLIT_CORNERS] {
    let cut = (rect.h / 2.0).min(rect.w / 2.0);
    let middle = rect.y + rect.h / 2.0;
    [
        Point::new(rect.x + cut, rect.y),
        Point::new(rect.right() - cut, rect.y),
        Point::new(rect.right(), middle),
        Point::new(rect.right() - cut, rect.bottom()),
        Point::new(rect.x + cut, rect.bottom()),
        Point::new(rect.x, middle),
    ]
}

pub fn knot(pen: &mut Pen<'_>, centre: Point, reach: f32, tones: &Tones) {
    pen.polygon(&diamond(centre, reach), color(tones.accent));
}

pub fn tip(pen: &mut Pen<'_>, centre: Point, tones: &Tones) {
    knot(pen, centre, THREAD.tip, tones);
}

pub fn bead(pen: &mut Pen<'_>, centre: Point, reach: f32, fill: f32, tones: &Tones) {
    pen.polygon(&diamond(centre, reach), color(tones.accent));
    let inner = (reach - RIM).max(0.0);
    pen.polygon(&diamond(centre, inner), color(tones.shade));
    if fill > FILL_SHOWN {
        pen.polygon(&diamond(centre, inner * fill.min(1.0)), color(tones.accent));
    }
}

pub struct PlateLook {
    pub on: bool,
    pub hover: bool,
    pub open: f32,
}

pub const PLAIN: PlateLook = PlateLook { on: false, hover: false, open: 1.0 };

pub fn plate(pen: &mut Pen<'_>, rect: Rect, label: &str, look: &PlateLook, tones: &Tones) {
    if rect.w < 2.0 {
        return;
    }
    pen.polygon(&slit(rect), color(tones.accent));
    let inner = Rect::new(
        rect.x + PLATE.rim,
        rect.y + PLATE.rim,
        rect.w - 2.0 * PLATE.rim,
        rect.h - 2.0 * PLATE.rim,
    );
    let face = if look.on { tones.accent } else { tones.shade };
    pen.polygon(&slit(inner), color(face));
    let reveal = ((look.open - PLATE.reveal) / (1.0 - PLATE.reveal)).clamp(0.0, 1.0);
    if reveal <= 0.0 {
        return;
    }
    let ink = if look.on {
        tones.shade
    } else if look.hover {
        tones.accent
    } else {
        tones.text
    };
    let text = shown(label);
    let centre = Point::new(rect.x + rect.w / 2.0, rect.y + rect.h / 2.0);
    let words = Words::new(&text, centre, PLATE.text, color(ink))
        .centered()
        .middle()
        .tracked(PLATE.tracking);
    if reveal >= 1.0 && words.advance() + PLATE.pad <= rect.w {
        pen.words(words);
        return;
    }
    let bounds = iced::Rectangle::new(Point::new(rect.x, rect.y), iced::Size::new(rect.w, rect.h));
    pen.clipped(bounds, |pen| pen.faded(reveal, |pen| pen.words(words)));
}
