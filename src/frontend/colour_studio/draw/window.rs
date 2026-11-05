use iced::{Color, Point, Rectangle, Vector};

use super::pen::{Pen, Words};
use super::shapes::{self, Ink, Plate, Shadow, Touch};
use super::{entering, faint, touch};
use crate::frontend::colour_studio::controls::Plate as PlateId;
use crate::frontend::colour_studio::motion::{BASE, FAST, Look, SLOW, Swap, eased};
use crate::frontend::colour_studio::state::ColourStudio;
use crate::frontend::colour_studio::text;
use crate::frontend::colour_studio::tokens::{
    ACTIONS_DELAY, ACTIONS_PAD, BUTTON_GAP, CLOSE_RIGHT, HEADER_HEIGHT, HEADER_TOP, INDEX_TINT,
    INDEX_WIDTH, NAME_INPUT, OUTLINE_MIX, PLATE_TEXT, PLATE_TEXT_DELAY, SHADOW_ALPHA, SHADOW_BLUR,
    SHADOW_DROP, WINDOW, WINDOW_EDGE_ALPHA,
};
use crate::frontend::components::lerp_color;
use crate::frontend::theme::Palette;
use crate::frontend::ui::UI_FONT;
use crate::frontend::ui::cached_line_width;
use crate::i18n::tr;

const SHADOW_RINGS: usize = 12;
const EDGE: f32 = 1.0;
const USE: &str = "colour-studio-use";
const UNDO: &str = "colour-studio-undo";
const SAVE_AS: &str = "colour-studio-save-as";
const CLOSE: &str = "colour-studio-close";

#[derive(Clone, Copy)]
pub struct Chrome {
    pub surface: Color,
    pub index: Color,
    pub variant: Color,
    pub text: Color,
    pub muted: Color,
    pub outline: Color,
    pub primary: Color,
}

impl Chrome {
    pub fn new(palette: &Palette) -> Self {
        Self {
            surface: palette.surface,
            index: lerp_color(palette.surface, palette.surface_variant, INDEX_TINT),
            variant: palette.surface_variant,
            text: palette.surface_text,
            muted: palette.outline,
            outline: lerp_color(palette.outline, palette.surface, OUTLINE_MIX),
            primary: palette.primary,
        }
    }

    pub fn plate(&self, picked: bool) -> Ink {
        if picked {
            Ink { rim: self.primary, face: self.primary, text: self.surface }
        } else {
            Ink { rim: self.primary, face: self.surface, text: self.text }
        }
    }
}

pub fn swap(pen: &mut Pen<'_>, swap: &Swap, now: f32, words: Words<'_>) {
    if let Some(leg) = swap.outgoing(now) {
        pen.faded(leg.alpha, |pen| {
            pen.shifted(Vector::new(leg.dx, leg.dy), |pen| {
                pen.words(Words { content: &swap.was, ..words });
            });
        });
    }
    let leg = swap.incoming(now);
    pen.faded(leg.alpha, |pen| {
        pen.shifted(Vector::new(leg.dx, leg.dy), |pen| {
            pen.words(Words { content: &swap.now, ..words });
        });
    });
}

pub struct Action {
    pub id: Option<PlateId>,
    pub plate: Plate<'static>,
    pub mounted: f32,
}

#[derive(Clone, Copy)]
enum Slot {
    Name,
    Apply(PlateId),
    Labelled(PlateId, &'static str),
}

fn entries(studio: &ColourStudio) -> Vec<(Slot, f32)> {
    let mounts = &studio.mounts;
    let close = (Slot::Labelled(PlateId::Close, tr(CLOSE)), 0.0);
    if !studio.content_age().is_finite() {
        return vec![close];
    }
    if studio.naming.is_some() {
        return vec![
            (Slot::Name, mounts.naming),
            (Slot::Labelled(PlateId::Save, tr("colour-studio-save")), mounts.naming),
            (Slot::Labelled(PlateId::Cancel, tr("colour-studio-cancel")), mounts.naming),
        ];
    }
    let mut found = Vec::new();
    if studio.picking.is_some() {
        if studio.saved_area().is_some() {
            let clear = Slot::Labelled(PlateId::ClearArea, tr("colour-studio-clear-area"));
            found.push((clear, mounts.picking));
        }
        let cancel = Slot::Labelled(PlateId::CancelPick, tr("colour-studio-cancel"));
        found.push((cancel, mounts.picking));
    } else {
        let apply = if studio.undoable() { PlateId::Undo } else { PlateId::Use };
        found.push((Slot::Labelled(PlateId::SaveAs, tr(SAVE_AS)), mounts.main));
        found.push((Slot::Apply(apply), mounts.main));
    }
    found.push(close);
    found
}

pub fn backing(found: &[Action]) -> Option<Rectangle> {
    let left = found.first()?.plate.left - ACTIONS_PAD;
    let right = WINDOW.width - CLOSE_RIGHT;
    Some(Rectangle { x: left, y: HEADER_TOP, width: right - left, height: HEADER_HEIGHT })
}

pub fn header_limit() -> f32 {
    let width = |key| Plate::new(tr(key), 0.0, 0.0).width();
    let apply = width(USE).max(width(UNDO));
    WINDOW.width
        - CLOSE_RIGHT
        - width(CLOSE)
        - width(SAVE_AS)
        - apply
        - 2.0 * BUTTON_GAP
        - ACTIONS_PAD
}

pub fn actions(studio: &ColourStudio) -> Vec<Action> {
    let middle = HEADER_TOP + HEADER_HEIGHT / 2.0;
    let mut right = WINDOW.width - CLOSE_RIGHT;
    let mut found: Vec<Action> = entries(studio)
        .into_iter()
        .rev()
        .map(|(slot, mounted)| {
            let (id, plate) = match slot {
                Slot::Name => (None, Plate::new("", 0.0, middle).wide(NAME_INPUT)),
                Slot::Apply(id) => {
                    let width = cached_line_width(&studio.swaps.apply.now, PLATE_TEXT, UI_FONT);
                    (Some(id), Plate::new("", 0.0, middle).wide(width))
                }
                Slot::Labelled(id, label) => (Some(id), Plate::new(label, 0.0, middle)),
            };
            let plate = plate.ending(right);
            right = plate.left - BUTTON_GAP;
            Action { id, plate, mounted }
        })
        .collect();
    found.reverse();
    found
}

fn shadow(pen: &mut Pen<'_>, alpha: f32) {
    let window = Rectangle { x: 0.0, y: 0.0, width: WINDOW.width, height: WINDOW.height };
    let cast = Shadow {
        drop: SHADOW_DROP,
        blur: SHADOW_BLUR,
        alpha: SHADOW_ALPHA * alpha,
        rings: SHADOW_RINGS,
    };
    shapes::shadow(pen, window, cast);
}

pub fn frame(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome) {
    let open = eased(studio.now, 0.0, SLOW);
    let window = Rectangle { x: 0.0, y: 0.0, width: WINDOW.width, height: WINDOW.height };
    shadow(pen, open);
    pen.fill_rect(window, faint(chrome.surface, open));
    let index = entering(studio.now, 0.0, SLOW, Look::slide_in);
    pen.looked(index, Point::ORIGIN, |pen| {
        pen.fill_rect(Rectangle { width: INDEX_WIDTH, ..window }, chrome.index);
    });
    pen.outline(
        &super::pen::corners(window),
        EDGE,
        faint(chrome.outline, WINDOW_EDGE_ALPHA * open),
    );
}

fn name_field(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, plate: &Plate<'_>) {
    let Some(name) = &studio.naming else {
        return;
    };
    let age = studio.now - studio.mounts.naming;
    let focus = Touch { focus: true, ..Touch::default() };
    let bounds = shapes::plate(pen, plate, chrome.plate(false), focus, age);
    let reveal = eased(age, BASE * PLATE_TEXT_DELAY, FAST);
    let at = Point::new(plate.text_left(), bounds.center_y());
    pen.faded(reveal, |pen| {
        if name.is_empty() {
            let hint = text::save_placeholder(studio);
            pen.words(Words::new(&hint, at, PLATE_TEXT, chrome.muted).middle());
        } else {
            pen.words(Words::new(name, at, PLATE_TEXT, chrome.text).middle());
        }
        let typed = cached_line_width(name, PLATE_TEXT, UI_FONT);
        let caret = at.x + typed + 1.0;
        let half = PLATE_TEXT / 2.0;
        pen.line(Point::new(caret, at.y - half), Point::new(caret, at.y + half), EDGE, chrome.text);
    });
}

pub fn chrome(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome) {
    let look = entering(studio.now, ACTIONS_DELAY, SLOW, Look::lift);
    let found = actions(studio);
    pen.looked(look, Point::ORIGIN, |pen| {
        if let Some(rect) = backing(&found) {
            pen.fill_rect(rect, chrome.surface);
        }
        for action in found {
            let Some(id) = action.id else {
                name_field(pen, studio, chrome, &action.plate);
                continue;
            };
            let picked = matches!(id, PlateId::Use | PlateId::Save);
            let age = studio.now - action.mounted;
            let bounds =
                shapes::plate(pen, &action.plate, chrome.plate(picked), touch(studio, id), age);
            if matches!(id, PlateId::Use | PlateId::Undo) {
                let ink = chrome.plate(picked).text;
                let reveal = eased(age, BASE * PLATE_TEXT_DELAY, FAST);
                pen.faded(reveal, |pen| {
                    swap(
                        pen,
                        &studio.swaps.apply,
                        studio.now,
                        Words::new("", bounds.center(), PLATE_TEXT, ink).centered().middle(),
                    );
                });
            }
        }
    });
}
