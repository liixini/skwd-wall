#[cfg(test)]
mod tests;

use iced::widget::canvas::Path;
use iced::{Point, Size};

use crate::frontend::ui::{folio_diagonal_edges, rtl};

pub(super) const HEIGHT: f32 = 43.0;
pub(super) const WIDTH: f32 = 210.0;
pub(super) const CUT: f32 = 8.0;
pub(super) const EDGE: f32 = 0.5;
pub(super) const BORDER_WIDTH: f32 = 1.0;

fn vertices(size: Size, cut: f32, inset: f32) -> [Point; 6] {
    let right = (size.width - inset).max(inset);
    let bottom = (size.height - inset).max(inset);
    let cut = cut.min((right - inset).max(0.0)).min((bottom - inset).max(0.0));
    [
        Point::new(inset + cut, inset),
        Point::new(right, inset),
        Point::new(right, bottom - cut),
        Point::new(right - cut, bottom),
        Point::new(inset, bottom),
        Point::new(inset, inset + cut),
    ]
}

fn polygon(points: &[Point]) -> Path {
    Path::new(|builder| {
        if let Some(first) = points.first() {
            builder.move_to(*first);
            for point in &points[1..] {
                builder.line_to(*point);
            }
            builder.close();
        }
    })
}

pub(super) fn fill(size: Size, scale: f32) -> Path {
    polygon(&vertices(size, CUT * scale, 0.0))
}

pub(super) fn outline(size: Size, scale: f32) -> Path {
    polygon(&vertices(size, CUT * scale, EDGE * scale))
}

fn wipe_vertices(size: Size, scale: f32, mix: f32, right_to_left: bool) -> Vec<Point> {
    if mix <= 0.0 || size.height <= 0.0 {
        return Vec::new();
    }
    let shape = vertices(size, CUT * scale, 0.0);
    if mix >= 1.0 {
        return shape.to_vec();
    }
    let (top, bottom) = folio_diagonal_edges(size.width, size.height, mix);
    let distance = |point: Point| {
        let edge = top + (bottom - top) * point.y / size.height;
        if right_to_left { size.width - edge - point.x } else { point.x - edge }
    };
    let mut clipped = Vec::with_capacity(shape.len() + 2);
    let mut previous = shape[shape.len() - 1];
    let mut previous_distance = distance(previous);
    for point in shape {
        let point_distance = distance(point);
        if (previous_distance <= 0.0) != (point_distance <= 0.0) {
            let fraction = previous_distance / (previous_distance - point_distance);
            clipped.push(Point::new(
                previous.x + (point.x - previous.x) * fraction,
                previous.y + (point.y - previous.y) * fraction,
            ));
        }
        if point_distance <= 0.0 {
            clipped.push(point);
        }
        previous = point;
        previous_distance = point_distance;
    }
    clipped
}

pub(super) fn wipe(size: Size, scale: f32, mix: f32) -> Path {
    polygon(&wipe_vertices(size, scale, mix, rtl()))
}
