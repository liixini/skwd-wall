use iced::{Color, Point, Rectangle};

use super::pen::{Pen, Words};
use super::shapes::{self, Plate, diamond, hollow};
use super::{Chrome, entering, faint, hover, touch};
use crate::domain::studio::{Group, ModeChoice, Source};
use crate::frontend::colour_studio::controls::{Hit, Plate as PlateId};
use crate::frontend::colour_studio::layout::{self, Head};
use crate::frontend::colour_studio::motion::{BASE, EASE, Look, SLOW, linear};
use crate::frontend::colour_studio::state::ColourStudio;
use crate::frontend::colour_studio::text;
use crate::frontend::colour_studio::tokens::{
    INDEX_WIDTH, KID_DELAY, KID_DOT, KID_DOT_GAP, KID_DOTS, KID_GAP, KID_HEIGHT, KID_LEFT,
    KID_LINE, KID_MARK_LEFT, KID_PANE_RIGHT, KID_RIGHT, KID_SIZE, LABEL_LINE, MARK_DELAY,
    MODES_DELAY, MODES_LABEL_SIZE, MODES_LEFT, MODES_PLATE_GAP, MODES_ROW_GAP, NAV_MARK, NAV_NODE,
    NAV_NODE_EDGE, NODE_DELAY, NODE_LEFT, PANE_BAR_ALPHA, PANE_BAR_INSET, PANE_BAR_RIGHT,
    PANE_FADE, PANE_THUMB, PLATE_SMALL, RAIL_ALPHA, RAIL_LEFT, SAVED_DELAY, SECTION_DELAY,
    SECTION_GAP, SECTION_HEIGHT, SECTION_LEFT, SECTION_MARK_LEFT, SECTION_RIGHT, SECTION_SIZE,
};
use crate::frontend::components::lerp_color;
use crate::frontend::ui::UI_FONT;
use crate::frontend::ui::cached_line_width;

const TRAIL_LINE: f32 = 1.0;
const DOTS_WIDTH: f32 = KID_DOTS as f32 * KID_DOT + (KID_DOTS as f32 - 1.0) * KID_DOT_GAP;

#[derive(Debug, Clone, PartialEq)]
pub enum Pick {
    Source(Source),
    Saved(usize),
}

#[derive(Debug, Clone)]
pub struct Kid {
    pub pick: Pick,
    pub rect: Rectangle,
    pub on: bool,
    pub pane: bool,
    pub index: usize,
}

impl Kid {
    pub fn hit(&self) -> Hit {
        match &self.pick {
            Pick::Source(source) => Hit::Kid(source.key()),
            Pick::Saved(index) => Hit::Saved(*index),
        }
    }
}

pub fn pane_rect(studio: &ColourStudio) -> Option<(Group, Rectangle)> {
    let opened = studio.plan.opened.as_ref()?;
    let height = opened.pane?;
    let top = studio.opened_top.value(studio.now);
    Some((opened.group, Rectangle { x: 0.0, y: top, width: INDEX_WIDTH, height }))
}

pub fn saved_kids(studio: &ColourStudio) -> Vec<Kid> {
    let top = studio.head_top(Head::Saved) + SECTION_HEIGHT;
    (0..studio.shown_saved().len())
        .map(|index| Kid {
            pick: Pick::Saved(index),
            rect: Rectangle {
                x: 0.0,
                y: top + layout::kid_top(index),
                width: INDEX_WIDTH,
                height: KID_HEIGHT,
            },
            on: false,
            pane: false,
            index,
        })
        .collect()
}

pub fn kids(studio: &ColourStudio) -> Vec<Kid> {
    let Some(opened) = studio.plan.opened.as_ref() else {
        return Vec::new();
    };
    let top = studio.opened_top.value(studio.now);
    let scroll = if opened.pane.is_some() { studio.pane_scroll(opened.group) } else { 0.0 };
    opened
        .kids
        .iter()
        .enumerate()
        .map(|(index, source)| Kid {
            pick: Pick::Source(source.clone()),
            rect: Rectangle {
                x: 0.0,
                y: top + layout::kid_top(index) - scroll,
                width: INDEX_WIDTH,
                height: KID_HEIGHT,
            },
            on: *source == studio.model.source,
            pane: opened.pane.is_some(),
            index,
        })
        .collect()
}

fn kid_colors(studio: &ColourStudio, kid: &Kid) -> Option<[Color; KID_DOTS]> {
    match &kid.pick {
        Pick::Source(source) => studio.dots_for(source),
        Pick::Saved(index) => {
            let palette = studio.saved_picks.get(*index).copied().flatten()?;
            Some(
                crate::frontend::colour_studio::state::ACCENTS
                    .map(|role| crate::frontend::theme::rgb_color(palette.get(role))),
            )
        }
    }
}

fn kid_label(studio: &ColourStudio, kid: &Kid) -> String {
    match &kid.pick {
        Pick::Source(source) => text::kid(studio, source),
        Pick::Saved(index) => {
            studio.shown_saved().get(*index).map(|entry| entry.name.clone()).unwrap_or_default()
        }
    }
}

fn kid(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, kid: &Kid) {
    let middle = kid.rect.center_y();
    let right = if kid.pane { KID_PANE_RIGHT } else { KID_RIGHT };
    let dots = kid_colors(studio, kid);
    let reserved = if dots.is_some() { DOTS_WIDTH + KID_GAP } else { 0.0 };
    let width = INDEX_WIDTH - KID_LEFT - right - reserved;
    let base = if kid.on { chrome.primary } else { chrome.muted };
    let color =
        if kid.on { base } else { lerp_color(base, chrome.text, hover(studio, &kid.hit())) };
    let label = kid_label(studio, kid);
    pen.words(
        Words::new(&label, Point::new(KID_LEFT, middle), KID_SIZE, color)
            .middle()
            .wrap(width)
            .line(KID_LINE),
    );
    if let Some(dots) = dots {
        let start = INDEX_WIDTH - right - DOTS_WIDTH + KID_DOT / 2.0;
        for (index, dot) in dots.iter().enumerate() {
            let x = start + index as f32 * (KID_DOT + KID_DOT_GAP);
            diamond(pen, Point::new(x, middle), KID_DOT, *dot);
        }
    }
}

fn node(pen: &mut Pen<'_>, chrome: &Chrome, middle: f32) {
    let at = Point::new(NODE_LEFT + NAV_NODE / 2.0, middle);
    diamond(pen, at, NAV_NODE, chrome.surface);
    hollow(pen, at, NAV_NODE - NAV_NODE_EDGE, chrome.outline, NAV_NODE_EDGE);
}

fn rail(pen: &mut Pen<'_>, chrome: &Chrome, top: f32, bottom: f32, grow: f32) {
    if bottom <= top {
        return;
    }
    let length = (bottom - top) * grow;
    pen.line(
        Point::new(RAIL_LEFT, top),
        Point::new(RAIL_LEFT, top + length),
        TRAIL_LINE,
        faint(chrome.outline, RAIL_ALPHA),
    );
}

fn mark(pen: &mut Pen<'_>, chrome: &Chrome, left: f32, middle: f32) {
    diamond(pen, Point::new(left + NAV_MARK / 2.0, middle), NAV_MARK, chrome.primary);
}

fn section_label_end(label: &str) -> f32 {
    SECTION_LEFT + cached_line_width(label, SECTION_SIZE, UI_FONT) + SECTION_GAP
}

fn sections(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, ready: f32) {
    let look = entering(ready, SECTION_DELAY, SLOW, Look::lift);
    pen.looked(look, Point::ORIGIN, |pen| {
        for section in &studio.plan.sections {
            let top = studio.head_top(section.head);
            let middle = top + SECTION_HEIGHT / 2.0;
            let label = text::head(section.head);
            let on = section.open || section.holds;
            let lit = match section.head {
                Head::Group(group) => hover(studio, &Hit::Section(group)),
                Head::Saved => 0.0,
            };
            let color = if on { chrome.text } else { lerp_color(chrome.muted, chrome.text, lit) };
            pen.words(
                Words::new(label, Point::new(SECTION_LEFT, middle), SECTION_SIZE, color).middle(),
            );
            let grow = match section.head {
                Head::Group(group) => studio.trails[group.index()].value(studio.now),
                Head::Saved => 0.0,
            };
            if grow > 0.0 {
                let start = section_label_end(label);
                let end = INDEX_WIDTH - SECTION_RIGHT;
                let reach = start + (end - start) * grow;
                pen.gradient_line(
                    Point::new(start, middle),
                    Point::new(reach, middle),
                    TRAIL_LINE,
                    &[(0.0, chrome.primary), (1.0, faint(chrome.primary, 1.0 - grow))],
                );
            }
        }
    });
}

fn saved(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, ready: f32) {
    let mounted = ready.min(studio.now - studio.saved_at);
    for entry in saved_kids(studio) {
        let delay = entry.index as f32 * SAVED_DELAY;
        pen.looked(entering(mounted, delay, BASE, Look::lift), Point::ORIGIN, |pen| {
            kid(pen, studio, chrome, &entry);
        });
        pen.looked(entering(mounted, 0.0, BASE, Look::fade), Point::ORIGIN, |pen| {
            node(pen, chrome, entry.rect.center_y());
        });
    }
}

fn opened(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, ready: f32) {
    let Some(opened) = studio.plan.opened.as_ref() else {
        return;
    };
    let mounted = ready.min(studio.now - studio.group_at);
    let top = studio.opened_top.value(studio.now);
    let entries = kids(studio);
    let grow = EASE.at(linear(mounted, 0.0, SLOW));
    let first = top + KID_HEIGHT / 2.0;
    let last = top + opened.content() - KID_HEIGHT / 2.0;
    if let Some(height) = opened.pane {
        let rect = Rectangle { x: 0.0, y: top, width: INDEX_WIDTH, height };
        let scroll = studio.pane_scroll(opened.group);
        let look = entering(mounted, 0.0, BASE, Look::lift);
        pen.looked(look, Point::ORIGIN, |pen| {
            pen.clipped(rect, |pen| {
                rail(pen, chrome, first - scroll, last - scroll, grow);
                for entry in &entries {
                    if entry.rect.y + KID_HEIGHT < rect.y || entry.rect.y > rect.y + rect.height {
                        continue;
                    }
                    if !entry.on {
                        node(pen, chrome, entry.rect.center_y());
                    }
                    kid(pen, studio, chrome, entry);
                    if entry.on {
                        mark(pen, chrome, KID_MARK_LEFT, entry.rect.center_y());
                    }
                }
                scrollbar(pen, studio, chrome, rect, opened.max_scroll(), scroll);
                pen.layer();
                edges(pen, chrome, rect, scroll, opened.max_scroll());
            });
        });
        return;
    }
    rail(pen, chrome, first, last, grow);
    for entry in &entries {
        let delay = NODE_DELAY.0 + entry.index as f32 * NODE_DELAY.1;
        if !entry.on {
            pen.looked(entering(mounted, delay, BASE, Look::fade), Point::ORIGIN, |pen| {
                node(pen, chrome, entry.rect.center_y());
            });
        }
        let delay = KID_DELAY.0 + entry.index as f32 * KID_DELAY.1;
        pen.looked(entering(mounted, delay, BASE, Look::lift), Point::ORIGIN, |pen| {
            kid(pen, studio, chrome, entry);
        });
    }
}

fn scrollbar(
    pen: &mut Pen<'_>,
    studio: &ColourStudio,
    chrome: &Chrome,
    rect: Rectangle,
    max: f32,
    scroll: f32,
) {
    if max <= 0.0 {
        return;
    }
    let x = INDEX_WIDTH - PANE_BAR_RIGHT - TRAIL_LINE / 2.0;
    let (top, bottom) = (rect.y + PANE_BAR_INSET, rect.y + rect.height - PANE_BAR_INSET);
    pen.line(
        Point::new(x, top),
        Point::new(x, bottom),
        TRAIL_LINE,
        faint(chrome.outline, PANE_BAR_ALPHA),
    );
    let _ = studio;
    let travel = bottom - top - PANE_THUMB;
    let at = top + PANE_THUMB / 2.0 + travel * (scroll / max).clamp(0.0, 1.0);
    diamond(pen, Point::new(x, at), PANE_THUMB, chrome.muted);
}

fn edges(pen: &mut Pen<'_>, chrome: &Chrome, rect: Rectangle, scroll: f32, max: f32) {
    let clear = Color { a: 0.0, ..chrome.index };
    if scroll > 0.0 {
        let band = Rectangle { height: PANE_FADE, ..rect };
        pen.gradient_rect(
            band,
            Point::new(0.0, band.y),
            Point::new(0.0, band.y + PANE_FADE),
            &[(0.0, chrome.index), (1.0, clear)],
        );
    }
    if scroll < max {
        let band = Rectangle { y: rect.y + rect.height - PANE_FADE, height: PANE_FADE, ..rect };
        pen.gradient_rect(
            band,
            Point::new(0.0, band.y),
            Point::new(0.0, band.y + PANE_FADE),
            &[(0.0, clear), (1.0, chrome.index)],
        );
    }
}

pub fn mode_plates(studio: &ColourStudio) -> Vec<(ModeChoice, Plate<'static>)> {
    let top = studio.modes_top.value(studio.now);
    let middle = top + MODES_LABEL_SIZE * LABEL_LINE + MODES_ROW_GAP + PLATE_SMALL / 2.0;
    let mut left = MODES_LEFT;
    text::MODES
        .iter()
        .map(|(mode, _)| {
            let plate = Plate::new(text::mode(*mode), left, middle).small();
            left += plate.width() + MODES_PLATE_GAP;
            (*mode, plate)
        })
        .collect()
}

fn modes(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, ready: f32) {
    let look = entering(ready, MODES_DELAY, SLOW, Look::lift);
    pen.looked(look, Point::ORIGIN, |pen| {
        let top = studio.modes_top.value(studio.now);
        pen.words(Words::new(
            &studio.modes_label,
            Point::new(MODES_LEFT, top),
            MODES_LABEL_SIZE,
            chrome.muted,
        ));
        for (mode, plate) in mode_plates(studio) {
            let picked = mode == studio.model.mode_choice;
            shapes::plate(
                pen,
                &plate,
                chrome.plate(picked),
                touch(studio, PlateId::Mode(mode)),
                ready,
            );
        }
    });
}

pub fn draw(pen: &mut Pen<'_>, studio: &ColourStudio, chrome: &Chrome, ready: f32) {
    sections(pen, studio, chrome, ready);
    saved(pen, studio, chrome, ready);
    opened(pen, studio, chrome, ready);
    if let Some(section) = studio.plan.sections.iter().find(|section| section.holds) {
        let _ = section;
        let look = entering(ready, MARK_DELAY, SLOW, Look::fade);
        pen.looked(look, Point::ORIGIN, |pen| {
            mark(pen, chrome, SECTION_MARK_LEFT, studio.sec_mark.value(studio.now));
        });
    }
    if studio.kid_mark_on {
        let mounted = studio.now - studio.mounts.kid_mark;
        let look = entering(mounted, MARK_DELAY, SLOW, Look::fade);
        pen.looked(look, Point::ORIGIN, |pen| {
            mark(pen, chrome, KID_MARK_LEFT, studio.kid_mark.value(studio.now));
        });
    }
    modes(pen, studio, chrome, ready);
}
