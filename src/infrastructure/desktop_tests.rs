use image::{Rgba, RgbaImage};
use paper_control::Background;
use paper_geom::FillMode;

use super::{Request, fill_mode, load};

fn request(path: &std::path::Path, fill: FillMode) -> Request {
    Request {
        wallpaper: Some(path.to_path_buf()),
        fill,
        background: Background { color: [1, 2, 3], blur: false },
        size: [4, 2],
        outlines: Vec::new(),
        font: None,
    }
}

fn picture(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let path = dir.path().join("wall.png");
    RgbaImage::from_pixel(2, 2, Rgba([200, 200, 200, 255])).save(&path).unwrap();
    path
}

#[test]
fn parses_fill() {
    assert_eq!(fill_mode("fit"), FillMode::Fit);
    assert_eq!(fill_mode(""), FillMode::Fill);
}

#[test]
fn letterboxes_fit() {
    let dir = tempfile::tempdir().unwrap();
    let backdrop = load(&request(&picture(&dir), FillMode::Fit)).backdrop.expect("backdrop");
    assert_eq!(backdrop.size, [4, 2]);
    assert_eq!(&backdrop.rgba[..4], [1, 2, 3, 255]);
    assert_eq!(&backdrop.rgba[4..8], [200, 200, 200, 255]);
    assert!(backdrop.matte.is_none());
}

#[test]
fn span_has_no_backdrop() {
    let dir = tempfile::tempdir().unwrap();
    assert!(load(&request(&picture(&dir), FillMode::Span)).backdrop.is_none());
}

#[test]
fn embedded_font_default() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(load(&request(&picture(&dir), FillMode::Fill)).font, paper_widgets::FONT);
}
