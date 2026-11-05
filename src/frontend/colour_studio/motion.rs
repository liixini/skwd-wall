use iced::Color;

use super::tokens::{LIFT, POP_FROM, SLIDE_IN, SWAP_DELAY, SWAP_X, SWAP_Y, TURN_FROM};
use crate::frontend::animation::Bezier;
use crate::frontend::components::lerp_color;

pub const FAST: f32 = 0.18;
pub const BASE: f32 = 0.32;
pub const SLOW: f32 = 0.55;
pub const WHEEL_MOVE: f32 = 0.55;
pub const WHEEL_TINT: f32 = 0.4;

pub use crate::frontend::animation::{EASE, eased, linear};
pub const SNAP: Bezier = Bezier(0.2, 0.9, 0.25, 1.0);
pub const CSS_EASE: Bezier = Bezier(0.25, 0.1, 0.25, 1.0);

pub use crate::frontend::components::Look;

impl Look {
    pub fn lift(progress: f32) -> Self {
        Self { alpha: progress, dy: LIFT * (1.0 - progress), ..Self::REST }
    }

    pub fn slide_in(progress: f32) -> Self {
        Self { alpha: progress, dx: -SLIDE_IN * (1.0 - progress), ..Self::REST }
    }

    pub fn fade(progress: f32) -> Self {
        Self { alpha: progress, ..Self::REST }
    }

    pub fn pop(progress: f32) -> Self {
        Self { alpha: progress, scale: POP_FROM + (1.0 - POP_FROM) * progress, ..Self::REST }
    }

    pub fn turn(progress: f32) -> Self {
        Self { alpha: progress, turn: TURN_FROM * (1.0 - progress), ..Self::REST }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glide {
    from: f32,
    to: f32,
    start: f32,
    duration: f32,
    curve: Bezier,
}

impl Glide {
    pub fn new(at: f32, duration: f32) -> Self {
        Self { from: at, to: at, start: f32::NEG_INFINITY, duration, curve: EASE }
    }

    pub fn with_curve(self, curve: Bezier) -> Self {
        Self { curve, ..self }
    }

    pub fn value(&self, now: f32) -> f32 {
        self.from + (self.to - self.from) * self.curve.at(linear(now, self.start, self.duration))
    }

    pub fn target(&self) -> f32 {
        self.to
    }

    pub fn toward(&mut self, to: f32, now: f32) -> f32 {
        if (to - self.to).abs() < f32::EPSILON {
            return self.start + self.duration;
        }
        self.from = self.value(now);
        self.to = to;
        self.start = now;
        now + self.duration
    }

    pub fn jump(&mut self, to: f32) {
        self.from = to;
        self.to = to;
        self.start = f32::NEG_INFINITY;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    from: Color,
    to: Color,
    start: f32,
    duration: f32,
}

impl Tint {
    pub fn new(color: Color, duration: f32) -> Self {
        Self { from: color, to: color, start: f32::NEG_INFINITY, duration }
    }

    pub fn value(&self, now: f32) -> Color {
        lerp_color(self.from, self.to, EASE.at(linear(now, self.start, self.duration)))
    }

    pub fn toward(&mut self, to: Color, now: f32) -> f32 {
        if to == self.to {
            return self.start + self.duration;
        }
        self.from = self.value(now);
        self.to = to;
        self.start = now;
        now + self.duration
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Swap {
    pub now: String,
    pub was: String,
    start: f32,
    dir: f32,
    axis: Axis,
    next: Option<(String, f32)>,
}

pub struct Leg {
    pub alpha: f32,
    pub dx: f32,
    pub dy: f32,
}

impl Swap {
    pub fn new(axis: Axis) -> Self {
        Self {
            now: String::new(),
            was: String::new(),
            start: f32::NEG_INFINITY,
            dir: 1.0,
            axis,
            next: None,
        }
    }

    pub fn set(&mut self, text: String, dir: f32, now: f32, quiet: bool) -> f32 {
        if quiet || self.now.is_empty() {
            self.now = text;
            self.was.clear();
            self.next = None;
            self.start = f32::NEG_INFINITY;
            return f32::NEG_INFINITY;
        }
        if now < self.end() {
            self.next = (text != self.now).then_some((text, dir));
            return self.end();
        }
        if text == self.now {
            return f32::NEG_INFINITY;
        }
        self.begin(text, dir, now)
    }

    pub fn settle(&mut self, now: f32) -> f32 {
        if now < self.end() {
            return f32::NEG_INFINITY;
        }
        match self.next.take() {
            Some((text, dir)) => self.begin(text, dir, now),
            None => f32::NEG_INFINITY,
        }
    }

    fn end(&self) -> f32 {
        self.start + SWAP_DELAY + BASE
    }

    fn begin(&mut self, text: String, dir: f32, now: f32) -> f32 {
        self.was = std::mem::replace(&mut self.now, text);
        self.dir = dir.signum();
        self.start = now;
        self.end()
    }

    fn shift(&self, amount: f32) -> (f32, f32) {
        let span = match self.axis {
            Axis::X => SWAP_X,
            Axis::Y => SWAP_Y,
        } * amount
            * self.dir;
        match self.axis {
            Axis::X => (span, 0.0),
            Axis::Y => (0.0, span),
        }
    }

    pub fn outgoing(&self, now: f32) -> Option<Leg> {
        let progress = eased(now, self.start, FAST);
        if self.was.is_empty() || progress >= 1.0 {
            return None;
        }
        let (dx, dy) = self.shift(-progress);
        Some(Leg { alpha: 1.0 - progress, dx, dy })
    }

    pub fn incoming(&self, now: f32) -> Leg {
        let progress = eased(now, self.start + SWAP_DELAY, BASE);
        let (dx, dy) = self.shift(1.0 - progress);
        Leg { alpha: progress, dx, dy }
    }
}

#[cfg(test)]
mod tests;

pub struct Crossfade<K, T> {
    key: Option<K>,
    shown: Option<T>,
    previous: Option<T>,
    start: f32,
    queued: Option<(K, T)>,
}

impl<K, T> Default for Crossfade<K, T> {
    fn default() -> Self {
        Self { key: None, shown: None, previous: None, start: f32::NEG_INFINITY, queued: None }
    }
}

impl<K: PartialEq, T> Crossfade<K, T> {
    pub fn shown(&self) -> Option<&T> {
        self.shown.as_ref()
    }

    pub fn previous(&self, now: f32) -> Option<(&T, f32)> {
        let fade = 1.0 - eased(now, self.start, SLOW);
        self.previous.as_ref().filter(|_| fade > 0.0).map(|old| (old, fade))
    }

    pub fn show(&mut self, key: K, item: T, now: f32) -> f32 {
        if self.key.as_ref() == Some(&key) {
            self.queued = None;
            return f32::NEG_INFINITY;
        }
        if self.previous.is_some() {
            self.queued = Some((key, item));
            return self.start + SLOW;
        }
        self.begin(key, item, now)
    }

    pub fn settle(&mut self, now: f32) -> f32 {
        if now < self.start + SLOW {
            return f32::NEG_INFINITY;
        }
        self.previous = None;
        match self.queued.take() {
            Some((key, item)) => self.begin(key, item, now),
            None => f32::NEG_INFINITY,
        }
    }

    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.shown
            .iter_mut()
            .chain(self.previous.iter_mut())
            .chain(self.queued.iter_mut().map(|(_, item)| item))
    }

    fn begin(&mut self, key: K, item: T, now: f32) -> f32 {
        self.key = Some(key);
        let Some(old) = self.shown.replace(item) else {
            return f32::NEG_INFINITY;
        };
        self.previous = Some(old);
        self.start = now;
        now + SLOW
    }
}
