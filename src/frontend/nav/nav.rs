use crate::frontend::animation::Spring;

pub fn step(idx: usize, delta: i32, len: usize, wrap: bool) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let last = len as i32 - 1;
    let target = idx as i32 + delta;
    let next = if wrap && target < 0 && idx == 0 {
        last
    } else if wrap && target > last && idx as i32 == last {
        0
    } else {
        target.clamp(0, last)
    };
    (next != idx as i32).then_some(next as usize)
}

pub fn grid_step(idx: usize, dy: i32, cols: usize, len: usize, wrap: bool) -> Option<usize> {
    let cols = cols.max(1);
    let col = idx % cols;
    step(idx / cols, dy, len.saturating_sub(col).div_ceil(cols), wrap).map(|row| row * cols + col)
}

pub fn follow(spring: &mut Spring, item_top: f32, item_h: f32, view_h: f32, max: f32) -> bool {
    if item_top < spring.target {
        spring.retarget(item_top.max(0.0));
        true
    } else if item_top + item_h > spring.target + view_h {
        spring.retarget((item_top + item_h - view_h).clamp(0.0, max));
        true
    } else {
        false
    }
}
