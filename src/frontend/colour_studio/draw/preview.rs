use iced::{Color, Point, Rectangle, Vector};

use super::mock::{self, Desk, Space};
use super::pen::Pen;
use super::shapes::{self, Plate, Shadow, Touch, diamond};
use super::{Chrome, entering, faint};
use crate::frontend::colour_studio::controls::View;
use crate::frontend::colour_studio::layout::Frame;
use crate::frontend::colour_studio::motion::Crossfade;
use crate::frontend::colour_studio::motion::{BASE, CSS_EASE, EASE, Look, SLOW, eased, linear};
use crate::frontend::colour_studio::state::{ColourStudio, Scene, SceneKey};
use crate::frontend::colour_studio::text;
use crate::frontend::colour_studio::tokens::{
    CUT_INTRO, CUT_SHADOW, CUT_WIDTH, DESK_AT, DESK_DRAW, DESK_LINE, DESK_RIM, DESK_RIM_HOLD,
    DESK_RIM_WIDTH, DESK_STEP, FRAME_ALPHA, GRIP_GRAB, GRIP_SIZE, INTRO_ZOOM, PLATE_HEIGHT,
    PREVIEW, PREVIEW_EDGE, PREVIEW_EDGE_RADIUS, STRIP_DELAY, STRIP_EDGE, STRIP_HEIGHT, STRIP_INSET,
    STRIP_PITCH, STRIP_SIZE, TAG_INSET, TAG_INTRO,
};
use crate::frontend::theme::rgb_color;
use crate::i18n::tr;

const WINDOWS: [(Desk, Rectangle); 3] = [
    (Desk::Browser, Rectangle { x: 1.4, y: 1.4, width: 55.0, height: 59.7 }),
    (Desk::Chat, Rectangle { x: 58.2, y: 1.4, width: 40.4, height: 34.0 }),
    (Desk::Terminal, Rectangle { x: 58.2, y: 37.2, width: 40.4, height: 23.9 }),
];
const WINDOW_RADIUS: f32 = 0.9;
const WINDOW_SHADOW_DROP: f32 = 0.4;
const WINDOW_SHADOW_BLUR: f32 = 1.6;
const WINDOW_SHADOW_ALPHA: f32 = 0.44;
const WINDOW_REACH: f32 = 4.0;
const SHADOW_RINGS: usize = 8;
const CUT_SHADOW_ALPHA: f32 = 0.4;
const EDGE: f32 = 1.0;

pub fn cover(rect: Rectangle, aspect: f32) -> Rectangle {
    let share = crate::domain::studio::Frame::covering(rect.width / rect.height, aspect);
    Rectangle {
        x: rect.x + share.left * rect.width,
        y: rect.y + share.top * rect.height,
        width: share.width * rect.width,
        height: share.height * rect.height,
    }
}

fn shadow(pen: &mut Pen<'_>, rect: Rectangle, unit: f32) {
    let cast = Shadow {
        drop: WINDOW_SHADOW_DROP * unit,
        blur: WINDOW_SHADOW_BLUR * unit,
        alpha: WINDOW_SHADOW_ALPHA,
        rings: SHADOW_RINGS,
    };
    super::shapes::shadow(pen, rect, cast);
}

fn rim(pen: &mut Pen<'_>, rect: Rectangle, radius: f32, color: Color, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let inner = Rectangle {
        x: rect.x + DESK_RIM_WIDTH / 2.0,
        y: rect.y + DESK_RIM_WIDTH / 2.0,
        width: rect.width - DESK_RIM_WIDTH,
        height: rect.height - DESK_RIM_WIDTH,
    };
    pen.rounded_outline(inner, radius - DESK_RIM_WIDTH / 2.0, DESK_RIM_WIDTH, faint(color, alpha));
}

fn drawn(age: f32, rect: Rectangle, unit: f32) -> Option<Rectangle> {
    let share = linear(age, 0.0, DESK_DRAW);
    if share <= 0.0 {
        return None;
    }
    if share < DESK_LINE {
        let grow = EASE.at(share / DESK_LINE);
        return Some(Rectangle { width: rect.width * grow, height: DESK_RIM_WIDTH, ..rect });
    }
    let grow = EASE.at((share - DESK_LINE) / (1.0 - DESK_LINE));
    let reach = WINDOW_REACH * unit * grow;
    Some(Rectangle {
        x: rect.x - reach,
        y: rect.y - reach,
        width: rect.width + 2.0 * reach,
        height: DESK_RIM_WIDTH + (rect.height - DESK_RIM_WIDTH + reach) * grow + reach,
    })
}

fn rim_alpha(age: f32) -> f32 {
    let share = linear(age, 0.0, DESK_RIM);
    if share <= 0.0 || share >= 1.0 {
        return 0.0;
    }
    if share < DESK_RIM_HOLD {
        return 1.0;
    }
    1.0 - CSS_EASE.at((share - DESK_RIM_HOLD) / (1.0 - DESK_RIM_HOLD))
}

fn desk(pen: &mut Pen<'_>, scene: &Scene, intro: Option<f32>) {
    let picture = cover(PREVIEW, scene.aspect);
    pen.image(&scene.image, picture, 1.0);
    let space = Space::new(PREVIEW);
    let ring = rgb_color(scene.palette.accent);
    for (index, (which, at)) in WINDOWS.iter().enumerate() {
        let rect = space.at(at.x, at.y, at.width, at.height);
        let age = intro.map(|age| age - (DESK_AT + index as f32 * DESK_STEP));
        let radius = WINDOW_RADIUS * space.unit;
        let window = |pen: &mut Pen<'_>| {
            shadow(pen, rect, space.unit);
            pen.layer();
            pen.clipped(rect, |pen| {
                mock::app(pen, *which, rect, &scene.palette, scene.terminal, radius);
            });
            if let Some(age) = age {
                rim(pen, rect, radius, ring, rim_alpha(age));
            }
        };
        match age {
            Some(age) => {
                if let Some(reveal) = drawn(age, rect, space.unit) {
                    pen.clipped(reveal, window);
                }
            }
            None => window(pen),
        }
    }
}

fn scene(pen: &mut Pen<'_>, scene: &Scene, intro: Option<f32>) {
    match scene.view {
        View::Desktop => desk(pen, scene, intro),
        View::Browser => {
            mock::app(pen, Desk::Browser, PREVIEW, &scene.palette, scene.terminal, 0.0);
        }
        View::Chat => mock::app(pen, Desk::Chat, PREVIEW, &scene.palette, scene.terminal, 0.0),
        View::Music => super::apps::music(pen, PREVIEW, &scene.palette),
        View::Code => super::apps::code(pen, PREVIEW, &scene.palette),
    }
}

pub fn intro_zoom(frame: &Frame, aspect: f32) -> (Vector, f32) {
    let screen = frame.viewport_design();
    let backdrop = cover(screen, aspect);
    let shown = cover(PREVIEW, aspect);
    let zoom = backdrop.width / shown.width;
    let centre = screen.center();
    let middle = PREVIEW.center();
    let offset = Vector::new(
        centre.x - PREVIEW.x - zoom * (middle.x - PREVIEW.x),
        centre.y - PREVIEW.y - zoom * (middle.y - PREVIEW.y),
    );
    (offset, zoom)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Current,
    Previous,
    Live,
    LivePrevious,
}

fn faded_scene(pen: &mut Pen<'_>, old: Option<(&Scene, f32)>, intro: Option<f32>) {
    if let Some((old, fade)) = old {
        pen.faded(fade, |pen| scene(pen, old, intro));
    }
}

fn recoloured(from: &Scene, to: &Scene) -> bool {
    from.image == to.image
        && from.view == to.view
        && from.terminal == to.terminal
        && from.aspect.to_bits() == to.aspect.to_bits()
}

fn layered(fade: &Crossfade<SceneKey, Scene>, now: f32) -> (Option<Scene>, Option<(&Scene, f32)>) {
    match (fade.shown(), fade.previous(now)) {
        (Some(new), Some((old, left))) if recoloured(old, new) => {
            let palette = old.palette.mix(&new.palette, 1.0 - left);
            (Some(Scene { palette, ..new.clone() }), None)
        }
        (shown, previous) => (shown.cloned(), previous),
    }
}

fn content(pen: &mut Pen<'_>, studio: &ColourStudio, intro: Option<f32>, layer: Layer) {
    let shown = veiled(studio);
    let compared = studio.model.compare && shown > 0.0;
    let cut = PREVIEW.x + PREVIEW.width * studio.split.value(studio.now);
    let left = Rectangle { width: cut - PREVIEW.x, ..PREVIEW };
    let (current, previous) = match layer {
        Layer::Current | Layer::Previous => layered(&studio.scenes, studio.now),
        Layer::Live | Layer::LivePrevious => layered(&studio.compared, studio.now),
    };
    pen.clipped(PREVIEW, |pen| match layer {
        Layer::Current => {
            if let Some(current) = &current {
                scene(pen, current, intro);
            }
        }
        Layer::Previous => faded_scene(pen, previous, intro),
        Layer::Live if compared => pen.clipped(left, |pen| {
            if let Some(live) = &current {
                pen.faded(shown, |pen| scene(pen, live, intro));
            }
        }),
        Layer::LivePrevious if compared => pen.clipped(left, |pen| {
            pen.faded(shown, |pen| faded_scene(pen, previous, intro));
        }),
        Layer::Live | Layer::LivePrevious => {}
    });
}

fn veiled(studio: &ColourStudio) -> f32 {
    1.0 - studio.veil.value(studio.now)
}

fn tags(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, intro: Option<f32>) {
    let shown = veiled(studio);
    if !studio.model.compare || studio.model.live.is_none() || shown <= 0.0 {
        return;
    }
    pen.faded(shown, |pen| split_marks(pen, studio, chrome, intro));
}

fn split_marks(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, intro: Option<f32>) {
    let split = studio.split.value(studio.now);
    let cut = PREVIEW.x + PREVIEW.width * split;
    let mounted = studio.now - studio.mounts.tags;
    let (cut_age, tag_age) = match intro {
        Some(age) => (age - CUT_INTRO, age - TAG_INTRO),
        None => (mounted, mounted),
    };
    let grow = EASE.at(linear(cut_age, 0.0, BASE));
    pen.stretched(Point::new(cut, PREVIEW.y), (1.0, grow), |pen| {
        let line = Rectangle {
            x: cut - CUT_WIDTH / 2.0,
            y: PREVIEW.y,
            width: CUT_WIDTH,
            height: PREVIEW.height,
        };
        let halo =
            Rectangle { x: line.x - CUT_SHADOW, width: line.width + 2.0 * CUT_SHADOW, ..line };
        pen.fill_rect(halo, Color { a: CUT_SHADOW_ALPHA, ..Color::BLACK });
        pen.fill_rect(line, chrome.text);
        let grip = Point::new(cut, PREVIEW.center_y());
        let grab = if studio.sliding { GRIP_GRAB } else { 1.0 };
        pen.zoomed(grip, grab, |pen| diamond(pen, grip, GRIP_SIZE, chrome.text));
    });
    let middle = PREVIEW.y + TAG_INSET + PLATE_HEIGHT / 2.0;
    let now_plate = Plate::new(tr("colour-studio-now"), PREVIEW.x + TAG_INSET, middle);
    pen.faded(studio.tags[0].value(studio.now), |pen| {
        shapes::plate(pen, &now_plate, chrome.plate(false), Touch::default(), tag_age);
    });
    let name = text::source(&studio.model.source);
    let named = Plate::new(&name, 0.0, middle).ending(PREVIEW.x + PREVIEW.width - TAG_INSET);
    pen.faded(studio.tags[1].value(studio.now), |pen| {
        shapes::plate(pen, &named, chrome.plate(true), Touch::default(), tag_age);
    });
}

pub fn strip(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome) {
    let shown = entering(studio.content_age(), STRIP_DELAY, SLOW, Look::fade).alpha
        * (1.0 - studio.hidden.value(studio.now));
    if shown <= 0.0 {
        return;
    }
    let count = studio.swatches.len() as f32;
    let left = PREVIEW.x + PREVIEW.width - STRIP_INSET - count * STRIP_PITCH;
    let middle = PREVIEW.y + PREVIEW.height - STRIP_INSET - STRIP_HEIGHT / 2.0;
    pen.faded(shown, |pen| {
        for (index, tint) in studio.swatches.iter().enumerate() {
            let centre = Point::new(left + (index as f32 + 0.5) * STRIP_PITCH, middle);
            diamond(pen, centre, STRIP_SIZE + 2.0 * STRIP_EDGE, chrome.surface);
            diamond(pen, centre, STRIP_SIZE, tint.value(studio.now));
        }
    });
}

pub fn edge(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome) {
    let look = entering(studio.now, 0.0, SLOW, Look::fade);
    pen.looked(look, Point::ORIGIN, |pen| {
        pen.ring(PREVIEW_EDGE, PREVIEW_EDGE_RADIUS, EDGE, faint(chrome.outline, FRAME_ALPHA));
    });
}

pub fn draw(
    pen: &mut Pen<'_>,
    studio: &ColourStudio,
    chrome: &Chrome,
    frame: &Frame,
    layer: Layer,
) {
    let intro = studio.intro_age();
    let zoom = intro.map(|age| {
        let (offset, by) = intro_zoom(frame, studio.aspect());
        let left = 1.0 - eased(age, 0.0, INTRO_ZOOM);
        (offset * left, 1.0 + (by - 1.0) * left)
    });
    match zoom {
        Some((offset, by)) => pen.shifted(offset, |pen| {
            pen.zoomed(Point::new(PREVIEW.x, PREVIEW.y), by, |pen| {
                content(pen, studio, intro, layer);
            });
        }),
        None => content(pen, studio, None, layer),
    }
    if layer == Layer::Live {
        tags(pen, studio, chrome, intro);
    }
}
