use iced::advanced::text::Alignment as TextAlignment;
use iced::widget::canvas::{
    Fill, Frame, Geometry, Gradient, LineDash, Path, Stroke, Text, gradient,
};
use iced::widget::image::Handle;
use iced::{Color, Point, Rectangle, Size, Vector, alignment};

use crate::frontend::ui::{UI_FONT, anchor_rtl, cached_line_width, shapes_across};

const DEFAULT_LINE: f32 = 1.2;
const GLYPH_BYTES: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    pub alpha: f32,
    pub dx: f32,
    pub dy: f32,
    pub scale: f32,
    pub turn: f32,
}

impl Look {
    pub const REST: Self = Self { alpha: 1.0, dx: 0.0, dy: 0.0, scale: 1.0, turn: 0.0 };
}

pub trait Layers {
    fn frame(&mut self) -> &mut Frame;
    fn clip(&self) -> Rectangle;
    fn begin(&mut self, clip: Rectangle);
}

pub struct Sheet<'r> {
    renderer: &'r iced::Renderer,
    clip: Rectangle,
    done: Vec<Frame>,
    current: Frame,
}

impl<'r> Sheet<'r> {
    pub fn new(renderer: &'r iced::Renderer, bounds: Rectangle) -> Self {
        Self {
            renderer,
            clip: bounds,
            done: Vec::new(),
            current: Frame::with_bounds(renderer, bounds),
        }
    }

    pub fn finish(mut self) -> Vec<Geometry> {
        self.done.push(self.current);
        self.done.into_iter().map(Frame::into_geometry).collect()
    }
}

impl Layers for Sheet<'_> {
    fn frame(&mut self) -> &mut Frame {
        &mut self.current
    }

    fn clip(&self) -> Rectangle {
        self.clip
    }

    fn begin(&mut self, clip: Rectangle) {
        let next = Frame::with_bounds(self.renderer, clip);
        self.done.push(std::mem::replace(&mut self.current, next));
        self.clip = clip;
    }
}

pub struct Unclipped<'f> {
    frame: &'f mut Frame,
    clip: Rectangle,
}

impl<'f> Unclipped<'f> {
    pub fn new(frame: &'f mut Frame, bounds: Rectangle) -> Self {
        Self { frame, clip: bounds }
    }
}

impl Layers for Unclipped<'_> {
    fn frame(&mut self) -> &mut Frame {
        self.frame
    }

    fn clip(&self) -> Rectangle {
        self.clip
    }

    fn begin(&mut self, clip: Rectangle) {
        self.clip = clip;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Affine {
    pub const IDENTITY: Self = Self { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 };

    pub fn place(origin: Vector, scale: f32) -> Self {
        Self { a: scale, d: scale, e: origin.x, f: origin.y, ..Self::IDENTITY }
    }

    fn translate(offset: Vector) -> Self {
        Self { e: offset.x, f: offset.y, ..Self::IDENTITY }
    }

    fn scale(x: f32, y: f32) -> Self {
        Self { a: x, d: y, ..Self::IDENTITY }
    }

    fn rotate(degrees: f32) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();
        Self { a: cos, b: sin, c: -sin, d: cos, ..Self::IDENTITY }
    }

    fn about(pivot: Point, inner: Self) -> Self {
        Self::translate(Vector::new(pivot.x, pivot.y))
            .then(inner)
            .then(Self::translate(Vector::new(-pivot.x, -pivot.y)))
    }

    pub fn then(self, inner: Self) -> Self {
        Self {
            a: self.a * inner.a + self.c * inner.b,
            b: self.b * inner.a + self.d * inner.b,
            c: self.a * inner.c + self.c * inner.d,
            d: self.b * inner.c + self.d * inner.d,
            e: self.a * inner.e + self.c * inner.f + self.e,
            f: self.b * inner.e + self.d * inner.f + self.f,
        }
    }

    pub fn apply(&self, at: Point) -> Point {
        Point::new(self.a * at.x + self.c * at.y + self.e, self.b * at.x + self.d * at.y + self.f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Images,
    Shapes,
}

pub struct Pen<'a> {
    sheet: &'a mut (dyn Layers + 'a),
    ink: Ink,
    map: Affine,
    unit: f32,
    alpha: f32,
}

#[derive(Clone, Copy)]
pub struct Words<'a> {
    pub content: &'a str,
    pub font: iced::Font,
    pub at: Point,
    pub size: f32,
    pub color: Color,
    pub align: TextAlignment,
    pub middle: bool,
    pub width: f32,
    pub line: f32,
    pub tracking: f32,
}

impl<'a> Words<'a> {
    pub fn new(content: &'a str, at: Point, size: f32, color: Color) -> Self {
        Self {
            content,
            font: UI_FONT,
            at,
            size,
            color,
            align: TextAlignment::Left,
            middle: false,
            width: f32::INFINITY,
            line: DEFAULT_LINE,
            tracking: 0.0,
        }
    }

    pub fn tracked(self, tracking: f32) -> Self {
        Self { tracking, ..self }
    }

    pub fn tracks(&self) -> bool {
        self.tracking != 0.0 && !shapes_across(self.content)
    }

    pub fn advance(&self) -> f32 {
        let width = cached_line_width(self.content, self.size, self.font);
        if self.tracks() {
            width + self.tracking * (self.content.chars().count().saturating_sub(1)) as f32
        } else {
            width
        }
    }

    pub fn centered(self) -> Self {
        Self { align: TextAlignment::Center, ..self }
    }

    pub fn right(self) -> Self {
        Self { align: TextAlignment::Right, ..self }
    }

    pub fn middle(self) -> Self {
        Self { middle: true, ..self }
    }

    pub fn font(self, font: iced::Font) -> Self {
        Self { font, ..self }
    }

    pub fn wrap(self, width: f32) -> Self {
        Self { width, ..self }
    }

    pub fn line(self, line: f32) -> Self {
        Self { line, ..self }
    }
}

impl<'a> Pen<'a> {
    pub fn new(sheet: &'a mut (dyn Layers + 'a), origin: Vector, scale: f32, ink: Ink) -> Self {
        Self { sheet, ink, map: Affine::place(origin, scale), unit: scale, alpha: 1.0 }
    }

    fn inner(&mut self, map: Affine, unit: f32, alpha: f32) -> Pen<'_> {
        Pen { sheet: &mut *self.sheet, ink: self.ink, map, unit, alpha }
    }

    fn hidden(&self) -> bool {
        self.ink != Ink::Shapes || self.alpha <= 0.0
    }

    pub fn faded<R>(&mut self, alpha: f32, body: impl FnOnce(&mut Pen<'_>) -> R) -> R {
        let (map, unit, alpha) = (self.map, self.unit, self.alpha * alpha);
        body(&mut self.inner(map, unit, alpha))
    }

    pub fn shifted<R>(&mut self, offset: Vector, body: impl FnOnce(&mut Pen<'_>) -> R) -> R {
        let map = self.map.then(Affine::translate(offset));
        let (unit, alpha) = (self.unit, self.alpha);
        body(&mut self.inner(map, unit, alpha))
    }

    pub fn zoomed<R>(&mut self, pivot: Point, by: f32, body: impl FnOnce(&mut Pen<'_>) -> R) -> R {
        let map = self.map.then(Affine::about(pivot, Affine::scale(by, by)));
        let (unit, alpha) = (self.unit * by, self.alpha);
        body(&mut self.inner(map, unit, alpha))
    }

    pub fn stretched<R>(
        &mut self,
        pivot: Point,
        by: (f32, f32),
        body: impl FnOnce(&mut Pen<'_>) -> R,
    ) -> R {
        let map = self.map.then(Affine::about(pivot, Affine::scale(by.0, by.1)));
        let (unit, alpha) = (self.unit, self.alpha);
        body(&mut self.inner(map, unit, alpha))
    }

    pub fn looked<R>(
        &mut self,
        look: Look,
        pivot: Point,
        body: impl FnOnce(&mut Pen<'_>) -> R,
    ) -> R {
        if look.alpha <= 0.0 {
            return body(&mut self.inner(self.map, self.unit, 0.0));
        }
        let around = Affine::about(
            pivot,
            Affine::rotate(-look.turn).then(Affine::scale(look.scale, look.scale)),
        );
        let map = self.map.then(Affine::translate(Vector::new(look.dx, look.dy))).then(around);
        let (unit, alpha) = (self.unit * look.scale, self.alpha * look.alpha);
        body(&mut self.inner(map, unit, alpha))
    }

    pub fn layer(&mut self) {
        let clip = self.sheet.clip();
        self.sheet.begin(clip);
    }

    pub fn clipped<R>(&mut self, rect: Rectangle, body: impl FnOnce(&mut Pen<'_>) -> R) -> R {
        let outer = self.sheet.clip();
        let inside = outer
            .intersection(&self.rect(rect))
            .unwrap_or(Rectangle::new(outer.position(), Size::ZERO));
        self.sheet.begin(inside);
        let (map, unit, alpha) = (self.map, self.unit, self.alpha);
        let out = body(&mut self.inner(map, unit, alpha));
        self.sheet.begin(outer);
        out
    }

    pub fn point(&self, at: Point) -> Point {
        self.map.apply(at)
    }

    pub fn rect(&self, rect: Rectangle) -> Rectangle {
        let corners = corners(rect).map(|corner| self.point(corner));
        let (mut low, mut high) = (corners[0], corners[0]);
        for corner in &corners[1..] {
            low = Point::new(low.x.min(corner.x), low.y.min(corner.y));
            high = Point::new(high.x.max(corner.x), high.y.max(corner.y));
        }
        Rectangle::new(low, Size::new(high.x - low.x, high.y - low.y))
    }

    fn tint(&self, color: Color) -> Color {
        Color { a: color.a * self.alpha, ..color }
    }

    fn shape(&self, points: &[Point]) -> Option<Path> {
        let (first, rest) = points.split_first()?;
        Some(Path::new(|builder| {
            builder.move_to(self.point(*first));
            for point in rest {
                builder.line_to(self.point(*point));
            }
            builder.close();
        }))
    }

    pub fn fill_rect(&mut self, rect: Rectangle, color: Color) {
        self.polygon(&corners(rect), color);
    }

    pub fn polygon(&mut self, points: &[Point], color: Color) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 || color.a <= 0.0 {
            return;
        }
        if let Some(path) = self.shape(points) {
            let color = self.tint(color);
            self.sheet.frame().fill(&path, color);
        }
    }

    pub fn outline(&mut self, points: &[Point], width: f32, color: Color) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 || color.a <= 0.0 {
            return;
        }
        if let Some(path) = self.shape(points) {
            let stroke =
                Stroke::default().with_width(width * self.unit).with_color(self.tint(color));
            self.sheet.frame().stroke(&path, stroke);
        }
    }

    pub fn line(&mut self, from: Point, to: Point, width: f32, color: Color) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 || color.a <= 0.0 {
            return;
        }
        let path = Path::line(self.point(from), self.point(to));
        let stroke = Stroke::default().with_width(width * self.unit).with_color(self.tint(color));
        self.sheet.frame().stroke(&path, stroke);
    }

    pub fn polyline(&mut self, points: &[Point], width: f32, color: Color) {
        let Some((first, rest)) = points.split_first() else { return };
        if self.hidden() || color.a <= 0.0 || rest.is_empty() {
            return;
        }
        let path = Path::new(|builder| {
            builder.move_to(self.point(*first));
            for point in rest {
                builder.line_to(self.point(*point));
            }
        });
        let stroke = Stroke::default().with_width(width * self.unit).with_color(self.tint(color));
        self.sheet.frame().stroke(&path, stroke);
    }

    pub fn curve(&mut self, from: Point, control: Point, to: Point, width: f32, color: Color) {
        if self.hidden() || color.a <= 0.0 {
            return;
        }
        let path = Path::new(|builder| {
            builder.move_to(self.point(from));
            builder.quadratic_curve_to(self.point(control), self.point(to));
        });
        let stroke = Stroke::default().with_width(width * self.unit).with_color(self.tint(color));
        self.sheet.frame().stroke(&path, stroke);
    }

    pub fn gradient_line(&mut self, from: Point, to: Point, width: f32, stops: &[(f32, Color)]) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 {
            return;
        }
        let (start, end) = (self.point(from), self.point(to));
        let ramp = stops.iter().fold(gradient::Linear::new(start, end), |ramp, (offset, color)| {
            ramp.add_stop(*offset, self.tint(*color))
        });
        let stroke = Stroke {
            style: Gradient::Linear(ramp).into(),
            ..Stroke::default().with_width(width * self.unit)
        };
        self.sheet.frame().stroke(&Path::line(start, end), stroke);
    }

    pub fn gradient_segment(
        &mut self,
        start: Point,
        end: Point,
        from: Point,
        to: Point,
        width: f32,
        stops: &[(f32, Color)],
    ) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 {
            return;
        }
        let ramp = stops.iter().fold(
            gradient::Linear::new(self.point(start), self.point(end)),
            |ramp, (offset, color)| ramp.add_stop(*offset, self.tint(*color)),
        );
        let stroke = Stroke {
            style: Gradient::Linear(ramp).into(),
            ..Stroke::default().with_width(width * self.unit)
        };
        let path = Path::line(self.point(from), self.point(to));
        self.sheet.frame().stroke(&path, stroke);
    }

    pub fn gradient_rect(
        &mut self,
        rect: Rectangle,
        from: Point,
        to: Point,
        stops: &[(f32, Color)],
    ) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 {
            return;
        }
        let Some(path) = self.shape(&corners(rect)) else {
            return;
        };
        let ramp = stops.iter().fold(
            gradient::Linear::new(self.point(from), self.point(to)),
            |ramp, (offset, color)| ramp.add_stop(*offset, self.tint(*color)),
        );
        let fill = Fill { style: Gradient::Linear(ramp).into(), ..Fill::default() };
        self.sheet.frame().fill(&path, fill);
    }

    pub fn arc(
        &mut self,
        centre: Point,
        radius: f32,
        sweep: (f32, f32),
        width: f32,
        color: Color,
        dash: Option<f32>,
    ) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 || color.a <= 0.0 || sweep.1 <= 0.0 {
            return;
        }
        let centre = self.point(centre);
        let radius = radius * self.unit;
        let path = Path::new(|builder| {
            builder.arc(iced::widget::canvas::path::Arc {
                center: centre,
                radius,
                start_angle: iced::Radians(sweep.0.to_radians()),
                end_angle: iced::Radians((sweep.0 + sweep.1).to_radians()),
            });
        });
        let stroke = Stroke::default().with_width(width * self.unit).with_color(self.tint(color));
        let segments = dash.map(|length| [length * self.unit, length * self.unit]);
        let stroke = match &segments {
            Some(segments) => Stroke { line_dash: LineDash { segments, offset: 0 }, ..stroke },
            None => stroke,
        };
        self.sheet.frame().stroke(&path, stroke);
    }

    fn rounded_path(&self, rect: Rectangle, radius: f32) -> Path {
        let screen = self.rect(rect);
        let reach = (radius * self.unit).min(screen.width / 2.0).min(screen.height / 2.0).max(0.0);
        Path::rounded_rectangle(screen.position(), screen.size(), reach.into())
    }

    pub fn rounded(&mut self, rect: Rectangle, radius: f32, color: Color) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 || color.a <= 0.0 || rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let path = self.rounded_path(rect, radius);
        let color = self.tint(color);
        self.sheet.frame().fill(&path, color);
    }

    pub fn rounded_gradient(
        &mut self,
        rect: Rectangle,
        radius: f32,
        from: Point,
        to: Point,
        stops: &[(f32, Color)],
    ) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 {
            return;
        }
        let path = self.rounded_path(rect, radius);
        let ramp = stops.iter().fold(
            gradient::Linear::new(self.point(from), self.point(to)),
            |ramp, (offset, color)| ramp.add_stop(*offset, self.tint(*color)),
        );
        let fill = Fill { style: Gradient::Linear(ramp).into(), ..Fill::default() };
        self.sheet.frame().fill(&path, fill);
    }

    pub fn ring(&mut self, rect: Rectangle, radius: f32, width: f32, color: Color) {
        let inner = Rectangle {
            x: rect.x + width / 2.0,
            y: rect.y + width / 2.0,
            width: rect.width - width,
            height: rect.height - width,
        };
        self.rounded_outline(inner, (radius - width / 2.0).max(0.0), width, color);
    }

    pub fn disc(&mut self, centre: Point, radius: f32, color: Color) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 || color.a <= 0.0 {
            return;
        }
        let path = Path::circle(self.point(centre), radius * self.unit);
        let color = self.tint(color);
        self.sheet.frame().fill(&path, color);
    }

    pub fn rounded_outline(&mut self, rect: Rectangle, radius: f32, width: f32, color: Color) {
        if self.hidden() {
            return;
        }
        if self.alpha <= 0.0 || color.a <= 0.0 {
            return;
        }
        let path = self.rounded_path(rect, radius);
        let stroke = Stroke::default().with_width(width * self.unit).with_color(self.tint(color));
        self.sheet.frame().stroke(&path, stroke);
    }

    pub fn image(&mut self, handle: &Handle, rect: Rectangle, opacity: f32) {
        let opacity = opacity * self.alpha;
        if opacity <= 0.0 {
            return;
        }
        if self.ink != Ink::Images {
            return;
        }
        let bounds = self.rect(rect);
        let picture = iced::advanced::image::Image::new(handle.clone()).opacity(opacity);
        self.sheet.frame().draw_image(bounds, picture);
    }

    pub fn words(&mut self, words: Words<'_>) {
        if self.hidden() {
            return;
        }
        if words.content.is_empty() || self.alpha <= 0.0 || words.color.a <= 0.0 {
            return;
        }
        if words.tracks() {
            self.tracked_words(words);
            return;
        }
        let size = words.size * self.unit;
        let (position, align_x) =
            anchor_rtl(words.content, self.point(words.at), words.align, size, words.font);
        let text = Text {
            content: words.content.to_string(),
            position,
            max_width: words.width * self.unit,
            color: self.tint(words.color),
            size: size.into(),
            line_height: iced::widget::text::LineHeight::Relative(words.line),
            font: words.font,
            align_x,
            align_y: if words.middle {
                alignment::Vertical::Center
            } else {
                alignment::Vertical::Top
            },
            ..Text::default()
        };
        self.sheet.frame().fill_text(text);
    }
}

impl Pen<'_> {
    fn tracked_words(&mut self, words: Words<'_>) {
        let start = match words.align {
            TextAlignment::Center => words.at.x - words.advance() / 2.0,
            TextAlignment::Right => words.at.x - words.advance(),
            _ => words.at.x,
        };
        let mut before = String::new();
        for (index, glyph) in words.content.chars().enumerate() {
            let x = start
                + cached_line_width(&before, words.size, words.font)
                + words.tracking * index as f32;
            before.push(glyph);
            let mut cell = [0; GLYPH_BYTES];
            let single = Words {
                content: glyph.encode_utf8(&mut cell),
                at: Point::new(x, words.at.y),
                align: TextAlignment::Left,
                tracking: 0.0,
                ..words
            };
            self.words(single);
        }
    }
}

pub fn corners(rect: Rectangle) -> [Point; 4] {
    [
        rect.position(),
        Point::new(rect.x + rect.width, rect.y),
        Point::new(rect.x + rect.width, rect.y + rect.height),
        Point::new(rect.x, rect.y + rect.height),
    ]
}
