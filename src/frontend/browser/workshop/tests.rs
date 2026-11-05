use crate::contracts::browser::WorkshopPreview;

use super::{description, grouped, preview_at, utc_date};

#[test]
fn dates_cross_leap_day() {
    assert_eq!(utc_date(0), "1970-01-01");
    assert_eq!(utc_date(1_709_164_800), "2024-02-29");
    assert_eq!(utc_date(1_709_251_200), "2024-03-01");
    assert_eq!(utc_date(u64::MAX), "9999-12-31");
}

#[test]
fn description_keeps_unicode() {
    let raw = "é".repeat(400);
    let (copy, clipped) = description(&raw, false);
    assert!(clipped);
    assert_eq!(copy.chars().count(), 361);
    assert_eq!(description(&raw, true).0, raw);
}

#[test]
fn keeps_gallery_sequence() {
    let previews = [
        WorkshopPreview::Youtube { id: String::from("first") },
        WorkshopPreview::Image { url: String::from("image-a") },
        WorkshopPreview::Youtube { id: String::from("second") },
        WorkshopPreview::Image { url: String::from("image-b") },
    ];
    assert_eq!(preview_at(&previews, 0), None);
    assert_eq!(preview_at(&previews, 1), Some((0, &previews[0])));
    assert_eq!(preview_at(&previews, 2), Some((1, &previews[1])));
    assert_eq!(preview_at(&previews, 4), Some((3, &previews[3])));
    assert_eq!(preview_at(&previews, 5), None);
    assert_eq!(previews[0].poster_url(), "https://i.ytimg.com/vi/first/hqdefault.jpg");
    assert!(previews[0].is_video());
    assert!(!previews[1].is_video());
}

#[test]
fn groups_thousands() {
    assert_eq!(grouped(0), "0");
    assert_eq!(grouped(999), "999");
    assert_eq!(grouped(9_812), "9\u{202f}812");
    assert_eq!(grouped(1_184_233), "1\u{202f}184\u{202f}233");
}
