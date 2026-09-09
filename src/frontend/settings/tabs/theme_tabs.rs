#[allow(clippy::wildcard_imports)]
use super::*;
use crate::i18n::{tr, tr_args};

pub(super) fn authority_options(
    available: &[String],
    current: &str,
) -> Vec<(&'static str, String)> {
    let detected = |id: &str| available.iter().any(|candidate| candidate == id);
    let mut out = vec![("skwd", "skwd-wall".to_string())];
    if detected("caelestia") || current == "caelestia" {
        out.push(("caelestia", tr("settings-theme-authority-caelestia").to_string()));
    }
    if detected("noctalia") || current == "noctalia" {
        out.push(("noctalia", tr("settings-theme-authority-noctalia").to_string()));
    }
    if detected("dms") || current == "dms" {
        out.push(("dms", tr("settings-theme-authority-dms").to_string()));
    }
    if detected("end4") || current == "end4" {
        out.push(("end4", tr("settings-theme-authority-end4").to_string()));
    }
    out
}

pub(super) fn engine_options(available: &[String], current: &str) -> Vec<(&'static str, String)> {
    let engines: [(&'static str, &'static str); 5] = [
        ("skwd-iris", tr("settings-theme-engine-iris-builtin")),
        ("matugen", tr("settings-theme-engine-matugen")),
        ("wallust", tr("settings-theme-engine-wallust")),
        ("pywal", tr("settings-theme-engine-pywal")),
        ("iris", tr("settings-theme-engine-iris-external")),
    ];
    let compatibility: [(&'static str, &'static str); 3] = [
        ("native", tr("settings-theme-engine-native-legacy")),
        ("skwd-pywal", tr("settings-theme-engine-pywal-legacy")),
        ("skwd-wallust", tr("settings-theme-engine-wallust-legacy")),
    ];
    let detected = |id: &str| available.iter().any(|candidate| candidate == id);
    let mut out = Vec::new();
    for (id, label) in engines {
        if id == "skwd-iris" || detected(id) || current == id {
            out.push((id, label.to_string()));
        }
    }
    for (id, label) in compatibility {
        if current == id {
            out.push((id, label.to_string()));
        }
    }
    out
}

pub(super) fn tab_theme(builder: &mut Builder<'_>, backends: &[String]) {
    let cfg = builder.cfg;
    let effective = cfg.theme_backend();
    let policy = match cfg.text(keys::theme::POLICY).as_str() {
        "wallpaper" | "fixed" | "off" => cfg.text(keys::theme::POLICY),
        _ => match effective.as_str() {
            "static" => "fixed",
            "off" => "off",
            _ => "wallpaper",
        }
        .to_string(),
    };
    let authority = match cfg.text(keys::theme::AUTHORITY).as_str() {
        "skwd" | "caelestia" | "noctalia" | "dms" | "end4" => cfg.text(keys::theme::AUTHORITY),
        _ => match effective.as_str() {
            "noctalia" => "noctalia",
            "dms" => "dms",
            "caelestia" => "caelestia",
            "end4" => "end4",
            _ => "skwd",
        }
        .to_string(),
    };
    let configured_engine = cfg.text(keys::theme::ENGINE);
    let engine = if configured_engine.is_empty() {
        if matches!(
            effective.as_str(),
            "native"
                | "skwd-iris"
                | "skwd-pywal"
                | "skwd-wallust"
                | "matugen"
                | "wallust"
                | "pywal"
                | "iris"
        ) {
            effective.clone()
        } else {
            "skwd-iris".to_string()
        }
    } else {
        configured_engine
    };

    builder.card(tr("settings-theme-behaviour-card"), tr("settings-theme-behaviour-card-desc"));
    builder.dropdown_cur(
        tr("settings-theme-policy-label"),
        tr("settings-theme-policy-desc"),
        keys::theme::POLICY,
        &[
            ("wallpaper", tr("settings-theme-policy-wallpaper")),
            ("fixed", tr("settings-theme-policy-fixed")),
            ("off", tr("settings-theme-policy-off")),
        ],
        policy.clone(),
    );

    if policy == "wallpaper" {
        builder.card(tr("settings-theme-driver-card"), tr("settings-theme-driver-card-desc"));
        let owned = authority_options(backends, &authority);
        let opts: Vec<(&str, &str)> =
            owned.iter().map(|(id, label)| (*id, label.as_str())).collect();
        builder.dropdown_cur(
            tr("settings-theme-authority-label"),
            tr("settings-theme-authority-desc"),
            keys::theme::AUTHORITY,
            &opts,
            authority.clone(),
        );
    }

    if policy == "wallpaper" && authority == "skwd" {
        builder.card(tr("settings-theme-engine-card"), tr("settings-theme-engine-card-desc"));
        let owned = engine_options(backends, &engine);
        let opts: Vec<(&str, &str)> =
            owned.iter().map(|(id, label)| (*id, label.as_str())).collect();
        builder.dropdown_cur(
            tr("settings-theme-engine-label"),
            tr("settings-theme-engine-desc"),
            keys::theme::ENGINE,
            &opts,
            engine.clone(),
        );
    }

    if policy == "wallpaper"
        && authority == "skwd"
        && matches!(
            engine.as_str(),
            "native" | "skwd-iris" | "skwd-pywal" | "skwd-wallust" | "wallust" | "pywal" | "iris"
        )
    {
        builder
            .card(tr("settings-theme-appearance-card"), tr("settings-theme-appearance-card-desc"));
        builder.dropdown(
            tr("settings-theme-scheme-label"),
            tr("settings-theme-scheme-desc"),
            keys::theme::SCHEME,
            &[
                ("tonal-spot", tr("settings-theme-scheme-tonal-spot")),
                ("vibrant", tr("settings-theme-scheme-vibrant")),
                ("expressive", tr("settings-theme-scheme-expressive")),
                ("neutral", tr("settings-theme-scheme-neutral")),
                ("monochrome", tr("settings-theme-scheme-monochrome")),
                ("fidelity", tr("settings-theme-scheme-fidelity")),
                ("content", tr("settings-theme-scheme-content")),
                ("rainbow", tr("settings-theme-scheme-rainbow")),
                ("fruit-salad", tr("settings-theme-scheme-fruit-salad")),
            ],
        );
        builder.dropdown(
            tr("settings-theme-finish-label"),
            tr("settings-theme-finish-desc"),
            keys::theme::STYLE,
            &[
                ("natural", tr("settings-theme-finish-natural")),
                ("pastel", tr("settings-theme-finish-pastel")),
                ("muted", tr("settings-theme-finish-muted")),
                ("vibrant", tr("settings-theme-finish-vibrant")),
            ],
        );
        builder.dropdown(
            tr("settings-theme-variant-label"),
            tr("settings-theme-variant-desc"),
            keys::matugen::MODE,
            &[
                ("dark", tr("settings-theme-variant-dark")),
                ("light", tr("settings-theme-variant-light")),
                ("auto", tr("settings-theme-variant-auto")),
            ],
        );
    }
    if policy == "wallpaper" {
        builder.card(tr("settings-theme-designer-label"), tr("settings-theme-profile-desc"));
        builder.action(
            tr("settings-theme-designer-label"),
            tr("settings-theme-designer-desc"),
            ActionId::OpenThemeDesigner,
            tr("settings-theme-designer-action"),
        );
    }
    match (policy.as_str(), authority.as_str(), engine.as_str()) {
        ("fixed", _, _) => {
            builder.card(tr("settings-theme-fixed-card"), tr("settings-theme-fixed-card-desc"));
            let mut options = cfg.palette_presets();
            for name in cfg.saved_theme_names() {
                options.push((
                    name.clone(),
                    tr_args!("settings-theme-static-theme-yours", name => &name),
                ));
            }
            options
                .push(("custom".to_string(), tr("settings-theme-static-theme-custom").to_string()));
            let current = cfg.text(keys::theme::STATIC_THEME);
            builder.row(
                tr("settings-theme-static-theme-label"),
                tr("settings-theme-static-theme-desc"),
                Control::Dropdown {
                    palettes: cfg.palette_colors(),
                    path: keys::theme::STATIC_THEME.to_string(),
                    options,
                    current,
                },
            );
            builder.action(
                tr("settings-theme-designer-label"),
                tr("settings-theme-designer-desc"),
                ActionId::OpenThemeDesigner,
                tr("settings-theme-designer-action"),
            );
            builder.text_field(
                tr("settings-theme-custom-colors-label"),
                tr("settings-theme-custom-colors-desc"),
                keys::theme::CUSTOM_COLORS,
                "#1e1e2e, #89b4fa, #f5c2e7",
            );
        }
        ("wallpaper", "skwd", "matugen") => {
            builder.card(tr("settings-theme-matugen-card"), tr("settings-theme-matugen-card-desc"));
            builder.dropdown(
                tr("settings-theme-scheme-label"),
                tr("settings-theme-matugen-scheme-desc"),
                keys::matugen::SCHEME_TYPE,
                &[
                    ("scheme-content", tr("settings-theme-scheme-content")),
                    ("scheme-expressive", tr("settings-theme-scheme-expressive")),
                    ("scheme-fidelity", tr("settings-theme-scheme-fidelity")),
                    ("scheme-fruit-salad", tr("settings-theme-scheme-fruit-salad")),
                    ("scheme-monochrome", tr("settings-theme-scheme-monochrome")),
                    ("scheme-neutral", tr("settings-theme-scheme-neutral")),
                    ("scheme-rainbow", tr("settings-theme-scheme-rainbow")),
                    ("scheme-tonal-spot", tr("settings-theme-scheme-tonal-spot")),
                    ("scheme-vibrant", tr("settings-theme-scheme-vibrant")),
                ],
            );
            builder.dropdown(
                tr("settings-theme-variant-label"),
                tr("settings-theme-variant-desc"),
                keys::matugen::MODE,
                &[
                    ("dark", tr("settings-theme-variant-dark")),
                    ("light", tr("settings-theme-variant-light")),
                    ("auto", tr("settings-theme-variant-auto")),
                ],
            );
            builder.dropdown(
                tr("settings-theme-interface-finish-label"),
                tr("settings-theme-interface-finish-desc"),
                keys::theme::STYLE,
                &[
                    ("natural", tr("settings-theme-finish-natural-plain")),
                    ("pastel", tr("settings-theme-finish-pastel")),
                    ("muted", tr("settings-theme-finish-muted")),
                    ("vibrant", tr("settings-theme-finish-vibrant")),
                ],
            );
            builder.dropdown(
                tr("settings-theme-color-index-label"),
                tr("settings-theme-color-index-desc"),
                keys::matugen::COLOR_INDEX,
                &[
                    ("0", tr("settings-theme-color-index-primary")),
                    ("1", "1"),
                    ("2", "2"),
                    ("3", "3"),
                ],
            );
            builder.text_field(
                tr("settings-theme-contrast-label"),
                tr("settings-theme-contrast-desc"),
                keys::matugen::CONTRAST,
                "0.00",
            );
        }
        ("wallpaper", "skwd", "wallust") => {
            builder.card(tr("settings-theme-wallust-card"), "");
            builder.text_field(
                tr("settings-theme-wallust-palette-label"),
                tr("settings-theme-wallust-palette-desc"),
                keys::theme::WALLUST_PALETTE,
                "dark16",
            );
        }
        ("wallpaper", "skwd", "native") => {
            builder.card(tr("settings-theme-native-card"), tr("settings-theme-native-card-desc"));
        }
        ("wallpaper", "noctalia", _) => {
            builder.card(
                tr("settings-theme-external-card"),
                tr("settings-theme-external-card-desc-noctalia"),
            );
            builder.dropdown(
                tr("settings-theme-imported-variant-label"),
                tr("settings-theme-imported-variant-desc-noctalia"),
                keys::matugen::MODE,
                &[
                    ("dark", tr("settings-theme-variant-dark")),
                    ("light", tr("settings-theme-variant-light")),
                    ("auto", tr("settings-theme-variant-auto")),
                ],
            );
        }
        ("wallpaper", "dms", _) => {
            builder.card(
                tr("settings-theme-external-card"),
                tr("settings-theme-external-card-desc-dms"),
            );
            builder.dropdown(
                tr("settings-theme-imported-variant-label"),
                tr("settings-theme-imported-variant-desc-dms"),
                keys::matugen::MODE,
                &[
                    ("dark", tr("settings-theme-variant-dark")),
                    ("light", tr("settings-theme-variant-light")),
                    ("auto", tr("settings-theme-variant-auto")),
                ],
            );
        }
        _ => {}
    }
}

pub(super) fn tab_integrations(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    let lock_mode = cfg.text(keys::plasma::LOCK_SCREEN_MODE);
    builder.card(
        tr("settings-integrations-plasma-card"),
        tr("settings-integrations-plasma-card-desc"),
    );
    builder.dropdown_cur(
        tr("settings-integrations-lock-mode-label"),
        tr("settings-integrations-lock-mode-desc"),
        keys::plasma::LOCK_SCREEN_MODE,
        &[
            ("off", tr("settings-integrations-lock-off")),
            ("static", tr("settings-integrations-lock-static")),
            ("follow", tr("settings-integrations-lock-follow")),
        ],
        lock_mode.clone(),
    );
    if lock_mode == "static" {
        builder.text_field(
            tr("settings-integrations-lock-image-label"),
            tr("settings-integrations-lock-image-desc"),
            keys::plasma::LOCK_SCREEN_IMAGE,
            "~/Pictures/lock-screen.jpg",
        );
    } else if lock_mode == "follow" {
        builder.dropdown(
            tr("settings-integrations-lock-dynamic-label"),
            tr("settings-integrations-lock-dynamic-desc"),
            keys::plasma::LOCK_SCREEN_DYNAMIC,
            &[
                ("poster", tr("settings-integrations-lock-poster")),
                ("live", tr("settings-integrations-lock-live")),
            ],
        );
    }
    builder.card(
        tr("settings-integrations-noctalia-card"),
        tr("settings-integrations-noctalia-card-desc"),
    );
    builder.toggle(
        tr("settings-integrations-noctalia-hover-label"),
        tr("settings-integrations-noctalia-hover-desc"),
        keys::noctalia::HOVER_PREVIEW,
    );
    builder.text_field(
        tr("settings-integrations-noctalia-bin-label"),
        tr("settings-integrations-noctalia-bin-desc"),
        keys::paths::NOCTALIA_BIN,
        "noctalia",
    );
    builder.card(tr("settings-integrations-dms-card"), tr("settings-integrations-dms-card-desc"));
    builder.toggle(
        tr("settings-integrations-dms-hover-label"),
        tr("settings-integrations-dms-hover-desc"),
        keys::dms::HOVER_PREVIEW,
    );
}

pub(super) fn tab_matugen(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(tr("settings-matugen-external-card"), tr("settings-matugen-external-card-desc"));
    builder.toggle(
        tr("settings-general-feature-matugen-label"),
        tr("settings-general-feature-matugen-desc"),
        keys::features::MATUGEN,
    );
    if !cfg.flag_default_true(keys::features::MATUGEN) {
        return;
    }
    builder.text_field(
        tr("settings-matugen-config-label"),
        tr("settings-matugen-config-desc"),
        keys::matugen::DEFAULT_CONFIG,
        "/path/to/matugen.config.toml",
    );
    builder.text_field(
        tr("settings-matugen-command-label"),
        tr("settings-matugen-command-desc"),
        keys::system::EXTERNAL_MATUGEN_COMMAND,
        "matugen -c %config% image %path%",
    );
    builder.card(
        tr("settings-matugen-integrations-card"),
        tr("settings-matugen-integrations-card-desc"),
    );
    let count = cfg.array_len(keys::integrations::LIST);
    for idx in 0..count {
        builder.text_field(
            tr("settings-matugen-name-label"),
            tr("settings-matugen-name-desc"),
            &format!("integrations.{idx}.name"),
            "kitty",
        );
        builder.text_field(
            tr("settings-matugen-template-label"),
            tr("settings-matugen-template-desc"),
            &format!("integrations.{idx}.template"),
            "kitty.toml",
        );
        builder.text_field(
            tr("settings-matugen-output-label"),
            tr("settings-matugen-output-desc"),
            &format!("integrations.{idx}.output"),
            "~/.config/kitty/colors.toml",
        );
        builder.text_field(
            tr("settings-matugen-reload-label"),
            tr("settings-matugen-reload-desc"),
            &format!("integrations.{idx}.reload"),
            "kill -SIGUSR1 $(pgrep kitty)",
        );
        builder.toggle(
            tr("settings-matugen-live-preview-label"),
            tr("settings-matugen-live-preview-desc"),
            &format!("integrations.{idx}.livePreview"),
        );
        builder.action(
            "",
            tr("settings-matugen-remove-desc"),
            ActionId::RemoveIntegration(idx as u16),
            tr("settings-matugen-remove-action"),
        );
    }
    builder.action(
        tr("settings-matugen-add-label"),
        tr("settings-matugen-add-desc"),
        ActionId::AddIntegration,
        tr("settings-matugen-add-action"),
    );
}

pub(super) fn tab_niri(builder: &mut Builder<'_>, themes: &[String]) {
    builder.card(tr("settings-niri-backdrop-card"), tr("settings-niri-backdrop-card-desc"));
    builder.toggle(
        tr("settings-niri-show-label"),
        tr("settings-niri-show-desc"),
        keys::niri::OVERVIEW_BACKDROP,
    );
    builder.toggle(
        tr("settings-niri-follow-label"),
        tr("settings-niri-follow-desc"),
        keys::niri::BACKDROP_FOLLOW_WALLPAPER,
    );
    builder.text_field(
        tr("settings-niri-image-label"),
        tr("settings-niri-image-desc"),
        keys::niri::BACKDROP,
        "~/Pictures/overview.jpg",
    );
    builder.action(
        tr("settings-niri-refresh-label"),
        tr("settings-niri-refresh-desc"),
        ActionId::RefreshBackdrop,
        tr("settings-niri-refresh-action"),
    );
    builder.row(
        tr("settings-niri-rule-label"),
        tr("settings-niri-rule-desc"),
        Control::Code { snippet: NIRI_SNIPPET },
    );
    builder.action(
        tr("settings-niri-copy-label"),
        tr("settings-niri-copy-desc"),
        ActionId::CopyLayerRule,
        tr("settings-niri-copy-action"),
    );
    builder.toggle(
        tr("settings-niri-blur-label"),
        tr("settings-niri-blur-desc"),
        keys::niri::OVERVIEW_BACKDROP_BLUR_ENABLED,
    );
    builder.num(
        tr("settings-niri-blur-radius-label"),
        tr("settings-niri-blur-radius-desc"),
        keys::niri::OVERVIEW_BACKDROP_BLUR,
        "",
    );
    builder.num(
        tr("settings-niri-dim-label"),
        tr("settings-niri-dim-desc"),
        keys::niri::BACKDROP_DIM,
        "%",
    );
    builder.toggle(
        tr("settings-niri-auto-theme-label"),
        tr("settings-niri-auto-theme-desc"),
        keys::niri::BACKDROP_AUTO_THEME,
    );
    builder.theme_dropdown(
        tr("settings-niri-theme-label"),
        tr("settings-niri-theme-desc"),
        keys::niri::BACKDROP_THEME,
        themes,
    );
}
