use std::borrow::Cow;
use std::f32::consts::PI;

use paper_widgets::Tones;

use super::motion::Motor;
use super::tokens::{FRAME, FRAMING, PLATE, TIMING};
use crate::domain::strand::{Rect, zone_top};
use crate::frontend::animation::{eased, linear};

pub struct Selection<Id> {
    pub id: Option<Id>,
    pub at: f32,
    pub frame: Option<[Motor; 4]>,
    pub out: Option<(Rect, Tones, f32)>,
    pub bar: f32,
    pub fold: Option<f32>,
}

#[derive(Clone)]
pub struct BarPlate<T> {
    pub tool: T,
    pub label: Cow<'static, str>,
    pub rect: Rect,
    pub on: bool,
    pub dim: bool,
    pub rank: usize,
}

#[derive(Clone)]
pub struct Bar<T> {
    pub plates: Vec<BarPlate<T>>,
    pub centre: f32,
    pub y: f32,
    pub below: bool,
    pub total: f32,
}

impl<Id: Copy + PartialEq> Selection<Id> {
    pub fn new() -> Self {
        Self { id: None, at: 0.0, frame: None, out: None, bar: 0.0, fold: None }
    }

    pub fn pick(&mut self, id: Id, at: f32) {
        self.id = Some(id);
        self.at = at;
        self.frame = None;
        self.bar = at + TIMING.bar_lead;
        self.fold = None;
    }

    pub fn clear(&mut self) {
        self.id = None;
        self.frame = None;
    }

    pub fn frame_rect(&self) -> Option<Rect> {
        let [x, y, w, h] = self.frame.as_ref()?.each_ref().map(Motor::value);
        Some(Rect::new(x, y, w, h))
    }

    pub fn retire(&mut self, tones: Option<Tones>, now: f32) -> bool {
        let (Some(frame), Some(tones)) = (self.frame_rect(), tones) else { return false };
        self.out = Some((frame, tones, now));
        true
    }

    pub fn follow(&mut self, bounds: Option<Rect>, dt: f32) {
        let Some(bounds) = bounds else {
            self.frame = None;
            return;
        };
        let target = bounds.padded(FRAME.pad);
        let values = [target.x, target.y, target.w, target.h];
        let frame =
            self.frame.get_or_insert_with(|| values.map(|value| Motor::new(value, FRAMING)));
        for (motor, value) in frame.iter_mut().zip(values) {
            motor.aim(value);
            motor.tick(dt);
        }
    }

    pub fn expire(&mut self, now: f32) {
        self.out = self.out.filter(|(_, _, at)| now <= at + TIMING.deselect);
    }

    pub fn fold_now(&mut self, now: f32) {
        self.fold = Some(now);
    }

    pub fn unfurl(&mut self, now: f32) {
        self.bar = now;
        self.fold = None;
    }

    pub fn folding(&self, now: f32) -> f32 {
        self.fold.map_or(1.0, |at| 1.0 - eased(now, at, TIMING.fold))
    }

    pub fn opening<T>(&self, now: f32, plate: &BarPlate<T>) -> f32 {
        let start = self.bar + TIMING.bar_delay + plate.rank as f32 * TIMING.stagger;
        eased(now, start, TIMING.unfurl) * self.folding(now)
    }

    pub fn motors(&self) -> impl Iterator<Item = &Motor> {
        self.frame.iter().flatten()
    }
}

impl<T> Bar<T> {
    pub fn place(mut plates: Vec<BarPlate<T>>, frame: Rect, output: [f32; 2]) -> Self {
        let total = plates.last().map_or(0.0, |plate| plate.rect.right());
        let [screen, high] = output;
        let centre = (frame.x + frame.w / 2.0)
            .clamp(total / 2.0 + FRAME.edge, screen - total / 2.0 - FRAME.edge);
        let mut y = frame.y - PLATE.lift - PLATE.height;
        let mut below = false;
        if y < FRAME.edge {
            y = frame.bottom() + PLATE.lift;
            below = true;
            if y + PLATE.height > zone_top(high) - FRAME.edge {
                y = (frame.y + PLATE.lift).clamp(FRAME.edge, high - FRAME.edge - PLATE.height);
            }
        }
        for plate in &mut plates {
            plate.rect.x += centre - total / 2.0;
            plate.rect.y = y;
        }
        let mut order: Vec<(usize, f32)> = plates
            .iter()
            .enumerate()
            .map(|(index, plate)| (index, (plate.rect.centre()[0] - centre).abs()))
            .collect();
        order.sort_by(|one, other| one.1.total_cmp(&other.1));
        for (rank, (index, _)) in order.into_iter().enumerate() {
            plates[index].rank = rank;
        }
        Self { plates, centre, y, below, total }
    }
}

impl<T> Bar<T> {
    pub fn at(mut self, y: f32, below: bool) -> Self {
        for plate in &mut self.plates {
            plate.rect.y = y;
        }
        self.y = y;
        self.below = below;
        self
    }

    pub fn overlaps(&self, others: &[Rect]) -> bool {
        self.plates.iter().any(|plate| others.iter().any(|other| plate.rect.overlaps(*other)))
    }
}

pub fn shake(now: f32, at: f32) -> f32 {
    let share = linear(now, at, TIMING.shake);
    (share * PI * FRAME.shake_turns).sin() * FRAME.shake * (1.0 - share)
}
