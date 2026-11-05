#[cfg(test)]
mod tests;

use crate::frontend::animation::{Bezier, MotionProfile, MotionTier};

pub(crate) struct Choreography {
    pub open_ms: f32,
    pub surface_ms: f32,
    pub trace_ms: f32,
    pub content_ms: f32,
    pub emblem_ms: f32,
    pub emblem_stagger_ms: f32,
    pub nav_stagger_ms: f32,
    pub row_stagger_ms: f32,
    pub minimum_ms: f32,
    pub roll: Bezier,
    pub reveal: Bezier,
    pub stroke: Bezier,
}

pub(crate) const SETTINGS_MOTION: Choreography = Choreography {
    open_ms: 1_700.0,
    surface_ms: 220.0,
    trace_ms: 640.0,
    content_ms: 420.0,
    emblem_ms: 540.0,
    emblem_stagger_ms: 60.0,
    nav_stagger_ms: 28.0,
    row_stagger_ms: 40.0,
    minimum_ms: 35.0,
    roll: Bezier(0.4, 0.0, 0.2, 1.0),
    reveal: Bezier(0.2, 0.75, 0.2, 1.0),
    stroke: Bezier(0.65, 0.0, 0.35, 1.0),
};

impl Choreography {
    pub fn reduced(&self, motion: MotionProfile) -> bool {
        motion.duration_ms(MotionTier::Fast) <= self.minimum_ms
    }

    pub fn entrance(&self, progress: f32, index: usize, navigation: bool) -> f32 {
        if progress >= 1.0 {
            return 1.0;
        }
        let stagger = if navigation { self.nav_stagger_ms } else { self.row_stagger_ms };
        let delay = (index as f32 * stagger).min(self.open_ms - self.content_ms);
        let elapsed = progress * self.open_ms - delay;
        self.reveal.at((elapsed / self.content_ms).clamp(0.0, 1.0))
    }
}
