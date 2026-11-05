use std::collections::HashMap;

use iced::widget::canvas;

use paper_widgets::Tones;

use super::labels;
use super::model::{Board, Target, Tool};
use crate::domain::strand::{Guide, HOVER, Item, KindSpec, Rect, RouteSpec, route};
use crate::frontend::strand::labels::{DONE, cancel, group_width, plate_width};
use crate::frontend::strand::motion::Motor;
use crate::frontend::strand::selection::Selection;
use crate::frontend::strand::spool::{BeadAt, Lineup, Spool};
use crate::frontend::strand::tokens::{DRAG, FRAME, GROUP, SCRIM, SIZE, THREAD, TIMING};
use crate::i18n::tr;

const SETTLED: f32 = 0.001;

pub(crate) struct Object {
    pub(crate) x: Motor,
    pub(crate) y: Motor,
    pub(crate) w: Motor,
    pub(crate) h: Motor,
}

impl Object {
    fn new(rect: Rect) -> Self {
        Self {
            x: Motor::new(rect.x, DRAG),
            y: Motor::new(rect.y, DRAG),
            w: Motor::new(rect.w, SIZE),
            h: Motor::new(rect.h, SIZE),
        }
    }

    pub(crate) fn shown(&self) -> Rect {
        Rect::new(self.x.value(), self.y.value(), self.w.value(), self.h.value())
    }

    pub(crate) fn target(&self) -> Rect {
        Rect::new(self.x.target(), self.y.target(), self.w.target(), self.h.target())
    }

    pub(crate) fn aim(&mut self, rect: Rect) {
        self.x.aim(rect.x);
        self.y.aim(rect.y);
        self.w.aim(rect.w);
        self.h.aim(rect.h);
    }

    fn motors(&self) -> [&Motor; 4] {
        [&self.x, &self.y, &self.w, &self.h]
    }

    fn tick(&mut self, dt: f32) {
        for motor in [&mut self.x, &mut self.y, &mut self.w, &mut self.h] {
            motor.tick(dt);
        }
    }

    pub(crate) fn still(&self) -> bool {
        self.motors().iter().all(|motor| motor.still())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Choice {
    Recipient(usize),
    Fit,
    Reconnect,
    Apply,
    Skip,
    Forget,
    Keep,
    Saved,
    Consent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Hit {
    Tool(Tool),
    Gem(usize),
    Undo,
    Item(Item),
    Object(Target),
    Button(usize),
    Choice(Choice),
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Extra {
    Preview,
    Displays,
}

pub(crate) enum Press {
    Plate(Hit),
    Gem { corner: usize, anchor: [f32; 2], snapped: bool },
    Object { target: Target, grab: [f32; 2], start: [f32; 2], moved: bool },
    Button { id: String, start: [f32; 2], moved: bool },
    Bead { bead: BeadAt, start: [f32; 2], pulling: bool },
    Kind,
    Empty,
}

pub(crate) enum Job {
    Leave,
}

pub struct Arrange {
    pub(crate) now: f32,
    pub(crate) busy: f32,
    pub(crate) output: [f32; 2],
    pub(crate) board: Board,
    pub(crate) objects: [Object; Target::ALL.len()],
    pub(crate) tones: Tones,
    pub(crate) selection: Selection<Target>,
    pub(crate) press: Option<Press>,
    pub(crate) hover: Option<(Hit, f32)>,
    pub(crate) spool: Spool,
    pub(crate) guides: Vec<Guide>,
    pub(crate) guide_seen: HashMap<(bool, i32), f32>,
    pub(crate) shake: Option<f32>,
    pub(crate) flash: Option<f32>,
    pub(crate) jobs: Vec<(f32, Job)>,
    pub(crate) reverting: bool,
    pub(crate) amount: Motor,
    pub(crate) preview: bool,
    pub(crate) nudged: Option<(Target, f32)>,
    pub(crate) shoot: Option<(String, BeadAt)>,
    pub(crate) pointer: [f32; 2],
    pub(crate) restoring: bool,
    pub(crate) fresh: bool,
    pub(crate) thread: canvas::Cache,
}

impl Arrange {
    pub fn new(output: [f32; 2], board: Board, tones: Tones) -> Self {
        let counts: Vec<usize> = board.shelves.iter().map(|shelf| shelf.beads.len()).collect();
        let objects = board.rects.map(Object::new);
        let mut arrange = Self {
            now: 0.0,
            busy: 0.0,
            output,
            board,
            objects,
            tones,
            selection: Selection::new(),
            press: None,
            hover: None,
            spool: Spool::new(output, &counts),
            guides: Vec::new(),
            guide_seen: HashMap::new(),
            shake: None,
            flash: None,
            jobs: Vec::new(),
            reverting: false,
            amount: Motor::new(0.0, SCRIM),
            preview: false,
            nudged: None,
            shoot: None,
            pointer: [output[0] / 2.0, output[1] / 2.0],
            restoring: false,
            fresh: false,
            thread: canvas::Cache::new(),
        };
        arrange.spool.entered = Some(0.0);
        arrange.amount.aim(1.0);
        let items = counts.iter().map(|count| count + 2).sum::<usize>() + EXTRA_ITEMS;
        arrange.hold(TIMING.item_delay + TIMING.item + TIMING.stagger * items as f32);
        arrange.fill();
        arrange.relayout();
        arrange
    }

    pub fn sync(&mut self, board: Board, tones: Tones) {
        let changed: Vec<Target> = Target::ALL
            .into_iter()
            .filter(|target| board.rects[target.index()] != self.board.rects[target.index()])
            .collect();
        let dragged = self.dragged();
        for target in Target::ALL {
            if Some(target) != dragged {
                self.objects[target.index()].aim(board.rects[target.index()]);
            }
        }
        if self.restoring
            && let Some(first) = changed.first()
        {
            self.select(*first);
        }
        self.restoring = false;
        if let Some((id, bead)) = self.shoot.take() {
            let floating = Target::floating(&id)
                .filter(|target| board.present[target.index()])
                .map(|target| board.rects[target.index()]);
            let button =
                board.buttons.iter().find(|button| button.id == id).map(|button| button.rect);
            match button.or(floating) {
                Some(rect) => self.strike(bead, rect),
                None => self.shoot = Some((id, bead)),
            }
        }
        let stale = board.shelves != self.board.shelves
            || board.steps != self.board.steps
            || board.displays != self.board.displays
            || board.copy.is_some() != self.board.copy.is_some()
            || tones != self.tones;
        self.board = board;
        self.tones = tones;
        if self.selection.id.is_some_and(|target| !self.board.present[target.index()]) {
            self.deselect();
        }
        self.fill();
        self.relayout();
        if stale {
            self.thread.clear();
        }
    }

    pub(crate) fn targets(&self) -> Vec<Target> {
        Target::ALL.into_iter().filter(|target| self.board.present[target.index()]).collect()
    }

    pub(crate) fn dragged(&self) -> Option<Target> {
        match &self.press {
            Some(Press::Object { target, moved: true, .. }) => Some(*target),
            Some(Press::Gem { .. }) => Some(Target::Picker),
            _ => None,
        }
    }

    pub(crate) fn shown(&self, target: Target) -> Option<Rect> {
        let object = &self.objects[target.index()];
        (!object.still()).then(|| object.shown())
    }

    pub(crate) fn rect(&self, target: Target) -> Rect {
        self.objects[target.index()].shown()
    }

    pub fn hold(&mut self, span: f32) {
        self.busy = self.busy.max(self.now + span);
    }

    pub(crate) fn later(&mut self, delay: f32, job: Job) {
        let at = self.now + delay;
        self.busy = self.busy.max(at);
        self.jobs.push((at, job));
    }

    pub(crate) fn reach(&self) -> f32 {
        self.spool.reach(self.now)
    }

    pub(crate) fn fade(&self) -> f32 {
        if self.spool.leaving || self.preview { self.amount.value() } else { 1.0 }
    }

    pub fn hits_ready(&self) -> bool {
        !self.spool.leaving && !self.reverting && !self.preview && self.reach() > THREAD.reach_show
    }

    pub fn entering(&self) -> bool {
        !self.spool.leaving && !self.reverting && self.reach() <= THREAD.reach_show
    }

    pub fn finished(&self) -> bool {
        self.spool.leaving && self.amount.value() <= SETTLED && self.amount.still()
    }

    pub fn previewing(&self) -> bool {
        self.preview
    }

    pub fn reverting(&self) -> bool {
        self.reverting
    }

    pub(crate) fn selected(&self) -> Option<Target> {
        self.selection.id
    }

    pub fn animating(&self) -> bool {
        self.now < self.busy || self.restless()
    }

    fn restless(&self) -> bool {
        let mut motors: Vec<&Motor> = self.objects.iter().flat_map(Object::motors).collect();
        motors.extend(self.spool.motors());
        motors.extend(self.selection.motors());
        motors.push(&self.amount);
        motors.iter().any(|motor| !motor.still())
    }

    pub fn tick(&mut self, dt: f32) {
        self.now += dt;
        let now = self.now;
        self.amount.tick(dt);
        let (due, rest): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.jobs).into_iter().partition(|(at, _)| *at <= now);
        self.jobs = rest;
        for (_, job) in due {
            match job {
                Job::Leave => self.leave(),
            }
        }
        self.spool.tick(now, dt);
        for object in &mut self.objects {
            object.tick(dt);
        }
        self.selection.expire(now);
        self.shake = self.shake.filter(|at| now <= at + TIMING.shake);
        self.flash = self.flash.filter(|at| now <= at + TIMING.readout);
        let bounds = self.selection.id.map(|target| self.rect(target));
        self.selection.follow(bounds, dt);
        self.fill();
        self.relayout();
        if !self.thread_still() {
            self.thread.clear();
        }
    }

    pub(crate) fn thread_still(&self) -> bool {
        let now = self.now;
        let Some(laid) = self.spool.laid.as_ref() else { return false };
        let entered = self.spool.entered.unwrap_or_default();
        let entrance =
            TIMING.item_delay + TIMING.item + TIMING.stagger * laid.plan.items.len() as f32;
        let landed = now >= entered + TIMING.enter.max(entrance);
        let moving = laid.plan.items.iter().any(|placed| self.spool.item_moving(now, placed.item));
        let labelled = self.hover.is_some_and(|(_, at)| now < at + TIMING.label)
            || self.spool.drop.is_some_and(|(_, at)| now < at + TIMING.label);
        landed
            && !moving
            && !labelled
            && self.spool.motors().all(Motor::still)
            && self.amount.still()
            && self.spool.toast.is_none()
            && self.spool.threads.is_empty()
            && self.spool.travels.is_empty()
            && now >= self.spool.pulse + TIMING.pulse
    }

    pub(crate) fn extras(&self) -> Vec<Extra> {
        let mut extras = vec![Extra::Preview];
        if self.board.displays {
            extras.push(Extra::Displays);
        }
        extras
    }

    pub(crate) fn bead_id(&self, bead: BeadAt) -> Option<&str> {
        let shelf = self.board.shelves.get(bead.0)?;
        shelf.beads.get(bead.1).map(|bead| bead.id.as_str())
    }

    pub(crate) fn pullable(&self, bead: BeadAt) -> bool {
        !self.shown_bead(bead) && self.bead_id(bead).and_then(Target::floating).is_none()
    }

    pub(crate) fn bead_of(&self, id: &str) -> Option<BeadAt> {
        self.board.shelves.iter().enumerate().find_map(|(kind, shelf)| {
            shelf.beads.iter().position(|bead| bead.id == id).map(|style| (kind, style))
        })
    }

    pub(crate) fn shown_bead(&self, bead: BeadAt) -> bool {
        self.board
            .shelves
            .get(bead.0)
            .and_then(|shelf| shelf.beads.get(bead.1))
            .is_some_and(|bead| bead.shown)
    }

    fn fill(&mut self) {
        let placed: Vec<Vec<bool>> = self
            .board
            .shelves
            .iter()
            .map(|shelf| shelf.beads.iter().map(|bead| bead.shown).collect())
            .collect();
        self.spool.fill(&placed);
    }

    pub(crate) fn relayout(&mut self) {
        let skip = self.dragged();
        let mut boxes: Vec<Rect> = self
            .targets()
            .into_iter()
            .filter(|target| Some(*target) != skip)
            .map(|target| self.rect(target))
            .collect();
        boxes.extend(self.panel().into_iter().map(|slot| slot.rect));
        let kinds: Vec<KindSpec> = self
            .board
            .shelves
            .iter()
            .map(|shelf| KindSpec {
                plate: plate_width(labels::family(shelf.family)),
                group: group_width(shelf.beads.len()),
                styles: shelf.beads.len(),
            })
            .collect();
        let extras: Vec<f32> =
            self.extras().into_iter().map(|extra| plate_width(labels::extra(extra))).collect();
        let cancel = cancel(self.board.steps);
        let route = route(&RouteSpec {
            output: self.output,
            kinds: &kinds,
            cancel: plate_width(&cancel),
            done: plate_width(tr(DONE)),
            extras: &extras,
            boxes: &boxes,
            crown: HOVER + GROUP.stem + GROUP.height,
            pad: FRAME.pad,
            gem: FRAME.gem * FRAME.gem_grow,
            edge: FRAME.edge,
        });
        let lineup = Lineup { kinds: &kinds, cancel, extras: &extras, route, crossing: true };
        let before = self.spool.laid.as_ref().map(|laid| laid.signature.clone());
        let hold = self.spool.relayout(self.now, lineup);
        self.hold(hold);
        if self.spool.laid.as_ref().map(|laid| &laid.signature) != before.as_ref() {
            self.thread.clear();
        }
    }
}

const EXTRA_ITEMS: usize = 4;
