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
