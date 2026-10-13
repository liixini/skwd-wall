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
fn folder_path_local_media() {
    assert_eq!(wallpaper("anime/seasonal/x.webp").folder_path(), "anime/seasonal");
    assert_eq!(wallpaper("top.webp").folder_path(), "");
    assert_eq!(wallpaper("one/two.png").folder_path(), "one");

    let video = Wallpaper {
        name: String::from("videos/clip.mp4"),
        kind: WallpaperKind::Video,
        ..Wallpaper::default()
    };
    assert_eq!(video.folder_path(), "videos");
    let workshop = Wallpaper { kind: WallpaperKind::We, ..video };
    assert_eq!(workshop.folder_path(), "");
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
fn folder_roots_sorted() {
    let catalog = Catalog {
        items: vec![
            Wallpaper {
                path: "/walls/anime/seasonal/2024/x.webp".into(),
                ..wallpaper("anime/seasonal/2024/x.webp")
            },
            Wallpaper {
                path: "/walls/anime/seasonal/y.webp".into(),
                ..wallpaper("anime/seasonal/y.webp")
            },
            Wallpaper { path: "/extra/photos/z.png".into(), ..wallpaper("/extra/photos/z.png") },
            Wallpaper { path: "/walls/top.png".into(), ..wallpaper("top.png") },
            Wallpaper {
                path: "/walls/anime/clip.mp4".into(),
                name: String::from("anime/clip.mp4"),
                kind: WallpaperKind::Video,
                ..Wallpaper::default()
            },
        ],
        ..Catalog::default()
    };

    assert_eq!(
        catalog.available_folders(&[String::from("/walls"), String::from("/extra")], None),
        [
            "/walls",
            "/walls/anime",
            "/walls/anime/seasonal",
            "/walls/anime/seasonal/2024",
            "/extra",
            "/extra/photos"
        ]
    );
}

#[test]
fn hidden_roots_follow_reveal() {
    let catalog = Catalog {
        items: vec![
            Wallpaper { key: "a".into(), path: "/walls/a.png".into(), ..wallpaper("a.png") },
            Wallpaper { key: "b".into(), path: "/private/b.png".into(), ..wallpaper("b.png") },
            Wallpaper { key: "c".into(), path: "/junk/c.png".into(), ..wallpaper("c.png") },
        ],
        profile_hidden: ["b".to_string()].into(),
        profile_excluded: ["c".to_string()].into(),
        ..Catalog::default()
    };
    let roots = ["/walls", "/private", "/junk", "/empty"].map(str::to_string);
    assert_eq!(catalog.available_folders(&roots, None), roots);
    assert_eq!(catalog.available_folders(&roots, Some(false)), ["/walls", "/empty"]);
    assert_eq!(catalog.available_folders(&roots, Some(true)), ["/walls", "/private", "/empty"]);
}

#[test]
fn nested_roots_distinct() {
    let catalog = Catalog {
        items: vec![
            Wallpaper { path: "/walls/photos/a.png".into(), ..wallpaper("photos/a.png") },
            Wallpaper { path: "/walls/nested/photos/b.png".into(), ..wallpaper("photos/b.png") },
            Wallpaper { path: "/other/photos/c.png".into(), ..wallpaper("/other/photos/c.png") },
        ],
        ..Catalog::default()
    };
    let roots = ["/walls", "/walls/nested", "/other"].map(str::to_string);
    assert_eq!(
        catalog.available_folders(&roots, None),
        [
            "/walls",
            "/walls/photos",
            "/walls/nested",
            "/walls/nested/photos",
            "/other",
            "/other/photos"
        ]
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
