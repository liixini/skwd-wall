use iced::{Color, Point};
use skwd_palette::studio::oklch;

use super::pen::{Pen, Words};
use super::shapes::{diamond, hollow};
use super::{Chrome, entering, faint, hatch};
use crate::frontend::colour_studio::layout;
use crate::frontend::colour_studio::motion::{BASE, EASE, Look, SLOW, linear};
use crate::frontend::colour_studio::state::{ColourStudio, WHEEL_BINS};
use crate::frontend::colour_studio::text;
use crate::frontend::colour_studio::tokens::{
    WHEEL_BOX, WHEEL_CAPTION_GAP, WHEEL_CAPTION_LINE, WHEEL_CAPTION_SIZE, WHEEL_DELAY,
    WHEEL_DOT_DELAY, WHEEL_LEFT, WHEEL_NOTE_DELAY,
};
use crate::frontend::theme::rgb_color;

const RING: f32 = 0.36;
const TICKS: f32 = 0.44;
const TICK_MIN: f32 = 0.012;
const TICK_BASE: f32 = 0.014;
const TICK_GROW: f32 = 0.05;
const TICK_LIGHT: f32 = 0.72;
const TICK_CHROMA: f32 = 0.13;
const TICK_QUIET: f32 = 0.22;
const TICK_PRESENT: f32 = 0.01;
const RING_ALPHA: f32 = 0.28;
const RING_WIDTH: f32 = 1.5;
const RING_DASH: f32 = 3.0;
const RING_START: f32 = -90.0;
const FULL_TURN: f32 = 360.0;
const CENTRE: f32 = 6.0;
const CENTRE_ALPHA: f32 = 0.5;
const DOT_SIZES: [f32; 3] = [0.075, 0.058, 0.048];
const DOT_RIM: f32 = 2.0;
const DOT_GAP: f32 = 1.5;
const DEGREES_PER_BIN: f32 = 10.0;
const BIN_MIDDLE: f32 = 5.0;

fn polar(centre: Point, hue: f32, radius: f32) -> Point {
    let angle = hue.to_radians();
    Point::new(centre.x + radius * angle.cos(), centre.y - radius * angle.sin())
}

pub fn draw(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, ready: f32) {
    let Some(size) = studio.wheel_size else {
        return;
    };
    let mounted = ready.min(studio.now - studio.mounts.wheel);
    let top = studio.wheel_top.value(studio.now);
    let rect = layout::wheel_rect(top, size);
    let centre = rect.center();
    let radius = size * RING;
    let now = studio.now;
    pen.looked(entering(mounted, WHEEL_DELAY, SLOW, Look::turn), centre, |pen| {
        let sweep = EASE.at(linear(mounted, 0.0, SLOW)) * FULL_TURN;
        pen.arc(
            centre,
            radius,
            (RING_START, sweep),
            RING_WIDTH,
            faint(chrome.text, RING_ALPHA),
            Some(RING_DASH),
        );
        let origin = rect.position();
        let shape_age = now - studio.wheel.shape_at;
        if shape_age < BASE {
            let old = hatch::zones(&studio.wheel.was, size);
            hatch::draw(pen, origin, &old, chrome.text, None, hatch::fading(shape_age));
        }
        let fresh = hatch::zones(&studio.wheel.shape, size);
        let age = if shape_age.is_finite() { Some(shape_age) } else { Some(mounted) };
        hatch::draw(pen, origin, &fresh, chrome.text, age, 1.0);
        for (bin, glide) in studio.wheel.ticks.iter().enumerate().take(WHEEL_BINS) {
            let strength = glide.value(now);
            let hue = bin as f32 * DEGREES_PER_BIN + BIN_MIDDLE;
            let side = (size * (TICK_BASE + strength * TICK_GROW)).max(size * TICK_MIN);
            let alpha = if strength > TICK_PRESENT { 1.0 } else { TICK_QUIET };
            let color = rgb_color(oklch(TICK_LIGHT, TICK_CHROMA, hue));
            diamond(pen, polar(centre, hue, size * TICKS), side, faint(color, alpha));
        }
        diamond(pen, centre, CENTRE, faint(chrome.text, CENTRE_ALPHA));
        let dots = studio.wheel.dots.iter().zip(&studio.wheel.tints).zip(DOT_SIZES);
        let placed: Vec<(Point, Color, f32)> = dots
            .map(|(((x, y), tint), share)| {
                let at =
                    Point::new(centre.x + x.value(now) * radius, centre.y + y.value(now) * radius);
                (at, tint.value(now), size * share)
            })
            .collect();
        for (at, color, side) in placed.into_iter().rev() {
            pen.looked(entering(mounted, WHEEL_DOT_DELAY, BASE, Look::pop), at, |pen| {
                hollow(pen, at, side + 2.0 * DOT_GAP + DOT_RIM, chrome.text, DOT_RIM);
                diamond(pen, at, side, color);
            });
        }
    });
    let note = Point::new(WHEEL_LEFT + WHEEL_BOX / 2.0, rect.y + size + WHEEL_CAPTION_GAP);
    pen.looked(entering(mounted, WHEEL_NOTE_DELAY, SLOW, Look::lift), Point::ORIGIN, |pen| {
        pen.words(
            Words::new(text::wheel_note(studio), note, WHEEL_CAPTION_SIZE, chrome.muted)
                .centered()
                .wrap(WHEEL_BOX)
                .line(WHEEL_CAPTION_LINE),
        );
    });
}
