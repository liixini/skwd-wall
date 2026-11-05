use serde_json::{Value, json};

use crate::contracts::picker::palette_keys;
use crate::domain::theme::{Candidate, HoverColors, ThemeRole};

pub fn profile_in_use(root: &serde_json::Value, name: &str) -> bool {
    skwd_config::get(root, skwd_config::keys::theme::APP_PROFILES)
        .and_then(serde_json::Value::as_object)
        .is_some_and(|profiles| profiles.values().any(|profile| profile.as_str() == Some(name)))
        || skwd_config::get(root, skwd_config::keys::integrations::LIST)
            .and_then(serde_json::Value::as_array)
            .is_some_and(|entries| {
                entries.iter().any(|entry| {
                    entry
                        .get(skwd_config::keys::integrations::PROFILE)
                        .and_then(serde_json::Value::as_str)
                        == Some(name)
                })
            })
}

const ROLE_KEYS: [(&str, ThemeRole); ThemeRole::ALL.len()] = [
    ("primary", ThemeRole::Primary),
    ("primaryText", ThemeRole::PrimaryText),
    ("tertiary", ThemeRole::Tertiary),
    ("surface", ThemeRole::Surface),
    ("surfaceText", ThemeRole::SurfaceText),
    ("surfaceVariant", ThemeRole::SurfaceVariant),
    ("surfaceContainer", ThemeRole::SurfaceContainer),
    ("background", ThemeRole::Background),
    ("outline", ThemeRole::Outline),
];
const SCHEME_VERSION_KEY: &str = "_schemeVersion";
const SCHEME_VERSION: u64 = 1;
const DARK_MODE: &str = "dark";
const LIGHT_MODE: &str = "light";
const COLORS_KEY: &str = "colors";
const COLOR_KEY: &str = "color";

pub fn decode_candidate(value: &Value) -> Candidate {
    let dark = value
        .get("_scheme")
        .and_then(|scheme| scheme.get("is_dark_mode"))
        .and_then(Value::as_bool)
        .unwrap_or_else(|| {
            value
                .get("background")
                .and_then(Value::as_str)
                .and_then(skwd_palette::parse_hex)
                .is_none_or(|color| color.lum() < 128.0)
        });
    decode_candidate_variant(value, dark)
}

pub fn decode_candidate_variant(value: &Value, dark: bool) -> Candidate {
    let Some(doc) = skwd_palette::material::from_palette(value, dark, "tonal-spot") else {
        return Candidate::default();
    };
    let hover = |dark| {
        let mode = if dark { DARK_MODE } else { LIGHT_MODE };
        Some(HoverColors {
            fill: skwd_palette::material::role(&doc, palette_keys::HOVER[0], mode)
                .and_then(|color| skwd_palette::material::parse_seed(&color))?,
            text: skwd_palette::material::role(&doc, palette_keys::HOVER_TEXT[1], mode)
                .and_then(|color| skwd_palette::material::parse_seed(&color))?,
        })
    };
    Candidate {
        colors: skwd_palette::material::colors(&doc, dark),
        alternate: skwd_palette::material::colors(&doc, !dark),
        dark,
        hover: hover(dark),
        alternate_hover: hover(!dark),
    }
}

pub fn encode_candidate(candidate: &Candidate, include_aliases: bool) -> Value {
    let mut map = serde_json::Map::new();
    for (key, role) in ROLE_KEYS {
        map.insert(key.to_string(), json!(candidate.colors[role.index()]));
    }
    if include_aliases {
        map.insert(
            "on_primary".to_string(),
            json!(candidate.colors[ThemeRole::PrimaryText.index()]),
        );
        map.insert(
            "on_surface".to_string(),
            json!(candidate.colors[ThemeRole::SurfaceText.index()]),
        );
    }
    let (dark, light) = if candidate.dark {
        (&candidate.colors, &candidate.alternate)
    } else {
        (&candidate.alternate, &candidate.colors)
    };
    let mut document = skwd_palette::material::from_colors(dark, light, candidate.dark);
    let (dark_hover, light_hover) = if candidate.dark {
        (&candidate.hover, &candidate.alternate_hover)
    } else {
        (&candidate.alternate_hover, &candidate.hover)
    };
    for (mode, hover) in [(DARK_MODE, dark_hover), (LIGHT_MODE, light_hover)] {
        if let Some(hover) = hover {
            document[COLORS_KEY][palette_keys::HOVER[0]][mode][COLOR_KEY] = json!(hover.fill);
            document[COLORS_KEY][palette_keys::HOVER_TEXT[1]][mode][COLOR_KEY] = json!(hover.text);
        }
    }
    skwd_palette::material::select_mode(&mut document, candidate.dark);
    if let Some(hover) = &candidate.hover {
        map.insert(palette_keys::HOVER[0].to_string(), json!(hover.fill));
        map.insert(palette_keys::HOVER_TEXT[0].to_string(), json!(hover.text));
    }
    map.insert("_scheme".to_string(), document);
    map.insert(SCHEME_VERSION_KEY.to_string(), json!(SCHEME_VERSION));
    Value::Object(map)
}

pub fn upsert_saved(themes: &[Value], name: &str, candidate: &Candidate) -> Vec<Value> {
    let mut output = themes.to_vec();
    let mut entry = encode_candidate(candidate, false);
    if let Some(map) = entry.as_object_mut() {
        map.insert("name".to_string(), json!(name));
    }
    match output.iter().position(|theme| saved_name(theme) == Some(name)) {
        Some(position) => output[position] = entry,
        None => output.push(entry),
    }
    output
}

pub fn remove_saved(themes: &[Value], name: &str) -> Vec<Value> {
    themes.iter().filter(|theme| saved_name(theme) != Some(name)).cloned().collect()
}

pub fn saved_names(themes: &[Value]) -> Vec<String> {
    themes
        .iter()
        .filter_map(saved_name)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn saved_palettes(themes: &[Value], dark: Option<bool>) -> Vec<(String, Candidate)> {
    themes
        .iter()
        .filter_map(|theme| {
            let name = saved_name(theme)?.trim();
            (!name.is_empty()).then(|| {
                let variant = dark.filter(|_| {
                    theme.get(SCHEME_VERSION_KEY).and_then(Value::as_u64) == Some(SCHEME_VERSION)
                });
                let candidate = variant.map_or_else(
                    || decode_candidate(theme),
                    |dark| decode_candidate_variant(theme, dark),
                );
                (name.to_string(), candidate)
            })
        })
        .collect()
}

pub fn find_saved(themes: &[Value], name: &str) -> Option<Candidate> {
    themes.iter().find(|theme| saved_name(theme) == Some(name)).map(decode_candidate)
}

fn saved_name(theme: &Value) -> Option<&str> {
    theme.get("name").and_then(Value::as_str)
}
