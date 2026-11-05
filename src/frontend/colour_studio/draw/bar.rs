use iced::{Color, Point, Rectangle, Vector};
use skwd_palette::studio::{Oklch, SHADE_RANGE, SHADE_STEP, from_oklch, to_oklch};

use super::pen::{Pen, Words};
use super::shapes::{self, Plate, Touch, diamond, hollow};
use super::window::swap;
use super::{Chrome, entering, faint, hover};
use crate::frontend::colour_studio::controls::{Hit, Knob, Row, TUNABLE, View};
use crate::frontend::colour_studio::motion::{BASE, EASE, Look, SLOW, linear};
use crate::frontend::colour_studio::state::ColourStudio;
use crate::frontend::colour_studio::text::{self, ROW_KEYS};
use crate::frontend::colour_studio::tokens::{
    BAND_DELAY, BAND_MARK_DELAY, CONTENT_RIGHT, KNOB_GAP, KNOB_GRAB, KNOB_SIZE, MARKER_LEFT,
    MARKER_SIZE, MARKER_TOP, NODE_GAP, NODE_PICKED, NODE_SEGMENT_DELAY, NODE_SIZE, PLATE_SMALL,
    POP_CONTROL, POP_CUT, POP_DETAIL_LINE, POP_DETAIL_SIZE, POP_GAP, POP_NODES, POP_PAD_BOTTOM,
    POP_PAD_SIDE, POP_PAD_TOP, POP_PLATE_GAP, POP_ROW_GAP, POP_SHIFT, POP_SLIDER, SEGMENT_DELAY,
    SWITCH_END, SWITCH_END_AT, SWITCH_KNOB, SWITCH_KNOB_EDGE, SWITCH_OFF_AT, SWITCH_ON_AT,
    SWITCH_WIDTH, SWITCH_WIRE_OFF, SWITCH_WIRE_ON, TAB_DELAY, TAB_HEIGHT, TAB_LABEL_LINE,
    TAB_LABEL_SIZE, TAB_LABEL_TOP, TAB_PAD, TAB_STEP_SIZE, TAB_VALUE_LINE, TAB_VALUE_SIZE,
    TAB_VALUE_TOP, TABS, TICK_GAP, TICK_SIZE, TRACK_ALPHA, TRACK_ON, TRACK_ON_RISE,
};
use crate::frontend::colour_studio::update::range;
use crate::frontend::components::lerp_color;
use crate::frontend::theme::rgb_color;
use crate::frontend::ui::UI_FONT;
use crate::frontend::ui::cached_line_width;
use crate::i18n::tr;

const TICKS: [f32; 3] = [0.0, 0.5, 1.0];
const EDGE: f32 = 1.0;
const EPSILON: f32 = 1e-6;
const NODE_COUNT: usize = 7;

pub struct Pop {
    pub rect: Rectangle,
    pub control: Rectangle,
}

fn keys(row: Row) -> (&'static str, &'static str) {
    let (_, label, detail) = ROW_KEYS[row.index()];
    (label, detail)
}

pub fn tab_needs() -> [f32; Row::ALL.len()] {
    Row::ALL.map(|row| {
        let label = cached_line_width(tr(keys(row).0), TAB_LABEL_SIZE, UI_FONT);
        let size =
            if matches!(row, Row::View | Row::FineTune) { TAB_STEP_SIZE } else { TAB_VALUE_SIZE };
        let value = text::values(row)
            .iter()
            .map(|value| cached_line_width(value, size, UI_FONT))
            .fold(0.0, f32::max);
        TAB_PAD + label.max(value)
    })
}

fn option_labels(row: Row) -> Vec<&'static str> {
    match row {
        Row::View => View::ALL.iter().map(|view| text::view(*view)).collect(),
        Row::FineTune => TUNABLE.iter().map(|role| text::role(*role)).collect(),
        _ => Vec::new(),
    }
}

fn picked_option(studio: &ColourStudio, row: Row) -> Option<usize> {
    match row {
        Row::View => View::ALL.iter().position(|view| *view == studio.view),
        Row::FineTune => TUNABLE.iter().position(|role| *role == studio.model.role),
        _ => None,
    }
}

fn control_size(row: Row) -> (f32, f32) {
    match row {
        Row::View | Row::FineTune => {
            let labels = option_labels(row);
            let plates: f32 =
                labels.iter().map(|label| Plate::new(label, 0.0, 0.0).small().width()).sum();
            let gaps = labels.len().saturating_sub(1) as f32 * POP_PLATE_GAP;
            (plates + gaps, PLATE_SMALL)
        }
        Row::Warmth | Row::Colour | Row::Contrast => (POP_SLIDER, POP_CONTROL),
        Row::Shade => (POP_NODES, POP_CONTROL),
        Row::Compare => (SWITCH_WIDTH, POP_CONTROL),
    }
}

pub fn pop(studio: &ColourStudio, row: Row) -> Pop {
    let tab = studio.tabs[row.index()];
    let detail = cached_line_width(tr(keys(row).1), POP_DETAIL_SIZE, UI_FONT);
    let (width, height) = control_size(row);
    let outer = Rectangle {
        x: 0.0,
        y: tab.y + tab.height + POP_GAP,
        width: detail.max(width) + 2.0 * POP_PAD_SIDE,
        height: POP_PAD_TOP + POP_DETAIL_LINE + POP_ROW_GAP + height + POP_PAD_BOTTOM,
    };
    let rect = Rectangle { x: tab.x.max(TABS.x).min(CONTENT_RIGHT - outer.width), ..outer };
    let control = Rectangle {
        x: rect.x + POP_PAD_SIDE,
        y: rect.y + POP_PAD_TOP + POP_DETAIL_LINE + POP_ROW_GAP,
        width,
        height,
    };
    Pop { rect, control }
}

pub fn pop_offset(studio: &ColourStudio, row: Row) -> Vector {
    let left = studio.pop_x.value(studio.now) - pop(studio, row).rect.x;
    let top = -POP_SHIFT * (1.0 - studio.pop.value(studio.now));
    Vector::new(left, top)
}

pub fn options(studio: &ColourStudio, row: Row) -> Vec<(usize, Plate<'static>)> {
    let control = pop(studio, row).control;
    let middle = control.y + control.height / 2.0;
    let mut left = control.x;
    option_labels(row)
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let plate = Plate::new(label, left, middle).small();
            left += plate.width() + POP_PLATE_GAP;
            (index, plate)
        })
        .collect()
}

pub fn track(studio: &ColourStudio, knob: Knob) -> Rectangle {
    let control = pop(studio, knob.row()).control;
    Rectangle { y: control.y + control.height / 2.0, height: 0.0, ..control }
}

pub fn fraction(knob: Knob, value: f32) -> f32 {
    let ((low, high), _) = range(knob);
    (value - low) / (high - low)
}

pub fn node_x(studio: &ColourStudio, index: usize) -> f32 {
    let rail = pop(studio, Row::Shade).control;
    rail.x + index as f32 * rail.width / (NODE_COUNT - 1) as f32
}

fn tab_size(studio: &ColourStudio, row: Row, content: &str, requested: f32) -> f32 {
    let room = (studio.tabs[row.index()].width - TAB_PAD).max(0.0);
    let width = cached_line_width(content, requested, UI_FONT);
    if width > room { requested * room / width } else { requested }
}

pub fn value_rect(studio: &ColourStudio, row: Row) -> Rectangle {
    let tab = studio.tabs[row.index()];
    let shown = &studio.swaps.values[row.index()].now;
    Rectangle {
        x: tab.x + TAB_PAD,
        y: tab.y + TAB_VALUE_TOP,
        width: cached_line_width(shown, tab_size(studio, row, shown, TAB_STEP_SIZE), UI_FONT),
        height: TAB_VALUE_LINE,
    }
}

fn chain(pen: &mut Pen<'_>, rail: Rectangle, at: f32, centre: f32, chrome: &Chrome, grow: f32) {
    let (low, high) = (at.min(centre), at.max(centre));
    let gap = |point: f32| if (point - at).abs() < EPSILON { KNOB_GAP } else { TICK_GAP };
    for pair in TICKS.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let parts = if at > from + EPSILON && at < to - EPSILON {
            [(from, at), (at, to)]
        } else if at <= from + EPSILON {
            [(from, from), (from, to)]
        } else {
            [(from, to), (to, to)]
        };
        for (start, end) in parts {
            let left = rail.x + start * rail.width + gap(start);
            let width = (end * rail.width - gap(end) - start * rail.width - gap(start)).max(0.0);
            if width <= 0.0 {
                continue;
            }
            let on = end - start > EPSILON && start >= low - EPSILON && end <= high + EPSILON;
            pen.stretched(Point::new(left, rail.y), (grow, 1.0), |pen| {
                if on {
                    pen.fill_rect(
                        Rectangle { x: left, y: rail.y - TRACK_ON_RISE, width, height: TRACK_ON },
                        chrome.primary,
                    );
                } else {
                    pen.fill_rect(
                        Rectangle { x: left, y: rail.y, width, height: EDGE },
                        faint(chrome.outline, TRACK_ALPHA),
                    );
                }
            });
        }
    }
    for tick in TICKS {
        let inside =
            tick >= low - EPSILON && tick <= high + EPSILON && (at - centre).abs() > EPSILON;
        let edge = if inside { chrome.primary } else { chrome.outline };
        let middle = Point::new(rail.x + tick * rail.width, rail.y + EDGE / 2.0);
        hollow(pen, middle, TICK_SIZE - EDGE, edge, EDGE);
    }
}

fn slider(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, knob: Knob, age: f32) {
    let ((_, _), neutral) = range(knob);
    let at = studio.knobs[knob as usize].value(studio.now);
    let delay = SEGMENT_DELAY.0 + knob.row().index() as f32 * SEGMENT_DELAY.1;
    let grow = EASE.at(linear(age, delay, SLOW));
    let rail = track(studio, knob);
    chain(pen, rail, at, fraction(knob, neutral), chrome, grow);
    let grabbed = if studio.knob_drag == Some(knob) { KNOB_GRAB } else { 1.0 };
    let middle = Point::new(rail.x + at * rail.width, rail.y + EDGE / 2.0);
    pen.zoomed(middle, grabbed, |pen| diamond(pen, middle, KNOB_SIZE, chrome.primary));
}

fn shade_fill(studio: &ColourStudio, step: i8) -> Option<Color> {
    let role = studio.model.role;
    let base = to_oklch(studio.untuned(role)?.get(role));
    let light = base.light + f32::from(step) * SHADE_STEP;
    Some(rgb_color(from_oklch(Oklch { light, ..base })))
}

fn shade(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, age: f32) {
    let control = pop(studio, Row::Shade).control;
    let middle = control.y + control.height / 2.0;
    let current = studio.model.tuning.shades[studio.model.role.index()];
    let gap = (node_x(studio, 1) - node_x(studio, 0) - 2.0 * NODE_GAP).max(0.0);
    let grow = EASE.at(linear(age, NODE_SEGMENT_DELAY, SLOW));
    for index in 0..NODE_COUNT - 1 {
        let left = node_x(studio, index) + NODE_GAP;
        pen.stretched(Point::new(left, middle), (grow, 1.0), |pen| {
            pen.fill_rect(
                Rectangle { x: left, y: middle, width: gap, height: EDGE },
                faint(chrome.outline, TRACK_ALPHA),
            );
        });
    }
    for (index, step) in (SHADE_RANGE.0..=SHADE_RANGE.1).enumerate() {
        let at = Point::new(node_x(studio, index), middle + EDGE / 2.0);
        let lit = hover(studio, &Hit::Node(step));
        let on = step == current;
        let size = if on { NODE_PICKED } else { NODE_SIZE + (NODE_PICKED - NODE_SIZE) * lit };
        if on && let Some(fill) = shade_fill(studio, step) {
            diamond(pen, at, size, fill);
        }
        let edge = if step == 0 && !on { chrome.muted } else { chrome.outline };
        hollow(pen, at, size + EDGE, edge, EDGE);
    }
}

fn switch(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome) {
    let control = pop(studio, Row::Compare).control;
    let share = studio.switch.value(studio.now);
    let mix = |off: f32, on: f32| off + (on - off) * share;
    let wire = lerp_color(chrome.outline, chrome.primary, share);
    let left = control.x;
    let middle = control.y + control.height / 2.0 + EDGE / 2.0;
    let start = left + mix(SWITCH_WIRE_OFF.0, SWITCH_WIRE_ON.0);
    let end = left + SWITCH_WIDTH - mix(SWITCH_WIRE_OFF.1, SWITCH_WIRE_ON.1);
    pen.fill_rect(
        Rectangle { x: start, y: middle - EDGE / 2.0, width: end - start, height: EDGE },
        wire,
    );
    hollow(pen, Point::new(left + SWITCH_END_AT, middle), SWITCH_END + EDGE, wire, EDGE);
    let knob = Point::new(left + mix(SWITCH_OFF_AT, SWITCH_ON_AT), middle);
    diamond(pen, knob, SWITCH_KNOB, faint(chrome.primary, share));
    hollow(pen, knob, SWITCH_KNOB + SWITCH_KNOB_EDGE, wire, SWITCH_KNOB_EDGE);
}

fn plates(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, row: Row, age: f32) {
    let picked = picked_option(studio, row);
    for (index, plate) in options(studio, row) {
        let touch = Touch { hover: hover(studio, &Hit::Option(row, index)), ..Touch::default() };
        shapes::plate(pen, &plate, chrome.plate(picked == Some(index)), touch, age);
    }
}

fn tab_value(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, row: Row) {
    let tab = studio.tabs[row.index()];
    let at = Point::new(tab.x + TAB_PAD, tab.y + TAB_VALUE_TOP);
    let (size, color) = match row {
        Row::View | Row::FineTune => {
            let lit = hover(studio, &Hit::Value(row));
            (TAB_STEP_SIZE, lerp_color(chrome.text, chrome.primary, lit))
        }
        Row::Compare => {
            let share = studio.switch.value(studio.now);
            (TAB_VALUE_SIZE, lerp_color(chrome.muted, chrome.primary, share))
        }
        _ => (TAB_VALUE_SIZE, chrome.muted),
    };
    let values = &studio.swaps.values[row.index()];
    let widest = if cached_line_width(&values.was, size, UI_FONT)
        > cached_line_width(&values.now, size, UI_FONT)
    {
        &values.was
    } else {
        &values.now
    };
    let size = tab_size(studio, row, widest, size);
    swap(
        pen,
        &studio.swaps.values[row.index()],
        studio.now,
        Words::new("", at, size, color).line(TAB_VALUE_LINE / size),
    );
}

pub fn draw(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, ready: f32) {
    let band = Rectangle {
        x: studio.band[0].value(studio.now),
        y: TABS.y,
        width: studio.band[1].value(studio.now),
        height: TAB_HEIGHT,
    };
    pen.looked(entering(ready, BAND_DELAY, BASE, Look::fade), Point::ORIGIN, |pen| {
        pen.fill_rect(band, chrome.variant);
    });
    pen.looked(entering(ready, BAND_MARK_DELAY, BASE, Look::fade), Point::ORIGIN, |pen| {
        let half = MARKER_SIZE / 2.0;
        let centre = Point::new(band.x + MARKER_LEFT + half, band.y + MARKER_TOP + half);
        diamond(pen, centre, MARKER_SIZE, chrome.primary);
    });
    for row in Row::ALL {
        let tab = studio.tabs[row.index()];
        let delay = TAB_DELAY.0 + row.index() as f32 * TAB_DELAY.1;
        pen.looked(entering(ready, delay, SLOW, Look::lift), Point::ORIGIN, |pen| {
            let label = tr(keys(row).0);
            let size = tab_size(studio, row, label, TAB_LABEL_SIZE);
            pen.words(
                Words::new(
                    label,
                    Point::new(tab.x + TAB_PAD, tab.y + TAB_LABEL_TOP),
                    size,
                    chrome.text,
                )
                .line(TAB_LABEL_LINE / size),
            );
            tab_value(pen, studio, chrome, row);
        });
    }
}

fn cut(rect: Rectangle) -> [Point; 6] {
    let (left, top) = (rect.x, rect.y);
    let (right, bottom) = (rect.x + rect.width, rect.y + rect.height);
    [
        Point::new(left + POP_CUT, top),
        Point::new(right, top),
        Point::new(right, bottom - POP_CUT),
        Point::new(right - POP_CUT, bottom),
        Point::new(left, bottom),
        Point::new(left, top + POP_CUT),
    ]
}

pub fn popover(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome) {
    let shown = studio.pop.value(studio.now) * (1.0 - studio.hidden.value(studio.now));
    if shown <= 0.0 || !studio.content_age().is_finite() {
        return;
    }
    let row = studio.row;
    let found = pop(studio, row);
    let age = studio.now - studio.pop_at.max(studio.row_at);
    let offset = pop_offset(studio, row);
    pen.faded(shown, |pen| {
        pen.shifted(offset, |pen| {
            pen.polygon(&cut(found.rect), chrome.surface);
            let at = Point::new(found.rect.x + POP_PAD_SIDE, found.rect.y + POP_PAD_TOP);
            pen.words(
                Words::new(tr(keys(row).1), at, POP_DETAIL_SIZE, chrome.muted)
                    .line(POP_DETAIL_LINE / POP_DETAIL_SIZE),
            );
            match row {
                Row::View | Row::FineTune => plates(pen, studio, chrome, row, age),
                Row::Warmth => slider(pen, studio, chrome, Knob::Warmth, age),
                Row::Colour => slider(pen, studio, chrome, Knob::Colour, age),
                Row::Contrast => slider(pen, studio, chrome, Knob::Contrast, age),
                Row::Shade => shade(pen, studio, chrome, age),
                Row::Compare => switch(pen, studio, chrome),
            }
        });
    });
}
