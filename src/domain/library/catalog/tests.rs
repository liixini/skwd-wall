#![cfg(test)]

use super::catalog::{Catalog, Wallpaper, WallpaperColor, WallpaperKind};

fn wallpaper(name: &str) -> Wallpaper {
    Wallpaper { name: name.to_string(), kind: WallpaperKind::Static, ..Wallpaper::default() }
}

#[test]
fn kind_names_stable() {
    assert_eq!(WallpaperKind::Static.as_str(), "static");
    assert_eq!(WallpaperKind::Video.as_str(), "video");
    assert_eq!(WallpaperKind::We.as_str(), "we");
}

#[test]
fn we_video_effective_kind() {
    let scene = Wallpaper { kind: WallpaperKind::We, ..Wallpaper::default() };
    assert_eq!(scene.effective_kind(), WallpaperKind::We);

    let video = Wallpaper {
        kind: WallpaperKind::We,
        video_file: String::from("/workshop/clip.mp4"),
        ..Wallpaper::default()
    };
    assert_eq!(video.effective_kind(), WallpaperKind::Video);
}

#[test]
fn folder_path_static_only() {
    assert_eq!(wallpaper("anime/seasonal/x.webp").folder_path(), "anime/seasonal");
    assert_eq!(wallpaper("top.webp").folder_path(), "");
    assert_eq!(wallpaper("one/two.png").folder_path(), "one");

    let video = Wallpaper {
        name: String::from("videos/clip.mp4"),
        kind: WallpaperKind::Video,
        ..Wallpaper::default()
    };
    assert_eq!(video.folder_path(), "");
}

#[test]
fn wallpaper_identity_keys() {
    let local = Wallpaper {
        key: String::from("static:a.png"),
        name: String::from("a.png"),
        ..Wallpaper::default()
    };
    assert_eq!(local.dedup_key(), "a.png");
    assert_eq!(local.applied_key(), "a");

    let workshop = Wallpaper {
        key: String::from("we:111"),
        name: String::from("renamed scene"),
        kind: WallpaperKind::We,
        we_id: String::from("111"),
        ..Wallpaper::default()
    };
    assert_eq!(workshop.dedup_key(), "111");
    assert_eq!(workshop.applied_key(), "111");
}

#[test]
fn applied_key_final_extension() {
    assert_eq!(wallpaper("archive.tar.webp").applied_key(), "archive.tar");
    assert_eq!(wallpaper(".hidden").applied_key(), ".hidden");
    assert_eq!(wallpaper("extensionless").applied_key(), "extensionless");
}

#[test]
fn catalog_metadata_by_key() {
    let item = Wallpaper {
        key: String::from("static:a.png"),
        name: String::from("a.png"),
        ..Wallpaper::default()
    };
    let mut catalog = Catalog { items: vec![item.clone()], ..Catalog::default() };
    catalog.tags.insert(item.key.clone(), vec![String::from("forest"), String::from("green")]);
    catalog.favourites.insert(item.key.clone());
    catalog.colors.insert(item.key.clone(), WallpaperColor { hue: 7 });

    assert_eq!(catalog.tags_for(&item), ["forest", "green"]);
    assert!(catalog.is_favourite(&item));
    assert_eq!(catalog.colors.get(&item.key), Some(&WallpaperColor { hue: 7 }));

    let missing = Wallpaper { key: String::from("missing"), ..Wallpaper::default() };
    assert!(catalog.tags_for(&missing).is_empty());
    assert!(!catalog.is_favourite(&missing));
}

#[test]
fn folder_prefixes_sorted() {
    let catalog = Catalog {
        items: vec![
            wallpaper("anime/seasonal/2024/x.webp"),
            wallpaper("anime/seasonal/y.webp"),
            wallpaper("photos/z.png"),
            wallpaper("top.png"),
            Wallpaper {
                name: String::from("ignored/video.mp4"),
                kind: WallpaperKind::Video,
                ..Wallpaper::default()
            },
        ],
        ..Catalog::default()
    };

    assert_eq!(
        catalog.available_folders(),
        ["anime", "anime/seasonal", "anime/seasonal/2024", "photos"]
    );
}

#[test]
fn backdrop_preserves_animated_sources() {
    let video = Wallpaper {
        kind: WallpaperKind::Video,
        path: "/wall/clip.mp4".into(),
        thumb: "/cache/frame.png".into(),
        ..Default::default()
    };
    assert_eq!(video.backdrop_source(), "/wall/clip.mp4");
    let mut scene = Wallpaper {
        kind: WallpaperKind::We,
        key: "we:3260370312".into(),
        thumb: "/cache/scene.png".into(),
        ..Default::default()
    };
    assert_eq!(scene.backdrop_source(), "we:3260370312");
    scene.we_id = "123456".into();
    assert_eq!(scene.backdrop_source(), "we:123456");
}
