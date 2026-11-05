use paper_control::{Depth, Face};

use super::layout::view;
use super::spool::{Strike, Thread, Travel};
use super::state::{Dying, Fate, Job, Live, Out, Press, Shake, Strand, Swap, Tool, Wipe, bead_of};
use super::tokens::{KICKS, LOOK, SIZE, THREAD, TIMING};
use crate::app::EditHistory;
use crate::domain::strand::{
    Command, Saved, Spot, cycle, detent, fit, natural_height, restack, shortest, summit, tallest,
    width_at,
};

const STEP_LABEL: &str = "strand";

pub enum Effect {
    None,
    Save(Vec<Saved>),
}

impl Strand {
    pub fn record(&mut self) {
        self.flush_order();
        let snapshot = self.snapshot();
        self.commit(snapshot);
    }

    fn commit(&mut self, snapshot: Vec<Spot>) {
        if self.history.record(snapshot, STEP_LABEL.to_owned()) {
            self.nudged = None;
            self.dismiss_toast();
            self.bump();
        }
    }

    fn bump(&mut self) {
        self.spool.pulse = self.now;
        self.hold(TIMING.pulse);
    }

    fn flush_order(&mut self) {
        let (now, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.jobs)
            .into_iter()
            .partition(|(_, job)| matches!(job, Job::Shift(..) | Job::Reorder(_)));
        self.jobs = rest;
        for (_, job) in now {
            self.run(job);
        }
    }

    pub fn pull_home(&mut self) {
        let Some(pull) = self.pull.take() else { return };
        let mut live = pull.live;
        let from = [live.x.value(), live.y.value()];
        live.dying = Some(Dying { fate: Fate::Home, at: self.now, from });
        self.widgets.push(live);
        self.hold(TIMING.home);
    }

    pub fn dismiss_toast(&mut self) {
        self.spool.dismiss_toast(self.now);
    }

    pub fn toast_removed(&mut self, face: Face) {
        let hold = self.spool.toast(self.now, bead_of(face), super::labels::removed(face));
        self.hold(hold);
    }

    pub fn select(&mut self, id: u32) {
        let Some(born) = self.find(id).map(|live| live.born) else { return };
        if self.selection.id.is_some_and(|current| current != id) {
            self.frame_out();
        }
        let at = born.map_or(self.now, |born| born.max(self.now));
        self.selection.pick(id, at);
        if let Some(face) = self.find(id).map(|live| live.face) {
            self.kick(face, KICKS.select, false);
        }
        self.hold(at - self.now + TIMING.select + TIMING.unfurl + TIMING.bar_delay);
    }

    fn frame_out(&mut self) {
        let tones = self.selected().and_then(|live| live.chrome);
        if self.selection.retire(tones, self.now) {
            self.hold(TIMING.deselect);
        }
    }

    pub fn deselect(&mut self) {
        if self.selected().is_some() {
            self.frame_out();
        }
        self.selection.clear();
    }

    pub fn raise_up(&mut self, id: u32) {
        let snapshot = self.snapshot();
        let Some(index) = snapshot.iter().position(|spot| spot.id == id) else { return };
        if let Some(to) = summit(&snapshot, index) {
            self.swap_to(id, Some(snapshot[to].id), to, true);
        }
    }

    fn swap_to(&mut self, id: u32, other: Option<u32>, to: usize, up: bool) {
        let (mine, theirs) =
            if up { (LOOK.swap_up, -LOOK.swap_down) } else { (-LOOK.swap_down, LOOK.swap_up) };
        let now = self.now;
        if let Some(live) = self.find_mut(id) {
            live.swap = Some(Swap { at: now, amp: mine });
        }
        if let Some(live) = other.and_then(|other| self.find_mut(other)) {
            live.swap = Some(Swap { at: now, amp: theirs });
        }
        self.later(TIMING.order / 2.0, Job::Shift(id, to));
        self.hold(TIMING.order);
    }

    fn place_at(&mut self, id: u32, to: usize) {
        let Some(from) = self.widgets.iter().position(|live| live.id == id) else { return };
        let live = self.widgets.remove(from);
        let slot = self
            .widgets
            .iter()
            .enumerate()
            .filter(|(_, other)| other.dying.is_none())
            .nth(to)
            .map_or(self.widgets.len(), |(slot, _)| slot);
        self.widgets.insert(slot, live);
    }

    pub fn tab(&mut self, forward: bool) {
        let mut order: Vec<(u32, f32)> =
            self.snapshot().iter().map(|spot| (spot.id, spot.bounds().x)).collect();
        if order.is_empty() {
            return;
        }
        order.sort_by(|one, other| one.1.total_cmp(&other.1));
        let count = order.len();
        let at = self.selection.id.and_then(|id| order.iter().position(|(other, _)| *other == id));
        let next = match at {
            None if forward => 0,
            None => count - 1,
            Some(at) if forward => (at + 1) % count,
            Some(at) => (at + count - 1) % count,
        };
        let id = order[next].0;
        if self.selection.frame.is_none() {
            self.select(id);
            self.raise_up(id);
            return;
        }
        self.selection.id = Some(id);
        self.selection.at = self.now - TIMING.gem_pop - TIMING.gem_stagger * FRAME_CORNERS;
        self.selection.unfurl(self.now);
        if let Some(face) = self.find(id).map(|live| live.face) {
            self.kick(face, KICKS.select, false);
        }
        self.raise_up(id);
        self.hold(TIMING.unfurl + TIMING.bar_delay + TIMING.stagger * FRAME_PLATES);
    }

    pub fn set_depth(&mut self, id: u32, depth: Depth) -> bool {
        let now = self.now;
        let Some(live) = self.find_mut(id).filter(|live| live.depth != depth) else {
            return false;
        };
        live.wipe = Some(Wipe { at: now, from: live.depth });
        live.depth = depth;
        live.hidden = false;
        if depth == Depth::Behind {
            self.flash_hatch();
        }
        self.hold(TIMING.wipe);
        true
    }

    pub fn flash_hatch(&mut self) {
        if self.hatch_image.is_none()
            && let (Some(stage), Some(tones)) = (self.stage.as_ref(), self.whole)
        {
            self.hatch_image = stage.hatch(tones.accent.rgb, self.scale);
        }
        if self.hatch_image.is_some() {
            self.hatch = Some(self.now);
            self.hold(TIMING.hatch);
        }
    }

    pub fn cycle_style(&mut self, id: u32, forward: bool) -> bool {
        let snapshot = self.snapshot();
        let Some(index) = snapshot.iter().position(|spot| spot.id == id) else { return false };
        let Some(face) = cycle(&snapshot, index, forward) else {
            self.shake = Some((self.now, Shake::Style));
            self.hold(TIMING.shake);
            return false;
        };
        self.morph(id, face);
        let output = self.output;
        if let Some(live) = self.find_mut(id) {
            let height = natural_height(face, output);
            live.h.zeta(SIZE.zeta);
            live.h.aim(height);
            let centre = refit([live.x.target(), live.y.target()], face, height, output);
            live.aim(centre);
        }
        true
    }

    fn morph(&mut self, id: u32, face: Face) {
        let now = self.now;
        let Some(live) = self.find_mut(id) else { return };
        let old = live.face;
        live.out = Some(Out {
            face: old,
            at: now,
            centre: [live.x.value(), live.y.value()],
            height: live.h.value(),
        });
        live.face = face;
        live.reveal = Some(now + TIMING.out * TIMING.reveal_at);
        if let Some(proxy) = self.rasters.remove(&id) {
            self.outgoing.insert(id, proxy);
        }
        self.kick(old, KICKS.style_out, false);
        self.kick(face, KICKS.style_in, false);
        self.hold(TIMING.out * TIMING.reveal_at + super::state::reveal_span());
    }

    pub fn remove(&mut self, id: u32) {
        let now = self.now;
        let Some(live) = self.find(id) else { return };
        let rect = view(self, live).rect;
        let face = live.face;
        let from = rect.centre();
        if let Some(live) = self.find_mut(id) {
            live.dying = Some(Dying { fate: Fate::Collapse, at: now, from });
        }
        self.spool.travels.push(Travel {
            bead: bead_of(face),
            from,
            at: now + TIMING.collapse * TIMING.travel_lead,
        });
        if self.selection.id == Some(id) {
            self.deselect();
        }
        self.hold(TIMING.collapse + TIMING.travel);
    }

    pub fn home(&mut self, id: u32) {
        let now = self.now;
        let Some(live) = self.find_mut(id) else { return };
        let from = [live.x.value(), live.y.value()];
        let face = live.face;
        live.dying = Some(Dying { fate: Fate::Home, at: now, from });
        let slot = self.slot(face);
        self.spool.sag.along.aim(slot[0]);
        self.spool.sag.catch_at = Some(now + TIMING.home * TIMING.catch_at);
        if self.selection.id == Some(id) {
            self.deselect();
        }
        self.hold(TIMING.home);
    }

    pub fn revive(&mut self, spot: Spot, index: usize) {
        self.widgets.retain(|live| live.id != spot.id);
        let mut live = Live::new(spot);
        let born = self.now + TIMING.shoot * TIMING.born_at;
        live.born = Some(born);
        live.reveal = Some(born);
        let bounds = spot.bounds();
        self.spool.threads.push(Thread {
            bead: bead_of(spot.face),
            strike: Strike::Shoot { to: [bounds.centre()[0], bounds.bottom() + THREAD.tip_gap] },
            at: self.now,
        });
        self.widgets.push(live);
        self.place_at(spot.id, index);
        self.kick(spot.face, KICKS.revive, true);
        self.hold(TIMING.shoot + TIMING.bloom + super::state::reveal_span());
    }

    pub fn restore(&mut self, spot: Spot) {
        let Some(live) = self.find(spot.id) else { return };
        let (face, depth) = (live.face, live.depth);
        if face != spot.face {
            self.morph(spot.id, spot.face);
        }
        if depth != spot.depth {
            self.set_depth(spot.id, spot.depth);
        }
        if let Some(live) = self.find_mut(spot.id) {
            live.h.zeta(SIZE.zeta);
            live.aim(spot.centre);
            live.h.aim(spot.height);
        }
    }

    pub fn nudge(&mut self, offset: [f32; 2]) {
        let Some(id) = self.selection.id else { return };
        let output = self.output;
        let Some(live) = self.find_mut(id) else { return };
        let bounds = live.bounds();
        let size = [bounds.w, bounds.h];
        let centre: [f32; 2] = std::array::from_fn(|axis| {
            fit(bounds.centre()[axis] + offset[axis], size[axis] / 2.0, output[axis])
        });
        live.aim(centre);
        let recent =
            self.nudged.is_some_and(|(last, at)| last == id && self.now - at < TIMING.nudge);
        if recent && self.history.index() > 0 {
            self.history.amend(self.snapshot());
        } else {
            self.record();
        }
        self.nudged = Some((id, self.now));
    }

    pub fn detent(&mut self, up: bool) {
        let Some(id) = self.selection.id else { return };
        let output = self.output;
        let Some(live) = self.find(id) else { return };
        let share = live.h.target() / output[1];
        let face = live.face;
        let Some(next) = detent(share, up, [shortest(), tallest(face, output)]) else {
            self.shake = Some((self.now, Shake::Style));
            self.hold(TIMING.shake);
            return;
        };
        if let Some(live) = self.find_mut(id) {
            let height = next * output[1];
            live.h.zeta(KICKS.click_zeta);
            live.h.aim(height);
            let centre = refit([live.x.target(), live.y.target()], face, height, output);
            live.aim(centre);
        }
        self.flash = Some((id, self.now));
        self.later(TIMING.readout, Job::Calm(id));
        self.record();
    }

    pub fn act(&mut self, tool: Tool) {
        let Some(id) = self.selection.id else { return };
        match tool {
            Tool::Behind | Tool::Front => {
                let wanted = if tool == Tool::Behind { Depth::Behind } else { Depth::Front };
                if self.set_depth(id, wanted) {
                    self.record();
                }
            }
            Tool::Prev | Tool::Style | Tool::Next => {
                if self.cycle_style(id, tool != Tool::Prev) {
                    self.record();
                }
            }
            Tool::Raise | Tool::Lower => self.restack_selected(id, tool == Tool::Raise),
            Tool::Remove => self.remove_recorded(id),
        }
    }

    fn restack_selected(&mut self, id: u32, up: bool) {
        self.flush_order();
        let mut snapshot = self.snapshot();
        let Some(index) = snapshot.iter().position(|spot| spot.id == id) else { return };
        let Some(to) = restack(&snapshot, index, up) else {
            self.shake = Some((self.now, Shake::Order));
            self.hold(TIMING.shake);
            return;
        };
        let other = snapshot[to].id;
        self.swap_to(id, Some(other), to, up);
        crate::domain::strand::shift(&mut snapshot, index, to);
        self.commit(snapshot);
    }

    fn remove_recorded(&mut self, id: u32) {
        let Some(face) = self.find(id).map(|live| live.face) else { return };
        self.remove(id);
        self.record();
        self.toast_removed(face);
    }

    pub fn command(&mut self, command: Command) -> Effect {
        if !self.hits_ready() {
            return Effect::None;
        }
        let pressing = self.press.is_some();
        let selected = self.selection.id;
        match command {
            Command::Undo if !pressing => self.step(true),
            Command::Redo if !pressing => self.step(false),
            Command::Deselect => self.deselect(),
            Command::Cancel => self.revert(),
            Command::Tab(forward) if !pressing => self.tab(forward),
            Command::Nudge(offset) if !pressing => self.nudge(offset),
            Command::Detent(up) => self.detent(up),
            Command::Depth => {
                let depth = selected.and_then(|id| self.find(id)).map(|live| live.depth);
                let tool = if depth == Some(Depth::Behind) { Tool::Front } else { Tool::Behind };
                self.act(tool);
            }
            Command::Style(forward) => self.act(if forward { Tool::Next } else { Tool::Prev }),
            Command::Raise => self.act(Tool::Raise),
            Command::Lower => self.act(Tool::Lower),
            Command::Remove => self.act(Tool::Remove),
            _ => {}
        }
        Effect::None
    }

    pub fn step(&mut self, undo: bool) {
        if self.press.is_some() || self.reverting {
            return;
        }
        self.flush_order();
        let moved = if undo { self.history.undo() } else { self.history.redo() };
        let target: Option<Vec<Spot>> = moved.cloned();
        let Some(target) = target else {
            self.bump();
            return;
        };
        self.dismiss_toast();
        self.apply(&target, undo);
        self.bump();
    }

    fn apply(&mut self, target: &[Spot], undo: bool) {
        let current = self.snapshot();
        for spot in current.iter().filter(|spot| !target.iter().any(|want| want.id == spot.id)) {
            if undo { self.home(spot.id) } else { self.remove(spot.id) }
        }
        for (index, spot) in target.iter().enumerate() {
            match current.iter().find(|have| have.id == spot.id) {
                None => self.revive(*spot, index),
                Some(have) if have != spot => {
                    self.restore(*spot);
                    if self.selection.id != Some(spot.id) {
                        self.select(spot.id);
                    }
                }
                Some(_) => {}
            }
        }
        let kept: Vec<u32> = current
            .iter()
            .map(|spot| spot.id)
            .filter(|id| target.iter().any(|want| want.id == *id))
            .collect();
        let wanted: Vec<u32> =
            target.iter().map(|spot| spot.id).filter(|id| kept.contains(id)).collect();
        if kept == wanted {
            return;
        }
        let distance = |at: usize, id: u32| {
            wanted.iter().position(|want| *want == id).map_or(0, |goal| goal.abs_diff(at))
        };
        let mover = kept.iter().enumerate().max_by_key(|(at, id)| distance(*at, **id));
        if let Some((at, &id)) = mover
            && let Some(goal) = wanted.iter().position(|want| *want == id)
        {
            let other = kept.get(goal).copied().filter(|other| *other != id);
            self.swap_to(id, other, goal, goal > at);
        }
    }

    pub fn revert(&mut self) {
        if self.reverting || self.spool.leaving || !self.ready() {
            return;
        }
        self.pull_home();
        self.press = None;
        self.guides.clear();
        self.hover = None;
        self.deselect();
        self.dismiss_toast();
        self.wind();
        self.spool.drop = None;
        self.spool.sag.depth.aim(0.0);
        let original = self.history.original().clone();
        let current = self.snapshot();
        let mut step = 0.0;
        for spot in &current {
            let job = match original.iter().find(|want| want.id == spot.id) {
                Some(want) => Job::Restore(*want),
                None => Job::Home(spot.id),
            };
            self.later(step * TIMING.revert_stagger, job);
            step += 1.0;
        }
        for (index, spot) in original.iter().enumerate() {
            if !current.iter().any(|have| have.id == spot.id) {
                self.later(step * TIMING.revert_stagger, Job::Revive(*spot, index));
                step += 1.0;
            }
        }
        let end = step * TIMING.revert_stagger;
        self.later(
            end + TIMING.flight,
            Job::Reorder(original.iter().map(|spot| spot.id).collect()),
        );
        self.history = EditHistory::new(original, STEP_LABEL.to_owned());
        self.bump();
        self.reverting = true;
        self.later(TIMING.revert_hold.max(end + TIMING.flight + TIMING.order), Job::Leave);
    }

    pub fn done(&mut self) -> Effect {
        if self.reverting || self.spool.leaving || !self.ready() {
            return Effect::None;
        }
        let items = self.items();
        self.leave();
        Effect::Save(items)
    }

    pub fn leave(&mut self) {
        self.reverting = false;
        self.spool.leaving = true;
        self.amount.aim(0.0);
        self.deselect();
        self.pull_home();
        self.press = None;
        self.guides.clear();
        self.dismiss_toast();
        self.wind();
        self.spool.drop = None;
        self.spool.sag.depth.aim(0.0);
        self.hold(TIMING.deselect);
    }

    pub fn arranging(&mut self) {
        self.arranging = Some(self.now);
        self.hold(TIMING.busy);
    }

    pub fn unspool(&mut self, kind: usize) {
        let hold = self.spool.unspool(kind);
        self.hold(hold);
    }

    pub fn wind(&mut self) {
        let hold = self.spool.wind();
        self.hold(hold);
    }

    pub fn run(&mut self, job: Job) {
        match job {
            Job::Restore(spot) => self.restore(spot),
            Job::Home(id) => self.home(id),
            Job::Revive(spot, index) => self.revive(spot, index),
            Job::Reorder(ids) => {
                for (index, id) in ids.iter().enumerate() {
                    self.place_at(*id, index);
                }
            }
            Job::Shift(id, to) => self.place_at(id, to),
            Job::Calm(id) => {
                let pressing = self.press.is_some();
                if let Some(live) = self.find_mut(id).filter(|_| !pressing) {
                    live.h.zeta(SIZE.zeta);
                }
            }
            Job::Leave => self.leave(),
        }
    }
}

const FRAME_CORNERS: f32 = 4.0;
const FRAME_PLATES: f32 = 8.0;

pub fn refit(centre: [f32; 2], face: Face, height: f32, output: [f32; 2]) -> [f32; 2] {
    let size = [width_at(face, height), height];
    std::array::from_fn(|axis| fit(centre[axis], size[axis] / 2.0, output[axis]))
}

pub fn pressed_id(press: Option<&Press>) -> Option<u32> {
    match press {
        Some(Press::Widget { id, .. } | Press::Gem { id, .. }) => Some(*id),
        _ => None,
    }
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
