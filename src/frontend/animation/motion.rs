use super::{Spring, Tween, approach, approach_k};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionTier {
    Fast,
    Standard,
    Slow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionProfile {
    fast: f32,
    standard: f32,
    slow: f32,
}

impl Default for MotionProfile {
    fn default() -> Self {
        Self::new(180.0, 250.0, 450.0)
    }
}

impl MotionProfile {
    pub fn new(fast_ms: f32, standard_ms: f32, slow_ms: f32) -> Self {
        Self {
            fast: fast_ms.clamp(35.0, 2_000.0),
            standard: standard_ms.clamp(35.0, 2_000.0),
            slow: slow_ms.clamp(35.0, 4_000.0),
        }
    }

    pub fn duration_ms(self, tier: MotionTier) -> f32 {
        match tier {
            MotionTier::Fast => self.fast,
            MotionTier::Standard => self.standard,
            MotionTier::Slow => self.slow,
        }
    }

    pub fn duration_seconds(self, tier: MotionTier) -> f32 {
        self.duration_ms(tier) / 1_000.0
    }

    pub fn spring(self, x: f32, tier: MotionTier) -> Spring {
        Spring::for_duration_ms(x, self.duration_ms(tier))
    }

    pub fn tween(self, x: f32, tier: MotionTier) -> Tween {
        Tween::for_duration_ms(x, self.duration_ms(tier))
    }

    pub fn override_spring(self, x: f32, milliseconds: f32) -> Spring {
        Spring::for_duration_ms(x, milliseconds.max(self.minimum_duration_ms()))
    }

    pub fn override_tween(self, x: f32, milliseconds: f32) -> Tween {
        Tween::for_duration_ms(x, milliseconds.max(self.minimum_duration_ms()))
    }

    pub fn retime_spring(self, spring: &mut Spring, tier: MotionTier) {
        self.retime_spring_override(spring, self.duration_ms(tier));
    }

    pub fn retime_spring_override(self, spring: &mut Spring, milliseconds: f32) {
        let mut retimed = self.override_spring(spring.x, milliseconds);
        retimed.retarget(spring.target);
        *spring = retimed;
    }

    pub fn retime_tween(self, tween: &mut Tween, tier: MotionTier) {
        self.retime_tween_override(tween, self.duration_ms(tier));
    }

    pub fn retime_tween_override(self, tween: &mut Tween, milliseconds: f32) {
        let target = tween.target;
        *tween = self.override_tween(tween.x, milliseconds);
        tween.retarget(target);
    }

    pub fn approach_tau(self, tier: MotionTier) -> f32 {
        self.duration_seconds(tier) / 4.0
    }

    pub fn approach(self, current: f32, target: f32, dt: f32, tier: MotionTier) -> f32 {
        approach(current, target, dt, self.approach_tau(tier))
    }

    pub fn approach_k(self, dt: f32, tier: MotionTier) -> f32 {
        approach_k(dt, self.approach_tau(tier))
    }

    fn minimum_duration_ms(self) -> f32 {
        self.fast.min(self.standard).min(self.slow).min(35.0)
    }
}
