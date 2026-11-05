use paper_control::{Depth, Face};

use super::edit::Effect;
use super::layout::view;
use super::spool::{Strike, Thread};
use super::state::{Hit, Live, Press, Pull, Strand, bead_of, whip};
use super::tokens::{DRAG_START, FRAME, KICKS, LOOK, SIZE, THREAD, TIMING};
use crate::domain::strand::{
    CORNERS, Grip, Guide, Item, KINDS, Kind, Rect, Spot, natural_height, snap, zone_top,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrandMsg {
    Press([f32; 2]),
    Move([f32; 2]),
    Release([f32; 2]),
    Leave,
}

fn distance(one: [f32; 2], other: [f32; 2]) -> f32 {
    (one[0] - other[0]).hypot(one[1] - other[1])
}

impl Strand {
    pub fn update(&mut self, message: StrandMsg) -> Effect {
        match message {
            StrandMsg::Press(at) => self.press_at(at),
            StrandMsg::Move(at) => self.move_to(at),
            StrandMsg::Release(at) => return self.release(at),
            StrandMsg::Leave => self.abandon(),
        }
        Effect::None
    }

    pub fn widget_at(&self, at: [f32; 2]) -> Option<u32> {
        let mut found: Vec<(usize, &Live)> = self
            .widgets
            .iter()
            .enumerate()
            .filter(|(_, live)| live.dying.is_none())
            .filter(|(_, live)| view(self, live).rect.padded(FRAME.widget_pad).contains(at))
            .collect();
        found.sort_by(|(one_index, one), (other_index, other)| {
            let front = |live: &Live| live.depth == Depth::Front;
            front(other)
                .cmp(&front(one))
                .then(other_index.cmp(one_index))
                .then(one.bounds().area().total_cmp(&other.bounds().area()))
        });
        found.first().map(|(_, live)| live.id)
    }

    pub fn hit(&self, at: [f32; 2]) -> Option<Hit> {
        if let (Some(live), Some(frame)) = (self.selected(), self.frame_rect()) {
            let bar = self.bar(live, frame);
            let hit = bar.plates.iter().find(|plate| {
                self.selection.opening(self.now, plate) > FRAME.hit_open && plate.rect.contains(at)
            });
            if let Some(plate) = hit {
                return Some(Hit::Tool(plate.tool));
            }
            for (corner, sign) in CORNERS.iter().enumerate() {
                let gem = [
                    if sign[0] < 0.0 { frame.x } else { frame.right() },
                    if sign[1] < 0.0 { frame.y } else { frame.bottom() },
                ];
                let reach = Rect::centred(gem, [2.0 * FRAME.gem_hit; 2]);
                if reach.contains(at) {
                    return Some(Hit::Gem(corner));
                }
            }
        }
        if let Some(toast) =
            self.spool.toast_plates(self.now).filter(|toast| toast.up > FRAME.hit_open)
            && self.hits_ready()
            && toast.undo_rect().contains(at)
        {
            return Some(Hit::Undo);
        }
        let widget = self.widget_at(at);
        if widget.is_none()
            && self.hits_ready()
            && let Some(item) = self.spool.hit(self.now, at)
        {
            return Some(Hit::Item(item));
        }
        widget.map(Hit::Widget)
    }

    fn press_at(&mut self, at: [f32; 2]) {
        if self.reverting || self.spool.leaving || !self.ready() {
            return;
        }
        let hit = self.hit(at);
        let keeps = matches!(hit, Some(Hit::Item(Item::Kind(kind) | Item::Group(kind))) if self.spool.open == Some(kind));
        if !keeps {
            self.wind();
        }
        self.press = Some(match hit {
            Some(Hit::Gem(corner)) => {
                let Some(live) = self.selected() else { return };
                let grip = Grip::new(live.bounds(), corner, at);
                let id = live.id;
                self.selection.fold_now(self.now);
                self.hold(TIMING.fold);
                Press::Gem { id, corner, grip, snapped: false, before: self.snapshot() }
            }
            Some(
                hit @ (Hit::Tool(_)
                | Hit::Undo
                | Hit::Item(Item::Cancel | Item::Done | Item::Extra(_))),
            ) => Press::Plate(hit),
            Some(Hit::Item(Item::Kind(kind) | Item::Group(kind))) => {
                self.unspool(kind);
                Press::Kind
            }
            Some(Hit::Item(Item::Bead(kind, style))) => {
                Press::Bead { face: KINDS[kind].styles()[style], start: at, pulling: false }
            }
            Some(Hit::Widget(id)) => {
                if self.selection.id != Some(id) {
                    self.select(id);
                    self.raise_up(id);
                }
                let centre = self.find(id).map_or(at, |live| live.bounds().centre());
                let grab = [centre[0] - at[0], centre[1] - at[1]];
                Press::Widget { id, grab, start: at, moved: false, before: self.snapshot() }
            }
            None => {
                self.deselect();
                Press::Empty
            }
        });
        self.relayout();
    }

    fn move_to(&mut self, at: [f32; 2]) {
        let Some(press) = self.press.take() else {
            self.hover_at(at);
            return;
        };
        let press = match press {
            Press::Bead { face, start, pulling } => {
                let pulling = pulling
                    || (!self.placed(face) && !self.full() && distance(at, start) > DRAG_START);
                if pulling {
                    if self.pull.is_none() {
                        self.start_pull(face);
                    }
                    self.move_pull(at);
                }
                Press::Bead { face, start, pulling }
            }
            Press::Widget { id, grab, start, moved, before } => {
                let moved = moved || distance(at, start) > DRAG_START;
                if moved {
                    self.drag(id, at, grab);
                }
                Press::Widget { id, grab, start, moved, before }
            }
            Press::Gem { id, corner, grip, snapped, before } => {
                let snapped = self.resize(id, &grip, at, snapped);
                Press::Gem { id, corner, grip, snapped, before }
            }
            other => other,
        };
        self.press = Some(press);
    }

    fn hover_at(&mut self, at: [f32; 2]) {
        let hit = self.hit(at);
        if self.hover.map(|(current, _)| current) != hit {
            self.hover = hit.map(|hit| (hit, self.now));
            self.hold(TIMING.label);
        }
        if let Some(Hit::Item(Item::Kind(kind) | Item::Group(kind))) = hit {
            self.unspool(kind);
        }
        let hovered = match hit {
            Some(Hit::Item(Item::Bead(kind, style))) => Some(KINDS[kind].styles()[style]),
            _ => None,
        };
        self.size_beads(hovered, None);
    }

    pub fn size_beads(&mut self, hovered: Option<Face>, dropping: Option<Face>) {
        let hovered_kind = match self.hover {
            Some((Hit::Item(Item::Kind(kind) | Item::Group(kind)), _)) => Some(kind),
            _ => None,
        };
        self.spool.size_beads(hovered.map(bead_of), hovered_kind, dropping.map(bead_of));
    }

    fn start_pull(&mut self, face: Face) {
        let slot = self.slot(face);
        self.seq += 1;
        let spot = Spot {
            id: self.seq,
            face,
            depth: Kind::of(face).depth(),
            centre: slot,
            height: natural_height(face, self.output),
        };
        let mut live = Live::new(spot);
        live.born = Some(self.now);
        live.reveal = Some(self.now);
        live.lift.aim(LOOK.lift);
        self.pull = Some(Pull { live, whip: whip(slot) });
        self.deselect();
        self.hold(TIMING.bloom + super::state::reveal_span());
    }

    fn others(&self, id: u32) -> Vec<Rect> {
        self.widgets
            .iter()
            .filter(|live| live.dying.is_none() && live.id != id)
            .map(Live::bounds)
            .collect()
    }

    fn snapped(&mut self, id: u32, bounds: Rect, wanted: [f32; 2]) -> [f32; 2] {
        let others = self.others(id);
        let snapped = snap([bounds.w, bounds.h], wanted, &others, self.output);
        for guide in &snapped.guides {
            self.guide_seen.entry(guide.key()).or_insert(self.now);
        }
        let keys: Vec<(bool, i32)> = snapped.guides.iter().map(Guide::key).collect();
        self.guide_seen.retain(|key, _| keys.contains(key));
        self.guides = snapped.guides;
        self.hold(TIMING.guide);
        snapped.centre
    }

    fn move_pull(&mut self, at: [f32; 2]) {
        let Some(pull) = self.pull.as_ref() else { return };
        let (id, bounds) = (pull.live.id, pull.live.bounds());
        let centre = self.snapped(id, bounds, at);
        if let Some(pull) = self.pull.as_mut() {
            pull.live.aim(centre);
        }
        self.zone(at, None);
    }

    fn drag(&mut self, id: u32, at: [f32; 2], grab: [f32; 2]) {
        let Some(live) = self.find_mut(id) else { return };
        if (live.lift.target() - LOOK.lift).abs() > f32::EPSILON {
            live.hidden = false;
            live.lift.aim(LOOK.lift);
            self.selection.fold_now(self.now);
            self.hold(TIMING.fold);
        }
        let Some(live) = self.find(id) else { return };
        let (bounds, face) = (live.bounds(), live.face);
        let centre = self.snapped(id, bounds, [at[0] + grab[0], at[1] + grab[1]]);
        if let Some(live) = self.find_mut(id) {
            live.aim(centre);
        }
        self.zone(at, Some(face));
    }

    fn zone(&mut self, at: [f32; 2], dropping: Option<Face>) {
        let top = zone_top(self.output[1]);
        let inside = at[1] > top;
        let squash = if inside { LOOK.squash } else { 1.0 };
        let target = match (&self.press, self.pull.as_mut()) {
            (_, Some(pull)) => Some(&mut pull.live),
            (Some(Press::Widget { id, .. }), None) => {
                let id = *id;
                self.widgets.iter_mut().find(|live| live.id == id)
            }
            _ => None,
        };
        if let Some(live) = target {
            live.squash.aim(squash);
        }
        let key = dropping.filter(|_| inside);
        if key.map(bead_of) != self.spool.drop.map(|(bead, _)| bead) {
            self.spool.drop = key.map(|face| (bead_of(face), self.now));
            self.hold(TIMING.label);
        }
        self.spool.sag_toward(at, key.is_some());
        self.size_beads(None, key);
    }

    fn resize(&mut self, id: u32, grip: &Grip, at: [f32; 2], was: bool) -> bool {
        let output = self.output;
        let Some(live) = self.find_mut(id) else { return was };
        let sized = grip.resize(at, live.face, output);
        if sized.snapped && !was {
            live.h.zeta(KICKS.click_zeta);
        }
        if !sized.snapped {
            live.h.zeta(SIZE.zeta);
        }
        live.h.aim(sized.share * output[1]);
        let centre = grip.centre(live.face, live.h.target());
        live.aim(centre);
        self.hold(TIMING.readout);
        sized.snapped
    }

    fn release(&mut self, at: [f32; 2]) -> Effect {
        let Some(press) = self.press.take() else { return Effect::None };
        self.guides.clear();
        self.guide_seen.clear();
        let dropping = self.spool.drop.take().is_some();
        let effect = match press {
            Press::Plate(hit) => {
                if self.hit(at) == Some(hit) {
                    return self.press_plate(hit);
                }
                Effect::None
            }
            Press::Bead { pulling: true, .. } => {
                self.end_pull(at);
                Effect::None
            }
            Press::Bead { face, .. } => {
                self.click_bead(face);
                Effect::None
            }
            Press::Widget { id, moved: true, before, .. } => {
                self.drop_widget(id, at, &before, dropping);
                Effect::None
            }
            Press::Gem { id, .. } => {
                if let Some(live) = self.find_mut(id) {
                    live.h.zeta(SIZE.zeta);
                }
                self.unfurl();
                self.record();
                Effect::None
            }
            Press::Widget { .. } | Press::Kind | Press::Empty => Effect::None,
        };
        self.size_beads(None, None);
        self.relayout();
        effect
    }

    fn press_plate(&mut self, hit: Hit) -> Effect {
        match hit {
            Hit::Tool(tool) => self.act(tool),
            Hit::Undo => self.step(true),
            Hit::Item(Item::Cancel) => self.revert(),
            Hit::Item(Item::Done) => return self.done(),
            Hit::Gem(_) | Hit::Item(_) | Hit::Widget(_) => {}
        }
        Effect::None
    }

    fn unfurl(&mut self) {
        self.selection.unfurl(self.now);
        self.hold(TIMING.unfurl + TIMING.bar_delay + TIMING.stagger * BAR_PLATES);
    }

    fn end_pull(&mut self, at: [f32; 2]) {
        if at[1] > zone_top(self.output[1]) {
            self.pull_home();
            return;
        }
        let Some(pull) = self.pull.take() else { return };
        let mut live = pull.live;
        live.squash.aim(1.0);
        live.lift.aim(1.0);
        let rect = view(self, &live).rect;
        self.spool.threads.push(Thread {
            bead: bead_of(live.face),
            strike: Strike::Retract { from: [rect.centre()[0], rect.bottom() + THREAD.tip_gap] },
            at: self.now,
        });
        let id = live.id;
        self.widgets.push(live);
        self.select(id);
        self.record();
        self.hold(TIMING.retract);
    }

    fn click_bead(&mut self, face: Face) {
        if let Some(id) = self
            .widgets
            .iter()
            .find(|live| live.dying.is_none() && live.face == face)
            .map(|live| live.id)
        {
            self.select(id);
            self.raise_up(id);
            return;
        }
        if self.full() {
            return;
        }
        self.seq += 1;
        let spot = Spot::usual(self.seq, face, self.output);
        let mut live = Live::new(spot);
        let born = self.now + TIMING.shoot * TIMING.born_at;
        live.born = Some(born);
        live.reveal = Some(born);
        let bounds = spot.bounds();
        self.spool.threads.push(Thread {
            bead: bead_of(face),
            strike: Strike::Shoot { to: [bounds.centre()[0], bounds.bottom() + THREAD.tip_gap] },
            at: self.now,
        });
        self.widgets.push(live);
        self.select(spot.id);
        self.record();
        self.hold(TIMING.shoot + TIMING.retract + TIMING.bloom);
    }

    fn drop_widget(&mut self, id: u32, at: [f32; 2], before: &[Spot], dropping: bool) {
        if dropping || at[1] > zone_top(self.output[1]) {
            let face = self.find(id).map(|live| live.face);
            self.home(id);
            self.record();
            if let Some(face) = face {
                self.toast_removed(face);
            }
            return;
        }
        let _ = before;
        let scale = self.scale;
        let hidden = self.find(id).is_some_and(|live| {
            live.depth == Depth::Behind
                && self.stage.as_ref().is_some_and(|stage| stage.hidden(live.bounds(), scale))
        });
        let mut newly = false;
        if let Some(live) = self.find_mut(id) {
            live.lift.aim(1.0);
            live.squash.aim(1.0);
            newly = hidden && !live.hidden;
            live.hidden = hidden;
        }
        if newly {
            self.flash_hatch();
        }
        self.spool.sag.depth.aim(0.0);
        self.unfurl();
        self.record();
    }

    fn abandon(&mut self) {
        let Some(press) = self.press.take() else {
            self.hover = None;
            return;
        };
        self.guides.clear();
        self.spool.drop = None;
        self.spool.sag.depth.aim(0.0);
        self.pull_home();
        if let Press::Widget { id, moved: true, before, .. } | Press::Gem { id, before, .. } = press
            && let Some(spot) = before.iter().find(|spot| spot.id == id).copied()
        {
            if let Some(live) = self.find_mut(id) {
                live.lift.aim(1.0);
                live.squash.aim(1.0);
                live.h.zeta(SIZE.zeta);
                live.aim(spot.centre);
                live.h.aim(spot.height);
            }
            self.unfurl();
        }
        self.size_beads(None, None);
    }
}

const BAR_PLATES: f32 = 8.0;
