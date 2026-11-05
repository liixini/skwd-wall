#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn centred(centre: [f32; 2], size: [f32; 2]) -> Self {
        Self::new(centre[0] - size[0] / 2.0, centre[1] - size[1] / 2.0, size[0], size[1])
    }

    pub fn right(self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(self) -> f32 {
        self.y + self.h
    }

    pub fn centre(self) -> [f32; 2] {
        [self.x + self.w / 2.0, self.y + self.h / 2.0]
    }

    pub fn padded(self, pad: f32) -> Self {
        Self::new(self.x - pad, self.y - pad, self.w + 2.0 * pad, self.h + 2.0 * pad)
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.x < other.right()
            && other.x < self.right()
            && self.y < other.bottom()
            && other.y < self.bottom()
    }

    pub fn contains(self, at: [f32; 2]) -> bool {
        (self.x..=self.right()).contains(&at[0]) && (self.y..=self.bottom()).contains(&at[1])
    }

    pub fn area(self) -> f32 {
        self.w * self.h
    }
}

pub fn fit(centre: f32, half: f32, size: f32) -> f32 {
    if 2.0 * half >= size { size / 2.0 } else { centre.clamp(half, size - half) }
}
