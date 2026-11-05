#![cfg(test)]

use super::saved::decode_candidate;
use serde_json::json;

use crate::contracts::picker::PaletteSpec;

use super::{
    decode_palette, encode_candidate, find_saved, load_palette, remove_saved, saved_names,
    saved_palettes, upsert_saved,
};

#[test]
fn palette_alias_precedence() {
    let spec = decode_palette(&json!({
        "primaryText": "#000000",
        "on_primary": "#ffffff",
        "surface_variant": "#010101",
        "surfaceContainer": "#020202"
    }));
    assert_eq!(spec.primary_text.as_deref(), Some("#000000"));
    assert_eq!(spec.surface_variant.as_deref(), Some("#010101"));
    assert_eq!(spec.surface_container.as_deref(), Some("#020202"));
    assert!(spec.primary.is_none());
}

#[test]
fn loader_falls_back() {
    let root = std::env::temp_dir().join(format!("skwd-palette-loader-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    assert_eq!(load_palette(root.to_str().unwrap()), PaletteSpec::default());
    std::fs::write(root.join("colors.json"), "not json").unwrap();
    assert_eq!(load_palette(root.to_str().unwrap()), PaletteSpec::default());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn candidate_codec_aliases() {
    let candidate = crate::domain::theme::Candidate::from_preset("nord").unwrap();
    let value = encode_candidate(&candidate, true);
    assert_eq!(value["on_primary"], value["primaryText"]);
    assert_eq!(value["on_surface"], value["surfaceText"]);
    assert_eq!(decode_candidate(&value), candidate);
}

#[test]
fn saved_theme_storage() {
    let nord = crate::domain::theme::Candidate::from_preset("nord").unwrap();
    let dracula = crate::domain::theme::Candidate::from_preset("dracula").unwrap();

    let one = upsert_saved(&[], "Mine", &nord);
    assert_eq!(saved_names(&one), vec!["Mine"]);
    let replaced = upsert_saved(&one, "Mine", &dracula);
    assert_eq!(replaced.len(), 1);
    assert_eq!(find_saved(&replaced, "Mine"), Some(dracula.clone()));

    let two = upsert_saved(&replaced, "Other", &nord);
    assert_eq!(
        saved_palettes(&two, None),
        vec![("Mine".to_string(), dracula), ("Other".to_string(), nord.clone())]
    );
    let remaining = remove_saved(&two, "Mine");
    assert_eq!(saved_names(&remaining), vec!["Other"]);
    assert_eq!(find_saved(&remaining, "missing"), None);
}

#[test]
fn full_scheme_round_trip_keeps_every_role_in_both_variants() {
    let mut candidate = crate::domain::theme::Candidate::from_seed("#84d1ce", true).unwrap();
    for index in 0..crate::domain::theme::THEME_ROLE_COUNT {
        candidate.colors[index] = format!("#{:06x}", 0x0012_3400 + index);
        candidate.alternate[index] = format!("#{:06x}", 0x00ab_cd00 + index);
    }
    for dark in [true, false] {
        candidate.set_dark(dark);
        let saved = upsert_saved(&[], "Full scheme", &candidate);
        assert_eq!(find_saved(&saved, "Full scheme"), Some(candidate.clone()));
        assert_eq!(saved[0]["_scheme"]["colors"].as_object().unwrap().len(), 50);
    }
}

#[test]
fn legacy_light_profile_keeps_its_original_colours() {
    let value = json!({"primary": "#112233", "surface": "#fafafa", "primaryText": "#445566"});
    let candidate = super::decode_candidate_variant(&value, false);
    assert!(!candidate.dark);
    assert_eq!(candidate.colors[0], "#112233");
    assert_eq!(candidate.colors[1], "#445566");
    assert_eq!(candidate.colors[3], "#fafafa");
    assert!(candidate.colors.iter().all(|hex| crate::domain::theme::hex_to_hsv(hex).is_some()));
}

#[test]
fn extended_text_aliases() {
    let aliases = ["on_secondary", "on_tertiary", "on_error", "on_surface_variant", "on_hover"];
    let camel = ["secondaryText", "tertiaryText", "errorText", "surfaceVariantText", "hoverText"];
    let mut values = serde_json::Map::new();
    for (index, alias) in aliases.into_iter().enumerate() {
        values.insert(alias.to_owned(), json!(format!("#{index:06x}")));
    }
    let spec = decode_palette(&values.clone().into());
    let fields = [
        spec.secondary_text,
        spec.tertiary_text,
        spec.error_text,
        spec.surface_variant_text,
        spec.hover_text,
    ];
    for (index, field) in fields.into_iter().enumerate() {
        assert_eq!(field, Some(format!("#{index:06x}")));
    }
    for key in camel {
        values.insert(key.to_owned(), json!("#abcdef"));
    }
    let spec = decode_palette(&values.into());
    for field in [
        spec.secondary_text,
        spec.tertiary_text,
        spec.error_text,
        spec.surface_variant_text,
        spec.hover_text,
    ] {
        assert_eq!(field.as_deref(), Some("#abcdef"));
    }
}

#[test]
fn material_outline_stays_literal() {
    let spec = decode_palette(&json!({"outline": "#010203", "outline_variant": "#040506"}));
    assert_eq!(spec.outline.as_deref(), Some("#010203"));
}

#[test]
fn saved_hover_variants() {
    let candidate = crate::domain::theme::Candidate::from_seed("#84d1ce", true).unwrap();
    let mut saved = encode_candidate(&candidate, true);
    for (mode, fill, text) in [("dark", "#123456", "#fedcba"), ("light", "#abcdef", "#102030")] {
        saved["_scheme"]["colors"]["hover"][mode]["color"] = json!(fill);
        saved["_scheme"]["colors"]["on_hover"][mode]["color"] = json!(text);
    }
    let mut candidate = super::decode_candidate_variant(&saved, true);
    let original = candidate.clone();
    for (dark, fill, text) in [(true, "#123456", "#fedcba"), (false, "#abcdef", "#102030")] {
        candidate.set_dark(dark);
        let palette = crate::frontend::theme::Palette::from_candidate(&candidate);
        assert_eq!(palette.hover, crate::frontend::theme::parse_hex(fill).unwrap());
        assert_eq!(palette.hover_text, crate::frontend::theme::parse_hex(text).unwrap());
        let saved = encode_candidate(&candidate, true);
        assert_eq!(saved["hover"], fill);
        assert_eq!(saved["hoverText"], text);
        assert_eq!(super::decode_candidate_variant(&saved, dark), candidate);
        assert!(candidate.same_colors(&original));
    }
}
