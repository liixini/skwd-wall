use skwd_palette::studio::{Area, Role, Shape, StudioPalette, Tuning, from_theme};

use super::*;

fn palette(base: &str) -> StudioPalette {
    let mut palette = from_theme(&skwd_palette::preset("nord").expect("nord"));
    palette.base = skwd_palette::parse_hex(base).expect("hex");
    palette
}

fn snapshot() -> Snapshot {
    Snapshot {
        image: "/w/red.webp".into(),
        seed: "#d77e52".into(),
        wheel: vec![0.0; 36],
        methods: vec![Variant {
            key: "subject".into(),
            dark: palette("#101010"),
            light: palette("#fafafa"),
            shape: Shape::Plain,
        }],
        materials: vec![Variant {
            key: "content".into(),
            dark: palette("#111111"),
            light: palette("#f0f0f0"),
            shape: Shape::Plain,
        }],
        dark: None,
    }
}

fn studio(source: &str) -> Studio {
    let mut studio = Studio::new(Source::parse(source), Mode::Dark, Tuning::default());
    studio.snapshot = Some(snapshot());
    studio
}

#[test]
fn groups_follow_source() {
    assert_eq!(Group::of(&Source::parse("theme:nord")), Group::Themes);
    assert_eq!(Group::of(&Source::parse("material:content")), Group::Matugen);
    assert_eq!(Group::of(&Source::parse("subject")), Group::Image);
}

#[test]
fn method_follows_mode() {
    let mut studio = studio("subject");
    assert_eq!(studio.palette().expect("dark").base.hex(), "#101010");
    studio.set_mode(Mode::Light);
    assert_eq!(studio.palette().expect("light").base.hex(), "#fafafa");
}

#[test]
fn themes_switch_twin() {
    let mut studio = studio("theme:catppuccin-frappe");
    studio.set_mode(Mode::Light);
    assert_eq!(studio.source, Source::Theme("catppuccin-latte".into()));
    let mut lone = self::studio("theme:nord");
    lone.set_mode(Mode::Light);
    assert_eq!(lone.source, Source::Theme("nord".into()));
}

#[test]
fn material_uses_seed() {
    let studio = studio("material:content");
    assert!(studio.palette().is_some());
    let mut empty = studio.clone();
    empty.snapshot = None;
    assert!(empty.palette().is_none());
}

#[test]
fn tuning_applies() {
    let mut studio = studio("subject");
    let plain = studio.palette().expect("plain");
    studio.tuning.shades[Role::Accent.index()] = 2;
    assert_ne!(studio.palette().expect("tuned").accent, plain.accent);
}

#[test]
fn catalogue_lists_groups() {
    let studio = studio("subject");
    assert_eq!(studio.catalogue(Group::Image), [Source::Method("subject".into()), area()]);
    assert!(studio.catalogue(Group::Theory).is_empty());
    assert_eq!(studio.catalogue(Group::Themes).len(), 30);
    assert_eq!(studio.catalogue(Group::Matugen).len(), 10);
    assert!(!studio.order().contains(&area()));
}

#[test]
fn groups_split_families() {
    assert_eq!(Group::of(&Source::parse("harmonic")), Group::Theory);
    assert_eq!(Group::of(&Source::parse("pigment")), Group::Image);
    assert_eq!(Group::of(&area()), Group::Image);
}

#[test]
fn twin_returns_to_variant() {
    let mut studio = studio("theme:catppuccin-macchiato");
    studio.set_mode(Mode::Light);
    assert_eq!(studio.source, Source::Theme("catppuccin-latte".into()));
    studio.set_mode(Mode::Dark);
    assert_eq!(studio.source, Source::Theme("catppuccin-macchiato".into()));
}

#[test]
fn metals_stay_listed() {
    let mut studio = studio("theme:copper");
    studio.set_mode(Mode::Light);
    assert_eq!(studio.source, Source::Theme("copper".into()));
    assert!(!studio.palette().expect("copper").is_dark());
    assert_eq!(fixed_theme("copper"), None);
    assert_eq!(fixed_theme("nord"), Some(true));
}

#[test]
fn material_reads_snapshot() {
    let studio = studio("material:content");
    assert_eq!(studio.palette().expect("content").base.hex(), "#111111");
    assert!(studio.swatch(&Source::parse("material:smart")).is_none());
}

fn pixels() -> Pixels {
    let (width, height) = (16, 10);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for index in 0..width * height {
        let red = if index % width < width / 2 { 220 } else { 30 };
        rgba.extend_from_slice(&[red, 40, 60, 255]);
    }
    Pixels { rgba, width, height, aspect: 1.6 }
}

#[test]
fn area_reads_box() {
    let picked = Picked::read(&pixels(), Area { left: 0.0, top: 0.0, right: 0.5, bottom: 1.0 })
        .expect("box");
    let mut studio = studio("area");
    assert!(studio.palette().is_none());
    studio.area = Some(picked);
    assert!(studio.palette().is_some());
    assert!(studio.order().contains(&area()));
    assert_eq!(studio.wheel().len(), 36);
}

#[test]
fn cover_maps_both_ways() {
    let frame = Frame::cover(2.0);
    let shown = Area { left: 0.25, top: 0.25, right: 0.75, bottom: 0.75 };
    let image = frame.to_image(shown);
    assert!(image.left > shown.left && image.right < shown.right);
    let back = frame.to_preview(image);
    assert!((back.left - shown.left).abs() < 1e-5);
    assert!((back.bottom - shown.bottom).abs() < 1e-5);
}

#[test]
fn saved_keeps_three() {
    let entry = |name: &str| Saved {
        name: name.into(),
        source: area(),
        area: Some(Area { left: 0.1, top: 0.2, right: 0.6, bottom: 0.9 }),
        tuning: Tuning::default(),
    };
    let mut list = Vec::new();
    for name in ["a", "b", "c", "d"] {
        list = store(&list, entry(name));
    }
    list = store(&list, entry("b"));
    assert_eq!(list.len(), 4);
    let names: Vec<&str> = shown(&list).iter().map(|saved| saved.name.as_str()).collect();
    assert_eq!(names, ["c", "d", "b"]);
}
