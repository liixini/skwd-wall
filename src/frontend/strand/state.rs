use std::collections::HashMap;

use paper_control::{Depth, Face, MAX_WIDGETS, Widget};
use paper_widgets::{Moment, Tones};

use super::motion::Motor;
use super::raster::{Hatch, Key, Proxy, Scenery, Stage, chrome, tones};
use super::selection::Selection;
use super::spool::{BeadAt, Spool};
use super::tokens::{DRAG, Feel, LIFTING, LOOK, SCRIM, SIZE, SQUASHING, TIMING, WHIP};
use crate::app::EditHistory;
use crate::domain::strand::{Grip, Guide, Item, KINDS, Kind, Rect, Saved, Spot, style_index};

const SETTLED: f32 = 0.001;
const STEP_LABEL: &str = "strand";

pub struct Out {
    pub face: Face,
    pub at: f32,
    pub centre: [f32; 2],
    pub height: f32,
}

pub struct Wipe {
    pub at: f32,
    pub from: Depth,
}

pub struct Swap {
    pub at: f32,
    pub amp: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fate {
    Collapse,
    Home,
}

pub struct Dying {
    pub fate: Fate,
    pub at: f32,
    pub from: [f32; 2],
}

pub struct Live {
    pub id: u32,
    pub face: Face,
    pub depth: Depth,
    pub x: Motor,
    pub y: Motor,
    pub h: Motor,
    pub lift: Motor,
    pub squash: Motor,
    pub born: Option<f32>,
    pub reveal: Option<f32>,
    pub out: Option<Out>,
    pub wipe: Option<Wipe>,
    pub swap: Option<Swap>,
    pub dying: Option<Dying>,
    pub hidden: bool,
    pub tones: Option<Tones>,
    pub chrome: Option<Tones>,
    pub toned: Option<[i32; 3]>,
}

impl Live {
    pub fn new(spot: Spot) -> Self {
        Self {
            id: spot.id,
            face: spot.face,
            depth: spot.depth,
            x: Motor::new(spot.centre[0], DRAG),
            y: Motor::new(spot.centre[1], DRAG),
            h: Motor::new(spot.height, SIZE),
            lift: Motor::new(1.0, LIFTING),
            squash: Motor::new(1.0, SQUASHING),
            born: None,
            reveal: None,
            out: None,
            wipe: None,
            swap: None,
            dying: None,
            hidden: false,
            tones: None,
            chrome: None,
            toned: None,
        }
    }

    pub fn spot(&self) -> Spot {
        Spot {
            id: self.id,
            face: self.face,
            depth: self.depth,
            centre: [self.x.target(), self.y.target()],
            height: self.h.target(),
        }
    }

    pub fn bounds(&self) -> Rect {
        self.spot().bounds()
    }

    pub fn aim(&mut self, centre: [f32; 2]) {
        self.x.aim(centre[0]);
        self.y.aim(centre[1]);
    }

    fn motors(&self) -> [&Motor; 5] {
        [&self.x, &self.y, &self.h, &self.lift, &self.squash]
    }

    fn tick(&mut self, dt: f32) {
        for motor in [&mut self.x, &mut self.y, &mut self.h, &mut self.lift, &mut self.squash] {
            motor.tick(dt);
        }
    }

    fn expire(&mut self, now: f32) {
        self.born = self.born.filter(|at| now < at + TIMING.bloom);
        self.reveal = self.reveal.filter(|at| now < at + reveal_span());
        self.out = self.out.take().filter(|out| now < out.at + TIMING.out);
        self.wipe = self.wipe.take().filter(|wipe| now < wipe.at + TIMING.wipe);
        self.swap = self.swap.take().filter(|swap| now < swap.at + TIMING.order);
    }

    pub fn gone(&self, now: f32) -> bool {
        self.dying.as_ref().is_some_and(|dying| now >= dying.at + dying.fate.duration())
    }
}

impl Fate {
    pub fn duration(self) -> f32 {
        match self {
            Self::Collapse => TIMING.collapse,
            Self::Home => TIMING.home,
        }
    }
}

pub fn reveal_span() -> f32 {
    TIMING.glyph_cap * TIMING.glyph_stagger + TIMING.glyph
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Behind,
    Front,
    Raise,
    Lower,
    Prev,
    Style,
    Next,
    Remove,
}

impl Tool {
    pub const ALL: [Self; 8] = [
        Self::Behind,
        Self::Front,
        Self::Raise,
        Self::Lower,
        Self::Prev,
        Self::Style,
        Self::Next,
        Self::Remove,
    ];

    pub fn leads(self) -> bool {
        matches!(self, Self::Raise | Self::Prev | Self::Remove)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hit {
    Tool(Tool),
    Gem(usize),
    Undo,
    Item(Item),
    Widget(u32),
}

pub enum Press {
    Plate(Hit),
    Gem { id: u32, corner: usize, grip: Grip, snapped: bool, before: Vec<Spot> },
    Bead { face: Face, start: [f32; 2], pulling: bool },
    Widget { id: u32, grab: [f32; 2], start: [f32; 2], moved: bool, before: Vec<Spot> },
    Kind,
    Empty,
}

pub struct Pull {
    pub live: Live,
    pub whip: [Motor; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shake {
    Style,
    Order,
}

pub enum Job {
    Restore(Spot),
    Home(u32),
    Revive(Spot, usize),
    Reorder(Vec<u32>),
    Shift(u32, usize),
    Calm(u32),
    Leave,
}

pub struct Strand {
    pub now: f32,
    pub busy: f32,
    pub output: [f32; 2],
    pub scale: f32,
    pub name: Option<String>,
    pub room: usize,
    pub scenery: Option<Scenery>,
    pub stage: Option<Stage>,
    pub hatch_image: Option<Hatch>,
    pub widgets: Vec<Live>,
    pub rasters: HashMap<u32, Proxy>,
    pub outgoing: HashMap<u32, Proxy>,
    pub history: EditHistory<Vec<Spot>>,
    pub held: Vec<usize>,
    pub sources: HashMap<u32, (Spot, usize, Widget)>,
    pub seq: u32,
    pub selection: Selection<u32>,
    pub press: Option<Press>,
    pub hover: Option<(Hit, f32)>,
    pub spool: Spool,
    pub guides: Vec<Guide>,
    pub guide_seen: HashMap<(bool, i32), f32>,
    pub hatch: Option<f32>,
    pub flash: Option<(u32, f32)>,
    pub arranging: Option<f32>,
    pub jobs: Vec<(f32, Job)>,
    pub reverting: bool,
    pub amount: Motor,
    pub pull: Option<Pull>,
    pub nudged: Option<(u32, f32)>,
    pub shake: Option<(f32, Shake)>,
    pub band: Option<Tones>,
    pub whole: Option<Tones>,
    pub scene: Option<paper_control::WidgetScene>,
    pub minute: Option<std::time::Instant>,
}

fn bead_counts() -> Vec<usize> {
    KINDS.iter().map(|kind| kind.styles().len()).collect()
}

pub fn bead_of(face: Face) -> BeadAt {
    (Kind::of(face).index(), style_index(face))
}

fn item_count() -> f32 {
    KINDS.iter().map(|kind| kind.styles().len() + 2).sum::<usize>() as f32 + 2.0
}

fn motor(value: f32, feel: Feel) -> Motor {
    Motor::new(value, feel)
}

impl Strand {
    pub fn new(output: [f32; 2], scale: f32, name: Option<String>) -> Self {
        Self {
            now: 0.0,
            busy: 0.0,
            output,
            scale,
            name,
            room: MAX_WIDGETS,
            scenery: None,
            stage: None,
            hatch_image: None,
            widgets: Vec::new(),
            rasters: HashMap::new(),
            outgoing: HashMap::new(),
            history: EditHistory::new(Vec::new(), STEP_LABEL.to_owned()),
            held: Vec::new(),
            sources: HashMap::new(),
            seq: 0,
            selection: Selection::new(),
            press: None,
            hover: None,
            spool: Spool::new(output, &bead_counts()),
            guides: Vec::new(),
            guide_seen: HashMap::new(),
            hatch: None,
            flash: None,
            arranging: None,
            jobs: Vec::new(),
            reverting: false,
            amount: motor(0.0, SCRIM),
            pull: None,
            nudged: None,
            shake: None,
            band: None,
            whole: None,
            scene: None,
            minute: None,
        }
    }

    pub fn adopt(&mut self, widgets: Vec<Widget>, scenery: Scenery, stage: Option<Stage>) {
        for (index, widget) in widgets.into_iter().enumerate() {
            self.seq += 1;
            let Some(spot) = Spot::placed(self.seq, &widget, self.output, self.scale) else {
                self.held.push(index);
                continue;
            };
            let spot = spot.kept(self.output);
            self.sources.insert(spot.id, (spot, index, widget));
            self.widgets.push(Live::new(spot));
        }
        self.history = EditHistory::new(self.snapshot(), STEP_LABEL.to_owned());
        self.scenery = Some(scenery);
        self.stage = stage;
        let band = Rect::new(
            0.0,
            crate::domain::strand::zone_top(self.output[1]),
            self.output[0],
            crate::domain::strand::BAND,
        );
        let whole = Rect::new(0.0, 0.0, self.output[0], self.output[1]);
        if let Some(scenery) = &self.scenery {
            self.band = Some(chrome(scenery, self.stage.as_ref(), band, self.scale));
            self.whole = Some(chrome(scenery, self.stage.as_ref(), whole, self.scale));
        }
        self.spool.entered = Some(self.now);
        self.amount.aim(1.0);
        self.hold(TIMING.item_delay + TIMING.item + TIMING.stagger * item_count());
        self.paint();
        self.relayout();
    }

    pub fn ready(&self) -> bool {
        self.spool.entered.is_some()
    }

    pub fn set_moment(&mut self, moment: Moment) {
        if let Some(scenery) = self.scenery.as_mut() {
            scenery.moment = moment;
        }
        self.paint();
    }

    pub fn hold(&mut self, span: f32) {
        self.busy = self.busy.max(self.now + span);
    }

    pub fn later(&mut self, delay: f32, job: Job) {
        let at = self.now + delay;
        self.busy = self.busy.max(at);
        self.jobs.push((at, job));
    }

    pub fn snapshot(&self) -> Vec<Spot> {
        self.widgets.iter().filter(|live| live.dying.is_none()).map(Live::spot).collect()
    }

    pub fn find(&self, id: u32) -> Option<&Live> {
        self.widgets.iter().find(|live| live.id == id && live.dying.is_none())
    }

    pub fn find_mut(&mut self, id: u32) -> Option<&mut Live> {
        self.widgets.iter_mut().find(|live| live.id == id && live.dying.is_none())
    }

    pub fn selected(&self) -> Option<&Live> {
        self.selection.id.and_then(|id| self.find(id))
    }

    pub fn placed(&self, face: Face) -> bool {
        self.widgets.iter().any(|live| live.dying.is_none() && live.face == face)
            || self.pull.as_ref().is_some_and(|pull| pull.live.face == face)
    }

    pub fn full(&self) -> bool {
        self.snapshot().len() + self.held.len() >= self.room
    }

    pub fn kick(&mut self, face: Face, velocity: f32, group: bool) {
        self.spool.kick(bead_of(face), velocity, group);
    }

    pub fn cancel_count(&self) -> usize {
        self.history.index()
    }

    pub fn items(&self) -> Vec<Saved> {
        let mut items: Vec<(Option<usize>, Saved)> = self
            .snapshot()
            .into_iter()
            .map(|spot| match self.sources.get(&spot.id) {
                Some((original, index, _)) if *original == spot => {
                    (Some(*index), Saved::Kept(*index))
                }
                Some((_, index, widget)) => (
                    Some(*index),
                    Saved::Placed(spot.widget(self.output, self.scale, widget.output.clone())),
                ),
                None => {
                    (None, Saved::Placed(spot.widget(self.output, self.scale, self.name.clone())))
                }
            })
            .collect();
        for &held in &self.held {
            let at = items
                .iter()
                .position(|(origin, _)| origin.is_none_or(|origin| origin > held))
                .unwrap_or(items.len());
            items.insert(at, (Some(held), Saved::Kept(held)));
        }
        items.into_iter().map(|(_, item)| item).collect()
    }

    pub fn finished(&self) -> bool {
        self.spool.leaving && self.amount.value() <= SETTLED && self.amount.still()
    }

    pub fn animating(&self) -> bool {
        self.now < self.busy || self.restless()
    }

    fn restless(&self) -> bool {
        let lives = self.widgets.iter().chain(self.pull.as_ref().map(|pull| &pull.live));
        let mut motors: Vec<&Motor> = lives.flat_map(Live::motors).collect();
        motors.extend(self.spool.motors());
        motors.push(&self.amount);
        motors.extend(self.selection.motors());
        motors.extend(self.pull.iter().flat_map(|pull| &pull.whip));
        motors.iter().any(|motor| !motor.still())
    }

    pub fn tick(&mut self, dt: f32) {
        self.now += dt;
        self.amount.tick(dt);
        self.run_jobs();
        let now = self.now;
        self.spool.tick(now, dt);
        let resizing = match &self.press {
            Some(Press::Gem { id, grip, .. }) => Some((*id, *grip)),
            _ => None,
        };
        for live in &mut self.widgets {
            live.tick(dt);
            if let Some((_, grip)) = resizing.filter(|(id, _)| *id == live.id) {
                let centre = grip.centre(live.face, live.h.value());
                live.x.jump(centre[0]);
                live.y.jump(centre[1]);
            }
            live.expire(now);
        }
        self.whip(dt);
        let gone: Vec<u32> =
            self.widgets.iter().filter(|live| live.gone(now)).map(|live| live.id).collect();
        self.widgets.retain(|live| !live.gone(now));
        for id in gone {
            self.rasters.remove(&id);
        }
        self.hatch = self.hatch.filter(|at| now <= at + TIMING.hatch);
        self.selection.expire(now);
        self.shake = self.shake.filter(|(at, _)| now <= at + TIMING.shake);
        self.arranging = self.arranging.filter(|at| now <= at + TIMING.busy);
        self.flash = self.flash.filter(|(_, at)| now <= at + TIMING.readout);
        self.outgoing
            .retain(|id, _| self.widgets.iter().any(|live| live.id == *id && live.out.is_some()));
        self.follow_frame(dt);
        self.fill_beads();
        self.paint();
        self.relayout();
    }

    fn whip(&mut self, dt: f32) {
        let Some(pull) = self.pull.as_ref() else { return };
        let from = self.slot(pull.live.face);
        let rect = super::layout::view(self, &pull.live).rect;
        let to = [rect.centre()[0], rect.bottom() + super::tokens::THREAD.tip_gap];
        let now = self.now;
        let Some(pull) = self.pull.as_mut() else { return };
        pull.live.tick(dt);
        pull.live.expire(now);
        let target = [
            f32::midpoint(from[0], to[0]),
            f32::midpoint(from[1], to[1]) + super::tokens::THREAD.whip,
        ];
        for (whip, goal) in pull.whip.iter_mut().zip(target) {
            whip.aim(goal);
            whip.tick(dt);
        }
    }

    fn follow_frame(&mut self, dt: f32) {
        let bounds = self.selected().map(|live| super::layout::look_box(self, live));
        self.selection.follow(bounds, dt);
    }

    fn fill_beads(&mut self) {
        let placed: Vec<Vec<bool>> = KINDS
            .iter()
            .map(|kind| kind.styles().iter().map(|face| self.placed(*face)).collect())
            .collect();
        self.spool.fill(&placed);
    }

    fn run_jobs(&mut self) {
        let now = self.now;
        let (due, rest): (Vec<_>, Vec<_>) =
            std::mem::take(&mut self.jobs).into_iter().partition(|(at, _)| *at <= now);
        self.jobs = rest;
        for (_, job) in due {
            self.run(job);
        }
    }

    pub fn paint(&mut self) {
        let Some(scenery) = self.scenery.as_ref() else { return };
        let moment = scenery.moment;
        for live in self.widgets.iter_mut().chain(self.pull.as_mut().map(|pull| &mut pull.live)) {
            if !live.h.still() && self.rasters.contains_key(&live.id) {
                continue;
            }
            let bounds = live.bounds();
            let rounded = [bounds.x, bounds.y, bounds.h].map(|value| value.round() as i32);
            let resting = live.x.still() && live.y.still();
            if live.tones.is_none() || (resting && live.toned != Some(rounded)) {
                live.tones = Some(tones(scenery, self.stage.as_ref(), bounds, self.scale));
                live.chrome = Some(chrome(scenery, self.stage.as_ref(), bounds, self.scale));
                live.toned = Some(rounded);
            }
            let key = Key {
                face: live.face,
                height: (live.h.target() * self.scale).round() as u32,
                moment,
                tones: live.tones.expect("widget tones"),
            };
            if self.rasters.get(&live.id).is_none_or(|proxy| proxy.key != key) {
                self.rasters.insert(live.id, Proxy::paint(key, scenery));
            }
        }
        self.cut();
    }

    fn cut(&mut self) {
        let Some(stage) = self.stage.as_ref().filter(|stage| stage.matte.is_some()) else {
            return;
        };
        let lives = self.widgets.iter().chain(self.pull.as_ref().map(|pull| &pull.live));
        let behind = lives.filter(|live| {
            live.depth == Depth::Behind
                || live.wipe.as_ref().is_some_and(|wipe| wipe.from == Depth::Behind)
        });
        let areas: Vec<(u32, Rect)> =
            behind.map(|live| (live.id, super::layout::view(self, live).rect)).collect();
        for (id, area) in areas {
            if let Some(proxy) = self.rasters.get_mut(&id) {
                proxy.cut(stage, area, self.scale);
            }
        }
    }
}

pub fn whip(at: [f32; 2]) -> [Motor; 2] {
    at.map(|value| motor(value, WHIP))
}

pub fn lifted(live: &Live) -> f32 {
    ((live.lift.value() - 1.0) / (LOOK.lift - 1.0)).clamp(0.0, 1.0)
}
