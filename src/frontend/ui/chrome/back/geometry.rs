use iced::widget::canvas::{Frame, Path, Text};
use iced::{Point, Rectangle, Size, Vector, border};

use crate::frontend::scene::BackPanel;
use crate::frontend::scene::layout::CardCorners;

use super::layout::BackLayout;

#[derive(Debug, Clone, Copy)]
pub(super) struct BackGeometry {
    rtl: bool,
    m11: f32,
    m12: f32,
    m21: f32,
    m22: f32,
    m31: f32,
    m32: f32,
}

impl BackGeometry {
    pub(super) fn new(panel: &BackPanel, layout: &BackLayout) -> Self {
        let (card_x, card_y, card_width, card_height) = layout.card;
        let hw = (card_width * 0.5).max(1.0);
        let hh = (card_height * 0.5).max(1.0);
        let cx = card_x + hw;
        let cy = card_y + hh;
        let sx = panel.skew * 0.5;
        let ty = panel.edge_tilt * 0.5;
        let bx = (hw - sx.abs()).max(1.0);
        let by = (hh - ty.abs()).max(1.0);
        let m11 = bx / hw;
        let m12 = -ty / hw;
        let m21 = -sx / hh;
        let m22 = by / hh;
        let m31 = cx - cx * m11 - cy * m21;
        let m32 = cy - cx * m12 - cy * m22;
        Self { rtl: layout.rtl, m11, m12, m21, m22, m31, m32 }
    }

    pub(super) fn rtl(self) -> bool {
        self.rtl
    }

    pub(super) fn point(self, point: Point) -> Point {
        Point::new(
            point.x * self.m11 + point.y * self.m21 + self.m31,
            point.x * self.m12 + point.y * self.m22 + self.m32,
        )
    }

    fn inverse_point(self, point: Point) -> Point {
        let det = self.m11 * self.m22 - self.m12 * self.m21;
        if det.abs() < 0.000_001 {
            return point;
        }
        let x = point.x - self.m31;
        let y = point.y - self.m32;
        Point::new((self.m22 * x - self.m21 * y) / det, (-self.m12 * x + self.m11 * y) / det)
    }

    fn transform(self) -> iced::widget::canvas::path::lyon_path::math::Transform {
        iced::widget::canvas::path::lyon_path::math::Transform::new(
            self.m11, self.m12, self.m21, self.m22, self.m31, self.m32,
        )
    }

    pub(super) fn path(self, path: &Path) -> Path {
        path.transform(&self.transform())
    }

    pub(super) fn rounded_rectangle(self, rectangle: Rectangle, radii: [f32; 4]) -> Path {
        self.path(&Path::rounded_rectangle(
            rectangle.position(),
            rectangle.size(),
            border::Radius {
                top_left: radii[0],
                top_right: radii[1],
                bottom_right: radii[2],
                bottom_left: radii[3],
            },
        ))
    }

    pub(super) fn outline(self, rectangle: Rectangle, corners: CardCorners) -> Path {
        if !corners.cut {
            return self.rounded_rectangle(rectangle, corners.sizes);
        }
        let across = self.m11.hypot(self.m12).max(f32::EPSILON);
        let down = self.m21.hypot(self.m22).max(f32::EPSILON);
        let cuts = corners.sizes.map(|size| [size / across, size / down]);
        self.path(&crate::frontend::components::chamfered(rectangle, cuts))
    }

    pub(super) fn line(self, from: Point, to: Point) -> Path {
        Path::line(self.point(from), self.point(to))
    }

    pub(super) fn fill_text(self, frame: &mut Frame, mut text: Text) {
        let position = self.point(text.position);
        text.position = Point::ORIGIN;
        frame.with_save(|frame| {
            frame.translate(Vector::new(position.x, position.y));
            frame.rotate(self.text_angle());
            frame.fill_text(text);
        });
    }

    fn text_angle(self) -> f32 {
        self.m12.atan2(self.m11)
    }

    fn contains(self, rectangle: Rectangle, point: Point) -> bool {
        rectangle.contains(self.inverse_point(point))
    }

    fn bounds(self, rectangle: Rectangle) -> Rectangle {
        let points = [
            self.point(rectangle.position()),
            self.point(Point::new(rectangle.x + rectangle.width, rectangle.y)),
            self.point(Point::new(rectangle.x + rectangle.width, rectangle.y + rectangle.height)),
            self.point(Point::new(rectangle.x, rectangle.y + rectangle.height)),
        ];
        let min_x = points.iter().map(|point| point.x).fold(f32::INFINITY, f32::min);
        let max_x = points.iter().map(|point| point.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = points.iter().map(|point| point.y).fold(f32::INFINITY, f32::min);
        let max_y = points.iter().map(|point| point.y).fold(f32::NEG_INFINITY, f32::max);
        Rectangle::new(Point::new(min_x, min_y), Size::new(max_x - min_x, max_y - min_y))
    }
}

fn rectangle(tuple: (f32, f32, f32, f32)) -> Rectangle {
    Rectangle::new(Point::new(tuple.0, tuple.1), Size::new(tuple.2, tuple.3))
}

pub fn back_contains(
    panel: &BackPanel,
    layout: &BackLayout,
    target: (f32, f32, f32, f32),
    x: f32,
    y: f32,
) -> bool {
    BackGeometry::new(panel, layout).contains(rectangle(target), Point::new(x, y))
}

pub fn back_bounds(
    panel: &BackPanel,
    layout: &BackLayout,
    target: (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    let bounds = BackGeometry::new(panel, layout).bounds(rectangle(target));
    (bounds.x, bounds.y, bounds.width, bounds.height)
}

#[cfg(test)]
mod tests;
