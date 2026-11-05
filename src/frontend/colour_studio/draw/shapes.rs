use iced::{Color, Point, Rectangle};

use super::pen::{Pen, Words};
use crate::frontend::colour_studio::motion::{BASE, EASE, FAST, linear};
use crate::frontend::colour_studio::tokens::{
    PLATE_HEIGHT, PLATE_PAD, PLATE_RIM, PLATE_SHAPE_AT, PLATE_SMALL, PLATE_SMALL_PAD, PLATE_TEXT,
    PLATE_TEXT_DELAY,
};
use crate::frontend::components::lerp_color;
use crate::frontend::ui::UI_FONT;
use crate::frontend::ui::cached_line_width;

const FACE_TIP: f32 = 2.12;
const FACE_SIDE: f32 = 0.62;
const CORNERS: usize = 6;

pub fn diamond_points(centre: Point, side: f32) -> [Point; 4] {
    let half = side * std::f32::consts::FRAC_1_SQRT_2;
    [
        Point::new(centre.x, centre.y - half),
        Point::new(centre.x + half, centre.y),
        Point::new(centre.x, centre.y + half),
        Point::new(centre.x - half, centre.y),
    ]
}

pub fn diamond(pen: &mut Pen<'_>, centre: Point, side: f32, fill: Color) {
    pen.polygon(&diamond_points(centre, side), fill);
}

pub fn hollow(pen: &mut Pen<'_>, centre: Point, side: f32, edge: Color, width: f32) {
    pen.outline(&diamond_points(centre, side), width, edge);
}

#[derive(Clone, Copy)]
pub struct Ink {
    pub rim: Color,
    pub face: Color,
    pub text: Color,
}

#[derive(Clone, Copy, Default)]
pub struct Touch {
    pub hover: f32,
    pub press: f32,
    pub focus: bool,
}

#[derive(Clone, Copy)]
pub struct Plate<'a> {
    pub label: &'a str,
    pub left: f32,
    pub middle: f32,
    pub small: bool,
    pub width: Option<f32>,
}

impl<'a> Plate<'a> {
    pub fn new(label: &'a str, left: f32, middle: f32) -> Self {
        Self { label, left, middle, small: false, width: None }
    }

    pub fn small(self) -> Self {
        Self { small: true, ..self }
    }

    pub fn wide(self, width: f32) -> Self {
        Self { width: Some(width), ..self }
    }

    pub fn ending(self, right: f32) -> Self {
        Self { left: right - self.width(), ..self }
    }

    pub fn height(&self) -> f32 {
        if self.small { PLATE_SMALL } else { PLATE_HEIGHT }
    }

    fn pad(&self) -> f32 {
        if self.small { PLATE_SMALL_PAD } else { PLATE_PAD }
    }

    pub fn width(&self) -> f32 {
        let inner =
            self.width.unwrap_or_else(|| cached_line_width(self.label, PLATE_TEXT, UI_FONT));
        inner + 2.0 * self.pad()
    }

    pub fn bounds(&self) -> Rectangle {
        let height = self.height();
        Rectangle { x: self.left, y: self.middle - height / 2.0, width: self.width(), height }
    }

    pub fn text_left(&self) -> f32 {
        self.left + self.pad()
    }
}

fn outline(bounds: Rectangle) -> [Point; CORNERS] {
    let half = bounds.height / 2.0;
    let (left, right) = (bounds.x, bounds.x + bounds.width);
    let (top, bottom, middle) = (bounds.y, bounds.y + bounds.height, bounds.center_y());
    [
        Point::new(left + half, top),
        Point::new(right - half, top),
        Point::new(right, middle),
        Point::new(right - half, bottom),
        Point::new(left + half, bottom),
        Point::new(left, middle),
    ]
}

fn face(bounds: Rectangle) -> [Point; CORNERS] {
    let half = bounds.height / 2.0;
    let (left, right) = (bounds.x, bounds.x + bounds.width);
    let (top, bottom, middle) =
        (bounds.y + PLATE_RIM, bounds.y + bounds.height - PLATE_RIM, bounds.center_y());
    [
        Point::new(left + half + FACE_SIDE, top),
        Point::new(right - half - FACE_SIDE, top),
        Point::new(right - FACE_TIP, middle),
        Point::new(right - half - FACE_SIDE, bottom),
        Point::new(left + half + FACE_SIDE, bottom),
        Point::new(left + FACE_TIP, middle),
    ]
}

fn seed(bounds: Rectangle) -> [Point; CORNERS] {
    let half = bounds.height / 2.0;
    let centre = bounds.center();
    let (top, bottom) = (bounds.y, bounds.y + bounds.height);
    [
        Point::new(centre.x, top),
        Point::new(centre.x, top),
        Point::new(centre.x + half, centre.y),
        Point::new(centre.x, bottom),
        Point::new(centre.x, bottom),
        Point::new(centre.x - half, centre.y),
    ]
}

fn mix(from: &[Point; CORNERS], to: &[Point; CORNERS], share: f32) -> [Point; CORNERS] {
    std::array::from_fn(|index| {
        let (a, b) = (from[index], to[index]);
        Point::new(a.x + (b.x - a.x) * share, a.y + (b.y - a.y) * share)
    })
}

pub fn opening(bounds: Rectangle, age: f32) -> Option<[Point; CORNERS]> {
    let share = linear(age, 0.0, BASE);
    if share <= 0.0 {
        return None;
    }
    let full = outline(bounds);
    if share >= 1.0 {
        return Some(full);
    }
    let centre = bounds.center();
    if share < PLATE_SHAPE_AT {
        let grow = EASE.at(share / PLATE_SHAPE_AT);
        return Some(mix(&[centre; CORNERS], &seed(bounds), grow));
    }
    let grow = EASE.at((share - PLATE_SHAPE_AT) / (1.0 - PLATE_SHAPE_AT));
    Some(mix(&seed(bounds), &full, grow))
}

fn inside(edge: (Point, Point), at: Point) -> bool {
    let (a, b) = edge;
    (b.x - a.x) * (at.y - a.y) - (b.y - a.y) * (at.x - a.x) >= 0.0
}

fn crossing(edge: (Point, Point), from: Point, to: Point) -> Point {
    let (a, b) = edge;
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let (ex, ey) = (b.x - a.x, b.y - a.y);
    let denominator = ex * dy - ey * dx;
    if denominator.abs() < f32::EPSILON {
        return from;
    }
    let share = (ex * (a.y - from.y) - ey * (a.x - from.x)) / denominator;
    Point::new(from.x + dx * share, from.y + dy * share)
}

pub fn intersect(subject: &[Point], window: &[Point]) -> Vec<Point> {
    let mut out: Vec<Point> = subject.to_vec();
    for index in 0..window.len() {
        if out.is_empty() {
            break;
        }
        let edge = (window[index], window[(index + 1) % window.len()]);
        let input = std::mem::take(&mut out);
        for (step, current) in input.iter().enumerate() {
            let previous = input[(step + input.len() - 1) % input.len()];
            match (inside(edge, *current), inside(edge, previous)) {
                (true, true) => out.push(*current),
                (true, false) => {
                    out.push(crossing(edge, previous, *current));
                    out.push(*current);
                }
                (false, true) => out.push(crossing(edge, previous, *current)),
                (false, false) => {}
            }
        }
    }
    out
}

pub fn plate(pen: &mut Pen<'_>, plate: &Plate<'_>, ink: Ink, touch: Touch, age: f32) -> Rectangle {
    let bounds = plate.bounds();
    let Some(shape) = opening(bounds, age) else {
        return bounds;
    };
    let pressed = 1.0 - (1.0 - crate::frontend::colour_studio::tokens::PLATE_PRESS) * touch.press;
    let picked = ink.face == ink.rim;
    let hover_mix = if picked {
        crate::frontend::colour_studio::tokens::PLATE_PICKED_HOVER
    } else {
        crate::frontend::colour_studio::tokens::PLATE_HOVER
    };
    let lit = if picked { ink.text } else { ink.rim };
    let mut face_color =
        lerp_color(ink.face, lerp_color(ink.face, lit, 1.0 - hover_mix), touch.hover);
    if touch.focus {
        face_color = lerp_color(
            ink.face,
            ink.rim,
            1.0 - crate::frontend::colour_studio::tokens::PLATE_FOCUS,
        );
    }
    pen.zoomed(bounds.center(), pressed, |pen| {
        pen.polygon(&shape, ink.rim);
        let inner = intersect(&face(bounds), &shape);
        pen.polygon(&inner, face_color);
        let reveal = super::super::motion::eased(age, BASE * PLATE_TEXT_DELAY, FAST);
        if plate.width.is_none() {
            pen.faded(reveal, |pen| {
                pen.words(
                    Words::new(plate.label, bounds.center(), PLATE_TEXT, ink.text)
                        .centered()
                        .middle(),
                );
            });
        }
    });
    bounds
}

#[derive(Clone, Copy)]
pub struct Shadow {
    pub drop: f32,
    pub blur: f32,
    pub alpha: f32,
    pub rings: usize,
}

pub fn shadow(pen: &mut Pen<'_>, rect: Rectangle, cast: Shadow) {
    let step = cast.blur / cast.rings as f32;
    let share = cast.alpha / cast.rings as f32;
    for ring in 0..cast.rings {
        let spread = step * (ring as f32 + 0.5) - cast.blur / 2.0;
        let shaded = Rectangle {
            x: rect.x - spread,
            y: rect.y + cast.drop - spread,
            width: rect.width + 2.0 * spread,
            height: rect.height + 2.0 * spread,
        };
        pen.fill_rect(shaded, Color { a: share, ..Color::BLACK });
    }
}
