use super::*;

#[test]
fn uri_escapes_unsafe_bytes() {
    let path = Path::new("/home/me/Pictures/Gwenni good, 2026/ä.png");
    assert_eq!(file_uri(path), "file:///home/me/Pictures/Gwenni%20good%2C%202026/%C3%A4.png");
}

#[test]
fn uri_keeps_raw_bytes() {
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(std::ffi::OsStr::from_bytes(b"/w/\xff.png"));
    assert_eq!(file_uri(path), "file:///w/%FF.png");
}
