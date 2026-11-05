use iced::{Color, Point, Rectangle, Vector};
use skwd_palette::studio::Area;

use super::Chrome;
use super::pen::{Pen, Words};
use super::preview::cover;
use super::shapes::{self, Plate, Touch};
use super::window::swap;
use crate::domain::studio::Pixels;
use crate::frontend::colour_studio::motion::{BASE, eased};
use crate::frontend::colour_studio::state::{ColourStudio, Load};
use crate::frontend::colour_studio::tokens::{
    PICK_DASH, PICK_EDGE, PICK_GHOST_ALPHA, PLATE_HEIGHT, PLATE_TEXT, PREVIEW, TAG_INSET,
};
use crate::frontend::ui::UI_FONT;
use crate::frontend::ui::cached_line_width;

const SAMPLE_STEP: f32 = 3.0;
const CHANNEL: f32 = 255.0;
const RGBA: usize = 4;

pub fn shown_rect(area: Area) -> Rectangle {
    Rectangle {
        x: PREVIEW.x + area.left * PREVIEW.width,
        y: PREVIEW.y + area.top * PREVIEW.height,
        width: (area.right - area.left) * PREVIEW.width,
        height: (area.bottom - area.top) * PREVIEW.height,
    }
}

fn sample(pixels: &Pixels, picture: Rectangle, at: Point) -> Color {
    let x = ((at.x - picture.x) / picture.width * pixels.width as f32).floor();
    let y = ((at.y - picture.y) / picture.height * pixels.height as f32).floor();
    let col = (x.max(0.0) as usize).min(pixels.width.saturating_sub(1));
    let row = (y.max(0.0) as usize).min(pixels.height.saturating_sub(1));
    let start = (row * pixels.width + col) * RGBA;
    let channel = |offset: usize| {
        f32::from(pixels.rgba.get(start + offset).copied().unwrap_or_default()) / CHANNEL
    };
    Color::from_rgb(1.0 - channel(0), 1.0 - channel(1), 1.0 - channel(2))
}

fn edge(
    pen: &mut Pen<'_>,
    pixels: Option<&Pixels>,
    picture: Rectangle,
    from: Point,
    to: Point,
    dash: Option<f32>,
    alpha: f32,
) {
    let length = (to.x - from.x).hypot(to.y - from.y);
    let steps = (length / SAMPLE_STEP).ceil().max(1.0) as usize;
    let along = Vector::new(
        (to.x - from.x) / length.max(f32::EPSILON),
        (to.y - from.y) / length.max(f32::EPSILON),
    );
    for step in 0..steps {
        let start = step as f32 * SAMPLE_STEP;
        let end = (start + SAMPLE_STEP).min(length);
        let a = Point::new(from.x + along.x * start, from.y + along.y * start);
        let b = Point::new(from.x + along.x * end, from.y + along.y * end);
        if let Some(dash) = dash
            && ((start / dash) as usize) % 2 == 1
        {
            continue;
        }
        let middle = Point::new(f32::midpoint(a.x, b.x), f32::midpoint(a.y, b.y));
        let ink = pixels.map_or(Color::WHITE, |pixels| sample(pixels, picture, middle));
        let shade = Color { a: alpha, ..ink };
        pen.line(a, b, PICK_EDGE, shade);
    }
}

fn frame(
    pen: &mut Pen<'_>,
    pixels: Option<&Pixels>,
    picture: Rectangle,
    rect: Rectangle,
    dash: Option<f32>,
    alpha: f32,
) {
    let inset = PICK_EDGE / 2.0;
    let corners = [
        Point::new(rect.x + inset, rect.y + inset),
        Point::new(rect.x + rect.width - inset, rect.y + inset),
        Point::new(rect.x + rect.width - inset, rect.y + rect.height - inset),
        Point::new(rect.x + inset, rect.y + rect.height - inset),
    ];
    for index in 0..corners.len() {
        edge(
            pen,
            pixels,
            picture,
            corners[index],
            corners[(index + 1) % corners.len()],
            dash,
            alpha,
        );
    }
}

pub fn tag_plate(studio: &ColourStudio) -> Plate<'static> {
    let width = cached_line_width(&studio.swaps.pick.now, PLATE_TEXT, UI_FONT);
    Plate::new("", PREVIEW.x + TAG_INSET, PREVIEW.y + TAG_INSET + PLATE_HEIGHT / 2.0).wide(width)
}

pub fn draw(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome) {
    let veil = studio.veil.value(studio.now);
    if veil <= 0.0 {
        return;
    }
    let Load::Ready(decoded) = &studio.decoded else {
        return;
    };
    let picture = cover(PREVIEW, studio.aspect());
    let [left, top, right, bottom] = studio.clip.map(|glide| glide.value(studio.now));
    let shown = shown_rect(Area { left, top, right, bottom });
    let pixels = Some(&decoded.pixels);
    pen.faded(veil, |pen| {
        pen.clipped(PREVIEW, |pen| {
            pen.image(&decoded.grey, picture, 1.0);
            pen.clipped(shown, |pen| pen.image(&studio.wallpaper.image, picture, 1.0));
            let picking = studio.picking.as_ref();
            let draft = picking.and_then(|pick| pick.draft);
            if draft.is_none()
                && picking.is_some()
                && let Some(saved) = studio.saved_area()
            {
                let ghost = shown_rect(studio.frame().to_preview(saved));
                frame(pen, pixels, picture, ghost, Some(PICK_DASH), PICK_GHOST_ALPHA);
            }
            if let Some(draft) = draft {
                let fade = eased(studio.now, studio.mounts.draft, BASE);
                frame(pen, pixels, picture, shown_rect(draft), None, fade);
            }
        });
        let plate = tag_plate(studio);
        let age = studio.now - studio.mounts.picking;
        let bounds = shapes::plate(pen, &plate, chrome.plate(false), Touch::default(), age);
        let at = Point::new(plate.text_left(), bounds.center_y());
        swap(
            pen,
            &studio.swaps.pick,
            studio.now,
            Words::new("", at, PLATE_TEXT, chrome.text).middle(),
        );
    });
}
