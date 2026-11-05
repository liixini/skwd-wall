use iced::{Point, Rectangle, Size};
use paper_control::Depth;

use super::shapes::{color, point, tip};
use crate::domain::strand::{Rect, frame};
use crate::frontend::animation::{EASE, eased, linear};
use crate::frontend::components::Pen;
use crate::frontend::strand::layout::view;
use crate::frontend::strand::state::{Live, Strand, lifted};
use crate::frontend::strand::tokens::{FRAME, LOOK, THREAD, TIMING};

const VISIBLE: f32 = 0.002;
const HATCH_FADE: f32 = 1.0 - LOOK.hatch_hold;

fn rectangle(rect: Rect) -> Rectangle {
    Rectangle::new(Point::new(rect.x, rect.y), Size::new(rect.w, rect.h))
}

pub fn stage_alpha(strand: &Strand) -> f32 {
    match strand.spool.entered {
        _ if strand.spool.leaving => strand.amount.value().clamp(0.0, 1.0),
        Some(at) => eased(strand.now, at, TIMING.enter),
        None => 0.0,
    }
}

pub fn backdrop(pen: &mut Pen<'_>, strand: &Strand) {
    let Some(stage) = strand.stage.as_ref() else { return };
    let [width, height] = strand.output;
    pen.image(&stage.image, rectangle(Rect::new(0.0, 0.0, width, height)), stage_alpha(strand));
}

pub fn dim(pen: &mut Pen<'_>, strand: &Strand) {
    let Some(tones) = strand.whole else { return };
    let amount = strand.amount.value().clamp(0.0, 1.0);
    if amount <= VISIBLE {
        return;
    }
    let [width, height] = strand.output;
    let shade = color(tones.shade.fade(LOOK.scrim * amount));
    pen.fill_rect(rectangle(Rect::new(0.0, 0.0, width, height)), shade);
}

fn hatch_alpha(strand: &Strand) -> Option<f32> {
    let share = linear(strand.now, strand.hatch?, TIMING.hatch);
    let envelope = if share < LOOK.hatch_rise {
        share / LOOK.hatch_rise
    } else if share < LOOK.hatch_hold {
        1.0
    } else {
        1.0 - (share - LOOK.hatch_hold) / HATCH_FADE
    };
    Some(LOOK.hatch_alpha * envelope)
}

struct Shot<'a> {
    live: &'a Live,
    depth: Depth,
    clip: Option<Rect>,
    alpha: f32,
}

fn shoot(pen: &mut Pen<'_>, strand: &Strand, shot: &Shot<'_>) {
    let live = shot.live;
    let shown = view(strand, live);
    let behind = shot.depth == Depth::Behind
        && strand.stage.as_ref().is_some_and(|stage| stage.matte.is_some());
    let paint = |pen: &mut Pen<'_>| {
        if let (Some(out), Some(proxy)) = (live.out.as_ref(), strand.outgoing.get(&live.id)) {
            let share = linear(strand.now, out.at, TIMING.out);
            let [width, tall] = frame(out.face);
            let scale = out.height / tall;
            let centre = [out.centre[0], out.centre[1] + LOOK.out_drop * share];
            let rect = Rect::centred(centre, [width * scale, tall * scale]);
            pen.image(
                &proxy.plain,
                rectangle(rect),
                shot.alpha * shown.alpha * (1.0 - EASE.at(share)),
            );
        }
        let Some(proxy) = strand.rasters.get(&live.id) else { return };
        let handle = match (&proxy.cut, behind) {
            (Some((_, cut)), true) => cut,
            _ => &proxy.plain,
        };
        pen.image(handle, rectangle(shown.rect), shot.alpha * shown.alpha);
    };
    match shot.clip {
        Some(clip) => pen.clipped(rectangle(clip), |pen| paint(pen)),
        None => paint(pen),
    }
}

pub fn widgets(pen: &mut Pen<'_>, strand: &Strand) {
    let stage = stage_alpha(strand);
    let [width, height] = strand.output;
    let lives = strand.widgets.iter().chain(strand.pull.as_ref().map(|pull| &pull.live));
    let mut fronts = Vec::new();
    for live in lives {
        let shown = view(strand, live);
        if shown.alpha <= VISIBLE || shown.rect.h <= VISIBLE {
            continue;
        }
        let parts = match &live.wipe {
            Some(wipe) => {
                let line = shown.rect.x + eased(strand.now, wipe.at, TIMING.wipe) * shown.rect.w;
                vec![
                    (live.depth, Some(Rect::new(0.0, 0.0, line, height))),
                    (wipe.from, Some(Rect::new(line, 0.0, width - line, height))),
                ]
            }
            None => vec![(live.depth, None)],
        };
        for (depth, clip) in parts {
            let shot = Shot { live, depth, clip, alpha: stage };
            if depth == Depth::Behind {
                shoot(pen, strand, &shot);
            } else {
                fronts.push(shot);
            }
        }
        let raised = lifted(live);
        let chosen = live.hidden && strand.selection.id == Some(live.id);
        let ghost = if raised > FRAME_GHOST {
            raised
        } else if chosen {
            1.0
        } else {
            0.0
        };
        if live.depth == Depth::Behind && live.wipe.is_none() && ghost > 0.0 {
            fronts.push(Shot {
                live,
                depth: Depth::Front,
                clip: None,
                alpha: stage * LOOK.ghost * ghost,
            });
        }
    }
    if let (Some(alpha), Some(hatch), Some(backdrop)) =
        (hatch_alpha(strand), strand.hatch_image.as_ref(), strand.stage.as_ref())
    {
        let unit = [width / backdrop.size[0] as f32, height / backdrop.size[1] as f32];
        let [left, top, columns, rows] = hatch.bounds.map(|side| side as f32);
        let at = Rect::new(left * unit[0], top * unit[1], columns * unit[0], rows * unit[1]);
        pen.image(&hatch.image, rectangle(at), alpha * stage);
    }
    for shot in &fronts {
        shoot(pen, strand, shot);
    }
}

const FRAME_GHOST: f32 = 0.01;

pub fn wipes(pen: &mut Pen<'_>, strand: &Strand) {
    for live in &strand.widgets {
        let (Some(wipe), Some(tones)) = (&live.wipe, live.chrome) else { continue };
        let rect = view(strand, live).rect;
        let x = rect.x + eased(strand.now, wipe.at, TIMING.wipe) * rect.w;
        let (top, bottom) = (rect.y - FRAME.wipe_reach, rect.bottom() + FRAME.wipe_reach);
        pen.line(point([x, top]), point([x, bottom]), THREAD.width, color(tones.accent));
        tip(pen, point([x, top]), &tones);
        tip(pen, point([x, bottom]), &tones);
    }
}
