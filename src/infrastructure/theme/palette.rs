use serde_json::Value;

use crate::contracts::picker::{PaletteSpec, palette_keys};

pub fn load_palette(cache_dir: &str) -> PaletteSpec {
    let path = format!("{cache_dir}/colors.json");
    let Ok(text) = std::fs::read_to_string(path) else {
        return PaletteSpec::default();
    };
    serde_json::from_str::<Value>(&text)
        .map_or_else(|_| PaletteSpec::default(), |value| decode_palette(&value))
}

pub fn decode_palette(value: &Value) -> PaletteSpec {
    let pick = |keys: &[&str]| {
        keys.iter().find_map(|key| value.get(*key).and_then(Value::as_str).map(String::from))
    };
    PaletteSpec {
        primary: pick(&["primary"]),
        primary_text: pick(&["primaryText", "on_primary"]),
        surface: pick(&["surface"]),
        surface_text: pick(&["surfaceText", "on_surface"]),
        surface_variant: pick(&["surfaceVariant", "surface_variant"]),
        surface_container: pick(&["surfaceContainer", "surface_container"]),
        background: pick(&["background"]),
        outline: pick(&["outline"]),
        tertiary: pick(&["tertiary"]),
        secondary: pick(palette_keys::SECONDARY),
        secondary_text: pick(palette_keys::SECONDARY_TEXT),
        tertiary_text: pick(palette_keys::TERTIARY_TEXT),
        error: pick(palette_keys::ERROR),
        error_text: pick(palette_keys::ERROR_TEXT),
        surface_variant_text: pick(palette_keys::SURFACE_VARIANT_TEXT),
        shadow: pick(palette_keys::SHADOW),
        hover: pick(palette_keys::HOVER),
        hover_text: pick(palette_keys::HOVER_TEXT),
    }
}
