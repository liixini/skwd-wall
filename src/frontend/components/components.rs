use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[cfg(test)]
pub use crate::contracts::display::set_surface_scale;
pub use crate::contracts::display::surface_scale;
use iced::mouse::ScrollDelta;
use iced::widget::canvas::Path;
use iced::{Color, Point};

static LAST_NOTCH_MS: AtomicU64 = AtomicU64::new(0);
const NOTCH_ECHO_MS: u64 = 100;

fn whole_notches(val: f32, sf: f32) -> f32 {
    if val == 0.0 {
        return 0.0;
    }
    let notches = (val / sf).round();
    if notches == 0.0 { val.signum() } else { notches }
}

fn wheel_now_ms() -> u64 {
    static T0: OnceLock<Instant> = OnceLock::new();
    T0.get_or_init(Instant::now).elapsed().as_millis() as u64 + 1
}

pub fn norm_wheel(delta: &ScrollDelta) -> Option<ScrollDelta> {
    norm_wheel_tracked(delta, wheel_now_ms(), &LAST_NOTCH_MS)
}

#[cfg(test)]
pub fn wheel_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let guard = LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    LAST_NOTCH_MS.store(0, Ordering::Relaxed);
    set_surface_scale(0.0);
    guard
}

pub(super) fn norm_wheel_tracked(
    delta: &ScrollDelta,
    now_ms: u64,
    last_notch: &AtomicU64,
) -> Option<ScrollDelta> {
    let sf = surface_scale();
    match delta {
        ScrollDelta::Lines { x, y } => {
            last_notch.store(now_ms, Ordering::Relaxed);
            Some(ScrollDelta::Lines { x: whole_notches(*x, sf), y: whole_notches(*y, sf) })
        }
        ScrollDelta::Pixels { x, y } => {
            let last = last_notch.load(Ordering::Relaxed);
            if last != 0 && now_ms.saturating_sub(last) < NOTCH_ECHO_MS {
                return None;
            }
            Some(ScrollDelta::Pixels { x: *x / sf, y: *y / sf })
        }
    }
}

pub fn with_alpha(col: Color, a: f32) -> Color {
    Color { a, ..col }
}

pub fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
}

pub fn parallelogram(x: f32, y: f32, w: f32, h: f32, skew: f32) -> Path {
    Path::new(|builder| {
        builder.move_to(Point::new(x + skew, y));
        builder.line_to(Point::new(x + w, y));
        builder.line_to(Point::new(x + w - skew, y + h));
        builder.line_to(Point::new(x, y + h));
        builder.close();
    })
}

pub fn chamfered(rect: iced::Rectangle, cuts: [[f32; 2]; 4]) -> Path {
    let [top_left, top_right, bottom_right, bottom_left] = cuts;
    let (left, top) = (rect.x, rect.y);
    let (right, bottom) = (rect.x + rect.width, rect.y + rect.height);
    let points = [
        Point::new(left + top_left[0], top),
        Point::new(right - top_right[0], top),
        Point::new(right, top + top_right[1]),
        Point::new(right, bottom - bottom_right[1]),
        Point::new(right - bottom_right[0], bottom),
        Point::new(left + bottom_left[0], bottom),
        Point::new(left, bottom - bottom_left[1]),
        Point::new(left, top + top_left[1]),
    ];
    Path::new(|builder| {
        builder.move_to(points[0]);
        for point in &points[1..] {
            builder.line_to(*point);
        }
        builder.close();
    })
}

pub fn cut_rect(x: f32, y: f32, w: f32, h: f32, cut: f32) -> Path {
    Path::new(|builder| {
        builder.move_to(Point::new(x + cut, y));
        builder.line_to(Point::new(x + w, y));
        builder.line_to(Point::new(x + w, y + h - cut));
        builder.line_to(Point::new(x + w - cut, y + h));
        builder.line_to(Point::new(x, y + h));
        builder.line_to(Point::new(x, y + cut));
        builder.close();
    })
}
