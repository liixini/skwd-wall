use std::collections::HashMap;

use super::labels::{self, DONE, group_width, plate_width};
use super::motion::Motor;
use super::tokens::{
    BEAD_FILL, BEAD_SIZE, DRAG, FRAME, GROUP, HIT_BEAD, KICKS, LOOK, PLATE, SAG, SAG_ALONG,
    SLOT_MOVE, THREAD, TIMING,
};
use crate::domain::strand::{
    BEAD, HOVER, Item, KindSpec, Placed, Plan, Rect, Route, StrandSpec, plan, shut, thread_y,
    zone_of,
};
use crate::frontend::animation::{eased, lerp, linear};
use crate::i18n::tr;

pub type BeadAt = (usize, usize);

const SPOOL_TAIL: f32 = 8.0;
const TOAST_GONE: f32 = 0.005;
pub const FOOT_GAP: f32 = 2.0;

pub struct Bead {
    pub size: Motor,
    pub fill: Motor,
}

impl Bead {
    fn new() -> Self {
        Self { size: Motor::new(BEAD, BEAD_SIZE), fill: Motor::new(0.0, BEAD_FILL) }
    }

    pub fn kick(&mut self, velocity: f32) {
        self.size.kick(velocity);
    }
}

pub struct Sag {
    pub along: Motor,
    pub depth: Motor,
    pub catch_at: Option<f32>,
}

pub struct Slot {
    pub from: f32,
    pub to: f32,
    pub at: f32,
}

pub struct Rise {
    pub from: Option<f32>,
    pub to: Option<f32>,
    pub at: f32,
}

pub enum Strike {
    Retract { from: [f32; 2] },
    Shoot { to: [f32; 2] },
}

impl Strike {
    pub fn duration(&self) -> f32 {
        match self {
            Self::Retract { .. } => TIMING.retract,
            Self::Shoot { .. } => TIMING.shoot + TIMING.retract,
        }
    }
}

pub struct Thread {
    pub bead: BeadAt,
    pub strike: Strike,
    pub at: f32,
}

pub struct Travel {
    pub bead: BeadAt,
    pub from: [f32; 2],
    pub at: f32,
}

pub struct Toast {
    pub bead: BeadAt,
    pub label: String,
    pub at: f32,
    pub leave: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub width: u32,
    pub open: Option<usize>,
    pub cancel: String,
    pub kinds: Vec<usize>,
    pub extras: Vec<i32>,
    pub spans: Vec<[i32; 2]>,
    pub lane: i32,
    pub dock: Vec<[i32; 2]>,
}

pub struct Laid {
    pub signature: Signature,
    pub plan: Plan,
    pub ranks: HashMap<Item, usize>,
    pub spans: Vec<[f32; 2]>,
}

pub struct Lineup<'a> {
    pub kinds: &'a [KindSpec],
    pub cancel: String,
    pub extras: &'a [f32],
    pub route: Route,
    pub crossing: bool,
}

pub struct Spool {
    pub output: [f32; 2],
    pub entered: Option<f32>,
    pub leaving: bool,
    pub open: Option<usize>,
    pub spooled: (Option<usize>, Option<usize>),
    pub slots: HashMap<Item, Slot>,
    pub rises: HashMap<Item, Rise>,
    pub lane: Motor,
    pub laid: Option<Laid>,
    pub beads: Vec<Vec<Bead>>,
    pub groups: Vec<Bead>,
    pub sag: Sag,
    pub drop: Option<(BeadAt, f32)>,
    pub threads: Vec<Thread>,
    pub travels: Vec<Travel>,
    pub toast: Option<Toast>,
    pub pulse: f32,
}

pub struct ToastPlates {
    pub label: String,
    pub up: f32,
    pub x: f32,
    pub y: f32,
    pub label_width: f32,
    pub undo_width: f32,
    pub slot: [f32; 2],
    pub foot: f32,
}

impl ToastPlates {
    pub fn undo_rect(&self) -> Rect {
        Rect::new(self.x + self.label_width + PLATE.gap, self.y, self.undo_width, PLATE.height)
    }
}

impl Spool {
    pub fn new(output: [f32; 2], kinds: &[usize]) -> Self {
        Self {
            output,
            entered: None,
            leaving: false,
            open: None,
            spooled: (None, None),
            slots: HashMap::new(),
            rises: HashMap::new(),
            lane: Motor::new(thread_y(output[1]), DRAG),
            laid: None,
            beads: kinds.iter().map(|count| (0..*count).map(|_| Bead::new()).collect()).collect(),
            groups: kinds.iter().map(|_| Bead::new()).collect(),
            sag: Sag {
                along: Motor::new(output[0] / 2.0, SAG_ALONG),
                depth: Motor::new(0.0, SAG),
                catch_at: None,
            },
            drop: None,
            threads: Vec::new(),
            travels: Vec::new(),
            toast: None,
            pulse: f32::NEG_INFINITY,
        }
    }

    pub fn thread_at(&self, x: f32) -> f32 {
        let distance = (x - self.sag.along.value()) / THREAD.sag_width;
        self.lane.value() + self.sag.depth.value() * (-distance * distance).exp()
    }

    pub fn zone(&self) -> [f32; 2] {
        zone_of(self.lane.target())
    }

    pub fn in_zone(&self, y: f32) -> bool {
        let [top, floor] = self.zone();
        y > top && y <= floor
    }

    pub fn plate_y(&self, now: f32, item: Item, x: f32, width: f32) -> f32 {
        let thread = self.thread_at(x + width / 2.0) - PLATE.height / 2.0;
        let Some(rise) = self.rises.get(&item) else { return thread };
        let (from, to) = (rise.from.unwrap_or(thread), rise.to.unwrap_or(thread));
        lerp(from, to, eased(now, rise.at, TIMING.spool))
    }

    pub fn docked(&self) -> Vec<Rect> {
        let Some(laid) = self.laid.as_ref() else { return Vec::new() };
        laid.plan
            .items
            .iter()
            .filter_map(|placed| {
                placed.top.map(|top| Rect::new(placed.x, top, placed.reach[1], PLATE.height))
            })
            .collect()
    }

    pub fn held(&self) -> Vec<Rect> {
        let Some(laid) = self.laid.as_ref() else { return Vec::new() };
        let lane = self.lane.target();
        let visible = laid.plan.items.iter().filter(|placed| !placed.off && placed.top.is_none());
        let row = visible.flat_map(|placed| {
            let [from, to] = placed.reach;
            let width = to - from;
            match placed.item {
                Item::Group(_) => {
                    let top = lane - HOVER - GROUP.stem - GROUP.height;
                    vec![
                        Rect::new(placed.x + from, top, width, GROUP.height),
                        Rect::centred([placed.x, lane], [2.0 * HOVER; 2]),
                    ]
                }
                Item::Bead(..) => vec![Rect::centred([placed.x, lane], [2.0 * HOVER; 2])],
                _ => {
                    vec![Rect::new(placed.x + from, lane - PLATE.height / 2.0, width, PLATE.height)]
                }
            }
        });
        row.chain(self.docked()).collect()
    }

    pub fn item_x(&self, now: f32, item: Item) -> f32 {
        self.slots.get(&item).map_or(self.output[0] / 2.0, |slot| {
            lerp(slot.from, slot.to, eased(now, slot.at, TIMING.spool))
        })
    }

    pub fn item_moving(&self, now: f32, item: Item) -> bool {
        self.slots.get(&item).is_some_and(|slot| linear(now, slot.at, TIMING.spool) < 1.0)
            || self.rises.get(&item).is_some_and(|rise| linear(now, rise.at, TIMING.spool) < 1.0)
    }

    pub fn slot(&self, now: f32, bead: BeadAt) -> [f32; 2] {
        let x = self.item_x(now, Item::Bead(bead.0, bead.1));
        [x, self.thread_at(x)]
    }

    pub fn reach(&self, now: f32) -> f32 {
        match self.entered {
            Some(at) if !self.leaving => eased(now, at, TIMING.enter),
            _ => 1.0,
        }
    }

    pub fn drop_in(&self, now: f32, item: Item) -> f32 {
        let (Some(at), Some(laid)) = (self.entered, self.laid.as_ref()) else { return 0.0 };
        if self.leaving {
            return 1.0;
        }
        let rank = laid.ranks.get(&item).copied().unwrap_or_default() as f32;
        eased(now, at + TIMING.item_delay + rank * TIMING.stagger, TIMING.item)
    }

    pub fn folded(&self, kind: usize) -> bool {
        self.laid.as_ref().is_some_and(|laid| laid.plan.folded[kind])
    }

    pub fn off(&self, item: Item) -> bool {
        self.laid.as_ref().is_some_and(|laid| {
            laid.plan.items.iter().any(|placed| placed.item == item && placed.off)
        })
    }

    pub fn relayout(&mut self, now: f32, lineup: Lineup<'_>) -> f32 {
        let Lineup { kinds, cancel, extras, route, crossing } = lineup;
        let Route { lane, margin, spans: obstacles, dock } = route;
        let counts: Vec<usize> = kinds.iter().map(|kind| kind.styles).collect();
        let signature = Signature {
            width: self.output[0].round() as u32,
            open: self.open,
            cancel: cancel.clone(),
            kinds: counts.clone(),
            extras: extras.iter().map(|width| width.round() as i32).collect(),
            spans: obstacles.iter().map(|span| span.map(|edge| edge.round() as i32)).collect(),
            lane: lane.round() as i32,
            dock: dock.iter().map(|spot| spot.map(|edge| edge.round() as i32)).collect(),
        };
        if self.laid.as_ref().is_some_and(|laid| laid.signature == signature) {
            return 0.0;
        }
        self.fit_beads(&counts);
        if self.laid.is_none() {
            self.lane.jump(lane);
        }
        self.lane.aim(lane);
        let spec = StrandSpec {
            width: self.output[0],
            kinds,
            cancel: plate_width(&cancel),
            done: plate_width(tr(DONE)),
            extras,
            open: self.open.filter(|kind| *kind < counts.len()),
            obstacles: &obstacles,
            margin,
            dock: &dock,
        };
        let mut plan = plan(&spec);
        let closed = self.open.is_some_and(|kind| plan.folded.get(kind).copied().unwrap_or(false));
        let crossed = crossing && closed;
        if crossed {
            plan = shut(&StrandSpec { obstacles: &[], ..spec });
        }
        let middle = self.output[0] / 2.0;
        let mut order: Vec<(Item, f32)> = plan
            .items
            .iter()
            .map(|placed| (placed.item, (placed.centre() - middle).abs()))
            .collect();
        order.sort_by(|one, other| one.1.total_cmp(&other.1));
        let ranks: HashMap<Item, usize> =
            order.iter().enumerate().map(|(rank, (item, _))| (*item, rank)).collect();
        let mut hold: f32 = 0.0;
        for placed in &plan.items {
            let delay = match placed.item {
                Item::Bead(kind, style) => {
                    let count = counts[kind];
                    if self.spooled.0 == Some(kind) {
                        style as f32 * TIMING.spool_stagger
                    } else if self.spooled.1 == Some(kind) {
                        (count - 1 - style) as f32 * TIMING.spool_stagger
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            };
            hold = hold.max(self.tween(now, placed.item, placed.x, delay));
            hold = hold.max(self.rise(now, placed));
        }
        let spans = if crossed { Vec::new() } else { obstacles };
        self.laid = Some(Laid { signature, plan, ranks, spans });
        hold
    }

    fn fit_beads(&mut self, counts: &[usize]) {
        self.beads.resize_with(counts.len(), Vec::new);
        for (row, count) in self.beads.iter_mut().zip(counts) {
            row.resize_with(*count, Bead::new);
        }
        self.groups.resize_with(counts.len(), Bead::new);
    }

    fn tween(&mut self, now: f32, item: Item, to: f32, delay: f32) -> f32 {
        let current = self.item_x(now, item);
        match self.slots.get_mut(&item) {
            None => {
                self.slots.insert(item, Slot { from: to, to, at: f32::NEG_INFINITY });
                0.0
            }
            Some(slot) if (slot.to - to).abs() > SLOT_MOVE => {
                *slot = Slot { from: current, to, at: now + delay };
                delay + TIMING.spool
            }
            Some(_) => 0.0,
        }
    }

    fn rise(&mut self, now: f32, placed: &Placed) -> f32 {
        let same = |rise: &Rise| match (rise.to, placed.top) {
            (Some(one), Some(other)) => (one - other).abs() <= SLOT_MOVE,
            (None, None) => true,
            _ => false,
        };
        match self.rises.get(&placed.item) {
            None if placed.top.is_none() => 0.0,
            Some(rise) if same(rise) => 0.0,
            _ if self.laid.is_none() => {
                let settled = Rise { from: placed.top, to: placed.top, at: f32::NEG_INFINITY };
                self.rises.insert(placed.item, settled);
                0.0
            }
            _ => {
                let x = self.item_x(now, placed.item);
                let from = Some(self.plate_y(now, placed.item, x, placed.reach[1]));
                self.rises.insert(placed.item, Rise { from, to: placed.top, at: now });
                TIMING.spool
            }
        }
    }

    pub fn hit(&self, now: f32, at: [f32; 2]) -> Option<Item> {
        let laid = self.laid.as_ref()?;
        laid.plan.items.iter().filter(|placed| !placed.off).find_map(|placed| {
            let x = self.item_x(now, placed.item);
            let rect = match placed.item {
                Item::Bead(..) => Rect::centred([x, self.thread_at(x)], [2.0 * HIT_BEAD; 2]),
                Item::Group(kind) => {
                    let y = self.thread_at(x);
                    let plate = group_width(self.beads[kind].len());
                    let top = y - HOVER - GROUP.stem - GROUP.height;
                    let left = (x - HIT_BEAD).min(x - plate / 2.0);
                    Rect::new(left, top, (2.0 * HIT_BEAD).max(plate), y + HIT_BEAD - top)
                }
                Item::Kind(kind) if !(self.folded(kind) || self.open == Some(kind)) => {
                    return None;
                }
                Item::Kind(_) | Item::Extra(_) | Item::Cancel | Item::Done => {
                    let width = placed.reach[1];
                    Rect::new(x, self.plate_y(now, placed.item, x, width), width, PLATE.height)
                }
            };
            rect.contains(at).then_some(placed.item)
        })
    }

    pub fn size_beads(
        &mut self,
        hovered: Option<BeadAt>,
        hovered_kind: Option<usize>,
        dropping: Option<BeadAt>,
    ) {
        let drop_kind = dropping.map(|bead| bead.0);
        for (kind, row) in self.beads.iter_mut().enumerate() {
            for (style, bead) in row.iter_mut().enumerate() {
                let big = hovered == Some((kind, style)) || dropping == Some((kind, style));
                bead.size.aim(if big { HOVER } else { BEAD });
            }
        }
        for (kind, group) in self.groups.iter_mut().enumerate() {
            let big = hovered_kind == Some(kind) || drop_kind == Some(kind);
            group.size.aim(if big { HOVER } else { BEAD });
        }
    }

    pub fn fill(&mut self, placed: &[Vec<bool>]) {
        for (kind, row) in placed.iter().enumerate() {
            for (style, filled) in row.iter().enumerate() {
                if let Some(bead) = self.beads.get_mut(kind).and_then(|row| row.get_mut(style)) {
                    bead.fill.aim(if *filled { 1.0 } else { 0.0 });
                }
            }
            if let Some(group) = self.groups.get_mut(kind) {
                group.fill.aim(if row.iter().any(|filled| *filled) { 1.0 } else { 0.0 });
            }
        }
    }

    pub fn kick(&mut self, bead: BeadAt, velocity: f32, group: bool) {
        if let Some(motion) = self.beads.get_mut(bead.0).and_then(|row| row.get_mut(bead.1)) {
            motion.kick(velocity);
        }
        if group && let Some(motion) = self.groups.get_mut(bead.0) {
            motion.kick(velocity);
        }
    }

    pub fn unspool(&mut self, kind: usize) -> f32 {
        if self.open == Some(kind) {
            return 0.0;
        }
        self.spooled = (Some(kind), self.open);
        self.open = Some(kind);
        TIMING.spool + SPOOL_TAIL * TIMING.spool_stagger
    }

    pub fn wind(&mut self) -> f32 {
        let Some(kind) = self.open.take() else { return 0.0 };
        self.spooled = (None, Some(kind));
        TIMING.spool + SPOOL_TAIL * TIMING.spool_stagger
    }

    pub fn sag_toward(&mut self, at: [f32; 2], dropping: bool) {
        self.sag.depth.zeta(SAG.zeta);
        if dropping {
            let top = self.zone()[0];
            let deep = ((at[1] - top) / THREAD.sag_reach).clamp(THREAD.sag_floor, 1.0);
            self.sag.along.aim(at[0]);
            self.sag.depth.aim(THREAD.sag_max * deep);
        } else {
            self.sag.depth.aim(0.0);
        }
    }

    pub fn tick(&mut self, now: f32, dt: f32) {
        if self.sag.catch_at.is_some_and(|at| now >= at) {
            self.sag.catch_at = None;
            self.sag.depth.zeta(KICKS.catch_zeta);
            self.sag.depth.aim(0.0);
            self.sag.depth.kick(KICKS.catch);
        }
        self.sag.along.tick(dt);
        self.sag.depth.tick(dt);
        self.lane.tick(dt);
        if self.sag.depth.still() {
            self.sag.depth.zeta(SAG.zeta);
        }
        for bead in self.beads.iter_mut().flatten().chain(&mut self.groups) {
            bead.size.tick(dt);
            bead.fill.tick(dt);
        }
        let landed: Vec<BeadAt> = self
            .travels
            .iter()
            .filter(|travel| now >= travel.at + TIMING.travel)
            .map(|travel| travel.bead)
            .collect();
        self.travels.retain(|travel| now < travel.at + TIMING.travel);
        for bead in landed {
            self.kick(bead, KICKS.land, true);
        }
        self.threads.retain(|thread| now < thread.at + thread.strike.duration());
        self.toast = self.toast.take().filter(|toast| now <= toast.leave + TIMING.toast_out);
    }

    pub fn motors(&self) -> impl Iterator<Item = &Motor> {
        self.beads
            .iter()
            .flatten()
            .chain(&self.groups)
            .flat_map(|bead| [&bead.size, &bead.fill])
            .chain([&self.sag.along, &self.sag.depth, &self.lane])
    }

    pub fn toast(&mut self, now: f32, bead: BeadAt, label: String) -> f32 {
        self.toast =
            Some(Toast { bead, label, at: now, leave: now + TIMING.toast - TIMING.toast_out });
        TIMING.toast
    }

    pub fn dismiss_toast(&mut self, now: f32) {
        if let Some(toast) = self.toast.as_mut() {
            toast.leave = toast.leave.min(now);
        }
    }

    fn toast_top(&self, top: f32, slot: f32, span: [f32; 2]) -> f32 {
        let docked = self.docked();
        let top = if top < FRAME.edge { slot + HOVER + PLATE.gap } else { top };
        let rect = Rect::new(span[0], top, span[1], PLATE.height);
        if !docked.iter().any(|plate| plate.overlaps(rect)) {
            return top;
        }
        let first = docked.iter().map(|plate| plate.y).fold(f32::INFINITY, f32::min);
        let above = first - PLATE.gap - PLATE.height;
        if above >= FRAME.edge {
            return above;
        }
        docked.iter().map(|plate| plate.bottom()).fold(top, f32::max) + PLATE.gap
    }

    pub fn toast_plates(&self, now: f32) -> Option<ToastPlates> {
        let toast = self.toast.as_ref()?;
        let slot = self.slot(now, toast.bead);
        let capped = self.off(Item::Bead(toast.bead.0, toast.bead.1));
        let up = eased(now, toast.at, TIMING.toast_in)
            * (1.0 - eased(now, toast.leave, TIMING.toast_out));
        if up <= TOAST_GONE {
            return None;
        }
        let label = toast.label.clone();
        let label_width = plate_width(&label);
        let undo_width = plate_width(tr(labels::UNDO));
        let total = label_width + PLATE.gap + undo_width;
        let x = (slot[0] - total / 2.0).clamp(FRAME.edge, self.output[0] - FRAME.edge - total);
        let raised = if capped { GROUP.stem + GROUP.height } else { 0.0 };
        let foot = slot[1] - HOVER - FOOT_GAP - raised;
        let capped_rise = if capped { GROUP.height } else { 0.0 };
        let top = slot[1] - LOOK.toast_rise - PLATE.height - capped_rise;
        let top = self.toast_top(top, slot[1], [x, total]);
        let y = lerp(slot[1] - PLATE.height / 2.0, top, up);
        Some(ToastPlates { label, up, x, y, label_width, undo_width, slot, foot })
    }
}
