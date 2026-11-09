use super::model::{ArrangeMsg, Effect, MIN_FRAME, MIN_WIDTH, Pointer, Step, Target};
use super::state::{Arrange, Choice, Extra, Hit, Press};
use crate::domain::strand::{CORNERS, Item, Rect, cells, snap, stretch};
use crate::frontend::strand::draw::selection::corner;
use crate::frontend::strand::spool::{BeadAt, Strike, Thread, Travel};
use crate::frontend::strand::tokens::{DRAG_START, FRAME, THREAD, TIMING};

fn distance(one: [f32; 2], other: [f32; 2]) -> f32 {
    (one[0] - other[0]).hypot(one[1] - other[1])
}

impl Arrange {
    pub fn update(&mut self, pointer: Pointer) -> Vec<Effect> {
        let effects = match pointer {
            Pointer::Press(at) => self.press_at(at),
            Pointer::Move(at) => self.move_to(at),
            Pointer::Release(at) => self.release(at),
            Pointer::Leave => self.abandon(),
        };
        if !matches!(self.press, Some(Press::Object { moved: true, .. } | Press::Gem { .. })) {
            self.thread.clear();
        }
        effects
    }

    pub(crate) fn object_at(&self, at: [f32; 2]) -> Option<Target> {
        let mut found: Vec<Target> = self
            .targets()
            .into_iter()
            .filter(|target| {
                let pad =
                    if self.selection.id == Some(*target) { FRAME.pad } else { FRAME.widget_pad };
                self.rect(*target).padded(pad).contains(at)
            })
            .collect();
        found.sort_by(|one, other| self.rect(*one).area().total_cmp(&self.rect(*other).area()));
        found.first().copied()
    }

    pub(crate) fn hit(&self, at: [f32; 2]) -> Option<Hit> {
        if self.preview {
            return self.back_rect().contains(at).then_some(Hit::Back);
        }
        if let Some(slot) = self.panel().into_iter().find(|slot| slot.rect.contains(at)) {
            return slot.choice.map(Hit::Choice);
        }
        if self.hits_ready()
            && let Some(item @ (Item::Extra(_) | Item::Cancel | Item::Done)) =
                self.spool.hit(self.now, at)
        {
            return Some(Hit::Item(item));
        }
        if let (Some(target), Some(frame)) = (self.selection.id, self.framed()) {
            let bar = self.bar(target, frame);
            let plate = bar.plates.iter().find(|plate| {
                self.selection.opening(self.now, plate) > FRAME.hit_open && plate.rect.contains(at)
            });
            if let Some(plate) = plate {
                return Some(Hit::Tool(plate.tool));
            }
            if target == Target::Picker && self.board.resizable {
                for index in 0..CORNERS.len() {
                    let reach = Rect::centred(corner(frame, index), [2.0 * FRAME.gem_hit; 2]);
                    if reach.contains(at) {
                        return Some(Hit::Gem(index));
                    }
                }
            }
        }
        if !self.hits_ready() {
            return None;
        }
        if let Some(toast) =
            self.spool.toast_plates(self.now).filter(|toast| toast.up > FRAME.hit_open)
            && toast.undo_rect().contains(at)
        {
            return Some(Hit::Undo);
        }
        let thread = self.spool.hit(self.now, at);
        if thread.is_some() && self.spool.in_zone(at[1]) {
            return thread.map(Hit::Item);
        }
        if self.selection.id == Some(Target::Bar)
            && let Some(index) =
                self.board.buttons.iter().position(|button| button.rect.contains(at))
        {
            return Some(Hit::Button(index));
        }
        self.object_at(at).map(Hit::Object).or(thread.map(Hit::Item))
    }

    fn press_at(&mut self, at: [f32; 2]) -> Vec<Effect> {
        self.pointer = at;
        self.fresh = false;
        if self.reverting || self.spool.leaving {
            return Vec::new();
        }
        let hit = self.hit(at);
        let keeps = matches!(hit, Some(Hit::Item(Item::Kind(kind) | Item::Group(kind))) if self.spool.open == Some(kind));
        if !keeps && !self.preview {
            let hold = self.spool.wind();
            self.hold(hold);
        }
        self.press = Some(match hit {
            Some(Hit::Gem(index)) => {
                let rect = self.objects[Target::Picker.index()].target();
                self.selection.fold_now(self.now);
                self.hold(TIMING.fold);
                Press::Gem { corner: index, anchor: corner(rect, (index + 2) % 4), snapped: false }
            }
            Some(hit @ (Hit::Tool(_) | Hit::Undo | Hit::Choice(_) | Hit::Back)) => {
                Press::Plate(hit)
            }
            Some(hit @ Hit::Item(Item::Extra(_) | Item::Cancel | Item::Done)) => Press::Plate(hit),
            Some(Hit::Item(Item::Kind(kind) | Item::Group(kind))) => {
                let hold = self.spool.unspool(kind);
                self.hold(hold);
                Press::Kind
            }
            Some(Hit::Item(Item::Bead(kind, style))) => {
                Press::Bead { bead: (kind, style), start: at, pulling: false }
            }
            Some(Hit::Button(index)) => {
                let id = self.board.buttons[index].id.clone();
                Press::Button { id, start: at, moved: false }
            }
            Some(Hit::Object(target)) => {
                if self.selection.id != Some(target) {
                    self.select(target);
                }
                let centre = self.rect(target).centre();
                let grab = [centre[0] - at[0], centre[1] - at[1]];
                Press::Object { target, grab, start: at, moved: false }
            }
            None => {
                if !self.preview {
                    self.deselect();
                }
                Press::Empty
            }
        });
        self.relayout();
        Vec::new()
    }

    fn move_to(&mut self, at: [f32; 2]) -> Vec<Effect> {
        self.pointer = at;
        let Some(press) = self.press.take() else {
            self.hover_at(at);
            return Vec::new();
        };
        let mut effects = Vec::new();
        let press = match press {
            Press::Object { target, grab, start, moved } => {
                let moved = moved || distance(at, start) > DRAG_START;
                if moved {
                    if self.selection.fold.is_none() {
                        self.selection.fold_now(self.now);
                        self.hold(TIMING.fold);
                    }
                    let rect = self.drag(target, [at[0] + grab[0], at[1] + grab[1]]);
                    effects.push(Effect::Place(target, rect));
                }
                Press::Object { target, grab, start, moved }
            }
            Press::Gem { corner: index, anchor, snapped } => {
                let (effect, snapped) = self.resize(index, anchor, at, snapped);
                effects.push(effect);
                Press::Gem { corner: index, anchor, snapped }
            }
            Press::Button { id, start, moved } => {
                let moved = moved || distance(at, start) > DRAG_START;
                if moved {
                    effects.push(Effect::Reorder(id.clone(), at));
                    let bead = self.bead_of(&id);
                    self.zone(at, bead);
                }
                Press::Button { id, start, moved }
            }
            Press::Bead { bead, start, pulling } => {
                let ready = self.pullable(bead) && distance(at, start) > DRAG_START;
                let id = self.bead_id(bead).map(str::to_owned);
                match id {
                    Some(id) if pulling => effects.push(Effect::Reorder(id, at)),
                    Some(id) if ready => effects.push(Effect::Show(id, Some(at))),
                    _ => {}
                }
                let pulling = pulling || ready;
                if pulling {
                    self.zone(at, Some(bead));
                }
                Press::Bead { bead, start, pulling }
            }
            other => other,
        };
        self.press = Some(press);
        effects
    }

    fn hover_at(&mut self, at: [f32; 2]) {
        let hit = self.hit(at);
        if self.hover.map(|(current, _)| current) != hit {
            self.hover = hit.map(|hit| (hit, self.now));
            self.hold(TIMING.label);
        }
        if let Some(Hit::Item(Item::Kind(kind) | Item::Group(kind))) = hit {
            let hold = self.spool.unspool(kind);
            self.hold(hold);
        }
        self.size_beads(None);
    }

    pub(crate) fn size_beads(&mut self, dropping: Option<BeadAt>) {
        let hovered = match self.hover {
            Some((Hit::Item(Item::Bead(kind, style)), _)) => Some((kind, style)),
            _ => None,
        };
        let kind = match self.hover {
            Some((Hit::Item(Item::Kind(kind) | Item::Group(kind)), _)) => Some(kind),
            _ => None,
        };
        self.spool.size_beads(hovered, kind, dropping);
    }

    fn others(&self, target: Target) -> Vec<Rect> {
        self.targets()
            .into_iter()
            .filter(|other| *other != target)
            .filter(|other| target.edit_id().is_some() || other.edit_id().is_none())
            .filter(|other| target != Target::Picker || !self.board.attached[other.index()])
            .map(|other| self.objects[other.index()].target())
            .collect()
    }

    fn guided(&mut self, guides: Vec<crate::domain::strand::Guide>) {
        for guide in &guides {
            self.guide_seen.entry(guide.key()).or_insert(self.now);
        }
        let keys: Vec<(bool, i32)> = guides.iter().map(crate::domain::strand::Guide::key).collect();
        self.guide_seen.retain(|key, _| keys.contains(key));
        self.guides = guides;
        self.hold(TIMING.guide);
    }

    fn drag(&mut self, target: Target, wanted: [f32; 2]) -> Rect {
        let bounds = self.objects[target.index()].target();
        let others = self.others(target);
        let snapped = snap([bounds.w, bounds.h], wanted, &others, self.output);
        self.guided(snapped.guides);
        let rect = Rect::centred(snapped.centre, [bounds.w, bounds.h]);
        self.objects[target.index()].aim(rect);
        rect
    }

    fn resize(
        &mut self,
        index: usize,
        anchor: [f32; 2],
        at: [f32; 2],
        was: bool,
    ) -> (Effect, bool) {
        let others = self.others(Target::Picker);
        let grip = [anchor, CORNERS[index]];
        let stretched = stretch(grip, at, [MIN_WIDTH, MIN_FRAME], &others, self.output);
        let snapped = !stretched.guides.is_empty();
        self.guided(stretched.guides);
        self.hold(TIMING.readout);
        let object = &mut self.objects[Target::Picker.index()];
        let feel = if snapped && !was {
            Some(crate::frontend::strand::tokens::KICKS.click_zeta)
        } else {
            None
        };
        if let Some(zeta) = feel {
            object.w.zeta(zeta);
            object.h.zeta(zeta);
        }
        let Some(grid) = self.board.grid else {
            object.aim(stretched.rect);
            return (Effect::Place(Target::Picker, stretched.rect), snapped);
        };
        let count = |span: f32, axis: usize| {
            cells(span, grid.cell[axis], grid.gap[axis], super::model::MAX_CELLS)
        };
        let found = [count(stretched.rect.w, 0), count(stretched.rect.h, 1)];
        let size: [f32; 2] = std::array::from_fn(|axis| {
            grid.cell[axis] * found[axis] as f32 + grid.gap[axis] * (found[axis] - 1) as f32
        });
        let sign = CORNERS[index];
        let x = if sign[0] > 0.0 { anchor[0] } else { anchor[0] - size[0] };
        let y = if sign[1] > 0.0 { anchor[1] } else { anchor[1] - size[1] };
        let rect = Rect::new(x, y, size[0], size[1]);
        let clicked = found != grid.cells;
        object.aim(rect);
        (Effect::Grid(rect, found), clicked || snapped)
    }

    fn zone(&mut self, at: [f32; 2], dropping: Option<BeadAt>) {
        let inside = self.spool.in_zone(at[1]);
        let key = dropping.filter(|_| inside);
        if key != self.spool.drop.map(|(bead, _)| bead) {
            self.spool.drop = key.map(|bead| (bead, self.now));
            self.hold(TIMING.label);
        }
        self.spool.sag_toward(at, key.is_some());
        self.size_beads(key);
    }

    fn release(&mut self, at: [f32; 2]) -> Vec<Effect> {
        self.pointer = at;
        let Some(press) = self.press.take() else { return Vec::new() };
        let dropping = self.spool.drop.take().is_some();
        let inside = self.spool.in_zone(at[1]);
        let mut effects = Vec::new();
        match press {
            Press::Plate(hit) => {
                if self.hit(at) == Some(hit) {
                    effects = self.press_plate(hit);
                }
            }
            Press::Object { target, moved: true, .. } => {
                let rect = self.objects[target.index()].target();
                effects.push(Effect::Commit(target, self.anchor_of(target, rect), Step::Move));
                self.unfurl();
            }
            Press::Gem { .. } => {
                let object = &mut self.objects[Target::Picker.index()];
                object.w.zeta(crate::frontend::strand::tokens::SIZE.zeta);
                object.h.zeta(crate::frontend::strand::tokens::SIZE.zeta);
                let rect = object.target();
                let step = if self.board.grid.is_some() { Step::Grid } else { Step::Size };
                effects.push(Effect::Commit(
                    Target::Picker,
                    self.anchor_of(Target::Picker, rect),
                    step,
                ));
                self.unfurl();
            }
            Press::Button { id, moved: true, .. } => {
                if dropping || inside {
                    effects.push(Effect::Hide(id.clone()));
                    self.hidden(&id, at);
                } else {
                    effects.push(Effect::Drop);
                }
            }
            Press::Bead { bead, pulling: true, .. } => {
                if dropping || inside {
                    effects.push(Effect::Abandon);
                    self.spool.travels.push(Travel { bead, from: at, at: self.now });
                    self.hold(TIMING.travel);
                } else {
                    effects.push(Effect::Drop);
                    self.spool.threads.push(Thread {
                        bead,
                        strike: Strike::Retract { from: at },
                        at: self.now,
                    });
                    self.select(Target::Bar);
                    self.hold(TIMING.retract);
                }
            }
            Press::Bead { bead, .. } => effects = self.click_bead(bead),
            Press::Object { .. } | Press::Button { .. } | Press::Kind | Press::Empty => {}
        }
        self.guides.clear();
        self.guide_seen.clear();
        self.spool.sag_toward(at, false);
        self.size_beads(None);
        self.relayout();
        effects
    }

    pub(crate) fn hidden(&mut self, id: &str, from: [f32; 2]) {
        let Some(bead) = self.bead_of(id) else { return };
        let title = self.board.shelves[bead.0].beads[bead.1].title.clone();
        self.spool.travels.push(Travel {
            bead,
            from,
            at: self.now + TIMING.collapse * TIMING.travel_lead,
        });
        self.spool.sag.catch_at = Some(self.now + TIMING.travel * TIMING.catch_at);
        let hold = self.spool.toast(self.now, bead, super::labels::hidden(&title));
        self.fresh = true;
        self.hold(hold.max(TIMING.travel + TIMING.collapse));
    }

    fn click_bead(&mut self, bead: BeadAt) -> Vec<Effect> {
        let Some(id) = self.bead_id(bead).map(str::to_owned) else { return Vec::new() };
        let target = Target::floating(&id).unwrap_or(Target::Bar);
        if self.shown_bead(bead) {
            self.select(target);
            self.spool.kick(bead, crate::frontend::strand::tokens::KICKS.select, false);
            return Vec::new();
        }
        self.shoot = Some((id.clone(), bead));
        self.select(target);
        vec![Effect::Show(id, None)]
    }

    pub(crate) fn strike(&mut self, bead: BeadAt, rect: Rect) {
        self.spool.threads.push(Thread {
            bead,
            strike: Strike::Shoot { to: [rect.centre()[0], rect.bottom() + THREAD.tip_gap] },
            at: self.now,
        });
        self.hold(TIMING.shoot + TIMING.retract);
    }

    fn press_plate(&mut self, hit: Hit) -> Vec<Effect> {
        match hit {
            Hit::Tool(tool) => self.act(tool),
            Hit::Undo => self.step(true),
            Hit::Back => self.back(),
            Hit::Item(Item::Cancel) => self.revert(),
            Hit::Item(Item::Done) => vec![Effect::Message(ArrangeMsg::Done)],
            Hit::Item(Item::Extra(index)) => match self.extras().get(index) {
                Some(Extra::Preview) => self.enter_preview(),
                Some(Extra::Displays) if self.board.copy.is_some() => {
                    vec![Effect::Message(ArrangeMsg::CloseDisplays)]
                }
                Some(Extra::Displays) => vec![Effect::Message(ArrangeMsg::OpenDisplays)],
                None => Vec::new(),
            },
            Hit::Choice(choice) => self.choose(choice),
            Hit::Gem(_) | Hit::Item(_) | Hit::Object(_) | Hit::Button(_) => Vec::new(),
        }
    }

    fn choose(&self, choice: Choice) -> Vec<Effect> {
        let message = match choice {
            Choice::Recipient(index) => {
                let Some(copy) = &self.board.copy else { return Vec::new() };
                let Some(recipient) = copy.recipients.get(index) else { return Vec::new() };
                ArrangeMsg::ReviewDisplay(recipient.name.clone())
            }
            Choice::Fit => ArrangeMsg::FitDisplay,
            Choice::Reconnect => {
                let on = self
                    .board
                    .copy
                    .as_ref()
                    .and_then(|copy| copy.review.as_ref())
                    .is_some_and(|review| review.save_disconnected);
                ArrangeMsg::AllowDisconnected(!on)
            }
            Choice::Apply => ArrangeMsg::ApplyDisplay,
            Choice::Skip => ArrangeMsg::SkipDisplay,
            Choice::Forget => {
                let Some(review) = self.board.copy.as_ref().and_then(|copy| copy.review.as_ref())
                else {
                    return Vec::new();
                };
                ArrangeMsg::ForgetDisplay(review.output.clone())
            }
            Choice::Keep => ArrangeMsg::ResolveConflict(true),
            Choice::Saved => ArrangeMsg::ResolveConflict(false),
            Choice::Consent => ArrangeMsg::ConfirmDisconnected,
        };
        vec![Effect::Message(message)]
    }

    pub(crate) fn unfurl(&mut self) {
        self.selection.unfurl(self.now);
        self.hold(TIMING.unfurl + TIMING.bar_delay + TIMING.stagger * BAR_PLATES);
    }

    fn abandon(&mut self) -> Vec<Effect> {
        let Some(press) = self.press.take() else {
            self.hover = None;
            return Vec::new();
        };
        self.guides.clear();
        self.spool.drop = None;
        self.spool.sag_toward(self.pointer, false);
        self.size_beads(None);
        match press {
            Press::Object { moved: true, .. }
            | Press::Gem { .. }
            | Press::Button { moved: true, .. }
            | Press::Bead { pulling: true, .. } => {
                self.unfurl();
                vec![Effect::Abandon]
            }
            _ => Vec::new(),
        }
    }
}

const BAR_PLATES: f32 = 6.0;
