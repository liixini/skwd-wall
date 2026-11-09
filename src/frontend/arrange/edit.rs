use super::model::{ArrangeMsg, Effect, MIN_FRAME, MIN_WIDTH, Target, Tool};
use super::state::{Arrange, Job};
use crate::domain::strand::{Command, Rect, detent, fit};
use crate::frontend::strand::tokens::{KICKS, SIZE, TIMING};

impl Arrange {
    pub(crate) fn select(&mut self, target: Target) {
        if self.selection.id.is_some_and(|current| current != target) {
            self.frame_out();
        }
        self.selection.pick(target, self.now);
        self.hold(TIMING.select + TIMING.unfurl + TIMING.bar_delay);
    }

    fn frame_out(&mut self) {
        if self.selection.retire(Some(self.tones), self.now) {
            self.hold(TIMING.deselect);
        }
    }

    pub(crate) fn deselect(&mut self) {
        if self.selection.id.is_some() {
            self.frame_out();
        }
        self.selection.clear();
    }

    fn tab(&mut self, forward: bool) {
        let targets = self.targets();
        let count = targets.len();
        let current = self.selection.id.and_then(|id| targets.iter().position(|one| *one == id));
        let next = match current {
            None => 0,
            Some(at) if forward => (at + 1) % count,
            Some(at) => (at + count - 1) % count,
        };
        let target = targets[next];
        if self.selection.id.is_none() {
            self.select(target);
            return;
        }
        let at = self.now - TIMING.select - TIMING.gem_pop;
        self.selection.pick(target, at);
        self.selection.unfurl(self.now);
        self.hold(TIMING.unfurl + TIMING.bar_delay);
    }

    fn nudge(&mut self, offset: [f32; 2]) -> Vec<Effect> {
        let Some(target) = self.selection.id else { return Vec::new() };
        let output = self.output;
        let bounds = self.objects[target.index()].target();
        let centre: [f32; 2] = std::array::from_fn(|axis| {
            let size = if axis == 0 { bounds.w } else { bounds.h };
            fit(bounds.centre()[axis] + offset[axis], size / 2.0, output[axis])
        });
        let rect = Rect::centred(centre, [bounds.w, bounds.h]);
        self.objects[target.index()].aim(rect);
        let recent =
            self.nudged.is_some_and(|(last, at)| last == target && self.now - at < TIMING.nudge);
        self.nudged = Some((target, self.now));
        let anchor = self.kept_anchor(target, rect);
        vec![Effect::Nudge(target, rect, anchor, recent && self.board.steps > 0)]
    }

    fn detent(&mut self, up: bool) -> Vec<Effect> {
        if self.selection.id != Some(Target::Picker)
            || !self.board.resizable
            || self.board.grid.is_some()
        {
            self.shake = Some(self.now);
            self.hold(TIMING.shake);
            return Vec::new();
        }
        let output = self.output;
        let bounds = self.objects[Target::Picker.index()].target();
        let ratio = bounds.w / bounds.h.max(1.0);
        let tallest = output[1].min(output[0] / ratio) / output[1];
        let least = MIN_FRAME.max(MIN_WIDTH / ratio) / output[1];
        let Some(share) = detent(bounds.h / output[1], up, [least, tallest]) else {
            self.shake = Some(self.now);
            self.hold(TIMING.shake);
            return Vec::new();
        };
        let height = share * output[1];
        let width = height * ratio;
        let centre: [f32; 2] = std::array::from_fn(|axis| {
            let size = if axis == 0 { width } else { height };
            fit(bounds.centre()[axis], size / 2.0, output[axis])
        });
        let rect = Rect::centred(centre, [width, height]);
        let object = &mut self.objects[Target::Picker.index()];
        object.w.zeta(KICKS.click_zeta);
        object.h.zeta(KICKS.click_zeta);
        object.aim(rect);
        self.flash = Some(self.now);
        self.hold(TIMING.readout);
        let anchor = self.kept_anchor(Target::Picker, rect);
        vec![Effect::Nudge(Target::Picker, rect, anchor, false)]
    }

    pub(crate) fn act(&mut self, tool: Tool) -> Vec<Effect> {
        let Some(target) = self.selection.id else { return Vec::new() };
        match tool {
            Tool::Prev | Tool::Next | Tool::Layout => vec![Effect::Layout(tool != Tool::Prev)],
            Tool::Spacing => vec![Effect::Spacing],
            Tool::Corners => vec![Effect::Corners],
            Tool::Hide => self.hide(target),
            Tool::Reset => vec![Effect::Reset(target)],
        }
    }

    fn hide(&mut self, target: Target) -> Vec<Effect> {
        let Some(id) = target.edit_id() else { return Vec::new() };
        let from = self.rect(target).centre();
        self.deselect();
        self.hidden(id, from);
        vec![Effect::Hide(id.to_owned())]
    }

    pub fn command(&mut self, command: Command) -> Vec<Effect> {
        self.thread.clear();
        if self.preview {
            return if command == Command::Cancel || command == Command::Deselect {
                self.back()
            } else {
                Vec::new()
            };
        }
        if !self.hits_ready() {
            return Vec::new();
        }
        let pressing = self.press.is_some();
        match command {
            Command::Undo if !pressing => self.step(true),
            Command::Redo if !pressing => self.step(false),
            Command::Deselect => {
                self.deselect();
                Vec::new()
            }
            Command::Cancel if self.board.copy.is_some() => {
                vec![Effect::Message(ArrangeMsg::CloseDisplays)]
            }
            Command::Cancel => self.revert(),
            Command::Tab(forward) if !pressing => {
                self.tab(forward);
                Vec::new()
            }
            Command::Nudge(offset) if !pressing => self.nudge(offset),
            Command::Detent(up) if !pressing => self.detent(up),
            Command::Style(forward) if self.selection.id == Some(Target::Picker) => {
                vec![Effect::Layout(forward)]
            }
            _ => Vec::new(),
        }
    }

    pub(crate) fn step(&mut self, undo: bool) -> Vec<Effect> {
        if self.press.is_some() || self.reverting {
            return Vec::new();
        }
        self.spool.dismiss_toast(self.now);
        self.bump();
        self.restoring = true;
        vec![Effect::Message(if undo { ArrangeMsg::Undo } else { ArrangeMsg::Redo })]
    }

    pub fn bump(&mut self) {
        self.spool.pulse = self.now;
        self.nudged = None;
        self.hold(TIMING.pulse);
    }

    pub fn recorded(&mut self) {
        if !std::mem::take(&mut self.fresh) {
            self.spool.dismiss_toast(self.now);
        }
        self.bump();
    }

    pub(crate) fn revert(&mut self) -> Vec<Effect> {
        if self.reverting || self.spool.leaving {
            return Vec::new();
        }
        self.press = None;
        self.guides.clear();
        self.hover = None;
        self.deselect();
        self.spool.dismiss_toast(self.now);
        let hold = self.spool.wind();
        self.hold(hold);
        self.spool.drop = None;
        self.spool.sag_toward(self.pointer, false);
        self.bump();
        self.reverting = true;
        self.later(TIMING.revert_hold.max(TIMING.flight + TIMING.order), Job::Leave);
        vec![Effect::Message(ArrangeMsg::Cancel)]
    }

    pub fn leave(&mut self) {
        self.spool.leaving = true;
        self.preview = false;
        self.amount.aim(0.0);
        self.deselect();
        self.press = None;
        self.guides.clear();
        self.spool.dismiss_toast(self.now);
        let hold = self.spool.wind();
        self.hold(hold.max(TIMING.deselect));
        self.spool.drop = None;
        self.spool.sag_toward(self.pointer, false);
        for object in &mut self.objects {
            object.w.zeta(SIZE.zeta);
            object.h.zeta(SIZE.zeta);
        }
    }

    pub(crate) fn enter_preview(&mut self) -> Vec<Effect> {
        self.deselect();
        let hold = self.spool.wind();
        self.hold(hold.max(TIMING.label));
        self.spool.dismiss_toast(self.now);
        self.preview = true;
        self.amount.aim(0.0);
        vec![Effect::Message(ArrangeMsg::Preview)]
    }

    pub(crate) fn preview_to(&mut self, on: bool) {
        if on != self.preview {
            let _ = if on { self.enter_preview() } else { self.back() };
        }
    }

    pub(crate) fn back(&mut self) -> Vec<Effect> {
        if !self.preview {
            return Vec::new();
        }
        self.preview = false;
        self.amount.aim(1.0);
        self.hold(TIMING.label);
        vec![Effect::Message(ArrangeMsg::Preview)]
    }
}
