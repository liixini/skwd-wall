#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use super::Bezier;

pub const CALM: Bezier = Bezier(0.45, 0.0, 0.55, 1.0);
pub const CSS_EASE: Bezier = Bezier(0.25, 0.1, 0.25, 1.0);

#[derive(Clone, Copy)]
pub struct Timing {
    pub delay: Duration,
    pub duration: Duration,
    pub curve: Bezier,
}

impl Timing {
    pub const fn new(duration: Duration, curve: Bezier) -> Self {
        Self { delay: Duration::ZERO, duration, curve }
    }

    pub const fn after(self, delay: Duration) -> Self {
        Self { delay, ..self }
    }
}

#[derive(Clone, Copy)]
pub struct Transition {
    pub x: f32,
    from: f32,
    target: f32,
    run: Option<(Instant, Timing)>,
}

impl Transition {
    pub fn new(x: f32) -> Self {
        Self { x, from: x, target: x, run: None }
    }

    pub fn flag(on: bool) -> Self {
        Self::new(f32::from(u8::from(on)))
    }

    pub fn moving(&self) -> bool {
        self.run.is_some()
    }

    pub fn toward(&mut self, target: f32, now: Instant, timing: Timing) {
        if target == self.target {
            return;
        }
        self.from = self.x;
        self.target = target;
        self.run = Some((now, timing));
    }

    pub fn toward_flag(&mut self, on: bool, now: Instant, timing: Timing) {
        self.toward(f32::from(u8::from(on)), now, timing);
    }

    pub fn tick(&mut self, now: Instant) -> bool {
        let Some((start, timing)) = self.run else {
            return false;
        };
        let elapsed = now.saturating_duration_since(start).saturating_sub(timing.delay);
        let progress = if timing.duration.is_zero() {
            1.0
        } else {
            (elapsed.as_secs_f32() / timing.duration.as_secs_f32()).min(1.0)
        };
        if progress >= 1.0 {
            *self = Self::new(self.target);
            return false;
        }
        self.x = self.from + (self.target - self.from) * timing.curve.at(progress);
        true
    }
}
