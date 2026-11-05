use super::tokens::Feel;
use crate::frontend::animation::Spring;

const BACK_CUBE: f32 = 2.9;
const BACK_SQUARE: f32 = 1.9;

pub fn back(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    let rest = t - 1.0;
    1.0 + BACK_CUBE * rest * rest * rest + BACK_SQUARE * rest * rest
}

#[derive(Debug, Clone, Copy)]
pub struct Motor {
    spring: Spring,
    precision: f32,
}

impl Motor {
    pub fn new(value: f32, feel: Feel) -> Self {
        let spring = Spring::new(value, feel.omega * feel.omega, 2.0 * feel.zeta * feel.omega);
        Self { spring, precision: feel.precision }
    }

    pub fn value(&self) -> f32 {
        self.spring.x
    }

    pub fn target(&self) -> f32 {
        self.spring.target
    }

    pub fn aim(&mut self, target: f32) {
        self.spring.retarget(target);
    }

    pub fn jump(&mut self, value: f32) {
        self.spring.snap(value);
    }

    pub fn kick(&mut self, velocity: f32) {
        self.spring.v += velocity;
    }

    pub fn zeta(&mut self, zeta: f32) {
        self.spring.set_zeta(zeta);
    }

    pub fn still(&self) -> bool {
        self.spring.settled_with_precision(self.precision)
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if self.still() {
            self.spring.snap(self.spring.target);
            return false;
        }
        self.spring.tick_with_precision(dt, self.precision);
        true
    }
}
