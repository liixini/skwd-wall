use skwd_palette::studio::AREA_EDGE;

use super::state::Decoded;
use crate::domain::studio::Pixels;

const GREY_WEIGHTS: (f32, f32, f32) = (0.2126, 0.7152, 0.0722);
const RGBA: usize = 4;
const RGB: usize = 3;

pub fn decode(path: &str) -> Option<Decoded> {
    let picture = image::open(path).ok()?;
    let (width, height) = (picture.width(), picture.height());
    if width == 0 || height == 0 {
        return None;
    }
    let aspect = width as f32 / height as f32;
    let small = if width.max(height) > AREA_EDGE {
        picture.resize(AREA_EDGE, AREA_EDGE, image::imageops::FilterType::Triangle)
    } else {
        picture.clone()
    };
    let reduced = small.to_rgba8();
    let pixels = Pixels {
        width: reduced.width() as usize,
        height: reduced.height() as usize,
        rgba: reduced.into_raw(),
        aspect,
    };
    let mut grey = picture.to_rgba8().into_raw();
    for pixel in grey.chunks_exact_mut(RGBA) {
        let (red, green, blue) = (f32::from(pixel[0]), f32::from(pixel[1]), f32::from(pixel[2]));
        let level =
            (red * GREY_WEIGHTS.0 + green * GREY_WEIGHTS.1 + blue * GREY_WEIGHTS.2).round() as u8;
        pixel[..RGB].fill(level);
    }
    let grey = iced::widget::image::Handle::from_rgba(width, height, grey);
    Some(Decoded { pixels, grey })
}
