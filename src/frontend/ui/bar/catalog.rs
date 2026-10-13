use crate::contracts::settings::wallpaper_kind::{STATIC, VIDEO, WE};
use crate::domain::library::filter::ResolutionPreset;
use crate::i18n::tr;

pub const TYPES: [&str; 4] = ["", STATIC, VIDEO, WE];

pub const SORTS: [(&str, &str); 7] = [
    ("date", "\u{f00f0}"),
    ("recent", "\u{f02da}"),
    ("color", "\u{f03d8}"),
    ("pop", "\u{f0238}"),
    ("richness", "\u{f0b74}"),
    ("minimalist", "\u{f0764}"),
    ("res", "\u{f0a24}"),
];

pub const ORIENTS: [&str; 3] = ["", "landscape", "portrait"];

pub fn type_label_key(key: &str) -> &'static str {
    match key {
        STATIC => "filter-bar-type-pic",
        VIDEO => "filter-bar-type-vid",
        WE => "filter-bar-type-we",
        _ => "filter-bar-type-all",
    }
}

pub fn sort_label_key(mode: &str) -> &'static str {
    match mode {
        "date" => "filter-bar-sort-date",
        "recent" => "filter-bar-sort-recent",
        "color" => "filter-bar-sort-color",
        "pop" => "filter-bar-sort-pop",
        "richness" => "filter-bar-sort-richness",
        "minimalist" => "filter-bar-sort-minimalist",
        _ => "filter-bar-sort-res",
    }
}

pub fn cycle_key<'a>(keys: &[&'a str], current: &str, backwards: bool) -> Option<&'a str> {
    let len = keys.len();
    if len == 0 {
        return None;
    }
    let next = match keys.iter().position(|key| *key == current) {
        Some(index) if backwards => (index + len - 1) % len,
        Some(index) => (index + 1) % len,
        None if backwards => len - 1,
        None => 0,
    };
    Some(keys[next])
}

pub fn next_orient(current: &str) -> &'static str {
    let index = ORIENTS.iter().position(|key| *key == current).unwrap_or(0);
    ORIENTS[(index + 1) % ORIENTS.len()]
}

pub fn orient_label(current: &str) -> &'static str {
    match ORIENTS.iter().copied().find(|key| *key == current).unwrap_or("") {
        "landscape" => tr("filter-bar-orient-wide"),
        "portrait" => tr("filter-bar-orient-tall"),
        _ => tr("filter-bar-orient-any"),
    }
}

pub fn next_resolution(current: &str, presets: &[ResolutionPreset], shape: &str) -> String {
    let mut visible = presets.iter().filter(|preset| preset.matches_shape(shape));
    if current.is_empty() {
        return visible.next().map_or_else(String::new, ResolutionPreset::key);
    }
    let Some(active) = visible.position(|preset| preset.key() == current) else {
        return String::new();
    };
    presets
        .iter()
        .filter(|preset| preset.matches_shape(shape))
        .nth(active + 1)
        .map_or_else(String::new, ResolutionPreset::key)
}

pub fn resolution_label(current: &str, presets: &[ResolutionPreset]) -> String {
    if current.is_empty() {
        return tr("filter-bar-size").to_string();
    }
    presets
        .iter()
        .find(|preset| preset.key() == current)
        .map_or_else(|| current.to_uppercase(), |preset| preset.label.to_uppercase())
}

pub(crate) const ICON_FAV: &str = "\u{f02d1}";
pub(super) const ICON_RANDOM: &str = "\u{f076e}";
pub(super) const ICON_FOLDER: &str = "\u{f024b}";
pub(crate) const ICON_HIDDEN: &str = "\u{f05f9}";

pub const MENU_MAX_ROWS: usize = 12;
pub const MENU_ROW_H: f32 = 26.0;

pub const DROP_ARROW: &str = "\u{25be}";
pub const DROP_ARROW_UP: &str = "\u{25b4}";
