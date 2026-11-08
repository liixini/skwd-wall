use std::ops::RangeInclusive;

use crate::frontend::animation::Spring;

#[derive(Clone, Copy)]
pub enum Ends {
    Stop,
    Wrap,
    Loop,
}

impl Ends {
    pub fn wrap_if(wrap: bool) -> Self {
        if wrap { Self::Wrap } else { Self::Stop }
    }
}

pub fn step(idx: usize, delta: i32, len: usize, ends: Ends) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let last = len as i64 - 1;
    let target = idx as i64 + i64::from(delta);
    let next = match ends {
        Ends::Loop => target.rem_euclid(len as i64),
        Ends::Wrap if target < 0 && idx == 0 => last,
        Ends::Wrap if target > last && idx as i64 == last => 0,
        _ => target.clamp(0, last),
    };
    (next != idx as i64).then_some(next as usize)
}

pub fn grid_step(idx: usize, dy: i32, cols: usize, len: usize, ends: Ends) -> Option<usize> {
    let cols = cols.max(1);
    let col = idx % cols;
    step(idx / cols, dy, len.saturating_sub(col).div_ceil(cols), ends).map(|row| row * cols + col)
}

pub fn follow(
    spring: &mut Spring,
    item_top: f32,
    item_h: f32,
    view_h: f32,
    bounds: RangeInclusive<f32>,
) -> bool {
    let (lo, hi) = bounds.into_inner();
    if item_top < spring.target {
        spring.retarget(item_top.clamp(lo, hi));
        true
    } else if item_top + item_h > spring.target + view_h {
        spring.retarget((item_top + item_h - view_h).clamp(lo, hi));
        true
    } else {
        false
    }
}

pub fn land(spring: &mut Spring, target: f32, reach: f32) {
    let gap = target - spring.x;
    if gap.abs() > reach {
        spring.snap(target - reach.copysign(gap));
    }
    spring.retarget(target);
}

pub fn lap(pos: f32, around: f32, period: f32) -> f32 {
    ((around - pos) / period).round()
}

pub fn rebase(spring: &mut Spring, period: f32) {
    let shift = (spring.x / period).floor() * period;
    spring.x -= shift;
    spring.target -= shift;
}
