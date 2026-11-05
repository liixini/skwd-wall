use iced::{Point, Vector};
use paper_widgets::Tones;

use super::labels;
use super::model::{DisplayGeometry, Target};
use super::state::{Arrange, Extra, Hit, Press};
use crate::domain::strand::{Item, Rect};
use crate::frontend::animation::eased;
use crate::frontend::components::Pen;
use crate::frontend::strand::draw::selection::{
    beside, brackets, clamp_plate, gems, lines, retired, rule, tick_shown, toolbar, under,
};
use crate::frontend::strand::draw::shapes::{PLAIN, PlateLook, color, plate, point, tip};
use crate::frontend::strand::draw::thread::{self, Look};
use crate::frontend::strand::labels::{cancel, plate_width};
use crate::frontend::strand::tokens::{FRAME, PLATE, THREAD, TIMING};
use crate::i18n::tr;

const VISIBLE: f32 = 0.002;
const SHOWN: f32 = 0.01;
const DIAGRAM_ALPHA: f32 = 0.5;

pub fn thread(pen: &mut Pen<'_>, arrange: &Arrange) {
    spool(pen, arrange, &arrange.tones);
}

pub fn chrome(pen: &mut Pen<'_>, arrange: &Arrange) {
    let tones = arrange.tones;
    guides(pen, arrange, &tones);
    ticks(pen, arrange, &tones);
    whip(pen, arrange, &tones);
    let fade = arrange.fade();
    if fade > VISIBLE {
        pen.faded(fade, |pen| frame(pen, arrange, &tones));
    }
    panel(pen, arrange, &tones);
    if arrange.preview {
        let hover = arrange.hover.is_some_and(|(hit, _)| hit == Hit::Back);
        let shown = 1.0 - arrange.amount.value();
        let look = PlateLook { on: true, hover, open: 1.0 };
        pen.faded(shown, |pen| plate(pen, arrange.back_rect(), tr(labels::BACK), &look, &tones));
    }
}

fn guides(pen: &mut Pen<'_>, arrange: &Arrange, tones: &Tones) {
    let Some(target) = arrange.dragged() else { return };
    let me = arrange.rect(target);
    for guide in &arrange.guides {
        let seen = arrange.guide_seen.get(&guide.key()).copied().unwrap_or(arrange.now);
        let grow = eased(arrange.now, seen, TIMING.guide);
        rule(pen, guide, (me, grow), arrange.output, tones);
    }
}

fn ticks(pen: &mut Pen<'_>, arrange: &Arrange, tones: &Tones) {
    for (order, target) in arrange.targets().into_iter().enumerate() {
        if arrange.selection.id == Some(target) {
            continue;
        }
        let shown = if arrange.spool.leaving || arrange.preview {
            arrange.amount.value()
        } else {
            tick_shown(arrange.now, 0.0, order as f32)
        };
        if shown <= SHOWN {
            continue;
        }
        let rect = arrange.rect(target).padded(FRAME.tick_pad);
        brackets(pen, rect, color(tones.accent.fade(FRAME.tick_alpha * shown)));
    }
}

fn spool(pen: &mut Pen<'_>, arrange: &Arrange, tones: &Tones) {
    if arrange.spool.laid.is_none() {
        return;
    }
    let fade = arrange.fade();
    if fade <= VISIBLE {
        return;
    }
    let drop = (1.0 - arrange.amount.value()) * THREAD.exit_drop;
    let kinds: Vec<&str> =
        arrange.board.shelves.iter().map(|shelf| labels::family(shelf.family)).collect();
    let counts: Vec<usize> = arrange
        .board
        .shelves
        .iter()
        .map(|shelf| shelf.beads.iter().filter(|bead| bead.shown).count())
        .collect();
    let extras: Vec<(&str, bool)> = arrange
        .extras()
        .into_iter()
        .map(|extra| {
            (labels::extra(extra), extra == Extra::Displays && arrange.board.copy.is_some())
        })
        .collect();
    let cancel = cancel(arrange.board.steps);
    let look = Look {
        now: arrange.now,
        tones: *tones,
        hover: match arrange.hover {
            Some((Hit::Item(item), _)) => Some(item),
            _ => None,
        },
        undo: arrange.hover.is_some_and(|(hit, _)| hit == Hit::Undo),
        kinds: &kinds,
        counts: &counts,
        cancel: &cancel,
        extras: &extras,
    };
    pen.shifted(Vector::new(0.0, drop), |pen| {
        pen.faded(fade, |pen| {
            thread::line(pen, &arrange.spool, arrange.now, tones);
            thread::strikes(pen, &arrange.spool, &look);
            thread::items(pen, &arrange.spool, &look);
            thread::travels(pen, &arrange.spool, &look);
            label(pen, arrange, &look);
            thread::toast(pen, &arrange.spool, &look);
        });
    });
}

fn whip(pen: &mut Pen<'_>, arrange: &Arrange, tones: &Tones) {
    let Some(Press::Bead { bead, pulling: true, .. }) = &arrange.press else { return };
    if arrange.spool.drop.is_some() {
        return;
    }
    let slot = arrange.spool.slot(arrange.now, *bead);
    let to = arrange.pointer;
    let control =
        Point::new(f32::midpoint(slot[0], to[0]), f32::midpoint(slot[1], to[1]) + THREAD.whip);
    pen.curve(point(slot), control, point(to), THREAD.width, color(tones.accent));
    tip(pen, point(to), tones);
}

fn label(pen: &mut Pen<'_>, arrange: &Arrange, look: &Look<'_>) {
    if let Some((bead, at)) = arrange.spool.drop {
        thread::tag(pen, &arrange.spool, look, bead, (tr(labels::RELEASE), at, true));
        return;
    }
    let (Some((Hit::Item(Item::Bead(kind, style)), at)), None) = (arrange.hover, &arrange.press)
    else {
        return;
    };
    let Some(shelf) = arrange.board.shelves.get(kind) else { return };
    let Some(bead) = shelf.beads.get(style) else { return };
    let text = labels::bead(shelf.family, &bead.title, bead.shown);
    thread::tag(pen, &arrange.spool, look, (kind, style), (&text, at, false));
}

fn frame(pen: &mut Pen<'_>, arrange: &Arrange, tones: &Tones) {
    let now = arrange.now;
    retired(pen, &arrange.selection, now);
    let (Some(target), Some(rect)) = (arrange.selection.id, arrange.framed()) else {
        return;
    };
    lines(pen, rect, eased(now, arrange.selection.at, TIMING.select), tones);
    let gripping = match &arrange.press {
        Some(Press::Gem { corner, .. }) => Some(*corner),
        _ => None,
    };
    if target == Target::Picker {
        let hovered = match arrange.hover {
            Some((Hit::Gem(corner), _)) => Some(corner),
            _ => None,
        };
        gems(pen, rect, (arrange.selection.at, now), (gripping, hovered), tones);
    }
    let bar = arrange.bar(target, rect);
    let hover = match arrange.hover {
        Some((Hit::Tool(tool), _)) => Some(tool),
        _ => None,
    };
    let shaking = |tool| arrange.shaking(tool);
    toolbar(pen, (&arrange.selection, now), (rect, &bar), hover, shaking, tones);
    readout(pen, arrange, (target, rect), tones);
}

fn readout(pen: &mut Pen<'_>, arrange: &Arrange, framed: (Target, Rect), tones: &Tones) {
    let (target, rect) = framed;
    let output = arrange.output;
    let shown = arrange.objects[target.index()].target();
    match &arrange.press {
        Some(Press::Object { target: dragged, moved: true, .. }) if *dragged == target => {
            let text = arrange.anchor_text(target, shown);
            plate(pen, under(rect, plate_width(&text), output), &text, &PLAIN, tones);
        }
        Some(Press::Gem { corner, snapped, .. }) if target == Target::Picker => {
            let text = arrange
                .board
                .grid
                .map_or_else(|| labels::size(shown), |grid| labels::cells(cells_of(shown, grid)));
            let width = plate_width(&text) + FRAME.readout_pad;
            let look = PlateLook { on: *snapped, ..PLAIN };
            plate(pen, beside(rect, *corner, width, output), &text, &look, tones);
        }
        None if target == Target::Picker => {
            let Some(at) = arrange.flash else { return };
            let text = labels::size(shown);
            let width = plate_width(&text) + FRAME.readout_pad;
            let fade = 1.0 - eased(arrange.now, at + TIMING.readout - TIMING.label, TIMING.label);
            let spot = [rect.right() + FRAME.readout_side, rect.bottom() + FRAME.readout_drop];
            let look = PlateLook { on: true, ..PLAIN };
            pen.faded(fade, |pen| {
                plate(pen, clamp_plate(spot, width, output), &text, &look, tones);
            });
        }
        _ => {}
    }
}

fn cells_of(rect: Rect, grid: super::model::Grid) -> [usize; 2] {
    let count = |span: f32, axis: usize| {
        crate::domain::strand::cells(span, grid.cell[axis], grid.gap[axis], super::model::MAX_CELLS)
    };
    [count(rect.w, 0), count(rect.h, 1)]
}

fn panel(pen: &mut Pen<'_>, arrange: &Arrange, tones: &Tones) {
    let shown = if arrange.preview || arrange.spool.leaving { arrange.amount.value() } else { 1.0 };
    if shown <= VISIBLE {
        return;
    }
    pen.faded(shown, |pen| {
        for slot in arrange.panel() {
            let hover = slot.choice.is_some()
                && arrange.hover.is_some_and(|(hit, _)| slot.choice.map(Hit::Choice) == Some(hit));
            let look = PlateLook { on: slot.on, hover, open: 1.0 };
            plate(pen, slot.rect, &slot.label, &look, tones);
        }
        if let (Some(review), Some(copy)) = (arrange.review(), arrange.board.copy.as_ref())
            && let Some(display) = copy.review.as_ref()
        {
            diagram(pen, review.before, &display.before, tr("arrange-display-before"), tones);
            diagram(pen, review.after, &display.after, tr("arrange-display-after"), tones);
        }
    });
}

fn diagram(pen: &mut Pen<'_>, area: Rect, geometry: &DisplayGeometry, title: &str, tones: &Tones) {
    let (width, _) = geometry.viewport;
    let scale = area.w / width.max(1.0);
    let ink = color(tones.accent);
    let outline = |pen: &mut Pen<'_>, rect: Rect, alpha: f32| {
        let corners = [
            point([rect.x, rect.y]),
            point([rect.right(), rect.y]),
            point([rect.right(), rect.bottom()]),
            point([rect.x, rect.bottom()]),
            point([rect.x, rect.y]),
        ];
        pen.faded(alpha, |pen| pen.polyline(&corners, THREAD.width, ink));
    };
    let face = iced::Rectangle::new(Point::new(area.x, area.y), iced::Size::new(area.w, area.h));
    pen.fill_rect(face, color(tones.shade));
    outline(pen, area, DIAGRAM_ALPHA);
    let place = |rect: iced::Rectangle| {
        Rect::new(
            area.x + rect.x * scale,
            area.y + rect.y * scale,
            rect.width * scale,
            rect.height * scale,
        )
    };
    let floating = geometry.arrange.iter().chain(&geometry.swatch);
    for rect in geometry.picker.iter().chain([&geometry.bar, &geometry.search]).chain(floating) {
        outline(pen, place(*rect), 1.0);
    }
    let width = plate_width(title);
    let label = Rect::new(
        area.x + area.w / 2.0 - width / 2.0,
        area.y - PLATE.height - PLATE.gap,
        width,
        PLATE.height,
    );
    plate(pen, label, title, &PLAIN, tones);
}
