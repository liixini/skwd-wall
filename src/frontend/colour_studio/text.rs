use skwd_palette::studio::{Role, SHADE_RANGE};

use super::controls::{Knob, Row, TUNABLE, View};
use super::layout::Head;
use super::state::{ColourStudio, Load};
use crate::domain::studio::{Group, ModeChoice, Source, fixed_theme, is_area, listed_theme};
use crate::i18n::{tr, tr_args};

const SAVE_PLACEHOLDER: &str = "colour-studio-save-placeholder";
const SHADE_LIGHTER: &str = "colour-studio-shade-lighter";
const SHADE_DARKER: &str = "colour-studio-shade-darker";

const METHODS: [(&str, &str); 12] = [
    ("tonal", "colour-studio-method-tonal"),
    ("pigment", "colour-studio-method-pigment"),
    ("subject", "colour-studio-method-subject"),
    ("spatial", "colour-studio-method-spatial"),
    ("atmosphere", "colour-studio-method-atmosphere"),
    ("harmonic", "colour-studio-method-harmonic"),
    ("templates", "colour-studio-method-templates"),
    ("gamut", "colour-studio-method-gamut"),
    ("temperature", "colour-studio-method-temperature"),
    ("splittone", "colour-studio-method-splittone"),
    ("mood", "colour-studio-method-mood"),
    ("inkpaper", "colour-studio-method-inkpaper"),
];

const MATUGEN: [(&str, &str); 10] = [
    ("tonal-spot", "colour-studio-matugen-tonal-spot"),
    ("content", "colour-studio-matugen-content"),
    ("fidelity", "colour-studio-matugen-fidelity"),
    ("expressive", "colour-studio-matugen-expressive"),
    ("vibrant", "colour-studio-matugen-vibrant"),
    ("monochrome", "colour-studio-matugen-monochrome"),
    ("fruit-salad", "colour-studio-matugen-fruit-salad"),
    ("neutral", "colour-studio-matugen-neutral"),
    ("rainbow", "colour-studio-matugen-rainbow"),
    ("smart", "colour-studio-matugen-smart"),
];

const GROUPS: [(Group, &str); 4] = [
    (Group::Image, "colour-studio-group-image"),
    (Group::Theory, "colour-studio-group-theory"),
    (Group::Themes, "colour-studio-group-themes"),
    (Group::Matugen, "colour-studio-group-matugen"),
];

const VIEWS: [(View, &str); 5] = [
    (View::Desktop, "colour-studio-view-desktop"),
    (View::Browser, "colour-studio-view-browser"),
    (View::Chat, "colour-studio-view-chat"),
    (View::Music, "colour-studio-view-music"),
    (View::Code, "colour-studio-view-code"),
];

const ROLES: [(Role, &str); 7] = [
    (Role::Base, "colour-studio-role-base"),
    (Role::Panel, "colour-studio-role-panel"),
    (Role::Text, "colour-studio-role-text"),
    (Role::Quiet, "colour-studio-role-quiet"),
    (Role::Accent, "colour-studio-role-accent"),
    (Role::Second, "colour-studio-role-second"),
    (Role::Third, "colour-studio-role-third"),
];

pub const MODES: [(ModeChoice, &str); 3] = [
    (ModeChoice::Dark, "colour-studio-mode-dark"),
    (ModeChoice::Light, "colour-studio-mode-light"),
    (ModeChoice::ByTime, "colour-studio-mode-by-time"),
];

pub const ROW_KEYS: [(Row, &str, &str); 7] = [
    (Row::View, "colour-studio-row-view", "colour-studio-row-view-detail"),
    (Row::Warmth, "colour-studio-row-warmth", "colour-studio-row-warmth-detail"),
    (Row::Colour, "colour-studio-row-colour", "colour-studio-row-colour-detail"),
    (Row::Contrast, "colour-studio-row-contrast", "colour-studio-row-contrast-detail"),
    (Row::FineTune, "colour-studio-row-fine-tune", "colour-studio-row-fine-tune-detail"),
    (Row::Shade, "colour-studio-row-shade", "colour-studio-row-shade-detail"),
    (Row::Compare, "colour-studio-row-compare", "colour-studio-row-compare-detail"),
];

struct Lean {
    up: &'static str,
    down: &'static str,
    zero: &'static str,
    scale: f32,
    centre: f32,
    span: f32,
}

const WARMTH: Lean = Lean {
    up: "colour-studio-warmer",
    down: "colour-studio-cooler",
    zero: "colour-studio-neutral",
    scale: 100.0,
    centre: 0.0,
    span: 1.0,
};
const COLOUR: Lean = Lean {
    up: "colour-studio-richer",
    down: "colour-studio-softer",
    zero: "colour-studio-as-made",
    scale: 60.0,
    centre: 1.0,
    span: 0.6,
};
const CONTRAST: Lean = Lean {
    up: "colour-studio-higher",
    down: "colour-studio-lower",
    zero: "colour-studio-normal",
    scale: 100.0,
    centre: 0.0,
    span: 1.0,
};

fn lookup<T: PartialEq + Copy>(table: &[(T, &'static str)], item: T) -> &'static str {
    table.iter().find(|(key, _)| *key == item).map_or("", |(_, value)| *value)
}

fn named(table: &[(&str, &'static str)], key: &str) -> String {
    table
        .iter()
        .find(|(item, _)| *item == key)
        .map_or_else(|| key.to_string(), |(_, name)| tr(name).to_string())
}

pub fn group(group: Group) -> &'static str {
    tr(lookup(&GROUPS, group))
}

pub fn head(head: Head) -> &'static str {
    match head {
        Head::Saved => tr("colour-studio-group-saved"),
        Head::Group(group) => self::group(group),
    }
}

pub fn view(view: View) -> &'static str {
    tr(lookup(&VIEWS, view))
}

pub fn role(role: Role) -> &'static str {
    tr(lookup(&ROLES, role))
}

pub fn mode(mode: ModeChoice) -> &'static str {
    tr(lookup(&MODES, mode))
}

fn theme_label(name: &str) -> String {
    skwd_palette::PRESETS
        .iter()
        .find(|(key, _)| *key == name)
        .map_or_else(|| name.to_string(), |(_, label)| (*label).to_string())
}

pub fn source(source: &Source) -> String {
    match source {
        Source::Method(_) if is_area(source) => tr("colour-studio-area").to_string(),
        Source::Method(name) => named(&METHODS, name),
        Source::Theme(name) => theme_label(name),
        Source::Material(scheme) => named(&MATUGEN, scheme),
    }
}

pub fn kid(studio: &ColourStudio, source: &Source) -> String {
    if !is_area(source) {
        return self::source(source);
    }
    let drafting = studio.picking.as_ref().is_some_and(|pick| pick.draft.is_some());
    if studio.saved_area().is_some() || drafting {
        tr("colour-studio-area").to_string()
    } else {
        tr("colour-studio-area-pick-one").to_string()
    }
}

pub fn knob(knob: Knob, value: f32) -> String {
    let lean = match knob {
        Knob::Warmth => &WARMTH,
        Knob::Colour => &COLOUR,
        Knob::Contrast => &CONTRAST,
    };
    let share = (value - lean.centre) / lean.span;
    let amount = (share.abs() * lean.scale).round();
    if amount == 0.0 {
        return tr(lean.zero).to_string();
    }
    let (word, sign) = if share > 0.0 { (lean.up, '+') } else { (lean.down, '−') };
    format!("{} {sign}{amount}", tr(word))
}

pub fn shade(step: i8) -> String {
    match step {
        0 => tr("colour-studio-as-made").to_string(),
        lighter if lighter > 0 => tr_args!(SHADE_LIGHTER, steps => lighter.to_string()),
        darker => tr_args!(SHADE_DARKER, steps => (-darker).to_string()),
    }
}

pub fn values(row: Row) -> Vec<String> {
    let lean = |lean: &Lean| {
        let most = lean.scale.round();
        vec![
            tr(lean.zero).to_string(),
            format!("{} +{most}", tr(lean.up)),
            format!("{} −{most}", tr(lean.down)),
        ]
    };
    match row {
        Row::View => View::ALL.iter().map(|item| view(*item).to_string()).collect(),
        Row::FineTune => TUNABLE.iter().map(|item| role(*item).to_string()).collect(),
        Row::Warmth => lean(&WARMTH),
        Row::Colour => lean(&COLOUR),
        Row::Contrast => lean(&CONTRAST),
        Row::Shade => (SHADE_RANGE.0..=SHADE_RANGE.1).map(shade).collect(),
        Row::Compare => {
            vec![tr("colour-studio-on").to_string(), tr("colour-studio-off").to_string()]
        }
    }
}

pub fn value(studio: &ColourStudio, row: Row) -> String {
    match row {
        Row::View => view(studio.view).to_string(),
        Row::FineTune => role(studio.model.role).to_string(),
        Row::Warmth | Row::Colour | Row::Contrast => {
            Knob::of(row).map(|knob| self::knob(knob, studio.knob(knob))).unwrap_or_default()
        }
        Row::Shade => shade(studio.model.tuning.shades[studio.model.role.index()]),
        Row::Compare if studio.model.compare => tr("colour-studio-on").to_string(),
        Row::Compare => tr("colour-studio-off").to_string(),
    }
}

pub fn apply(undo: bool) -> &'static str {
    if undo { tr("colour-studio-undo") } else { tr("colour-studio-use") }
}

pub fn pick_tag(studio: &ColourStudio) -> String {
    if matches!(studio.decoded, Load::Failed) {
        return tr("colour-studio-pick-failed").to_string();
    }
    let drafting = studio.picking.as_ref().is_some_and(|pick| pick.draft.is_some());
    if studio.saved_area().is_some() && !drafting {
        tr("colour-studio-pick-again").to_string()
    } else {
        tr("colour-studio-pick-tag").to_string()
    }
}

pub fn modes_label(studio: &ColourStudio) -> String {
    let Source::Theme(name) = &studio.model.source else {
        return tr("colour-studio-modes").to_string();
    };
    let Some(dark) = fixed_theme(name) else {
        return tr("colour-studio-modes").to_string();
    };
    let twin = skwd_palette::relatives(name)
        .any(|kin| listed_theme(kin) && fixed_theme(kin).is_some_and(|other| other != dark));
    if twin {
        return tr("colour-studio-modes").to_string();
    }
    let key = if dark { "colour-studio-fixed-dark" } else { "colour-studio-fixed-light" };
    tr_args!(key, name => source(&studio.model.source))
}

pub fn wheel_note(studio: &ColourStudio) -> &'static str {
    if is_area(&studio.model.source) {
        tr("colour-studio-wheel-area")
    } else {
        tr("colour-studio-wheel")
    }
}

pub fn save_placeholder(studio: &ColourStudio) -> String {
    let name = source(&studio.model.source).to_lowercase();
    tr_args!(SAVE_PLACEHOLDER, wallpaper => studio.wallpaper.label.as_str(), name => name.as_str())
}

#[cfg(test)]
mod tests;
