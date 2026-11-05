use crate::frontend::animation::EASE;

const MAX_STEP: f32 = 0.05;
const MS_PER_SECOND: f32 = 1_000.0;

#[derive(Debug, Clone, Copy)]
pub(super) struct FlipClock {
    pub x: f32,
    pub target: f32,
    from: f32,
    age: f32,
    span: f32,
    total: f32,
}

impl FlipClock {
    pub(super) fn new(total_ms: f32) -> Self {
        Self {
            x: 0.0,
            target: 0.0,
            from: 0.0,
            age: 0.0,
            span: 0.0,
            total: total_ms / MS_PER_SECOND,
        }
    }

    pub(super) fn retime(&mut self, total_ms: f32) {
        self.total = total_ms / MS_PER_SECOND;
    }

    pub(super) fn retarget(&mut self, target: f32) {
        if target == self.target {
            return;
        }
        self.from = self.x;
        self.target = target;
        self.age = 0.0;
        self.span = self.total * (target - self.x).abs();
    }

    pub(super) fn snap(&mut self, value: f32) {
        self.x = value;
        self.from = value;
        self.target = value;
    }

    pub(super) fn tick(&mut self, dt: f32) {
        if self.settled() {
            return;
        }
        self.age += dt.min(MAX_STEP);
        if self.age >= self.span {
            self.x = self.target;
            return;
        }
        self.x = self.from + (self.target - self.from) * EASE.at(self.age / self.span);
    }

    pub(super) fn settled(&self) -> bool {
        self.x == self.target
    }
}
