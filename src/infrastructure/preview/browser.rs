use std::{fs::File, io::BufReader, path::Path, sync::Arc, time::Duration};

use image::{
    AnimationDecoder, DynamicImage, Frames, GenericImageView, ImageDecoder, ImageFormat,
    ImageReader, Limits, Rgba, RgbaImage,
    codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder},
};

use crate::contracts::browser::{PreviewFrame, PreviewFrames, PreviewSize};

const MAX_ENCODED_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SOURCE_EDGE: u32 = 16_384;
const MAX_SOURCE_PIXELS: u64 = 48 * 1024 * 1024;
const MAX_ANIMATED_SOURCE_EDGE: u32 = 2048;
const MAX_ANIMATED_SOURCE_PIXELS: u64 = 4 * 1024 * 1024;
const MAX_DECODE_ALLOC: u64 = 192 * 1024 * 1024;
const MAX_ANIMATED_FRAMES: usize = 120;
const MIN_DELAY_MS: u64 = 33;
const MAX_DELAY_MS: u64 = 10_000;
const SMOOTH_DELAY: Duration = Duration::from_millis(MIN_DELAY_MS);
const AMBIENT_DELAY: Duration = Duration::from_millis(66);
const HERO: Budget = Budget {
    still_edge: Some(1920),
    animated_edge: 960,
    frame_bytes: 32 << 20,
    min_delay: SMOOTH_DELAY,
};
const THUMBNAIL: Budget = Budget {
    still_edge: Some(160),
    animated_edge: 160,
    frame_bytes: 8 << 20,
    min_delay: AMBIENT_DELAY,
};
const CARD: Budget = Budget {
    still_edge: None,
    animated_edge: 256,
    frame_bytes: 16 << 20,
    min_delay: AMBIENT_DELAY,
};
const FILE_LIMIT_ERROR: &str = "preview file is missing or too large";
const DIMENSION_ERROR: &str = "preview dimensions exceed limit";
const EMPTY_ANIMATION_ERROR: &str = "animated preview has no frames";

struct Budget {
    still_edge: Option<u32>,
    animated_edge: u32,
    frame_bytes: usize,
    min_delay: Duration,
}

pub fn decode_browser_preview(path: &Path, size: PreviewSize) -> Result<PreviewFrames, String> {
    let metadata = path.metadata().map_err(|err| err.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_ENCODED_BYTES {
        return Err(String::from(FILE_LIMIT_ERROR));
    }
    let budget = match size {
        PreviewSize::Hero => &HERO,
        PreviewSize::Thumbnail => &THUMBNAIL,
        PreviewSize::Card => &CARD,
    };
    let file = File::open(path).map_err(|err| err.to_string())?;
    let mut reader = ImageReader::new(BufReader::new(file))
        .with_guessed_format()
        .map_err(|err| err.to_string())?;
    match reader.format() {
        Some(ImageFormat::Gif) => {
            let decoder = GifDecoder::new(reader.into_inner()).map_err(|err| err.to_string())?;
            let dimensions = decoder.dimensions();
            animation(limited(decoder)?.into_frames(), dimensions, budget)
        }
        Some(ImageFormat::WebP) => {
            let decoder = WebPDecoder::new(reader.into_inner()).map_err(|err| err.to_string())?;
            if decoder.has_animation() {
                let dimensions = decoder.dimensions();
                animation(limited(decoder)?.into_frames(), dimensions, budget)
            } else {
                still(decoder, budget)
            }
        }
        Some(ImageFormat::Png) => {
            let decoder = PngDecoder::with_limits(reader.into_inner(), limits())
                .map_err(|err| err.to_string())?;
            if decoder.is_apng().map_err(|err| err.to_string())? {
                let dimensions = decoder.dimensions();
                let decoder = limited(decoder)?.apng().map_err(|err| err.to_string())?;
                animation(decoder.into_frames(), dimensions, budget)
            } else {
                still(decoder, budget)
            }
        }
        _ => {
            reader.limits(limits());
            still(reader.into_decoder().map_err(|err| err.to_string())?, budget)
        }
    }
}

fn limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SOURCE_EDGE);
    limits.max_image_height = Some(MAX_SOURCE_EDGE);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    limits
}

fn limited<D: ImageDecoder>(mut decoder: D) -> Result<D, String> {
    decoder.set_limits(limits()).map_err(|err| err.to_string())?;
    Ok(decoder)
}

fn still(decoder: impl ImageDecoder, budget: &Budget) -> Result<PreviewFrames, String> {
    let Some(edge) = budget.still_edge else {
        return Ok(PreviewFrames { frames: Vec::new(), duration: Duration::ZERO });
    };
    let (width, height) = decoder.dimensions();
    if !valid_dimensions(width, height, MAX_SOURCE_EDGE, MAX_SOURCE_PIXELS) {
        return Err(String::from(DIMENSION_ERROR));
    }
    let image = DynamicImage::from_decoder(limited(decoder)?).map_err(|err| err.to_string())?;
    Ok(single(if width.max(height) > edge { shrink(&image, edge) } else { image.to_rgba8() }))
}

fn animation(
    frames: Frames<'_>,
    (width, height): (u32, u32),
    budget: &Budget,
) -> Result<PreviewFrames, String> {
    if !valid_dimensions(width, height, MAX_SOURCE_EDGE, MAX_SOURCE_PIXELS) {
        return Err(String::from(DIMENSION_ERROR));
    }
    let animate =
        valid_dimensions(width, height, MAX_ANIMATED_SOURCE_EDGE, MAX_ANIMATED_SOURCE_PIXELS);
    if !animate && budget.still_edge.is_none() {
        return Ok(PreviewFrames { frames: Vec::new(), duration: Duration::ZERO });
    }
    let edge = budget.animated_edge.max(budget.still_edge.unwrap_or_default());
    let mut decoded = Vec::new();
    let mut duration = Duration::ZERO;
    let mut bytes = 0;
    for frame in frames {
        let frame = frame.map_err(|err| err.to_string())?;
        if !animate {
            return Ok(single(shrink(&frame.into_buffer(), edge)));
        }
        if decoded.len() == MAX_ANIMATED_FRAMES {
            return Ok(first_frame(decoded));
        }
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay = Duration::from_millis(
            (u64::from(numerator) / u64::from(denominator)).clamp(MIN_DELAY_MS, MAX_DELAY_MS),
        );
        duration += delay;
        if let Some(last) = decoded.last_mut().filter(|last| last.delay < budget.min_delay) {
            last.delay += delay;
            continue;
        }
        let rgba = shrink(&frame.into_buffer(), budget.animated_edge);
        bytes += rgba.len();
        let (width, height) = rgba.dimensions();
        decoded.push(PreviewFrame { width, height, rgba: Arc::from(rgba.into_raw()), delay });
        if bytes > budget.frame_bytes {
            return Ok(first_frame(decoded));
        }
    }
    match decoded.len() {
        0 => Err(String::from(EMPTY_ANIMATION_ERROR)),
        1 => Ok(first_frame(decoded)),
        _ => Ok(PreviewFrames { frames: decoded, duration }),
    }
}

fn valid_dimensions(width: u32, height: u32, edge: u32, pixels: u64) -> bool {
    width > 0
        && height > 0
        && width <= edge
        && height <= edge
        && u64::from(width) * u64::from(height) <= pixels
}

fn shrink<I: GenericImageView<Pixel = Rgba<u8>>>(image: &I, edge: u32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let longest = width.max(height).max(1);
    let fit = |side: u32| {
        (u64::from(side) * u64::from(edge.min(longest)) / u64::from(longest)).max(1) as u32
    };
    image::imageops::thumbnail(image, fit(width), fit(height))
}

fn single(rgba: RgbaImage) -> PreviewFrames {
    let (width, height) = rgba.dimensions();
    PreviewFrames {
        frames: vec![PreviewFrame {
            width,
            height,
            rgba: Arc::from(rgba.into_raw()),
            delay: Duration::ZERO,
        }],
        duration: Duration::ZERO,
    }
}

fn first_frame(mut frames: Vec<PreviewFrame>) -> PreviewFrames {
    frames.truncate(1);
    if let Some(frame) = frames.first_mut() {
        frame.delay = Duration::ZERO;
    }
    PreviewFrames { frames, duration: Duration::ZERO }
}

#[cfg(test)]
mod tests;
