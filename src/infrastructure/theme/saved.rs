use serde_json::{Value, json};

use crate::domain::theme::{Candidate, ThemeRole};

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
    Candidate {
        colors: skwd_palette::material::colors(&doc, dark),
        alternate: skwd_palette::material::colors(&doc, !dark),
        dark,
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
    map.insert(
        "_scheme".to_string(),
        skwd_palette::material::from_colors(dark, light, candidate.dark),
    );
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
