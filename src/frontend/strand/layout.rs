use std::borrow::Cow;
use std::f32::consts::PI;

use paper_control::{Depth, Face};

use super::labels::{self, group_width, plate_width};
use super::selection::{Bar, BarPlate, shake};
use super::spool::Lineup;
use super::state::{Fate, Live, Shake, Strand, Tool, bead_of};
use super::tokens::{LOOK, PLATE, THREAD, TIMING};
use crate::domain::strand::{KINDS, Kind, KindSpec, Rect, Route, frame, obstacles, peers};
use crate::frontend::animation::{EASE, eased, lerp, linear};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub rect: Rect,
    pub alpha: f32,
}

pub fn view(strand: &Strand, live: &Live) -> View {
    let now = strand.now;
    let [width, tall] = frame(live.face);
    let mut centre = [live.x.value(), live.y.value()];
    let mut alpha = 1.0;
    let mut scale = live.h.value() / tall * live.lift.value() * live.squash.value();
    if let Some(swap) = &live.swap {
        scale *= 1.0 + swap.amp * (PI * linear(now, swap.at, TIMING.order)).sin();
    }
    if let Some(born) = live.born {
        let bloom = linear(now, born, TIMING.bloom);
        scale *= LOOK.bloom_from + (1.0 - LOOK.bloom_from) * EASE.at(bloom);
        alpha = (bloom * LOOK.bloom_alpha).min(1.0);
    }
    if let Some(wipe) = &live.wipe {
        let dip = (PI * linear(now, wipe.at, TIMING.wipe)).sin() * LOOK.dip;
        scale *= if live.depth == Depth::Behind { 1.0 - dip } else { 1.0 + dip };
    }
    if let Some(dying) = &live.dying {
        let share = linear(now, dying.at, dying.fate.duration());
        match dying.fate {
            Fate::Collapse => {
                scale *= 1.0 - share * share;
                alpha *= 1.0 - share * share;
            }
            Fate::Home => {
                let slot = strand.slot(live.face);
                let eased = EASE.at(share);
                centre = std::array::from_fn(|axis| lerp(dying.from[axis], slot[axis], eased));
                scale *= 1.0 - eased * LOOK.home_shrink;
                alpha *= 1.0 - eased * eased;
            }
        }
    }
    if let Some(reveal) = live.reveal {
        let rise = eased(now, reveal, TIMING.glyph);
        centre[1] -= LOOK.glyph_rise * (1.0 - rise);
        alpha *= rise;
    }
    View { rect: Rect::centred(centre, [width * scale, tall * scale]), alpha }
}

pub fn look_box(strand: &Strand, live: &Live) -> Rect {
    view(strand, live).rect
}

impl Strand {
    pub fn slot(&self, face: Face) -> [f32; 2] {
        self.spool.slot(self.now, bead_of(face))
    }

    pub fn reach(&self) -> f32 {
        self.spool.reach(self.now)
    }

    pub fn fade(&self) -> f32 {
        if self.spool.leaving { self.amount.value() } else { 1.0 }
    }

    pub fn hits_ready(&self) -> bool {
        !self.spool.leaving && !self.reverting && self.reach() > THREAD.reach_show
    }

    pub fn entering(&self) -> bool {
        !self.ready() || (!self.spool.leaving && !self.reverting && !self.hits_ready())
    }

    pub fn relayout(&mut self) {
        if !self.ready() {
            return;
        }
        let skip = super::edit::pressed_id(self.press.as_ref());
        let boxes = self
            .widgets
            .iter()
            .filter(|live| live.dying.is_none() && Some(live.id) != skip)
            .map(Live::bounds);
        let kinds: Vec<KindSpec> = KINDS
            .iter()
            .map(|kind| KindSpec {
                plate: plate_width(labels::kind(*kind)),
                group: group_width(kind.styles().len()),
                styles: kind.styles().len(),
            })
            .collect();
        let lineup = Lineup {
            kinds: &kinds,
            cancel: labels::cancel(self.cancel_count()),
            extras: &[],
            route: Route::rest(self.output[1], obstacles(boxes, self.output[1])),
            crossing: false,
        };
        let hold = self.spool.relayout(self.now, lineup);
        self.hold(hold);
    }

    pub fn frame_rect(&self) -> Option<Rect> {
        self.selection.frame_rect()
    }

    pub fn bar(&self, live: &Live, frame: Rect) -> Bar<Tool> {
        let snapshot = self.snapshot();
        let position = snapshot.iter().position(|spot| spot.id == live.id);
        let (above, below) = position.map_or((false, false), |at| {
            let found: Vec<usize> = peers(&snapshot, at).collect();
            (found.iter().any(|other| *other > at), found.iter().any(|other| *other < at))
        });
        let widest = Kind::of(live.face)
            .styles()
            .iter()
            .map(|face| plate_width(labels::style(*face)))
            .fold(0.0, f32::max);
        let mut plates = Vec::new();
        let mut x = 0.0;
        for tool in Tool::ALL {
            if tool.leads() {
                x += PLATE.group - PLATE.gap;
            }
            let label = labels::tool(tool, live.face);
            let width = if tool == Tool::Style { widest } else { plate_width(label) };
            let on = match tool {
                Tool::Behind => live.depth == Depth::Behind,
                Tool::Front => live.depth == Depth::Front,
                _ => false,
            };
            let dim = match tool {
                Tool::Raise => !above,
                Tool::Lower => !below,
                _ => false,
            };
            plates.push(BarPlate {
                tool,
                label: Cow::Borrowed(label),
                rect: Rect::new(x, 0.0, width, PLATE.height),
                on,
                dim,
                rank: 0,
            });
            x += width + PLATE.gap;
        }
        Bar::place(plates, frame, self.output)
    }

    pub fn shaking(&self, tool: Tool) -> f32 {
        let Some((at, which)) = self.shake else { return 0.0 };
        let hit = match which {
            Shake::Style => matches!(tool, Tool::Prev | Tool::Style | Tool::Next),
            Shake::Order => matches!(tool, Tool::Raise | Tool::Lower),
        };
        if hit { shake(self.now, at) } else { 0.0 }
    }
}
