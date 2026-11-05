use iced::widget::image::Handle;
use paper_control::{Face, Labels, Palette};
use paper_widgets::{Canvas, Ink, Lettering, Moment, Scene, Tones, luminance};

use super::tokens::LOOK;
use crate::domain::strand::{Rect, frame};

const CHANNELS: usize = 4;
const ALPHA: usize = 3;
const OPAQUE: f32 = 255.0;
const DARK_GROUND: f32 = 0.22;
const MUTED_SHARE: f32 = 0.38;

pub struct Scenery {
    pub palette: Palette,
    pub labels: Labels,
    pub lettering: Lettering,
    pub moment: Moment,
}

pub struct Stage {
    pub size: [u32; 2],
    pub image: Handle,
    pub matte: Option<Vec<u8>>,
}

pub struct Hatch {
    pub image: Handle,
    pub bounds: [usize; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    pub face: Face,
    pub height: u32,
    pub moment: Moment,
    pub tones: Tones,
}

pub struct Proxy {
    pub key: Key,
    pub size: [usize; 2],
    pixels: Vec<[u8; 4]>,
    pub plain: Handle,
    pub cut: Option<([i32; 4], Handle)>,
}

impl Stage {
    pub fn new(size: [u32; 2], rgba: Vec<u8>, matte: Option<Vec<u8>>) -> Self {
        Self { size, image: Handle::from_rgba(size[0], size[1], rgba), matte }
    }

    fn rgba(&self) -> &[u8] {
        match &self.image {
            Handle::Rgba { pixels, .. } => pixels,
            Handle::Path(..) | Handle::Bytes(..) => &[],
        }
    }

    pub fn ground(&self, area: Rect, scale: f32) -> f32 {
        let [width, height] = self.size.map(|side| side as usize);
        let left = ((area.x * scale).max(0.0) as usize).min(width);
        let top = ((area.y * scale).max(0.0) as usize).min(height);
        let columns = ((area.w * scale) as usize).min(width - left);
        let rows = ((area.h * scale) as usize).min(height - top);
        paper_widgets::ground(self.rgba(), width, [left, top, columns, rows])
    }

    fn matte_at(&self, at: [f32; 2]) -> u8 {
        let Some(matte) = &self.matte else { return 0 };
        let [width, height] = self.size.map(i64::from);
        let (x, y) = (at[0].floor() as i64, at[1].floor() as i64);
        if !(0..width).contains(&x) || !(0..height).contains(&y) {
            return 0;
        }
        matte[(y * width + x) as usize]
    }

    pub fn hidden(&self, area: Rect, scale: f32) -> bool {
        if self.matte.is_none() {
            return false;
        }
        let cell = LOOK.mask_cell;
        let (columns, rows) = ((area.w / cell).ceil().max(1.0), (area.h / cell).ceil().max(1.0));
        let mut covered = 0.0;
        for row in 0..rows as usize {
            for column in 0..columns as usize {
                let at = [
                    (area.x + (column as f32 + 0.5) * cell) * scale,
                    (area.y + (row as f32 + 0.5) * cell) * scale,
                ];
                if self.matte_at(at) > LOOK.mask_alpha {
                    covered += 1.0;
                }
            }
        }
        covered / (columns * rows) >= LOOK.hidden
    }

    pub fn hatch(&self, accent: [u8; 3], scale: f32) -> Option<Hatch> {
        let matte = self.matte.as_ref()?;
        let width = self.size[0] as usize;
        let [left, top, columns, rows] = covered(matte, width)?;
        let pitch = LOOK.hatch_pitch * scale;
        let line = LOOK.hatch_line * scale * std::f32::consts::SQRT_2;
        let mut rgba = vec![0; columns * rows * CHANNELS];
        for (index, pixel) in rgba.chunks_exact_mut(CHANNELS).enumerate() {
            let (x, y) = (left + index % columns, top + index / columns);
            let cover = matte[y * width + x];
            if cover == 0 || (x as f32 - y as f32).rem_euclid(pitch) >= line {
                continue;
            }
            pixel[..ALPHA].copy_from_slice(&accent);
            pixel[ALPHA] = cover;
        }
        let image = Handle::from_rgba(columns as u32, rows as u32, rgba);
        Some(Hatch { image, bounds: [left, top, columns, rows] })
    }
}

fn covered(matte: &[u8], width: usize) -> Option<[usize; 4]> {
    let mut found: Option<[usize; 4]> = None;
    for (row, line) in matte.chunks_exact(width).enumerate() {
        let (Some(first), Some(last)) =
            (line.iter().position(|&cover| cover > 0), line.iter().rposition(|&cover| cover > 0))
        else {
            continue;
        };
        let [left, top, right, _] = found.unwrap_or([first, row, last, row]);
        found = Some([left.min(first), top, right.max(last), row]);
    }
    found.map(|[left, top, right, bottom]| [left, top, right - left + 1, bottom - top + 1])
}

pub fn tones(scenery: &Scenery, stage: Option<&Stage>, area: Rect, scale: f32) -> Tones {
    let ground = stage.map_or(0.0, |stage| stage.ground(area, scale));
    Tones::over(&scenery.palette, ground)
}

pub fn chrome(scenery: &Scenery, stage: Option<&Stage>, area: Rect, scale: f32) -> Tones {
    let ground = stage.map_or(0.0, |stage| stage.ground(area, scale));
    ink(&scenery.palette, ground)
}

pub fn ink(palette: &Palette, ground: f32) -> Tones {
    let Palette { text, shade, .. } = *palette;
    let (light, dark) =
        if luminance(text) >= luminance(shade) { (text, shade) } else { (shade, text) };
    let (text, shade) = if ground < DARK_GROUND { (light, dark) } else { (dark, light) };
    let muted = std::array::from_fn(|channel| {
        let [from, to] = [text[channel], shade[channel]].map(f32::from);
        (from + (to - from) * MUTED_SHARE).round() as u8
    });
    Tones {
        text: Ink::solid(text),
        accent: Tones::over(palette, luminance(shade)).accent,
        muted: Ink::solid(muted),
        shade: Ink::solid(shade),
    }
}

impl Proxy {
    pub fn paint(key: Key, scenery: &Scenery) -> Self {
        let size = frame(key.face);
        let mut canvas = Canvas::scaled(size, key.height as f32 / size[1]);
        let scene = Scene {
            lettering: &scenery.lettering,
            tones: key.tones,
            moment: key.moment,
            labels: &scenery.labels,
        };
        paper_widgets::draw(key.face, &mut canvas, &scene);
        let size = [canvas.width, canvas.height];
        let plain = handle(&canvas.pixels, size, None);
        Self { key, size, pixels: canvas.pixels, plain, cut: None }
    }

    pub fn cut(&mut self, stage: &Stage, area: Rect, scale: f32) {
        let bounds = [area.x, area.y, area.w, area.h].map(|side| (side * scale).round() as i32);
        if self.cut.as_ref().is_none_or(|(at, _)| *at != bounds) {
            let [width, height] = self.size;
            let step = [area.w * scale / width as f32, area.h * scale / height as f32];
            let cover: Vec<u8> = (0..width * height)
                .map(|index| {
                    let (column, row) = ((index % width) as f32, (index / width) as f32);
                    stage.matte_at([
                        area.x * scale + (column + 0.5) * step[0],
                        area.y * scale + (row + 0.5) * step[1],
                    ])
                })
                .collect();
            self.cut = Some((bounds, handle(&self.pixels, self.size, Some(&cover))));
        }
    }
}

fn handle(pixels: &[[u8; 4]], size: [usize; 2], cover: Option<&[u8]>) -> Handle {
    let mut rgba = Vec::with_capacity(pixels.len() * CHANNELS);
    for (index, pixel) in pixels.iter().enumerate() {
        let keep = cover.map_or(1.0, |cover| 1.0 - f32::from(cover[index]) / OPAQUE);
        let alpha = f32::from(pixel[ALPHA]) * keep;
        if alpha <= 0.0 {
            rgba.extend_from_slice(&[0; CHANNELS]);
            continue;
        }
        let lift = OPAQUE / f32::from(pixel[ALPHA]);
        for channel in &pixel[..ALPHA] {
            rgba.push((f32::from(*channel) * lift).round().min(OPAQUE) as u8);
        }
        rgba.push(alpha.round() as u8);
    }
    Handle::from_rgba(size[0] as u32, size[1] as u32, rgba)
}
