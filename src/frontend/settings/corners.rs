use crate::contracts::settings::SettingsSource;
use crate::contracts::settings::corners::{
    BOTTOM_LEFT, BOTTOM_RIGHT, CORNERS, CornerKeys, CornerStyle, TOP_LEFT, TOP_RIGHT,
};
use crate::i18n::tr;

pub(super) const STYLE_TITLE: &str = "settings-layout-corner-style";
pub(super) const SAME_TITLE: &str = "settings-layout-corners-same";
pub(super) const CELLS: [usize; CORNERS] = [TOP_LEFT, TOP_RIGHT, BOTTOM_LEFT, BOTTOM_RIGHT];

const STYLE_NAMES: [(CornerStyle, &str); CornerStyle::ALL.len()] = [
    (CornerStyle::Straight, "settings-layout-corner-straight"),
    (CornerStyle::Round, "settings-layout-corner-round"),
    (CornerStyle::Chamfered, "settings-layout-corner-chamfered"),
];

pub(super) struct Words {
    pub hint: &'static str,
    pub all: &'static str,
    pub corners: [&'static str; CORNERS],
}

const RADIUS: Words = Words {
    hint: "settings-layout-corners-hint",
    all: "settings-layout-corner-all-radius",
    corners: [
        "settings-layout-corner-tl-radius",
        "settings-layout-corner-tr-radius",
        "settings-layout-corner-br-radius",
        "settings-layout-corner-bl-radius",
    ],
};

const CUT: Words = Words {
    hint: "settings-layout-corners-hint-cut",
    all: "settings-layout-corner-all-cut",
    corners: [
        "settings-layout-corner-tl-cut",
        "settings-layout-corner-tr-cut",
        "settings-layout-corner-br-cut",
        "settings-layout-corner-bl-cut",
    ],
};

pub(super) fn style(cfg: &dyn SettingsSource, keys: &CornerKeys) -> CornerStyle {
    CornerStyle::from_key(&cfg.text(keys.style)).unwrap_or(CornerStyle::Straight)
}

pub(super) fn words(style: CornerStyle) -> &'static Words {
    if style == CornerStyle::Chamfered { &CUT } else { &RADIUS }
}

pub(super) fn options() -> [(&'static str, &'static str); CornerStyle::ALL.len()] {
    STYLE_NAMES.map(|(style, name)| (style.key(), tr(name)))
}
