#![cfg(test)]

use super::*;
use serde_json::json;

fn palette(value: &serde_json::Value) -> Palette {
    Palette::from_spec(&crate::infrastructure::theme::decode_palette(value))
}

#[test]
fn parse_hex_shapes() {
    let color = parse_hex("#112233").unwrap();
    assert!((color.r - 0x11 as f32 / 255.0).abs() < 1e-6);
    assert!((color.g - 0x22 as f32 / 255.0).abs() < 1e-6);
    assert!((color.b - 0x33 as f32 / 255.0).abs() < 1e-6);
    assert!((color.a - 1.0).abs() < 1e-6);
    let color = parse_hex("#80ff0000").unwrap();
    assert!((color.r - 1.0).abs() < 1e-6, "AARRGGBB");
    assert!(color.g.abs() < 1e-6 && color.b.abs() < 1e-6);
    assert!((color.a - 0x80 as f32 / 255.0).abs() < 0.01);
    assert!(parse_hex("#fff").is_none());
    assert!(parse_hex("#12345").is_none());
    assert!(parse_hex("#1234567").is_none());
    assert_eq!(parse_hex("112233"), parse_hex("#112233"));
    assert!(parse_hex("#gg0000").is_none());
}

#[test]
fn from_value_partial() {
    let pal = palette(&json!({"on_surface": "#112233"}));
    let hex = parse_hex("#112233").unwrap();
    assert!((pal.surface_text.r - hex.r).abs() < 1e-6);
    let base = Palette::default();
    assert_eq!(format!("{:?}", pal.primary), format!("{:?}", base.primary));
    assert_eq!(format!("{:?}", pal.outline), format!("{:?}", base.outline));
    let pal = palette(&json!({
        "primaryText": "#000000",
        "on_primary": "#ffffff",
    }));
    assert!(pal.primary_text.r.abs() < 1e-6);
    let pal = palette(&json!({"surface": "not-a-color"}));
    assert_eq!(format!("{:?}", pal.surface), format!("{:?}", base.surface));
}

#[test]
fn lerp_endpoints_midpoint() {
    let black = palette(&json!({ "primary": "#000000" }));
    let white = palette(&json!({ "primary": "#ffffff" }));
    assert!((black.lerp(&white, 0.0).primary.r - black.primary.r).abs() < 1e-6);
    assert!((black.lerp(&white, 1.0).primary.r - white.primary.r).abs() < 1e-6);
    assert!((black.lerp(&white, 0.5).primary.r - 0.5).abs() < 0.02);
    assert!((black.lerp(&white, 2.0).primary.r - 1.0).abs() < 1e-6);
}

#[test]
fn noctalia_snake_keys() {
    let base = Palette::default();
    let pal = palette(&json!({
        "surface_variant": "#010101",
        "surface_container": "#020202",
    }));
    assert_ne!(format!("{:?}", pal.surface_variant), format!("{:?}", base.surface_variant));
    assert_ne!(format!("{:?}", pal.surface_container), format!("{:?}", base.surface_container));
    let camel = palette(&json!({ "surfaceVariant": "#030303" }));
    assert_eq!(
        format!("{:?}", camel.surface_variant),
        format!("{:?}", palette(&json!({"surface_variant": "#030303"})).surface_variant)
    );
}

#[test]
fn role_descriptor_order() {
    for (index, descriptor) in ROLES.into_iter().enumerate() {
        assert_eq!(descriptor.index, index);
        assert!(!descriptor.name.is_empty());
        assert!(!descriptor.description.is_empty());
    }
}

fn colours(pal: &Palette) -> [iced::Color; 18] {
    [
        pal.primary,
        pal.primary_text,
        pal.surface,
        pal.surface_text,
        pal.surface_variant,
        pal.surface_container,
        pal.background,
        pal.outline,
        pal.tertiary,
        pal.secondary,
        pal.secondary_text,
        pal.tertiary_text,
        pal.error,
        pal.error_text,
        pal.surface_variant_text,
        pal.shadow,
        pal.hover,
        pal.hover_text,
    ]
}

#[test]
fn studio_colour_pairs() {
    let mut studio = Palette::default().studio();
    studio.second = skwd_palette::Rgb(20, 80, 160);
    let pal = Palette::from_studio(&studio);
    assert_eq!(pal.secondary, rgb_color(studio.second));
    assert_eq!(pal.primary_text, rgb_color(studio.on_accent));
    assert_eq!(pal.error, rgb_color(studio.third));
    assert_eq!(pal.error_text, pal.background);
    assert_eq!(pal.hover, pal.surface_variant);
    assert_eq!(pal.hover_text, pal.surface_text);
}

#[test]
fn complete_palette_interpolates() {
    let keys = [
        "primary",
        "primaryText",
        "surface",
        "surfaceText",
        "surfaceVariant",
        "surfaceContainer",
        "background",
        "outline",
        "tertiary",
        "secondary",
        "secondaryText",
        "tertiaryText",
        "error",
        "errorText",
        "surfaceVariantText",
        "shadow",
        "hover",
        "hoverText",
    ];
    let values: serde_json::Map<_, _> = keys
        .into_iter()
        .enumerate()
        .map(|(index, key)| {
            (key.to_owned(), json!(format!("#{:06x}", 0x0010_2030 + index * 0x0001_0101)))
        })
        .collect();
    let decoded = palette(&values.clone().into());
    for (key, color) in keys.into_iter().zip(colours(&decoded)) {
        assert_eq!(color, parse_hex(values[key].as_str().unwrap()).unwrap(), "{key}");
    }
    let other = Palette::default();
    for (actual, expected) in
        colours(&decoded.lerp(&other, -1.0)).into_iter().zip(colours(&decoded))
    {
        for (channel, target) in [actual.r, actual.g, actual.b, actual.a]
            .into_iter()
            .zip([expected.r, expected.g, expected.b, expected.a])
        {
            assert!((channel - target).abs() < 1e-6);
        }
    }
    for (actual, expected) in colours(&decoded.lerp(&other, 2.0)).into_iter().zip(colours(&other)) {
        for (channel, target) in [actual.r, actual.g, actual.b, actual.a]
            .into_iter()
            .zip([expected.r, expected.g, expected.b, expected.a])
        {
            assert!((channel - target).abs() < 1e-6);
        }
    }
    for ((start, end), actual) in
        colours(&decoded).into_iter().zip(colours(&other)).zip(colours(&decoded.lerp(&other, 0.5)))
    {
        assert_eq!(actual, crate::frontend::components::lerp_color(start, end, 0.5));
    }
}

#[test]
fn incomplete_palette_fallbacks() {
    let pal = palette(&json!({"tertiary": "#010203", "background": "#040506",
        "surfaceVariant": "#070809", "surfaceText": "#101112", "primaryText": "#131415",
        "error": "invalid", "hover": "invalid"}));
    assert_eq!(pal.error, pal.tertiary);
    assert_eq!(pal.error_text, pal.background);
    assert_eq!(pal.hover, pal.surface_variant);
    assert_eq!(pal.hover_text, pal.surface_text);
    assert_eq!(pal.surface_variant_text, pal.surface_text);
    assert_eq!(pal.secondary, pal.tertiary);
    assert_eq!(pal.secondary_text, pal.primary_text);
    assert_eq!(pal.tertiary_text, pal.primary_text);
    assert_eq!(pal.shadow, iced::Color::BLACK);
}

#[test]
fn candidate_extended_roles() {
    let mut candidate = crate::domain::theme::Candidate::default();
    for (index, color) in candidate.colors.iter_mut().enumerate() {
        *color = format!("#{:06x}", 0x0010_2030 + index * 0x0001_0101);
    }
    let pal = Palette::from_candidate(&candidate);
    for (key, actual) in [
        ("secondary", pal.secondary),
        ("on_secondary", pal.secondary_text),
        ("on_tertiary", pal.tertiary_text),
        ("error", pal.error),
        ("on_error", pal.error_text),
        ("on_surface_variant", pal.surface_variant_text),
        ("shadow", pal.shadow),
    ] {
        let index = skwd_palette::material::ROLE_KEYS.iter().position(|role| *role == key).unwrap();
        assert_eq!(actual, parse_hex(&candidate.colors[index]).unwrap(), "{key}");
    }
    assert_eq!(pal.hover, pal.surface_variant);
    assert_eq!(pal.hover_text, pal.surface_text);
}

#[test]
fn dark_chrome_keeps_roles() {
    let pal = palette(&json!({"secondaryText": "#010203", "tertiaryText": "#040506",
        "errorText": "#070809", "surfaceVariantText": "#101112", "hover": "#131415",
        "hoverText": "#161718", "shadow": "#192021"}));
    let dark = crate::frontend::effects::dark_chrome(&pal);
    assert_eq!(dark.secondary_text, pal.secondary_text);
    assert_eq!(dark.tertiary_text, pal.tertiary_text);
    assert_eq!(dark.error_text, pal.error_text);
    assert_eq!(dark.surface_variant_text, pal.surface_variant_text);
    assert_eq!(dark.hover, pal.hover);
    assert_eq!(dark.hover_text, pal.hover_text);
    assert_eq!(dark.shadow, pal.shadow);
}
