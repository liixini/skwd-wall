#[cfg(target_os = "linux")]
use super::{image_sources, path_bytes};

#[cfg(target_os = "linux")]
#[test]
fn preserves_original_images() {
    for format in [
        image::ImageFormat::Png,
        image::ImageFormat::Jpeg,
        image::ImageFormat::WebP,
        image::ImageFormat::Bmp,
        image::ImageFormat::Gif,
        image::ImageFormat::Tiff,
    ] {
        let image = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            11,
            7,
            image::Rgba([17, 28, 39, 255]),
        ));
        let image = if format == image::ImageFormat::Jpeg {
            image::DynamicImage::ImageRgb8(image.to_rgb8())
        } else {
            image
        };
        let mut encoded = std::io::Cursor::new(Vec::new());
        image.write_to(&mut encoded, format).unwrap();
        let original = encoded.into_inner();
        let decoded = image::load_from_memory(&original).unwrap().into_rgba8();
        let sources = image_sources(original.clone()).unwrap();
        assert_eq!(
            sources[0].mime_type,
            wl_clipboard_rs::copy::MimeType::Specific(format.to_mime_type().into())
        );
        assert_eq!(sources[0].source, wl_clipboard_rs::copy::Source::Bytes(original.into()));
        let png = sources
            .iter()
            .find(|source| {
                source.mime_type == wl_clipboard_rs::copy::MimeType::Specific("image/png".into())
            })
            .unwrap();
        let wl_clipboard_rs::copy::Source::Bytes(bytes) = &png.source else {
            panic!("image bytes")
        };
        assert_eq!(image::load_from_memory(bytes).unwrap().into_rgba8(), decoded);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_corrupt_image() {
    assert!(image_sources(b"not an image".to_vec()).is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn copies_absolute_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("source.jpg");
    std::fs::write(&path, b"file").unwrap();
    assert_eq!(path_bytes(&path).unwrap(), path.to_str().unwrap().as_bytes());
}

#[cfg(target_os = "linux")]
#[test]
fn resolves_relative_path() {
    let dir = tempfile::tempdir_in(".").unwrap();
    let cwd = std::env::current_dir().unwrap();
    let path = dir.path().strip_prefix(&cwd).unwrap().join("image with spaces.jpg");
    assert!(!path.is_absolute());
    std::fs::write(&path, b"file").unwrap();
    let absolute = path.canonicalize().unwrap();
    assert_eq!(path_bytes(&path).unwrap(), absolute.to_str().unwrap().as_bytes());
}

#[cfg(target_os = "linux")]
#[test]
fn rejects_nonutf8_path() {
    use std::os::unix::ffi::OsStringExt;

    let dir = tempfile::tempdir().unwrap();
    let name = std::ffi::OsString::from_vec(vec![b'i', b'm', 0xff]);
    let path = dir.path().join(name);
    std::fs::write(&path, b"file").unwrap();
    assert!(path_bytes(&path).is_err());
}
