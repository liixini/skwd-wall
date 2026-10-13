#![cfg(test)]

use super::filter::*;
use crate::domain::library::catalog::{Catalog, Wallpaper, WallpaperColor, WallpaperKind};

fn item(key: &str) -> Wallpaper {
    Wallpaper {
        key: key.into(),
        kind: WallpaperKind::Static,
        name: format!("{key}.webp"),
        ..Default::default()
    }
}

fn with_items(items: Vec<Wallpaper>) -> Catalog {
    Catalog { items, ..Catalog::default() }
}

fn tagged_catalog() -> Catalog {
    let mut catalog = with_items(vec![item("a"), item("b"), item("c")]);
    catalog.tags.insert("a".into(), vec!["forest".into(), "green".into()]);
    catalog.tags.insert("b".into(), vec!["forest".into(), "ocean".into()]);
    catalog
}

fn keys(catalog: &Catalog, out: &[u32]) -> Vec<String> {
    out.iter().map(|&idx| catalog.items[idx as usize].key.clone()).collect()
}

fn named(key: &str, name: &str) -> Wallpaper {
    Wallpaper {
        key: key.into(),
        kind: WallpaperKind::Static,
        name: name.into(),
        ..Default::default()
    }
}

fn folder_catalog() -> Catalog {
    with_items(vec![
        named("top", "top.png"),
        named("as", "anime/seasonal/x.png"),
        named("as24", "anime/seasonal/2024/y.png"),
        named("ac", "anime/classic/z.png"),
        named("ph", "photos/p.png"),
        named("ls", "Landscapes/s.png"),
    ])
}

#[test]
fn absolute_folder_options() {
    let catalog = with_items(vec![
        named("poster", "/home/me/posters/a.jpg"),
        named("nested", "/home/me/posters/archive/b.jpg"),
        named("hidden", "/home/me/posters/.private/c.jpg"),
        named("ancestor", "/home/me/.private/posters/d.jpg"),
        named("legacy", "home/me/posters/e.jpg"),
    ]);
    let options = catalog.available_folders(&[String::from("/home/me/posters")], None);
    assert!(options.iter().any(|folder| folder == "/home/me/posters"));
    for folder in options {
        let filters = Filters { folder, ..Default::default() };
        assert!(!filter_sort(&catalog, &filters).is_empty(), "{}", filters.folder);
    }
    let mut filters = Filters { folder: "/home/me/posters".into(), ..Default::default() };
    assert_eq!(filter_sort(&catalog, &filters).len(), 3);
    filters.folder = "home/me/posters".into();
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), ["legacy"]);
}

#[test]
fn reveal_skips_excluded() {
    let mut catalog = with_items(vec![
        named("plain", "a.png"),
        named("card", "b.png"),
        named("folder", "anime/c.png"),
        named("gone", "sort/d.png"),
    ]);
    catalog.hidden.insert("card".into());
    catalog.profile_hidden.insert("folder".into());
    catalog.profile_excluded.insert("gone".into());
    let mut filters = Filters::default();
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), ["plain"]);
    filters.reveal = true;
    let mut shown = keys(&catalog, &filter_sort(&catalog, &filters));
    shown.sort();
    assert_eq!(shown, ["card", "folder", "plain"]);
}

#[test]
fn folder_options_follow_reveal() {
    let mut catalog = with_items(vec![
        Wallpaper { path: "/walls/a.png".into(), ..named("plain", "a.png") },
        Wallpaper { path: "/walls/anime/b.png".into(), ..named("anime", "anime/b.png") },
        Wallpaper { path: "/walls/sort/c.png".into(), ..named("sort", "sort/c.png") },
    ]);
    catalog.profile_hidden.insert("anime".into());
    catalog.profile_excluded.insert("sort".into());
    let roots = [String::from("/walls")];
    assert_eq!(catalog.available_folders(&roots, Some(false)), ["/walls"]);
    assert_eq!(catalog.available_folders(&roots, Some(true)), ["/walls", "/walls/anime"]);
    assert_eq!(catalog.available_folders(&roots, None).len(), 3);
}

#[test]
fn folder_descendants() {
    let catalog = folder_catalog();
    let filters = Filters { folder: "anime/seasonal".into(), ..Default::default() };
    let mut got = keys(&catalog, &filter_sort(&catalog, &filters));
    got.sort();
    assert_eq!(got, vec!["as", "as24"]);
}

#[test]
fn folder_segment_boundary() {
    let catalog = folder_catalog();
    let filters = Filters { folder: "Landscape".into(), ..Default::default() };
    assert!(keys(&catalog, &filter_sort(&catalog, &filters)).is_empty());
}

#[test]
fn absolute_source_matches_paths() {
    let catalog = with_items(vec![
        Wallpaper { name: "photo.png".into(), path: "/source/photo.png".into(), ..item("photo") },
        Wallpaper {
            name: "clip.mp4".into(),
            path: "/source/clip.mp4".into(),
            video_file: "/source/clip.mp4".into(),
            kind: WallpaperKind::Video,
            ..item("clip")
        },
        Wallpaper { name: "other.png".into(), path: "/source2/other.png".into(), ..item("other") },
    ]);
    let filters = Filters { folder: "/source".into(), ..Default::default() };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), ["photo", "clip"]);
}

#[test]
fn folder_sentinels() {
    let catalog = folder_catalog();
    let mut filters = Filters { folder: String::new(), ..Default::default() };
    assert_eq!(filter_sort(&catalog, &filters).len(), catalog.items.len());
    filters.folder = "*".into();
    assert_eq!(filter_sort(&catalog, &filters).len(), catalog.items.len());
}

#[test]
fn all_source_roots() {
    let catalog = with_items(vec![
        Wallpaper {
            key: "static:a".into(),
            name: "/first/a.png".into(),
            path: "/first/a.png".into(),
            ..item("a")
        },
        Wallpaper {
            key: "video:b".into(),
            name: "b.mp4".into(),
            path: "/second/b.mp4".into(),
            kind: WallpaperKind::Video,
            ..item("b")
        },
        Wallpaper {
            key: "static:c".into(),
            name: "/first/nested/c.png".into(),
            path: "/first/nested/c.png".into(),
            ..item("c")
        },
        Wallpaper {
            key: "video:d".into(),
            name: "d.mp4".into(),
            path: "/second/nested/d.mp4".into(),
            kind: WallpaperKind::Video,
            ..item("d")
        },
        Wallpaper {
            key: "static:e".into(),
            name: "/first2/e.png".into(),
            path: "/first2/e.png".into(),
            ..item("e")
        },
        Wallpaper {
            key: "we:123".into(),
            name: "Workshop".into(),
            kind: WallpaperKind::Video,
            path: "/workshop/movie.mp4".into(),
            ..item("we")
        },
        Wallpaper {
            key: "we:nested".into(),
            name: "Artist/Title".into(),
            kind: WallpaperKind::Video,
            path: "/first/movie.mp4".into(),
            ..item("nested-we")
        },
        Wallpaper {
            key: "generative:pond".into(),
            name: "Pond".into(),
            path: "/generated/pond".into(),
            ..item("pond")
        },
        Wallpaper { key: "we:456".into(), kind: WallpaperKind::We, ..item("scene") },
        named("legacy", "old.png"),
        Wallpaper {
            key: "static:relative".into(),
            name: "relative.png".into(),
            path: "relative.png".into(),
            ..item("relative")
        },
        named("legacy-nested", "old/deep.png"),
    ]);
    let filters = Filters { folder: String::new(), ..Default::default() };
    let mut shown = keys(&catalog, &filter_sort(&catalog, &filters));
    shown.sort_unstable();
    let mut expected = catalog.items.iter().map(|item| item.key.as_str()).collect::<Vec<_>>();
    expected.sort_unstable();
    assert_eq!(shown, expected);
    for key in ["static:c", "video:d", "we:456", "generative:pond"] {
        assert!(shown.iter().any(|shown| shown == key));
    }
}

#[test]
fn tag_include() {
    let catalog = tagged_catalog();
    let filters = Filters { tags: vec!["green".into()], ..Default::default() };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["a"]);
}

#[test]
fn tag_exclude() {
    let catalog = tagged_catalog();
    let filters = Filters { tags: vec!["-ocean".into()], ..Default::default() };
    let mut got = keys(&catalog, &filter_sort(&catalog, &filters));
    got.sort();
    assert_eq!(got, vec!["a", "c"]);
}

fn metadata_item(key: &str, width: i64, height: i64, duration_ms: i64, filesize: i64) -> Wallpaper {
    Wallpaper {
        key: key.into(),
        name: format!("{key}.webp"),
        kind: WallpaperKind::Video,
        width,
        height,
        duration_ms,
        filesize,
        ..Default::default()
    }
}

#[test]
fn numeric_ranges_evaluate_indexed_metadata() {
    let catalog = with_items(vec![
        metadata_item("short-hd", 1920, 1080, 20_000, 5 * 1024 * 1024),
        metadata_item("long-4k", 3840, 2160, 90_000, 40 * 1024 * 1024),
        metadata_item("unknown", 0, 0, 0, 0),
    ]);
    let filters = Filters {
        numeric: crate::domain::library::search::parse_numeric_query(
            "width:1920.. duration:<30s size:..10MiB",
        ),
        ..Default::default()
    };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), ["short-hd"]);
}

#[test]
fn numeric_or_and_exclusion_compose() {
    let catalog = with_items(vec![
        metadata_item("wide", 3840, 1080, 15_000, 10),
        metadata_item("tall", 1080, 3840, 15_000, 10),
        metadata_item("long", 1920, 1080, 120_000, 10),
        metadata_item("still", 1920, 1080, 0, 10),
    ]);
    let filters = Filters {
        numeric: crate::domain::library::search::parse_numeric_query(
            "width:>3000 | height:>3000 -duration:>30s",
        ),
        ..Default::default()
    };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), ["wide", "tall"]);
}

#[test]
fn numeric_predicates_are_anded_with_tag_match_modes() {
    let mut catalog = with_items(vec![
        metadata_item("forest-hd", 1920, 1080, 10_000, 10),
        metadata_item("ocean-4k", 3840, 2160, 10_000, 10),
        metadata_item("city-4k", 3840, 2160, 10_000, 10),
    ]);
    catalog.tags.insert("forest-hd".into(), vec!["forest".into()]);
    catalog.tags.insert("ocean-4k".into(), vec!["ocean".into()]);
    catalog.tags.insert("city-4k".into(), vec!["city".into()]);
    let filters = Filters {
        tags: vec!["forest".into(), "ocean".into()],
        tags_match_any: true,
        numeric: crate::domain::library::search::parse_numeric_query("res:>=2560x1440"),
        ..Default::default()
    };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), ["ocean-4k"]);
}

fn varied(key: &str, hue: i64, sat: i64, mtime: i64, richness: i64, applies: i64) -> Wallpaper {
    Wallpaper {
        key: key.into(),
        kind: WallpaperKind::Static,
        name: format!("{key}.webp"),
        hue,
        sat,
        mtime,
        richness,
        apply_count: applies,
        last_applied: applies * 100 + mtime,
        ..Default::default()
    }
}

fn varied_items() -> Vec<Wallpaper> {
    vec![
        varied("i0", 5, 40, 100, 7, 2),
        varied("i1", 99, 80, 300, 3, 0),
        varied("i2", 30, 40, 100, 7, 2),
        varied("i3", 5, 90, 500, 12, 5),
        varied("i4", 60, 10, 300, 3, 0),
        varied("i5", 99, 80, 200, 9, 1),
        varied("i6", 0, 55, 700, 0, 5),
        varied("i7", 30, 40, 100, 7, 3),
        varied("i8", 85, 5, 900, 15, 0),
        varied("i9", 12, 55, 500, 9, 2),
    ]
}

#[test]
fn insert_index_parity() {
    let items = varied_items();
    for sort in ["color", "date", "recent", "pop", "richness", "minimalist", "res", "unknown"] {
        let filters = Filters { sort: sort.into(), ..Default::default() };
        for index in 1..items.len() {
            let grown = with_items(items[..=index].to_vec());
            let prev = with_items(items[..index].to_vec());
            let mut incremental = filter_sort(&prev, &filters);
            let pos = insert_index(&grown, &filters, &incremental, index as u32);
            incremental.insert(pos, index as u32);
            assert_eq!(incremental, filter_sort(&grown, &filters), "sort={sort} item {index}");
        }
    }
}

#[test]
fn recent_sort_by_order() {
    let mut old_many = varied("old-many", 0, 0, 300, 0, 20);
    old_many.last_applied = 2;
    let mut newest_once = varied("newest-once", 0, 0, 100, 0, 1);
    newest_once.last_applied = 3;
    let mut never = varied("never", 0, 0, 900, 0, 0);
    never.last_applied = 0;
    let catalog = with_items(vec![old_many, newest_once, never]);
    let filters = Filters { sort: "recent".into(), ..Default::default() };
    assert_eq!(
        keys(&catalog, &filter_sort(&catalog, &filters)),
        vec!["newest-once", "old-many", "never"]
    );
}

#[test]
fn color_sort_hue_99() {
    let catalog = with_items(vec![varied("grey", 99, 90, 0, 0, 0), varied("red", 98, 10, 0, 0, 0)]);
    let filters = Filters::default();
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["red", "grey"]);
}

#[test]
fn color_filter_uses_hue() {
    let mut catalog = with_items(vec![varied("x", 5, 0, 0, 0, 0), varied("y", 99, 0, 0, 0, 0)]);
    catalog.colors.insert("x".into(), WallpaperColor { hue: 7 });
    let mut filters = Filters { color: 5, ..Default::default() };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["x"]);
    filters.color = 99;
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["y"]);
}

#[test]
fn kind_filter_effective() {
    let mut we_vid = item("wev");
    we_vid.kind = WallpaperKind::We;
    we_vid.video_file = "/we/1/clip.mp4".into();
    let mut we_scene = item("wes");
    we_scene.kind = WallpaperKind::We;
    let catalog = with_items(vec![item("st"), we_vid, we_scene]);
    let filters = Filters { kind: "video".into(), ..Default::default() };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["wev"]);
    let filters = Filters { kind: "we".into(), ..Default::default() };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["wes"]);
}

#[test]
fn favourites_only() {
    let mut catalog = tagged_catalog();
    catalog.favourites.insert("b".into());
    let filters = Filters { favourites_only: true, ..Default::default() };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["b"]);
}

#[test]
fn tag_all_vs_any() {
    let catalog = tagged_catalog();
    let mut filters = Filters { tags: vec!["forest".into(), "green".into()], ..Default::default() };
    filters.tags_match_any = false;
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["a"]);
    filters.tags_match_any = true;
    let mut got = keys(&catalog, &filter_sort(&catalog, &filters));
    got.sort();
    assert_eq!(got, vec!["a", "b"]);
}

fn sized(key: &str, w: i64, h: i64) -> Wallpaper {
    Wallpaper {
        key: key.into(),
        kind: WallpaperKind::Static,
        name: format!("{key}.webp"),
        width: w,
        height: h,
        ..Default::default()
    }
}

#[test]
fn orient_landscape_portrait() {
    let catalog = with_items(vec![
        sized("wide", 3440, 1440),
        sized("hd", 1920, 1080),
        sized("tall", 1080, 1920),
        sized("square", 1000, 1000),
        sized("unknown", 0, 0),
    ]);
    let mut filters = Filters { orient: "landscape".into(), ..Filters::default() };
    filters.sort = "date".into();
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["wide", "hd", "square"]);
    filters.orient = "portrait".into();
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["tall"]);
    filters.orient = String::new();
    assert_eq!(filter_sort(&catalog, &filters).len(), 5);
}

#[test]
fn resolution_exact_match() {
    let catalog = with_items(vec![
        sized("fhd", 1920, 1080),
        sized("fhd-portrait", 1080, 1920),
        sized("near", 1920, 1081),
        sized("uhd", 3840, 2160),
    ]);
    let mut filters = Filters { resolution: "1920x1080".into(), ..Filters::default() };
    filters.sort = "date".into();
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["fhd"]);
    filters.resolution = "3840×2160".into();
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["uhd"]);
    filters.resolution = String::new();
    assert_eq!(filter_sort(&catalog, &filters).len(), 4);
}

#[test]
fn resolution_ranges_are_orientation_specific() {
    let catalog = with_items(vec![
        sized("fhd", 1920, 1080),
        sized("portrait", 1440, 2560),
        sized("ultrawide", 3440, 1440),
        sized("uhd", 3840, 2160),
    ]);
    let filters = Filters {
        resolution: "wide:1920x1080..2560x1440".into(),
        sort: "date".into(),
        ..Filters::default()
    };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["fhd"]);

    let filters = Filters {
        resolution: "tall:1080x1920..1440x2560".into(),
        sort: "date".into(),
        ..Filters::default()
    };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["portrait"]);

    let filters = Filters {
        resolution: "wide:3840x2160..".into(),
        sort: "date".into(),
        ..Filters::default()
    };
    assert_eq!(keys(&catalog, &filter_sort(&catalog, &filters)), vec!["uhd"]);
}

#[test]
fn res_sort_area_width() {
    let catalog = with_items(vec![
        sized("hd", 1920, 1080),
        sized("uhd", 3840, 2160),
        sized("wide", 3440, 1440),
        sized("tallhd", 1080, 1920),
    ]);
    let filters = Filters { sort: "res".into(), ..Filters::default() };
    assert_eq!(
        keys(&catalog, &filter_sort(&catalog, &filters)),
        vec!["uhd", "wide", "hd", "tallhd"]
    );
}
