#![cfg(test)]

use super::{
    NOCTALIA_SCHEMES, PYWAL_SATURATIONS, SKWD_SCHEMES, SKWD_STYLES, STATIC_THEMES, THEME_BACKENDS,
    THEME_MODES, THEME_SCHEMES, WALLUST_COLORSPACES, WALLUST_PALETTES,
};
use crate::i18n::Catalog;

#[test]
fn theme_labels_all_locales() {
    let tables: [&[(&str, &str)]; 10] = [
        &THEME_BACKENDS,
        &STATIC_THEMES,
        &THEME_SCHEMES,
        &THEME_MODES,
        &SKWD_STYLES,
        &SKWD_SCHEMES,
        &WALLUST_PALETTES,
        &WALLUST_COLORSPACES,
        &PYWAL_SATURATIONS,
        &NOCTALIA_SCHEMES,
    ];
    for locale in ["en-US", "sv-SE", "es-ES"] {
        let catalog = Catalog::for_locale(locale);
        for table in tables {
            for (key, label_key) in table {
                let label = catalog.format(label_key, None);
                assert!(!label.is_empty(), "{locale} {label_key}");
                assert_ne!(&label, label_key, "{locale} {key}");
            }
        }
    }
}
