use crate::contracts::picker::theme_setting;
use crate::i18n::tr;

use super::super::super::misc::text_width;
use super::super::super::theme_bar::{
    NOCTALIA_SCHEMES, PYWAL_SATURATIONS, SKWD_SCHEMES, SKWD_STYLES, STATIC_THEMES, THEME_BACKENDS,
    THEME_MODES, THEME_SCHEMES, ThemeBar, WALLUST_COLORSPACES, WALLUST_PALETTES,
};
use super::super::action::BarAction;
use super::types::{BarItem, dropdown_item};

pub(super) fn theme_items(theme: &ThemeBar, scale: f32, menu_up: bool) -> Vec<BarItem> {
    let mut items: Vec<BarItem> = Vec::new();
    let mut x = 0.0f32;
    let current = THEME_BACKENDS
        .iter()
        .find(|(key, _)| *key == theme.backend)
        .map_or(tr("theme-bar-skwd-colour"), |(_, label_key)| tr(label_key));
    dropdown_item(
        &mut items,
        &mut x,
        scale,
        "\u{f03d8}",
        current,
        menu_up,
        theme.menu_open,
        BarAction::ThemeBackendToggle,
    );
    push_option(
        &mut items,
        &mut x,
        scale,
        tr("theme-bar-pin-settings"),
        theme.wallpaper_settings_pinned,
        BarAction::ThemePinSettings,
    );
    let out = &mut items;
    if !matches!(theme.backend.as_str(), "off" | "static") {
        push_label(out, &mut x, scale, tr("theme-bar-mode"));
        push_options(out, &mut x, scale, &THEME_MODES, theme_setting::MODE, &theme.mode);
        if theme.backend == "matugen" {
            push_option(
                out,
                &mut x,
                scale,
                tr("theme-bar-smart"),
                theme.mode == "smart",
                BarAction::ThemeOpt(theme_setting::MODE, "smart"),
            );
        }
    }
    match theme.backend.as_str() {
        "static" => {
            push_options(
                out,
                &mut x,
                scale,
                &STATIC_THEMES,
                theme_setting::STATIC_THEME,
                &theme.static_theme,
            );
        }
        "matugen" | "dms" => {
            push_options(
                out,
                &mut x,
                scale,
                &THEME_SCHEMES,
                theme_setting::SCHEME_TYPE,
                &theme.scheme,
            );
            if theme.backend == "matugen" {
                push_option(
                    out,
                    &mut x,
                    scale,
                    tr("theme-bar-smart"),
                    theme.scheme == "scheme-smart",
                    BarAction::ThemeOpt(theme_setting::SCHEME_TYPE, "scheme-smart"),
                );
            }
            for (index, value) in ["0", "1", "2", "3"].into_iter().enumerate() {
                let active = theme.color_index as usize == index;
                let action = BarAction::ThemeOpt(theme_setting::COLOR_INDEX, value);
                push_option(out, &mut x, scale, &(index + 1).to_string(), active, action);
            }
        }
        "wallust" => {
            push_options(
                out,
                &mut x,
                scale,
                &WALLUST_PALETTES,
                theme_setting::WALLUST_PALETTE,
                &theme.wallust_palette,
            );
            push_options(
                out,
                &mut x,
                scale,
                &WALLUST_COLORSPACES,
                theme_setting::WALLUST_COLORSPACE,
                &theme.wallust_colorspace,
            );
        }
        "pywal" => {
            push_options(
                out,
                &mut x,
                scale,
                &PYWAL_SATURATIONS,
                theme_setting::PYWAL_SATURATE,
                &theme.pywal_saturate,
            );
        }
        "noctalia" => {
            push_options(
                out,
                &mut x,
                scale,
                &NOCTALIA_SCHEMES,
                theme_setting::NOCTALIA_SCHEME,
                &theme.noctalia_scheme,
            );
            let action = BarAction::ThemeOpt(theme_setting::NOCTALIA_PURE_BLACK, "toggle");
            push_option(
                out,
                &mut x,
                scale,
                tr("theme-bar-oled-black"),
                theme.noctalia_pure_black,
                action,
            );
        }
        "skwd-iris" | "native" => {
            push_label(out, &mut x, scale, tr("theme-bar-style"));
            push_options(out, &mut x, scale, &SKWD_STYLES, theme_setting::STYLE, &theme.style);
            push_label(out, &mut x, scale, tr("theme-bar-scheme"));
            push_options(
                out,
                &mut x,
                scale,
                &SKWD_SCHEMES,
                theme_setting::SCHEME,
                &theme.iris_scheme,
            );
        }
        "skwd-pywal" | "skwd-wallust" => {
            push_label(out, &mut x, scale, tr("theme-bar-style"));
            push_options(out, &mut x, scale, &SKWD_STYLES, theme_setting::STYLE, &theme.style);
        }
        _ => {}
    }
    items
}

fn push_options(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    options: &[(&'static str, &'static str)],
    setting: &'static str,
    selected: &str,
) {
    for &(key, label_key) in options {
        push_option(
            items,
            x,
            scale,
            tr(label_key),
            key == selected,
            BarAction::ThemeOpt(setting, key),
        );
    }
}

fn push_label(items: &mut Vec<BarItem>, x: &mut f32, scale: f32, text: &'static str) {
    *x += 8.0 * scale;
    let size = 9.0 * scale;
    let width = text_width(text, size, false) + 4.0 * scale;
    items.push(BarItem {
        x: *x,
        y: 0.0,
        w: width,
        h: 24.0 * scale,
        skew: 0.0,
        label: text.to_string(),
        nerd: false,
        text_size: size,
        swatch: None,
        notice: None,
        active: false,
        action: None,
        z: 1,
    });
    *x += width;
}

fn push_option(
    items: &mut Vec<BarItem>,
    x: &mut f32,
    scale: f32,
    label: &str,
    active: bool,
    action: BarAction,
) {
    let size = 10.0 * scale;
    let skew = 10.0 * scale;
    let width = text_width(label, size, false) + 24.0 * scale + skew;
    items.push(BarItem {
        x: *x,
        y: 0.0,
        w: width,
        h: 24.0 * scale,
        skew,
        label: label.to_string(),
        nerd: false,
        text_size: size,
        swatch: None,
        notice: None,
        active,
        action: Some(action),
        z: if active { 10 } else { 1 },
    });
    *x += width - skew;
}
