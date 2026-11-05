use iced::{Color, Point};
use skwd_palette::studio::Shape;

use super::pen::Pen;
use crate::frontend::colour_studio::motion::{BASE, FAST, SLOW, eased};

const RING: f32 = 0.36;
const GAP: f32 = 6.0;
const OVERSHOOT: f32 = 2.0;
const WIDTH: f32 = 1.4;
const SECTOR_STEP: f32 = 10.0;
const MIN_STEPS: f32 = 2.0;
const JITTER: (f32, f32, f32) = (12.9898, 78.233, 43_758.547);
const TILT: f32 = -45.0;
const WOBBLE: f32 = 1.6;
const OPACITY: (f32, f32) = (0.72, 0.28);
const STROKE_GAP: f32 = 0.024;
const LEAD: f32 = 0.08;
const STOPS: [(f32, f32); 3] = [(0.0, 0.42), (0.55, 0.58), (1.0, 0.2)];

pub struct Stroke {
    pub index: usize,
    pub start: Point,
    pub length: f32,
    pub angle: f32,
    pub opacity: f32,
    pub zone: usize,
}

fn polar(centre: f32, hue: f32, radius: f32) -> Point {
    let angle = hue.to_radians();
    Point::new(centre + radius * angle.cos(), centre - radius * angle.sin())
}

pub fn zones(shape: &Shape, size: f32) -> Vec<Vec<Point>> {
    let centre = size / 2.0;
    let radius = size * RING;
    match shape {
        Shape::Plain => Vec::new(),
        Shape::Mask(points) if points.is_empty() => Vec::new(),
        Shape::Mask(points) => {
            vec![points.iter().map(|[hue, reach]| polar(centre, *hue, radius * reach)).collect()]
        }
        Shape::Sectors(sectors) => sectors
            .iter()
            .map(|[middle, width]| {
                let steps = (width / SECTOR_STEP).ceil().max(MIN_STEPS) as usize;
                let mut points = vec![Point::new(centre, centre)];
                for step in 0..=steps {
                    let hue = middle - width / 2.0 + width * step as f32 / steps as f32;
                    points.push(polar(centre, hue, radius));
                }
                points
            })
            .collect(),
    }
}

fn jitter(index: usize) -> f32 {
    let wave = ((index as f32) * JITTER.0 + JITTER.1).sin() * JITTER.2;
    wave - wave.floor()
}

pub fn strokes(polygons: &[Vec<Point>]) -> Vec<Stroke> {
    let normal = (std::f32::consts::FRAC_1_SQRT_2, std::f32::consts::FRAC_1_SQRT_2);
    let along = (std::f32::consts::FRAC_1_SQRT_2, -std::f32::consts::FRAC_1_SQRT_2);
    let dot = |point: Point, axis: (f32, f32)| point.x * axis.0 + point.y * axis.1;
    let mut out = Vec::new();
    for (zone, points) in polygons.iter().enumerate() {
        let projected: Vec<f32> = points.iter().map(|point| dot(*point, normal)).collect();
        let low = projected.iter().copied().fold(f32::INFINITY, f32::min);
        let high = projected.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut offset = (low / GAP).ceil() * GAP;
        while offset <= high {
            let mut cuts = Vec::new();
            for index in 0..points.len() {
                let (a, b) = (points[index], points[(index + 1) % points.len()]);
                let (da, db) = (dot(a, normal), dot(b, normal));
                if (da - db).abs() < f32::EPSILON || (offset - da) * (offset - db) > 0.0 {
                    continue;
                }
                let share = (offset - da) / (db - da);
                let cut = Point::new(a.x + share * (b.x - a.x), a.y + share * (b.y - a.y));
                cuts.push(dot(cut, along));
            }
            if cuts.len() >= 2 {
                let first = cuts.iter().copied().fold(f32::INFINITY, f32::min) - OVERSHOOT;
                let last = cuts.iter().copied().fold(f32::NEG_INFINITY, f32::max) + OVERSHOOT;
                let index = out.len();
                let random = jitter(index);
                out.push(Stroke {
                    index,
                    start: Point::new(
                        offset * normal.0 + first * along.0,
                        offset * normal.1 + first * along.1,
                    ),
                    length: last - first,
                    angle: TILT + (random - 0.5) * WOBBLE,
                    opacity: OPACITY.0 + random * OPACITY.1,
                    zone,
                });
            }
            offset += GAP;
        }
    }
    out
}

fn inside(points: &[Point], at: Point) -> bool {
    let mut hit = false;
    for index in 0..points.len() {
        let (a, b) = (points[index], points[(index + points.len() - 1) % points.len()]);
        if (a.y > at.y) != (b.y > at.y) && at.x < (b.x - a.x) * (at.y - a.y) / (b.y - a.y) + a.x {
            hit = !hit;
        }
    }
    hit
}

fn spans(points: &[Point], start: Point, end: Point) -> Vec<(f32, f32)> {
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let mut cuts = vec![0.0, 1.0];
    for index in 0..points.len() {
        let (a, b) = (points[index], points[(index + 1) % points.len()]);
        let (ex, ey) = (b.x - a.x, b.y - a.y);
        let denominator = dx * ey - dy * ex;
        if denominator.abs() < f32::EPSILON {
            continue;
        }
        let along = ((a.x - start.x) * ey - (a.y - start.y) * ex) / denominator;
        let edge = ((a.x - start.x) * dy - (a.y - start.y) * dx) / denominator;
        if (0.0..=1.0).contains(&along) && (0.0..=1.0).contains(&edge) {
            cuts.push(along);
        }
    }
    cuts.sort_by(f32::total_cmp);
    cuts.windows(2)
        .filter(|pair| pair[1] - pair[0] > f32::EPSILON)
        .filter(|pair| {
            let middle = f32::midpoint(pair[0], pair[1]);
            inside(points, Point::new(start.x + dx * middle, start.y + dy * middle))
        })
        .map(|pair| (pair[0], pair[1]))
        .collect()
}

pub fn pace(count: usize) -> f32 {
    STROKE_GAP.min((SLOW - FAST) / (count.saturating_sub(1).max(1)) as f32)
}

pub fn draw(
    pen: &mut Pen<'_>,
    origin: Point,
    polygons: &[Vec<Point>],
    ink: Color,
    age: Option<f32>,
    fade: f32,
) {
    let found = strokes(polygons);
    let pace = pace(found.len());
    let shift = |point: Point| Point::new(origin.x + point.x, origin.y + point.y);
    for stroke in &found {
        let drawn = age.map_or(1.0, |age| eased(age, LEAD + stroke.index as f32 * pace, FAST));
        if drawn <= 0.0 {
            continue;
        }
        let (sin, cos) = stroke.angle.to_radians().sin_cos();
        let end =
            Point::new(stroke.start.x + cos * stroke.length, stroke.start.y + sin * stroke.length);
        let zone = &polygons[stroke.zone];
        let colors = STOPS.map(|(offset, share)| {
            (offset, Color { a: ink.a * share * stroke.opacity * fade, ..ink })
        });
        for (from, to) in spans(zone, stroke.start, end) {
            let to = to.min(drawn);
            if to <= from {
                continue;
            }
            let at = |share: f32| {
                shift(Point::new(
                    stroke.start.x + (end.x - stroke.start.x) * share,
                    stroke.start.y + (end.y - stroke.start.y) * share,
                ))
            };
            pen.gradient_segment(at(0.0), at(1.0), at(from), at(to), WIDTH, &colors);
        }
    }
}

pub fn fading(age: f32) -> f32 {
    1.0 - eased(age, 0.0, BASE)
}
