use iced::{Point, Vector};
use paper_widgets::Tones;

use super::shapes::{PLAIN, PlateLook, bead, color, knot, plate, point, slit, tip};
use crate::domain::strand::{BEAD, END, HOVER, Item, KINDS, Kind, Rect, thread_y};
use crate::frontend::animation::{EASE, eased, lerp, linear};
use crate::frontend::components::{Pen, Words};
use crate::frontend::strand::labels::{self, ARRANGING, RELEASE, plate_width};
use crate::frontend::strand::layout::view;
use crate::frontend::strand::motion::back;
use crate::frontend::strand::spool::{BeadAt, FOOT_GAP, Spool, Strike};
use crate::frontend::strand::state::{Hit, Strand, bead_of};
use crate::frontend::strand::tokens::{FRAME, GROUP, LOOK, PLATE, THREAD, TIMING};
use crate::i18n::tr;

const VISIBLE: f32 = 0.002;
const SAMPLE_SLACK: f32 = 0.1;

pub struct Look<'a> {
    pub now: f32,
    pub tones: Tones,
    pub hover: Option<Item>,
    pub undo: bool,
    pub kinds: &'a [&'a str],
    pub counts: &'a [usize],
    pub cancel: &'a str,
    pub extras: &'a [(&'a str, bool)],
}

pub fn strand(pen: &mut Pen<'_>, strand: &Strand) {
    let (Some(tones), Some(_)) = (strand.band, strand.spool.laid.as_ref()) else { return };
    let fade = strand.fade();
    if fade <= VISIBLE {
        return;
    }
    let drop =
        if strand.spool.leaving { (1.0 - strand.amount.value()) * THREAD.exit_drop } else { 0.0 };
    let kinds: Vec<&str> = KINDS.iter().map(|kind| labels::kind(*kind)).collect();
    let counts: Vec<usize> = KINDS.iter().map(|kind| count(strand, *kind)).collect();
    let cancel = labels::cancel(strand.cancel_count());
    let look = Look {
        now: strand.now,
        tones,
        hover: match strand.hover {
            Some((Hit::Item(item), _)) => Some(item),
            _ => None,
        },
        undo: strand.hover.is_some_and(|(hit, _)| hit == Hit::Undo),
        kinds: &kinds,
        counts: &counts,
        cancel: &cancel,
        extras: &[],
    };
    pen.shifted(Vector::new(0.0, drop), |pen| {
        pen.faded(fade, |pen| {
            line(pen, &strand.spool, strand.now, &tones);
            whip(pen, strand, &tones);
            strikes(pen, &strand.spool, &look);
            items(pen, &strand.spool, &look);
            travels(pen, &strand.spool, &look);
            bead_label(pen, strand, &look);
            toast(pen, &strand.spool, &look);
            arranging(pen, strand, &tones);
        });
    });
}

pub fn line(pen: &mut Pen<'_>, spool: &Spool, now: f32, tones: &Tones) {
    let width = spool.output[0];
    let half = (width / 2.0 - END) * spool.reach(now);
    let spans = spool.laid.as_ref().map_or(&[][..], |laid| laid.spans.as_slice());
    let mut run: Vec<Point> = Vec::new();
    let mut x = width / 2.0 - half;
    let ink = color(tones.accent);
    while x <= width / 2.0 + half + SAMPLE_SLACK {
        if spans.iter().any(|span| x > span[0] && x < span[1]) {
            pen.polyline(&run, THREAD.width, ink);
            run.clear();
        } else {
            run.push(Point::new(x, spool.thread_at(x)));
        }
        x += THREAD.sample;
    }
    pen.polyline(&run, THREAD.width, ink);
    if spool.reach(now) > THREAD.reach_show {
        for x in [END, width - END] {
            knot(pen, Point::new(x, spool.thread_at(x)), THREAD.knot, tones);
        }
    }
}

fn whip(pen: &mut Pen<'_>, strand: &Strand, tones: &Tones) {
    let Some(pull) = &strand.pull else { return };
    let rect = view(strand, &pull.live).rect;
    let from = point(strand.slot(pull.live.face));
    let to = Point::new(rect.centre()[0], rect.bottom() + THREAD.tip_gap);
    let control = Point::new(pull.whip[0].value(), pull.whip[1].value());
    pen.curve(from, control, to, THREAD.width, color(tones.accent));
    tip(pen, to, tones);
}

pub fn strikes(pen: &mut Pen<'_>, spool: &Spool, look: &Look<'_>) {
    let ink = color(look.tones.accent);
    for thread in &spool.threads {
        let slot = spool.slot(look.now, thread.bead);
        let share = linear(look.now, thread.at, thread.strike.duration());
        let (end, slack) = match thread.strike {
            Strike::Shoot { to } => {
                let split = TIMING.shoot / thread.strike.duration();
                if share < split {
                    let out = EASE.at(share / split);
                    (blend(slot, to, out), THREAD.shoot * out)
                } else {
                    let back = EASE.at((share - split) / (1.0 - split));
                    (blend(to, slot, back), THREAD.shoot * (1.0 - back))
                }
            }
            Strike::Retract { from } => {
                let back = EASE.at(share);
                (blend(from, slot, back), THREAD.retract * (1.0 - back))
            }
        };
        let control =
            Point::new(f32::midpoint(slot[0], end[0]), f32::midpoint(slot[1], end[1]) + slack);
        pen.curve(point(slot), control, point(end), THREAD.width, ink);
        tip(pen, point(end), &look.tones);
    }
}

pub fn blend(from: [f32; 2], to: [f32; 2], share: f32) -> [f32; 2] {
    [lerp(from[0], to[0], share), lerp(from[1], to[1], share)]
}

fn count(strand: &Strand, kind: Kind) -> usize {
    let placed = strand
        .widgets
        .iter()
        .filter(|live| live.dying.is_none() && Kind::of(live.face) == kind)
        .count();
    placed + usize::from(strand.pull.as_ref().is_some_and(|pull| Kind::of(pull.live.face) == kind))
}

pub fn items(pen: &mut Pen<'_>, spool: &Spool, look: &Look<'_>) {
    let Some(laid) = spool.laid.as_ref() else { return };
    let tones = &look.tones;
    let travelling: Vec<BeadAt> = spool.travels.iter().map(|travel| travel.bead).collect();
    for placed in &laid.plan.items {
        let drop_in = spool.drop_in(look.now, placed.item);
        if drop_in <= 0.0 {
            continue;
        }
        let x = spool.item_x(look.now, placed.item);
        let dy = -THREAD.drop * (1.0 - drop_in);
        pen.faded(drop_in, |pen| match placed.item {
            Item::Bead(kind, style) => {
                if placed.off && !spool.item_moving(look.now, placed.item) {
                    return;
                }
                if travelling.contains(&(kind, style)) {
                    return;
                }
                let motion = &spool.beads[kind][style];
                let centre = Point::new(x, spool.thread_at(x) + dy);
                bead(pen, centre, motion.size.value() * drop_in, motion.fill.value(), tones);
            }
            Item::Group(kind) => {
                if placed.off {
                    return;
                }
                let placed_count = look.counts.get(kind).copied().unwrap_or_default();
                group(pen, spool, (kind, x, spool.thread_at(x) + dy, drop_in), placed_count, tones);
            }
            Item::Kind(kind) => {
                let width = placed.reach[1];
                let y = spool.thread_at(x + width / 2.0) - PLATE.height / 2.0 + dy;
                let folds = spool.folded(kind) || spool.open == Some(kind);
                let lit = matches!(look.hover, Some(Item::Kind(other) | Item::Group(other)) if other == kind);
                let shown = PlateLook { hover: folds && (spool.open == Some(kind) || lit), ..PLAIN };
                let label = look.kinds.get(kind).copied().unwrap_or_default();
                plate(pen, Rect::new(x, y, width, PLATE.height), label, &shown, tones);
            }
            Item::Extra(index) => {
                let width = placed.reach[1];
                let y = spool.plate_y(look.now, placed.item, x, width) + dy;
                let (label, on) = look.extras.get(index).copied().unwrap_or_default();
                let shown = PlateLook { on, hover: look.hover == Some(placed.item), open: 1.0 };
                plate(pen, Rect::new(x, y, width, PLATE.height), label, &shown, tones);
            }
            Item::Cancel | Item::Done => {
                let width = placed.reach[1];
                let y = spool.plate_y(look.now, placed.item, x, width) + dy;
                let done = placed.item == Item::Done;
                let pulse = !done && look.now < spool.pulse + TIMING.pulse;
                let hover = pulse || look.hover == Some(placed.item);
                let label = if done { tr(labels::DONE) } else { look.cancel };
                let shown = PlateLook { on: done, hover, open: 1.0 };
                plate(pen, Rect::new(x, y, width, PLATE.height), label, &shown, tones);
            }
        });
    }
}

fn group(
    pen: &mut Pen<'_>,
    spool: &Spool,
    at: (usize, f32, f32, f32),
    placed: usize,
    tones: &Tones,
) {
    let (kind, x, y, drop_in) = at;
    let motion = &spool.groups[kind];
    let reach = motion.size.value() * drop_in;
    bead(pen, Point::new(x, y), reach, motion.fill.value(), tones);
    let styles = spool.beads[kind].len();
    let width = labels::group_width(styles);
    let foot = y - HOVER - GROUP.stem;
    let top = foot - GROUP.height;
    pen.line(Point::new(x, y - reach), Point::new(x, foot), THREAD.width, color(tones.accent));
    let rect = Rect::new(x - width / 2.0, top, width, GROUP.height);
    pen.polygon(&slit(rect), color(tones.accent));
    let inner = Rect::new(
        rect.x + PLATE.rim,
        rect.y + PLATE.rim,
        rect.w - 2.0 * PLATE.rim,
        rect.h - 2.0 * PLATE.rim,
    );
    let filled = placed > 0;
    pen.polygon(&slit(inner), color(if filled { tones.accent } else { tones.shade }));
    let ink = if filled { tones.shade } else { tones.text };
    let text = labels::group(placed, styles);
    pen.words(
        Words::new(&text, Point::new(x, top + GROUP.height / 2.0), PLATE.text, color(ink))
            .centered()
            .middle(),
    );
}

pub fn travels(pen: &mut Pen<'_>, spool: &Spool, look: &Look<'_>) {
    let tones = &look.tones;
    for travel in &spool.travels {
        let slot = spool.slot(look.now, travel.bead);
        if look.now < travel.at {
            let lead = TIMING.collapse * TIMING.travel_lead;
            let pop = back(linear(look.now, travel.at - lead, lead));
            bead(pen, point(travel.from), BEAD * pop, 1.0, tones);
            continue;
        }
        let share = eased(look.now, travel.at, TIMING.travel);
        let mut at = blend(travel.from, slot, share);
        at[1] -= (std::f32::consts::PI * share).sin() * LOOK.travel_rise;
        let control = Point::new(
            f32::midpoint(slot[0], at[0]),
            slot[1].max(at[1]) + THREAD.travel * (1.0 - share),
        );
        pen.faded(THREAD.travel_alpha, |pen| {
            pen.curve(point(slot), control, point(at), THREAD.width, color(tones.accent));
        });
        bead(pen, point(at), BEAD, 1.0 - share * share, tones);
    }
}

fn bead_label(pen: &mut Pen<'_>, strand: &Strand, look: &Look<'_>) {
    let (bead, text, start, on) = if let Some((bead, at)) = strand.spool.drop {
        (bead, tr(RELEASE).to_owned(), at, true)
    } else if let (Some((Hit::Item(Item::Bead(kind, style)), at)), None) =
        (strand.hover, &strand.press)
    {
        let face = KINDS[kind].styles()[style];
        (bead_of(face), labels::bead(face, strand.placed(face)), at, false)
    } else {
        return;
    };
    tag(pen, &strand.spool, look, bead, (&text, start, on));
}

pub fn tag(
    pen: &mut Pen<'_>,
    spool: &Spool,
    look: &Look<'_>,
    bead: BeadAt,
    label: (&str, f32, bool),
) {
    let (text, start, on) = label;
    let shown = eased(look.now, start, TIMING.label);
    let [x, _] = spool.slot(look.now, bead);
    let width = plate_width(text);
    let left = (x - width / 2.0).clamp(FRAME.edge, spool.output[0] - FRAME.edge - width);
    let thread = spool.thread_at(x);
    let y = thread - FRAME.bead_label_gap - PLATE.height - FRAME.bead_label_rise * (1.0 - shown);
    pen.faded(shown, |pen| {
        let ink = color(look.tones.accent);
        pen.line(
            Point::new(x, y + PLATE.height),
            Point::new(x, thread - HOVER - FOOT_GAP),
            THREAD.width,
            ink,
        );
        let plate_look = PlateLook { on, ..PLAIN };
        plate(pen, Rect::new(left, y, width, PLATE.height), text, &plate_look, &look.tones);
    });
}

pub fn toast(pen: &mut Pen<'_>, spool: &Spool, look: &Look<'_>) {
    let Some(plates) = spool.toast_plates(look.now) else { return };
    let tones = &look.tones;
    pen.faded((plates.up * LOOK.toast_alpha).min(1.0), |pen| {
        let [x, _] = plates.slot;
        if plates.foot > plates.y + PLATE.height {
            let ink = color(tones.accent);
            pen.line(
                Point::new(x, plates.y + PLATE.height),
                Point::new(x, plates.foot),
                THREAD.width,
                ink,
            );
            tip(pen, Point::new(x, plates.y + PLATE.height), tones);
        }
        let label = Rect::new(plates.x, plates.y, plates.label_width, PLATE.height);
        plate(pen, label, &plates.label, &PLAIN, tones);
        let plate_look = PlateLook { on: true, hover: look.undo, open: 1.0 };
        plate(pen, plates.undo_rect(), tr(labels::UNDO), &plate_look, tones);
    });
}

fn arranging(pen: &mut Pen<'_>, strand: &Strand, tones: &Tones) {
    let Some(at) = strand.arranging else { return };
    let shown = eased(strand.now, at, TIMING.label)
        * (1.0 - eased(strand.now, at + TIMING.busy - TIMING.label, TIMING.label));
    notice(pen, strand.output, tr(ARRANGING), shown, tones);
}

pub fn notice(pen: &mut Pen<'_>, output: [f32; 2], text: &str, shown: f32, tones: &Tones) {
    let width = plate_width(text);
    let y =
        thread_y(output[1]) - LOOK.busy_rise - PLATE.height - FRAME.bead_label_rise * (1.0 - shown);
    let look = PlateLook { on: true, ..PLAIN };
    let rect = Rect::new(output[0] / 2.0 - width / 2.0, y, width, PLATE.height);
    pen.faded(shown, |pen| plate(pen, rect, text, &look, tones));
}
