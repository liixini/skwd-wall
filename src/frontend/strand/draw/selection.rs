use iced::Color;
use paper_widgets::Tones;

use super::shapes::{PLAIN, PlateLook, bead, color, plate, point, tip};
use crate::domain::strand::{CORNERS, Guide, Rect, zone_top};
use crate::frontend::animation::{eased, lerp, linear};
use crate::frontend::components::Pen;
use crate::frontend::strand::labels::{self, plate_width};
use crate::frontend::strand::layout::view;
use crate::frontend::strand::motion::back;
use crate::frontend::strand::selection::{Bar, Selection};
use crate::frontend::strand::state::{Hit, Press, Strand};
use crate::frontend::strand::tokens::{FRAME, PLATE, THREAD, TIMING};

const FRAME_GONE: f32 = 0.001;
const SHOWN: f32 = 0.01;

pub fn ticks(pen: &mut Pen<'_>, strand: &Strand) {
    let Some(entered) = strand.spool.entered else { return };
    let mut index = 0.0;
    for live in strand.widgets.iter().filter(|live| live.dying.is_none()) {
        let order = index;
        index += 1.0;
        if strand.selection.id == Some(live.id) {
            continue;
        }
        let Some(tones) = live.chrome else { continue };
        let shown = if strand.spool.leaving {
            strand.amount.value()
        } else {
            tick_shown(strand.now, entered, order)
        };
        if shown <= SHOWN {
            continue;
        }
        let rect = view(strand, live).rect.padded(FRAME.tick_pad);
        brackets(pen, rect, color(tones.accent.fade(FRAME.tick_alpha * shown)));
    }
}

pub fn tick_shown(now: f32, entered: f32, order: f32) -> f32 {
    let start = entered + TIMING.tick_delay + order * TIMING.tick_stagger;
    eased(now, start, TIMING.item)
}

pub fn brackets(pen: &mut Pen<'_>, rect: Rect, ink: Color) {
    let arm = FRAME.tick;
    let corners = [
        ([rect.x, rect.y], [1.0, 1.0]),
        ([rect.right(), rect.y], [-1.0, 1.0]),
        ([rect.right(), rect.bottom()], [-1.0, -1.0]),
        ([rect.x, rect.bottom()], [1.0, -1.0]),
    ];
    for ([x, y], [dx, dy]) in corners {
        let bracket = [point([x + dx * arm, y]), point([x, y]), point([x, y + dy * arm])];
        pen.polyline(&bracket, THREAD.width, ink);
    }
}

pub fn guides(pen: &mut Pen<'_>, strand: &Strand) {
    let Some(tones) = strand.whole else { return };
    let dragged = match &strand.press {
        Some(Press::Widget { id, .. }) => strand.find(*id),
        Some(Press::Bead { .. }) => strand.pull.as_ref().map(|pull| &pull.live),
        _ => None,
    };
    let Some(live) = dragged else { return };
    let me = view(strand, live).rect;
    for guide in &strand.guides {
        let seen = strand.guide_seen.get(&guide.key()).copied().unwrap_or(strand.now);
        let grow = eased(strand.now, seen, TIMING.guide);
        rule(pen, guide, (me, grow), strand.output, &tones);
    }
}

pub fn rule(pen: &mut Pen<'_>, guide: &Guide, grown: (Rect, f32), output: [f32; 2], tones: &Tones) {
    let (me, grow) = grown;
    let ink = color(tones.accent.fade(FRAME.guide_alpha));
    let vertical = guide.vertical;
    let along = if vertical { me.centre()[1] } else { me.centre()[0] };
    let (start, end) = match guide.with {
        Some(other) if vertical => (
            me.y.min(other.y) - FRAME.guide_reach,
            me.bottom().max(other.bottom()) + FRAME.guide_reach,
        ),
        Some(other) => (
            me.x.min(other.x) - FRAME.guide_reach,
            me.right().max(other.right()) + FRAME.guide_reach,
        ),
        None if vertical => (0.0, output[1]),
        None => (0.0, output[0]),
    };
    let (from, to) = (lerp(along, start, grow), lerp(along, end, grow));
    let at =
        |value: f32| if vertical { point([guide.at, value]) } else { point([value, guide.at]) };
    pen.line(at(from), at(to), THREAD.width, ink);
    if guide.with.is_some() && grow > FRAME.guide_tips {
        for end in [start, end] {
            pen.faded(FRAME.guide_alpha, |pen| tip(pen, at(end), tones));
        }
    }
}

pub fn lines(pen: &mut Pen<'_>, frame: Rect, share: f32, tones: &Tones) {
    if share <= 0.0 {
        return;
    }
    let ink = color(tones.accent);
    let (half_x, half_y) = (frame.w / 2.0 * share, frame.h / 2.0 * share);
    let (left, top, right, bottom) = (frame.x, frame.y, frame.right(), frame.bottom());
    let arms = [
        [[left + half_x, top], [left, top], [left, top + half_y]],
        [[right - half_x, top], [right, top], [right, top + half_y]],
        [[right - half_x, bottom], [right, bottom], [right, bottom - half_y]],
        [[left + half_x, bottom], [left, bottom], [left, bottom - half_y]],
    ];
    for arm in arms {
        pen.polyline(&arm.map(point), THREAD.width, ink);
    }
}

pub fn corner(frame: Rect, index: usize) -> [f32; 2] {
    let sign = CORNERS[index];
    [
        if sign[0] < 0.0 { frame.x } else { frame.right() },
        if sign[1] < 0.0 { frame.y } else { frame.bottom() },
    ]
}

pub fn retired<Id: Copy + PartialEq>(pen: &mut Pen<'_>, selection: &Selection<Id>, now: f32) {
    let Some((rect, tones, at)) = selection.out else { return };
    let share = 1.0 - eased(now, at, TIMING.deselect);
    lines(pen, rect, share, &tones);
    for index in 0..CORNERS.len() {
        bead(pen, point(corner(rect, index)), FRAME.gem * share, 0.0, &tones);
    }
}

pub fn gems(
    pen: &mut Pen<'_>,
    rect: Rect,
    timing: (f32, f32),
    hands: (Option<usize>, Option<usize>),
    tones: &Tones,
) {
    let (at, now) = timing;
    let (gripping, hovered) = hands;
    for index in 0..CORNERS.len() {
        let start = at + index as f32 * TIMING.gem_stagger;
        let pop = back(linear(now, start, TIMING.gem_pop));
        let active = gripping == Some(index);
        let grow = if active || hovered == Some(index) { FRAME.gem_grow } else { 1.0 };
        let fill = if active { 1.0 } else { 0.0 };
        bead(pen, point(corner(rect, index)), FRAME.gem * pop * grow, fill, tones);
    }
}

pub fn frame(pen: &mut Pen<'_>, strand: &Strand) {
    let now = strand.now;
    retired(pen, &strand.selection, now);
    let (Some(live), Some(rect)) = (strand.selected(), strand.frame_rect()) else { return };
    let Some(tones) = live.chrome else { return };
    if live.born.is_some_and(|born| now < born) {
        return;
    }
    lines(pen, rect, eased(now, strand.selection.at, TIMING.select), &tones);
    let gripping = match &strand.press {
        Some(Press::Gem { corner, .. }) => Some(*corner),
        _ => None,
    };
    let hovered = match strand.hover {
        Some((Hit::Gem(corner), _)) => Some(corner),
        _ => None,
    };
    gems(pen, rect, (strand.selection.at, now), (gripping, hovered), &tones);
    let bar = strand.bar(live, rect);
    let hover = match strand.hover {
        Some((Hit::Tool(tool), _)) => Some(tool),
        _ => None,
    };
    let framed = (rect, &bar);
    toolbar(pen, (&strand.selection, now), framed, hover, |tool| strand.shaking(tool), &tones);
    readout(pen, strand, rect, &tones);
}

pub fn toolbar<Id: Copy + PartialEq, T: Copy + PartialEq>(
    pen: &mut Pen<'_>,
    timing: (&Selection<Id>, f32),
    framed: (Rect, &Bar<T>),
    hover: Option<T>,
    shaking: impl Fn(T) -> f32,
    tones: &Tones,
) {
    let (selection, now) = timing;
    let (rect, bar) = framed;
    let fold = selection.folding(now);
    if fold <= FRAME_GONE {
        return;
    }
    let stem = eased(now, selection.bar, TIMING.stem) * fold;
    let left = bar.centre - bar.total / 2.0;
    let x =
        (rect.x + rect.w / 2.0).clamp(left + FRAME.stem_inset, left + bar.total - FRAME.stem_inset);
    let from = if bar.below { rect.bottom() } else { rect.y };
    let to = if bar.below { bar.y } else { bar.y + PLATE.height };
    if stem > 0.0 && (to - from).abs() > FRAME.stem_min {
        pen.line(
            point([x, from]),
            point([x, lerp(from, to, stem)]),
            THREAD.width,
            color(tones.accent),
        );
        tip(pen, point([x, from]), tones);
    }
    for slot in &bar.plates {
        let open = selection.opening(now, slot);
        if open <= SHOWN {
            continue;
        }
        let centre = lerp(bar.centre, slot.rect.centre()[0], open) + shaking(slot.tool);
        let width = slot.rect.w * open;
        let alpha = (open * FRAME.unfurl_alpha).min(1.0) * if slot.dim { FRAME.dim } else { 1.0 };
        let look = PlateLook { on: slot.on, hover: hover == Some(slot.tool), open };
        let shown = Rect::new(centre - width / 2.0, bar.y, width, PLATE.height);
        pen.faded(alpha, |pen| plate(pen, shown, &slot.label, &look, tones));
    }
}

pub fn clamp_plate(at: [f32; 2], width: f32, output: [f32; 2]) -> Rect {
    let x = at[0].clamp(FRAME.edge, output[0] - FRAME.edge - width);
    let y = at[1].clamp(FRAME.edge, output[1] - FRAME.edge - PLATE.height);
    Rect::new(x, y, width, PLATE.height)
}

pub fn under(rect: Rect, width: f32, output: [f32; 2]) -> Rect {
    let mut y = rect.bottom() + FRAME.readout_gap;
    if y + PLATE.height > zone_top(output[1]) - FRAME.edge {
        y = rect.y - FRAME.readout_gap - PLATE.height;
    }
    if y < FRAME.edge {
        y = rect.y + FRAME.readout_gap;
    }
    let x = (rect.x + rect.w / 2.0 - width / 2.0).clamp(FRAME.edge, output[0] - FRAME.edge - width);
    Rect::new(x, y, width, PLATE.height)
}

pub fn beside(rect: Rect, index: usize, width: f32, output: [f32; 2]) -> Rect {
    let sign = CORNERS[index];
    let [gx, gy] = corner(rect, index);
    let x = if sign[0] > 0.0 { gx + FRAME.readout_side } else { gx - FRAME.readout_side - width };
    let y = if sign[1] > 0.0 {
        gy + FRAME.readout_drop
    } else {
        gy - FRAME.readout_drop - PLATE.height
    };
    clamp_plate([x, y], width, output)
}

fn readout(pen: &mut Pen<'_>, strand: &Strand, rect: Rect, tones: &Tones) {
    let Some(live) = strand.selected() else { return };
    let [width, height] = strand.output;
    if strand.press.is_none()
        && let Some((_, at)) = strand.flash.filter(|(id, _)| *id == live.id)
    {
        let text = labels::percent(live.h.target() / height);
        let plate_wide = plate_width(&text) + FRAME.readout_pad;
        let fade = 1.0 - eased(strand.now, at + TIMING.readout - TIMING.label, TIMING.label);
        let at = [rect.right() + FRAME.readout_side, rect.bottom() + FRAME.readout_drop];
        let look = PlateLook { on: true, ..PLAIN };
        pen.faded(fade, |pen| {
            plate(pen, clamp_plate(at, plate_wide, strand.output), &text, &look, tones);
        });
        return;
    }
    match &strand.press {
        Some(Press::Widget { id, moved: true, .. })
            if *id == live.id && strand.spool.drop.is_none() =>
        {
            let bounds = live.bounds();
            let [cx, cy] = bounds.centre();
            let (pin, _) =
                paper_widgets::pin([bounds.x, bounds.y, bounds.w, bounds.h], [width, height]);
            let [column, row] = pin.anchor.map(|anchor| (anchor * 2.0) as usize);
            let dx = [bounds.x, cx - width / 2.0, width - bounds.right()][column];
            let dy = [bounds.y, cy - height / 2.0, height - bounds.bottom()][row];
            let text = labels::anchor(row, column, [dx, dy]);
            let shown = under(rect, plate_width(&text), strand.output);
            plate(pen, shown, &text, &PLAIN, tones);
        }
        Some(Press::Gem { id, corner: index, snapped, .. }) if *id == live.id => {
            let text = labels::percent(live.h.target() / height);
            let plate_wide = plate_width(&text) + FRAME.readout_pad;
            let look = PlateLook { on: *snapped, ..PLAIN };
            plate(pen, beside(rect, *index, plate_wide, strand.output), &text, &look, tones);
        }
        _ => {}
    }
}
