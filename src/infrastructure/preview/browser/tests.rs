use std::{fs, time::Duration};

use image::{Delay, Frame, Rgba, RgbaImage, codecs::gif::GifEncoder};

use crate::contracts::browser::preview_frame_index;

use crate::contracts::browser::PreviewSize;

use super::{MAX_ENCODED_BYTES, decode_browser_preview};

fn scratch() -> tempfile::TempDir {
    let target = std::env::var_os("CARGO_TARGET_DIR").unwrap_or_else(|| "target".into());
    fs::create_dir_all(&target).unwrap();
    tempfile::tempdir_in(target).unwrap()
}

#[test]
fn gif_ignores_extension() {
    let dir = scratch();
    let path = dir.path().join("cached.jpg");
    let file = fs::File::create(&path).unwrap();
    let mut encoder = GifEncoder::new(file);
    let red = RgbaImage::from_pixel(2, 1, Rgba([255, 0, 0, 255]));
    let blue = RgbaImage::from_pixel(2, 1, Rgba([0, 0, 255, 255]));
    encoder
        .encode_frames([
            Frame::from_parts(red, 0, 0, Delay::from_numer_denom_ms(20, 1)),
            Frame::from_parts(blue, 0, 0, Delay::from_numer_denom_ms(100, 1)),
        ])
        .unwrap();
    drop(encoder);

    let preview = decode_browser_preview(&path, PreviewSize::Hero).unwrap();
    assert_eq!(preview.frames.len(), 2);
    assert_eq!(preview.frames[0].delay, Duration::from_millis(33));
    assert_eq!(preview.frames[1].delay, Duration::from_millis(100));
    let delays = preview.frames.iter().map(|frame| frame.delay).collect::<Vec<_>>();
    assert_eq!(preview_frame_index(&delays, preview.duration, Duration::from_millis(32)), 0);
    assert_eq!(preview_frame_index(&delays, preview.duration, Duration::from_millis(33)), 1);
    assert_eq!(preview_frame_index(&delays, preview.duration, Duration::from_millis(133)), 0);
    assert_eq!(&preview.frames[1].rgba[..4], &[0, 0, 255, 255]);
}

#[test]
fn thumbnail_animates() {
    let dir = scratch();
    let path = dir.path().join("animated.jpg");
    let file = fs::File::create(&path).unwrap();
    let mut encoder = GifEncoder::new(file);
    let red = RgbaImage::from_pixel(320, 160, Rgba([255, 0, 0, 255]));
    let blue = RgbaImage::from_pixel(320, 160, Rgba([0, 0, 255, 255]));
    encoder
        .encode_frames([
            Frame::from_parts(red, 0, 0, Delay::from_numer_denom_ms(100, 1)),
            Frame::from_parts(blue, 0, 0, Delay::from_numer_denom_ms(100, 1)),
        ])
        .unwrap();
    drop(encoder);

    let thumb = decode_browser_preview(&path, PreviewSize::Thumbnail).unwrap();
    assert!(thumb.animated());
    assert_eq!((thumb.frames[0].width, thumb.frames[0].height), (160, 80));
    assert_eq!(&thumb.frames[1].rgba[..4], &[0, 0, 255, 255]);
    assert_eq!(thumb.duration, Duration::from_millis(200));
}

#[test]
fn card_skips_stills() {
    let dir = scratch();
    let path = dir.path().join("still.jpg");
    RgbaImage::from_pixel(4, 4, Rgba([9, 9, 9, 255]))
        .save_with_format(&path, image::ImageFormat::Png)
        .unwrap();
    assert!(decode_browser_preview(&path, PreviewSize::Card).unwrap().frames.is_empty());
}

#[test]
fn oversized_still_scales() {
    let dir = scratch();
    let path = dir.path().join("huge.png");
    RgbaImage::from_pixel(9000, 4200, Rgba([5, 6, 7, 255])).save(&path).unwrap();
    let preview = decode_browser_preview(&path, PreviewSize::Hero).unwrap();
    assert_eq!((preview.frames[0].width, preview.frames[0].height), (1920, 896));
}

#[test]
fn oversized_gif_stills() {
    let dir = scratch();
    let path = dir.path().join("wide.gif");
    let file = fs::File::create(&path).unwrap();
    let mut encoder = GifEncoder::new(file);
    let frame = RgbaImage::from_pixel(2600, 8, Rgba([0, 255, 0, 255]));
    encoder
        .encode_frames([
            Frame::from_parts(frame.clone(), 0, 0, Delay::from_numer_denom_ms(100, 1)),
            Frame::from_parts(frame, 0, 0, Delay::from_numer_denom_ms(100, 1)),
        ])
        .unwrap();
    drop(encoder);
    let preview = decode_browser_preview(&path, PreviewSize::Hero).unwrap();
    assert!(!preview.animated());
    assert_eq!(preview.frames[0].width, 1920);
}

#[test]
fn rejects_invalid_bytes() {
    let dir = scratch();
    let path = dir.path().join("broken.png");
    fs::write(&path, b"invalid").unwrap();
    assert!(decode_browser_preview(&path, PreviewSize::Hero).is_err());
}

#[test]
fn accepts_ultrawide_still() {
    let dir = scratch();
    let path = dir.path().join("wide.png");
    RgbaImage::from_pixel(5120, 32, Rgba([3, 7, 11, 255])).save(&path).unwrap();
    let preview = decode_browser_preview(&path, PreviewSize::Hero).unwrap();
    assert_eq!(preview.frames[0].width, 1920);
    assert_eq!(preview.frames.len(), 1);
}

#[test]
fn rejects_large_file() {
    let dir = scratch();
    let path = dir.path().join("large.png");
    let file = fs::File::create(&path).unwrap();
    file.set_len(MAX_ENCODED_BYTES + 1).unwrap();
    assert!(decode_browser_preview(&path, PreviewSize::Hero).is_err());
}

#[test]
fn still_has_no_timeline() {
    let dir = scratch();
    let path = dir.path().join("image.bin");
    let image = RgbaImage::from_pixel(2, 2, Rgba([1, 2, 3, 255]));
    image.save_with_format(&path, image::ImageFormat::Png).unwrap();
    let preview = decode_browser_preview(&path, PreviewSize::Hero).unwrap();
    assert_eq!(preview.frames.len(), 1);
    assert_eq!(preview_frame_index(&[preview.frames[0].delay], preview.duration, Duration::MAX), 0);
    assert_eq!(preview.frames[0].rgba.len(), 16);
}
