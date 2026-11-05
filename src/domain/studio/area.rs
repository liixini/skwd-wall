use skwd_palette::studio::{Area, Patch, StudioPalette, area_palette, area_patches, area_wheel};

pub const PREVIEW_ASPECT: f32 = 1.6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl Frame {
    pub fn cover(aspect: f32) -> Self {
        Self::covering(PREVIEW_ASPECT, aspect)
    }

    pub fn covering(box_aspect: f32, aspect: f32) -> Self {
        let width = (aspect / box_aspect).max(1.0);
        let height = (box_aspect / aspect).max(1.0);
        Self { left: (1.0 - width) / 2.0, top: (1.0 - height) / 2.0, width, height }
    }

    pub fn to_image(self, area: Area) -> Area {
        let x = |share: f32| ((share - self.left) / self.width).clamp(0.0, 1.0);
        let y = |share: f32| ((share - self.top) / self.height).clamp(0.0, 1.0);
        Area { left: x(area.left), top: y(area.top), right: x(area.right), bottom: y(area.bottom) }
    }

    pub fn to_preview(self, area: Area) -> Area {
        let x = |share: f32| self.left + share * self.width;
        let y = |share: f32| self.top + share * self.height;
        Area { left: x(area.left), top: y(area.top), right: x(area.right), bottom: y(area.bottom) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pixels {
    pub rgba: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub aspect: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Picked {
    pub area: Area,
    pub patches: Vec<Patch>,
    pub dark: StudioPalette,
    pub light: StudioPalette,
    pub wheel: Vec<f32>,
}

impl Picked {
    pub fn read(pixels: &Pixels, area: Area) -> Option<Self> {
        let patches = area_patches(&pixels.rgba, pixels.width, pixels.height, area);
        Some(Self {
            area,
            dark: area_palette(&patches, true)?,
            light: area_palette(&patches, false)?,
            wheel: area_wheel(&patches).to_vec(),
            patches,
        })
    }

    pub fn palette(&self, dark: bool) -> StudioPalette {
        if dark { self.dark } else { self.light }
    }
}
