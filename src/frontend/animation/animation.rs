pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn window(p: f32, start: f32, end: f32) -> f32 {
    smoothstep((p - start) / (end - start))
}

pub fn approach_k(dt: f32, tau: f32) -> f32 {
    1.0 - (-dt.min(0.05) / tau.max(1e-6)).exp()
}

pub fn approach(cur: f32, target: f32, dt: f32, tau: f32) -> f32 {
    cur + (target - cur) * approach_k(dt, tau)
}

#[derive(Debug, Clone, Copy)]
pub struct Approach {
    pub x: f32,
    pub target: f32,
    tau: f32,
}

impl Approach {
    pub fn new(x: f32, target: f32, tau: f32) -> Self {
        Self { x, target, tau }
    }

    pub fn toward(&mut self, target: f32) {
        self.target = target;
    }

    pub fn set_tau(&mut self, tau: f32) {
        self.tau = tau.max(1e-6);
    }

    #[allow(dead_code)]
    pub fn snap(&mut self, value: f32) {
        self.x = value;
        self.target = value;
    }

    pub fn restart(&mut self, from: f32) {
        self.x = from;
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.x = approach(self.x, self.target, dt, self.tau);
        if (self.x - self.target).abs() < 1e-3 {
            self.x = self.target;
        }
        !self.settled()
    }

    pub fn settled(&self) -> bool {
        self.x == self.target
    }

    pub fn ease(&self) -> f32 {
        smoothstep(self.x)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Timeline {
    tracks: Vec<(&'static str, Approach)>,
}

impl Timeline {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    pub fn with(mut self, name: &'static str, x0: f32, target: f32, tau: f32) -> Self {
        self.tracks.push((name, Approach::new(x0, target, tau)));
        self
    }

    fn track(&self, name: &str) -> Option<&Approach> {
        self.tracks.iter().find(|(key, _)| *key == name).map(|(_, anim)| anim)
    }

    fn track_mut(&mut self, name: &str) -> Option<&mut Approach> {
        self.tracks.iter_mut().find(|(key, _)| *key == name).map(|(_, anim)| anim)
    }

    pub fn get(&self, name: &str) -> f32 {
        self.track(name).map_or(0.0, |anim| anim.x)
    }

    pub fn ease(&self, name: &str) -> f32 {
        self.track(name).map_or(0.0, Approach::ease)
    }

    pub fn toward(&mut self, name: &str, target: f32) {
        if let Some(anim) = self.track_mut(name) {
            anim.toward(target);
        }
    }

    pub fn set_tau(&mut self, name: &str, tau: f32) {
        if let Some(anim) = self.track_mut(name) {
            anim.set_tau(tau);
        }
    }

    #[allow(dead_code)]
    pub fn snap(&mut self, name: &str, value: f32) {
        if let Some(anim) = self.track_mut(name) {
            anim.snap(value);
        }
    }

    pub fn restart(&mut self, name: &str, from: f32) {
        if let Some(anim) = self.track_mut(name) {
            anim.restart(from);
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for (_, anim) in &mut self.tracks {
            moving |= anim.tick(dt);
        }
        moving
    }

    pub fn settled(&self) -> bool {
        self.tracks.iter().all(|(_, anim)| anim.settled())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tween {
    pub x: f32,
    pub target: f32,
    rate: f32,
}

impl Tween {
    pub fn for_duration_ms(x: f32, ms: f32) -> Self {
        Self { x, target: x, rate: 1000.0 / ms.max(1.0) }
    }

    #[cfg(test)]
    pub fn duration_ms(&self) -> f32 {
        1000.0 / self.rate
    }

    pub fn retarget(&mut self, target: f32) {
        self.target = target;
    }

    pub fn snap(&mut self, value: f32) {
        self.x = value;
        self.target = value;
    }

    pub fn run(&mut self, from: f32, to: f32) {
        self.x = from;
        self.target = to;
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let step = self.rate * dt.min(0.05);
        if self.x < self.target {
            self.x = (self.x + step).min(self.target);
        } else if self.x > self.target {
            self.x = (self.x - step).max(self.target);
        }
        if (self.x - self.target).abs() < 1e-4 {
            self.x = self.target;
        }
        !self.settled()
    }

    pub fn settled(&self) -> bool {
        self.x == self.target
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Spring {
    pub x: f32,
    pub v: f32,
    pub target: f32,
    k: f32,
    c: f32,
}

impl Spring {
    pub fn new(x: f32, k: f32, c: f32) -> Self {
        Self { x, v: 0.0, target: x, k, c }
    }

    pub fn for_duration_ms(x: f32, ms: f32) -> Self {
        let omega = 6.64 / (ms / 1000.0);
        Self::new(x, omega * omega, 2.0 * omega)
    }

    #[cfg(test)]
    pub fn duration_ms(&self) -> f32 {
        6_640.0 / self.k.sqrt()
    }

    pub fn retarget(&mut self, target: f32) {
        self.target = target;
    }

    pub fn snap(&mut self, value: f32) {
        self.x = value;
        self.v = 0.0;
        self.target = value;
    }

    pub fn tick(&mut self, dt: f32) {
        self.tick_with_precision(dt, 0.05);
    }

    pub fn tick_with_precision(&mut self, dt: f32, precision: f32) {
        let mut remaining = dt.min(0.05);
        let hop = 1.0 / 240.0;
        while remaining > 0.0 {
            let step = remaining.min(hop);
            self.v += (-self.k * (self.x - self.target) - self.c * self.v) * step;
            self.x += self.v * step;
            remaining -= step;
        }
        if self.settled_with_precision(precision) {
            self.x = self.target;
            self.v = 0.0;
        }
    }

    pub fn settled(&self) -> bool {
        self.settled_with_precision(0.05)
    }

    pub fn settled_with_precision(&self, precision: f32) -> bool {
        (self.x - self.target).abs() < precision && self.v.abs() < precision * 10.0
    }

    pub fn value(&self) -> f32 {
        if self.settled() { self.target } else { self.x }
    }

    pub fn set_zeta(&mut self, zeta: f32) {
        self.c = 2.0 * self.k.sqrt() * zeta.max(0.05);
    }
}

const SOLVE_STEPS: usize = 8;
const SOLVE_EPSILON: f32 = 1e-5;
const SOLVE_HALVES: usize = 24;
const CUBIC: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bezier(pub f32, pub f32, pub f32, pub f32);

pub const EASE: Bezier = Bezier(0.2, 0.8, 0.2, 1.0);

pub fn linear(age: f32, delay: f32, duration: f32) -> f32 {
    ((age - delay) / duration).clamp(0.0, 1.0)
}

pub fn eased(age: f32, delay: f32, duration: f32) -> f32 {
    EASE.at(linear(age, delay, duration))
}

impl Bezier {
    fn axis(t: f32, first: f32, second: f32) -> f32 {
        let rest = 1.0 - t;
        CUBIC * rest * rest * t * first + CUBIC * rest * t * t * second + t * t * t
    }

    fn slope(t: f32, first: f32, second: f32) -> f32 {
        let rest = 1.0 - t;
        CUBIC * (rest * rest * first + 2.0 * rest * t * (second - first) + t * t * (1.0 - second))
    }

    pub fn at(self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        if x <= 0.0 || x >= 1.0 {
            return x;
        }
        let Self(x1, y1, x2, y2) = self;
        let mut t = x;
        for _ in 0..SOLVE_STEPS {
            let error = Self::axis(t, x1, x2) - x;
            if error.abs() < SOLVE_EPSILON {
                return Self::axis(t, y1, y2);
            }
            let slope = Self::slope(t, x1, x2);
            if slope.abs() < SOLVE_EPSILON {
                break;
            }
            t = (t - error / slope).clamp(0.0, 1.0);
        }
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..SOLVE_HALVES {
            t = f32::midpoint(low, high);
            if Self::axis(t, x1, x2) < x {
                low = t;
            } else {
                high = t;
            }
        }
        Self::axis(t, y1, y2)
    }
}
