use crate::contracts::settings::{SettingsSource, transitions, wallpaper_kind};
use crate::i18n::tr;

pub const TABS: [&str; 14] = [
    "picker",
    "filter",
    "position",
    "displays",
    "motion",
    "playback",
    "performance",
    "library",
    "sources",
    "search",
    "automation",
    "theme",
    "integrations",
    "language",
];

pub(super) const MOTION_SPEEDS: [&str; 3] = ["fast", "standard", "slow"];

pub fn visible_tabs(_cfg: &dyn SettingsSource) -> Vec<(&'static str, &'static str)> {
    TABS.iter().map(|key| (*key, tr(tab_label_key(key)))).collect()
}

fn tab_label_key(tab: &str) -> &'static str {
    match tab {
        "filter" => "settings-tab-filter",
        "position" => "settings-tab-position",
        "displays" => "settings-tab-displays",
        "motion" => "settings-tab-motion",
        "playback" => "settings-tab-playback",
        "performance" => "settings-tab-performance",
        "library" => "settings-tab-library",
        "sources" => "settings-tab-sources",
        "search" => "settings-tab-search",
        "automation" => "settings-tab-automation",
        "theme" => "settings-tab-theme",
        "integrations" => "settings-tab-integrations",
        "language" => "settings-tab-language",
        _ => "settings-tab-picker",
    }
}

pub(super) fn motion_speed_options() -> [(&'static str, &'static str); 3] {
    MOTION_SPEEDS.map(|key| {
        let label = match key {
            "fast" => "settings-motion-fast",
            "slow" => "settings-motion-slow",
            _ => "settings-motion-standard",
        };
        (key, tr(label))
    })
}

pub fn is_picker_layout_section(tab: &str, section: usize) -> bool {
    const LAYOUT_AND_PRESETS: usize = 1;
    tab == "picker" && section == LAYOUT_AND_PRESETS
}

pub fn is_transition_preview_section(tab: &str, section: usize) -> bool {
    const WALLPAPER_TRANSITIONS: usize = 3;
    tab == "motion" && section == WALLPAPER_TRANSITIONS
}

pub(super) const MODES: [&str; 4] = ["slices", "hex", "wall", "sandy"];
pub(super) const POST_TYPES: [&str; 4] =
    ["all", wallpaper_kind::STATIC, wallpaper_kind::VIDEO, wallpaper_kind::WE];
pub(super) const FILL_MODES: [&str; 6] = ["fill", "fit", "stretch", "center", "tile", "span"];
pub(super) const ENGINES: [(&str, &str); 2] = [("skwd-paper", "Skwd-paper"), ("awww", "awww")];
pub(super) const VIDEO_ENGINES: [&str; 2] = ["vulkan", "tinier"];
pub(super) const AWWW_TYPES: [&str; 14] = [
    "none", "simple", "fade", "wipe", "wave", "grow", "center", "outer", "left", "right", "top",
    "bottom", "any", "random",
];
pub(super) const AWWW_FILTERS: [(&str, &str); 5] = [
    ("Nearest", "Nearest"),
    ("Bilinear", "Bilinear"),
    ("CatmullRom", "CatmullRom"),
    ("Mitchell", "Mitchell"),
    ("Lanczos3", "Lanczos3"),
];
pub const SHADERS: [&str; transitions::TRANSITIONS.len()] = {
    let mut out = [""; transitions::TRANSITIONS.len()];
    let mut i = 0;
    while i < transitions::TRANSITIONS.len() {
        out[i] = transitions::TRANSITIONS[i].key;
        i += 1;
    }
    out
};
pub const SHADER_FAMILIES: [(&str, &str, &str); 6] = {
    use transitions::FAMILIES;
    let mut out = [("", "", ""); 6];
    let mut i = 0;
    while i < FAMILIES.len() {
        out[i] = (FAMILIES[i].key(), FAMILIES[i].label(), FAMILIES[i].desc());
        i += 1;
    }
    out
};

pub fn shader_family(shader: &str) -> &'static str {
    transitions::family_of(shader).key()
}

pub fn shaders_in_family(family: &str) -> Vec<&'static str> {
    SHADERS.iter().copied().filter(|name| shader_family(name) == family).collect()
}

pub const SHADER_FAMILY_KEY: &str = "transition.family";

pub fn family_default(current: &str, family: &str) -> String {
    if shader_family(current) == family {
        return current.to_string();
    }
    shaders_in_family(family)
        .first()
        .map_or_else(|| String::from("random"), |name| (*name).to_string())
}

pub(super) const NIRI_SNIPPET: &str = "layer-rule {\n    match namespace=\"^skwd-paper-backdrop$\"\n    place-within-backdrop true\n}";
