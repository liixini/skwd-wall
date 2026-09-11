use crate::frontend::browser::{
    RATIOS, RESOLUTIONS, SORT_KEYS, STEAM_CATEGORIES, STEAM_RESOLUTIONS, STEAM_SORTS,
    STEAM_TREND_DAYS, STEAM_TYPES, TOP_RANGES,
};
use crate::frontend::ui::browser_bar::{
    steam_category_label_key, steam_resolution_label, steam_sort_label_key,
    steam_trend_days_label_key, steam_type_label_key, wallhaven_range_label_key,
    wallhaven_sort_label_key,
};
use crate::i18n::tr;

use super::{Builder, keys};

pub(super) fn wallhaven(builder: &mut Builder<'_>) {
    use keys::wallhaven as defaults;
    builder.card(tr("settings-source-defaults-card"), tr("settings-source-defaults-desc"));
    apply_button(builder, keys::sources::WALLHAVEN_SHOW_APPLY_BUTTON);
    builder.dropdown(
        tr("settings-source-default-sort"),
        tr("settings-source-defaults-desc"),
        defaults::DEFAULT_SORT,
        &SORT_KEYS.map(|key| (key, tr(wallhaven_sort_label_key(key)))),
    );
    builder.dropdown(
        tr("settings-source-default-range"),
        "",
        defaults::DEFAULT_TOP_RANGE,
        &TOP_RANGES.map(|(key, _)| (key, tr(wallhaven_range_label_key(key)))),
    );
    for (key, label) in [
        (defaults::DEFAULT_ATLEAST, "settings-source-default-min"),
        (defaults::DEFAULT_ATMOST, "settings-source-default-max"),
    ] {
        builder.dropdown(
            tr(label),
            "",
            key,
            &RESOLUTIONS
                .map(|(key, label)| (key, if key.is_empty() { tr("browser-any") } else { label })),
        );
    }
    builder.dropdown(
        tr("settings-source-default-ratio"),
        "",
        defaults::DEFAULT_RATIOS,
        &RATIOS.map(|(key, label)| (key, if key.is_empty() { tr("browser-any") } else { label })),
    );
    for (key, label) in [
        (defaults::DEFAULT_GENERAL, "browser-general"),
        (defaults::DEFAULT_ANIME, "browser-anime"),
        (defaults::DEFAULT_PEOPLE, "browser-people"),
    ] {
        builder.toggle(tr(label), tr("settings-source-default-category-desc"), key);
    }
    for (key, label) in [
        (defaults::DEFAULT_SFW, "browser-sfw"),
        (defaults::DEFAULT_SKETCHY, "browser-sketchy"),
        (defaults::DEFAULT_NSFW, "browser-nsfw"),
    ] {
        builder.toggle(tr(label), tr("settings-source-default-purity-desc"), key);
    }
}

pub(super) fn steam(builder: &mut Builder<'_>) {
    use keys::steam as defaults;
    builder.card(tr("settings-source-defaults-card"), tr("settings-source-defaults-desc"));
    apply_button(builder, keys::sources::STEAM_SHOW_APPLY_BUTTON);
    builder.dropdown(
        tr("settings-source-default-sort"),
        tr("settings-source-defaults-desc"),
        defaults::DEFAULT_SORT,
        &STEAM_SORTS.map(|key| (key, tr(steam_sort_label_key(key)))),
    );
    builder.dropdown(
        tr("settings-source-default-range"),
        "",
        defaults::DEFAULT_TREND_DAYS,
        &STEAM_TREND_DAYS.map(|key| (key, tr(steam_trend_days_label_key(key)))),
    );
    builder.dropdown(
        tr("settings-source-default-type"),
        "",
        defaults::DEFAULT_TYPE,
        &STEAM_TYPES.map(|key| (key, tr(steam_type_label_key(key)))),
    );
    builder.dropdown(
        tr("settings-source-default-resolution"),
        "",
        defaults::DEFAULT_RESOLUTION,
        &STEAM_RESOLUTIONS.map(|key| (key, steam_resolution_label(key))),
    );
    builder.dropdown(
        tr("settings-source-default-category"),
        "",
        defaults::DEFAULT_CATEGORY,
        &STEAM_CATEGORIES.map(|key| (key, tr(steam_category_label_key(key)))),
    );
    builder.toggle(
        tr("browser-nsfw"),
        tr("settings-source-default-purity-desc"),
        defaults::DEFAULT_NSFW,
    );
}

pub(super) fn apply_button(builder: &mut Builder<'_>, key: &str) {
    builder.toggle(tr("settings-sources-apply-label"), tr("settings-sources-apply-desc"), key);
}
