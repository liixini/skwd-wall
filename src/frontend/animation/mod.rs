#[allow(clippy::module_inception)]
mod animation;
mod motion;
mod tests;
mod transition;

pub use animation::{
    Bezier, EASE, Spring, Timeline, Tween, approach, approach_k, ease_out_cubic, eased, lerp,
    linear, smoothstep, window,
};
pub use motion::{MotionProfile, MotionTier};
pub use transition::{CALM, CSS_EASE, Timing, Transition};
