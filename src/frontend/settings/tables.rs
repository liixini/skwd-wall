use crate::contracts::settings::{FolderKind, transitions, wallpaper_kind};
use crate::i18n::tr;

use super::SectionId;
pub(crate) use super::category;

pub const TABS: [&str; 10] = [
    category::LIBRARY,
    category::SOURCES,
    category::PICKER,
    category::DISPLAYS,
    category::PLAYBACK,
    category::TRANSITIONS,
    category::THEME,
    category::AUTOMATION,
    category::INTEGRATIONS,
    category::APP,
];

pub(super) const MOTION_SPEEDS: [&str; 3] = ["fast", "standard", "slow"];

pub(super) const LEGACY_ROUTES: [(&str, &str, &str); 6] = [
    ("search", category::PICKER, super::bar::SEARCH),
    ("filter", category::PICKER, super::bar::FILTER),
    ("position", category::PICKER, super::layout_place::PLACEMENT),
    ("motion", category::PICKER, super::layout_place::OPENING),
    ("performance", category::APP, super::system::POWER),
    ("language", category::APP, super::system::LANGUAGE),
];

pub fn canonical_category(tab: String) -> String {
    let route = LEGACY_ROUTES.iter().find(|(old, _, _)| *old == tab);
    route.map_or(tab, |(_, category, _)| category.to_string())
}

pub(super) fn category_label(tab: &str) -> &'static str {
    tr(tab_label_key(tab))
}

const PICKER_LABEL_KEY: &str = "settings-tab-picker";
const CATEGORY_LABEL_KEYS: [(&str, &str); TABS.len()] = [
    (category::LIBRARY, "settings-tab-library"),
    (category::SOURCES, "settings-tab-sources"),
    (category::PICKER, PICKER_LABEL_KEY),
    (category::DISPLAYS, "settings-tab-displays"),
    (category::PLAYBACK, "settings-tab-playback"),
    (category::TRANSITIONS, "settings-tab-transitions"),
    (category::THEME, "settings-tab-theme"),
    (category::AUTOMATION, "settings-tab-automation"),
    (category::INTEGRATIONS, "settings-tab-integrations"),
    (category::APP, "settings-tab-app"),
];

fn tab_label_key(tab: &str) -> &'static str {
    CATEGORY_LABEL_KEYS
        .iter()
        .find(|(category, _)| *category == tab)
        .map_or(PICKER_LABEL_KEY, |(_, key)| *key)
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

pub fn is_picker_layout_section(section: Option<SectionId>) -> bool {
    const LAYOUT_AND_PRESETS: SectionId =
        SectionId::new(category::PICKER, "settings-section-layout-presets");
    section == Some(LAYOUT_AND_PRESETS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSection {
    Folders(FolderKind),
    Profiles,
}

pub fn page_section(section: SectionId) -> Option<PageSection> {
    const FOLDERS: &str = "settings-section-folders";
    const EXCLUDED: &str = "settings-paths-excluded-card";
    const PROFILES: &str = "settings-profiles-card";
    if section.category != category::LIBRARY {
        return None;
    }
    match section.key {
        FOLDERS => Some(PageSection::Folders(FolderKind::Source)),
        EXCLUDED => Some(PageSection::Folders(FolderKind::Excluded)),
        PROFILES => Some(PageSection::Profiles),
        _ => None,
    }
}

pub fn is_transition_preview_section(section: Option<SectionId>) -> bool {
    const WALLPAPER_TRANSITIONS: SectionId =
        SectionId::new(category::TRANSITIONS, "settings-section-wallpaper-transitions");
    section == Some(WALLPAPER_TRANSITIONS)
}

pub(super) const POST_TYPES: [&str; 4] =
    ["all", wallpaper_kind::STATIC, wallpaper_kind::VIDEO, wallpaper_kind::WE];
pub(super) const FILL_MODES: [&str; 6] = ["fill", "fit", "stretch", "center", "tile", "span"];
pub(super) const ENGINES: [(&str, &str); 2] = [("skwd-paper", "Skwd-paper"), ("awww", "awww")];
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
pub const SHADER_FAMILIES: [(&str, &str, &str); transitions::FAMILIES.len()] = {
    use transitions::FAMILIES;
    let mut out = [("", "", ""); FAMILIES.len()];
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
pub const WEATHER_MODE_KEY: &str = "weather.mode";
pub const DRAWING_SCOPE_KEY: &str = "drawing.prepareScope";
pub const DRAWING_LIBRARY_SCOPE: &str = "library";
pub const WEATHER_REVIEW_KEY: &str = "weather.review";
pub const WEATHER_OFF: &str = "off";
pub const WEATHER_MODES: [&str; 4] = [
    WEATHER_OFF,
    crate::contracts::settings::keys::weather::SOURCE_OUTDOORS,
    crate::contracts::settings::keys::weather::SOURCE_PLAYLISTS,
    crate::contracts::settings::keys::weather::SOURCE_SEMANTIC,
];
pub const WEATHER_CONDITIONS: [(&str, &str); 7] = [
    ("clear", "settings-automation-condition-clear"),
    ("cloudy", "settings-automation-condition-cloudy"),
    ("rainy", "settings-automation-condition-rainy"),
    ("snowy", "settings-automation-condition-snowy"),
    ("stormy", "settings-automation-condition-stormy"),
    ("foggy", "settings-automation-condition-foggy"),
    ("windy", "settings-automation-condition-windy"),
];

pub(super) const STEAM_METHODS: [&str; 2] = ["steam", "steamcmd"];

pub fn family_default(current: &str, family: &str) -> String {
    if shader_family(current) == family {
        return current.to_string();
    }
    shaders_in_family(family)
        .first()
        .map_or_else(|| String::from("random"), |name| (*name).to_string())
}

pub(crate) const NIRI_SNIPPET: &str = "layer-rule {\n    match namespace=\"^skwd-paper-backdrop$\"\n    place-within-backdrop true\n}\n\nlayer-rule {\n    match namespace=\"^skwd-paper-stationary$\"\n    place-within-backdrop true\n}\n\nlayout {\n    background-color \"transparent\"\n}\n\noverview {\n    workspace-shadow { off; }\n}";
